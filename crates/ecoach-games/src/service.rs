use std::collections::BTreeMap;

use chrono::Utc;
use ecoach_substrate::{clamp_bp, to_bp, BasisPoints, DomainEvent, EcoachError, EcoachResult};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};

use crate::models::{
    ContrastPairSummary, GameAnswerResult, GameLeaderboardEntry, GameSession, GameSummary,
    GameType, MindstackState, StartGameInput, StartTrapsSessionInput, SubmitGameAnswerInput,
    SubmitTrapConfusionReasonInput, SubmitTrapRoundInput, TrapChoiceOption, TrapRoundCard,
    TrapRoundResult, TrapSessionReview, TrapSessionSnapshot, TrapsMode, TrapsState, TugOfWarState,
};

const STREAK_BONUS_MULTIPLIER: f64 = 0.10;
const SPEED_BONUS_THRESHOLD_MS: i64 = 5_000;
const BASE_CORRECT_POINTS: i64 = 100;
const BASE_INCORRECT_POINTS: i64 = 0;
const MISCONCEPTION_PENALTY: i64 = 25;
const TUG_CORRECT_MOVE: i64 = 2;
const TUG_INCORRECT_MOVE: i64 = -3;
const TUG_WIN_POSITION: i64 = 10;
const TUG_LOSE_POSITION: i64 = -10;
const MINDSTACK_CORRECT_CLEAR: i64 = 1;
const MINDSTACK_INCORRECT_STACK: i64 = 2;
const MINDSTACK_MAX_HEIGHT: i64 = 15;
const BOTH_CHOICE_CODE: &str = "both";
const NEITHER_CHOICE_CODE: &str = "neither";

#[derive(Debug, Clone)]
struct ContrastPairContext {
    id: i64,
    pair_code: Option<String>,
    title: String,
    topic_id: Option<i64>,
    left_label: String,
    right_label: String,
    summary_text: Option<String>,
    trap_strength: BasisPoints,
    difficulty_score: BasisPoints,
    confusion_score: BasisPoints,
    last_accuracy_bp: BasisPoints,
    recommended_mode: String,
}

fn map_game_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<GameSession> {
    Ok(GameSession {
        id: row.get(0)?,
        student_id: row.get(1)?,
        game_type: row.get(2)?,
        subject_id: row.get(3)?,
        session_state: row.get(4)?,
        score: row.get(5)?,
        rounds_total: row.get(6)?,
        rounds_played: row.get(7)?,
        streak: row.get(8)?,
        best_streak: row.get(9)?,
        created_at: row.get(10)?,
        completed_at: row.get(11)?,
    })
}

fn map_stored_trap_round(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredTrapRound> {
    Ok(StoredTrapRound {
        id: row.get(0)?,
        pair_id: row.get(1)?,
        round_number: row.get(2)?,
        mode: row.get(3)?,
        lane: row.get(4)?,
        prompt_text: row.get(5)?,
        prompt_payload_json: row.get(6)?,
        options_json: row.get(7)?,
        correct_choice_code: row.get(8)?,
        correct_choice_label: row.get(9)?,
        explanation_text: row.get(10)?,
        reveal_count: row.get(11)?,
        max_reveal_count: row.get(12)?,
        answered_at: row.get(13)?,
    })
}

fn parse_json_value(raw: &str) -> EcoachResult<Value> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_json_choices(raw: &str) -> EcoachResult<Vec<TrapChoiceOption>> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_traps_metadata(value: &Value) -> EcoachResult<TrapsMetadata> {
    Ok(TrapsMetadata {
        pair_id: value["pair_id"]
            .as_i64()
            .ok_or_else(|| EcoachError::Validation("traps metadata missing pair_id".to_string()))?,
        pair_title: value["pair_title"]
            .as_str()
            .unwrap_or("Contrast Pair")
            .to_string(),
        left_label: value["left_label"].as_str().unwrap_or("Left").to_string(),
        right_label: value["right_label"].as_str().unwrap_or("Right").to_string(),
        summary_text: value["summary_text"].as_str().map(ToString::to_string),
        mode: value["mode"]
            .as_str()
            .unwrap_or("difference_drill")
            .to_string(),
        recommended_mode: value["recommended_mode"]
            .as_str()
            .unwrap_or("difference_drill")
            .to_string(),
        correct_discriminations: value["correct_discriminations"].as_i64().unwrap_or(0),
        total_discriminations: value["total_discriminations"].as_i64().unwrap_or(0),
        current_round_id: value["current_round_id"].as_i64(),
        current_round_number: value["current_round_number"].as_i64().unwrap_or(1),
    })
}

fn resolve_recommended_traps_mode(
    difference_drill_bp: BasisPoints,
    similarity_trap_bp: BasisPoints,
    know_difference_bp: BasisPoints,
    which_is_which_bp: BasisPoints,
    unmask_bp: BasisPoints,
    confusion_score: BasisPoints,
) -> &'static str {
    if difference_drill_bp == 0 || confusion_score >= 7000 {
        TrapsMode::DifferenceDrill.as_str()
    } else if similarity_trap_bp < 6500 {
        TrapsMode::SimilarityTrap.as_str()
    } else if know_difference_bp < 6500 {
        TrapsMode::KnowTheDifference.as_str()
    } else if which_is_which_bp < 7000 {
        TrapsMode::WhichIsWhich.as_str()
    } else if unmask_bp < 7000 {
        TrapsMode::Unmask.as_str()
    } else {
        TrapsMode::WhichIsWhich.as_str()
    }
}

fn default_timer_seconds(mode: TrapsMode) -> i64 {
    match mode {
        TrapsMode::DifferenceDrill => 6,
        TrapsMode::SimilarityTrap => 8,
        TrapsMode::KnowTheDifference => 20,
        TrapsMode::WhichIsWhich => 3,
        TrapsMode::Unmask => 10,
    }
}

fn render_prompt_text(
    mode: &str,
    prompt_text: &str,
    prompt_payload: &Value,
    reveal_count: i64,
    review_mode: bool,
) -> String {
    if mode != TrapsMode::Unmask.as_str() {
        return prompt_text.to_string();
    }

    let clues = prompt_payload["clues"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();
    if clues.is_empty() {
        return prompt_text.to_string();
    }
    if review_mode {
        return clues.join(" -> ");
    }

    let index = (reveal_count.saturating_sub(1) as usize).min(clues.len().saturating_sub(1));
    clues[index].clone()
}

fn calculate_trap_score(
    mode: TrapsMode,
    is_correct: bool,
    skipped: bool,
    streak: i64,
    response_time_ms: i64,
    reveal_count: i64,
) -> i64 {
    if skipped || !is_correct {
        return 0;
    }

    let base = match mode {
        TrapsMode::DifferenceDrill => 120,
        TrapsMode::SimilarityTrap => 130,
        TrapsMode::KnowTheDifference => 140,
        TrapsMode::WhichIsWhich => 100,
        TrapsMode::Unmask => 160,
    };
    let speed_threshold = trap_speed_threshold_ms(mode);
    let speed_bonus = if response_time_ms < speed_threshold {
        ((speed_threshold - response_time_ms) / 120).clamp(0, 60)
    } else {
        0
    };
    let reveal_penalty = if mode == TrapsMode::Unmask {
        ((reveal_count - 1).max(0) * 20).min(60)
    } else {
        0
    };
    let streak_bonus =
        ((streak - 1).max(0) as f64 * STREAK_BONUS_MULTIPLIER * base as f64).round() as i64;

    (base + speed_bonus + streak_bonus - reveal_penalty).max(20)
}

fn classify_trap_confusion_signal(
    mode: TrapsMode,
    is_correct: bool,
    timed_out: bool,
    response_time_ms: i64,
    reveal_count: i64,
) -> &'static str {
    if timed_out {
        return "timed_out";
    }
    if !is_correct && response_time_ms <= trap_speed_threshold_ms(mode) / 2 {
        return "impulsive_confusion";
    }
    if !is_correct {
        return "concept_confusion";
    }
    if mode == TrapsMode::Unmask && reveal_count > 1 {
        return "late_certainty";
    }
    if response_time_ms > trap_speed_threshold_ms(mode) {
        return "hesitation";
    }
    "clean_discrimination"
}

fn trap_speed_threshold_ms(mode: TrapsMode) -> i64 {
    match mode {
        TrapsMode::DifferenceDrill => 5_000,
        TrapsMode::SimilarityTrap => 6_000,
        TrapsMode::KnowTheDifference => 12_000,
        TrapsMode::WhichIsWhich => 2_500,
        TrapsMode::Unmask => 4_500,
    }
}

fn trap_mode_performance_bp(
    mode: TrapsMode,
    is_correct: bool,
    timed_out: bool,
    response_time_ms: i64,
) -> BasisPoints {
    if timed_out {
        return 1500;
    }
    if !is_correct {
        return 2500;
    }
    let threshold = trap_speed_threshold_ms(mode) as f64;
    let speed_ratio = ((threshold - response_time_ms as f64) / threshold).clamp(0.0, 1.0);
    clamp_bp((6500.0 + speed_ratio * 3500.0).round() as i64)
}

fn rolling_average(previous_average: i64, previous_count: i64, next_value: i64) -> i64 {
    if previous_count <= 0 {
        return next_value.max(0);
    }

    ((previous_average * previous_count) + next_value.max(0)) / (previous_count + 1)
}

fn rolling_bp(previous: BasisPoints, next: BasisPoints, sample_count: i64) -> BasisPoints {
    if sample_count <= 1 {
        return next;
    }
    clamp_bp((((previous as i64) * (sample_count - 1)) + next as i64) / sample_count)
}

fn choice_label_for_code(options: &[TrapChoiceOption], code: &str) -> Option<String> {
    options
        .iter()
        .find(|option| option.code == code)
        .map(|option| option.label.clone())
}

fn build_difference_drill_rounds(
    pair: &ContrastPairContext,
    atoms: &[ContrastAtomContext],
    round_count: usize,
) -> EcoachResult<Vec<TrapRoundBlueprint>> {
    let candidates: Vec<&ContrastAtomContext> = atoms
        .iter()
        .filter(|atom| matches!(atom.ownership_type.as_str(), "left_only" | "right_only"))
        .collect();
    if candidates.is_empty() {
        return Err(EcoachError::Validation(format!(
            "pair {} does not contain left/right contrast atoms",
            pair.title
        )));
    }

    Ok(candidates
        .into_iter()
        .take(round_count)
        .map(|atom| {
            let correct_choice_code = if atom.ownership_type == "left_only" {
                "left"
            } else {
                "right"
            };
            TrapRoundBlueprint {
                atom_id: Some(atom.id),
                lane: atom.lane.clone(),
                prompt_text: atom.atom_text.clone(),
                prompt_payload: json!({
                    "mode_family": "sorting",
                    "ownership_type": atom.ownership_type,
                }),
                answer_options: vec![
                    TrapChoiceOption {
                        code: "left".to_string(),
                        label: pair.left_label.clone(),
                    },
                    TrapChoiceOption {
                        code: "right".to_string(),
                        label: pair.right_label.clone(),
                    },
                ],
                correct_choice_code: correct_choice_code.to_string(),
                correct_choice_label: if correct_choice_code == "left" {
                    pair.left_label.clone()
                } else {
                    pair.right_label.clone()
                },
                explanation_text: contrast_explanation(pair, atom, correct_choice_code),
                max_reveal_count: 1,
            }
        })
        .collect())
}

fn build_similarity_trap_rounds(
    pair: &ContrastPairContext,
    atoms: &[ContrastAtomContext],
    round_count: usize,
) -> EcoachResult<Vec<TrapRoundBlueprint>> {
    if atoms.is_empty() {
        return Err(EcoachError::Validation(format!(
            "pair {} does not contain trap atoms",
            pair.title
        )));
    }

    Ok(atoms
        .iter()
        .take(round_count)
        .map(|atom| {
            let correct_choice_code = match atom.ownership_type.as_str() {
                "left_only" => "left",
                "right_only" => "right",
                "both" => BOTH_CHOICE_CODE,
                _ => NEITHER_CHOICE_CODE,
            };
            TrapRoundBlueprint {
                atom_id: Some(atom.id),
                lane: atom.lane.clone(),
                prompt_text: atom.atom_text.clone(),
                prompt_payload: json!({
                    "mode_family": "overlap",
                    "ownership_type": atom.ownership_type,
                }),
                answer_options: vec![
                    TrapChoiceOption {
                        code: "left".to_string(),
                        label: pair.left_label.clone(),
                    },
                    TrapChoiceOption {
                        code: "right".to_string(),
                        label: pair.right_label.clone(),
                    },
                    TrapChoiceOption {
                        code: BOTH_CHOICE_CODE.to_string(),
                        label: "Both".to_string(),
                    },
                    TrapChoiceOption {
                        code: NEITHER_CHOICE_CODE.to_string(),
                        label: "Neither".to_string(),
                    },
                ],
                correct_choice_code: correct_choice_code.to_string(),
                correct_choice_label: similarity_choice_label(pair, correct_choice_code),
                explanation_text: contrast_explanation(pair, atom, correct_choice_code),
                max_reveal_count: 1,
            }
        })
        .collect())
}

fn build_know_difference_rounds(
    pair: &ContrastPairContext,
    atoms: &[ContrastAtomContext],
    round_count: usize,
) -> EcoachResult<Vec<TrapRoundBlueprint>> {
    let left_atoms: Vec<&ContrastAtomContext> = atoms
        .iter()
        .filter(|atom| atom.ownership_type == "left_only")
        .collect();
    let right_atoms: Vec<&ContrastAtomContext> = atoms
        .iter()
        .filter(|atom| atom.ownership_type == "right_only")
        .collect();
    if left_atoms.is_empty() || right_atoms.is_empty() {
        return Err(EcoachError::Validation(format!(
            "pair {} needs left and right atoms for know-the-difference mode",
            pair.title
        )));
    }
    let both_atom = atoms.iter().find(|atom| atom.ownership_type == "both");
    let neither_atom = atoms.iter().find(|atom| atom.ownership_type == "neither");

    let mut rounds = Vec::new();
    for index in 0..round_count {
        let focus_left = index % 2 == 0;
        let correct_atom = if focus_left {
            left_atoms[index % left_atoms.len()]
        } else {
            right_atoms[index % right_atoms.len()]
        };
        let opposite_atom = if focus_left {
            right_atoms[index % right_atoms.len()]
        } else {
            left_atoms[index % left_atoms.len()]
        };
        let shared_label = both_atom
            .map(|atom| atom.atom_text.clone())
            .unwrap_or_else(|| {
                pair.summary_text
                    .clone()
                    .unwrap_or_else(|| "Both ideas can appear in the same example.".to_string())
            });
        let neither_label = neither_atom
            .map(|atom| atom.atom_text.clone())
            .unwrap_or_else(|| "They always mean exactly the same thing.".to_string());
        let mut choices = vec![
            TrapChoiceOption {
                code: "A".to_string(),
                label: correct_atom.atom_text.clone(),
            },
            TrapChoiceOption {
                code: "B".to_string(),
                label: opposite_atom.atom_text.clone(),
            },
            TrapChoiceOption {
                code: "C".to_string(),
                label: shared_label,
            },
            TrapChoiceOption {
                code: "D".to_string(),
                label: neither_label,
            },
        ];
        let rotation = index % choices.len();
        rotate_choices(&mut choices, rotation);
        let correct_choice_code = choices
            .iter()
            .find(|choice| choice.label == correct_atom.atom_text)
            .map(|choice| choice.code.clone())
            .unwrap_or_else(|| "A".to_string());

        rounds.push(TrapRoundBlueprint {
            atom_id: Some(correct_atom.id),
            lane: correct_atom.lane.clone(),
            prompt_text: format!(
                "Which statement best describes {} rather than {}?",
                if focus_left {
                    &pair.left_label
                } else {
                    &pair.right_label
                },
                if focus_left {
                    &pair.right_label
                } else {
                    &pair.left_label
                }
            ),
            prompt_payload: json!({ "focus": if focus_left { "left" } else { "right" } }),
            answer_options: choices,
            correct_choice_code,
            correct_choice_label: correct_atom.atom_text.clone(),
            explanation_text: contrast_explanation(
                pair,
                correct_atom,
                if focus_left { "left" } else { "right" },
            ),
            max_reveal_count: 1,
        });
    }

    Ok(rounds)
}

fn build_which_is_which_rounds(
    pair: &ContrastPairContext,
    atoms: &[ContrastAtomContext],
    round_count: usize,
) -> EcoachResult<Vec<TrapRoundBlueprint>> {
    let mut candidates: Vec<&ContrastAtomContext> = atoms
        .iter()
        .filter(|atom| {
            matches!(atom.ownership_type.as_str(), "left_only" | "right_only")
                && atom.is_speed_ready
        })
        .collect();
    if candidates.is_empty() {
        candidates = atoms
            .iter()
            .filter(|atom| matches!(atom.ownership_type.as_str(), "left_only" | "right_only"))
            .collect();
    }
    if candidates.is_empty() {
        return Err(EcoachError::Validation(format!(
            "pair {} has no speed-suitable atoms",
            pair.title
        )));
    }

    Ok(candidates
        .into_iter()
        .take(round_count)
        .map(|atom| {
            let correct_choice_code = if atom.ownership_type == "left_only" {
                "left"
            } else {
                "right"
            };
            TrapRoundBlueprint {
                atom_id: Some(atom.id),
                lane: atom.lane.clone(),
                prompt_text: atom.atom_text.clone(),
                prompt_payload: json!({
                    "mode_family": "rapid_recognition",
                    "speed_ready": atom.is_speed_ready,
                }),
                answer_options: vec![
                    TrapChoiceOption {
                        code: "left".to_string(),
                        label: pair.left_label.clone(),
                    },
                    TrapChoiceOption {
                        code: "right".to_string(),
                        label: pair.right_label.clone(),
                    },
                ],
                correct_choice_code: correct_choice_code.to_string(),
                correct_choice_label: if correct_choice_code == "left" {
                    pair.left_label.clone()
                } else {
                    pair.right_label.clone()
                },
                explanation_text: contrast_explanation(pair, atom, correct_choice_code),
                max_reveal_count: 1,
            }
        })
        .collect())
}

fn build_unmask_rounds(
    pair: &ContrastPairContext,
    atoms: &[ContrastAtomContext],
    round_count: usize,
) -> EcoachResult<Vec<TrapRoundBlueprint>> {
    let left_atoms: Vec<&ContrastAtomContext> = atoms
        .iter()
        .filter(|atom| atom.ownership_type == "left_only")
        .collect();
    let right_atoms: Vec<&ContrastAtomContext> = atoms
        .iter()
        .filter(|atom| atom.ownership_type == "right_only")
        .collect();
    if left_atoms.is_empty() || right_atoms.is_empty() {
        return Err(EcoachError::Validation(format!(
            "pair {} needs both left and right clue ladders for unmask mode",
            pair.title
        )));
    }

    let mut rounds = Vec::new();
    for index in 0..round_count {
        let target_left = index % 2 == 0;
        let target_atoms = if target_left {
            &left_atoms
        } else {
            &right_atoms
        };
        let lead_atom = target_atoms[index % target_atoms.len()];
        let clues = target_atoms
            .iter()
            .take(3)
            .map(|atom| atom.atom_text.clone())
            .collect::<Vec<_>>();
        let payload = json!({
            "clues": clues,
            "target": if target_left { "left" } else { "right" },
        });

        rounds.push(TrapRoundBlueprint {
            atom_id: Some(lead_atom.id),
            lane: lead_atom.lane.clone(),
            prompt_text: "Identify the concept before all the clues are revealed.".to_string(),
            prompt_payload: payload.clone(),
            answer_options: vec![
                TrapChoiceOption {
                    code: "left".to_string(),
                    label: pair.left_label.clone(),
                },
                TrapChoiceOption {
                    code: "right".to_string(),
                    label: pair.right_label.clone(),
                },
            ],
            correct_choice_code: if target_left { "left" } else { "right" }.to_string(),
            correct_choice_label: if target_left {
                pair.left_label.clone()
            } else {
                pair.right_label.clone()
            },
            explanation_text: contrast_explanation(
                pair,
                lead_atom,
                if target_left { "left" } else { "right" },
            ),
            max_reveal_count: clues_len_from_payload(&payload).max(1),
        });
    }

    Ok(rounds)
}

fn rotate_choices(choices: &mut Vec<TrapChoiceOption>, shift: usize) {
    choices.rotate_left(shift);
    for (index, choice) in choices.iter_mut().enumerate() {
        choice.code = match index {
            0 => "A",
            1 => "B",
            2 => "C",
            _ => "D",
        }
        .to_string();
    }
}

fn similarity_choice_label(pair: &ContrastPairContext, correct_choice_code: &str) -> String {
    match correct_choice_code {
        "left" => pair.left_label.clone(),
        "right" => pair.right_label.clone(),
        BOTH_CHOICE_CODE => "Both".to_string(),
        _ => "Neither".to_string(),
    }
}

fn contrast_explanation(
    pair: &ContrastPairContext,
    atom: &ContrastAtomContext,
    correct_choice_code: &str,
) -> String {
    let base = atom
        .explanation_text
        .clone()
        .unwrap_or_else(|| atom.atom_text.clone());
    let choice_label = similarity_choice_label(pair, correct_choice_code);
    if let Some(summary) = &pair.summary_text {
        format!(
            "{} {} This clue belongs to {}.",
            base, summary, choice_label
        )
    } else {
        format!("{} This clue belongs to {}.", base, choice_label)
    }
}

fn clues_len_from_payload(payload: &Value) -> i64 {
    payload["clues"]
        .as_array()
        .map(|clues| clues.len() as i64)
        .unwrap_or(1)
}

fn dominant_confusion_reason(profile: &BTreeMap<String, i64>) -> Option<String> {
    profile
        .iter()
        .max_by(|left, right| left.1.cmp(right.1).then_with(|| left.0.cmp(right.0)))
        .map(|(reason, _)| reason.clone())
}

fn build_traps_remediation_actions(
    accuracy_bp: BasisPoints,
    confusion_score: BasisPoints,
    weakest_lane: Option<&str>,
    timed_out_count: i64,
    dominant_confusion_reason: Option<&str>,
    recommended_next_mode: &str,
) -> Vec<String> {
    let mut actions = Vec::new();

    if confusion_score >= 7000 || accuracy_bp < 6000 {
        actions.push(format!(
            "Run {} next so the pair is rebuilt before speed is emphasized.",
            recommended_next_mode
        ));
    }
    if let Some(lane) = weakest_lane {
        actions.push(format!(
            "Re-teach the {} lane explicitly before the next mixed contrast round.",
            lane
        ));
    }
    if timed_out_count > 0 {
        actions.push(
            "Slow the next contrast set down and require calm discrimination before timed work."
                .to_string(),
        );
    }
    if let Some(reason_code) = dominant_confusion_reason {
        actions.push(format!(
            "Target the dominant confusion pattern: {}.",
            reason_code.replace('_', " ")
        ));
    }
    if actions.is_empty() {
        actions.push(
            "Advance to the next contrast mode and keep the pair in mixed review.".to_string(),
        );
    }

    actions.truncate(4);
    actions
}

fn build_game_focus_signals(
    game_type: &str,
    accuracy_bp: BasisPoints,
    average_response_time_ms: i64,
    misconception_hits: bool,
    abandoned: bool,
) -> Vec<String> {
    let mut signals = Vec::new();

    if abandoned {
        signals.push("abandoned_session".to_string());
    }
    if accuracy_bp < 5500 {
        signals.push("accuracy_fragile".to_string());
    } else if accuracy_bp >= 8000 {
        signals.push("high_accuracy".to_string());
    }
    if average_response_time_ms >= 5_500 {
        signals.push("slow_decision_cycle".to_string());
    } else if average_response_time_ms > 0 && average_response_time_ms <= 2_000 {
        signals.push("fast_confident_execution".to_string());
    }
    if misconception_hits {
        signals.push("misconception_pressure".to_string());
    }
    if game_type == GameType::Mindstack.as_str() {
        signals.push("stack_discipline".to_string());
    } else if game_type == GameType::TugOfWar.as_str() {
        signals.push("competitive_pressure".to_string());
    } else if game_type == GameType::Traps.as_str() {
        signals.push("contrast_discrimination".to_string());
    }

    signals.truncate(4);
    signals
}

fn recommend_game_next_step(
    game_type: &str,
    focus_signals: &[String],
    misconception_hits: bool,
) -> String {
    if focus_signals
        .iter()
        .any(|signal| signal == "accuracy_fragile")
    {
        return if game_type == GameType::Traps.as_str() {
            "Return to a calmer contrast-remediation pass before another speed round.".to_string()
        } else {
            "Shift back to repair-focused practice before replaying the game at full pressure."
                .to_string()
        };
    }
    if misconception_hits {
        return "Revisit the linked misconception explanation before the next attempt.".to_string();
    }
    if focus_signals
        .iter()
        .any(|signal| signal == "slow_decision_cycle")
    {
        return "Repeat the mode with a smaller timer and keep the same difficulty surface."
            .to_string();
    }
    "Advance to the next pressure layer or mixed set while the signal is clean.".to_string()
}

fn count_correct_ratio(correct_count: i64, total_count: i64) -> BasisPoints {
    if total_count > 0 {
        to_bp(correct_count as f64 / total_count as f64)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_identity::{CreateAccountInput, IdentityService};
    use ecoach_storage::run_runtime_migrations;
    use ecoach_substrate::{AccountType, EntitlementTier};
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn difference_drill_session_tracks_rounds_and_review() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Akosua".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        let service = GamesService::new(&conn);
        let pairs = service
            .list_traps_pairs(student.id, subject_id, &[topic_id])
            .expect("contrast pairs should be listed");

        assert_eq!(pairs.len(), 1);

        let snapshot = service
            .start_traps_session(&StartTrapsSessionInput {
                student_id: student.id,
                subject_id,
                topic_ids: vec![topic_id],
                pair_id: Some(pairs[0].pair_id),
                mode: TrapsMode::DifferenceDrill,
                round_count: 4,
                timer_seconds: Some(6),
            })
            .expect("difference drill session should start");

        assert_eq!(snapshot.rounds.len(), 4);

        for (index, round) in snapshot.rounds.iter().enumerate() {
            let correct_code = load_round_correct_code(&conn, round.id);
            let selected_choice_code = if index == 1 {
                round
                    .answer_options
                    .iter()
                    .find(|option| option.code != correct_code)
                    .map(|option| option.code.clone())
            } else {
                Some(correct_code.clone())
            };

            let result = service
                .submit_trap_round(&SubmitTrapRoundInput {
                    game_session_id: snapshot.session.id,
                    round_id: round.id,
                    selected_choice_code: selected_choice_code.clone(),
                    response_time_ms: if index == 1 { 3_800 } else { 1_900 },
                    timed_out: false,
                })
                .expect("trap round should submit");

            if index == 1 {
                assert!(!result.is_correct);
                service
                    .record_trap_confusion_reason(&SubmitTrapConfusionReasonInput {
                        round_id: round.id,
                        reason_code: "feature_confusion".to_string(),
                        reason_text: Some("I mixed up which feature belonged where.".to_string()),
                    })
                    .expect("confusion reason should persist");
            }
        }

        let state = service
            .get_traps_state(snapshot.session.id)
            .expect("traps state should load");
        let review = service
            .get_traps_review(snapshot.session.id)
            .expect("traps review should load");
        let summary = service
            .get_summary(snapshot.session.id)
            .expect("summary should load");

        assert_eq!(state.total_discriminations, 4);
        assert_eq!(review.rounds.len(), 4);
        assert!(review
            .rounds
            .iter()
            .any(|round| { round.confusion_reason_code.as_deref() == Some("feature_confusion") }));
        assert!(review.weakest_lane.is_some());
        assert!(!review.remediation_actions.is_empty());
        assert!(!review.recommended_next_mode.is_empty());
        assert_eq!(
            review.dominant_confusion_reason.as_deref(),
            Some("feature_confusion")
        );
        assert_eq!(summary.rounds_played, 4);
        assert!(summary
            .focus_signals
            .iter()
            .any(|signal| signal == "contrast_discrimination"));
        assert!(summary.recommended_next_step.is_some());
    }

    #[test]
    fn unmask_round_reveals_more_clues_before_submission() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Yaw".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        let service = GamesService::new(&conn);
        let snapshot = service
            .start_traps_session(&StartTrapsSessionInput {
                student_id: student.id,
                subject_id,
                topic_ids: vec![topic_id],
                pair_id: None,
                mode: TrapsMode::Unmask,
                round_count: 4,
                timer_seconds: Some(8),
            })
            .expect("unmask session should start");

        let first_round = snapshot.rounds.first().expect("unmask round should exist");
        let revealed = service
            .reveal_unmask_clue(snapshot.session.id, first_round.id)
            .expect("next clue should reveal");
        let correct_code = load_round_correct_code(&conn, first_round.id);
        let result = service
            .submit_trap_round(&SubmitTrapRoundInput {
                game_session_id: snapshot.session.id,
                round_id: first_round.id,
                selected_choice_code: Some(correct_code),
                response_time_ms: 2_400,
                timed_out: false,
            })
            .expect("revealed unmask round should submit");

        assert_eq!(revealed.reveal_count, first_round.reveal_count + 1);
        assert_ne!(revealed.prompt_text, first_round.prompt_text);
        assert!(result.is_correct);
    }

    #[test]
    fn mindstack_session_updates_board_and_completes_on_overflow() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Esi".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        let (question_id, _correct_option_id, wrong_option_id) =
            load_fraction_question_options(&conn);
        let service = GamesService::new(&conn);
        let session = service
            .start_game_session(&StartGameInput {
                student_id: student.id,
                game_type: GameType::Mindstack,
                subject_id,
                topic_ids: vec![topic_id],
                question_count: 8,
            })
            .expect("mindstack session should start");

        let initial_state = service
            .get_mindstack_state(session.id)
            .expect("mindstack state should load");
        assert_eq!(initial_state.board_height, 0);

        let mut last_result = None;
        for _ in 0..8 {
            let result = service
                .submit_answer(&SubmitGameAnswerInput {
                    game_session_id: session.id,
                    question_id,
                    selected_option_id: wrong_option_id,
                    response_time_ms: 5_800,
                })
                .expect("mindstack answer should submit");
            last_result = Some(result);
        }

        let final_state = service
            .get_mindstack_state(session.id)
            .expect("mindstack state should load after answers");
        let summary = service
            .get_summary(session.id)
            .expect("completed mindstack summary should load");
        let last_result = last_result.expect("there should be a last result");

        assert!(final_state.board_height >= 15);
        assert_eq!(summary.rounds_played, 8);
        assert!(summary.misconception_hits >= 1);
        assert!(summary
            .focus_signals
            .iter()
            .any(|signal| signal == "misconception_pressure"));
        assert!(last_result.session_complete);
        assert_eq!(last_result.effect_type, "stack_overflow_game_over");
    }

    #[test]
    fn tug_of_war_session_tracks_position_and_difficulty() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let identity = IdentityService::new(&conn);
        let student = identity
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Kojo".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student account should be created");

        let (subject_id, topic_id) = load_fraction_scope(&conn);
        let (question_id, correct_option_id, _wrong_option_id) =
            load_fraction_question_options(&conn);
        let service = GamesService::new(&conn);
        let session = service
            .start_game_session(&StartGameInput {
                student_id: student.id,
                game_type: GameType::TugOfWar,
                subject_id,
                topic_ids: vec![topic_id],
                question_count: 5,
            })
            .expect("tug of war session should start");

        let initial_state = service
            .get_tug_of_war_state(session.id)
            .expect("initial tug state should load");
        assert_eq!(initial_state.position, 0);
        assert_eq!(initial_state.opponent_difficulty, 5000);

        let mut last_result = None;
        for _ in 0..5 {
            let result = service
                .submit_answer(&SubmitGameAnswerInput {
                    game_session_id: session.id,
                    question_id,
                    selected_option_id: correct_option_id,
                    response_time_ms: 1_700,
                })
                .expect("tug answer should submit");
            last_result = Some(result);
        }

        let final_state = service
            .get_tug_of_war_state(session.id)
            .expect("tug state should load after answers");
        let summary = service
            .get_summary(session.id)
            .expect("completed tug summary should load");
        let last_result = last_result.expect("there should be a last result");

        assert_eq!(final_state.position, 10);
        assert!(final_state.opponent_difficulty > 5000);
        assert_eq!(summary.rounds_played, 5);
        assert!(summary
            .focus_signals
            .iter()
            .any(|signal| signal == "competitive_pressure"));
        assert_eq!(last_result.effect_type, "tug_win");
        assert!(last_result.session_complete);
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
    }

    fn install_sample_pack(conn: &Connection) {
        let service = PackService::new(conn);
        service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
    }

    fn load_fraction_scope(conn: &Connection) -> (i64, i64) {
        conn.query_row(
            "SELECT s.id, t.id
             FROM subjects s
             INNER JOIN topics t ON t.subject_id = s.id
             WHERE s.code = 'MATH' AND t.code = 'FRA'
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("fractions scope should exist")
    }

    fn load_round_correct_code(conn: &Connection, round_id: i64) -> String {
        conn.query_row(
            "SELECT correct_choice_code FROM traps_rounds WHERE id = ?1",
            [round_id],
            |row| row.get(0),
        )
        .expect("correct choice should exist")
    }

    fn load_fraction_question_options(conn: &Connection) -> (i64, i64, i64) {
        conn.query_row(
            "SELECT q.id,
                    MAX(CASE WHEN qo.is_correct = 1 THEN qo.id END) AS correct_option_id,
                    MAX(CASE WHEN qo.is_correct = 0 THEN qo.id END) AS wrong_option_id
             FROM questions q
             INNER JOIN question_options qo ON qo.question_id = q.id
             INNER JOIN topics t ON t.id = q.topic_id
             WHERE t.code = 'FRA'
             GROUP BY q.id
             ORDER BY q.id ASC
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("fraction question options should exist")
    }

    fn sample_pack_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate directory should have workspace parent")
            .parent()
            .expect("workspace root should exist")
            .join("packs")
            .join("math-bece-sample")
    }
}

#[derive(Debug, Clone)]
struct ContrastAtomContext {
    id: i64,
    ownership_type: String,
    atom_text: String,
    lane: String,
    explanation_text: Option<String>,
    is_speed_ready: bool,
}

#[derive(Debug, Clone)]
struct TrapRoundBlueprint {
    atom_id: Option<i64>,
    lane: String,
    prompt_text: String,
    prompt_payload: Value,
    answer_options: Vec<TrapChoiceOption>,
    correct_choice_code: String,
    correct_choice_label: String,
    explanation_text: String,
    max_reveal_count: i64,
}

#[derive(Debug, Clone)]
struct TrapsMetadata {
    pair_id: i64,
    pair_title: String,
    left_label: String,
    right_label: String,
    summary_text: Option<String>,
    mode: String,
    recommended_mode: String,
    correct_discriminations: i64,
    total_discriminations: i64,
    current_round_id: Option<i64>,
    current_round_number: i64,
}

#[derive(Debug, Default, Clone)]
struct StudentContrastMetrics {
    accuracy_bp: BasisPoints,
    fluency_bp: BasisPoints,
    confusion_score: BasisPoints,
    difference_drill_bp: BasisPoints,
    similarity_trap_bp: BasisPoints,
    know_difference_bp: BasisPoints,
    which_is_which_bp: BasisPoints,
    unmask_bp: BasisPoints,
    rounds_played: i64,
    rounds_correct: i64,
    timed_out_count: i64,
    average_response_time_ms: i64,
    weakest_lane: Option<String>,
    last_mode: Option<String>,
    confusion_reason_profile: BTreeMap<String, i64>,
}

#[derive(Debug, Clone)]
struct StoredTrapRound {
    id: i64,
    pair_id: i64,
    round_number: i64,
    mode: String,
    lane: String,
    prompt_text: String,
    prompt_payload_json: String,
    options_json: String,
    correct_choice_code: String,
    correct_choice_label: String,
    explanation_text: String,
    reveal_count: i64,
    max_reveal_count: i64,
    answered_at: Option<String>,
}

pub struct GamesService<'a> {
    conn: &'a Connection,
}

impl<'a> GamesService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn start_game_session(&self, input: &StartGameInput) -> EcoachResult<GameSession> {
        let game_type_str = input.game_type.as_str();
        let topic_ids_json = serde_json::to_string(&input.topic_ids)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        let now = Utc::now().to_rfc3339();
        let question_count = input.question_count.max(5) as i64;

        let metadata = match input.game_type {
            GameType::Mindstack => json!({
                "board_height": 0,
                "cleared_rows": 0,
                "pending_block_type": "standard",
                "topic_ids": input.topic_ids,
            }),
            GameType::TugOfWar => json!({
                "position": 0,
                "opponent_difficulty": 5000,
                "topic_ids": input.topic_ids,
            }),
            GameType::Traps => json!({
                "correct_discriminations": 0,
                "total_discriminations": 0,
                "topic_ids": input.topic_ids,
            }),
        };

        self.conn
            .execute(
                "INSERT INTO game_sessions (
                    student_id, game_type, subject_id, session_state, score, rounds_total,
                    rounds_played, streak, best_streak, topic_ids_json, metadata_json, created_at
                 ) VALUES (?1, ?2, ?3, 'active', 0, ?4, 0, 0, 0, ?5, ?6, ?7)",
                params![
                    input.student_id,
                    game_type_str,
                    input.subject_id,
                    question_count,
                    topic_ids_json,
                    serde_json::to_string(&metadata)
                        .map_err(|e| EcoachError::Serialization(e.to_string()))?,
                    now,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let session_id = self.conn.last_insert_rowid();
        self.append_event(
            "game",
            DomainEvent::new(
                "game.session_started",
                session_id.to_string(),
                json!({
                    "student_id": input.student_id,
                    "game_type": game_type_str,
                    "subject_id": input.subject_id,
                    "question_count": question_count,
                }),
            ),
        )?;

        self.get_session(session_id)
    }

    pub fn submit_answer(&self, input: &SubmitGameAnswerInput) -> EcoachResult<GameAnswerResult> {
        let session = self.get_session(input.game_session_id)?;
        if session.session_state != "active" {
            return Err(EcoachError::Validation(format!(
                "game session {} is not active (state: {})",
                input.game_session_id, session.session_state
            )));
        }

        let (is_correct, misconception_id): (bool, Option<i64>) = self
            .conn
            .query_row(
                "SELECT qo.is_correct, qo.misconception_id
                 FROM question_options qo
                 WHERE qo.id = ?1 AND qo.question_id = ?2",
                params![input.selected_option_id, input.question_id],
                |row| Ok((row.get::<_, i64>(0)? == 1, row.get::<_, Option<i64>>(1)?)),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let misconception_triggered = misconception_id.is_some() && !is_correct;
        let game_type = GameType::from_str(&session.game_type).ok_or_else(|| {
            EcoachError::Validation(format!("unknown game type: {}", session.game_type))
        })?;
        let streak = if is_correct { session.streak + 1 } else { 0 };
        let streak_bonus = if is_correct {
            ((streak - 1).max(0) as f64 * STREAK_BONUS_MULTIPLIER * BASE_CORRECT_POINTS as f64)
                .round() as i64
        } else {
            0
        };
        let speed_bonus = if is_correct && input.response_time_ms < SPEED_BONUS_THRESHOLD_MS {
            ((SPEED_BONUS_THRESHOLD_MS - input.response_time_ms) / 100).min(50)
        } else {
            0
        };
        let misconception_pen = if misconception_triggered {
            MISCONCEPTION_PENALTY
        } else {
            0
        };
        let base_points = if is_correct {
            BASE_CORRECT_POINTS
        } else {
            BASE_INCORRECT_POINTS
        };
        let points_earned = (base_points + streak_bonus + speed_bonus - misconception_pen).max(0);
        let new_score = session.score + points_earned;
        let best_streak = streak.max(session.best_streak);
        let round_number = session.rounds_played + 1;
        let session_complete = round_number >= session.rounds_total;
        let effect_type = self.compute_game_effect(
            game_type,
            input.game_session_id,
            is_correct,
            misconception_triggered,
        )?;

        self.conn
            .execute(
                "INSERT INTO game_answer_events (
                    game_session_id, question_id, selected_option_id, was_correct,
                    response_time_ms, points_earned, streak_at_answer, misconception_triggered,
                    effect_type, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    input.game_session_id,
                    input.question_id,
                    input.selected_option_id,
                    if is_correct { 1 } else { 0 },
                    input.response_time_ms,
                    points_earned,
                    streak,
                    if misconception_triggered { 1 } else { 0 },
                    &effect_type,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let new_state = if session_complete {
            "completed"
        } else {
            "active"
        };
        self.conn
            .execute(
                "UPDATE game_sessions
                 SET score = ?1, rounds_played = ?2, streak = ?3, best_streak = ?4,
                     session_state = ?5, completed_at = CASE WHEN ?5 = 'completed' THEN datetime('now') ELSE completed_at END
                 WHERE id = ?6",
                params![new_score, round_number, streak, best_streak, new_state, input.game_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let explanation: Option<String> = self
            .conn
            .query_row(
                "SELECT explanation_text FROM questions WHERE id = ?1",
                [input.question_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .flatten();

        self.append_event(
            "game",
            DomainEvent::new(
                "game.answer_submitted",
                input.game_session_id.to_string(),
                json!({
                    "question_id": input.question_id,
                    "is_correct": is_correct,
                    "points_earned": points_earned,
                    "streak": streak,
                    "effect_type": effect_type,
                    "session_complete": session_complete,
                }),
            ),
        )?;

        if session_complete {
            let correct_answers = self.count_correct_game_answers(input.game_session_id)?;
            let accuracy_bp = count_correct_ratio(correct_answers, round_number);
            let focus_signals = build_game_focus_signals(
                &session.game_type,
                accuracy_bp,
                input.response_time_ms,
                misconception_triggered,
                false,
            );
            let recommended_next_step = recommend_game_next_step(
                &session.game_type,
                &focus_signals,
                misconception_triggered,
            );
            self.append_event(
                "game",
                DomainEvent::new(
                    "game.session_completed",
                    input.game_session_id.to_string(),
                    json!({
                        "student_id": session.student_id,
                        "subject_id": session.subject_id,
                        "game_type": session.game_type,
                        "score": new_score,
                        "rounds_played": round_number,
                        "accuracy_bp": accuracy_bp,
                        "focus_signals": focus_signals,
                        "recommended_next_step": recommended_next_step,
                    }),
                ),
            )?;
        }

        Ok(GameAnswerResult {
            is_correct,
            points_earned,
            new_score,
            streak,
            effect_type,
            round_number,
            session_complete,
            explanation,
            misconception_triggered,
        })
    }

    pub fn list_traps_pairs(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_ids: &[i64],
    ) -> EcoachResult<Vec<ContrastPairSummary>> {
        let mut pairs = self.load_contrast_pairs(student_id, subject_id)?;
        if !topic_ids.is_empty() {
            pairs.retain(|pair| match pair.topic_id {
                Some(topic_id) => topic_ids.contains(&topic_id),
                None => true,
            });
        }

        Ok(pairs
            .into_iter()
            .map(|pair| ContrastPairSummary {
                pair_id: pair.id,
                pair_code: pair.pair_code,
                title: pair.title,
                left_label: pair.left_label,
                right_label: pair.right_label,
                summary_text: pair.summary_text,
                trap_strength: pair.trap_strength,
                difficulty_score: pair.difficulty_score,
                confusion_score: pair.confusion_score,
                last_accuracy_bp: pair.last_accuracy_bp,
                recommended_mode: pair.recommended_mode,
                available_modes: vec![
                    TrapsMode::DifferenceDrill.as_str().to_string(),
                    TrapsMode::SimilarityTrap.as_str().to_string(),
                    TrapsMode::KnowTheDifference.as_str().to_string(),
                    TrapsMode::WhichIsWhich.as_str().to_string(),
                    TrapsMode::Unmask.as_str().to_string(),
                ],
            })
            .collect())
    }

    pub fn start_traps_session(
        &self,
        input: &StartTrapsSessionInput,
    ) -> EcoachResult<TrapSessionSnapshot> {
        let pair = self.select_contrast_pair(
            input.student_id,
            input.subject_id,
            &input.topic_ids,
            input.pair_id,
        )?;
        let atoms = self.load_contrast_atoms(pair.id)?;
        let rounds =
            self.build_traps_rounds(input.mode, &pair, &atoms, input.round_count.max(4))?;
        if rounds.is_empty() {
            return Err(EcoachError::Validation(
                "no traps rounds could be generated for the selected pair".to_string(),
            ));
        }

        let metadata = json!({
            "pair_id": pair.id,
            "pair_title": pair.title,
            "left_label": pair.left_label,
            "right_label": pair.right_label,
            "summary_text": pair.summary_text,
            "mode": input.mode.as_str(),
            "recommended_mode": pair.recommended_mode,
            "correct_discriminations": 0,
            "total_discriminations": 0,
            "current_round_id": Value::Null,
            "current_round_number": 1,
            "timer_seconds": input.timer_seconds.unwrap_or(default_timer_seconds(input.mode)),
        });

        self.conn
            .execute(
                "INSERT INTO game_sessions (
                    student_id, game_type, subject_id, session_state, score, rounds_total,
                    rounds_played, streak, best_streak, topic_ids_json, metadata_json, created_at
                 ) VALUES (?1, 'traps', ?2, 'active', 0, ?3, 0, 0, 0, ?4, ?5, ?6)",
                params![
                    input.student_id,
                    input.subject_id,
                    rounds.len() as i64,
                    serde_json::to_string(&input.topic_ids)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&metadata)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let session_id = self.conn.last_insert_rowid();
        let mut first_round_id = None;
        for (index, round) in rounds.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO traps_rounds (
                        game_session_id, pair_id, atom_id, round_number, mode, lane, prompt_text,
                        prompt_payload_json, options_json, correct_choice_code, correct_choice_label,
                        explanation_text, max_reveal_count
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                    params![
                        session_id,
                        pair.id,
                        round.atom_id,
                        (index + 1) as i64,
                        input.mode.as_str(),
                        round.lane,
                        round.prompt_text,
                        serde_json::to_string(&round.prompt_payload)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        serde_json::to_string(&round.answer_options)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        round.correct_choice_code,
                        round.correct_choice_label,
                        round.explanation_text,
                        round.max_reveal_count,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if index == 0 {
                first_round_id = Some(self.conn.last_insert_rowid());
            }
        }

        self.update_traps_metadata(session_id, |metadata| {
            metadata["current_round_id"] = first_round_id.map_or(Value::Null, Value::from);
            metadata["current_round_number"] = json!(1);
        })?;

        self.append_event(
            "game",
            DomainEvent::new(
                "traps.session_started",
                session_id.to_string(),
                json!({
                    "student_id": input.student_id,
                    "subject_id": input.subject_id,
                    "pair_id": pair.id,
                    "mode": input.mode.as_str(),
                    "rounds_total": rounds.len(),
                }),
            ),
        )?;

        self.get_traps_snapshot(session_id)
    }

    pub fn get_traps_snapshot(&self, game_session_id: i64) -> EcoachResult<TrapSessionSnapshot> {
        let (session, metadata) = self.load_traps_session(game_session_id)?;
        let state = self.build_traps_state(&session, &metadata)?;

        let mut statement = self
            .conn
            .prepare(
                "SELECT id, round_number, pair_id, mode, lane, prompt_text, prompt_payload_json,
                        options_json, reveal_count, max_reveal_count, answered_at
                 FROM traps_rounds
                 WHERE game_session_id = ?1
                 ORDER BY round_number ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([game_session_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, i64>(9)?,
                    row.get::<_, Option<String>>(10)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut rounds = Vec::new();
        for row in rows {
            let (
                id,
                round_number,
                pair_id,
                mode,
                lane,
                prompt_text,
                prompt_payload_json,
                options_json,
                reveal_count,
                max_reveal_count,
                answered_at,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let prompt_payload = parse_json_value(&prompt_payload_json)?;
            let answer_options = parse_json_choices(&options_json)?;
            let status = if answered_at.is_some() {
                "answered"
            } else if metadata.current_round_id == Some(id) && session.session_state == "active" {
                "active"
            } else {
                "queued"
            };

            rounds.push(TrapRoundCard {
                id,
                round_number,
                pair_id,
                mode: mode.clone(),
                lane,
                prompt_text: render_prompt_text(
                    &mode,
                    &prompt_text,
                    &prompt_payload,
                    reveal_count,
                    false,
                ),
                prompt_payload,
                answer_options,
                reveal_count,
                max_reveal_count,
                status: status.to_string(),
            });
        }

        Ok(TrapSessionSnapshot {
            session,
            state,
            left_label: metadata.left_label,
            right_label: metadata.right_label,
            summary_text: metadata.summary_text,
            recommended_mode: metadata.recommended_mode,
            rounds,
        })
    }

    pub fn submit_trap_round(&self, input: &SubmitTrapRoundInput) -> EcoachResult<TrapRoundResult> {
        let (session, metadata) = self.load_traps_session(input.game_session_id)?;
        if session.session_state != "active" {
            return Err(EcoachError::Validation(format!(
                "traps session {} is not active",
                input.game_session_id
            )));
        }

        let round = self.load_trap_round(input.game_session_id, input.round_id)?;
        if round.answered_at.is_some() {
            return Err(EcoachError::Validation(
                "trap round has already been answered".to_string(),
            ));
        }

        let mode = TrapsMode::from_str(&round.mode)
            .ok_or_else(|| EcoachError::Validation(format!("unknown traps mode {}", round.mode)))?;
        let answer_options = parse_json_choices(&round.options_json)?;
        let selected_choice_label = input
            .selected_choice_code
            .as_deref()
            .and_then(|code| choice_label_for_code(&answer_options, code));
        let is_correct = !input.timed_out
            && input.selected_choice_code.as_deref() == Some(round.correct_choice_code.as_str());
        let skipped = input.timed_out || input.selected_choice_code.is_none();
        let streak = if is_correct { session.streak + 1 } else { 0 };
        let best_streak = streak.max(session.best_streak);
        let score_earned = calculate_trap_score(
            mode,
            is_correct,
            skipped,
            streak,
            input.response_time_ms,
            round.reveal_count,
        );
        let new_score = session.score + score_earned;
        let next_round_id =
            self.find_next_trap_round_id(input.game_session_id, round.round_number)?;
        let session_complete = next_round_id.is_none();
        let new_state = if session_complete {
            "completed"
        } else {
            "active"
        };
        let confusion_signal = classify_trap_confusion_signal(
            mode,
            is_correct,
            input.timed_out,
            input.response_time_ms,
            round.reveal_count,
        )
        .to_string();

        self.conn
            .execute(
                "UPDATE traps_rounds
                 SET selected_choice_code = ?1,
                     selected_choice_label = ?2,
                     is_correct = ?3,
                     response_time_ms = ?4,
                     timed_out = ?5,
                     skipped = ?6,
                     score_earned = ?7,
                     answered_at = ?8
                 WHERE id = ?9",
                params![
                    input.selected_choice_code,
                    selected_choice_label,
                    if is_correct { 1 } else { 0 },
                    input.response_time_ms,
                    if input.timed_out { 1 } else { 0 },
                    if skipped { 1 } else { 0 },
                    score_earned,
                    Utc::now().to_rfc3339(),
                    input.round_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO game_answer_events (
                    game_session_id, question_id, selected_option_id, was_correct,
                    response_time_ms, points_earned, streak_at_answer, misconception_triggered,
                    effect_type, created_at
                 ) VALUES (?1, NULL, NULL, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
                params![
                    input.game_session_id,
                    if is_correct { 1 } else { 0 },
                    input.response_time_ms,
                    score_earned,
                    streak,
                    confusion_signal,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE game_sessions
                 SET score = ?1,
                     rounds_played = ?2,
                     streak = ?3,
                     best_streak = ?4,
                     session_state = ?5,
                     completed_at = CASE WHEN ?5 = 'completed' THEN datetime('now') ELSE completed_at END
                 WHERE id = ?6",
                params![
                    new_score,
                    session.rounds_played + 1,
                    streak,
                    best_streak,
                    new_state,
                    input.game_session_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.update_traps_metadata(input.game_session_id, |metadata_json| {
            metadata_json["correct_discriminations"] =
                json!(metadata.correct_discriminations + if is_correct { 1 } else { 0 });
            metadata_json["total_discriminations"] = json!(metadata.total_discriminations + 1);
            metadata_json["current_round_id"] = next_round_id.map_or(Value::Null, Value::from);
            metadata_json["current_round_number"] = json!(round.round_number + 1);
        })?;

        let updated_confusion_score = self.upsert_student_contrast_state_round(
            session.student_id,
            metadata.pair_id,
            mode,
            &round.lane,
            is_correct,
            input.response_time_ms,
            input.timed_out,
        )?;

        self.append_event(
            "game",
            DomainEvent::new(
                "traps.round_answered",
                input.game_session_id.to_string(),
                json!({
                    "round_id": input.round_id,
                    "pair_id": metadata.pair_id,
                    "mode": mode.as_str(),
                    "lane": round.lane,
                    "is_correct": is_correct,
                    "timed_out": input.timed_out,
                    "score_earned": score_earned,
                    "session_complete": session_complete,
                    "confusion_score": updated_confusion_score,
                }),
            ),
        )?;

        if session_complete {
            let completed_correct =
                metadata.correct_discriminations + if is_correct { 1 } else { 0 };
            let completed_total = metadata.total_discriminations + 1;
            let accuracy_bp = if completed_total > 0 {
                to_bp(completed_correct as f64 / completed_total as f64)
            } else {
                0
            };
            let metrics =
                self.load_student_contrast_metrics(session.student_id, metadata.pair_id)?;
            let recommended_next_mode = resolve_recommended_traps_mode(
                metrics.difference_drill_bp,
                metrics.similarity_trap_bp,
                metrics.know_difference_bp,
                metrics.which_is_which_bp,
                metrics.unmask_bp,
                metrics.confusion_score,
            )
            .to_string();
            let dominant_confusion_reason =
                dominant_confusion_reason(&metrics.confusion_reason_profile);
            let remediation_actions = build_traps_remediation_actions(
                accuracy_bp,
                metrics.confusion_score,
                metrics.weakest_lane.as_deref(),
                metrics.timed_out_count,
                dominant_confusion_reason.as_deref(),
                &recommended_next_mode,
            );
            self.append_event(
                "game",
                DomainEvent::new(
                    "traps.session_completed",
                    input.game_session_id.to_string(),
                    json!({
                        "student_id": session.student_id,
                        "subject_id": session.subject_id,
                        "pair_id": metadata.pair_id,
                        "pair_title": metadata.pair_title,
                        "mode": metadata.mode,
                        "accuracy_bp": accuracy_bp,
                        "confusion_score": metrics.confusion_score,
                        "weakest_lane": metrics.weakest_lane,
                        "timed_out_count": metrics.timed_out_count,
                        "recommended_next_mode": recommended_next_mode,
                        "dominant_confusion_reason": dominant_confusion_reason,
                        "remediation_actions": remediation_actions,
                    }),
                ),
            )?;
        }

        Ok(TrapRoundResult {
            round_id: input.round_id,
            round_number: round.round_number,
            is_correct,
            score_earned,
            new_score,
            streak,
            session_complete,
            selected_choice_code: input.selected_choice_code.clone(),
            selected_choice_label,
            correct_choice_code: round.correct_choice_code,
            correct_choice_label: round.correct_choice_label,
            explanation_text: round.explanation_text,
            confusion_signal,
            next_round_id,
        })
    }

    pub fn reveal_unmask_clue(
        &self,
        game_session_id: i64,
        round_id: i64,
    ) -> EcoachResult<TrapRoundCard> {
        let (session, metadata) = self.load_traps_session(game_session_id)?;
        if session.session_state != "active" {
            return Err(EcoachError::Validation(
                "unmask clue reveal requires an active session".to_string(),
            ));
        }

        let round = self.load_trap_round(game_session_id, round_id)?;
        if round.mode != TrapsMode::Unmask.as_str() {
            return Err(EcoachError::Validation(
                "clue reveal is only available for unmask rounds".to_string(),
            ));
        }
        if round.answered_at.is_some() {
            return Err(EcoachError::Validation(
                "cannot reveal more clues for an answered round".to_string(),
            ));
        }
        if round.reveal_count >= round.max_reveal_count {
            return Err(EcoachError::Validation(
                "all clues for this unmask round are already visible".to_string(),
            ));
        }

        let next_reveal_count = round.reveal_count + 1;
        self.conn
            .execute(
                "UPDATE traps_rounds SET reveal_count = ?1 WHERE id = ?2",
                params![next_reveal_count, round_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.append_event(
            "game",
            DomainEvent::new(
                "traps.unmask_clue_revealed",
                game_session_id.to_string(),
                json!({
                    "round_id": round_id,
                    "pair_id": metadata.pair_id,
                    "reveal_count": next_reveal_count,
                }),
            ),
        )?;

        let prompt_payload = parse_json_value(&round.prompt_payload_json)?;
        Ok(TrapRoundCard {
            id: round.id,
            round_number: round.round_number,
            pair_id: round.pair_id,
            mode: round.mode.clone(),
            lane: round.lane,
            prompt_text: render_prompt_text(
                TrapsMode::Unmask.as_str(),
                &round.prompt_text,
                &prompt_payload,
                next_reveal_count,
                false,
            ),
            prompt_payload,
            answer_options: parse_json_choices(&round.options_json)?,
            reveal_count: next_reveal_count,
            max_reveal_count: round.max_reveal_count,
            status: if metadata.current_round_id == Some(round.id) {
                "active".to_string()
            } else {
                "queued".to_string()
            },
        })
    }

    pub fn record_trap_confusion_reason(
        &self,
        input: &SubmitTrapConfusionReasonInput,
    ) -> EcoachResult<()> {
        let (game_session_id, pair_id, mode, student_id) = self
            .conn
            .query_row(
                "SELECT tr.game_session_id, tr.pair_id, tr.mode, gs.student_id
                 FROM traps_rounds tr
                 INNER JOIN game_sessions gs ON gs.id = tr.game_session_id
                 WHERE tr.id = ?1",
                [input.round_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE traps_rounds
                 SET confusion_reason_code = ?1,
                     confusion_reason_text = ?2
                 WHERE id = ?3",
                params![input.reason_code, input.reason_text, input.round_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let traps_mode = TrapsMode::from_str(&mode)
            .ok_or_else(|| EcoachError::Validation(format!("unknown traps mode {}", mode)))?;
        self.upsert_student_contrast_reason(student_id, pair_id, traps_mode, &input.reason_code)?;

        self.append_event(
            "game",
            DomainEvent::new(
                "traps.confusion_reason_recorded",
                game_session_id.to_string(),
                json!({
                    "round_id": input.round_id,
                    "pair_id": pair_id,
                    "mode": mode,
                    "reason_code": input.reason_code,
                }),
            ),
        )?;

        Ok(())
    }

    pub fn get_traps_review(&self, game_session_id: i64) -> EcoachResult<TrapSessionReview> {
        let (session, metadata) = self.load_traps_session(game_session_id)?;
        let mut statement = self
            .conn
            .prepare(
                "SELECT round_number, mode, lane, prompt_text, prompt_payload_json, reveal_count,
                        correct_choice_label, selected_choice_label, COALESCE(is_correct, 0),
                        timed_out, response_time_ms, confusion_reason_code, confusion_reason_text,
                        explanation_text, id
                 FROM traps_rounds
                 WHERE game_session_id = ?1
                 ORDER BY round_number ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([game_session_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, i64>(8)? == 1,
                    row.get::<_, i64>(9)? == 1,
                    row.get::<_, Option<i64>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, String>(13)?,
                    row.get::<_, i64>(14)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut rounds = Vec::new();
        let mut correct = 0_i64;
        let mut total = 0_i64;
        let mut timed_out_count = 0_i64;
        for row in rows {
            let (
                round_number,
                mode,
                lane,
                prompt_text,
                prompt_payload_json,
                reveal_count,
                correct_choice_label,
                selected_choice_label,
                is_correct,
                timed_out,
                response_time_ms,
                confusion_reason_code,
                confusion_reason_text,
                explanation_text,
                round_id,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            total += 1;
            correct += if is_correct { 1 } else { 0 };
            timed_out_count += if timed_out { 1 } else { 0 };
            let prompt_payload = parse_json_value(&prompt_payload_json)?;
            rounds.push(crate::models::TrapReviewRound {
                round_id,
                round_number,
                mode: mode.clone(),
                lane,
                prompt_text: render_prompt_text(
                    &mode,
                    &prompt_text,
                    &prompt_payload,
                    reveal_count,
                    true,
                ),
                selected_choice_label,
                correct_choice_label,
                is_correct,
                timed_out,
                response_time_ms,
                confusion_reason_code,
                confusion_reason_text,
                explanation_text,
            });
        }

        let accuracy_bp = if total > 0 {
            to_bp(correct as f64 / total as f64)
        } else {
            0
        };
        let contrast_metrics =
            self.load_student_contrast_metrics(session.student_id, metadata.pair_id)?;
        let recommended_next_mode = resolve_recommended_traps_mode(
            contrast_metrics.difference_drill_bp,
            contrast_metrics.similarity_trap_bp,
            contrast_metrics.know_difference_bp,
            contrast_metrics.which_is_which_bp,
            contrast_metrics.unmask_bp,
            contrast_metrics.confusion_score,
        )
        .to_string();
        let dominant_confusion_reason =
            dominant_confusion_reason(&contrast_metrics.confusion_reason_profile);
        let remediation_actions = build_traps_remediation_actions(
            accuracy_bp,
            contrast_metrics.confusion_score,
            contrast_metrics.weakest_lane.as_deref(),
            timed_out_count,
            dominant_confusion_reason.as_deref(),
            &recommended_next_mode,
        );

        Ok(TrapSessionReview {
            session_id: game_session_id,
            pair_id: metadata.pair_id,
            pair_title: metadata.pair_title,
            mode: metadata.mode,
            score: session.score,
            accuracy_bp,
            confusion_score: contrast_metrics.confusion_score,
            weakest_lane: contrast_metrics.weakest_lane,
            timed_out_count,
            recommended_next_mode,
            dominant_confusion_reason,
            remediation_actions,
            rounds,
        })
    }

    pub fn pause_session(&self, game_session_id: i64) -> EcoachResult<()> {
        let affected = self
            .conn
            .execute(
                "UPDATE game_sessions SET session_state = 'paused' WHERE id = ?1 AND session_state = 'active'",
                [game_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(EcoachError::Validation(
                "session is not active or does not exist".to_string(),
            ));
        }
        Ok(())
    }

    pub fn resume_session(&self, game_session_id: i64) -> EcoachResult<()> {
        let affected = self
            .conn
            .execute(
                "UPDATE game_sessions SET session_state = 'active' WHERE id = ?1 AND session_state = 'paused'",
                [game_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(EcoachError::Validation(
                "session is not paused or does not exist".to_string(),
            ));
        }
        Ok(())
    }

    pub fn abandon_session(&self, game_session_id: i64) -> EcoachResult<()> {
        let affected = self
            .conn
            .execute(
                "UPDATE game_sessions
                 SET session_state = 'abandoned', completed_at = datetime('now')
                 WHERE id = ?1 AND session_state IN ('active', 'paused')",
                [game_session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(EcoachError::Validation(
                "session cannot be abandoned (already completed or does not exist)".to_string(),
            ));
        }
        self.append_event(
            "game",
            DomainEvent::new(
                "game.session_abandoned",
                game_session_id.to_string(),
                json!({}),
            ),
        )?;
        Ok(())
    }

    pub fn get_session(&self, game_session_id: i64) -> EcoachResult<GameSession> {
        self.conn
            .query_row(
                "SELECT id, student_id, game_type, subject_id, session_state, score,
                        rounds_total, rounds_played, streak, best_streak, created_at, completed_at
                 FROM game_sessions WHERE id = ?1",
                [game_session_id],
                map_game_session,
            )
            .map_err(|e| {
                EcoachError::NotFound(format!("game session {} not found: {}", game_session_id, e))
            })
    }

    fn count_correct_game_answers(&self, game_session_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COALESCE(SUM(was_correct), 0)
                 FROM game_answer_events
                 WHERE game_session_id = ?1",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }

    pub fn get_summary(&self, game_session_id: i64) -> EcoachResult<GameSummary> {
        let session = self.get_session(game_session_id)?;
        if session.session_state != "completed" && session.session_state != "abandoned" {
            return Err(EcoachError::Validation(
                "session is still in progress".to_string(),
            ));
        }

        let stats: (i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(was_correct), 0), CAST(COALESCE(AVG(response_time_ms), 0) AS INTEGER),
                        COALESCE(SUM(misconception_triggered), 0)
                 FROM game_answer_events WHERE game_session_id = ?1",
                [game_session_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let (total, correct, avg_time_ms, misconception_hits) = stats;
        let accuracy_bp: BasisPoints = if total > 0 {
            to_bp(correct as f64 / total as f64)
        } else {
            0
        };

        let performance_label = match accuracy_bp {
            0..=2999 => "needs_practice",
            3000..=5999 => "building",
            6000..=7999 => "strong",
            8000..=9499 => "excellent",
            _ => "perfect_run",
        }
        .to_string();
        let focus_signals = build_game_focus_signals(
            &session.game_type,
            accuracy_bp,
            avg_time_ms,
            misconception_hits > 0,
            session.session_state == "abandoned",
        );
        let recommended_next_step =
            recommend_game_next_step(&session.game_type, &focus_signals, misconception_hits > 0);

        Ok(GameSummary {
            session_id: game_session_id,
            game_type: session.game_type,
            score: session.score,
            accuracy_bp,
            rounds_played: session.rounds_played,
            best_streak: session.best_streak,
            average_response_time_ms: avg_time_ms,
            misconception_hits,
            performance_label,
            focus_signals,
            recommended_next_step: Some(recommended_next_step),
        })
    }

    pub fn list_sessions_for_student(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<GameSession>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, student_id, game_type, subject_id, session_state, score,
                        rounds_total, rounds_played, streak, best_streak, created_at, completed_at
                 FROM game_sessions
                 WHERE student_id = ?1
                 ORDER BY created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, limit as i64], map_game_session)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(sessions)
    }

    pub fn get_leaderboard(
        &self,
        game_type: GameType,
        limit: usize,
    ) -> EcoachResult<Vec<GameLeaderboardEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT gs.student_id, a.display_name, gs.game_type,
                        MAX(gs.score) AS best_score, COUNT(*) AS games_played
                 FROM game_sessions gs
                 INNER JOIN accounts a ON a.id = gs.student_id
                 WHERE gs.game_type = ?1 AND gs.session_state = 'completed'
                 GROUP BY gs.student_id
                 ORDER BY best_score DESC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![game_type.as_str(), limit as i64], |row| {
                Ok(GameLeaderboardEntry {
                    student_id: row.get(0)?,
                    display_name: row.get(1)?,
                    game_type: row.get(2)?,
                    best_score: row.get(3)?,
                    games_played: row.get(4)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(entries)
    }

    pub fn get_mindstack_state(&self, game_session_id: i64) -> EcoachResult<MindstackState> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1 AND game_type = 'mindstack'",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::NotFound(format!("mindstack session not found: {}", e)))?;
        let val: Value = serde_json::from_str(&metadata_json)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        Ok(MindstackState {
            board_height: val["board_height"].as_i64().unwrap_or(0),
            cleared_rows: val["cleared_rows"].as_i64().unwrap_or(0),
            pending_block_type: val["pending_block_type"]
                .as_str()
                .unwrap_or("standard")
                .to_string(),
        })
    }

    pub fn get_tug_of_war_state(&self, game_session_id: i64) -> EcoachResult<TugOfWarState> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1 AND game_type = 'tug_of_war'",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::NotFound(format!("tug_of_war session not found: {}", e)))?;
        let val: Value = serde_json::from_str(&metadata_json)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        Ok(TugOfWarState {
            position: val["position"].as_i64().unwrap_or(0),
            opponent_difficulty: clamp_bp(val["opponent_difficulty"].as_i64().unwrap_or(5000)),
        })
    }

    pub fn get_traps_state(&self, game_session_id: i64) -> EcoachResult<TrapsState> {
        let (session, metadata) = self.load_traps_session(game_session_id)?;
        self.build_traps_state(&session, &metadata)
    }

    fn build_traps_state(
        &self,
        session: &GameSession,
        metadata: &TrapsMetadata,
    ) -> EcoachResult<TrapsState> {
        let confusion_score = self
            .load_student_contrast_metrics(session.student_id, metadata.pair_id)?
            .confusion_score;
        Ok(TrapsState {
            pair_id: metadata.pair_id,
            pair_title: metadata.pair_title.clone(),
            mode: metadata.mode.clone(),
            correct_discriminations: metadata.correct_discriminations,
            total_discriminations: metadata.total_discriminations,
            confusion_score,
            current_round_id: metadata.current_round_id,
            current_round_number: metadata.current_round_number,
        })
    }

    fn load_traps_session(
        &self,
        game_session_id: i64,
    ) -> EcoachResult<(GameSession, TrapsMetadata)> {
        let (session, metadata_json): (GameSession, String) = self
            .conn
            .query_row(
                "SELECT id, student_id, game_type, subject_id, session_state, score,
                        rounds_total, rounds_played, streak, best_streak, created_at, completed_at,
                        metadata_json
                 FROM game_sessions
                 WHERE id = ?1 AND game_type = 'traps'",
                [game_session_id],
                |row| Ok((map_game_session(row)?, row.get::<_, String>(12)?)),
            )
            .map_err(|err| {
                EcoachError::NotFound(format!(
                    "traps session {} not found: {}",
                    game_session_id, err
                ))
            })?;
        let metadata_value = parse_json_value(&metadata_json)?;
        Ok((session, parse_traps_metadata(&metadata_value)?))
    }

    fn select_contrast_pair(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_ids: &[i64],
        pair_id: Option<i64>,
    ) -> EcoachResult<ContrastPairContext> {
        let mut pairs = self.load_contrast_pairs(student_id, subject_id)?;
        if !topic_ids.is_empty() {
            pairs.retain(|pair| match pair.topic_id {
                Some(topic_id) => topic_ids.contains(&topic_id),
                None => true,
            });
        }

        if let Some(pair_id) = pair_id {
            return pairs
                .into_iter()
                .find(|pair| pair.id == pair_id)
                .ok_or_else(|| {
                    EcoachError::NotFound(format!("contrast pair {} not found", pair_id))
                });
        }

        pairs.into_iter().next().ok_or_else(|| {
            EcoachError::Validation(
                "no contrast pairs are available for the requested subject scope".to_string(),
            )
        })
    }

    fn load_contrast_pairs(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<ContrastPairContext>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    cp.id,
                    cp.pair_code,
                    cp.title,
                    cp.subject_id,
                    cp.topic_id,
                    COALESCE(cp.left_label, lk.title),
                    COALESCE(cp.right_label, rk.title),
                    cp.summary_text,
                    cp.trap_strength,
                    cp.difficulty_score,
                    COALESCE(scs.confusion_score, cp.trap_strength),
                    COALESCE(scs.accuracy_bp, 0),
                    COALESCE(scs.difference_drill_bp, 0),
                    COALESCE(scs.similarity_trap_bp, 0),
                    COALESCE(scs.know_difference_bp, 0),
                    COALESCE(scs.which_is_which_bp, 0),
                    COALESCE(scs.unmask_bp, 0)
                 FROM contrast_pairs cp
                 INNER JOIN knowledge_entries lk ON lk.id = cp.left_entry_id
                 INNER JOIN knowledge_entries rk ON rk.id = cp.right_entry_id
                 LEFT JOIN student_contrast_states scs
                    ON scs.student_id = ?1 AND scs.pair_id = cp.id
                 WHERE cp.subject_id = ?2
                 ORDER BY COALESCE(scs.confusion_score, cp.trap_strength) DESC,
                          cp.trap_strength DESC,
                          cp.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                Ok(ContrastPairContext {
                    id: row.get(0)?,
                    pair_code: row.get(1)?,
                    title: row.get(2)?,
                    topic_id: row.get(4)?,
                    left_label: row.get(5)?,
                    right_label: row.get(6)?,
                    summary_text: row.get(7)?,
                    trap_strength: clamp_bp(row.get::<_, i64>(8)?),
                    difficulty_score: clamp_bp(row.get::<_, i64>(9)?),
                    confusion_score: clamp_bp(row.get::<_, i64>(10)?),
                    last_accuracy_bp: clamp_bp(row.get::<_, i64>(11)?),
                    recommended_mode: resolve_recommended_traps_mode(
                        clamp_bp(row.get::<_, i64>(12)?),
                        clamp_bp(row.get::<_, i64>(13)?),
                        clamp_bp(row.get::<_, i64>(14)?),
                        clamp_bp(row.get::<_, i64>(15)?),
                        clamp_bp(row.get::<_, i64>(16)?),
                        clamp_bp(row.get::<_, i64>(10)?),
                    )
                    .to_string(),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut pairs = Vec::new();
        for row in rows {
            pairs.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(pairs)
    }

    fn load_contrast_atoms(&self, pair_id: i64) -> EcoachResult<Vec<ContrastAtomContext>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, ownership_type, atom_text, lane, explanation_text,
                        difficulty_score, is_speed_ready, reveal_order
                 FROM contrast_evidence_atoms
                 WHERE pair_id = ?1
                 ORDER BY reveal_order ASC, difficulty_score ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([pair_id], |row| {
                Ok(ContrastAtomContext {
                    id: row.get(0)?,
                    ownership_type: row.get(1)?,
                    atom_text: row.get(2)?,
                    lane: row.get(3)?,
                    explanation_text: row.get(4)?,
                    is_speed_ready: row.get::<_, i64>(6)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut atoms = Vec::new();
        for row in rows {
            atoms.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(atoms)
    }

    fn build_traps_rounds(
        &self,
        mode: TrapsMode,
        pair: &ContrastPairContext,
        atoms: &[ContrastAtomContext],
        round_count: usize,
    ) -> EcoachResult<Vec<TrapRoundBlueprint>> {
        match mode {
            TrapsMode::DifferenceDrill => build_difference_drill_rounds(pair, atoms, round_count),
            TrapsMode::SimilarityTrap => build_similarity_trap_rounds(pair, atoms, round_count),
            TrapsMode::KnowTheDifference => build_know_difference_rounds(pair, atoms, round_count),
            TrapsMode::WhichIsWhich => build_which_is_which_rounds(pair, atoms, round_count),
            TrapsMode::Unmask => build_unmask_rounds(pair, atoms, round_count),
        }
    }

    fn find_next_trap_round_id(
        &self,
        game_session_id: i64,
        round_number: i64,
    ) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT id
                 FROM traps_rounds
                 WHERE game_session_id = ?1
                   AND round_number > ?2
                   AND answered_at IS NULL
                 ORDER BY round_number ASC
                 LIMIT 1",
                params![game_session_id, round_number],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn update_traps_metadata<F>(&self, game_session_id: i64, mutator: F) -> EcoachResult<()>
    where
        F: FnOnce(&mut Value),
    {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut metadata = parse_json_value(&metadata_json)?;
        mutator(&mut metadata);
        self.conn
            .execute(
                "UPDATE game_sessions SET metadata_json = ?1 WHERE id = ?2",
                params![
                    serde_json::to_string(&metadata)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    game_session_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn load_trap_round(
        &self,
        game_session_id: i64,
        round_id: i64,
    ) -> EcoachResult<StoredTrapRound> {
        self.conn
            .query_row(
                "SELECT id, pair_id, round_number, mode, lane, prompt_text, prompt_payload_json,
                        options_json, correct_choice_code, correct_choice_label, explanation_text,
                        reveal_count, max_reveal_count, answered_at
                 FROM traps_rounds
                 WHERE id = ?1 AND game_session_id = ?2",
                params![round_id, game_session_id],
                map_stored_trap_round,
            )
            .map_err(|err| {
                EcoachError::NotFound(format!(
                    "trap round {} not found for session {}: {}",
                    round_id, game_session_id, err
                ))
            })
    }

    fn load_student_contrast_metrics(
        &self,
        student_id: i64,
        pair_id: i64,
    ) -> EcoachResult<StudentContrastMetrics> {
        self.conn
            .query_row(
                "SELECT accuracy_bp, fluency_bp, confusion_score, difference_drill_bp,
                        similarity_trap_bp, know_difference_bp, which_is_which_bp, unmask_bp,
                        rounds_played, rounds_correct, timed_out_count, average_response_time_ms,
                        weakest_lane, last_mode, confusion_reason_profile_json
                 FROM student_contrast_states
                 WHERE student_id = ?1 AND pair_id = ?2",
                params![student_id, pair_id],
                |row| {
                    let reason_json: String = row.get(14)?;
                    let confusion_reason_profile =
                        serde_json::from_str(&reason_json).unwrap_or_default();
                    Ok(StudentContrastMetrics {
                        accuracy_bp: clamp_bp(row.get::<_, i64>(0)?),
                        fluency_bp: clamp_bp(row.get::<_, i64>(1)?),
                        confusion_score: clamp_bp(row.get::<_, i64>(2)?),
                        difference_drill_bp: clamp_bp(row.get::<_, i64>(3)?),
                        similarity_trap_bp: clamp_bp(row.get::<_, i64>(4)?),
                        know_difference_bp: clamp_bp(row.get::<_, i64>(5)?),
                        which_is_which_bp: clamp_bp(row.get::<_, i64>(6)?),
                        unmask_bp: clamp_bp(row.get::<_, i64>(7)?),
                        rounds_played: row.get(8)?,
                        rounds_correct: row.get(9)?,
                        timed_out_count: row.get(10)?,
                        average_response_time_ms: row.get(11)?,
                        weakest_lane: row.get(12)?,
                        last_mode: row.get(13)?,
                        confusion_reason_profile,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .map_or_else(|| Ok(StudentContrastMetrics::default()), Ok)
    }

    fn upsert_student_contrast_state_round(
        &self,
        student_id: i64,
        pair_id: i64,
        mode: TrapsMode,
        lane: &str,
        is_correct: bool,
        response_time_ms: i64,
        timed_out: bool,
    ) -> EcoachResult<BasisPoints> {
        let mut metrics = self.load_student_contrast_metrics(student_id, pair_id)?;
        let previous_rounds = metrics.rounds_played.max(0);
        metrics.rounds_played += 1;
        metrics.rounds_correct += if is_correct { 1 } else { 0 };
        metrics.timed_out_count += if timed_out { 1 } else { 0 };
        metrics.average_response_time_ms = rolling_average(
            metrics.average_response_time_ms,
            previous_rounds,
            response_time_ms,
        );
        metrics.accuracy_bp = if metrics.rounds_played > 0 {
            to_bp(metrics.rounds_correct as f64 / metrics.rounds_played as f64)
        } else {
            0
        };
        let round_mode_bp = trap_mode_performance_bp(mode, is_correct, timed_out, response_time_ms);
        metrics.fluency_bp = rolling_bp(metrics.fluency_bp, round_mode_bp, metrics.rounds_played);
        match mode {
            TrapsMode::DifferenceDrill => {
                metrics.difference_drill_bp = rolling_bp(
                    metrics.difference_drill_bp,
                    round_mode_bp,
                    metrics.rounds_played,
                );
            }
            TrapsMode::SimilarityTrap => {
                metrics.similarity_trap_bp = rolling_bp(
                    metrics.similarity_trap_bp,
                    round_mode_bp,
                    metrics.rounds_played,
                );
            }
            TrapsMode::KnowTheDifference => {
                metrics.know_difference_bp = rolling_bp(
                    metrics.know_difference_bp,
                    round_mode_bp,
                    metrics.rounds_played,
                );
            }
            TrapsMode::WhichIsWhich => {
                metrics.which_is_which_bp = rolling_bp(
                    metrics.which_is_which_bp,
                    round_mode_bp,
                    metrics.rounds_played,
                );
            }
            TrapsMode::Unmask => {
                metrics.unmask_bp =
                    rolling_bp(metrics.unmask_bp, round_mode_bp, metrics.rounds_played);
            }
        }
        if !is_correct || timed_out {
            metrics.weakest_lane = Some(lane.to_string());
        }
        metrics.last_mode = Some(mode.as_str().to_string());
        metrics.confusion_score = clamp_bp(
            10_000 - (((metrics.accuracy_bp as i64 * 3) + (metrics.fluency_bp as i64 * 2)) / 5),
        );
        self.persist_student_contrast_metrics(student_id, pair_id, &metrics)?;
        Ok(metrics.confusion_score)
    }

    fn upsert_student_contrast_reason(
        &self,
        student_id: i64,
        pair_id: i64,
        mode: TrapsMode,
        reason_code: &str,
    ) -> EcoachResult<()> {
        let mut metrics = self.load_student_contrast_metrics(student_id, pair_id)?;
        *metrics
            .confusion_reason_profile
            .entry(reason_code.to_string())
            .or_insert(0) += 1;
        metrics.last_mode = Some(mode.as_str().to_string());
        self.persist_student_contrast_metrics(student_id, pair_id, &metrics)
    }

    fn persist_student_contrast_metrics(
        &self,
        student_id: i64,
        pair_id: i64,
        metrics: &StudentContrastMetrics,
    ) -> EcoachResult<()> {
        let confusion_reason_profile_json =
            serde_json::to_string(&metrics.confusion_reason_profile)
                .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO student_contrast_states (
                    student_id, pair_id, accuracy_bp, fluency_bp, confusion_score,
                    difference_drill_bp, similarity_trap_bp, know_difference_bp,
                    which_is_which_bp, unmask_bp, rounds_played, rounds_correct,
                    timed_out_count, average_response_time_ms, weakest_lane, last_mode,
                    confusion_reason_profile_json, last_practiced_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?18)
                 ON CONFLICT(student_id, pair_id) DO UPDATE SET
                    accuracy_bp = excluded.accuracy_bp,
                    fluency_bp = excluded.fluency_bp,
                    confusion_score = excluded.confusion_score,
                    difference_drill_bp = excluded.difference_drill_bp,
                    similarity_trap_bp = excluded.similarity_trap_bp,
                    know_difference_bp = excluded.know_difference_bp,
                    which_is_which_bp = excluded.which_is_which_bp,
                    unmask_bp = excluded.unmask_bp,
                    rounds_played = excluded.rounds_played,
                    rounds_correct = excluded.rounds_correct,
                    timed_out_count = excluded.timed_out_count,
                    average_response_time_ms = excluded.average_response_time_ms,
                    weakest_lane = excluded.weakest_lane,
                    last_mode = excluded.last_mode,
                    confusion_reason_profile_json = excluded.confusion_reason_profile_json,
                    last_practiced_at = excluded.last_practiced_at,
                    updated_at = excluded.updated_at",
                params![
                    student_id,
                    pair_id,
                    metrics.accuracy_bp,
                    metrics.fluency_bp,
                    metrics.confusion_score,
                    metrics.difference_drill_bp,
                    metrics.similarity_trap_bp,
                    metrics.know_difference_bp,
                    metrics.which_is_which_bp,
                    metrics.unmask_bp,
                    metrics.rounds_played,
                    metrics.rounds_correct,
                    metrics.timed_out_count,
                    metrics.average_response_time_ms,
                    metrics.weakest_lane,
                    metrics.last_mode,
                    confusion_reason_profile_json,
                    Utc::now().to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn compute_game_effect(
        &self,
        game_type: GameType,
        game_session_id: i64,
        is_correct: bool,
        misconception_triggered: bool,
    ) -> EcoachResult<String> {
        match game_type {
            GameType::Mindstack => {
                self.advance_mindstack(game_session_id, is_correct, misconception_triggered)
            }
            GameType::TugOfWar => self.advance_tug_of_war(game_session_id, is_correct),
            GameType::Traps => self.advance_traps(game_session_id, is_correct),
        }
    }

    fn advance_mindstack(
        &self,
        game_session_id: i64,
        is_correct: bool,
        misconception_triggered: bool,
    ) -> EcoachResult<String> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let mut val: Value = serde_json::from_str(&metadata_json)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        let mut height = val["board_height"].as_i64().unwrap_or(0);
        let mut cleared = val["cleared_rows"].as_i64().unwrap_or(0);

        let effect = if is_correct {
            cleared += MINDSTACK_CORRECT_CLEAR;
            height = (height - 1).max(0);
            if misconception_triggered {
                "clear_with_warning"
            } else {
                "clear_row"
            }
        } else {
            height += MINDSTACK_INCORRECT_STACK;
            if height >= MINDSTACK_MAX_HEIGHT {
                self.conn
                    .execute(
                        "UPDATE game_sessions SET session_state = 'completed', completed_at = datetime('now') WHERE id = ?1",
                        [game_session_id],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
                "stack_overflow_game_over"
            } else {
                "stack_block"
            }
        };

        val["board_height"] = json!(height);
        val["cleared_rows"] = json!(cleared);
        val["pending_block_type"] = json!(if cleared % 5 == 0 && cleared > 0 {
            "bonus"
        } else {
            "standard"
        });

        self.conn
            .execute(
                "UPDATE game_sessions SET metadata_json = ?1 WHERE id = ?2",
                params![
                    serde_json::to_string(&val)
                        .map_err(|e| EcoachError::Serialization(e.to_string()))?,
                    game_session_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(effect.to_string())
    }

    fn advance_tug_of_war(&self, game_session_id: i64, is_correct: bool) -> EcoachResult<String> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let mut val: Value = serde_json::from_str(&metadata_json)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        let mut position = val["position"].as_i64().unwrap_or(0);
        let opponent_diff = val["opponent_difficulty"].as_i64().unwrap_or(5000);

        let effect = if is_correct {
            position = (position + TUG_CORRECT_MOVE).min(TUG_WIN_POSITION);
            if position >= TUG_WIN_POSITION {
                self.conn
                    .execute(
                        "UPDATE game_sessions SET session_state = 'completed', completed_at = datetime('now') WHERE id = ?1",
                        [game_session_id],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
                "tug_win"
            } else {
                "tug_pull_forward"
            }
        } else {
            position = (position + TUG_INCORRECT_MOVE).max(TUG_LOSE_POSITION);
            if position <= TUG_LOSE_POSITION {
                self.conn
                    .execute(
                        "UPDATE game_sessions SET session_state = 'completed', completed_at = datetime('now') WHERE id = ?1",
                        [game_session_id],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
                "tug_loss"
            } else {
                "tug_pull_back"
            }
        };

        let new_diff = if position > 3 {
            (opponent_diff + 300).min(9500)
        } else if position < -3 {
            (opponent_diff - 200).max(2000)
        } else {
            opponent_diff
        };
        val["position"] = json!(position);
        val["opponent_difficulty"] = json!(new_diff);

        self.conn
            .execute(
                "UPDATE game_sessions SET metadata_json = ?1 WHERE id = ?2",
                params![
                    serde_json::to_string(&val)
                        .map_err(|e| EcoachError::Serialization(e.to_string()))?,
                    game_session_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(effect.to_string())
    }

    fn advance_traps(&self, game_session_id: i64, is_correct: bool) -> EcoachResult<String> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let mut val: Value = serde_json::from_str(&metadata_json)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        let mut correct_disc = val["correct_discriminations"].as_i64().unwrap_or(0);
        let mut total_disc = val["total_discriminations"].as_i64().unwrap_or(0);
        total_disc += 1;
        let effect = if is_correct {
            correct_disc += 1;
            "trap_avoided"
        } else {
            "trap_triggered"
        };
        val["correct_discriminations"] = json!(correct_disc);
        val["total_discriminations"] = json!(total_disc);
        self.conn
            .execute(
                "UPDATE game_sessions SET metadata_json = ?1 WHERE id = ?2",
                params![
                    serde_json::to_string(&val)
                        .map_err(|e| EcoachError::Serialization(e.to_string()))?,
                    game_session_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(effect.to_string())
    }

    fn append_event(&self, aggregate_kind: &str, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    event.event_id,
                    event.event_type,
                    aggregate_kind,
                    event.aggregate_id,
                    event.trace_id,
                    payload_json,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }
}
