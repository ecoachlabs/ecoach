use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// Topic Proof System: certifies whether a learner truly owns a topic
/// based on accuracy, speed, transfer, variation, pressure, and mistake recurrence.
pub struct TopicProofEngine<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofTier {
    NotReady,
    Emerging,
    Functional,
    Strong,
    Certified,
    Expert,
}

impl ProofTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotReady => "not_ready",
            Self::Emerging => "emerging",
            Self::Functional => "functional",
            Self::Strong => "strong",
            Self::Certified => "certified",
            Self::Expert => "expert",
        }
    }

    pub fn from_composite(score: BasisPoints) -> Self {
        match score {
            0..=2000 => Self::NotReady,
            2001..=4000 => Self::Emerging,
            4001..=6000 => Self::Functional,
            6001..=7500 => Self::Strong,
            7501..=9000 => Self::Certified,
            _ => Self::Expert,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::NotReady => "Not Ready Yet",
            Self::Emerging => "Emerging Understanding",
            Self::Functional => "Functional Knowledge",
            Self::Strong => "Strong Command",
            Self::Certified => "Certified Proficiency",
            Self::Expert => "Expert Mastery",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicProofCertification {
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub proof_tier: String,
    pub proof_label: String,
    pub accuracy_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub transfer_score: BasisPoints,
    pub variation_score: BasisPoints,
    pub pressure_score: BasisPoints,
    pub mistake_recurrence_score: BasisPoints,
    pub reasoning_score: BasisPoints,
    pub composite_score: BasisPoints,
    pub evidence_count: i64,
}

impl<'a> TopicProofEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Assess and certify a learner's proof tier for a topic.
    pub fn assess_topic_proof(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
    ) -> EcoachResult<TopicProofCertification> {
        let topic_name = self
            .conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get(0)
            })
            .map_err(|e| EcoachError::NotFound(format!("topic {topic_id}: {e}")))?;

        let accuracy = self.compute_accuracy_score(student_id, topic_id)?;
        let speed = self.compute_speed_score(student_id, topic_id)?;
        let transfer = self.compute_transfer_score(student_id, topic_id)?;
        let variation = self.compute_variation_score(student_id, topic_id)?;
        let pressure = self.compute_pressure_score(student_id, topic_id)?;
        let mistake_recurrence = self.compute_mistake_recurrence_score(student_id, topic_id)?;
        let reasoning = self.compute_reasoning_score(student_id, topic_id)?;

        let evidence_count = self.count_evidence(student_id, topic_id)?;

        // Composite: weighted blend
        let composite = clamp_bp(
            ((accuracy as f64 * 0.25
                + speed as f64 * 0.15
                + transfer as f64 * 0.15
                + variation as f64 * 0.10
                + pressure as f64 * 0.15
                + (10_000 - mistake_recurrence as i64) as f64 * 0.10
                + reasoning as f64 * 0.10)
                .round() as i64),
        );

        // Evidence penalty: need at least 10 attempts for certified, 20 for expert
        let evidence_adjusted = if evidence_count < 5 {
            clamp_bp((composite as i64 * 6 / 10).min(4000))
        } else if evidence_count < 10 {
            clamp_bp((composite as i64 * 8 / 10).min(7500))
        } else {
            composite
        };

        let tier = ProofTier::from_composite(evidence_adjusted);

        // Persist
        let now = chrono::Utc::now().to_rfc3339();
        let certified_at = if tier == ProofTier::Certified || tier == ProofTier::Expert {
            Some(now.clone())
        } else {
            None
        };

        self.conn
            .execute(
                "INSERT INTO topic_proof_certifications
                    (student_id, subject_id, topic_id, proof_tier,
                     accuracy_score, speed_score, transfer_score, variation_score,
                     pressure_score, mistake_recurrence_score, reasoning_score,
                     composite_score, evidence_count, last_assessed_at, certified_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
                 ON CONFLICT(student_id, topic_id) DO UPDATE SET
                    proof_tier = ?4, accuracy_score = ?5, speed_score = ?6,
                    transfer_score = ?7, variation_score = ?8, pressure_score = ?9,
                    mistake_recurrence_score = ?10, reasoning_score = ?11,
                    composite_score = ?12, evidence_count = ?13,
                    last_assessed_at = ?14, certified_at = COALESCE(?15, certified_at),
                    updated_at = datetime('now')",
                params![
                    student_id, subject_id, topic_id, tier.as_str(),
                    accuracy as i64, speed as i64, transfer as i64, variation as i64,
                    pressure as i64, mistake_recurrence as i64, reasoning as i64,
                    evidence_adjusted as i64, evidence_count, now, certified_at,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(TopicProofCertification {
            student_id,
            subject_id,
            topic_id,
            topic_name,
            proof_tier: tier.as_str().into(),
            proof_label: tier.display_label().into(),
            accuracy_score: accuracy,
            speed_score: speed,
            transfer_score: transfer,
            variation_score: variation,
            pressure_score: pressure,
            mistake_recurrence_score: mistake_recurrence,
            reasoning_score: reasoning,
            composite_score: evidence_adjusted,
            evidence_count,
        })
    }

    /// List all proof certifications for a student in a subject.
    pub fn list_proofs(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<TopicProofCertification>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT tpc.topic_id, t.name, tpc.proof_tier,
                        tpc.accuracy_score, tpc.speed_score, tpc.transfer_score,
                        tpc.variation_score, tpc.pressure_score,
                        tpc.mistake_recurrence_score, tpc.reasoning_score,
                        tpc.composite_score, tpc.evidence_count
                 FROM topic_proof_certifications tpc
                 INNER JOIN topics t ON t.id = tpc.topic_id
                 WHERE tpc.student_id = ?1 AND tpc.subject_id = ?2
                 ORDER BY tpc.composite_score DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, subject_id], |row| {
                let tier_str: String = row.get(2)?;
                let tier = ProofTier::from_composite(clamp_bp(row.get::<_, i64>(10)?));
                Ok(TopicProofCertification {
                    student_id,
                    subject_id,
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    proof_tier: tier_str,
                    proof_label: tier.display_label().into(),
                    accuracy_score: clamp_bp(row.get::<_, i64>(3)?),
                    speed_score: clamp_bp(row.get::<_, i64>(4)?),
                    transfer_score: clamp_bp(row.get::<_, i64>(5)?),
                    variation_score: clamp_bp(row.get::<_, i64>(6)?),
                    pressure_score: clamp_bp(row.get::<_, i64>(7)?),
                    mistake_recurrence_score: clamp_bp(row.get::<_, i64>(8)?),
                    reasoning_score: clamp_bp(row.get::<_, i64>(9)?),
                    composite_score: clamp_bp(row.get::<_, i64>(10)?),
                    evidence_count: row.get(11)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut proofs = Vec::new();
        for row in rows {
            proofs.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(proofs)
    }

    // -----------------------------------------------------------------------
    // Score computation
    // -----------------------------------------------------------------------

    fn compute_accuracy_score(&self, student_id: i64, topic_id: i64) -> EcoachResult<BasisPoints> {
        let (total, correct): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END)
                 FROM student_question_attempts
                 WHERE student_id = ?1 AND question_id IN (
                     SELECT id FROM questions WHERE topic_id = ?2
                 )",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0));

        if total == 0 {
            return Ok(0);
        }
        Ok(to_bp(correct as f64 / total as f64))
    }

    fn compute_speed_score(&self, student_id: i64, topic_id: i64) -> EcoachResult<BasisPoints> {
        let avg_time: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(response_time_ms), 0)
                 FROM student_question_attempts
                 WHERE student_id = ?1 AND is_correct = 1
                   AND question_id IN (SELECT id FROM questions WHERE topic_id = ?2)",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .unwrap_or(60_000);

        // Fast = high score. 5s = 10000, 30s = 5000, 60s+ = low
        let score = if avg_time <= 5_000 {
            10_000
        } else if avg_time <= 15_000 {
            8_000
        } else if avg_time <= 30_000 {
            6_000
        } else if avg_time <= 45_000 {
            4_000
        } else {
            2_000
        };
        Ok(clamp_bp(score))
    }

    fn compute_transfer_score(&self, student_id: i64, topic_id: i64) -> EcoachResult<BasisPoints> {
        let (total, correct): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END)
                 FROM student_question_attempts
                 WHERE student_id = ?1 AND was_transfer_variant = 1
                   AND question_id IN (SELECT id FROM questions WHERE topic_id = ?2)",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0));

        if total == 0 {
            return Ok(3000); // neutral — no transfer evidence
        }
        Ok(to_bp(correct as f64 / total as f64))
    }

    fn compute_variation_score(&self, student_id: i64, topic_id: i64) -> EcoachResult<BasisPoints> {
        // How many distinct question families answered
        let families: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT q.family_id)
                 FROM student_question_attempts sqa
                 INNER JOIN questions q ON q.id = sqa.question_id
                 WHERE sqa.student_id = ?1 AND q.topic_id = ?2 AND q.family_id IS NOT NULL",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // More families = higher score. 5+ = strong.
        Ok(clamp_bp((families as f64 / 5.0).min(1.0) as i64 * 10_000))
    }

    fn compute_pressure_score(&self, student_id: i64, topic_id: i64) -> EcoachResult<BasisPoints> {
        let (total, correct): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END)
                 FROM student_question_attempts
                 WHERE student_id = ?1 AND was_timed = 1
                   AND question_id IN (SELECT id FROM questions WHERE topic_id = ?2)",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0));

        if total == 0 {
            return Ok(3000);
        }
        Ok(to_bp(correct as f64 / total as f64))
    }

    fn compute_mistake_recurrence_score(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<BasisPoints> {
        // High recurrence = bad. Count misconceptions triggered multiple times.
        let recurring: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM (
                     SELECT misconception_triggered_id, COUNT(*) AS cnt
                     FROM student_question_attempts
                     WHERE student_id = ?1 AND misconception_triggered_id IS NOT NULL
                       AND question_id IN (SELECT id FROM questions WHERE topic_id = ?2)
                     GROUP BY misconception_triggered_id
                     HAVING cnt >= 2
                 )",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // 0 recurring = 0 (good), 3+ = 10000 (bad)
        Ok(clamp_bp((recurring as f64 / 3.0).min(1.0) as i64 * 10_000))
    }

    fn compute_reasoning_score(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<BasisPoints> {
        // Based on stepped attempts if available
        let (total, correct): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), SUM(overall_correct)
                 FROM stepped_attempts
                 WHERE student_id = ?1 AND question_id IN (
                     SELECT id FROM questions WHERE topic_id = ?2
                 ) AND completed_at IS NOT NULL",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0));

        if total == 0 {
            // No stepped data — use general mastery as proxy
            let mastery: i64 = self
                .conn
                .query_row(
                    "SELECT COALESCE(mastery_score, 0) FROM student_topic_states
                     WHERE student_id = ?1 AND topic_id = ?2",
                    params![student_id, topic_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            return Ok(clamp_bp(mastery));
        }

        Ok(to_bp(correct as f64 / total as f64))
    }

    fn count_evidence(&self, student_id: i64, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM student_question_attempts
                 WHERE student_id = ?1 AND question_id IN (
                     SELECT id FROM questions WHERE topic_id = ?2
                 )",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }
}
