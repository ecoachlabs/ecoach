use ecoach_knowledge_gap::{
    CreateGapRepairPlanInput, GapRepairPlan, KnowledgeGapService, RepairItemStatus,
};

use crate::{error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapScoreCardDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub gap_score: i64,
    pub mastery_score: i64,
    pub severity_label: String,
    pub repair_priority: i64,
    pub has_active_repair_plan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapRepairPlanDto {
    pub id: i64,
    pub topic_id: i64,
    pub topic_name: Option<String>,
    pub status: String,
    pub priority_score: i64,
    pub severity_label: String,
    pub dominant_focus: String,
    pub recommended_session_type: String,
    pub item_count: usize,
    pub progress_percent: i64,
}

pub type GapRepairPlanDetailDto = GapRepairPlan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapDashboardDto {
    pub critical_gap_count: usize,
    pub active_repair_count: usize,
    pub topics_solidified: i64,
    pub gaps: Vec<GapScoreCardDto>,
    pub repairs: Vec<GapRepairPlanDto>,
}

pub fn list_priority_gaps(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<GapScoreCardDto>, CommandError> {
    state.with_connection(|conn| {
        let service = KnowledgeGapService::new(conn);
        let cards = service.list_priority_gaps(student_id, limit)?;
        Ok(cards
            .into_iter()
            .map(|c| GapScoreCardDto {
                topic_id: c.topic_id,
                topic_name: c.topic_name,
                gap_score: c.gap_score as i64,
                mastery_score: c.mastery_score as i64,
                severity_label: c.severity_label,
                repair_priority: c.repair_priority as i64,
                has_active_repair_plan: c.has_active_repair_plan,
            })
            .collect())
    })
}

pub fn generate_repair_plan(
    state: &AppState,
    student_id: i64,
    topic_id: i64,
) -> Result<GapRepairPlanDto, CommandError> {
    state.with_connection(|conn| {
        let service = KnowledgeGapService::new(conn);
        let plan = service.create_repair_plan(&CreateGapRepairPlanInput {
            student_id,
            topic_id,
        })?;
        Ok(GapRepairPlanDto {
            id: plan.id,
            topic_id: plan.topic_id,
            topic_name: plan.topic_name,
            status: plan.status,
            priority_score: plan.priority_score as i64,
            severity_label: plan.severity_label,
            dominant_focus: plan.dominant_focus,
            recommended_session_type: plan.recommended_session_type,
            item_count: plan.items.len(),
            progress_percent: plan.progress_percent as i64,
        })
    })
}

pub fn get_repair_plan(
    state: &AppState,
    plan_id: i64,
) -> Result<GapRepairPlanDetailDto, CommandError> {
    state.with_connection(|conn| Ok(KnowledgeGapService::new(conn).get_repair_plan(plan_id)?))
}

pub fn advance_repair_item(
    state: &AppState,
    item_id: i64,
    completed: bool,
) -> Result<GapRepairPlanDto, CommandError> {
    state.with_connection(|conn| {
        let service = KnowledgeGapService::new(conn);
        let status = if completed {
            RepairItemStatus::Completed
        } else {
            RepairItemStatus::Skipped
        };
        let plan = service.advance_repair_item(item_id, status)?;
        Ok(GapRepairPlanDto {
            id: plan.id,
            topic_id: plan.topic_id,
            topic_name: plan.topic_name,
            status: plan.status,
            priority_score: plan.priority_score as i64,
            severity_label: plan.severity_label,
            dominant_focus: plan.dominant_focus,
            recommended_session_type: plan.recommended_session_type,
            item_count: plan.items.len(),
            progress_percent: plan.progress_percent as i64,
        })
    })
}

pub fn get_gap_dashboard(
    state: &AppState,
    student_id: i64,
) -> Result<GapDashboardDto, CommandError> {
    state.with_connection(|conn| {
        let service = KnowledgeGapService::new(conn);
        let dashboard = service.get_gap_dashboard(student_id)?;
        Ok(GapDashboardDto {
            critical_gap_count: dashboard.critical_gaps.len(),
            active_repair_count: dashboard.active_repairs.len(),
            topics_solidified: dashboard.solidification_progress.topics_solidified,
            gaps: dashboard
                .critical_gaps
                .into_iter()
                .map(|c| GapScoreCardDto {
                    topic_id: c.topic_id,
                    topic_name: c.topic_name,
                    gap_score: c.gap_score as i64,
                    mastery_score: c.mastery_score as i64,
                    severity_label: c.severity_label,
                    repair_priority: c.repair_priority as i64,
                    has_active_repair_plan: c.has_active_repair_plan,
                })
                .collect(),
            repairs: dashboard
                .active_repairs
                .into_iter()
                .map(|p| GapRepairPlanDto {
                    id: p.id,
                    topic_id: p.topic_id,
                    topic_name: p.topic_name,
                    status: p.status,
                    priority_score: p.priority_score as i64,
                    severity_label: p.severity_label,
                    dominant_focus: p.dominant_focus,
                    recommended_session_type: p.recommended_session_type,
                    item_count: p.items.len(),
                    progress_percent: p.progress_percent as i64,
                })
                .collect(),
        })
    })
}

// ── Knowledge Gap Deep commands ──

pub fn capture_gap_snapshot(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<ecoach_knowledge_gap::GapSnapshotResult, CommandError> {
    state.with_connection(|conn| {
        Ok(KnowledgeGapService::new(conn).capture_gap_snapshot(student_id, subject_id)?)
    })
}

pub fn list_gap_trend(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    limit: usize,
) -> Result<Vec<ecoach_knowledge_gap::GapTrendPoint>, CommandError> {
    state.with_connection(|conn| {
        Ok(KnowledgeGapService::new(conn).list_gap_trend(student_id, subject_id, limit)?)
    })
}

pub fn list_gap_feed(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    limit: usize,
) -> Result<Vec<ecoach_knowledge_gap::GapFeedItem>, CommandError> {
    state.with_connection(|conn| {
        Ok(KnowledgeGapService::new(conn).list_gap_feed(student_id, subject_id, limit)?)
    })
}
