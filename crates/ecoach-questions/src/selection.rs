use ecoach_substrate::{EcoachError, EcoachResult, from_bp};
use rusqlite::Connection;

use crate::models::{Question, QuestionSelectionRequest, SelectedQuestion};

pub struct QuestionSelector<'a> {
    conn: &'a Connection,
}

impl<'a> QuestionSelector<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn select_questions(
        &self,
        request: &QuestionSelectionRequest,
    ) -> EcoachResult<Vec<SelectedQuestion>> {
        let candidates = self.get_candidate_pool(request)?;
        let mut scored = candidates
            .into_iter()
            .map(|question| {
                let fit_score = self.compute_candidate_fit(&question, request);
                SelectedQuestion {
                    question,
                    fit_score,
                }
            })
            .collect::<Vec<_>>();

        scored.sort_by(|left, right| {
            right
                .fit_score
                .partial_cmp(&left.fit_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(request.target_question_count);
        Ok(scored)
    }

    fn get_candidate_pool(
        &self,
        request: &QuestionSelectionRequest,
    ) -> EcoachResult<Vec<Question>> {
        if request.topic_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = request
            .topic_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT id, subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                    explanation_text, difficulty_level, estimated_time_seconds, marks, primary_skill_id
             FROM questions
             WHERE is_active = 1 AND subject_id = ?1 AND topic_id IN ({})
             ORDER BY updated_at DESC, id DESC",
            placeholders
        );

        let mut params_vec: Vec<rusqlite::types::Value> =
            Vec::with_capacity(request.topic_ids.len() + 1);
        params_vec.push(request.subject_id.into());
        for topic_id in &request.topic_ids {
            params_vec.push((*topic_id).into());
        }

        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
                Ok(Question {
                    id: row.get(0)?,
                    subject_id: row.get(1)?,
                    topic_id: row.get(2)?,
                    subtopic_id: row.get(3)?,
                    family_id: row.get(4)?,
                    stem: row.get(5)?,
                    question_format: row.get(6)?,
                    explanation_text: row.get(7)?,
                    difficulty_level: row.get(8)?,
                    estimated_time_seconds: row.get(9)?,
                    marks: row.get(10)?,
                    primary_skill_id: row.get(11)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut questions = Vec::new();
        for row in rows {
            let question = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if !request.recently_seen_question_ids.contains(&question.id) {
                questions.push(question);
            }
        }
        Ok(questions)
    }

    fn compute_candidate_fit(
        &self,
        question: &Question,
        request: &QuestionSelectionRequest,
    ) -> f64 {
        let scope_match = if request.topic_ids.contains(&question.topic_id) {
            1.0
        } else {
            0.0
        };
        let difficulty_fit = request
            .target_difficulty
            .map(|target| {
                1.0 - (from_bp(question.difficulty_level).abs_diff(from_bp(target))).min(1.0)
            })
            .unwrap_or(0.75);
        let weakness_match = if request.weakness_topic_ids.contains(&question.topic_id) {
            1.0
        } else {
            0.4
        };
        let variety_bonus = if request.timed && question.estimated_time_seconds <= 45 {
            0.9
        } else {
            0.6
        };
        let recency_penalty = if request.recently_seen_question_ids.contains(&question.id) {
            1.0
        } else {
            0.0
        };

        0.25 * scope_match
            + 0.20 * difficulty_fit
            + 0.20 * weakness_match
            + 0.15 * variety_bonus
            + 0.10 * (1.0 - recency_penalty)
            + 0.10 * if request.timed { 1.0 } else { 0.7 }
    }
}

trait AbsDiff {
    fn abs_diff(self, other: Self) -> Self;
}

impl AbsDiff for f64 {
    fn abs_diff(self, other: Self) -> Self {
        (self - other).abs()
    }
}
