use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Extended Beat Yesterday intelligence: state machine, missing scores,
/// streak logic, badges, weekly reviews, growth quality, teacher/parent views.
pub struct BeatYesterdayDeepEngine<'a> {
    conn: &'a Connection,
}

// ---------------------------------------------------------------------------
// 6-state climb machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClimbState {
    Entry,
    Settling,
    Climbing,
    Accelerating,
    Strained,
    Recovery,
}

impl ClimbState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::Settling => "settling",
            Self::Climbing => "climbing",
            Self::Accelerating => "accelerating",
            Self::Strained => "strained",
            Self::Recovery => "recovery",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "settling" => Self::Settling,
            "climbing" => Self::Climbing,
            "accelerating" => Self::Accelerating,
            "strained" => Self::Strained,
            "recovery" => Self::Recovery,
            _ => Self::Entry,
        }
    }
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatYesterdayExtendedProfile {
    pub student_id: i64,
    pub subject_id: i64,
    pub climb_state: String,
    pub momentum_trend: String,
    pub growth_quality: String,
    pub speed_readiness_score: BasisPoints,
    pub confidence_score: BasisPoints,
    pub recovery_need_score: BasisPoints,
    pub streak_days: i64,
    pub badges_earned: Vec<EarnedBadge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnedBadge {
    pub badge_code: String,
    pub badge_name: String,
    pub earned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyReview {
    pub student_id: i64,
    pub subject_id: i64,
    pub week_start: String,
    pub sessions_completed: i64,
    pub avg_attempts_per_day: i64,
    pub avg_correctness_bp: BasisPoints,
    pub avg_pace_ms: i64,
    pub biggest_win: Option<String>,
    pub biggest_challenge: Option<String>,
    pub consistency_streak: i64,
    pub momentum_trend: String,
    pub next_week_primary_focus: Option<String>,
    pub next_week_secondary_focus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherClimbOverview {
    pub student_id: i64,
    pub student_name: String,
    pub climb_state: String,
    pub momentum_trend: String,
    pub streak_days: i64,
    pub current_mode: String,
    pub momentum_score: BasisPoints,
    pub strain_score: BasisPoints,
    pub growth_quality: String,
    pub biggest_bottleneck: Option<String>,
    pub disengagement_risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorClassification {
    pub concept_gap: i64,
    pub careless: i64,
    pub time_pressure: i64,
    pub misread: i64,
    pub guessed: i64,
    pub incomplete_reasoning: i64,
    pub repeated_misconception: i64,
}

impl<'a> BeatYesterdayDeepEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // -----------------------------------------------------------------------
    // Extended profile with all missing scores
    // -----------------------------------------------------------------------

    pub fn get_extended_profile(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BeatYesterdayExtendedProfile> {
        let (climb_state, momentum_trend, growth_quality, speed_readiness,
             confidence, recovery_need, streak): (String, String, String, i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COALESCE(climb_state, 'entry'), COALESCE(momentum_trend, 'steady'),
                        COALESCE(growth_quality, 'unknown'),
                        COALESCE(speed_readiness_score, 0), COALESCE(confidence_score, 5000),
                        COALESCE(recovery_need_score_v2, 0), COALESCE(streak_days, 0)
                 FROM beat_yesterday_profiles
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                          row.get(4)?, row.get(5)?, row.get(6)?)),
            )
            .unwrap_or(("entry".into(), "steady".into(), "unknown".into(), 0, 5000, 0, 0));

        let badges = self.list_earned_badges(student_id, subject_id)?;

        Ok(BeatYesterdayExtendedProfile {
            student_id,
            subject_id,
            climb_state,
            momentum_trend,
            growth_quality,
            speed_readiness_score: clamp_bp(speed_readiness),
            confidence_score: clamp_bp(confidence),
            recovery_need_score: clamp_bp(recovery_need),
            streak_days: streak,
            badges_earned: badges,
        })
    }

    // -----------------------------------------------------------------------
    // Compute and update all missing scores after a daily climb completes
    // -----------------------------------------------------------------------

    pub fn update_extended_scores(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<()> {
        let speed_readiness = self.compute_speed_readiness(student_id, subject_id)?;
        let confidence = self.compute_confidence(student_id, subject_id)?;
        let recovery_need = self.compute_recovery_need(student_id, subject_id)?;
        let momentum_trend = self.compute_momentum_trend(student_id, subject_id)?;
        let growth_quality = self.compute_growth_quality(student_id, subject_id)?;
        let climb_state = self.compute_climb_state(student_id, subject_id)?;
        let streak = self.compute_streak(student_id, subject_id)?;

        self.conn
            .execute(
                "UPDATE beat_yesterday_profiles
                 SET speed_readiness_score = ?1, confidence_score = ?2,
                     recovery_need_score_v2 = ?3, momentum_trend = ?4,
                     growth_quality = ?5, climb_state = ?6, streak_days = ?7,
                     updated_at = datetime('now')
                 WHERE student_id = ?8 AND subject_id = ?9",
                params![
                    speed_readiness as i64, confidence as i64, recovery_need as i64,
                    momentum_trend, growth_quality, climb_state.as_str(),
                    streak, student_id, subject_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Check and award badges
        self.check_badges(student_id, subject_id)?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Score computations
    // -----------------------------------------------------------------------

    fn compute_speed_readiness(&self, student_id: i64, subject_id: i64) -> EcoachResult<BasisPoints> {
        // speed_readiness = 0.4*accuracy_stability + 0.25*topic_familiarity + 0.2*low_skip + 0.15*completion_consistency
        let accuracy_stability = self.recent_accuracy_stability(student_id, subject_id)?;
        let completion_rate = self.recent_completion_rate(student_id, subject_id)?;

        Ok(clamp_bp(
            (0.4 * accuracy_stability as f64
                + 0.25 * 5000.0 // topic familiarity: neutral proxy
                + 0.20 * completion_rate as f64
                + 0.15 * completion_rate as f64)
                .round() as i64,
        ))
    }

    fn compute_confidence(&self, student_id: i64, subject_id: i64) -> EcoachResult<BasisPoints> {
        let completion = self.recent_completion_rate(student_id, subject_id)?;
        let streak: i64 = self.conn.query_row(
            "SELECT COALESCE(streak_days, 0) FROM beat_yesterday_profiles WHERE student_id = ?1 AND subject_id = ?2",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        let beat_rate = self.recent_beat_rate(student_id, subject_id)?;

        // Confidence from: completion behavior + streak + beat rate
        Ok(clamp_bp(
            (0.35 * completion as f64
                + 0.30 * beat_rate as f64
                + 0.20 * (streak.min(10) as f64 / 10.0 * 10_000.0)
                + 0.15 * 5000.0) // baseline
                .round() as i64,
        ))
    }

    fn compute_recovery_need(&self, student_id: i64, subject_id: i64) -> EcoachResult<BasisPoints> {
        let (strain, momentum): (i64, i64) = self.conn.query_row(
            "SELECT COALESCE(strain_score, 0), COALESCE(momentum_score, 5000)
             FROM beat_yesterday_profiles WHERE student_id = ?1 AND subject_id = ?2",
            params![student_id, subject_id], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((0, 5000));

        let recent_drops = self.count_recent_drops(student_id, subject_id)?;

        // High strain + low momentum + recent drops = high recovery need
        Ok(clamp_bp(
            (0.40 * strain as f64
                + 0.30 * (10_000 - momentum) as f64
                + 0.30 * (recent_drops.min(5) as f64 / 5.0 * 10_000.0))
                .round() as i64,
        ))
    }

    fn compute_momentum_trend(&self, student_id: i64, subject_id: i64) -> EcoachResult<String> {
        let scores: Vec<i64> = {
            let mut stmt = self.conn.prepare(
                "SELECT momentum_score FROM beat_yesterday_daily_summaries
                 WHERE student_id = ?1 AND subject_id = ?2
                 ORDER BY summary_date DESC LIMIT 5",
            ).map_err(|e| EcoachError::Storage(e.to_string()))?;

            stmt.query_map(params![student_id, subject_id], |row| row.get(0))
                .map_err(|e| EcoachError::Storage(e.to_string()))?
                .filter_map(|r| r.ok())
                .collect()
        };

        if scores.len() < 2 {
            return Ok("steady".into());
        }

        let recent_avg = scores.iter().take(2).sum::<i64>() / 2;
        let older_avg = scores.iter().skip(2).sum::<i64>() / scores.len().saturating_sub(2).max(1) as i64;

        let delta = recent_avg - older_avg;
        Ok(if delta > 1000 {
            "rising"
        } else if delta > 300 {
            "steady_rise"
        } else if delta < -1000 {
            "falling"
        } else if delta < -300 {
            "slight_decline"
        } else {
            "steady"
        }.into())
    }

    fn compute_growth_quality(&self, student_id: i64, subject_id: i64) -> EcoachResult<String> {
        // Check last summary for unhealthy patterns
        let latest: Option<(bool, bool, bool, i64)> = self.conn.query_row(
            "SELECT beat_attempt_target, beat_accuracy_target, beat_pace_target,
                    COALESCE(guessed_count, 0)
             FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
             ORDER BY summary_date DESC LIMIT 1",
            params![student_id, subject_id],
            |row| Ok((row.get::<_,i64>(0)? == 1, row.get::<_,i64>(1)? == 1,
                      row.get::<_,i64>(2)? == 1, row.get(3)?)),
        ).optional().map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(match latest {
            Some((beat_vol, beat_acc, beat_pace, guessed)) => {
                if beat_vol && !beat_acc && guessed > 3 {
                    "unhealthy_volume" // attempts up but accuracy crashed, lots of guessing
                } else if beat_pace && !beat_acc {
                    "unhealthy_speed" // faster but less accurate
                } else if beat_vol && beat_acc && beat_pace {
                    "strong"
                } else if beat_acc {
                    "healthy"
                } else {
                    "neutral"
                }
            }
            None => "unknown",
        }.into())
    }

    fn compute_climb_state(&self, student_id: i64, subject_id: i64) -> EcoachResult<ClimbState> {
        let (momentum, strain, streak, recovery_need): (i64, i64, i64, i64) = self.conn.query_row(
            "SELECT COALESCE(momentum_score, 5000), COALESCE(strain_score, 0),
                    COALESCE(streak_days, 0), COALESCE(recovery_need_score, 0)
             FROM beat_yesterday_profiles WHERE student_id = ?1 AND subject_id = ?2",
            params![student_id, subject_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).unwrap_or((5000, 0, 0, 0));

        let session_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        Ok(if strain >= 7000 || recovery_need >= 7000 {
            ClimbState::Strained
        } else if momentum < 3000 && session_count > 3 {
            ClimbState::Recovery
        } else if session_count <= 3 {
            ClimbState::Entry
        } else if streak < 3 || momentum < 5000 {
            ClimbState::Settling
        } else if momentum >= 7500 && streak >= 5 {
            ClimbState::Accelerating
        } else {
            ClimbState::Climbing
        })
    }

    fn compute_streak(&self, student_id: i64, subject_id: i64) -> EcoachResult<i64> {
        let dates: Vec<String> = {
            let mut stmt = self.conn.prepare(
                "SELECT summary_date FROM beat_yesterday_daily_summaries
                 WHERE student_id = ?1 AND subject_id = ?2
                   AND (beat_attempt_target = 1 OR beat_accuracy_target = 1 OR beat_pace_target = 1)
                 ORDER BY summary_date DESC LIMIT 30",
            ).map_err(|e| EcoachError::Storage(e.to_string()))?;

            stmt.query_map(params![student_id, subject_id], |row| row.get(0))
                .map_err(|e| EcoachError::Storage(e.to_string()))?
                .filter_map(|r| r.ok())
                .collect()
        };

        if dates.is_empty() { return Ok(0); }

        let today = chrono::Utc::now().date_naive();
        let mut streak = 0i64;
        let mut expected = today;

        for date_str in &dates {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                if date == expected || date == expected - chrono::Duration::days(1) {
                    streak += 1;
                    expected = date - chrono::Duration::days(1);
                } else {
                    break;
                }
            }
        }
        Ok(streak)
    }

    // -----------------------------------------------------------------------
    // Badge system
    // -----------------------------------------------------------------------

    fn check_badges(&self, student_id: i64, subject_id: i64) -> EcoachResult<()> {
        let streak = self.compute_streak(student_id, subject_id)?;

        // First Beat
        let total_beats: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
               AND (beat_attempt_target = 1 OR beat_accuracy_target = 1 OR beat_pace_target = 1)",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        if total_beats >= 1 { self.award_badge(student_id, subject_id, "first_beat")?; }
        if streak >= 3 { self.award_badge(student_id, subject_id, "three_day_climb")?; }
        if streak >= 5 { self.award_badge(student_id, subject_id, "five_day_climb")?; }

        // Triple beat (all 3 targets in one day)
        let triple: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
               AND beat_attempt_target = 1 AND beat_accuracy_target = 1 AND beat_pace_target = 1",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);
        if triple >= 1 { self.award_badge(student_id, subject_id, "balanced_growth")?; }

        // Completion streak (7 days)
        let completion_streak: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
               AND summary_date >= date('now', '-7 days')",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);
        if completion_streak >= 7 { self.award_badge(student_id, subject_id, "no_quit_week")?; }

        // Recovery comeback
        let recovery_return: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2 AND recovery_mode_triggered = 1",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);
        if recovery_return >= 1 && streak >= 2 {
            self.award_badge(student_id, subject_id, "recovery_comeback")?;
        }

        Ok(())
    }

    fn award_badge(&self, student_id: i64, subject_id: i64, badge_code: &str) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO beat_yesterday_earned_badges
                    (student_id, subject_id, badge_code)
                 VALUES (?1, ?2, ?3)",
                params![student_id, subject_id, badge_code],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    fn list_earned_badges(&self, student_id: i64, subject_id: i64) -> EcoachResult<Vec<EarnedBadge>> {
        let mut stmt = self.conn.prepare(
            "SELECT eb.badge_code, bd.badge_name, eb.earned_at
             FROM beat_yesterday_earned_badges eb
             INNER JOIN beat_yesterday_badge_definitions bd ON bd.badge_code = eb.badge_code
             WHERE eb.student_id = ?1 AND eb.subject_id = ?2
             ORDER BY eb.earned_at DESC",
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt.query_map(params![student_id, subject_id], |row| {
            Ok(EarnedBadge {
                badge_code: row.get(0)?,
                badge_name: row.get(1)?,
                earned_at: row.get(2)?,
            })
        }).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut badges = Vec::new();
        for row in rows { badges.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?); }
        Ok(badges)
    }

    // -----------------------------------------------------------------------
    // Weekly review
    // -----------------------------------------------------------------------

    pub fn generate_weekly_review(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<WeeklyReview> {
        let week_end = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let week_start = (chrono::Utc::now() - chrono::Duration::days(7))
            .format("%Y-%m-%d").to_string();

        let (sessions, avg_attempts, avg_correct, avg_pace): (i64, i64, i64, i64) = self.conn.query_row(
            "SELECT COUNT(*), COALESCE(AVG(actual_attempts), 0),
                    COALESCE(AVG(actual_correct), 0),
                    COALESCE(AVG(actual_avg_response_time_ms), 0)
             FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
               AND summary_date >= ?3 AND summary_date <= ?4",
            params![student_id, subject_id, week_start, week_end],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).unwrap_or((0, 0, 0, 0));

        let avg_correctness = if avg_attempts > 0 {
            clamp_bp(((avg_correct as f64 / avg_attempts.max(1) as f64) * 10_000.0).round() as i64)
        } else { 0 };

        let streak = self.compute_streak(student_id, subject_id)?;
        let momentum_trend = self.compute_momentum_trend(student_id, subject_id)?;

        // Determine biggest win/challenge from error patterns
        let biggest_win = if avg_correctness >= 7000 {
            Some("Strong accuracy this week".into())
        } else if sessions >= 5 {
            Some("Great consistency this week".into())
        } else {
            None
        };

        let biggest_challenge = if avg_correctness < 5000 && sessions > 0 {
            Some("Accuracy needs focused repair".into())
        } else if sessions < 3 {
            Some("Consistency could improve".into())
        } else {
            None
        };

        // Persist
        self.conn.execute(
            "INSERT INTO beat_yesterday_weekly_reviews
                (student_id, subject_id, week_start, week_end, sessions_completed,
                 avg_attempts_per_day, avg_correctness_bp, avg_pace_ms,
                 biggest_win, biggest_challenge, consistency_streak, momentum_trend)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(student_id, subject_id, week_start) DO UPDATE SET
                sessions_completed = ?5, avg_attempts_per_day = ?6,
                avg_correctness_bp = ?7, avg_pace_ms = ?8,
                biggest_win = ?9, biggest_challenge = ?10,
                consistency_streak = ?11, momentum_trend = ?12",
            params![
                student_id, subject_id, week_start, week_end,
                sessions, avg_attempts, avg_correctness as i64, avg_pace,
                biggest_win, biggest_challenge, streak, momentum_trend,
            ],
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(WeeklyReview {
            student_id, subject_id,
            week_start: week_start.clone(),
            sessions_completed: sessions,
            avg_attempts_per_day: avg_attempts,
            avg_correctness_bp: avg_correctness,
            avg_pace_ms: avg_pace,
            biggest_win, biggest_challenge,
            consistency_streak: streak,
            momentum_trend,
            next_week_primary_focus: None,
            next_week_secondary_focus: None,
        })
    }

    // -----------------------------------------------------------------------
    // Teacher class overview
    // -----------------------------------------------------------------------

    pub fn get_teacher_class_overview(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Vec<TeacherClimbOverview>> {
        let mut stmt = self.conn.prepare(
            "SELECT byp.student_id, a.display_name,
                    COALESCE(byp.climb_state, 'entry'),
                    COALESCE(byp.momentum_trend, 'steady'),
                    COALESCE(byp.streak_days, 0),
                    COALESCE(byp.current_mode, 'balanced'),
                    COALESCE(byp.momentum_score, 5000),
                    COALESCE(byp.strain_score, 0),
                    COALESCE(byp.growth_quality, 'unknown')
             FROM beat_yesterday_profiles byp
             INNER JOIN accounts a ON a.id = byp.student_id
             WHERE byp.subject_id = ?1
             ORDER BY byp.momentum_score DESC",
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt.query_map([subject_id], |row| {
            let momentum: i64 = row.get(6)?;
            let strain: i64 = row.get(7)?;
            let streak: i64 = row.get(4)?;

            let risk = if strain >= 7000 { "high" }
                else if momentum < 3000 && streak == 0 { "medium" }
                else { "low" };

            Ok(TeacherClimbOverview {
                student_id: row.get(0)?,
                student_name: row.get(1)?,
                climb_state: row.get(2)?,
                momentum_trend: row.get(3)?,
                streak_days: streak,
                current_mode: row.get(5)?,
                momentum_score: clamp_bp(momentum),
                strain_score: clamp_bp(strain),
                growth_quality: row.get(8)?,
                biggest_bottleneck: None, // would require per-student topic analysis
                disengagement_risk: risk.into(),
            })
        }).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows { result.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?); }
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn recent_accuracy_stability(&self, student_id: i64, subject_id: i64) -> EcoachResult<i64> {
        let accuracies: Vec<f64> = {
            let mut stmt = self.conn.prepare(
                "SELECT CASE WHEN actual_attempts > 0 THEN CAST(actual_correct AS REAL) / actual_attempts ELSE 0 END
                 FROM beat_yesterday_daily_summaries
                 WHERE student_id = ?1 AND subject_id = ?2
                 ORDER BY summary_date DESC LIMIT 5",
            ).map_err(|e| EcoachError::Storage(e.to_string()))?;
            stmt.query_map(params![student_id, subject_id], |row| row.get(0))
                .map_err(|e| EcoachError::Storage(e.to_string()))?
                .filter_map(|r| r.ok()).collect()
        };

        if accuracies.len() < 2 { return Ok(5000); }

        let avg: f64 = accuracies.iter().sum::<f64>() / accuracies.len() as f64;
        let variance: f64 = accuracies.iter().map(|a| (a - avg).powi(2)).sum::<f64>() / accuracies.len() as f64;

        // Low variance = high stability
        Ok(clamp_bp(((1.0 - variance.sqrt().min(1.0)) * 10_000.0).round() as i64) as i64)
    }

    fn recent_completion_rate(&self, student_id: i64, subject_id: i64) -> EcoachResult<i64> {
        let (targets, completed): (i64, i64) = self.conn.query_row(
            "SELECT COUNT(*), SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END)
             FROM beat_yesterday_daily_targets
             WHERE student_id = ?1 AND subject_id = ?2
               AND target_date >= date('now', '-14 days')",
            params![student_id, subject_id], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((0, 0));
        if targets == 0 { return Ok(5000); }
        Ok(to_bp(completed as f64 / targets as f64) as i64)
    }

    fn recent_beat_rate(&self, student_id: i64, subject_id: i64) -> EcoachResult<i64> {
        let (total, beats): (i64, i64) = self.conn.query_row(
            "SELECT COUNT(*),
                    SUM(CASE WHEN beat_attempt_target = 1 OR beat_accuracy_target = 1 THEN 1 ELSE 0 END)
             FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
               AND summary_date >= date('now', '-14 days')",
            params![student_id, subject_id], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((0, 0));
        if total == 0 { return Ok(5000); }
        Ok(to_bp(beats as f64 / total as f64) as i64)
    }

    fn count_recent_drops(&self, student_id: i64, subject_id: i64) -> EcoachResult<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM beat_yesterday_daily_summaries
             WHERE student_id = ?1 AND subject_id = ?2
               AND summary_date >= date('now', '-7 days')
               AND beat_attempt_target = 0 AND beat_accuracy_target = 0",
            params![student_id, subject_id], |row| row.get(0),
        ).map_err(|e| EcoachError::Storage(e.to_string()))
    }
}
