use chrono::Utc;
use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    GameAnswerResult, GameLeaderboardEntry, GameSession, GameSummary, GameType, MindstackState,
    StartGameInput, SubmitGameAnswerInput, TrapsState, TugOfWarState,
};

// ── Scoring constants ──

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

pub struct GamesService<'a> {
    conn: &'a Connection,
}

impl<'a> GamesService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // ── Session lifecycle ──

    pub fn start_game_session(&self, input: &StartGameInput) -> EcoachResult<GameSession> {
        let game_type_str = input.game_type.as_str();
        let topic_ids_json = serde_json::to_string(&input.topic_ids)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        let now = Utc::now().to_rfc3339();
        let question_count = input.question_count.max(5) as i64;

        // Build initial game-specific metadata
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

        // Check correctness from the question's options table
        let (is_correct, misconception_id): (bool, Option<i64>) = self
            .conn
            .query_row(
                "SELECT qo.is_correct, qo.misconception_id
                 FROM question_options qo
                 WHERE qo.id = ?1 AND qo.question_id = ?2",
                params![input.selected_option_id, input.question_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)? == 1,
                        row.get::<_, Option<i64>>(1)?,
                    ))
                },
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let misconception_triggered = misconception_id.is_some() && !is_correct;
        let game_type = GameType::from_str(&session.game_type).ok_or_else(|| {
            EcoachError::Validation(format!("unknown game type: {}", session.game_type))
        })?;

        // Compute points
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

        // Compute game-specific effect
        let effect_type = self.compute_game_effect(
            game_type,
            input.game_session_id,
            is_correct,
            misconception_triggered,
        )?;

        // Record the answer event
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

        // Update session state
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
                params![
                    new_score,
                    round_number,
                    streak,
                    best_streak,
                    new_state,
                    input.game_session_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Fetch explanation for the question
        let explanation: Option<String> = self
            .conn
            .query_row(
                "SELECT explanation FROM questions WHERE id = ?1",
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

    // ── Queries ──

    pub fn get_session(&self, game_session_id: i64) -> EcoachResult<GameSession> {
        self.conn
            .query_row(
                "SELECT id, student_id, game_type, subject_id, session_state, score,
                        rounds_total, rounds_played, streak, best_streak, created_at, completed_at
                 FROM game_sessions WHERE id = ?1",
                [game_session_id],
                |row| {
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
                },
            )
            .map_err(|e| EcoachError::NotFound(format!("game session {} not found: {}", game_session_id, e)))
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
                "SELECT COUNT(*), SUM(was_correct), COALESCE(AVG(response_time_ms), 0),
                        SUM(misconception_triggered)
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
            .query_map(params![student_id, limit as i64], |row| {
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
            })
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

    // ── Game-specific state queries ──

    pub fn get_mindstack_state(&self, game_session_id: i64) -> EcoachResult<MindstackState> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1 AND game_type = 'mindstack'",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::NotFound(format!("mindstack session not found: {}", e)))?;
        let val: serde_json::Value = serde_json::from_str(&metadata_json)
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
        let val: serde_json::Value = serde_json::from_str(&metadata_json)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        Ok(TugOfWarState {
            position: val["position"].as_i64().unwrap_or(0),
            opponent_difficulty: clamp_bp(val["opponent_difficulty"].as_i64().unwrap_or(5000)),
        })
    }

    pub fn get_traps_state(&self, game_session_id: i64) -> EcoachResult<TrapsState> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1 AND game_type = 'traps'",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::NotFound(format!("traps session not found: {}", e)))?;
        let val: serde_json::Value = serde_json::from_str(&metadata_json)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        Ok(TrapsState {
            pair_id: val["pair_id"].as_i64().unwrap_or(0),
            correct_discriminations: val["correct_discriminations"].as_i64().unwrap_or(0),
            total_discriminations: val["total_discriminations"].as_i64().unwrap_or(0),
        })
    }

    // ── Internal helpers ──

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

        let mut val: serde_json::Value = serde_json::from_str(&metadata_json)
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
                // Force-complete the session (game over)
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
        let block_type = if cleared % 5 == 0 && cleared > 0 {
            "bonus"
        } else {
            "standard"
        };
        val["pending_block_type"] = json!(block_type);

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

    fn advance_tug_of_war(
        &self,
        game_session_id: i64,
        is_correct: bool,
    ) -> EcoachResult<String> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut val: serde_json::Value = serde_json::from_str(&metadata_json)
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

        // Adaptive opponent difficulty: increase if student is winning
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

    fn advance_traps(
        &self,
        game_session_id: i64,
        is_correct: bool,
    ) -> EcoachResult<String> {
        let metadata_json: String = self
            .conn
            .query_row(
                "SELECT metadata_json FROM game_sessions WHERE id = ?1",
                [game_session_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut val: serde_json::Value = serde_json::from_str(&metadata_json)
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
