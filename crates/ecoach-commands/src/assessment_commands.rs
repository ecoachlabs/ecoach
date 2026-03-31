use ecoach_elite::EliteService;
use ecoach_past_papers::{PastPaperInverseSignal, PastPapersService};
use ecoach_sessions::SessionService;

use crate::{dtos, error::CommandError, state::AppState};

pub type PastPaperInverseSignalDto = PastPaperInverseSignal;
pub type EliteProfileDto = dtos::EliteProfileDto;
pub type EliteTopicProfileDto = dtos::EliteTopicProfileDto;
pub type EliteSessionBlueprintDto = dtos::EliteSessionBlueprintDto;
pub type EliteBlueprintReportDto = dtos::EliteBlueprintReportDto;
pub type PastPaperComebackSignalDto = dtos::PastPaperComebackSignalDto;
pub type SessionRemediationPlanDto = dtos::QuestionRemediationPlanDto;
pub type SessionEvidenceFabricDto = dtos::SessionEvidenceFabricDto;

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
        Ok(EliteService::new(conn)
            .build_session_blueprint(student_id, subject_id)?
            .into())
    })
}

pub fn get_elite_profile(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Option<EliteProfileDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn)
            .get_profile(student_id, subject_id)?
            .map(EliteProfileDto::from))
    })
}

pub fn list_elite_topic_domination(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    limit: usize,
) -> Result<Vec<EliteTopicProfileDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn)
            .list_topic_domination(student_id, subject_id, limit)?
            .into_iter()
            .map(EliteTopicProfileDto::from)
            .collect())
    })
}

pub fn build_elite_session_blueprint_report(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<EliteBlueprintReportDto, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn)
            .build_session_blueprint_report(student_id, subject_id)?
            .into())
    })
}

pub fn list_comeback_candidate_families(
    state: &AppState,
    subject_id: i64,
    topic_id: Option<i64>,
    limit: usize,
) -> Result<Vec<PastPaperComebackSignalDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(PastPapersService::new(conn)
            .list_comeback_candidate_families(subject_id, topic_id, limit)?
            .into_iter()
            .map(PastPaperComebackSignalDto::from)
            .collect())
    })
}

pub fn list_session_remediation_plans(
    state: &AppState,
    session_id: i64,
    limit: usize,
) -> Result<Vec<SessionRemediationPlanDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(SessionService::new(conn)
            .list_session_remediation_plans(session_id, limit)?
            .into_iter()
            .map(SessionRemediationPlanDto::from)
            .collect())
    })
}

pub fn get_session_evidence_fabric(
    state: &AppState,
    session_id: i64,
    limit_events: usize,
) -> Result<Option<SessionEvidenceFabricDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(SessionService::new(conn)
            .get_session_evidence_fabric(session_id, limit_events)?
            .map(SessionEvidenceFabricDto::from))
    })
}

// ── Elite Mode deep commands ──

pub fn score_elite_session(
    state: &AppState,
    student_id: i64,
    session_id: i64,
    session_class: &str,
) -> Result<ecoach_elite::EliteSessionScore, CommandError> {
    state.with_connection(|conn| {
        let service = EliteService::new(conn);
        let score = service.score_session(student_id, session_id, session_class)?;
        Ok(score)
    })
}

pub fn list_elite_personal_bests(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<(String, i64, String)>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn).list_personal_bests(student_id, subject_id)?)
    })
}

pub fn check_elite_badges(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<String>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn).check_and_award_badges(student_id, subject_id)?)
    })
}

pub fn list_elite_earned_badges(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<Vec<(String, String, String)>, CommandError> {
    state.with_connection(|conn| {
        Ok(EliteService::new(conn).list_earned_elite_badges(student_id, subject_id)?)
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_identity::CreateAccountInput;
    use ecoach_questions::QuestionService;
    use ecoach_sessions::{PracticeSessionStartInput, SessionAnswerInput, SessionService};
    use ecoach_substrate::{AccountType, EntitlementTier};

    use crate::{identity_commands, state::AppState};

    use super::*;

    #[test]
    fn assessment_commands_surface_comeback_candidates_and_session_evidence() {
        let state = AppState::in_memory().expect("in-memory command state should build");
        state
            .with_connection(|conn| {
                PackService::new(conn).install_pack(&sample_pack_path())?;
                Ok(())
            })
            .expect("sample pack should install");

        let account = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Afia".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("account should create");

        let (subject_id, _topic_id, session_id) = state
            .with_connection(|conn| {
                let question_id: i64 = conn.query_row(
                    "SELECT id FROM questions WHERE family_id IS NOT NULL ORDER BY id ASC LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .map_err(storage_error)?;
                let subject_id: i64 = conn
                    .query_row(
                        "SELECT subject_id FROM questions WHERE id = ?1",
                        [question_id],
                        |row| row.get(0),
                    )
                    .map_err(storage_error)?;
                let topic_id: i64 = conn
                    .query_row(
                        "SELECT COALESCE(qf.topic_id, q.topic_id)
                         FROM questions q
                         LEFT JOIN question_families qf ON qf.id = q.family_id
                         WHERE q.id = ?1",
                        [question_id],
                        |row| row.get(0),
                    )
                    .map_err(storage_error)?;

                let papers = PastPapersService::new(conn);
                let early_paper = papers.create_paper_set(subject_id, 2021, "Past Paper 2021")?;
                let late_paper = papers.create_paper_set(subject_id, 2023, "Past Paper 2023")?;
                papers.link_question_to_paper(early_paper, question_id, None, Some("1"))?;
                papers.link_question_to_paper(late_paper, question_id, None, Some("2"))?;
                papers.recompute_family_analytics(subject_id)?;

                let sessions = SessionService::new(conn);
                let (session, _) = sessions.start_practice_session(&PracticeSessionStartInput {
                    student_id: account.id,
                    subject_id,
                    topic_ids: vec![topic_id],
                    question_count: 2,
                    is_timed: true,
                })?;
                let snapshot = sessions
                    .get_session_snapshot(session.id)?
                    .expect("session snapshot should exist");
                let options =
                    QuestionService::new(conn).list_options(snapshot.items[0].question_id)?;
                let misconception_option = options
                    .iter()
                    .find(|option| option.misconception_id.is_some())
                    .or_else(|| options.iter().find(|option| !option.is_correct))
                    .expect("answer option should exist");
                sessions.record_answer(
                    session.id,
                    &SessionAnswerInput {
                        item_id: snapshot.items[0].id,
                        selected_option_id: misconception_option.id,
                        response_time_ms: Some(48_000),
                    },
                )?;
                sessions.complete_session(session.id)?;

                Ok::<(i64, i64, i64), crate::CommandError>((subject_id, topic_id, session.id))
            })
            .expect("seed data should prepare");

        let comeback = list_comeback_candidate_families(&state, subject_id, None, 5)
            .expect("comeback candidates should load");
        let remediation = list_session_remediation_plans(&state, session_id, 3)
            .expect("session remediation plans should load");
        let fabric = get_session_evidence_fabric(&state, session_id, 10)
            .expect("session evidence fabric should load")
            .expect("session evidence fabric should exist");

        assert!(!comeback.is_empty());
        assert!(comeback[0].comeback_score >= 0);
        assert!(!remediation.is_empty());
        assert_eq!(fabric.session_id, session_id);
        assert!(!fabric.remediation_plans.is_empty());
        assert!(
            fabric
                .evidence_records
                .iter()
                .any(|record| record.event_type == "session.remediation_planned")
        );
    }

    fn sample_pack_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate directory should have workspace parent")
            .parent()
            .expect("workspace root should exist")
            .join("packs")
            .join("math-bece-sample")
    }

    fn storage_error(err: impl ToString) -> crate::CommandError {
        crate::CommandError {
            code: "storage_error".to_string(),
            message: err.to_string(),
        }
    }
}
