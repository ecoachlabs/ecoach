use ecoach_forecast::ForecastEngine;
use ecoach_mock_centre::{
    CompileMockInput, MockCentreService, MockDiagnosisEngine, SubmitMockAnswerInput,
};

use crate::{error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSessionDto {
    pub id: i64,
    pub subject_id: i64,
    pub mock_type: String,
    pub status: String,
    pub duration_minutes: i64,
    pub question_count: i64,
    pub answered_count: i64,
    pub time_remaining_seconds: Option<i64>,
    pub paper_year: Option<String>,
    pub blueprint_id: Option<i64>,
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
    pub mock_type: String,
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
            mock_type: session.mock_type,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
            blueprint_id: session.blueprint_id,
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
            mock_type: session.mock_type,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
            blueprint_id: session.blueprint_id,
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
            mock_type: session.mock_type,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
            blueprint_id: session.blueprint_id,
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
            mock_type: session.mock_type,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
            blueprint_id: session.blueprint_id,
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
                mock_type: s.mock_type,
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

// ── start_first_mock: auto-compile a diagnostic mock for first-time users ──

pub fn start_first_mock(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<MockSessionDto, CommandError> {
    state.with_connection(|conn| {
        let service = MockCentreService::new(conn);

        // Check if student has any completed mocks for this subject
        let existing = service.list_mock_sessions(student_id, 1)?;
        if !existing.is_empty() {
            return Err(ecoach_substrate::EcoachError::Validation(
                "student already has mock history; use compile_mock instead".into(),
            )
            .into());
        }

        // Auto-compile a diagnostic mock with sensible defaults
        let input = CompileMockInput {
            student_id,
            subject_id,
            duration_minutes: 60,
            question_count: 20,
            topic_ids: Vec::new(), // all topics
            paper_year: None,
            mock_type: Some("diagnostic".into()),
            blueprint_id: None,
        };

        let session = service.compile_mock(&input)?;
        Ok(MockSessionDto {
            id: session.id,
            subject_id: session.subject_id,
            mock_type: session.mock_type,
            status: session.status,
            duration_minutes: session.duration_minutes,
            question_count: session.question_count,
            answered_count: session.answered_count,
            time_remaining_seconds: session.time_remaining_seconds,
            paper_year: session.paper_year,
            blueprint_id: session.blueprint_id,
        })
    })
}

// ── Phase 5: New commands ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockQuestionReviewDto {
    pub question_id: i64,
    pub stem: String,
    pub selected_option_text: Option<String>,
    pub correct_option_text: Option<String>,
    pub explanation: Option<String>,
    pub was_correct: bool,
    pub topic_name: String,
    pub response_time_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockCentreSnapshotDto {
    pub student_id: i64,
    pub subject_id: i64,
    pub available_mock_types: Vec<String>,
    pub total_mocks_completed: i64,
    pub latest_grade: Option<String>,
    pub latest_percentage: Option<f64>,
    pub has_forecast_blueprint: bool,
    pub recommended_mock_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockDeepDiagnosisDto {
    pub mock_session_id: i64,
    pub weakness_count: usize,
    pub strength_count: usize,
    pub broken_link_count: usize,
    pub misconception_hit_count: usize,
    pub predicted_score_bp: Option<i64>,
    pub pacing_label: String,
    pub action_count: usize,
}

pub fn flag_mock_question(
    state: &AppState,
    mock_session_id: i64,
    question_id: i64,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        // Get session_id from mock_session
        let session_id: i64 = conn
            .query_row(
                "SELECT session_id FROM mock_sessions WHERE id = ?1",
                [mock_session_id],
                |row| row.get(0),
            )
            .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;

        conn.execute(
            "UPDATE session_items SET flagged = 1, updated_at = datetime('now')
             WHERE session_id = ?1 AND question_id = ?2",
            rusqlite::params![session_id, question_id],
        )
        .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;

        Ok(())
    })
}

pub fn get_mock_question_review(
    state: &AppState,
    mock_session_id: i64,
    question_id: i64,
) -> Result<MockQuestionReviewDto, CommandError> {
    state.with_connection(|conn| {
        let session_id: i64 = conn
            .query_row(
                "SELECT session_id FROM mock_sessions WHERE id = ?1",
                [mock_session_id],
                |row| row.get(0),
            )
            .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;

        let dto = conn
            .query_row(
                "SELECT q.id, q.stem, q.explanation_text,
                        si.is_correct, si.response_time_ms,
                        COALESCE(t.name, 'Unknown'),
                        (SELECT option_text FROM question_options WHERE id = si.selected_option_id),
                        (SELECT option_text FROM question_options WHERE question_id = q.id AND is_correct = 1 LIMIT 1)
                 FROM session_items si
                 INNER JOIN questions q ON q.id = si.question_id
                 LEFT JOIN topics t ON t.id = si.source_topic_id
                 WHERE si.session_id = ?1 AND si.question_id = ?2",
                rusqlite::params![session_id, question_id],
                |row| {
                    Ok(MockQuestionReviewDto {
                        question_id: row.get(0)?,
                        stem: row.get(1)?,
                        explanation: row.get(2)?,
                        was_correct: row.get::<_, i64>(3)? == 1,
                        response_time_ms: row.get(4)?,
                        topic_name: row.get(5)?,
                        selected_option_text: row.get(6)?,
                        correct_option_text: row.get(7)?,
                    })
                },
            )
            .map_err(|e| {
                ecoach_substrate::EcoachError::NotFound(format!(
                    "question {} not found in mock {}: {}",
                    question_id, mock_session_id, e
                ))
            })?;

        Ok(dto)
    })
}

pub fn get_mock_centre_snapshot(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
) -> Result<MockCentreSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let (total_completed, latest_grade, latest_percentage): (i64, Option<String>, Option<f64>) = conn
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM mock_sessions WHERE student_id = ?1 AND subject_id = ?2 AND status = 'completed'),
                    (SELECT grade FROM mock_sessions WHERE student_id = ?1 AND subject_id = ?2 AND status = 'completed' ORDER BY completed_at DESC LIMIT 1),
                    (SELECT percentage FROM mock_sessions WHERE student_id = ?1 AND subject_id = ?2 AND status = 'completed' ORDER BY completed_at DESC LIMIT 1)",
                rusqlite::params![student_id, subject_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| ecoach_substrate::EcoachError::Storage(e.to_string()))?;

        let engine = ForecastEngine::new(conn);
        let has_blueprint = engine.get_latest_blueprint(subject_id)?.is_some();

        let recommended = if has_blueprint {
            Some("forecast".into())
        } else if total_completed == 0 {
            Some("diagnostic".into())
        } else {
            Some("forecast".into())
        };

        Ok(MockCentreSnapshotDto {
            student_id,
            subject_id,
            available_mock_types: vec![
                "forecast".into(),
                "diagnostic".into(),
                "remediation".into(),
                "final_exam".into(),
                "shock".into(),
                "wisdom".into(),
            ],
            total_mocks_completed: total_completed,
            latest_grade,
            latest_percentage,
            has_forecast_blueprint: has_blueprint,
            recommended_mock_type: recommended,
        })
    })
}

pub fn get_deep_diagnosis(
    state: &AppState,
    mock_session_id: i64,
) -> Result<MockDeepDiagnosisDto, CommandError> {
    state.with_connection(|conn| {
        let engine = MockDiagnosisEngine::new(conn);
        let diagnosis = engine.diagnose(mock_session_id)?;

        Ok(MockDeepDiagnosisDto {
            mock_session_id,
            weakness_count: diagnosis.topic_weaknesses.len(),
            strength_count: diagnosis.topic_strengths.len(),
            broken_link_count: diagnosis.broken_links.len(),
            misconception_hit_count: diagnosis.misconception_hits.len(),
            predicted_score_bp: diagnosis
                .predicted_exam_score
                .map(|p| p.predicted_score_bp as i64),
            pacing_label: diagnosis.timing_diagnosis.pacing_label,
            action_count: diagnosis.action_plan.len(),
        })
    })
}
