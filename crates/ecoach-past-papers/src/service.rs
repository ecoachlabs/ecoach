use std::collections::{BTreeMap, BTreeSet};

use ecoach_substrate::{clamp_bp, BasisPoints, EcoachError, EcoachResult};
use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{
    CreateFamilyEdgeInput, FamilyRecurrenceMetric, FamilyRelationshipEdge, FamilyReplacementTrail,
    FamilyStory, InverseAppearancePair, PaperDna, PastPaperComebackSignal, PastPaperCourseSummary,
    PastPaperFamilyAnalytics, PastPaperInverseSignal, PastPaperSection, PastPaperSectionKind,
    PastPaperSet, PastPaperSetSummary, PastPaperTopicCount, PastPaperYear, QuestionAssetMeta,
    StudentFamilyPerformance,
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

    /// Update the metadata row for an existing paper set. Used when the
    /// admin edits a paper's title / year / paper_code after creation.
    pub fn update_paper_set(
        &self,
        paper_id: i64,
        exam_year: i64,
        paper_code: Option<&str>,
        title: &str,
    ) -> EcoachResult<()> {
        let changed = self
            .conn
            .execute(
                "UPDATE past_paper_sets
                 SET exam_year = ?2, paper_code = ?3, title = ?4
                 WHERE id = ?1",
                params![paper_id, exam_year, paper_code, title],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if changed == 0 {
            return Err(EcoachError::NotFound(format!(
                "past paper set {} not found",
                paper_id
            )));
        }
        Ok(())
    }

    /// Drop every question-link for a paper. Used before re-saving an
    /// edited paper — we regenerate all links from the new question
    /// list rather than diffing, which keeps the authoring code trivial.
    pub fn delete_paper_question_links(&self, paper_id: i64) -> EcoachResult<i64> {
        let removed = self
            .conn
            .execute(
                "DELETE FROM past_paper_question_links WHERE paper_id = ?1",
                [paper_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(removed as i64)
    }

    /// Delete a paper set. The `ON DELETE CASCADE` on
    /// past_paper_question_links (migration 016) cleans up links. The
    /// underlying `questions` rows are not removed — they keep
    /// independent lifecycle and are shared with practice flows.
    pub fn delete_paper_set(&self, paper_id: i64) -> EcoachResult<()> {
        let removed = self
            .conn
            .execute("DELETE FROM past_paper_sets WHERE id = ?1", [paper_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if removed == 0 {
            return Err(EcoachError::NotFound(format!(
                "past paper set {} not found",
                paper_id
            )));
        }
        Ok(())
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

    pub fn list_comeback_candidate_families(
        &self,
        subject_id: i64,
        topic_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<PastPaperComebackSignal>> {
        let analytics = self.list_high_frequency_families(subject_id, topic_id, limit.max(8))?;
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
        let year_span = (latest_subject_year - earliest_subject_year).max(1);

        let scored = analytics
            .into_iter()
            .map(|item| {
                let dormant_years = latest_subject_year
                    .saturating_sub(item.last_seen_year.unwrap_or(latest_subject_year));
                let dormant_score = scale_score(dormant_years, year_span);
                let paper_breadth_score = scale_score(item.paper_count, 6);
                let historical_strength_score = clamp_bp(
                    ((0.60 * item.recurrence_score as f64)
                        + (0.25 * item.coappearance_score as f64)
                        + (0.15 * paper_breadth_score as f64))
                        .round() as i64,
                );
                let comeback_score = clamp_bp(
                    ((f64::from(historical_strength_score) * f64::from(dormant_score)) / 10_000.0)
                        .round() as i64
                        + (i64::from(item.replacement_score) / 4),
                ) as BasisPoints;
                PastPaperComebackSignal {
                    family_id: item.family_id,
                    family_code: item.family_code,
                    family_name: item.family_name,
                    topic_id: item.topic_id,
                    comeback_score,
                    historical_strength_score,
                    dormant_years,
                    recurrence_score: item.recurrence_score,
                    replacement_score: item.replacement_score,
                    paper_count: item.paper_count,
                    last_seen_year: item.last_seen_year,
                    rationale: comeback_rationale(
                        dormant_years,
                        historical_strength_score,
                        item.last_seen_year,
                    ),
                }
            })
            .collect::<Vec<_>>();

        let mut signals = scored;
        signals.sort_by(|left, right| {
            right
                .comeback_score
                .cmp(&left.comeback_score)
                .then(
                    right
                        .historical_strength_score
                        .cmp(&left.historical_strength_score),
                )
                .then(right.dormant_years.cmp(&left.dormant_years))
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

    // ── Exam Intelligence methods (idea13) ──

    pub fn get_paper_dna(&self, paper_set_id: i64) -> EcoachResult<Option<PaperDna>> {
        self.conn
            .query_row(
                "SELECT id, paper_set_id, recall_vs_reasoning_ratio, novelty_score,
                        story_summary, dominant_families_json, computed_at
                 FROM paper_dna WHERE paper_set_id = ?1",
                [paper_set_id],
                |row| {
                    Ok(PaperDna {
                        id: row.get(0)?,
                        paper_set_id: row.get(1)?,
                        recall_vs_reasoning_ratio: row.get(2)?,
                        novelty_score: row.get(3)?,
                        story_summary: row.get(4)?,
                        dominant_families_json: row.get(5)?,
                        computed_at: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }

    pub fn upsert_paper_dna(
        &self,
        paper_set_id: i64,
        recall_vs_reasoning: BasisPoints,
        novelty: BasisPoints,
        story_summary: Option<&str>,
        dominant_families_json: &str,
        topic_distribution_json: &str,
        cognitive_balance_json: &str,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO paper_dna (
                    paper_set_id, recall_vs_reasoning_ratio, novelty_score,
                    story_summary, dominant_families_json, topic_distribution_json,
                    cognitive_balance_json
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7)
                 ON CONFLICT(paper_set_id) DO UPDATE SET
                    recall_vs_reasoning_ratio = excluded.recall_vs_reasoning_ratio,
                    novelty_score = excluded.novelty_score,
                    story_summary = excluded.story_summary,
                    dominant_families_json = excluded.dominant_families_json,
                    topic_distribution_json = excluded.topic_distribution_json,
                    cognitive_balance_json = excluded.cognitive_balance_json,
                    computed_at = datetime('now')",
                params![
                    paper_set_id,
                    recall_vs_reasoning,
                    novelty,
                    story_summary,
                    dominant_families_json,
                    topic_distribution_json,
                    cognitive_balance_json,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn create_family_edge(&self, input: &CreateFamilyEdgeInput) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO family_relationship_edges (
                    source_family_id, target_family_id, edge_type,
                    strength_score, confidence_score, support_count, evidence_json
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7)
                 ON CONFLICT(source_family_id, target_family_id, edge_type) DO UPDATE SET
                    strength_score = excluded.strength_score,
                    confidence_score = excluded.confidence_score,
                    support_count = excluded.support_count,
                    evidence_json = excluded.evidence_json,
                    updated_at = datetime('now')",
                params![
                    input.source_family_id,
                    input.target_family_id,
                    input.edge_type,
                    input.strength_score,
                    input.confidence_score,
                    input.support_count,
                    input.evidence_json,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_family_edges(
        &self,
        family_id: i64,
        edge_type: Option<&str>,
    ) -> EcoachResult<Vec<FamilyRelationshipEdge>> {
        let sql = if let Some(et) = edge_type {
            format!(
                "SELECT id, source_family_id, target_family_id, edge_type,
                        strength_score, confidence_score, support_count
                 FROM family_relationship_edges
                 WHERE (source_family_id = ?1 OR target_family_id = ?1)
                   AND edge_type = '{}'
                 ORDER BY strength_score DESC",
                et
            )
        } else {
            "SELECT id, source_family_id, target_family_id, edge_type,
                    strength_score, confidence_score, support_count
             FROM family_relationship_edges
             WHERE source_family_id = ?1 OR target_family_id = ?1
             ORDER BY strength_score DESC"
                .to_string()
        };

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([family_id], |row| {
                Ok(FamilyRelationshipEdge {
                    id: row.get(0)?,
                    source_family_id: row.get(1)?,
                    target_family_id: row.get(2)?,
                    edge_type: row.get(3)?,
                    strength_score: row.get(4)?,
                    confidence_score: row.get(5)?,
                    support_count: row.get(6)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut edges = Vec::new();
        for row in rows {
            edges.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(edges)
    }

    pub fn list_inverse_pairs(&self, family_id: i64) -> EcoachResult<Vec<InverseAppearancePair>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT family_a_id, family_b_id, iai_score_bp,
                        directional_a_suppresses_b_bp, directional_b_suppresses_a_bp,
                        support_papers, is_mutual, likely_explanation
                 FROM inverse_appearance_pairs
                 WHERE family_a_id = ?1 OR family_b_id = ?1
                 ORDER BY iai_score_bp DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([family_id], |row| {
                Ok(InverseAppearancePair {
                    family_a_id: row.get(0)?,
                    family_b_id: row.get(1)?,
                    iai_score_bp: row.get(2)?,
                    directional_a_suppresses_b_bp: row.get(3)?,
                    directional_b_suppresses_a_bp: row.get(4)?,
                    support_papers: row.get(5)?,
                    is_mutual: row.get::<_, i64>(6)? == 1,
                    likely_explanation: row.get(7)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut pairs = Vec::new();
        for row in rows {
            pairs.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(pairs)
    }

    pub fn list_replacement_trails(
        &self,
        family_id: i64,
    ) -> EcoachResult<Vec<FamilyReplacementTrail>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT old_family_id, new_family_id, replacement_index_bp,
                        iai_component_bp, chrono_shift_bp, topic_overlap_bp, cognitive_overlap_bp
                 FROM family_replacement_trails
                 WHERE old_family_id = ?1 OR new_family_id = ?1
                 ORDER BY replacement_index_bp DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([family_id], |row| {
                Ok(FamilyReplacementTrail {
                    old_family_id: row.get(0)?,
                    new_family_id: row.get(1)?,
                    replacement_index_bp: row.get(2)?,
                    iai_component_bp: row.get(3)?,
                    chrono_shift_bp: row.get(4)?,
                    topic_overlap_bp: row.get(5)?,
                    cognitive_overlap_bp: row.get(6)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut trails = Vec::new();
        for row in rows {
            trails.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(trails)
    }

    pub fn get_student_family_performance(
        &self,
        student_id: i64,
        family_id: i64,
    ) -> EcoachResult<Option<StudentFamilyPerformance>> {
        self.conn
            .query_row(
                "SELECT student_id, family_id, attempt_count, accuracy_rate_bp,
                        confidence_calibration_bp, classical_form_accuracy_bp,
                        mutated_form_accuracy_bp, trap_fall_rate_bp, recovery_progress_bp
                 FROM student_family_performance
                 WHERE student_id = ?1 AND family_id = ?2",
                params![student_id, family_id],
                |row| {
                    Ok(StudentFamilyPerformance {
                        student_id: row.get(0)?,
                        family_id: row.get(1)?,
                        attempt_count: row.get(2)?,
                        accuracy_rate_bp: row.get(3)?,
                        confidence_calibration_bp: row.get(4)?,
                        classical_form_accuracy_bp: row.get(5)?,
                        mutated_form_accuracy_bp: row.get(6)?,
                        trap_fall_rate_bp: row.get(7)?,
                        recovery_progress_bp: row.get(8)?,
                    })
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }

    pub fn upsert_student_family_performance(
        &self,
        student_id: i64,
        family_id: i64,
        accuracy_bp: BasisPoints,
        trap_fall_bp: BasisPoints,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO student_family_performance (
                    student_id, family_id, attempt_count, accuracy_rate_bp, trap_fall_rate_bp
                 ) VALUES (?1,?2,1,?3,?4)
                 ON CONFLICT(student_id, family_id) DO UPDATE SET
                    attempt_count = attempt_count + 1,
                    accuracy_rate_bp = excluded.accuracy_rate_bp,
                    trap_fall_rate_bp = excluded.trap_fall_rate_bp,
                    updated_at = datetime('now'),
                    last_attempted_at = datetime('now')",
                params![student_id, family_id, accuracy_bp, trap_fall_bp],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn get_family_story(
        &self,
        family_id: i64,
        story_type: &str,
    ) -> EcoachResult<Option<FamilyStory>> {
        self.conn
            .query_row(
                "SELECT id, family_id, story_type, headline, narrative, recommendation
                 FROM family_stories
                 WHERE family_id = ?1 AND story_type = ?2",
                params![family_id, story_type],
                |row| {
                    Ok(FamilyStory {
                        id: row.get(0)?,
                        family_id: row.get(1)?,
                        story_type: row.get(2)?,
                        headline: row.get(3)?,
                        narrative: row.get(4)?,
                        recommendation: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }

    pub fn upsert_family_story(
        &self,
        family_id: i64,
        story_type: &str,
        headline: &str,
        narrative: &str,
        recommendation: Option<&str>,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO family_stories (family_id, story_type, headline, narrative, recommendation)
                 VALUES (?1,?2,?3,?4,?5)
                 ON CONFLICT(family_id, story_type) DO UPDATE SET
                    headline = excluded.headline,
                    narrative = excluded.narrative,
                    recommendation = excluded.recommendation,
                    generated_at = datetime('now')",
                params![family_id, story_type, headline, narrative, recommendation],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn list_family_recurrence_metrics(
        &self,
        subject_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<FamilyRecurrenceMetric>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT family_id, subject_id, total_papers_in_window, papers_appeared,
                        recurrence_rate_bp, persistence_score_bp, dormancy_max_years,
                        last_appearance_year, first_appearance_year, mutation_trend,
                        current_relevance_bp
                 FROM family_recurrence_metrics
                 WHERE subject_id = ?1
                 ORDER BY recurrence_rate_bp DESC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![subject_id, limit as i64], |row| {
                Ok(FamilyRecurrenceMetric {
                    family_id: row.get(0)?,
                    subject_id: row.get(1)?,
                    total_papers: row.get(2)?,
                    papers_appeared: row.get(3)?,
                    recurrence_rate_bp: row.get(4)?,
                    persistence_score_bp: row.get(5)?,
                    dormancy_max_years: row.get(6)?,
                    last_appearance_year: row.get(7)?,
                    first_appearance_year: row.get(8)?,
                    mutation_trend: row.get(9)?,
                    current_relevance_bp: row.get(10)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(metrics)
    }

    // ── Index computation methods (idea13 deep) ──

    /// Compute IAI (Inverse Appearance Index) for two families.
    /// IAI(F1,F2) = max(0, (ExpectedJoint - ObservedJoint) / ExpectedJoint)
    pub fn compute_iai(
        &self,
        family_a_id: i64,
        family_b_id: i64,
        subject_id: i64,
    ) -> EcoachResult<(BasisPoints, BasisPoints, BasisPoints)> {
        let total_papers: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT id) FROM past_paper_sets WHERE subject_id = ?1",
                [subject_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        if total_papers == 0 {
            return Ok((0, 0, 0));
        }

        let papers_with_a: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT l.paper_id) FROM past_paper_question_links l
             JOIN questions q ON q.id = l.question_id
             WHERE q.family_id = ?1",
                [family_a_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let papers_with_b: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT l.paper_id) FROM past_paper_question_links l
             JOIN questions q ON q.id = l.question_id
             WHERE q.family_id = ?1",
                [family_b_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let papers_with_both: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT la.paper_id)
             FROM past_paper_question_links la
             JOIN questions qa ON qa.id = la.question_id AND qa.family_id = ?1
             JOIN past_paper_question_links lb ON lb.paper_id = la.paper_id
             JOIN questions qb ON qb.id = lb.question_id AND qb.family_id = ?2",
                params![family_a_id, family_b_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let p_a = papers_with_a as f64 / total_papers as f64;
        let p_b = papers_with_b as f64 / total_papers as f64;
        let p_both = papers_with_both as f64 / total_papers as f64;
        let expected = p_a * p_b;

        // IAI score
        let iai = if expected > 0.0 {
            ((expected - p_both).max(0.0) / expected * 10_000.0).round() as i64
        } else {
            0
        };

        // Directional: A suppresses B
        let diai_a_suppresses_b = if p_b > 0.0 && papers_with_a > 0 {
            let p_b_given_a = papers_with_both as f64 / papers_with_a as f64;
            ((1.0 - p_b_given_a / p_b).max(0.0) * 10_000.0).round() as i64
        } else {
            0
        };

        // Directional: B suppresses A
        let diai_b_suppresses_a = if p_a > 0.0 && papers_with_b > 0 {
            let p_a_given_b = papers_with_both as f64 / papers_with_b as f64;
            ((1.0 - p_a_given_b / p_a).max(0.0) * 10_000.0).round() as i64
        } else {
            0
        };

        Ok((
            clamp_bp(iai),
            clamp_bp(diai_a_suppresses_b),
            clamp_bp(diai_b_suppresses_a),
        ))
    }

    /// Compute and persist IAI for all family pairs in a subject
    pub fn compute_inverse_pairs_for_subject(&self, subject_id: i64) -> EcoachResult<i64> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT q.family_id FROM questions q
             JOIN past_paper_question_links l ON l.question_id = q.id
             JOIN past_paper_sets p ON p.id = l.paper_id
             WHERE p.subject_id = ?1 AND q.family_id IS NOT NULL",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let family_ids: Vec<i64> = stmt
            .query_map([subject_id], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        let mut pairs_created = 0i64;
        for i in 0..family_ids.len() {
            for j in (i + 1)..family_ids.len() {
                let (iai, a_sup_b, b_sup_a) =
                    self.compute_iai(family_ids[i], family_ids[j], subject_id)?;

                if iai >= 2000 {
                    let is_mutual = a_sup_b >= 2000 && b_sup_a >= 2000;
                    self.conn
                        .execute(
                            "INSERT INTO inverse_appearance_pairs (
                            family_a_id, family_b_id, iai_score_bp,
                            directional_a_suppresses_b_bp, directional_b_suppresses_a_bp,
                            support_papers, is_mutual
                         ) VALUES (?1,?2,?3,?4,?5,0,?6)
                         ON CONFLICT(family_a_id, family_b_id) DO UPDATE SET
                            iai_score_bp = excluded.iai_score_bp,
                            directional_a_suppresses_b_bp = excluded.directional_a_suppresses_b_bp,
                            directional_b_suppresses_a_bp = excluded.directional_b_suppresses_a_bp,
                            is_mutual = excluded.is_mutual,
                            computed_at = datetime('now')",
                            params![
                                family_ids[i],
                                family_ids[j],
                                iai,
                                a_sup_b,
                                b_sup_a,
                                is_mutual as i64
                            ],
                        )
                        .map_err(|e| EcoachError::Storage(e.to_string()))?;
                    pairs_created += 1;
                }
            }
        }
        Ok(pairs_created)
    }

    /// Compute CAS (Co-Appearance Strength) using Jaccard index
    pub fn compute_co_appearance_strength(
        &self,
        family_a_id: i64,
        family_b_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let papers_a: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT l.paper_id) FROM past_paper_question_links l
             JOIN questions q ON q.id = l.question_id WHERE q.family_id = ?1",
                [family_a_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let papers_b: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT l.paper_id) FROM past_paper_question_links l
             JOIN questions q ON q.id = l.question_id WHERE q.family_id = ?1",
                [family_b_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let papers_both: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT la.paper_id)
             FROM past_paper_question_links la
             JOIN questions qa ON qa.id = la.question_id AND qa.family_id = ?1
             JOIN past_paper_question_links lb ON lb.paper_id = la.paper_id
             JOIN questions qb ON qb.id = lb.question_id AND qb.family_id = ?2",
                params![family_a_id, family_b_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let union = papers_a + papers_b - papers_both;
        if union == 0 {
            return Ok(0);
        }

        Ok(clamp_bp(
            ((papers_both as f64 / union as f64) * 10_000.0).round() as i64,
        ))
    }

    /// Compute FRR (Family Recurrence Rate) and persist metrics
    pub fn compute_family_recurrence(
        &self,
        family_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let total_papers: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT id) FROM past_paper_sets WHERE subject_id = ?1",
                [subject_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        if total_papers == 0 {
            return Ok(0);
        }

        let papers_appeared: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT l.paper_id) FROM past_paper_question_links l
             JOIN questions q ON q.id = l.question_id
             WHERE q.family_id = ?1",
                [family_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let frr =
            clamp_bp(((papers_appeared as f64 / total_papers as f64) * 10_000.0).round() as i64);

        // Get year range for persistence/dormancy
        let years: Vec<i64> = {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT DISTINCT ps.exam_year
                 FROM past_paper_question_links l
                 JOIN questions q ON q.id = l.question_id
                 JOIN past_paper_sets ps ON ps.id = l.paper_id
                 WHERE q.family_id = ?1
                 ORDER BY ps.exam_year",
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            stmt.query_map([family_id], |row| row.get(0))
                .map_err(|e| EcoachError::Storage(e.to_string()))?
                .filter_map(|r| r.ok())
                .collect()
        };

        let first_year = years.first().copied();
        let last_year = years.last().copied();

        // Persist
        self.conn
            .execute(
                "INSERT INTO family_recurrence_metrics (
                family_id, subject_id, total_papers_in_window, papers_appeared,
                recurrence_rate_bp, first_appearance_year, last_appearance_year
             ) VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(family_id, subject_id) DO UPDATE SET
                total_papers_in_window = excluded.total_papers_in_window,
                papers_appeared = excluded.papers_appeared,
                recurrence_rate_bp = excluded.recurrence_rate_bp,
                first_appearance_year = excluded.first_appearance_year,
                last_appearance_year = excluded.last_appearance_year,
                computed_at = datetime('now')",
                params![
                    family_id,
                    subject_id,
                    total_papers,
                    papers_appeared,
                    frr,
                    first_year,
                    last_year
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(frr)
    }

    /// Compute RI (Replacement Index) for two families
    /// RI = 0.35*IAI + 0.25*ChronoShift + 0.20*TopicOverlap + 0.20*IntentSimilarity
    pub fn compute_replacement_index(
        &self,
        old_family_id: i64,
        new_family_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let (iai, _, _) = self.compute_iai(old_family_id, new_family_id, subject_id)?;

        // ChronoShift: old declining, new rising
        let old_recent: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT l.paper_id) FROM past_paper_question_links l
             JOIN questions q ON q.id = l.question_id
             JOIN past_paper_sets ps ON ps.id = l.paper_id
             WHERE q.family_id = ?1 AND ps.exam_year >= (
                 SELECT MAX(exam_year) - 4 FROM past_paper_sets WHERE subject_id = ?2
             )",
                params![old_family_id, subject_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let new_recent: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT l.paper_id) FROM past_paper_question_links l
             JOIN questions q ON q.id = l.question_id
             JOIN past_paper_sets ps ON ps.id = l.paper_id
             WHERE q.family_id = ?1 AND ps.exam_year >= (
                 SELECT MAX(exam_year) - 4 FROM past_paper_sets WHERE subject_id = ?2
             )",
                params![new_family_id, subject_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let chrono_shift = if new_recent > old_recent {
            clamp_bp(
                ((new_recent - old_recent) as f64 / (new_recent + old_recent).max(1) as f64
                    * 10_000.0)
                    .round() as i64,
            )
        } else {
            0
        };

        // TopicOverlap: same topic_id
        let topic_overlap: BasisPoints = {
            let same_topic: i64 = self
                .conn
                .query_row(
                    "SELECT CASE WHEN q1.topic_id = q2.topic_id THEN 10000 ELSE 0 END
                 FROM (SELECT topic_id FROM question_families WHERE id = ?1) q1,
                      (SELECT topic_id FROM question_families WHERE id = ?2) q2",
                    params![old_family_id, new_family_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);
            clamp_bp(same_topic)
        };

        let ri = clamp_bp(
            (iai as f64 * 0.35
                + chrono_shift as f64 * 0.25
                + topic_overlap as f64 * 0.20
                + topic_overlap as f64 * 0.20)
                .round() as i64,
        );

        // Persist
        self.conn
            .execute(
                "INSERT INTO family_replacement_trails (
                old_family_id, new_family_id, replacement_index_bp,
                iai_component_bp, chrono_shift_bp, topic_overlap_bp, cognitive_overlap_bp
             ) VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(old_family_id, new_family_id) DO UPDATE SET
                replacement_index_bp = excluded.replacement_index_bp,
                iai_component_bp = excluded.iai_component_bp,
                chrono_shift_bp = excluded.chrono_shift_bp,
                topic_overlap_bp = excluded.topic_overlap_bp,
                computed_at = datetime('now')",
                params![
                    old_family_id,
                    new_family_id,
                    ri,
                    iai,
                    chrono_shift,
                    topic_overlap,
                    topic_overlap
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(ri)
    }

    pub fn list_student_weak_families(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<StudentFamilyPerformance>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT student_id, family_id, attempt_count, accuracy_rate_bp,
                        confidence_calibration_bp, classical_form_accuracy_bp,
                        mutated_form_accuracy_bp, trap_fall_rate_bp, recovery_progress_bp
                 FROM student_family_performance
                 WHERE student_id = ?1 AND attempt_count >= 2
                 ORDER BY accuracy_rate_bp ASC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, limit as i64], |row| {
                Ok(StudentFamilyPerformance {
                    student_id: row.get(0)?,
                    family_id: row.get(1)?,
                    attempt_count: row.get(2)?,
                    accuracy_rate_bp: row.get(3)?,
                    confidence_calibration_bp: row.get(4)?,
                    classical_form_accuracy_bp: row.get(5)?,
                    mutated_form_accuracy_bp: row.get(6)?,
                    trap_fall_rate_bp: row.get(7)?,
                    recovery_progress_bp: row.get(8)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut weak = Vec::new();
        for row in rows {
            weak.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(weak)
    }

    // ── Past Questions browser ─────────────────────────────────────
    //
    // These feed the student-facing "Past Questions" page: a monochrome
    // accordion of courses → years → sections. Keep the queries flat
    // and aggregate-only; the view is a browser, not a detail page.
    // ──────────────────────────────────────────────────────────────

    /// One row per subject that has at least one linked past paper.
    /// Powers the collapsed course list.
    pub fn list_past_paper_courses(&self) -> EcoachResult<Vec<PastPaperCourseSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT s.id, s.name, s.code,
                        COUNT(DISTINCT pps.id) AS paper_count,
                        MIN(pps.exam_year)     AS first_year,
                        MAX(pps.exam_year)     AS last_year,
                        COUNT(ppql.id)         AS total_questions
                 FROM subjects s
                 INNER JOIN past_paper_sets pps            ON pps.subject_id = s.id
                 LEFT  JOIN past_paper_question_links ppql ON ppql.paper_id = pps.id
                 WHERE s.is_active = 1
                 GROUP BY s.id, s.name, s.code
                 HAVING paper_count > 0
                 ORDER BY s.display_order ASC, s.name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([], |row| {
                Ok(PastPaperCourseSummary {
                    subject_id: row.get(0)?,
                    subject_name: row.get(1)?,
                    subject_code: row.get(2)?,
                    paper_count: row.get(3)?,
                    first_year: row.get(4)?,
                    last_year: row.get(5)?,
                    total_questions: row.get(6)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    /// All past papers for a subject, newest year first. Each paper
    /// carries its section breakdown plus topic_ids / keyword tags so
    /// the frontend can render filter chips without a second round-trip.
    pub fn list_past_papers_for_subject(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Vec<PastPaperYear>> {
        // Step 1: paper-level header rows.
        let mut header_stmt = self
            .conn
            .prepare(
                "SELECT id, exam_year, title, paper_code
                 FROM past_paper_sets
                 WHERE subject_id = ?1
                 ORDER BY exam_year DESC, id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let header_rows = header_stmt
            .query_map([subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut papers: Vec<(i64, i64, String, Option<String>)> = Vec::new();
        for row in header_rows {
            papers.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        if papers.is_empty() {
            return Ok(Vec::new());
        }

        // Step 2: per-paper, per-section aggregates. We fold format
        // counts into a SectionKind.
        let mut section_stmt = self
            .conn
            .prepare(
                "SELECT COALESCE(ppql.section_label, '')       AS section_label,
                        q.question_format,
                        COUNT(*)                                AS cnt
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 WHERE ppql.paper_id = ?1
                 GROUP BY section_label, q.question_format
                 ORDER BY section_label ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        // Step 3: per-paper distinct topic_ids.
        let mut topic_stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT q.topic_id
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 WHERE ppql.paper_id = ?1 AND q.topic_id IS NOT NULL
                 ORDER BY q.topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        // Step 4a: family-name keywords when the questions belong to curated families.
        let mut family_keyword_stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT qf.family_name
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q         ON q.id = ppql.question_id
                 INNER JOIN question_families qf ON qf.id = q.family_id
                 WHERE ppql.paper_id = ?1
                 ORDER BY qf.family_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        // Step 4b: topic-name keywords. These guarantee useful filters even when
        // the paper contains authored items without a question family.
        let mut topic_keyword_stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT t.name
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 INNER JOIN topics t    ON t.id = q.topic_id
                 WHERE ppql.paper_id = ?1
                 ORDER BY t.name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        // Step 4c: question text, used to derive searchable keyword chips such as
        // 'profit', 'circle', or 'probability' when family metadata is absent.
        let mut text_keyword_stmt = self
            .conn
            .prepare(
                "SELECT q.stem, COALESCE(q.explanation_text, '')
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 WHERE ppql.paper_id = ?1
                 ORDER BY ppql.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut out: Vec<PastPaperYear> = Vec::with_capacity(papers.len());
        for (paper_id, exam_year, title, paper_code) in papers {
            // Sections: group format counts by label, classify kind.
            let section_rows = section_stmt
                .query_map([paper_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                })
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            let mut section_map: BTreeMap<String, SectionAggregate> = BTreeMap::new();
            for row in section_rows {
                let (label, format, count) =
                    row.map_err(|err| EcoachError::Storage(err.to_string()))?;
                let entry = section_map
                    .entry(label.clone())
                    .or_insert_with(|| SectionAggregate {
                        label,
                        total: 0,
                        objective: 0,
                        essay: 0,
                    });
                entry.total += count;
                if is_objective_format(&format) {
                    entry.objective += count;
                } else {
                    entry.essay += count;
                }
            }
            let sections: Vec<PastPaperSection> = section_map
                .into_values()
                .map(|agg| PastPaperSection {
                    section_label: agg.label,
                    section_kind: classify_section(agg.objective, agg.essay),
                    question_count: agg.total,
                })
                .collect();

            // Topic ids.
            let topic_rows = topic_stmt
                .query_map([paper_id], |row| row.get::<_, i64>(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let mut topic_ids: Vec<i64> = Vec::new();
            for row in topic_rows {
                topic_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }

            // Keyword chips from family names, topic names, and question text.
            let family_keyword_rows = family_keyword_stmt
                .query_map([paper_id], |row| row.get::<_, String>(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let mut family_keywords: Vec<String> = Vec::new();
            for row in family_keyword_rows {
                family_keywords.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }

            let topic_keyword_rows = topic_keyword_stmt
                .query_map([paper_id], |row| row.get::<_, String>(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let mut topic_keywords: Vec<String> = Vec::new();
            for row in topic_keyword_rows {
                topic_keywords.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }

            let text_rows = text_keyword_stmt
                .query_map([paper_id], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let mut text_fragments: Vec<String> = Vec::new();
            for row in text_rows {
                let (stem, explanation) =
                    row.map_err(|err| EcoachError::Storage(err.to_string()))?;
                text_fragments.push(stem);
                if !explanation.trim().is_empty() {
                    text_fragments.push(explanation);
                }
            }

            let keywords = build_paper_keywords(&topic_keywords, &family_keywords, &text_fragments);

            out.push(PastPaperYear {
                paper_id,
                exam_year,
                title,
                paper_code,
                sections,
                topic_ids,
                keywords,
            });
        }

        Ok(out)
    }

    /// Per-topic question tally across all past papers of a subject.
    /// Drives the Past Questions "Topic" view: for each topic tagged to
    /// at least one past-paper question, returns the topic name plus
    /// question counts split by format — objectives (mcq, true/false)
    /// vs essay (everything else). A topic appears in the list only if
    /// it has at least one question in either bucket.
    pub fn list_past_paper_topic_counts(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Vec<PastPaperTopicCount>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT t.id, t.name,
                        COUNT(*) AS question_count,
                        SUM(CASE WHEN q.question_format IN ('mcq','true_false') THEN 1 ELSE 0 END) AS objective_count,
                        SUM(CASE WHEN q.question_format NOT IN ('mcq','true_false') THEN 1 ELSE 0 END) AS essay_count
                 FROM past_paper_question_links ppql
                 INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                 INNER JOIN questions q         ON q.id = ppql.question_id
                 INNER JOIN topics t            ON t.id = q.topic_id
                 WHERE pps.subject_id = ?1
                   AND q.is_active = 1
                   AND q.topic_id IS NOT NULL
                 GROUP BY t.id, t.name
                 ORDER BY question_count DESC, t.name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([subject_id], |row| {
                Ok(PastPaperTopicCount {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    question_count: row.get(2)?,
                    objective_count: row.get(3)?,
                    essay_count: row.get(4)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut out: Vec<PastPaperTopicCount> = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(out)
    }

    /// Ordered question ids across *all* past papers of a subject that
    /// are tagged to a specific topic, filtered by question format.
    /// `format_filter` must be "objective", "essay", or "all" — the
    /// Past Questions topic view never requests the mixed "all" bucket
    /// (objectives and essays are exposed as two independent sessions
    /// per the product rule), but the filter is kept permissive for
    /// future callers. Ordered newest-year first, then by the paper's
    /// own question-number string.
    pub fn list_subject_topic_past_question_ids(
        &self,
        subject_id: i64,
        topic_id: i64,
        format_filter: &str,
    ) -> EcoachResult<Vec<i64>> {
        let sql = match format_filter {
            "objective" => {
                "SELECT ppql.question_id
                 FROM past_paper_question_links ppql
                 INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                 INNER JOIN questions q         ON q.id = ppql.question_id
                 WHERE pps.subject_id = ?1
                   AND q.topic_id = ?2
                   AND q.is_active = 1
                   AND q.question_format IN ('mcq','true_false')
                 ORDER BY pps.exam_year DESC, ppql.question_number ASC, ppql.id ASC"
            }
            "essay" => {
                "SELECT ppql.question_id
                 FROM past_paper_question_links ppql
                 INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                 INNER JOIN questions q         ON q.id = ppql.question_id
                 WHERE pps.subject_id = ?1
                   AND q.topic_id = ?2
                   AND q.is_active = 1
                   AND q.question_format NOT IN ('mcq','true_false')
                 ORDER BY pps.exam_year DESC, ppql.question_number ASC, ppql.id ASC"
            }
            _ => {
                "SELECT ppql.question_id
                 FROM past_paper_question_links ppql
                 INNER JOIN past_paper_sets pps ON pps.id = ppql.paper_id
                 INNER JOIN questions q         ON q.id = ppql.question_id
                 WHERE pps.subject_id = ?1
                   AND q.topic_id = ?2
                   AND q.is_active = 1
                 ORDER BY pps.exam_year DESC, ppql.question_number ASC, ppql.id ASC"
            }
        };

        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![subject_id, topic_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(ids)
    }

    /// Ordered list of (question_id, display_order) for a given
    /// paper+section. Used by SessionService to build a session that
    /// exactly mirrors paper order — no selector sampling.
    pub fn list_section_question_ids(
        &self,
        paper_id: i64,
        section_label: &str,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT ppql.question_id
                 FROM past_paper_question_links ppql
                 INNER JOIN questions q ON q.id = ppql.question_id
                 WHERE ppql.paper_id = ?1
                   AND COALESCE(ppql.section_label, '') = ?2
                   AND q.is_active = 1
                 ORDER BY ppql.question_number ASC, ppql.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![paper_id, section_label], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(ids)
    }

    // ── Image / asset attachments ────────────────────────────────────
    //
    // Past-paper diagrams and figures attach to a question. `scope`
    // tells us WHERE the image appears (stem / option / explanation);
    // `scope_ref` points at the specific option_id when scope='option'.
    // Bytes live in SQLite as BLOB — simpler backup story, no
    // file-system path leaks. See migration 103.
    // ────────────────────────────────────────────────────────────────

    pub fn attach_question_asset(
        &self,
        question_id: i64,
        scope: &str,
        scope_ref: Option<i64>,
        mime_type: &str,
        bytes: &[u8],
        alt_text: Option<&str>,
    ) -> EcoachResult<QuestionAssetMeta> {
        if !matches!(scope, "stem" | "option" | "explanation") {
            return Err(EcoachError::Validation(format!(
                "invalid asset scope: {}",
                scope
            )));
        }
        if bytes.is_empty() {
            return Err(EcoachError::Validation(
                "attached file is empty".to_string(),
            ));
        }

        // Highest existing position + 1 keeps insertion order stable for
        // admins who attach images sequentially.
        let next_position: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), 0) + 1
                 FROM question_assets
                 WHERE question_id = ?1 AND scope = ?2
                   AND COALESCE(scope_ref, -1) = COALESCE(?3, -1)",
                params![question_id, scope, scope_ref],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(1);

        self.conn
            .execute(
                "INSERT INTO question_assets (
                    question_id, scope, scope_ref, mime_type, byte_size, data,
                    position, alt_text
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    question_id,
                    scope,
                    scope_ref,
                    mime_type,
                    bytes.len() as i64,
                    bytes,
                    next_position,
                    alt_text,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let asset_id = self.conn.last_insert_rowid();
        Ok(QuestionAssetMeta {
            asset_id,
            question_id,
            scope: scope.to_string(),
            scope_ref,
            mime_type: mime_type.to_string(),
            byte_size: bytes.len() as i64,
            position: next_position,
            alt_text: alt_text.map(str::to_string),
        })
    }

    pub fn delete_question_asset(&self, asset_id: i64) -> EcoachResult<()> {
        let removed = self
            .conn
            .execute("DELETE FROM question_assets WHERE id = ?1", [asset_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if removed == 0 {
            return Err(EcoachError::NotFound(format!(
                "asset {} not found",
                asset_id
            )));
        }
        Ok(())
    }

    pub fn list_question_assets(
        &self,
        question_id: i64,
    ) -> EcoachResult<Vec<QuestionAssetMeta>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, question_id, scope, scope_ref, mime_type, byte_size,
                        position, alt_text
                 FROM question_assets
                 WHERE question_id = ?1
                 ORDER BY scope ASC, position ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = stmt
            .query_map([question_id], |row| {
                Ok(QuestionAssetMeta {
                    asset_id: row.get(0)?,
                    question_id: row.get(1)?,
                    scope: row.get(2)?,
                    scope_ref: row.get(3)?,
                    mime_type: row.get(4)?,
                    byte_size: row.get(5)?,
                    position: row.get(6)?,
                    alt_text: row.get(7)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    /// Bulk fetch of assets for a batch of question ids — used by the
    /// editor to show every thumbnail without an N+1 round-trip.
    pub fn list_question_assets_for_questions(
        &self,
        question_ids: &[i64],
    ) -> EcoachResult<Vec<QuestionAssetMeta>> {
        if question_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = question_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT id, question_id, scope, scope_ref, mime_type, byte_size,
                    position, alt_text
             FROM question_assets
             WHERE question_id IN ({})
             ORDER BY question_id, scope ASC, position ASC, id ASC",
            placeholders
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut params_vec: Vec<rusqlite::types::Value> =
            Vec::with_capacity(question_ids.len());
        for id in question_ids {
            params_vec.push((*id).into());
        }
        let rows = stmt
            .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
                Ok(QuestionAssetMeta {
                    asset_id: row.get(0)?,
                    question_id: row.get(1)?,
                    scope: row.get(2)?,
                    scope_ref: row.get(3)?,
                    mime_type: row.get(4)?,
                    byte_size: row.get(5)?,
                    position: row.get(6)?,
                    alt_text: row.get(7)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    /// Raw bytes + mime for a single asset. The frontend wraps these
    /// in a Blob + object URL for `<img>` rendering.
    pub fn get_question_asset_bytes(
        &self,
        asset_id: i64,
    ) -> EcoachResult<Option<(String, Vec<u8>)>> {
        self.conn
            .query_row(
                "SELECT mime_type, data FROM question_assets WHERE id = ?1",
                [asset_id],
                |row| {
                    let mime: String = row.get(0)?;
                    let data: Vec<u8> = row.get(1)?;
                    Ok((mime, data))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }
}

struct SectionAggregate {
    label: String,
    total: i64,
    objective: i64,
    essay: i64,
}

fn is_objective_format(format: &str) -> bool {
    matches!(format, "mcq" | "true_false")
}

fn build_paper_keywords(
    topic_keywords: &[String],
    family_keywords: &[String],
    text_fragments: &[String],
) -> Vec<String> {
    let mut keywords: BTreeSet<String> = BTreeSet::new();

    for phrase in topic_keywords.iter().chain(family_keywords.iter()) {
        if let Some(cleaned) = normalize_keyword_phrase(phrase) {
            keywords.insert(cleaned);
        }
    }

    let mut token_counts: BTreeMap<String, i64> = BTreeMap::new();
    for fragment in text_fragments {
        for token in extract_search_tokens(fragment) {
            *token_counts.entry(token).or_insert(0) += 1;
        }
    }

    let mut ranked_tokens: Vec<(String, i64)> = token_counts.into_iter().collect();
    ranked_tokens.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));

    for (token, _) in ranked_tokens.into_iter().take(18) {
        keywords.insert(token);
    }

    keywords.into_iter().collect()
}

fn normalize_keyword_phrase(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn extract_search_tokens(input: &str) -> Vec<String> {
    input
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter_map(|part| {
            let token = part.trim().to_ascii_lowercase();
            if token.len() < 4
                || token.chars().all(|ch| ch.is_ascii_digit())
                || STOPWORDS.contains(&token.as_str())
            {
                None
            } else {
                Some(token)
            }
        })
        .collect()
}

const STOPWORDS: &[&str] = &[
    "about",
    "above",
    "after",
    "again",
    "altogether",
    "among",
    "answer",
    "book",
    "books",
    "calculate",
    "class",
    "correct",
    "decimal",
    "degrees",
    "determine",
    "each",
    "exercise",
    "exterior",
    "fair",
    "find",
    "following",
    "from",
    "given",
    "interior",
    "lowest",
    "once",
    "opposite",
    "parallel",
    "quantity",
    "random",
    "score",
    "scored",
    "section",
    "side",
    "sides",
    "simplest",
    "solve",
    "student",
    "students",
    "table",
    "take",
    "their",
    "there",
    "these",
    "total",
    "travelled",
    "value",
    "written",
    "what",
    "when",
    "which",
    "wide",
    "with",
];

fn classify_section(objective: i64, essay: i64) -> PastPaperSectionKind {
    match (objective, essay) {
        (o, 0) if o > 0 => PastPaperSectionKind::Objective,
        (0, e) if e > 0 => PastPaperSectionKind::Essay,
        (0, 0) => PastPaperSectionKind::Objective, // Empty section: default so UI doesn't read "Mixed" on 0 Q's
        _ => PastPaperSectionKind::Mixed,
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

fn comeback_rationale(
    dormant_years: i64,
    historical_strength_score: BasisPoints,
    last_seen_year: Option<i64>,
) -> String {
    if dormant_years >= 2 && historical_strength_score >= 7_000 {
        format!(
            "Historically strong family that has been absent for {} years since {:?}, so it is primed for a comeback.",
            dormant_years, last_seen_year
        )
    } else if dormant_years >= 1 {
        "This family has gone quiet recently despite meaningful historical strength, so it deserves comeback monitoring.".to_string()
    } else {
        "Recent paper history keeps this family visible, but its comeback urgency is lower than truly dormant patterns.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

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

    #[test]
    fn comeback_candidates_reward_historical_strength_plus_dormancy() {
        let conn = Connection::open_in_memory().expect("in-memory db should open");
        seed_schema(&conn);
        seed_family_history(&conn);

        let service = PastPapersService::new(&conn);
        let comeback = service
            .list_comeback_candidate_families(1, None, 2)
            .expect("comeback candidates should compute");

        assert_eq!(comeback.len(), 2);
        assert_eq!(comeback[0].family_id, 10);
        assert!(comeback[0].dormant_years >= comeback[1].dormant_years);
        assert!(comeback[0].rationale.contains("comeback"));
    }

    #[test]
    fn paper_years_include_topic_and_text_keywords_without_family_links() {
        let conn = Connection::open_in_memory().expect("in-memory db should open");
        seed_schema(&conn);

        conn.execute(
            "CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                display_order INTEGER DEFAULT 0
            )",
            [],
        )
        .expect("topics schema should insert");

        conn.execute(
            "INSERT INTO topics (id, subject_id, name, display_order)
             VALUES (100, 1, 'Fractions, Decimals and Percentages', 1)",
            [],
        )
        .expect("topic should insert");
        conn.execute(
            "INSERT INTO past_paper_sets (id, subject_id, exam_year, title)
             VALUES (7, 1, 2024, 'Paper 2024')",
            [],
        )
        .expect("paper should insert");
        conn.execute(
            "INSERT INTO questions (id, subject_id, family_id, topic_id)
             VALUES (700, 1, NULL, 100)",
            [],
        )
        .expect("question shell should insert");
        conn.execute(
            "UPDATE questions
             SET question_format = 'mcq',
                 stem = 'Find the probability of picking a blue counter from a bag.',
                 explanation_text = 'The probability is the favourable outcomes over total outcomes.'",
            [],
        )
        .expect("question text should update");
        conn.execute(
            "INSERT INTO past_paper_question_links (paper_id, question_id, section_label, question_number)
             VALUES (7, 700, 'A', '1')",
            [],
        )
        .expect("link should insert");

        let years = PastPapersService::new(&conn)
            .list_past_papers_for_subject(1)
            .expect("paper years should load");

        assert_eq!(years.len(), 1);
        assert!(years[0].topic_ids.contains(&100));
        assert!(years[0]
            .keywords
            .iter()
            .any(|keyword| keyword.contains("Fractions")));
        assert!(years[0]
            .keywords
            .iter()
            .any(|keyword| keyword == "probability"));
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
                topic_id INTEGER,
                question_format TEXT DEFAULT 'mcq',
                stem TEXT DEFAULT '',
                explanation_text TEXT DEFAULT ''
            )",
            "CREATE TABLE past_paper_sets (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                exam_year INTEGER NOT NULL,
                paper_code TEXT,
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
