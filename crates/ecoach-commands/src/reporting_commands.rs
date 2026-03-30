use ecoach_reporting::{
    AdminOversightService, AdminOversightSnapshot, HouseholdDashboardSnapshot,
    ParentDashboardSnapshot, ParentInsightService,
};

use crate::{error::CommandError, state::AppState};

pub type ParentDashboardSnapshotDto = ParentDashboardSnapshot;
pub type HouseholdDashboardSnapshotDto = HouseholdDashboardSnapshot;
pub type AdminOversightSnapshotDto = AdminOversightSnapshot;

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
