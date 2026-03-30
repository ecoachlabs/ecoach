use std::collections::{BTreeMap, BTreeSet};

use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{
    PastPaperFamilyAnalytics, PastPaperInverseSignal, PastPaperSet, PastPaperSetSummary,
};

pub struct PastPapersService<'a> {
    conn: &'a Connection,
}

impl<'a> PastPapersService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create_paper_set(
        &self,
        subject_id: i64,
        exam_year: i64,
        title: &str,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO past_paper_sets (subject_id, exam_year, title) VALUES (?1, ?2, ?3)",
                params![subject_id, exam_year, title],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_paper_set(&self, paper_id: i64) -> EcoachResult<Option<PastPaperSet>> {
        self.conn
            .query_row(
                "SELECT id, subject_id, exam_year, paper_code, title
                 FROM past_paper_sets
                 WHERE id = ?1",
                [paper_id],
                |row| {
                    Ok(PastPaperSet {
                        id: row.get(0)?,
                        subject_id: row.get(1)?,
                        exam_year: row.get(2)?,
                        paper_code: row.get(3)?,
                        title: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn link_question_to_paper(
        &self,
        paper_id: i64,
        question_id: i64,
        section_label: Option<&str>,
        question_number: Option<&str>,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO past_paper_question_links (paper_id, question_id, section_label, question_number)
                 VALUES (?1, ?2, ?3, ?4)",
                params![paper_id, question_id, section_label, question_number],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn recompute_family_analytics(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Vec<PastPaperFamilyAnalytics>> {
        let families = self.load_family_counts(subject_id)?;
        let max_paper_count = families
            .values()
            .map(|item| item.paper_count)
            .max()
            .unwrap_or(1);
        let max_cofamily_count = families
            .values()
            .map(|item| item.co_family_ids.len() as i64)
            .max()
            .unwrap_or(1);
        let max_subject_year = self
            .conn
            .query_row(
                "SELECT MAX(exam_year) FROM past_paper_sets WHERE subject_id = ?1",
                [subject_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(0);
        let min_subject_year = self
            .conn
            .query_row(
                "SELECT MIN(exam_year) FROM past_paper_sets WHERE subject_id = ?1",
                [subject_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(max_subject_year);
        let year_span = (max_subject_year - min_subject_year).max(1);

        self.conn
            .execute(
                "DELETE FROM question_family_analytics
                 WHERE family_id IN (SELECT id FROM question_families WHERE subject_id = ?1)",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut analytics = Vec::new();
        for family in families.into_values() {
            let recurrence_score = scale_score(family.paper_count, max_paper_count);
            let coappearance_score =
                scale_score(family.co_family_ids.len() as i64, max_cofamily_count.max(1));
            let replacement_gap =
                max_subject_year.saturating_sub(family.last_seen_year.unwrap_or(max_subject_year));
            let replacement_score = scale_score(replacement_gap, year_span);

            self.conn
                .execute(
                    "INSERT INTO question_family_analytics (family_id, recurrence_score, coappearance_score, replacement_score)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        family.family_id,
                        recurrence_score,
                        coappearance_score,
                        replacement_score,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            analytics.push(PastPaperFamilyAnalytics {
                family_id: family.family_id,
                family_code: family.family_code,
                family_name: family.family_name,
                topic_id: family.topic_id,
                recurrence_score,
                coappearance_score,
                replacement_score,
                paper_count: family.paper_count,
                last_seen_year: family.last_seen_year,
            });
        }

        analytics.sort_by(|left, right| {
            right
                .recurrence_score
                .cmp(&left.recurrence_score)
                .then(right.coappearance_score.cmp(&left.coappearance_score))
                .then(left.family_name.cmp(&right.family_name))
        });
        Ok(analytics)
    }

    pub fn list_high_frequency_families(
        &self,
        subject_id: i64,
        topic_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<PastPaperFamilyAnalytics>> {
        let sql = if topic_id.is_some() {
            "SELECT qf.id, qf.family_code, qf.family_name, qf.topic_id,
                    qfa.recurrence_score, qfa.coappearance_score, qfa.replacement_score,
                    COALESCE((
                        SELECT COUNT(DISTINCT ppql.paper_id)
                        FROM past_paper_question_links ppql
                        INNER JOIN questions q ON q.id = ppql.question_id
                        WHERE q.family_id = qf.id
                    ), 0) AS paper_count,
                    (
                        SELECT MAX(pps.exam_year)
                        FROM past_paper_question_links ppql
                        INNER JOIN questions q ON q.id = ppql.question_id
                        INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                        WHERE q.family_id = qf.id
                    ) AS last_seen_year
             FROM question_families qf
             LEFT JOIN question_family_analytics qfa ON qfa.family_id = qf.id
             WHERE qf.subject_id = ?1 AND qf.topic_id = ?2
             ORDER BY qfa.recurrence_score DESC, qfa.coappearance_score DESC, qf.family_name ASC
             LIMIT ?3"
        } else {
            "SELECT qf.id, qf.family_code, qf.family_name, qf.topic_id,
                    qfa.recurrence_score, qfa.coappearance_score, qfa.replacement_score,
                    COALESCE((
                        SELECT COUNT(DISTINCT ppql.paper_id)
                        FROM past_paper_question_links ppql
                        INNER JOIN questions q ON q.id = ppql.question_id
                        WHERE q.family_id = qf.id
                    ), 0) AS paper_count,
                    (
                        SELECT MAX(pps.exam_year)
                        FROM past_paper_question_links ppql
                        INNER JOIN questions q ON q.id = ppql.question_id
                        INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                        WHERE q.family_id = qf.id
                    ) AS last_seen_year
             FROM question_families qf
             LEFT JOIN question_family_analytics qfa ON qfa.family_id = qf.id
             WHERE qf.subject_id = ?1
             ORDER BY qfa.recurrence_score DESC, qfa.coappearance_score DESC, qf.family_name ASC
             LIMIT ?2"
        };

        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mapper = |row: &rusqlite::Row<'_>| {
            Ok(PastPaperFamilyAnalytics {
                family_id: row.get(0)?,
                family_code: row.get(1)?,
                family_name: row.get(2)?,
                topic_id: row.get(3)?,
                recurrence_score: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as BasisPoints,
                coappearance_score: row.get::<_, Option<i64>>(5)?.unwrap_or(0) as BasisPoints,
                replacement_score: row.get::<_, Option<i64>>(6)?.unwrap_or(0) as BasisPoints,
                paper_count: row.get(7)?,
                last_seen_year: row.get(8)?,
            })
        };

        let rows = if let Some(topic_id) = topic_id {
            statement
                .query_map(params![subject_id, topic_id, limit as i64], mapper)
                .map_err(|err| EcoachError::Storage(err.to_string()))?
        } else {
            statement
                .query_map(params![subject_id, limit as i64], mapper)
                .map_err(|err| EcoachError::Storage(err.to_string()))?
        };

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn list_papers_for_family(&self, family_id: i64) -> EcoachResult<Vec<PastPaperSetSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT pps.id, pps.exam_year, pps.title, COUNT(ppql.id) AS question_count
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                 WHERE q.family_id = ?1
                 GROUP BY pps.id, pps.exam_year, pps.title
                 ORDER BY pps.exam_year DESC, pps.id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([family_id], |row| {
                Ok(PastPaperSetSummary {
                    paper_id: row.get(0)?,
                    exam_year: row.get(1)?,
                    title: row.get(2)?,
                    question_count: row.get(3)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn list_inverse_pressure_families(
        &self,
        subject_id: i64,
        topic_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<PastPaperInverseSignal>> {
        let analytics = self.list_high_frequency_families(subject_id, topic_id, limit.max(8))?;
        let max_inverse = analytics
            .iter()
            .map(|item| {
                composite_inverse_pressure(
                    item.recurrence_score,
                    item.coappearance_score,
                    item.replacement_score,
                )
            })
            .max()
            .unwrap_or(1);

        let mut signals = analytics
            .into_iter()
            .map(|item| {
                let raw_inverse = composite_inverse_pressure(
                    item.recurrence_score,
                    item.coappearance_score,
                    item.replacement_score,
                );
                let inverse_pressure_score = scale_score(raw_inverse, max_inverse.max(1));
                PastPaperInverseSignal {
                    family_id: item.family_id,
                    family_code: item.family_code,
                    family_name: item.family_name,
                    topic_id: item.topic_id,
                    inverse_pressure_score,
                    recurrence_score: item.recurrence_score,
                    coappearance_score: item.coappearance_score,
                    replacement_score: item.replacement_score,
                    paper_count: item.paper_count,
                    last_seen_year: item.last_seen_year,
                    rationale: inverse_rationale(
                        item.recurrence_score,
                        item.coappearance_score,
                        item.replacement_score,
                        item.last_seen_year,
                    ),
                }
            })
            .collect::<Vec<_>>();

        signals.sort_by(|left, right| {
            right
                .inverse_pressure_score
                .cmp(&left.inverse_pressure_score)
                .then(right.replacement_score.cmp(&left.replacement_score))
                .then(right.coappearance_score.cmp(&left.coappearance_score))
                .then(left.family_name.cmp(&right.family_name))
        });
        signals.truncate(limit.max(1));
        Ok(signals)
    }

    fn load_family_counts(&self, subject_id: i64) -> EcoachResult<BTreeMap<i64, FamilyAggregate>> {
        let mut family_map = BTreeMap::new();
        let mut statement = self
            .conn
            .prepare(
                "SELECT qf.id, qf.family_code, qf.family_name, qf.topic_id, pps.id, pps.exam_year
             FROM question_families qf
             LEFT JOIN questions q ON q.family_id = qf.id
             LEFT JOIN past_paper_question_links ppql ON ppql.question_id = q.id
             LEFT JOIN past_paper_sets pps ON pps.id = ppql.paper_id
             WHERE qf.subject_id = ?1
             ORDER BY qf.id ASC, pps.exam_year ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for row in rows {
            let (family_id, family_code, family_name, topic_id, paper_id, exam_year) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let entry = family_map
                .entry(family_id)
                .or_insert_with(|| FamilyAggregate {
                    family_id,
                    family_code,
                    family_name,
                    topic_id,
                    paper_ids: BTreeSet::new(),
                    co_family_ids: BTreeSet::new(),
                    last_seen_year: None,
                    paper_count: 0,
                });
            if let Some(paper_id) = paper_id {
                entry.paper_ids.insert(paper_id);
                entry.paper_count = entry.paper_ids.len() as i64;
            }
            if exam_year.is_some() {
                entry.last_seen_year = exam_year;
            }
        }

        let mut co_statement = self
            .conn
            .prepare(
                "SELECT q1.family_id, q2.family_id
             FROM past_paper_question_links l1
             INNER JOIN questions q1 ON q1.id = l1.question_id
             INNER JOIN past_paper_question_links l2 ON l2.paper_id = l1.paper_id AND l2.id <> l1.id
             INNER JOIN questions q2 ON q2.id = l2.question_id
             WHERE q1.family_id IS NOT NULL
               AND q2.family_id IS NOT NULL
               AND q1.family_id <> q2.family_id
               AND q1.subject_id = ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let co_rows = co_statement
            .query_map([subject_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for row in co_rows {
            let (family_id, co_family_id) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(entry) = family_map.get_mut(&family_id) {
                entry.co_family_ids.insert(co_family_id);
            }
        }

        Ok(family_map)
    }
}

struct FamilyAggregate {
    family_id: i64,
    family_code: String,
    family_name: String,
    topic_id: Option<i64>,
    paper_ids: BTreeSet<i64>,
    co_family_ids: BTreeSet<i64>,
    last_seen_year: Option<i64>,
    paper_count: i64,
}

fn scale_score(value: i64, max_value: i64) -> BasisPoints {
    if max_value <= 0 {
        return 0;
    }
    clamp_bp(((value as f64 / max_value as f64) * 10_000.0).round() as i64) as BasisPoints
}

fn composite_inverse_pressure(
    recurrence_score: BasisPoints,
    coappearance_score: BasisPoints,
    replacement_score: BasisPoints,
) -> i64 {
    i64::from(clamp_bp(
        (0.45 * replacement_score as f64
            + 0.30 * coappearance_score as f64
            + 0.25 * recurrence_score as f64)
            .round() as i64,
    ))
}

fn inverse_rationale(
    recurrence_score: BasisPoints,
    coappearance_score: BasisPoints,
    replacement_score: BasisPoints,
    last_seen_year: Option<i64>,
) -> String {
    if replacement_score >= 7_500 && recurrence_score >= 4_500 {
        format!(
            "Historically recurring family that has gone quiet since {:?}, so replacement pressure is building.",
            last_seen_year
        )
    } else if coappearance_score >= 7_000 {
        "Often appears alongside other recurring family patterns, so it is a likely comeback candidate.".to_string()
    } else if replacement_score >= 6_000 {
        "Recent absence is now long enough that the family is becoming overdue for a return."
            .to_string()
    } else {
        "Keep this family warm because its paper-history shape suggests hidden comeback risk."
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn inverse_pressure_families_prioritize_overdue_but_recurrent_patterns() {
        let conn = Connection::open_in_memory().expect("in-memory db should open");
        seed_schema(&conn);
        seed_family_history(&conn);

        let service = PastPapersService::new(&conn);
        let inverse = service
            .list_inverse_pressure_families(1, None, 2)
            .expect("inverse pressure families should compute");

        assert_eq!(inverse.len(), 2);
        assert_eq!(inverse[0].family_id, 10);
        assert!(inverse[0].inverse_pressure_score >= inverse[1].inverse_pressure_score);
        assert!(inverse[0].rationale.contains("Historically recurring"));
    }

    fn seed_schema(conn: &Connection) {
        for sql in [
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
                family_id INTEGER,
                topic_id INTEGER
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
            "CREATE TABLE question_family_analytics (
                id INTEGER PRIMARY KEY,
                family_id INTEGER NOT NULL,
                recurrence_score INTEGER NOT NULL,
                coappearance_score INTEGER NOT NULL,
                replacement_score INTEGER NOT NULL,
                updated_at TEXT
            )",
        ] {
            conn.execute(sql, [])
                .expect("schema statement should execute");
        }
    }

    fn seed_family_history(conn: &Connection) {
        for (family_id, code, name) in [
            (10_i64, "ALG_REENTRY", "Algebra Re-entry"),
            (20, "ALG_SHARED", "Algebra Shared"),
        ] {
            conn.execute(
                "INSERT INTO question_families (id, family_code, family_name, subject_id, topic_id)
                 VALUES (?1, ?2, ?3, 1, 100)",
                params![family_id, code, name],
            )
            .expect("family should insert");
        }
        for (question_id, family_id) in [(1000_i64, 10_i64), (1001, 10), (2000, 20)] {
            conn.execute(
                "INSERT INTO questions (id, subject_id, family_id, topic_id) VALUES (?1, 1, ?2, 100)",
                params![question_id, family_id],
            )
            .expect("question should insert");
        }
        for (paper_id, year) in [(1_i64, 2021_i64), (2, 2022), (3, 2025)] {
            conn.execute(
                "INSERT INTO past_paper_sets (id, subject_id, exam_year, title)
                 VALUES (?1, 1, ?2, 'Paper')",
                params![paper_id, year],
            )
            .expect("paper should insert");
        }
        for (paper_id, question_id) in [(1_i64, 1000_i64), (2, 1001), (3, 2000)] {
            conn.execute(
                "INSERT INTO past_paper_question_links (paper_id, question_id) VALUES (?1, ?2)",
                params![paper_id, question_id],
            )
            .expect("paper link should insert");
        }
        conn.execute(
            "INSERT INTO question_family_analytics (family_id, recurrence_score, coappearance_score, replacement_score)
             VALUES (10, 8400, 7600, 9200)",
            [],
        )
        .expect("first analytics should insert");
        conn.execute(
            "INSERT INTO question_family_analytics (family_id, recurrence_score, coappearance_score, replacement_score)
             VALUES (20, 6600, 5400, 4800)",
            [],
        )
        .expect("second analytics should insert");
    }
}
