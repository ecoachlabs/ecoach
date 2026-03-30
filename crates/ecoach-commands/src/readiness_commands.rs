use ecoach_reporting::{DashboardService, ParentInsightService};

use crate::{error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessReportDto {
    pub student_id: i64,
    pub overall_readiness_band: String,
    pub coverage_percent: f64,
    pub subjects: Vec<SubjectReadinessDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectReadinessDto {
    pub subject_id: i64,
    pub subject_name: String,
    pub readiness_band: String,
    pub total_topic_count: usize,
    pub mastered_topic_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentDigestDto {
    pub parent_id: i64,
    pub parent_name: String,
    pub students: Vec<ParentStudentDigestDto>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentStudentDigestDto {
    pub student_id: i64,
    pub student_name: String,
    pub overall_readiness_band: String,
    pub active_risks: Vec<String>,
    pub recommendations: Vec<String>,
    pub weekly_memo: String,
}

pub fn get_readiness_report(
    state: &AppState,
    student_id: i64,
) -> Result<ReadinessReportDto, CommandError> {
    state.with_connection(|conn| {
        let service = DashboardService::new(conn);
        let dashboard = service.get_student_dashboard(student_id)?;
        let subjects: Vec<SubjectReadinessDto> = dashboard
            .subject_summaries
            .into_iter()
            .map(|s| SubjectReadinessDto {
                subject_id: s.subject_id,
                subject_name: s.subject_name,
                readiness_band: s.readiness_band,
                total_topic_count: s.total_topic_count,
                mastered_topic_count: s.mastered_topic_count,
            })
            .collect();

        let coverage = if !subjects.is_empty() {
            let total: usize = subjects.iter().map(|s| s.total_topic_count).sum();
            let mastered: usize = subjects.iter().map(|s| s.mastered_topic_count).sum();
            if total > 0 {
                mastered as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok(ReadinessReportDto {
            student_id,
            overall_readiness_band: dashboard.overall_readiness_band,
            coverage_percent: coverage,
            subjects,
        })
    })
}

pub fn generate_parent_digest(
    state: &AppState,
    parent_id: i64,
) -> Result<ParentDigestDto, CommandError> {
    state.with_connection(|conn| {
        let service = ParentInsightService::new(conn);
        let snapshot = service.build_parent_dashboard(parent_id)?;
        let students = snapshot
            .students
            .into_iter()
            .map(|s| ParentStudentDigestDto {
                student_id: s.student_id,
                student_name: s.student_name,
                overall_readiness_band: s.overall_readiness_band,
                active_risks: s.active_risks.into_iter().map(|r| r.title).collect(),
                recommendations: s.recommendations,
                weekly_memo: s.weekly_memo,
            })
            .collect();
        Ok(ParentDigestDto {
            parent_id: snapshot.parent_id,
            parent_name: snapshot.parent_name,
            students,
            generated_at: snapshot.generated_at,
        })
    })
}
