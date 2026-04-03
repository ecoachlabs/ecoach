use std::collections::HashSet;

use chrono::{DateTime, NaiveDateTime, Utc};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    AddLibraryNoteInput, AddShelfItemInput, BuildRevisionPackFromTemplateInput,
    ContinueLearningCard, CreateCustomShelfInput, CustomLibraryShelf, ExamHotspot,
    GeneratedLibraryShelf, LearningPathStep, LibraryHomeSnapshot, LibraryItem, LibraryItemAction,
    LibraryItemStateHistoryEntry, LibraryNote, LibrarySearchInput, LibrarySearchResult,
    LibraryShelfItem, LibraryTagDefinition, OfflineLibraryItem, PersonalizedLearningPath,
    RecordLibraryItemActionInput, RevisionPackItem, RevisionPackSummary, RevisionPackTemplate,
    SaveLibraryItemInput, SavedQuestionCard, TeachActionPlan, TeachActionStep, TeachExplanation,
    TeachExplanationUpsertInput, TeachLesson, TeachMicroCheck, TeachMicroCheckInput,
    TopicLibrarySnapshot, TopicRelationshipHint, TutorInteraction, TutorInteractionInput,
    TutorResponse, UpdateLibraryItemInput,
};

pub struct LibraryService<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone)]
struct BundleSequenceCandidate {
    bundle_id: i64,
    title: String,
    reason: String,
}

#[derive(Debug, Clone)]
struct RelationshipTopicCandidate {
    related_topic_id: i64,
    related_topic_name: String,
    relation_type: String,
    strength_score: BasisPoints,
}

#[derive(Debug, Clone)]
struct TeachNodeCandidate {
    node_id: i64,
    node_title: String,
}

#[derive(Debug, Clone)]
struct QuestionOptionSummary {
    label: Option<String>,
    text: String,
    is_correct: bool,
}

#[derive(Debug, Clone)]
struct TutorQuestionContext {
    topic_id: Option<i64>,
    stem: String,
    options: Vec<QuestionOptionSummary>,
    correct_option_text: Option<String>,
}

impl<'a> LibraryService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn save_item(
        &self,
        student_id: i64,
        item_type: &str,
        item_ref_id: i64,
    ) -> EcoachResult<i64> {
        self.save_item_with_metadata(
            student_id,
            &SaveLibraryItemInput {
                item_type: item_type.to_string(),
                item_ref_id,
                state: "saved".to_string(),
                tags: Vec::new(),
                note_text: None,
                topic_id: None,
                urgency_score: 5000,
                subject_id: None,
                subtopic_id: None,
                difficulty_bp: None,
                exam_frequency_bp: None,
                source: None,
                goal_id: None,
                calendar_event_id: None,
            },
        )
    }

    pub fn save_item_with_metadata(
        &self,
        student_id: i64,
        input: &SaveLibraryItemInput,
    ) -> EcoachResult<i64> {
        let (
            inferred_topic_id,
            inferred_subject_id,
            inferred_subtopic_id,
            inferred_difficulty_bp,
            inferred_exam_frequency_bp,
            inferred_source,
        ) = self.resolve_library_item_dimensions(
            &input.item_type,
            input.item_ref_id,
            input.topic_id,
            input.subject_id,
            input.subtopic_id,
            input.difficulty_bp,
            input.exam_frequency_bp,
            input.source.as_deref(),
        )?;
        let tags_json = serde_json::to_string(&input.tags)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let existing_id = self
            .conn
            .query_row(
                "SELECT id
                 FROM library_items
                 WHERE student_id = ?1
                   AND item_type = ?2
                   AND item_ref_id = ?3
                 ORDER BY id DESC
                 LIMIT 1",
                params![student_id, input.item_type, input.item_ref_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some(existing_id) = existing_id {
            let previous_state = self
                .conn
                .query_row(
                    "SELECT state FROM library_items WHERE id = ?1",
                    [existing_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.conn
                .execute(
                    "UPDATE library_items
                     SET state = ?1,
                         tags_json = ?2,
                         note_text = ?3,
                         topic_id = ?4,
                         subject_id = ?5,
                         subtopic_id = ?6,
                         urgency_score = ?7,
                         difficulty_bp = ?8,
                         exam_frequency_bp = ?9,
                         source = ?10,
                         goal_id = ?11,
                         calendar_event_id = ?12,
                         updated_at = datetime('now')
                     WHERE id = ?13",
                    params![
                        input.state,
                        tags_json,
                        input.note_text,
                        inferred_topic_id,
                        inferred_subject_id,
                        inferred_subtopic_id,
                        input.urgency_score,
                        inferred_difficulty_bp,
                        inferred_exam_frequency_bp,
                        inferred_source,
                        input.goal_id,
                        input.calendar_event_id,
                        existing_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if previous_state.as_deref() != Some(input.state.as_str()) {
                self.insert_state_history(
                    existing_id,
                    previous_state.as_deref(),
                    &input.state,
                    Some("metadata_update"),
                )?;
            }
            return Ok(existing_id);
        }

        self.conn
            .execute(
                "INSERT INTO library_items (
                    student_id,
                    item_type,
                    item_ref_id,
                    state,
                    tags_json,
                    note_text,
                    topic_id,
                    subject_id,
                    subtopic_id,
                    urgency_score,
                    difficulty_bp,
                    exam_frequency_bp,
                    source,
                    goal_id,
                    calendar_event_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    student_id,
                    input.item_type,
                    input.item_ref_id,
                    input.state,
                    tags_json,
                    input.note_text,
                    inferred_topic_id,
                    inferred_subject_id,
                    inferred_subtopic_id,
                    input.urgency_score,
                    inferred_difficulty_bp,
                    inferred_exam_frequency_bp,
                    inferred_source,
                    input.goal_id,
                    input.calendar_event_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let library_item_id = self.conn.last_insert_rowid();
        self.insert_state_history(
            library_item_id,
            None,
            &input.state,
            Some("saved_to_library"),
        )?;
        Ok(library_item_id)
    }

    pub fn update_item_metadata(
        &self,
        library_item_id: i64,
        state: &str,
        tags: &[String],
        note_text: Option<&str>,
        urgency_score: BasisPoints,
    ) -> EcoachResult<()> {
        self.update_item_details(
            library_item_id,
            &UpdateLibraryItemInput {
                state: state.to_string(),
                tags: tags.to_vec(),
                note_text: note_text.map(|value| value.to_string()),
                urgency_score,
                topic_id: None,
                subject_id: None,
                subtopic_id: None,
                difficulty_bp: None,
                exam_frequency_bp: None,
                source: None,
                goal_id: None,
                calendar_event_id: None,
            },
            None,
        )
    }

    pub fn update_item_details(
        &self,
        library_item_id: i64,
        input: &UpdateLibraryItemInput,
        reason: Option<&str>,
    ) -> EcoachResult<()> {
        let tags_json = serde_json::to_string(&input.tags)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let previous_state = self
            .conn
            .query_row(
                "SELECT state FROM library_items WHERE id = ?1",
                [library_item_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE library_items
                 SET state = ?1,
                     tags_json = ?2,
                     note_text = ?3,
                     topic_id = COALESCE(?4, topic_id),
                     subject_id = COALESCE(?5, subject_id),
                     subtopic_id = COALESCE(?6, subtopic_id),
                     urgency_score = ?7,
                     difficulty_bp = COALESCE(?8, difficulty_bp),
                     exam_frequency_bp = COALESCE(?9, exam_frequency_bp),
                     source = COALESCE(?10, source),
                     goal_id = COALESCE(?11, goal_id),
                     calendar_event_id = COALESCE(?12, calendar_event_id),
                     updated_at = datetime('now')
                 WHERE id = ?13",
                params![
                    input.state,
                    tags_json,
                    input.note_text,
                    input.topic_id,
                    input.subject_id,
                    input.subtopic_id,
                    input.urgency_score,
                    input.difficulty_bp,
                    input.exam_frequency_bp,
                    input.source,
                    input.goal_id,
                    input.calendar_event_id,
                    library_item_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if previous_state.as_deref() != Some(input.state.as_str()) {
            self.insert_state_history(
                library_item_id,
                previous_state.as_deref(),
                &input.state,
                reason,
            )?;
        }
        Ok(())
    }

    pub fn list_items(&self, student_id: i64) -> EcoachResult<Vec<LibraryItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    id,
                    student_id,
                    item_type,
                    item_ref_id,
                    state,
                    tags_json,
                    note_text,
                    topic_id,
                    subject_id,
                    subtopic_id,
                    urgency_score,
                    difficulty_bp,
                    exam_frequency_bp,
                    source,
                    goal_id,
                    calendar_event_id,
                    last_opened_at,
                    open_count,
                    study_count,
                    created_at,
                    updated_at
                 FROM library_items
                 WHERE student_id = ?1
                 ORDER BY urgency_score DESC, updated_at DESC, id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        let rows = statement
            .query_map([student_id], parse_library_item_row)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn remove_item(&self, library_item_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "DELETE FROM library_item_actions WHERE library_item_id = ?1",
                [library_item_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM library_item_state_history WHERE library_item_id = ?1",
                [library_item_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute("DELETE FROM library_items WHERE id = ?1", [library_item_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn list_item_state_history(
        &self,
        library_item_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryItemStateHistoryEntry>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, library_item_id, from_state, to_state, reason, changed_at
                 FROM library_item_state_history
                 WHERE library_item_id = ?1
                 ORDER BY changed_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![library_item_id, limit as i64], |row| {
                Ok(LibraryItemStateHistoryEntry {
                    id: row.get(0)?,
                    library_item_id: row.get(1)?,
                    from_state: row.get(2)?,
                    to_state: row.get(3)?,
                    reason: row.get(4)?,
                    changed_at: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut history = Vec::new();
        for row in rows {
            history.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(history)
    }

    pub fn record_item_action(
        &self,
        student_id: i64,
        library_item_id: i64,
        input: &RecordLibraryItemActionInput,
    ) -> EcoachResult<i64> {
        let context_json = serde_json::to_string(&input.context)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO library_item_actions (
                    student_id,
                    library_item_id,
                    action_type,
                    context_json
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![student_id, library_item_id, input.action_type, context_json],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let action_id = self.conn.last_insert_rowid();

        match input.action_type.as_str() {
            "opened" => {
                self.conn
                    .execute(
                        "UPDATE library_items
                         SET open_count = open_count + 1,
                             last_opened_at = datetime('now'),
                             updated_at = datetime('now')
                         WHERE id = ?1",
                        [library_item_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
            "studied" | "tested" | "tested_self" | "turned_flashcard" => {
                self.conn
                    .execute(
                        "UPDATE library_items
                         SET study_count = study_count + 1,
                             updated_at = datetime('now')
                         WHERE id = ?1",
                        [library_item_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
            "marked_weak" => {
                self.mark_library_item_state_and_tag(
                    library_item_id,
                    "weak",
                    "needs_practice",
                    Some("action_marked_weak"),
                )?;
            }
            "marked_mastered" => {
                self.mark_library_item_state_and_tag(
                    library_item_id,
                    "mastered",
                    "revision_done",
                    Some("action_marked_mastered"),
                )?;
            }
            "marked_exam_critical" => {
                self.add_tag_to_library_item(library_item_id, "exam_critical")?;
            }
            "marked_confusing" => {
                self.add_tag_to_library_item(library_item_id, "confusing")?;
            }
            _ => {}
        }

        Ok(action_id)
    }

    pub fn list_item_actions(
        &self,
        library_item_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryItemAction>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, student_id, library_item_id, action_type, context_json, created_at
                 FROM library_item_actions
                 WHERE library_item_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![library_item_id, limit as i64], |row| {
                Ok(LibraryItemAction {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    library_item_id: row.get(2)?,
                    action_type: row.get(3)?,
                    context: parse_value_sql(row.get(4)?)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut actions = Vec::new();
        for row in rows {
            actions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(actions)
    }

    pub fn add_library_note(&self, input: &AddLibraryNoteInput) -> EcoachResult<i64> {
        let context_json = serde_json::to_string(&input.context)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO library_notes (
                    student_id,
                    library_item_id,
                    topic_id,
                    note_type,
                    title,
                    note_text,
                    context_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    input.student_id,
                    input.library_item_id,
                    input.topic_id,
                    input.note_type,
                    input.title,
                    input.note_text,
                    context_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let note_id = self.conn.last_insert_rowid();

        if let Some(library_item_id) = input.library_item_id {
            self.record_item_action(
                input.student_id,
                library_item_id,
                &RecordLibraryItemActionInput {
                    action_type: "added_note".to_string(),
                    context: json!({
                        "note_id": note_id,
                        "note_type": input.note_type,
                    }),
                },
            )?;
        }

        Ok(note_id)
    }

    pub fn list_library_notes(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        library_item_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryNote>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    id,
                    student_id,
                    library_item_id,
                    topic_id,
                    note_type,
                    title,
                    note_text,
                    context_json,
                    created_at,
                    updated_at
                 FROM library_notes
                 WHERE student_id = ?1
                   AND (?2 IS NULL OR topic_id = ?2)
                   AND (?3 IS NULL OR library_item_id = ?3)
                 ORDER BY updated_at DESC, id DESC
                 LIMIT ?4",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![student_id, topic_id, library_item_id, limit as i64],
                |row| {
                    Ok(LibraryNote {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        library_item_id: row.get(2)?,
                        topic_id: row.get(3)?,
                        note_type: row.get(4)?,
                        title: row.get(5)?,
                        note_text: row.get(6)?,
                        context: parse_value_sql(row.get(7)?)?,
                        created_at: row.get(8)?,
                        updated_at: row.get(9)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut notes = Vec::new();
        for row in rows {
            notes.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(notes)
    }

    pub fn list_library_tag_definitions(&self) -> EcoachResult<Vec<LibraryTagDefinition>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, tag_code, display_name, category, description, color_hint, is_system
                 FROM library_tag_definitions
                 ORDER BY category ASC, display_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok(LibraryTagDefinition {
                    id: row.get(0)?,
                    tag_code: row.get(1)?,
                    display_name: row.get(2)?,
                    category: row.get(3)?,
                    description: row.get(4)?,
                    color_hint: row.get(5)?,
                    is_system: row.get::<_, i64>(6)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(tags)
    }

    pub fn list_saved_questions(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<SavedQuestionCard>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    li.id,
                    q.id,
                    q.topic_id,
                    t.name,
                    q.stem,
                    li.state,
                    qf.family_name,
                    COALESCE((
                        SELECT COUNT(*)
                        FROM question_glossary_links qgl
                        WHERE qgl.question_id = q.id
                    ), 0),
                    li.urgency_score,
                    li.created_at
                 FROM library_items li
                 INNER JOIN questions q ON q.id = li.item_ref_id
                 INNER JOIN topics t ON t.id = q.topic_id
                 LEFT JOIN question_families qf ON qf.id = q.family_id
                 WHERE li.student_id = ?1
                   AND li.item_type = 'question'
                 ORDER BY li.urgency_score DESC, li.updated_at DESC, li.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(SavedQuestionCard {
                    library_item_id: row.get(0)?,
                    question_id: row.get(1)?,
                    topic_id: row.get(2)?,
                    topic_name: row.get(3)?,
                    stem: row.get(4)?,
                    state: row.get(5)?,
                    related_family_name: row.get(6)?,
                    linked_knowledge_count: row.get(7)?,
                    urgency_score: row.get(8)?,
                    saved_at: row.get(9)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    pub fn build_home_snapshot(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<LibraryHomeSnapshot> {
        let now = Utc::now().to_rfc3339();
        let pending_review_count = self.count_scalar(
            "SELECT COUNT(*)
             FROM coach_mission_memories
             WHERE student_id = ?1
               AND review_status = 'pending'",
            params![student_id],
        )?;
        let fading_concept_count = self.count_scalar(
            "SELECT COUNT(*)
             FROM memory_states
             WHERE student_id = ?1
               AND (
                    memory_state IN ('fragile', 'at_risk', 'fading', 'collapsed', 'rebuilding')
                    OR (review_due_at IS NOT NULL AND review_due_at <= ?2)
               )",
            params![student_id, now],
        )?;
        let untouched_saved_count = self.count_scalar(
            "SELECT COUNT(*)
             FROM library_items
             WHERE student_id = ?1
               AND state IN ('saved', 'new', 'revisit')",
            params![student_id],
        )?;
        let learning_paths = self.build_personalized_learning_paths(student_id, limit.min(3))?;
        let generated_shelves = self.build_generated_shelves(student_id, limit)?;

        Ok(LibraryHomeSnapshot {
            due_now_count: pending_review_count + fading_concept_count + untouched_saved_count,
            pending_review_count,
            fading_concept_count,
            untouched_saved_count,
            continue_card: self.get_continue_learning_card(student_id)?,
            learning_paths,
            generated_shelves,
            saved_questions: self.list_saved_questions(student_id, limit)?,
        })
    }

    pub fn search_library(
        &self,
        student_id: i64,
        input: &LibrarySearchInput,
        limit: usize,
    ) -> EcoachResult<Vec<LibrarySearchResult>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut seen = HashSet::new();
        let mut results = Vec::new();
        let query = input
            .query
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        for item in self.list_items(student_id)? {
            let (title, subtitle, topic_name, subject_name) = self.resolve_item_display_context(
                &item.item_type,
                item.item_ref_id,
                item.topic_id,
            )?;
            let result = LibrarySearchResult {
                item_type: item.item_type.clone(),
                item_ref_id: Some(item.item_ref_id),
                library_item_id: Some(item.id),
                title: title.clone(),
                subtitle: subtitle.clone(),
                state: Some(item.state.clone()),
                topic_id: item.topic_id,
                topic_name,
                subject_id: item.subject_id,
                subject_name,
                tags: item.tags.clone(),
                reason: self.search_reason_for_library_item(&item, &title),
                match_score: self.search_score_for_library_item(
                    &item,
                    query,
                    &title,
                    subtitle.as_deref(),
                ),
                metadata: json!({
                    "urgency_score": item.urgency_score,
                    "difficulty_bp": item.difficulty_bp,
                    "exam_frequency_bp": item.exam_frequency_bp,
                    "goal_id": item.goal_id,
                    "calendar_event_id": item.calendar_event_id,
                }),
            };
            if self.matches_search_filters(&result, input) {
                let key = format!(
                    "{}:{}:{}",
                    result.item_type,
                    result.library_item_id.unwrap_or(0),
                    result.item_ref_id.unwrap_or(0)
                );
                if seen.insert(key) {
                    results.push(result);
                }
            }
        }

        self.push_topic_search_results(student_id, input, query, &mut seen, &mut results)?;
        self.push_entry_search_results(input, query, &mut seen, &mut results)?;
        self.push_question_search_results(student_id, input, query, &mut seen, &mut results)?;
        self.push_family_search_results(student_id, input, query, &mut seen, &mut results)?;

        results.sort_by(|left, right| {
            right
                .match_score
                .cmp(&left.match_score)
                .then(left.title.cmp(&right.title))
                .then(left.item_type.cmp(&right.item_type))
        });
        results.truncate(limit);
        Ok(results)
    }

    pub fn list_exam_hotspots(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<ExamHotspot>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    frm.family_id,
                    qf.family_name,
                    frm.subject_id,
                    qf.topic_id,
                    t.name,
                    frm.recurrence_rate_bp,
                    frm.persistence_score_bp,
                    frm.current_relevance_bp,
                    sfp.accuracy_rate_bp,
                    frm.last_appearance_year,
                    frm.first_appearance_year
                 FROM family_recurrence_metrics frm
                 INNER JOIN question_families qf ON qf.id = frm.family_id
                 LEFT JOIN topics t ON t.id = qf.topic_id
                 LEFT JOIN student_family_performance sfp
                    ON sfp.student_id = ?1
                   AND sfp.family_id = frm.family_id
                 WHERE (?2 IS NULL OR frm.subject_id = ?2)
                 ORDER BY
                    frm.current_relevance_bp DESC,
                    frm.recurrence_rate_bp DESC,
                    frm.persistence_score_bp DESC,
                    frm.family_id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id, limit as i64], |row| {
                let recurrence_rate_bp: BasisPoints = row.get(5)?;
                let persistence_score_bp: BasisPoints = row.get(6)?;
                let current_relevance_bp: BasisPoints = row.get(7)?;
                let last_appearance_year: Option<i64> = row.get(9)?;
                Ok(ExamHotspot {
                    family_id: row.get(0)?,
                    family_name: row.get(1)?,
                    subject_id: row.get(2)?,
                    topic_id: row.get(3)?,
                    topic_name: row.get(4)?,
                    recurrence_rate_bp,
                    persistence_score_bp,
                    current_relevance_bp,
                    student_accuracy_bp: row.get(8)?,
                    last_appearance_year,
                    first_appearance_year: row.get(10)?,
                    reason: format!(
                        "Appears often ({} bp), stays persistent ({} bp), and was last seen in {}.",
                        recurrence_rate_bp,
                        persistence_score_bp,
                        last_appearance_year
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "an unknown year".to_string())
                    ),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut hotspots = Vec::new();
        for row in rows {
            hotspots.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(hotspots)
    }

    pub fn get_topic_library_snapshot(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<TopicLibrarySnapshot> {
        let (topic_name, subject_id, subject_name, exam_weight): (String, i64, String, i64) = self
            .conn
            .query_row(
                "SELECT t.name, t.subject_id, s.name, t.exam_weight
                 FROM topics t
                 INNER JOIN subjects s ON s.id = t.subject_id
                 WHERE t.id = ?1",
                [topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|err| EcoachError::NotFound(format!("topic not found: {}", err)))?;

        let state = self
            .conn
            .query_row(
                "SELECT mastery_score, gap_score, mastery_state
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let saved_items = self
            .list_items(student_id)?
            .into_iter()
            .filter(|item| item.topic_id == Some(topic_id))
            .take(limit)
            .collect::<Vec<_>>();
        let notes = self.list_library_notes(student_id, Some(topic_id), None, limit)?;
        let relationship_hints = self.list_topic_relationship_hints(topic_id, limit.min(6))?;
        let exam_hotspots = self
            .list_exam_hotspots(student_id, Some(subject_id), limit)?
            .into_iter()
            .filter(|item| item.topic_id == Some(topic_id))
            .collect::<Vec<_>>();
        let weak_diagnoses = self.list_recent_diagnoses(student_id, topic_id, limit.min(5))?;
        let question_family_names = self.list_question_family_names_for_topic(topic_id, limit)?;
        let formula_titles = self.list_formula_titles_for_topic(topic_id, limit.min(5))?;
        let concept_chain_titles = relationship_hints
            .iter()
            .map(|hint| format!("{} -> {}", hint.from_title, hint.to_title))
            .collect::<Vec<_>>();
        let recommended_actions = self.topic_snapshot_recommended_actions(
            state.as_ref().map(|value| value.0),
            state.as_ref().map(|value| value.1),
            notes.len() as i64,
            exam_hotspots.len() as i64,
        );

        Ok(TopicLibrarySnapshot {
            topic_id,
            topic_name,
            subject_id,
            subject_name,
            mastery_score: state.as_ref().map(|value| value.0),
            gap_score: state.as_ref().map(|value| value.1),
            mastery_state: state.map(|value| value.2),
            exam_weight: exam_weight.clamp(0, 10_000) as BasisPoints,
            saved_item_count: saved_items.len() as i64,
            note_count: notes.len() as i64,
            weak_diagnoses,
            question_family_names,
            formula_titles,
            concept_chain_titles,
            saved_items,
            notes,
            relationship_hints,
            exam_hotspots,
            recommended_actions,
        })
    }

    pub fn create_custom_shelf(
        &self,
        student_id: i64,
        input: &CreateCustomShelfInput,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO library_shelves (
                    student_id,
                    shelf_type,
                    title,
                    description,
                    icon_hint,
                    generated,
                    priority_order,
                    last_refreshed_at
                 ) VALUES (?1, 'custom', ?2, ?3, ?4, 0, 50, datetime('now'))",
                params![student_id, input.title, input.description, input.icon_hint],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn add_item_to_custom_shelf(
        &self,
        student_id: i64,
        shelf_id: i64,
        input: &AddShelfItemInput,
    ) -> EcoachResult<i64> {
        let shelf_exists = self
            .conn
            .query_row(
                "SELECT 1
                 FROM library_shelves
                 WHERE id = ?1 AND student_id = ?2 AND generated = 0",
                params![shelf_id, student_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .is_some();
        if !shelf_exists {
            return Err(EcoachError::NotFound(
                "custom shelf not found for student".to_string(),
            ));
        }

        let metadata_json = serde_json::to_string(&input.metadata)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let sequence_order: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sequence_order), -1) + 1
                 FROM library_shelf_items
                 WHERE shelf_id = ?1",
                [shelf_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO library_shelf_items (
                    shelf_id,
                    item_type,
                    item_ref_id,
                    title,
                    subtitle,
                    reason,
                    rank_score,
                    metadata_json,
                    sequence_order
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    shelf_id,
                    input.item_type,
                    input.item_ref_id,
                    input.title,
                    input.subtitle,
                    input.reason,
                    input.rank_score,
                    metadata_json,
                    sequence_order,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let shelf_item_id = self.conn.last_insert_rowid();
        self.conn
            .execute(
                "UPDATE library_shelves
                 SET item_count = item_count + 1,
                     last_refreshed_at = datetime('now')
                 WHERE id = ?1",
                [shelf_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(shelf_item_id)
    }

    pub fn list_custom_shelves(
        &self,
        student_id: i64,
        include_items: bool,
        item_limit: usize,
    ) -> EcoachResult<Vec<CustomLibraryShelf>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, shelf_type, title, description, icon_hint, generated, item_count
                 FROM library_shelves
                 WHERE student_id = ?1 AND generated = 0
                 ORDER BY priority_order ASC, title ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i64>(5)? == 1,
                    row.get::<_, i64>(6)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut shelves = Vec::new();
        for row in rows {
            let (id, shelf_type, title, description, icon_hint, generated, item_count) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let items = if include_items {
                self.list_shelf_items(id, item_limit)?
            } else {
                Vec::new()
            };
            shelves.push(CustomLibraryShelf {
                id,
                shelf_type,
                title,
                description,
                icon_hint,
                generated,
                item_count,
                items,
            });
        }
        Ok(shelves)
    }

    pub fn list_offline_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<OfflineLibraryItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    lia.library_item_id,
                    li.item_type,
                    li.item_ref_id,
                    MAX(lia.created_at),
                    lia.context_json
                 FROM library_item_actions lia
                 INNER JOIN library_items li ON li.id = lia.library_item_id
                 WHERE lia.student_id = ?1
                   AND lia.action_type = 'downloaded_offline'
                 GROUP BY lia.library_item_id, li.item_type, li.item_ref_id, lia.context_json
                 ORDER BY MAX(lia.created_at) DESC, lia.library_item_id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    parse_value_sql(row.get(4)?)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            let (library_item_id, item_type, item_ref_id, downloaded_at, metadata) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let (title, _, topic_name, _) =
                self.resolve_item_display_context(&item_type, item_ref_id, None)?;
            items.push(OfflineLibraryItem {
                library_item_id,
                item_type,
                item_ref_id,
                title,
                topic_name,
                downloaded_at,
                metadata,
            });
        }
        Ok(items)
    }

    pub fn build_personalized_learning_paths(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<PersonalizedLearningPath>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let now = Utc::now().to_rfc3339();
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    sts.topic_id,
                    t.name,
                    sts.mastery_score,
                    sts.gap_score,
                    sts.priority_score,
                    sts.trend_state,
                    sts.decay_risk,
                    sts.is_blocked,
                    sts.next_review_at
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                 ORDER BY
                    sts.is_blocked DESC,
                    CASE
                        WHEN sts.next_review_at IS NOT NULL AND sts.next_review_at <= ?2 THEN 0
                        ELSE 1
                    END,
                    sts.priority_score DESC,
                    sts.gap_score DESC,
                    sts.mastery_score ASC,
                    sts.topic_id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, now, limit as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)? == 1,
                    row.get::<_, Option<String>>(8)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut paths = Vec::new();
        for row in rows {
            let (
                topic_id,
                topic_name,
                mastery_score,
                gap_score,
                priority_score,
                trend_state,
                decay_risk,
                is_blocked,
                next_review_at,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let relationship_hints = self.list_topic_relationship_hints(topic_id, 4)?;
            let related_topic_names = self.related_topic_names_for_hints(&relationship_hints)?;
            let bundle_sequence = self.list_bundle_sequence_for_topic(student_id, topic_id, 2)?;
            let linked_question_ids =
                self.list_saved_question_ids_for_topic(student_id, topic_id, 2)?;
            let practice_question_id = linked_question_ids.first().copied().or_else(|| {
                self.list_active_question_ids_for_topic(topic_id, 1)
                    .ok()
                    .and_then(|ids| ids.first().copied())
            });

            let activity_type = self.learning_path_activity_type(
                mastery_score,
                gap_score,
                decay_risk,
                is_blocked,
                &trend_state,
            );
            let reason = self.learning_path_reason(
                gap_score,
                decay_risk,
                is_blocked,
                next_review_at.is_some(),
                &trend_state,
                &bundle_sequence,
                relationship_hints.first(),
            );
            let steps = self.build_learning_path_steps(
                topic_id,
                &topic_name,
                &activity_type,
                &relationship_hints,
                &bundle_sequence,
                practice_question_id,
            );

            paths.push(PersonalizedLearningPath {
                topic_id,
                topic_name,
                activity_type,
                priority_score: priority_score.clamp(0, 10_000) as BasisPoints,
                reason,
                mastery_score,
                gap_score,
                recommended_bundle_ids: bundle_sequence
                    .iter()
                    .map(|bundle| bundle.bundle_id)
                    .collect(),
                recommended_bundle_titles: bundle_sequence
                    .iter()
                    .map(|bundle| bundle.title.clone())
                    .collect(),
                related_topic_names,
                relationship_hints,
                steps,
            });
        }

        Ok(paths)
    }

    pub fn refresh_generated_shelves(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<GeneratedLibraryShelf>> {
        let mut shelves = self.build_generated_shelves(student_id, limit)?;

        self.conn
            .execute(
                "DELETE FROM library_shelf_items
                 WHERE shelf_id IN (
                    SELECT id FROM library_shelves WHERE student_id = ?1 AND generated = 1
                 )",
                [student_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM library_shelves WHERE student_id = ?1 AND generated = 1",
                [student_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for shelf in &mut shelves {
            let rule_id = self.lookup_shelf_rule_id(&shelf.shelf_type)?;
            self.conn
                .execute(
                    "INSERT INTO library_shelves (
                        student_id,
                        shelf_type,
                        title,
                        description,
                        icon_hint,
                        generated,
                        item_count,
                        priority_order,
                        last_refreshed_at,
                        rule_id
                     ) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, datetime('now'), ?8)",
                    params![
                        student_id,
                        shelf.shelf_type,
                        shelf.title,
                        shelf.description,
                        shelf.icon_hint,
                        shelf.items.len() as i64,
                        shelf.priority_order,
                        rule_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let shelf_id = self.conn.last_insert_rowid();
            shelf.shelf_id = Some(shelf_id);

            for (index, item) in shelf.items.iter().enumerate() {
                let metadata_json = serde_json::to_string(&item.metadata)
                    .map_err(|err| EcoachError::Serialization(err.to_string()))?;
                self.conn
                    .execute(
                        "INSERT INTO library_shelf_items (
                            shelf_id,
                            item_type,
                            item_ref_id,
                            title,
                            subtitle,
                            reason,
                            rank_score,
                            metadata_json,
                            sequence_order
                         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            shelf_id,
                            item.item_type,
                            item.item_ref_id,
                            item.title,
                            item.subtitle,
                            item.reason,
                            item.rank_score,
                            metadata_json,
                            index as i64,
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
        }

        Ok(shelves)
    }

    pub fn build_revision_pack(
        &self,
        student_id: i64,
        title: &str,
        question_limit: usize,
    ) -> EcoachResult<RevisionPackSummary> {
        self.build_revision_pack_from_template(
            student_id,
            &BuildRevisionPackFromTemplateInput {
                template_code: "weak_area".to_string(),
                title: Some(title.to_string()),
                item_limit: Some(question_limit),
                subject_id: None,
            },
        )
    }

    pub fn list_revision_packs(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<RevisionPackSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    rp.id,
                    rp.title,
                    rp.source_type,
                    rpt.template_code,
                    rp.question_count,
                    rp.estimated_minutes,
                    rp.difficulty_profile,
                    rp.status,
                    rp.metadata_json,
                    rp.created_at
                 FROM revision_packs rp
                 LEFT JOIN revision_pack_templates rpt ON rpt.id = rp.template_id
                 WHERE rp.student_id = ?1
                 ORDER BY rp.created_at DESC, rp.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    parse_value_sql(row.get(8)?)?,
                    row.get::<_, String>(9)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut packs = Vec::new();
        for row in rows {
            let (
                pack_id,
                title,
                source_type,
                template_code,
                question_count,
                estimated_minutes,
                difficulty_profile,
                status,
                metadata,
                created_at_raw,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            packs.push(RevisionPackSummary {
                pack_id,
                title,
                source_type,
                template_code,
                topic_ids: metadata
                    .get("topic_ids")
                    .and_then(|value| value.as_array())
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|value| value.as_i64())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
                question_count,
                estimated_minutes,
                difficulty_profile,
                status,
                created_at: parse_datetime(&created_at_raw)?,
            });
        }
        Ok(packs)
    }

    pub fn list_revision_pack_templates(&self) -> EcoachResult<Vec<RevisionPackTemplate>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    id,
                    template_code,
                    display_name,
                    description,
                    pack_type,
                    selection_strategy,
                    default_item_count,
                    difficulty_profile,
                    topic_scope,
                    include_explanations,
                    include_worked_examples,
                    time_estimate_minutes
                 FROM revision_pack_templates
                 WHERE is_active = 1
                 ORDER BY display_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok(RevisionPackTemplate {
                    id: row.get(0)?,
                    template_code: row.get(1)?,
                    display_name: row.get(2)?,
                    description: row.get(3)?,
                    pack_type: row.get(4)?,
                    selection_strategy: row.get(5)?,
                    default_item_count: row.get(6)?,
                    difficulty_profile: row.get(7)?,
                    topic_scope: row.get(8)?,
                    include_explanations: row.get::<_, i64>(9)? == 1,
                    include_worked_examples: row.get::<_, i64>(10)? == 1,
                    time_estimate_minutes: row.get(11)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut templates = Vec::new();
        for row in rows {
            templates.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(templates)
    }

    pub fn build_revision_pack_from_template(
        &self,
        student_id: i64,
        input: &BuildRevisionPackFromTemplateInput,
    ) -> EcoachResult<RevisionPackSummary> {
        let template = self
            .find_revision_pack_template(&input.template_code)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "revision pack template not found: {}",
                    input.template_code
                ))
            })?;
        let item_limit = input
            .item_limit
            .unwrap_or(template.default_item_count.max(1) as usize);
        let (question_ids, topic_ids, source_type) = self.select_question_ids_for_template(
            student_id,
            &template,
            item_limit,
            input.subject_id,
        )?;
        if question_ids.is_empty() {
            return Err(EcoachError::Validation(
                "no questions were available for the selected revision pack template".to_string(),
            ));
        }
        let title = input
            .title
            .clone()
            .unwrap_or_else(|| template.display_name.clone());
        self.insert_revision_pack(
            student_id,
            &title,
            Some(&template),
            &source_type,
            topic_ids,
            question_ids,
            input.subject_id,
        )
    }

    pub fn create_custom_revision_pack(
        &self,
        student_id: i64,
        title: &str,
        question_ids: &[i64],
        subject_id: Option<i64>,
    ) -> EcoachResult<RevisionPackSummary> {
        if question_ids.is_empty() {
            return Err(EcoachError::Validation(
                "custom revision packs need at least one question".to_string(),
            ));
        }
        let topic_ids = self.collect_topic_ids_for_questions(question_ids)?;
        self.insert_revision_pack(
            student_id,
            title,
            None,
            "custom",
            topic_ids,
            question_ids.to_vec(),
            subject_id,
        )
    }

    pub fn list_revision_pack_items(&self, pack_id: i64) -> EcoachResult<Vec<RevisionPackItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, item_type, item_ref_id, sequence_order, required, metadata_json
                 FROM revision_pack_items
                 WHERE pack_id = ?1
                 ORDER BY sequence_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut rows = statement
            .query([pack_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            let metadata_json: String = row
                .get(5)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            items.push(RevisionPackItem {
                id: row
                    .get(0)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                item_type: row
                    .get(1)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                item_ref_id: row
                    .get(2)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                sequence_order: row
                    .get(3)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                required: row
                    .get::<_, i64>(4)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?
                    == 1,
                metadata: parse_value(&metadata_json)?,
            });
        }
        Ok(items)
    }

    pub fn build_teach_action_plan(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<TeachActionPlan> {
        let topic_name: String = self
            .conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get(0)
            })
            .map_err(|err| EcoachError::NotFound(format!("topic not found: {}", err)))?;

        let (mastery_score, gap_score, trend_state, decay_risk, is_blocked, next_review_at): (
            i64,
            i64,
            String,
            i64,
            bool,
            Option<String>,
        ) = self
            .conn
            .query_row(
                "SELECT
                    mastery_score,
                    gap_score,
                    trend_state,
                    decay_risk,
                    is_blocked,
                    next_review_at
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get::<_, i64>(4)? == 1,
                        row.get(5)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((0, 10_000, "critical".to_string(), 0, false, None));

        let fragile_memory_count = self.count_scalar(
            "SELECT COUNT(*)
             FROM memory_states
             WHERE student_id = ?1
               AND topic_id = ?2
               AND memory_state IN ('fragile', 'at_risk', 'fading', 'collapsed', 'rebuilding')",
            params![student_id, topic_id],
        )?;

        let review_due = next_review_at.is_some();
        let action_type = if is_blocked || gap_score >= 6500 {
            "reteach_foundation"
        } else if fragile_memory_count > 0 || decay_risk >= 6000 || review_due {
            "memory_reactivation"
        } else if mastery_score < 5500
            || matches!(trend_state.as_str(), "fragile" | "declining" | "critical")
        {
            "guided_practice"
        } else {
            "exam_linking"
        }
        .to_string();
        let readiness_band = readiness_band_for_action(&action_type).to_string();
        let support_intensity = self
            .support_intensity_for_topic(
                mastery_score,
                gap_score,
                fragile_memory_count,
                decay_risk,
                is_blocked,
                &trend_state,
            )
            .to_string();

        let mut linked_question_ids =
            self.list_saved_question_ids_for_topic(student_id, topic_id, limit)?;
        if linked_question_ids.len() < limit {
            for question_id in self.list_active_question_ids_for_topic(
                topic_id,
                limit.saturating_sub(linked_question_ids.len()),
            )? {
                if !linked_question_ids.contains(&question_id) {
                    linked_question_ids.push(question_id);
                }
            }
        }

        let linked_entries = self.list_topic_entries(topic_id, limit)?;
        let linked_entry_ids = linked_entries.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let linked_entry_titles = linked_entries
            .iter()
            .map(|(_, title)| title.clone())
            .collect::<Vec<_>>();
        let target_node_titles = self.list_target_node_titles(topic_id, limit)?;
        let relationship_hints = self.list_topic_relationship_hints(topic_id, limit)?;
        let diagnostic_focuses = self.list_diagnostic_focuses(student_id, topic_id, 4)?;
        let recent_diagnoses = self.list_recent_diagnoses(student_id, topic_id, 3)?;
        let primary_prompt = self.build_teach_prompt(
            &topic_name,
            &action_type,
            mastery_score,
            gap_score,
            fragile_memory_count,
            &linked_entry_titles,
            &target_node_titles,
        );
        let recommended_sequence = self.build_teach_sequence(
            &topic_name,
            &action_type,
            &support_intensity,
            fragile_memory_count,
            &linked_question_ids,
            &linked_entry_ids,
            &linked_entry_titles,
            &target_node_titles,
            &relationship_hints,
            &diagnostic_focuses,
        );

        Ok(TeachActionPlan {
            student_id,
            topic_id,
            topic_name,
            action_type,
            readiness_band,
            support_intensity,
            mastery_score,
            gap_score,
            fragile_memory_count,
            primary_prompt,
            diagnostic_focuses,
            recent_diagnoses,
            linked_question_ids,
            linked_entry_ids,
            linked_entry_titles,
            target_node_titles,
            relationship_hints,
            recommended_sequence,
        })
    }

    pub fn upsert_teach_explanation(
        &self,
        node_id: i64,
        input: &TeachExplanationUpsertInput,
    ) -> EcoachResult<i64> {
        if input.explanation_level.trim().is_empty() {
            return Err(EcoachError::Validation(
                "explanation level cannot be empty".to_string(),
            ));
        }

        let structured_breakdown_json = serde_json::to_string(&input.structured_breakdown)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let worked_examples_json = serde_json::to_string(&input.worked_examples)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let common_mistakes_json = serde_json::to_string(&input.common_mistakes)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let related_concepts_json = serde_json::to_string(&input.related_concepts)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let visual_asset_refs_json = serde_json::to_string(&input.visual_asset_refs)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO teach_explanations (
                    node_id,
                    explanation_level,
                    hero_summary,
                    why_it_matters,
                    simple_explanation,
                    structured_breakdown_json,
                    worked_examples_json,
                    common_mistakes_json,
                    exam_appearance_notes,
                    pattern_recognition_tips,
                    related_concepts_json,
                    visual_asset_refs_json,
                    subject_style
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ON CONFLICT(node_id, explanation_level) DO UPDATE SET
                    hero_summary = excluded.hero_summary,
                    why_it_matters = excluded.why_it_matters,
                    simple_explanation = excluded.simple_explanation,
                    structured_breakdown_json = excluded.structured_breakdown_json,
                    worked_examples_json = excluded.worked_examples_json,
                    common_mistakes_json = excluded.common_mistakes_json,
                    exam_appearance_notes = excluded.exam_appearance_notes,
                    pattern_recognition_tips = excluded.pattern_recognition_tips,
                    related_concepts_json = excluded.related_concepts_json,
                    visual_asset_refs_json = excluded.visual_asset_refs_json,
                    subject_style = excluded.subject_style,
                    updated_at = datetime('now')",
                params![
                    node_id,
                    input.explanation_level.trim(),
                    input.hero_summary.as_deref(),
                    input.why_it_matters.as_deref(),
                    input.simple_explanation.as_deref(),
                    structured_breakdown_json,
                    worked_examples_json,
                    common_mistakes_json,
                    input.exam_appearance_notes.as_deref(),
                    input.pattern_recognition_tips.as_deref(),
                    related_concepts_json,
                    visual_asset_refs_json,
                    input.subject_style.as_deref(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let id = self
            .conn
            .query_row(
                "SELECT id FROM teach_explanations
                 WHERE node_id = ?1 AND explanation_level = ?2",
                params![node_id, input.explanation_level.trim()],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(id)
    }

    pub fn add_teach_micro_check(
        &self,
        explanation_id: i64,
        input: &TeachMicroCheckInput,
    ) -> EcoachResult<i64> {
        if input.check_type.trim().is_empty() {
            return Err(EcoachError::Validation(
                "micro-check type cannot be empty".to_string(),
            ));
        }
        if input.prompt.trim().is_empty() {
            return Err(EcoachError::Validation(
                "micro-check prompt cannot be empty".to_string(),
            ));
        }
        if input.correct_answer.trim().is_empty() {
            return Err(EcoachError::Validation(
                "micro-check answer cannot be empty".to_string(),
            ));
        }

        let distractor_answers_json = serde_json::to_string(&input.distractor_answers)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO teach_micro_checks (
                    explanation_id,
                    check_type,
                    prompt,
                    correct_answer,
                    distractor_answers_json,
                    explanation_if_wrong,
                    position_index
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    explanation_id,
                    input.check_type.trim(),
                    input.prompt.trim(),
                    input.correct_answer.trim(),
                    distractor_answers_json,
                    input.explanation_if_wrong.as_deref(),
                    input.position_index,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_teach_lesson(
        &self,
        topic_id: i64,
        explanation_level: Option<&str>,
        micro_check_limit: usize,
    ) -> EcoachResult<TeachLesson> {
        let topic_name = self.topic_name(topic_id)?;
        let node = self.primary_teach_node_for_topic(topic_id)?;
        let explanation_level = explanation_level
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("core");
        let question_summaries =
            self.list_question_summaries_for_topic(topic_id, micro_check_limit.max(1))?;

        let (mut explanation, generated) = if let Some(node) = node.as_ref() {
            if let Some(explanation) =
                self.load_teach_explanation_for_node(node.node_id, Some(explanation_level))?
            {
                (explanation, false)
            } else {
                (
                    self.build_synthesized_teach_explanation(
                        topic_id,
                        &topic_name,
                        node.node_id,
                        &node.node_title,
                        explanation_level,
                        &question_summaries,
                    )?,
                    true,
                )
            }
        } else {
            (
                self.build_synthesized_teach_explanation(
                    topic_id,
                    &topic_name,
                    topic_id,
                    &topic_name,
                    explanation_level,
                    &question_summaries,
                )?,
                true,
            )
        };

        let node_id = node.as_ref().map(|node| node.node_id);
        let node_title = node.as_ref().map(|node| node.node_title.clone());
        let mut micro_checks = if generated {
            self.build_synthesized_micro_checks(
                explanation.id,
                &topic_name,
                explanation
                    .simple_explanation
                    .as_deref()
                    .or(explanation.hero_summary.as_deref()),
                &question_summaries,
                micro_check_limit,
            )
        } else {
            self.load_teach_micro_checks(explanation.id, micro_check_limit)?
        };

        if !generated && micro_checks.len() < micro_check_limit {
            let extra = self.build_synthesized_micro_checks(
                explanation.id,
                &topic_name,
                explanation
                    .simple_explanation
                    .as_deref()
                    .or(explanation.hero_summary.as_deref()),
                &question_summaries,
                micro_check_limit - micro_checks.len(),
            );
            micro_checks.extend(extra);
        }

        micro_checks.truncate(micro_check_limit);
        explanation.topic_id = Some(topic_id);
        explanation.topic_name = Some(topic_name.clone());

        Ok(TeachLesson {
            topic_id,
            topic_name,
            node_id,
            node_title,
            explanation_level: explanation.explanation_level.clone(),
            explanation: Some(explanation),
            micro_checks,
            generated,
        })
    }

    pub fn ask_tutor(&self, input: &TutorInteractionInput) -> EcoachResult<TutorResponse> {
        let question_context = if let Some(question_id) = input.question_id {
            self.load_question_tutor_context(question_id)?
        } else {
            None
        };

        let topic_id = input
            .topic_id
            .or_else(|| question_context.as_ref().and_then(|ctx| ctx.topic_id));
        let topic_name = if let Some(topic_id) = topic_id {
            Some(self.topic_name(topic_id)?)
        } else {
            None
        };
        let lesson = if let Some(topic_id) = topic_id {
            Some(self.get_teach_lesson(topic_id, Some("core"), 3)?)
        } else {
            None
        };
        let recent_history = self.list_recent_tutor_interactions(input.student_id, 3)?;
        let recent_diagnoses = if let Some(topic_id) = topic_id {
            self.list_recent_diagnoses(input.student_id, topic_id, 3)?
        } else {
            Vec::new()
        };
        let related_topic_names = if let Some(topic_id) = topic_id {
            let hints = self.list_topic_relationship_hints(topic_id, 3)?;
            self.related_topic_names_for_hints(&hints)?
        } else {
            Vec::new()
        };
        let related_entry_ids = if let Some(topic_id) = topic_id {
            self.list_topic_entries(topic_id, 3)?
                .into_iter()
                .map(|(entry_id, _)| entry_id)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let related_question_ids = if let Some(topic_id) = topic_id {
            self.collect_related_question_ids(input.student_id, topic_id, input.question_id, 3)?
        } else {
            input.question_id.into_iter().collect::<Vec<_>>()
        };
        let student_state = if let Some(topic_id) = topic_id {
            self.load_student_topic_state(input.student_id, topic_id)?
        } else {
            None
        };

        let response_text = self.build_tutor_response_text(
            input,
            topic_name.as_deref(),
            lesson.as_ref(),
            question_context.as_ref(),
            &recent_history,
            &recent_diagnoses,
            student_state.as_ref(),
        );
        let context_summary = self.build_tutor_context_summary(
            input,
            topic_name.as_deref(),
            lesson.as_ref(),
            question_context.as_ref(),
            &recent_history,
            &recent_diagnoses,
            student_state.as_ref(),
        );
        let suggested_next_steps = self.build_tutor_next_steps(
            input,
            lesson.as_ref(),
            question_context.as_ref(),
            &recent_diagnoses,
            student_state.as_ref(),
        );

        Ok(TutorResponse {
            student_id: input.student_id,
            topic_id,
            question_id: input.question_id,
            interaction_type: input.interaction_type.clone(),
            prompt_text: input.prompt_text.clone(),
            response_text,
            context_summary,
            suggested_next_steps,
            related_question_ids,
            related_entry_ids,
            related_topic_names,
        })
    }

    pub fn log_tutor_interaction(&self, input: &TutorInteractionInput) -> EcoachResult<i64> {
        let context_json = serde_json::to_string(&input.context)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO tutor_interactions (
                    student_id,
                    session_id,
                    question_id,
                    topic_id,
                    interaction_type,
                    prompt_text,
                    response_text,
                    context_json,
                    was_helpful
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    input.student_id,
                    input.session_id,
                    input.question_id,
                    input.topic_id,
                    input.interaction_type.as_str(),
                    input.prompt_text.as_deref(),
                    input.response_text.as_deref(),
                    context_json,
                    input.was_helpful.map(|value| value as i64),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_recent_tutor_interactions(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<TutorInteraction>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    id,
                    student_id,
                    session_id,
                    question_id,
                    topic_id,
                    interaction_type,
                    prompt_text,
                    response_text,
                    context_json,
                    was_helpful,
                    created_at
                 FROM tutor_interactions
                 WHERE student_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(TutorInteraction {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    session_id: row.get(2)?,
                    question_id: row.get(3)?,
                    topic_id: row.get(4)?,
                    interaction_type: row.get(5)?,
                    prompt_text: row.get(6)?,
                    response_text: row.get(7)?,
                    context: parse_value_sql(row.get::<_, String>(8)?)?,
                    was_helpful: row.get::<_, Option<i64>>(9)?.map(|value| value != 0),
                    created_at: row.get(10)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut interactions = Vec::new();
        for row in rows {
            interactions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(interactions)
    }

    pub fn list_topic_relationship_hints(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<TopicRelationshipHint>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut hints = self.collect_direct_relationship_hints(topic_id, limit * 2)?;
        if hints.len() < limit {
            let root_topic_name = self.topic_name(topic_id)?;
            let bridge_topics = self.list_related_topics_for_topic(topic_id, limit)?;
            for bridge_topic in bridge_topics {
                if hints.len() >= limit {
                    break;
                }
                if let Some(path_hint) =
                    self.build_bridge_relationship_hint(topic_id, &root_topic_name, &bridge_topic)?
                {
                    push_unique_relationship_hint(&mut hints, path_hint);
                }
            }
        }

        hints.sort_by(|left, right| {
            right
                .strength_score
                .cmp(&left.strength_score)
                .then(left.hop_count.cmp(&right.hop_count))
                .then(left.relation_type.cmp(&right.relation_type))
        });
        hints.truncate(limit);
        Ok(hints)
    }

    fn build_generated_shelves(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<GeneratedLibraryShelf>> {
        let mut shelves = Vec::new();
        let due_now = self.build_due_now_shelf(student_id, limit)?;
        if !due_now.items.is_empty() {
            shelves.push(due_now);
        }

        let memory_shelf = self.make_generated_shelf(
            "memory_shelf",
            "Memory Shelf",
            Some("brain"),
            2,
            self.list_memory_shelf_items(student_id, limit)?,
        )?;
        if !memory_shelf.items.is_empty() {
            shelves.push(memory_shelf);
        }

        let mistake_bank = self.make_generated_shelf(
            "mistake_bank",
            "Mistake Bank",
            Some("warning"),
            3,
            self.list_mistake_bank_items(student_id, limit)?,
        )?;
        if !mistake_bank.items.is_empty() {
            shelves.push(mistake_bank);
        }

        let weak_topics = self.make_generated_shelf(
            "weak_topics",
            "My Weak Concepts",
            Some("trend_down"),
            4,
            self.list_weak_topic_shelf_items(student_id, limit)?,
        )?;
        if !weak_topics.items.is_empty() {
            shelves.push(weak_topics);
        }

        let saved_questions = self.make_generated_shelf(
            "saved_questions",
            "Saved Questions",
            Some("bookmark"),
            5,
            self.list_saved_question_shelf_items(student_id, limit)?,
        )?;
        if !saved_questions.items.is_empty() {
            shelves.push(saved_questions);
        }

        let exam_hotspots = self.make_generated_shelf(
            "exam_hotspots",
            "Exam Hotspots",
            Some("bolt"),
            6,
            self.list_exam_hotspot_shelf_items(student_id, limit)?,
        )?;
        if !exam_hotspots.items.is_empty() {
            shelves.push(exam_hotspots);
        }

        let near_wins = self.make_generated_shelf(
            "near_wins",
            "Almost There",
            Some("spark"),
            7,
            self.list_near_win_shelf_items(student_id, limit)?,
        )?;
        if !near_wins.items.is_empty() {
            shelves.push(near_wins);
        }

        let untouched_important = self.make_generated_shelf(
            "untouched_important",
            "Untouched but Important",
            Some("target"),
            8,
            self.list_untouched_important_items(student_id, limit)?,
        )?;
        if !untouched_important.items.is_empty() {
            shelves.push(untouched_important);
        }

        let things_i_forget = self.make_generated_shelf(
            "things_i_forget",
            "Things I Keep Forgetting",
            Some("history"),
            9,
            self.list_things_i_forget_items(student_id, limit)?,
        )?;
        if !things_i_forget.items.is_empty() {
            shelves.push(things_i_forget);
        }

        let teach_me_again = self.make_generated_shelf(
            "teach_me_again",
            "Teach Me Again",
            Some("refresh"),
            10,
            self.list_teach_me_again_items(student_id, limit)?,
        )?;
        if !teach_me_again.items.is_empty() {
            shelves.push(teach_me_again);
        }

        let formula_bank = self.make_generated_shelf(
            "formula_bank",
            "Formula Bank",
            Some("function"),
            11,
            self.list_formula_bank_shelf_items(student_id, limit)?,
        )?;
        if !formula_bank.items.is_empty() {
            shelves.push(formula_bank);
        }

        let concept_chains = self.make_generated_shelf(
            "concept_chains",
            "Concept Chains",
            Some("link"),
            12,
            self.list_concept_chain_items(student_id, limit)?,
        )?;
        if !concept_chains.items.is_empty() {
            shelves.push(concept_chains);
        }

        Ok(shelves)
    }

    fn build_due_now_shelf(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<GeneratedLibraryShelf> {
        let mut items = Vec::new();

        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    cmm.id,
                    COALESCE(t.name, 'Mission review'),
                    cmm.next_action_type,
                    cmm.review_due_at,
                    cmm.accuracy_score
                 FROM coach_mission_memories cmm
                 LEFT JOIN topics t ON t.id = cmm.topic_id
                 WHERE cmm.student_id = ?1
                   AND cmm.review_status = 'pending'
                 ORDER BY
                    CASE WHEN cmm.review_due_at IS NULL THEN 1 ELSE 0 END,
                    cmm.review_due_at ASC,
                    cmm.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(LibraryShelfItem {
                    item_type: "mission_review".to_string(),
                    item_ref_id: Some(row.get(0)?),
                    title: format!("Review {}", row.get::<_, String>(1)?),
                    subtitle: row.get(2)?,
                    reason: "A completed mission still needs review and next-action confirmation."
                        .to_string(),
                    rank_score: row.get::<_, Option<i64>>(4)?.unwrap_or(8500) as BasisPoints,
                    metadata: json!({
                        "review_due_at": row.get::<_, Option<String>>(3)?,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        if items.len() < limit {
            let remaining = limit - items.len();
            items.extend(self.list_memory_shelf_items(student_id, remaining)?);
        }

        if items.len() < limit {
            let remaining = limit - items.len();
            let saved_questions = self.list_saved_questions(student_id, remaining)?;
            for question in saved_questions {
                if matches!(question.state.as_str(), "saved" | "new" | "revisit") {
                    items.push(LibraryShelfItem {
                        item_type: "question".to_string(),
                        item_ref_id: Some(question.question_id),
                        title: truncate_text(&question.stem, 72),
                        subtitle: Some(question.topic_name),
                        reason: "This saved question has not been fully worked through yet."
                            .to_string(),
                        rank_score: question.urgency_score,
                        metadata: json!({
                            "library_item_id": question.library_item_id,
                        }),
                    });
                }
                if items.len() >= limit {
                    break;
                }
            }
        }

        self.make_generated_shelf("due_now", "Due Now", Some("clock"), 1, items)
    }

    fn list_memory_shelf_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    ms.id,
                    ms.topic_id,
                    t.name,
                    ms.node_id,
                    an.canonical_title,
                    ms.memory_state,
                    ms.memory_strength,
                    ms.decay_risk,
                    ms.review_due_at
                 FROM memory_states ms
                 LEFT JOIN topics t ON t.id = ms.topic_id
                 LEFT JOIN academic_nodes an ON an.id = ms.node_id
                 WHERE ms.student_id = ?1
                 ORDER BY
                    CASE
                        WHEN ms.review_due_at IS NOT NULL AND ms.review_due_at <= ?2 THEN 0
                        WHEN ms.memory_state IN ('collapsed', 'fading', 'at_risk', 'fragile') THEN 1
                        ELSE 2
                    END,
                    ms.decay_risk DESC,
                    ms.review_due_at ASC,
                    ms.id DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(
                params![student_id, Utc::now().to_rfc3339(), limit as i64],
                |row| {
                    let topic_name: Option<String> = row.get(2)?;
                    let node_title: Option<String> = row.get(4)?;
                    let memory_state: String = row.get(5)?;
                    let title = node_title
                        .or_else(|| topic_name.clone())
                        .unwrap_or_else(|| "Memory item".to_string());

                    Ok(LibraryShelfItem {
                        item_type: "memory".to_string(),
                        item_ref_id: row.get(0)?,
                        title,
                        subtitle: topic_name,
                        reason: format!(
                            "Memory state is {} and needs reinforcement.",
                            memory_state
                        ),
                        rank_score: row.get::<_, i64>(7)? as BasisPoints,
                        metadata: json!({
                            "topic_id": row.get::<_, Option<i64>>(1)?,
                            "node_id": row.get::<_, Option<i64>>(3)?,
                            "memory_state": memory_state,
                            "memory_strength": row.get::<_, i64>(6)?,
                            "review_due_at": row.get::<_, Option<String>>(8)?,
                        }),
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_mistake_bank_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    wad.topic_id,
                    t.name,
                    wad.primary_diagnosis,
                    COUNT(*) AS occurrence_count,
                    SUM(
                        CASE wad.severity
                            WHEN 'high' THEN 3000
                            WHEN 'medium' THEN 2000
                            ELSE 1000
                        END
                    ) AS score
                 FROM wrong_answer_diagnoses wad
                 LEFT JOIN topics t ON t.id = wad.topic_id
                 WHERE wad.student_id = ?1
                 GROUP BY wad.topic_id, t.name, wad.primary_diagnosis
                 ORDER BY score DESC, occurrence_count DESC, wad.topic_id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let topic_name: Option<String> = row.get(1)?;
                let diagnosis: String = row.get(2)?;
                let occurrence_count: i64 = row.get(3)?;
                let raw_score: i64 = row.get(4)?;

                Ok(LibraryShelfItem {
                    item_type: "mistake_cluster".to_string(),
                    item_ref_id: row.get(0)?,
                    title: format!("{} x{}", diagnosis.replace('_', " "), occurrence_count),
                    subtitle: topic_name,
                    reason: "This error pattern is recurring and should be repaired directly."
                        .to_string(),
                    rank_score: raw_score.clamp(0, 10_000) as BasisPoints,
                    metadata: json!({
                        "primary_diagnosis": diagnosis,
                        "occurrence_count": occurrence_count,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_weak_topic_shelf_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    sts.topic_id,
                    t.name,
                    sts.mastery_score,
                    sts.gap_score,
                    sts.priority_score,
                    sts.mastery_state
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                 ORDER BY sts.priority_score DESC, sts.gap_score DESC, sts.mastery_score ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let topic_name: String = row.get(1)?;
                let mastery_state: String = row.get(5)?;
                Ok(LibraryShelfItem {
                    item_type: "topic".to_string(),
                    item_ref_id: Some(row.get(0)?),
                    title: topic_name,
                    subtitle: Some(format!("mastery {}", mastery_state)),
                    reason: "This topic still has a high gap and should shape revision priority."
                        .to_string(),
                    rank_score: row.get(4)?,
                    metadata: json!({
                        "mastery_score": row.get::<_, i64>(2)?,
                        "gap_score": row.get::<_, i64>(3)?,
                        "mastery_state": mastery_state,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn get_continue_learning_card(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<ContinueLearningCard>> {
        let active_mission = self
            .conn
            .query_row(
                "SELECT
                    m.id,
                    m.title,
                    m.activity_type,
                    m.primary_topic_id,
                    t.name
                 FROM coach_missions m
                 LEFT JOIN topics t ON t.id = m.primary_topic_id
                 WHERE m.student_id = ?1
                   AND m.status IN ('active', 'pending')
                 ORDER BY
                    CASE m.status WHEN 'active' THEN 0 ELSE 1 END,
                    m.created_at DESC,
                    m.id DESC
                 LIMIT 1",
                [student_id],
                |row| {
                    Ok(ContinueLearningCard {
                        title: row.get(1)?,
                        activity_type: row.get(2)?,
                        topic_id: row.get(3)?,
                        topic_name: row.get(4)?,
                        mission_id: Some(row.get(0)?),
                        session_id: None,
                        route: format!("/coach/missions/{}", row.get::<_, i64>(0)?),
                        reason: None,
                        priority_score: None,
                        recommended_bundle_ids: Vec::new(),
                        related_topic_names: Vec::new(),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if active_mission.is_some() {
            return Ok(active_mission);
        }

        self.conn
            .query_row(
                "SELECT
                    s.id,
                    s.session_type,
                    COALESCE(
                        si.source_topic_id,
                        (
                            SELECT q.topic_id
                            FROM session_items si2
                            INNER JOIN questions q ON q.id = si2.question_id
                            WHERE si2.session_id = s.id
                            ORDER BY si2.display_order ASC
                            LIMIT 1
                        )
                    ) AS topic_id,
                    t.name
                 FROM sessions s
                 LEFT JOIN session_items si
                    ON si.session_id = s.id
                   AND si.display_order = s.active_item_index
                 LEFT JOIN topics t
                    ON t.id = COALESCE(
                        si.source_topic_id,
                        (
                            SELECT q.topic_id
                            FROM session_items si3
                            INNER JOIN questions q ON q.id = si3.question_id
                            WHERE si3.session_id = s.id
                            ORDER BY si3.display_order ASC
                            LIMIT 1
                        )
                    )
                 WHERE s.student_id = ?1
                   AND s.status IN ('active', 'paused')
                 ORDER BY
                    CASE s.status WHEN 'active' THEN 0 ELSE 1 END,
                    COALESCE(s.last_activity_at, s.updated_at, s.created_at) DESC,
                    s.id DESC
                 LIMIT 1",
                [student_id],
                |row| {
                    let session_id: i64 = row.get(0)?;
                    let session_type: String = row.get(1)?;
                    Ok(ContinueLearningCard {
                        title: format!("Continue {}", session_type.replace('_', " ")),
                        activity_type: session_type,
                        topic_id: row.get(2)?,
                        topic_name: row.get(3)?,
                        mission_id: None,
                        session_id: Some(session_id),
                        route: format!("/sessions/{}", session_id),
                        reason: None,
                        priority_score: None,
                        recommended_bundle_ids: Vec::new(),
                        related_topic_names: Vec::new(),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .map_or_else(
                || {
                    let paths = self.build_personalized_learning_paths(student_id, 1)?;
                    Ok(paths.into_iter().next().map(|path| ContinueLearningCard {
                        title: format!("Resume {}", path.topic_name),
                        activity_type: path.activity_type,
                        topic_id: Some(path.topic_id),
                        topic_name: Some(path.topic_name),
                        mission_id: None,
                        session_id: None,
                        route: format!("/library/topics/{}", path.topic_id),
                        reason: Some(path.reason),
                        priority_score: Some(path.priority_score),
                        recommended_bundle_ids: path.recommended_bundle_ids,
                        related_topic_names: path.related_topic_names,
                    }))
                },
                |card| Ok(Some(card)),
            )
    }

    fn list_topic_entries(&self, topic_id: i64, limit: usize) -> EcoachResult<Vec<(i64, String)>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, title
                 FROM knowledge_entries
                 WHERE topic_id = ?1
                   AND status = 'active'
                 ORDER BY importance_score DESC, difficulty_level ASC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![topic_id, limit as i64], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(entries)
    }

    fn list_target_node_titles(&self, topic_id: i64, limit: usize) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT canonical_title
                 FROM academic_nodes
                 WHERE topic_id = ?1
                   AND is_active = 1
                 ORDER BY foundation_weight DESC, exam_relevance_score DESC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![topic_id, limit as i64], |row| row.get(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut titles = Vec::new();
        for row in rows {
            titles.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(titles)
    }

    fn build_teach_prompt(
        &self,
        topic_name: &str,
        action_type: &str,
        mastery_score: i64,
        gap_score: i64,
        fragile_memory_count: i64,
        linked_entry_titles: &[String],
        target_node_titles: &[String],
    ) -> String {
        let anchor_entry = linked_entry_titles
            .first()
            .cloned()
            .unwrap_or_else(|| "the key definition set".to_string());
        let anchor_node = target_node_titles
            .first()
            .cloned()
            .unwrap_or_else(|| "the core concept".to_string());
        match action_type {
            "reteach_foundation" => format!(
                "Reteach {} from first principles. Anchor on {}, then walk the learner through {} before moving to questions. Current mastery is {} and gap pressure is {}.",
                topic_name, anchor_entry, anchor_node, mastery_score, gap_score
            ),
            "memory_reactivation" => format!(
                "Reactivate {} through quick retrieval on {} and {}. There are {} fragile memory traces, so start with recall before fresh drilling.",
                topic_name, anchor_entry, anchor_node, fragile_memory_count
            ),
            "guided_practice" => format!(
                "Guide the learner through {} using {} as the explanation anchor, then shift into scaffolded practice around {}.",
                topic_name, anchor_entry, anchor_node
            ),
            _ => format!(
                "Link {} to exam-style performance. Use {} and {} to connect knowledge to timed question work.",
                topic_name, anchor_entry, anchor_node
            ),
        }
    }

    fn support_intensity_for_topic(
        &self,
        mastery_score: i64,
        gap_score: i64,
        fragile_memory_count: i64,
        decay_risk: i64,
        is_blocked: bool,
        trend_state: &str,
    ) -> &'static str {
        if is_blocked
            || gap_score >= 7500
            || matches!(trend_state, "critical")
            || mastery_score <= 2500
        {
            "high"
        } else if fragile_memory_count >= 2
            || gap_score >= 5000
            || decay_risk >= 5000
            || matches!(trend_state, "fragile" | "declining")
        {
            "medium"
        } else {
            "low"
        }
    }

    fn list_diagnostic_focuses(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<String>> {
        let mut focuses = Vec::new();
        let profile = self
            .conn
            .query_row(
                "SELECT
                    knowledge_gap_score,
                    conceptual_confusion_score,
                    recognition_failure_score,
                    execution_error_score,
                    carelessness_score,
                    pressure_breakdown_score,
                    expression_weakness_score,
                    speed_error_score
                 FROM student_error_profiles
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some((
            knowledge_gap_score,
            conceptual_confusion_score,
            recognition_failure_score,
            execution_error_score,
            carelessness_score,
            pressure_breakdown_score,
            expression_weakness_score,
            speed_error_score,
        )) = profile
        {
            if knowledge_gap_score >= 4500 {
                push_unique(
                    &mut focuses,
                    "Rebuild missing prerequisite knowledge before pushing for speed.".to_string(),
                );
            }
            if conceptual_confusion_score >= 4500 {
                push_unique(
                    &mut focuses,
                    "Contrast the nearby concepts explicitly so the learner can name the boundary."
                        .to_string(),
                );
            }
            if recognition_failure_score >= 4500 {
                push_unique(
                    &mut focuses,
                    "Train recognition cues first so the learner can spot when this idea is in play."
                        .to_string(),
                );
            }
            if execution_error_score >= 4500 {
                push_unique(
                    &mut focuses,
                    "Model the procedure step by step, then fade the scaffold once the path is stable."
                        .to_string(),
                );
            }
            if carelessness_score >= 4500 {
                push_unique(
                    &mut focuses,
                    "Require a deliberate final verification pass to catch avoidable slips."
                        .to_string(),
                );
            }
            if pressure_breakdown_score >= 4500 || speed_error_score >= 4500 {
                push_unique(
                    &mut focuses,
                    "Keep the pace calm and accurate first; only add speed after the method is stable."
                        .to_string(),
                );
            }
            if expression_weakness_score >= 4500 {
                push_unique(
                    &mut focuses,
                    "Ask the learner to explain each move out loud or in writing before moving on."
                        .to_string(),
                );
            }
        }

        if focuses.len() < limit {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT primary_diagnosis, recommended_action
                     FROM wrong_answer_diagnoses
                     WHERE student_id = ?1
                       AND topic_id = ?2
                     ORDER BY created_at DESC, id DESC
                     LIMIT ?3",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map(params![student_id, topic_id, limit as i64], |row| {
                    let diagnosis: String = row.get(0)?;
                    let action: String = row.get(1)?;
                    Ok(format!(
                        "Recent diagnosis {} suggests: {}.",
                        diagnosis, action
                    ))
                })
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                push_unique(
                    &mut focuses,
                    row.map_err(|err| EcoachError::Storage(err.to_string()))?,
                );
                if focuses.len() >= limit {
                    break;
                }
            }
        }

        focuses.truncate(limit);
        Ok(focuses)
    }

    fn list_recent_diagnoses(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT primary_diagnosis, severity, recommended_action
                 FROM wrong_answer_diagnoses
                 WHERE student_id = ?1
                   AND topic_id = ?2
                 ORDER BY created_at DESC, id DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, topic_id, limit as i64], |row| {
                let diagnosis: String = row.get(0)?;
                let severity: String = row.get(1)?;
                let action: String = row.get(2)?;
                Ok(format!("{} ({}) -> {}", diagnosis, severity, action))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut diagnoses = Vec::new();
        for row in rows {
            diagnoses.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(diagnoses)
    }

    fn primary_teach_node_for_topic(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Option<TeachNodeCandidate>> {
        self.conn
            .query_row(
                "SELECT
                    an.id,
                    an.canonical_title
                 FROM academic_nodes an
                 WHERE an.topic_id = ?1
                   AND an.is_active = 1
                 ORDER BY an.foundation_weight DESC, an.exam_relevance_score DESC, an.id ASC
                 LIMIT 1",
                [topic_id],
                |row| {
                    Ok(TeachNodeCandidate {
                        node_id: row.get(0)?,
                        node_title: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_teach_explanation_for_node(
        &self,
        node_id: i64,
        explanation_level: Option<&str>,
    ) -> EcoachResult<Option<TeachExplanation>> {
        if let Some(level) = explanation_level {
            if let Some(explanation) = self
                .conn
                .query_row(
                    "SELECT
                        te.id,
                        te.node_id,
                        an.topic_id,
                        t.name,
                        an.canonical_title,
                        te.explanation_level,
                        te.hero_summary,
                        te.why_it_matters,
                        te.simple_explanation,
                        te.structured_breakdown_json,
                        te.worked_examples_json,
                        te.common_mistakes_json,
                        te.exam_appearance_notes,
                        te.pattern_recognition_tips,
                        te.related_concepts_json,
                        te.visual_asset_refs_json,
                        te.subject_style,
                        te.created_at,
                        te.updated_at
                     FROM teach_explanations te
                     INNER JOIN academic_nodes an ON an.id = te.node_id
                     LEFT JOIN topics t ON t.id = an.topic_id
                     WHERE te.node_id = ?1
                       AND te.explanation_level = ?2
                     LIMIT 1",
                    params![node_id, level],
                    |row| {
                        Ok(TeachExplanation {
                            id: row.get(0)?,
                            node_id: row.get(1)?,
                            topic_id: row.get(2)?,
                            topic_name: row.get(3)?,
                            explanation_level: row.get(5)?,
                            hero_summary: row.get(6)?,
                            why_it_matters: row.get(7)?,
                            simple_explanation: row.get(8)?,
                            structured_breakdown: parse_value_sql(row.get::<_, String>(9)?)?,
                            worked_examples: parse_value_sql(row.get::<_, String>(10)?)?,
                            common_mistakes: parse_value_sql(row.get::<_, String>(11)?)?,
                            exam_appearance_notes: row.get(12)?,
                            pattern_recognition_tips: row.get(13)?,
                            related_concepts: parse_value_sql(row.get::<_, String>(14)?)?,
                            visual_asset_refs: parse_value_sql(row.get::<_, String>(15)?)?,
                            subject_style: row.get(16)?,
                            created_at: row.get(17)?,
                            updated_at: row.get(18)?,
                        })
                    },
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
            {
                return Ok(Some(explanation));
            }
        }

        self.conn
            .query_row(
                "SELECT
                    te.id,
                    te.node_id,
                    an.topic_id,
                    t.name,
                    an.canonical_title,
                    te.explanation_level,
                    te.hero_summary,
                    te.why_it_matters,
                    te.simple_explanation,
                    te.structured_breakdown_json,
                    te.worked_examples_json,
                    te.common_mistakes_json,
                    te.exam_appearance_notes,
                    te.pattern_recognition_tips,
                    te.related_concepts_json,
                    te.visual_asset_refs_json,
                    te.subject_style,
                    te.created_at,
                    te.updated_at
                 FROM teach_explanations te
                 INNER JOIN academic_nodes an ON an.id = te.node_id
                 LEFT JOIN topics t ON t.id = an.topic_id
                 WHERE te.node_id = ?1
                 ORDER BY CASE WHEN te.explanation_level = 'core' THEN 0 ELSE 1 END,
                          te.updated_at DESC,
                          te.id ASC
                 LIMIT 1",
                [node_id],
                |row| {
                    Ok(TeachExplanation {
                        id: row.get(0)?,
                        node_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        topic_name: row.get(3)?,
                        explanation_level: row.get(5)?,
                        hero_summary: row.get(6)?,
                        why_it_matters: row.get(7)?,
                        simple_explanation: row.get(8)?,
                        structured_breakdown: parse_value_sql(row.get::<_, String>(9)?)?,
                        worked_examples: parse_value_sql(row.get::<_, String>(10)?)?,
                        common_mistakes: parse_value_sql(row.get::<_, String>(11)?)?,
                        exam_appearance_notes: row.get(12)?,
                        pattern_recognition_tips: row.get(13)?,
                        related_concepts: parse_value_sql(row.get::<_, String>(14)?)?,
                        visual_asset_refs: parse_value_sql(row.get::<_, String>(15)?)?,
                        subject_style: row.get(16)?,
                        created_at: row.get(17)?,
                        updated_at: row.get(18)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_teach_micro_checks(
        &self,
        explanation_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<TeachMicroCheck>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    id,
                    explanation_id,
                    check_type,
                    prompt,
                    correct_answer,
                    distractor_answers_json,
                    explanation_if_wrong,
                    position_index,
                    created_at
                 FROM teach_micro_checks
                 WHERE explanation_id = ?1
                 ORDER BY position_index ASC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![explanation_id, limit as i64], |row| {
                let distractor_answers = parse_string_vec_sql(row.get::<_, String>(5)?)?;
                Ok(TeachMicroCheck {
                    id: row.get(0)?,
                    explanation_id: row.get(1)?,
                    check_type: row.get(2)?,
                    prompt: row.get(3)?,
                    correct_answer: row.get(4)?,
                    distractor_answers,
                    explanation_if_wrong: row.get(6)?,
                    position_index: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut micro_checks = Vec::new();
        for row in rows {
            micro_checks.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(micro_checks)
    }

    fn list_question_summaries_for_topic(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<(i64, String)>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut statement = self
            .conn
            .prepare(
                "SELECT id, stem
                 FROM questions
                 WHERE topic_id = ?1
                   AND is_active = 1
                 ORDER BY difficulty_level ASC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![topic_id, limit as i64], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut questions = Vec::new();
        for row in rows {
            questions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(questions)
    }

    fn build_synthesized_teach_explanation(
        &self,
        topic_id: i64,
        topic_name: &str,
        node_id: i64,
        node_title: &str,
        explanation_level: &str,
        question_summaries: &[(i64, String)],
    ) -> EcoachResult<TeachExplanation> {
        let entry_summaries = self.list_topic_entries(topic_id, 3)?;
        let relationship_hints = self.list_topic_relationship_hints(topic_id, 3)?;
        let related_topic_names = self.related_topic_names_for_hints(&relationship_hints)?;
        let entry_titles = entry_summaries
            .iter()
            .map(|(_, title)| title.clone())
            .collect::<Vec<_>>();
        let worked_examples = if question_summaries.is_empty() {
            json!([format!(
                "Use {} to explain the core idea of {}",
                node_title, topic_name
            )])
        } else {
            json!(
                question_summaries
                    .iter()
                    .map(|(_, stem)| stem.clone())
                    .collect::<Vec<_>>()
            )
        };
        let common_mistakes = if relationship_hints.is_empty() {
            json!([
                format!("Skip the connection back to {}.", node_title),
                format!(
                    "Treat {} as isolated instead of linked to nearby ideas.",
                    topic_name
                )
            ])
        } else {
            json!(
                relationship_hints
                    .iter()
                    .map(|hint| hint.explanation.clone())
                    .collect::<Vec<_>>()
            )
        };
        let now = Utc::now().to_rfc3339();

        Ok(TeachExplanation {
            id: 0,
            node_id,
            topic_id: Some(topic_id),
            topic_name: Some(topic_name.to_string()),
            explanation_level: explanation_level.to_string(),
            hero_summary: Some(format!(
                "Teach {} from the anchor concept {}.",
                topic_name, node_title
            )),
            why_it_matters: Some(format!(
                "This lesson keeps {} grounded in the concept {} and its related ideas.",
                topic_name, node_title
            )),
            simple_explanation: Some(if entry_titles.is_empty() {
                format!(
                    "Start with {} and explain how it supports {}.",
                    node_title, topic_name
                )
            } else {
                format!(
                    "Start with {} and link it to {}.",
                    node_title,
                    entry_titles
                        .first()
                        .cloned()
                        .unwrap_or_else(|| topic_name.to_string())
                )
            }),
            structured_breakdown: json!({
                "topic": topic_name,
                "node": node_title,
                "level": explanation_level,
                "key_entries": entry_titles,
                "related_topics": related_topic_names,
                "question_count": question_summaries.len(),
            }),
            worked_examples,
            common_mistakes,
            exam_appearance_notes: Some(format!(
                "Expect {} to show up as a direct concept check or an applied question.",
                topic_name
            )),
            pattern_recognition_tips: Some(format!(
                "Look for cues that point back to {} and its linked ideas.",
                node_title
            )),
            related_concepts: json!(relationship_hints),
            visual_asset_refs: json!([]),
            subject_style: Some("adaptive".to_string()),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    fn build_synthesized_micro_checks(
        &self,
        explanation_id: i64,
        topic_name: &str,
        explanation_anchor: Option<&str>,
        question_summaries: &[(i64, String)],
        limit: usize,
    ) -> Vec<TeachMicroCheck> {
        if limit == 0 {
            return Vec::new();
        }

        let anchor = explanation_anchor.unwrap_or(topic_name);
        let core_answer = format!("Use the lesson anchor {} to answer it accurately.", anchor);
        let mut micro_checks = Vec::new();

        if question_summaries.is_empty() {
            micro_checks.push(TeachMicroCheck {
                id: -1,
                explanation_id,
                check_type: "retrieval".to_string(),
                prompt: format!("State the core idea behind {} in one sentence.", topic_name),
                correct_answer: core_answer.clone(),
                distractor_answers: vec![
                    format!("Treat {} as a memorization-only topic.", topic_name),
                    format!("Ignore the anchor idea {}.", anchor),
                ],
                explanation_if_wrong: Some(format!(
                    "Return to {} and explain how it supports {}.",
                    anchor, topic_name
                )),
                position_index: 0,
                created_at: Utc::now().to_rfc3339(),
            });
            micro_checks.truncate(limit);
            return micro_checks;
        }

        for (index, (_, stem)) in question_summaries.iter().enumerate() {
            if micro_checks.len() >= limit {
                break;
            }
            micro_checks.push(TeachMicroCheck {
                id: -1 - index as i64,
                explanation_id,
                check_type: "retrieval".to_string(),
                prompt: format!("Micro-check {}: {}", index + 1, truncate_text(stem, 140)),
                correct_answer: core_answer.clone(),
                distractor_answers: vec![
                    format!("Use the wrong shortcut instead of {}.", anchor),
                    format!("Focus on surface wording without the underlying concept."),
                ],
                explanation_if_wrong: Some(format!(
                    "Re-read the anchor {} and connect it to the lesson summary.",
                    anchor
                )),
                position_index: index as i64,
                created_at: Utc::now().to_rfc3339(),
            });
        }

        micro_checks.truncate(limit);
        micro_checks
    }

    fn load_question_tutor_context(
        &self,
        question_id: i64,
    ) -> EcoachResult<Option<TutorQuestionContext>> {
        let question = self
            .conn
            .query_row(
                "SELECT id, topic_id, stem
                 FROM questions
                 WHERE id = ?1
                   AND is_active = 1",
                [question_id],
                |row| {
                    Ok(TutorQuestionContext {
                        topic_id: row.get(1)?,
                        stem: row.get(2)?,
                        options: Vec::new(),
                        correct_option_text: None,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let Some(mut question) = question else {
            return Ok(None);
        };

        let options = self.list_question_option_summaries(question_id)?;
        question.correct_option_text = options
            .iter()
            .find(|option| option.is_correct)
            .map(|option| option.text.clone());
        question.options = options;
        Ok(Some(question))
    }

    fn list_question_option_summaries(
        &self,
        question_id: i64,
    ) -> EcoachResult<Vec<QuestionOptionSummary>> {
        if !self.table_exists("question_options")? {
            return Ok(Vec::new());
        }

        let mut statement = self
            .conn
            .prepare(
                "SELECT option_label, option_text, is_correct
                 FROM question_options
                 WHERE question_id = ?1
                 ORDER BY position ASC, option_label ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([question_id], |row| {
                Ok(QuestionOptionSummary {
                    label: row.get(0)?,
                    text: row.get(1)?,
                    is_correct: row.get::<_, i64>(2)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut options = Vec::new();
        for row in rows {
            options.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(options)
    }

    fn table_exists(&self, table_name: &str) -> EcoachResult<bool> {
        self.conn
            .query_row(
                "SELECT 1
                 FROM sqlite_master
                 WHERE type = 'table'
                   AND name = ?1
                 LIMIT 1",
                [table_name],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map(|value| value.is_some())
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_student_topic_state(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<Option<(i64, i64, String, i64, bool, Option<String>)>> {
        self.conn
            .query_row(
                "SELECT
                    mastery_score,
                    gap_score,
                    trend_state,
                    decay_risk,
                    is_blocked,
                    next_review_at
                 FROM student_topic_states
                 WHERE student_id = ?1
                   AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get::<_, i64>(4)? == 1,
                        row.get(5)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn collect_related_question_ids(
        &self,
        student_id: i64,
        topic_id: i64,
        question_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut question_ids =
            self.list_saved_question_ids_for_topic(student_id, topic_id, limit)?;
        if let Some(question_id) = question_id {
            if !question_ids.contains(&question_id) {
                question_ids.insert(0, question_id);
            }
        }

        if question_ids.len() < limit {
            for question_id in self.list_active_question_ids_for_topic(
                topic_id,
                limit.saturating_sub(question_ids.len()),
            )? {
                if !question_ids.contains(&question_id) {
                    question_ids.push(question_id);
                }
            }
        }

        question_ids.truncate(limit);
        Ok(question_ids)
    }

    fn build_tutor_response_text(
        &self,
        input: &TutorInteractionInput,
        topic_name: Option<&str>,
        lesson: Option<&TeachLesson>,
        question_context: Option<&TutorQuestionContext>,
        recent_history: &[TutorInteraction],
        recent_diagnoses: &[String],
        student_state: Option<&(i64, i64, String, i64, bool, Option<String>)>,
    ) -> String {
        let lesson_anchor = lesson
            .and_then(|lesson| lesson.explanation.as_ref())
            .and_then(|explanation| explanation.hero_summary.as_deref())
            .or_else(|| {
                lesson
                    .and_then(|lesson| lesson.explanation.as_ref())
                    .and_then(|explanation| explanation.simple_explanation.as_deref())
            })
            .or(topic_name)
            .unwrap_or("the topic");
        let question_stem = question_context
            .map(|context| truncate_text(&context.stem, 160))
            .unwrap_or_else(|| "the current question".to_string());
        let diagnosis_hint = recent_diagnoses
            .first()
            .map(|value| truncate_text(value, 160))
            .unwrap_or_else(|| "a stable recall path".to_string());
        let history_hint = recent_history
            .first()
            .and_then(|interaction| interaction.response_text.as_deref())
            .map(|value| truncate_text(value, 120));
        let state_hint = student_state.map(
            |(mastery, gap, trend, decay_risk, blocked, next_review_at)| {
                format!(
                    "mastery {}, gap {}, trend {}, decay risk {}, blocked {}, next review {}",
                    mastery,
                    gap,
                    trend,
                    decay_risk,
                    blocked,
                    next_review_at.as_deref().unwrap_or("not scheduled")
                )
            },
        );

        match input.interaction_type.as_str() {
            "why_correct" => format!(
                "{} is correct because {}. For {}, focus on the anchor rule rather than the surface wording.",
                question_stem,
                lesson_anchor,
                topic_name.unwrap_or("this topic")
            ),
            "why_wrong" => format!(
                "The main issue looks like {}. Rebuild the idea from {} and then retry {}.",
                diagnosis_hint, lesson_anchor, question_stem
            ),
            "teach_from_scratch" => format!(
                "Start from first principles: {}. Build the topic slowly, then connect it back to {}.",
                lesson_anchor, question_stem
            ),
            "explain_simply" => format!(
                "In simple terms, {}. If you want the shortest version: {}.",
                lesson_anchor,
                lesson
                    .and_then(|lesson| lesson.explanation.as_ref())
                    .and_then(|explanation| explanation.simple_explanation.as_deref())
                    .unwrap_or(lesson_anchor)
            ),
            "show_example" => format!(
                "A useful example path here is to start from {} and work it through {}.",
                lesson_anchor, question_stem
            ),
            "worked_example" => format!(
                "Worked example: use {} as the anchor, then walk through {} one deliberate step at a time.",
                lesson_anchor, question_stem
            ),
            "step_by_step" => format!(
                "1. Anchor on {}.\n2. Apply it to {}.\n3. Check the answer against the lesson rule.",
                lesson_anchor, question_stem
            ),
            "show_shortcut" => format!(
                "Shortcut: look for the cue that points back to {} before you commit to an answer on {}.",
                lesson_anchor, question_stem
            ),
            "exam_trick" => format!(
                "Exam trick: spot the signal that maps {} back to {} before you spend time expanding every option.",
                question_stem, lesson_anchor
            ),
            "compare_options" => {
                if let Some(context) = question_context {
                    if !context.options.is_empty() {
                        let options = context
                            .options
                            .iter()
                            .map(|option| {
                                let label = option.label.as_deref().unwrap_or("?");
                                format!("{}: {}", label, option.text)
                            })
                            .collect::<Vec<_>>()
                            .join(" | ");
                        return format!(
                            "Compare the options against the anchor {}. {}. The option that best matches the lesson is {}.",
                            lesson_anchor,
                            options,
                            context
                                .correct_option_text
                                .as_deref()
                                .unwrap_or("the choice that matches the lesson rule")
                        );
                    }
                }
                format!(
                    "Compare each option against {} and keep the one that follows the lesson rule most closely.",
                    lesson_anchor
                )
            }
            "compare_similar" => format!(
                "Compare this question with the anchor {} and look for the shared structure before reacting to surface changes.",
                lesson_anchor
            ),
            "why_options_wrong" => format!(
                "Check each option against {}. The wrong options usually drift because they break the anchor rule or repeat {}.",
                lesson_anchor, diagnosis_hint
            ),
            "pattern_hint" => format!(
                "Pattern hint: look for the cue that maps the problem back to {}. Recent history suggests {}.",
                lesson_anchor,
                history_hint.unwrap_or_else(|| "keep the process calm and deliberate".to_string())
            ),
            "turn_into_flashcards" => format!(
                "Turn {} into flashcards by separating the core rule, the common trap, and one quick retrieval cue.",
                lesson_anchor
            ),
            "timed_drill" => format!(
                "Build a short timed drill around {} and then use {} as the immediate review question.",
                lesson_anchor, question_stem
            ),
            "test_me_now" => format!(
                "Test yourself immediately on {} by solving {} without support, then compare it back to the anchor.",
                lesson_anchor, question_stem
            ),
            "practice_questions" => format!(
                "Practice questions should stay close to {} first, then widen into variants once {} feels stable.",
                lesson_anchor, question_stem
            ),
            _ => format!(
                "Use {} to answer {}. {}",
                lesson_anchor,
                question_stem,
                state_hint.unwrap_or_else(|| {
                    format!(
                        "The best next move is to apply the lesson rule and then verify the result."
                    )
                })
            ),
        }
    }

    fn build_tutor_context_summary(
        &self,
        input: &TutorInteractionInput,
        topic_name: Option<&str>,
        lesson: Option<&TeachLesson>,
        question_context: Option<&TutorQuestionContext>,
        recent_history: &[TutorInteraction],
        recent_diagnoses: &[String],
        student_state: Option<&(i64, i64, String, i64, bool, Option<String>)>,
    ) -> String {
        let mut parts = Vec::new();
        if let Some(topic_name) = topic_name {
            parts.push(format!("topic {}", topic_name));
        }
        if let Some(lesson) = lesson {
            if let Some(explanation) = lesson.explanation.as_ref() {
                if let Some(hero_summary) = explanation.hero_summary.as_deref() {
                    parts.push(format!("lesson {}", truncate_text(hero_summary, 120)));
                }
            }
            parts.push(if lesson.generated {
                "synthesized lesson".to_string()
            } else {
                "stored lesson".to_string()
            });
        }
        if let Some(question_context) = question_context {
            parts.push(format!(
                "question {}",
                truncate_text(&question_context.stem, 100)
            ));
        }
        if let Some((mastery, gap, trend, decay_risk, blocked, next_review_at)) = student_state {
            parts.push(format!(
                "mastery {} gap {} trend {} decay {} blocked {} review {}",
                mastery,
                gap,
                trend,
                decay_risk,
                blocked,
                next_review_at.as_deref().unwrap_or("not scheduled")
            ));
        }
        if !recent_diagnoses.is_empty() {
            parts.push(format!(
                "{} recent diagnosis{}",
                recent_diagnoses.len(),
                if recent_diagnoses.len() == 1 {
                    ""
                } else {
                    "es"
                }
            ));
        }
        if !recent_history.is_empty() {
            parts.push(format!(
                "{} prior tutor interaction{}",
                recent_history.len(),
                if recent_history.len() == 1 { "" } else { "s" }
            ));
        }
        parts.push(format!("request {}", input.interaction_type));
        parts.join("; ")
    }

    fn build_tutor_next_steps(
        &self,
        input: &TutorInteractionInput,
        lesson: Option<&TeachLesson>,
        question_context: Option<&TutorQuestionContext>,
        recent_diagnoses: &[String],
        student_state: Option<&(i64, i64, String, i64, bool, Option<String>)>,
    ) -> Vec<String> {
        let mut steps = Vec::new();
        if let Some(lesson) = lesson {
            steps.push(format!("Review the {} lesson summary.", lesson.topic_name));
            if let Some(micro_check) = lesson.micro_checks.first() {
                steps.push(format!("Try the first micro-check: {}", micro_check.prompt));
            }
        }
        if let Some(question_context) = question_context {
            steps.push(format!(
                "Retry the question: {}",
                truncate_text(&question_context.stem, 110)
            ));
        }
        if let Some(diagnosis) = recent_diagnoses.first() {
            steps.push(format!(
                "Avoid the latest mistake: {}",
                truncate_text(diagnosis, 110)
            ));
        }
        if let Some((_, gap, _, _, blocked, _)) = student_state {
            if *blocked || *gap >= 6500 {
                steps.push(
                    "Keep the next attempt slow and precise before adding speed.".to_string(),
                );
            }
        }
        match input.interaction_type.as_str() {
            "turn_into_flashcards" => {
                steps.push("Write one flashcard for the rule and one for the trap.".to_string());
            }
            "timed_drill" | "test_me_now" | "practice_questions" => {
                steps
                    .push("Use a short burst first, then review the miss immediately.".to_string());
            }
            _ => {}
        }
        if steps.is_empty() {
            steps.push(format!("Continue with {}", input.interaction_type));
        }
        steps.truncate(4);
        steps
    }

    fn build_teach_sequence(
        &self,
        topic_name: &str,
        action_type: &str,
        support_intensity: &str,
        fragile_memory_count: i64,
        linked_question_ids: &[i64],
        linked_entry_ids: &[i64],
        linked_entry_titles: &[String],
        target_node_titles: &[String],
        relationship_hints: &[TopicRelationshipHint],
        diagnostic_focuses: &[String],
    ) -> Vec<TeachActionStep> {
        let anchor_entry = linked_entry_titles
            .first()
            .cloned()
            .unwrap_or_else(|| "the key explanation".to_string());
        let anchor_node = target_node_titles
            .first()
            .cloned()
            .unwrap_or_else(|| "the central concept".to_string());
        let mut steps = Vec::new();

        let (title, prompt) = match action_type {
            "reteach_foundation" => (
                "Reset the foundation".to_string(),
                format!(
                    "Use {} support to rebuild {} from first principles. Anchor on {} before introducing {}.",
                    support_intensity, topic_name, anchor_entry, anchor_node
                ),
            ),
            "memory_reactivation" => (
                "Reactivate recall".to_string(),
                format!(
                    "Start with rapid retrieval on {} and {}. There are {} fragile memory traces to stabilize.",
                    anchor_entry, anchor_node, fragile_memory_count
                ),
            ),
            "guided_practice" => (
                "Model then fade support".to_string(),
                format!(
                    "Walk through {} with {} as the explanation anchor, then fade to supported practice around {}.",
                    topic_name, anchor_entry, anchor_node
                ),
            ),
            _ => (
                "Bridge to exam use".to_string(),
                format!(
                    "Keep {} concise, then link it directly to question work through {} and {}.",
                    topic_name, anchor_entry, anchor_node
                ),
            ),
        };
        steps.push(TeachActionStep {
            sequence_no: (steps.len() + 1) as i64,
            step_type: "anchor".to_string(),
            title,
            prompt,
            linked_question_ids: Vec::new(),
            linked_entry_ids: linked_entry_ids.iter().copied().take(2).collect(),
            focus_labels: vec![anchor_entry.clone(), anchor_node.clone()],
        });

        if let Some(hint) = relationship_hints.first() {
            steps.push(TeachActionStep {
                sequence_no: (steps.len() + 1) as i64,
                step_type: "relationship".to_string(),
                title: format!(
                    "Connect {} to {}",
                    truncate_text(&hint.from_title, 36),
                    truncate_text(&hint.to_title, 36)
                ),
                prompt: hint.explanation.clone(),
                linked_question_ids: Vec::new(),
                linked_entry_ids: linked_entry_ids.iter().copied().take(2).collect(),
                focus_labels: vec![
                    hint.relation_type.clone(),
                    hint.from_title.clone(),
                    hint.to_title.clone(),
                ],
            });
        }

        if let Some(focus) = diagnostic_focuses.first() {
            steps.push(TeachActionStep {
                sequence_no: (steps.len() + 1) as i64,
                step_type: "repair".to_string(),
                title: "Repair the dominant failure mode".to_string(),
                prompt: focus.clone(),
                linked_question_ids: Vec::new(),
                linked_entry_ids: linked_entry_ids.iter().copied().take(2).collect(),
                focus_labels: diagnostic_focuses.iter().take(2).cloned().collect(),
            });
        }

        if !linked_question_ids.is_empty() {
            steps.push(TeachActionStep {
                sequence_no: (steps.len() + 1) as i64,
                step_type: "practice".to_string(),
                title: "Re-apply in questions".to_string(),
                prompt: format!(
                    "Work through {} linked questions in order, keeping the explanation visible on the first pass and removing support on the second.",
                    linked_question_ids.len()
                ),
                linked_question_ids: linked_question_ids.iter().copied().take(4).collect(),
                linked_entry_ids: linked_entry_ids.iter().copied().take(2).collect(),
                focus_labels: vec![support_intensity.to_string()],
            });
        }

        steps.push(TeachActionStep {
            sequence_no: (steps.len() + 1) as i64,
            step_type: "check".to_string(),
            title: "Check recall and transfer".to_string(),
            prompt: if fragile_memory_count > 0 {
                "End by asking for an explanation from memory, then a fresh example without prompts."
                    .to_string()
            } else {
                "End by asking for a clean explanation and one transfer example without prompts."
                    .to_string()
            },
            linked_question_ids: linked_question_ids.iter().copied().rev().take(1).collect(),
            linked_entry_ids: linked_entry_ids.iter().copied().take(1).collect(),
            focus_labels: target_node_titles.iter().take(2).cloned().collect(),
        });

        steps
    }

    fn learning_path_activity_type(
        &self,
        mastery_score: i64,
        gap_score: i64,
        decay_risk: i64,
        is_blocked: bool,
        trend_state: &str,
    ) -> String {
        if is_blocked || gap_score >= 6500 {
            "topic_repair".to_string()
        } else if decay_risk >= 6000 {
            "memory_return".to_string()
        } else if mastery_score < 5500
            || matches!(trend_state, "fragile" | "declining" | "critical")
        {
            "guided_practice".to_string()
        } else {
            "exam_bridge".to_string()
        }
    }

    fn learning_path_reason(
        &self,
        gap_score: i64,
        decay_risk: i64,
        is_blocked: bool,
        review_due: bool,
        trend_state: &str,
        bundle_sequence: &[BundleSequenceCandidate],
        relationship_hint: Option<&TopicRelationshipHint>,
    ) -> String {
        let mut reasons = Vec::new();
        if is_blocked || gap_score >= 6500 {
            reasons
                .push("gap pressure is high, so this topic needs a repair-first path".to_string());
        } else if decay_risk >= 6000 || review_due {
            reasons.push(
                "review timing and decay risk make this the best memory return target".to_string(),
            );
        } else if matches!(trend_state, "fragile" | "declining" | "critical") {
            reasons.push(
                "recent trend signals show the topic is slipping and needs guided reinforcement"
                    .to_string(),
            );
        }
        if let Some(bundle) = bundle_sequence.first() {
            reasons.push(format!(
                "bundle {} is the cleanest next knowledge route",
                bundle.title
            ));
        }
        if let Some(hint) = relationship_hint {
            reasons.push(format!(
                "{} connects directly to the next repair step",
                truncate_text(&hint.to_title, 40)
            ));
        }
        if reasons.is_empty() {
            "this is the highest-priority available learning path right now".to_string()
        } else {
            capitalize_first(&format!("{}.", reasons.join(", ")))
        }
    }

    fn build_learning_path_steps(
        &self,
        topic_id: i64,
        topic_name: &str,
        activity_type: &str,
        relationship_hints: &[TopicRelationshipHint],
        bundle_sequence: &[BundleSequenceCandidate],
        practice_question_id: Option<i64>,
    ) -> Vec<LearningPathStep> {
        let mut steps = Vec::new();
        let intro_detail = match activity_type {
            "topic_repair" => format!(
                "Reset {} from first principles before asking for fast performance.",
                topic_name
            ),
            "memory_return" => format!(
                "Bring {} back into active recall before new question work.",
                topic_name
            ),
            "guided_practice" => format!(
                "Work through {} with support and fade that support across the path.",
                topic_name
            ),
            _ => format!(
                "Bridge {} into exam-style use without losing the core explanation.",
                topic_name
            ),
        };
        steps.push(LearningPathStep {
            sequence_no: 1,
            step_type: "focus".to_string(),
            title: format!("Start with {}", topic_name),
            detail: intro_detail,
            topic_id: Some(topic_id),
            bundle_id: None,
            question_id: None,
        });

        if let Some(hint) = relationship_hints.first() {
            let relationship_title = match hint.focus_topic_id {
                Some(focus_topic_id) if focus_topic_id != topic_id => self
                    .topic_name(focus_topic_id)
                    .unwrap_or_else(|_| hint.to_title.clone()),
                _ => hint.to_title.clone(),
            };
            steps.push(LearningPathStep {
                sequence_no: (steps.len() + 1) as i64,
                step_type: "relationship".to_string(),
                title: format!("Use {}", truncate_text(&relationship_title, 40)),
                detail: hint.explanation.clone(),
                topic_id: hint.focus_topic_id,
                bundle_id: None,
                question_id: None,
            });
        }

        if let Some(bundle) = bundle_sequence.first() {
            steps.push(LearningPathStep {
                sequence_no: (steps.len() + 1) as i64,
                step_type: "bundle".to_string(),
                title: format!("Study {}", bundle.title),
                detail: bundle.reason.clone(),
                topic_id: Some(topic_id),
                bundle_id: Some(bundle.bundle_id),
                question_id: None,
            });
        }

        if let Some(question_id) = practice_question_id {
            steps.push(LearningPathStep {
                sequence_no: (steps.len() + 1) as i64,
                step_type: "practice".to_string(),
                title: "Check the path in a question".to_string(),
                detail: "Apply the repaired idea immediately so the explanation turns into usable performance."
                    .to_string(),
                topic_id: Some(topic_id),
                bundle_id: None,
                question_id: Some(question_id),
            });
        }

        steps
    }

    fn collect_direct_relationship_hints(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<TopicRelationshipHint>> {
        let mut hints = Vec::new();

        let mut node_stmt = self
            .conn
            .prepare(
                "SELECT
                    ne.edge_type,
                    COALESCE(an_from.canonical_title, t_from.name),
                    COALESCE(an_to.canonical_title, t_to.name),
                    COALESCE(an_from.topic_id, CASE WHEN ne.from_node_type = 'topic' THEN ne.from_node_id END),
                    COALESCE(an_to.topic_id, CASE WHEN ne.to_node_type = 'topic' THEN ne.to_node_id END),
                    ne.strength_score
                 FROM node_edges ne
                 LEFT JOIN academic_nodes an_from
                    ON ne.from_node_type = 'academic_node'
                   AND an_from.id = ne.from_node_id
                 LEFT JOIN academic_nodes an_to
                    ON ne.to_node_type = 'academic_node'
                   AND an_to.id = ne.to_node_id
                 LEFT JOIN topics t_from
                    ON ne.from_node_type = 'topic'
                   AND t_from.id = ne.from_node_id
                 LEFT JOIN topics t_to
                    ON ne.to_node_type = 'topic'
                   AND t_to.id = ne.to_node_id
                 WHERE (
                        an_from.topic_id = ?1
                        OR an_to.topic_id = ?1
                        OR (ne.from_node_type = 'topic' AND ne.from_node_id = ?1)
                        OR (ne.to_node_type = 'topic' AND ne.to_node_id = ?1)
                 )
                   AND ne.edge_type IN ('prerequisite', 'related', 'confused_with', 'contrasts_with', 'dependent')
                 ORDER BY ne.strength_score DESC, ne.id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = node_stmt
            .query_map(params![topic_id, limit as i64], |row| {
                let from_topic_id: Option<i64> = row.get(3)?;
                let to_topic_id: Option<i64> = row.get(4)?;
                let focus_topic_id = match (from_topic_id, to_topic_id) {
                    (Some(from_id), Some(to_id)) if from_id == topic_id && to_id != topic_id => {
                        Some(to_id)
                    }
                    (Some(from_id), Some(to_id)) if to_id == topic_id && from_id != topic_id => {
                        Some(from_id)
                    }
                    (Some(from_id), None) if from_id != topic_id => Some(from_id),
                    (None, Some(to_id)) if to_id != topic_id => Some(to_id),
                    _ => None,
                };
                Ok(TopicRelationshipHint {
                    relation_type: row.get(0)?,
                    from_title: row.get(1)?,
                    to_title: row.get(2)?,
                    explanation: String::new(),
                    hop_count: 1,
                    strength_score: row.get::<_, i64>(5)?.clamp(0, 10_000) as BasisPoints,
                    focus_topic_id,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            let mut hint = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            hint.explanation = self.relationship_explanation(
                &hint.relation_type,
                &hint.from_title,
                &hint.to_title,
            );
            push_unique_relationship_hint(&mut hints, hint);
        }

        let remaining = limit.saturating_sub(hints.len());
        if remaining > 0 {
            let mut knowledge_stmt = self
                .conn
                .prepare(
                    "SELECT
                        kr.relation_type,
                        from_ke.title,
                        to_ke.title,
                        from_ke.topic_id,
                        to_ke.topic_id,
                        kr.strength_score
                     FROM knowledge_relations kr
                     INNER JOIN knowledge_entries from_ke ON from_ke.id = kr.from_entry_id
                     INNER JOIN knowledge_entries to_ke ON to_ke.id = kr.to_entry_id
                     WHERE from_ke.topic_id = ?1 OR to_ke.topic_id = ?1
                     ORDER BY kr.strength_score DESC, kr.id ASC
                     LIMIT ?2",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = knowledge_stmt
                .query_map(params![topic_id, remaining as i64], |row| {
                    let from_topic_id: Option<i64> = row.get(3)?;
                    let to_topic_id: Option<i64> = row.get(4)?;
                    let focus_topic_id = match (from_topic_id, to_topic_id) {
                        (Some(from_id), Some(to_id))
                            if from_id == topic_id && to_id != topic_id =>
                        {
                            Some(to_id)
                        }
                        (Some(from_id), Some(to_id))
                            if to_id == topic_id && from_id != topic_id =>
                        {
                            Some(from_id)
                        }
                        (Some(from_id), None) if from_id != topic_id => Some(from_id),
                        (None, Some(to_id)) if to_id != topic_id => Some(to_id),
                        _ => None,
                    };
                    Ok(TopicRelationshipHint {
                        relation_type: row.get(0)?,
                        from_title: row.get(1)?,
                        to_title: row.get(2)?,
                        explanation: String::new(),
                        hop_count: 1,
                        strength_score: row.get::<_, i64>(5)?.clamp(0, 10_000) as BasisPoints,
                        focus_topic_id,
                    })
                })
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                let mut hint = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
                hint.explanation = self.relationship_explanation(
                    &hint.relation_type,
                    &hint.from_title,
                    &hint.to_title,
                );
                push_unique_relationship_hint(&mut hints, hint);
            }
        }

        Ok(hints)
    }

    fn list_related_topics_for_topic(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<RelationshipTopicCandidate>> {
        let mut topics = Vec::new();
        let direct_hints = self.collect_direct_relationship_hints(topic_id, limit * 2)?;
        for hint in direct_hints {
            if let Some(related_topic_id) = hint.focus_topic_id {
                let related_topic_name = if related_topic_id == topic_id {
                    continue;
                } else {
                    self.topic_name(related_topic_id)?
                };
                if !topics.iter().any(|existing: &RelationshipTopicCandidate| {
                    existing.related_topic_id == related_topic_id
                }) {
                    topics.push(RelationshipTopicCandidate {
                        related_topic_id,
                        related_topic_name,
                        relation_type: hint.relation_type,
                        strength_score: hint.strength_score,
                    });
                }
                if topics.len() >= limit {
                    break;
                }
            }
        }
        Ok(topics)
    }

    fn build_bridge_relationship_hint(
        &self,
        root_topic_id: i64,
        root_topic_name: &str,
        bridge_topic: &RelationshipTopicCandidate,
    ) -> EcoachResult<Option<TopicRelationshipHint>> {
        let bridge_hints =
            self.collect_direct_relationship_hints(bridge_topic.related_topic_id, 4)?;
        for hint in bridge_hints {
            if let Some(target_topic_id) = hint.focus_topic_id {
                if target_topic_id == root_topic_id
                    || target_topic_id == bridge_topic.related_topic_id
                {
                    continue;
                }
                let target_topic_name = self.topic_name(target_topic_id)?;
                return Ok(Some(TopicRelationshipHint {
                    relation_type: "pathway".to_string(),
                    from_title: root_topic_name.to_string(),
                    to_title: target_topic_name.clone(),
                    explanation: format!(
                        "Bridge {} through {} via a {} link before expecting stable performance on {}.",
                        root_topic_name,
                        bridge_topic.related_topic_name,
                        bridge_topic.relation_type.replace('_', " "),
                        target_topic_name
                    ),
                    hop_count: 2,
                    strength_score: bridge_topic.strength_score.min(hint.strength_score),
                    focus_topic_id: Some(target_topic_id),
                }));
            }
        }
        Ok(None)
    }

    fn topic_name(&self, topic_id: i64) -> EcoachResult<String> {
        self.conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn related_topic_names_for_hints(
        &self,
        hints: &[TopicRelationshipHint],
    ) -> EcoachResult<Vec<String>> {
        let mut names = Vec::new();
        for hint in hints {
            if let Some(focus_topic_id) = hint.focus_topic_id {
                push_unique(&mut names, self.topic_name(focus_topic_id)?);
            }
        }
        Ok(names)
    }

    fn list_bundle_sequence_for_topic(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<BundleSequenceCandidate>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    kb.id,
                    kb.title,
                    SUM(CASE WHEN ses.review_due_at IS NOT NULL THEN 1 ELSE 0 END) AS due_review_count,
                    MAX(COALESCE(ses.confusion_score, 0)) AS max_confusion_score,
                    SUM(CASE WHEN COALESCE(ses.recall_strength, 10000) <= 3500 THEN 1 ELSE 0 END) AS weak_recall_count
                 FROM knowledge_bundles kb
                 LEFT JOIN knowledge_bundle_items kbi ON kbi.bundle_id = kb.id
                 LEFT JOIN student_entry_state ses
                    ON ses.entry_id = kbi.entry_id
                   AND ses.user_id = ?1
                 WHERE kb.topic_id = ?2
                 GROUP BY kb.id, kb.title, kb.exam_relevance_score, kb.difficulty_level
                 ORDER BY
                    due_review_count DESC,
                    max_confusion_score DESC,
                    weak_recall_count DESC,
                    kb.exam_relevance_score DESC,
                    kb.difficulty_level ASC,
                    kb.id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, topic_id, limit as i64], |row| {
                let title: String = row.get(1)?;
                let due_review_count: i64 = row.get(2)?;
                let max_confusion_score: i64 = row.get(3)?;
                let weak_recall_count: i64 = row.get(4)?;
                let reason = if due_review_count > 0 {
                    format!(
                        "{} has {} review-due glossary anchor(s) to return to first.",
                        title, due_review_count
                    )
                } else if max_confusion_score >= 4500 {
                    format!(
                        "{} contains the strongest confusion hotspot for this topic.",
                        title
                    )
                } else if weak_recall_count > 0 {
                    format!(
                        "{} is the cleanest bundle for rebuilding recall before practice.",
                        title
                    )
                } else {
                    format!(
                        "{} is the best structured bundle to continue with next.",
                        title
                    )
                };
                Ok(BundleSequenceCandidate {
                    bundle_id: row.get(0)?,
                    title,
                    reason,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut bundles = Vec::new();
        for row in rows {
            bundles.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(bundles)
    }

    fn relationship_explanation(
        &self,
        relation_type: &str,
        from_title: &str,
        to_title: &str,
    ) -> String {
        match relation_type {
            "prerequisite" | "dependent" => {
                format!(
                    "Teach {} before expecting stable performance on {}.",
                    from_title, to_title
                )
            }
            "confused_with" | "contrasts_with" => {
                format!(
                    "Contrast {} with {} explicitly because learners can mix them up.",
                    from_title, to_title
                )
            }
            "related" => format!(
                "Connect {} to {} so the learner sees the wider structure.",
                from_title, to_title
            ),
            _ => format!(
                "Use the relationship between {} and {} during explanation.",
                from_title, to_title
            ),
        }
    }

    fn count_scalar<P>(&self, sql: &str, params: P) -> EcoachResult<i64>
    where
        P: rusqlite::Params,
    {
        self.conn
            .query_row(sql, params, |row| row.get(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_saved_question_ids_for_topic(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.id
                 FROM library_items li
                 INNER JOIN questions q ON q.id = li.item_ref_id
                 WHERE li.student_id = ?1
                   AND li.item_type = 'question'
                   AND q.topic_id = ?2
                 ORDER BY li.urgency_score DESC, li.updated_at DESC, li.id DESC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, topic_id, limit as i64], |row| {
                row.get(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(ids)
    }

    fn list_active_question_ids_for_topic(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id
                 FROM questions
                 WHERE topic_id = ?1
                   AND is_active = 1
                 ORDER BY difficulty_level ASC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![topic_id, limit as i64], |row| row.get(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(ids)
    }

    fn resolve_library_item_dimensions(
        &self,
        item_type: &str,
        item_ref_id: i64,
        topic_id: Option<i64>,
        subject_id: Option<i64>,
        subtopic_id: Option<i64>,
        difficulty_bp: Option<BasisPoints>,
        exam_frequency_bp: Option<BasisPoints>,
        source: Option<&str>,
    ) -> EcoachResult<(
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<BasisPoints>,
        Option<BasisPoints>,
        Option<String>,
    )> {
        match item_type {
            "question" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT subject_id, topic_id, subtopic_id, difficulty_level, source_type, family_id
                         FROM questions
                         WHERE id = ?1",
                        [item_ref_id],
                        |row| {
                            Ok((
                                row.get::<_, i64>(0)?,
                                row.get::<_, i64>(1)?,
                                row.get::<_, Option<i64>>(2)?,
                                row.get::<_, i64>(3)?,
                                row.get::<_, Option<String>>(4)?,
                                row.get::<_, Option<i64>>(5)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((
                    inferred_subject_id,
                    inferred_topic_id,
                    inferred_subtopic_id,
                    inferred_difficulty,
                    inferred_source,
                    family_id,
                )) = row
                {
                    let inferred_exam_frequency = if exam_frequency_bp.is_some() {
                        exam_frequency_bp
                    } else if let Some(family_id) = family_id {
                        if self.table_exists("family_recurrence_metrics")? {
                            self.conn
                                .query_row(
                                    "SELECT recurrence_rate_bp
                                     FROM family_recurrence_metrics
                                     WHERE family_id = ?1 AND subject_id = ?2",
                                    params![family_id, inferred_subject_id],
                                    |row| row.get::<_, i64>(0),
                                )
                                .optional()
                                .map_err(|err| EcoachError::Storage(err.to_string()))?
                                .map(|value| value.clamp(0, 10_000) as BasisPoints)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    return Ok((
                        topic_id.or(Some(inferred_topic_id)),
                        subject_id.or(Some(inferred_subject_id)),
                        subtopic_id.or(inferred_subtopic_id),
                        difficulty_bp.or(Some(inferred_difficulty.clamp(0, 10_000) as BasisPoints)),
                        inferred_exam_frequency,
                        source
                            .map(|value| value.to_string())
                            .or(inferred_source)
                            .or_else(|| Some("question".to_string())),
                    ));
                }
            }
            "topic" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT subject_id, exam_weight FROM topics WHERE id = ?1",
                        [item_ref_id],
                        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((inferred_subject_id, inferred_exam_weight)) = row {
                    return Ok((
                        Some(item_ref_id),
                        subject_id.or(Some(inferred_subject_id)),
                        subtopic_id,
                        difficulty_bp,
                        exam_frequency_bp
                            .or(Some(inferred_exam_weight.clamp(0, 10_000) as BasisPoints)),
                        source
                            .map(|value| value.to_string())
                            .or_else(|| Some("curriculum".to_string())),
                    ));
                }
            }
            "knowledge_entry" | "entry" | "note" | "summary" | "lesson" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT ke.topic_id, t.subject_id, ke.difficulty_level, ke.importance_score
                         FROM knowledge_entries ke
                         LEFT JOIN topics t ON t.id = ke.topic_id
                         WHERE ke.id = ?1",
                        [item_ref_id],
                        |row| {
                            Ok((
                                row.get::<_, Option<i64>>(0)?,
                                row.get::<_, Option<i64>>(1)?,
                                row.get::<_, i64>(2)?,
                                row.get::<_, i64>(3)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((
                    inferred_topic_id,
                    inferred_subject_id,
                    inferred_difficulty,
                    inferred_importance,
                )) = row
                {
                    return Ok((
                        topic_id.or(inferred_topic_id),
                        subject_id.or(inferred_subject_id),
                        subtopic_id,
                        difficulty_bp.or(Some(inferred_difficulty.clamp(0, 10_000) as BasisPoints)),
                        exam_frequency_bp
                            .or(Some(inferred_importance.clamp(0, 10_000) as BasisPoints)),
                        source
                            .map(|value| value.to_string())
                            .or_else(|| Some("knowledge".to_string())),
                    ));
                }
            }
            "formula" | "academic_node" | "node" | "definition" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT an.topic_id, t.subject_id, an.exam_relevance_score, an.foundation_weight
                         FROM academic_nodes an
                         LEFT JOIN topics t ON t.id = an.topic_id
                         WHERE an.id = ?1",
                        [item_ref_id],
                        |row| {
                            Ok((
                                row.get::<_, Option<i64>>(0)?,
                                row.get::<_, Option<i64>>(1)?,
                                row.get::<_, i64>(2)?,
                                row.get::<_, i64>(3)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((
                    inferred_topic_id,
                    inferred_subject_id,
                    inferred_exam_relevance,
                    inferred_foundation,
                )) = row
                {
                    return Ok((
                        topic_id.or(inferred_topic_id),
                        subject_id.or(inferred_subject_id),
                        subtopic_id,
                        difficulty_bp.or(Some(inferred_foundation.clamp(0, 10_000) as BasisPoints)),
                        exam_frequency_bp
                            .or(Some(inferred_exam_relevance.clamp(0, 10_000) as BasisPoints)),
                        source
                            .map(|value| value.to_string())
                            .or_else(|| Some("curriculum".to_string())),
                    ));
                }
            }
            _ => {}
        }

        Ok((
            topic_id,
            subject_id,
            subtopic_id,
            difficulty_bp,
            exam_frequency_bp,
            source.map(|value| value.to_string()),
        ))
    }

    fn insert_state_history(
        &self,
        library_item_id: i64,
        from_state: Option<&str>,
        to_state: &str,
        reason: Option<&str>,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO library_item_state_history (
                    library_item_id,
                    from_state,
                    to_state,
                    reason
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![library_item_id, from_state, to_state, reason],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn add_tag_to_library_item(&self, library_item_id: i64, tag: &str) -> EcoachResult<()> {
        let current = self
            .conn
            .query_row(
                "SELECT tags_json FROM library_items WHERE id = ?1",
                [library_item_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some(current) = current else {
            return Ok(());
        };
        let mut tags = parse_tags(&current)?;
        if !tags.iter().any(|value| value == tag) {
            tags.push(tag.to_string());
            self.conn
                .execute(
                    "UPDATE library_items
                     SET tags_json = ?1,
                         updated_at = datetime('now')
                     WHERE id = ?2",
                    params![
                        serde_json::to_string(&tags)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        library_item_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn mark_library_item_state_and_tag(
        &self,
        library_item_id: i64,
        new_state: &str,
        tag: &str,
        reason: Option<&str>,
    ) -> EcoachResult<()> {
        let row = self
            .conn
            .query_row(
                "SELECT state, tags_json FROM library_items WHERE id = ?1",
                [library_item_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some((previous_state, tags_json)) = row else {
            return Ok(());
        };
        let mut tags = parse_tags(&tags_json)?;
        if !tags.iter().any(|value| value == tag) {
            tags.push(tag.to_string());
        }
        self.conn
            .execute(
                "UPDATE library_items
                 SET state = ?1,
                     tags_json = ?2,
                     updated_at = datetime('now')
                 WHERE id = ?3",
                params![
                    new_state,
                    serde_json::to_string(&tags)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    library_item_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if previous_state != new_state {
            self.insert_state_history(library_item_id, Some(&previous_state), new_state, reason)?;
        }
        Ok(())
    }

    fn resolve_item_display_context(
        &self,
        item_type: &str,
        item_ref_id: i64,
        topic_id_hint: Option<i64>,
    ) -> EcoachResult<(String, Option<String>, Option<String>, Option<String>)> {
        match item_type {
            "question" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT q.stem, t.name, s.name
                         FROM questions q
                         LEFT JOIN topics t ON t.id = q.topic_id
                         LEFT JOIN subjects s ON s.id = q.subject_id
                         WHERE q.id = ?1",
                        [item_ref_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                row.get::<_, Option<String>>(2)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((stem, topic_name, subject_name)) = row {
                    let title = truncate_text(&stem, 96);
                    return Ok((title, topic_name.clone(), topic_name, subject_name));
                }
            }
            "topic" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT t.name, s.name
                         FROM topics t
                         LEFT JOIN subjects s ON s.id = t.subject_id
                         WHERE t.id = ?1",
                        [item_ref_id],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((topic_name, subject_name)) = row {
                    return Ok((
                        topic_name.clone(),
                        subject_name.clone(),
                        Some(topic_name),
                        subject_name,
                    ));
                }
            }
            "knowledge_entry" | "entry" | "note" | "summary" | "lesson" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT ke.title, t.name, s.name
                         FROM knowledge_entries ke
                         LEFT JOIN topics t ON t.id = ke.topic_id
                         LEFT JOIN subjects s ON s.id = t.subject_id
                         WHERE ke.id = ?1",
                        [item_ref_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                row.get::<_, Option<String>>(2)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((title, topic_name, subject_name)) = row {
                    return Ok((title, topic_name.clone(), topic_name, subject_name));
                }
            }
            "formula" | "academic_node" | "node" | "definition" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT an.canonical_title, t.name, s.name
                         FROM academic_nodes an
                         LEFT JOIN topics t ON t.id = an.topic_id
                         LEFT JOIN subjects s ON s.id = t.subject_id
                         WHERE an.id = ?1",
                        [item_ref_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                row.get::<_, Option<String>>(2)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((title, topic_name, subject_name)) = row {
                    return Ok((title, topic_name.clone(), topic_name, subject_name));
                }
            }
            "revision_pack" => {
                let title = self
                    .conn
                    .query_row(
                        "SELECT title FROM revision_packs WHERE id = ?1",
                        [item_ref_id],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some(title) = title {
                    return Ok((title, None, None, None));
                }
            }
            "question_family" => {
                let row = self
                    .conn
                    .query_row(
                        "SELECT qf.family_name, t.name, s.name
                         FROM question_families qf
                         LEFT JOIN topics t ON t.id = qf.topic_id
                         LEFT JOIN subjects s ON s.id = qf.subject_id
                         WHERE qf.id = ?1",
                        [item_ref_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                row.get::<_, Option<String>>(2)?,
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                if let Some((title, topic_name, subject_name)) = row {
                    return Ok((title, topic_name.clone(), topic_name, subject_name));
                }
            }
            _ => {}
        }

        let topic_name = if let Some(topic_id) = topic_id_hint {
            self.topic_name(topic_id).ok()
        } else {
            None
        };
        Ok((
            format!("{} {}", capitalize_first(item_type), item_ref_id),
            topic_name.clone(),
            topic_name,
            None,
        ))
    }

    fn search_reason_for_library_item(&self, item: &LibraryItem, title: &str) -> String {
        if item.open_count == 0 && item.exam_frequency_bp.unwrap_or(0) >= 5000 {
            return format!("{} is high-value and still untouched.", title);
        }
        if item.tags.iter().any(|tag| tag == "keep_forgetting") {
            return format!("{} was tagged as something you keep forgetting.", title);
        }
        if matches!(item.state.as_str(), "weak" | "revisit" | "saved" | "new") {
            return format!(
                "{} is still in an active library state ({}).",
                title, item.state
            );
        }
        format!("{} matches your saved library context.", title)
    }

    fn search_score_for_library_item(
        &self,
        item: &LibraryItem,
        query: Option<&str>,
        title: &str,
        subtitle: Option<&str>,
    ) -> BasisPoints {
        let mut score = item.urgency_score;
        score = score.saturating_add(item.exam_frequency_bp.unwrap_or(0) / 3);
        if let Some(query) = query {
            if matches_query_text(title, Some(query)) {
                score = score.saturating_add(2000).min(10_000);
            } else if matches_query_text(subtitle.unwrap_or_default(), Some(query)) {
                score = score.saturating_add(800).min(10_000);
            }
        }
        score.clamp(0, 10_000)
    }

    fn matches_search_filters(
        &self,
        result: &LibrarySearchResult,
        input: &LibrarySearchInput,
    ) -> bool {
        if !input.item_types.is_empty()
            && !input
                .item_types
                .iter()
                .any(|value| value == &result.item_type)
        {
            return false;
        }
        if !input.states.is_empty() {
            let Some(state) = result.state.as_ref() else {
                return false;
            };
            if !input.states.iter().any(|value| value == state) {
                return false;
            }
        }
        if !input.tags.is_empty()
            && !input
                .tags
                .iter()
                .all(|value| result.tags.iter().any(|tag| tag == value))
        {
            return false;
        }
        if input.subject_id.is_some() && result.subject_id != input.subject_id {
            return false;
        }
        if input.topic_id.is_some() && result.topic_id != input.topic_id {
            return false;
        }
        if input.only_wrong && result.metadata["wrong_answer_count"].as_i64().unwrap_or(0) == 0 {
            return false;
        }
        if input.only_near_mastery {
            let mastery_score = result.metadata["mastery_score"].as_i64().unwrap_or(0);
            if !(6500..=8500).contains(&mastery_score) {
                return false;
            }
        }
        if input.only_untouched && result.metadata["open_count"].as_i64().unwrap_or(1) != 0 {
            return false;
        }
        if input.high_frequency_only
            && result.metadata["exam_frequency_bp"].as_i64().unwrap_or(0) < 5000
            && result.metadata["recurrence_rate_bp"].as_i64().unwrap_or(0) < 6000
            && result.metadata["exam_weight"].as_i64().unwrap_or(0) < 6000
        {
            return false;
        }
        if input.due_only
            && result.metadata["review_due_at"].is_null()
            && result.metadata["is_due"].as_bool() != Some(true)
        {
            return false;
        }
        if input.downloaded_only && result.metadata["downloaded"].as_bool() != Some(true) {
            return false;
        }
        true
    }

    fn push_topic_search_results(
        &self,
        student_id: i64,
        input: &LibrarySearchInput,
        query: Option<&str>,
        seen: &mut HashSet<String>,
        results: &mut Vec<LibrarySearchResult>,
    ) -> EcoachResult<()> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT t.id, t.name, t.subject_id, s.name, t.exam_weight
                 FROM topics t
                 INNER JOIN subjects s ON s.id = t.subject_id
                 WHERE (?1 IS NULL OR t.subject_id = ?1)
                   AND (?2 IS NULL OR t.id = ?2 OR t.parent_topic_id = ?2)
                 ORDER BY t.exam_weight DESC, t.name ASC
                 LIMIT 60",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![input.subject_id, input.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            let (topic_id, title, subject_id, subject_name, exam_weight) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if !matches_query_text(&title, query) && !matches_query_text(&subject_name, query) {
                continue;
            }
            let wrong_answer_count = self.count_scalar(
                "SELECT COUNT(*) FROM wrong_answer_diagnoses WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
            )?;
            let mastery_score = self
                .conn
                .query_row(
                    "SELECT mastery_score FROM student_topic_states WHERE student_id = ?1 AND topic_id = ?2",
                    params![student_id, topic_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .unwrap_or(0);
            let result = LibrarySearchResult {
                item_type: "topic".to_string(),
                item_ref_id: Some(topic_id),
                library_item_id: None,
                title: title.clone(),
                subtitle: Some(subject_name.clone()),
                state: None,
                topic_id: Some(topic_id),
                topic_name: Some(title.clone()),
                subject_id: Some(subject_id),
                subject_name: Some(subject_name.clone()),
                tags: Vec::new(),
                reason: format!("Topic explorer entry for {}.", title),
                match_score: (exam_weight + query_bonus(&title, &subject_name, query))
                    .clamp(0, 10_000) as BasisPoints,
                metadata: json!({
                    "exam_weight": exam_weight,
                    "wrong_answer_count": wrong_answer_count,
                    "mastery_score": mastery_score,
                }),
            };
            if self.matches_search_filters(&result, input) {
                let key = format!("topic:{}", topic_id);
                if seen.insert(key) {
                    results.push(result);
                }
            }
        }
        Ok(())
    }

    fn push_entry_search_results(
        &self,
        input: &LibrarySearchInput,
        query: Option<&str>,
        seen: &mut HashSet<String>,
        results: &mut Vec<LibrarySearchResult>,
    ) -> EcoachResult<()> {
        if !self.table_exists("knowledge_entries")? {
            return Ok(());
        }
        let mut statement = self
            .conn
            .prepare(
                "SELECT ke.id, ke.title, ke.topic_id, t.name, t.subject_id, s.name, ke.importance_score, ke.difficulty_level
                 FROM knowledge_entries ke
                 LEFT JOIN topics t ON t.id = ke.topic_id
                 LEFT JOIN subjects s ON s.id = t.subject_id
                 WHERE (?1 IS NULL OR t.subject_id = ?1)
                   AND (?2 IS NULL OR ke.topic_id = ?2)
                 ORDER BY ke.importance_score DESC, ke.difficulty_level ASC, ke.id ASC
                 LIMIT 60",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![input.subject_id, input.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            let (
                id,
                title,
                topic_id,
                topic_name,
                subject_id,
                subject_name,
                importance_score,
                difficulty_level,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if !matches_query_text(&title, query)
                && !matches_query_text(topic_name.as_deref().unwrap_or_default(), query)
            {
                continue;
            }
            let result = LibrarySearchResult {
                item_type: "knowledge_entry".to_string(),
                item_ref_id: Some(id),
                library_item_id: None,
                title: title.clone(),
                subtitle: topic_name.clone(),
                state: None,
                topic_id,
                topic_name,
                subject_id,
                subject_name: subject_name.clone(),
                tags: Vec::new(),
                reason: format!(
                    "Knowledge entry linked to {}.",
                    subject_name
                        .clone()
                        .unwrap_or_else(|| "the syllabus".to_string())
                ),
                match_score: (importance_score
                    + query_bonus(&title, subject_name.as_deref().unwrap_or_default(), query))
                .clamp(0, 10_000) as BasisPoints,
                metadata: json!({
                    "difficulty_bp": difficulty_level,
                    "exam_frequency_bp": importance_score,
                }),
            };
            if self.matches_search_filters(&result, input) {
                let key = format!("knowledge_entry:{}", id);
                if seen.insert(key) {
                    results.push(result);
                }
            }
        }
        Ok(())
    }

    fn push_question_search_results(
        &self,
        student_id: i64,
        input: &LibrarySearchInput,
        query: Option<&str>,
        seen: &mut HashSet<String>,
        results: &mut Vec<LibrarySearchResult>,
    ) -> EcoachResult<()> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    q.id,
                    q.stem,
                    q.topic_id,
                    t.name,
                    q.subject_id,
                    s.name,
                    q.difficulty_level,
                    q.family_id
                 FROM questions q
                 INNER JOIN topics t ON t.id = q.topic_id
                 INNER JOIN subjects s ON s.id = q.subject_id
                 WHERE q.is_active = 1
                   AND (?1 IS NULL OR q.subject_id = ?1)
                   AND (?2 IS NULL OR q.topic_id = ?2)
                 ORDER BY q.difficulty_level ASC, q.id DESC
                 LIMIT 80",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![input.subject_id, input.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            let (
                question_id,
                stem,
                topic_id,
                topic_name,
                subject_id,
                subject_name,
                difficulty_level,
                family_id,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if !matches_query_text(&stem, query) && !matches_query_text(&topic_name, query) {
                continue;
            }
            let wrong_answer_count = self.count_scalar(
                "SELECT COUNT(*) FROM wrong_answer_diagnoses WHERE student_id = ?1 AND question_id = ?2",
                params![student_id, question_id],
            )?;
            let library_item_id = self
                .conn
                .query_row(
                    "SELECT id
                     FROM library_items
                     WHERE student_id = ?1
                       AND item_type = 'question'
                       AND item_ref_id = ?2
                     ORDER BY id DESC
                     LIMIT 1",
                    params![student_id, question_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let recurrence_rate_bp = if let Some(family_id) = family_id {
                self.conn
                    .query_row(
                        "SELECT recurrence_rate_bp
                         FROM family_recurrence_metrics
                         WHERE family_id = ?1 AND subject_id = ?2",
                        params![family_id, subject_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?
                    .unwrap_or(0)
            } else {
                0
            };
            let downloaded = if let Some(library_item_id) = library_item_id {
                self.conn
                    .query_row(
                        "SELECT 1
                         FROM library_item_actions
                         WHERE library_item_id = ?1
                           AND action_type = 'downloaded_offline'
                         ORDER BY created_at DESC
                         LIMIT 1",
                        [library_item_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?
                    .is_some()
            } else {
                false
            };
            let result = LibrarySearchResult {
                item_type: "question".to_string(),
                item_ref_id: Some(question_id),
                library_item_id,
                title: truncate_text(&stem, 96),
                subtitle: Some(topic_name.clone()),
                state: None,
                topic_id: Some(topic_id),
                topic_name: Some(topic_name.clone()),
                subject_id: Some(subject_id),
                subject_name: Some(subject_name),
                tags: Vec::new(),
                reason: if wrong_answer_count > 0 {
                    format!(
                        "Question has {} recorded wrong attempt(s).",
                        wrong_answer_count
                    )
                } else {
                    "Question is available for direct practice.".to_string()
                },
                match_score: (10_000 - difficulty_level
                    + recurrence_rate_bp / 2
                    + query_bonus(&stem, &topic_name, query))
                .clamp(0, 10_000) as BasisPoints,
                metadata: json!({
                    "difficulty_bp": difficulty_level,
                    "wrong_answer_count": wrong_answer_count,
                    "recurrence_rate_bp": recurrence_rate_bp,
                    "downloaded": downloaded,
                }),
            };
            if self.matches_search_filters(&result, input) {
                let key = format!("question:{}", question_id);
                if seen.insert(key) {
                    results.push(result);
                }
            }
        }
        Ok(())
    }

    fn push_family_search_results(
        &self,
        student_id: i64,
        input: &LibrarySearchInput,
        query: Option<&str>,
        seen: &mut HashSet<String>,
        results: &mut Vec<LibrarySearchResult>,
    ) -> EcoachResult<()> {
        if !self.table_exists("question_families")? {
            return Ok(());
        }
        let mut statement = self
            .conn
            .prepare(
                "SELECT qf.id, qf.family_name, qf.topic_id, t.name, qf.subject_id, s.name
                 FROM question_families qf
                 LEFT JOIN topics t ON t.id = qf.topic_id
                 LEFT JOIN subjects s ON s.id = qf.subject_id
                 WHERE (?1 IS NULL OR qf.subject_id = ?1)
                   AND (?2 IS NULL OR qf.topic_id = ?2)
                 ORDER BY qf.family_name ASC
                 LIMIT 60",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![input.subject_id, input.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            let (family_id, family_name, topic_id, topic_name, subject_id, subject_name) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if !matches_query_text(&family_name, query)
                && !matches_query_text(topic_name.as_deref().unwrap_or_default(), query)
            {
                continue;
            }
            let recurrence_rate_bp = self
                .conn
                .query_row(
                    "SELECT recurrence_rate_bp
                     FROM family_recurrence_metrics
                     WHERE family_id = ?1 AND subject_id = ?2",
                    params![family_id, subject_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .unwrap_or(0);
            let wrong_answer_count = self.count_scalar(
                "SELECT COUNT(*)
                 FROM wrong_answer_diagnoses wad
                 INNER JOIN questions q ON q.id = wad.question_id
                 WHERE wad.student_id = ?1
                   AND q.family_id = ?2",
                params![student_id, family_id],
            )?;
            let result = LibrarySearchResult {
                item_type: "question_family".to_string(),
                item_ref_id: Some(family_id),
                library_item_id: None,
                title: family_name.clone(),
                subtitle: topic_name.clone(),
                state: None,
                topic_id,
                topic_name,
                subject_id: Some(subject_id),
                subject_name,
                tags: Vec::new(),
                reason: "Question family from the past-exam vault.".to_string(),
                match_score: (recurrence_rate_bp + query_bonus(&family_name, "", query))
                    .clamp(0, 10_000) as BasisPoints,
                metadata: json!({
                    "recurrence_rate_bp": recurrence_rate_bp,
                    "wrong_answer_count": wrong_answer_count,
                }),
            };
            if self.matches_search_filters(&result, input) {
                let key = format!("question_family:{}", family_id);
                if seen.insert(key) {
                    results.push(result);
                }
            }
        }
        Ok(())
    }

    fn list_question_family_names_for_topic(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT family_name
                 FROM question_families
                 WHERE topic_id = ?1
                 ORDER BY family_name ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![topic_id, limit as i64], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(names)
    }

    fn list_formula_titles_for_topic(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT canonical_title
                 FROM academic_nodes
                 WHERE topic_id = ?1
                   AND node_type = 'formula'
                 ORDER BY exam_relevance_score DESC, foundation_weight DESC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![topic_id, limit as i64], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut titles = Vec::new();
        for row in rows {
            titles.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(titles)
    }

    fn topic_snapshot_recommended_actions(
        &self,
        mastery_score: Option<i64>,
        gap_score: Option<i64>,
        note_count: i64,
        hotspot_count: i64,
    ) -> Vec<String> {
        let mut actions = Vec::new();
        if gap_score.unwrap_or(0) >= 6000 {
            actions.push("Teach this topic again from first principles.".to_string());
        }
        if mastery_score.unwrap_or(0) >= 6500 && mastery_score.unwrap_or(0) < 8500 {
            actions.push(
                "Use a confidence-building revision pack to turn this into a stable strength."
                    .to_string(),
            );
        }
        if hotspot_count > 0 {
            actions.push("Review the exam hotspots before doing mixed practice.".to_string());
        }
        if note_count == 0 {
            actions.push(
                "Add a trap warning or memory hook so this topic becomes personal.".to_string(),
            );
        }
        if actions.is_empty() {
            actions
                .push("Keep this topic warm with short review and one timed question.".to_string());
        }
        actions.truncate(4);
        actions
    }

    fn list_shelf_items(&self, shelf_id: i64, limit: usize) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT item_type, item_ref_id, title, subtitle, reason, rank_score, metadata_json
                 FROM library_shelf_items
                 WHERE shelf_id = ?1
                 ORDER BY sequence_order ASC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![shelf_id, limit as i64], |row| {
                Ok(LibraryShelfItem {
                    item_type: row.get(0)?,
                    item_ref_id: row.get(1)?,
                    title: row.get(2)?,
                    subtitle: row.get(3)?,
                    reason: row.get(4)?,
                    rank_score: row.get(5)?,
                    metadata: parse_value_sql(row.get(6)?)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn lookup_shelf_rule_id(&self, shelf_type: &str) -> EcoachResult<Option<i64>> {
        self.conn
            .query_row(
                "SELECT id FROM shelf_generation_rules WHERE shelf_type = ?1",
                [shelf_type],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn find_revision_pack_template(
        &self,
        template_code: &str,
    ) -> EcoachResult<Option<RevisionPackTemplate>> {
        self.conn
            .query_row(
                "SELECT
                    id,
                    template_code,
                    display_name,
                    description,
                    pack_type,
                    selection_strategy,
                    default_item_count,
                    difficulty_profile,
                    topic_scope,
                    include_explanations,
                    include_worked_examples,
                    time_estimate_minutes
                 FROM revision_pack_templates
                 WHERE template_code = ?1
                   AND is_active = 1",
                [template_code],
                |row| {
                    Ok(RevisionPackTemplate {
                        id: row.get(0)?,
                        template_code: row.get(1)?,
                        display_name: row.get(2)?,
                        description: row.get(3)?,
                        pack_type: row.get(4)?,
                        selection_strategy: row.get(5)?,
                        default_item_count: row.get(6)?,
                        difficulty_profile: row.get(7)?,
                        topic_scope: row.get(8)?,
                        include_explanations: row.get::<_, i64>(9)? == 1,
                        include_worked_examples: row.get::<_, i64>(10)? == 1,
                        time_estimate_minutes: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn select_question_ids_for_template(
        &self,
        student_id: i64,
        template: &RevisionPackTemplate,
        item_limit: usize,
        subject_id: Option<i64>,
    ) -> EcoachResult<(Vec<i64>, Vec<i64>, String)> {
        let (question_ids, topic_ids) = match template.template_code.as_str() {
            "weak_area" => {
                let weak_topics = self.list_weak_topic_shelf_items(student_id, 4)?;
                let topic_ids = weak_topics
                    .iter()
                    .filter_map(|item| item.item_ref_id)
                    .collect::<Vec<_>>();
                (
                    self.collect_question_ids_from_topics(
                        student_id, &topic_ids, item_limit, true, false,
                    )?,
                    topic_ids,
                )
            }
            "mock_prep" => {
                let topic_ids = self.list_priority_topic_ids(student_id, subject_id, 5, None)?;
                (
                    self.collect_question_ids_from_topics(
                        student_id, &topic_ids, item_limit, false, true,
                    )?,
                    topic_ids,
                )
            }
            "formula_rescue" => {
                let topic_ids = self.list_formula_priority_topic_ids(student_id, subject_id, 4)?;
                (
                    self.collect_question_ids_from_topics(
                        student_id, &topic_ids, item_limit, false, false,
                    )?,
                    topic_ids,
                )
            }
            "last_minute" => {
                let topic_ids = self.list_high_yield_topic_ids(student_id, subject_id, 4)?;
                (
                    self.collect_question_ids_from_topics(
                        student_id, &topic_ids, item_limit, true, false,
                    )?,
                    topic_ids,
                )
            }
            "likely_exam" => {
                let families = self.list_exam_hotspots(student_id, subject_id, 5)?;
                let topic_ids = families
                    .iter()
                    .filter_map(|item| item.topic_id)
                    .collect::<Vec<_>>();
                let family_ids = families
                    .iter()
                    .map(|item| item.family_id)
                    .collect::<Vec<_>>();
                (
                    self.collect_question_ids_from_families(&family_ids, item_limit, false)?,
                    topic_ids,
                )
            }
            "things_i_fail" => {
                let question_ids =
                    self.list_recurring_mistake_question_ids(student_id, item_limit)?;
                let topic_ids = self.collect_topic_ids_for_questions(&question_ids)?;
                (question_ids, topic_ids)
            }
            "memory_rescue" => {
                let topic_ids = self.list_fading_topic_ids(student_id, subject_id, 4)?;
                (
                    self.collect_question_ids_from_topics(
                        student_id, &topic_ids, item_limit, false, false,
                    )?,
                    topic_ids,
                )
            }
            "confidence_build" => {
                let topic_ids = self.list_near_mastery_topic_ids(student_id, subject_id, 4)?;
                (
                    self.collect_question_ids_from_topics(
                        student_id, &topic_ids, item_limit, true, false,
                    )?,
                    topic_ids,
                )
            }
            _ => {
                let topic_ids = self.list_priority_topic_ids(student_id, subject_id, 4, None)?;
                (
                    self.collect_question_ids_from_topics(
                        student_id, &topic_ids, item_limit, false, false,
                    )?,
                    topic_ids,
                )
            }
        };
        Ok((
            question_ids,
            topic_ids,
            format!("generated_{}", template.template_code),
        ))
    }

    fn insert_revision_pack(
        &self,
        student_id: i64,
        title: &str,
        template: Option<&RevisionPackTemplate>,
        source_type: &str,
        topic_ids: Vec<i64>,
        question_ids: Vec<i64>,
        subject_id: Option<i64>,
    ) -> EcoachResult<RevisionPackSummary> {
        let metadata = json!({
            "pack_strategy": source_type,
            "topic_ids": topic_ids,
            "question_count": question_ids.len(),
            "subject_id": subject_id,
        });
        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO revision_packs (
                    student_id,
                    title,
                    source_type,
                    template_id,
                    pack_type,
                    subject_id,
                    question_count,
                    estimated_minutes,
                    difficulty_profile,
                    status,
                    metadata_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'ready', ?10)",
                params![
                    student_id,
                    title,
                    source_type,
                    template.map(|value| value.id),
                    template
                        .map(|value| value.pack_type.clone())
                        .unwrap_or_else(|| "custom".to_string()),
                    subject_id,
                    question_ids.len() as i64,
                    template.and_then(|value| value.time_estimate_minutes),
                    template
                        .map(|value| value.difficulty_profile.clone())
                        .unwrap_or_else(|| "custom".to_string()),
                    metadata_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let pack_id = self.conn.last_insert_rowid();
        for (index, question_id) in question_ids.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO revision_pack_items (
                        pack_id,
                        item_type,
                        item_ref_id,
                        sequence_order,
                        required,
                        metadata_json
                     ) VALUES (?1, 'question', ?2, ?3, 1, '{}')",
                    params![pack_id, question_id, index as i64],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        let created_at_raw: String = self
            .conn
            .query_row(
                "SELECT created_at FROM revision_packs WHERE id = ?1",
                [pack_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(RevisionPackSummary {
            pack_id,
            title: title.to_string(),
            source_type: Some(source_type.to_string()),
            template_code: template.map(|value| value.template_code.clone()),
            topic_ids,
            question_count: question_ids.len() as i64,
            estimated_minutes: template.and_then(|value| value.time_estimate_minutes),
            difficulty_profile: template.map(|value| value.difficulty_profile.clone()),
            status: Some("ready".to_string()),
            created_at: parse_datetime(&created_at_raw)?,
        })
    }

    fn collect_topic_ids_for_questions(&self, question_ids: &[i64]) -> EcoachResult<Vec<i64>> {
        let mut topic_ids = Vec::new();
        for question_id in question_ids {
            let topic_id = self
                .conn
                .query_row(
                    "SELECT topic_id FROM questions WHERE id = ?1",
                    [question_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(topic_id) = topic_id {
                if !topic_ids.contains(&topic_id) {
                    topic_ids.push(topic_id);
                }
            }
        }
        Ok(topic_ids)
    }

    fn make_generated_shelf(
        &self,
        shelf_type: &str,
        fallback_title: &str,
        icon_hint: Option<&str>,
        fallback_priority: i64,
        items: Vec<LibraryShelfItem>,
    ) -> EcoachResult<GeneratedLibraryShelf> {
        let (title, description, priority_order) =
            self.load_shelf_descriptor(shelf_type, fallback_title, fallback_priority)?;
        Ok(GeneratedLibraryShelf {
            shelf_id: None,
            shelf_type: shelf_type.to_string(),
            title,
            description,
            icon_hint: icon_hint.map(|value| value.to_string()),
            priority_order,
            generated: true,
            items,
        })
    }

    fn load_shelf_descriptor(
        &self,
        shelf_type: &str,
        fallback_title: &str,
        fallback_priority: i64,
    ) -> EcoachResult<(String, Option<String>, i64)> {
        if !self.table_exists("shelf_generation_rules")? {
            return Ok((fallback_title.to_string(), None, fallback_priority));
        }
        let value = self
            .conn
            .query_row(
                "SELECT display_name, description, priority_order
                 FROM shelf_generation_rules
                 WHERE shelf_type = ?1 AND is_active = 1",
                [shelf_type],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(value.unwrap_or_else(|| (fallback_title.to_string(), None, fallback_priority)))
    }

    fn list_saved_question_shelf_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        Ok(self
            .list_saved_questions(student_id, limit)?
            .into_iter()
            .map(|item| LibraryShelfItem {
                item_type: "question".to_string(),
                item_ref_id: Some(item.question_id),
                title: truncate_text(&item.stem, 72),
                subtitle: Some(item.topic_name),
                reason: format!(
                    "{} linked repair item(s) and state {}",
                    item.linked_knowledge_count, item.state
                ),
                rank_score: item.urgency_score,
                metadata: json!({
                    "library_item_id": item.library_item_id,
                    "family_name": item.related_family_name,
                    "open_count": 0,
                }),
            })
            .collect())
    }

    fn list_exam_hotspot_shelf_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        Ok(self
            .list_exam_hotspots(student_id, None, limit)?
            .into_iter()
            .map(|item| LibraryShelfItem {
                item_type: "question_family".to_string(),
                item_ref_id: Some(item.family_id),
                title: item.family_name,
                subtitle: item.topic_name,
                reason: item.reason.clone(),
                rank_score: item.current_relevance_bp,
                metadata: json!({
                    "recurrence_rate_bp": item.recurrence_rate_bp,
                    "persistence_score_bp": item.persistence_score_bp,
                    "current_relevance_bp": item.current_relevance_bp,
                    "exam_frequency_bp": item.current_relevance_bp,
                }),
            })
            .collect())
    }

    fn list_near_win_shelf_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, t.name, mastery_score, gap_score, priority_score
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                   AND sts.mastery_score BETWEEN 6500 AND 8500
                   AND sts.gap_score <= 3500
                 ORDER BY sts.gap_score ASC, sts.priority_score DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok(LibraryShelfItem {
                    item_type: "topic".to_string(),
                    item_ref_id: Some(row.get(0)?),
                    title: row.get(1)?,
                    subtitle: Some("near mastery".to_string()),
                    reason:
                        "This topic is close to becoming stable if you push it over the line now."
                            .to_string(),
                    rank_score: row.get(4)?,
                    metadata: json!({
                        "mastery_score": row.get::<_, i64>(2)?,
                        "gap_score": row.get::<_, i64>(3)?,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_untouched_important_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, item_type, item_ref_id, topic_id, exam_frequency_bp
                 FROM library_items
                 WHERE student_id = ?1
                   AND open_count = 0
                   AND COALESCE(exam_frequency_bp, 0) >= 5000
                 ORDER BY exam_frequency_bp DESC, urgency_score DESC, id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            let (library_item_id, item_type, item_ref_id, topic_id, exam_frequency_bp) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let (title, subtitle, _, _) =
                self.resolve_item_display_context(&item_type, item_ref_id, topic_id)?;
            items.push(LibraryShelfItem {
                item_type,
                item_ref_id: Some(item_ref_id),
                title,
                subtitle,
                reason: "Saved, important, and still never opened.".to_string(),
                rank_score: exam_frequency_bp.clamp(0, 10_000) as BasisPoints,
                metadata: json!({
                    "library_item_id": library_item_id,
                    "exam_frequency_bp": exam_frequency_bp,
                    "open_count": 0,
                }),
            });
        }
        Ok(items)
    }

    fn list_things_i_forget_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT ms.topic_id, t.name, ms.memory_state, ms.decay_risk, ms.review_due_at
                 FROM memory_states ms
                 LEFT JOIN topics t ON t.id = ms.topic_id
                 WHERE ms.student_id = ?1
                   AND ms.memory_state IN ('fading', 'collapsed', 'at_risk', 'fragile', 'rebuilding')
                 ORDER BY ms.decay_risk DESC, ms.review_due_at ASC, ms.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let memory_state: String = row.get(2)?;
                let review_due_at: Option<String> = row.get(4)?;
                Ok(LibraryShelfItem {
                    item_type: "topic".to_string(),
                    item_ref_id: row.get(0)?,
                    title: row
                        .get::<_, Option<String>>(1)?
                        .unwrap_or_else(|| "Memory topic".to_string()),
                    subtitle: Some(memory_state.clone()),
                    reason: format!(
                        "{} keeps slipping and needs deliberate reinforcement.",
                        memory_state
                    ),
                    rank_score: row.get::<_, i64>(3)? as BasisPoints,
                    metadata: json!({
                        "review_due_at": review_due_at,
                        "is_due": review_due_at.is_some(),
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_teach_me_again_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, t.name, mastery_score, gap_score, decay_risk
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                   AND (sts.gap_score >= 5000 OR sts.decay_risk >= 5000)
                 ORDER BY sts.gap_score DESC, sts.decay_risk DESC, sts.priority_score DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let mastery_score: i64 = row.get(2)?;
                let gap_score: i64 = row.get(3)?;
                Ok(LibraryShelfItem {
                    item_type: "topic".to_string(),
                    item_ref_id: Some(row.get(0)?),
                    title: row.get(1)?,
                    subtitle: Some("needs reteach".to_string()),
                    reason: "This topic should be taught again before you rely on it in a mixed session.".to_string(),
                    rank_score: ((gap_score + row.get::<_, i64>(4)?) / 2).clamp(0, 10_000) as BasisPoints,
                    metadata: json!({
                        "mastery_score": mastery_score,
                        "gap_score": gap_score,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_formula_bank_shelf_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    an.id,
                    an.canonical_title,
                    t.name,
                    an.exam_relevance_score,
                    COALESCE(sts.mastery_score, 0)
                 FROM academic_nodes an
                 LEFT JOIN topics t ON t.id = an.topic_id
                 LEFT JOIN student_topic_states sts
                    ON sts.student_id = ?1
                   AND sts.topic_id = an.topic_id
                 WHERE an.node_type = 'formula'
                   AND an.is_active = 1
                 ORDER BY an.exam_relevance_score DESC, sts.mastery_score ASC, an.id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                let exam_relevance_score: i64 = row.get(3)?;
                let mastery_score: i64 = row.get(4)?;
                Ok(LibraryShelfItem {
                    item_type: "formula".to_string(),
                    item_ref_id: Some(row.get(0)?),
                    title: row.get(1)?,
                    subtitle: row.get(2)?,
                    reason: "Keep this formula close because it is both testable and easy to forget under pressure.".to_string(),
                    rank_score: ((exam_relevance_score + (10_000 - mastery_score)) / 2)
                        .clamp(0, 10_000) as BasisPoints,
                    metadata: json!({
                        "exam_frequency_bp": exam_relevance_score,
                        "mastery_score": mastery_score,
                    }),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn list_concept_chain_items(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LibraryShelfItem>> {
        let weak_topics = self.list_weak_topic_shelf_items(student_id, limit)?;
        let mut items = Vec::new();
        for weak_topic in weak_topics {
            let Some(topic_id) = weak_topic.item_ref_id else {
                continue;
            };
            let hints = self.list_topic_relationship_hints(topic_id, 1)?;
            if let Some(hint) = hints.first() {
                items.push(LibraryShelfItem {
                    item_type: "concept_chain".to_string(),
                    item_ref_id: Some(topic_id),
                    title: format!("{} -> {}", hint.from_title, hint.to_title),
                    subtitle: Some(hint.relation_type.clone()),
                    reason: hint.explanation.clone(),
                    rank_score: hint.strength_score,
                    metadata: json!({
                        "focus_topic_id": hint.focus_topic_id,
                        "hop_count": hint.hop_count,
                    }),
                });
            }
            if items.len() >= limit {
                break;
            }
        }
        Ok(items)
    }

    fn collect_question_ids_from_topics(
        &self,
        student_id: i64,
        topic_ids: &[i64],
        limit: usize,
        saved_first: bool,
        harder_first: bool,
    ) -> EcoachResult<Vec<i64>> {
        let mut question_ids = Vec::new();
        let mut seen = HashSet::new();
        if saved_first {
            for topic_id in topic_ids {
                for question_id in
                    self.list_saved_question_ids_for_topic(student_id, *topic_id, limit)?
                {
                    if seen.insert(question_id) {
                        question_ids.push(question_id);
                    }
                    if question_ids.len() >= limit {
                        return Ok(question_ids);
                    }
                }
            }
        }
        for topic_id in topic_ids {
            for question_id in
                self.list_active_question_ids_for_topic_ranked(*topic_id, limit, harder_first)?
            {
                if seen.insert(question_id) {
                    question_ids.push(question_id);
                }
                if question_ids.len() >= limit {
                    return Ok(question_ids);
                }
            }
        }
        Ok(question_ids)
    }

    fn collect_question_ids_from_families(
        &self,
        family_ids: &[i64],
        limit: usize,
        harder_first: bool,
    ) -> EcoachResult<Vec<i64>> {
        let mut question_ids = Vec::new();
        let mut seen = HashSet::new();
        for family_id in family_ids {
            let order = if harder_first { "DESC" } else { "ASC" };
            let sql = format!(
                "SELECT id
                 FROM questions
                 WHERE is_active = 1
                   AND family_id = ?1
                 ORDER BY difficulty_level {}, id ASC
                 LIMIT ?2",
                order
            );
            let mut statement = self
                .conn
                .prepare(&sql)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map(params![family_id, limit as i64], |row| row.get::<_, i64>(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                let question_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
                if seen.insert(question_id) {
                    question_ids.push(question_id);
                }
                if question_ids.len() >= limit {
                    return Ok(question_ids);
                }
            }
        }
        Ok(question_ids)
    }

    fn list_priority_topic_ids(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        limit: usize,
        mastery_range: Option<(i64, i64)>,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.topic_id, sts.mastery_score
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1
                   AND (?2 IS NULL OR t.subject_id = ?2)
                 ORDER BY sts.priority_score DESC, sts.gap_score DESC, sts.mastery_score ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id, limit as i64 * 3], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut topic_ids = Vec::new();
        for row in rows {
            let (topic_id, mastery_score) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some((min_mastery, max_mastery)) = mastery_range {
                if !(min_mastery..=max_mastery).contains(&mastery_score) {
                    continue;
                }
            }
            if !topic_ids.contains(&topic_id) {
                topic_ids.push(topic_id);
            }
            if topic_ids.len() >= limit {
                break;
            }
        }
        Ok(topic_ids)
    }

    fn list_formula_priority_topic_ids(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT an.topic_id
                 FROM academic_nodes an
                 LEFT JOIN topics t ON t.id = an.topic_id
                 LEFT JOIN student_topic_states sts
                    ON sts.student_id = ?1
                   AND sts.topic_id = an.topic_id
                 WHERE an.node_type = 'formula'
                   AND (?2 IS NULL OR t.subject_id = ?2)
                 ORDER BY an.exam_relevance_score DESC, sts.gap_score DESC, an.topic_id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id, limit as i64], |row| {
                row.get::<_, Option<i64>>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut topic_ids = Vec::new();
        for row in rows {
            if let Some(topic_id) = row.map_err(|err| EcoachError::Storage(err.to_string()))? {
                topic_ids.push(topic_id);
            }
        }
        Ok(topic_ids)
    }

    fn list_high_yield_topic_ids(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let hotspots = self.list_exam_hotspots(student_id, subject_id, limit)?;
        let mut topic_ids = hotspots
            .iter()
            .filter_map(|item| item.topic_id)
            .collect::<Vec<_>>();
        if topic_ids.len() < limit {
            for topic_id in self.list_priority_topic_ids(student_id, subject_id, limit, None)? {
                if !topic_ids.contains(&topic_id) {
                    topic_ids.push(topic_id);
                }
                if topic_ids.len() >= limit {
                    break;
                }
            }
        }
        Ok(topic_ids)
    }

    fn list_recurring_mistake_question_ids(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT question_id
                 FROM wrong_answer_diagnoses
                 WHERE student_id = ?1
                 GROUP BY question_id
                 ORDER BY COUNT(*) DESC, MAX(created_at) DESC, question_id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, limit as i64], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut question_ids = Vec::new();
        for row in rows {
            question_ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(question_ids)
    }

    fn list_fading_topic_ids(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT ms.topic_id
                 FROM memory_states ms
                 LEFT JOIN topics t ON t.id = ms.topic_id
                 WHERE ms.student_id = ?1
                   AND (?2 IS NULL OR t.subject_id = ?2)
                   AND ms.memory_state IN ('fragile', 'at_risk', 'fading', 'collapsed', 'rebuilding')
                 ORDER BY ms.decay_risk DESC, ms.review_due_at ASC, ms.topic_id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id, limit as i64], |row| {
                row.get::<_, Option<i64>>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut topic_ids = Vec::new();
        for row in rows {
            if let Some(topic_id) = row.map_err(|err| EcoachError::Storage(err.to_string()))? {
                topic_ids.push(topic_id);
            }
        }
        Ok(topic_ids)
    }

    fn list_near_mastery_topic_ids(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        self.list_priority_topic_ids(student_id, subject_id, limit, Some((6500, 8500)))
    }

    fn list_active_question_ids_for_topic_ranked(
        &self,
        topic_id: i64,
        limit: usize,
        harder_first: bool,
    ) -> EcoachResult<Vec<i64>> {
        let order = if harder_first { "DESC" } else { "ASC" };
        let sql = format!(
            "SELECT id
             FROM questions
             WHERE topic_id = ?1
               AND is_active = 1
             ORDER BY difficulty_level {}, id ASC
             LIMIT ?2",
            order
        );
        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![topic_id, limit as i64], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(ids)
    }
}

fn parse_library_item_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LibraryItem> {
    Ok(LibraryItem {
        id: row.get(0)?,
        student_id: row.get(1)?,
        item_type: row.get(2)?,
        item_ref_id: row.get(3)?,
        state: row.get(4)?,
        tags: parse_string_vec_sql(row.get(5)?)?,
        note_text: row.get(6)?,
        topic_id: row.get(7)?,
        subject_id: row.get(8)?,
        subtopic_id: row.get(9)?,
        urgency_score: row.get(10)?,
        difficulty_bp: row.get(11)?,
        exam_frequency_bp: row.get(12)?,
        source: row.get(13)?,
        goal_id: row.get(14)?,
        calendar_event_id: row.get(15)?,
        last_opened_at: row.get(16)?,
        open_count: row.get(17)?,
        study_count: row.get(18)?,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
    })
}

fn matches_query_text(value: &str, query: Option<&str>) -> bool {
    let Some(query) = query else {
        return true;
    };
    value.to_lowercase().contains(&query.to_lowercase())
}

fn query_bonus(primary: &str, secondary: &str, query: Option<&str>) -> i64 {
    let Some(query) = query else {
        return 0;
    };
    if matches_query_text(primary, Some(query)) {
        return 2000;
    }
    if matches_query_text(secondary, Some(query)) {
        return 900;
    }
    0
}

fn parse_tags(raw: &str) -> EcoachResult<Vec<String>> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_value(raw: &str) -> EcoachResult<Value> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_value_sql(raw: String) -> rusqlite::Result<Value> {
    serde_json::from_str(&raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })
}

fn parse_string_vec_sql(raw: String) -> rusqlite::Result<Vec<String>> {
    serde_json::from_str(&raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })
}

fn parse_datetime(raw: &str) -> EcoachResult<DateTime<Utc>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(raw) {
        return Ok(parsed.with_timezone(&Utc));
    }

    NaiveDateTime::parse_from_str(raw, "%Y-%m-%d %H:%M:%S")
        .map(|value| value.and_utc())
        .map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn truncate_text(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        return value.to_string();
    }

    let truncated: String = value.chars().take(max_len.saturating_sub(3)).collect();
    format!("{}...", truncated.trim_end())
}

fn readiness_band_for_action(action_type: &str) -> &'static str {
    match action_type {
        "reteach_foundation" => "repair_now",
        "memory_reactivation" => "memory_rebuild",
        "guided_practice" => "guided_growth",
        _ => "exam_bridge",
    }
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn push_unique_relationship_hint(
    hints: &mut Vec<TopicRelationshipHint>,
    hint: TopicRelationshipHint,
) {
    if !hints.iter().any(|existing| {
        existing.relation_type == hint.relation_type
            && existing.from_title == hint.from_title
            && existing.to_title == hint.to_title
    }) {
        hints.push(hint);
    }
}

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.collect::<String>()),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use serde_json::json;

    use super::LibraryService;
    use crate::models::{
        AddLibraryNoteInput, AddShelfItemInput, BuildRevisionPackFromTemplateInput,
        CreateCustomShelfInput, LibrarySearchInput, RecordLibraryItemActionInput,
        SaveLibraryItemInput, TeachExplanationUpsertInput, TeachMicroCheckInput,
        TutorInteractionInput,
    };

    fn create_teach_tutor_schema(conn: &Connection) {
        conn.execute_batch(
            "
            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE academic_nodes (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                canonical_title TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                foundation_weight INTEGER NOT NULL DEFAULT 0,
                exam_relevance_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE teach_explanations (
                id INTEGER PRIMARY KEY,
                node_id INTEGER NOT NULL,
                explanation_level TEXT NOT NULL,
                hero_summary TEXT,
                why_it_matters TEXT,
                simple_explanation TEXT,
                structured_breakdown_json TEXT NOT NULL,
                worked_examples_json TEXT NOT NULL,
                common_mistakes_json TEXT NOT NULL,
                exam_appearance_notes TEXT,
                pattern_recognition_tips TEXT,
                related_concepts_json TEXT NOT NULL,
                visual_asset_refs_json TEXT NOT NULL,
                subject_style TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(node_id, explanation_level)
            );
            CREATE TABLE teach_micro_checks (
                id INTEGER PRIMARY KEY,
                explanation_id INTEGER NOT NULL,
                check_type TEXT NOT NULL,
                prompt TEXT NOT NULL,
                correct_answer TEXT NOT NULL,
                distractor_answers_json TEXT NOT NULL,
                explanation_if_wrong TEXT,
                position_index INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE questions (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER NOT NULL,
                stem TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                difficulty_level INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE question_options (
                id INTEGER PRIMARY KEY,
                question_id INTEGER NOT NULL,
                option_label TEXT,
                option_text TEXT NOT NULL,
                is_correct INTEGER NOT NULL DEFAULT 0,
                misconception_id INTEGER,
                distractor_intent TEXT,
                position INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE knowledge_entries (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                title TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                importance_score INTEGER NOT NULL DEFAULT 0,
                difficulty_level INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE library_items (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                item_ref_id INTEGER NOT NULL,
                state TEXT NOT NULL DEFAULT 'saved',
                tags_json TEXT NOT NULL DEFAULT '[]',
                note_text TEXT,
                topic_id INTEGER,
                urgency_score INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE student_topic_states (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                mastery_score INTEGER NOT NULL,
                gap_score INTEGER NOT NULL,
                trend_state TEXT NOT NULL,
                decay_risk INTEGER NOT NULL DEFAULT 0,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                next_review_at TEXT
            );
            CREATE TABLE wrong_answer_diagnoses (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                primary_diagnosis TEXT NOT NULL,
                recommended_action TEXT NOT NULL,
                severity TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE tutor_interactions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                session_id INTEGER,
                question_id INTEGER,
                topic_id INTEGER,
                interaction_type TEXT NOT NULL,
                prompt_text TEXT,
                response_text TEXT,
                context_json TEXT NOT NULL,
                was_helpful INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE node_edges (
                id INTEGER PRIMARY KEY,
                edge_type TEXT NOT NULL,
                from_node_type TEXT NOT NULL,
                from_node_id INTEGER NOT NULL,
                to_node_type TEXT NOT NULL,
                to_node_id INTEGER NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE knowledge_relations (
                id INTEGER PRIMARY KEY,
                from_entry_id INTEGER NOT NULL,
                to_entry_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE memory_states (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER,
                memory_state TEXT NOT NULL
            );
            ",
        )
        .expect("schema");
    }

    fn create_library_intelligence_schema(conn: &Connection) {
        conn.execute_batch(
            "
            CREATE TABLE subjects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                parent_topic_id INTEGER,
                name TEXT NOT NULL,
                exam_weight INTEGER NOT NULL DEFAULT 5000
            );
            CREATE TABLE question_families (
                id INTEGER PRIMARY KEY,
                family_name TEXT NOT NULL,
                subject_id INTEGER NOT NULL,
                topic_id INTEGER
            );
            CREATE TABLE questions (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                subtopic_id INTEGER,
                family_id INTEGER,
                stem TEXT NOT NULL,
                difficulty_level INTEGER NOT NULL DEFAULT 5000,
                source_type TEXT,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE academic_nodes (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                node_type TEXT NOT NULL,
                canonical_title TEXT NOT NULL,
                exam_relevance_score INTEGER NOT NULL DEFAULT 5000,
                foundation_weight INTEGER NOT NULL DEFAULT 5000,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE knowledge_entries (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                title TEXT NOT NULL,
                difficulty_level INTEGER NOT NULL DEFAULT 0,
                importance_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE student_topic_states (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                mastery_score INTEGER NOT NULL,
                mastery_state TEXT NOT NULL DEFAULT 'partial',
                gap_score INTEGER NOT NULL DEFAULT 0,
                priority_score INTEGER NOT NULL DEFAULT 0,
                trend_state TEXT NOT NULL DEFAULT 'stable',
                decay_risk INTEGER NOT NULL DEFAULT 0,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                next_review_at TEXT
            );
            CREATE TABLE memory_states (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER,
                node_id INTEGER,
                memory_state TEXT NOT NULL,
                memory_strength INTEGER NOT NULL DEFAULT 0,
                decay_risk INTEGER NOT NULL DEFAULT 0,
                review_due_at TEXT
            );
            CREATE TABLE wrong_answer_diagnoses (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                question_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                primary_diagnosis TEXT NOT NULL,
                recommended_action TEXT NOT NULL,
                severity TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE coach_mission_memories (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER,
                review_status TEXT NOT NULL DEFAULT 'pending',
                next_action_type TEXT,
                review_due_at TEXT,
                accuracy_score INTEGER
            );
            CREATE TABLE coach_missions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                activity_type TEXT NOT NULL,
                primary_topic_id INTEGER,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE sessions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                session_type TEXT NOT NULL,
                active_item_index INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                last_activity_at TEXT,
                updated_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE session_items (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL,
                source_topic_id INTEGER,
                question_id INTEGER,
                display_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE question_glossary_links (
                id INTEGER PRIMARY KEY,
                question_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL
            );
            CREATE TABLE family_recurrence_metrics (
                id INTEGER PRIMARY KEY,
                family_id INTEGER NOT NULL,
                subject_id INTEGER NOT NULL,
                recurrence_rate_bp INTEGER NOT NULL DEFAULT 0,
                persistence_score_bp INTEGER NOT NULL DEFAULT 0,
                current_relevance_bp INTEGER NOT NULL DEFAULT 0,
                last_appearance_year INTEGER,
                first_appearance_year INTEGER
            );
            CREATE TABLE student_family_performance (
                student_id INTEGER NOT NULL,
                family_id INTEGER NOT NULL,
                accuracy_rate_bp INTEGER
            );
            CREATE TABLE library_items (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                item_ref_id INTEGER NOT NULL,
                state TEXT NOT NULL DEFAULT 'saved',
                tags_json TEXT NOT NULL DEFAULT '[]',
                note_text TEXT,
                topic_id INTEGER,
                subject_id INTEGER,
                subtopic_id INTEGER,
                urgency_score INTEGER NOT NULL DEFAULT 5000,
                difficulty_bp INTEGER,
                exam_frequency_bp INTEGER,
                source TEXT,
                goal_id INTEGER,
                calendar_event_id INTEGER,
                last_opened_at TEXT,
                open_count INTEGER NOT NULL DEFAULT 0,
                study_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE library_item_state_history (
                id INTEGER PRIMARY KEY,
                library_item_id INTEGER NOT NULL,
                from_state TEXT,
                to_state TEXT NOT NULL,
                reason TEXT,
                changed_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE library_item_actions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                library_item_id INTEGER NOT NULL,
                action_type TEXT NOT NULL,
                context_json TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE library_notes (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                library_item_id INTEGER,
                topic_id INTEGER,
                note_type TEXT NOT NULL,
                title TEXT,
                note_text TEXT NOT NULL,
                context_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE library_shelves (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                shelf_type TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                icon_hint TEXT,
                generated INTEGER NOT NULL DEFAULT 1,
                item_count INTEGER NOT NULL DEFAULT 0,
                priority_order INTEGER NOT NULL DEFAULT 10,
                last_refreshed_at TEXT,
                rule_id INTEGER
            );
            CREATE TABLE library_shelf_items (
                id INTEGER PRIMARY KEY,
                shelf_id INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                item_ref_id INTEGER,
                title TEXT NOT NULL,
                subtitle TEXT,
                reason TEXT NOT NULL,
                rank_score INTEGER NOT NULL DEFAULT 0,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                sequence_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE shelf_generation_rules (
                id INTEGER PRIMARY KEY,
                shelf_type TEXT NOT NULL,
                display_name TEXT NOT NULL,
                description TEXT,
                is_active INTEGER NOT NULL DEFAULT 1,
                priority_order INTEGER NOT NULL DEFAULT 10
            );
            CREATE TABLE revision_pack_templates (
                id INTEGER PRIMARY KEY,
                template_code TEXT NOT NULL,
                display_name TEXT NOT NULL,
                description TEXT,
                pack_type TEXT NOT NULL,
                selection_strategy TEXT NOT NULL,
                default_item_count INTEGER NOT NULL DEFAULT 15,
                difficulty_profile TEXT NOT NULL DEFAULT 'balanced',
                topic_scope TEXT NOT NULL DEFAULT 'weak_topics',
                include_explanations INTEGER NOT NULL DEFAULT 1,
                include_worked_examples INTEGER NOT NULL DEFAULT 1,
                time_estimate_minutes INTEGER,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE revision_packs (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                source_type TEXT,
                template_id INTEGER,
                pack_type TEXT,
                subject_id INTEGER,
                question_count INTEGER NOT NULL DEFAULT 0,
                estimated_minutes INTEGER,
                difficulty_profile TEXT,
                status TEXT NOT NULL DEFAULT 'ready',
                metadata_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE revision_pack_items (
                id INTEGER PRIMARY KEY,
                pack_id INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                item_ref_id INTEGER NOT NULL,
                sequence_order INTEGER NOT NULL DEFAULT 0,
                required INTEGER NOT NULL DEFAULT 1,
                metadata_json TEXT NOT NULL DEFAULT '{}'
            );
            CREATE TABLE node_edges (
                id INTEGER PRIMARY KEY,
                edge_type TEXT NOT NULL,
                from_node_type TEXT NOT NULL,
                from_node_id INTEGER NOT NULL,
                to_node_type TEXT NOT NULL,
                to_node_id INTEGER NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE knowledge_relations (
                id INTEGER PRIMARY KEY,
                from_entry_id INTEGER NOT NULL,
                to_entry_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE library_tag_definitions (
                id INTEGER PRIMARY KEY,
                tag_code TEXT NOT NULL,
                display_name TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT,
                color_hint TEXT,
                is_system INTEGER NOT NULL DEFAULT 1
            );
            ",
        )
        .expect("idea16 schema");
    }

    #[test]
    fn build_teach_action_plan_surfaces_diagnostic_and_relationship_depth() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "
            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE student_topic_states (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                mastery_score INTEGER NOT NULL,
                gap_score INTEGER NOT NULL,
                trend_state TEXT NOT NULL,
                decay_risk INTEGER NOT NULL DEFAULT 0,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                next_review_at TEXT
            );
            CREATE TABLE memory_states (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER,
                memory_state TEXT NOT NULL
            );
            CREATE TABLE questions (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER NOT NULL,
                stem TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                difficulty_level INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE library_items (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                item_ref_id INTEGER NOT NULL,
                urgency_score INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE knowledge_entries (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                title TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                importance_score INTEGER NOT NULL DEFAULT 0,
                difficulty_level INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE academic_nodes (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                canonical_title TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                foundation_weight INTEGER NOT NULL DEFAULT 0,
                exam_relevance_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE node_edges (
                id INTEGER PRIMARY KEY,
                edge_type TEXT NOT NULL,
                from_node_type TEXT NOT NULL,
                from_node_id INTEGER NOT NULL,
                to_node_type TEXT NOT NULL,
                to_node_id INTEGER NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE knowledge_relations (
                id INTEGER PRIMARY KEY,
                from_entry_id INTEGER NOT NULL,
                to_entry_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE student_error_profiles (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                knowledge_gap_score INTEGER NOT NULL DEFAULT 0,
                conceptual_confusion_score INTEGER NOT NULL DEFAULT 0,
                recognition_failure_score INTEGER NOT NULL DEFAULT 0,
                execution_error_score INTEGER NOT NULL DEFAULT 0,
                carelessness_score INTEGER NOT NULL DEFAULT 0,
                pressure_breakdown_score INTEGER NOT NULL DEFAULT 0,
                expression_weakness_score INTEGER NOT NULL DEFAULT 0,
                speed_error_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE wrong_answer_diagnoses (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                primary_diagnosis TEXT NOT NULL,
                recommended_action TEXT NOT NULL,
                severity TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )
        .expect("schema");

        conn.execute(
            "INSERT INTO topics (id, name) VALUES (1, 'Quadratic Equations')",
            [],
        )
        .expect("topic");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, trend_state, decay_risk, is_blocked, next_review_at
             ) VALUES (42, 1, 3200, 7200, 'declining', 6800, 1, '2026-03-29T12:00:00Z')",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO memory_states (id, student_id, topic_id, memory_state)
             VALUES (1, 42, 1, 'fragile')",
            [],
        )
        .expect("memory state");
        conn.execute(
            "INSERT INTO questions (id, topic_id, stem, is_active, difficulty_level)
             VALUES (10, 1, 'Solve x^2 - 5x + 6 = 0', 1, 1200)",
            [],
        )
        .expect("saved question");
        conn.execute(
            "INSERT INTO questions (id, topic_id, stem, is_active, difficulty_level)
             VALUES (11, 1, 'Rewrite the quadratic in vertex form', 1, 1800)",
            [],
        )
        .expect("active question");
        conn.execute(
            "INSERT INTO library_items (id, student_id, item_type, item_ref_id, urgency_score)
             VALUES (1, 42, 'question', 10, 9000)",
            [],
        )
        .expect("library item");
        conn.execute(
            "INSERT INTO knowledge_entries (id, topic_id, title, status, importance_score, difficulty_level)
             VALUES (101, 1, 'Completing the square', 'active', 9000, 2000)",
            [],
        )
        .expect("entry one");
        conn.execute(
            "INSERT INTO knowledge_entries (id, topic_id, title, status, importance_score, difficulty_level)
             VALUES (102, 1, 'Vertex form meaning', 'active', 8500, 1500)",
            [],
        )
        .expect("entry two");
        conn.execute(
            "INSERT INTO academic_nodes (id, topic_id, canonical_title, is_active, foundation_weight, exam_relevance_score)
             VALUES (201, 1, 'Recognise when a quadratic can be factored', 1, 9500, 8000)",
            [],
        )
        .expect("node one");
        conn.execute(
            "INSERT INTO academic_nodes (id, topic_id, canonical_title, is_active, foundation_weight, exam_relevance_score)
             VALUES (202, 1, 'Connect factorisation to roots', 1, 9000, 7600)",
            [],
        )
        .expect("node two");
        conn.execute(
            "INSERT INTO node_edges (
                id, edge_type, from_node_type, from_node_id, to_node_type, to_node_id, strength_score
             ) VALUES (1, 'prerequisite', 'academic_node', 201, 'academic_node', 202, 9100)",
            [],
        )
        .expect("node edge");
        conn.execute(
            "INSERT INTO knowledge_relations (
                id, from_entry_id, to_entry_id, relation_type, strength_score
             ) VALUES (1, 101, 102, 'contrasts_with', 8800)",
            [],
        )
        .expect("knowledge relation");
        conn.execute(
            "INSERT INTO student_error_profiles (
                student_id, topic_id, knowledge_gap_score, conceptual_confusion_score,
                recognition_failure_score, execution_error_score, carelessness_score,
                pressure_breakdown_score, expression_weakness_score, speed_error_score
             ) VALUES (42, 1, 7200, 6100, 0, 5600, 0, 5200, 4800, 0)",
            [],
        )
        .expect("error profile");
        conn.execute(
            "INSERT INTO wrong_answer_diagnoses (
                id, student_id, topic_id, primary_diagnosis, recommended_action, severity, created_at
             ) VALUES (
                1, 42, 1, 'sign confusion', 'Rebuild the sign pattern before solving', 'high', '2026-03-29T10:00:00Z'
             )",
            [],
        )
        .expect("diagnosis");

        let plan = LibraryService::new(&conn)
            .build_teach_action_plan(42, 1, 4)
            .expect("teach plan");

        assert_eq!(plan.action_type, "reteach_foundation");
        assert_eq!(plan.readiness_band, "repair_now");
        assert_eq!(plan.support_intensity, "high");
        assert!(!plan.diagnostic_focuses.is_empty());
        assert!(
            plan.diagnostic_focuses
                .iter()
                .any(|focus| focus.contains("Contrast") || focus.contains("contrast"))
        );
        assert!(
            plan.recent_diagnoses
                .iter()
                .any(|diagnosis| diagnosis.contains("sign confusion"))
        );
        assert!(!plan.relationship_hints.is_empty());
        assert!(plan.recommended_sequence.len() >= 4);
        assert!(
            plan.recommended_sequence
                .iter()
                .any(|step| step.step_type == "relationship")
        );
        assert!(
            plan.recommended_sequence
                .iter()
                .any(|step| step.step_type == "repair")
        );
    }

    #[test]
    fn personalized_learning_paths_prioritize_bundle_and_relationship_route() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "
            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE student_topic_states (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                mastery_score INTEGER NOT NULL,
                gap_score INTEGER NOT NULL,
                priority_score INTEGER NOT NULL DEFAULT 0,
                trend_state TEXT NOT NULL,
                decay_risk INTEGER NOT NULL DEFAULT 0,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                next_review_at TEXT
            );
            CREATE TABLE node_edges (
                id INTEGER PRIMARY KEY,
                edge_type TEXT NOT NULL,
                from_node_type TEXT NOT NULL,
                from_node_id INTEGER NOT NULL,
                to_node_type TEXT NOT NULL,
                to_node_id INTEGER NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE academic_nodes (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                canonical_title TEXT NOT NULL
            );
            CREATE TABLE knowledge_entries (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                title TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                importance_score INTEGER NOT NULL DEFAULT 0,
                difficulty_level INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE knowledge_relations (
                id INTEGER PRIMARY KEY,
                from_entry_id INTEGER NOT NULL,
                to_entry_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE knowledge_bundles (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                topic_id INTEGER,
                exam_relevance_score INTEGER NOT NULL DEFAULT 0,
                difficulty_level INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE knowledge_bundle_items (
                id INTEGER PRIMARY KEY,
                bundle_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                sequence_order INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE student_entry_state (
                user_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                confusion_score INTEGER NOT NULL DEFAULT 0,
                recall_strength INTEGER NOT NULL DEFAULT 0,
                linked_wrong_answer_count INTEGER NOT NULL DEFAULT 0,
                review_due_at TEXT
            );
            CREATE TABLE questions (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                difficulty_level INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE library_items (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                item_ref_id INTEGER NOT NULL,
                urgency_score INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE coach_missions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                activity_type TEXT NOT NULL,
                primary_topic_id INTEGER,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE sessions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                session_type TEXT NOT NULL,
                active_item_index INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                last_activity_at TEXT,
                updated_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE session_items (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL,
                source_topic_id INTEGER,
                question_id INTEGER,
                display_order INTEGER NOT NULL DEFAULT 0
            );
            ",
        )
        .expect("schema");

        conn.execute(
            "INSERT INTO topics (id, name) VALUES
                (1, 'Algebraic Fractions'),
                (2, 'Equivalent Fractions'),
                (3, 'Factorisation')",
            [],
        )
        .expect("topics");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, priority_score, trend_state, decay_risk, is_blocked, next_review_at
             ) VALUES (42, 1, 2800, 7600, 9100, 'declining', 6400, 1, '2026-03-29T10:00:00Z')",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO node_edges (
                id, edge_type, from_node_type, from_node_id, to_node_type, to_node_id, strength_score
             ) VALUES
                (1, 'prerequisite', 'topic', 2, 'topic', 1, 9300),
                (2, 'related', 'topic', 3, 'topic', 2, 8600)",
            [],
        )
        .expect("node edges");
        conn.execute(
            "INSERT INTO knowledge_entries (id, topic_id, title, status, importance_score, difficulty_level)
             VALUES
                (101, 1, 'Simplify before multiplying', 'active', 9000, 1200),
                (102, 1, 'Common denominator check', 'active', 8700, 1500)",
            [],
        )
        .expect("entries");
        conn.execute(
            "INSERT INTO knowledge_bundles (id, title, topic_id, exam_relevance_score, difficulty_level)
             VALUES (10, 'Fractions Recovery', 1, 9200, 1300)",
            [],
        )
        .expect("bundle");
        conn.execute(
            "INSERT INTO knowledge_bundle_items (id, bundle_id, entry_id, sequence_order)
             VALUES
                (1, 10, 101, 0),
                (2, 10, 102, 1)",
            [],
        )
        .expect("bundle items");
        conn.execute(
            "INSERT INTO student_entry_state (
                user_id, entry_id, confusion_score, recall_strength, linked_wrong_answer_count, review_due_at
             ) VALUES
                (42, 101, 7200, 2200, 2, '2026-03-29T09:00:00Z'),
                (42, 102, 4800, 2600, 1, NULL)",
            [],
        )
        .expect("entry states");
        conn.execute(
            "INSERT INTO questions (id, topic_id, is_active, difficulty_level)
             VALUES (50, 1, 1, 1000)",
            [],
        )
        .expect("question");
        conn.execute(
            "INSERT INTO library_items (id, student_id, item_type, item_ref_id, urgency_score)
             VALUES (1, 42, 'question', 50, 8800)",
            [],
        )
        .expect("library item");

        let service = LibraryService::new(&conn);
        let paths = service
            .build_personalized_learning_paths(42, 2)
            .expect("learning paths");

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].topic_id, 1);
        assert_eq!(paths[0].activity_type, "topic_repair");
        assert_eq!(
            paths[0].recommended_bundle_titles,
            vec!["Fractions Recovery".to_string()]
        );
        assert!(
            paths[0]
                .related_topic_names
                .iter()
                .any(|name| name == "Equivalent Fractions")
        );
        assert!(
            paths[0]
                .relationship_hints
                .iter()
                .any(|hint| hint.hop_count == 2 && hint.to_title == "Factorisation")
        );
        assert!(
            paths[0]
                .steps
                .iter()
                .any(|step| step.step_type == "bundle" && step.bundle_id == Some(10))
        );

        let continue_card = service
            .get_continue_learning_card(42)
            .expect("continue card")
            .expect("fallback continue card");
        assert_eq!(continue_card.topic_id, Some(1));
        assert_eq!(continue_card.recommended_bundle_ids, vec![10]);
        assert!(
            continue_card
                .related_topic_names
                .iter()
                .any(|name| name == "Equivalent Fractions")
        );
        assert!(continue_card.reason.is_some());
    }

    #[test]
    fn teach_lesson_round_trips_explanations_and_micro_checks() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_teach_tutor_schema(&conn);

        conn.execute(
            "INSERT INTO topics (id, name) VALUES (1, 'Linear Equations')",
            [],
        )
        .expect("topic");
        conn.execute(
            "INSERT INTO academic_nodes (
                id, topic_id, canonical_title, is_active, foundation_weight, exam_relevance_score
             ) VALUES (10, 1, 'Isolate the variable', 1, 9000, 8500)",
            [],
        )
        .expect("node");
        conn.execute(
            "INSERT INTO questions (id, topic_id, stem, is_active, difficulty_level)
             VALUES
                (100, 1, 'Solve 2x + 3 = 11', 1, 1000),
                (101, 1, 'Check the rearrangement step', 1, 1200)",
            [],
        )
        .expect("questions");

        let service = LibraryService::new(&conn);
        let base_input = TeachExplanationUpsertInput {
            explanation_level: "core".to_string(),
            hero_summary: Some("Strip the equation down one step at a time.".to_string()),
            why_it_matters: Some(
                "It keeps the learner focused on the inverse operation.".to_string(),
            ),
            simple_explanation: Some("Subtract the constant, then divide to finish.".to_string()),
            structured_breakdown: json!({"steps": ["remove constant", "divide coefficient"]}),
            worked_examples: json!(["2x + 3 = 11 -> x = 4"]),
            common_mistakes: json!([
                "subtracting the wrong side",
                "stopping before the final check"
            ]),
            exam_appearance_notes: Some("Appears as a direct rearrangement question.".to_string()),
            pattern_recognition_tips: Some(
                "Watch for a constant term attached to the variable.".to_string(),
            ),
            related_concepts: json!(["inverse operations", "balancing equations"]),
            visual_asset_refs: json!(["board-scratch"]),
            subject_style: Some("calm".to_string()),
        };

        let explanation_id = service
            .upsert_teach_explanation(10, &base_input)
            .expect("insert explanation");

        let mut updated_input = base_input.clone();
        updated_input.hero_summary = Some("Updated summary for the anchor concept.".to_string());
        let same_id = service
            .upsert_teach_explanation(10, &updated_input)
            .expect("update explanation");
        assert_eq!(explanation_id, same_id);

        service
            .add_teach_micro_check(
                explanation_id,
                &TeachMicroCheckInput {
                    check_type: "retrieval".to_string(),
                    prompt: "What is the first move?".to_string(),
                    correct_answer: "Subtract 3 from both sides.".to_string(),
                    distractor_answers: vec!["Add 3 to both sides.".to_string()],
                    explanation_if_wrong: Some("Undo the constant first.".to_string()),
                    position_index: 1,
                },
            )
            .expect("micro check one");
        service
            .add_teach_micro_check(
                explanation_id,
                &TeachMicroCheckInput {
                    check_type: "retrieval".to_string(),
                    prompt: "What do you do after removing the constant?".to_string(),
                    correct_answer: "Divide by the coefficient.".to_string(),
                    distractor_answers: vec!["Multiply by the coefficient.".to_string()],
                    explanation_if_wrong: Some("Finish the inverse operation chain.".to_string()),
                    position_index: 0,
                },
            )
            .expect("micro check two");

        let lesson = service
            .get_teach_lesson(1, Some("core"), 2)
            .expect("teach lesson");

        assert_eq!(lesson.topic_id, 1);
        assert_eq!(lesson.node_id, Some(10));
        assert_eq!(lesson.node_title.as_deref(), Some("Isolate the variable"));
        assert!(!lesson.generated);
        let explanation = lesson.explanation.as_ref().expect("lesson explanation");
        assert_eq!(explanation.id, explanation_id);
        assert_eq!(
            explanation.hero_summary.as_deref(),
            Some("Updated summary for the anchor concept.")
        );
        assert_eq!(lesson.micro_checks.len(), 2);
        assert_eq!(lesson.micro_checks[0].position_index, 0);
        assert_eq!(
            lesson.micro_checks[0].prompt,
            "What do you do after removing the constant?"
        );
        assert_eq!(lesson.micro_checks[1].position_index, 1);
        assert_eq!(
            lesson.micro_checks[1].correct_answer,
            "Subtract 3 from both sides."
        );
    }

    #[test]
    fn tutor_response_uses_topic_question_and_history_context() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_teach_tutor_schema(&conn);

        conn.execute(
            "INSERT INTO topics (id, name) VALUES (1, 'Quadratic Equations')",
            [],
        )
        .expect("topic");
        conn.execute(
            "INSERT INTO academic_nodes (
                id, topic_id, canonical_title, is_active, foundation_weight, exam_relevance_score
             ) VALUES (20, 1, 'Factor when the roots are visible', 1, 9500, 8600)",
            [],
        )
        .expect("node");
        conn.execute(
            r#"INSERT INTO teach_explanations (
                id,
                node_id,
                explanation_level,
                hero_summary,
                why_it_matters,
                simple_explanation,
                structured_breakdown_json,
                worked_examples_json,
                common_mistakes_json,
                exam_appearance_notes,
                pattern_recognition_tips,
                related_concepts_json,
                visual_asset_refs_json,
                subject_style
             ) VALUES (
                1,
                20,
                'core',
                'Factor the equation by finding the matching pair.',
                'It is the fastest path when the factors are obvious.',
                'Look for two numbers that multiply to the constant and add to the middle term.',
                '{}',
                '["x^2 + 5x + 6 = 0 -> (x + 2)(x + 3) = 0"]',
                '["mixing up the sign pattern"]',
                'Shows up as a core factorisation question.',
                'Look for the pair that fits both the product and the sum.',
                '["factorisation", "sign pattern"]',
                '[]',
                'exam'
             )"#,
            [],
        )
        .expect("teach explanation");
        conn.execute(
            r#"INSERT INTO teach_micro_checks (
                id, explanation_id, check_type, prompt, correct_answer,
                distractor_answers_json, explanation_if_wrong, position_index
             ) VALUES (
                1, 1, 'retrieval', 'What pair fits the constant 6 and middle term 5?',
                '2 and 3', '["1 and 6"]', 'Use the pair that matches both conditions.', 0
             )"#,
            [],
        )
        .expect("micro check");
        conn.execute(
            "INSERT INTO knowledge_entries (
                id, topic_id, title, status, importance_score, difficulty_level
             ) VALUES
                (201, 1, 'Product-sum pairing', 'active', 9000, 1200),
                (202, 1, 'Zero-product rule', 'active', 8600, 1400)",
            [],
        )
        .expect("entries");
        conn.execute(
            "INSERT INTO questions (id, topic_id, stem, is_active, difficulty_level)
             VALUES
                (100, 1, 'Which option factors x^2 + 5x + 6 = 0?', 1, 1000),
                (101, 1, 'What two numbers multiply to 6 and add to 5?', 1, 1200)",
            [],
        )
        .expect("questions");
        conn.execute(
            "INSERT INTO question_options (
                id, question_id, option_label, option_text, is_correct, position
             ) VALUES
                (1, 100, 'A', '(x + 1)(x + 6) = 0', 0, 0),
                (2, 100, 'B', '(x + 2)(x + 3) = 0', 1, 1),
                (3, 100, 'C', '(x - 2)(x - 3) = 0', 0, 2)",
            [],
        )
        .expect("options");
        conn.execute(
            "INSERT INTO library_items (
                id, student_id, item_type, item_ref_id, state, tags_json, note_text, topic_id, urgency_score
             ) VALUES (1, 42, 'question', 100, 'saved', '[]', NULL, 1, 8500)",
            [],
        )
        .expect("library item");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, trend_state, decay_risk, is_blocked, next_review_at
             ) VALUES (42, 1, 3200, 6800, 'declining', 5400, 0, '2026-03-31T10:00:00Z')",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO wrong_answer_diagnoses (
                id, student_id, topic_id, primary_diagnosis, recommended_action, severity, created_at
             ) VALUES (
                1, 42, 1, 'sign confusion', 'Check the sign pattern before choosing factors.', 'high', '2026-03-31T09:15:00Z'
             )",
            [],
        )
        .expect("diagnosis");

        let service = LibraryService::new(&conn);
        service
            .log_tutor_interaction(&TutorInteractionInput {
                student_id: 42,
                session_id: Some(7),
                question_id: Some(100),
                topic_id: Some(1),
                interaction_type: "explain_simply".to_string(),
                prompt_text: Some("Earlier warm-up".to_string()),
                response_text: Some(
                    "Use the factor pair that matches both the sum and product.".to_string(),
                ),
                context: json!({"seed": true}),
                was_helpful: Some(true),
            })
            .expect("seed history");

        let response = service
            .ask_tutor(&TutorInteractionInput {
                student_id: 42,
                session_id: Some(8),
                question_id: Some(100),
                topic_id: Some(1),
                interaction_type: "compare_options".to_string(),
                prompt_text: Some("Why is option B best?".to_string()),
                response_text: None,
                context: json!({"source": "unit-test"}),
                was_helpful: None,
            })
            .expect("tutor response");

        assert_eq!(response.topic_id, Some(1));
        assert_eq!(response.question_id, Some(100));
        assert!(response.response_text.contains("(x + 2)(x + 3) = 0"));
        assert!(response.context_summary.contains("stored lesson"));
        assert!(
            response
                .context_summary
                .contains("1 prior tutor interaction")
        );
        assert!(!response.suggested_next_steps.is_empty());
        assert!(response.related_question_ids.contains(&100));
        assert!(response.related_question_ids.contains(&101));
        assert_eq!(response.related_entry_ids, vec![201, 202]);

        let logged_id = service
            .log_tutor_interaction(&TutorInteractionInput {
                student_id: 42,
                session_id: Some(8),
                question_id: Some(100),
                topic_id: Some(1),
                interaction_type: "compare_options".to_string(),
                prompt_text: Some("Why is option B best?".to_string()),
                response_text: Some(response.response_text.clone()),
                context: json!({"source": "unit-test", "generated": true}),
                was_helpful: Some(true),
            })
            .expect("log response");
        assert!(logged_id > 0);

        let interactions = service
            .list_recent_tutor_interactions(42, 2)
            .expect("recent interactions");
        assert_eq!(interactions.len(), 2);
        assert_eq!(
            interactions[0].response_text.as_deref(),
            Some(response.response_text.as_str())
        );
        assert_eq!(interactions[0].topic_id, Some(1));
        assert_eq!(interactions[0].question_id, Some(100));
        assert_eq!(interactions[0].context["generated"], json!(true));
        assert_eq!(
            interactions[1].response_text.as_deref(),
            Some("Use the factor pair that matches both the sum and product.")
        );
    }

    #[test]
    fn idea16_library_intelligence_supports_actions_search_notes_and_custom_shelves() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_library_intelligence_schema(&conn);

        conn.execute(
            "INSERT INTO subjects (id, name) VALUES (1, 'Mathematics')",
            [],
        )
        .expect("subject");
        conn.execute(
            "INSERT INTO topics (id, subject_id, name, exam_weight) VALUES (10, 1, 'Fractions', 8400)",
            [],
        )
        .expect("topic");
        conn.execute(
            "INSERT INTO question_families (id, family_name, subject_id, topic_id)
             VALUES (20, 'Fraction simplification', 1, 10)",
            [],
        )
        .expect("family");
        conn.execute(
            "INSERT INTO questions (
                id, subject_id, topic_id, family_id, stem, difficulty_level, source_type, is_active
             ) VALUES (
                100, 1, 10, 20, 'Simplify 6/9 to its lowest terms', 2400, 'past_question', 1
             )",
            [],
        )
        .expect("question");
        conn.execute(
            "INSERT INTO knowledge_entries (id, topic_id, title, difficulty_level, importance_score)
             VALUES (200, 10, 'Equivalent fractions rule', 1200, 8200)",
            [],
        )
        .expect("entry");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, gap_score, priority_score, trend_state, decay_risk
             ) VALUES (42, 10, 4200, 'partial', 7100, 9000, 'declining', 6800)",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO wrong_answer_diagnoses (
                id, student_id, question_id, topic_id, primary_diagnosis, recommended_action, severity
             ) VALUES (1, 42, 100, 10, 'factor confusion', 'Reduce with a common factor', 'high')",
            [],
        )
        .expect("diagnosis");
        conn.execute(
            "INSERT INTO shelf_generation_rules (id, shelf_type, display_name, description, priority_order)
             VALUES (1, 'due_now', 'Due Now', 'Immediate attention', 1)",
            [],
        )
        .expect("shelf rule");
        conn.execute(
            "INSERT INTO library_tag_definitions (id, tag_code, display_name, category, description)
             VALUES (1, 'exam_critical', 'Exam Critical', 'exam', 'High-value exam item')",
            [],
        )
        .expect("tag def");

        let service = LibraryService::new(&conn);
        let library_item_id = service
            .save_item_with_metadata(
                42,
                &SaveLibraryItemInput {
                    item_type: "question".to_string(),
                    item_ref_id: 100,
                    state: "saved".to_string(),
                    tags: vec!["keep_forgetting".to_string()],
                    note_text: Some("Always reduce first.".to_string()),
                    topic_id: Some(10),
                    urgency_score: 8700,
                    subject_id: Some(1),
                    subtopic_id: None,
                    difficulty_bp: Some(2400),
                    exam_frequency_bp: Some(7900),
                    source: Some("past_question".to_string()),
                    goal_id: None,
                    calendar_event_id: None,
                },
            )
            .expect("save rich library item");

        let action_id = service
            .record_item_action(
                42,
                library_item_id,
                &RecordLibraryItemActionInput {
                    action_type: "downloaded_offline".to_string(),
                    context: json!({"device": "tablet"}),
                },
            )
            .expect("download action");
        assert!(action_id > 0);
        service
            .record_item_action(
                42,
                library_item_id,
                &RecordLibraryItemActionInput {
                    action_type: "marked_exam_critical".to_string(),
                    context: json!({}),
                },
            )
            .expect("mark critical");
        service
            .record_item_action(
                42,
                library_item_id,
                &RecordLibraryItemActionInput {
                    action_type: "marked_weak".to_string(),
                    context: json!({}),
                },
            )
            .expect("mark weak");

        let note_id = service
            .add_library_note(&AddLibraryNoteInput {
                student_id: 42,
                library_item_id: Some(library_item_id),
                topic_id: Some(10),
                note_type: "trap_warning".to_string(),
                title: Some("Watch the common factor".to_string()),
                note_text: "Check both numerator and denominator for shared factors.".to_string(),
                context: json!({"source": "self"}),
            })
            .expect("note");
        assert!(note_id > 0);

        let search_results = service
            .search_library(
                42,
                &LibrarySearchInput {
                    query: Some("fractions".to_string()),
                    subject_id: Some(1),
                    topic_id: Some(10),
                    item_types: vec!["question".to_string(), "topic".to_string()],
                    states: Vec::new(),
                    tags: Vec::new(),
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
        assert!(
            search_results
                .iter()
                .any(|item| item.item_type == "question")
        );
        assert!(search_results.iter().any(|item| item.item_type == "topic"));

        let shelf_id = service
            .create_custom_shelf(
                42,
                &CreateCustomShelfInput {
                    title: "Last-Minute Prep".to_string(),
                    description: Some("Need this tonight".to_string()),
                    icon_hint: Some("flash".to_string()),
                },
            )
            .expect("custom shelf");
        service
            .add_item_to_custom_shelf(
                42,
                shelf_id,
                &AddShelfItemInput {
                    item_type: "question".to_string(),
                    item_ref_id: Some(100),
                    title: "Simplify 6/9 to its lowest terms".to_string(),
                    subtitle: Some("Fractions".to_string()),
                    reason: "Tonight's rescue item".to_string(),
                    rank_score: 8800,
                    metadata: json!({"library_item_id": library_item_id}),
                },
            )
            .expect("shelf item");

        let custom_shelves = service
            .list_custom_shelves(42, true, 10)
            .expect("custom shelves");
        assert_eq!(custom_shelves.len(), 1);
        assert_eq!(custom_shelves[0].items.len(), 1);

        let offline_items = service.list_offline_items(42, 10).expect("offline items");
        assert_eq!(offline_items.len(), 1);
        assert_eq!(offline_items[0].item_ref_id, 100);

        let state_history = service
            .list_item_state_history(library_item_id, 10)
            .expect("state history");
        assert!(!state_history.is_empty());
        assert_eq!(state_history[0].to_state, "weak");

        let actions = service
            .list_item_actions(library_item_id, 10)
            .expect("actions");
        assert!(
            actions
                .iter()
                .any(|item| item.action_type == "downloaded_offline")
        );

        let notes = service
            .list_library_notes(42, Some(10), Some(library_item_id), 10)
            .expect("notes");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note_type, "trap_warning");

        let tags = service
            .list_library_tag_definitions()
            .expect("tag definitions");
        assert_eq!(tags[0].tag_code, "exam_critical");
    }

    #[test]
    fn idea16_revision_templates_snapshot_and_generated_shelves_work() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_library_intelligence_schema(&conn);

        conn.execute(
            "INSERT INTO subjects (id, name) VALUES (1, 'Mathematics')",
            [],
        )
        .expect("subject");
        conn.execute(
            "INSERT INTO topics (id, subject_id, name, exam_weight) VALUES
                (10, 1, 'Fractions', 8400),
                (11, 1, 'Ratios', 7900)",
            [],
        )
        .expect("topics");
        conn.execute(
            "INSERT INTO question_families (id, family_name, subject_id, topic_id)
             VALUES (20, 'Fraction simplification', 1, 10)",
            [],
        )
        .expect("family");
        conn.execute(
            "INSERT INTO questions (
                id, subject_id, topic_id, family_id, stem, difficulty_level, source_type, is_active
             ) VALUES
                (100, 1, 10, 20, 'Simplify 8/12 to its lowest terms', 2000, 'past_question', 1),
                (101, 1, 10, 20, 'Find the fraction equivalent to 2/3', 2600, 'past_question', 1),
                (102, 1, 11, NULL, 'Share 24 in the ratio 1:3', 3400, 'authored', 1)",
            [],
        )
        .expect("questions");
        conn.execute(
            "INSERT INTO academic_nodes (
                id, topic_id, node_type, canonical_title, exam_relevance_score, foundation_weight, is_active
             ) VALUES (300, 10, 'formula', 'Equivalent fraction rule', 9000, 3000, 1)",
            [],
        )
        .expect("formula");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, gap_score, priority_score, trend_state, decay_risk, next_review_at
             ) VALUES
                (42, 10, 7100, 'fragile', 2500, 9200, 'fragile', 6200, '2026-04-01T08:00:00Z'),
                (42, 11, 3200, 'partial', 7000, 8500, 'declining', 5800, NULL)",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO memory_states (
                id, student_id, topic_id, node_id, memory_state, memory_strength, decay_risk, review_due_at
             ) VALUES (1, 42, 10, 300, 'fading', 2400, 7600, '2026-04-01T08:00:00Z')",
            [],
        )
        .expect("memory state");
        conn.execute(
            "INSERT INTO wrong_answer_diagnoses (
                id, student_id, question_id, topic_id, primary_diagnosis, recommended_action, severity
             ) VALUES (1, 42, 100, 10, 'factor confusion', 'Reduce with a common factor', 'high')",
            [],
        )
        .expect("diagnosis");
        conn.execute(
            "INSERT INTO family_recurrence_metrics (
                id, family_id, subject_id, recurrence_rate_bp, persistence_score_bp, current_relevance_bp, last_appearance_year, first_appearance_year
             ) VALUES (1, 20, 1, 8700, 8100, 9000, 2025, 2019)",
            [],
        )
        .expect("recurrence");
        conn.execute(
            "INSERT INTO student_family_performance (student_id, family_id, accuracy_rate_bp)
             VALUES (42, 20, 4300)",
            [],
        )
        .expect("family performance");
        conn.execute(
            "INSERT INTO node_edges (
                id, edge_type, from_node_type, from_node_id, to_node_type, to_node_id, strength_score
             ) VALUES (1, 'prerequisite', 'topic', 11, 'topic', 10, 8800)",
            [],
        )
        .expect("node edge");
        conn.execute(
            "INSERT INTO revision_pack_templates (
                id, template_code, display_name, description, pack_type, selection_strategy, default_item_count, difficulty_profile, topic_scope, time_estimate_minutes
             ) VALUES
                (1, 'weak_area', 'Weak Area Pack', 'Weak topics first', 'weak_area', 'weakness_weighted', 10, 'balanced', 'weak_topics', 30),
                (2, 'likely_exam', 'Likely Exam Questions', 'Exam intelligence weighted', 'likely_exam', 'recurrence_weighted', 10, 'balanced', 'high_frequency', 35)",
            [],
        )
        .expect("templates");
        conn.execute_batch(
            "
            INSERT INTO shelf_generation_rules (id, shelf_type, display_name, description, priority_order) VALUES
                (1, 'due_now', 'Due Now', 'Immediate attention', 1),
                (2, 'memory_shelf', 'Memory Shelf', 'Fading memory', 2),
                (3, 'mistake_bank', 'Mistake Bank', 'Recurring mistakes', 3),
                (4, 'weak_topics', 'My Weak Concepts', 'Topics needing repair', 4),
                (5, 'saved_questions', 'Saved Questions', 'Saved questions', 5),
                (6, 'exam_hotspots', 'Exam Hotspots', 'Frequently tested', 6),
                (7, 'near_wins', 'Almost There', 'Close to mastery', 7),
                (8, 'untouched_important', 'Untouched but Important', 'Important untouched items', 8),
                (9, 'things_i_forget', 'Things I Keep Forgetting', 'Recurring fades', 9),
                (10, 'teach_me_again', 'Teach Me Again', 'Needs another teaching pass', 10),
                (11, 'formula_bank', 'Formula Bank', 'High-value formulas', 11),
                (12, 'concept_chains', 'Concept Chains', 'Prerequisite links', 12);
            ",
        )
        .expect("shelf rules");

        let service = LibraryService::new(&conn);
        service
            .save_item_with_metadata(
                42,
                &SaveLibraryItemInput {
                    item_type: "question".to_string(),
                    item_ref_id: 100,
                    state: "saved".to_string(),
                    tags: vec!["keep_forgetting".to_string()],
                    note_text: None,
                    topic_id: Some(10),
                    urgency_score: 8900,
                    subject_id: Some(1),
                    subtopic_id: None,
                    difficulty_bp: Some(2000),
                    exam_frequency_bp: Some(9000),
                    source: Some("past_question".to_string()),
                    goal_id: None,
                    calendar_event_id: None,
                },
            )
            .expect("saved item");

        let generated_shelves = service
            .refresh_generated_shelves(42, 20)
            .expect("generated shelves");
        assert!(
            generated_shelves
                .iter()
                .any(|shelf| shelf.shelf_type == "exam_hotspots")
        );
        assert!(
            generated_shelves
                .iter()
                .any(|shelf| shelf.shelf_type == "formula_bank")
        );
        assert!(
            generated_shelves
                .iter()
                .any(|shelf| shelf.shelf_type == "teach_me_again")
        );

        let likely_pack = service
            .build_revision_pack_from_template(
                42,
                &BuildRevisionPackFromTemplateInput {
                    template_code: "likely_exam".to_string(),
                    title: Some("Likely Exam Sprint".to_string()),
                    item_limit: Some(5),
                    subject_id: Some(1),
                },
            )
            .expect("likely exam pack");
        assert_eq!(likely_pack.template_code.as_deref(), Some("likely_exam"));
        assert!(likely_pack.question_count > 0);

        let custom_pack = service
            .create_custom_revision_pack(42, "My Rescue Pack", &[100, 102], Some(1))
            .expect("custom pack");
        assert_eq!(custom_pack.source_type.as_deref(), Some("custom"));
        assert_eq!(custom_pack.question_count, 2);

        let snapshot = service
            .get_topic_library_snapshot(42, 10, 10)
            .expect("topic snapshot");
        assert_eq!(snapshot.topic_name, "Fractions");
        assert!(!snapshot.exam_hotspots.is_empty());
        assert!(
            snapshot
                .formula_titles
                .iter()
                .any(|title| title == "Equivalent fraction rule")
        );
        assert!(
            snapshot
                .recommended_actions
                .iter()
                .any(|action| action.contains("exam hotspots") || action.contains("Teach"))
        );

        let packs = service.list_revision_packs(42, 10).expect("revision packs");
        assert_eq!(packs.len(), 2);
    }
}
