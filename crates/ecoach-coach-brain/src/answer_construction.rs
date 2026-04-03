use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;

use crate::mastery_map::MasteryMapService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerRubricStep {
    pub label: String,
    pub expected_answer: String,
    pub keywords: Vec<String>,
    pub marks: i64,
    pub required: bool,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerRubricInput {
    pub rubric_id: Option<i64>,
    pub question_id: i64,
    pub rubric_type: String,
    pub total_marks: i64,
    pub steps: Vec<AnswerRubricStep>,
    pub mandatory_steps: Vec<String>,
    pub full_answer_example: Option<String>,
    pub concise_answer_example: Option<String>,
    pub weak_answer_example: Option<String>,
    pub common_omissions: Vec<String>,
    pub marking_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerRubric {
    pub id: i64,
    pub question_id: i64,
    pub rubric_type: String,
    pub total_marks: i64,
    pub steps: Vec<AnswerRubricStep>,
    pub mandatory_steps: Vec<String>,
    pub full_answer_example: Option<String>,
    pub concise_answer_example: Option<String>,
    pub weak_answer_example: Option<String>,
    pub common_omissions: Vec<String>,
    pub marking_notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructedAnswerStepInput {
    pub label: Option<String>,
    pub answer_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructedAnswerEvaluationInput {
    pub student_id: i64,
    pub session_id: i64,
    pub question_id: i64,
    pub submitted_steps: Vec<ConstructedAnswerStepInput>,
    pub answer_text: Option<String>,
    pub response_time_ms: Option<i64>,
    pub confidence_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructedAnswerEvaluation {
    pub submission_id: i64,
    pub student_id: i64,
    pub session_id: i64,
    pub question_id: i64,
    pub rubric_id: i64,
    pub question_format: String,
    pub rubric_type: String,
    pub marks_awarded: i64,
    pub marks_possible: i64,
    pub step_quality_bp: BasisPoints,
    pub matched_steps: Vec<String>,
    pub omitted_steps: Vec<String>,
    pub extra_steps: Vec<String>,
    pub feedback: Vec<String>,
}

pub struct AnswerConstructionService<'a> {
    conn: &'a Connection,
}

impl<'a> AnswerConstructionService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn upsert_answer_rubric(&self, input: AnswerRubricInput) -> EcoachResult<AnswerRubric> {
        let steps_json = serde_json::to_string(&input.steps)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let mandatory_steps_json = serde_json::to_string(&input.mandatory_steps)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let common_omissions_json = serde_json::to_string(&input.common_omissions)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let rubric_type = normalize_rubric_type(&input.rubric_type);
        let total_marks = input.total_marks.max(1);

        let rubric_id = if let Some(rubric_id) = input.rubric_id {
            self.conn
                .execute(
                    "UPDATE answer_rubrics
                     SET question_id = ?1,
                         rubric_type = ?2,
                         total_marks = ?3,
                         steps_json = ?4,
                         mandatory_steps_json = ?5,
                         full_answer_example = ?6,
                         concise_answer_example = ?7,
                         weak_answer_example = ?8,
                         common_omissions_json = ?9,
                         marking_notes = ?10
                     WHERE id = ?11",
                    params![
                        input.question_id,
                        rubric_type,
                        total_marks,
                        steps_json,
                        mandatory_steps_json,
                        input.full_answer_example,
                        input.concise_answer_example,
                        input.weak_answer_example,
                        common_omissions_json,
                        input.marking_notes,
                        rubric_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            rubric_id
        } else if let Some(existing_id) = self.load_latest_rubric_id(input.question_id)? {
            self.conn
                .execute(
                    "UPDATE answer_rubrics
                     SET rubric_type = ?1,
                         total_marks = ?2,
                         steps_json = ?3,
                         mandatory_steps_json = ?4,
                         full_answer_example = ?5,
                         concise_answer_example = ?6,
                         weak_answer_example = ?7,
                         common_omissions_json = ?8,
                         marking_notes = ?9
                     WHERE id = ?10",
                    params![
                        rubric_type,
                        total_marks,
                        steps_json,
                        mandatory_steps_json,
                        input.full_answer_example,
                        input.concise_answer_example,
                        input.weak_answer_example,
                        common_omissions_json,
                        input.marking_notes,
                        existing_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            existing_id
        } else {
            self.conn
                .execute(
                    "INSERT INTO answer_rubrics (
                        question_id, rubric_type, total_marks, steps_json,
                        mandatory_steps_json, full_answer_example, concise_answer_example,
                        weak_answer_example, common_omissions_json, marking_notes
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        input.question_id,
                        rubric_type,
                        total_marks,
                        steps_json,
                        mandatory_steps_json,
                        input.full_answer_example,
                        input.concise_answer_example,
                        input.weak_answer_example,
                        common_omissions_json,
                        input.marking_notes,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.conn.last_insert_rowid()
        };

        self.load_rubric_by_id(rubric_id)?
            .ok_or_else(|| EcoachError::NotFound("rubric was not persisted".to_string()))
    }

    pub fn get_answer_rubric(&self, question_id: i64) -> EcoachResult<Option<AnswerRubric>> {
        self.conn
            .query_row(
                "SELECT id, question_id, rubric_type, total_marks, steps_json,
                        mandatory_steps_json, full_answer_example, concise_answer_example,
                        weak_answer_example, common_omissions_json, marking_notes, created_at
                 FROM answer_rubrics
                 WHERE question_id = ?1
                 ORDER BY id DESC
                 LIMIT 1",
                [question_id],
                map_rubric_row,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn evaluate_constructed_answer(
        &self,
        input: ConstructedAnswerEvaluationInput,
    ) -> EcoachResult<ConstructedAnswerEvaluation> {
        let rubric = self.get_answer_rubric(input.question_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!(
                "no answer rubric found for question {}",
                input.question_id
            ))
        })?;
        let question = self.load_question(input.question_id)?;
        let submitted_texts = build_submitted_texts(&input);
        let submitted_text_joined = submitted_texts.join("\n");

        let evaluation = evaluate_against_rubric(
            &rubric,
            &question.question_stem,
            &question.question_format,
            question.marks,
            &submitted_texts,
            submitted_text_joined.as_str(),
        );

        let submitted_steps_json = serde_json::to_string(&input.submitted_steps)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let matched_steps_json = serde_json::to_string(&evaluation.matched_steps)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let omitted_steps_json = serde_json::to_string(&evaluation.omitted_steps)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let extra_steps_json = serde_json::to_string(&evaluation.extra_steps)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let feedback_json = serde_json::to_string(&json!({
            "question_stem": question.question_stem,
            "question_format": question.question_format,
            "rubric_type": rubric.rubric_type,
            "feedback": evaluation.feedback,
            "matched_steps": evaluation.matched_steps,
            "omitted_steps": evaluation.omitted_steps,
            "extra_steps": evaluation.extra_steps,
            "response_time_ms": input.response_time_ms,
            "confidence_level": input.confidence_level,
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO step_submissions (
                    student_id, session_id, question_id, rubric_id,
                    submitted_steps_json, matched_steps_json, omitted_steps_json,
                    extra_steps_json, step_quality_bp, marks_awarded, marks_possible,
                    feedback_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    input.student_id,
                    input.session_id,
                    input.question_id,
                    rubric.id,
                    submitted_steps_json,
                    matched_steps_json,
                    omitted_steps_json,
                    extra_steps_json,
                    evaluation.step_quality_bp as i64,
                    evaluation.marks_awarded,
                    evaluation.marks_possible,
                    feedback_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let submission_id = self.conn.last_insert_rowid();

        MasteryMapService::new(self.conn)
            .refresh_mastery_map(input.student_id, question.subject_id)?;

        Ok(ConstructedAnswerEvaluation {
            submission_id,
            student_id: input.student_id,
            session_id: input.session_id,
            question_id: input.question_id,
            rubric_id: rubric.id,
            question_format: question.question_format,
            rubric_type: rubric.rubric_type,
            marks_awarded: evaluation.marks_awarded,
            marks_possible: evaluation.marks_possible,
            step_quality_bp: evaluation.step_quality_bp,
            matched_steps: evaluation.matched_steps,
            omitted_steps: evaluation.omitted_steps,
            extra_steps: evaluation.extra_steps,
            feedback: evaluation.feedback,
        })
    }

    fn load_latest_rubric_id(&self, question_id: i64) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT id FROM answer_rubrics
                 WHERE question_id = ?1
                 ORDER BY id DESC
                 LIMIT 1",
                [question_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_rubric_by_id(&self, rubric_id: i64) -> EcoachResult<Option<AnswerRubric>> {
        self.conn
            .query_row(
                "SELECT id, question_id, rubric_type, total_marks, steps_json,
                        mandatory_steps_json, full_answer_example, concise_answer_example,
                        weak_answer_example, common_omissions_json, marking_notes, created_at
                 FROM answer_rubrics
                 WHERE id = ?1",
                [rubric_id],
                map_rubric_row,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_question(&self, question_id: i64) -> EcoachResult<QuestionRow> {
        self.conn
            .query_row(
                "SELECT subject_id, topic_id, stem, question_format, marks
                 FROM questions
                 WHERE id = ?1",
                [question_id],
                |row| {
                    Ok(QuestionRow {
                        subject_id: row.get(0)?,
                        question_stem: row.get(2)?,
                        question_format: row.get(3)?,
                        marks: row.get(4)?,
                    })
                },
            )
            .map_err(|err| EcoachError::NotFound(format!("question {question_id}: {err}")))
    }
}

struct QuestionRow {
    subject_id: i64,
    question_stem: String,
    question_format: String,
    marks: i64,
}

struct EvaluationDraft {
    matched_steps: Vec<String>,
    omitted_steps: Vec<String>,
    extra_steps: Vec<String>,
    marks_awarded: i64,
    marks_possible: i64,
    step_quality_bp: BasisPoints,
    feedback: Vec<String>,
}

fn map_rubric_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AnswerRubric> {
    let steps_json: String = row.get(4)?;
    let mandatory_steps_json: Option<String> = row.get(5)?;
    let common_omissions_json: Option<String> = row.get(9)?;
    Ok(AnswerRubric {
        id: row.get(0)?,
        question_id: row.get(1)?,
        rubric_type: row.get(2)?,
        total_marks: row.get(3)?,
        steps: serde_json::from_str(&steps_json).unwrap_or_default(),
        mandatory_steps: serde_json::from_str(mandatory_steps_json.as_deref().unwrap_or("[]"))
            .unwrap_or_default(),
        full_answer_example: row.get(6)?,
        concise_answer_example: row.get(7)?,
        weak_answer_example: row.get(8)?,
        common_omissions: serde_json::from_str(common_omissions_json.as_deref().unwrap_or("[]"))
            .unwrap_or_default(),
        marking_notes: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn build_submitted_texts(input: &ConstructedAnswerEvaluationInput) -> Vec<String> {
    let mut texts = input
        .submitted_steps
        .iter()
        .map(|step| step.answer_text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();

    if let Some(answer_text) = input.answer_text.as_ref() {
        let trimmed = answer_text.trim();
        if !trimmed.is_empty() {
            texts.push(trimmed.to_string());
        }
    }

    texts
}

fn evaluate_against_rubric(
    rubric: &AnswerRubric,
    question_stem: &str,
    question_format: &str,
    question_marks: i64,
    submitted_texts: &[String],
    submitted_joined: &str,
) -> EvaluationDraft {
    let normalized_joined = normalize_text(submitted_joined);
    let mut used_submission_indices = HashSet::new();
    let mut matched_steps = Vec::new();
    let mut omitted_steps = Vec::new();
    let mut feedback = Vec::new();
    let mut marks_awarded = 0i64;

    if rubric.steps.is_empty() {
        let fallback =
            score_holistic_answer(rubric, question_stem, question_format, &normalized_joined);
        marks_awarded = fallback;
        if fallback > 0 {
            matched_steps.push("holistic_response".to_string());
            feedback.push("Holistic answer appears to cover the main idea.".to_string());
        } else {
            feedback.push(
                "Holistic answer does not yet demonstrate the expected structure.".to_string(),
            );
        }
    } else {
        for rubric_step in &rubric.steps {
            let mut best_match: Option<(usize, i64)> = None;

            for (index, submitted) in submitted_texts.iter().enumerate() {
                if used_submission_indices.contains(&index) {
                    continue;
                }

                let score = step_match_score(rubric_step, submitted, question_stem);
                if score >= 55 && best_match.map(|(_, best)| score > best).unwrap_or(true) {
                    best_match = Some((index, score));
                }
            }

            if let Some((index, _score)) = best_match {
                used_submission_indices.insert(index);
                matched_steps.push(rubric_step.label.clone());
                marks_awarded += rubric_step.marks.max(0);
                if let Some(feedback_line) = rubric_step.feedback.clone() {
                    feedback.push(feedback_line);
                }
            } else if rubric_step.required || rubric.mandatory_steps.contains(&rubric_step.label) {
                omitted_steps.push(rubric_step.label.clone());
            }
        }
    }

    let extra_steps = submitted_texts
        .iter()
        .enumerate()
        .filter_map(|(index, text)| {
            if used_submission_indices.contains(&index) {
                None
            } else {
                Some(text.clone())
            }
        })
        .collect::<Vec<_>>();

    if !omitted_steps.is_empty() {
        feedback.push(format!("Missing steps: {}", omitted_steps.join(", ")));
    }
    if !extra_steps.is_empty() {
        feedback.push(format!("Extra steps recorded: {}", extra_steps.join(" | ")));
    }
    if matched_steps.is_empty() && !submitted_texts.is_empty() {
        feedback.push("Answer was recorded but did not convincingly match the rubric.".to_string());
    } else if matched_steps.len() == rubric.steps.len() && !rubric.steps.is_empty() {
        feedback.push("All rubric steps were covered.".to_string());
    }

    let marks_possible = rubric.total_marks.max(question_marks).max(1);
    let marks_awarded = marks_awarded.min(marks_possible);
    let step_quality_bp = clamp_bp((marks_awarded * 10_000) / marks_possible);

    EvaluationDraft {
        matched_steps,
        omitted_steps,
        extra_steps,
        marks_awarded,
        marks_possible,
        step_quality_bp,
        feedback,
    }
}

fn score_holistic_answer(
    rubric: &AnswerRubric,
    question_stem: &str,
    question_format: &str,
    submitted_joined: &str,
) -> i64 {
    let mut score = 0i64;
    let submitted = submitted_joined.trim();
    if submitted.is_empty() {
        return 0;
    }

    let examples = [
        rubric.full_answer_example.as_deref(),
        rubric.concise_answer_example.as_deref(),
        rubric.weak_answer_example.as_deref(),
    ];
    for example in examples.into_iter().flatten() {
        let example_norm = normalize_text(example);
        if example_norm.is_empty() {
            continue;
        }

        let overlap = overlap_score(&example_norm, submitted);
        if overlap >= 75 {
            score = rubric.total_marks;
            break;
        } else if overlap >= 45 {
            score = score.max((rubric.total_marks * 2) / 3);
        } else if overlap >= 20 {
            score = score.max((rubric.total_marks + 1) / 2);
        }
    }

    if score == 0 {
        let stem_overlap = overlap_score(&normalize_text(question_stem), submitted);
        let format_bonus = if matches!(question_format, "short_answer" | "numeric") {
            10
        } else {
            0
        };
        if stem_overlap >= 40 {
            score = (rubric.total_marks / 2).max(1);
        } else if submitted.len() > 8 {
            score = (rubric.total_marks / 3).max(1);
        }
        score = (score + format_bonus).min(rubric.total_marks);
    }

    score.min(rubric.total_marks)
}

fn step_match_score(step: &AnswerRubricStep, submitted: &str, question_stem: &str) -> i64 {
    let submitted_norm = normalize_text(submitted);
    let expected_norm = normalize_text(&step.expected_answer);
    let label_norm = normalize_text(&step.label);
    let stem_norm = normalize_text(question_stem);

    let mut score = 0i64;
    if !label_norm.is_empty() && submitted_norm.contains(&label_norm) {
        score += 20;
    }
    if !expected_norm.is_empty() {
        if submitted_norm.contains(&expected_norm) || expected_norm.contains(&submitted_norm) {
            score += 40;
        }
        score += keyword_match_score(&submitted_norm, &step.keywords).min(30);
    }
    if !stem_norm.is_empty() && submitted_norm.contains(&stem_norm) {
        score += 10;
    }
    if submitted_norm == expected_norm && !submitted_norm.is_empty() {
        score += 20;
    }
    score.min(100)
}

fn keyword_match_score(submitted_norm: &str, keywords: &[String]) -> i64 {
    if keywords.is_empty() {
        return 0;
    }

    let mut hits = 0i64;
    for keyword in keywords {
        let normalized_keyword = normalize_text(keyword);
        if !normalized_keyword.is_empty() && submitted_norm.contains(&normalized_keyword) {
            hits += 1;
        }
    }

    ((hits * 40) / keywords.len() as i64).min(40)
}

fn overlap_score(reference_norm: &str, submitted: &str) -> i64 {
    let reference_terms: Vec<&str> = reference_norm
        .split_whitespace()
        .filter(|term| term.len() > 2)
        .collect();
    if reference_terms.is_empty() {
        return 0;
    }

    let mut hits = 0i64;
    for term in &reference_terms {
        if submitted.contains(term) {
            hits += 1;
        }
    }

    ((hits * 100) / reference_terms.len() as i64).min(100)
}

fn normalize_text(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_rubric_type(input: &str) -> String {
    match input.trim().to_ascii_lowercase().as_str() {
        "holistic" => "holistic".to_string(),
        "criterion" => "criterion".to_string(),
        _ => "step_based".to_string(),
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
    fn answer_construction_service_upserts_and_evaluates() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_id: i64 = conn
            .query_row(
                "SELECT id FROM topics WHERE subject_id = ?1 ORDER BY id ASC LIMIT 1",
                [subject_id],
                |row| row.get(0),
            )
            .expect("topic should exist");
        let question_id = seed_question(&conn, subject_id, topic_id);
        let session_id = seed_session(&conn, subject_id);

        let service = AnswerConstructionService::new(&conn);
        let rubric = service
            .upsert_answer_rubric(AnswerRubricInput {
                rubric_id: None,
                question_id,
                rubric_type: "step_based".into(),
                total_marks: 4,
                steps: vec![
                    AnswerRubricStep {
                        label: "step_one".into(),
                        expected_answer: "Identify the constant term".into(),
                        keywords: vec!["constant".into(), "term".into()],
                        marks: 2,
                        required: true,
                        feedback: Some("Name the constant term clearly.".into()),
                    },
                    AnswerRubricStep {
                        label: "step_two".into(),
                        expected_answer: "Substitute into the formula".into(),
                        keywords: vec!["substitute".into(), "formula".into()],
                        marks: 2,
                        required: true,
                        feedback: Some("Show the substitution step.".into()),
                    },
                ],
                mandatory_steps: vec!["step_one".into(), "step_two".into()],
                full_answer_example: Some(
                    "Identify the constant term and substitute into the formula".into(),
                ),
                concise_answer_example: Some("constant term, substitute".into()),
                weak_answer_example: Some("substitute".into()),
                common_omissions: vec!["forgot substitution".into()],
                marking_notes: Some(
                    "Award partial credit only when both steps are visible.".into(),
                ),
            })
            .expect("rubric should upsert");
        assert_eq!(rubric.total_marks, 4);

        let retrieved = service
            .get_answer_rubric(question_id)
            .expect("rubric should load")
            .expect("rubric should exist");
        assert_eq!(retrieved.id, rubric.id);

        let evaluation = service
            .evaluate_constructed_answer(ConstructedAnswerEvaluationInput {
                student_id: 1,
                session_id,
                question_id,
                submitted_steps: vec![
                    ConstructedAnswerStepInput {
                        label: Some("step_one".into()),
                        answer_text: "The constant term is 5".into(),
                    },
                    ConstructedAnswerStepInput {
                        label: Some("step_two".into()),
                        answer_text: "Substitute the constant into the formula".into(),
                    },
                ],
                answer_text: Some(
                    "Identify the constant term then substitute into the formula".into(),
                ),
                response_time_ms: Some(42_000),
                confidence_level: Some("confident".into()),
            })
            .expect("evaluation should succeed");

        assert_eq!(evaluation.rubric_id, rubric.id);
        assert_eq!(evaluation.marks_possible, 4);
        assert!(evaluation.marks_awarded >= 2);
        assert!(!evaluation.matched_steps.is_empty());

        let mastery_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM mastery_map_nodes WHERE student_id = ?1 AND subject_id = ?2",
                params![1, subject_id],
                |row| row.get(0),
            )
            .expect("mastery map should refresh");
        assert!(mastery_rows > 0);
    }

    fn seed_student(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'student', 'Ada', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (1, '[\"MATH\"]', 60)",
            [],
        )
        .expect("student profile should insert");
    }

    fn seed_session(conn: &Connection, subject_id: i64) -> i64 {
        conn.execute(
            "INSERT INTO sessions (
                student_id, session_type, subject_id, topic_ids, question_count, duration_minutes,
                is_timed, status, started_at, total_questions, answered_questions, correct_questions,
                accuracy_score, avg_response_time_ms
             ) VALUES (1, 'practice', ?1, '[1]', 1, 30, 1, 'active', datetime('now'), 1, 0, 0, 0, 0)",
            [subject_id],
        )
        .expect("session should insert");
        conn.last_insert_rowid()
    }

    fn seed_question(conn: &Connection, subject_id: i64, topic_id: i64) -> i64 {
        conn.execute(
            "INSERT INTO questions (
                subject_id, topic_id, stem, question_format, marks, is_active
             ) VALUES (?1, ?2, 'Show your reasoning', 'short_answer', 4, 1)",
            params![subject_id, topic_id],
        )
        .expect("question should insert");
        conn.last_insert_rowid()
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
