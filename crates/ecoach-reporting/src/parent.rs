use chrono::{Datelike, Duration, Utc};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::dashboard::{DashboardService, StudentDashboard, SubjectSummary};
use crate::strategy::{load_strategy_summary, ReportingStrategySummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentDashboardSnapshot {
    pub parent_id: i64,
    pub parent_name: String,
    pub students: Vec<ParentStudentSummary>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentStudentSummary {
    pub student_id: i64,
    pub student_name: String,
    pub overall_readiness_band: String,
    pub exam_target: Option<String>,
    pub active_risks: Vec<ParentRiskSummary>,
    pub recommendations: Vec<String>,
    pub trend_summary: Vec<String>,
    pub weekly_memo: String,
    pub subject_summaries: Vec<SubjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentRiskSummary {
    pub severity: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseholdDashboardSnapshot {
    pub parent_id: i64,
    pub parent_name: String,
    pub household_attention_level: String,
    pub students_needing_attention: usize,
    pub active_interventions: usize,
    pub household_actions: Vec<HouseholdActionItem>,
    pub students: Vec<HouseholdStudentSnapshot>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseholdStudentSnapshot {
    pub student_id: i64,
    pub student_name: String,
    pub overall_readiness_band: String,
    pub attention_level: String,
    pub exam_target: Option<String>,
    pub exam_target_date: Option<String>,
    pub strategy_summary: Option<ReportingStrategySummary>,
    pub active_risks: Vec<ParentRiskSummary>,
    pub active_interventions: Vec<HouseholdInterventionSummary>,
    pub household_actions: Vec<HouseholdActionItem>,
    pub weekly_memo: String,
    pub subject_summaries: Vec<SubjectSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseholdInterventionSummary {
    pub intervention_id: i64,
    pub title: String,
    pub status: String,
    pub linked_risk_title: Option<String>,
    pub risk_severity: Option<String>,
    pub progress_percent: BasisPoints,
    pub next_step: Option<String>,
    pub target_topic_name: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseholdActionItem {
    pub urgency: String,
    pub title: String,
    pub detail: String,
}

pub struct ParentInsightService<'a> {
    conn: &'a Connection,
}

impl<'a> ParentInsightService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn build_parent_dashboard(&self, parent_id: i64) -> EcoachResult<ParentDashboardSnapshot> {
        let parent_name: String = self
            .conn
            .query_row(
                "SELECT display_name FROM accounts WHERE id = ?1",
                [parent_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let students = self.linked_students(parent_id)?;
        let dashboard_service = DashboardService::new(self.conn);
        let generated_at = Utc::now().to_rfc3339();

        let mut summaries = Vec::new();
        for (student_id, student_name) in students {
            let student_dashboard = dashboard_service.get_student_dashboard(student_id)?;
            let active_risks = self.sync_derived_risks(student_id, &student_dashboard)?;
            let recommendations = self.generate_recommendations(student_id)?;
            let trend_summary = self.generate_trend_summary(student_id)?;
            let weekly_memo = self.upsert_weekly_memo(
                student_id,
                &student_dashboard,
                &active_risks,
                &recommendations,
            )?;
            self.upsert_parent_dashboard(
                parent_id,
                student_id,
                &student_dashboard.overall_readiness_band,
                &active_risks,
                &trend_summary,
            )?;

            summaries.push(ParentStudentSummary {
                student_id,
                student_name,
                overall_readiness_band: student_dashboard.overall_readiness_band.clone(),
                exam_target: student_dashboard.exam_target.clone(),
                active_risks,
                recommendations,
                trend_summary,
                weekly_memo,
                subject_summaries: student_dashboard.subject_summaries,
            });
        }

        Ok(ParentDashboardSnapshot {
            parent_id,
            parent_name,
            students: summaries,
            generated_at,
        })
    }

    pub fn build_household_dashboard(
        &self,
        parent_id: i64,
    ) -> EcoachResult<HouseholdDashboardSnapshot> {
        let parent_name: String = self
            .conn
            .query_row(
                "SELECT display_name FROM accounts WHERE id = ?1",
                [parent_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let students = self.linked_students(parent_id)?;
        let dashboard_service = DashboardService::new(self.conn);
        let generated_at = Utc::now().to_rfc3339();

        let mut summaries = Vec::new();
        let mut aggregate_actions: Vec<HouseholdActionItem> = Vec::new();
        for (student_id, student_name) in students {
            let student_dashboard = dashboard_service.get_student_dashboard(student_id)?;
            let active_risks = self.sync_derived_risks(student_id, &student_dashboard)?;
            let recommendations = self.generate_recommendations(student_id)?;
            let trend_summary = self.generate_trend_summary(student_id)?;
            let weekly_memo = self.upsert_weekly_memo(
                student_id,
                &student_dashboard,
                &active_risks,
                &recommendations,
            )?;
            self.upsert_parent_dashboard(
                parent_id,
                student_id,
                &student_dashboard.overall_readiness_band,
                &active_risks,
                &trend_summary,
            )?;

            let exam_target_date = self.student_exam_target_date(student_id)?;
            let strategy_summary = load_strategy_summary(self.conn, student_id)?;
            let active_interventions = self.list_household_interventions(student_id)?;
            let attention_level = derive_attention_level(
                &student_dashboard.overall_readiness_band,
                &active_risks,
                &active_interventions,
            );
            let household_actions = self.build_household_actions(
                student_id,
                &student_dashboard,
                strategy_summary.as_ref(),
                &active_risks,
                &active_interventions,
            )?;
            aggregate_actions.extend(
                household_actions
                    .iter()
                    .filter(|item| item.urgency != "low")
                    .cloned(),
            );

            summaries.push(HouseholdStudentSnapshot {
                student_id,
                student_name,
                overall_readiness_band: student_dashboard.overall_readiness_band.clone(),
                attention_level,
                exam_target: student_dashboard.exam_target.clone(),
                exam_target_date,
                strategy_summary,
                active_risks,
                active_interventions,
                household_actions,
                weekly_memo,
                subject_summaries: student_dashboard.subject_summaries,
            });
        }

        let students_needing_attention = summaries
            .iter()
            .filter(|student| student.attention_level != "low")
            .count();
        let active_interventions = summaries
            .iter()
            .map(|student| student.active_interventions.len())
            .sum();
        let household_attention_level = if summaries
            .iter()
            .any(|student| student.attention_level == "high")
        {
            "high".to_string()
        } else if students_needing_attention > 0 {
            "medium".to_string()
        } else {
            "low".to_string()
        };
        let mut household_actions = dedupe_household_actions(aggregate_actions);
        if household_actions.is_empty() {
            household_actions.push(HouseholdActionItem {
                urgency: "low".to_string(),
                title: "Maintain the household study routine".to_string(),
                detail: "No household-level escalation is needed right now beyond protecting normal study time.".to_string(),
            });
        }

        Ok(HouseholdDashboardSnapshot {
            parent_id,
            parent_name,
            household_attention_level,
            students_needing_attention,
            active_interventions,
            household_actions,
            students: summaries,
            generated_at,
        })
    }

    fn linked_students(&self, parent_id: i64) -> EcoachResult<Vec<(i64, String)>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT a.id, a.display_name
                 FROM parent_student_links p
                 JOIN accounts a ON a.id = p.student_account_id
                 WHERE p.parent_account_id = ?1
                 ORDER BY a.display_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([parent_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut students = Vec::new();
        for row in rows {
            students.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(students)
    }

    fn sync_derived_risks(
        &self,
        student_id: i64,
        dashboard: &StudentDashboard,
    ) -> EcoachResult<Vec<ParentRiskSummary>> {
        let mut risks = Vec::new();

        if matches!(
            dashboard.overall_readiness_band.as_str(),
            "At Risk" | "Not Ready"
        ) {
            risks.push(ParentRiskSummary {
                severity: "high".to_string(),
                title: "Low readiness".to_string(),
                description: "Current performance trends suggest the learner is not yet on track for the exam.".to_string(),
            });
        }

        let weak_topic_count: usize = dashboard
            .subject_summaries
            .iter()
            .map(|item| item.weak_topic_count)
            .sum();
        if weak_topic_count >= 3 {
            risks.push(ParentRiskSummary {
                severity: "medium".to_string(),
                title: "Weak topic cluster".to_string(),
                description: format!(
                    "{} topics are currently below the safe mastery threshold and need focused repair.",
                    weak_topic_count
                ),
            });
        }

        if self.has_inactivity_risk(student_id)? {
            risks.push(ParentRiskSummary {
                severity: "medium".to_string(),
                title: "Low study activity".to_string(),
                description: "Recent learner activity has dropped, which can slow readiness gains."
                    .to_string(),
            });
        }

        if self.has_memory_risk(student_id)? {
            risks.push(ParentRiskSummary {
                severity: "medium".to_string(),
                title: "Reviews overdue".to_string(),
                description:
                    "Important memory checks are now overdue and should be revisited soon."
                        .to_string(),
            });
        }

        for risk in &risks {
            self.upsert_risk_flag(student_id, risk)?;
        }

        let mut persisted = self.persisted_active_risks(student_id)?;
        if persisted.is_empty() {
            persisted = risks;
        }
        Ok(persisted)
    }

    fn generate_recommendations(&self, student_id: i64) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT t.name
                 FROM student_topic_states sts
                 JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                 ORDER BY sts.priority_score DESC, sts.gap_score DESC
                 LIMIT 3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([student_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut recommendations = Vec::new();
        for row in rows {
            let topic_name = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            recommendations.push(format!("Prioritize focused repair on {}.", topic_name));
        }

        if recommendations.is_empty() {
            recommendations.push(
                "Maintain the current study rhythm and keep reviewing completed work.".to_string(),
            );
        }

        Ok(recommendations)
    }

    fn generate_trend_summary(&self, student_id: i64) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT t.name, sts.trend_state
                 FROM student_topic_states sts
                 JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                 ORDER BY CASE sts.trend_state
                    WHEN 'critical' THEN 0
                    WHEN 'declining' THEN 1
                    WHEN 'fragile' THEN 2
                    WHEN 'stable' THEN 3
                    ELSE 4 END,
                    sts.priority_score DESC
                 LIMIT 3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([student_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut trends = Vec::new();
        for row in rows {
            let (topic_name, trend_state) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let summary = match trend_state.as_str() {
                "critical" => format!(
                    "{} is currently critical and needs immediate attention.",
                    topic_name
                ),
                "declining" => format!(
                    "{} is trending downward and should be reviewed this week.",
                    topic_name
                ),
                "fragile" => format!("{} is fragile and needs reinforcement.", topic_name),
                "improving" => format!("{} is improving with recent effort.", topic_name),
                _ => format!("{} is stable for now.", topic_name),
            };
            trends.push(summary);
        }
        Ok(trends)
    }

    fn upsert_weekly_memo(
        &self,
        student_id: i64,
        dashboard: &StudentDashboard,
        risks: &[ParentRiskSummary],
        recommendations: &[String],
    ) -> EcoachResult<String> {
        let week_start = current_week_start();
        let memo_body = build_parent_memo(dashboard, risks, recommendations);
        let metadata_json = serde_json::json!({
            "overall_readiness_band": dashboard.overall_readiness_band,
            "risk_count": risks.len(),
            "recommendation_count": recommendations.len(),
        })
        .to_string();

        let existing_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id
                 FROM weekly_memos
                 WHERE student_id = ?1 AND audience = 'parent' AND week_start = ?2
                 LIMIT 1",
                params![student_id, week_start],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some(id) = existing_id {
            self.conn
                .execute(
                    "UPDATE weekly_memos
                     SET memo_body = ?1, metadata_json = ?2
                     WHERE id = ?3",
                    params![memo_body, metadata_json, id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO weekly_memos (student_id, audience, week_start, memo_body, metadata_json)
                     VALUES (?1, 'parent', ?2, ?3, ?4)",
                    params![student_id, week_start, memo_body, metadata_json],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(memo_body)
    }

    fn upsert_parent_dashboard(
        &self,
        parent_id: i64,
        student_id: i64,
        readiness: &str,
        risks: &[ParentRiskSummary],
        trends: &[String],
    ) -> EcoachResult<()> {
        let risk_summary_json = serde_json::to_string(risks)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let trend_summary_json = serde_json::to_string(trends)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO parent_dashboards (
                    parent_account_id, student_account_id, readiness_band, risk_summary_json, trend_summary_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(parent_account_id, student_account_id) DO UPDATE SET
                    readiness_band = excluded.readiness_band,
                    risk_summary_json = excluded.risk_summary_json,
                    trend_summary_json = excluded.trend_summary_json,
                    updated_at = datetime('now')",
                params![parent_id, student_id, readiness, risk_summary_json, trend_summary_json],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn upsert_risk_flag(&self, student_id: i64, risk: &ParentRiskSummary) -> EcoachResult<()> {
        let existing_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id
                 FROM risk_flags
                 WHERE student_id = ?1 AND title = ?2 AND status IN ('active', 'monitoring')
                 LIMIT 1",
                params![student_id, risk.title],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some(id) = existing_id {
            self.conn
                .execute(
                    "UPDATE risk_flags
                     SET severity = ?1, description = ?2, status = 'active'
                     WHERE id = ?3",
                    params![risk.severity, risk.description, id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        } else {
            self.conn
                .execute(
                    "INSERT INTO risk_flags (student_id, severity, title, description, status, created_at)
                     VALUES (?1, ?2, ?3, ?4, 'active', ?5)",
                    params![
                        student_id,
                        risk.severity,
                        risk.title,
                        risk.description,
                        Utc::now().to_rfc3339(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn persisted_active_risks(&self, student_id: i64) -> EcoachResult<Vec<ParentRiskSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT severity, title, COALESCE(description, '')
                 FROM risk_flags
                 WHERE student_id = ?1 AND status IN ('active', 'monitoring')
                 ORDER BY CASE severity
                    WHEN 'high' THEN 0
                    WHEN 'medium' THEN 1
                    ELSE 2 END,
                    created_at DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([student_id], |row| {
                Ok(ParentRiskSummary {
                    severity: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut risks = Vec::new();
        for row in rows {
            risks.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(risks)
    }

    fn student_exam_target_date(&self, student_id: i64) -> EcoachResult<Option<String>> {
        self.conn
            .query_row(
                "SELECT exam_target_date
                 FROM student_profiles
                 WHERE account_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
            .map(|value| value.flatten())
    }

    fn list_household_interventions(
        &self,
        student_id: i64,
    ) -> EcoachResult<Vec<HouseholdInterventionSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT ir.id, ir.title, ir.status, ir.summary_json, ir.updated_at,
                        rf.title, rf.severity
                 FROM intervention_records ir
                 LEFT JOIN risk_flags rf ON rf.id = ir.risk_flag_id
                 WHERE ir.student_id = ?1 AND ir.status IN ('active', 'review', 'escalated')
                 ORDER BY CASE rf.severity
                    WHEN 'critical' THEN 0
                    WHEN 'high' THEN 1
                    WHEN 'medium' THEN 2
                    ELSE 3 END,
                    ir.updated_at DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([student_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut interventions = Vec::new();
        for row in rows {
            let (
                intervention_id,
                title,
                status,
                summary_json,
                updated_at,
                linked_risk_title,
                risk_severity,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let steps: Vec<InterventionStepRecord> =
                serde_json::from_str(&summary_json).unwrap_or_default();
            let progress_percent = self.compute_intervention_progress(student_id, &steps)?;
            let next_step = steps
                .iter()
                .find(|step| self.step_score(student_id, step).unwrap_or(0) < 9000)
                .map(|step| {
                    format_intervention_step(
                        step,
                        self.topic_name(step.target_topic_id)
                            .ok()
                            .flatten()
                            .as_deref(),
                    )
                });
            let target_topic_name = steps
                .iter()
                .find_map(|step| self.topic_name(step.target_topic_id).ok().flatten());

            interventions.push(HouseholdInterventionSummary {
                intervention_id,
                title,
                status,
                linked_risk_title,
                risk_severity,
                progress_percent,
                next_step,
                target_topic_name,
                updated_at,
            });
        }

        Ok(interventions)
    }

    fn build_household_actions(
        &self,
        student_id: i64,
        dashboard: &StudentDashboard,
        strategy_summary: Option<&ReportingStrategySummary>,
        risks: &[ParentRiskSummary],
        interventions: &[HouseholdInterventionSummary],
    ) -> EcoachResult<Vec<HouseholdActionItem>> {
        let mut actions = Vec::new();

        if matches!(
            dashboard.overall_readiness_band.as_str(),
            "At Risk" | "Not Ready"
        ) {
            actions.push(HouseholdActionItem {
                urgency: "high".to_string(),
                title: "Protect this week's repair time".to_string(),
                detail:
                    "The learner needs a protected repair block before more new content is added."
                        .to_string(),
            });
        }

        if let Some(risk) = risks
            .iter()
            .find(|risk| matches!(risk.severity.as_str(), "critical" | "high"))
        {
            actions.push(HouseholdActionItem {
                urgency: risk.severity.clone(),
                title: format!("Respond to {}", risk.title),
                detail: risk.description.clone(),
            });
        }

        if let Some(intervention) = interventions.first() {
            let detail = intervention.next_step.clone().unwrap_or_else(|| {
                "Review the current intervention plan and confirm the next step.".to_string()
            });
            actions.push(HouseholdActionItem {
                urgency: "medium".to_string(),
                title: format!("Check intervention: {}", intervention.title),
                detail,
            });
        }

        if let Some(strategy_summary) = strategy_summary {
            if let Some(topic_name) = strategy_summary.priority_topics.first() {
                actions.push(HouseholdActionItem {
                    urgency: if strategy_summary.strategy_mode == "rescue" {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                    title: format!("Support the {} strategy", strategy_summary.strategy_mode),
                    detail: format!(
                        "The current premium strategy is centered on {}, so protect time for {} first.",
                        strategy_summary.strategy_mode, topic_name
                    ),
                });
            }
            if let Some(action) = strategy_summary.household_actions.first() {
                actions.push(HouseholdActionItem {
                    urgency: "medium".to_string(),
                    title: "Follow the premium household guidance".to_string(),
                    detail: action.clone(),
                });
            }
            if let Some(signal) = strategy_summary.recent_focus_signals.first() {
                actions.push(HouseholdActionItem {
                    urgency: "medium".to_string(),
                    title: "Respond to the latest learner signal".to_string(),
                    detail: format!(
                        "Recent evidence is showing {}, so keep the next study block narrow and supportive.",
                        signal.replace('_', " ")
                    ),
                });
            }
            if let Some(mode) = strategy_summary.recommended_game_modes.first() {
                actions.push(HouseholdActionItem {
                    urgency: "low".to_string(),
                    title: "Use the next remediation mode intentionally".to_string(),
                    detail: format!(
                        "If the learner opens games, {} is the recommended next mode for the current weakness.",
                        mode
                    ),
                });
            }
        }

        if self.has_memory_risk(student_id)? {
            actions.push(HouseholdActionItem {
                urgency: "medium".to_string(),
                title: "Prioritize overdue review work".to_string(),
                detail:
                    "Memory checks are overdue, so the next session should revisit older material."
                        .to_string(),
            });
        }

        if actions.is_empty() {
            actions.push(HouseholdActionItem {
                urgency: "low".to_string(),
                title: "Maintain the routine".to_string(),
                detail: "No acute household action is needed beyond protecting normal study time."
                    .to_string(),
            });
        }

        Ok(dedupe_household_actions(actions))
    }

    fn compute_intervention_progress(
        &self,
        student_id: i64,
        steps: &[InterventionStepRecord],
    ) -> EcoachResult<BasisPoints> {
        if steps.is_empty() {
            return Ok(0);
        }

        let mut total = 0;
        for step in steps {
            total += self.step_score(student_id, step)?;
        }
        Ok((total / steps.len() as i64).clamp(0, 10_000) as BasisPoints)
    }

    fn step_score(&self, student_id: i64, step: &InterventionStepRecord) -> EcoachResult<i64> {
        if let Some(topic_id) = step.target_topic_id {
            let state: Option<(BasisPoints, Option<String>, i64)> = self
                .conn
                .query_row(
                    "SELECT mastery_score, last_seen_at, is_blocked
                     FROM student_topic_states
                     WHERE student_id = ?1 AND topic_id = ?2",
                    params![student_id, topic_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

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
                return Ok(2_500);
            }
        }

        let recent_sessions: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM sessions
                 WHERE student_id = ?1 AND started_at >= ?2",
                params![student_id, (Utc::now() - Duration::days(7)).to_rfc3339()],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if recent_sessions > 0 {
            return Ok(5_000);
        }
        Ok(2_500)
    }

    fn topic_name(&self, topic_id: Option<i64>) -> EcoachResult<Option<String>> {
        let Some(topic_id) = topic_id else {
            return Ok(None);
        };

        self.conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn has_inactivity_risk(&self, student_id: i64) -> EcoachResult<bool> {
        let last_seen: Option<String> = self
            .conn
            .query_row(
                "SELECT MAX(last_seen_at) FROM student_topic_states WHERE student_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let Some(last_seen) = last_seen else {
            return Ok(true);
        };
        let parsed = chrono::DateTime::parse_from_rfc3339(&last_seen)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        Ok((Utc::now() - parsed.with_timezone(&Utc)).num_days() >= 4)
    }

    fn has_memory_risk(&self, student_id: i64) -> EcoachResult<bool> {
        let due_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM memory_states
                 WHERE student_id = ?1
                   AND review_due_at IS NOT NULL
                   AND review_due_at <= ?2
                   AND decay_risk >= 6000",
                params![student_id, Utc::now().to_rfc3339()],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(due_count > 0)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct InterventionStepRecord {
    action: String,
    target_topic_id: Option<i64>,
    target_minutes: Option<i64>,
}

fn derive_attention_level(
    readiness_band: &str,
    risks: &[ParentRiskSummary],
    interventions: &[HouseholdInterventionSummary],
) -> String {
    if matches!(readiness_band, "At Risk" | "Not Ready")
        || risks
            .iter()
            .any(|risk| matches!(risk.severity.as_str(), "critical" | "high"))
    {
        return "high".to_string();
    }
    if !interventions.is_empty() || !risks.is_empty() {
        return "medium".to_string();
    }
    "low".to_string()
}

fn format_intervention_step(step: &InterventionStepRecord, topic_name: Option<&str>) -> String {
    let mut detail = step.action.clone();
    if let Some(topic_name) = topic_name {
        detail.push_str(" on ");
        detail.push_str(topic_name);
    }
    if let Some(target_minutes) = step.target_minutes {
        detail.push_str(&format!(" for {} minutes", target_minutes));
    }
    detail
}

fn dedupe_household_actions(actions: Vec<HouseholdActionItem>) -> Vec<HouseholdActionItem> {
    let mut deduped = Vec::new();
    for action in actions {
        if deduped
            .iter()
            .any(|existing: &HouseholdActionItem| existing.title == action.title)
        {
            continue;
        }
        deduped.push(action);
        if deduped.len() == 4 {
            break;
        }
    }
    deduped
}

fn was_recent(raw: Option<&str>, days: i64) -> bool {
    raw.and_then(parse_timestamp)
        .map(|timestamp| (Utc::now() - timestamp).num_days() <= days)
        .unwrap_or(false)
}

fn parse_timestamp(raw: &str) -> Option<chrono::DateTime<Utc>> {
    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(raw) {
        return Some(timestamp.with_timezone(&Utc));
    }

    chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|timestamp| chrono::DateTime::<Utc>::from_naive_utc_and_offset(timestamp, Utc))
}

fn build_parent_memo(
    dashboard: &StudentDashboard,
    risks: &[ParentRiskSummary],
    recommendations: &[String],
) -> String {
    let strongest_subject = dashboard
        .subject_summaries
        .iter()
        .max_by_key(|item| readiness_rank(&item.readiness_band))
        .map(|item| item.subject_name.clone())
        .unwrap_or_else(|| "current study subjects".to_string());

    let lead = format!(
        "This week the learner is in the {} readiness band, with strongest momentum in {}.",
        dashboard.overall_readiness_band, strongest_subject
    );
    let risk_line = if let Some(risk) = risks.first() {
        format!(" Main concern: {}.", risk.title)
    } else {
        " No major risk flag is active right now.".to_string()
    };
    let action_line = recommendations.first().cloned().unwrap_or_else(|| {
        "Keep the current plan steady and encourage consistent daily study.".to_string()
    });

    format!("{}{} {}", lead, risk_line, action_line)
}

fn current_week_start() -> String {
    let today = Utc::now().date_naive();
    let week_start = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    week_start.to_string()
}

fn readiness_rank(label: &str) -> BasisPoints {
    match label {
        "Exam Ready" => 10_000,
        "Strong" => 8_000,
        "Building" => 6_000,
        "At Risk" => 4_000,
        _ => 2_000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn household_dashboard_includes_interventions_and_actions() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_test_schema(&conn);

        conn.execute(
            "INSERT INTO accounts (id, display_name, entitlement_tier)
             VALUES (1, 'Parent', 'standard'), (2, 'Learner', 'premium')",
            [],
        )
        .expect("accounts");
        conn.execute(
            "INSERT INTO parent_student_links (parent_account_id, student_account_id)
             VALUES (1, 2)",
            [],
        )
        .expect("link");
        conn.execute(
            "INSERT INTO student_profiles (account_id, exam_target, exam_target_date, daily_study_budget_minutes)
             VALUES (2, 'BECE', '2026-06-01', 75)",
            [],
        )
        .expect("profile");
        conn.execute(
            "INSERT INTO subjects (id, name, display_order) VALUES (10, 'Math', 0)",
            [],
        )
        .expect("subject");
        conn.execute(
            "INSERT INTO topics (id, subject_id, name) VALUES (11, 10, 'Algebra')",
            [],
        )
        .expect("topic");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, priority_score, gap_score, trend_state, last_seen_at, is_blocked, next_review_at
             ) VALUES (2, 11, 2200, 9000, 7800, 'critical', '2026-03-28T09:00:00Z', 1, '2026-03-30T09:00:00Z')",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO risk_flags (id, student_id, severity, title, description, status, created_at)
             VALUES (5, 2, 'critical', 'Algebra collapse', 'Immediate repair needed', 'active', '2026-03-28T09:00:00Z')",
            [],
        )
        .expect("risk flag");
        conn.execute(
            "INSERT INTO intervention_records (
                id, student_id, risk_flag_id, title, status, summary_json, created_at, updated_at
             ) VALUES (9, 2, 5, 'Algebra rescue', 'active', ?1, '2026-03-28T09:00:00Z', '2026-03-29T09:00:00Z')",
            [serde_json::to_string(&vec![InterventionStepRecord {
                action: "Rebuild algebra basics".to_string(),
                target_topic_id: Some(11),
                target_minutes: Some(45),
            }])
            .expect("steps")],
        )
        .expect("intervention");
        conn.execute(
            "INSERT INTO memory_states (student_id, review_due_at, decay_risk)
             VALUES (2, '2026-03-28T09:00:00Z', 7000)",
            [],
        )
        .expect("memory state");
        conn.execute(
            "INSERT INTO coach_plans (student_id, current_phase, daily_budget_minutes, status, updated_at)
             VALUES (2, 'foundation', 90, 'active', '2026-03-29T09:00:00Z')",
            [],
        )
        .expect("coach plan");

        let service = ParentInsightService::new(&conn);
        let snapshot = service
            .build_household_dashboard(1)
            .expect("household dashboard");

        assert_eq!(snapshot.students.len(), 1);
        assert_eq!(snapshot.household_attention_level, "high");
        assert_eq!(snapshot.students[0].attention_level, "high");
        assert_eq!(
            snapshot.students[0]
                .strategy_summary
                .as_ref()
                .map(|strategy| strategy.strategy_mode.as_str()),
            Some("rescue")
        );
        assert_eq!(snapshot.students[0].active_interventions.len(), 1);
        assert!(!snapshot.students[0].household_actions.is_empty());
    }

    fn create_test_schema(conn: &Connection) {
        conn.execute_batch(
            "
            CREATE TABLE accounts (
                id INTEGER PRIMARY KEY,
                display_name TEXT NOT NULL,
                entitlement_tier TEXT NOT NULL
            );
            CREATE TABLE parent_student_links (
                parent_account_id INTEGER NOT NULL,
                student_account_id INTEGER NOT NULL
            );
            CREATE TABLE student_profiles (
                account_id INTEGER PRIMARY KEY,
                exam_target TEXT,
                exam_target_date TEXT,
                daily_study_budget_minutes INTEGER
            );
            CREATE TABLE subjects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                display_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                name TEXT NOT NULL
            );
            CREATE TABLE student_topic_states (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                mastery_score INTEGER NOT NULL,
                priority_score INTEGER NOT NULL,
                gap_score INTEGER NOT NULL,
                trend_state TEXT NOT NULL,
                last_seen_at TEXT,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                next_review_at TEXT
            );
            CREATE TABLE risk_flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                student_id INTEGER NOT NULL,
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
            CREATE TABLE weekly_memos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                student_id INTEGER NOT NULL,
                audience TEXT NOT NULL,
                week_start TEXT NOT NULL,
                memo_body TEXT NOT NULL,
                metadata_json TEXT NOT NULL
            );
            CREATE TABLE parent_dashboards (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                parent_account_id INTEGER NOT NULL,
                student_account_id INTEGER NOT NULL,
                readiness_band TEXT,
                risk_summary_json TEXT NOT NULL,
                trend_summary_json TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(parent_account_id, student_account_id)
            );
            CREATE TABLE memory_states (
                student_id INTEGER NOT NULL,
                review_due_at TEXT,
                decay_risk INTEGER NOT NULL
            );
            CREATE TABLE sessions (
                student_id INTEGER NOT NULL,
                started_at TEXT,
                status TEXT NOT NULL
            );
            CREATE TABLE coach_plans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                student_id INTEGER NOT NULL,
                current_phase TEXT,
                daily_budget_minutes INTEGER,
                status TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )
        .expect("test schema");
    }
}
