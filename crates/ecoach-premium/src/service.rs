use chrono::Utc;
use ecoach_substrate::{DomainEvent, EcoachError, EcoachResult, EntitlementTier};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    CreateInterventionInput, CreateRiskFlagInput, InterventionRecord, InterventionStatus,
    InterventionStep, PremiumFeature, RiskDashboard, RiskFlag, RiskFlagStatus, RiskSeverity,
    StudentEntitlementSnapshot,
};

// ── Mastery thresholds for auto-detection ──
const RISK_CRITICAL_MASTERY_BP: i64 = 2500;
const RISK_HIGH_MASTERY_BP: i64 = 4000;
const INACTIVITY_DANGER_DAYS: i64 = 3;

pub struct PremiumService<'a> {
    conn: &'a Connection,
}

impl<'a> PremiumService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // ── Entitlement gating ──

    pub fn check_entitlement(
        &self,
        student_id: i64,
    ) -> EcoachResult<EntitlementTier> {
        let tier_str: String = self
            .conn
            .query_row(
                "SELECT entitlement_tier FROM accounts WHERE id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|e| {
                EcoachError::NotFound(format!("account {} not found: {}", student_id, e))
            })?;

        match tier_str.as_str() {
            "premium" => Ok(EntitlementTier::Premium),
            "elite" => Ok(EntitlementTier::Elite),
            _ => Ok(EntitlementTier::Standard),
        }
    }

    pub fn require_premium_or_elite(&self, student_id: i64) -> EcoachResult<EntitlementTier> {
        let tier = self.check_entitlement(student_id)?;
        match tier {
            EntitlementTier::Standard => Err(EcoachError::Unauthorized(
                "premium or elite entitlement required".to_string(),
            )),
            _ => Ok(tier),
        }
    }

    pub fn is_feature_enabled(
        &self,
        student_id: i64,
        feature_key: &str,
    ) -> EcoachResult<bool> {
        let tier = self.check_entitlement(student_id)?;

        // Check feature flag table for overrides
        let override_enabled: Option<i64> = self
            .conn
            .query_row(
                "SELECT enabled FROM premium_feature_flags
                 WHERE feature_key = ?1 AND (student_id IS NULL OR student_id = ?2)
                 ORDER BY student_id DESC
                 LIMIT 1",
                params![feature_key, student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        if let Some(enabled) = override_enabled {
            return Ok(enabled == 1);
        }

        // Default tier-based gating
        let required_tier: Option<String> = self
            .conn
            .query_row(
                "SELECT tier_required FROM premium_features WHERE feature_key = ?1",
                [feature_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        match required_tier {
            Some(required) => match required.as_str() {
                "standard" => Ok(true),
                "premium" => Ok(matches!(tier, EntitlementTier::Premium | EntitlementTier::Elite)),
                "elite" => Ok(matches!(tier, EntitlementTier::Elite)),
                _ => Ok(false),
            },
            None => Ok(false), // unknown feature
        }
    }

    pub fn list_features_for_student(
        &self,
        student_id: i64,
    ) -> EcoachResult<Vec<PremiumFeature>> {
        let tier = self.check_entitlement(student_id)?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT pf.feature_key, pf.display_name, pf.tier_required,
                        COALESCE(pff.enabled, CASE
                            WHEN pf.tier_required = 'standard' THEN 1
                            WHEN pf.tier_required = 'premium' AND ?1 IN ('premium', 'elite') THEN 1
                            WHEN pf.tier_required = 'elite' AND ?1 = 'elite' THEN 1
                            ELSE 0
                        END) AS enabled
                 FROM premium_features pf
                 LEFT JOIN premium_feature_flags pff
                     ON pff.feature_key = pf.feature_key
                     AND (pff.student_id = ?2 OR pff.student_id IS NULL)
                 ORDER BY pf.display_name",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![tier.as_str(), student_id], |row| {
                Ok(PremiumFeature {
                    feature_key: row.get(0)?,
                    display_name: row.get(1)?,
                    tier_required: row.get(2)?,
                    enabled: row.get::<_, i64>(3)? == 1,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut features = Vec::new();
        for row in rows {
            features.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(features)
    }

    // ── Risk flags ──

    pub fn create_risk_flag(&self, input: &CreateRiskFlagInput) -> EcoachResult<RiskFlag> {
        self.require_premium_or_elite(input.student_id)?;

        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO risk_flags (student_id, topic_id, severity, title, description, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6)",
                params![
                    input.student_id,
                    input.topic_id,
                    input.severity.as_str(),
                    input.title,
                    input.description,
                    now,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let flag_id = self.conn.last_insert_rowid();

        self.append_event(
            "risk",
            DomainEvent::new(
                "premium.risk_flag_created",
                flag_id.to_string(),
                json!({
                    "student_id": input.student_id,
                    "topic_id": input.topic_id,
                    "severity": input.severity.as_str(),
                    "title": input.title,
                }),
            ),
        )?;

        self.get_risk_flag(flag_id)
    }

    pub fn get_risk_flag(&self, flag_id: i64) -> EcoachResult<RiskFlag> {
        self.conn
            .query_row(
                "SELECT rf.id, rf.student_id, rf.topic_id, t.name, rf.severity, rf.title,
                        rf.description, rf.status, rf.created_at, rf.resolved_at
                 FROM risk_flags rf
                 LEFT JOIN topics t ON t.id = rf.topic_id
                 WHERE rf.id = ?1",
                [flag_id],
                |row| {
                    Ok(RiskFlag {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        topic_name: row.get(3)?,
                        severity: row.get(4)?,
                        title: row.get(5)?,
                        description: row.get(6)?,
                        status: row.get(7)?,
                        created_at: row.get(8)?,
                        resolved_at: row.get(9)?,
                    })
                },
            )
            .map_err(|e| EcoachError::NotFound(format!("risk flag {} not found: {}", flag_id, e)))
    }

    pub fn list_active_risk_flags(&self, student_id: i64) -> EcoachResult<Vec<RiskFlag>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT rf.id, rf.student_id, rf.topic_id, t.name, rf.severity, rf.title,
                        rf.description, rf.status, rf.created_at, rf.resolved_at
                 FROM risk_flags rf
                 LEFT JOIN topics t ON t.id = rf.topic_id
                 WHERE rf.student_id = ?1 AND rf.status IN ('active', 'monitoring')
                 ORDER BY CASE rf.severity
                    WHEN 'critical' THEN 0
                    WHEN 'high' THEN 1
                    WHEN 'medium' THEN 2
                    WHEN 'low' THEN 3
                 END, rf.created_at DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([student_id], |row| {
                Ok(RiskFlag {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    topic_id: row.get(2)?,
                    topic_name: row.get(3)?,
                    severity: row.get(4)?,
                    title: row.get(5)?,
                    description: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    resolved_at: row.get(9)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut flags = Vec::new();
        for row in rows {
            flags.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(flags)
    }

    pub fn update_risk_flag_status(
        &self,
        flag_id: i64,
        new_status: RiskFlagStatus,
    ) -> EcoachResult<RiskFlag> {
        let resolved_clause = if matches!(new_status, RiskFlagStatus::Resolved) {
            ", resolved_at = datetime('now')"
        } else {
            ""
        };
        let sql = format!(
            "UPDATE risk_flags SET status = ?1{} WHERE id = ?2",
            resolved_clause
        );
        let affected = self
            .conn
            .execute(&sql, params![new_status.as_str(), flag_id])
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(EcoachError::NotFound(format!(
                "risk flag {} not found",
                flag_id
            )));
        }

        self.append_event(
            "risk",
            DomainEvent::new(
                "premium.risk_flag_status_changed",
                flag_id.to_string(),
                json!({ "new_status": new_status.as_str() }),
            ),
        )?;

        self.get_risk_flag(flag_id)
    }

    pub fn auto_detect_risk_flags(&self, student_id: i64) -> EcoachResult<Vec<RiskFlag>> {
        self.require_premium_or_elite(student_id)?;

        let mut created_flags = Vec::new();

        // Detect topics with critically low mastery
        let mut stmt = self
            .conn
            .prepare(
                "SELECT sts.topic_id, t.name, sts.mastery_score
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1 AND sts.mastery_score < ?2
                 AND NOT EXISTS (
                     SELECT 1 FROM risk_flags rf
                     WHERE rf.student_id = sts.student_id
                     AND rf.topic_id = sts.topic_id
                     AND rf.status IN ('active', 'monitoring')
                 )",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let weak_topics: Vec<(i64, String, i64)> = stmt
            .query_map(params![student_id, RISK_HIGH_MASTERY_BP], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        for (topic_id, topic_name, mastery) in weak_topics {
            let severity = if mastery < RISK_CRITICAL_MASTERY_BP {
                RiskSeverity::Critical
            } else if mastery < RISK_HIGH_MASTERY_BP {
                RiskSeverity::High
            } else {
                RiskSeverity::Medium
            };

            let flag = self.create_risk_flag(&CreateRiskFlagInput {
                student_id,
                topic_id: Some(topic_id),
                severity,
                title: format!("Low mastery in {}", topic_name),
                description: Some(format!(
                    "Mastery score is {} bp, below the {} bp threshold",
                    mastery,
                    RISK_HIGH_MASTERY_BP
                )),
            })?;
            created_flags.push(flag);
        }

        // Detect inactivity risk
        let inactive_days: Option<i64> = self
            .conn
            .query_row(
                "SELECT CAST(julianday('now') - julianday(MAX(started_at)) AS INTEGER)
                 FROM sessions WHERE student_id = ?1 AND status IN ('active', 'completed')",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .flatten();

        if let Some(days) = inactive_days {
            if days >= INACTIVITY_DANGER_DAYS {
                let existing_inactivity: Option<i64> = self
                    .conn
                    .query_row(
                        "SELECT id FROM risk_flags
                         WHERE student_id = ?1 AND topic_id IS NULL AND title LIKE 'Inactivity%'
                         AND status IN ('active', 'monitoring')",
                        [student_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;

                if existing_inactivity.is_none() {
                    let severity = if days >= INACTIVITY_DANGER_DAYS * 3 {
                        RiskSeverity::High
                    } else {
                        RiskSeverity::Medium
                    };
                    let flag = self.create_risk_flag(&CreateRiskFlagInput {
                        student_id,
                        topic_id: None,
                        severity,
                        title: format!("Inactivity: {} days since last session", days),
                        description: Some(
                            "Student has not completed a study session within the danger threshold"
                                .to_string(),
                        ),
                    })?;
                    created_flags.push(flag);
                }
            }
        }

        Ok(created_flags)
    }

    // ── Interventions ──

    pub fn create_intervention(
        &self,
        input: &CreateInterventionInput,
    ) -> EcoachResult<InterventionRecord> {
        self.require_premium_or_elite(input.student_id)?;

        let steps_json = serde_json::to_string(&input.steps)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO intervention_records (
                    student_id, risk_flag_id, title, status, summary_json, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?5)",
                params![
                    input.student_id,
                    input.risk_flag_id,
                    input.title,
                    steps_json,
                    now,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let intervention_id = self.conn.last_insert_rowid();

        self.append_event(
            "intervention",
            DomainEvent::new(
                "premium.intervention_created",
                intervention_id.to_string(),
                json!({
                    "student_id": input.student_id,
                    "risk_flag_id": input.risk_flag_id,
                    "title": input.title,
                    "step_count": input.steps.len(),
                }),
            ),
        )?;

        self.get_intervention(intervention_id)
    }

    pub fn get_intervention(&self, intervention_id: i64) -> EcoachResult<InterventionRecord> {
        self.conn
            .query_row(
                "SELECT id, student_id, risk_flag_id, title, status, summary_json, created_at, updated_at
                 FROM intervention_records WHERE id = ?1",
                [intervention_id],
                |row| {
                    let steps_json: String = row.get(5)?;
                    let steps: Vec<InterventionStep> =
                        serde_json::from_str(&steps_json).unwrap_or_default();
                    Ok(InterventionRecord {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        risk_flag_id: row.get(2)?,
                        title: row.get(3)?,
                        status: row.get(4)?,
                        progress_percent: 0,
                        steps,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| {
                EcoachError::NotFound(format!(
                    "intervention {} not found: {}",
                    intervention_id, e
                ))
            })
    }

    pub fn list_active_interventions(
        &self,
        student_id: i64,
    ) -> EcoachResult<Vec<InterventionRecord>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, student_id, risk_flag_id, title, status, summary_json, created_at, updated_at
                 FROM intervention_records
                 WHERE student_id = ?1 AND status IN ('active', 'review')
                 ORDER BY created_at DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([student_id], |row| {
                let steps_json: String = row.get(5)?;
                let steps: Vec<InterventionStep> =
                    serde_json::from_str(&steps_json).unwrap_or_default();
                Ok(InterventionRecord {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    risk_flag_id: row.get(2)?,
                    title: row.get(3)?,
                    status: row.get(4)?,
                    progress_percent: 0,
                    steps,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut interventions = Vec::new();
        for row in rows {
            interventions.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(interventions)
    }

    pub fn update_intervention_status(
        &self,
        intervention_id: i64,
        new_status: InterventionStatus,
    ) -> EcoachResult<InterventionRecord> {
        let affected = self
            .conn
            .execute(
                "UPDATE intervention_records SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
                params![new_status.as_str(), intervention_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(EcoachError::NotFound(format!(
                "intervention {} not found",
                intervention_id
            )));
        }

        // If intervention resolved and linked to a risk flag, set flag to monitoring
        if matches!(new_status, InterventionStatus::Resolved) {
            let risk_flag_id: Option<i64> = self
                .conn
                .query_row(
                    "SELECT risk_flag_id FROM intervention_records WHERE id = ?1",
                    [intervention_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| EcoachError::Storage(e.to_string()))?
                .flatten();

            if let Some(flag_id) = risk_flag_id {
                self.conn
                    .execute(
                        "UPDATE risk_flags SET status = 'monitoring' WHERE id = ?1 AND status = 'active'",
                        [flag_id],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
            }
        }

        self.append_event(
            "intervention",
            DomainEvent::new(
                "premium.intervention_status_changed",
                intervention_id.to_string(),
                json!({ "new_status": new_status.as_str() }),
            ),
        )?;

        self.get_intervention(intervention_id)
    }

    // ── Risk dashboard ──

    pub fn get_risk_dashboard(&self, student_id: i64) -> EcoachResult<RiskDashboard> {
        self.require_premium_or_elite(student_id)?;

        let counts: (i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT
                    SUM(CASE WHEN severity = 'critical' AND status = 'active' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN severity = 'high' AND status = 'active' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN severity = 'medium' AND status = 'active' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN severity = 'low' AND status = 'active' THEN 1 ELSE 0 END)
                 FROM risk_flags WHERE student_id = ?1",
                [student_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let flags = self.list_active_risk_flags(student_id)?;
        let interventions = self.list_active_interventions(student_id)?;

        Ok(RiskDashboard {
            student_id,
            critical_count: counts.0,
            high_count: counts.1,
            medium_count: counts.2,
            low_count: counts.3,
            active_interventions: interventions.len() as i64,
            flags,
            interventions,
        })
    }

    // ── Entitlement snapshot ──

    pub fn get_entitlement_snapshot(
        &self,
        student_id: i64,
    ) -> EcoachResult<StudentEntitlementSnapshot> {
        let tier = self.check_entitlement(student_id)?;

        let active_flags: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM risk_flags WHERE student_id = ?1 AND status = 'active'",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let active_interventions: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM intervention_records WHERE student_id = ?1 AND status IN ('active', 'review')",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let features = self.list_features_for_student(student_id)?;
        let enabled_keys: Vec<String> = features
            .into_iter()
            .filter(|f| f.enabled)
            .map(|f| f.feature_key)
            .collect();

        Ok(StudentEntitlementSnapshot {
            student_id,
            tier: tier.as_str().to_string(),
            active_risk_flags: active_flags,
            active_interventions,
            premium_features_enabled: enabled_keys,
        })
    }

    // ── Internal ──

    fn append_event(&self, aggregate_kind: &str, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    event.event_id,
                    event.event_type,
                    aggregate_kind,
                    event.aggregate_id,
                    event.trace_id,
                    payload_json,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }
}
