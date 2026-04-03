use ecoach_sessions::{
    CoachMissionSessionPlan, CustomTestStartInput, FocusModeConfig, MockBlueprintInput,
    PracticeSessionStartInput, SessionPresenceEvent, SessionPresenceEventInput,
    SessionPresenceSnapshot, SessionService,
};
use serde_json::Value;

use crate::{
    dtos::{MockBlueprintDto, SessionSnapshotDto, SessionSummaryDto},
    error::CommandError,
    state::AppState,
};

pub fn start_practice_session(
    state: &AppState,
    input: PracticeSessionStartInput,
) -> Result<SessionSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = SessionService::new(conn);
        let (session, _) = service.start_practice_session(&input)?;
        let snapshot = service
            .get_session_snapshot(session.id)?
            .ok_or_else(|| CommandError {
                code: "not_found".to_string(),
                message: format!("session {} snapshot was not created", session.id),
            })?;
        Ok(SessionSnapshotDto::from(snapshot))
    })
}

pub type CoachMissionSessionPlanDto = CoachMissionSessionPlan;

pub fn start_coach_mission_session(
    state: &AppState,
    student_id: i64,
) -> Result<CoachMissionSessionPlanDto, CommandError> {
    state.with_connection(|conn| {
        Ok(SessionService::new(conn).start_coach_mission_session(student_id)?)
    })
}

pub fn compose_custom_test(
    state: &AppState,
    input: CustomTestStartInput,
) -> Result<SessionSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = SessionService::new(conn);
        let (session, _) = service.start_custom_test(&input)?;
        let snapshot = service
            .get_session_snapshot(session.id)?
            .ok_or_else(|| CommandError {
                code: "not_found".to_string(),
                message: format!("session {} snapshot was not created", session.id),
            })?;
        Ok(SessionSnapshotDto::from(snapshot))
    })
}

pub fn generate_mock_blueprint(
    state: &AppState,
    input: MockBlueprintInput,
) -> Result<MockBlueprintDto, CommandError> {
    state.with_connection(|conn| {
        let service = SessionService::new(conn);
        let blueprint = service.generate_mock_blueprint(&input)?;
        Ok(MockBlueprintDto::from(blueprint))
    })
}

pub fn start_mock_session(
    state: &AppState,
    blueprint_id: i64,
) -> Result<SessionSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = SessionService::new(conn);
        let (session, _) = service.start_mock_session(blueprint_id)?;
        let snapshot = service
            .get_session_snapshot(session.id)?
            .ok_or_else(|| CommandError {
                code: "not_found".to_string(),
                message: format!("session {} snapshot was not created", session.id),
            })?;
        Ok(SessionSnapshotDto::from(snapshot))
    })
}

pub fn complete_session(
    state: &AppState,
    session_id: i64,
) -> Result<SessionSummaryDto, CommandError> {
    state.with_connection(|conn| {
        let service = SessionService::new(conn);
        let summary = service.complete_session(session_id)?;
        Ok(SessionSummaryDto::from(summary))
    })
}

pub type FocusModeConfigDto = FocusModeConfig;
pub type SessionPresenceEventDto = SessionPresenceEvent;
pub type SessionPresenceEventInputDto = SessionPresenceEventInput;
pub type SessionPresenceSnapshotDto = SessionPresenceSnapshot;

pub fn enable_focus_mode(
    state: &AppState,
    session_id: i64,
    focus_goal: Option<String>,
    break_schedule_json: Option<Value>,
    ambient_profile: Option<String>,
) -> Result<FocusModeConfigDto, CommandError> {
    state.with_connection(|conn| {
        let service = SessionService::new(conn);
        service.enable_focus_mode(session_id, focus_goal, break_schedule_json, ambient_profile)?;
        service
            .get_focus_mode_config(session_id)?
            .ok_or_else(|| CommandError {
                code: "not_found".to_string(),
                message: format!("focus mode config for session {} was not found", session_id),
            })
    })
}

pub fn get_focus_mode_config(
    state: &AppState,
    session_id: i64,
) -> Result<Option<FocusModeConfigDto>, CommandError> {
    state.with_connection(|conn| Ok(SessionService::new(conn).get_focus_mode_config(session_id)?))
}

pub fn manual_stop_session(
    state: &AppState,
    session_id: i64,
    reason: Option<String>,
) -> Result<SessionSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let snapshot = SessionService::new(conn).manual_stop_session(session_id, reason)?;
        Ok(SessionSnapshotDto::from(snapshot))
    })
}

pub fn record_session_presence_event(
    state: &AppState,
    session_id: i64,
    input: SessionPresenceEventInputDto,
) -> Result<SessionPresenceSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(SessionService::new(conn).record_session_presence_event(session_id, &input)?)
    })
}

pub fn get_session_presence_snapshot(
    state: &AppState,
    session_id: i64,
) -> Result<Option<SessionPresenceSnapshotDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(SessionService::new(conn).get_session_presence_snapshot(session_id)?)
    })
}

pub fn list_session_presence_events(
    state: &AppState,
    session_id: i64,
    limit: usize,
) -> Result<Vec<SessionPresenceEventDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(SessionService::new(conn).list_session_presence_events(session_id, limit)?)
    })
}
