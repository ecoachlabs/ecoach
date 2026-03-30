use ecoach_questions::{
    QuestionGenerationRequestInput, QuestionReactor, QuestionService, QuestionSlotSpec,
};

use crate::{
    dtos::{
        DuplicateCheckResultDto, GeneratedQuestionDraftDto, QuestionFamilyChoiceDto,
        QuestionFamilyHealthDto, QuestionGenerationRequestDto, QuestionLineageGraphDto,
        RelatedQuestionDto,
    },
    error::CommandError,
    state::AppState,
};

pub fn choose_reactor_family(
    state: &AppState,
    slot_spec: QuestionSlotSpec,
) -> Result<Option<QuestionFamilyChoiceDto>, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(reactor
            .get_best_family_for_slot(&slot_spec)?
            .map(QuestionFamilyChoiceDto::from))
    })
}

pub fn create_question_generation_request(
    state: &AppState,
    input: QuestionGenerationRequestInput,
) -> Result<QuestionGenerationRequestDto, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        let request = reactor.create_generation_request(&input)?;
        Ok(QuestionGenerationRequestDto::from(request))
    })
}

pub fn process_question_generation_request(
    state: &AppState,
    request_id: i64,
) -> Result<Vec<GeneratedQuestionDraftDto>, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(reactor
            .process_generation_request(request_id)?
            .into_iter()
            .map(GeneratedQuestionDraftDto::from)
            .collect())
    })
}

pub fn get_question_lineage(
    state: &AppState,
    question_id: i64,
) -> Result<QuestionLineageGraphDto, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(QuestionLineageGraphDto::from(
            reactor.get_question_lineage(question_id)?,
        ))
    })
}

pub fn get_question_family_health(
    state: &AppState,
    family_id: i64,
) -> Result<Option<QuestionFamilyHealthDto>, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(reactor
            .get_family_health(family_id)?
            .map(QuestionFamilyHealthDto::from))
    })
}

pub fn list_related_questions(
    state: &AppState,
    question_id: i64,
    relation_type: Option<String>,
    limit: usize,
) -> Result<Vec<RelatedQuestionDto>, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(service
            .list_related_questions(question_id, relation_type.as_deref(), limit)?
            .into_iter()
            .map(RelatedQuestionDto::from)
            .collect())
    })
}

pub fn detect_near_duplicate(
    state: &AppState,
    stem: String,
    family_id: Option<i64>,
    topic_id: Option<i64>,
) -> Result<DuplicateCheckResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(DuplicateCheckResultDto::from(
            service.detect_near_duplicate(&stem, family_id, topic_id)?,
        ))
    })
}
