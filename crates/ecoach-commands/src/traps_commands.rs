use ecoach_games::{
    GamesService, StartTrapsSessionInput, SubmitTrapConfusionReasonInput, SubmitTrapRoundInput,
};

use crate::{
    dtos::{
        ContrastPairProfileDto, ContrastPairSummaryDto, TrapMisconceptionReasonDto, TrapReviewDto,
        TrapRoundResultDto, TrapSessionSnapshotDto,
    },
    error::CommandError,
    state::AppState,
};

pub fn list_traps_pairs(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    topic_ids: Vec<i64>,
) -> Result<Vec<ContrastPairSummaryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let pairs = service.list_traps_pairs(student_id, subject_id, &topic_ids)?;
        Ok(pairs
            .into_iter()
            .map(ContrastPairSummaryDto::from)
            .collect())
    })
}

pub fn get_contrast_pair_profile(
    state: &AppState,
    student_id: i64,
    pair_id: i64,
) -> Result<ContrastPairProfileDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let profile = service.get_contrast_pair_profile(student_id, pair_id)?;
        Ok(ContrastPairProfileDto::from(profile))
    })
}

pub fn list_trap_misconception_reasons(
    state: &AppState,
    mode: Option<String>,
) -> Result<Vec<TrapMisconceptionReasonDto>, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let reasons = service.list_trap_misconception_reasons(mode.as_deref())?;
        Ok(reasons
            .into_iter()
            .map(TrapMisconceptionReasonDto::from)
            .collect())
    })
}

pub fn start_traps_session(
    state: &AppState,
    input: StartTrapsSessionInput,
) -> Result<TrapSessionSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let snapshot = service.start_traps_session(&input)?;
        Ok(TrapSessionSnapshotDto::from(snapshot))
    })
}

pub fn submit_trap_round(
    state: &AppState,
    input: SubmitTrapRoundInput,
) -> Result<TrapRoundResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let result = service.submit_trap_round(&input)?;
        Ok(TrapRoundResultDto::from(result))
    })
}

pub fn record_trap_confusion_reason(
    state: &AppState,
    input: SubmitTrapConfusionReasonInput,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        service.record_trap_confusion_reason(&input)?;
        Ok(())
    })
}

pub fn get_trap_review(state: &AppState, session_id: i64) -> Result<TrapReviewDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let review = service.get_traps_review(session_id)?;
        Ok(TrapReviewDto::from(review))
    })
}
