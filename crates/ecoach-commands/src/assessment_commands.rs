use ecoach_elite::{EliteService, EliteSessionBlueprint};
use ecoach_past_papers::{PastPaperInverseSignal, PastPapersService};

use crate::{error::CommandError, state::AppState};

pub type PastPaperInverseSignalDto = PastPaperInverseSignal;
pub type EliteSessionBlueprintDto = EliteSessionBlueprint;

pub fn list_inverse_pressure_families(
    state: &AppState,
    subject_id: i64,
    topic_id: Option<i64>,
    limit: usize,
) -> Result<Vec<PastPaperInverseSignalDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(PastPapersService::new(conn)
            .list_inverse_pressure_families(subject_id, topic_id, limit)?)
    })
}

pub fn build_elite_session_blueprint(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<EliteSessionBlueprintDto, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn).build_session_blueprint(student_id, subject_id)?)
    })
}
