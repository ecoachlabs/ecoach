use std::collections::BTreeSet;
use std::str::FromStr;

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use ecoach_substrate::{
    clamp_bp, BasisPoints, DomainEvent, EcoachError, EcoachResult, EngineRegistry, EntitlementTier,
    FabricOrchestrationSummary,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;

use crate::models::{
    CreateInterventionInput, CreateRiskFlagInput, InterventionRecord, InterventionStatus,
    InterventionStep, PremiumFeature, PremiumPriorityTopic, PremiumStrategySnapshot, RiskDashboard,
    RiskFlag, RiskFlagStatus, RiskSeverity, StudentEntitlementSnapshot,
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

    pub fn check_entitlement(&self, student_id: i64) -> EcoachResult<EntitlementTier> {
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

    pub fn is_feature_enabled(&self, student_id: i64, feature_key: &str) -> EcoachResult<bool> {
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
                "premium" => Ok(matches!(
                    tier,
                    EntitlementTier::Premium | EntitlementTier::Elite
                )),
                "elite" => Ok(matches!(tier, EntitlementTier::Elite)),
                _ => Ok(false),
            },
            None => Ok(false), // unknown feature
        }
    }

    pub fn list_features_for_student(&self, student_id: i64) -> EcoachResult<Vec<PremiumFeature>> {
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
                    mastery, RISK_HIGH_MASTERY_BP
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
        let row = self
            .conn
            .query_row(
                "SELECT id, student_id, risk_flag_id, title, status, summary_json, created_at, updated_at
                 FROM intervention_records WHERE id = ?1",
                [intervention_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                    ))
                },
            )
            .map_err(|e| {
                EcoachError::NotFound(format!(
                    "intervention {} not found: {}",
                    intervention_id, e
                ))
            })?;

        self.build_intervention_record(row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7)
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
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut interventions = Vec::new();
        for row in rows {
            let row = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
            interventions.push(self.build_intervention_record(
                row.0, row.1, row.2, row.3, row.4, row.5, row.6, row.7,
            )?);
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

    pub fn get_strategy_snapshot(&self, student_id: i64) -> EcoachResult<PremiumStrategySnapshot> {
        let tier = self.require_premium_or_elite(student_id)?;
        let (student_name, exam_target, exam_target_date, profile_budget): (
            String,
            Option<String>,
            Option<String>,
            Option<i64>,
        ) = self
            .conn
            .query_row(
                "SELECT a.display_name, sp.exam_target, sp.exam_target_date, sp.daily_study_budget_minutes
                 FROM accounts a
                 LEFT JOIN student_profiles sp ON sp.account_id = a.id
                 WHERE a.id = ?1",
                [student_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let overall_readiness_score: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(CAST(AVG(mastery_score) AS INTEGER), 0)
                 FROM student_topic_states
                 WHERE student_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let overall_readiness_band = readiness_band(overall_readiness_score).to_string();

        let (active_risk_count, critical_risk_count): (i64, i64) = self
            .conn
            .query_row(
                "SELECT
                    COUNT(*),
                    SUM(CASE WHEN severity = 'critical' THEN 1 ELSE 0 END)
                 FROM risk_flags
                 WHERE student_id = ?1 AND status IN ('active', 'monitoring')",
                [student_id],
                |row| Ok((row.get(0)?, row.get::<_, Option<i64>>(1)?.unwrap_or(0))),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let active_intervention_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM intervention_records
                 WHERE student_id = ?1 AND status IN ('active', 'review', 'escalated')",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let overdue_review_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM memory_states
                 WHERE student_id = ?1
                   AND review_due_at IS NOT NULL
                   AND review_due_at <= ?2
                   AND decay_risk >= 5000",
                params![student_id, Utc::now().to_rfc3339()],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let (current_phase, plan_budget): (Option<String>, Option<i64>) = self
            .conn
            .query_row(
                "SELECT current_phase, daily_budget_minutes
                 FROM coach_plans
                 WHERE student_id = ?1 AND status IN ('active', 'stale')
                 ORDER BY updated_at DESC, id DESC
                 LIMIT 1",
                [student_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .unwrap_or((None, None));

        let priority_topics = self.list_priority_topics(student_id, 5)?;
        let top_risk_titles = self.list_top_risk_titles(student_id, 3)?;
        let inactive_days = self.inactive_days(student_id)?;
        let daily_budget_minutes = plan_budget.or(profile_budget);
        let recent_focus_signals = self.recent_focus_signals(student_id, 6)?;
        let recommended_game_modes = self.recommended_game_modes(student_id, 3)?;
        let strategy_mode = resolve_strategy_mode(
            overall_readiness_score,
            critical_risk_count,
            active_risk_count,
            overdue_review_count,
            inactive_days,
        )
        .to_string();
        let coach_actions = build_coach_actions(
            &strategy_mode,
            &priority_topics,
            overdue_review_count,
            active_intervention_count,
            inactive_days,
            current_phase.as_deref(),
            &recent_focus_signals,
            &recommended_game_modes,
        );
        let household_actions = build_household_actions(
            &strategy_mode,
            overdue_review_count,
            critical_risk_count,
            active_intervention_count,
            inactive_days,
            daily_budget_minutes,
            &recent_focus_signals,
            &recommended_game_modes,
        );
        let orchestration = FabricOrchestrationSummary::from_available_inputs(
            &EngineRegistry::core_runtime(),
            strategy_available_inputs(
                active_risk_count,
                overdue_review_count,
                !priority_topics.is_empty(),
                !recent_focus_signals.is_empty(),
            ),
        );

        Ok(PremiumStrategySnapshot {
            student_id,
            student_name,
            tier: tier.as_str().to_string(),
            strategy_mode,
            overall_readiness_score: overall_readiness_score.clamp(0, 10_000) as BasisPoints,
            overall_readiness_band,
            exam_target,
            exam_target_date,
            current_phase,
            daily_budget_minutes,
            inactive_days,
            overdue_review_count,
            active_risk_count,
            critical_risk_count,
            active_intervention_count,
            priority_topics,
            top_risk_titles,
            recent_focus_signals,
            recommended_game_modes,
            coach_actions,
            household_actions,
            orchestration,
        })
    }

    fn build_intervention_record(
        &self,
        id: i64,
        student_id: i64,
        risk_flag_id: Option<i64>,
        title: String,
        status: String,
        steps_json: String,
        created_at: String,
        updated_at: String,
    ) -> EcoachResult<InterventionRecord> {
        let steps: Vec<InterventionStep> = serde_json::from_str(&steps_json).unwrap_or_default();
        let progress_percent = self.compute_intervention_progress(student_id, &steps)?;
        Ok(InterventionRecord {
            id,
            student_id,
            risk_flag_id,
            title,
            status,
            steps,
            progress_percent: progress_percent.clamp(0, 10_000) as BasisPoints,
            created_at,
            updated_at,
        })
    }

    fn compute_intervention_progress(
        &self,
        student_id: i64,
        steps: &[InterventionStep],
    ) -> EcoachResult<i64> {
        if steps.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        for step in steps {
            total += self.score_intervention_step(student_id, step)?;
        }
        Ok((total / steps.len() as i64).clamp(0, 10_000))
    }

    fn score_intervention_step(
        &self,
        student_id: i64,
        step: &InterventionStep,
    ) -> EcoachResult<i64> {
        if let Some(topic_id) = step.target_topic_id {
            let state: Option<(i64, Option<String>, i64)> = self
                .conn
                .query_row(
                    "SELECT mastery_score, last_seen_at, is_blocked
                     FROM student_topic_states
                     WHERE student_id = ?1 AND topic_id = ?2",
                    params![student_id, topic_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(|e| EcoachError::Storage(e.to_string()))?;

            if let Some((mastery_score, last_seen_at, is_blocked)) = state {
                if mastery_score >= 7200 {
                    return Ok(10_000);
                }
                if is_blocked == 1 {
                    return Ok(1_500);
                }
                if was_recent(last_seen_at.as_deref(), 7) {
                    return Ok(6_500);
                }
                if mastery_score >= 5500 {
                    return Ok(6_000);
                }
                if mastery_score >= 4000 {
                    return Ok(4_000);
                }
                return Ok(2_000);
            }
        }

        let recent_session_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM sessions
                 WHERE student_id = ?1
                   AND started_at >= ?2
                   AND status IN ('active', 'completed')",
                params![student_id, (Utc::now() - Duration::days(7)).to_rfc3339()],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        if recent_session_count > 0 {
            return Ok(5_000);
        }
        Ok(2_500)
    }

    fn list_priority_topics(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<PremiumPriorityTopic>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT sts.topic_id, t.name, sts.mastery_score, sts.gap_score, sts.priority_score,
                        sts.trend_state, sts.is_blocked, sts.next_review_at
                 FROM student_topic_states sts
                 JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                 ORDER BY sts.priority_score DESC, sts.gap_score DESC, t.name ASC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, limit as i64], |row| {
                Ok(PremiumPriorityTopic {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    mastery_score: row.get(2)?,
                    gap_score: row.get(3)?,
                    priority_score: row.get(4)?,
                    trend_state: row.get(5)?,
                    is_blocked: row.get::<_, i64>(6)? == 1,
                    next_review_at: row.get(7)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut topics = Vec::new();
        for row in rows {
            topics.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(topics)
    }

    fn list_top_risk_titles(&self, student_id: i64, limit: usize) -> EcoachResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT title
                 FROM risk_flags
                 WHERE student_id = ?1 AND status IN ('active', 'monitoring')
                 ORDER BY CASE severity
                    WHEN 'critical' THEN 0
                    WHEN 'high' THEN 1
                    WHEN 'medium' THEN 2
                    ELSE 3 END,
                    created_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, limit as i64], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut titles = Vec::new();
        for row in rows {
            titles.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(titles)
    }

    fn inactive_days(&self, student_id: i64) -> EcoachResult<Option<i64>> {
        let last_activity: Option<String> = self
            .conn
            .query_row(
                "SELECT MAX(activity_at)
                 FROM (
                    SELECT MAX(started_at) AS activity_at
                    FROM sessions
                    WHERE student_id = ?1 AND status IN ('active', 'completed')
                    UNION ALL
                    SELECT MAX(last_seen_at) AS activity_at
                    FROM student_topic_states
                    WHERE student_id = ?1
                 )",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(last_activity
            .as_deref()
            .and_then(parse_timestamp)
            .map(|timestamp| (Utc::now() - timestamp).num_days()))
    }

    fn recent_focus_signals(&self, student_id: i64, limit: usize) -> EcoachResult<Vec<String>> {
        if !self.table_exists("runtime_events")? {
            return Ok(Vec::new());
        }
        let mut statement = self
            .conn
            .prepare(
                "SELECT event_type, payload_json
                 FROM runtime_events
                 WHERE aggregate_kind IN ('session', 'game')
                 ORDER BY occurred_at DESC, event_id DESC
                 LIMIT ?1",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let rows = statement
            .query_map(params![(limit as i64).max(12)], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut signals = BTreeSet::new();
        for row in rows {
            let (event_type, payload_json) =
                row.map_err(|e| EcoachError::Storage(e.to_string()))?;
            let payload = serde_json::from_str::<serde_json::Value>(&payload_json)
                .unwrap_or_else(|_| json!({}));
            if payload["student_id"].as_i64() != Some(student_id) {
                continue;
            }
            for signal in extract_focus_signals(&event_type, &payload) {
                signals.insert(signal);
            }
        }

        Ok(signals.into_iter().collect())
    }

    fn recommended_game_modes(&self, student_id: i64, limit: usize) -> EcoachResult<Vec<String>> {
        if !self.table_exists("student_contrast_states")? {
            return Ok(Vec::new());
        }
        let mut statement = self
            .conn
            .prepare(
                "SELECT confusion_score, difference_drill_bp, similarity_trap_bp,
                        know_difference_bp, which_is_which_bp, unmask_bp
                 FROM student_contrast_states
                 WHERE student_id = ?1
                 ORDER BY confusion_score DESC, updated_at DESC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(resolve_recommended_mode(
                    clamp_bp(row.get::<_, i64>(0)?),
                    clamp_bp(row.get::<_, i64>(1)?),
                    clamp_bp(row.get::<_, i64>(2)?),
                    clamp_bp(row.get::<_, i64>(3)?),
                    clamp_bp(row.get::<_, i64>(4)?),
                    clamp_bp(row.get::<_, i64>(5)?),
                ))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut modes = BTreeSet::new();
        for row in rows {
            modes.insert(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }

        Ok(modes.into_iter().collect())
    }

    fn table_exists(&self, table_name: &str) -> EcoachResult<bool> {
        let exists = self
            .conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master
                    WHERE type = 'table' AND name = ?1
                 )",
                [table_name],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(exists == 1)
    }

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

fn readiness_band(score: i64) -> &'static str {
    match score {
        8500..=10000 => "Exam Ready",
        7000..=8499 => "Strong",
        5500..=6999 => "Building",
        4000..=5499 => "At Risk",
        _ => "Not Ready",
    }
}

fn resolve_strategy_mode(
    readiness_score: i64,
    critical_risk_count: i64,
    active_risk_count: i64,
    overdue_review_count: i64,
    inactive_days: Option<i64>,
) -> &'static str {
    if critical_risk_count > 0 || readiness_score < 4000 {
        return "rescue";
    }
    if inactive_days.unwrap_or(0) >= INACTIVITY_DANGER_DAYS || overdue_review_count >= 3 {
        return "stabilize";
    }
    if active_risk_count >= 3 || readiness_score < 7000 {
        return "repair";
    }
    "accelerate"
}

fn build_coach_actions(
    strategy_mode: &str,
    priority_topics: &[PremiumPriorityTopic],
    overdue_review_count: i64,
    active_intervention_count: i64,
    inactive_days: Option<i64>,
    current_phase: Option<&str>,
    recent_focus_signals: &[String],
    recommended_game_modes: &[String],
) -> Vec<String> {
    let mut actions = Vec::new();

    if matches!(strategy_mode, "rescue" | "repair") {
        if let Some(topic) = priority_topics.first() {
            actions.push(format!(
                "Open a focused repair block for {} before introducing new scope.",
                topic.topic_name
            ));
        }
    }
    if overdue_review_count > 0 {
        actions.push(format!(
            "Clear {} overdue review obligation(s) before the next expansion mission.",
            overdue_review_count
        ));
    }
    if active_intervention_count > 0 {
        actions.push("Keep intervention cases in active review until risks downgrade.".to_string());
    }
    if inactive_days.unwrap_or(0) >= INACTIVITY_DANGER_DAYS {
        actions.push("Re-establish learner rhythm with a same-day study session.".to_string());
    }
    if let Some(phase) = current_phase {
        actions.push(format!(
            "Current plan phase is {}, so keep sequencing aligned to it.",
            phase
        ));
    }
    if let Some(signal) = recent_focus_signals.first() {
        actions.push(format!(
            "Recent learner evidence is signaling {}, so route the next block through that weakness.",
            signal.replace('_', " ")
        ));
    }
    if let Some(mode) = recommended_game_modes.first() {
        actions.push(format!(
            "Use {} as the next game-mode remediation surface if contrast work is available.",
            mode
        ));
    }
    if actions.is_empty() {
        actions.push(
            "Maintain the current plan and continue reinforcing priority topics.".to_string(),
        );
    }

    actions.truncate(4);
    actions
}

fn build_household_actions(
    strategy_mode: &str,
    overdue_review_count: i64,
    critical_risk_count: i64,
    active_intervention_count: i64,
    inactive_days: Option<i64>,
    daily_budget_minutes: Option<i64>,
    recent_focus_signals: &[String],
    recommended_game_modes: &[String],
) -> Vec<String> {
    let mut actions = Vec::new();

    if critical_risk_count > 0 || strategy_mode == "rescue" {
        actions.push("Guardian check-in is needed today to protect study time.".to_string());
    }
    if inactive_days.unwrap_or(0) >= INACTIVITY_DANGER_DAYS {
        actions.push(
            "Help the learner restart with a short session in the next 24 hours.".to_string(),
        );
    }
    if overdue_review_count > 0 {
        actions.push("Support a review-first session instead of starting a new topic.".to_string());
    }
    if active_intervention_count > 0 {
        actions.push(
            "Review active intervention steps together at the next household check-in.".to_string(),
        );
    }
    if let Some(minutes) = daily_budget_minutes {
        actions.push(format!(
            "Protect the next {}-minute study window from interruptions.",
            minutes
        ));
    }
    if let Some(signal) = recent_focus_signals.first() {
        actions.push(format!(
            "The latest learner evidence shows {}, so avoid broadening scope until that settles.",
            signal.replace('_', " ")
        ));
    }
    if let Some(mode) = recommended_game_modes.first() {
        actions.push(format!(
            "If the learner uses games, prefer the {} mode next because it matches the current weakness.",
            mode
        ));
    }
    if actions.is_empty() {
        actions
            .push("Keep the current routine steady and encourage daily consistency.".to_string());
    }

    actions.truncate(4);
    actions
}

fn extract_focus_signals(event_type: &str, payload: &serde_json::Value) -> Vec<String> {
    let mut signals = Vec::new();

    if let Some(payload_signals) = payload["focus_signals"].as_array() {
        for signal in payload_signals {
            if let Some(signal) = signal.as_str() {
                signals.push(signal.to_string());
            }
        }
    }

    match event_type {
        "session.interpreted" => {
            if let Some(tags) = payload["interpretation_tags"].as_array() {
                for tag in tags {
                    if let Some(tag) = tag.as_str() {
                        signals.push(tag.to_string());
                    }
                }
            }
        }
        "traps.session_completed" => {
            if let Some(mode) = payload["recommended_next_mode"].as_str() {
                signals.push(format!("trap_mode_{mode}"));
            }
            if let Some(reason) = payload["dominant_confusion_reason"].as_str() {
                signals.push(format!("confusion_{}", reason));
            }
        }
        "game.session_completed" => {
            if let Some(step) = payload["recommended_next_step"].as_str() {
                signals.push(step.replace(' ', "_").to_lowercase());
            }
        }
        _ => {}
    }

    signals.sort();
    signals.dedup();
    signals
}

fn resolve_recommended_mode(
    confusion_score: BasisPoints,
    difference_drill_bp: BasisPoints,
    similarity_trap_bp: BasisPoints,
    know_difference_bp: BasisPoints,
    which_is_which_bp: BasisPoints,
    unmask_bp: BasisPoints,
) -> String {
    if difference_drill_bp == 0 || confusion_score >= 7000 {
        "difference_drill".to_string()
    } else if similarity_trap_bp < 6500 {
        "similarity_trap".to_string()
    } else if know_difference_bp < 6500 {
        "know_the_difference".to_string()
    } else if which_is_which_bp < 7000 {
        "which_is_which".to_string()
    } else if unmask_bp < 7000 {
        "unmask".to_string()
    } else {
        "which_is_which".to_string()
    }
}

fn strategy_available_inputs(
    active_risk_count: i64,
    overdue_review_count: i64,
    has_priority_topics: bool,
    has_recent_focus_signals: bool,
) -> Vec<String> {
    let mut inputs = vec![
        "student_truth".to_string(),
        "learner_evidence_fabric".to_string(),
    ];
    if active_risk_count > 0 {
        inputs.push("coach_state".to_string());
    }
    if overdue_review_count > 0 || has_priority_topics {
        inputs.push("readiness_signals".to_string());
    }
    if has_recent_focus_signals {
        inputs.push("session_outcomes".to_string());
    }
    inputs
}

fn was_recent(raw: Option<&str>, days: i64) -> bool {
    raw.and_then(parse_timestamp)
        .map(|timestamp| (Utc::now() - timestamp).num_days() <= days)
        .unwrap_or(false)
}

fn parse_timestamp(raw: &str) -> Option<DateTime<Utc>> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(raw) {
        return Some(timestamp.with_timezone(&Utc));
    }

    NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|timestamp| DateTime::<Utc>::from_naive_utc_and_offset(timestamp, Utc))
        .or_else(|| {
            DateTime::<Utc>::from_str(raw)
                .ok()
                .map(|timestamp| timestamp.with_timezone(&Utc))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn strategy_snapshot_surfaces_rescue_state_and_priority_topics() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_test_schema(&conn);
        seed_premium_features(&conn);

        conn.execute(
            "INSERT INTO accounts (id, display_name, entitlement_tier) VALUES (1, 'Ama', 'premium')",
            [],
        )
        .expect("student account");
        conn.execute(
            "INSERT INTO student_profiles (account_id, exam_target, exam_target_date, daily_study_budget_minutes)
             VALUES (1, 'BECE', '2026-06-01', 75)",
            [],
        )
        .expect("student profile");
        conn.execute(
            "INSERT INTO topics (id, name) VALUES (10, 'Algebra'), (11, 'Geometry')",
            [],
        )
        .expect("topics");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, priority_score, trend_state, is_blocked, next_review_at, last_seen_at
             ) VALUES
                (1, 10, 1800, 8200, 9500, 'critical', 1, '2026-03-28T09:00:00Z', '2026-03-20T09:00:00Z'),
                (1, 11, 6200, 3800, 5000, 'fragile', 0, '2026-03-30T09:00:00Z', '2026-03-28T09:00:00Z')",
            [],
        )
        .expect("topic states");
        conn.execute(
            "INSERT INTO risk_flags (student_id, topic_id, severity, title, status, created_at)
             VALUES
                (1, 10, 'critical', 'Algebra collapse', 'active', '2026-03-28T09:00:00Z'),
                (1, 11, 'high', 'Geometry fragile', 'monitoring', '2026-03-27T09:00:00Z')",
            [],
        )
        .expect("risk flags");
        conn.execute(
            "INSERT INTO intervention_records (id, student_id, risk_flag_id, title, status, summary_json, created_at, updated_at)
             VALUES (41, 1, 1, 'Algebra rescue', 'active', ?1, '2026-03-28T09:00:00Z', '2026-03-29T09:00:00Z')",
            [serde_json::to_string(&vec![InterventionStep {
                action: "Rebuild algebra basics".to_string(),
                target_topic_id: Some(10),
                target_minutes: Some(45),
            }])
            .expect("steps json")],
        )
        .expect("intervention");
        conn.execute(
            "INSERT INTO memory_states (student_id, review_due_at, decay_risk)
             VALUES (1, '2026-03-28T08:00:00Z', 7000)",
            [],
        )
        .expect("memory risk");
        conn.execute(
            "INSERT INTO sessions (student_id, started_at, status)
             VALUES (1, '2026-03-24T09:00:00Z', 'completed')",
            [],
        )
        .expect("session");
        conn.execute(
            "INSERT INTO coach_plans (student_id, current_phase, daily_budget_minutes, status, updated_at)
             VALUES (1, 'foundation', 90, 'active', '2026-03-29T09:00:00Z')",
            [],
        )
        .expect("coach plan");
        conn.execute(
            "INSERT INTO student_contrast_states (
                student_id, pair_id, confusion_score, difference_drill_bp, similarity_trap_bp,
                know_difference_bp, which_is_which_bp, unmask_bp, updated_at
             ) VALUES (1, 21, 7800, 4200, 5100, 6200, 5800, 6400, '2026-03-29T09:30:00Z')",
            [],
        )
        .expect("contrast state");
        conn.execute(
            "INSERT INTO runtime_events (
                event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
             ) VALUES
                ('evt-1', 'traps.session_completed', 'game', '77', 'trace-1', ?1, '2026-03-29T10:00:00Z'),
                ('evt-2', 'session.interpreted', 'session', '88', 'trace-2', ?2, '2026-03-29T11:00:00Z')",
            params![
                json!({
                    "student_id": 1,
                    "recommended_next_mode": "difference_drill",
                    "dominant_confusion_reason": "feature_confusion",
                    "remediation_actions": ["Slow down contrast work"],
                })
                .to_string(),
                json!({
                    "student_id": 1,
                    "interpretation_tags": ["pressure_breakdown", "review_requested"],
                })
                .to_string(),
            ],
        )
        .expect("runtime events");

        let service = PremiumService::new(&conn);
        let snapshot = service.get_strategy_snapshot(1).expect("strategy snapshot");

        assert_eq!(snapshot.strategy_mode, "rescue");
        assert_eq!(snapshot.critical_risk_count, 1);
        assert_eq!(snapshot.priority_topics[0].topic_name, "Algebra");
        assert!(snapshot
            .recent_focus_signals
            .iter()
            .any(|signal| signal == "pressure_breakdown"));
        assert!(snapshot
            .recommended_game_modes
            .iter()
            .any(|mode| mode == "difference_drill"));
        assert!(!snapshot.coach_actions.is_empty());
        assert!(!snapshot.household_actions.is_empty());
        assert!(snapshot
            .orchestration
            .consumer_targets
            .iter()
            .any(|target| target.engine_key == "reporting"));
    }

    #[test]
    fn intervention_progress_tracks_topic_recovery() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_test_schema(&conn);
        seed_premium_features(&conn);

        conn.execute(
            "INSERT INTO accounts (id, display_name, entitlement_tier) VALUES (1, 'Kojo', 'premium')",
            [],
        )
        .expect("student account");
        conn.execute("INSERT INTO topics (id, name) VALUES (7, 'Fractions')", [])
            .expect("topic");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, priority_score, trend_state, is_blocked, next_review_at, last_seen_at
             ) VALUES (1, 7, 7600, 2400, 3000, 'improving', 0, NULL, '2026-03-29T09:00:00Z')",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO intervention_records (id, student_id, risk_flag_id, title, status, summary_json, created_at, updated_at)
             VALUES (5, 1, NULL, 'Fractions recovery', 'active', ?1, '2026-03-28T09:00:00Z', '2026-03-29T09:00:00Z')",
            [serde_json::to_string(&vec![InterventionStep {
                action: "Practice fractions".to_string(),
                target_topic_id: Some(7),
                target_minutes: Some(30),
            }])
            .expect("steps json")],
        )
        .expect("intervention");

        let service = PremiumService::new(&conn);
        let intervention = service.get_intervention(5).expect("intervention");

        assert!(intervention.progress_percent >= 9000);
    }

    fn seed_premium_features(conn: &Connection) {
        conn.execute(
            "INSERT INTO premium_features (feature_key, display_name, tier_required)
             VALUES ('risk_dashboard', 'Risk Dashboard', 'premium')",
            [],
        )
        .expect("premium feature");
    }

    fn create_test_schema(conn: &Connection) {
        conn.execute_batch(
            "
            CREATE TABLE accounts (
                id INTEGER PRIMARY KEY,
                display_name TEXT NOT NULL,
                entitlement_tier TEXT NOT NULL
            );
            CREATE TABLE student_profiles (
                account_id INTEGER PRIMARY KEY,
                exam_target TEXT,
                exam_target_date TEXT,
                daily_study_budget_minutes INTEGER
            );
            CREATE TABLE premium_features (
                feature_key TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                tier_required TEXT NOT NULL
            );
            CREATE TABLE premium_feature_flags (
                feature_key TEXT NOT NULL,
                student_id INTEGER,
                enabled INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE risk_flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                student_id INTEGER NOT NULL,
                topic_id INTEGER,
                severity TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE intervention_records (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                risk_flag_id INTEGER,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                summary_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE student_topic_states (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                mastery_score INTEGER NOT NULL,
                gap_score INTEGER NOT NULL,
                priority_score INTEGER NOT NULL,
                trend_state TEXT NOT NULL,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                next_review_at TEXT,
                last_seen_at TEXT
            );
            CREATE TABLE memory_states (
                student_id INTEGER NOT NULL,
                review_due_at TEXT,
                decay_risk INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE sessions (
                student_id INTEGER NOT NULL,
                started_at TEXT,
                status TEXT NOT NULL
            );
            CREATE TABLE coach_plans (
                student_id INTEGER NOT NULL,
                current_phase TEXT,
                daily_budget_minutes INTEGER,
                status TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                id INTEGER PRIMARY KEY AUTOINCREMENT
            );
            CREATE TABLE runtime_events (
                event_id TEXT,
                event_type TEXT,
                aggregate_kind TEXT,
                aggregate_id TEXT,
                trace_id TEXT,
                payload_json TEXT,
                occurred_at TEXT
            );
            CREATE TABLE student_contrast_states (
                student_id INTEGER NOT NULL,
                pair_id INTEGER NOT NULL,
                confusion_score INTEGER NOT NULL DEFAULT 0,
                difference_drill_bp INTEGER NOT NULL DEFAULT 0,
                similarity_trap_bp INTEGER NOT NULL DEFAULT 0,
                know_difference_bp INTEGER NOT NULL DEFAULT 0,
                which_is_which_bp INTEGER NOT NULL DEFAULT 0,
                unmask_bp INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT
            );
            ",
        )
        .expect("test schema");
    }
}
