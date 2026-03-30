use chrono::Utc;
use ecoach_substrate::EcoachResult;
use ecoach_substrate::{BasisPoints, EcoachError};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::dashboard::DashboardService;
use crate::strategy::{ReportingStrategySummary, load_strategy_summary};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminOversightSnapshot {
    pub admin_id: i64,
    pub admin_name: String,
    pub total_students: usize,
    pub critical_students: usize,
    pub households_needing_attention: usize,
    pub active_interventions: i64,
    pub generated_at: String,
    pub students: Vec<AdminStudentOversight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminStudentOversight {
    pub student_id: i64,
    pub student_name: String,
    pub parent_count: i64,
    pub entitlement_tier: String,
    pub overall_readiness_band: String,
    pub strategy_summary: Option<ReportingStrategySummary>,
    pub weak_topic_count: usize,
    pub active_risk_count: i64,
    pub critical_risk_count: i64,
    pub active_intervention_count: i64,
    pub overdue_review_count: i64,
    pub inactive_days: Option<i64>,
    pub attention_level: String,
    pub top_risks: Vec<String>,
    pub follow_up_actions: Vec<String>,
}

pub struct AdminOversightService<'a> {
    conn: &'a Connection,
}

impl<'a> AdminOversightService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn build_admin_oversight_snapshot(
        &self,
        admin_id: i64,
    ) -> EcoachResult<AdminOversightSnapshot> {
        let admin_name: String = self
            .conn
            .query_row(
                "SELECT display_name
                 FROM accounts
                 WHERE id = ?1 AND account_type = 'admin'",
                [admin_id],
                |row| row.get(0),
            )
            .map_err(|err| {
                EcoachError::Unauthorized(format!("admin {} not found: {}", admin_id, err))
            })?;

        let dashboard_service = DashboardService::new(self.conn);
        let students = self.list_students()?;
        let generated_at = Utc::now().to_rfc3339();

        let mut rows = Vec::new();
        let mut critical_students = 0usize;
        let mut households_needing_attention = 0usize;
        let mut active_interventions = 0i64;

        for (student_id, student_name, entitlement_tier) in students {
            let dashboard = dashboard_service.get_student_dashboard(student_id)?;
            let strategy_summary = load_strategy_summary(self.conn, student_id)?;
            let parent_count = self.parent_count(student_id)?;
            let (active_risk_count, critical_risk_count) = self.risk_counts(student_id)?;
            let intervention_count = self.active_intervention_count(student_id)?;
            let overdue_review_count = self.overdue_review_count(student_id)?;
            let inactive_days = self.inactive_days(student_id)?;
            let top_risks = self.top_risks(student_id, 3)?;

            let weak_topic_count = dashboard
                .subject_summaries
                .iter()
                .map(|subject| subject.weak_topic_count)
                .sum();
            let attention_level = derive_attention_level(
                &dashboard.overall_readiness_band,
                critical_risk_count,
                active_risk_count,
                intervention_count,
                overdue_review_count,
                inactive_days,
            );
            let follow_up_actions = build_follow_up_actions(
                &dashboard.overall_readiness_band,
                strategy_summary.as_ref(),
                parent_count,
                critical_risk_count,
                intervention_count,
                overdue_review_count,
                inactive_days,
            );

            if critical_risk_count > 0
                || matches!(
                    dashboard.overall_readiness_band.as_str(),
                    "At Risk" | "Not Ready"
                )
            {
                critical_students += 1;
            }
            if parent_count > 0 && attention_level == "high" {
                households_needing_attention += 1;
            }
            active_interventions += intervention_count;

            rows.push(AdminStudentOversight {
                student_id,
                student_name,
                parent_count,
                entitlement_tier,
                overall_readiness_band: dashboard.overall_readiness_band,
                strategy_summary,
                weak_topic_count,
                active_risk_count,
                critical_risk_count,
                active_intervention_count: intervention_count,
                overdue_review_count,
                inactive_days,
                attention_level,
                top_risks,
                follow_up_actions,
            });
        }

        rows.sort_by(|left, right| {
            attention_rank(&left.attention_level)
                .cmp(&attention_rank(&right.attention_level))
                .then_with(|| right.critical_risk_count.cmp(&left.critical_risk_count))
                .then_with(|| left.student_name.cmp(&right.student_name))
        });

        Ok(AdminOversightSnapshot {
            admin_id,
            admin_name,
            total_students: rows.len(),
            critical_students,
            households_needing_attention,
            active_interventions,
            generated_at,
            students: rows,
        })
    }

    fn list_students(&self) -> EcoachResult<Vec<(i64, String, String)>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, display_name, entitlement_tier
                 FROM accounts
                 WHERE account_type = 'student' AND COALESCE(status, 'active') != 'archived'
                 ORDER BY display_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut students = Vec::new();
        for row in rows {
            students.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(students)
    }

    fn parent_count(&self, student_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM parent_student_links
                 WHERE student_account_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn risk_counts(&self, student_id: i64) -> EcoachResult<(i64, i64)> {
        self.conn
            .query_row(
                "SELECT
                    COUNT(*),
                    SUM(CASE WHEN severity = 'critical' THEN 1 ELSE 0 END)
                 FROM risk_flags
                 WHERE student_id = ?1 AND status IN ('active', 'monitoring')",
                [student_id],
                |row| Ok((row.get(0)?, row.get::<_, Option<i64>>(1)?.unwrap_or(0))),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn active_intervention_count(&self, student_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM intervention_records
                 WHERE student_id = ?1 AND status IN ('active', 'review', 'escalated')",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn overdue_review_count(&self, student_id: i64) -> EcoachResult<i64> {
        self.conn
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
            .map_err(|err| EcoachError::Storage(err.to_string()))
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
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(last_activity
            .as_deref()
            .and_then(parse_timestamp)
            .map(|timestamp| (Utc::now() - timestamp).num_days()))
    }

    fn top_risks(&self, student_id: i64, limit: usize) -> EcoachResult<Vec<String>> {
        let mut statement = self
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
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut risks = Vec::new();
        for row in rows {
            risks.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(risks)
    }
}

fn derive_attention_level(
    readiness_band: &str,
    critical_risk_count: i64,
    active_risk_count: i64,
    intervention_count: i64,
    overdue_review_count: i64,
    inactive_days: Option<i64>,
) -> String {
    if critical_risk_count > 0 || matches!(readiness_band, "At Risk" | "Not Ready") {
        return "high".to_string();
    }
    if active_risk_count > 0
        || intervention_count > 0
        || overdue_review_count > 0
        || inactive_days.unwrap_or(0) >= 4
    {
        return "medium".to_string();
    }
    "low".to_string()
}

fn build_follow_up_actions(
    readiness_band: &str,
    strategy_summary: Option<&ReportingStrategySummary>,
    parent_count: i64,
    critical_risk_count: i64,
    intervention_count: i64,
    overdue_review_count: i64,
    inactive_days: Option<i64>,
) -> Vec<String> {
    let mut actions = Vec::new();

    if critical_risk_count > 0 || matches!(readiness_band, "At Risk" | "Not Ready") {
        actions.push("Escalate this learner into an immediate oversight review.".to_string());
    }
    if intervention_count > 0 {
        actions.push("Keep active intervention records under weekly review.".to_string());
    }
    if overdue_review_count > 0 {
        actions.push("Prompt overdue review work before new topic expansion.".to_string());
    }
    if inactive_days.unwrap_or(0) >= 4 {
        actions.push("Trigger a re-engagement check because recent activity is low.".to_string());
    }
    if let Some(strategy_summary) = strategy_summary {
        actions.push(format!(
            "Current premium strategy mode is {}, so keep follow-up aligned to that plan.",
            strategy_summary.strategy_mode
        ));
        if let Some(signal) = strategy_summary.recent_focus_signals.first() {
            actions.push(format!(
                "Recent learner evidence is flagging {}, so review that surface before expansion.",
                signal.replace('_', " ")
            ));
        }
        if let Some(mode) = strategy_summary.recommended_game_modes.first() {
            actions.push(format!(
                "If remediation games are available, route the learner into {} next.",
                mode
            ));
        }
    }
    if parent_count == 0 {
        actions.push("No linked household is present, so admin follow-up is required.".to_string());
    }
    if actions.is_empty() {
        actions.push(
            "Monitor the next reporting cycle and keep the learner on the current plan."
                .to_string(),
        );
    }

    actions.truncate(4);
    actions
}

fn attention_rank(level: &str) -> BasisPoints {
    match level {
        "high" => 0,
        "medium" => 1,
        _ => 2,
    }
}

fn parse_timestamp(raw: &str) -> Option<chrono::DateTime<Utc>> {
    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(raw) {
        return Some(timestamp.with_timezone(&Utc));
    }

    chrono::NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|timestamp| chrono::DateTime::<Utc>::from_naive_utc_and_offset(timestamp, Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn admin_oversight_prioritizes_at_risk_students() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_test_schema(&conn);

        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, entitlement_tier, status)
             VALUES
                (1, 'admin', 'Ops Admin', 'standard', 'active'),
                (2, 'student', 'Ama', 'premium', 'active'),
                (3, 'parent', 'Ama Parent', 'standard', 'active')",
            [],
        )
        .expect("accounts");
        conn.execute(
            "INSERT INTO parent_student_links (parent_account_id, student_account_id)
             VALUES (3, 2)",
            [],
        )
        .expect("link");
        conn.execute(
            "INSERT INTO student_profiles (account_id, exam_target, exam_target_date, daily_study_budget_minutes)
             VALUES (2, 'BECE', '2026-06-01', 75)",
            [],
        )
        .expect("profile");
        conn.execute("INSERT INTO subjects (id, name) VALUES (10, 'Math')", [])
            .expect("subject");
        conn.execute(
            "INSERT INTO topics (id, subject_id, name) VALUES (11, 10, 'Algebra')",
            [],
        )
        .expect("topic");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, priority_score, trend_state, is_blocked, next_review_at, last_seen_at
             ) VALUES (2, 11, 2200, 7800, 9000, 'critical', 1, '2026-03-30T09:00:00Z', '2026-03-20T09:00:00Z')",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO risk_flags (student_id, severity, title, status, created_at)
             VALUES (2, 'critical', 'Algebra collapse', 'active', '2026-03-28T09:00:00Z')",
            [],
        )
        .expect("risk flag");
        conn.execute(
            "INSERT INTO intervention_records (student_id, status)
             VALUES (2, 'active')",
            [],
        )
        .expect("intervention");
        conn.execute(
            "INSERT INTO memory_states (student_id, review_due_at, decay_risk)
             VALUES (2, '2026-03-28T09:00:00Z', 7000)",
            [],
        )
        .expect("memory state");
        conn.execute(
            "INSERT INTO sessions (student_id, started_at, status)
             VALUES (2, '2026-03-24T09:00:00Z', 'completed')",
            [],
        )
        .expect("session");
        conn.execute(
            "INSERT INTO coach_plans (student_id, current_phase, daily_budget_minutes, status, updated_at)
             VALUES (2, 'foundation', 90, 'active', '2026-03-29T09:00:00Z')",
            [],
        )
        .expect("coach plan");

        let service = AdminOversightService::new(&conn);
        let snapshot = service
            .build_admin_oversight_snapshot(1)
            .expect("admin snapshot");

        assert_eq!(snapshot.total_students, 1);
        assert_eq!(snapshot.critical_students, 1);
        assert_eq!(snapshot.households_needing_attention, 1);
        assert_eq!(snapshot.students[0].attention_level, "high");
        assert_eq!(
            snapshot.students[0]
                .strategy_summary
                .as_ref()
                .map(|strategy| strategy.strategy_mode.as_str()),
            Some("rescue")
        );
        assert!(!snapshot.students[0].follow_up_actions.is_empty());
    }

    fn create_test_schema(conn: &Connection) {
        conn.execute_batch(
            "
            CREATE TABLE accounts (
                id INTEGER PRIMARY KEY,
                account_type TEXT NOT NULL,
                display_name TEXT NOT NULL,
                entitlement_tier TEXT NOT NULL,
                status TEXT NOT NULL
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
                gap_score INTEGER NOT NULL,
                priority_score INTEGER NOT NULL,
                trend_state TEXT NOT NULL,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                next_review_at TEXT,
                last_seen_at TEXT
            );
            CREATE TABLE risk_flags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                student_id INTEGER NOT NULL,
                severity TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE intervention_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                student_id INTEGER NOT NULL,
                status TEXT NOT NULL
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
