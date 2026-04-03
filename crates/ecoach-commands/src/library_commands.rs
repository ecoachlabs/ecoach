use ecoach_glossary::{
    ConceptMapView, FormulaLabView, GlossaryAudioProgram, GlossaryAudioQueueSnapshot,
    GlossaryComparisonView, GlossaryEntryDetail, GlossaryHomeSnapshot, GlossarySearchGroup,
    GlossarySearchInput, GlossarySearchResponse, GlossarySearchResult, GlossarySearchSuggestion,
    GlossaryService, GlossaryTestAttemptResult, GlossaryTestSessionDetail, KnowledgeBundle,
    QuestionKnowledgeLink, StartGlossaryAudioQueueInput, SubmitGlossaryTestAttemptInput,
    UpdateGlossaryAudioQueueInput,
};
use ecoach_library::{
    AddLibraryNoteInput, AddShelfItemInput, BuildRevisionPackFromTemplateInput,
    ContinueLearningCard, CreateCustomShelfInput, CustomLibraryShelf, ExamHotspot,
    LibraryHomeSnapshot, LibraryItem, LibraryItemAction, LibraryItemStateHistoryEntry, LibraryNote,
    LibrarySearchInput, LibraryService, LibraryTagDefinition, OfflineLibraryItem,
    RecordLibraryItemActionInput, RevisionPackItem, RevisionPackSummary, RevisionPackTemplate,
    SaveLibraryItemInput, TeachActionPlan, TeachExplanation, TeachExplanationUpsertInput,
    TeachLesson, TeachMicroCheck, TeachMicroCheckInput, TopicLibrarySnapshot,
    TopicRelationshipHint, TutorInteraction, TutorInteractionInput, TutorResponse,
    UpdateLibraryItemInput,
};

use crate::{
    dtos::{KnowledgeBundleSequenceItemDto, PersonalizedLearningPathDto},
    error::CommandError,
    state::AppState,
};
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
pub struct LibrarySearchResultDto {
    pub item_type: String,
    pub item_ref_id: Option<i64>,
    pub library_item_id: Option<i64>,
    pub title: String,
    pub subtitle: Option<String>,
    pub state: Option<String>,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub subject_id: Option<i64>,
    pub subject_name: Option<String>,
    pub tags: Vec<String>,
    pub reason: String,
    pub match_score: u16,
    pub metadata: serde_json::Value,
}

pub type LibraryItemRecordDto = LibraryItem;
pub type LibraryItemActionDto = LibraryItemAction;
pub type LibraryItemStateHistoryEntryDto = LibraryItemStateHistoryEntry;
pub type LibraryNoteDto = LibraryNote;
pub type LibrarySearchInputDto = LibrarySearchInput;
pub type LibraryTagDefinitionDto = LibraryTagDefinition;
pub type RevisionPackTemplateDto = RevisionPackTemplate;
pub type UpdateLibraryItemInputDto = UpdateLibraryItemInput;
pub type SaveLibraryItemInputDto = SaveLibraryItemInput;
pub type RecordLibraryItemActionInputDto = RecordLibraryItemActionInput;
pub type AddLibraryNoteInputDto = AddLibraryNoteInput;
pub type BuildRevisionPackFromTemplateInputDto = BuildRevisionPackFromTemplateInput;
pub type CustomLibraryShelfDto = CustomLibraryShelf;
pub type AddShelfItemInputDto = AddShelfItemInput;
pub type CreateCustomShelfInputDto = CreateCustomShelfInput;
pub type OfflineLibraryItemDto = OfflineLibraryItem;
pub type TopicLibrarySnapshotDto = TopicLibrarySnapshot;
pub type ExamHotspotDto = ExamHotspot;
pub type GlossarySearchInputDto = GlossarySearchInput;
pub type GlossarySearchResponseDto = GlossarySearchResponse;
pub type GlossarySearchGroupDto = GlossarySearchGroup;
pub type GlossarySearchResultDto = GlossarySearchResult;
pub type GlossarySearchSuggestionDto = GlossarySearchSuggestion;
pub type GlossaryEntryDetailDto = GlossaryEntryDetail;
pub type GlossaryHomeSnapshotDto = GlossaryHomeSnapshot;
pub type GlossaryComparisonViewDto = GlossaryComparisonView;
pub type FormulaLabViewDto = FormulaLabView;
pub type ConceptMapViewDto = ConceptMapView;
pub type GlossaryAudioQueueSnapshotDto = GlossaryAudioQueueSnapshot;
pub type StartGlossaryAudioQueueInputDto = StartGlossaryAudioQueueInput;
pub type UpdateGlossaryAudioQueueInputDto = UpdateGlossaryAudioQueueInput;
pub type GlossaryTestSessionDetailDto = GlossaryTestSessionDetail;
pub type GlossaryTestAttemptResultDto = GlossaryTestAttemptResult;
pub type CreateGlossaryTestInputDto = ecoach_glossary::CreateGlossaryTestInput;
pub type SubmitGlossaryTestAttemptInputDto = SubmitGlossaryTestAttemptInput;
pub type GlossaryInteractionInputDto = ecoach_glossary::GlossaryInteractionInput;

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
pub type TeachExplanationDto = TeachExplanation;
pub type TeachMicroCheckDto = TeachMicroCheck;
pub type TeachLessonDto = TeachLesson;
pub type TopicRelationshipHintDto = TopicRelationshipHint;
pub type TutorInteractionDto = TutorInteraction;
pub type TutorResponseDto = TutorResponse;

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

pub fn save_library_item_with_metadata(
    state: &AppState,
    student_id: i64,
    input: SaveLibraryItemInputDto,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).save_item_with_metadata(student_id, &input)?)
    })
}

pub fn update_library_item(
    state: &AppState,
    library_item_id: i64,
    input: UpdateLibraryItemInputDto,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).update_item_details(library_item_id, &input, None)?)
    })
}

pub fn remove_library_item(state: &AppState, library_item_id: i64) -> Result<(), CommandError> {
    state.with_connection(|conn| Ok(LibraryService::new(conn).remove_item(library_item_id)?))
}

pub fn list_library_items(
    state: &AppState,
    student_id: i64,
) -> Result<Vec<LibraryItemRecordDto>, CommandError> {
    state.with_connection(|conn| Ok(LibraryService::new(conn).list_items(student_id)?))
}

pub fn list_library_item_state_history(
    state: &AppState,
    library_item_id: i64,
    limit: usize,
) -> Result<Vec<LibraryItemStateHistoryEntryDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_item_state_history(library_item_id, limit)?)
    })
}

pub fn record_library_item_action(
    state: &AppState,
    student_id: i64,
    library_item_id: i64,
    input: RecordLibraryItemActionInputDto,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).record_item_action(student_id, library_item_id, &input)?)
    })
}

pub fn list_library_item_actions(
    state: &AppState,
    library_item_id: i64,
    limit: usize,
) -> Result<Vec<LibraryItemActionDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_item_actions(library_item_id, limit)?)
    })
}

pub fn add_library_note(
    state: &AppState,
    input: AddLibraryNoteInputDto,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| Ok(LibraryService::new(conn).add_library_note(&input)?))
}

pub fn list_library_notes(
    state: &AppState,
    student_id: i64,
    topic_id: Option<i64>,
    library_item_id: Option<i64>,
    limit: usize,
) -> Result<Vec<LibraryNoteDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_library_notes(
            student_id,
            topic_id,
            library_item_id,
            limit,
        )?)
    })
}

pub fn search_library(
    state: &AppState,
    student_id: i64,
    input: LibrarySearchInputDto,
    limit: usize,
) -> Result<Vec<LibrarySearchResultDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn)
            .search_library(student_id, &input, limit)?
            .into_iter()
            .map(|item| LibrarySearchResultDto {
                item_type: item.item_type,
                item_ref_id: item.item_ref_id,
                library_item_id: item.library_item_id,
                title: item.title,
                subtitle: item.subtitle,
                state: item.state,
                topic_id: item.topic_id,
                topic_name: item.topic_name,
                subject_id: item.subject_id,
                subject_name: item.subject_name,
                tags: item.tags,
                reason: item.reason,
                match_score: item.match_score,
                metadata: item.metadata,
            })
            .collect())
    })
}

pub fn list_revision_pack_templates(
    state: &AppState,
) -> Result<Vec<RevisionPackTemplateDto>, CommandError> {
    state.with_connection(|conn| Ok(LibraryService::new(conn).list_revision_pack_templates()?))
}

pub fn build_revision_pack_from_template(
    state: &AppState,
    student_id: i64,
    input: BuildRevisionPackFromTemplateInputDto,
) -> Result<RevisionPackSummaryDto, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).build_revision_pack_from_template(student_id, &input)?)
    })
}

pub fn list_revision_packs(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<RevisionPackSummaryDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_revision_packs(student_id, limit)?)
    })
}

pub fn create_custom_revision_pack(
    state: &AppState,
    student_id: i64,
    title: String,
    question_ids: Vec<i64>,
    subject_id: Option<i64>,
) -> Result<RevisionPackSummaryDto, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).create_custom_revision_pack(
            student_id,
            &title,
            &question_ids,
            subject_id,
        )?)
    })
}

pub fn list_exam_hotspots(
    state: &AppState,
    student_id: i64,
    subject_id: Option<i64>,
    limit: usize,
) -> Result<Vec<ExamHotspotDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_exam_hotspots(student_id, subject_id, limit)?)
    })
}

pub fn get_topic_library_snapshot(
    state: &AppState,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> Result<TopicLibrarySnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).get_topic_library_snapshot(student_id, topic_id, limit)?)
    })
}

pub fn create_custom_shelf(
    state: &AppState,
    student_id: i64,
    input: CreateCustomShelfInputDto,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).create_custom_shelf(student_id, &input)?)
    })
}

pub fn add_item_to_custom_shelf(
    state: &AppState,
    student_id: i64,
    shelf_id: i64,
    input: AddShelfItemInputDto,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).add_item_to_custom_shelf(student_id, shelf_id, &input)?)
    })
}

pub fn list_custom_shelves(
    state: &AppState,
    student_id: i64,
    include_items: bool,
    item_limit: usize,
) -> Result<Vec<CustomLibraryShelfDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_custom_shelves(student_id, include_items, item_limit)?)
    })
}

pub fn list_offline_library_items(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<OfflineLibraryItemDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_offline_items(student_id, limit)?)
    })
}

pub fn list_library_tag_definitions(
    state: &AppState,
) -> Result<Vec<LibraryTagDefinitionDto>, CommandError> {
    state.with_connection(|conn| Ok(LibraryService::new(conn).list_library_tag_definitions()?))
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

pub fn search_catalog(
    state: &AppState,
    input: GlossarySearchInputDto,
    limit: usize,
) -> Result<GlossarySearchResponseDto, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).search_catalog(&input, limit)?))
}

pub fn search_suggestions(
    state: &AppState,
    query: String,
    limit: usize,
) -> Result<Vec<GlossarySearchSuggestionDto>, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).search_suggestions(&query, limit)?))
}

pub fn search_voice(
    state: &AppState,
    query: String,
    student_id: Option<i64>,
    limit: usize,
) -> Result<GlossarySearchResponseDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).search_voice(&query, student_id, limit)?)
    })
}

pub fn get_entry_detail(
    state: &AppState,
    student_id: Option<i64>,
    entry_id: i64,
    relation_limit: usize,
    bundle_limit: usize,
) -> Result<GlossaryEntryDetailDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).get_entry_detail(
            student_id,
            entry_id,
            relation_limit,
            bundle_limit,
        )?)
    })
}

pub fn build_home_snapshot(
    state: &AppState,
    student_id: i64,
    subject_id: Option<i64>,
    limit: usize,
) -> Result<GlossaryHomeSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).build_home_snapshot(student_id, subject_id, limit)?)
    })
}

pub fn build_compare_view(
    state: &AppState,
    left_entry_id: i64,
    right_entry_id: i64,
) -> Result<GlossaryComparisonViewDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).build_compare_view(left_entry_id, right_entry_id)?)
    })
}

pub fn get_formula_lab(state: &AppState, entry_id: i64) -> Result<FormulaLabViewDto, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).get_formula_lab(entry_id)?))
}

pub fn build_concept_map(
    state: &AppState,
    entry_id: i64,
    depth: usize,
    limit: usize,
) -> Result<ConceptMapViewDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).build_concept_map(entry_id, depth, limit)?)
    })
}

pub fn record_interaction(
    state: &AppState,
    input: GlossaryInteractionInputDto,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).record_interaction(&input)?))
}

pub fn start_audio_queue(
    state: &AppState,
    student_id: i64,
    input: StartGlossaryAudioQueueInputDto,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).start_audio_queue(student_id, &input)?)
    })
}

pub fn next_audio_queue(
    state: &AppState,
    student_id: i64,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).next_audio_queue(student_id)?))
}

pub fn previous_audio_queue(
    state: &AppState,
    student_id: i64,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).previous_audio_queue(student_id)?))
}

pub fn update_audio_queue(
    state: &AppState,
    student_id: i64,
    input: UpdateGlossaryAudioQueueInputDto,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).update_audio_queue(student_id, &input)?)
    })
}

pub fn current_audio_queue(
    state: &AppState,
    student_id: i64,
) -> Result<GlossaryAudioQueueSnapshotDto, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).current_audio_queue(student_id)?))
}

pub fn create_glossary_test_session(
    state: &AppState,
    student_id: i64,
    input: CreateGlossaryTestInputDto,
) -> Result<GlossaryTestSessionDetailDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).create_glossary_test_session(student_id, &input)?)
    })
}

pub fn get_glossary_test_session(
    state: &AppState,
    session_id: i64,
) -> Result<GlossaryTestSessionDetailDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn).get_glossary_test_session(session_id)?)
    })
}

pub fn submit_glossary_test_attempt(
    state: &AppState,
    student_id: i64,
    session_id: i64,
    input: SubmitGlossaryTestAttemptInputDto,
) -> Result<GlossaryTestAttemptResultDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn)
            .submit_glossary_test_attempt(student_id, session_id, &input)?)
    })
}

pub fn rebuild_search_index(state: &AppState) -> Result<usize, CommandError> {
    state.with_connection(|conn| Ok(GlossaryService::new(conn).rebuild_search_index()?))
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

pub fn list_personalized_learning_paths(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<PersonalizedLearningPathDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn)
            .build_personalized_learning_paths(student_id, limit)?
            .into_iter()
            .map(PersonalizedLearningPathDto::from)
            .collect())
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

pub fn list_glossary_bundle_sequence_for_topic(
    state: &AppState,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> Result<Vec<KnowledgeBundleSequenceItemDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn)
            .list_bundle_sequence_for_topic(student_id, topic_id, limit)?
            .into_iter()
            .map(KnowledgeBundleSequenceItemDto::from)
            .collect())
    })
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

pub fn build_personalized_glossary_audio_program_for_topic(
    state: &AppState,
    student_id: i64,
    topic_id: i64,
    limit: usize,
) -> Result<GlossaryAudioProgramDto, CommandError> {
    state.with_connection(|conn| {
        Ok(GlossaryService::new(conn)
            .build_personalized_audio_program_for_topic(student_id, topic_id, limit)?)
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

pub fn build_personalized_glossary_audio_program_for_question(
    state: &AppState,
    student_id: i64,
    question_id: i64,
    limit: usize,
) -> Result<GlossaryAudioProgramDto, CommandError> {
    state.with_connection(|conn| {
        Ok(
            GlossaryService::new(conn).build_personalized_audio_program_for_question(
                student_id,
                question_id,
                limit,
            )?,
        )
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

pub fn upsert_teach_explanation(
    state: &AppState,
    node_id: i64,
    input: TeachExplanationUpsertInput,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).upsert_teach_explanation(node_id, &input)?)
    })
}

pub fn add_teach_micro_check(
    state: &AppState,
    explanation_id: i64,
    input: TeachMicroCheckInput,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).add_teach_micro_check(explanation_id, &input)?)
    })
}

pub fn get_teach_lesson(
    state: &AppState,
    topic_id: i64,
    explanation_level: Option<String>,
    micro_check_limit: usize,
) -> Result<TeachLessonDto, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).get_teach_lesson(
            topic_id,
            explanation_level.as_deref(),
            micro_check_limit,
        )?)
    })
}

pub fn ask_tutor(
    state: &AppState,
    input: TutorInteractionInput,
) -> Result<TutorResponseDto, CommandError> {
    state.with_connection(|conn| {
        let service = LibraryService::new(conn);
        let response = service.ask_tutor(&input)?;
        let mut logged_input = input.clone();
        logged_input.response_text = Some(response.response_text.clone());
        logged_input.context = serde_json::json!({
            "context_summary": response.context_summary,
            "suggested_next_steps": response.suggested_next_steps,
            "related_question_ids": response.related_question_ids,
            "related_entry_ids": response.related_entry_ids,
            "related_topic_names": response.related_topic_names,
        });
        let _ = service.log_tutor_interaction(&logged_input)?;
        Ok(response)
    })
}

pub fn list_recent_tutor_interactions(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<TutorInteractionDto>, CommandError> {
    state.with_connection(|conn| {
        Ok(LibraryService::new(conn).list_recent_tutor_interactions(student_id, limit)?)
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

#[cfg(test)]
mod tests {
    use super::{
        add_library_note, build_compare_view, build_concept_map, build_home_snapshot,
        build_revision_pack_from_template, create_custom_revision_pack, create_custom_shelf,
        create_glossary_test_session, current_audio_queue, get_entry_detail, get_formula_lab,
        get_glossary_test_session, list_custom_shelves, list_exam_hotspots,
        list_glossary_bundle_sequence_for_topic, list_library_item_actions,
        list_library_item_state_history, list_library_items, list_library_tag_definitions,
        list_offline_library_items, list_personalized_learning_paths, list_revision_pack_templates,
        next_audio_queue, previous_audio_queue, rebuild_search_index, record_interaction,
        record_library_item_action, remove_library_item, save_library_item_with_metadata,
        search_catalog, search_library, search_suggestions, search_voice, start_audio_queue,
        submit_glossary_test_attempt, update_audio_queue, update_library_item,
    };
    use crate::state::AppState;
    use ecoach_glossary::{
        CreateGlossaryTestInput, GlossaryInteractionInput, GlossarySearchInput,
        StartGlossaryAudioQueueInput, SubmitGlossaryTestAttemptInput,
        UpdateGlossaryAudioQueueInput,
    };
    use ecoach_library::{
        AddLibraryNoteInput, BuildRevisionPackFromTemplateInput, CreateCustomShelfInput,
        LibrarySearchInput, RecordLibraryItemActionInput, SaveLibraryItemInput,
        UpdateLibraryItemInput,
    };
    use serde_json::json;

    #[test]
    fn library_commands_surface_personalized_paths_and_bundle_sequences() {
        let state = AppState::in_memory().expect("in-memory app state");
        state
            .with_connection(|conn| {
                conn.execute_batch(
                    "
                    INSERT INTO curriculum_versions (id, name, version_label, status)
                    VALUES (1, 'Test Curriculum', 'v1', 'published');
                    INSERT INTO subjects (id, curriculum_version_id, code, name)
                    VALUES (1, 1, 'MTH', 'Mathematics');
                    INSERT INTO accounts (
                        id, account_type, display_name, pin_hash, pin_salt, entitlement_tier, status, first_run
                    ) VALUES (42, 'student', 'Kwame', 'hash', 'salt', 'standard', 'active', 0);
                    INSERT INTO topics (id, subject_id, name) VALUES
                        (1, 1, 'Algebraic Fractions'),
                        (2, 1, 'Equivalent Fractions'),
                        (3, 1, 'Factorisation');
                    INSERT INTO student_topic_states (
                        student_id, topic_id, mastery_score, gap_score, priority_score, trend_state, decay_risk, is_blocked, next_review_at
                    ) VALUES (42, 1, 2800, 7600, 9100, 'declining', 6400, 1, '2026-03-29T10:00:00Z');
                    INSERT INTO node_edges (
                        id, edge_type, from_node_type, from_node_id, to_node_type, to_node_id, strength_score
                    ) VALUES
                        (1, 'prerequisite', 'topic', 2, 'topic', 1, 9300),
                        (2, 'related', 'topic', 3, 'topic', 2, 8600);
                    INSERT INTO knowledge_entries (id, subject_id, topic_id, entry_type, title, status, importance_score, difficulty_level) VALUES
                        (101, 1, 1, 'procedure', 'Simplify before multiplying', 'active', 9000, 1200),
                        (102, 1, 1, 'rule', 'Common denominator check', 'active', 8700, 1500);
                    INSERT INTO knowledge_bundles (id, title, bundle_type, topic_id, exam_relevance_score, difficulty_level)
                    VALUES (10, 'Fractions Recovery', 'repair', 1, 9200, 1300);
                    INSERT INTO knowledge_bundle_items (id, bundle_id, entry_id, item_role, sequence_order) VALUES
                        (1, 10, 101, 'anchor', 0),
                        (2, 10, 102, 'check', 1);
                    INSERT INTO student_entry_state (
                        user_id, entry_id, confusion_score, recall_strength, linked_wrong_answer_count, review_due_at
                    ) VALUES
                        (42, 101, 7200, 2200, 2, '2026-03-29T09:00:00Z'),
                        (42, 102, 4800, 2600, 1, NULL);
                    INSERT INTO questions (id, subject_id, topic_id, stem, is_active, difficulty_level)
                    VALUES (50, 1, 1, 'Simplify the algebraic fraction', 1, 1000);
                    INSERT INTO library_items (id, student_id, item_type, item_ref_id, urgency_score)
                    VALUES (1, 42, 'question', 50, 8800);
                    ",
                )
                .expect("seed data");
                Ok(())
            })
            .expect("seed should insert");

        let paths =
            list_personalized_learning_paths(&state, 42, 3).expect("personalized path dtos");
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].topic_id, 1);
        assert!(
            paths[0]
                .relationship_hints
                .iter()
                .any(|hint| hint.hop_count == 2 && hint.to_title == "Factorisation")
        );
        assert!(paths[0].steps.iter().any(|step| step.bundle_id == Some(10)));

        let bundles = list_glossary_bundle_sequence_for_topic(&state, 42, 1, 3)
            .expect("bundle sequence dtos");
        assert_eq!(bundles.len(), 1);
        assert_eq!(bundles[0].bundle_id, 10);
        assert!(bundles[0].due_review_count >= 1);
        assert_eq!(bundles[0].focus_entry_ids, vec![101, 102]);
    }

    #[test]
    fn library_commands_surface_rich_library_flows() {
        let state = AppState::in_memory().expect("in-memory app state");
        state
            .with_connection(|conn| {
                conn.execute_batch(
                    "
                    INSERT INTO curriculum_versions (id, name, version_label, status)
                    VALUES (1, 'Test Curriculum', 'v1', 'published');
                    INSERT INTO accounts (
                        id, account_type, display_name, pin_hash, pin_salt, entitlement_tier, status, first_run
                    ) VALUES (42, 'student', 'Ama', 'hash', 'salt', 'standard', 'active', 0);
                    INSERT INTO subjects (id, curriculum_version_id, code, name)
                    VALUES (1, 1, 'MTH', 'Mathematics');
                    INSERT INTO topics (id, subject_id, name, exam_weight) VALUES (1, 1, 'Fractions', 7200);
                    INSERT INTO questions (id, subject_id, topic_id, stem, is_active, difficulty_level, family_id)
                    VALUES (50, 1, 1, 'Simplify 1/2 + 1/4', 1, 1100, NULL);
                    INSERT INTO library_items (id, student_id, item_type, item_ref_id, state, tags_json, topic_id, urgency_score)
                    VALUES (1, 42, 'question', 50, 'saved', '[]', 1, 8800);
                    INSERT INTO student_topic_states (
                        student_id, topic_id, mastery_score, gap_score, priority_score, trend_state, decay_risk, is_blocked, next_review_at
                    ) VALUES (42, 1, 7200, 2600, 8900, 'stable', 4200, 0, NULL);
                    INSERT INTO memory_states (id, student_id, topic_id, memory_state, decay_risk)
                    VALUES (1, 42, 1, 'fragile', 6800);
                    ",
                )
                .expect("base seed");
                conn.execute_batch(
                    "
                    INSERT INTO question_families (id, family_code, family_name, subject_id, topic_id, family_type)
                    VALUES (10, 'fractions_family', 'Fractions Family', 1, 1, 'exam_structure');
                    INSERT INTO family_recurrence_metrics (
                        id, family_id, subject_id, total_papers_in_window, papers_appeared, recurrence_rate_bp,
                        family_density_bp, persistence_score_bp, dormancy_max_years, last_appearance_year,
                        first_appearance_year, current_relevance_bp
                    ) VALUES (1, 10, 1, 6, 4, 8400, 7600, 7900, 2, 2026, 2020, 8100);
                    INSERT INTO questions (id, subject_id, topic_id, stem, is_active, difficulty_level, family_id)
                    VALUES (51, 1, 1, 'What is 1/2 + 1/4?', 1, 1200, 10);
                    INSERT INTO academic_nodes (id, topic_id, node_type, canonical_title, exam_relevance_score, foundation_weight)
                    VALUES (11, 1, 'formula', 'Common denominator rule', 8700, 8200);
                    ",
                )
                .expect("library intelligence seed");
                Ok(())
            })
            .expect("seed should insert");

        let saved_id = save_library_item_with_metadata(
            &state,
            42,
            SaveLibraryItemInput {
                item_type: "question".to_string(),
                item_ref_id: 50,
                state: "saved".to_string(),
                tags: vec!["exam_critical".to_string()],
                note_text: Some("Watch the denominator".to_string()),
                topic_id: Some(1),
                urgency_score: 9000,
                subject_id: Some(1),
                subtopic_id: None,
                difficulty_bp: Some(1100),
                exam_frequency_bp: Some(8400),
                source: Some("custom".to_string()),
                goal_id: None,
                calendar_event_id: None,
            },
        )
        .expect("rich save");
        assert!(saved_id > 0);

        update_library_item(
            &state,
            saved_id,
            UpdateLibraryItemInput {
                state: "revisit".to_string(),
                tags: vec!["exam_critical".to_string(), "keep_forgetting".to_string()],
                note_text: Some("Revisit before the next quiz".to_string()),
                urgency_score: 9100,
                topic_id: Some(1),
                subject_id: Some(1),
                subtopic_id: None,
                difficulty_bp: Some(1200),
                exam_frequency_bp: Some(8600),
                source: Some("updated".to_string()),
                goal_id: None,
                calendar_event_id: None,
            },
        )
        .expect("update item");

        let items = list_library_items(&state, 42).expect("items");
        assert!(!items.is_empty());
        assert!(items.iter().any(|item| item.id == saved_id));

        record_library_item_action(
            &state,
            42,
            saved_id,
            RecordLibraryItemActionInput {
                action_type: "opened".to_string(),
                context: json!({"source": "unit-test"}),
            },
        )
        .expect("action");
        add_library_note(
            &state,
            AddLibraryNoteInput {
                student_id: 42,
                library_item_id: Some(saved_id),
                topic_id: Some(1),
                note_type: "memory_hook".to_string(),
                title: Some("Short hook".to_string()),
                note_text: "Denominator first, then combine.".to_string(),
                context: json!({"kind": "hook"}),
            },
        )
        .expect("note");

        let history = list_library_item_state_history(&state, saved_id, 10).expect("history");
        assert!(!history.is_empty());
        let actions = list_library_item_actions(&state, saved_id, 10).expect("actions");
        assert!(!actions.is_empty());

        let search_results = search_library(
            &state,
            42,
            LibrarySearchInput {
                query: Some("fractions".to_string()),
                subject_id: Some(1),
                topic_id: Some(1),
                item_types: vec!["question".to_string()],
                states: vec![],
                tags: vec![],
                only_wrong: false,
                only_near_mastery: false,
                only_untouched: false,
                high_frequency_only: false,
                due_only: false,
                downloaded_only: false,
            },
            10,
        )
        .expect("search");
        assert!(!search_results.is_empty());

        let templates = list_revision_pack_templates(&state).expect("templates");
        assert!(!templates.is_empty());
        let pack = build_revision_pack_from_template(
            &state,
            42,
            BuildRevisionPackFromTemplateInput {
                template_code: "weak_area".to_string(),
                title: Some("Weak area rescue".to_string()),
                item_limit: Some(1),
                subject_id: Some(1),
            },
        )
        .expect("template pack");
        assert!(!pack.topic_ids.is_empty());

        let custom_pack =
            create_custom_revision_pack(&state, 42, "Custom pack".to_string(), vec![50], Some(1))
                .expect("custom pack");
        assert_eq!(custom_pack.question_count, 1);

        let hotspots = list_exam_hotspots(&state, 42, Some(1), 10).expect("hotspots");
        assert!(!hotspots.is_empty());

        let snapshot = super::get_topic_library_snapshot(&state, 42, 1, 10).expect("snapshot");
        assert_eq!(snapshot.topic_id, 1);

        let shelf_id = create_custom_shelf(
            &state,
            42,
            CreateCustomShelfInput {
                title: "My Rescue Shelf".to_string(),
                description: Some("Personal catch-up shelf".to_string()),
                icon_hint: Some("spark".to_string()),
            },
        )
        .expect("custom shelf");
        assert!(shelf_id > 0);

        let shelves = list_custom_shelves(&state, 42, true, 10).expect("shelves");
        assert_eq!(shelves.len(), 1);

        let tag_defs = list_library_tag_definitions(&state).expect("tags");
        assert!(!tag_defs.is_empty());

        let _offline_items = list_offline_library_items(&state, 42, 10).expect("offline");

        remove_library_item(&state, saved_id).expect("remove item");
    }

    #[test]
    fn library_commands_surface_rich_glossary_flow() {
        let state = AppState::in_memory().expect("in-memory app state");
        seed_glossary_runtime_flow(&state);

        let rebuilt = rebuild_search_index(&state).expect("search index rebuild");
        assert_eq!(rebuilt, 3);

        let search = search_catalog(
            &state,
            GlossarySearchInput {
                query: "fraction".to_string(),
                student_id: Some(42),
                subject_id: Some(1),
                topic_id: Some(1),
                include_bundles: true,
                include_questions: true,
                include_confusions: true,
                include_audio_ready_only: false,
            },
            10,
        )
        .expect("catalog search");
        assert_eq!(search.normalized_query, "fraction");
        assert!(
            search
                .groups
                .iter()
                .any(|group| group.group_key == "best_match" && !group.results.is_empty())
        );

        let suggestions =
            search_suggestions(&state, "fraction".to_string(), 5).expect("search suggestions");
        assert!(!suggestions.is_empty());

        let voice =
            search_voice(&state, "fraction".to_string(), Some(42), 5).expect("voice search");
        assert!(!voice.groups.is_empty());

        let detail = get_entry_detail(&state, Some(42), 501, 8, 4).expect("entry detail");
        assert_eq!(detail.entry.id, 501);
        assert_eq!(detail.aliases.len(), 1);
        assert_eq!(detail.content_blocks.len(), 1);
        assert!(detail.student_state.is_some());
        assert!(!detail.confusion_pairs.is_empty());

        let compare = build_compare_view(&state, 501, 502).expect("compare view");
        assert!(!compare.shared_relation_types.is_empty());
        assert!(compare.distinction_explanation.is_some());

        let formula_lab = get_formula_lab(&state, 502).expect("formula lab");
        assert_eq!(formula_lab.entry.id, 502);

        let concept_map = build_concept_map(&state, 501, 2, 10).expect("concept map");
        assert!(concept_map.nodes.len() >= 2);

        let home = build_home_snapshot(&state, 42, Some(1), 10).expect("home snapshot");
        assert!(!home.discover.is_empty());
        assert!(!home.exam_hotspots.is_empty());

        let interaction_id = record_interaction(
            &state,
            GlossaryInteractionInput {
                student_id: Some(42),
                entry_id: Some(501),
                bundle_id: Some(9001),
                question_id: Some(7001),
                event_type: "opened_entry".to_string(),
                query_text: Some("fraction".to_string()),
                metadata: json!({"source": "command-test"}),
            },
        )
        .expect("record interaction");
        assert!(interaction_id > 0);

        let audio = start_audio_queue(
            &state,
            42,
            StartGlossaryAudioQueueInput {
                source_type: "entry".to_string(),
                source_id: 501,
                limit: 3,
                teaching_mode: Some("standard".to_string()),
                include_examples: true,
                include_misconceptions: true,
            },
        )
        .expect("start audio queue");
        assert!(audio.current_program_id.is_some());
        assert!(audio.program.is_some());

        let advanced = next_audio_queue(&state, 42).expect("next audio queue");
        assert_eq!(advanced.current_program_id, audio.current_program_id);

        let rewound = previous_audio_queue(&state, 42).expect("previous audio queue");
        assert_eq!(rewound.current_program_id, audio.current_program_id);

        let updated = update_audio_queue(
            &state,
            42,
            UpdateGlossaryAudioQueueInput {
                playback_speed: Some(1.25),
                include_examples: Some(false),
                include_misconceptions: Some(false),
                is_playing: Some(false),
            },
        )
        .expect("update audio queue");
        assert!((updated.playback_speed - 1.25).abs() < f64::EPSILON);
        assert!(!updated.include_examples);
        assert!(!updated.include_misconceptions);
        assert!(!updated.is_playing);

        let current = current_audio_queue(&state, 42).expect("current audio queue");
        assert_eq!(current.current_program_id, audio.current_program_id);

        let session = create_glossary_test_session(
            &state,
            42,
            CreateGlossaryTestInput {
                test_mode: "context_recognition".to_string(),
                topic_id: Some(1),
                bundle_id: None,
                entry_ids: vec![501],
                entry_count: 1,
                duration_seconds: Some(90),
                difficulty_level: Some(5600),
            },
        )
        .expect("create test session");
        assert_eq!(session.student_id, 42);
        assert_eq!(session.items.len(), 1);

        let fetched_session =
            get_glossary_test_session(&state, session.session_id).expect("fetch test session");
        assert_eq!(fetched_session.session_id, session.session_id);

        let attempt = submit_glossary_test_attempt(
            &state,
            42,
            session.session_id,
            SubmitGlossaryTestAttemptInput {
                entry_id: 501,
                student_response: "Fraction".to_string(),
                time_seconds: Some(12),
            },
        )
        .expect("submit test attempt");
        assert!(attempt.is_correct);
        assert!(attempt.mastery_score > 0);
    }

    fn seed_glossary_runtime_flow(state: &AppState) {
        state
            .with_connection(|conn| {
                conn.execute_batch(
                    "
                    INSERT INTO curriculum_versions (id, name, version_label, status)
                    VALUES (1, 'Glossary Runtime', 'v1', 'published');
                    INSERT INTO subjects (id, curriculum_version_id, code, name)
                    VALUES (1, 1, 'MTH', 'Mathematics');
                    INSERT INTO topics (id, subject_id, name, exam_weight)
                    VALUES (1, 1, 'Fractions', 7300);
                    INSERT INTO accounts (
                        id, account_type, display_name, pin_hash, pin_salt, entitlement_tier, status, first_run
                    ) VALUES (42, 'student', 'Ama', 'hash', 'salt', 'standard', 'active', 0);
                    INSERT INTO knowledge_entries (
                        id, subject_id, topic_id, entry_type, title, canonical_name, slug,
                        short_text, full_text, simple_text, technical_text, exam_text,
                        importance_score, difficulty_level, grade_band, status, audio_available,
                        has_formula, confusion_pair_count, example_count, misconception_count,
                        exam_relevance_score, priority_score, phonetic_text
                    ) VALUES
                        (501, 1, 1, 'definition', 'Fraction', 'Fraction', 'fraction',
                         'A part of a whole', 'A fraction shows part of a whole.',
                         'A fraction is part of something', 'A rational expression with numerator and denominator.',
                         'Use fractions when parts of a whole are involved.',
                         9100, 1200, 'JHS', 'active', 1, 0, 1, 1, 1, 9200, 9300, 'frak-shun'),
                        (502, 1, 1, 'formula', 'Fraction Addition Rule', 'Fraction Addition Rule', 'fraction-addition-rule',
                         'Add with common denominators', 'Add fractions by creating a common denominator.',
                         'Make denominators alike before adding', 'a/b + c/d = (ad + bc)/bd',
                         'Remember the common denominator step.',
                         9000, 1300, 'JHS', 'active', 1, 1, 1, 1, 1, 9300, 9400, NULL),
                        (503, 1, 1, 'concept', 'Common Denominator', 'Common Denominator', 'common-denominator',
                         'Shared denominator', 'A common denominator is the shared bottom number.',
                         'The same number on the bottom', 'The least common multiple of denominators.',
                         'Find the shared denominator first.',
                         8800, 1100, 'JHS', 'active', 0, 0, 0, 0, 0, 8600, 8700, NULL);
                    INSERT INTO entry_aliases (id, entry_id, alias_text, alias_type) VALUES
                        (1, 501, 'parts of a whole', 'synonym'),
                        (2, 502, 'fraction sum rule', 'synonym');
                    INSERT INTO definition_meta (
                        id, entry_id, definition_text, short_definition, real_world_meaning,
                        non_examples, context_clues
                    ) VALUES
                        (1, 501, 'A fraction shows part of a whole.', 'Part of a whole',
                         'Used when sharing items or measuring parts', 'A whole number by itself',
                         'Look for the numerator and denominator');
                    INSERT INTO formula_meta (
                        id, entry_id, formula_expression, formula_speech, variables_json, units_json,
                        when_to_use, when_not_to_use, rearrangements_json, derivation_summary
                    ) VALUES
                        (1, 502, 'a/b + c/d = (ad + bc)/bd', 'a over b plus c over d equals a d plus b c over b d',
                         '[\"a\", \"b\", \"c\", \"d\"]', NULL,
                         'Use when adding fractions with different denominators.',
                         'Do not use when denominators are already the same without simplifying.',
                         '[\"a/b = ?\", \"c/d = ?\"]',
                         'Create equal denominators first, then combine numerators.');
                    INSERT INTO concept_meta (
                        id, entry_id, concept_explanation, intuition_summary, related_visual_keywords,
                        misconception_signals
                    ) VALUES
                        (1, 503, 'A common denominator is a shared bottom number.',
                         'Make the bottoms match before combining fractions.',
                         '[\"bar model\", \"grid\", \"pizza\"]',
                         '[\"just add denominators\", \"bottom numbers stay different\"]');
                    INSERT INTO entry_examples (
                        id, entry_id, sequence_order, example_text, context_type, difficulty_level,
                        worked_solution_text, is_exam_style
                    ) VALUES
                        (1, 501, 1, 'One half of a pizza is 1/2.', 'real_world', 900,
                         '1/2 means one out of two equal parts.', 0);
                    INSERT INTO entry_misconceptions (
                        id, entry_id, misconception_text, cause_explanation, correction_explanation,
                        confusion_pair_entry_id, misconception_source, severity_bp
                    ) VALUES
                        (1, 501, 'Adding denominators directly', 'Students focus on the bottom numbers.',
                         'Keep denominators aligned before adding.', 502, 'curated', 7800),
                        (2, 502, 'Use the numerator rule on denominators', 'The formula is remembered incompletely.',
                         'Multiply across, then add the results.', 501, 'curated', 7000);
                    INSERT INTO entry_content_blocks (id, entry_id, block_type, order_index, content_json) VALUES
                        (1, 501, 'definition', 1, '{\"heading\":\"Definition\",\"text\":\"A fraction shows part of a whole.\"}');
                    INSERT INTO entry_audio_segments (
                        id, entry_id, segment_type, script_text, duration_seconds, teaching_mode, is_auto_generated
                    ) VALUES
                        (1, 501, 'definition', 'A fraction shows part of a whole.', 15, 'standard', 1),
                        (2, 502, 'formula_speech', 'a over b plus c over d equals a d plus b c over b d', 18, 'standard', 1);
                    INSERT INTO knowledge_relations (
                        id, from_entry_id, to_entry_id, relation_type, strength_score, explanation
                    ) VALUES
                        (1, 501, 502, 'related', 9200, 'Addition uses the shared denominator rule.'),
                        (2, 501, 503, 'prerequisite', 8800, 'Understand the common denominator first.');
                    INSERT INTO knowledge_bundles (
                        id, title, bundle_type, subject_id, topic_id, description, difficulty_level, exam_relevance_score
                    ) VALUES
                        (9001, 'Fractions Fix', 'repair', 1, 1, 'Core fraction recovery bundle', 1300, 9400);
                    INSERT INTO knowledge_bundle_items (
                        id, bundle_id, entry_id, item_role, sequence_order, required
                    ) VALUES
                        (1, 9001, 501, 'anchor', 0, 1),
                        (2, 9001, 502, 'support', 1, 1);
                    INSERT INTO questions (id, subject_id, topic_id, stem, is_active, difficulty_level)
                    VALUES
                        (7001, 1, 1, 'What is one half of a pizza?', 1, 900);
                    INSERT INTO question_glossary_links (
                        id, question_id, entry_id, relation_type, confidence_score, is_primary, link_source, link_reason
                    ) VALUES
                        (1, 7001, 501, 'repair_support', 9400, 1, 'manual', 'Supports fraction recovery'),
                        (2, 7001, 502, 'repair_support', 9000, 0, 'manual', 'Supports fraction addition');
                    INSERT INTO confusion_pairs (
                        id, entry_id_1, entry_id_2, distinction_explanation, common_confusion_reason,
                        clue_to_distinguish, example_sentence_1, example_sentence_2, confusion_frequency_bp
                    ) VALUES
                        (1, 501, 502, 'Fractions name parts; the rule adds them.', 'The two are both used together.',
                         'Ask whether you are naming a part or adding parts.',
                         '1/2 names a part of a whole.',
                         '1/2 + 1/4 uses the addition rule.',
                         8500);
                    INSERT INTO neighbor_intruder_mappings (
                        id, entry_id, neighbor_entry_ids_json, intruder_entry_ids_json
                    ) VALUES
                        (1, 501, '[502,503]', '[7001]');
                    INSERT INTO student_entry_state (
                        user_id, entry_id, familiarity_state, mastery_score, confusion_score, recall_strength,
                        last_viewed_at, last_played_at, last_tested_at, review_due_at, open_count,
                        linked_wrong_answer_count, recognition_score, connection_score, application_score,
                        retention_score, test_count, test_pass_count, spaced_review_due_at, at_risk_threshold_date
                    ) VALUES
                        (42, 501, 'unseen', 1200, 7800, 1800, NULL, NULL, NULL, '2026-03-31T10:00:00Z',
                         0, 1, 2200, 1800, 1500, 1400, 1, 0, NULL, NULL),
                        (42, 502, 'seen', 2200, 3200, 5200, NULL, NULL, NULL, NULL,
                         0, 0, 2600, 2100, 2000, 1900, 0, 0, NULL, NULL);
                ",
                )
                .expect("glossary seed");
                Ok(())
            })
            .expect("seed should insert");
    }
}
