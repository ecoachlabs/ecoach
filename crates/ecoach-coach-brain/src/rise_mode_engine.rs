use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Rise Mode: Last-to-First transformation engine.
/// 4-stage journey for the weakest students to become the strongest.
pub struct RiseModeEngine<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformationStage {
    Rescue,
    Stabilize,
    Accelerate,
    Dominate,
    Completed,
}

impl TransformationStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rescue => "rescue",
            Self::Stabilize => "stabilize",
            Self::Accelerate => "accelerate",
            Self::Dominate => "dominate",
            Self::Completed => "completed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "rescue" => Self::Rescue,
            "stabilize" => Self::Stabilize,
            "accelerate" => Self::Accelerate,
            "dominate" => Self::Dominate,
            "completed" => Self::Completed,
            _ => Self::Rescue,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Rescue => "Stage 1: Rescue — Stop the bleeding",
            Self::Stabilize => "Stage 2: Stabilize — Make correct thinking repeatable",
            Self::Accelerate => "Stage 3: Accelerate — Build speed and independence",
            Self::Dominate => "Stage 4: Dominate — Outperform everyone",
            Self::Completed => "Transformation Complete",
        }
    }

    pub fn purpose(self) -> &'static str {
        match self {
            Self::Rescue => {
                "Find the lowest-level gaps, rebuild understanding, get first wins quickly"
            }
            Self::Stabilize => {
                "Repeated practice, concept clusters, misconception correction, reliability"
            }
            Self::Accelerate => "Timed drills, mixed topics, pressure tolerance, faster recall",
            Self::Dominate => {
                "Advanced variants, trap questions, exam strategy, speed + accuracy together"
            }
            Self::Completed => "You have been transformed from struggling to strong",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiseModeProfile {
    pub student_id: i64,
    pub subject_id: i64,
    pub current_stage: String,
    pub stage_label: String,
    pub stage_purpose: String,
    pub foundation_score: BasisPoints,
    pub recall_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub accuracy_score: BasisPoints,
    pub pressure_stability_score: BasisPoints,
    pub misconception_density_score: BasisPoints,
    pub momentum_score: BasisPoints,
    pub transformation_readiness_score: BasisPoints,
    pub confidence_score: BasisPoints,
    pub weakness_map: Value,
    pub recovery_plan: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTransitionResult {
    pub previous_stage: String,
    pub new_stage: String,
    pub reason: String,
    pub next_stage_purpose: String,
}

impl<'a> RiseModeEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Enter Rise Mode for a student in a subject. Runs initial assessment.
    pub fn enter_rise_mode(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<RiseModeProfile> {
        // Compute all 8 scores
        let foundation = self.compute_foundation_score(student_id, subject_id)?;
        let recall = self.compute_recall_score(student_id, subject_id)?;
        let speed = self.compute_speed_score(student_id, subject_id)?;
        let accuracy = self.compute_accuracy_score(student_id, subject_id)?;
        let pressure = self.compute_pressure_stability(student_id, subject_id)?;
        let misconception = self.compute_misconception_density(student_id, subject_id)?;
        let momentum = 5000u16; // neutral at start
        let confidence = self.compute_confidence_score(student_id, subject_id)?;

        // Build weakness map
        let weakness_map = self.build_weakness_map(student_id, subject_id)?;

        // Build recovery plan based on stage
        let stage = self.determine_initial_stage(foundation, accuracy, speed);
        let recovery_plan = self.build_recovery_plan(student_id, subject_id, stage)?;

        let readiness = self.compute_transformation_readiness(
            foundation,
            recall,
            speed,
            accuracy,
            pressure,
            misconception,
        );

        let weakness_json = serde_json::to_string(&weakness_map).unwrap_or_else(|_| "{}".into());
        let plan_json = serde_json::to_string(&recovery_plan).unwrap_or_else(|_| "{}".into());

        self.conn
            .execute(
                "INSERT INTO rise_mode_profiles
                    (student_id, subject_id, current_stage,
                     foundation_score, recall_score, speed_score, accuracy_score,
                     pressure_stability_score, misconception_density_score,
                     momentum_score, transformation_readiness_score, confidence_score,
                     weakness_map_json, recovery_plan_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                 ON CONFLICT(student_id, subject_id) DO UPDATE SET
                    current_stage = ?3, foundation_score = ?4, recall_score = ?5,
                    speed_score = ?6, accuracy_score = ?7, pressure_stability_score = ?8,
                    misconception_density_score = ?9, momentum_score = ?10,
                    transformation_readiness_score = ?11, confidence_score = ?12,
                    weakness_map_json = ?13, recovery_plan_json = ?14,
                    stage_entered_at = datetime('now'), updated_at = datetime('now')",
                params![
                    student_id,
                    subject_id,
                    stage.as_str(),
                    foundation as i64,
                    recall as i64,
                    speed as i64,
                    accuracy as i64,
                    pressure as i64,
                    misconception as i64,
                    momentum as i64,
                    readiness as i64,
                    confidence as i64,
                    weakness_json,
                    plan_json,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(RiseModeProfile {
            student_id,
            subject_id,
            current_stage: stage.as_str().into(),
            stage_label: stage.display_label().into(),
            stage_purpose: stage.purpose().into(),
            foundation_score: foundation,
            recall_score: recall,
            speed_score: speed,
            accuracy_score: accuracy,
            pressure_stability_score: pressure,
            misconception_density_score: misconception,
            momentum_score: momentum,
            transformation_readiness_score: readiness,
            confidence_score: confidence,
            weakness_map,
            recovery_plan,
        })
    }

    /// Get the current Rise Mode profile.
    pub fn get_profile(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Option<RiseModeProfile>> {
        self.conn
            .query_row(
                "SELECT current_stage, foundation_score, recall_score, speed_score,
                        accuracy_score, pressure_stability_score, misconception_density_score,
                        momentum_score, transformation_readiness_score, confidence_score,
                        weakness_map_json, recovery_plan_json
                 FROM rise_mode_profiles
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
                |row| {
                    let stage_str: String = row.get(0)?;
                    let stage = TransformationStage::from_str(&stage_str);
                    let weakness_json: String = row.get(10)?;
                    let plan_json: String = row.get(11)?;

                    Ok(RiseModeProfile {
                        student_id,
                        subject_id,
                        current_stage: stage_str,
                        stage_label: stage.display_label().into(),
                        stage_purpose: stage.purpose().into(),
                        foundation_score: clamp_bp(row.get::<_, i64>(1)?),
                        recall_score: clamp_bp(row.get::<_, i64>(2)?),
                        speed_score: clamp_bp(row.get::<_, i64>(3)?),
                        accuracy_score: clamp_bp(row.get::<_, i64>(4)?),
                        pressure_stability_score: clamp_bp(row.get::<_, i64>(5)?),
                        misconception_density_score: clamp_bp(row.get::<_, i64>(6)?),
                        momentum_score: clamp_bp(row.get::<_, i64>(7)?),
                        transformation_readiness_score: clamp_bp(row.get::<_, i64>(8)?),
                        confidence_score: clamp_bp(row.get::<_, i64>(9)?),
                        weakness_map: serde_json::from_str(&weakness_json).unwrap_or(json!({})),
                        recovery_plan: serde_json::from_str(&plan_json).unwrap_or(json!({})),
                    })
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }

    /// Check if student is ready to advance to the next stage.
    pub fn check_stage_transition(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Option<StageTransitionResult>> {
        let profile = self
            .get_profile(student_id, subject_id)?
            .ok_or_else(|| EcoachError::NotFound("no rise mode profile".into()))?;

        let current = TransformationStage::from_str(&profile.current_stage);
        let next = match current {
            TransformationStage::Rescue => {
                if profile.foundation_score >= 5000 && profile.accuracy_score >= 4000 {
                    Some((
                        TransformationStage::Stabilize,
                        "Foundation rebuilt — ready to stabilize",
                    ))
                } else {
                    None
                }
            }
            TransformationStage::Stabilize => {
                if profile.accuracy_score >= 6500
                    && profile.recall_score >= 5000
                    && profile.misconception_density_score < 3000
                {
                    Some((
                        TransformationStage::Accelerate,
                        "Thinking is reliable — ready to accelerate",
                    ))
                } else {
                    None
                }
            }
            TransformationStage::Accelerate => {
                if profile.speed_score >= 6000
                    && profile.pressure_stability_score >= 6000
                    && profile.accuracy_score >= 7500
                {
                    Some((
                        TransformationStage::Dominate,
                        "Speed and composure achieved — ready to dominate",
                    ))
                } else {
                    None
                }
            }
            TransformationStage::Dominate => {
                if profile.transformation_readiness_score >= 8500 {
                    Some((
                        TransformationStage::Completed,
                        "Transformation complete — you have risen",
                    ))
                } else {
                    None
                }
            }
            TransformationStage::Completed => None,
        };

        if let Some((new_stage, reason)) = next {
            self.conn
                .execute(
                    "UPDATE rise_mode_profiles SET current_stage = ?1,
                         stage_entered_at = datetime('now'), updated_at = datetime('now')
                     WHERE student_id = ?2 AND subject_id = ?3",
                    params![new_stage.as_str(), student_id, subject_id],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;

            Ok(Some(StageTransitionResult {
                previous_stage: current.as_str().into(),
                new_stage: new_stage.as_str().into(),
                reason: reason.into(),
                next_stage_purpose: new_stage.purpose().into(),
            }))
        } else {
            Ok(None)
        }
    }

    // -----------------------------------------------------------------------
    // Score computation
    // -----------------------------------------------------------------------

    fn compute_foundation_score(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let avg_mastery: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(mastery_score), 0) FROM student_topic_states
             WHERE student_id = ?1 AND topic_id IN (SELECT id FROM topics WHERE subject_id = ?2)",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(clamp_bp(avg_mastery))
    }

    fn compute_recall_score(&self, student_id: i64, subject_id: i64) -> EcoachResult<BasisPoints> {
        let avg_retention: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(retention_score), 0) FROM student_topic_states
             WHERE student_id = ?1 AND topic_id IN (SELECT id FROM topics WHERE subject_id = ?2)",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(clamp_bp(avg_retention))
    }

    fn compute_speed_score(&self, student_id: i64, subject_id: i64) -> EcoachResult<BasisPoints> {
        let avg_speed: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(speed_score), 0) FROM student_topic_states
             WHERE student_id = ?1 AND topic_id IN (SELECT id FROM topics WHERE subject_id = ?2)",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(clamp_bp(avg_speed))
    }

    fn compute_accuracy_score(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let (total, correct): (i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END)
             FROM student_question_attempts
             WHERE student_id = ?1 AND question_id IN (
                 SELECT id FROM questions WHERE subject_id = ?2
             ) AND created_at >= datetime('now', '-30 days')",
                params![student_id, subject_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0));
        if total == 0 {
            return Ok(0);
        }
        Ok(to_bp(correct as f64 / total as f64))
    }

    fn compute_pressure_stability(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let avg_pci: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(pressure_collapse_index), 5000) FROM student_topic_states
             WHERE student_id = ?1 AND topic_id IN (SELECT id FROM topics WHERE subject_id = ?2)",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(5000);
        Ok(clamp_bp(10_000 - avg_pci)) // invert: high PCI = low stability
    }

    fn compute_misconception_density(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM learner_misconception_states
             WHERE student_id = ?1 AND subject_id = ?2 AND current_status IN ('active', 'suspected')",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);
        Ok(clamp_bp((count as f64 / 5.0).min(1.0) as i64 * 10_000))
    }

    fn compute_confidence_score(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let avg_conf: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(confidence_score), 5000) FROM student_topic_states
             WHERE student_id = ?1 AND topic_id IN (SELECT id FROM topics WHERE subject_id = ?2)",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(5000);
        Ok(clamp_bp(avg_conf))
    }

    fn compute_transformation_readiness(
        &self,
        foundation: BasisPoints,
        recall: BasisPoints,
        speed: BasisPoints,
        accuracy: BasisPoints,
        pressure: BasisPoints,
        misconception: BasisPoints,
    ) -> BasisPoints {
        clamp_bp(
            ((foundation as f64 * 0.20
                + recall as f64 * 0.15
                + speed as f64 * 0.15
                + accuracy as f64 * 0.25
                + pressure as f64 * 0.15
                + (10_000 - misconception as i64) as f64 * 0.10)
                .round() as i64),
        )
    }

    fn determine_initial_stage(
        &self,
        foundation: BasisPoints,
        accuracy: BasisPoints,
        speed: BasisPoints,
    ) -> TransformationStage {
        if foundation < 3000 || accuracy < 3000 {
            TransformationStage::Rescue
        } else if accuracy < 6000 || foundation < 5000 {
            TransformationStage::Stabilize
        } else if speed < 5000 {
            TransformationStage::Accelerate
        } else {
            TransformationStage::Dominate
        }
    }

    fn build_weakness_map(&self, student_id: i64, subject_id: i64) -> EcoachResult<Value> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.name, sts.mastery_score, sts.gap_score
             FROM student_topic_states sts
             INNER JOIN topics t ON t.id = sts.topic_id
             WHERE sts.student_id = ?1 AND t.subject_id = ?2
             ORDER BY sts.gap_score DESC LIMIT 10",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows: Vec<Value> = stmt
            .query_map(params![student_id, subject_id], |row| {
                Ok(json!({
                    "topic": row.get::<_, String>(1)?,
                    "mastery": row.get::<_, i64>(2)?,
                    "gap": row.get::<_, i64>(3)?,
                }))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(json!({"weak_topics": rows}))
    }

    fn build_recovery_plan(
        &self,
        student_id: i64,
        subject_id: i64,
        stage: TransformationStage,
    ) -> EcoachResult<Value> {
        let plan = match stage {
            TransformationStage::Rescue => json!({
                "weeks": [
                    {"week": 1, "focus": "Find and repair lowest-level gaps", "activity": "foundation_repair"},
                    {"week": 2, "focus": "Rebuild core concepts", "activity": "concept_rebuild"},
                    {"week": 3, "focus": "First checkpoints and confidence wins", "activity": "checkpoint"},
                ]
            }),
            TransformationStage::Stabilize => json!({
                "weeks": [
                    {"week": 1, "focus": "Repeated practice on repaired concepts", "activity": "repeated_practice"},
                    {"week": 2, "focus": "Misconception correction", "activity": "misconception_repair"},
                    {"week": 3, "focus": "Mixed-topic reliability", "activity": "mixed_practice"},
                ]
            }),
            TransformationStage::Accelerate => json!({
                "weeks": [
                    {"week": 1, "focus": "Timed drills and speed building", "activity": "speed_drills"},
                    {"week": 2, "focus": "Mixed-topic pressure sets", "activity": "pressure_sets"},
                    {"week": 3, "focus": "Exam-style conditioning", "activity": "exam_conditioning"},
                ]
            }),
            TransformationStage::Dominate => json!({
                "weeks": [
                    {"week": 1, "focus": "Advanced variants and trap questions", "activity": "advanced_variants"},
                    {"week": 2, "focus": "Speed + accuracy together", "activity": "speed_accuracy"},
                    {"week": 3, "focus": "Hard mixed challenge sets", "activity": "challenge_sets"},
                ]
            }),
            TransformationStage::Completed => json!({"status": "complete"}),
        };
        Ok(plan)
    }
}
