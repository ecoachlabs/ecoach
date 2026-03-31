use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json;

/// Watch Me Solve / Breakpoint Detection Engine.
/// Captures step-by-step solving and detects the first wrong turn.
pub struct SteppedQuestionEngine<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepTemplate {
    pub step_number: i64,
    pub step_label: String,
    pub step_instruction: String,
    pub correct_answer: String,
    pub common_wrong_answers: Vec<WrongStepOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrongStepOption {
    pub answer: String,
    pub misconception: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAttemptData {
    pub step_number: i64,
    pub student_answer: String,
    pub is_correct: bool,
    pub response_time_ms: i64,
    pub misconception_detected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteppedAttemptResult {
    pub attempt_id: i64,
    pub question_id: i64,
    pub total_steps: i64,
    pub steps_completed: i64,
    pub breakpoint_step: Option<i64>,
    pub breakpoint_reason: Option<String>,
    pub thinking_map: ThinkingMap,
    pub overall_correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingMap {
    pub steps: Vec<ThinkingMapStep>,
    pub first_wrong_turn: Option<i64>,
    pub reasoning_type: String,
    pub diagnosis_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingMapStep {
    pub step_number: i64,
    pub label: String,
    pub correct: bool,
    pub misconception: Option<String>,
}

impl<'a> SteppedQuestionEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Start a stepped attempt for a question that has a step template.
    pub fn start_stepped_attempt(
        &self,
        student_id: i64,
        question_id: i64,
        session_id: Option<i64>,
    ) -> EcoachResult<i64> {
        let template = self.load_template(question_id)?;

        self.conn
            .execute(
                "INSERT INTO stepped_attempts
                    (student_id, question_id, template_id, session_id, total_steps)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    student_id, question_id, template.id, session_id, template.total_steps,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Record a single step in the attempt.
    pub fn record_step(
        &self,
        attempt_id: i64,
        step: StepAttemptData,
    ) -> EcoachResult<()> {
        // Load existing steps
        let existing_json: String = self
            .conn
            .query_row(
                "SELECT steps_data_json FROM stepped_attempts WHERE id = ?1",
                [attempt_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::NotFound(format!("attempt {attempt_id}: {e}")))?;

        let mut steps: Vec<StepAttemptData> =
            serde_json::from_str(&existing_json).unwrap_or_default();
        steps.push(step);

        let updated_json = serde_json::to_string(&steps)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "UPDATE stepped_attempts
                 SET steps_data_json = ?1, steps_completed = ?2
                 WHERE id = ?3",
                params![updated_json, steps.len() as i64, attempt_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Complete the stepped attempt and detect breakpoint.
    pub fn complete_stepped_attempt(
        &self,
        attempt_id: i64,
    ) -> EcoachResult<SteppedAttemptResult> {
        let (question_id, template_id, total_steps, steps_json): (i64, i64, i64, String) = self
            .conn
            .query_row(
                "SELECT question_id, template_id, total_steps, steps_data_json
                 FROM stepped_attempts WHERE id = ?1",
                [attempt_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| EcoachError::NotFound(format!("attempt {attempt_id}: {e}")))?;

        let steps: Vec<StepAttemptData> =
            serde_json::from_str(&steps_json).unwrap_or_default();

        // Load template for step labels
        let template_steps_json: String = self
            .conn
            .query_row(
                "SELECT steps_json FROM stepped_question_templates WHERE id = ?1",
                [template_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "[]".into());

        let template_steps: Vec<StepTemplate> =
            serde_json::from_str(&template_steps_json).unwrap_or_default();

        let reasoning_type: String = self
            .conn
            .query_row(
                "SELECT subject_reasoning_type FROM stepped_question_templates WHERE id = ?1",
                [template_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "procedural".into());

        // Detect breakpoint: first wrong step
        let mut breakpoint_step: Option<i64> = None;
        let mut breakpoint_reason: Option<String> = None;
        let mut thinking_steps = Vec::new();
        let mut all_correct = true;

        for step in &steps {
            let label = template_steps
                .iter()
                .find(|t| t.step_number == step.step_number)
                .map(|t| t.step_label.clone())
                .unwrap_or_else(|| format!("Step {}", step.step_number));

            thinking_steps.push(ThinkingMapStep {
                step_number: step.step_number,
                label,
                correct: step.is_correct,
                misconception: step.misconception_detected.clone(),
            });

            if !step.is_correct && breakpoint_step.is_none() {
                breakpoint_step = Some(step.step_number);
                breakpoint_reason = step
                    .misconception_detected
                    .clone()
                    .or_else(|| Some("Incorrect reasoning at this step".into()));
                all_correct = false;
            }
        }

        let diagnosis_summary = if all_correct {
            "All steps completed correctly. Understanding appears solid.".into()
        } else if let Some(bp) = breakpoint_step {
            format!(
                "Thinking broke at step {}. {}",
                bp,
                breakpoint_reason.as_deref().unwrap_or("Unknown reason")
            )
        } else {
            "Attempt incomplete.".into()
        };

        let thinking_map = ThinkingMap {
            steps: thinking_steps,
            first_wrong_turn: breakpoint_step,
            reasoning_type: reasoning_type.clone(),
            diagnosis_summary: diagnosis_summary.clone(),
        };

        let thinking_json = serde_json::to_string(&thinking_map)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Persist results
        self.conn
            .execute(
                "UPDATE stepped_attempts
                 SET breakpoint_step = ?1, breakpoint_reason = ?2,
                     thinking_map_json = ?3, overall_correct = ?4,
                     completed_at = datetime('now')
                 WHERE id = ?5",
                params![
                    breakpoint_step,
                    breakpoint_reason,
                    thinking_json,
                    if all_correct { 1 } else { 0 },
                    attempt_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(SteppedAttemptResult {
            attempt_id,
            question_id,
            total_steps,
            steps_completed: steps.len() as i64,
            breakpoint_step,
            breakpoint_reason: breakpoint_reason.clone(),
            thinking_map,
            overall_correct: all_correct,
        })
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn load_template(&self, question_id: i64) -> EcoachResult<TemplateRow> {
        self.conn
            .query_row(
                "SELECT id, total_steps FROM stepped_question_templates WHERE question_id = ?1",
                [question_id],
                |row| Ok(TemplateRow { id: row.get(0)?, total_steps: row.get(1)? }),
            )
            .map_err(|e| {
                EcoachError::NotFound(format!(
                    "no stepped template for question {question_id}: {e}"
                ))
            })
    }
}

struct TemplateRow {
    id: i64,
    total_steps: i64,
}
