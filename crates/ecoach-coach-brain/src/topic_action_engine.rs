use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Topic action modes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopicActionMode {
    Learn,
    Repair,
    Revision,
    Expert,
}

impl TopicActionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Learn => "learn",
            Self::Repair => "repair",
            Self::Revision => "revision",
            Self::Expert => "expert",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "learn" => Self::Learn,
            "repair" => Self::Repair,
            "revision" => Self::Revision,
            "expert" => Self::Expert,
            _ => Self::Learn,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Learn => "Teach Me This Topic",
            Self::Repair => "Fix My Weakness In This Topic",
            Self::Revision => "Help Me Revise This Topic",
            Self::Expert => "Make Me An Expert In This Topic",
        }
    }

    pub fn purpose_text(self) -> &'static str {
        match self {
            Self::Learn => "We'll teach this topic step by step, starting with the basics and building toward harder questions.",
            Self::Repair => "We'll find the exact part you are struggling with and help you repair it.",
            Self::Revision => "We'll quickly check how fresh this topic still is and bring back the most important ideas.",
            Self::Expert => "We'll challenge your reasoning, speed, and mastery to a much higher level.",
        }
    }
}

// ---------------------------------------------------------------------------
// Topic action session output
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicActionSession {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub action_mode: String,
    pub status: String,
    pub progress_score: BasisPoints,
    pub subtopics_total: i64,
    pub subtopics_completed: i64,
    pub mastery_at_start: BasisPoints,
    pub diagnosis: Option<TopicDiagnosis>,
    pub repair_path: Option<Vec<RepairStep>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDiagnosis {
    pub overall_mastery: BasisPoints,
    pub strong_subtopics: Vec<String>,
    pub weak_subtopics: Vec<String>,
    pub root_cause: String,
    pub symptom_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairStep {
    pub step_number: i64,
    pub subtopic_name: String,
    pub step_type: String,
    pub status: String,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

pub struct TopicActionEngine<'a> {
    conn: &'a Connection,
}

impl<'a> TopicActionEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Start a topic action session in one of the 4 modes.
    pub fn start_topic_action(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: i64,
        mode: TopicActionMode,
        symptom_input: Option<&str>,
    ) -> EcoachResult<TopicActionSession> {
        let topic_name = self.load_topic_name(topic_id)?;

        // Get current mastery
        let mastery_at_start = self.load_topic_mastery(student_id, topic_id)?;

        // Count subtopics
        let subtopics = self.load_subtopics(topic_id)?;
        let subtopics_total = subtopics.len() as i64;

        // Build diagnosis for repair mode
        let diagnosis = if mode == TopicActionMode::Repair {
            Some(self.diagnose_topic_weakness(student_id, topic_id, &subtopics, symptom_input)?)
        } else {
            None
        };

        // Build repair path
        let repair_path = match mode {
            TopicActionMode::Repair => {
                Some(self.build_repair_path(&subtopics, &diagnosis)?)
            }
            TopicActionMode::Learn => {
                Some(self.build_learn_path(&subtopics)?)
            }
            TopicActionMode::Revision => {
                Some(self.build_revision_path(student_id, &subtopics)?)
            }
            TopicActionMode::Expert => {
                Some(self.build_expert_path(&subtopics)?)
            }
        };

        let diagnosis_json = diagnosis
            .as_ref()
            .map(|d| serde_json::to_string(d).unwrap_or_default());
        let repair_path_json = repair_path
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());
        let symptom_json = symptom_input.map(|s| json!({"symptom": s}).to_string());

        self.conn
            .execute(
                "INSERT INTO topic_action_sessions
                    (student_id, subject_id, topic_id, action_mode, status,
                     diagnosis_json, repair_path_json, subtopics_total,
                     mastery_at_start, symptom_input_json)
                 VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6, ?7, ?8, ?9)",
                params![
                    student_id, subject_id, topic_id, mode.as_str(),
                    diagnosis_json, repair_path_json, subtopics_total,
                    mastery_at_start as i64, symptom_json,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let session_id = self.conn.last_insert_rowid();

        Ok(TopicActionSession {
            id: session_id,
            student_id,
            subject_id,
            topic_id,
            topic_name,
            action_mode: mode.as_str().into(),
            status: "active".into(),
            progress_score: 0,
            subtopics_total,
            subtopics_completed: 0,
            mastery_at_start,
            diagnosis,
            repair_path,
        })
    }

    /// Record progress in a topic action session.
    pub fn record_subtopic_completed(
        &self,
        session_id: i64,
        step_number: i64,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE topic_action_sessions
                 SET subtopics_completed = subtopics_completed + 1,
                     progress_score = MIN(10000, progress_score + (10000 / MAX(subtopics_total, 1)))
                 WHERE id = ?1",
                [session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Update repair_path step status
        let path_json: Option<String> = self
            .conn
            .query_row(
                "SELECT repair_path_json FROM topic_action_sessions WHERE id = ?1",
                [session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .flatten();

        if let Some(json_str) = path_json {
            if let Ok(mut steps) = serde_json::from_str::<Vec<RepairStep>>(&json_str) {
                if let Some(step) = steps.iter_mut().find(|s| s.step_number == step_number) {
                    step.status = "completed".into();
                }
                let updated = serde_json::to_string(&steps).unwrap_or(json_str);
                self.conn
                    .execute(
                        "UPDATE topic_action_sessions SET repair_path_json = ?1 WHERE id = ?2",
                        params![updated, session_id],
                    )
                    .ok();
            }
        }

        Ok(())
    }

    /// Complete a topic action session.
    pub fn complete_topic_action(
        &self,
        session_id: i64,
    ) -> EcoachResult<TopicActionSummary> {
        let (student_id, topic_id, mode_str, mastery_start): (i64, i64, String, i64) = self
            .conn
            .query_row(
                "SELECT student_id, topic_id, action_mode, mastery_at_start
                 FROM topic_action_sessions WHERE id = ?1",
                [session_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| EcoachError::NotFound(format!("session {session_id}: {e}")))?;

        let mastery_now = self.load_topic_mastery(student_id, topic_id)?;
        let mastery_delta = mastery_now as i64 - mastery_start;

        self.conn
            .execute(
                "UPDATE topic_action_sessions
                 SET status = 'completed', mastery_at_end = ?1, progress_score = 10000,
                     completed_at = datetime('now')
                 WHERE id = ?2",
                params![mastery_now as i64, session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mode = TopicActionMode::from_str(&mode_str);
        let summary_for_parent = match mode {
            TopicActionMode::Learn => format!(
                "Your child built this topic from the beginning. Mastery moved from {} to {}.",
                mastery_start / 100, mastery_now / 100
            ),
            TopicActionMode::Repair => format!(
                "The system identified specific weak spots and focused on repairing them. Mastery improved by {}.",
                mastery_delta / 100
            ),
            TopicActionMode::Revision => format!(
                "Your child refreshed prior learning. Current mastery is {}%.",
                mastery_now / 100
            ),
            TopicActionMode::Expert => format!(
                "Your child worked on advanced challenges to sharpen reasoning and speed. Mastery: {}%.",
                mastery_now / 100
            ),
        };

        Ok(TopicActionSummary {
            session_id,
            action_mode: mode_str,
            mastery_at_start: clamp_bp(mastery_start),
            mastery_at_end: mastery_now,
            mastery_delta,
            parent_summary: summary_for_parent,
        })
    }

    /// Get the active topic action session for a student + topic.
    pub fn get_active_session(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<Option<TopicActionSession>> {
        let row = self
            .conn
            .query_row(
                "SELECT tas.id, tas.subject_id, tas.action_mode, tas.status,
                        tas.progress_score, tas.subtopics_total, tas.subtopics_completed,
                        tas.mastery_at_start, tas.diagnosis_json, tas.repair_path_json,
                        t.name
                 FROM topic_action_sessions tas
                 INNER JOIN topics t ON t.id = tas.topic_id
                 WHERE tas.student_id = ?1 AND tas.topic_id = ?2 AND tas.status = 'active'
                 ORDER BY tas.created_at DESC LIMIT 1",
                params![student_id, topic_id],
                |row| {
                    let diagnosis_json: Option<String> = row.get(8)?;
                    let repair_path_json: Option<String> = row.get(9)?;

                    let diagnosis = diagnosis_json
                        .and_then(|j| serde_json::from_str(&j).ok());
                    let repair_path = repair_path_json
                        .and_then(|j| serde_json::from_str(&j).ok());

                    Ok(TopicActionSession {
                        id: row.get(0)?,
                        student_id,
                        subject_id: row.get(1)?,
                        topic_id,
                        topic_name: row.get(10)?,
                        action_mode: row.get(2)?,
                        status: row.get(3)?,
                        progress_score: clamp_bp(row.get::<_, i64>(4)?),
                        subtopics_total: row.get(5)?,
                        subtopics_completed: row.get(6)?,
                        mastery_at_start: clamp_bp(row.get::<_, i64>(7)?),
                        diagnosis,
                        repair_path,
                    })
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(row)
    }

    // -----------------------------------------------------------------------
    // Mode-specific path builders
    // -----------------------------------------------------------------------

    fn build_learn_path(&self, subtopics: &[(i64, String)]) -> EcoachResult<Vec<RepairStep>> {
        Ok(subtopics
            .iter()
            .enumerate()
            .map(|(i, (_, name))| RepairStep {
                step_number: (i + 1) as i64,
                subtopic_name: name.clone(),
                step_type: "learn".into(),
                status: if i == 0 { "active" } else { "locked" }.into(),
            })
            .collect())
    }

    fn build_repair_path(
        &self,
        subtopics: &[(i64, String)],
        diagnosis: &Option<TopicDiagnosis>,
    ) -> EcoachResult<Vec<RepairStep>> {
        let weak_names: Vec<&str> = diagnosis
            .as_ref()
            .map(|d| d.weak_subtopics.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();

        let mut steps = Vec::new();
        let mut step_num = 1;

        // Weak subtopics first
        for (_, name) in subtopics {
            if weak_names.contains(&name.as_str()) {
                steps.push(RepairStep {
                    step_number: step_num,
                    subtopic_name: name.clone(),
                    step_type: "repair".into(),
                    status: if step_num == 1 { "active" } else { "locked" }.into(),
                });
                step_num += 1;
            }
        }

        // Add checkpoint at end
        steps.push(RepairStep {
            step_number: step_num,
            subtopic_name: "Mixed Checkpoint".into(),
            step_type: "checkpoint".into(),
            status: "locked".into(),
        });

        Ok(steps)
    }

    fn build_revision_path(
        &self,
        student_id: i64,
        subtopics: &[(i64, String)],
    ) -> EcoachResult<Vec<RepairStep>> {
        let mut steps = vec![
            RepairStep {
                step_number: 1,
                subtopic_name: "Quick Recall Scan".into(),
                step_type: "recall_scan".into(),
                status: "active".into(),
            },
            RepairStep {
                step_number: 2,
                subtopic_name: "Concept Refresh".into(),
                step_type: "concept_refresh".into(),
                status: "locked".into(),
            },
            RepairStep {
                step_number: 3,
                subtopic_name: "Mixed Revision Questions".into(),
                step_type: "mixed_revision".into(),
                status: "locked".into(),
            },
            RepairStep {
                step_number: 4,
                subtopic_name: "Stability Check".into(),
                step_type: "stability_check".into(),
                status: "locked".into(),
            },
        ];

        Ok(steps)
    }

    fn build_expert_path(&self, subtopics: &[(i64, String)]) -> EcoachResult<Vec<RepairStep>> {
        Ok(vec![
            RepairStep {
                step_number: 1,
                subtopic_name: "Elite Baseline Check".into(),
                step_type: "elite_baseline".into(),
                status: "active".into(),
            },
            RepairStep {
                step_number: 2,
                subtopic_name: "Stretch Challenges".into(),
                step_type: "stretch".into(),
                status: "locked".into(),
            },
            RepairStep {
                step_number: 3,
                subtopic_name: "Trap & Variation Work".into(),
                step_type: "trap_variation".into(),
                status: "locked".into(),
            },
            RepairStep {
                step_number: 4,
                subtopic_name: "Speed & Precision Rounds".into(),
                step_type: "speed_precision".into(),
                status: "locked".into(),
            },
            RepairStep {
                step_number: 5,
                subtopic_name: "Mastery Gauntlet".into(),
                step_type: "mastery_gauntlet".into(),
                status: "locked".into(),
            },
        ])
    }

    // -----------------------------------------------------------------------
    // Topic-level diagnosis for repair mode
    // -----------------------------------------------------------------------

    fn diagnose_topic_weakness(
        &self,
        student_id: i64,
        topic_id: i64,
        subtopics: &[(i64, String)],
        symptom_input: Option<&str>,
    ) -> EcoachResult<TopicDiagnosis> {
        let overall_mastery = self.load_topic_mastery(student_id, topic_id)?;

        let mut strong = Vec::new();
        let mut weak = Vec::new();

        for (sub_id, sub_name) in subtopics {
            // Check if there are skill states for this subtopic's nodes
            let sub_mastery: i64 = self
                .conn
                .query_row(
                    "SELECT COALESCE(AVG(mastery_score), 0) FROM student_skill_states
                     WHERE student_id = ?1 AND node_id IN (
                         SELECT id FROM academic_nodes WHERE topic_id = ?2
                     )",
                    params![student_id, sub_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if sub_mastery >= 7000 {
                strong.push(sub_name.clone());
            } else {
                weak.push(sub_name.clone());
            }
        }

        // If no skill states, check topic-level attempt data
        if strong.is_empty() && weak.is_empty() {
            // Use topic mastery to infer
            if overall_mastery >= 5000 {
                strong.push("General understanding".into());
            }
            weak.push("Needs diagnostic evidence".into());
        }

        // Root cause from error profiles
        let root_cause = self.detect_root_cause(student_id, topic_id)?;

        Ok(TopicDiagnosis {
            overall_mastery,
            strong_subtopics: strong,
            weak_subtopics: weak,
            root_cause,
            symptom_summary: symptom_input.map(|s| s.to_string()),
        })
    }

    fn detect_root_cause(&self, student_id: i64, topic_id: i64) -> EcoachResult<String> {
        // Check error profiles
        let (knowledge_gap, conceptual, execution, carelessness, pressure): (i64, i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COALESCE(knowledge_gap_score, 0), COALESCE(conceptual_confusion_score, 0),
                        COALESCE(execution_error_score, 0), COALESCE(carelessness_score, 0),
                        COALESCE(pressure_breakdown_score, 0)
                 FROM student_error_profiles
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .unwrap_or((0, 0, 0, 0, 0));

        let max = *[knowledge_gap, conceptual, execution, carelessness, pressure]
            .iter()
            .max()
            .unwrap_or(&0);

        if max == 0 {
            return Ok("Not enough evidence yet to determine root cause".into());
        }

        Ok(if max == knowledge_gap {
            "Missing foundational knowledge in key concepts".into()
        } else if max == conceptual {
            "Confusion between related concepts or methods".into()
        } else if max == execution {
            "Understands the concept but makes errors in execution".into()
        } else if max == carelessness {
            "Knowledge is present but careless errors are frequent".into()
        } else {
            "Performance breaks down under time pressure".into()
        })
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn load_topic_name(&self, topic_id: i64) -> EcoachResult<String> {
        self.conn
            .query_row(
                "SELECT name FROM topics WHERE id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::NotFound(format!("topic {topic_id}: {e}")))
    }

    fn load_topic_mastery(&self, student_id: i64, topic_id: i64) -> EcoachResult<BasisPoints> {
        let mastery: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(mastery_score, 0) FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(clamp_bp(mastery))
    }

    fn load_subtopics(&self, topic_id: i64) -> EcoachResult<Vec<(i64, String)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name FROM topics
                 WHERE parent_topic_id = ?1 AND is_active = 1
                 ORDER BY display_order, id",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([topic_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut subtopics = Vec::new();
        for row in rows {
            subtopics.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }

        // If no subtopics, use the topic itself
        if subtopics.is_empty() {
            let name = self.load_topic_name(topic_id)?;
            subtopics.push((topic_id, name));
        }

        Ok(subtopics)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicActionSummary {
    pub session_id: i64,
    pub action_mode: String,
    pub mastery_at_start: BasisPoints,
    pub mastery_at_end: BasisPoints,
    pub mastery_delta: i64,
    pub parent_summary: String,
}
