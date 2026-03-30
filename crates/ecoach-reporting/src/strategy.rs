use ecoach_premium::{PremiumService, PremiumStrategySnapshot};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingStrategySummary {
    pub tier: String,
    pub strategy_mode: String,
    pub overall_readiness_score: BasisPoints,
    pub overall_readiness_band: String,
    pub current_phase: Option<String>,
    pub priority_topics: Vec<String>,
    pub coach_actions: Vec<String>,
    pub household_actions: Vec<String>,
}

pub fn load_strategy_summary(
    conn: &Connection,
    student_id: i64,
) -> EcoachResult<Option<ReportingStrategySummary>> {
    match PremiumService::new(conn).get_strategy_snapshot(student_id) {
        Ok(snapshot) => Ok(Some(from_snapshot(snapshot))),
        Err(EcoachError::Unauthorized(_)) => Ok(None),
        Err(EcoachError::NotFound(_)) => Ok(None),
        Err(err) => Err(err),
    }
}

fn from_snapshot(snapshot: PremiumStrategySnapshot) -> ReportingStrategySummary {
    ReportingStrategySummary {
        tier: snapshot.tier,
        strategy_mode: snapshot.strategy_mode,
        overall_readiness_score: snapshot.overall_readiness_score,
        overall_readiness_band: snapshot.overall_readiness_band,
        current_phase: snapshot.current_phase,
        priority_topics: snapshot
            .priority_topics
            .into_iter()
            .map(|topic| topic.topic_name)
            .collect(),
        coach_actions: snapshot.coach_actions,
        household_actions: snapshot.household_actions,
    }
}
