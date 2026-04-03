use ecoach_memory::{
    CompleteInterventionStepInput, InterventionPlanRecord, MemoryCohortAnalytics,
    MemoryKnowledgeStateDetail, MemoryReturnLoop, MemoryReviewQueueItem, MemoryService,
    RecordMemoryEvidenceInput, ReviewScheduleItemRecord, StudentInterferenceEdge,
    TopicKnowledgeMap, TopicMemorySummary,
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
pub type MemoryKnowledgeStateDto = MemoryKnowledgeStateDetail;
pub type ReviewScheduleItemDto = ReviewScheduleItemRecord;
pub type ActiveInterventionDto = InterventionPlanRecord;
pub type InterventionStepInputDto = CompleteInterventionStepInput;
pub type MemoryCohortAnalyticsDto = MemoryCohortAnalytics;
pub type StudentInterferenceEdgeDto = StudentInterferenceEdge;
pub type TopicKnowledgeMapDto = TopicKnowledgeMap;

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

pub fn get_memory_knowledge_state(
    state: &AppState,
    student_id: i64,
    node_id: i64,
) -> Result<MemoryKnowledgeStateDto, CommandError> {
    state.with_connection(|conn| {
        Ok(MemoryService::new(conn).get_memory_knowledge_state(student_id, node_id)?)
    })
}

pub fn list_memory_review_schedule(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<ReviewScheduleItemDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(MemoryService::new(conn).list_memory_review_schedule(student_id, limit)?)
    })
}

pub fn list_active_interventions(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<ActiveInterventionDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(MemoryService::new(conn).list_active_interventions(student_id, limit)?)
    })
}

pub fn complete_intervention_step(
    state: &AppState,
    input: InterventionStepInputDto,
) -> Result<ActiveInterventionDto, CommandError> {
    state.with_connection(|conn| Ok(MemoryService::new(conn).complete_intervention_step(&input)?))
}

pub fn force_recompute_knowledge_state(
    state: &AppState,
    student_id: i64,
    knowledge_unit_id: i64,
) -> Result<MemoryKnowledgeStateDto, CommandError> {
    state.with_connection(|conn| {
        Ok(MemoryService::new(conn)
            .force_recompute_knowledge_state(student_id, knowledge_unit_id)?)
    })
}

pub fn get_memory_cohort_analytics(
    state: &AppState,
    topic_id: i64,
    hotspot_limit: usize,
) -> Result<MemoryCohortAnalyticsDto, CommandError> {
    state.with_connection(|conn| {
        Ok(MemoryService::new(conn).get_memory_cohort_analytics(topic_id, hotspot_limit)?)
    })
}

pub fn list_student_interference_edges(
    state: &AppState,
    student_id: i64,
    node_id: i64,
    limit: usize,
) -> Result<Vec<StudentInterferenceEdgeDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(MemoryService::new(conn).list_student_interference_edges(student_id, node_id, limit)?)
    })
}

pub fn get_topic_knowledge_map(
    state: &AppState,
    topic_id: i64,
) -> Result<TopicKnowledgeMapDto, CommandError> {
    state.with_connection(|conn| Ok(MemoryService::new(conn).get_topic_knowledge_map(topic_id)?))
}

// ── Memory Intelligence commands ──

pub fn recompute_skill_memory(
    state: &AppState,
    student_id: i64,
    node_id: i64,
) -> Result<ecoach_coach_brain::MemoryScoreUpdate, CommandError> {
    state.with_connection(|conn| {
        Ok(ecoach_coach_brain::MemoryIntelligenceEngine::new(conn)
            .recompute_skill_memory(student_id, node_id)?)
    })
}

pub fn get_skill_proof_status(
    state: &AppState,
    student_id: i64,
    node_id: i64,
) -> Result<Vec<ecoach_coach_brain::ProofStatus>, CommandError> {
    state.with_connection(|conn| {
        Ok(ecoach_coach_brain::MemoryIntelligenceEngine::new(conn)
            .get_proof_status(student_id, node_id)?)
    })
}
