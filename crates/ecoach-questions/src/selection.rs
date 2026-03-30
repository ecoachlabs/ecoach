use std::collections::BTreeMap;

use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, from_bp};
use rusqlite::{Connection, params};

use crate::models::{Question, QuestionSelectionRequest, SelectedQuestion};

pub struct QuestionSelector<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone)]
struct CandidateContext {
    question: Question,
    family_quality_score: i64,
    family_calibration_score: i64,
    family_health_status: String,
    family_recurrence_score: i64,
    family_replacement_score: i64,
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
        let scored = candidates
            .into_iter()
            .map(|candidate| {
                let fit_score = self.compute_candidate_fit(&candidate, request);
                SelectedQuestion {
                    question: candidate.question,
                    fit_score,
                }
            })
            .collect::<Vec<_>>();

        Ok(self.select_diverse_mix(scored, request.target_question_count))
    }

    fn get_candidate_pool(
        &self,
        request: &QuestionSelectionRequest,
    ) -> EcoachResult<Vec<CandidateContext>> {
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
            "SELECT q.id, q.subject_id, q.topic_id, q.subtopic_id, q.family_id, q.stem, q.question_format,
                    q.explanation_text, q.difficulty_level, q.estimated_time_seconds, q.marks, q.primary_skill_id,
                    COALESCE(qfh.quality_score, 5400),
                    COALESCE(qfh.calibration_score, 5200),
                    COALESCE(qfh.health_status, 'warming'),
                    COALESCE(qfa.recurrence_score, 0),
                    COALESCE(qfa.replacement_score, 0)
             FROM questions q
             LEFT JOIN question_family_health qfh ON qfh.family_id = q.family_id
             LEFT JOIN question_family_analytics qfa ON qfa.family_id = q.family_id
             WHERE q.is_active = 1 AND q.subject_id = ?1 AND q.topic_id IN ({})
             ORDER BY q.updated_at DESC, q.id DESC",
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
                Ok(CandidateContext {
                    question: Question {
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
                    },
                    family_quality_score: row.get(12)?,
                    family_calibration_score: row.get(13)?,
                    family_health_status: row.get(14)?,
                    family_recurrence_score: row.get(15)?,
                    family_replacement_score: row.get(16)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut questions = Vec::new();
        for row in rows {
            let question = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if !request
                .recently_seen_question_ids
                .contains(&question.question.id)
            {
                questions.push(question);
            }
        }
        Ok(questions)
    }

    fn compute_candidate_fit(
        &self,
        candidate: &CandidateContext,
        request: &QuestionSelectionRequest,
    ) -> f64 {
        let question = &candidate.question;
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
        let family_quality = (candidate.family_quality_score as f64 / 10_000.0).clamp(0.0, 1.0);
        let family_calibration =
            (candidate.family_calibration_score as f64 / 10_000.0).clamp(0.0, 1.0);
        let exam_pressure = ((candidate
            .family_recurrence_score
            .max(candidate.family_replacement_score)) as f64
            / 10_000.0)
            .clamp(0.0, 1.0);
        let family_state_bonus = match candidate.family_health_status.as_str() {
            "gold" => 1.0,
            "active" => 0.82,
            "warming" => 0.65,
            "fragile" => 0.45,
            "missing" => 0.25,
            _ => 0.55,
        };
        let remediation_bias = if request.weakness_topic_ids.contains(&question.topic_id) {
            match candidate.family_health_status.as_str() {
                "fragile" => 1.0,
                "warming" => 0.86,
                "missing" => 0.72,
                "active" => 0.64,
                "gold" => 0.48,
                _ => 0.7,
            }
        } else {
            family_state_bonus
        };
        let recency_penalty = if request.recently_seen_question_ids.contains(&question.id) {
            1.0
        } else {
            0.0
        };

        0.20 * scope_match
            + 0.18 * difficulty_fit
            + 0.18 * weakness_match
            + 0.12 * variety_bonus
            + 0.10 * family_quality
            + 0.08 * family_calibration
            + 0.08 * exam_pressure
            + 0.04 * remediation_bias
            + 0.02 * if request.timed { 1.0 } else { 0.7 }
            + 0.10 * (1.0 - recency_penalty)
    }

    fn select_diverse_mix(
        &self,
        mut scored: Vec<SelectedQuestion>,
        target_count: usize,
    ) -> Vec<SelectedQuestion> {
        let target_count = target_count.max(1);
        let mut selected = Vec::new();
        let mut family_counts = BTreeMap::new();

        while !scored.is_empty() && selected.len() < target_count {
            let mut best_index = 0usize;
            let mut best_score = f64::MIN;

            for (index, candidate) in scored.iter().enumerate() {
                let family_repeat_penalty = candidate
                    .question
                    .family_id
                    .and_then(|family_id| family_counts.get(&family_id).copied())
                    .unwrap_or(0) as f64
                    * 0.14;
                let similarity_penalty = self
                    .max_similarity_to_selected(candidate.question.id, &selected)
                    .map(|score| score as f64 / 10_000.0 * 0.20)
                    .unwrap_or(0.0);
                let adjusted_score =
                    candidate.fit_score - family_repeat_penalty - similarity_penalty;
                if adjusted_score > best_score {
                    best_score = adjusted_score;
                    best_index = index;
                }
            }

            let mut winner = scored.swap_remove(best_index);
            winner.fit_score = winner.fit_score.max(best_score);
            if let Some(family_id) = winner.question.family_id {
                *family_counts.entry(family_id).or_insert(0usize) += 1;
            }
            selected.push(winner);
        }

        selected
    }

    fn max_similarity_to_selected(
        &self,
        question_id: i64,
        selected: &[SelectedQuestion],
    ) -> Option<i64> {
        if selected.is_empty() {
            return None;
        }

        let mut best = None;
        for item in selected {
            let similarity = self
                .conn
                .query_row(
                    "SELECT MAX(similarity_score)
                     FROM question_graph_edges
                     WHERE (from_question_id = ?1 AND to_question_id = ?2)
                        OR (from_question_id = ?2 AND to_question_id = ?1)",
                    params![question_id, item.question.id],
                    |row| row.get::<_, Option<i64>>(0),
                )
                .ok()
                .flatten()
                .map(clamp_bp)
                .unwrap_or(0);
            best = Some(best.map_or(similarity, |current: BasisPoints| current.max(similarity)));
        }
        best.map(|v| v as i64)
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

#[cfg(test)]
mod tests {
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn selector_prefers_exam_pressure_and_diverse_families() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");

        seed_selection_fixture(&conn);

        let selector = QuestionSelector::new(&conn);
        let selected = selector
            .select_questions(&QuestionSelectionRequest {
                subject_id: 1,
                topic_ids: vec![10],
                target_question_count: 2,
                target_difficulty: Some(5_200),
                weakness_topic_ids: vec![10],
                recently_seen_question_ids: Vec::new(),
                timed: true,
            })
            .expect("selection should succeed");

        assert_eq!(selected.len(), 2);
        assert_ne!(
            selected[0].question.family_id,
            selected[1].question.family_id
        );
        assert_eq!(selected[0].question.family_id, Some(100));
    }

    fn seed_selection_fixture(conn: &Connection) {
        conn.execute(
            "INSERT INTO curriculum_versions (id, name, version_label) VALUES (1, 'Test', 'v1')",
            [],
        )
        .expect("curriculum version should insert");
        conn.execute(
            "INSERT INTO subjects (id, curriculum_version_id, code, name) VALUES (1, 1, 'MATH', 'Math')",
            [],
        )
        .expect("subject should insert");
        conn.execute(
            "INSERT INTO topics (id, subject_id, code, name) VALUES (10, 1, 'ALG', 'Algebra')",
            [],
        )
        .expect("topic should insert");
        conn.execute(
            "INSERT INTO question_families (id, family_code, family_name, subject_id, topic_id)
             VALUES (100, 'ALG_CORE', 'Algebra Core', 1, 10)",
            [],
        )
        .expect("first family should insert");
        conn.execute(
            "INSERT INTO question_families (id, family_code, family_name, subject_id, topic_id)
             VALUES (200, 'ALG_TRAP', 'Algebra Trap', 1, 10)",
            [],
        )
        .expect("second family should insert");

        for (id, family_id, stem, difficulty, time_seconds) in [
            (
                1000_i64,
                100_i64,
                "Solve for x in a simple equation",
                5_100_i64,
                35_i64,
            ),
            (
                1001,
                100,
                "Solve for x in a simple equation again",
                5_300,
                38,
            ),
            (
                2000,
                200,
                "Spot the trap in the algebra simplification",
                5_000,
                32,
            ),
        ] {
            conn.execute(
                "INSERT INTO questions (
                    id, subject_id, topic_id, family_id, stem, question_format,
                    difficulty_level, estimated_time_seconds, marks, is_active
                 ) VALUES (?1, 1, 10, ?2, ?3, 'mcq', ?4, ?5, 1, 1)",
                params![id, family_id, stem, difficulty, time_seconds],
            )
            .expect("question should insert");
        }

        conn.execute(
            "INSERT INTO question_family_health (
                family_id, total_instances, generated_instances, active_instances,
                recent_attempts, recent_correct_attempts, avg_response_time_ms,
                misconception_hit_count, freshness_score, calibration_score,
                quality_score, health_status, updated_at
             ) VALUES (100, 2, 0, 2, 8, 6, 26000, 1, 7600, 7400, 7800, 'active', datetime('now'))",
            [],
        )
        .expect("first family health should insert");
        conn.execute(
            "INSERT INTO question_family_health (
                family_id, total_instances, generated_instances, active_instances,
                recent_attempts, recent_correct_attempts, avg_response_time_ms,
                misconception_hit_count, freshness_score, calibration_score,
                quality_score, health_status, updated_at
             ) VALUES (200, 1, 0, 1, 3, 1, 51000, 2, 5200, 4300, 4100, 'fragile', datetime('now'))",
            [],
        )
        .expect("second family health should insert");
        conn.execute(
            "INSERT INTO question_family_analytics (family_id, recurrence_score, coappearance_score, replacement_score)
             VALUES (100, 9000, 6200, 7800)",
            [],
        )
        .expect("first family analytics should insert");
        conn.execute(
            "INSERT INTO question_family_analytics (family_id, recurrence_score, coappearance_score, replacement_score)
             VALUES (200, 4500, 4000, 8600)",
            [],
        )
        .expect("second family analytics should insert");
        conn.execute(
            "INSERT INTO question_graph_edges (
                from_question_id, to_question_id, relation_type, similarity_score, rationale
             ) VALUES (1000, 1001, 'isomorphic_cluster', 9800, 'near duplicate core stem')",
            [],
        )
        .expect("graph edge should insert");
    }
}
