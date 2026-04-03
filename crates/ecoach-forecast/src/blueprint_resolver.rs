use ecoach_substrate::{EcoachError, EcoachResult, clamp_bp};
use rusqlite::Connection;

use crate::models::{
    BlueprintQuotas, ForecastBlueprint, MockType, StudentWeaknessSignal, TopicQuota,
};

pub struct BlueprintResolver<'a> {
    conn: &'a Connection,
}

impl<'a> BlueprintResolver<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Convert a forecast blueprint + student state + mock type into concrete quotas
    /// that the mock compiler can use to select questions.
    pub fn resolve(
        &self,
        blueprint: &ForecastBlueprint,
        mock_type: MockType,
        total_questions: usize,
        student_id: i64,
    ) -> EcoachResult<BlueprintQuotas> {
        let weaknesses = self.load_student_weaknesses(student_id)?;

        let topic_quotas =
            self.allocate_topic_quotas(blueprint, mock_type, total_questions, &weaknesses);

        let target_difficulty_mix =
            self.resolve_difficulty_mix(blueprint, mock_type, total_questions);

        let target_format_mix = self.resolve_format_mix(blueprint, total_questions);

        let min_surprise_items = match mock_type {
            MockType::Shock => (total_questions / 4).max(2),
            MockType::Forecast | MockType::FinalExam => (total_questions / 10).max(1),
            _ => 0,
        };

        Ok(BlueprintQuotas {
            mock_type,
            total_questions,
            topic_quotas,
            target_difficulty_mix,
            target_format_mix,
            min_surprise_items,
        })
    }

    // -----------------------------------------------------------------------
    // Topic quota allocation
    // -----------------------------------------------------------------------

    fn allocate_topic_quotas(
        &self,
        blueprint: &ForecastBlueprint,
        mock_type: MockType,
        total_questions: usize,
        weaknesses: &[StudentWeaknessSignal],
    ) -> Vec<TopicQuota> {
        if blueprint.topic_scores.is_empty() {
            return Vec::new();
        }

        // Weight adjustments per mock type
        let (blueprint_weight, weakness_weight) = match mock_type {
            MockType::Forecast | MockType::FinalExam => (0.70, 0.30),
            MockType::Diagnostic => (0.30, 0.70),
            MockType::Remediation => (0.20, 0.80),
            MockType::Shock => (0.50, 0.20),
            MockType::Wisdom => (0.60, 0.40),
        };

        // Compute blended scores per topic
        let mut scored: Vec<(i64, f64, bool)> = blueprint
            .topic_scores
            .iter()
            .map(|ts| {
                let weakness_boost = weaknesses
                    .iter()
                    .find(|w| w.topic_id == ts.topic_id)
                    .map(|w| w.gap_score as f64 / 10_000.0)
                    .unwrap_or(0.0);

                let blended = blueprint_weight * (ts.composite_score as f64 / 10_000.0)
                    + weakness_weight * weakness_boost;

                let is_surprise = ts.uncertainty_band == crate::models::UncertaintyBand::High
                    && mock_type == MockType::Shock;

                (ts.topic_id, blended, is_surprise)
            })
            .collect();

        // For Remediation: only keep weak topics
        if mock_type == MockType::Remediation {
            scored.retain(|(tid, _, _)| {
                weaknesses
                    .iter()
                    .any(|w| w.topic_id == *tid && w.gap_score >= 4000)
            });
        }

        // For Wisdom: only keep topics where student has reasonable mastery
        if mock_type == MockType::Wisdom {
            scored.retain(|(tid, _, _)| {
                weaknesses
                    .iter()
                    .find(|w| w.topic_id == *tid)
                    .map(|w| w.mastery_score >= 5000)
                    .unwrap_or(true)
            });
        }

        if scored.is_empty() {
            // Fallback: use all blueprint topics
            scored = blueprint
                .topic_scores
                .iter()
                .map(|ts| (ts.topic_id, ts.composite_score as f64 / 10_000.0, false))
                .collect();
        }

        // Normalize blended scores to sum to 1.0
        let total_score: f64 = scored.iter().map(|(_, s, _)| s).sum();
        let safe_total = if total_score <= 0.0 { 1.0 } else { total_score };

        // Allocate questions proportionally
        let mut quotas: Vec<TopicQuota> = scored
            .iter()
            .map(|(tid, score, is_surprise)| {
                let fraction = score / safe_total;
                let target = (fraction * total_questions as f64).round() as usize;
                TopicQuota {
                    topic_id: *tid,
                    target_count: target.max(1),
                    is_surprise: *is_surprise,
                }
            })
            .collect();

        // Trim or pad to match total_questions
        let allocated: usize = quotas.iter().map(|q| q.target_count).sum();
        if allocated > total_questions {
            // Trim from lowest-scored topics
            quotas.sort_by(|a, b| a.target_count.cmp(&b.target_count));
            let mut excess = allocated - total_questions;
            for quota in &mut quotas {
                if excess == 0 {
                    break;
                }
                if quota.target_count > 1 {
                    let reduce = (quota.target_count - 1).min(excess);
                    quota.target_count -= reduce;
                    excess -= reduce;
                }
            }
        } else if allocated < total_questions {
            // Add to highest-scored topics
            quotas.sort_by(|a, b| b.target_count.cmp(&a.target_count));
            let mut deficit = total_questions - allocated;
            for quota in &mut quotas {
                if deficit == 0 {
                    break;
                }
                quota.target_count += 1;
                deficit -= 1;
            }
        }

        // Sort by topic_id for stable output
        quotas.sort_by_key(|q| q.topic_id);
        quotas
    }

    // -----------------------------------------------------------------------
    // Difficulty and format mix
    // -----------------------------------------------------------------------

    fn resolve_difficulty_mix(
        &self,
        blueprint: &ForecastBlueprint,
        mock_type: MockType,
        total_questions: usize,
    ) -> Vec<(String, usize)> {
        if blueprint.difficulty_distribution.is_empty() {
            return vec![
                ("easy".into(), total_questions / 3),
                ("medium".into(), total_questions / 3),
                ("hard".into(), total_questions - 2 * (total_questions / 3)),
            ];
        }

        let difficulty_bias = match mock_type {
            MockType::Wisdom => 0.20, // push harder
            MockType::Shock => 0.15,
            MockType::Remediation => -0.15, // push easier
            _ => 0.0,
        };

        blueprint
            .difficulty_distribution
            .iter()
            .map(|ds| {
                let adjusted = (ds.probability_score as f64 / 10_000.0) + difficulty_bias;
                let count = (adjusted.max(0.05) * total_questions as f64).round() as usize;
                (ds.difficulty_band.clone(), count.max(1))
            })
            .collect()
    }

    fn resolve_format_mix(
        &self,
        blueprint: &ForecastBlueprint,
        total_questions: usize,
    ) -> Vec<(String, usize)> {
        if blueprint.format_distribution.is_empty() {
            return vec![("mcq".into(), total_questions)];
        }

        blueprint
            .format_distribution
            .iter()
            .map(|fs| {
                let count = ((fs.probability_score as f64 / 10_000.0) * total_questions as f64)
                    .round() as usize;
                (fs.format_code.clone(), count.max(1))
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Student state loading
    // -----------------------------------------------------------------------

    fn load_student_weaknesses(&self, student_id: i64) -> EcoachResult<Vec<StudentWeaknessSignal>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT topic_id, gap_score, mastery_score
                 FROM student_topic_states
                 WHERE student_id = ?1
                 ORDER BY gap_score DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([student_id], |row| {
                Ok(StudentWeaknessSignal {
                    topic_id: row.get(0)?,
                    gap_score: clamp_bp(row.get::<_, i64>(1)?),
                    mastery_score: clamp_bp(row.get::<_, i64>(2)?),
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut signals = Vec::new();
        for row in rows {
            signals.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(signals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    #[test]
    fn resolver_allocates_proportional_quotas() {
        let conn = Connection::open_in_memory().expect("open");
        conn.execute_batch(
            "CREATE TABLE student_topic_states (
                 id INTEGER PRIMARY KEY, student_id INTEGER, topic_id INTEGER,
                 gap_score INTEGER DEFAULT 0, mastery_score INTEGER DEFAULT 5000
             );",
        )
        .unwrap();

        let blueprint = make_blueprint();
        let resolver = BlueprintResolver::new(&conn);
        let quotas = resolver
            .resolve(&blueprint, MockType::Forecast, 20, 1)
            .expect("resolve");

        assert_eq!(quotas.mock_type, MockType::Forecast);
        let total_allocated: usize = quotas.topic_quotas.iter().map(|q| q.target_count).sum();
        assert_eq!(total_allocated, 20);
        assert!(quotas.topic_quotas.len() >= 2);
    }

    #[test]
    fn diagnostic_mock_emphasizes_weak_topics() {
        let conn = Connection::open_in_memory().expect("open");
        conn.execute_batch(
            "CREATE TABLE student_topic_states (
                 id INTEGER PRIMARY KEY, student_id INTEGER, topic_id INTEGER,
                 gap_score INTEGER DEFAULT 0, mastery_score INTEGER DEFAULT 5000
             );
             INSERT INTO student_topic_states (student_id, topic_id, gap_score, mastery_score)
             VALUES (1, 200, 8000, 2000);",
        )
        .unwrap();

        let blueprint = make_blueprint();
        let resolver = BlueprintResolver::new(&conn);
        let quotas = resolver
            .resolve(&blueprint, MockType::Diagnostic, 10, 1)
            .expect("resolve");

        // Topic 200 has high gap_score so should get more questions in diagnostic mode
        let topic_200 = quotas.topic_quotas.iter().find(|q| q.topic_id == 200);
        let topic_100 = quotas.topic_quotas.iter().find(|q| q.topic_id == 100);
        assert!(topic_200.is_some());
        assert!(topic_100.is_some());
        // Diagnostic mode: weakness_weight = 0.70, so topic 200 should get >= topic 100
        assert!(
            topic_200.unwrap().target_count >= topic_100.unwrap().target_count,
            "Weak topic (200) should get more or equal questions in diagnostic mode"
        );
    }

    #[test]
    fn shock_mock_includes_surprise_items() {
        let conn = Connection::open_in_memory().expect("open");
        conn.execute_batch(
            "CREATE TABLE student_topic_states (
                 id INTEGER PRIMARY KEY, student_id INTEGER, topic_id INTEGER,
                 gap_score INTEGER DEFAULT 0, mastery_score INTEGER DEFAULT 5000
             );",
        )
        .unwrap();

        let blueprint = make_blueprint();
        let resolver = BlueprintResolver::new(&conn);
        let quotas = resolver
            .resolve(&blueprint, MockType::Shock, 20, 1)
            .expect("resolve");

        assert!(quotas.min_surprise_items >= 2);
    }

    fn make_blueprint() -> ForecastBlueprint {
        ForecastBlueprint {
            snapshot_id: 1,
            subject_id: 1,
            total_papers_analyzed: 5,
            year_range_start: Some(2019),
            year_range_end: Some(2023),
            confidence_score: 7000,
            topic_scores: vec![
                ForecastTopicScore {
                    topic_id: 100,
                    topic_name: "Algebra".into(),
                    frequency_score: 8000,
                    recency_score: 9000,
                    trend_score: 6000,
                    bundle_strength: 5000,
                    syllabus_priority: 7000,
                    style_regime_fit: 5000,
                    examiner_goal_fit: 6000,
                    composite_score: 7200,
                    uncertainty_band: UncertaintyBand::Low,
                },
                ForecastTopicScore {
                    topic_id: 200,
                    topic_name: "Geometry".into(),
                    frequency_score: 5000,
                    recency_score: 6000,
                    trend_score: 5000,
                    bundle_strength: 4000,
                    syllabus_priority: 5000,
                    style_regime_fit: 5000,
                    examiner_goal_fit: 5000,
                    composite_score: 5100,
                    uncertainty_band: UncertaintyBand::High,
                },
            ],
            format_distribution: vec![
                ForecastFormatScore {
                    format_code: "mcq".into(),
                    probability_score: 6000,
                },
                ForecastFormatScore {
                    format_code: "structured".into(),
                    probability_score: 4000,
                },
            ],
            difficulty_distribution: vec![
                ForecastDifficultyScore {
                    difficulty_band: "easy".into(),
                    probability_score: 3000,
                },
                ForecastDifficultyScore {
                    difficulty_band: "medium".into(),
                    probability_score: 5000,
                },
                ForecastDifficultyScore {
                    difficulty_band: "hard".into(),
                    probability_score: 2000,
                },
            ],
            bundles: vec![ForecastBundle {
                bundle_key: "bundle_1".into(),
                topic_ids: vec![100, 200],
                co_occurrence_count: 3,
                strength_score: 7500,
            }],
            computed_at: "2024-01-01T00:00:00Z".into(),
        }
    }
}
