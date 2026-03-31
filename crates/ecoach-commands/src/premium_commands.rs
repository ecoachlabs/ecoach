use ecoach_premium::{
    CreateConciergeResponseInput, CreateInterventionInput, CreateMilestoneReviewInput,
    CreateParentCommunicationInput, CreatePremiumIntakeInput, CreateReadinessProfileInput,
    InterventionStatus, PremiumPriorityTopic, PremiumService, PremiumStrategySnapshot,
    RiskFlagStatus,
};

use crate::{dtos::FabricOrchestrationSummaryDto, error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFlagDto {
    pub id: i64,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionDto {
    pub id: i64,
    pub title: String,
    pub status: String,
    pub step_count: usize,
    pub risk_flag_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDashboardDto {
    pub critical_count: i64,
    pub high_count: i64,
    pub medium_count: i64,
    pub low_count: i64,
    pub active_interventions: i64,
    pub flags: Vec<RiskFlagDto>,
    pub interventions: Vec<InterventionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementSnapshotDto {
    pub tier: String,
    pub active_risk_flags: i64,
    pub active_interventions: i64,
    pub premium_features_enabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumPriorityTopicDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_score: i64,
    pub gap_score: i64,
    pub priority_score: i64,
    pub trend_state: String,
    pub is_blocked: bool,
    pub next_review_at: Option<String>,
}

impl From<PremiumPriorityTopic> for PremiumPriorityTopicDto {
    fn from(value: PremiumPriorityTopic) -> Self {
        Self {
            topic_id: value.topic_id,
            topic_name: value.topic_name,
            mastery_score: value.mastery_score as i64,
            gap_score: value.gap_score as i64,
            priority_score: value.priority_score as i64,
            trend_state: value.trend_state,
            is_blocked: value.is_blocked,
            next_review_at: value.next_review_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumStrategySnapshotDto {
    pub student_id: i64,
    pub student_name: String,
    pub tier: String,
    pub strategy_mode: String,
    pub overall_readiness_score: i64,
    pub overall_readiness_band: String,
    pub exam_target: Option<String>,
    pub exam_target_date: Option<String>,
    pub current_phase: Option<String>,
    pub daily_budget_minutes: Option<i64>,
    pub inactive_days: Option<i64>,
    pub overdue_review_count: i64,
    pub active_risk_count: i64,
    pub critical_risk_count: i64,
    pub active_intervention_count: i64,
    pub priority_topics: Vec<PremiumPriorityTopicDto>,
    pub top_risk_titles: Vec<String>,
    pub recent_focus_signals: Vec<String>,
    pub recommended_game_modes: Vec<String>,
    pub coach_actions: Vec<String>,
    pub household_actions: Vec<String>,
    pub orchestration: FabricOrchestrationSummaryDto,
}

impl From<PremiumStrategySnapshot> for PremiumStrategySnapshotDto {
    fn from(value: PremiumStrategySnapshot) -> Self {
        Self {
            student_id: value.student_id,
            student_name: value.student_name,
            tier: value.tier,
            strategy_mode: value.strategy_mode,
            overall_readiness_score: value.overall_readiness_score as i64,
            overall_readiness_band: value.overall_readiness_band,
            exam_target: value.exam_target,
            exam_target_date: value.exam_target_date,
            current_phase: value.current_phase,
            daily_budget_minutes: value.daily_budget_minutes,
            inactive_days: value.inactive_days,
            overdue_review_count: value.overdue_review_count,
            active_risk_count: value.active_risk_count,
            critical_risk_count: value.critical_risk_count,
            active_intervention_count: value.active_intervention_count,
            priority_topics: value
                .priority_topics
                .into_iter()
                .map(PremiumPriorityTopicDto::from)
                .collect(),
            top_risk_titles: value.top_risk_titles,
            recent_focus_signals: value.recent_focus_signals,
            recommended_game_modes: value.recommended_game_modes,
            coach_actions: value.coach_actions,
            household_actions: value.household_actions,
            orchestration: FabricOrchestrationSummaryDto::from(value.orchestration),
        }
    }
}

pub fn get_risk_dashboard(
    state: &AppState,
    student_id: i64,
) -> Result<RiskDashboardDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let dashboard = service.get_risk_dashboard(student_id)?;
        Ok(RiskDashboardDto {
            critical_count: dashboard.critical_count,
            high_count: dashboard.high_count,
            medium_count: dashboard.medium_count,
            low_count: dashboard.low_count,
            active_interventions: dashboard.active_interventions,
            flags: dashboard
                .flags
                .into_iter()
                .map(|f| RiskFlagDto {
                    id: f.id,
                    topic_id: f.topic_id,
                    topic_name: f.topic_name,
                    severity: f.severity,
                    title: f.title,
                    description: f.description,
                    status: f.status,
                })
                .collect(),
            interventions: dashboard
                .interventions
                .into_iter()
                .map(|i| InterventionDto {
                    id: i.id,
                    title: i.title,
                    status: i.status,
                    step_count: i.steps.len(),
                    risk_flag_id: i.risk_flag_id,
                })
                .collect(),
        })
    })
}

pub fn auto_detect_risks(
    state: &AppState,
    student_id: i64,
) -> Result<Vec<RiskFlagDto>, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let flags = service.auto_detect_risk_flags(student_id)?;
        Ok(flags
            .into_iter()
            .map(|f| RiskFlagDto {
                id: f.id,
                topic_id: f.topic_id,
                topic_name: f.topic_name,
                severity: f.severity,
                title: f.title,
                description: f.description,
                status: f.status,
            })
            .collect())
    })
}

pub fn create_intervention(
    state: &AppState,
    input: CreateInterventionInput,
) -> Result<InterventionDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let intervention = service.create_intervention(&input)?;
        Ok(InterventionDto {
            id: intervention.id,
            title: intervention.title,
            status: intervention.status,
            step_count: intervention.steps.len(),
            risk_flag_id: intervention.risk_flag_id,
        })
    })
}

pub fn resolve_risk_flag(state: &AppState, flag_id: i64) -> Result<RiskFlagDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let flag = service.update_risk_flag_status(flag_id, RiskFlagStatus::Resolved)?;
        Ok(RiskFlagDto {
            id: flag.id,
            topic_id: flag.topic_id,
            topic_name: flag.topic_name,
            severity: flag.severity,
            title: flag.title,
            description: flag.description,
            status: flag.status,
        })
    })
}

pub fn resolve_intervention(
    state: &AppState,
    intervention_id: i64,
) -> Result<InterventionDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let intervention =
            service.update_intervention_status(intervention_id, InterventionStatus::Resolved)?;
        Ok(InterventionDto {
            id: intervention.id,
            title: intervention.title,
            status: intervention.status,
            step_count: intervention.steps.len(),
            risk_flag_id: intervention.risk_flag_id,
        })
    })
}

pub fn check_entitlement(
    state: &AppState,
    student_id: i64,
) -> Result<EntitlementSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let snapshot = service.get_entitlement_snapshot(student_id)?;
        Ok(EntitlementSnapshotDto {
            tier: snapshot.tier,
            active_risk_flags: snapshot.active_risk_flags,
            active_interventions: snapshot.active_interventions,
            premium_features_enabled: snapshot.premium_features_enabled,
        })
    })
}

pub fn is_feature_enabled(
    state: &AppState,
    student_id: i64,
    feature_key: String,
) -> Result<bool, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let enabled = service.is_feature_enabled(student_id, &feature_key)?;
        Ok(enabled)
    })
}

pub fn get_strategy_snapshot(
    state: &AppState,
    student_id: i64,
) -> Result<PremiumStrategySnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(PremiumStrategySnapshotDto::from(
            PremiumService::new(conn).get_strategy_snapshot(student_id)?,
        ))
    })
}

// ── Premium concierge commands (idea12) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessProfileDto {
    pub id: i64,
    pub overall_readiness_bp: i64,
    pub overall_band: String,
    pub knowledge_solidity_bp: i64,
    pub speed_under_pressure_bp: i64,
    pub memory_stability_bp: i64,
    pub trajectory: Option<String>,
    pub interpretation: Option<String>,
    pub snapshot_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneReviewDto {
    pub id: i64,
    pub review_type: String,
    pub readiness_band: String,
    pub overall_trend: String,
    pub executive_position: String,
    pub forecast_summary: Option<String>,
    pub parent_guidance: Option<String>,
    pub review_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConciergeResponseDto {
    pub id: i64,
    pub question_family: Option<String>,
    pub parent_question: String,
    pub direct_answer: String,
    pub expected_outcome: Option<String>,
    pub parent_action_needed: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStateDto {
    pub primary_focus: Option<String>,
    pub secondary_focus: Option<String>,
    pub focus_reason: Option<String>,
    pub expected_outcome: Option<String>,
    pub mode_selection: Option<String>,
    pub next_review_date: Option<String>,
    pub last_shift_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentCommunicationDto {
    pub id: i64,
    pub comm_type: String,
    pub priority: i64,
    pub title: String,
    pub body: String,
    pub read_at: Option<String>,
    pub created_at: String,
}

pub fn snapshot_readiness(
    state: &AppState,
    input: CreateReadinessProfileInput,
) -> Result<ReadinessProfileDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let profile = service.snapshot_readiness_profile(&input)?;
        Ok(ReadinessProfileDto {
            id: profile.id,
            overall_readiness_bp: profile.overall_readiness_bp as i64,
            overall_band: profile.overall_band,
            knowledge_solidity_bp: profile.knowledge_solidity_bp as i64,
            speed_under_pressure_bp: profile.speed_under_pressure_bp as i64,
            memory_stability_bp: profile.memory_stability_bp as i64,
            trajectory: profile.trajectory,
            interpretation: profile.interpretation,
            snapshot_date: profile.snapshot_date,
        })
    })
}

pub fn list_readiness_trend(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<ReadinessProfileDto>, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let profiles = service.list_readiness_trend(student_id, limit)?;
        Ok(profiles
            .into_iter()
            .map(|p| ReadinessProfileDto {
                id: p.id,
                overall_readiness_bp: p.overall_readiness_bp as i64,
                overall_band: p.overall_band,
                knowledge_solidity_bp: p.knowledge_solidity_bp as i64,
                speed_under_pressure_bp: p.speed_under_pressure_bp as i64,
                memory_stability_bp: p.memory_stability_bp as i64,
                trajectory: p.trajectory,
                interpretation: p.interpretation,
                snapshot_date: p.snapshot_date,
            })
            .collect())
    })
}

pub fn create_milestone_review(
    state: &AppState,
    input: CreateMilestoneReviewInput,
) -> Result<MilestoneReviewDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let review = service.create_milestone_review(&input)?;
        Ok(MilestoneReviewDto {
            id: review.id,
            review_type: review.review_type,
            readiness_band: review.readiness_band,
            overall_trend: review.overall_trend,
            executive_position: review.executive_position,
            forecast_summary: review.forecast_summary,
            parent_guidance: review.parent_guidance,
            review_date: review.review_date,
        })
    })
}

pub fn list_milestone_reviews(
    state: &AppState,
    student_id: i64,
) -> Result<Vec<MilestoneReviewDto>, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let reviews = service.list_milestone_reviews(student_id)?;
        Ok(reviews
            .into_iter()
            .map(|r| MilestoneReviewDto {
                id: r.id,
                review_type: r.review_type,
                readiness_band: r.readiness_band,
                overall_trend: r.overall_trend,
                executive_position: r.executive_position,
                forecast_summary: r.forecast_summary,
                parent_guidance: r.parent_guidance,
                review_date: r.review_date,
            })
            .collect())
    })
}

pub fn create_concierge_response(
    state: &AppState,
    input: CreateConciergeResponseInput,
) -> Result<ConciergeResponseDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let response = service.create_concierge_response(&input)?;
        Ok(ConciergeResponseDto {
            id: response.id,
            question_family: response.question_family,
            parent_question: response.parent_question,
            direct_answer: response.direct_answer,
            expected_outcome: response.expected_outcome,
            parent_action_needed: response.parent_action_needed,
            created_at: response.created_at,
        })
    })
}

pub fn list_concierge_history(
    state: &AppState,
    parent_id: i64,
    student_id: i64,
    limit: usize,
) -> Result<Vec<ConciergeResponseDto>, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let responses = service.list_concierge_history(parent_id, student_id, limit)?;
        Ok(responses
            .into_iter()
            .map(|r| ConciergeResponseDto {
                id: r.id,
                question_family: r.question_family,
                parent_question: r.parent_question,
                direct_answer: r.direct_answer,
                expected_outcome: r.expected_outcome,
                parent_action_needed: r.parent_action_needed,
                created_at: r.created_at,
            })
            .collect())
    })
}

pub fn get_strategy_state(
    state: &AppState,
    student_id: i64,
) -> Result<StrategyStateDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let strategy = service.get_strategy_state(student_id)?;
        Ok(StrategyStateDto {
            primary_focus: strategy.primary_focus,
            secondary_focus: strategy.secondary_focus,
            focus_reason: strategy.focus_reason,
            expected_outcome: strategy.expected_outcome,
            mode_selection: strategy.mode_selection,
            next_review_date: strategy.next_review_date,
            last_shift_date: strategy.last_shift_date,
        })
    })
}

pub fn send_parent_communication(
    state: &AppState,
    input: CreateParentCommunicationInput,
) -> Result<ParentCommunicationDto, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let comm = service.send_parent_communication(&input)?;
        Ok(ParentCommunicationDto {
            id: comm.id,
            comm_type: comm.comm_type,
            priority: comm.priority,
            title: comm.title,
            body: comm.body,
            read_at: comm.read_at,
            created_at: comm.created_at,
        })
    })
}

pub fn list_parent_communications(
    state: &AppState,
    parent_id: i64,
    limit: usize,
) -> Result<Vec<ParentCommunicationDto>, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let comms = service.list_parent_communications(parent_id, limit)?;
        Ok(comms
            .into_iter()
            .map(|c| ParentCommunicationDto {
                id: c.id,
                comm_type: c.comm_type,
                priority: c.priority,
                title: c.title,
                body: c.body,
                read_at: c.read_at,
                created_at: c.created_at,
            })
            .collect())
    })
}

pub fn create_premium_intake(
    state: &AppState,
    input: CreatePremiumIntakeInput,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        let service = PremiumService::new(conn);
        let intake = service.create_premium_intake(&input)?;
        Ok(intake.id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecoach_substrate::{FabricConsumerTarget, FabricOrchestrationSummary};

    #[test]
    fn premium_strategy_snapshot_dto_preserves_focus_signals_and_orchestration() {
        let dto = PremiumStrategySnapshotDto::from(PremiumStrategySnapshot {
            student_id: 42,
            student_name: "Ama".to_string(),
            tier: "premium".to_string(),
            strategy_mode: "rescue".to_string(),
            overall_readiness_score: 6_700,
            overall_readiness_band: "At Risk".to_string(),
            exam_target: Some("WAEC".to_string()),
            exam_target_date: Some("2026-05-15".to_string()),
            current_phase: Some("stabilize".to_string()),
            daily_budget_minutes: Some(60),
            inactive_days: Some(2),
            overdue_review_count: 3,
            active_risk_count: 2,
            critical_risk_count: 1,
            active_intervention_count: 1,
            priority_topics: vec![PremiumPriorityTopic {
                topic_id: 7,
                topic_name: "Fractions".to_string(),
                mastery_score: 4_200,
                gap_score: 7_800,
                priority_score: 8_100,
                trend_state: "declining".to_string(),
                is_blocked: true,
                next_review_at: Some("2026-03-31T08:00:00Z".to_string()),
            }],
            top_risk_titles: vec!["retrieval fragility".to_string()],
            recent_focus_signals: vec!["confusion_spike".to_string()],
            recommended_game_modes: vec!["trapsense".to_string()],
            coach_actions: vec!["Reopen diagnosis".to_string()],
            household_actions: vec!["Protect repair time".to_string()],
            orchestration: FabricOrchestrationSummary {
                available_inputs: vec!["learner_evidence_fabric".to_string()],
                consumer_targets: vec![FabricConsumerTarget {
                    engine_key: "reporting".to_string(),
                    engine_title: "Reporting".to_string(),
                    matched_inputs: vec!["learner_evidence_fabric".to_string()],
                }],
            },
        });

        assert_eq!(
            dto.recent_focus_signals,
            vec!["confusion_spike".to_string()]
        );
        assert_eq!(dto.recommended_game_modes, vec!["trapsense".to_string()]);
        assert_eq!(dto.priority_topics[0].priority_score, 8_100);
        assert_eq!(
            dto.orchestration.consumer_targets[0].engine_key,
            "reporting"
        );
    }
}
