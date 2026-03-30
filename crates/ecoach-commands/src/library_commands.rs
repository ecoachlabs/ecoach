use ecoach_glossary::{
    GlossaryAudioProgram, GlossaryService, KnowledgeBundle, QuestionKnowledgeLink,
};
use ecoach_library::{
    ContinueLearningCard, LibraryHomeSnapshot, LibraryService, RevisionPackItem,
    RevisionPackSummary, TeachActionPlan, TopicRelationshipHint,
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
    use super::{list_glossary_bundle_sequence_for_topic, list_personalized_learning_paths};
    use crate::state::AppState;

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
}
