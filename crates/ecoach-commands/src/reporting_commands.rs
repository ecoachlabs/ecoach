use ecoach_reporting::{
    strategy::load_strategy_summary, AdminOversightService, AdminOversightSnapshot,
    HouseholdDashboardSnapshot, ParentDashboardSnapshot, ParentInsightService,
    ReportingStrategySummary,
};
use serde::{Deserialize, Serialize};

use crate::{dtos::FabricOrchestrationSummaryDto, error::CommandError, state::AppState};

pub type ParentDashboardSnapshotDto = ParentDashboardSnapshot;
pub type HouseholdDashboardSnapshotDto = HouseholdDashboardSnapshot;
pub type AdminOversightSnapshotDto = AdminOversightSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingStrategySummaryDto {
    pub tier: String,
    pub strategy_mode: String,
    pub overall_readiness_score: i64,
    pub overall_readiness_band: String,
    pub current_phase: Option<String>,
    pub priority_topics: Vec<String>,
    pub recent_focus_signals: Vec<String>,
    pub recommended_game_modes: Vec<String>,
    pub coach_actions: Vec<String>,
    pub household_actions: Vec<String>,
    pub orchestration: FabricOrchestrationSummaryDto,
}

impl From<ReportingStrategySummary> for ReportingStrategySummaryDto {
    fn from(value: ReportingStrategySummary) -> Self {
        Self {
            tier: value.tier,
            strategy_mode: value.strategy_mode,
            overall_readiness_score: value.overall_readiness_score as i64,
            overall_readiness_band: value.overall_readiness_band,
            current_phase: value.current_phase,
            priority_topics: value.priority_topics,
            recent_focus_signals: value.recent_focus_signals,
            recommended_game_modes: value.recommended_game_modes,
            coach_actions: value.coach_actions,
            household_actions: value.household_actions,
            orchestration: FabricOrchestrationSummaryDto::from(value.orchestration),
        }
    }
}

pub fn get_parent_dashboard(
    state: &AppState,
    parent_id: i64,
) -> Result<ParentDashboardSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(ParentInsightService::new(conn).build_parent_dashboard(parent_id)?)
    })
}

pub fn get_household_dashboard(
    state: &AppState,
    parent_id: i64,
) -> Result<HouseholdDashboardSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(ParentInsightService::new(conn).build_household_dashboard(parent_id)?)
    })
}

pub fn get_admin_oversight_snapshot(
    state: &AppState,
    admin_id: i64,
) -> Result<AdminOversightSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(AdminOversightService::new(conn).build_admin_oversight_snapshot(admin_id)?)
    })
}

pub fn get_reporting_strategy_summary(
    state: &AppState,
    student_id: i64,
) -> Result<Option<ReportingStrategySummaryDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(load_strategy_summary(conn, student_id)?.map(ReportingStrategySummaryDto::from))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecoach_substrate::{FabricConsumerTarget, FabricOrchestrationSummary};

    #[test]
    fn reporting_strategy_summary_dto_preserves_focus_signals_and_orchestration() {
        let dto = ReportingStrategySummaryDto::from(ReportingStrategySummary {
            tier: "premium".to_string(),
            strategy_mode: "rescue".to_string(),
            overall_readiness_score: 6_500,
            overall_readiness_band: "At Risk".to_string(),
            current_phase: Some("repair".to_string()),
            priority_topics: vec!["Fractions".to_string()],
            recent_focus_signals: vec!["retrieval_fragility".to_string()],
            recommended_game_modes: vec!["trapsense".to_string()],
            coach_actions: vec!["Rebuild core concept".to_string()],
            household_actions: vec!["Protect repair time".to_string()],
            orchestration: FabricOrchestrationSummary {
                available_inputs: vec!["premium_strategy".to_string()],
                consumer_targets: vec![FabricConsumerTarget {
                    engine_key: "reporting".to_string(),
                    engine_title: "Reporting".to_string(),
                    matched_inputs: vec!["premium_strategy".to_string()],
                }],
            },
        });

        assert_eq!(
            dto.recent_focus_signals,
            vec!["retrieval_fragility".to_string()]
        );
        assert_eq!(dto.recommended_game_modes, vec!["trapsense".to_string()]);
        assert_eq!(dto.orchestration.available_inputs.len(), 1);
    }
}
