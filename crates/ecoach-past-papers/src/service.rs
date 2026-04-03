use std::collections::{BTreeMap, BTreeSet};

use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{
    CreateFamilyEdgeInput, FamilyRecurrenceMetric, FamilyRelationshipEdge, FamilyReplacementTrail,
    FamilyStory, InverseAppearancePair, PaperDna, PastPaperComebackSignal,
    PastPaperFamilyAnalytics, PastPaperInverseSignal, PastPaperSet, PastPaperSetSummary,
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
