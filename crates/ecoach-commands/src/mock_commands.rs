use ecoach_mock_centre::{CompileMockInput, MockCentreService, SubmitMockAnswerInput};

use crate::{error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSessionDto {
    pub id: i64,
    pub subject_id: i64,
    pub status: String,
    pub duration_minutes: i64,
    pub question_count: i64,
    pub answered_count: i64,
    pub time_remaining_seconds: Option<i64>,
    pub paper_year: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockAnswerResultDto {
    pub question_id: i64,
    pub was_correct: bool,
    pub answered_count: i64,
    pub remaining_count: i64,
    pub time_remaining_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockReportDto {
    pub mock_session_id: i64,
    pub grade: String,
    pub percentage: f64,
    pub accuracy_bp: i64,
    pub total_score: i64,
    pub max_score: i64,
    pub time_used_seconds: i64,
    pub questions_answered: i64,
    pub questions_unanswered: i64,
    pub topic_count: usize,
    pub improvement_direction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSessionSummaryDto {
    pub id: i64,
    pub subject_id: i64,
    pub grade: Option<String>,
    pub percentage: Option<f64>,
    pub status: String,
    pub paper_year: Option<String>,
}

pub fn compile_mock(
    state: &AppState,
    input: CompileMockInput,
) -> Result<MockSessionDto, CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);
        let session = service.compile_mock(&input)?;
        Ok(MockSessionDto {
            id: session.id,
            subject_id: session.subject_id,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
        })
    })
}

pub fn start_mock(state: &AppState, mock_session_id: i64) -> Result<MockSessionDto, CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);
        let session = service.start_mock(mock_session_id)?;
        Ok(MockSessionDto {
            id: session.id,
            subject_id: session.subject_id,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
        })
    })
}

pub fn submit_mock_answer(
    state: &AppState,
    input: SubmitMockAnswerInput,
) -> Result<MockAnswerResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);
        let result = service.submit_answer(&input)?;
        Ok(MockAnswerResultDto {
            question_id: result.question_id,
            was_correct: result.was_correct,
            answered_count: result.answered_count,
            remaining_count: result.remaining_count,
            time_remaining_seconds: result.time_remaining_seconds,
        })
    })
}

pub fn get_mock_report(
    state: &AppState,
    mock_session_id: i64,
) -> Result<MockReportDto, CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);
        let report = service.get_report(mock_session_id)?;
        Ok(MockReportDto {
            mock_session_id: report.mock_session_id,
            grade: report.grade,
            percentage: report.percentage,
            accuracy_bp: report.accuracy_bp as i64,
            total_score: report.total_score,
            max_score: report.max_score,
            time_used_seconds: report.time_used_seconds,
            questions_answered: report.questions_answered,
            questions_unanswered: report.questions_unanswered,
            topic_count: report.topic_breakdown.len(),
            improvement_direction: report.improvement_vs_last.map(|d| d.direction),
        })
    })
}

pub fn pause_mock(state: &AppState, mock_session_id: i64) -> Result<MockSessionDto, CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);
        let session = service.pause_mock(mock_session_id)?;
        Ok(MockSessionDto {
            id: session.id,
            subject_id: session.subject_id,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
        })
    })
}

pub fn resume_mock(state: &AppState, mock_session_id: i64) -> Result<MockSessionDto, CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);
        let session = service.resume_mock(mock_session_id)?;
        Ok(MockSessionDto {
            id: session.id,
            subject_id: session.subject_id,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
        })
    })
}

pub fn list_mock_sessions(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<MockSessionSummaryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);
        let sessions = service.list_mock_sessions(student_id, limit)?;
        Ok(sessions
            .into_iter()
            .map(|s| MockSessionSummaryDto {
                id: s.id,
                subject_id: s.subject_id,
                grade: s.grade,
                percentage: s.percentage,
                status: s.status,
                paper_year: s.paper_year,
            })
            .collect())
    })
}

pub fn abandon_mock(state: &AppState, mock_session_id: i64) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);
        service.abandon_mock(mock_session_id)?;
        Ok(())
    })
}
