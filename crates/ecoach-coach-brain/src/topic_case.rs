use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCase {
    pub student_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub subject_code: String,
    pub priority_score: BasisPoints,
    pub mastery_score: BasisPoints,
    pub mastery_state: String,
    pub gap_score: BasisPoints,
    pub fragility_score: BasisPoints,
    pub pressure_collapse_index: BasisPoints,
    pub memory_state: String,
    pub memory_strength: BasisPoints,
    pub decay_risk: BasisPoints,
    pub evidence_count: i64,
    pub recent_attempt_count: i64,
    pub recent_accuracy: Option<BasisPoints>,
    pub active_blocker: Option<TopicCaseBlocker>,
    pub recent_diagnoses: Vec<TopicCaseDiagnosis>,
    pub active_hypotheses: Vec<TopicCaseHypothesis>,
    pub primary_hypothesis_code: String,
    pub diagnosis_certainty: BasisPoints,
    pub requires_probe: bool,
    pub recommended_intervention: TopicCaseIntervention,
    pub proof_gaps: Vec<String>,
    pub open_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCaseBlocker {
    pub reason: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCaseDiagnosis {
    pub diagnosis_id: i64,
    pub error_type: String,
    pub primary_diagnosis: String,
    pub severity: String,
    pub diagnosis_summary: String,
    pub recommended_action: String,
    pub confidence_score: BasisPoints,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCaseHypothesis {
    pub code: String,
    pub label: String,
    pub confidence_score: BasisPoints,
    pub evidence_summary: String,
    pub recommended_probe: Option<String>,
    pub recommended_response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCaseIntervention {
    pub mode: String,
    pub urgency: String,
    pub next_action_type: String,
    pub recommended_minutes: i64,
    pub reason: String,
}

pub fn build_topic_case(
    conn: &Connection,
    student_id: i64,
    topic_id: i64,
) -> EcoachResult<TopicCase> {
    let base = load_base_topic_state(conn, student_id, topic_id)?;
    let blocker = load_active_blocker(conn, student_id, topic_id)?;
    let memory = load_memory_snapshot(conn, student_id, topic_id)?;
    let error_profile = load_error_profile(conn, student_id, topic_id)?;
    let recent_diagnoses = load_recent_diagnoses(conn, student_id, topic_id, 4)?;
    let coach_evidence = load_recent_coach_evidence(conn, student_id, topic_id)?;

    let mut hypotheses = build_hypotheses(
        &base,
        blocker.as_ref(),
        memory.as_ref(),
        error_profile.as_ref(),
        &recent_diagnoses,
        &coach_evidence,
    );
    hypotheses.sort_by(|left, right| {
        right
            .confidence_score
            .cmp(&left.confidence_score)
            .then_with(|| left.code.cmp(&right.code))
    });
    if hypotheses.len() > 3 {
        hypotheses.truncate(3);
    }
    if hypotheses.is_empty() {
        hypotheses.push(TopicCaseHypothesis {
            code: "stabilize_and_probe".to_string(),
            label: "Need more evidence".to_string(),
            confidence_score: 4200,
            evidence_summary: "Current topic signals are too thin for a confident root-cause call."
                .to_string(),
            recommended_probe: Some(
                "Run a short mixed checkpoint with one recall item and one timed item.".to_string(),
            ),
            recommended_response: "Collect more evidence before escalating the coaching strategy."
                .to_string(),
        });
    }

    let diagnosis_certainty =
        compute_diagnosis_certainty(&hypotheses, coach_evidence.recent_attempt_count);
    let proof_gaps = build_proof_gaps(
        &base,
        blocker.as_ref(),
        memory.as_ref(),
        &recent_diagnoses,
        &coach_evidence,
        &hypotheses,
    );
    let open_questions = build_open_questions(
        &base,
        memory.as_ref(),
        error_profile.as_ref(),
        &coach_evidence,
        &hypotheses,
    );
    let requires_probe = diagnosis_certainty < 6500 || !open_questions.is_empty();
    let intervention = build_recommended_intervention(
        &hypotheses,
        blocker.as_ref(),
        &base,
        memory.as_ref(),
        &coach_evidence,
    );
    let priority_score = compute_case_priority(&base, blocker.as_ref(), &hypotheses);
    let inferred_memory_state = memory
        .as_ref()
        .map(|item| item.memory_state.clone())
        .unwrap_or_else(|| infer_memory_state(&base));
    let primary_hypothesis_code = hypotheses
        .first()
        .map(|item| item.code.clone())
        .unwrap_or_else(|| "stabilize_and_probe".to_string());

    Ok(TopicCase {
        student_id,
        topic_id: base.topic_id,
        topic_name: base.topic_name,
        subject_code: base.subject_code,
        priority_score,
        mastery_score: clamp_bp(base.mastery_score),
        mastery_state: base.mastery_state,
        gap_score: clamp_bp(base.gap_score),
        fragility_score: clamp_bp(base.fragility_score),
        pressure_collapse_index: clamp_bp(base.pressure_collapse_index),
        memory_state: inferred_memory_state,
        memory_strength: clamp_bp(
            memory
                .as_ref()
                .map(|item| item.memory_strength)
                .unwrap_or(base.memory_strength),
        ),
        decay_risk: clamp_bp(
            memory
                .as_ref()
                .map(|item| item.decay_risk)
                .unwrap_or(base.decay_risk),
        ),
        evidence_count: base.evidence_count,
        recent_attempt_count: coach_evidence.recent_attempt_count,
        recent_accuracy: coach_evidence.recent_accuracy.map(clamp_bp),
        active_blocker: blocker,
        recent_diagnoses,
        active_hypotheses: hypotheses,
        primary_hypothesis_code,
        diagnosis_certainty,
        requires_probe,
        recommended_intervention: intervention,
        proof_gaps,
        open_questions,
    })
}

pub fn list_priority_topic_cases(
    conn: &Connection,
    student_id: i64,
    limit: usize,
) -> EcoachResult<Vec<TopicCase>> {
    let mut statement = conn
        .prepare(
            "SELECT topic_id
             FROM student_topic_states
             WHERE student_id = ?1
             ORDER BY priority_score DESC, repair_priority DESC, gap_score DESC, topic_id ASC
             LIMIT ?2",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map(params![student_id, limit as i64], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut cases = Vec::new();
    for row in rows {
        let topic_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
        cases.push(build_topic_case(conn, student_id, topic_id)?);
    }

    cases.sort_by(|left, right| {
        right
            .priority_score
            .cmp(&left.priority_score)
            .then_with(|| right.diagnosis_certainty.cmp(&left.diagnosis_certainty))
            .then_with(|| left.topic_id.cmp(&right.topic_id))
    });
    Ok(cases)
}

#[derive(Debug)]
struct BaseTopicState {
    topic_id: i64,
    topic_name: String,
    subject_code: String,
    mastery_score: i64,
    mastery_state: String,
    gap_score: i64,
    priority_score: i64,
    fragility_score: i64,
    pressure_collapse_index: i64,
    decay_risk: i64,
    memory_strength: i64,
    evidence_count: i64,
    repair_priority: i64,
    coach_blocked_status: bool,
    misconception_recurrence: i64,
}

#[derive(Debug)]
struct MemorySnapshot {
    memory_state: String,
    memory_strength: i64,
    decay_risk: i64,
    review_due_at: Option<String>,
}

#[derive(Debug)]
struct ErrorProfile {
    knowledge_gap_score: i64,
    conceptual_confusion_score: i64,
    execution_error_score: i64,
    carelessness_score: i64,
    pressure_breakdown_score: i64,
    expression_weakness_score: i64,
    speed_error_score: i64,
}

#[derive(Debug)]
struct CoachEvidenceSummary {
    recent_attempt_count: i64,
    recent_accuracy: Option<i64>,
    recent_timed_accuracy: Option<i64>,
    recent_avg_latency_ms: Option<i64>,
}

fn load_base_topic_state(
    conn: &Connection,
    student_id: i64,
    topic_id: i64,
) -> EcoachResult<BaseTopicState> {
    conn.query_row(
        "SELECT sts.topic_id, t.name, s.code, sts.mastery_score, sts.mastery_state,
                sts.gap_score, sts.priority_score, sts.fragility_score,
                sts.pressure_collapse_index, sts.decay_risk, sts.memory_strength,
                sts.evidence_count, sts.repair_priority,
                COALESCE(ctp.blocked_status, 0), COALESCE(ctp.misconception_recurrence, 0)
         FROM student_topic_states sts
         INNER JOIN topics t ON t.id = sts.topic_id
         INNER JOIN subjects s ON s.id = t.subject_id
         LEFT JOIN coach_topic_profiles ctp
            ON ctp.student_id = sts.student_id AND ctp.topic_id = sts.topic_id
         WHERE sts.student_id = ?1 AND sts.topic_id = ?2",
        params![student_id, topic_id],
        |row| {
            Ok(BaseTopicState {
                topic_id: row.get(0)?,
                topic_name: row.get(1)?,
                subject_code: row.get(2)?,
                mastery_score: row.get(3)?,
                mastery_state: row.get(4)?,
                gap_score: row.get(5)?,
                priority_score: row.get(6)?,
                fragility_score: row.get(7)?,
                pressure_collapse_index: row.get(8)?,
                decay_risk: row.get(9)?,
                memory_strength: row.get(10)?,
                evidence_count: row.get(11)?,
                repair_priority: row.get(12)?,
                coach_blocked_status: row.get::<_, i64>(13)? == 1,
                misconception_recurrence: row.get(14)?,
            })
        },
    )
    .map_err(|err| {
        if matches!(err, rusqlite::Error::QueryReturnedNoRows) {
            EcoachError::NotFound(format!(
                "student topic state not found for student {} topic {}",
                student_id, topic_id
            ))
        } else {
            EcoachError::Storage(err.to_string())
        }
    })
}

fn load_active_blocker(
    conn: &Connection,
    student_id: i64,
    topic_id: i64,
) -> EcoachResult<Option<TopicCaseBlocker>> {
    conn.query_row(
        "SELECT reason, severity
         FROM coach_blockers
         WHERE student_id = ?1 AND topic_id = ?2 AND resolved_at IS NULL
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
        params![student_id, topic_id],
        |row| {
            Ok(TopicCaseBlocker {
                reason: row.get(0)?,
                severity: row.get(1)?,
            })
        },
    )
    .optional()
    .map_err(|err| EcoachError::Storage(err.to_string()))
}

fn load_memory_snapshot(
    conn: &Connection,
    student_id: i64,
    topic_id: i64,
) -> EcoachResult<Option<MemorySnapshot>> {
    conn.query_row(
        "SELECT memory_state, memory_strength, decay_risk, review_due_at
         FROM memory_states
         WHERE student_id = ?1 AND topic_id = ?2
         ORDER BY updated_at DESC, id DESC
         LIMIT 1",
        params![student_id, topic_id],
        |row| {
            Ok(MemorySnapshot {
                memory_state: row.get(0)?,
                memory_strength: row.get(1)?,
                decay_risk: row.get(2)?,
                review_due_at: row.get(3)?,
            })
        },
    )
    .optional()
    .map_err(|err| EcoachError::Storage(err.to_string()))
}

fn load_error_profile(
    conn: &Connection,
    student_id: i64,
    topic_id: i64,
) -> EcoachResult<Option<ErrorProfile>> {
    conn.query_row(
        "SELECT knowledge_gap_score, conceptual_confusion_score, execution_error_score,
                carelessness_score, pressure_breakdown_score, expression_weakness_score,
                speed_error_score
         FROM student_error_profiles
         WHERE student_id = ?1 AND topic_id = ?2",
        params![student_id, topic_id],
        |row| {
            Ok(ErrorProfile {
                knowledge_gap_score: row.get(0)?,
                conceptual_confusion_score: row.get(1)?,
                execution_error_score: row.get(2)?,
                carelessness_score: row.get(3)?,
                pressure_breakdown_score: row.get(4)?,
                expression_weakness_score: row.get(5)?,
                speed_error_score: row.get(6)?,
            })
        },
    )
    .optional()
    .map_err(|err| EcoachError::Storage(err.to_string()))
}

fn load_recent_diagnoses(
    conn: &Connection,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> EcoachResult<Vec<TopicCaseDiagnosis>> {
    let mut statement = conn
        .prepare(
            "SELECT id, error_type, primary_diagnosis, severity, diagnosis_summary,
                    recommended_action, confidence_score, created_at
             FROM wrong_answer_diagnoses
             WHERE student_id = ?1 AND topic_id = ?2
             ORDER BY created_at DESC, id DESC
             LIMIT ?3",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map(params![student_id, topic_id, limit as i64], |row| {
            Ok(TopicCaseDiagnosis {
                diagnosis_id: row.get(0)?,
                error_type: row.get(1)?,
                primary_diagnosis: row.get(2)?,
                severity: row.get(3)?,
                diagnosis_summary: row.get(4)?,
                recommended_action: row.get(5)?,
                confidence_score: clamp_bp(row.get::<_, i64>(6)?),
                created_at: row.get(7)?,
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut diagnoses = Vec::new();
    for row in rows {
        diagnoses.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(diagnoses)
}

fn load_recent_coach_evidence(
    conn: &Connection,
    student_id: i64,
    topic_id: i64,
) -> EcoachResult<CoachEvidenceSummary> {
    conn.query_row(
        "SELECT COALESCE(SUM(attempt_count), 0),
                CAST(ROUND(AVG(accuracy)) AS INTEGER),
                CAST(ROUND(AVG(timed_accuracy)) AS INTEGER),
                CAST(ROUND(AVG(avg_latency_ms)) AS INTEGER)
         FROM (
            SELECT attempt_count, accuracy, timed_accuracy, avg_latency_ms
            FROM coach_session_evidence
            WHERE student_id = ?1 AND topic_id = ?2
            ORDER BY completed_at DESC, id DESC
            LIMIT 5
         ) recent_evidence",
        params![student_id, topic_id],
        |row| {
            Ok(CoachEvidenceSummary {
                recent_attempt_count: row.get(0)?,
                recent_accuracy: row.get(1)?,
                recent_timed_accuracy: row.get(2)?,
                recent_avg_latency_ms: row.get(3)?,
            })
        },
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))
}

fn build_hypotheses(
    base: &BaseTopicState,
    blocker: Option<&TopicCaseBlocker>,
    memory: Option<&MemorySnapshot>,
    error_profile: Option<&ErrorProfile>,
    recent_diagnoses: &[TopicCaseDiagnosis],
    coach_evidence: &CoachEvidenceSummary,
) -> Vec<TopicCaseHypothesis> {
    let mut hypotheses = Vec::new();

    let blocked_score = blocked_topic_score(base, blocker);
    if blocked_score >= 5000 {
        hypotheses.push(TopicCaseHypothesis {
            code: "blocked_topic".to_string(),
            label: "Blocked topic".to_string(),
            confidence_score: clamp_bp(blocked_score),
            evidence_summary: blocker
                .map(|item| format!("Active blocker: {} ({})", item.reason, item.severity))
                .unwrap_or_else(|| {
                    "Coach topic profile still marks this topic as blocked.".to_string()
                }),
            recommended_probe: Some(
                "Run a short repair checkpoint before allowing forward progression.".to_string(),
            ),
            recommended_response:
                "Pause forward progression and route the learner into targeted repair.".to_string(),
        });
    }

    let confusion_score =
        conceptual_confusion_score(base, error_profile, recent_diagnoses, coach_evidence);
    if confusion_score >= 4500 {
        hypotheses.push(TopicCaseHypothesis {
            code: "conceptual_confusion".to_string(),
            label: "Conceptual confusion".to_string(),
            confidence_score: clamp_bp(confusion_score),
            evidence_summary: format!(
                "Confusion signals stay elevated through misconception recurrence and recent diagnoses on {}.",
                base.topic_name
            ),
            recommended_probe: Some(
                "Use a contrast checkpoint against the nearest confusing neighbor concept."
                    .to_string(),
            ),
            recommended_response:
                "Run contrast repair with explicit why-this-not-that reasoning.".to_string(),
        });
    }

    let memory_score = memory_decay_score(base, memory, coach_evidence);
    if memory_score >= 4500 {
        hypotheses.push(TopicCaseHypothesis {
            code: "memory_decay".to_string(),
            label: "Memory decay".to_string(),
            confidence_score: clamp_bp(memory_score),
            evidence_summary:
                "Recall strength is fragile and the topic needs delayed retrieval proof."
                    .to_string(),
            recommended_probe: Some(
                "Schedule a short retrieval check after a delay instead of immediate reteach."
                    .to_string(),
            ),
            recommended_response:
                "Use retrieval reactivation before escalating content difficulty.".to_string(),
        });
    }

    let pressure_score = pressure_collapse_score(base, error_profile, coach_evidence);
    if pressure_score >= 4500 {
        hypotheses.push(TopicCaseHypothesis {
            code: "pressure_collapse".to_string(),
            label: "Pressure collapse".to_string(),
            confidence_score: clamp_bp(pressure_score),
            evidence_summary:
                "The learner looks weaker under load than their calmer topic evidence suggests."
                    .to_string(),
            recommended_probe: Some(
                "Run a short timed burst after one calm success to isolate pressure effects."
                    .to_string(),
            ),
            recommended_response:
                "Shift into pressure conditioning instead of reteaching the whole topic."
                    .to_string(),
        });
    }

    let knowledge_gap_score = foundational_gap_score(base, error_profile, coach_evidence);
    if knowledge_gap_score >= 4500 {
        hypotheses.push(TopicCaseHypothesis {
            code: "knowledge_gap".to_string(),
            label: "Foundational gap".to_string(),
            confidence_score: clamp_bp(knowledge_gap_score),
            evidence_summary:
                "Core topic mastery is still too low for stable independent performance."
                    .to_string(),
            recommended_probe: Some(
                "Use one prerequisite check before assigning heavier mixed practice.".to_string(),
            ),
            recommended_response: "Rebuild the topic from its core prerequisite steps.".to_string(),
        });
    }

    let execution_score = execution_drift_score(error_profile, coach_evidence);
    if execution_score >= 4500 {
        hypotheses.push(TopicCaseHypothesis {
            code: "execution_drift".to_string(),
            label: "Execution drift".to_string(),
            confidence_score: clamp_bp(execution_score),
            evidence_summary:
                "The learner may know the idea but is leaking marks through execution drift or carelessness."
                    .to_string(),
            recommended_probe: Some(
                "Run one precision drill with visible step checks and error review.".to_string(),
            ),
            recommended_response:
                "Use precision repair before adding more content volume.".to_string(),
        });
    }

    hypotheses
}

fn build_recommended_intervention(
    hypotheses: &[TopicCaseHypothesis],
    blocker: Option<&TopicCaseBlocker>,
    base: &BaseTopicState,
    memory: Option<&MemorySnapshot>,
    coach_evidence: &CoachEvidenceSummary,
) -> TopicCaseIntervention {
    let primary = hypotheses
        .first()
        .map(|item| item.code.as_str())
        .unwrap_or("stabilize_and_probe");
    let (mode, next_action_type, default_minutes) = match primary {
        "blocked_topic" => ("contrast_repair", "start_repair", 25),
        "conceptual_confusion" => ("contrast_repair", "start_repair", 20),
        "memory_decay" => ("retrieval_reactivation", "start_repair", 15),
        "pressure_collapse" => ("pressure_conditioning", "adjust_plan", 20),
        "knowledge_gap" => ("foundation_rebuild", "start_repair", 25),
        "execution_drift" => ("precision_repair", "start_today_mission", 15),
        _ => ("stabilize_and_probe", "review_results", 12),
    };

    let urgency_score = blocked_topic_score(base, blocker)
        .max(base.priority_score)
        .max(base.repair_priority)
        .max(memory.map(|item| item.decay_risk).unwrap_or(0));
    let urgency = urgency_label(urgency_score);
    let recommended_minutes = if coach_evidence.recent_attempt_count == 0 {
        default_minutes.max(15)
    } else {
        default_minutes
    };

    let reason = match primary {
        "blocked_topic" => blocker
            .map(|item| format!("Repair is required because {}.", item.reason))
            .unwrap_or_else(|| "Repair is required before forward progression.".to_string()),
        "conceptual_confusion" => format!(
            "{} is showing confusion signals and needs contrast-based repair.",
            base.topic_name
        ),
        "memory_decay" => format!(
            "{} is slipping in memory and should be reactivated through retrieval.",
            base.topic_name
        ),
        "pressure_collapse" => format!(
            "{} needs timed hardening instead of more untimed exposure.",
            base.topic_name
        ),
        "knowledge_gap" => format!(
            "{} still needs foundational rebuilding before mixed practice.",
            base.topic_name
        ),
        "execution_drift" => format!(
            "{} needs precision repair to stop avoidable mark loss.",
            base.topic_name
        ),
        _ => format!(
            "Collect one cleaner evidence pass on {} before escalating strategy.",
            base.topic_name
        ),
    };

    TopicCaseIntervention {
        mode: mode.to_string(),
        urgency: urgency.to_string(),
        next_action_type: next_action_type.to_string(),
        recommended_minutes,
        reason,
    }
}

fn build_proof_gaps(
    base: &BaseTopicState,
    blocker: Option<&TopicCaseBlocker>,
    memory: Option<&MemorySnapshot>,
    recent_diagnoses: &[TopicCaseDiagnosis],
    coach_evidence: &CoachEvidenceSummary,
    hypotheses: &[TopicCaseHypothesis],
) -> Vec<String> {
    let mut proof_gaps = Vec::new();

    if base.evidence_count < 3 || coach_evidence.recent_attempt_count < 4 {
        proof_gaps.push("Need more independent topic evidence across sessions.".to_string());
    }
    if memory.is_none() || memory.and_then(|item| item.review_due_at.clone()).is_some() {
        proof_gaps.push("Delayed retrieval proof is still missing or overdue.".to_string());
    }
    if base.pressure_collapse_index >= 6000 && coach_evidence.recent_timed_accuracy.is_none() {
        proof_gaps.push("Timed resilience evidence is still missing.".to_string());
    }
    if hypotheses
        .iter()
        .any(|item| item.code == "conceptual_confusion")
    {
        proof_gaps
            .push("Contrast evidence against confusing neighbors is still needed.".to_string());
    }
    if blocker.is_some() {
        proof_gaps.push(
            "Repair checkpoint must succeed before the coach should unlock forward progression."
                .to_string(),
        );
    }
    if recent_diagnoses.is_empty() && base.gap_score >= 7000 {
        proof_gaps.push(
            "Root-cause probe is still needed because the topic is weak but under-diagnosed."
                .to_string(),
        );
    }

    proof_gaps
}

fn build_open_questions(
    base: &BaseTopicState,
    memory: Option<&MemorySnapshot>,
    error_profile: Option<&ErrorProfile>,
    coach_evidence: &CoachEvidenceSummary,
    hypotheses: &[TopicCaseHypothesis],
) -> Vec<String> {
    let mut open_questions = Vec::new();

    if hypotheses
        .first()
        .map(|item| item.confidence_score)
        .unwrap_or(0)
        < 6500
    {
        open_questions.push("Which cause is primary once fresh evidence is collected?".to_string());
    }
    if memory.is_none() {
        open_questions
            .push("Is the weakness structural or mainly a retrieval decay problem?".to_string());
    }
    if base.pressure_collapse_index >= 6000 && coach_evidence.recent_timed_accuracy.is_none() {
        open_questions.push("How much does timing amplify the weakness on this topic?".to_string());
    }
    if error_profile.is_none() && base.gap_score >= 7000 {
        open_questions.push("Which error family is dominant for this topic right now?".to_string());
    }

    open_questions
}

fn compute_case_priority(
    base: &BaseTopicState,
    blocker: Option<&TopicCaseBlocker>,
    hypotheses: &[TopicCaseHypothesis],
) -> BasisPoints {
    let hypothesis_peak = hypotheses
        .iter()
        .map(|item| item.confidence_score as i64)
        .max()
        .unwrap_or(0);
    let blocker_boost = if blocker.is_some() || base.coach_blocked_status {
        1000
    } else {
        0
    };
    clamp_bp(
        base.priority_score
            .max(base.repair_priority)
            .max(hypothesis_peak + blocker_boost),
    )
}

fn compute_diagnosis_certainty(
    hypotheses: &[TopicCaseHypothesis],
    recent_attempt_count: i64,
) -> BasisPoints {
    let primary = hypotheses
        .first()
        .map(|item| item.confidence_score as i64)
        .unwrap_or(0);
    let secondary = hypotheses
        .get(1)
        .map(|item| item.confidence_score as i64)
        .unwrap_or(0);
    let evidence_bonus = if recent_attempt_count >= 6 {
        800
    } else if recent_attempt_count >= 3 {
        400
    } else {
        0
    };
    clamp_bp((primary - secondary + 5000).clamp(2500, 9200) + evidence_bonus)
}

fn blocked_topic_score(base: &BaseTopicState, blocker: Option<&TopicCaseBlocker>) -> i64 {
    let mut score = 0;
    if blocker.is_some() {
        score = score.max(9500);
    }
    if base.coach_blocked_status {
        score = score.max(9000);
    }
    score.max(base.repair_priority)
}

fn conceptual_confusion_score(
    base: &BaseTopicState,
    error_profile: Option<&ErrorProfile>,
    recent_diagnoses: &[TopicCaseDiagnosis],
    coach_evidence: &CoachEvidenceSummary,
) -> i64 {
    let profile_score = error_profile
        .map(|item| item.conceptual_confusion_score)
        .unwrap_or(0);
    let misconception_score = (base.misconception_recurrence * 1800).clamp(0, 9000);
    let diagnosis_score = diagnosis_keyword_score(
        recent_diagnoses,
        &["confusion", "misconception", "boundary", "pair"],
    );
    let mixed_failure_signal = match (
        coach_evidence.recent_accuracy,
        coach_evidence.recent_timed_accuracy,
    ) {
        (Some(accuracy), Some(timed_accuracy)) if timed_accuracy + 1200 < accuracy => 6200,
        _ => 0,
    };

    profile_score
        .max(misconception_score)
        .max(diagnosis_score)
        .max(mixed_failure_signal)
}

fn memory_decay_score(
    base: &BaseTopicState,
    memory: Option<&MemorySnapshot>,
    coach_evidence: &CoachEvidenceSummary,
) -> i64 {
    let Some(memory) = memory else {
        return if base.decay_risk >= 6500 {
            base.decay_risk
        } else {
            0
        };
    };

    let state_bonus = match memory.memory_state.as_str() {
        "fading" => 7800,
        "collapsed" => 9000,
        "at_risk" => 7200,
        "rebuilding" => 6500,
        _ => 0,
    };
    let no_recent_evidence_penalty = if coach_evidence.recent_attempt_count == 0 {
        6000
    } else {
        0
    };

    memory
        .decay_risk
        .max(10_000 - memory.memory_strength)
        .max(base.decay_risk)
        .max(state_bonus)
        .max(no_recent_evidence_penalty)
}

fn pressure_collapse_score(
    base: &BaseTopicState,
    error_profile: Option<&ErrorProfile>,
    coach_evidence: &CoachEvidenceSummary,
) -> i64 {
    let profile_score = error_profile
        .map(|item| item.pressure_breakdown_score)
        .unwrap_or(0);
    let latency_signal = match (
        coach_evidence.recent_accuracy,
        coach_evidence.recent_avg_latency_ms,
    ) {
        (Some(accuracy), Some(avg_latency_ms)) if accuracy < 6000 && avg_latency_ms > 18_000 => {
            6500
        }
        _ => 0,
    };

    base.pressure_collapse_index
        .max(profile_score)
        .max(latency_signal)
}

fn foundational_gap_score(
    base: &BaseTopicState,
    error_profile: Option<&ErrorProfile>,
    coach_evidence: &CoachEvidenceSummary,
) -> i64 {
    let profile_score = error_profile
        .map(|item| item.knowledge_gap_score)
        .unwrap_or(0);
    let evidence_penalty = if coach_evidence.recent_attempt_count <= 1 {
        5800
    } else {
        0
    };

    base.gap_score
        .max(10_000 - base.mastery_score)
        .max(profile_score)
        .max(evidence_penalty)
}

fn execution_drift_score(
    error_profile: Option<&ErrorProfile>,
    coach_evidence: &CoachEvidenceSummary,
) -> i64 {
    let Some(error_profile) = error_profile else {
        return 0;
    };
    let timed_drop_signal = match (
        coach_evidence.recent_accuracy,
        coach_evidence.recent_timed_accuracy,
    ) {
        (Some(accuracy), Some(timed_accuracy)) if accuracy >= 6000 && timed_accuracy < 5000 => 6200,
        _ => 0,
    };

    error_profile
        .execution_error_score
        .max(error_profile.carelessness_score)
        .max(error_profile.expression_weakness_score)
        .max(error_profile.speed_error_score)
        .max(timed_drop_signal)
}

fn diagnosis_keyword_score(diagnoses: &[TopicCaseDiagnosis], keywords: &[&str]) -> i64 {
    diagnoses
        .iter()
        .map(|item| {
            let haystack = format!(
                "{} {} {}",
                item.primary_diagnosis, item.error_type, item.diagnosis_summary
            )
            .to_lowercase();
            if keywords.iter().any(|keyword| haystack.contains(keyword)) {
                item.confidence_score as i64
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0)
}

fn infer_memory_state(base: &BaseTopicState) -> String {
    if base.decay_risk >= 7500 {
        "fading".to_string()
    } else if base.memory_strength <= 3500 {
        "fragile".to_string()
    } else {
        "accessible".to_string()
    }
}

fn urgency_label(score: i64) -> &'static str {
    if score >= 8500 {
        "critical"
    } else if score >= 6500 {
        "high"
    } else if score >= 4500 {
        "medium"
    } else {
        "watch"
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn topic_case_builds_hypotheses_and_repair_intervention() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        insert_topic_state(
            &conn, student_id, topic_id, 3200, "fragile", 9100, 9300, 7600, 6800, 7600, 2800, 2,
            9200,
        );
        conn.execute(
            "INSERT INTO coach_topic_profiles (
                student_id, topic_id, mastery_estimate, fragility_score, speed_score,
                misconception_recurrence, evidence_count, attempt_count, blocked_status,
                repair_priority
             ) VALUES (?1, ?2, 3200, 7600, 4200, 4, 2, 5, 1, 9400)",
            params![student_id, topic_id],
        )
        .expect("coach topic profile should insert");
        conn.execute(
            "INSERT INTO student_error_profiles (
                student_id, topic_id, knowledge_gap_score, conceptual_confusion_score,
                execution_error_score, carelessness_score, pressure_breakdown_score,
                expression_weakness_score, speed_error_score
             ) VALUES (?1, ?2, 6200, 8400, 4100, 2800, 6900, 1900, 2200)",
            params![student_id, topic_id],
        )
        .expect("error profile should insert");
        conn.execute(
            "INSERT INTO memory_states (
                student_id, topic_id, memory_state, memory_strength, decay_risk, review_due_at
             ) VALUES (?1, ?2, 'fading', 2500, 8300, date('now'))",
            params![student_id, topic_id],
        )
        .expect("memory state should insert");
        conn.execute(
            "INSERT INTO coach_blockers (student_id, topic_id, reason, severity)
             VALUES (?1, ?2, 'repeated low mission accuracy', 'high')",
            params![student_id, topic_id],
        )
        .expect("blocker should insert");
        conn.execute(
            "INSERT INTO wrong_answer_diagnoses (
                student_id, question_id, topic_id, error_type, primary_diagnosis, severity,
                diagnosis_summary, recommended_action, confidence_score
             ) VALUES (
                ?1,
                (SELECT id FROM questions ORDER BY id ASC LIMIT 1),
                ?2,
                'misconception_triggered',
                'confusion_pair',
                'high',
                'Student keeps swapping closely related procedures.',
                'run contrast repair',
                8400
             )",
            params![student_id, topic_id],
        )
        .expect("diagnosis should insert");
        conn.execute(
            "INSERT INTO coach_session_evidence (
                student_id, topic_id, activity_type, attempt_count, correct_count, accuracy,
                timed_accuracy, avg_latency_ms, misconception_tags
             ) VALUES (?1, ?2, 'repair', 4, 1, 2500, 2100, 22000, '[\"sign confusion\"]')",
            params![student_id, topic_id],
        )
        .expect("coach evidence should insert");

        let case = build_topic_case(&conn, student_id, topic_id).expect("topic case should build");

        assert_eq!(case.primary_hypothesis_code, "blocked_topic");
        assert_eq!(case.recommended_intervention.mode, "contrast_repair");
        assert_eq!(
            case.recommended_intervention.next_action_type,
            "start_repair"
        );
        assert!(
            case.active_hypotheses
                .iter()
                .any(|item| item.code == "conceptual_confusion")
        );
        assert!(
            case.proof_gaps
                .iter()
                .any(|item| item.contains("Delayed retrieval"))
        );
        assert!(case.diagnosis_certainty >= 6500);
    }

    #[test]
    fn priority_topic_cases_sort_highest_risk_first() {
        let conn = open_test_database();
        let student_id = insert_student(&conn);
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        let subject_id: i64 = conn
            .query_row(
                "SELECT subject_id FROM topics WHERE id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("subject id should resolve");
        conn.execute(
            "INSERT INTO topics (subject_id, code, name, node_type, display_order, exam_weight, importance_weight)
             VALUES (?1, 'MATH-RISK-COMPARE', 'Risk Compare Topic', 'topic', 999, 4000, 4000)",
            [subject_id],
        )
        .expect("second topic should insert");
        let second_topic_id = conn.last_insert_rowid();

        insert_topic_state(
            &conn, student_id, topic_id, 3000, "fragile", 9200, 9500, 7600, 7200, 7600, 2600, 2,
            9400,
        );
        insert_topic_state(
            &conn,
            student_id,
            second_topic_id,
            7200,
            "stable",
            2800,
            3400,
            2000,
            1800,
            2200,
            7000,
            6,
            3000,
        );
        conn.execute(
            "INSERT INTO coach_blockers (student_id, topic_id, reason, severity)
             VALUES (?1, ?2, 'forward progress is unsafe', 'high')",
            params![student_id, topic_id],
        )
        .expect("blocker should insert");

        let cases =
            list_priority_topic_cases(&conn, student_id, 5).expect("cases should load cleanly");

        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].topic_id, topic_id);
        assert!(cases[0].priority_score > cases[1].priority_score);
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
    }

    fn insert_student(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('student', 'Ada', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        conn.last_insert_rowid()
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_topic_state(
        conn: &Connection,
        student_id: i64,
        topic_id: i64,
        mastery_score: i64,
        mastery_state: &str,
        gap_score: i64,
        priority_score: i64,
        fragility_score: i64,
        pressure_collapse_index: i64,
        decay_risk: i64,
        memory_strength: i64,
        evidence_count: i64,
        repair_priority: i64,
    ) {
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, gap_score, priority_score,
                fragility_score, pressure_collapse_index, decay_risk, memory_strength,
                evidence_count, repair_priority
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                student_id,
                topic_id,
                mastery_score,
                mastery_state,
                gap_score,
                priority_score,
                fragility_score,
                pressure_collapse_index,
                decay_risk,
                memory_strength,
                evidence_count,
                repair_priority,
            ],
        )
        .expect("topic state should insert");
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
