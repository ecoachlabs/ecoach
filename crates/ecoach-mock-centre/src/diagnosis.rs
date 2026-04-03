use std::collections::BTreeMap;

use ecoach_forecast::ForecastEngine;
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json;

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockDeepDiagnosis {
    pub mock_session_id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_weaknesses: Vec<TopicWeaknessSlice>,
    pub topic_strengths: Vec<TopicStrengthSlice>,
    pub broken_links: Vec<BrokenLinkDiagnosis>,
    pub misconception_hits: Vec<MisconceptionHit>,
    pub representation_gaps: Vec<RepresentationGap>,
    pub timing_diagnosis: TimingDiagnosis,
    pub confidence_diagnosis: ConfidenceDiagnosis,
    pub predicted_exam_score: Option<PredictedScore>,
    pub action_plan: Vec<RecommendedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicWeaknessSlice {
    pub topic_id: i64,
    pub topic_name: String,
    pub weakness_score: BasisPoints,
    pub mastery_gap: BasisPoints,
    pub misconception_pressure: BasisPoints,
    pub timed_gap: BasisPoints,
    pub accuracy_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStrengthSlice {
    pub topic_id: i64,
    pub topic_name: String,
    pub accuracy_bp: BasisPoints,
    pub correct: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenLinkDiagnosis {
    pub from_node_id: i64,
    pub to_node_id: i64,
    pub edge_type: String,
    pub from_title: String,
    pub to_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MisconceptionHit {
    pub misconception_id: i64,
    pub topic_id: i64,
    pub title: String,
    pub hit_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentationGap {
    pub format_code: String,
    pub accuracy_bp: BasisPoints,
    pub total_questions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingDiagnosis {
    pub avg_response_time_ms: i64,
    pub slow_but_correct_count: i64,
    pub fast_but_wrong_count: i64,
    pub collapsed_near_end: bool,
    pub pacing_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceDiagnosis {
    pub correct_and_confident: i64,
    pub correct_but_unsure: i64,
    pub wrong_but_confident: i64,
    pub wrong_and_unsure: i64,
    pub guessing_rate_bp: BasisPoints,
    pub overconfidence_rate_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedScore {
    pub predicted_score_bp: BasisPoints,
    pub predicted_range_low_bp: BasisPoints,
    pub predicted_range_high_bp: BasisPoints,
    pub confidence_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: String,
    pub topic_id: Option<i64>,
    pub reason: String,
    pub priority: i64,
}

// ---------------------------------------------------------------------------
// Diagnosis engine
// ---------------------------------------------------------------------------

pub struct MockDiagnosisEngine<'a> {
    conn: &'a Connection,
}

// Weakness(t) = 0.35*(1-Mastery) + 0.20*LinkBreakage + 0.15*MisconceptionPressure
//             + 0.10*RepresentationGap + 0.10*TimedGap + 0.05*GuessPenalty
//             + 0.05*RecencyDecay
const W_MASTERY_GAP: f64 = 0.35;
const W_LINK_BREAKAGE: f64 = 0.20;
const W_MISCONCEPTION: f64 = 0.15;
const W_REPRESENTATION: f64 = 0.10;
const W_TIMED_GAP: f64 = 0.10;
const W_GUESS: f64 = 0.05;
const W_DECAY: f64 = 0.05;

impl<'a> MockDiagnosisEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Run deep diagnosis on a completed mock session.
    pub fn diagnose(&self, mock_session_id: i64) -> EcoachResult<MockDeepDiagnosis> {
        let (student_id, subject_id, session_id) = self.load_mock_context(mock_session_id)?;

        let topic_stats = self.load_topic_stats(session_id)?;
        let misconception_hits = self.load_misconception_hits(session_id)?;
        let representation_gaps = self.load_representation_gaps(session_id)?;
        let timing = self.compute_timing_diagnosis(session_id)?;
        let confidence = self.compute_confidence_diagnosis(session_id)?;
        let broken_links = self.load_broken_links(session_id, subject_id)?;

        // Compute weakness scores per topic
        let mastery_map = self.load_mastery_map(student_id)?;
        let decay_map = self.load_decay_map(student_id)?;
        let misconception_map = build_misconception_pressure_map(&misconception_hits);

        let mut weaknesses: Vec<TopicWeaknessSlice> = Vec::new();
        let mut strengths: Vec<TopicStrengthSlice> = Vec::new();

        for ts in &topic_stats {
            let mastery = mastery_map.get(&ts.topic_id).copied().unwrap_or(5000) as f64;
            let mastery_gap = (10_000.0 - mastery) / 10_000.0;

            let link_breakage = if broken_links
                .iter()
                .any(|bl| bl.from_node_id == ts.topic_id || bl.to_node_id == ts.topic_id)
            {
                0.8
            } else {
                0.0
            };

            let misconception_pressure = misconception_map
                .get(&ts.topic_id)
                .map(|count| (*count as f64 / 5.0).min(1.0))
                .unwrap_or(0.0);

            let timed_gap = if timing.avg_response_time_ms > 0 && ts.total > 0 {
                let topic_avg_time = ts.total_time_ms / ts.total.max(1);
                if topic_avg_time > timing.avg_response_time_ms * 2 {
                    0.8
                } else if topic_avg_time > timing.avg_response_time_ms {
                    0.4
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let guess_penalty = if confidence.guessing_rate_bp > 3000 {
                0.6
            } else {
                confidence.guessing_rate_bp as f64 / 10_000.0
            };

            let decay = decay_map.get(&ts.topic_id).copied().unwrap_or(0) as f64 / 10_000.0;

            let weakness_score = clamp_bp(
                ((W_MASTERY_GAP * mastery_gap
                    + W_LINK_BREAKAGE * link_breakage
                    + W_MISCONCEPTION * misconception_pressure
                    + W_REPRESENTATION * 0.0 // computed globally, not per-topic
                    + W_TIMED_GAP * timed_gap
                    + W_GUESS * guess_penalty
                    + W_DECAY * decay)
                    * 10_000.0)
                    .round() as i64,
            );

            let accuracy = if ts.total > 0 {
                to_bp(ts.correct as f64 / ts.total as f64)
            } else {
                0
            };

            if accuracy >= 7000 && weakness_score < 3000 {
                strengths.push(TopicStrengthSlice {
                    topic_id: ts.topic_id,
                    topic_name: ts.topic_name.clone(),
                    accuracy_bp: accuracy,
                    correct: ts.correct,
                    total: ts.total,
                });
            } else {
                weaknesses.push(TopicWeaknessSlice {
                    topic_id: ts.topic_id,
                    topic_name: ts.topic_name.clone(),
                    weakness_score,
                    mastery_gap: clamp_bp((mastery_gap * 10_000.0).round() as i64),
                    misconception_pressure: clamp_bp(
                        (misconception_pressure * 10_000.0).round() as i64
                    ),
                    timed_gap: clamp_bp((timed_gap * 10_000.0).round() as i64),
                    accuracy_bp: accuracy,
                });
            }
        }

        weaknesses.sort_by(|a, b| b.weakness_score.cmp(&a.weakness_score));
        strengths.sort_by(|a, b| b.accuracy_bp.cmp(&a.accuracy_bp));

        // Predicted exam score
        let predicted = self.compute_predicted_score(subject_id, &topic_stats, &mastery_map)?;

        // Action plan
        let action_plan = self.build_action_plan(&weaknesses, &misconception_hits, &broken_links);

        let diagnosis = MockDeepDiagnosis {
            mock_session_id,
            student_id,
            subject_id,
            topic_weaknesses: weaknesses,
            topic_strengths: strengths,
            broken_links,
            misconception_hits,
            representation_gaps,
            timing_diagnosis: timing,
            confidence_diagnosis: confidence,
            predicted_exam_score: predicted,
            action_plan,
        };

        // Persist
        self.persist_diagnosis(&diagnosis)?;

        Ok(diagnosis)
    }

    /// Load a previously computed diagnosis.
    pub fn get_diagnosis(&self, mock_session_id: i64) -> EcoachResult<Option<MockDeepDiagnosis>> {
        let row = self
            .conn
            .query_row(
                "SELECT student_id, subject_id, predicted_exam_score,
                        predicted_exam_range_low, predicted_exam_range_high,
                        weakness_scores_json, broken_links_json, misconception_hits_json,
                        representation_gaps_json, timing_diagnosis_json,
                        confidence_diagnosis_json, action_plan_json
                 FROM mock_deep_diagnoses
                 WHERE mock_session_id = ?1
                 ORDER BY created_at DESC LIMIT 1",
                [mock_session_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, String>(10)?,
                        row.get::<_, String>(11)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let Some((
            student_id,
            subject_id,
            pred_score,
            pred_low,
            pred_high,
            weaknesses_json,
            broken_json,
            misconceptions_json,
            repr_json,
            timing_json,
            confidence_json,
            action_json,
        )) = row
        else {
            return Ok(None);
        };

        let predicted = pred_score.map(|score| PredictedScore {
            predicted_score_bp: clamp_bp(score),
            predicted_range_low_bp: clamp_bp(pred_low.unwrap_or(0)),
            predicted_range_high_bp: clamp_bp(pred_high.unwrap_or(10_000)),
            confidence_label: predict_confidence_label(clamp_bp(score)),
        });

        Ok(Some(MockDeepDiagnosis {
            mock_session_id,
            student_id,
            subject_id,
            topic_weaknesses: serde_json::from_str(&weaknesses_json).unwrap_or_default(),
            topic_strengths: Vec::new(), // not persisted separately
            broken_links: serde_json::from_str(&broken_json).unwrap_or_default(),
            misconception_hits: serde_json::from_str(&misconceptions_json).unwrap_or_default(),
            representation_gaps: serde_json::from_str(&repr_json).unwrap_or_default(),
            timing_diagnosis: serde_json::from_str(&timing_json)
                .unwrap_or(default_timing_diagnosis()),
            confidence_diagnosis: serde_json::from_str(&confidence_json)
                .unwrap_or(default_confidence_diagnosis()),
            predicted_exam_score: predicted,
            action_plan: serde_json::from_str(&action_json).unwrap_or_default(),
        }))
    }

    // -----------------------------------------------------------------------
    // Data loading
    // -----------------------------------------------------------------------

    fn load_mock_context(&self, mock_session_id: i64) -> EcoachResult<(i64, i64, i64)> {
        self.conn
            .query_row(
                "SELECT student_id, subject_id, session_id FROM mock_sessions WHERE id = ?1",
                [mock_session_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| EcoachError::NotFound(format!("mock session {mock_session_id}: {e}")))
    }

    fn load_topic_stats(&self, session_id: i64) -> EcoachResult<Vec<TopicStat>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT si.source_topic_id, COALESCE(t.name, 'Unknown'),
                        COUNT(*),
                        SUM(CASE WHEN si.is_correct = 1 THEN 1 ELSE 0 END),
                        COALESCE(SUM(si.response_time_ms), 0)
                 FROM session_items si
                 LEFT JOIN topics t ON t.id = si.source_topic_id
                 WHERE si.session_id = ?1 AND si.source_topic_id IS NOT NULL
                 GROUP BY si.source_topic_id",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([session_id], |row| {
                Ok(TopicStat {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    total: row.get(2)?,
                    correct: row.get(3)?,
                    total_time_ms: row.get(4)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut stats = Vec::new();
        for row in rows {
            stats.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(stats)
    }

    fn load_misconception_hits(&self, session_id: i64) -> EcoachResult<Vec<MisconceptionHit>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT qo.misconception_id, q.topic_id,
                        COALESCE(mp.title, 'Unknown misconception'),
                        COUNT(*) AS hit_count
                 FROM session_items si
                 INNER JOIN question_options qo ON qo.id = si.selected_option_id
                 INNER JOIN questions q ON q.id = si.question_id
                 LEFT JOIN misconception_patterns mp ON mp.id = qo.misconception_id
                 WHERE si.session_id = ?1
                   AND si.is_correct = 0
                   AND qo.misconception_id IS NOT NULL
                 GROUP BY qo.misconception_id, q.topic_id
                 ORDER BY hit_count DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([session_id], |row| {
                Ok(MisconceptionHit {
                    misconception_id: row.get(0)?,
                    topic_id: row.get(1)?,
                    title: row.get(2)?,
                    hit_count: row.get(3)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut hits = Vec::new();
        for row in rows {
            hits.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(hits)
    }

    fn load_representation_gaps(&self, session_id: i64) -> EcoachResult<Vec<RepresentationGap>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT q.question_format, COUNT(*),
                        SUM(CASE WHEN si.is_correct = 1 THEN 1 ELSE 0 END)
                 FROM session_items si
                 INNER JOIN questions q ON q.id = si.question_id
                 WHERE si.session_id = ?1 AND q.question_format IS NOT NULL
                 GROUP BY q.question_format
                 ORDER BY q.question_format",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([session_id], |row| {
                let format: String = row.get(0)?;
                let total: i64 = row.get(1)?;
                let correct: i64 = row.get(2)?;
                let accuracy = if total > 0 {
                    to_bp(correct as f64 / total as f64)
                } else {
                    0
                };
                Ok(RepresentationGap {
                    format_code: format,
                    accuracy_bp: accuracy,
                    total_questions: total,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut gaps = Vec::new();
        for row in rows {
            gaps.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(gaps)
    }

    fn compute_timing_diagnosis(&self, session_id: i64) -> EcoachResult<TimingDiagnosis> {
        let (total_items, total_time, answered): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*),
                        COALESCE(SUM(response_time_ms), 0),
                        SUM(CASE WHEN status = 'answered' THEN 1 ELSE 0 END)
                 FROM session_items WHERE session_id = ?1",
                [session_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let avg_time = if answered > 0 {
            total_time / answered
        } else {
            0
        };

        // Slow but correct: took more than 2x average and got it right
        let slow_correct: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM session_items
                 WHERE session_id = ?1 AND is_correct = 1 AND response_time_ms > ?2",
                params![session_id, avg_time * 2],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Fast but wrong: took less than half average and got it wrong
        let fast_wrong: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM session_items
                 WHERE session_id = ?1 AND is_correct = 0 AND response_time_ms > 0
                   AND response_time_ms < ?2",
                params![session_id, avg_time / 2],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Collapsed near end: last 25% of questions have much lower accuracy
        let collapsed = self.detect_end_collapse(session_id, total_items)?;

        let pacing_label = if collapsed {
            "collapsed_near_end"
        } else if slow_correct > answered / 3 {
            "slow_but_accurate"
        } else if fast_wrong > answered / 4 {
            "rushing"
        } else {
            "balanced"
        };

        Ok(TimingDiagnosis {
            avg_response_time_ms: avg_time,
            slow_but_correct_count: slow_correct,
            fast_but_wrong_count: fast_wrong,
            collapsed_near_end: collapsed,
            pacing_label: pacing_label.to_string(),
        })
    }

    fn detect_end_collapse(&self, session_id: i64, total_items: i64) -> EcoachResult<bool> {
        if total_items < 8 {
            return Ok(false);
        }
        let last_quarter_start = total_items * 3 / 4;

        let (early_correct, early_total): (i64, i64) = self
            .conn
            .query_row(
                "SELECT SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END), COUNT(*)
                 FROM session_items WHERE session_id = ?1 AND display_order < ?2",
                params![session_id, last_quarter_start],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 1));

        let (late_correct, late_total): (i64, i64) = self
            .conn
            .query_row(
                "SELECT SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END), COUNT(*)
                 FROM session_items WHERE session_id = ?1 AND display_order >= ?2",
                params![session_id, last_quarter_start],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 1));

        let early_accuracy = early_correct as f64 / early_total.max(1) as f64;
        let late_accuracy = late_correct as f64 / late_total.max(1) as f64;

        // Collapsed if late accuracy is 20%+ worse than early
        Ok(early_accuracy - late_accuracy >= 0.20)
    }

    fn compute_confidence_diagnosis(&self, session_id: i64) -> EcoachResult<ConfidenceDiagnosis> {
        let mut correct_confident: i64 = 0;
        let mut correct_unsure: i64 = 0;
        let mut wrong_confident: i64 = 0;
        let mut wrong_unsure: i64 = 0;
        let mut total_answered: i64 = 0;

        let mut stmt = self
            .conn
            .prepare(
                "SELECT is_correct, confidence_level
                 FROM session_items
                 WHERE session_id = ?1 AND status = 'answered'",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([session_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        for row in rows {
            let (is_correct, confidence) = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
            total_answered += 1;
            let is_confident = matches!(
                confidence.as_deref(),
                Some("high") | Some("confident") | Some("sure")
            );

            match (is_correct == 1, is_confident) {
                (true, true) => correct_confident += 1,
                (true, false) => correct_unsure += 1,
                (false, true) => wrong_confident += 1,
                (false, false) => wrong_unsure += 1,
            }
        }

        let total = total_answered.max(1);
        let guessing_rate = to_bp(wrong_unsure as f64 / total as f64);
        let overconfidence_rate = to_bp(wrong_confident as f64 / total as f64);

        Ok(ConfidenceDiagnosis {
            correct_and_confident: correct_confident,
            correct_but_unsure: correct_unsure,
            wrong_but_confident: wrong_confident,
            wrong_and_unsure: wrong_unsure,
            guessing_rate_bp: guessing_rate,
            overconfidence_rate_bp: overconfidence_rate,
        })
    }

    fn load_broken_links(
        &self,
        session_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<BrokenLinkDiagnosis>> {
        // Find node_edges where the student got questions wrong on both ends
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT ne.from_node_id, ne.to_node_id, ne.edge_type,
                    COALESCE(an1.canonical_title, 'Node ' || ne.from_node_id),
                    COALESCE(an2.canonical_title, 'Node ' || ne.to_node_id)
             FROM node_edges ne
             INNER JOIN academic_nodes an1 ON an1.id = ne.from_node_id
             INNER JOIN academic_nodes an2 ON an2.id = ne.to_node_id
             INNER JOIN topics t1 ON t1.id = an1.topic_id AND t1.subject_id = ?2
             WHERE EXISTS (
                 SELECT 1 FROM session_items si
                 INNER JOIN question_skill_links qsl ON qsl.question_id = si.question_id
                 WHERE si.session_id = ?1 AND si.is_correct = 0
                   AND qsl.node_id = ne.from_node_id
             )
             AND EXISTS (
                 SELECT 1 FROM session_items si
                 INNER JOIN question_skill_links qsl ON qsl.question_id = si.question_id
                 WHERE si.session_id = ?1 AND si.is_correct = 0
                   AND qsl.node_id = ne.to_node_id
             )
             LIMIT 10",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![session_id, subject_id], |row| {
                Ok(BrokenLinkDiagnosis {
                    from_node_id: row.get(0)?,
                    to_node_id: row.get(1)?,
                    edge_type: row.get(2)?,
                    from_title: row.get(3)?,
                    to_title: row.get(4)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut links = Vec::new();
        for row in rows {
            links.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(links)
    }

    fn load_mastery_map(&self, student_id: i64) -> EcoachResult<BTreeMap<i64, i64>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT topic_id, mastery_score FROM student_topic_states WHERE student_id = ?1",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([student_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut map = BTreeMap::new();
        for row in rows {
            let (tid, mastery) = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
            map.insert(tid, mastery);
        }
        Ok(map)
    }

    fn load_decay_map(&self, student_id: i64) -> EcoachResult<BTreeMap<i64, i64>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT topic_id, COALESCE(decay_risk, 0) FROM student_topic_states WHERE student_id = ?1",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([student_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut map = BTreeMap::new();
        for row in rows {
            let (tid, decay) = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
            map.insert(tid, decay);
        }
        Ok(map)
    }

    // -----------------------------------------------------------------------
    // Predicted exam score
    // -----------------------------------------------------------------------

    /// PredictedExamScore = Σ(BlueprintWeight_k * Mastery_k * TimingFactor_k
    ///                       * RetentionFactor_k * MisconceptionImmunity_k)
    fn compute_predicted_score(
        &self,
        subject_id: i64,
        topic_stats: &[TopicStat],
        mastery_map: &BTreeMap<i64, i64>,
    ) -> EcoachResult<Option<PredictedScore>> {
        let engine = ForecastEngine::new(self.conn);
        let blueprint = match engine.get_latest_blueprint(subject_id)? {
            Some(bp) => bp,
            None => return Ok(None),
        };

        if blueprint.topic_scores.is_empty() {
            return Ok(None);
        }

        let total_composite: f64 = blueprint
            .topic_scores
            .iter()
            .map(|ts| ts.composite_score as f64)
            .sum();
        let safe_total = if total_composite <= 0.0 {
            1.0
        } else {
            total_composite
        };

        let mut weighted_sum: f64 = 0.0;

        for ts in &blueprint.topic_scores {
            let blueprint_weight = ts.composite_score as f64 / safe_total;
            let mastery = mastery_map.get(&ts.topic_id).copied().unwrap_or(5000) as f64 / 10_000.0;

            // Timing factor: penalize if the topic was slow in the mock
            let timing_factor = topic_stats
                .iter()
                .find(|s| s.topic_id == ts.topic_id)
                .map(|s| {
                    if s.total > 0 && s.total_time_ms > 0 {
                        let avg_ms = s.total_time_ms / s.total;
                        if avg_ms > 120_000 { 0.7 } else { 1.0 }
                    } else {
                        0.9
                    }
                })
                .unwrap_or(0.85);

            // Retention factor: based on decay risk
            let retention = 1.0; // simplified; would use decay_map in production

            // Misconception immunity: based on whether misconceptions were triggered
            let immunity = 1.0; // simplified

            weighted_sum += blueprint_weight * mastery * timing_factor * retention * immunity;
        }

        let predicted = clamp_bp((weighted_sum * 10_000.0).round() as i64);
        let margin = 1000; // ±10% range
        let low = clamp_bp(predicted as i64 - margin);
        let high = clamp_bp(predicted as i64 + margin);

        Ok(Some(PredictedScore {
            predicted_score_bp: predicted,
            predicted_range_low_bp: low,
            predicted_range_high_bp: high,
            confidence_label: predict_confidence_label(predicted),
        }))
    }

    // -----------------------------------------------------------------------
    // Action plan
    // -----------------------------------------------------------------------

    fn build_action_plan(
        &self,
        weaknesses: &[TopicWeaknessSlice],
        misconceptions: &[MisconceptionHit],
        broken_links: &[BrokenLinkDiagnosis],
    ) -> Vec<RecommendedAction> {
        let mut actions = Vec::new();
        let mut priority = 1i64;

        // Top 3 weak topics → repair
        for tw in weaknesses.iter().take(3) {
            actions.push(RecommendedAction {
                action_type: "repair_topic".into(),
                topic_id: Some(tw.topic_id),
                reason: format!(
                    "{} has weakness score {} (accuracy {}bp)",
                    tw.topic_name, tw.weakness_score, tw.accuracy_bp
                ),
                priority,
            });
            priority += 1;
        }

        // Unresolved misconceptions → misconception drill
        for mh in misconceptions.iter().take(3) {
            actions.push(RecommendedAction {
                action_type: "misconception_drill".into(),
                topic_id: Some(mh.topic_id),
                reason: format!(
                    "Misconception '{}' triggered {} times",
                    mh.title, mh.hit_count
                ),
                priority,
            });
            priority += 1;
        }

        // Broken links → link repair
        for bl in broken_links.iter().take(2) {
            actions.push(RecommendedAction {
                action_type: "link_repair".into(),
                topic_id: None,
                reason: format!(
                    "Broken link between '{}' and '{}' ({})",
                    bl.from_title, bl.to_title, bl.edge_type
                ),
                priority,
            });
            priority += 1;
        }

        actions
    }

    // -----------------------------------------------------------------------
    // Persistence
    // -----------------------------------------------------------------------

    fn persist_diagnosis(&self, diagnosis: &MockDeepDiagnosis) -> EcoachResult<()> {
        let weaknesses_json = serde_json::to_string(&diagnosis.topic_weaknesses)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let broken_json = serde_json::to_string(&diagnosis.broken_links)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let misconceptions_json = serde_json::to_string(&diagnosis.misconception_hits)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let repr_json = serde_json::to_string(&diagnosis.representation_gaps)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let timing_json = serde_json::to_string(&diagnosis.timing_diagnosis)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let confidence_json = serde_json::to_string(&diagnosis.confidence_diagnosis)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let action_json = serde_json::to_string(&diagnosis.action_plan)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let (pred_score, pred_low, pred_high) = match &diagnosis.predicted_exam_score {
            Some(p) => (
                Some(p.predicted_score_bp as i64),
                Some(p.predicted_range_low_bp as i64),
                Some(p.predicted_range_high_bp as i64),
            ),
            None => (None, None, None),
        };

        self.conn
            .execute(
                "INSERT INTO mock_deep_diagnoses (
                    mock_session_id, student_id, subject_id,
                    predicted_exam_score, predicted_exam_range_low, predicted_exam_range_high,
                    weakness_scores_json, broken_links_json, misconception_hits_json,
                    representation_gaps_json, timing_diagnosis_json, confidence_diagnosis_json,
                    action_plan_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    diagnosis.mock_session_id,
                    diagnosis.student_id,
                    diagnosis.subject_id,
                    pred_score,
                    pred_low,
                    pred_high,
                    weaknesses_json,
                    broken_json,
                    misconceptions_json,
                    repr_json,
                    timing_json,
                    confidence_json,
                    action_json,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

use rusqlite::OptionalExtension;

#[derive(Debug, Clone)]
struct TopicStat {
    topic_id: i64,
    topic_name: String,
    total: i64,
    correct: i64,
    total_time_ms: i64,
}

fn build_misconception_pressure_map(hits: &[MisconceptionHit]) -> BTreeMap<i64, i64> {
    let mut map = BTreeMap::new();
    for hit in hits {
        *map.entry(hit.topic_id).or_insert(0) += hit.hit_count;
    }
    map
}

fn predict_confidence_label(score: BasisPoints) -> String {
    if score >= 8000 {
        "high".into()
    } else if score >= 6000 {
        "moderate".into()
    } else if score >= 4000 {
        "low".into()
    } else {
        "very_low".into()
    }
}

fn default_timing_diagnosis() -> TimingDiagnosis {
    TimingDiagnosis {
        avg_response_time_ms: 0,
        slow_but_correct_count: 0,
        fast_but_wrong_count: 0,
        collapsed_near_end: false,
        pacing_label: "unknown".into(),
    }
}

fn default_confidence_diagnosis() -> ConfidenceDiagnosis {
    ConfidenceDiagnosis {
        correct_and_confident: 0,
        correct_but_unsure: 0,
        wrong_but_confident: 0,
        wrong_and_unsure: 0,
        guessing_rate_bp: 0,
        overconfidence_rate_bp: 0,
    }
}
