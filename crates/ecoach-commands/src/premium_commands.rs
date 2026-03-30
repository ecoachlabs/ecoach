use ecoach_premium::{
    CreateInterventionInput, InterventionStatus, PremiumService, PremiumStrategySnapshot,
    RiskFlagStatus,
};

use crate::{error::CommandError, state::AppState};
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

pub type PremiumStrategySnapshotDto = PremiumStrategySnapshot;

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
    state.with_connection(|conn| Ok(PremiumService::new(conn).get_strategy_snapshot(student_id)?))
}
