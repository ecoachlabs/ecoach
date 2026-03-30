use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::forecast_engine::ForecastEngine;
use crate::pattern_miner::PatternMiner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationResult {
    pub subject_id: i64,
    pub holdout_year: i64,
    pub brier_score: BasisPoints,
    pub coverage_accuracy_bp: BasisPoints,
    pub topics_evaluated: i64,
}

pub struct CalibrationEngine<'a> {
    conn: &'a Connection,
}

impl<'a> CalibrationEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Backtest: hold out one year's paper, compute a forecast from remaining years,
    /// then compare predicted topic probabilities against actual topic presence
    /// in the held-out year.
    pub fn backtest_year(
        &self,
        subject_id: i64,
        holdout_year: i64,
    ) -> EcoachResult<CalibrationResult> {
        // Get the actual topics that appeared in the holdout year
        let actual_topics = self.load_topics_in_year(subject_id, holdout_year)?;
        if actual_topics.is_empty() {
            return Err(EcoachError::Validation(
                "no questions found for the holdout year".into(),
            ));
        }

        // Get all topics that appear in non-holdout years
        let all_topics = self.load_all_past_paper_topics(subject_id)?;
        if all_topics.is_empty() {
            return Err(EcoachError::Validation(
                "no past paper data outside holdout year".into(),
            ));
        }

        // Build a forecast from non-holdout data using the pattern miner
        let miner = PatternMiner::new(self.conn);
        let presence = miner.mine_topic_presence(subject_id)?;

        // Filter to exclude holdout year data
        let filtered: Vec<_> = presence
            .iter()
            .map(|tp| {
                let years_without_holdout: Vec<i64> = tp
                    .years_present
                    .iter()
                    .filter(|&&y| y != holdout_year)
                    .copied()
                    .collect();
                let questions_without_holdout = tp.total_questions
                    - tp.years_present.iter().filter(|&&y| y == holdout_year).count() as i64;
                (tp.topic_id, years_without_holdout, questions_without_holdout.max(0))
            })
            .filter(|(_, years, _)| !years.is_empty())
            .collect();

        // Simple frequency-based prediction for each topic
        let max_freq = filtered.iter().map(|(_, _, q)| *q).max().unwrap_or(1).max(1);
        let mut brier_sum: f64 = 0.0;
        let mut coverage_hits = 0i64;
        let mut topics_evaluated = 0i64;

        for &topic_id in &all_topics {
            let predicted_prob = filtered
                .iter()
                .find(|(tid, _, _)| *tid == topic_id)
                .map(|(_, _, q)| *q as f64 / max_freq as f64)
                .unwrap_or(0.0);

            let actual = if actual_topics.contains(&topic_id) {
                1.0
            } else {
                0.0
            };

            // Brier score component: (predicted - actual)^2
            brier_sum += (predicted_prob - actual).powi(2);
            topics_evaluated += 1;

            // Coverage: did we predict above 0.3 and it appeared?
            if predicted_prob >= 0.3 && actual == 1.0 {
                coverage_hits += 1;
            }
        }

        let brier_score = if topics_evaluated > 0 {
            // Lower is better; invert to BasisPoints (10000 = perfect, 0 = worst)
            let raw_brier = brier_sum / topics_evaluated as f64;
            clamp_bp(((1.0 - raw_brier) * 10_000.0).round() as i64)
        } else {
            5000
        };

        let actual_count = actual_topics.len() as i64;
        let coverage_accuracy = if actual_count > 0 {
            clamp_bp(((coverage_hits as f64 / actual_count as f64) * 10_000.0).round() as i64)
        } else {
            0
        };

        // Persist
        self.conn
            .execute(
                "INSERT INTO forecast_calibration_runs
                    (subject_id, holdout_year, brier_score, coverage_accuracy_bp, topics_evaluated)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![subject_id, holdout_year, brier_score as i64, coverage_accuracy as i64, topics_evaluated],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(CalibrationResult {
            subject_id,
            holdout_year,
            brier_score,
            coverage_accuracy_bp: coverage_accuracy,
            topics_evaluated,
        })
    }

    /// List all calibration results for a subject.
    pub fn list_calibration_results(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Vec<CalibrationResult>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT holdout_year, brier_score, coverage_accuracy_bp, topics_evaluated
                 FROM forecast_calibration_runs
                 WHERE subject_id = ?1
                 ORDER BY created_at DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([subject_id], |row| {
                Ok(CalibrationResult {
                    subject_id,
                    holdout_year: row.get(0)?,
                    brier_score: clamp_bp(row.get::<_, i64>(1)?),
                    coverage_accuracy_bp: clamp_bp(row.get::<_, i64>(2)?),
                    topics_evaluated: row.get(3)?,
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
    // Crosswalk
    // -----------------------------------------------------------------------

    /// Add a crosswalk entry mapping a legacy label to a current topic.
    pub fn add_crosswalk_entry(
        &self,
        legacy_label: &str,
        legacy_source: Option<&str>,
        current_topic_id: i64,
        confidence_score: BasisPoints,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO curriculum_crosswalk (legacy_label, legacy_source, current_topic_id, confidence_score)
                 VALUES (?1, ?2, ?3, ?4)",
                params![legacy_label, legacy_source, current_topic_id, confidence_score as i64],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Resolve a legacy label to a current topic ID.
    pub fn resolve_legacy_label(&self, legacy_label: &str) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT current_topic_id FROM curriculum_crosswalk
                 WHERE legacy_label = ?1
                 ORDER BY confidence_score DESC LIMIT 1",
                [legacy_label],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn load_topics_in_year(&self, subject_id: i64, year: i64) -> EcoachResult<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT q.topic_id
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                 WHERE q.subject_id = ?1 AND pps.exam_year = ?2 AND q.topic_id IS NOT NULL",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![subject_id, year], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(ids)
    }

    fn load_all_past_paper_topics(&self, subject_id: i64) -> EcoachResult<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT q.topic_id
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 WHERE q.subject_id = ?1 AND q.topic_id IS NOT NULL",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([subject_id], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(ids)
    }
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backtest_computes_brier_score_and_coverage() {
        let conn = Connection::open_in_memory().expect("open");
        seed_schema(&conn);
        seed_data(&conn);

        let engine = CalibrationEngine::new(&conn);
        let result = engine.backtest_year(1, 2023).expect("backtest");

        assert_eq!(result.subject_id, 1);
        assert_eq!(result.holdout_year, 2023);
        assert!(result.brier_score > 0);
        assert!(result.topics_evaluated >= 2);
    }

    #[test]
    fn crosswalk_resolves_legacy_labels() {
        let conn = Connection::open_in_memory().expect("open");
        conn.execute_batch(
            "CREATE TABLE topics (id INTEGER PRIMARY KEY, name TEXT);
             INSERT INTO topics (id, name) VALUES (100, 'Algebra');
             CREATE TABLE curriculum_crosswalk (
                 id INTEGER PRIMARY KEY, legacy_label TEXT, legacy_source TEXT,
                 current_topic_id INTEGER, confidence_score INTEGER DEFAULT 5000,
                 created_at TEXT DEFAULT (datetime('now'))
             );",
        )
        .unwrap();

        let engine = CalibrationEngine::new(&conn);
        engine
            .add_crosswalk_entry("Old Algebra", Some("2015 syllabus"), 100, 8000)
            .unwrap();

        let resolved = engine.resolve_legacy_label("Old Algebra").unwrap();
        assert_eq!(resolved, Some(100));

        let missing = engine.resolve_legacy_label("Nonexistent").unwrap();
        assert_eq!(missing, None);
    }

    fn seed_schema(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE subjects (id INTEGER PRIMARY KEY, name TEXT);
             CREATE TABLE topics (
                 id INTEGER PRIMARY KEY, subject_id INTEGER, name TEXT,
                 node_type TEXT, exam_weight INTEGER, importance_weight INTEGER,
                 is_active INTEGER DEFAULT 1
             );
             CREATE TABLE questions (
                 id INTEGER PRIMARY KEY, subject_id INTEGER, topic_id INTEGER,
                 stem TEXT, question_format TEXT, difficulty_level TEXT,
                 primary_cognitive_demand TEXT, is_active INTEGER DEFAULT 1
             );
             CREATE TABLE past_paper_sets (
                 id INTEGER PRIMARY KEY, subject_id INTEGER,
                 exam_year INTEGER, paper_code TEXT, title TEXT
             );
             CREATE TABLE past_paper_question_links (
                 id INTEGER PRIMARY KEY, paper_id INTEGER,
                 question_id INTEGER, section_label TEXT, question_number TEXT
             );
             CREATE TABLE forecast_calibration_runs (
                 id INTEGER PRIMARY KEY, subject_id INTEGER NOT NULL,
                 holdout_year INTEGER NOT NULL, brier_score INTEGER DEFAULT 0,
                 coverage_accuracy_bp INTEGER DEFAULT 0, topics_evaluated INTEGER DEFAULT 0,
                 created_at TEXT DEFAULT (datetime('now'))
             );",
        )
        .expect("schema");
    }

    fn seed_data(conn: &Connection) {
        conn.execute("INSERT INTO subjects (id, name) VALUES (1, 'Mathematics')", []).unwrap();
        conn.execute("INSERT INTO topics (id, subject_id, name, node_type) VALUES (100, 1, 'Algebra', 'topic')", []).unwrap();
        conn.execute("INSERT INTO topics (id, subject_id, name, node_type) VALUES (200, 1, 'Geometry', 'topic')", []).unwrap();

        for (pid, year) in [(1, 2021), (2, 2022), (3, 2023)] {
            conn.execute(
                "INSERT INTO past_paper_sets (id, subject_id, exam_year, title) VALUES (?1, 1, ?2, 'Paper')",
                params![pid, year],
            ).unwrap();
        }

        // Questions: Algebra in all years, Geometry in 2021 and 2023
        for (qid, tid) in [(1, 100), (2, 100), (3, 100), (4, 200), (5, 200)] {
            conn.execute(
                "INSERT INTO questions (id, subject_id, topic_id, stem, is_active) VALUES (?1, 1, ?2, 'stem', 1)",
                params![qid, tid],
            ).unwrap();
        }

        // Links: algebra in all papers, geometry in papers 1 and 3
        for (pid, qid) in [(1, 1), (2, 2), (3, 3), (1, 4), (3, 5)] {
            conn.execute(
                "INSERT INTO past_paper_question_links (paper_id, question_id) VALUES (?1, ?2)",
                params![pid, qid],
            ).unwrap();
        }
    }
}
