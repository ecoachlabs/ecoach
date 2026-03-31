use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// Memory Intelligence Engine: implements MSI, RAS, DCS, DecayRisk formulas,
/// proof dimension satisfaction, confirmation gate, recovery classifier,
/// and adaptive recheck scheduling from idea7.txt spec.
pub struct MemoryIntelligenceEngine<'a> {
    conn: &'a Connection,
}

// ---------------------------------------------------------------------------
// Scoring formulas from idea7
// ---------------------------------------------------------------------------

/// MSI = 0.30*A + 0.15*S + 0.20*R + 0.15*V + 0.10*I + 0.10*C
fn compute_msi(accuracy: f64, speed: f64, retention: f64, variant: f64, independence: f64, connection: f64) -> BasisPoints {
    clamp_bp(((0.30 * accuracy + 0.15 * speed + 0.20 * retention + 0.15 * variant + 0.10 * independence + 0.10 * connection) * 10_000.0).round() as i64)
}

/// RAS = 35*recent_accuracy + 20*speed + 20*independence + 15*consistency + 10*confidence
fn compute_ras(accuracy: f64, speed: f64, independence: f64, consistency: f64, confidence: f64) -> BasisPoints {
    clamp_bp(((0.35 * accuracy + 0.20 * speed + 0.20 * independence + 0.15 * consistency + 0.10 * confidence) * 10_000.0).round() as i64)
}

/// DCS = 25*time_sep + 20*variant + 15*embedded + 15*interference + 15*recheck + 10*relearning
fn compute_dcs(time_sep: f64, variant: f64, embedded: f64, interference: f64, recheck: f64, relearning: f64) -> BasisPoints {
    clamp_bp(((0.25 * time_sep + 0.20 * variant + 0.15 * embedded + 0.15 * interference + 0.15 * recheck + 0.10 * relearning) * 10_000.0).round() as i64)
}

/// DRS = 25*overdue + 20*accuracy_drop + 15*speed_drift + 15*relapse + 15*dependency + 10*interference
fn compute_drs(overdue: f64, accuracy_drop: f64, speed_drift: f64, relapse: f64, dependency: f64, interference: f64) -> BasisPoints {
    clamp_bp(((0.25 * overdue + 0.20 * accuracy_drop + 0.15 * speed_drift + 0.15 * relapse + 0.15 * dependency + 0.10 * interference) * 10_000.0).round() as i64)
}

// ---------------------------------------------------------------------------
// Recovery path types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryPathType {
    PureDecay,
    InterferenceConfusion,
    PrerequisiteBreak,
    SurfaceDependence,
    SpeedOnlyWeakness,
    UnderstandingNotMemory,
}

impl RecoveryPathType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PureDecay => "pure_decay",
            Self::InterferenceConfusion => "interference_confusion",
            Self::PrerequisiteBreak => "prerequisite_break",
            Self::SurfaceDependence => "surface_dependence",
            Self::SpeedOnlyWeakness => "speed_only_weakness",
            Self::UnderstandingNotMemory => "understanding_not_memory",
        }
    }
}

// ---------------------------------------------------------------------------
// Evidence tier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTier {
    Weak,
    Moderate,
    Strong,
    Durable,
}

impl EvidenceTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Weak => "weak",
            Self::Moderate => "moderate",
            Self::Strong => "strong",
            Self::Durable => "durable",
        }
    }

    /// Classify evidence tier from question characteristics
    pub fn classify(
        is_time_separated: bool,
        cue_level: i64,
        is_mixed_context: bool,
        is_interference: bool,
        recall_mode: &str,
    ) -> Self {
        if is_time_separated && (recall_mode == "free_recall" || is_mixed_context) {
            Self::Durable
        } else if is_time_separated || recall_mode == "free_recall" || is_interference {
            Self::Strong
        } else if cue_level <= 1 || recall_mode == "cued_recall" {
            Self::Moderate
        } else {
            Self::Weak
        }
    }

    pub fn weight(self) -> f64 {
        match self {
            Self::Weak => 0.8,
            Self::Moderate => 1.5,
            Self::Strong => 2.5,
            Self::Durable => 3.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryScoreUpdate {
    pub student_id: i64,
    pub node_id: i64,
    pub msi: BasisPoints,
    pub ras: BasisPoints,
    pub dcs: BasisPoints,
    pub drs: BasisPoints,
    pub new_state: String,
    pub state_changed: bool,
    pub dimensions_satisfied: Vec<String>,
    pub recovery_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStatus {
    pub dimension: String,
    pub status: String,
    pub evidence_count: i64,
}

impl<'a> MemoryIntelligenceEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // -----------------------------------------------------------------------
    // Main entry: recompute all scores for a student+skill
    // -----------------------------------------------------------------------

    pub fn recompute_skill_memory(
        &self,
        student_id: i64,
        node_id: i64,
    ) -> EcoachResult<MemoryScoreUpdate> {
        // Load raw metrics
        let (accuracy, speed, retention, variant, independence, connection) =
            self.load_memory_dimensions(student_id, node_id)?;

        let msi = compute_msi(accuracy, speed, retention, variant, independence, connection);

        // RAS components
        let consistency = self.load_consistency(student_id, node_id)?;
        let confidence = self.load_confidence_alignment(student_id, node_id)?;
        let ras = compute_ras(accuracy, speed, independence, consistency, confidence);

        // DCS components
        let time_sep = self.load_time_separated_score(student_id, node_id)?;
        let recheck = self.load_recheck_stability(student_id, node_id)?;
        let relearning = self.load_relearning_efficiency(student_id, node_id)?;
        let interference_res = self.load_interference_resistance(student_id, node_id)?;
        let dcs = compute_dcs(time_sep, variant, connection, interference_res, recheck, relearning);

        // DRS components
        let overdue = self.load_overdue_factor(student_id, node_id)?;
        let acc_drop = self.load_accuracy_drop(student_id, node_id)?;
        let speed_drift = self.load_speed_drift(student_id, node_id)?;
        let relapse_factor = self.load_relapse_factor(student_id, node_id)?;
        let dep_damage = 0.0; // would need prerequisite graph
        let int_growth = self.load_interference_growth(student_id, node_id)?;
        let drs = compute_drs(overdue, acc_drop, speed_drift, relapse_factor, dep_damage, int_growth);

        // Update proof dimensions
        let satisfied_dims = self.update_proof_dimensions(student_id, node_id)?;

        // Run confirmation gate / state machine
        let (new_state, state_changed) = self.run_state_machine(
            student_id, node_id, ras, dcs, drs, &satisfied_dims,
        )?;

        // Classify recovery path if fading/collapsed
        let recovery_path = if new_state == "fading" || new_state == "collapsed" {
            Some(self.classify_recovery_path(student_id, node_id)?)
        } else {
            None
        };

        // Persist scores
        self.conn.execute(
            "UPDATE memory_states SET
                memory_strength = ?1, retrieval_access_score = ?2,
                durability_confidence_score = ?3, decay_risk = ?4,
                memory_state = ?5, recall_fluency = ?6,
                last_state_transition_at = CASE WHEN ?7 = 1 THEN datetime('now') ELSE last_state_transition_at END,
                recovery_path_type = ?8,
                updated_at = datetime('now')
             WHERE student_id = ?9 AND node_id = ?10",
            params![
                msi as i64, ras as i64, dcs as i64, drs as i64,
                new_state, speed as i64 * 100,
                if state_changed { 1 } else { 0 },
                recovery_path.as_deref(),
                student_id, node_id,
            ],
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Schedule adaptive recheck
        self.schedule_adaptive_recheck(student_id, node_id, &new_state)?;

        Ok(MemoryScoreUpdate {
            student_id, node_id,
            msi, ras, dcs, drs,
            new_state: new_state.clone(),
            state_changed,
            dimensions_satisfied: satisfied_dims,
            recovery_path,
        })
    }

    // -----------------------------------------------------------------------
    // Confirmation gate / state machine (idea7 section 11 pseudo-logic)
    // -----------------------------------------------------------------------

    fn run_state_machine(
        &self,
        student_id: i64,
        node_id: i64,
        ras: BasisPoints,
        dcs: BasisPoints,
        drs: BasisPoints,
        satisfied_dims: &[String],
    ) -> EcoachResult<(String, bool)> {
        let current: String = self.conn.query_row(
            "SELECT memory_state FROM memory_states WHERE student_id = ?1 AND node_id = ?2",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or_else(|_| "seen".into());

        let template_ok = self.check_proof_template(student_id, node_id)?;
        let has_delay = satisfied_dims.contains(&"delayed_recall".to_string());
        let has_variant = satisfied_dims.contains(&"variant_transfer".to_string())
            || satisfied_dims.contains(&"representation_shift".to_string());
        let interference_block = self.check_interference_block(student_id, node_id)?;
        let dim_count = satisfied_dims.len();

        let rescue_open = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_sessions WHERE student_id = ?1
             AND primary_skill_ids_json LIKE '%' || ?2 || '%' AND status = 'active'",
            params![student_id, node_id], |row| row.get::<_, i64>(0),
        ).unwrap_or(0) > 0;

        let new_state = if ras < 2500 && dcs < 2000 {
            "collapsed"
        } else if drs > 7500 && ras < 4500 {
            "fading"
        } else if rescue_open {
            "rebuilding"
        } else if template_ok && has_delay && has_variant && !interference_block
            && ras >= 7000 && dcs >= 7000
        {
            // Check for locked_in (stronger than confirmed)
            if ras >= 8000 && dcs >= 8500 && self.has_long_interval_stability(student_id, node_id)? {
                "locked_in"
            } else {
                "confirmed"
            }
        } else if current == "recovered" && template_ok && has_delay {
            "confirmed"
        } else if drs > 5500 && ras >= 5500 {
            "at_risk"
        } else if ras >= 5500 && dim_count >= 2 {
            "anchoring"
        } else if ras >= 4500 {
            "fragile"
        } else if ras >= 3000 {
            "accessible"
        } else {
            "encoded"
        };

        let changed = new_state != current;
        Ok((new_state.into(), changed))
    }

    // -----------------------------------------------------------------------
    // Proof dimension satisfaction
    // -----------------------------------------------------------------------

    fn update_proof_dimensions(
        &self,
        student_id: i64,
        node_id: i64,
    ) -> EcoachResult<Vec<String>> {
        let dimensions = [
            "independent_recall", "delayed_recall", "variant_transfer",
            "embedded_use", "interference_resistance", "explanation_reasoning",
            "representation_shift", "sequence_relation", "speed_fluency",
        ];

        let mut satisfied = Vec::new();

        for dim in &dimensions {
            let supports_col = format!("supports_{}", dim);
            let count: i64 = self.conn.query_row(
                &format!(
                    "SELECT COUNT(*) FROM memory_evidence_events
                     WHERE student_id = ?1 AND node_id = ?2 AND {} = 1 AND is_correct = 1",
                    supports_col
                ),
                params![student_id, node_id],
                |row| row.get(0),
            ).unwrap_or(0);

            let status = if count >= 2 { "satisfied" } else if count >= 1 { "partial" } else { "none" };

            self.conn.execute(
                "INSERT INTO memory_proof_dimensions
                    (student_id, node_id, dimension_name, status, evidence_count)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(student_id, node_id, dimension_name) DO UPDATE SET
                    status = ?4, evidence_count = ?5",
                params![student_id, node_id, dim, status, count],
            ).map_err(|e| EcoachError::Storage(e.to_string()))?;

            if status == "satisfied" {
                satisfied.push(dim.to_string());
            }
        }

        Ok(satisfied)
    }

    fn check_proof_template(&self, student_id: i64, node_id: i64) -> EcoachResult<bool> {
        // Load template for this node's topic
        let required_json: Option<String> = self.conn.query_row(
            "SELECT mpt.required_dimensions_json
             FROM memory_proof_templates mpt
             INNER JOIN memory_states ms ON ms.proof_template_id = mpt.id
             WHERE ms.student_id = ?1 AND ms.node_id = ?2",
            params![student_id, node_id], |row| row.get(0),
        ).optional().map_err(|e| EcoachError::Storage(e.to_string()))?.flatten();

        let Some(json_str) = required_json else {
            return Ok(true); // no template = pass by default
        };

        let required: Vec<String> = serde_json::from_str(&json_str).unwrap_or_default();
        if required.is_empty() { return Ok(true); }

        // Check each required dimension is satisfied
        for dim in &required {
            let status: String = self.conn.query_row(
                "SELECT COALESCE(status, 'none') FROM memory_proof_dimensions
                 WHERE student_id = ?1 AND node_id = ?2 AND dimension_name = ?3",
                params![student_id, node_id, dim], |row| row.get(0),
            ).unwrap_or_else(|_| "none".into());

            if status != "satisfied" {
                return Ok(false);
            }
        }

        Ok(true)
    }

    // -----------------------------------------------------------------------
    // Recovery path classifier (idea7 section 14)
    // -----------------------------------------------------------------------

    fn classify_recovery_path(&self, student_id: i64, node_id: i64) -> EcoachResult<String> {
        let interference_risk: i64 = self.conn.query_row(
            "SELECT COALESCE(interference_risk_score, 0) FROM memory_states
             WHERE student_id = ?1 AND node_id = ?2",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);

        let relapse: i64 = self.conn.query_row(
            "SELECT COALESCE(relapse_count, 0) FROM memory_states
             WHERE student_id = ?1 AND node_id = ?2",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);

        // Check if accuracy is decent but speed is poor
        let (accuracy, speed): (f64, f64) = self.load_accuracy_and_speed(student_id, node_id)?;

        // Check variant vs canonical performance gap
        let variant_gap = self.load_variant_gap(student_id, node_id)?;

        // Check if the problem is understanding, not memory
        let understanding_failure = self.check_understanding_failure(student_id, node_id)?;

        let path = if understanding_failure {
            RecoveryPathType::UnderstandingNotMemory
        } else if interference_risk > 6000 {
            RecoveryPathType::InterferenceConfusion
        } else if accuracy >= 0.7 && speed < 0.4 {
            RecoveryPathType::SpeedOnlyWeakness
        } else if variant_gap > 0.3 {
            RecoveryPathType::SurfaceDependence
        } else if relapse >= 3 {
            RecoveryPathType::PrerequisiteBreak
        } else {
            RecoveryPathType::PureDecay
        };

        // Store the classification
        self.conn.execute(
            "UPDATE memory_states SET recovery_path_type = ?1 WHERE student_id = ?2 AND node_id = ?3",
            params![path.as_str(), student_id, node_id],
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(path.as_str().into())
    }

    // -----------------------------------------------------------------------
    // Adaptive recheck scheduler (idea7 section 12)
    // -----------------------------------------------------------------------

    fn schedule_adaptive_recheck(
        &self,
        student_id: i64,
        node_id: i64,
        state: &str,
    ) -> EcoachResult<()> {
        let (days_offset, schedule_type, target_dim) = match state {
            "fragile" => (1, "short_gap", Some("delayed_recall")),
            "anchoring" => (3, "medium_gap", Some("variant_transfer")),
            "confirmed" => (10, "long_gap", Some("delayed_recall")),
            "locked_in" => (30, "long_gap", Some("speed_fluency")),
            "at_risk" => (1, "relapse_followup", None),
            "recovered" => (2, "short_gap", Some("delayed_recall")),
            "rebuilding" => (1, "relapse_followup", None),
            _ => return Ok(()), // no recheck for seen/encoded/accessible/collapsed
        };

        let due_at = (chrono::Utc::now() + chrono::Duration::days(days_offset))
            .format("%Y-%m-%d %H:%M:%S").to_string();

        self.conn.execute(
            "INSERT OR REPLACE INTO recheck_schedules
                (student_id, node_id, due_at, schedule_type, status, target_proof_dimension)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
            params![student_id, node_id, due_at, schedule_type, target_dim],
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Get proof status for a skill
    // -----------------------------------------------------------------------

    pub fn get_proof_status(
        &self,
        student_id: i64,
        node_id: i64,
    ) -> EcoachResult<Vec<ProofStatus>> {
        let mut stmt = self.conn.prepare(
            "SELECT dimension_name, status, evidence_count
             FROM memory_proof_dimensions
             WHERE student_id = ?1 AND node_id = ?2
             ORDER BY dimension_name",
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt.query_map(params![student_id, node_id], |row| {
            Ok(ProofStatus {
                dimension: row.get(0)?,
                status: row.get(1)?,
                evidence_count: row.get(2)?,
            })
        }).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows { result.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?); }
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Dimension score loaders (simplified — would be richer in production)
    // -----------------------------------------------------------------------

    fn load_memory_dimensions(&self, student_id: i64, node_id: i64) -> EcoachResult<(f64, f64, f64, f64, f64, f64)> {
        let (total, correct, avg_time): (i64, i64, i64) = self.conn.query_row(
            "SELECT COUNT(*), SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END),
                    COALESCE(AVG(CASE WHEN is_correct = 1 THEN response_time_ms END), 30000)
             FROM memory_evidence_events WHERE student_id = ?1 AND node_id = ?2",
            params![student_id, node_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap_or((0, 0, 30000));

        let accuracy = if total > 0 { correct as f64 / total as f64 } else { 0.0 };
        let speed = (1.0 - (avg_time as f64 / 60000.0).min(1.0)).max(0.0);

        let time_sep_success: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2 AND is_time_separated = 1 AND is_correct = 1",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        let retention = (time_sep_success as f64 / 3.0).min(1.0);

        let variant_success: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2 AND variant_type IS NOT NULL
               AND variant_type != 'canonical' AND is_correct = 1",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        let variant = (variant_success as f64 / 3.0).min(1.0);

        let no_hint: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2 AND hint_used = 0 AND is_correct = 1",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        let independence = if total > 0 { no_hint as f64 / total as f64 } else { 0.0 };

        let mixed_success: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2 AND is_mixed_context = 1 AND is_correct = 1",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        let connection = (mixed_success as f64 / 2.0).min(1.0);

        Ok((accuracy, speed, retention, variant, independence, connection))
    }

    fn load_consistency(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        // Ratio of correct in last 5 vs overall
        let recent_acc: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(is_correct), 0.0) FROM (
                 SELECT is_correct FROM memory_evidence_events
                 WHERE student_id = ?1 AND node_id = ?2 ORDER BY occurred_at DESC LIMIT 5
             )", params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0.0);
        Ok(recent_acc)
    }

    fn load_confidence_alignment(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        Ok(0.5) // simplified — would compare confidence_self_rating vs actual correctness
    }

    fn load_time_separated_score(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        let count: i64 = self.conn.query_row(
            "SELECT COALESCE(time_separated_success_count, 0) FROM memory_states
             WHERE student_id = ?1 AND node_id = ?2",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        Ok((count as f64 / 3.0).min(1.0))
    }

    fn load_recheck_stability(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        let passed: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM recheck_schedules
             WHERE student_id = ?1 AND node_id = ?2 AND status = 'completed'",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        Ok((passed as f64 / 3.0).min(1.0))
    }

    fn load_relearning_efficiency(&self, _student_id: i64, _node_id: i64) -> EcoachResult<f64> {
        Ok(0.5) // simplified
    }

    fn load_interference_resistance(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        let total_interference: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2 AND interference_detected = 1",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        let correct_interference: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2 AND interference_detected = 1 AND is_correct = 1",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        if total_interference == 0 { return Ok(0.5); }
        Ok(correct_interference as f64 / total_interference as f64)
    }

    fn load_overdue_factor(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        let overdue: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM recheck_schedules
             WHERE student_id = ?1 AND node_id = ?2 AND status = 'pending'
               AND due_at < datetime('now')",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        Ok((overdue as f64 / 3.0).min(1.0))
    }

    fn load_accuracy_drop(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        let recent: f64 = self.load_consistency(student_id, node_id)?;
        let overall: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(is_correct), 0.5) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0.5);
        Ok((overall - recent).max(0.0).min(1.0))
    }

    fn load_speed_drift(&self, _student_id: i64, _node_id: i64) -> EcoachResult<f64> {
        Ok(0.0) // simplified
    }

    fn load_relapse_factor(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        let relapses: i64 = self.conn.query_row(
            "SELECT COALESCE(relapse_count, 0) FROM memory_states
             WHERE student_id = ?1 AND node_id = ?2",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        Ok((relapses as f64 / 5.0).min(1.0))
    }

    fn load_interference_growth(&self, _student_id: i64, _node_id: i64) -> EcoachResult<f64> {
        Ok(0.0) // simplified
    }

    fn check_interference_block(&self, student_id: i64, node_id: i64) -> EcoachResult<bool> {
        let risk: i64 = self.conn.query_row(
            "SELECT COALESCE(interference_risk_score, 0) FROM memory_states
             WHERE student_id = ?1 AND node_id = ?2",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);

        let interference_satisfied: String = self.conn.query_row(
            "SELECT COALESCE(status, 'none') FROM memory_proof_dimensions
             WHERE student_id = ?1 AND node_id = ?2 AND dimension_name = 'interference_resistance'",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or_else(|_| "none".into());

        Ok(risk > 7000 && interference_satisfied != "satisfied")
    }

    fn has_long_interval_stability(&self, student_id: i64, node_id: i64) -> EcoachResult<bool> {
        let long_gap_successes: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2
               AND delay_bucket IN ('long', 'very_long') AND is_correct = 1",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0);
        Ok(long_gap_successes >= 2)
    }

    fn load_accuracy_and_speed(&self, student_id: i64, node_id: i64) -> EcoachResult<(f64, f64)> {
        let dims = self.load_memory_dimensions(student_id, node_id)?;
        Ok((dims.0, dims.1))
    }

    fn load_variant_gap(&self, student_id: i64, node_id: i64) -> EcoachResult<f64> {
        let canonical_acc: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(is_correct), 0.5) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2
               AND (variant_type IS NULL OR variant_type = 'canonical')",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0.5);
        let variant_acc: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(is_correct), 0.5) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2
               AND variant_type IS NOT NULL AND variant_type != 'canonical'",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0.5);
        Ok((canonical_acc - variant_acc).max(0.0))
    }

    fn check_understanding_failure(&self, student_id: i64, node_id: i64) -> EcoachResult<bool> {
        // If accuracy is very low even with heavy cues, it's understanding not memory
        let heavy_cue_accuracy: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(is_correct), 0.5) FROM memory_evidence_events
             WHERE student_id = ?1 AND node_id = ?2 AND cue_level = 'heavy'",
            params![student_id, node_id], |row| row.get(0),
        ).unwrap_or(0.5);
        Ok(heavy_cue_accuracy < 0.4)
    }
}
