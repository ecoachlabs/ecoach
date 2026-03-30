use ecoach_diagnostics::{DiagnosticEngine, DiagnosticMode};

use crate::{error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRunDto {
    pub diagnostic_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicAnalyticsDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub classification: String,
    pub mastery_score: i64,
    pub confidence_score: i64,
    pub recommended_action: String,
}

pub fn launch_diagnostic(
    state: &AppState,
    student_id: i64,
    subject_id: i64,
    mode: String,
) -> Result<DiagnosticRunDto, CommandError> {
    state.with_connection(|conn| {
        let engine = DiagnosticEngine::new(conn);
        let diagnostic_mode = match mode.as_str() {
            "quick" => DiagnosticMode::Quick,
            "deep" => DiagnosticMode::Deep,
            _ => DiagnosticMode::Standard,
        };
        let diagnostic_id = engine.start_diagnostic(student_id, subject_id, diagnostic_mode)?;
        Ok(DiagnosticRunDto { diagnostic_id })
    })
}

pub fn get_diagnostic_report(
    state: &AppState,
    diagnostic_id: i64,
) -> Result<Vec<TopicAnalyticsDto>, CommandError> {
    state.with_connection(|conn| {
        let engine = DiagnosticEngine::new(conn);
        let analytics = engine.list_topic_analytics(diagnostic_id)?;
        Ok(analytics
            .into_iter()
            .map(|a| TopicAnalyticsDto {
                topic_id: a.topic_id,
                topic_name: a.topic_name,
                classification: a.classification,
                mastery_score: a.mastery_score as i64,
                confidence_score: a.confidence_score as i64,
                recommended_action: a.recommended_action,
            })
            .collect())
    })
}
