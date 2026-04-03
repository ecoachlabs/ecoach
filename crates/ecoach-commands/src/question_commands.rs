use ecoach_questions::{
    QuestionGenerationRequestInput, QuestionIntelligenceFilter, QuestionReactor,
    QuestionReviewActionInput, QuestionService, QuestionSlotSpec,
};

use crate::{
    dtos::{
        DuplicateCheckResultDto, GeneratedQuestionDraftDto, QuestionFamilyChoiceDto,
        QuestionFamilyHealthDto, QuestionGenerationRequestDto, QuestionIntelligenceSnapshotDto,
        QuestionLineageGraphDto, QuestionRemediationPlanDto, QuestionReviewQueueItemDto,
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

pub fn recommend_question_remediation_plan(
    state: &AppState,
    student_id: i64,
    slot_spec: QuestionSlotSpec,
) -> Result<Option<QuestionRemediationPlanDto>, CommandError> {
    state.with_connection(|conn| {
        let reactor = QuestionReactor::new(conn);
        Ok(reactor
            .recommend_remediation_plan(student_id, &slot_spec)?
            .map(QuestionRemediationPlanDto::from))
    })
}

pub fn get_question_intelligence(
    state: &AppState,
    question_id: i64,
) -> Result<Option<QuestionIntelligenceSnapshotDto>, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(service
            .get_question_intelligence(question_id)?
            .map(QuestionIntelligenceSnapshotDto::from))
    })
}

pub fn classify_question_intelligence(
    state: &AppState,
    question_id: i64,
    reclassify: bool,
) -> Result<QuestionIntelligenceSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(QuestionIntelligenceSnapshotDto::from(
            service.classify_question(question_id, reclassify)?,
        ))
    })
}

pub fn find_questions_by_intelligence_filter(
    state: &AppState,
    filter: QuestionIntelligenceFilter,
) -> Result<Vec<QuestionIntelligenceSnapshotDto>, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(service
            .find_questions_by_intelligence_filter(&filter)?
            .into_iter()
            .map(QuestionIntelligenceSnapshotDto::from)
            .collect())
    })
}

pub fn list_question_review_queue(
    state: &AppState,
    review_status: Option<String>,
    limit: usize,
) -> Result<Vec<QuestionReviewQueueItemDto>, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(service
            .list_question_review_queue(review_status.as_deref(), limit)?
            .into_iter()
            .map(QuestionReviewQueueItemDto::from)
            .collect())
    })
}

pub fn review_question_intelligence(
    state: &AppState,
    question_id: i64,
    input: QuestionReviewActionInput,
) -> Result<QuestionIntelligenceSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        Ok(QuestionIntelligenceSnapshotDto::from(
            service.review_question_intelligence(question_id, &input)?,
        ))
    })
}

pub fn queue_question_reclassification(
    state: &AppState,
    question_id: i64,
    trigger_reason: String,
    requested_by: Option<String>,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        let service = QuestionService::new(conn);
        service
            .queue_question_reclassification(question_id, &trigger_reason, requested_by.as_deref())
            .map_err(CommandError::from)
    })
}
