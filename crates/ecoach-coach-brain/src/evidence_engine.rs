use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// Interprets raw answer submissions into structured evidence events
/// and maintains the learner misconception state machine.
pub struct EvidenceInterpretationEngine<'a> {
    conn: &'a Connection,
}

// ---------------------------------------------------------------------------
// Evidence event output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEvent {
    pub id: i64,
    pub attempt_id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub testing_reason: Option<String>,
    pub evidence_weight: BasisPoints,
    pub mastery_delta: i64,
    pub stability_delta: i64,
    pub retention_delta: i64,
    pub transfer_delta: i64,
    pub timed_delta: i64,
    pub misconception_signal: Option<String>,
    pub hypothesis_result: Option<String>,
}

// ---------------------------------------------------------------------------
// Misconception state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MisconceptionStatus {
    Dormant,
    Suspected,
    Active,
    Reducing,
    ClearedButWatch,
    Cleared,
}

impl MisconceptionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dormant => "dormant",
            Self::Suspected => "suspected",
            Self::Active => "active",
            Self::Reducing => "reducing",
            Self::ClearedButWatch => "cleared_but_watch",
            Self::Cleared => "cleared",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "suspected" => Self::Suspected,
            "active" => Self::Active,
            "reducing" => Self::Reducing,
            "cleared_but_watch" => Self::ClearedButWatch,
            "cleared" => Self::Cleared,
            _ => Self::Dormant,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerMisconceptionSnapshot {
    pub misconception_id: i64,
    pub subject_id: i64,
    pub current_status: String,
    pub risk_score: BasisPoints,
    pub times_detected: i64,
    pub cleared_confidence: BasisPoints,
}

impl<'a> EvidenceInterpretationEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Interpret a completed answer attempt and produce an evidence event.
    /// Call this AFTER process_answer in student_model has written the attempt.
    pub fn interpret_attempt(
        &self,
        attempt_id: i64,
    ) -> EcoachResult<EvidenceEvent> {
        // Load the raw attempt
        let attempt = self.load_attempt(attempt_id)?;

        // Compute deltas based on answer context
        let mastery_delta = self.compute_mastery_delta(&attempt);
        let stability_delta = self.compute_stability_delta(&attempt);
        let retention_delta = self.compute_retention_delta(&attempt);
        let transfer_delta = self.compute_transfer_delta(&attempt);
        let timed_delta = self.compute_timed_delta(&attempt);
        let evidence_weight = self.compute_evidence_weight(&attempt);

        // Detect misconception signal from distractor choice
        let misconception_signal = if !attempt.is_correct {
            self.detect_misconception_signal(attempt.question_id, attempt.selected_option_id)?
        } else {
            None
        };

        // Determine hypothesis result
        let hypothesis_result = self.evaluate_hypothesis(&attempt);

        // Get testing reason from session_items if available
        let testing_reason = self.load_question_intent(attempt.session_id, attempt.question_id)?;

        // Persist the evidence event
        self.conn
            .execute(
                "INSERT INTO evidence_events
                    (attempt_id, student_id, subject_id, topic_id, node_id,
                     testing_reason, evidence_weight, mastery_delta, stability_delta,
                     retention_delta, transfer_delta, timed_delta,
                     misconception_signal, hypothesis_result)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    attempt_id,
                    attempt.student_id,
                    attempt.subject_id,
                    attempt.topic_id,
                    attempt.node_id,
                    testing_reason,
                    evidence_weight as i64,
                    mastery_delta,
                    stability_delta,
                    retention_delta,
                    transfer_delta,
                    timed_delta,
                    misconception_signal,
                    hypothesis_result,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let event_id = self.conn.last_insert_rowid();

        // Update misconception state machine if misconception was triggered
        if let Some(ref signal) = misconception_signal {
            if let Some(misconception_id) = attempt.misconception_triggered_id {
                self.update_misconception_state(
                    attempt.student_id,
                    attempt.subject_id,
                    misconception_id,
                    attempt.topic_id,
                    !attempt.is_correct,
                )?;
            }
        }

        // If correct and the misconception was previously active, start clearing it
        if attempt.is_correct {
            self.check_misconception_clearing(
                attempt.student_id,
                attempt.subject_id,
                attempt.topic_id,
            )?;
        }

        // Update skill node extended fields if we have a node_id
        if let Some(node_id) = attempt.node_id {
            self.update_skill_node_extended(
                attempt.student_id,
                node_id,
                &attempt,
                mastery_delta,
                stability_delta,
                transfer_delta,
                timed_delta,
            )?;
        }

        Ok(EvidenceEvent {
            id: event_id,
            attempt_id,
            student_id: attempt.student_id,
            subject_id: attempt.subject_id,
            topic_id: attempt.topic_id,
            node_id: attempt.node_id,
            testing_reason,
            evidence_weight,
            mastery_delta,
            stability_delta,
            retention_delta,
            transfer_delta,
            timed_delta,
            misconception_signal,
            hypothesis_result,
        })
    }

    /// Get active misconceptions for a student in a subject.
    pub fn list_active_misconceptions(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<LearnerMisconceptionSnapshot>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT misconception_id, subject_id, current_status, risk_score,
                        times_detected, cleared_confidence
                 FROM learner_misconception_states
                 WHERE student_id = ?1 AND subject_id = ?2
                   AND current_status NOT IN ('cleared', 'dormant')
                 ORDER BY risk_score DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, subject_id], |row| {
                Ok(LearnerMisconceptionSnapshot {
                    misconception_id: row.get(0)?,
                    subject_id: row.get(1)?,
                    current_status: row.get(2)?,
                    risk_score: clamp_bp(row.get::<_, i64>(3)?),
                    times_detected: row.get(4)?,
                    cleared_confidence: clamp_bp(row.get::<_, i64>(5)?),
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(results)
    }

    // -----------------------------------------------------------------------
    // Delta computation
    // -----------------------------------------------------------------------

    fn compute_mastery_delta(&self, attempt: &AttemptContext) -> i64 {
        if attempt.is_correct {
            // Higher delta for harder / more varied questions
            let base = if attempt.was_transfer_variant { 400 } else { 200 };
            let difficulty_bonus = (attempt.difficulty_level / 2000).min(3) as i64 * 50;
            base + difficulty_bonus
        } else {
            // Misconception-linked wrong answer = bigger drop
            let base = -200;
            let misconception_penalty = if attempt.misconception_triggered_id.is_some() {
                -150
            } else {
                0
            };
            base + misconception_penalty
        }
    }

    fn compute_stability_delta(&self, attempt: &AttemptContext) -> i64 {
        if attempt.is_correct {
            // Stability increases more for varied/delayed success
            if attempt.was_retention_check { 300 } else { 100 }
        } else {
            // Stability drops sharply for repeated errors
            if attempt.attempt_number > 1 { -400 } else { -200 }
        }
    }

    fn compute_retention_delta(&self, attempt: &AttemptContext) -> i64 {
        if attempt.was_retention_check {
            if attempt.is_correct { 500 } else { -600 }
        } else {
            0 // Only retention checks affect retention score
        }
    }

    fn compute_transfer_delta(&self, attempt: &AttemptContext) -> i64 {
        if attempt.was_transfer_variant {
            if attempt.is_correct { 400 } else { -300 }
        } else {
            0
        }
    }

    fn compute_timed_delta(&self, attempt: &AttemptContext) -> i64 {
        if attempt.was_timed {
            if attempt.is_correct {
                // Fast correct = big boost, slow correct = small boost
                if attempt.response_time_ms < 15_000 { 300 } else { 100 }
            } else {
                -200
            }
        } else {
            0
        }
    }

    fn compute_evidence_weight(&self, attempt: &AttemptContext) -> BasisPoints {
        let mut weight = 5000u16; // base

        // Higher evidence from novel/varied/delayed contexts
        if attempt.was_transfer_variant {
            weight = weight.saturating_add(2000);
        }
        if attempt.was_retention_check {
            weight = weight.saturating_add(1500);
        }
        if attempt.was_timed {
            weight = weight.saturating_add(1000);
        }
        if attempt.was_mixed_context {
            weight = weight.saturating_add(500);
        }

        // Lower evidence from hint-assisted or multi-attempt
        if attempt.hint_count > 0 {
            weight = weight.saturating_sub(2000);
        }
        if attempt.attempt_number > 1 {
            weight = weight.saturating_sub(1500);
        }

        clamp_bp(weight as i64)
    }

    fn evaluate_hypothesis(&self, attempt: &AttemptContext) -> Option<String> {
        // Simple hypothesis evaluation based on context flags
        if attempt.was_transfer_variant {
            if attempt.is_correct {
                Some("confirmed".into()) // Transfer hypothesis confirmed
            } else {
                Some("challenged".into()) // Learner can't transfer
            }
        } else if attempt.was_retention_check {
            if attempt.is_correct {
                Some("confirmed".into()) // Retention holds
            } else {
                Some("challenged".into()) // Knowledge fading
            }
        } else {
            Some("neutral".into())
        }
    }

    // -----------------------------------------------------------------------
    // Misconception state machine
    // -----------------------------------------------------------------------

    fn update_misconception_state(
        &self,
        student_id: i64,
        subject_id: i64,
        misconception_id: i64,
        topic_id: Option<i64>,
        was_triggered: bool,
    ) -> EcoachResult<()> {
        let existing = self
            .conn
            .query_row(
                "SELECT current_status, times_detected, risk_score
                 FROM learner_misconception_states
                 WHERE student_id = ?1 AND misconception_id = ?2",
                params![student_id, misconception_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let now = chrono::Utc::now().to_rfc3339();
        let related_json = topic_id
            .map(|t| format!("[{}]", t))
            .unwrap_or_else(|| "[]".into());

        match existing {
            Some((current_status, times_detected, risk_score)) => {
                if !was_triggered {
                    return Ok(()); // Only update on trigger
                }
                let status = MisconceptionStatus::from_str(&current_status);
                let new_times = times_detected + 1;

                // State transitions on trigger
                let new_status = match status {
                    MisconceptionStatus::Dormant => MisconceptionStatus::Suspected,
                    MisconceptionStatus::Suspected => {
                        if new_times >= 2 {
                            MisconceptionStatus::Active
                        } else {
                            MisconceptionStatus::Suspected
                        }
                    }
                    MisconceptionStatus::Active => MisconceptionStatus::Active,
                    MisconceptionStatus::Reducing => MisconceptionStatus::Active, // regression
                    MisconceptionStatus::ClearedButWatch => MisconceptionStatus::Suspected, // recurrence
                    MisconceptionStatus::Cleared => MisconceptionStatus::Suspected, // recurrence
                };

                let new_risk = clamp_bp(risk_score + 1500);

                self.conn
                    .execute(
                        "UPDATE learner_misconception_states
                         SET current_status = ?1, times_detected = ?2, risk_score = ?3,
                             last_detected_at = ?4, cleared_confidence = 0, updated_at = ?4
                         WHERE student_id = ?5 AND misconception_id = ?6",
                        params![
                            new_status.as_str(),
                            new_times,
                            new_risk as i64,
                            now,
                            student_id,
                            misconception_id,
                        ],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
            }
            None => {
                // First detection
                self.conn
                    .execute(
                        "INSERT INTO learner_misconception_states
                            (student_id, subject_id, misconception_id, risk_score,
                             first_detected_at, last_detected_at, times_detected,
                             current_status, related_node_ids_json)
                         VALUES (?1, ?2, ?3, 3000, ?4, ?4, 1, 'suspected', ?5)",
                        params![student_id, subject_id, misconception_id, now, related_json],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// When the student gets a correct answer on a topic where they had active misconceptions,
    /// start reducing those misconceptions.
    fn check_misconception_clearing(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<()> {
        let Some(topic_id) = topic_id else {
            return Ok(());
        };

        // Find active misconceptions on this topic
        let mut stmt = self
            .conn
            .prepare(
                "SELECT lms.misconception_id, lms.current_status, lms.cleared_confidence
                 FROM learner_misconception_states lms
                 INNER JOIN misconception_patterns mp ON mp.id = lms.misconception_id
                 WHERE lms.student_id = ?1 AND lms.subject_id = ?2
                   AND mp.topic_id = ?3
                   AND lms.current_status IN ('suspected', 'active', 'reducing')",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows: Vec<(i64, String, i64)> = stmt
            .query_map(params![student_id, subject_id, topic_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        let now = chrono::Utc::now().to_rfc3339();
        for (misconception_id, current_status, cleared_conf) in rows {
            let status = MisconceptionStatus::from_str(&current_status);
            let new_cleared = clamp_bp(cleared_conf + 2000);

            let new_status = match status {
                MisconceptionStatus::Active => MisconceptionStatus::Reducing,
                MisconceptionStatus::Suspected => MisconceptionStatus::Reducing,
                MisconceptionStatus::Reducing => {
                    if new_cleared >= 7000 {
                        MisconceptionStatus::ClearedButWatch
                    } else {
                        MisconceptionStatus::Reducing
                    }
                }
                other => other,
            };

            let new_risk = clamp_bp(
                self.conn
                    .query_row(
                        "SELECT risk_score FROM learner_misconception_states
                         WHERE student_id = ?1 AND misconception_id = ?2",
                        params![student_id, misconception_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .unwrap_or(5000)
                    - 1000,
            );

            self.conn
                .execute(
                    "UPDATE learner_misconception_states
                     SET current_status = ?1, cleared_confidence = ?2, risk_score = ?3, updated_at = ?4
                     WHERE student_id = ?5 AND misconception_id = ?6",
                    params![
                        new_status.as_str(),
                        new_cleared as i64,
                        new_risk as i64,
                        now,
                        student_id,
                        misconception_id,
                    ],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Skill node extended field updates
    // -----------------------------------------------------------------------

    fn update_skill_node_extended(
        &self,
        student_id: i64,
        node_id: i64,
        attempt: &AttemptContext,
        mastery_delta: i64,
        stability_delta: i64,
        transfer_delta: i64,
        timed_delta: i64,
    ) -> EcoachResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        // Update extended columns on student_skill_states
        self.conn
            .execute(
                "UPDATE student_skill_states SET
                    stability_score = MAX(0, MIN(10000, stability_score + ?1)),
                    transfer_strength = MAX(0, MIN(10000, transfer_strength + ?2)),
                    timed_strength = MAX(0, MIN(10000, timed_strength + ?3)),
                    coverage_depth = MIN(10000, coverage_depth + 200),
                    last_confirmed_at = CASE WHEN ?4 = 1 THEN ?5 ELSE last_confirmed_at END,
                    updated_at = ?5
                 WHERE student_id = ?6 AND node_id = ?7",
                params![
                    stability_delta,
                    transfer_delta,
                    timed_delta,
                    if attempt.is_correct { 1 } else { 0 },
                    now,
                    student_id,
                    node_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Update historical peak
        self.conn
            .execute(
                "UPDATE student_skill_states
                 SET historical_peak_level = CASE
                     WHEN mastery_score > COALESCE(
                         CAST(REPLACE(REPLACE(COALESCE(historical_peak_level, '0'), 'exam_ready', '10000'), 'stable', '8000') AS INTEGER),
                         0
                     ) THEN CAST(mastery_score AS TEXT)
                     ELSE historical_peak_level
                 END
                 WHERE student_id = ?1 AND node_id = ?2",
                params![student_id, node_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Update node_status based on current scores
        let (mastery, stability, retention, transfer, timed, evidence): (i64, i64, i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT mastery_score, stability_score, retention_confidence,
                        transfer_strength, timed_strength, evidence_count
                 FROM student_skill_states
                 WHERE student_id = ?1 AND node_id = ?2",
                params![student_id, node_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .unwrap_or((0, 5000, 5000, 0, 0, 0));

        let node_status = resolve_node_status(mastery, stability, retention, transfer, timed, evidence);

        self.conn
            .execute(
                "UPDATE student_skill_states SET node_status = ?1 WHERE student_id = ?2 AND node_id = ?3",
                params![node_status, student_id, node_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Data loading helpers
    // -----------------------------------------------------------------------

    fn load_attempt(&self, attempt_id: i64) -> EcoachResult<AttemptContext> {
        self.conn
            .query_row(
                "SELECT sqa.id, sqa.student_id, sqa.question_id, sqa.session_id,
                        sqa.is_correct, sqa.response_time_ms, sqa.confidence_level,
                        sqa.hint_count, sqa.attempt_number, sqa.selected_option_id,
                        sqa.was_timed, sqa.was_transfer_variant, sqa.was_retention_check,
                        sqa.was_mixed_context, sqa.misconception_triggered_id,
                        q.subject_id, q.topic_id, q.difficulty_level,
                        (SELECT node_id FROM question_skill_links WHERE question_id = q.id AND is_primary = 1 LIMIT 1)
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.id = ?1",
                [attempt_id],
                |row| {
                    Ok(AttemptContext {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        question_id: row.get(2)?,
                        session_id: row.get(3)?,
                        is_correct: row.get::<_, i64>(4)? == 1,
                        response_time_ms: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                        confidence_level: row.get(6)?,
                        hint_count: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                        attempt_number: row.get::<_, Option<i64>>(8)?.unwrap_or(1),
                        selected_option_id: row.get(9)?,
                        was_timed: row.get::<_, Option<i64>>(10)?.unwrap_or(0) == 1,
                        was_transfer_variant: row.get::<_, Option<i64>>(11)?.unwrap_or(0) == 1,
                        was_retention_check: row.get::<_, Option<i64>>(12)?.unwrap_or(0) == 1,
                        was_mixed_context: row.get::<_, Option<i64>>(13)?.unwrap_or(0) == 1,
                        misconception_triggered_id: row.get(14)?,
                        subject_id: row.get(15)?,
                        topic_id: row.get(16)?,
                        difficulty_level: row.get::<_, Option<i64>>(17)?.unwrap_or(5000),
                        node_id: row.get(18)?,
                    })
                },
            )
            .map_err(|e| EcoachError::NotFound(format!("attempt {} not found: {}", attempt_id, e)))
    }

    fn detect_misconception_signal(
        &self,
        question_id: i64,
        selected_option_id: i64,
    ) -> EcoachResult<Option<String>> {
        let signal: Option<String> = self
            .conn
            .query_row(
                "SELECT mp.title FROM question_options qo
                 INNER JOIN misconception_patterns mp ON mp.id = qo.misconception_id
                 WHERE qo.id = ?1 AND qo.question_id = ?2",
                params![selected_option_id, question_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(signal)
    }

    fn load_question_intent(
        &self,
        session_id: i64,
        question_id: i64,
    ) -> EcoachResult<Option<String>> {
        self.conn
            .query_row(
                "SELECT question_intent FROM session_items
                 WHERE session_id = ?1 AND question_id = ?2",
                params![session_id, question_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))
            .map(|r| r.flatten())
    }
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct AttemptContext {
    id: i64,
    student_id: i64,
    question_id: i64,
    session_id: i64,
    is_correct: bool,
    response_time_ms: i64,
    confidence_level: Option<String>,
    hint_count: i64,
    attempt_number: i64,
    selected_option_id: i64,
    was_timed: bool,
    was_transfer_variant: bool,
    was_retention_check: bool,
    was_mixed_context: bool,
    misconception_triggered_id: Option<i64>,
    subject_id: i64,
    topic_id: Option<i64>,
    difficulty_level: i64,
    node_id: Option<i64>,
}

/// Resolve the node_status string based on multi-dimensional scores.
fn resolve_node_status(
    mastery: i64,
    stability: i64,
    retention: i64,
    transfer: i64,
    timed: i64,
    evidence: i64,
) -> &'static str {
    if evidence == 0 {
        return "unseen";
    }
    if evidence < 3 {
        return "sampled";
    }
    if mastery >= 9000 && stability >= 8000 && retention >= 7000 && transfer >= 6000 && timed >= 6000 {
        return "mastered";
    }
    if mastery >= 8000 && stability >= 7000 && timed >= 5000 {
        return "exam_ready";
    }
    if mastery >= 7000 && stability >= 6000 {
        return "strongly_retained";
    }
    if mastery >= 6000 && stability >= 5000 {
        return "retained";
    }
    if mastery >= 5000 && stability < 4000 {
        return "fragile";
    }
    if stability >= 4000 && mastery >= 4000 {
        return "stabilizing";
    }
    if mastery >= 3000 {
        return "improving";
    }
    if retention < 3000 && evidence >= 5 {
        return "at_risk";
    }
    "sampled"
}
