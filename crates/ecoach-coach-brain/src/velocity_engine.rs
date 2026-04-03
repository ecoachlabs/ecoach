use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

/// Tracks learner progress velocity and manages route compression/deepening.
pub struct VelocityEngine<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocitySnapshot {
    pub student_id: i64,
    pub subject_id: i64,
    pub station_clearance_rate: BasisPoints,
    pub mastery_gain_rate: BasisPoints,
    pub retention_gain_rate: BasisPoints,
    pub mock_improvement_rate: BasisPoints,
    pub speed_gain_rate: BasisPoints,
    pub coverage_expansion_rate: BasisPoints,
    pub overall_velocity_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalFeasibility {
    pub student_id: i64,
    pub subject_id: i64,
    pub work_remaining_score: BasisPoints,
    pub capacity_score: BasisPoints,
    pub velocity_score: BasisPoints,
    pub feasibility_status: String,
    pub sessions_needed_estimate: i64,
    pub days_needed_estimate: i64,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionAction {
    pub action_type: String,
    pub target_station_id: Option<i64>,
    pub reason: String,
}

impl<'a> VelocityEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Compute and persist velocity from recent evidence events.
    pub fn compute_velocity(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<VelocitySnapshot> {
        // Station clearance: how many stations cleared in last 14 days
        let stations_cleared: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM journey_stations js
                 INNER JOIN journey_routes jr ON jr.id = js.route_id
                 WHERE jr.student_id = ?1 AND jr.subject_id = ?2
                   AND js.status IN ('completed', 'cleared')
                   AND js.completed_at >= datetime('now', '-14 days')",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let station_rate = clamp_bp((stations_cleared as f64 / 7.0 * 10_000.0).round() as i64);

        // Mastery gain: average positive mastery_delta from evidence events
        let mastery_gain: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(CASE WHEN mastery_delta > 0 THEN mastery_delta ELSE 0 END), 0)
                 FROM evidence_events
                 WHERE student_id = ?1 AND subject_id = ?2
                   AND created_at >= datetime('now', '-14 days')",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let mastery_rate = clamp_bp(mastery_gain * 10);

        // Retention gain: evidence from retention checks
        let retention_gain: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(CASE WHEN retention_delta > 0 THEN retention_delta ELSE 0 END), 0)
                 FROM evidence_events
                 WHERE student_id = ?1 AND subject_id = ?2
                   AND testing_reason = 'retention'
                   AND created_at >= datetime('now', '-14 days')",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let retention_rate = clamp_bp(retention_gain * 10);

        // Mock improvement: compare last 2 mock scores
        let mock_rate = self.compute_mock_improvement_rate(student_id, subject_id)?;

        // Speed gain: improvement in timed_delta
        let speed_gain: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(AVG(CASE WHEN timed_delta > 0 THEN timed_delta ELSE 0 END), 0)
                 FROM evidence_events
                 WHERE student_id = ?1 AND subject_id = ?2
                   AND created_at >= datetime('now', '-14 days')",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        let speed_rate = clamp_bp(speed_gain * 10);

        // Coverage expansion
        let coverage_rate = clamp_bp(station_rate as i64 * 8 / 10); // proxy

        let overall = (station_rate as i64
            + mastery_rate as i64
            + retention_rate as i64
            + mock_rate as i64
            + speed_rate as i64)
            / 5;
        let overall_label = match overall {
            0..=2000 => "stalled",
            2001..=4000 => "slow",
            4001..=6000 => "moderate",
            6001..=8000 => "fast",
            _ => "accelerating",
        };

        // Persist
        self.conn
            .execute(
                "INSERT INTO progress_velocity
                    (student_id, subject_id, station_clearance_rate, mastery_gain_rate,
                     retention_gain_rate, mock_improvement_rate, speed_gain_rate,
                     coverage_expansion_rate)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(student_id, subject_id) DO UPDATE SET
                    station_clearance_rate = ?3, mastery_gain_rate = ?4,
                    retention_gain_rate = ?5, mock_improvement_rate = ?6,
                    speed_gain_rate = ?7, coverage_expansion_rate = ?8,
                    computed_at = datetime('now')",
                params![
                    student_id,
                    subject_id,
                    station_rate as i64,
                    mastery_rate as i64,
                    retention_rate as i64,
                    mock_rate as i64,
                    speed_rate as i64,
                    coverage_rate as i64,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(VelocitySnapshot {
            student_id,
            subject_id,
            station_clearance_rate: station_rate,
            mastery_gain_rate: mastery_rate,
            retention_gain_rate: retention_rate,
            mock_improvement_rate: mock_rate,
            speed_gain_rate: speed_rate,
            coverage_expansion_rate: coverage_rate,
            overall_velocity_label: overall_label.into(),
        })
    }

    /// Check if the learner can reach their goal given current velocity.
    pub fn check_goal_feasibility(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<GoalFeasibility> {
        let velocity = self.compute_velocity(student_id, subject_id)?;

        // Load remaining work (uncompleted stations)
        let uncompleted_stations: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM journey_stations js
                 INNER JOIN journey_routes jr ON jr.id = js.route_id
                 WHERE jr.student_id = ?1 AND jr.subject_id = ?2 AND jr.status = 'active'
                   AND js.status NOT IN ('completed', 'cleared', 'skipped')",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let work_remaining = clamp_bp(
            (uncompleted_stations as f64 / uncompleted_stations.max(1) as f64 * 10_000.0).round()
                as i64,
        );

        // Load days remaining from deadline pressure
        let days_remaining: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(
                    CAST(julianday(exam_date) - julianday('now') AS INTEGER),
                    90
                 ) FROM journey_routes
                 WHERE student_id = ?1 AND subject_id = ?2 AND status = 'active'
                 ORDER BY created_at DESC LIMIT 1",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .unwrap_or(90);

        // Estimate sessions needed
        let avg_velocity = (velocity.station_clearance_rate as i64).max(1);
        let sessions_needed = if avg_velocity > 0 {
            (uncompleted_stations * 10_000 / avg_velocity).max(1)
        } else {
            uncompleted_stations * 3 // fallback: 3 sessions per station
        };

        let days_needed = (sessions_needed as f64 / 1.5).ceil() as i64; // assume ~1.5 sessions/day

        let capacity =
            clamp_bp((days_remaining as f64 / days_needed.max(1) as f64 * 10_000.0).round() as i64);

        let feasibility = if days_remaining >= days_needed * 2 {
            "comfortable"
        } else if days_remaining >= days_needed {
            "feasible"
        } else if days_remaining as f64 >= days_needed as f64 * 0.7 {
            "challenging"
        } else if days_remaining as f64 >= days_needed as f64 * 0.4 {
            "at_risk"
        } else {
            "needs_adjustment"
        };

        let recommendation = match feasibility {
            "comfortable" => "On track. Consider deepening mastery proof for stronger results.",
            "feasible" => "Achievable with consistent effort. Stay on current pace.",
            "challenging" => "Tight but possible. Consider shifting to high-yield mode.",
            "at_risk" => "Behind schedule. Route compression recommended.",
            _ => "Goal may need adjustment. Consider rescue mode or revised target.",
        };

        // Update plan control state
        self.conn
            .execute(
                "INSERT INTO plan_control_states
                    (student_id, subject_id, work_remaining_score, goal_gap_score, feasibility_status)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(student_id, subject_id) DO UPDATE SET
                    work_remaining_score = ?3, goal_gap_score = ?4,
                    feasibility_status = ?5, last_recalculated_at = datetime('now')",
                params![
                    student_id, subject_id,
                    work_remaining as i64,
                    (10_000 - capacity as i64).max(0),
                    feasibility,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(GoalFeasibility {
            student_id,
            subject_id,
            work_remaining_score: work_remaining,
            capacity_score: capacity,
            velocity_score: clamp_bp(avg_velocity),
            feasibility_status: feasibility.into(),
            sessions_needed_estimate: sessions_needed,
            days_needed_estimate: days_needed,
            recommendation: recommendation.into(),
        })
    }

    /// Compress route: reduce proof burden on low-weight stations.
    pub fn compress_route(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<CompressionAction>> {
        let mut actions = Vec::new();

        // Find low-priority stations that can have proof reduced
        let mut stmt = self
            .conn
            .prepare(
                "SELECT js.id, js.station_code, t.exam_weight, js.target_mastery_score
                 FROM journey_stations js
                 INNER JOIN journey_routes jr ON jr.id = js.route_id
                 LEFT JOIN topics t ON t.id = js.topic_id
                 WHERE jr.student_id = ?1 AND jr.subject_id = ?2 AND jr.status = 'active'
                   AND js.status NOT IN ('completed', 'cleared', 'skipped')
                 ORDER BY COALESCE(t.exam_weight, 5000) ASC
                 LIMIT 5",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows: Vec<(i64, String, i64, Option<i64>)> = stmt
            .query_map(params![student_id, subject_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get::<_, Option<i64>>(2)?.unwrap_or(5000),
                    row.get(3)?,
                ))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        for (station_id, station_code, exam_weight, target_mastery) in rows {
            if exam_weight < 4000 {
                // Low exam weight: reduce mastery target
                let reduced = target_mastery.map(|t| (t * 7 / 10).max(4000));
                if let Some(new_target) = reduced {
                    self.conn
                        .execute(
                            "UPDATE journey_stations SET target_mastery_score = ?1 WHERE id = ?2",
                            params![new_target, station_id],
                        )
                        .map_err(|e| EcoachError::Storage(e.to_string()))?;

                    actions.push(CompressionAction {
                        action_type: "reduce_proof".into(),
                        target_station_id: Some(station_id),
                        reason: format!(
                            "Reduced mastery target for {} (low exam weight {})",
                            station_code, exam_weight
                        ),
                    });
                }
            }
        }

        // Update compression level in plan control
        self.conn
            .execute(
                "UPDATE plan_control_states SET compression_level = compression_level + 1,
                     last_recalculated_at = datetime('now')
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
            )
            .ok(); // non-critical

        Ok(actions)
    }

    /// Deepen route: increase proof burden when time is comfortable.
    pub fn deepen_route(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<CompressionAction>> {
        let mut actions = Vec::new();

        // Find stations where we can raise the bar
        let mut stmt = self
            .conn
            .prepare(
                "SELECT js.id, js.station_code, js.target_mastery_score, js.target_accuracy_score
                 FROM journey_stations js
                 INNER JOIN journey_routes jr ON jr.id = js.route_id
                 WHERE jr.student_id = ?1 AND jr.subject_id = ?2 AND jr.status = 'active'
                   AND js.status NOT IN ('completed', 'cleared', 'skipped')
                   AND COALESCE(js.target_mastery_score, 7000) < 9000
                 LIMIT 5",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows: Vec<(i64, String, Option<i64>, Option<i64>)> = stmt
            .query_map(params![student_id, subject_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        for (station_id, station_code, target_mastery, target_accuracy) in rows {
            let new_mastery = target_mastery.map(|t| (t + 1000).min(9000)).unwrap_or(8000);
            let new_accuracy = target_accuracy.map(|t| (t + 500).min(9000)).unwrap_or(8000);

            self.conn
                .execute(
                    "UPDATE journey_stations SET target_mastery_score = ?1, target_accuracy_score = ?2
                     WHERE id = ?3",
                    params![new_mastery, new_accuracy, station_id],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;

            actions.push(CompressionAction {
                action_type: "deepen_proof".into(),
                target_station_id: Some(station_id),
                reason: format!(
                    "Raised mastery target for {} to {} (time comfortable)",
                    station_code, new_mastery
                ),
            });
        }

        // Update deepening level
        self.conn
            .execute(
                "UPDATE plan_control_states SET deepening_level = deepening_level + 1,
                     last_recalculated_at = datetime('now')
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
            )
            .ok();

        Ok(actions)
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn compute_mock_improvement_rate(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<BasisPoints> {
        let scores: Vec<f64> = {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT percentage FROM mock_sessions
                     WHERE student_id = ?1 AND subject_id = ?2
                       AND status = 'completed' AND percentage IS NOT NULL
                     ORDER BY completed_at DESC LIMIT 3",
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            stmt.query_map(params![student_id, subject_id], |row| row.get(0))
                .map_err(|e| EcoachError::Storage(e.to_string()))?
                .filter_map(|r| r.ok())
                .collect()
        };

        if scores.len() < 2 {
            return Ok(5000); // neutral
        }

        let latest = scores[0];
        let previous = scores[1];
        let delta = latest - previous; // percentage points improvement

        Ok(clamp_bp((5000.0 + delta * 200.0).round() as i64))
    }
}
