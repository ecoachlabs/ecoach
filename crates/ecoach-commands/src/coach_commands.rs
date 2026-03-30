use ecoach_coach_brain::{
    CoachNextAction, CoachStateResolution, ContentReadinessResolution, TopicCase,
    assess_content_readiness, list_priority_topic_cases, resolve_coach_state,
    resolve_next_coach_action,
};
use ecoach_reporting::{DashboardService, StudentDashboard};
use serde::{Deserialize, Serialize};

use crate::{error::CommandError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachStateDto {
    pub state: String,
    pub reason: String,
}

impl From<CoachStateResolution> for CoachStateDto {
    fn from(v: CoachStateResolution) -> Self {
        Self {
            state: format!("{:?}", v.state),
            reason: v.reason.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachNextActionDto {
    pub state: String,
    pub action_type: String,
    pub title: String,
    pub subtitle: String,
    pub estimated_minutes: Option<i64>,
    pub route: String,
}

impl From<CoachNextAction> for CoachNextActionDto {
    fn from(v: CoachNextAction) -> Self {
        Self {
            state: format!("{:?}", v.state),
            action_type: format!("{:?}", v.action_type),
            title: v.title,
            subtitle: v.subtitle,
            estimated_minutes: v.estimated_minutes,
            route: v.route,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentReadinessDto {
    pub status: String,
    pub subject_codes: Vec<String>,
    pub active_pack_count: i64,
    pub topic_count: i64,
    pub question_count: i64,
    pub reason: Option<String>,
}

impl From<ContentReadinessResolution> for ContentReadinessDto {
    fn from(v: ContentReadinessResolution) -> Self {
        Self {
            status: format!("{:?}", v.status),
            subject_codes: v.subject_codes,
            active_pack_count: v.active_pack_count,
            topic_count: v.topic_count,
            question_count: v.question_count,
            reason: v.reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCaseDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub subject_code: String,
    pub priority_score: i64,
    pub mastery_score: i64,
    pub mastery_state: String,
    pub gap_score: i64,
    pub fragility_score: i64,
    pub memory_strength: i64,
    pub decay_risk: i64,
    pub evidence_count: i64,
    pub requires_probe: bool,
    pub intervention_mode: String,
    pub intervention_urgency: String,
    pub intervention_reason: String,
}

impl From<TopicCase> for TopicCaseDto {
    fn from(v: TopicCase) -> Self {
        Self {
            topic_id: v.topic_id,
            topic_name: v.topic_name,
            subject_code: v.subject_code,
            priority_score: v.priority_score as i64,
            mastery_score: v.mastery_score as i64,
            mastery_state: v.mastery_state,
            gap_score: v.gap_score as i64,
            fragility_score: v.fragility_score as i64,
            memory_strength: v.memory_strength as i64,
            decay_risk: v.decay_risk as i64,
            evidence_count: v.evidence_count,
            requires_probe: v.requires_probe,
            intervention_mode: v.recommended_intervention.mode,
            intervention_urgency: v.recommended_intervention.urgency,
            intervention_reason: v.recommended_intervention.reason,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentDashboardDto {
    pub student_name: String,
    pub exam_target: Option<String>,
    pub overall_readiness_band: String,
    pub subjects: Vec<SubjectSummaryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectSummaryDto {
    pub subject_id: i64,
    pub subject_name: String,
    pub readiness_band: String,
    pub mastered_topic_count: usize,
    pub weak_topic_count: usize,
    pub total_topic_count: usize,
}

impl From<StudentDashboard> for StudentDashboardDto {
    fn from(v: StudentDashboard) -> Self {
        Self {
            student_name: v.student_name,
            exam_target: v.exam_target,
            overall_readiness_band: v.overall_readiness_band,
            subjects: v
                .subject_summaries
                .into_iter()
                .map(|s| SubjectSummaryDto {
                    subject_id: s.subject_id,
                    subject_name: s.subject_name,
                    readiness_band: s.readiness_band,
                    mastered_topic_count: s.mastered_topic_count,
                    weak_topic_count: s.weak_topic_count,
                    total_topic_count: s.total_topic_count,
                })
                .collect(),
        }
    }
}

pub fn get_coach_state(state: &AppState, student_id: i64) -> Result<CoachStateDto, CommandError> {
    state.with_connection(|conn| {
        let resolution = resolve_coach_state(conn, student_id)?;
        Ok(CoachStateDto::from(resolution))
    })
}

pub fn get_coach_next_action(
    state: &AppState,
    student_id: i64,
) -> Result<CoachNextActionDto, CommandError> {
    state.with_connection(|conn| {
        let action = resolve_next_coach_action(conn, student_id)?;
        Ok(CoachNextActionDto::from(action))
    })
}

pub fn get_content_readiness(
    state: &AppState,
    student_id: i64,
) -> Result<ContentReadinessDto, CommandError> {
    state.with_connection(|conn| {
        let readiness = assess_content_readiness(conn, student_id)?;
        Ok(ContentReadinessDto::from(readiness))
    })
}

pub fn get_priority_topics(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<TopicCaseDto>, CommandError> {
    state.with_connection(|conn| {
        let cases = list_priority_topic_cases(conn, student_id, limit)?;
        Ok(cases.into_iter().map(TopicCaseDto::from).collect())
    })
}

pub fn get_student_dashboard(
    state: &AppState,
    student_id: i64,
) -> Result<StudentDashboardDto, CommandError> {
    state.with_connection(|conn| {
        let service = DashboardService::new(conn);
        let dashboard = service.get_student_dashboard(student_id)?;
        Ok(StudentDashboardDto::from(dashboard))
    })
}
