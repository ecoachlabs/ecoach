use ecoach_glossary::{
    GlossaryAudioProgram, GlossaryService, KnowledgeBundle, QuestionKnowledgeLink,
};
use ecoach_library::{
    ContinueLearningCard, LibraryHomeSnapshot, LibraryService, RevisionPackItem,
    RevisionPackSummary, TeachActionPlan, TopicRelationshipHint,
};

use crate::{error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryShelfDto {
    pub shelf_type: String,
    pub title: String,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemDto {
    pub id: i64,
    pub item_type: String,
    pub title: String,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntryDto {
    pub id: i64,
    pub title: String,
    pub entry_type: String,
    pub short_text: Option<String>,
    pub topic_id: Option<i64>,
}

pub type LibraryHomeSnapshotDto = LibraryHomeSnapshot;
pub type ContinueLearningCardDto = ContinueLearningCard;
pub type RevisionPackSummaryDto = RevisionPackSummary;
pub type RevisionPackItemDto = RevisionPackItem;
pub type KnowledgeBundleDto = KnowledgeBundle;
pub type QuestionKnowledgeLinkDto = QuestionKnowledgeLink;
pub type GlossaryAudioProgramDto = GlossaryAudioProgram;
pub type TeachActionPlanDto = TeachActionPlan;
pub type TopicRelationshipHintDto = TopicRelationshipHint;

pub fn get_library_home(
    state: &AppState,
    student_id: i64,
) -> Result<Vec<LibraryShelfDto>, CommandError> {
    state.with_connection(|conn| {
        let service = LibraryService::new(conn);
        let snapshot = service.build_home_snapshot(student_id, 20)?;
        Ok(snapshot
            .generated_shelves
            .into_iter()
            .map(|shelf| LibraryShelfDto {
                shelf_type: shelf.shelf_type,
                title: shelf.title,
                item_count: shelf.items.len(),
            })
            .collect())
    })
}

pub fn save_library_item(
    state: &AppState,
    student_id: i64,
    item_type: String,
    reference_id: i64,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        let service = LibraryService::new(conn);
        let id = service.save_item(student_id, &item_type, reference_id)?;
        Ok(id)
    })
}

pub fn search_glossary(
    state: &AppState,
    query: String,
) -> Result<Vec<GlossaryEntryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = GlossaryService::new(conn);
        let entries = service.search_entries(&query)?;
        Ok(entries
            .into_iter()
            .map(|e| GlossaryEntryDto {
                id: e.id,
                title: e.title,
                entry_type: e.entry_type,
                short_text: e.short_text,
                topic_id: e.topic_id,
            })
            .collect())
    })
}

pub fn get_library_snapshot(
    state: &AppState,
    student_id: i64,
) -> Result<LibraryHomeSnapshotDto, CommandError> {
    state.with_connection(|conn| Ok(LibraryService::new(conn).build_home_snapshot(student_id, 20)?))
}

pub fn get_continue_learning_card(
    state: &AppState,
    student_id: i64,
) -> Result<Option<ContinueLearningCardDto>, CommandError> {
    state.with_connection(|conn| {
        let snapshot = LibraryService::new(conn).build_home_snapshot(student_id, 20)?;
        Ok(snapshot.continue_card)
    })
}

pub fn build_revision_pack(
    state: &AppState,
    student_id: i64,
    title: String,
    question_limit: usize,
) -> Result<RevisionPackSummaryDto, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).build_revision_pack(student_id, &title, question_limit)?)
    })
}

pub fn list_revision_pack_items(
    state: &AppState,
    pack_id: i64,
) -> Result<Vec<RevisionPackItemDto>, CommandError> {
    state.with_connection(|conn| Ok(LibraryService::new(conn).list_revision_pack_items(pack_id)?))
}

pub fn list_glossary_bundles_for_topic(
    state: &AppState,
    topic_id: i64,
) -> Result<Vec<KnowledgeBundleDto>, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).list_bundles_for_topic(topic_id)?))
}

pub fn list_glossary_entries_for_question(
    state: &AppState,
    question_id: i64,
) -> Result<Vec<QuestionKnowledgeLinkDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).list_entries_for_question(question_id)?)
    })
}

pub fn build_glossary_audio_program_for_topic(
    state: &AppState,
    topic_id: i64,
    limit: usize,
) -> Result<GlossaryAudioProgramDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).build_audio_program_for_topic(topic_id, limit)?)
    })
}

pub fn build_glossary_audio_program_for_question(
    state: &AppState,
    question_id: i64,
    limit: usize,
) -> Result<GlossaryAudioProgramDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).build_audio_program_for_question(question_id, limit)?)
    })
}

pub fn build_teach_action_plan(
    state: &AppState,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> Result<TeachActionPlanDto, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).build_teach_action_plan(student_id, topic_id, limit)?)
    })
}

pub fn list_topic_relationship_hints(
    state: &AppState,
    topic_id: i64,
    limit: usize,
) -> Result<Vec<TopicRelationshipHintDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_topic_relationship_hints(topic_id, limit)?)
    })
}
