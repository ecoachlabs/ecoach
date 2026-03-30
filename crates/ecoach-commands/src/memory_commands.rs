use ecoach_memory::{
    MemoryReturnLoop, MemoryReviewQueueItem, MemoryService, RecordMemoryEvidenceInput,
    TopicMemorySummary,
};

use crate::{error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecheckItemDto {
    pub id: i64,
    pub node_id: Option<i64>,
    pub topic_name: Option<String>,
    pub node_title: Option<String>,
    pub memory_state: Option<String>,
    pub memory_strength: Option<i64>,
    pub decay_risk: Option<i64>,
    pub due_at: String,
    pub schedule_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStateDto {
    pub id: i64,
    pub topic_id: Option<i64>,
    pub node_id: Option<i64>,
    pub memory_state: String,
    pub memory_strength: i64,
    pub recall_fluency: i64,
    pub decay_risk: i64,
    pub review_due_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDashboardDto {
    pub total_items: i64,
    pub healthy_count: i64,
    pub at_risk_count: i64,
    pub fading_count: i64,
    pub collapsed_count: i64,
    pub overdue_reviews: i64,
    pub average_strength: i64,
    pub next_review_due: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayBatchResultDto {
    pub items_processed: usize,
    pub items_decayed: usize,
    pub items_collapsed: usize,
    pub new_rechecks_scheduled: usize,
}

pub type MemoryReviewQueueItemDto = MemoryReviewQueueItem;
pub type TopicMemorySummaryDto = TopicMemorySummary;
pub type MemoryReturnLoopDto = MemoryReturnLoop;

pub fn get_review_queue(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<RecheckItemDto>, CommandError> {
    state.with_connection(|conn| {
        let service = MemoryService::new(conn);
        let items = service.get_due_rechecks(student_id, limit)?;
        Ok(items
            .into_iter()
            .map(|item| RecheckItemDto {
                id: item.id,
                node_id: item.node_id,
                topic_name: item.topic_name,
                node_title: item.node_title,
                memory_state: item.memory_state,
                memory_strength: item.memory_strength.map(|v| v as i64),
                decay_risk: item.decay_risk.map(|v| v as i64),
                due_at: item.due_at,
                schedule_type: item.schedule_type,
            })
            .collect())
    })
}

pub fn record_retrieval_attempt(
    state: &AppState,
    input: RecordMemoryEvidenceInput,
) -> Result<MemoryStateDto, CommandError> {
    state.with_connection(|conn| {
        let service = MemoryService::new(conn);
        let record = service.record_evidence(&input)?;
        Ok(MemoryStateDto {
            id: record.id,
            topic_id: record.topic_id,
            node_id: record.node_id,
            memory_state: record.memory_state,
            memory_strength: record.memory_strength as i64,
            recall_fluency: record.recall_fluency as i64,
            decay_risk: record.decay_risk as i64,
            review_due_at: record.review_due_at,
        })
    })
}

pub fn get_memory_dashboard(
    state: &AppState,
    student_id: i64,
) -> Result<MemoryDashboardDto, CommandError> {
    state.with_connection(|conn| {
        let service = MemoryService::new(conn);
        let dashboard = service.get_memory_dashboard(student_id)?;
        Ok(MemoryDashboardDto {
            total_items: dashboard.total_items,
            healthy_count: dashboard.healthy_count,
            at_risk_count: dashboard.at_risk_count,
            fading_count: dashboard.fading_count,
            collapsed_count: dashboard.collapsed_count,
            overdue_reviews: dashboard.overdue_reviews,
            average_strength: dashboard.average_strength as i64,
            next_review_due: dashboard.next_review_due,
        })
    })
}

pub fn process_decay_batch(
    state: &AppState,
    limit: usize,
) -> Result<DecayBatchResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = MemoryService::new(conn);
        let result = service.process_decay_batch(limit)?;
        Ok(DecayBatchResultDto {
            items_processed: result.items_processed,
            items_decayed: result.items_decayed,
            items_collapsed: result.items_collapsed,
            new_rechecks_scheduled: result.new_rechecks_scheduled,
        })
    })
}

pub fn complete_recheck(state: &AppState, recheck_id: i64) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = MemoryService::new(conn);
        service.complete_recheck(recheck_id)?;
        Ok(())
    })
}

pub fn build_review_queue(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<MemoryReviewQueueItemDto>, CommandError> {
    state
        .with_connection(|conn| Ok(MemoryService::new(conn).build_review_queue(student_id, limit)?))
}

pub fn list_memory_topic_summaries(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<TopicMemorySummaryDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(MemoryService::new(conn).list_topic_summaries(student_id, limit)?)
    })
}

pub fn build_memory_return_loop(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<MemoryReturnLoopDto, CommandError> {
    state.with_connection(|conn| Ok(MemoryService::new(conn).build_return_loop(student_id, limit)?))
}
