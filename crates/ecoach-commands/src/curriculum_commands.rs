use ecoach_curriculum::{CurriculumService, Subject, TopicSummary};
use serde::{Deserialize, Serialize};

use crate::{error::CommandError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectDto {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub code: String,
    pub name: String,
    pub display_order: i64,
}

impl From<Subject> for SubjectDto {
    fn from(v: Subject) -> Self {
        Self {
            id: v.id,
            curriculum_version_id: v.curriculum_version_id,
            code: v.code,
            name: v.name,
            display_order: v.display_order,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicDto {
    pub id: i64,
    pub subject_id: i64,
    pub parent_topic_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    pub node_type: String,
    pub display_order: i64,
}

impl From<TopicSummary> for TopicDto {
    fn from(v: TopicSummary) -> Self {
        Self {
            id: v.id,
            subject_id: v.subject_id,
            parent_topic_id: v.parent_topic_id,
            code: v.code,
            name: v.name,
            node_type: v.node_type,
            display_order: v.display_order,
        }
    }
}

pub fn list_subjects(
    state: &AppState,
    curriculum_version_id: i64,
) -> Result<Vec<SubjectDto>, CommandError> {
    state.with_connection(|conn| {
        let service = CurriculumService::new(conn);
        let subjects = service.get_subjects(curriculum_version_id)?;
        Ok(subjects.into_iter().map(SubjectDto::from).collect())
    })
}

pub fn list_topics(state: &AppState, subject_id: i64) -> Result<Vec<TopicDto>, CommandError> {
    state.with_connection(|conn| {
        let service = CurriculumService::new(conn);
        let topics = service.list_topics_for_subject(subject_id)?;
        Ok(topics.into_iter().map(TopicDto::from).collect())
    })
}
