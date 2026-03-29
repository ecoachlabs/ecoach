use ecoach_sessions::{
    CustomTestStartInput, MockBlueprintInput, PracticeSessionStartInput, SessionService,
};

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
