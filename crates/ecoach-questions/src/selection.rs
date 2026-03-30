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
    family_coappearance_score: i64,
    family_replacement_score: i64,
    family_paper_count: i64,
    family_last_seen_year: Option<i64>,
    subject_latest_exam_year: i64,
    subject_year_span: i64,
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

        let (subject_latest_exam_year, subject_year_span) =
            self.load_subject_year_bounds(request.subject_id)?;

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
                    COALESCE(qfa.coappearance_score, 0),
                    COALESCE(qfa.replacement_score, 0),
                    COALESCE((
                        SELECT COUNT(DISTINCT ppql.paper_id)
                        FROM past_paper_question_links ppql
                        INNER JOIN questions history_q ON history_q.id = ppql.question_id
                        WHERE history_q.family_id = q.family_id
                    ), 0) AS paper_count,
                    (
                        SELECT MAX(pps.exam_year)
                        FROM past_paper_question_links ppql
                        INNER JOIN questions history_q ON history_q.id = ppql.question_id
                        INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                        WHERE history_q.family_id = q.family_id
                    ) AS last_seen_year
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
                    family_coappearance_score: row.get(16)?,
                    family_replacement_score: row.get(17)?,
                    family_paper_count: row.get(18)?,
                    family_last_seen_year: row.get(19)?,
                    subject_latest_exam_year,
                    subject_year_span,
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
        let inverse_pressure = bp_to_ratio(composite_inverse_pressure(
            candidate.family_recurrence_score as BasisPoints,
            candidate.family_coappearance_score as BasisPoints,
            candidate.family_replacement_score as BasisPoints,
        ) as i64);
        let comeback_pressure = bp_to_ratio(compute_comeback_pressure(
            candidate.family_recurrence_score as BasisPoints,
            candidate.family_coappearance_score as BasisPoints,
            candidate.family_replacement_score as BasisPoints,
            candidate.family_paper_count,
            candidate.family_last_seen_year,
            candidate.subject_latest_exam_year,
            candidate.subject_year_span,
        ) as i64);
        let exam_pressure = (0.55 * inverse_pressure + 0.45 * comeback_pressure).clamp(0.0, 1.0);
        let paper_history_bonus =
            (candidate.family_paper_count.min(6) as f64 / 6.0).clamp(0.0, 1.0);
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
        let pressure_alignment = if request.timed {
            (0.60 * exam_pressure + 0.40 * family_state_bonus).clamp(0.0, 1.0)
        } else if request.weakness_topic_ids.contains(&question.topic_id) {
            (0.55 * remediation_bias + 0.45 * comeback_pressure).clamp(0.0, 1.0)
        } else {
            (0.65 * family_state_bonus + 0.35 * exam_pressure).clamp(0.0, 1.0)
        };

        0.20 * scope_match
            + 0.17 * difficulty_fit
            + 0.18 * weakness_match
            + 0.11 * variety_bonus
            + 0.10 * family_quality
            + 0.07 * family_calibration
            + 0.09 * inverse_pressure
            + 0.05 * comeback_pressure
            + 0.04 * remediation_bias
            + 0.03 * pressure_alignment
            + 0.10 * (1.0 - recency_penalty)
            + 0.06 * paper_history_bonus
    }

    fn load_subject_year_bounds(&self, subject_id: i64) -> EcoachResult<(i64, i64)> {
        let latest_subject_year = self
            .conn
            .query_row(
                "SELECT MAX(exam_year) FROM past_paper_sets WHERE subject_id = ?1",
                [subject_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(0);
        let earliest_subject_year = self
            .conn
            .query_row(
                "SELECT MIN(exam_year) FROM past_paper_sets WHERE subject_id = ?1",
                [subject_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(latest_subject_year);

        Ok((latest_subject_year, (latest_subject_year - earliest_subject_year).max(1)))
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

fn bp_to_ratio(value: i64) -> f64 {
    (value as f64 / 10_000.0).clamp(0.0, 1.0)
}

fn composite_inverse_pressure(
    recurrence_score: BasisPoints,
    coappearance_score: BasisPoints,
    replacement_score: BasisPoints,
) -> BasisPoints {
    clamp_bp(
        (0.45 * replacement_score as f64
            + 0.30 * coappearance_score as f64
            + 0.25 * recurrence_score as f64)
            .round() as i64,
    ) as BasisPoints
}

fn compute_comeback_pressure(
    recurrence_score: BasisPoints,
    coappearance_score: BasisPoints,
    replacement_score: BasisPoints,
    paper_count: i64,
    last_seen_year: Option<i64>,
    latest_subject_year: i64,
    year_span: i64,
) -> BasisPoints {
    let dormant_years =
        latest_subject_year.saturating_sub(last_seen_year.unwrap_or(latest_subject_year));
    let dormant_score = scale_score(dormant_years, year_span.max(1));
    let paper_breadth_score = scale_score(paper_count, 6);
    let historical_strength_score = clamp_bp(
        ((0.60 * recurrence_score as f64)
            + (0.25 * coappearance_score as f64)
            + (0.15 * paper_breadth_score as f64))
            .round() as i64,
    );

    clamp_bp(
        ((f64::from(historical_strength_score) * f64::from(dormant_score)) / 10_000.0).round()
            as i64
            + (i64::from(replacement_score) / 4),
    ) as BasisPoints
}

fn scale_score(value: i64, max_value: i64) -> BasisPoints {
    if value <= 0 || max_value <= 0 {
        0
    } else {
        clamp_bp(((value as f64 / max_value as f64) * 10_000.0).round() as i64) as BasisPoints
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn selector_prefers_exam_pressure_and_diverse_families() {
        let conn = Connection::open_in_memory().expect("in-memory db should open");
        seed_selection_schema(&conn);
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

    #[test]
    fn selector_surfaces_comeback_pressure_for_timed_selection() {
        let conn = Connection::open_in_memory().expect("in-memory db should open");
        seed_selection_schema(&conn);
        seed_selection_fixture(&conn);
        conn.execute(
            "UPDATE question_family_health
             SET health_status = 'active', quality_score = 7200, calibration_score = 7000
             WHERE family_id = 200",
            [],
        )
        .expect("second family health should update");
        conn.execute(
            "UPDATE question_family_analytics
             SET recurrence_score = 7200, coappearance_score = 7100, replacement_score = 9300
             WHERE family_id = 200",
            [],
        )
        .expect("second family analytics should update");
        conn.execute(
            "INSERT INTO past_paper_sets (id, subject_id, exam_year, title)
             VALUES (10, 1, 2021, 'Paper 2021'), (11, 1, 2024, 'Paper 2024')",
            [],
        )
        .expect("past paper sets should insert");
        conn.execute(
            "INSERT INTO past_paper_question_links (paper_id, question_id, section_label, question_number)
             VALUES (10, 2000, 'A', '1'), (11, 1000, 'A', '2')",
            [],
        )
        .expect("past paper links should insert");

        let selector = QuestionSelector::new(&conn);
        let selected = selector
            .select_questions(&QuestionSelectionRequest {
                subject_id: 1,
                topic_ids: vec![10],
                target_question_count: 1,
                target_difficulty: Some(5_100),
                weakness_topic_ids: vec![10],
                recently_seen_question_ids: Vec::new(),
                timed: true,
            })
            .expect("selection should succeed");

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].question.family_id, Some(200));
    }

    fn seed_selection_schema(conn: &Connection) {
        for sql in [
            "CREATE TABLE curriculum_versions (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                version_label TEXT NOT NULL
            )",
            "CREATE TABLE subjects (
                id INTEGER PRIMARY KEY,
                curriculum_version_id INTEGER NOT NULL,
                code TEXT NOT NULL,
                name TEXT NOT NULL
            )",
            "CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                code TEXT NOT NULL,
                name TEXT NOT NULL
            )",
            "CREATE TABLE question_families (
                id INTEGER PRIMARY KEY,
                family_code TEXT NOT NULL,
                family_name TEXT NOT NULL,
                subject_id INTEGER NOT NULL,
                topic_id INTEGER
            )",
            "CREATE TABLE questions (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                subtopic_id INTEGER,
                family_id INTEGER,
                stem TEXT NOT NULL,
                question_format TEXT NOT NULL,
                explanation_text TEXT,
                difficulty_level INTEGER NOT NULL,
                estimated_time_seconds INTEGER NOT NULL,
                marks INTEGER NOT NULL,
                primary_skill_id INTEGER,
                is_active INTEGER NOT NULL DEFAULT 1,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE TABLE question_family_health (
                family_id INTEGER PRIMARY KEY,
                total_instances INTEGER NOT NULL,
                generated_instances INTEGER NOT NULL,
                active_instances INTEGER NOT NULL,
                recent_attempts INTEGER NOT NULL,
                recent_correct_attempts INTEGER NOT NULL,
                avg_response_time_ms INTEGER NOT NULL,
                misconception_hit_count INTEGER NOT NULL,
                freshness_score INTEGER NOT NULL,
                calibration_score INTEGER NOT NULL,
                quality_score INTEGER NOT NULL,
                health_status TEXT NOT NULL,
                updated_at TEXT
            )",
            "CREATE TABLE question_family_analytics (
                family_id INTEGER PRIMARY KEY,
                recurrence_score INTEGER NOT NULL,
                coappearance_score INTEGER NOT NULL,
                replacement_score INTEGER NOT NULL
            )",
            "CREATE TABLE question_graph_edges (
                from_question_id INTEGER NOT NULL,
                to_question_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                similarity_score INTEGER NOT NULL,
                rationale TEXT
            )",
            "CREATE TABLE past_paper_sets (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                exam_year INTEGER NOT NULL,
                title TEXT NOT NULL
            )",
            "CREATE TABLE past_paper_question_links (
                id INTEGER PRIMARY KEY,
                paper_id INTEGER NOT NULL,
                question_id INTEGER NOT NULL,
                section_label TEXT,
                question_number TEXT
            )",
        ] {
            conn.execute(sql, []).expect("schema statement should apply");
        }
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
        conn.execute(
            "INSERT INTO past_paper_sets (id, subject_id, exam_year, title)
             VALUES (1, 1, 2022, 'Paper 2022'), (2, 1, 2025, 'Paper 2025')",
            [],
        )
        .expect("past paper sets should insert");
        conn.execute(
            "INSERT INTO past_paper_question_links (paper_id, question_id, section_label, question_number)
             VALUES (1, 1000, 'A', '1'), (2, 1001, 'A', '3'), (1, 2000, 'B', '2')",
            [],
        )
        .expect("past paper question links should insert");
    }
}
