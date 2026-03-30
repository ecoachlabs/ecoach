use std::collections::BTreeMap;

use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, params};
use serde_json;

use crate::models::{
    ForecastBlueprint, ForecastBundle, ForecastDifficultyScore, ForecastFormatScore,
    ForecastTopicScore, PatternProfile, UncertaintyBand,
};
use crate::pattern_miner::PatternMiner;

pub struct ForecastEngine<'a> {
    conn: &'a Connection,
}

// ---------------------------------------------------------------------------
// ForecastScore(u) = 0.25*Freq + 0.20*Recency + 0.15*Trend
//                  + 0.15*BundleStrength + 0.10*SyllabusPriority
//                  + 0.10*StyleRegimeFit + 0.05*ExaminerGoalFit
// ---------------------------------------------------------------------------
const W_FREQUENCY: f64 = 0.25;
const W_RECENCY: f64 = 0.20;
const W_TREND: f64 = 0.15;
const W_BUNDLE: f64 = 0.15;
const W_SYLLABUS: f64 = 0.10;
const W_STYLE_REGIME: f64 = 0.10;
const W_EXAMINER_GOAL: f64 = 0.05;

impl<'a> ForecastEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Compute a full forecast blueprint for a subject and persist it.
    pub fn compute_blueprint(&self, subject_id: i64) -> EcoachResult<ForecastBlueprint> {
        let miner = PatternMiner::new(self.conn);

        let profile = miner.build_pattern_profile(subject_id)?;
        let topic_presence = miner.mine_topic_presence(subject_id)?;
        let co_occurrences = miner.mine_co_occurrences(subject_id)?;
        let format_by_year = miner.mine_format_distribution_by_year(subject_id)?;

        if profile.total_papers == 0 {
            return Err(EcoachError::Validation(
                "No past papers found for this subject; cannot compute forecast.".into(),
            ));
        }

        let year_min = profile.year_range_start.unwrap_or(0);
        let year_max = profile.year_range_end.unwrap_or(0);
        let year_span = (year_max - year_min).max(1) as f64;

        // Bundle strength map: topic_id -> strength (max co-occurrence count)
        let max_co = co_occurrences
            .iter()
            .map(|c| c.papers_together)
            .max()
            .unwrap_or(1)
            .max(1);
        let mut bundle_strength_map: BTreeMap<i64, i64> = BTreeMap::new();
        for co in &co_occurrences {
            let score = scale(co.papers_together, max_co);
            let existing_a = bundle_strength_map.entry(co.topic_a).or_insert(0);
            if score > *existing_a {
                *existing_a = score;
            }
            let existing_b = bundle_strength_map.entry(co.topic_b).or_insert(0);
            if score > *existing_b {
                *existing_b = score;
            }
        }

        // Style regime fit: how much a topic's most recent format matches the overall
        // latest-year format distribution.
        let latest_year_formats = self.latest_year_format_distribution(&format_by_year, year_max);

        // Compute per-topic scores
        let max_questions = topic_presence
            .iter()
            .map(|t| t.total_questions)
            .max()
            .unwrap_or(1)
            .max(1);

        let mut topic_scores: Vec<ForecastTopicScore> = Vec::new();

        for topic in &topic_presence {
            let frequency = scale(topic.total_questions, max_questions);

            let recency = if let Some(&last_year) = topic.years_present.last() {
                let gap = year_max - last_year;
                // More recent → higher score: 10000 when gap=0, decays toward 0.
                clamp_bp(10_000 - scale(gap, year_span as i64) as i64) as i64
            } else {
                0
            };

            let trend = self.compute_trend(&topic.years_present, year_min, year_max);

            let bundle = *bundle_strength_map.get(&topic.topic_id).unwrap_or(&0);

            let syllabus_priority = self.load_syllabus_priority(topic.topic_id)?;

            let style_regime = self.compute_style_regime_fit(
                &topic.question_formats,
                &latest_year_formats,
            );

            let examiner_goal = self.compute_examiner_goal_fit(
                &topic.cognitive_demands,
                topic.total_questions,
                topic.years_present.len() as i64,
            );

            let composite = clamp_bp(
                (W_FREQUENCY * frequency as f64
                    + W_RECENCY * recency as f64
                    + W_TREND * trend as f64
                    + W_BUNDLE * bundle as f64
                    + W_SYLLABUS * syllabus_priority as f64
                    + W_STYLE_REGIME * style_regime as f64
                    + W_EXAMINER_GOAL * examiner_goal as f64)
                    .round() as i64,
            );

            let uncertainty = UncertaintyBand::from_composite(
                composite,
                topic.years_present.len() as i64,
            );

            topic_scores.push(ForecastTopicScore {
                topic_id: topic.topic_id,
                topic_name: topic.topic_name.clone(),
                frequency_score: clamp_bp(frequency),
                recency_score: clamp_bp(recency),
                trend_score: clamp_bp(trend),
                bundle_strength: clamp_bp(bundle),
                syllabus_priority: clamp_bp(syllabus_priority),
                style_regime_fit: clamp_bp(style_regime),
                examiner_goal_fit: clamp_bp(examiner_goal),
                composite_score: composite,
                uncertainty_band: uncertainty,
            });
        }

        // Sort by composite descending
        topic_scores.sort_by(|a, b| b.composite_score.cmp(&a.composite_score));

        // Format distribution
        let total_format_count: i64 = profile.format_counts.iter().map(|(_, c)| c).sum();
        let format_distribution: Vec<ForecastFormatScore> = profile
            .format_counts
            .iter()
            .map(|(code, count)| ForecastFormatScore {
                format_code: code.clone(),
                probability_score: clamp_bp(
                    ((*count as f64 / total_format_count.max(1) as f64) * 10_000.0).round()
                        as i64,
                ),
            })
            .collect();

        // Difficulty distribution
        let total_diff_count: i64 = profile.difficulty_counts.iter().map(|(_, c)| c).sum();
        let difficulty_distribution: Vec<ForecastDifficultyScore> = profile
            .difficulty_counts
            .iter()
            .map(|(band, count)| ForecastDifficultyScore {
                difficulty_band: band.clone(),
                probability_score: clamp_bp(
                    ((*count as f64 / total_diff_count.max(1) as f64) * 10_000.0).round() as i64,
                ),
            })
            .collect();

        // Build bundles from top co-occurrences
        let bundles = self.build_forecast_bundles(&co_occurrences, max_co);

        // Overall confidence: based on paper count + topic coverage
        let confidence = clamp_bp(
            ((profile.total_papers.min(20) as f64 / 20.0) * 5_000.0
                + (profile.topic_count.min(20) as f64 / 20.0) * 5_000.0)
                .round() as i64,
        );

        // Persist
        let snapshot_id =
            self.persist_snapshot(subject_id, &profile, confidence, &topic_scores,
                &format_distribution, &difficulty_distribution, &bundles)?;

        Ok(ForecastBlueprint {
            snapshot_id,
            subject_id,
            total_papers_analyzed: profile.total_papers,
            year_range_start: profile.year_range_start,
            year_range_end: profile.year_range_end,
            confidence_score: confidence,
            topic_scores,
            format_distribution,
            difficulty_distribution,
            bundles,
            computed_at: ecoach_substrate::now_utc().to_string(),
        })
    }

    /// Load the most recent forecast blueprint for a subject.
    pub fn get_latest_blueprint(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Option<ForecastBlueprint>> {
        let row = self
            .conn
            .query_row(
                "SELECT id, total_papers_analyzed, year_range_start, year_range_end,
                        blueprint_json, confidence_score, computed_at
                 FROM forecast_snapshots
                 WHERE subject_id = ?1
                 ORDER BY computed_at DESC
                 LIMIT 1",
                [subject_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, String>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let Some((snapshot_id, total_papers, year_start, year_end, blueprint_json, confidence, computed_at)) = row else {
            return Ok(None);
        };

        let stored: StoredBlueprint = serde_json::from_str(&blueprint_json)
            .map_err(|e| EcoachError::Storage(format!("blueprint json parse: {e}")))?;

        Ok(Some(ForecastBlueprint {
            snapshot_id,
            subject_id,
            total_papers_analyzed: total_papers,
            year_range_start: year_start,
            year_range_end: year_end,
            confidence_score: clamp_bp(confidence),
            topic_scores: stored.topic_scores,
            format_distribution: stored.format_distribution,
            difficulty_distribution: stored.difficulty_distribution,
            bundles: stored.bundles,
            computed_at,
        }))
    }

    // -----------------------------------------------------------------------
    // Scoring helpers
    // -----------------------------------------------------------------------

    /// Trend: is this topic appearing more or less frequently over time?
    /// Rising trend → high score. Declining → low score. Stable → mid.
    fn compute_trend(
        &self,
        years_present: &[i64],
        year_min: i64,
        year_max: i64,
    ) -> i64 {
        if years_present.len() < 2 {
            return 5000; // neutral
        }
        let midpoint = (year_min + year_max) / 2;
        let early_count = years_present.iter().filter(|&&y| y <= midpoint).count() as f64;
        let late_count = years_present.iter().filter(|&&y| y > midpoint).count() as f64;
        let total = (early_count + late_count).max(1.0);

        // Ratio of late appearances: 1.0 = all recent, 0.0 = all early
        let late_ratio = late_count / total;

        // Map [0,1] to [0, 10000] with 5000 as neutral
        clamp_bp((late_ratio * 10_000.0).round() as i64) as i64
    }

    /// Syllabus priority: use exam_weight and importance_weight from topics table.
    fn load_syllabus_priority(&self, topic_id: i64) -> EcoachResult<i64> {
        let result = self
            .conn
            .query_row(
                "SELECT COALESCE(exam_weight, 0), COALESCE(importance_weight, 0)
                 FROM topics WHERE id = ?1",
                [topic_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        match result {
            Some((exam_w, importance_w)) => {
                // Blend: 60% exam weight, 40% importance weight
                Ok(clamp_bp(
                    (0.60 * exam_w as f64 + 0.40 * importance_w as f64).round() as i64,
                ) as i64)
            }
            None => Ok(5000), // neutral default
        }
    }

    /// Style regime fit: how well a topic's question formats match the latest year's
    /// format distribution. Topics that use currently-favored formats score higher.
    fn compute_style_regime_fit(
        &self,
        topic_formats: &[String],
        latest_formats: &BTreeMap<String, f64>,
    ) -> i64 {
        if topic_formats.is_empty() || latest_formats.is_empty() {
            return 5000;
        }
        let mut total_match: f64 = 0.0;
        for fmt in topic_formats {
            total_match += latest_formats.get(fmt).unwrap_or(&0.0);
        }
        // Normalize by number of formats the topic uses
        let avg_match = total_match / topic_formats.len() as f64;
        clamp_bp((avg_match * 10_000.0).round() as i64) as i64
    }

    /// Examiner goal fit: topics tested with higher cognitive demands and more
    /// consistently across years are more likely exam targets.
    fn compute_examiner_goal_fit(
        &self,
        cognitive_demands: &[String],
        _total_questions: i64,
        years_count: i64,
    ) -> i64 {
        // Higher-order cognitive demands score more
        let demand_score: f64 = cognitive_demands
            .iter()
            .map(|d| match d.as_str() {
                "recall" | "knowledge" => 0.3,
                "comprehension" | "understanding" => 0.5,
                "application" => 0.7,
                "analysis" => 0.85,
                "synthesis" | "evaluation" | "creation" => 1.0,
                _ => 0.5,
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.5);

        // Consistency bonus: appearing across many years
        let consistency = (years_count as f64 / 10.0).min(1.0);

        let score = (0.60 * demand_score + 0.40 * consistency) * 10_000.0;
        clamp_bp(score.round() as i64) as i64
    }

    fn latest_year_format_distribution(
        &self,
        format_by_year: &[(i64, String, i64)],
        latest_year: i64,
    ) -> BTreeMap<String, f64> {
        let mut latest: BTreeMap<String, i64> = BTreeMap::new();
        let mut total = 0i64;
        for (year, fmt, count) in format_by_year {
            if *year == latest_year {
                *latest.entry(fmt.clone()).or_insert(0) += count;
                total += count;
            }
        }
        latest
            .into_iter()
            .map(|(fmt, count)| (fmt, count as f64 / total.max(1) as f64))
            .collect()
    }

    fn build_forecast_bundles(
        &self,
        co_occurrences: &[crate::models::TopicCoOccurrence],
        max_co: i64,
    ) -> Vec<ForecastBundle> {
        co_occurrences
            .iter()
            .take(20) // top 20 bundles
            .enumerate()
            .map(|(i, co)| {
                let strength = scale(co.papers_together, max_co);
                ForecastBundle {
                    bundle_key: format!("bundle_{}", i + 1),
                    topic_ids: vec![co.topic_a, co.topic_b],
                    co_occurrence_count: co.papers_together,
                    strength_score: clamp_bp(strength),
                }
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Persistence
    // -----------------------------------------------------------------------

    fn persist_snapshot(
        &self,
        subject_id: i64,
        profile: &PatternProfile,
        confidence: BasisPoints,
        topic_scores: &[ForecastTopicScore],
        format_dist: &[ForecastFormatScore],
        difficulty_dist: &[ForecastDifficultyScore],
        bundles: &[ForecastBundle],
    ) -> EcoachResult<i64> {
        let stored = StoredBlueprint {
            topic_scores: topic_scores.to_vec(),
            format_distribution: format_dist.to_vec(),
            difficulty_distribution: difficulty_dist.to_vec(),
            bundles: bundles.to_vec(),
        };
        let blueprint_json = serde_json::to_string(&stored)
            .map_err(|e| EcoachError::Storage(format!("serialize blueprint: {e}")))?;
        let pattern_json = serde_json::to_string(profile)
            .map_err(|e| EcoachError::Storage(format!("serialize profile: {e}")))?;

        self.conn
            .execute(
                "INSERT INTO forecast_snapshots
                    (subject_id, total_papers_analyzed, year_range_start, year_range_end,
                     blueprint_json, pattern_profile_json, confidence_score)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    subject_id,
                    profile.total_papers,
                    profile.year_range_start,
                    profile.year_range_end,
                    blueprint_json,
                    pattern_json,
                    confidence as i64,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let snapshot_id = self.conn.last_insert_rowid();

        // Persist topic scores
        for ts in topic_scores {
            self.conn
                .execute(
                    "INSERT INTO forecast_topic_scores
                        (snapshot_id, topic_id, frequency_score, recency_score, trend_score,
                         bundle_strength, syllabus_priority, style_regime_fit, examiner_goal_fit,
                         composite_score, uncertainty_band)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        snapshot_id,
                        ts.topic_id,
                        ts.frequency_score as i64,
                        ts.recency_score as i64,
                        ts.trend_score as i64,
                        ts.bundle_strength as i64,
                        ts.syllabus_priority as i64,
                        ts.style_regime_fit as i64,
                        ts.examiner_goal_fit as i64,
                        ts.composite_score as i64,
                        ts.uncertainty_band.as_str(),
                    ],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        // Persist format scores
        for fs in format_dist {
            self.conn
                .execute(
                    "INSERT INTO forecast_format_scores (snapshot_id, format_code, probability_score)
                     VALUES (?1, ?2, ?3)",
                    params![snapshot_id, fs.format_code, fs.probability_score as i64],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        // Persist difficulty scores
        for ds in difficulty_dist {
            self.conn
                .execute(
                    "INSERT INTO forecast_difficulty_scores (snapshot_id, difficulty_band, probability_score)
                     VALUES (?1, ?2, ?3)",
                    params![snapshot_id, ds.difficulty_band, ds.probability_score as i64],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        // Persist bundles
        for bundle in bundles {
            let topic_ids_json = serde_json::to_string(&bundle.topic_ids)
                .map_err(|e| EcoachError::Storage(format!("bundle json: {e}")))?;
            self.conn
                .execute(
                    "INSERT INTO forecast_bundles
                        (snapshot_id, bundle_key, topic_ids_json, co_occurrence_count, strength_score)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        snapshot_id,
                        bundle.bundle_key,
                        topic_ids_json,
                        bundle.co_occurrence_count,
                        bundle.strength_score as i64,
                    ],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        Ok(snapshot_id)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn scale(value: i64, max_value: i64) -> i64 {
    if max_value <= 0 {
        return 0;
    }
    clamp_bp(((value as f64 / max_value as f64) * 10_000.0).round() as i64) as i64
}

/// Stored in blueprint_json column for fast retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredBlueprint {
    topic_scores: Vec<ForecastTopicScore>,
    format_distribution: Vec<ForecastFormatScore>,
    difficulty_distribution: Vec<ForecastDifficultyScore>,
    bundles: Vec<ForecastBundle>,
}

use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forecast_engine_computes_and_persists_blueprint() {
        let conn = Connection::open_in_memory().expect("open");
        seed_full_schema(&conn);
        seed_past_paper_data(&conn);

        let engine = ForecastEngine::new(&conn);
        let blueprint = engine.compute_blueprint(1).expect("compute");

        assert_eq!(blueprint.subject_id, 1);
        assert_eq!(blueprint.total_papers_analyzed, 3);
        assert!(!blueprint.topic_scores.is_empty());
        assert!(blueprint.confidence_score > 0);

        // Verify persistence
        let loaded = engine
            .get_latest_blueprint(1)
            .expect("load")
            .expect("should exist");
        assert_eq!(loaded.snapshot_id, blueprint.snapshot_id);
        assert_eq!(loaded.topic_scores.len(), blueprint.topic_scores.len());
    }

    #[test]
    fn topic_with_higher_frequency_and_recency_scores_higher() {
        let conn = Connection::open_in_memory().expect("open");
        seed_full_schema(&conn);
        seed_past_paper_data(&conn);

        let engine = ForecastEngine::new(&conn);
        let blueprint = engine.compute_blueprint(1).expect("compute");

        // Topic 100 (Algebra) appears in all 3 papers, topic 200 (Geometry) in 2.
        let algebra = blueprint.topic_scores.iter().find(|t| t.topic_id == 100);
        let geometry = blueprint.topic_scores.iter().find(|t| t.topic_id == 200);

        assert!(algebra.is_some());
        assert!(geometry.is_some());
        assert!(
            algebra.unwrap().composite_score >= geometry.unwrap().composite_score,
            "Algebra (3 papers) should score >= Geometry (2 papers)"
        );
    }

    #[test]
    fn no_papers_returns_validation_error() {
        let conn = Connection::open_in_memory().expect("open");
        seed_full_schema(&conn);
        // No data seeded

        let engine = ForecastEngine::new(&conn);
        let result = engine.compute_blueprint(999);
        assert!(result.is_err());
    }

    fn seed_full_schema(conn: &Connection) {
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
             );
             CREATE TABLE forecast_snapshots (
                 id INTEGER PRIMARY KEY, subject_id INTEGER NOT NULL,
                 total_papers_analyzed INTEGER NOT NULL DEFAULT 0,
                 year_range_start INTEGER, year_range_end INTEGER,
                 blueprint_json TEXT NOT NULL DEFAULT '{}',
                 pattern_profile_json TEXT NOT NULL DEFAULT '{}',
                 confidence_score INTEGER NOT NULL DEFAULT 0,
                 computed_at TEXT NOT NULL DEFAULT (datetime('now'))
             );
             CREATE TABLE forecast_topic_scores (
                 id INTEGER PRIMARY KEY, snapshot_id INTEGER NOT NULL,
                 topic_id INTEGER NOT NULL, frequency_score INTEGER NOT NULL DEFAULT 0,
                 recency_score INTEGER NOT NULL DEFAULT 0, trend_score INTEGER NOT NULL DEFAULT 0,
                 bundle_strength INTEGER NOT NULL DEFAULT 0, syllabus_priority INTEGER NOT NULL DEFAULT 0,
                 style_regime_fit INTEGER NOT NULL DEFAULT 0, examiner_goal_fit INTEGER NOT NULL DEFAULT 0,
                 composite_score INTEGER NOT NULL DEFAULT 0, uncertainty_band TEXT NOT NULL DEFAULT 'medium'
             );
             CREATE TABLE forecast_format_scores (
                 id INTEGER PRIMARY KEY, snapshot_id INTEGER NOT NULL,
                 format_code TEXT NOT NULL, probability_score INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE forecast_difficulty_scores (
                 id INTEGER PRIMARY KEY, snapshot_id INTEGER NOT NULL,
                 difficulty_band TEXT NOT NULL, probability_score INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE forecast_bundles (
                 id INTEGER PRIMARY KEY, snapshot_id INTEGER NOT NULL,
                 bundle_key TEXT NOT NULL, topic_ids_json TEXT NOT NULL DEFAULT '[]',
                 co_occurrence_count INTEGER NOT NULL DEFAULT 0,
                 strength_score INTEGER NOT NULL DEFAULT 0
             );",
        )
        .expect("schema");
    }

    fn seed_past_paper_data(conn: &Connection) {
        conn.execute("INSERT INTO subjects (id, name) VALUES (1, 'Mathematics')", [])
            .unwrap();
        conn.execute(
            "INSERT INTO topics (id, subject_id, name, node_type, exam_weight, importance_weight)
             VALUES (100, 1, 'Algebra', 'topic', 7000, 8000)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO topics (id, subject_id, name, node_type, exam_weight, importance_weight)
             VALUES (200, 1, 'Geometry', 'topic', 5000, 6000)",
            [],
        )
        .unwrap();

        for (pid, year) in [(1, 2021), (2, 2022), (3, 2023)] {
            conn.execute(
                "INSERT INTO past_paper_sets (id, subject_id, exam_year, title) VALUES (?1, 1, ?2, 'Paper')",
                params![pid, year],
            )
            .unwrap();
        }

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

        for (pid, qid) in [(1, 1), (1, 4), (2, 2), (2, 5), (3, 3)] {
            conn.execute(
                "INSERT INTO past_paper_question_links (paper_id, question_id) VALUES (?1, ?2)",
                params![pid, qid],
            )
            .unwrap();
        }
    }
}
