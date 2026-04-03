use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{CanonicalIntelligenceStore, readiness_engine::ReadinessEngine};

// ---------------------------------------------------------------------------
// Route modes: how the journey behaves based on time, progress, and goals
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteMode {
    DeepMastery,
    Balanced,
    HighYield,
    Rescue,
    Reactivation,
}

impl RouteMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeepMastery => "deep_mastery",
            Self::Balanced => "balanced",
            Self::HighYield => "high_yield",
            Self::Rescue => "rescue",
            Self::Reactivation => "reactivation",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "deep_mastery" => Self::DeepMastery,
            "high_yield" => Self::HighYield,
            "rescue" => Self::Rescue,
            "reactivation" => Self::Reactivation,
            _ => Self::Balanced,
        }
    }
}

// ---------------------------------------------------------------------------
// Deadline pressure
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlinePressure {
    pub days_remaining: i64,
    pub study_days_remaining: i64,
    pub pressure_score: BasisPoints,
    pub urgency_label: String,
    pub feasibility_label: String,
    pub recommended_route_mode: RouteMode,
    pub weekly_sessions_needed: i64,
}

// ---------------------------------------------------------------------------
// Consistency snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencySnapshot {
    pub student_id: i64,
    pub subject_id: i64,
    pub streak_days: i64,
    pub study_days_last_14: i64,
    pub avg_daily_minutes_last_14: i64,
    pub total_questions_last_14: i64,
    pub avg_accuracy_bp: BasisPoints,
    pub pace_label: String,
}

// ---------------------------------------------------------------------------
// Knowledge map heat
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeMapNode {
    pub topic_id: i64,
    pub topic_name: String,
    pub heat_label: String,
    pub mastery_heat: BasisPoints,
    pub stability_heat: BasisPoints,
    pub misconception_heat: BasisPoints,
    pub coverage_heat: BasisPoints,
    pub momentum_heat: BasisPoints,
}

// ---------------------------------------------------------------------------
// Morale signal
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoraleSignal {
    pub signal_type: String,
    pub message: String,
    pub context: Value,
}

// ---------------------------------------------------------------------------
// Adaptation engine
// ---------------------------------------------------------------------------

pub struct JourneyAdaptationEngine<'a> {
    conn: &'a Connection,
}

impl<'a> JourneyAdaptationEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // -----------------------------------------------------------------------
    // Deadline pressure calculation
    // -----------------------------------------------------------------------

    pub fn compute_deadline_pressure(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<DeadlinePressure> {
        // Load exam date from journey_routes
        let exam_date: Option<String> = self
            .conn
            .query_row(
                "SELECT exam_date FROM journey_routes
                 WHERE student_id = ?1 AND subject_id = ?2 AND status = 'active'
                 ORDER BY created_at DESC LIMIT 1",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .flatten();

        let days_remaining = match &exam_date {
            Some(date_str) => {
                if let Ok(exam) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    let today = chrono::Utc::now().date_naive();
                    (exam - today).num_days().max(0)
                } else {
                    90 // default if unparseable
                }
            }
            None => 90, // no exam date set
        };

        // Load consistency to estimate study days
        let consistency = self.get_consistency_snapshot(student_id, subject_id)?;
        let study_ratio = if consistency.study_days_last_14 > 0 {
            consistency.study_days_last_14 as f64 / 14.0
        } else {
            0.5 // assume 50% if no history
        };
        let study_days_remaining = (days_remaining as f64 * study_ratio).round() as i64;

        // Load readiness to estimate remaining work
        let readiness =
            ReadinessEngine::new(self.conn).build_subject_readiness(student_id, subject_id)?;

        let remaining_work = 10_000 - readiness.readiness_score as i64;
        let capacity =
            (study_days_remaining * consistency.avg_daily_minutes_last_14.max(20)).max(1);

        // Pressure score: higher = more urgent
        // 10000 = exam tomorrow with lots of work; 0 = plenty of time
        let pressure = if days_remaining <= 0 {
            10_000
        } else {
            let raw = (remaining_work as f64 / capacity as f64) * 10_000.0;
            clamp_bp(raw.round() as i64) as i64
        };

        let urgency_label = match pressure {
            0..=2000 => "comfortable",
            2001..=4500 => "moderate",
            4501..=7000 => "tight",
            7001..=8500 => "urgent",
            _ => "critical",
        };

        let feasibility_label = if pressure <= 6000 {
            "feasible"
        } else if pressure <= 8000 {
            "challenging"
        } else if pressure <= 9000 {
            "at_risk"
        } else {
            "needs_adjustment"
        };

        let recommended_mode = match pressure {
            0..=2000 => RouteMode::DeepMastery,
            2001..=5000 => RouteMode::Balanced,
            5001..=7500 => RouteMode::HighYield,
            _ => RouteMode::Rescue,
        };

        // Override to reactivation if too many dormant topics
        let recommended_mode = if readiness.due_memory_count >= 5 && pressure < 8000 {
            RouteMode::Reactivation
        } else {
            recommended_mode
        };

        let weekly_sessions_needed = if days_remaining > 0 {
            let weeks = (days_remaining as f64 / 7.0).max(1.0);
            ((remaining_work as f64 / 500.0) / weeks).ceil() as i64
        } else {
            7
        };

        // Persist pressure to route
        self.conn
            .execute(
                "UPDATE journey_routes SET deadline_pressure_score = ?1, route_mode = ?2
                 WHERE student_id = ?3 AND subject_id = ?4 AND status = 'active'",
                params![pressure, recommended_mode.as_str(), student_id, subject_id,],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(DeadlinePressure {
            days_remaining,
            study_days_remaining,
            pressure_score: clamp_bp(pressure),
            urgency_label: urgency_label.into(),
            feasibility_label: feasibility_label.into(),
            recommended_route_mode: recommended_mode,
            weekly_sessions_needed,
        })
    }

    // -----------------------------------------------------------------------
    // Mid-journey adaptation: re-evaluate and potentially rebuild route
    // -----------------------------------------------------------------------

    pub fn adapt_route(&self, student_id: i64, subject_id: i64) -> EcoachResult<AdaptationResult> {
        let pressure = self.compute_deadline_pressure(student_id, subject_id)?;
        let consistency = self.get_consistency_snapshot(student_id, subject_id)?;

        // Load current route
        let route = self
            .conn
            .query_row(
                "SELECT id, route_mode, route_type FROM journey_routes
                 WHERE student_id = ?1 AND subject_id = ?2 AND status = 'active'
                 ORDER BY created_at DESC LIMIT 1",
                params![student_id, subject_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let Some((route_id, current_mode_str, current_route_type)) = route else {
            return Err(EcoachError::NotFound("no active route to adapt".into()));
        };

        let current_mode = RouteMode::from_str(&current_mode_str);
        let new_mode = pressure.recommended_route_mode;

        let mut actions = Vec::new();
        let mut needs_rebuild = false;

        // Check if mode should change
        if current_mode != new_mode {
            actions.push(format!(
                "route_mode_shift: {} -> {}",
                current_mode.as_str(),
                new_mode.as_str()
            ));

            self.conn
                .execute(
                    "UPDATE journey_routes SET route_mode = ?1, updated_at = datetime('now') WHERE id = ?2",
                    params![new_mode.as_str(), route_id],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        // Check if rescue mode requires route rebuild
        if new_mode == RouteMode::Rescue && current_mode != RouteMode::Rescue {
            needs_rebuild = true;
            actions.push("rescue_rebuild_triggered".into());
        }

        // Check consistency: if missed 5+ days in last 14, trigger replan
        if consistency.study_days_last_14 < 5 && consistency.streak_days == 0 {
            actions.push("consistency_warning: extended absence detected".into());
            if !needs_rebuild && new_mode != RouteMode::Reactivation {
                // Switch to reactivation
                self.conn
                    .execute(
                        "UPDATE journey_routes SET route_mode = 'reactivation', updated_at = datetime('now') WHERE id = ?1",
                        [route_id],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
                actions.push("route_mode_override: reactivation due to absence".into());
            }
        }

        // Check for stuck stations (station active for too long with low evidence)
        let stuck_station = self.detect_stuck_station(route_id)?;
        if stuck_station {
            actions.push("stuck_station_detected: may need station skip or repair".into());
        }

        // Generate morale signals
        let morale =
            self.generate_morale_signals(student_id, subject_id, &pressure, &consistency)?;
        for signal in &morale {
            actions.push(format!("morale: {}", signal.signal_type));
        }

        let result = AdaptationResult {
            previous_mode: current_mode,
            new_mode,
            needs_rebuild,
            pressure,
            consistency,
            actions,
            morale_signals: morale,
        };
        let canonical_store = CanonicalIntelligenceStore::new(self.conn);
        canonical_store.sync_adaptation_snapshot(student_id, subject_id, &result)?;
        canonical_store.refresh_subject_runtime(
            student_id,
            subject_id,
            Some(result.new_mode),
            Some(result.pressure.pressure_score as i64),
        )?;

        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Consistency tracking
    // -----------------------------------------------------------------------

    pub fn record_study_day(
        &self,
        student_id: i64,
        subject_id: i64,
        minutes: i64,
        questions: i64,
        accuracy_bp: BasisPoints,
    ) -> EcoachResult<()> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        self.conn
            .execute(
                "INSERT INTO study_consistency
                    (student_id, subject_id, study_date, sessions_completed, total_minutes, questions_answered, accuracy_bp)
                 VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6)
                 ON CONFLICT(student_id, subject_id, study_date) DO UPDATE SET
                    sessions_completed = sessions_completed + 1,
                    total_minutes = total_minutes + ?4,
                    questions_answered = questions_answered + ?5,
                    accuracy_bp = ?6",
                params![student_id, subject_id, today, minutes, questions, accuracy_bp as i64],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn get_consistency_snapshot(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<ConsistencySnapshot> {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let fourteen_ago = (chrono::Utc::now() - chrono::Duration::days(14))
            .format("%Y-%m-%d")
            .to_string();

        let (study_days, total_minutes, total_questions, avg_accuracy): (i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(total_minutes), 0),
                        COALESCE(SUM(questions_answered), 0),
                        COALESCE(AVG(accuracy_bp), 0)
                 FROM study_consistency
                 WHERE student_id = ?1 AND subject_id = ?2
                   AND study_date >= ?3 AND study_date <= ?4",
                params![student_id, subject_id, fourteen_ago, today],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap_or((0, 0, 0, 0));

        // Streak: consecutive days ending today
        let streak = self.compute_streak(student_id, subject_id)?;

        let avg_daily = if study_days > 0 {
            total_minutes / study_days
        } else {
            0
        };

        let pace_label = match study_days {
            0..=2 => "inactive",
            3..=5 => "light",
            6..=9 => "moderate",
            10..=12 => "strong",
            _ => "intense",
        };

        Ok(ConsistencySnapshot {
            student_id,
            subject_id,
            streak_days: streak,
            study_days_last_14: study_days,
            avg_daily_minutes_last_14: avg_daily,
            total_questions_last_14: total_questions,
            avg_accuracy_bp: clamp_bp(avg_accuracy),
            pace_label: pace_label.into(),
        })
    }

    // -----------------------------------------------------------------------
    // Knowledge map heat
    // -----------------------------------------------------------------------

    pub fn refresh_knowledge_map(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<KnowledgeMapNode>> {
        // Load all topics for the subject
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.name FROM topics t
                 WHERE t.subject_id = ?1 AND t.is_active = 1
                 ORDER BY t.display_order, t.id",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let topics: Vec<(i64, String)> = stmt
            .query_map([subject_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        let mut nodes = Vec::new();

        for (topic_id, topic_name) in &topics {
            let (mastery, gap, fragility, evidence_count, decay_risk): (i64, i64, i64, i64, i64) =
                self.conn
                    .query_row(
                        "SELECT COALESCE(mastery_score, 0), COALESCE(gap_score, 0),
                            COALESCE(fragility_score, 0), COALESCE(evidence_count, 0),
                            COALESCE(decay_risk, 0)
                     FROM student_topic_states
                     WHERE student_id = ?1 AND topic_id = ?2",
                        params![student_id, topic_id],
                        |row| {
                            Ok((
                                row.get(0)?,
                                row.get(1)?,
                                row.get(2)?,
                                row.get(3)?,
                                row.get(4)?,
                            ))
                        },
                    )
                    .unwrap_or((0, 0, 0, 0, 0));

            // Misconception count
            let misconception_count: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM wrong_answer_diagnoses
                     WHERE student_id = ?1 AND topic_id = ?2 AND misconception_id IS NOT NULL",
                    params![student_id, topic_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            // Recent activity (momentum)
            let recent_attempts: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM student_question_attempts
                     WHERE student_id = ?1 AND question_id IN (
                         SELECT id FROM questions WHERE topic_id = ?2
                     ) AND created_at >= datetime('now', '-7 days')",
                    params![student_id, topic_id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            let mastery_heat = clamp_bp(mastery);
            let stability_heat = clamp_bp(10_000 - fragility);
            let misconception_heat =
                clamp_bp((misconception_count as f64 / 5.0).min(1.0) as i64 * 10_000);
            let coverage_heat = if evidence_count >= 10 {
                10_000
            } else {
                clamp_bp((evidence_count as f64 / 10.0 * 10_000.0).round() as i64)
            };
            let momentum_heat = clamp_bp((recent_attempts as f64 / 10.0).min(1.0) as i64 * 10_000);

            let heat_label = if evidence_count == 0 {
                "unseen"
            } else if mastery >= 8000 && fragility < 2000 {
                "strong"
            } else if mastery >= 6000 {
                "developing"
            } else if misconception_count >= 3 {
                "misconception_active"
            } else if fragility >= 6000 {
                "fragile"
            } else if decay_risk >= 5000 {
                "at_risk"
            } else if recent_attempts > 0 {
                "improving"
            } else {
                "weak"
            };

            // Upsert to knowledge_map_heat
            self.conn
                .execute(
                    "INSERT INTO knowledge_map_heat
                        (student_id, topic_id, heat_label, mastery_heat, stability_heat,
                         misconception_heat, coverage_heat, momentum_heat)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(student_id, topic_id) DO UPDATE SET
                        heat_label = ?3, mastery_heat = ?4, stability_heat = ?5,
                        misconception_heat = ?6, coverage_heat = ?7, momentum_heat = ?8,
                        updated_at = datetime('now')",
                    params![
                        student_id,
                        topic_id,
                        heat_label,
                        mastery_heat as i64,
                        stability_heat as i64,
                        misconception_heat as i64,
                        coverage_heat as i64,
                        momentum_heat as i64,
                    ],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;

            nodes.push(KnowledgeMapNode {
                topic_id: *topic_id,
                topic_name: topic_name.clone(),
                heat_label: heat_label.into(),
                mastery_heat,
                stability_heat,
                misconception_heat,
                coverage_heat,
                momentum_heat,
            });
        }

        Ok(nodes)
    }

    // -----------------------------------------------------------------------
    // Morale feedback
    // -----------------------------------------------------------------------

    fn generate_morale_signals(
        &self,
        student_id: i64,
        subject_id: i64,
        pressure: &DeadlinePressure,
        consistency: &ConsistencySnapshot,
    ) -> EcoachResult<Vec<MoraleSignal>> {
        let mut signals = Vec::new();

        // Streak celebration
        if consistency.streak_days >= 7 {
            signals.push(MoraleSignal {
                signal_type: "streak_milestone".into(),
                message: format!(
                    "{} day streak! Consistency is your superpower.",
                    consistency.streak_days
                ),
                context: json!({"streak_days": consistency.streak_days}),
            });
        } else if consistency.streak_days >= 3 {
            signals.push(MoraleSignal {
                signal_type: "streak_building".into(),
                message: format!("{} days in a row. Keep going!", consistency.streak_days),
                context: json!({"streak_days": consistency.streak_days}),
            });
        }

        // Behind schedule but recoverable
        if pressure.urgency_label == "tight" || pressure.urgency_label == "urgent" {
            if pressure.feasibility_label == "feasible"
                || pressure.feasibility_label == "challenging"
            {
                signals.push(MoraleSignal {
                    signal_type: "behind_but_recoverable".into(),
                    message:
                        "You are behind schedule, but still recoverable with consistent effort."
                            .into(),
                    context: json!({
                        "days_remaining": pressure.days_remaining,
                        "weekly_sessions_needed": pressure.weekly_sessions_needed,
                    }),
                });
            }
        }

        // Accuracy improvement
        if consistency.avg_accuracy_bp >= 7500 && consistency.total_questions_last_14 >= 20 {
            signals.push(MoraleSignal {
                signal_type: "accuracy_strong".into(),
                message: "Your accuracy is strong. You're understanding the material well.".into(),
                context: json!({"accuracy_bp": consistency.avg_accuracy_bp}),
            });
        }

        // Extended absence
        if consistency.streak_days == 0 && consistency.study_days_last_14 <= 2 {
            signals.push(MoraleSignal {
                signal_type: "absence_warning".into(),
                message: "It has been a while. Let's pick up where you left off.".into(),
                context: json!({"study_days_last_14": consistency.study_days_last_14}),
            });
        }

        // Persist signals
        for signal in &signals {
            let context_json =
                serde_json::to_string(&signal.context).unwrap_or_else(|_| "{}".into());
            self.conn
                .execute(
                    "INSERT INTO morale_signals (student_id, subject_id, signal_type, message, context_json)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![student_id, subject_id, signal.signal_type, signal.message, context_json],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        Ok(signals)
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn compute_streak(&self, student_id: i64, subject_id: i64) -> EcoachResult<i64> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT study_date FROM study_consistency
                 WHERE student_id = ?1 AND subject_id = ?2
                 ORDER BY study_date DESC
                 LIMIT 30",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let dates: Vec<String> = stmt
            .query_map(params![student_id, subject_id], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        if dates.is_empty() {
            return Ok(0);
        }

        let today = chrono::Utc::now().date_naive();
        let mut streak = 0i64;
        let mut expected = today;

        for date_str in &dates {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                if date == expected {
                    streak += 1;
                    expected -= chrono::Duration::days(1);
                } else if date == expected - chrono::Duration::days(1) {
                    // Allow gap of 1 day (yesterday might not be today)
                    expected = date;
                    streak += 1;
                    expected -= chrono::Duration::days(1);
                } else {
                    break;
                }
            }
        }

        Ok(streak)
    }

    fn detect_stuck_station(&self, route_id: i64) -> EcoachResult<bool> {
        // A station is "stuck" if it's been active for more than 7 days with < 3 retries
        let stuck: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM journey_stations
                 WHERE route_id = ?1 AND status = 'active'
                   AND unlocked_at IS NOT NULL
                   AND julianday('now') - julianday(unlocked_at) > 7
                   AND retry_count < 3",
                [route_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(stuck > 0)
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationResult {
    pub previous_mode: RouteMode,
    pub new_mode: RouteMode,
    pub needs_rebuild: bool,
    pub pressure: DeadlinePressure,
    pub consistency: ConsistencySnapshot,
    pub actions: Vec<String>,
    pub morale_signals: Vec<MoraleSignal>,
}
