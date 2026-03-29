use chrono::{Datelike, Duration, Utc};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::dashboard::{DashboardService, StudentDashboard, SubjectSummary};

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
                    "INSERT INTO risk_flags (student_id, severity, title, description, status)
                     VALUES (?1, ?2, ?3, ?4, 'active')",
                    params![student_id, risk.severity, risk.title, risk.description],
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
