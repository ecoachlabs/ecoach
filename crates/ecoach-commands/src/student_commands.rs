use ecoach_student_model::{LearnerTruthSnapshot, StudentModelService, StudentTopicState};
use serde::{Deserialize, Serialize};

use crate::{error::CommandError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerTruthDto {
    pub student_id: i64,
    pub student_name: String,
    pub overall_mastery_score: i64,
    pub overall_readiness_band: String,
    pub pending_review_count: i64,
    pub due_memory_count: i64,
    pub topic_count: usize,
    pub skill_count: usize,
    pub memory_count: usize,
    pub diagnosis_count: usize,
}

impl From<LearnerTruthSnapshot> for LearnerTruthDto {
    fn from(v: LearnerTruthSnapshot) -> Self {
        Self {
            student_id: v.student_id,
            student_name: v.student_name,
            overall_mastery_score: v.overall_mastery_score as i64,
            overall_readiness_band: v.overall_readiness_band,
            pending_review_count: v.pending_review_count,
            due_memory_count: v.due_memory_count,
            topic_count: v.topic_summaries.len(),
            skill_count: v.skill_summaries.len(),
            memory_count: v.memory_summaries.len(),
            diagnosis_count: v.recent_diagnoses.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStateDto {
    pub topic_id: i64,
    pub mastery_score: i64,
    pub mastery_state: String,
    pub accuracy_score: i64,
    pub speed_score: i64,
    pub confidence_score: i64,
    pub retention_score: i64,
    pub gap_score: i64,
    pub priority_score: i64,
    pub trend_state: String,
    pub fragility_score: i64,
    pub pressure_collapse_index: i64,
    pub total_attempts: i64,
    pub correct_attempts: i64,
    pub memory_strength: i64,
}

impl From<StudentTopicState> for TopicStateDto {
    fn from(v: StudentTopicState) -> Self {
        Self {
            topic_id: v.topic_id,
            mastery_score: v.mastery_score as i64,
            mastery_state: v.mastery_state.as_str().to_string(),
            accuracy_score: v.accuracy_score as i64,
            speed_score: v.speed_score as i64,
            confidence_score: v.confidence_score as i64,
            retention_score: v.retention_score as i64,
            gap_score: v.gap_score as i64,
            priority_score: v.priority_score as i64,
            trend_state: v.trend_state,
            fragility_score: v.fragility_score as i64,
            pressure_collapse_index: v.pressure_collapse_index as i64,
            total_attempts: v.total_attempts,
            correct_attempts: v.correct_attempts,
            memory_strength: v.memory_strength as i64,
        }
    }
}

pub fn get_learner_truth(
    state: &AppState,
    student_id: i64,
) -> Result<LearnerTruthDto, CommandError> {
    state.with_connection(|conn| {
        let service = StudentModelService::new(conn);
        let snapshot = service.get_learner_truth_snapshot(student_id)?;
        Ok(LearnerTruthDto::from(snapshot))
    })
}

// Topic states are available via the learner truth snapshot's topic_summaries.
// Use get_learner_truth for comprehensive student state data.
