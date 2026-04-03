use std::collections::BTreeMap;

use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::Connection;

use crate::models::{PatternProfile, TopicCoOccurrence, TopicPaperPresence};

pub struct PatternMiner<'a> {
    conn: &'a Connection,
}

impl<'a> PatternMiner<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Build a complete pattern profile for a subject by scanning all past papers.
    pub fn build_pattern_profile(&self, subject_id: i64) -> EcoachResult<PatternProfile> {
        let (year_min, year_max, total_papers) = self.load_paper_range(subject_id)?;
        let format_counts = self.count_by_column(subject_id, "question_format")?;
        let difficulty_counts = self.count_by_column(subject_id, "difficulty_level")?;
        let cognitive_demand_counts =
            self.count_by_column(subject_id, "primary_cognitive_demand")?;
        let topic_count = self.count_topics(subject_id)?;

        Ok(PatternProfile {
            subject_id,
            total_papers,
            year_range_start: year_min,
            year_range_end: year_max,
            topic_count,
            format_counts,
            difficulty_counts,
            cognitive_demand_counts,
        })
    }

    /// Load per-topic presence data across all past papers for this subject.
    pub fn mine_topic_presence(&self, subject_id: i64) -> EcoachResult<Vec<TopicPaperPresence>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.name, pps.exam_year, q.question_format,
                        q.difficulty_level, q.primary_cognitive_demand
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                 INNER JOIN topics t ON t.id = q.topic_id
                 WHERE q.subject_id = ?1 AND q.is_active = 1
                 ORDER BY t.id, pps.exam_year",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut topic_map: BTreeMap<i64, TopicPaperPresence> = BTreeMap::new();

        for row in rows {
            let (topic_id, topic_name, year, format, difficulty, cognitive) =
                row.map_err(|e| EcoachError::Storage(e.to_string()))?;

            let entry = topic_map
                .entry(topic_id)
                .or_insert_with(|| TopicPaperPresence {
                    topic_id,
                    topic_name,
                    years_present: Vec::new(),
                    total_questions: 0,
                    question_formats: Vec::new(),
                    difficulty_bands: Vec::new(),
                    cognitive_demands: Vec::new(),
                });

            if !entry.years_present.contains(&year) {
                entry.years_present.push(year);
            }
            entry.total_questions += 1;
            if let Some(f) = format {
                if !entry.question_formats.contains(&f) {
                    entry.question_formats.push(f);
                }
            }
            if let Some(d) = difficulty {
                if !entry.difficulty_bands.contains(&d) {
                    entry.difficulty_bands.push(d);
                }
            }
            if let Some(c) = cognitive {
                if !entry.cognitive_demands.contains(&c) {
                    entry.cognitive_demands.push(c);
                }
            }
        }

        Ok(topic_map.into_values().collect())
    }

    /// Mine co-occurrence bundles: which topics appear together in the same paper.
    pub fn mine_co_occurrences(&self, subject_id: i64) -> EcoachResult<Vec<TopicCoOccurrence>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT q1.topic_id, q2.topic_id, COUNT(DISTINCT ppql1.paper_id) AS together
                 FROM past_paper_question_links ppql1
                 INNER JOIN questions q1 ON q1.id = ppql1.question_id
                 INNER JOIN past_paper_question_links ppql2
                     ON ppql2.paper_id = ppql1.paper_id AND ppql2.id <> ppql1.id
                 INNER JOIN questions q2 ON q2.id = ppql2.question_id
                 WHERE q1.subject_id = ?1
                   AND q1.topic_id IS NOT NULL
                   AND q2.topic_id IS NOT NULL
                   AND q1.topic_id < q2.topic_id
                 GROUP BY q1.topic_id, q2.topic_id
                 ORDER BY together DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([subject_id], |row| {
                Ok(TopicCoOccurrence {
                    topic_a: row.get(0)?,
                    topic_b: row.get(1)?,
                    papers_together: row.get(2)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(result)
    }

    /// Detect style regime changes: how format/difficulty distribution shifts over time.
    /// Returns (year, format_code, count) triples.
    pub fn mine_format_distribution_by_year(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Vec<(i64, String, i64)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT pps.exam_year, q.question_format, COUNT(*) AS cnt
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                 WHERE q.subject_id = ?1 AND q.question_format IS NOT NULL
                 GROUP BY pps.exam_year, q.question_format
                 ORDER BY pps.exam_year ASC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn load_paper_range(&self, subject_id: i64) -> EcoachResult<(Option<i64>, Option<i64>, i64)> {
        self.conn
            .query_row(
                "SELECT MIN(exam_year), MAX(exam_year), COUNT(*)
                 FROM past_paper_sets
                 WHERE subject_id = ?1",
                [subject_id],
                |row| {
                    Ok((
                        row.get::<_, Option<i64>>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }

    fn count_by_column(&self, subject_id: i64, column: &str) -> EcoachResult<Vec<(String, i64)>> {
        // Column name is internal only (never from user input), safe to interpolate.
        let sql = format!(
            "SELECT q.{col}, COUNT(*) AS cnt
             FROM past_paper_question_links ppql
             INNER JOIN questions q ON q.id = ppql.question_id
             WHERE q.subject_id = ?1 AND q.{col} IS NOT NULL
             GROUP BY q.{col}
             ORDER BY cnt DESC",
            col = column,
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([subject_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(result)
    }

    fn count_topics(&self, subject_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(DISTINCT q.topic_id)
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 WHERE q.subject_id = ?1",
                [subject_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    #[test]
    fn pattern_miner_builds_profile_from_past_papers() {
        let conn = Connection::open_in_memory().expect("open");
        seed_schema(&conn);
        seed_data(&conn);

        let miner = PatternMiner::new(&conn);
        let profile = miner.build_pattern_profile(1).expect("profile");

        assert_eq!(profile.subject_id, 1);
        assert_eq!(profile.total_papers, 3);
        assert_eq!(profile.year_range_start, Some(2021));
        assert_eq!(profile.year_range_end, Some(2023));
        assert!(profile.topic_count >= 2);
    }

    #[test]
    fn topic_presence_captures_years_and_formats() {
        let conn = Connection::open_in_memory().expect("open");
        seed_schema(&conn);
        seed_data(&conn);

        let miner = PatternMiner::new(&conn);
        let presence = miner.mine_topic_presence(1).expect("presence");

        assert!(presence.len() >= 2);
        let topic_100 = presence.iter().find(|t| t.topic_id == 100).unwrap();
        assert_eq!(topic_100.years_present.len(), 3);
        assert_eq!(topic_100.total_questions, 3);
    }

    #[test]
    fn co_occurrences_detect_topics_in_same_paper() {
        let conn = Connection::open_in_memory().expect("open");
        seed_schema(&conn);
        seed_data(&conn);

        let miner = PatternMiner::new(&conn);
        let co = miner.mine_co_occurrences(1).expect("co-occurrences");

        assert!(!co.is_empty());
        let pair = co
            .iter()
            .find(|c| {
                (c.topic_a == 100 && c.topic_b == 200) || (c.topic_a == 200 && c.topic_b == 100)
            })
            .expect("should find topics 100 and 200 co-occurring");
        assert!(pair.papers_together >= 2);
    }

    fn seed_schema(conn: &Connection) {
        conn.execute_batch(
            "CREATE TABLE subjects (id INTEGER PRIMARY KEY, name TEXT);
             CREATE TABLE topics (
                 id INTEGER PRIMARY KEY, subject_id INTEGER, parent_topic_id INTEGER,
                 code TEXT, name TEXT, node_type TEXT, display_order INTEGER,
                 exam_weight INTEGER, importance_weight INTEGER, is_active INTEGER DEFAULT 1
             );
             CREATE TABLE questions (
                 id INTEGER PRIMARY KEY, subject_id INTEGER, topic_id INTEGER,
                 family_id INTEGER, stem TEXT, question_format TEXT,
                 difficulty_level TEXT, primary_cognitive_demand TEXT,
                 is_active INTEGER DEFAULT 1
             );
             CREATE TABLE past_paper_sets (
                 id INTEGER PRIMARY KEY, subject_id INTEGER,
                 exam_year INTEGER, paper_code TEXT, title TEXT
             );
             CREATE TABLE past_paper_question_links (
                 id INTEGER PRIMARY KEY, paper_id INTEGER,
                 question_id INTEGER, section_label TEXT, question_number TEXT
             );",
        )
        .expect("schema");
    }

    fn seed_data(conn: &Connection) {
        conn.execute(
            "INSERT INTO subjects (id, name) VALUES (1, 'Mathematics')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO topics (id, subject_id, name, node_type) VALUES (100, 1, 'Algebra', 'topic')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO topics (id, subject_id, name, node_type) VALUES (200, 1, 'Geometry', 'topic')",
            [],
        )
        .unwrap();

        // 3 papers across 3 years
        for (pid, year) in [(1, 2021), (2, 2022), (3, 2023)] {
            conn.execute(
                "INSERT INTO past_paper_sets (id, subject_id, exam_year, title) VALUES (?1, 1, ?2, 'Paper')",
                params![pid, year],
            )
            .unwrap();
        }

        // Questions: topic 100 appears in all 3 papers, topic 200 in papers 1 and 2
        let questions: Vec<(i64, i64, &str, &str, &str)> = vec![
            (1, 100, "mcq", "easy", "recall"),
            (2, 100, "mcq", "medium", "application"),
            (3, 100, "structured", "hard", "analysis"),
            (4, 200, "mcq", "easy", "recall"),
            (5, 200, "structured", "medium", "application"),
        ];
        for (qid, tid, fmt, diff, cog) in &questions {
            conn.execute(
                "INSERT INTO questions (id, subject_id, topic_id, stem, question_format, difficulty_level, primary_cognitive_demand, is_active)
                 VALUES (?1, 1, ?2, 'stem', ?3, ?4, ?5, 1)",
                params![qid, tid, fmt, diff, cog],
            )
            .unwrap();
        }

        // Link questions to papers
        let links: Vec<(i64, i64)> = vec![
            (1, 1),
            (1, 4), // paper 1: algebra + geometry
            (2, 2),
            (2, 5), // paper 2: algebra + geometry
            (3, 3), // paper 3: algebra only
        ];
        for (pid, qid) in &links {
            conn.execute(
                "INSERT INTO past_paper_question_links (paper_id, question_id) VALUES (?1, ?2)",
                params![pid, qid],
            )
            .unwrap();
        }
    }
}
