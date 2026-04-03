use std::collections::HashSet;

use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    ConceptMapEdge, ConceptMapNode, ConceptMapView, ConceptMeta, ConfusionPairDetail,
    CreateGlossaryTestInput, DefinitionMeta, EntryAlias, EntryBundleReference, EntryContentBlock,
    EntryExample, EntryMisconception, FormulaLabView, FormulaMeta, GlossaryAudioProgram,
    GlossaryAudioQueueSnapshot, GlossaryComparisonView, GlossaryEntryDetail, GlossaryEntryFocus,
    GlossaryHomeSnapshot, GlossaryInteractionInput, GlossarySearchGroup, GlossarySearchInput,
    GlossarySearchResponse, GlossarySearchResult, GlossarySearchSuggestion,
    GlossaryTestAttemptResult, GlossaryTestItem, GlossaryTestSessionDetail, KnowledgeEntryProfile,
    KnowledgeRelationLink, LinkedQuestionSummary, NeighborIntruderMapping,
    StartGlossaryAudioQueueInput, StudentEntrySnapshot, SubmitGlossaryTestAttemptInput,
    UpdateGlossaryAudioQueueInput,
};

use super::{
    AudioProgramContext, GlossaryAudioSegment, GlossaryService, KnowledgeBundleSequenceItem,
    truncate_title,
};

impl<'a> GlossaryService<'a> {
    pub fn search_catalog(
        &self,
        input: &GlossarySearchInput,
        limit: usize,
    ) -> EcoachResult<GlossarySearchResponse> {
        let normalized_query = normalize_search_query(&input.query);
        let query_intent = detect_query_intent(&normalized_query);
        let effective_limit = limit.max(1);

        let mut entry_results =
            self.search_entry_results(input, &normalized_query, effective_limit)?;
        let bundle_results = if input.include_bundles {
            self.search_bundle_results(input, &normalized_query, effective_limit)?
        } else {
            Vec::new()
        };
        let question_results = if input.include_questions {
            self.search_question_results(input, &normalized_query, effective_limit)?
        } else {
            Vec::new()
        };
        let confusion_results = if input.include_confusions {
            self.search_confusion_results(&normalized_query, effective_limit)?
        } else {
            Vec::new()
        };

        entry_results.sort_by(|left, right| {
            right
                .match_score
                .cmp(&left.match_score)
                .then_with(|| left.title.cmp(&right.title))
        });

        let mut groups = Vec::new();
        let best_matches = entry_results
            .iter()
            .take(3)
            .cloned()
            .chain(bundle_results.iter().take(1).cloned())
            .chain(question_results.iter().take(1).cloned())
            .collect::<Vec<_>>();
        if !best_matches.is_empty() {
            groups.push(GlossarySearchGroup {
                group_key: "best_match".to_string(),
                title: "Best Match".to_string(),
                results: best_matches,
            });
        }

        let definitions = entry_results
            .iter()
            .filter(|item| item.entry_type.as_deref() == Some("definition"))
            .take(effective_limit)
            .cloned()
            .collect::<Vec<_>>();
        if !definitions.is_empty() {
            groups.push(GlossarySearchGroup {
                group_key: "definitions".to_string(),
                title: "Definitions".to_string(),
                results: definitions,
            });
        }

        let formulae = entry_results
            .iter()
            .filter(|item| item.entry_type.as_deref() == Some("formula"))
            .take(effective_limit)
            .cloned()
            .collect::<Vec<_>>();
        if !formulae.is_empty() {
            groups.push(GlossarySearchGroup {
                group_key: "formulae".to_string(),
                title: "Formulae".to_string(),
                results: formulae,
            });
        }

        let concepts = entry_results
            .iter()
            .filter(|item| {
                matches!(
                    item.entry_type.as_deref(),
                    Some("concept" | "key_concept" | "law" | "rule" | "theorem" | "method")
                )
            })
            .take(effective_limit)
            .cloned()
            .collect::<Vec<_>>();
        if !concepts.is_empty() {
            groups.push(GlossarySearchGroup {
                group_key: "concepts".to_string(),
                title: "Key Concepts".to_string(),
                results: concepts,
            });
        }

        if !bundle_results.is_empty() {
            groups.push(GlossarySearchGroup {
                group_key: "bundles".to_string(),
                title: "Learn Together Packs".to_string(),
                results: bundle_results.into_iter().take(effective_limit).collect(),
            });
        }

        if !confusion_results.is_empty() {
            groups.push(GlossarySearchGroup {
                group_key: "confusions".to_string(),
                title: "Commonly Confused".to_string(),
                results: confusion_results
                    .into_iter()
                    .take(effective_limit)
                    .collect(),
            });
        }

        if !question_results.is_empty() {
            groups.push(GlossarySearchGroup {
                group_key: "linked_questions".to_string(),
                title: "Linked Questions".to_string(),
                results: question_results.into_iter().take(effective_limit).collect(),
            });
        }

        Ok(GlossarySearchResponse {
            normalized_query,
            query_intent,
            groups,
        })
    }

    pub fn search_suggestions(
        &self,
        query: &str,
        limit: usize,
    ) -> EcoachResult<Vec<GlossarySearchSuggestion>> {
        let normalized_query = normalize_search_query(query);
        let mut suggestions = Vec::new();
        for result in self.search_entry_results(
            &GlossarySearchInput {
                query: query.to_string(),
                student_id: None,
                subject_id: None,
                topic_id: None,
                include_bundles: false,
                include_questions: false,
                include_confusions: false,
                include_audio_ready_only: false,
            },
            &normalized_query,
            limit,
        )? {
            suggestions.push(GlossarySearchSuggestion {
                suggestion: result.title,
                suggestion_type: result.entry_type.unwrap_or_else(|| "entry".to_string()),
                entry_id: result.entry_id,
                bundle_id: None,
                score: result.match_score,
            });
        }

        for result in self.search_bundle_results(
            &GlossarySearchInput {
                query: query.to_string(),
                student_id: None,
                subject_id: None,
                topic_id: None,
                include_bundles: true,
                include_questions: false,
                include_confusions: false,
                include_audio_ready_only: false,
            },
            &normalized_query,
            limit,
        )? {
            suggestions.push(GlossarySearchSuggestion {
                suggestion: result.title,
                suggestion_type: "bundle".to_string(),
                entry_id: None,
                bundle_id: result.bundle_id,
                score: result.match_score,
            });
        }

        suggestions.sort_by(|left, right| right.score.cmp(&left.score));
        suggestions.truncate(limit.max(1));
        Ok(suggestions)
    }

    pub fn search_voice(
        &self,
        query: &str,
        student_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<GlossarySearchResponse> {
        self.search_catalog(
            &GlossarySearchInput {
                query: query.to_string(),
                student_id,
                subject_id: None,
                topic_id: None,
                include_bundles: true,
                include_questions: true,
                include_confusions: true,
                include_audio_ready_only: false,
            },
            limit,
        )
    }

    pub fn start_audio_queue(
        &self,
        student_id: i64,
        input: &StartGlossaryAudioQueueInput,
    ) -> EcoachResult<GlossaryAudioQueueSnapshot> {
        let program = match input.source_type.as_str() {
            "topic" => self.build_personalized_audio_program_for_topic(
                student_id,
                input.source_id,
                input.limit.max(1),
            )?,
            "question" => self.build_personalized_audio_program_for_question(
                student_id,
                input.source_id,
                input.limit.max(1),
            )?,
            "bundle" => self.build_audio_program_for_bundle(
                student_id,
                input.source_id,
                input.limit.max(1),
            )?,
            "weakness" => self.build_weakness_audio_program(student_id, input.limit.max(1))?,
            "entry" => {
                self.build_audio_program_for_entry(student_id, input.source_id, input.limit.max(1))?
            }
            _ => {
                return Err(EcoachError::Validation(format!(
                    "unsupported glossary audio queue source {}",
                    input.source_type
                )));
            }
        };

        let program_id = self.persist_audio_program(
            student_id,
            &program,
            persisted_program_source(&input.source_type),
            input.source_id,
            input
                .teaching_mode
                .as_deref()
                .unwrap_or(&program.teaching_mode),
        )?;

        self.conn
            .execute(
                "INSERT INTO glossary_audio_queue_state (
                student_id,
                current_program_id,
                current_position,
                is_playing,
                playback_speed,
                include_examples,
                include_misconceptions,
                last_played_at,
                updated_at
             ) VALUES (?1, ?2, 0, 1, 1.0, ?3, ?4, datetime('now'), datetime('now'))
             ON CONFLICT(student_id) DO UPDATE SET
                current_program_id = excluded.current_program_id,
                current_position = excluded.current_position,
                is_playing = excluded.is_playing,
                playback_speed = excluded.playback_speed,
                include_examples = excluded.include_examples,
                include_misconceptions = excluded.include_misconceptions,
                last_played_at = excluded.last_played_at,
                updated_at = excluded.updated_at",
                params![
                    student_id,
                    program_id,
                    input.include_examples as i64,
                    input.include_misconceptions as i64,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.current_audio_queue(student_id)
    }

    pub fn next_audio_queue(&self, student_id: i64) -> EcoachResult<GlossaryAudioQueueSnapshot> {
        let snapshot = self.current_audio_queue(student_id)?;
        if let Some(program) = snapshot.program.as_ref() {
            let next_position = (snapshot.current_position + 1)
                .min(program.segments.len().saturating_sub(1) as i64);
            self.conn
                .execute(
                    "UPDATE glossary_audio_queue_state
                 SET current_position = ?2,
                     is_playing = 1,
                     last_played_at = datetime('now'),
                     updated_at = datetime('now')
                 WHERE student_id = ?1",
                    params![student_id, next_position],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        self.current_audio_queue(student_id)
    }

    pub fn previous_audio_queue(
        &self,
        student_id: i64,
    ) -> EcoachResult<GlossaryAudioQueueSnapshot> {
        let snapshot = self.current_audio_queue(student_id)?;
        let previous_position = snapshot.current_position.saturating_sub(1);
        self.conn
            .execute(
                "UPDATE glossary_audio_queue_state
             SET current_position = ?2,
                 updated_at = datetime('now')
             WHERE student_id = ?1",
                params![student_id, previous_position],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.current_audio_queue(student_id)
    }

    pub fn update_audio_queue(
        &self,
        student_id: i64,
        input: &UpdateGlossaryAudioQueueInput,
    ) -> EcoachResult<GlossaryAudioQueueSnapshot> {
        self.conn
            .execute(
                "UPDATE glossary_audio_queue_state
             SET playback_speed = COALESCE(?2, playback_speed),
                 include_examples = COALESCE(?3, include_examples),
                 include_misconceptions = COALESCE(?4, include_misconceptions),
                 is_playing = COALESCE(?5, is_playing),
                 updated_at = datetime('now')
             WHERE student_id = ?1",
                params![
                    student_id,
                    input.playback_speed,
                    input.include_examples.map(|value| value as i64),
                    input.include_misconceptions.map(|value| value as i64),
                    input.is_playing.map(|value| value as i64),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.current_audio_queue(student_id)
    }

    pub fn current_audio_queue(&self, student_id: i64) -> EcoachResult<GlossaryAudioQueueSnapshot> {
        let queue_state = self
            .conn
            .query_row(
                "SELECT
                    current_program_id,
                    current_position,
                    is_playing,
                    playback_speed,
                    include_examples,
                    include_misconceptions
                 FROM glossary_audio_queue_state
                 WHERE student_id = ?1",
                [student_id],
                |row| {
                    Ok((
                        row.get::<_, Option<i64>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)? == 1,
                        row.get::<_, f64>(3)?,
                        row.get::<_, i64>(4)? == 1,
                        row.get::<_, i64>(5)? == 1,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((None, 0, false, 1.0, true, true));

        let program = match queue_state.0 {
            Some(program_id) => Some(self.load_persisted_audio_program(program_id)?),
            None => None,
        };
        let current_segment = program
            .as_ref()
            .and_then(|program| program.segments.get(queue_state.1 as usize).cloned());

        Ok(GlossaryAudioQueueSnapshot {
            current_program_id: queue_state.0,
            current_position: queue_state.1,
            is_playing: queue_state.2,
            playback_speed: queue_state.3,
            include_examples: queue_state.4,
            include_misconceptions: queue_state.5,
            current_segment,
            program,
        })
    }

    pub fn create_glossary_test_session(
        &self,
        student_id: i64,
        input: &CreateGlossaryTestInput,
    ) -> EcoachResult<GlossaryTestSessionDetail> {
        let test_mode = normalize_test_mode(&input.test_mode);
        let entry_ids = self.select_test_entry_ids(student_id, input)?;
        self.conn
            .execute(
                "INSERT INTO glossary_test_sessions (
                student_id,
                test_mode,
                topic_id,
                bundle_id,
                entry_count,
                duration_seconds,
                difficulty_level,
                completion_rate_bp
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
                params![
                    student_id,
                    test_mode,
                    input.topic_id,
                    input.bundle_id,
                    entry_ids.len() as i64,
                    input.duration_seconds,
                    input.difficulty_level.unwrap_or(5000),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let session_id = self.conn.last_insert_rowid();

        for (index, entry_id) in entry_ids.into_iter().enumerate() {
            let item = self.build_test_item(entry_id, &test_mode)?;
            self.conn
                .execute(
                    "INSERT INTO glossary_test_items (
                    test_session_id,
                    sequence_no,
                    entry_id,
                    prompt_type,
                    prompt_text,
                    expected_answer,
                    options_json,
                    metadata_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        session_id,
                        (index + 1) as i64,
                        item.entry_id,
                        item.prompt_type,
                        item.prompt_text,
                        item.expected_answer,
                        serde_json::to_string(&item.options).unwrap_or_else(|_| "[]".to_string()),
                        item.metadata.to_string(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.get_glossary_test_session(session_id)
    }

    pub fn get_glossary_test_session(
        &self,
        session_id: i64,
    ) -> EcoachResult<GlossaryTestSessionDetail> {
        let header = self
            .conn
            .query_row(
                "SELECT
                    id,
                    student_id,
                    test_mode,
                    topic_id,
                    bundle_id,
                    entry_count,
                    duration_seconds,
                    difficulty_level,
                    recall_score_bp,
                    recognition_score_bp,
                    connection_score_bp,
                    application_score_bp,
                    retention_score_bp,
                    confidence_score_bp,
                    completion_rate_bp
                 FROM glossary_test_sessions
                 WHERE id = ?1",
                [session_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<i64>>(3)?,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, Option<i64>>(6)?,
                        row.get::<_, i64>(7)?,
                        row.get::<_, Option<i64>>(8)?,
                        row.get::<_, Option<i64>>(9)?,
                        row.get::<_, Option<i64>>(10)?,
                        row.get::<_, Option<i64>>(11)?,
                        row.get::<_, Option<i64>>(12)?,
                        row.get::<_, Option<i64>>(13)?,
                        row.get::<_, i64>(14)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut stmt = self
            .conn
            .prepare(
                "SELECT sequence_no, entry_id, prompt_type, prompt_text, expected_answer, options_json, metadata_json
                 FROM glossary_test_items
                 WHERE test_session_id = ?1
                 ORDER BY sequence_no ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let items = collect_rows(stmt.query_map([session_id], |row| {
            Ok(GlossaryTestItem {
                sequence_no: row.get(0)?,
                entry_id: row.get(1)?,
                prompt_type: row.get(2)?,
                prompt_text: row.get(3)?,
                expected_answer: row.get(4)?,
                options: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                metadata: parse_json_value(&row.get::<_, String>(6)?),
            })
        }))?;

        Ok(GlossaryTestSessionDetail {
            session_id: header.0,
            student_id: header.1,
            test_mode: header.2,
            topic_id: header.3,
            bundle_id: header.4,
            entry_count: header.5,
            duration_seconds: header.6,
            difficulty_level: header.7,
            recall_score_bp: header.8,
            recognition_score_bp: header.9,
            connection_score_bp: header.10,
            application_score_bp: header.11,
            retention_score_bp: header.12,
            confidence_score_bp: header.13,
            completion_rate_bp: header.14,
            items,
        })
    }

    pub fn submit_glossary_test_attempt(
        &self,
        student_id: i64,
        session_id: i64,
        input: &SubmitGlossaryTestAttemptInput,
    ) -> EcoachResult<GlossaryTestAttemptResult> {
        let session = self.get_glossary_test_session(session_id)?;
        let item = session
            .items
            .iter()
            .find(|item| item.entry_id == input.entry_id)
            .cloned()
            .ok_or_else(|| {
                EcoachError::Validation(format!(
                    "entry {} is not part of glossary test session {}",
                    input.entry_id, session_id
                ))
            })?;

        let acceptable_answers = self.acceptable_answers_for_item(&item)?;
        let normalized_response = normalize_answer(&input.student_response);
        let is_correct = acceptable_answers
            .iter()
            .any(|answer| normalize_answer(answer) == normalized_response);
        let attempt_no = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(attempt_no), 0) + 1
                 FROM glossary_test_attempts
                 WHERE test_session_id = ?1 AND entry_id = ?2",
                params![session_id, input.entry_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let (
            meaning_recall_bp,
            word_recognition_bp,
            formula_recall_bp,
            concept_recognition_bp,
            relationship_understanding_bp,
            confusion_resistance_bp,
            context_transfer_bp,
        ) = test_metric_scores(&session.test_mode, is_correct);

        self.conn
            .execute(
                "INSERT INTO glossary_test_attempts (
                test_session_id,
                entry_id,
                attempt_no,
                test_mode,
                student_response,
                is_correct,
                time_seconds,
                meaning_recall_bp,
                word_recognition_bp,
                spelling_accuracy_bp,
                formula_recall_bp,
                concept_recognition_bp,
                relationship_understanding_bp,
                confusion_resistance_bp,
                context_transfer_bp
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    session_id,
                    input.entry_id,
                    attempt_no,
                    session.test_mode,
                    input.student_response,
                    is_correct as i64,
                    input.time_seconds,
                    meaning_recall_bp,
                    word_recognition_bp,
                    if is_correct { 10000 } else { 0 },
                    formula_recall_bp,
                    concept_recognition_bp,
                    relationship_understanding_bp,
                    confusion_resistance_bp,
                    context_transfer_bp,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.ensure_student_entry_state(student_id, input.entry_id)?;
        self.apply_test_outcome(student_id, input.entry_id, is_correct, &session.test_mode)?;
        self.refresh_glossary_test_session_scores(session_id)?;

        let student_state = self
            .load_student_entry_snapshot(student_id, input.entry_id)?
            .ok_or_else(|| {
                EcoachError::Validation(format!(
                    "student state missing after recording glossary attempt for entry {}",
                    input.entry_id
                ))
            })?;

        Ok(GlossaryTestAttemptResult {
            is_correct,
            feedback: if is_correct {
                "Correct. Keep the meaning and the signal together.".to_string()
            } else {
                format!(
                    "Not quite. Expected {}.",
                    acceptable_answers
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "the linked glossary answer".to_string())
                )
            },
            updated_mastery_state: student_state.mastery_state.clone(),
            review_due_at: student_state.review_due_at.clone(),
            mastery_score: student_state.mastery_score,
            recognition_score: student_state.recognition_score,
            connection_score: student_state.connection_score,
            application_score: student_state.application_score,
            retention_score: student_state.retention_score,
        })
    }

    pub fn get_entry_detail(
        &self,
        student_id: Option<i64>,
        entry_id: i64,
        relation_limit: usize,
        bundle_limit: usize,
    ) -> EcoachResult<GlossaryEntryDetail> {
        let entry = self.load_entry_profile(entry_id)?;
        Ok(GlossaryEntryDetail {
            aliases: self.load_entry_aliases(entry_id)?,
            content_blocks: self.load_entry_content_blocks(entry_id)?,
            definition_meta: self.load_definition_meta(entry_id)?,
            formula_meta: self.load_formula_meta(entry_id)?,
            concept_meta: self.load_concept_meta(entry_id)?,
            examples: self.load_entry_examples(entry_id)?,
            misconceptions: self.load_entry_misconceptions(entry_id)?,
            relations: self.load_entry_relations(entry_id, relation_limit)?,
            bundles: self.load_entry_bundles(entry_id, bundle_limit)?,
            linked_questions: self.load_linked_questions(entry_id, bundle_limit)?,
            audio_segments: self.load_entry_audio_segments(entry_id, None)?,
            student_state: match student_id {
                Some(student_id) => self.load_student_entry_snapshot(student_id, entry_id)?,
                None => None,
            },
            confusion_pairs: self.load_confusion_pairs(entry_id, relation_limit)?,
            neighbor_intruder: self.load_neighbor_intruder(entry_id)?,
            entry,
        })
    }

    pub fn record_interaction(&self, input: &GlossaryInteractionInput) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO glossary_interaction_events (
                student_id, entry_id, bundle_id, question_id, event_type, query_text, metadata_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    input.student_id,
                    input.entry_id,
                    input.bundle_id,
                    input.question_id,
                    input.event_type,
                    input.query_text,
                    input.metadata.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let event_id = self.conn.last_insert_rowid();

        if let (Some(student_id), Some(entry_id)) = (input.student_id, input.entry_id) {
            self.ensure_student_entry_state(student_id, entry_id)?;
            self.apply_interaction_effects(student_id, entry_id, &input.event_type)?;
        }

        Ok(event_id)
    }

    pub fn rebuild_search_index(&self) -> EcoachResult<usize> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM knowledge_entries")
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut count = 0usize;
        for row in rows {
            self.refresh_search_index_for_entry(
                row.map_err(|err| EcoachError::Storage(err.to_string()))?,
            )?;
            count += 1;
        }
        Ok(count)
    }

    pub fn build_home_snapshot(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<GlossaryHomeSnapshot> {
        let effective_limit = limit.max(1) as i64;
        let discover = self.load_focus_entries(
            student_id,
            subject_id,
            effective_limit,
            "COALESCE(ke.priority_score, 0) DESC, COALESCE(ke.importance_score, 0) DESC, ke.title ASC",
            "high priority glossary item".to_string(),
        )?;
        let weak_entries = self.load_focus_entries(
            student_id,
            subject_id,
            effective_limit,
            "(COALESCE(ses.confusion_score, 0) + (10000 - COALESCE(ses.recall_strength, 0)) + COALESCE(ke.priority_score, 0)) DESC, ke.title ASC",
            "weak concept signal".to_string(),
        )?;
        let exam_hotspots = self.load_focus_entries(
            student_id,
            subject_id,
            effective_limit,
            "(COALESCE(ke.exam_relevance_score, 0) + COALESCE(ke.priority_score, 0) + COALESCE(ses.confusion_score, 0)) DESC, ke.title ASC",
            "exam hotspot".to_string(),
        )?;
        let recommended_bundles =
            if let Some(topic_id) = weak_entries.iter().find_map(|entry| entry.topic_id) {
                self.load_bundle_sequence_for_topic(student_id, topic_id, limit)?
            } else {
                Vec::new()
            };

        Ok(GlossaryHomeSnapshot {
            discover,
            weak_entries,
            exam_hotspots,
            recommended_bundles,
            audio_station_labels: vec![
                "Topic Radio".to_string(),
                "Formula Radio".to_string(),
                "Definition Radio".to_string(),
                "Weakness Radio".to_string(),
                "Exam Hotspot Radio".to_string(),
            ],
        })
    }

    pub fn build_compare_view(
        &self,
        left_entry_id: i64,
        right_entry_id: i64,
    ) -> EcoachResult<GlossaryComparisonView> {
        let left_entry = self.load_entry_profile(left_entry_id)?;
        let right_entry = self.load_entry_profile(right_entry_id)?;
        let left_relations = self.load_entry_relations(left_entry_id, 32)?;
        let right_relations = self.load_entry_relations(right_entry_id, 32)?;
        let left_relation_types = left_relations
            .iter()
            .map(|relation| relation.relation_type.clone())
            .collect::<HashSet<_>>();
        let shared_relation_types = right_relations
            .iter()
            .filter(|relation| left_relation_types.contains(&relation.relation_type))
            .map(|relation| relation.relation_type.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let confusion_pair = self
            .load_confusion_pair_between(left_entry_id, right_entry_id)?
            .or_else(|| {
                self.load_legacy_contrast_pair(left_entry_id, right_entry_id)
                    .ok()
                    .flatten()
            });

        Ok(GlossaryComparisonView {
            left_entry,
            right_entry,
            shared_relation_types,
            distinction_explanation: confusion_pair
                .as_ref()
                .map(|pair| pair.distinction_explanation.clone()),
            clue_to_distinguish: confusion_pair.and_then(|pair| pair.clue_to_distinguish),
            shared_bundles: self.load_shared_bundles(left_entry_id, right_entry_id)?,
            linked_question_ids: self.load_shared_question_ids(left_entry_id, right_entry_id)?,
        })
    }

    pub fn get_formula_lab(&self, entry_id: i64) -> EcoachResult<FormulaLabView> {
        let detail = self.get_entry_detail(None, entry_id, 12, 6)?;
        let formula_meta = detail.formula_meta.clone().ok_or_else(|| {
            EcoachError::Validation(format!("entry {} does not have formula metadata", entry_id))
        })?;
        Ok(FormulaLabView {
            entry: detail.entry,
            formula_meta,
            related_entries: detail.relations,
            examples: detail.examples,
            misconceptions: detail.misconceptions,
            linked_questions: detail.linked_questions,
        })
    }

    pub fn build_concept_map(
        &self,
        entry_id: i64,
        depth: usize,
        limit: usize,
    ) -> EcoachResult<ConceptMapView> {
        let mut visited = HashSet::from([entry_id]);
        let mut frontier = vec![entry_id];
        let mut nodes = vec![self.load_concept_map_node(entry_id)?];
        let mut edges = Vec::new();

        for _ in 0..depth.max(1) {
            if frontier.is_empty() || nodes.len() >= limit.max(1) {
                break;
            }
            let current_frontier = frontier.clone();
            frontier.clear();
            for current_id in current_frontier {
                for relation in self.load_entry_relations(current_id, limit.max(1))? {
                    edges.push(ConceptMapEdge {
                        from_entry_id: relation.from_entry_id,
                        to_entry_id: relation.to_entry_id,
                        relation_type: relation.relation_type.clone(),
                        strength_score: relation.strength_score,
                        explanation: relation.explanation.clone(),
                    });
                    let neighbor_id = if relation.from_entry_id == current_id {
                        relation.to_entry_id
                    } else {
                        relation.from_entry_id
                    };
                    if visited.insert(neighbor_id) {
                        nodes.push(self.load_concept_map_node(neighbor_id)?);
                        frontier.push(neighbor_id);
                    }
                    if nodes.len() >= limit.max(1) {
                        break;
                    }
                }
                if nodes.len() >= limit.max(1) {
                    break;
                }
            }
        }

        Ok(ConceptMapView {
            root_entry_id: entry_id,
            nodes,
            edges,
        })
    }

    fn load_entry_profile(&self, entry_id: i64) -> EcoachResult<KnowledgeEntryProfile> {
        self.conn
            .query_row(
                "SELECT
                    id,
                    subject_id,
                    topic_id,
                    subtopic_id,
                    entry_type,
                    title,
                    canonical_name,
                    slug,
                    short_text,
                    full_text,
                    simple_text,
                    technical_text,
                    exam_text,
                    importance_score,
                    difficulty_level,
                    grade_band,
                    status,
                    COALESCE(audio_available, 0),
                    COALESCE(has_formula, 0),
                    COALESCE(confusion_pair_count, 0),
                    COALESCE(example_count, 0),
                    COALESCE(misconception_count, 0),
                    COALESCE(exam_relevance_score, 5000),
                    COALESCE(priority_score, 5000),
                    phonetic_text,
                    created_at,
                    updated_at
                 FROM knowledge_entries
                 WHERE id = ?1",
                [entry_id],
                |row| {
                    Ok(KnowledgeEntryProfile {
                        id: row.get(0)?,
                        subject_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        subtopic_id: row.get(3)?,
                        entry_type: row.get(4)?,
                        title: row.get(5)?,
                        canonical_name: row.get(6)?,
                        slug: row.get(7)?,
                        short_text: row.get(8)?,
                        full_text: row.get(9)?,
                        simple_text: row.get(10)?,
                        technical_text: row.get(11)?,
                        exam_text: row.get(12)?,
                        importance_score: row.get(13)?,
                        difficulty_level: row.get(14)?,
                        grade_band: row.get(15)?,
                        status: row.get(16)?,
                        audio_available: row.get::<_, i64>(17)? == 1,
                        has_formula: row.get::<_, i64>(18)? == 1,
                        confusion_pair_count: row.get(19)?,
                        example_count: row.get(20)?,
                        misconception_count: row.get(21)?,
                        exam_relevance_score: row.get(22)?,
                        priority_score: row.get(23)?,
                        phonetic_text: row.get(24)?,
                        created_at: row.get(25)?,
                        updated_at: row.get(26)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_entry_aliases(&self, entry_id: i64) -> EcoachResult<Vec<EntryAlias>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT alias_text, COALESCE(alias_type, 'synonym')
                 FROM entry_aliases
                 WHERE entry_id = ?1
                 ORDER BY alias_type ASC, alias_text ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(stmt.query_map([entry_id], |row| {
            Ok(EntryAlias {
                alias_text: row.get(0)?,
                alias_type: row.get(1)?,
            })
        }))
    }

    fn load_entry_content_blocks(&self, entry_id: i64) -> EcoachResult<Vec<EntryContentBlock>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, block_type, order_index, content_json
                 FROM entry_content_blocks
                 WHERE entry_id = ?1
                 ORDER BY order_index ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(stmt.query_map([entry_id], |row| {
            let content_json: String = row.get(3)?;
            Ok(EntryContentBlock {
                id: row.get(0)?,
                block_type: row.get(1)?,
                order_index: row.get(2)?,
                content: parse_json_value(&content_json),
            })
        }))
    }

    fn load_definition_meta(&self, entry_id: i64) -> EcoachResult<Option<DefinitionMeta>> {
        self.conn
            .query_row(
                "SELECT
                    definition_text,
                    short_definition,
                    formal_definition,
                    COALESCE(formal_definition, definition_text),
                    real_world_meaning,
                    non_examples,
                    context_clues,
                    pronunciation_text
                 FROM definition_meta
                 WHERE entry_id = ?1",
                [entry_id],
                |row| {
                    Ok(DefinitionMeta {
                        definition_text: row.get(0)?,
                        short_definition: row.get(1)?,
                        formal_definition: row.get(2)?,
                        plain_english_definition: row.get(3)?,
                        real_world_meaning: row.get(4)?,
                        non_examples: row.get(5)?,
                        context_clues: row.get(6)?,
                        pronunciation_text: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_formula_meta(&self, entry_id: i64) -> EcoachResult<Option<FormulaMeta>> {
        self.conn
            .query_row(
                "SELECT
                    formula_expression,
                    formula_speech,
                    formula_latex,
                    variables_json,
                    units_json,
                    when_to_use,
                    when_not_to_use,
                    rearrangements_json,
                    assumptions_json,
                    common_errors_json,
                    derivation_summary,
                    worked_example_ids_json
                 FROM formula_meta
                 WHERE entry_id = ?1",
                [entry_id],
                |row| {
                    let formula_expression: String = row.get(0)?;
                    let formula_speech: Option<String> = row.get(1)?;
                    Ok(FormulaMeta {
                        formula_expression: formula_expression.clone(),
                        formula_speech: Some(
                            formula_speech
                                .unwrap_or_else(|| render_formula_speech(&formula_expression)),
                        ),
                        formula_latex: row.get(2)?,
                        variables: parse_json_value(&row.get::<_, String>(3)?),
                        units: row
                            .get::<_, Option<String>>(4)?
                            .map(|text| parse_json_value(&text)),
                        when_to_use: row.get(5)?,
                        when_not_to_use: row.get(6)?,
                        rearrangements: parse_string_array(row.get::<_, Option<String>>(7)?),
                        assumptions: parse_string_array(row.get::<_, Option<String>>(8)?),
                        common_errors: parse_string_array(row.get::<_, Option<String>>(9)?),
                        derivation_summary: row.get(10)?,
                        worked_example_ids: parse_i64_array(row.get::<_, Option<String>>(11)?),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_concept_meta(&self, entry_id: i64) -> EcoachResult<Option<ConceptMeta>> {
        self.conn
            .query_row(
                "SELECT
                    concept_explanation,
                    intuition_summary,
                    related_visual_keywords,
                    misconception_signals,
                    why_it_matters,
                    mastery_indicators_json
                 FROM concept_meta
                 WHERE entry_id = ?1",
                [entry_id],
                |row| {
                    Ok(ConceptMeta {
                        concept_explanation: row.get(0)?,
                        intuition_summary: row.get(1)?,
                        related_visual_keywords: split_loose_terms(
                            &row.get::<_, Option<String>>(2)?,
                        ),
                        misconception_signals: split_loose_terms(&row.get::<_, Option<String>>(3)?),
                        why_it_matters: row.get(4)?,
                        mastery_indicators: parse_string_array(row.get::<_, Option<String>>(5)?),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_entry_examples(&self, entry_id: i64) -> EcoachResult<Vec<EntryExample>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    id,
                    sequence_order,
                    example_text,
                    COALESCE(context_type, 'general'),
                    difficulty_level,
                    worked_solution_text,
                    is_exam_style
                 FROM entry_examples
                 WHERE entry_id = ?1
                 ORDER BY sequence_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(stmt.query_map([entry_id], |row| {
            Ok(EntryExample {
                id: row.get(0)?,
                sequence_order: row.get(1)?,
                example_text: row.get(2)?,
                context_type: row.get(3)?,
                difficulty_level: row.get(4)?,
                worked_solution_text: row.get(5)?,
                is_exam_style: row.get::<_, i64>(6)? == 1,
            })
        }))
    }

    fn load_entry_misconceptions(&self, entry_id: i64) -> EcoachResult<Vec<EntryMisconception>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    id,
                    misconception_text,
                    cause_explanation,
                    correction_explanation,
                    confusion_pair_entry_id,
                    misconception_source,
                    severity_bp
                 FROM entry_misconceptions
                 WHERE entry_id = ?1
                 ORDER BY severity_bp DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(stmt.query_map([entry_id], |row| {
            Ok(EntryMisconception {
                id: row.get(0)?,
                misconception_text: row.get(1)?,
                cause_explanation: row.get(2)?,
                correction_explanation: row.get(3)?,
                confusion_pair_entry_id: row.get(4)?,
                misconception_source: row.get(5)?,
                severity_bp: row.get(6)?,
            })
        }))
    }

    fn load_entry_relations(
        &self,
        entry_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<KnowledgeRelationLink>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    kr.id,
                    kr.relation_type,
                    kr.strength_score,
                    kr.explanation,
                    kr.from_entry_id,
                    kr.to_entry_id,
                    related_ke.title,
                    related_ke.entry_type,
                    related_ke.topic_id
                 FROM knowledge_relations kr
                 INNER JOIN knowledge_entries related_ke
                    ON related_ke.id = CASE
                        WHEN kr.from_entry_id = ?1 THEN kr.to_entry_id
                        ELSE kr.from_entry_id
                    END
                 WHERE kr.from_entry_id = ?1 OR kr.to_entry_id = ?1
                 ORDER BY kr.strength_score DESC, kr.id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(
            stmt.query_map(params![entry_id, limit.max(1) as i64], |row| {
                Ok(KnowledgeRelationLink {
                    relation_id: row.get(0)?,
                    relation_type: row.get(1)?,
                    strength_score: row.get(2)?,
                    explanation: row.get(3)?,
                    from_entry_id: row.get(4)?,
                    to_entry_id: row.get(5)?,
                    related_entry_title: row.get(6)?,
                    related_entry_type: row.get(7)?,
                    related_topic_id: row.get(8)?,
                })
            }),
        )
    }

    fn load_entry_bundles(
        &self,
        entry_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<EntryBundleReference>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    kb.id,
                    kb.title,
                    kb.bundle_type,
                    kb.description,
                    kbi.item_role,
                    kbi.sequence_order,
                    COALESCE(kbi.required, 1),
                    kb.difficulty_level,
                    kb.exam_relevance_score
                 FROM knowledge_bundle_items kbi
                 INNER JOIN knowledge_bundles kb ON kb.id = kbi.bundle_id
                 WHERE kbi.entry_id = ?1
                 ORDER BY kb.exam_relevance_score DESC, kbi.sequence_order ASC, kb.title ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(
            stmt.query_map(params![entry_id, limit.max(1) as i64], |row| {
                Ok(EntryBundleReference {
                    bundle_id: row.get(0)?,
                    title: row.get(1)?,
                    bundle_type: row.get(2)?,
                    description: row.get(3)?,
                    item_role: row.get(4)?,
                    sequence_order: row.get(5)?,
                    required: row.get::<_, i64>(6)? == 1,
                    difficulty_level: row.get(7)?,
                    exam_relevance_score: row.get(8)?,
                })
            }),
        )
    }

    fn load_linked_questions(
        &self,
        entry_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<LinkedQuestionSummary>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    qgl.question_id,
                    qgl.relation_type,
                    qgl.confidence_score,
                    qgl.is_primary,
                    qgl.link_source,
                    qgl.link_reason,
                    q.stem
                 FROM question_glossary_links qgl
                 LEFT JOIN questions q ON q.id = qgl.question_id
                 WHERE qgl.entry_id = ?1
                 ORDER BY qgl.is_primary DESC, qgl.confidence_score DESC, qgl.question_id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(
            stmt.query_map(params![entry_id, limit.max(1) as i64], |row| {
                Ok(LinkedQuestionSummary {
                    question_id: row.get(0)?,
                    relation_type: row.get(1)?,
                    confidence_score: row.get(2)?,
                    is_primary: row.get::<_, i64>(3)? == 1,
                    link_source: row.get(4)?,
                    link_reason: row.get(5)?,
                    stem: row.get(6)?,
                })
            }),
        )
    }

    fn load_entry_audio_segments(
        &self,
        entry_id: i64,
        teaching_mode: Option<&str>,
    ) -> EcoachResult<Vec<GlossaryAudioSegment>> {
        let sql = if teaching_mode.is_some() {
            "SELECT
                ROW_NUMBER() OVER (ORDER BY id ASC),
                segment_type,
                segment_type,
                script_text,
                entry_id,
                NULL,
                NULL,
                COALESCE(duration_seconds, 0)
             FROM entry_audio_segments
             WHERE entry_id = ?1 AND teaching_mode = ?2
             ORDER BY id ASC"
        } else {
            "SELECT
                ROW_NUMBER() OVER (ORDER BY id ASC),
                segment_type,
                segment_type,
                script_text,
                entry_id,
                NULL,
                NULL,
                COALESCE(duration_seconds, 0)
             FROM entry_audio_segments
             WHERE entry_id = ?1
             ORDER BY id ASC"
        };
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if let Some(mode) = teaching_mode {
            collect_rows(stmt.query_map(params![entry_id, mode], |row| {
                Ok(GlossaryAudioSegment {
                    sequence_no: row.get(0)?,
                    segment_type: row.get(1)?,
                    title: title_case(&row.get::<_, String>(2)?),
                    script_text: row.get(3)?,
                    entry_id: row.get(4)?,
                    prompt_text: row.get(5)?,
                    focus_reason: row.get(6)?,
                    duration_seconds: row.get(7)?,
                })
            }))
        } else {
            collect_rows(stmt.query_map(params![entry_id], |row| {
                Ok(GlossaryAudioSegment {
                    sequence_no: row.get(0)?,
                    segment_type: row.get(1)?,
                    title: title_case(&row.get::<_, String>(2)?),
                    script_text: row.get(3)?,
                    entry_id: row.get(4)?,
                    prompt_text: row.get(5)?,
                    focus_reason: row.get(6)?,
                    duration_seconds: row.get(7)?,
                })
            }))
        }
    }

    fn load_student_entry_snapshot(
        &self,
        student_id: i64,
        entry_id: i64,
    ) -> EcoachResult<Option<StudentEntrySnapshot>> {
        self.conn
            .query_row(
                "SELECT
                    ses.user_id,
                    ses.familiarity_state,
                    ses.mastery_score,
                    ses.confusion_score,
                    ses.recall_strength,
                    ses.open_count,
                    ses.linked_wrong_answer_count,
                    COALESCE(ses.recognition_score, 0),
                    COALESCE(ses.connection_score, 0),
                    COALESCE(ses.application_score, 0),
                    COALESCE(ses.retention_score, 0),
                    COALESCE(ses.test_count, 0),
                    COALESCE(ses.test_pass_count, 0),
                    ses.last_viewed_at,
                    ses.last_played_at,
                    ses.last_tested_at,
                    ses.review_due_at,
                    ses.spaced_review_due_at,
                    ses.at_risk_threshold_date,
                    gem.mastery_state,
                    COALESCE(gem.at_risk_flag, 0)
                 FROM student_entry_state ses
                 LEFT JOIN glossary_entry_mastery gem
                    ON gem.student_id = ses.user_id AND gem.entry_id = ses.entry_id
                 WHERE ses.user_id = ?1 AND ses.entry_id = ?2",
                params![student_id, entry_id],
                |row| {
                    Ok(StudentEntrySnapshot {
                        user_id: row.get(0)?,
                        familiarity_state: row.get(1)?,
                        mastery_score: row.get(2)?,
                        confusion_score: row.get(3)?,
                        recall_strength: row.get(4)?,
                        open_count: row.get(5)?,
                        linked_wrong_answer_count: row.get(6)?,
                        recognition_score: row.get(7)?,
                        connection_score: row.get(8)?,
                        application_score: row.get(9)?,
                        retention_score: row.get(10)?,
                        test_count: row.get(11)?,
                        test_pass_count: row.get(12)?,
                        last_viewed_at: row.get(13)?,
                        last_played_at: row.get(14)?,
                        last_tested_at: row.get(15)?,
                        review_due_at: row.get(16)?,
                        spaced_review_due_at: row.get(17)?,
                        at_risk_threshold_date: row.get(18)?,
                        mastery_state: row.get(19)?,
                        at_risk_flag: row.get::<_, i64>(20)? == 1,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_confusion_pairs(
        &self,
        entry_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<ConfusionPairDetail>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    CASE WHEN cp.entry_id_1 = ?1 THEN cp.entry_id_2 ELSE cp.entry_id_1 END,
                    related_ke.title,
                    cp.distinction_explanation,
                    cp.common_confusion_reason,
                    cp.clue_to_distinguish,
                    cp.example_sentence_1,
                    cp.example_sentence_2,
                    cp.confusion_frequency_bp
                 FROM confusion_pairs cp
                 INNER JOIN knowledge_entries related_ke
                    ON related_ke.id = CASE
                        WHEN cp.entry_id_1 = ?1 THEN cp.entry_id_2
                        ELSE cp.entry_id_1
                    END
                 WHERE cp.entry_id_1 = ?1 OR cp.entry_id_2 = ?1
                 ORDER BY cp.confusion_frequency_bp DESC, cp.id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let results = collect_rows(
            stmt.query_map(params![entry_id, limit.max(1) as i64], |row| {
                Ok(ConfusionPairDetail {
                    paired_entry_id: row.get(0)?,
                    paired_entry_title: row.get(1)?,
                    distinction_explanation: row.get(2)?,
                    common_confusion_reason: row.get(3)?,
                    clue_to_distinguish: row.get(4)?,
                    example_sentence_1: row.get(5)?,
                    example_sentence_2: row.get(6)?,
                    confusion_frequency_bp: row.get(7)?,
                })
            }),
        );
        match results {
            Ok(items) if !items.is_empty() => Ok(items),
            Ok(_) | Err(_) => self.load_legacy_contrast_pairs(entry_id, limit),
        }
    }

    fn load_neighbor_intruder(
        &self,
        entry_id: i64,
    ) -> EcoachResult<Option<NeighborIntruderMapping>> {
        self.conn
            .query_row(
                "SELECT neighbor_entry_ids_json, intruder_entry_ids_json
                 FROM neighbor_intruder_mappings
                 WHERE entry_id = ?1",
                [entry_id],
                |row| {
                    Ok(NeighborIntruderMapping {
                        neighbors: parse_i64_array(Some(row.get::<_, String>(0)?)),
                        intruders: parse_i64_array(Some(row.get::<_, String>(1)?)),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_confusion_pair_between(
        &self,
        left_entry_id: i64,
        right_entry_id: i64,
    ) -> EcoachResult<Option<ConfusionPairDetail>> {
        self.conn
            .query_row(
                "SELECT
                    CASE WHEN cp.entry_id_1 = ?1 THEN cp.entry_id_2 ELSE cp.entry_id_1 END,
                    CASE WHEN cp.entry_id_1 = ?1 THEN e2.title ELSE e1.title END,
                    cp.distinction_explanation,
                    cp.common_confusion_reason,
                    cp.clue_to_distinguish,
                    cp.example_sentence_1,
                    cp.example_sentence_2,
                    cp.confusion_frequency_bp
                 FROM confusion_pairs cp
                 INNER JOIN knowledge_entries e1 ON e1.id = cp.entry_id_1
                 INNER JOIN knowledge_entries e2 ON e2.id = cp.entry_id_2
                 WHERE (cp.entry_id_1 = ?1 AND cp.entry_id_2 = ?2)
                    OR (cp.entry_id_1 = ?2 AND cp.entry_id_2 = ?1)
                 LIMIT 1",
                params![left_entry_id, right_entry_id],
                |row| {
                    Ok(ConfusionPairDetail {
                        paired_entry_id: row.get(0)?,
                        paired_entry_title: row.get(1)?,
                        distinction_explanation: row.get(2)?,
                        common_confusion_reason: row.get(3)?,
                        clue_to_distinguish: row.get(4)?,
                        example_sentence_1: row.get(5)?,
                        example_sentence_2: row.get(6)?,
                        confusion_frequency_bp: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_legacy_contrast_pair(
        &self,
        left_entry_id: i64,
        right_entry_id: i64,
    ) -> EcoachResult<Option<ConfusionPairDetail>> {
        self.conn
            .query_row(
                "SELECT
                    CASE WHEN cp.left_entry_id = ?1 THEN cp.right_entry_id ELSE cp.left_entry_id END,
                    CASE WHEN cp.left_entry_id = ?1 THEN right_ke.title ELSE left_ke.title END,
                    cp.title,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    cp.trap_strength
                 FROM contrast_pairs cp
                 INNER JOIN knowledge_entries left_ke ON left_ke.id = cp.left_entry_id
                 INNER JOIN knowledge_entries right_ke ON right_ke.id = cp.right_entry_id
                 WHERE (cp.left_entry_id = ?1 AND cp.right_entry_id = ?2)
                    OR (cp.left_entry_id = ?2 AND cp.right_entry_id = ?1)
                 LIMIT 1",
                params![left_entry_id, right_entry_id],
                |row| {
                    Ok(ConfusionPairDetail {
                        paired_entry_id: row.get(0)?,
                        paired_entry_title: row.get(1)?,
                        distinction_explanation: row.get(2)?,
                        common_confusion_reason: row.get(3)?,
                        clue_to_distinguish: row.get(4)?,
                        example_sentence_1: row.get(5)?,
                        example_sentence_2: row.get(6)?,
                        confusion_frequency_bp: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_legacy_contrast_pairs(
        &self,
        entry_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<ConfusionPairDetail>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    CASE WHEN cp.left_entry_id = ?1 THEN cp.right_entry_id ELSE cp.left_entry_id END,
                    CASE WHEN cp.left_entry_id = ?1 THEN right_ke.title ELSE left_ke.title END,
                    cp.title,
                    NULL,
                    NULL,
                    NULL,
                    NULL,
                    cp.trap_strength
                 FROM contrast_pairs cp
                 INNER JOIN knowledge_entries left_ke ON left_ke.id = cp.left_entry_id
                 INNER JOIN knowledge_entries right_ke ON right_ke.id = cp.right_entry_id
                 WHERE cp.left_entry_id = ?1 OR cp.right_entry_id = ?1
                 ORDER BY cp.trap_strength DESC, cp.id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(
            stmt.query_map(params![entry_id, limit.max(1) as i64], |row| {
                Ok(ConfusionPairDetail {
                    paired_entry_id: row.get(0)?,
                    paired_entry_title: row.get(1)?,
                    distinction_explanation: row.get(2)?,
                    common_confusion_reason: row.get(3)?,
                    clue_to_distinguish: row.get(4)?,
                    example_sentence_1: row.get(5)?,
                    example_sentence_2: row.get(6)?,
                    confusion_frequency_bp: row.get(7)?,
                })
            }),
        )
    }

    fn load_shared_bundles(
        &self,
        left_entry_id: i64,
        right_entry_id: i64,
    ) -> EcoachResult<Vec<EntryBundleReference>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    kb.id,
                    kb.title,
                    kb.bundle_type,
                    kb.description,
                    NULL,
                    NULL,
                    1,
                    kb.difficulty_level,
                    kb.exam_relevance_score
                 FROM knowledge_bundles kb
                 WHERE EXISTS (
                        SELECT 1 FROM knowledge_bundle_items kbi
                        WHERE kbi.bundle_id = kb.id AND kbi.entry_id = ?1
                    )
                   AND EXISTS (
                        SELECT 1 FROM knowledge_bundle_items kbi
                        WHERE kbi.bundle_id = kb.id AND kbi.entry_id = ?2
                    )
                 ORDER BY kb.exam_relevance_score DESC, kb.title ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(
            stmt.query_map(params![left_entry_id, right_entry_id], |row| {
                Ok(EntryBundleReference {
                    bundle_id: row.get(0)?,
                    title: row.get(1)?,
                    bundle_type: row.get(2)?,
                    description: row.get(3)?,
                    item_role: row.get(4)?,
                    sequence_order: row.get(5)?,
                    required: row.get::<_, i64>(6)? == 1,
                    difficulty_level: row.get(7)?,
                    exam_relevance_score: row.get(8)?,
                })
            }),
        )
    }

    fn load_shared_question_ids(
        &self,
        left_entry_id: i64,
        right_entry_id: i64,
    ) -> EcoachResult<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT question_id
                 FROM question_glossary_links
                 WHERE entry_id = ?1
                   AND question_id IN (
                        SELECT question_id
                        FROM question_glossary_links
                        WHERE entry_id = ?2
                    )
                 ORDER BY question_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(
            stmt.query_map(params![left_entry_id, right_entry_id], |row| {
                row.get::<_, i64>(0)
            }),
        )
    }

    fn load_concept_map_node(&self, entry_id: i64) -> EcoachResult<ConceptMapNode> {
        self.conn
            .query_row(
                "SELECT id, title, entry_type, topic_id
                 FROM knowledge_entries
                 WHERE id = ?1",
                [entry_id],
                |row| {
                    Ok(ConceptMapNode {
                        entry_id: row.get(0)?,
                        title: row.get(1)?,
                        entry_type: row.get(2)?,
                        topic_id: row.get(3)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn ensure_student_entry_state(&self, student_id: i64, entry_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO student_entry_state (user_id, entry_id)
             VALUES (?1, ?2)
             ON CONFLICT(user_id, entry_id) DO NOTHING",
                params![student_id, entry_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO glossary_entry_mastery (student_id, entry_id)
             VALUES (?1, ?2)
             ON CONFLICT(student_id, entry_id) DO NOTHING",
                params![student_id, entry_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn apply_interaction_effects(
        &self,
        student_id: i64,
        entry_id: i64,
        event_type: &str,
    ) -> EcoachResult<()> {
        match event_type {
            "opened_entry" | "viewed_entry" | "entry_opened" => {
                self.conn
                    .execute(
                        "UPDATE student_entry_state
                     SET open_count = COALESCE(open_count, 0) + 1,
                         last_viewed_at = datetime('now'),
                         familiarity_state = CASE
                            WHEN familiarity_state IN ('unseen', '') OR familiarity_state IS NULL
                                THEN 'opened'
                            ELSE familiarity_state
                         END
                     WHERE user_id = ?1 AND entry_id = ?2",
                        params![student_id, entry_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                self.conn
                    .execute(
                        "UPDATE glossary_entry_mastery
                     SET mastery_state = CASE
                            WHEN mastery_state = 'unseen' THEN 'seen'
                            ELSE mastery_state
                         END,
                         updated_at = datetime('now')
                     WHERE student_id = ?1 AND entry_id = ?2",
                        params![student_id, entry_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
            "played_audio" | "audio_play" => {
                self.conn
                    .execute(
                        "UPDATE student_entry_state
                     SET last_played_at = datetime('now')
                     WHERE user_id = ?1 AND entry_id = ?2",
                        params![student_id, entry_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
            "question_failed" | "test_fail" => {
                self.conn
                    .execute(
                        "UPDATE student_entry_state
                     SET confusion_score = COALESCE(confusion_score, 0) + 700,
                         linked_wrong_answer_count = COALESCE(linked_wrong_answer_count, 0) + 1,
                         review_due_at = datetime('now', '+1 day'),
                         test_count = COALESCE(test_count, 0) + 1,
                         last_tested_at = datetime('now')
                     WHERE user_id = ?1 AND entry_id = ?2",
                        params![student_id, entry_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                self.conn
                    .execute(
                        "UPDATE glossary_entry_mastery
                     SET mastery_state = 'at_risk',
                         at_risk_flag = 1,
                         updated_at = datetime('now')
                     WHERE student_id = ?1 AND entry_id = ?2",
                        params![student_id, entry_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
            "question_passed" | "test_pass" => {
                self.conn
                    .execute(
                        "UPDATE student_entry_state
                     SET mastery_score = COALESCE(mastery_score, 0) + 600,
                         recall_strength = COALESCE(recall_strength, 0) + 450,
                         test_count = COALESCE(test_count, 0) + 1,
                         test_pass_count = COALESCE(test_pass_count, 0) + 1,
                         last_tested_at = datetime('now'),
                         review_due_at = datetime('now', '+5 day')
                     WHERE user_id = ?1 AND entry_id = ?2",
                        params![student_id, entry_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                self.conn.execute(
                    "UPDATE glossary_entry_mastery
                     SET mastery_state = CASE
                            WHEN mastery_state IN ('unseen', 'seen', 'explored') THEN 'understood'
                            WHEN mastery_state IN ('understood', 'recalled', 'recognized') THEN 'applied'
                            ELSE 'strong'
                         END,
                         consecutive_correct = COALESCE(consecutive_correct, 0) + 1,
                         at_risk_flag = 0,
                         updated_at = datetime('now')
                     WHERE student_id = ?1 AND entry_id = ?2",
                    params![student_id, entry_id],
                ).map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
            "revisited_after_forgetting" | "recovery" => {
                self.conn
                    .execute(
                        "UPDATE student_entry_state
                     SET review_due_at = datetime('now'),
                         spaced_review_due_at = datetime('now', '+1 day'),
                         at_risk_threshold_date = datetime('now', '+2 day')
                     WHERE user_id = ?1 AND entry_id = ?2",
                        params![student_id, entry_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                self.conn
                    .execute(
                        "UPDATE glossary_entry_mastery
                     SET mastery_state = 'at_risk',
                         at_risk_flag = 1,
                         updated_at = datetime('now')
                     WHERE student_id = ?1 AND entry_id = ?2",
                        params![student_id, entry_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
            _ => {}
        }
        Ok(())
    }

    fn refresh_search_index_for_entry(&self, entry_id: i64) -> EcoachResult<()> {
        let detail = self.get_entry_detail(None, entry_id, 12, 8)?;
        let alias_tokens = detail
            .aliases
            .iter()
            .map(|alias| alias.alias_text.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let misconception_text = detail
            .misconceptions
            .iter()
            .map(|item| item.misconception_text.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let formula_speech_text = detail
            .formula_meta
            .as_ref()
            .map(|meta| {
                format!(
                    "{} {}",
                    meta.formula_expression,
                    meta.formula_speech.clone().unwrap_or_default()
                )
            })
            .unwrap_or_default();
        let topic_labels = detail
            .entry
            .topic_id
            .map(|topic_id| self.topic_name(topic_id).unwrap_or_default())
            .unwrap_or_default();
        let bundle_labels = detail
            .bundles
            .iter()
            .map(|bundle| bundle.title.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let intent_keywords = detail
            .concept_meta
            .as_ref()
            .map(|meta| meta.misconception_signals.join(" "))
            .unwrap_or_default();

        self.conn
            .execute(
                "INSERT INTO glossary_search_index (
                entry_id,
                title_tokens,
                alias_tokens,
                full_text_content,
                simple_text_content,
                formula_speech_text,
                misconception_text,
                topic_labels,
                bundle_labels,
                intent_keywords,
                updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
             ON CONFLICT(entry_id) DO UPDATE SET
                title_tokens = excluded.title_tokens,
                alias_tokens = excluded.alias_tokens,
                full_text_content = excluded.full_text_content,
                simple_text_content = excluded.simple_text_content,
                formula_speech_text = excluded.formula_speech_text,
                misconception_text = excluded.misconception_text,
                topic_labels = excluded.topic_labels,
                bundle_labels = excluded.bundle_labels,
                intent_keywords = excluded.intent_keywords,
                updated_at = excluded.updated_at",
                params![
                    entry_id,
                    detail.entry.title,
                    alias_tokens,
                    detail.entry.full_text.unwrap_or_default(),
                    detail.entry.simple_text.unwrap_or_default(),
                    formula_speech_text,
                    misconception_text,
                    topic_labels,
                    bundle_labels,
                    intent_keywords,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn load_focus_entries(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        limit: i64,
        order_clause: &str,
        default_reason: String,
    ) -> EcoachResult<Vec<GlossaryEntryFocus>> {
        let sql = format!(
            "SELECT
                ke.id,
                ke.title,
                ke.entry_type,
                ke.topic_id,
                t.name,
                (COALESCE(ses.confusion_score, 0) + (10000 - COALESCE(ses.recall_strength, 0)) + COALESCE(ke.priority_score, 0)) AS need_score,
                COALESCE(ke.exam_relevance_score, 0),
                gem.mastery_state
             FROM knowledge_entries ke
             LEFT JOIN topics t ON t.id = ke.topic_id
             LEFT JOIN student_entry_state ses ON ses.entry_id = ke.id AND ses.user_id = ?1
             LEFT JOIN glossary_entry_mastery gem ON gem.entry_id = ke.id AND gem.student_id = ?1
             WHERE (?2 IS NULL OR ke.subject_id = ?2)
               AND ke.status = 'active'
             ORDER BY {}
             LIMIT ?3",
            order_clause
        );
        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(
            stmt.query_map(params![student_id, subject_id, limit], |row| {
                Ok(GlossaryEntryFocus {
                    entry_id: row.get(0)?,
                    title: row.get(1)?,
                    entry_type: row.get(2)?,
                    topic_id: row.get(3)?,
                    topic_name: row.get(4)?,
                    need_score: row.get(5)?,
                    exam_relevance_score: row.get(6)?,
                    match_reason: default_reason.clone(),
                    mastery_state: row.get(7)?,
                })
            }),
        )
    }

    fn search_entry_results(
        &self,
        input: &GlossarySearchInput,
        normalized_query: &str,
        limit: usize,
    ) -> EcoachResult<Vec<GlossarySearchResult>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    ke.id,
                    ke.title,
                    ke.entry_type,
                    ke.short_text,
                    ke.topic_id,
                    t.name,
                    COALESCE(ke.canonical_name, ''),
                    COALESCE(ke.slug, ''),
                    COALESCE(ke.full_text, ''),
                    COALESCE(ke.simple_text, ''),
                    COALESCE(ke.technical_text, ''),
                    COALESCE(ke.exam_text, ''),
                    COALESCE(ke.phonetic_text, ''),
                    COALESCE(ke.importance_score, 0),
                    COALESCE(ke.exam_relevance_score, 0),
                    COALESCE(ke.priority_score, 0),
                    COALESCE(ke.audio_available, 0),
                    COALESCE((SELECT group_concat(alias_text, ' ') FROM entry_aliases WHERE entry_id = ke.id), ''),
                    COALESCE((SELECT group_concat(misconception_text, ' ') FROM entry_misconceptions WHERE entry_id = ke.id), ''),
                    COALESCE((SELECT formula_expression || ' ' || COALESCE(formula_speech, '') FROM formula_meta WHERE entry_id = ke.id), ''),
                    COALESCE((
                        SELECT group_concat(bundle_tokens.label, ' ')
                        FROM (
                            SELECT kb.title AS label
                            FROM knowledge_bundle_items kbi
                            INNER JOIN knowledge_bundles kb ON kb.id = kbi.bundle_id
                            WHERE kbi.entry_id = ke.id
                        ) bundle_tokens
                    ), ''),
                    COALESCE((
                        SELECT group_concat(question_tokens.stem, ' ')
                        FROM (
                            SELECT q.stem AS stem
                            FROM question_glossary_links qgl
                            INNER JOIN questions q ON q.id = qgl.question_id
                            WHERE qgl.entry_id = ke.id
                        ) question_tokens
                    ), ''),
                    COALESCE(ses.confusion_score, 0),
                    COALESCE(ses.recall_strength, 0),
                    gem.mastery_state
                 FROM knowledge_entries ke
                 LEFT JOIN topics t ON t.id = ke.topic_id
                 LEFT JOIN student_entry_state ses
                    ON ses.entry_id = ke.id AND ses.user_id = ?1
                 LEFT JOIN glossary_entry_mastery gem
                    ON gem.entry_id = ke.id AND gem.student_id = ?1
                 WHERE (?2 IS NULL OR ke.subject_id = ?2)
                   AND (?3 IS NULL OR ke.topic_id = ?3)
                   AND ke.status = 'active'
                 ORDER BY ke.title ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(
                params![input.student_id, input.subject_id, input.topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, String>(9)?,
                        row.get::<_, String>(10)?,
                        row.get::<_, String>(11)?,
                        row.get::<_, String>(12)?,
                        row.get::<_, i64>(13)?,
                        row.get::<_, i64>(14)?,
                        row.get::<_, i64>(15)?,
                        row.get::<_, i64>(16)?,
                        row.get::<_, String>(17)?,
                        row.get::<_, String>(18)?,
                        row.get::<_, String>(19)?,
                        row.get::<_, String>(20)?,
                        row.get::<_, String>(21)?,
                        row.get::<_, i64>(22)?,
                        row.get::<_, i64>(23)?,
                        row.get::<_, Option<String>>(24)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let (
                entry_id,
                title,
                entry_type,
                short_text,
                topic_id,
                topic_name,
                canonical_name,
                slug,
                full_text,
                simple_text,
                technical_text,
                exam_text,
                phonetic_text,
                importance_score,
                exam_relevance_score,
                priority_score,
                audio_available,
                aliases,
                misconception_text,
                formula_text,
                bundle_labels,
                question_signals,
                confusion_score,
                recall_strength,
                mastery_state,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;

            if input.include_audio_ready_only && audio_available != 1 {
                continue;
            }

            let mut best_score = 0i64;
            let mut reason = None::<String>;
            for (field_label, text, weight) in [
                ("title", title.as_str(), 5000),
                ("canonical name", canonical_name.as_str(), 4300),
                ("alias", aliases.as_str(), 4200),
                ("formula", formula_text.as_str(), 4100),
                (
                    "short explanation",
                    short_text.as_deref().unwrap_or(""),
                    3600,
                ),
                ("simple explanation", simple_text.as_str(), 3300),
                ("exam explanation", exam_text.as_str(), 3200),
                ("technical explanation", technical_text.as_str(), 2800),
                ("misconception", misconception_text.as_str(), 2500),
                ("question signal", question_signals.as_str(), 2200),
                ("bundle label", bundle_labels.as_str(), 2100),
                ("phonetic text", phonetic_text.as_str(), 1800),
                ("slug", slug.as_str(), 1700),
                ("full explanation", full_text.as_str(), 1500),
            ] {
                let score = score_text_match(text, normalized_query, weight);
                if score > best_score {
                    best_score = score;
                    reason = Some(format!("matched {}", field_label));
                }
            }

            if best_score == 0 {
                continue;
            }
            best_score += importance_score / 5 + exam_relevance_score / 4 + priority_score / 4;
            best_score += confusion_score / 6;
            best_score += (10_000 - recall_strength).max(0) / 8;
            if matches!(
                mastery_state.as_deref(),
                Some("at_risk" | "understood" | "recalled")
            ) {
                best_score += 600;
            }

            results.push(GlossarySearchResult {
                result_type: "entry".to_string(),
                entry_id: Some(entry_id),
                bundle_id: None,
                question_id: None,
                title,
                subtitle: short_text,
                entry_type: Some(entry_type),
                topic_id,
                topic_name,
                match_reason: reason.unwrap_or_else(|| "matched glossary entry".to_string()),
                match_score: best_score,
                metadata: json!({
                    "aliases": aliases,
                    "formula_text": formula_text,
                    "question_signals": question_signals,
                    "mastery_state": mastery_state,
                }),
            });
        }

        results.sort_by(|left, right| right.match_score.cmp(&left.match_score));
        results.truncate(limit.max(1));
        Ok(results)
    }

    fn search_bundle_results(
        &self,
        input: &GlossarySearchInput,
        normalized_query: &str,
        limit: usize,
    ) -> EcoachResult<Vec<GlossarySearchResult>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    kb.id,
                    kb.title,
                    kb.bundle_type,
                    kb.description,
                    kb.topic_id,
                    t.name,
                    COALESCE(kb.exam_relevance_score, 0),
                    COALESCE((
                        SELECT group_concat(ke.title, ' ')
                        FROM knowledge_bundle_items kbi
                        INNER JOIN knowledge_entries ke ON ke.id = kbi.entry_id
                        WHERE kbi.bundle_id = kb.id
                    ), '')
                 FROM knowledge_bundles kb
                 LEFT JOIN topics t ON t.id = kb.topic_id
                 WHERE (?1 IS NULL OR kb.subject_id = ?1)
                   AND (?2 IS NULL OR kb.topic_id = ?2)
                 ORDER BY kb.title ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![input.subject_id, input.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut results = Vec::new();
        for row in rows {
            let (
                bundle_id,
                title,
                bundle_type,
                description,
                topic_id,
                topic_name,
                exam_relevance_score,
                entry_titles,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let mut score = score_text_match(&title, normalized_query, 4200);
            score = score.max(score_text_match(&bundle_type, normalized_query, 3200));
            score = score.max(score_text_match(
                description.as_deref().unwrap_or(""),
                normalized_query,
                2400,
            ));
            score = score.max(score_text_match(&entry_titles, normalized_query, 2200));
            if score == 0 {
                continue;
            }
            score += exam_relevance_score / 4;
            results.push(GlossarySearchResult {
                result_type: "bundle".to_string(),
                entry_id: None,
                bundle_id: Some(bundle_id),
                question_id: None,
                title,
                subtitle: description,
                entry_type: Some(bundle_type),
                topic_id,
                topic_name,
                match_reason: "matched bundle".to_string(),
                match_score: score,
                metadata: json!({ "entry_titles": entry_titles }),
            });
        }
        results.sort_by(|left, right| right.match_score.cmp(&left.match_score));
        results.truncate(limit.max(1));
        Ok(results)
    }

    fn search_question_results(
        &self,
        input: &GlossarySearchInput,
        normalized_query: &str,
        limit: usize,
    ) -> EcoachResult<Vec<GlossarySearchResult>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    q.id,
                    q.stem,
                    q.topic_id,
                    t.name,
                    COALESCE(MAX(qgl.confidence_score), 0)
                 FROM question_glossary_links qgl
                 INNER JOIN questions q ON q.id = qgl.question_id
                 LEFT JOIN topics t ON t.id = q.topic_id
                 INNER JOIN knowledge_entries ke ON ke.id = qgl.entry_id
                 WHERE (?1 IS NULL OR ke.subject_id = ?1)
                   AND (?2 IS NULL OR ke.topic_id = ?2)
                 GROUP BY q.id, q.stem, q.topic_id, t.name
                 ORDER BY q.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![input.subject_id, input.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut results = Vec::new();
        for row in rows {
            let (question_id, stem, topic_id, topic_name, confidence_score) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let mut score = score_text_match(&stem, normalized_query, 3400);
            if score == 0 {
                continue;
            }
            score += confidence_score / 3;
            results.push(GlossarySearchResult {
                result_type: "question".to_string(),
                entry_id: None,
                bundle_id: None,
                question_id: Some(question_id),
                title: truncate_title(&stem, 80),
                subtitle: Some("Exam-style linked question".to_string()),
                entry_type: None,
                topic_id,
                topic_name,
                match_reason: "matched question signal".to_string(),
                match_score: score,
                metadata: json!({ "stem": stem }),
            });
        }
        results.sort_by(|left, right| right.match_score.cmp(&left.match_score));
        results.truncate(limit.max(1));
        Ok(results)
    }

    fn search_confusion_results(
        &self,
        normalized_query: &str,
        limit: usize,
    ) -> EcoachResult<Vec<GlossarySearchResult>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    cp.entry_id_1,
                    e1.title,
                    cp.entry_id_2,
                    e2.title,
                    cp.distinction_explanation,
                    cp.common_confusion_reason,
                    cp.confusion_frequency_bp
                 FROM confusion_pairs cp
                 INNER JOIN knowledge_entries e1 ON e1.id = cp.entry_id_1
                 INNER JOIN knowledge_entries e2 ON e2.id = cp.entry_id_2
                 ORDER BY cp.confusion_frequency_bp DESC, cp.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut results = Vec::new();
        for row in rows {
            let (left_id, left_title, right_id, right_title, distinction, reason, frequency) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let combined = format!(
                "{} {} {} {}",
                left_title,
                right_title,
                distinction,
                reason.clone().unwrap_or_default()
            );
            let mut score = score_text_match(&combined, normalized_query, 3600);
            if score == 0 {
                continue;
            }
            score += frequency / 3;
            results.push(GlossarySearchResult {
                result_type: "confusion_pair".to_string(),
                entry_id: Some(left_id),
                bundle_id: None,
                question_id: None,
                title: format!("{} vs {}", left_title, right_title),
                subtitle: Some(distinction.clone()),
                entry_type: Some("comparison".to_string()),
                topic_id: None,
                topic_name: None,
                match_reason: "matched confusion pair".to_string(),
                match_score: score,
                metadata: json!({
                    "left_entry_id": left_id,
                    "right_entry_id": right_id,
                    "reason": reason,
                }),
            });
        }
        results.sort_by(|left, right| right.match_score.cmp(&left.match_score));
        results.truncate(limit.max(1));
        Ok(results)
    }

    fn build_audio_program_for_entry(
        &self,
        student_id: i64,
        entry_id: i64,
        _limit: usize,
    ) -> EcoachResult<GlossaryAudioProgram> {
        let entry_source = self.load_audio_source(entry_id)?;
        let bundles = match entry_source.topic_id {
            Some(topic_id) => self.list_bundles_for_topic(topic_id)?,
            None => Vec::new(),
        };
        let context = self.build_audio_program_context(student_id, entry_source.topic_id)?;
        Ok(self.compose_audio_program(
            format!("{} glossary audio", entry_source.title),
            "custom".to_string().as_str(),
            entry_source.topic_id,
            None,
            &bundles,
            &[entry_source],
            &context,
        ))
    }

    fn build_audio_program_for_bundle(
        &self,
        student_id: i64,
        bundle_id: i64,
        limit: usize,
    ) -> EcoachResult<GlossaryAudioProgram> {
        let (title, topic_id): (String, Option<i64>) = self
            .conn
            .query_row(
                "SELECT title, topic_id FROM knowledge_bundles WHERE id = ?1",
                [bundle_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT entry_id
                 FROM knowledge_bundle_items
                 WHERE bundle_id = ?1
                 ORDER BY sequence_order ASC, id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let entry_ids = collect_rows(
            stmt.query_map(params![bundle_id, limit.max(1) as i64], |row| {
                row.get::<_, i64>(0)
            }),
        )?;
        let mut entry_sources = Vec::new();
        for entry_id in entry_ids {
            entry_sources.push(self.load_audio_source(entry_id)?);
        }
        let bundles = vec![KnowledgeBundleSequenceItem {
            bundle_id,
            title: title.clone(),
            bundle_type: "bundle".to_string(),
            sequence_order: 0,
            focus_reason: "learn together bundle".to_string(),
            due_review_count: 0,
            focus_entry_ids: entry_sources.iter().map(|entry| entry.id).collect(),
            focus_entry_titles: entry_sources
                .iter()
                .map(|entry| entry.title.clone())
                .collect(),
        }];
        let context = self.build_audio_program_context(student_id, topic_id)?;
        Ok(self.compose_audio_program(
            format!("{} bundle audio", title),
            "bundle",
            topic_id,
            None,
            &self
                .list_bundles_for_topic(topic_id.unwrap_or_default())
                .unwrap_or_default(),
            &entry_sources,
            &AudioProgramContext {
                recommended_bundles: bundles,
                ..context
            },
        ))
    }

    fn build_weakness_audio_program(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<GlossaryAudioProgram> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT entry_id
                 FROM student_entry_state
                 WHERE user_id = ?1
                 ORDER BY confusion_score DESC, recall_strength ASC, linked_wrong_answer_count DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let entry_ids = collect_rows(
            stmt.query_map(params![student_id, limit.max(1) as i64], |row| {
                row.get::<_, i64>(0)
            }),
        )?;
        let mut entry_sources = Vec::new();
        for entry_id in entry_ids {
            entry_sources.push(self.load_audio_source(entry_id)?);
        }
        let topic_id = entry_sources.iter().find_map(|entry| entry.topic_id);
        let bundles = if let Some(topic_id) = topic_id {
            self.list_bundles_for_topic(topic_id)?
        } else {
            Vec::new()
        };
        let context = self.build_audio_program_context(student_id, topic_id)?;
        Ok(self.compose_audio_program(
            "Weakness glossary audio".to_string(),
            "weakness_flow",
            topic_id,
            None,
            &bundles,
            &entry_sources,
            &context,
        ))
    }

    fn persist_audio_program(
        &self,
        student_id: i64,
        program: &GlossaryAudioProgram,
        source_type: &str,
        source_id: i64,
        teaching_mode: &str,
    ) -> EcoachResult<i64> {
        let total_duration = program
            .segments
            .iter()
            .map(|segment| segment.duration_seconds)
            .sum::<i64>();
        self.conn
            .execute(
                "INSERT INTO glossary_audio_programs (
                title,
                source_type,
                source_id,
                teaching_mode,
                student_id,
                total_duration_seconds,
                item_count,
                status
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'ready')",
                params![
                    program.program_title,
                    source_type,
                    source_id,
                    teaching_mode,
                    student_id,
                    total_duration,
                    program.segments.len() as i64,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let program_id = self.conn.last_insert_rowid();
        for segment in &program.segments {
            self.conn
                .execute(
                    "INSERT INTO glossary_audio_program_items (
                    program_id,
                    sequence_no,
                    segment_id,
                    entry_id,
                    prompt_text,
                    focus_reason,
                    duration_seconds
                 ) VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6)",
                    params![
                        program_id,
                        segment.sequence_no,
                        segment.entry_id,
                        segment.prompt_text,
                        segment.focus_reason,
                        segment.duration_seconds,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(program_id)
    }

    fn load_persisted_audio_program(&self, program_id: i64) -> EcoachResult<GlossaryAudioProgram> {
        let header = self
            .conn
            .query_row(
                "SELECT title, source_type, teaching_mode
                 FROM glossary_audio_programs
                 WHERE id = ?1",
                [program_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT sequence_no, entry_id, prompt_text, focus_reason, duration_seconds
                 FROM glossary_audio_program_items
                 WHERE program_id = ?1
                 ORDER BY sequence_no ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let segments = collect_rows(stmt.query_map([program_id], |row| {
            let entry_id: Option<i64> = row.get(1)?;
            let title = match entry_id {
                Some(entry_id) => self
                    .conn
                    .query_row(
                        "SELECT title FROM knowledge_entries WHERE id = ?1",
                        [entry_id],
                        |title_row| title_row.get::<_, String>(0),
                    )
                    .unwrap_or_else(|_| "Glossary Segment".to_string()),
                None => "Glossary Segment".to_string(),
            };
            Ok(GlossaryAudioSegment {
                sequence_no: row.get(0)?,
                segment_type: "segment".to_string(),
                title,
                script_text: row
                    .get::<_, Option<String>>(2)?
                    .unwrap_or_else(|| "Continue the glossary audio flow.".to_string()),
                entry_id,
                prompt_text: row.get(2)?,
                focus_reason: row.get(3)?,
                duration_seconds: row.get(4)?,
            })
        }))?;

        Ok(GlossaryAudioProgram {
            program_title: header.0,
            source_type: header.1,
            teaching_mode: header.2,
            topic_id: None,
            question_id: None,
            bundle_ids: Vec::new(),
            recommended_bundles: Vec::new(),
            entry_ids: segments
                .iter()
                .filter_map(|segment| segment.entry_id)
                .collect(),
            listener_signals: Vec::new(),
            contrast_titles: Vec::new(),
            review_entry_ids: Vec::new(),
            review_entry_titles: Vec::new(),
            relationship_review_prompts: Vec::new(),
            segments,
        })
    }

    fn select_test_entry_ids(
        &self,
        student_id: i64,
        input: &CreateGlossaryTestInput,
    ) -> EcoachResult<Vec<i64>> {
        if !input.entry_ids.is_empty() {
            return Ok(input
                .entry_ids
                .iter()
                .copied()
                .take(input.entry_count.max(1))
                .collect());
        }
        if let Some(bundle_id) = input.bundle_id {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT entry_id
                     FROM knowledge_bundle_items
                     WHERE bundle_id = ?1
                     ORDER BY sequence_order ASC, id ASC
                     LIMIT ?2",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            return collect_rows(
                stmt.query_map(params![bundle_id, input.entry_count.max(1) as i64], |row| {
                    row.get::<_, i64>(0)
                }),
            );
        }
        let topic_id = input.topic_id.ok_or_else(|| {
            EcoachError::Validation(
                "glossary test session needs entry_ids, bundle_id, or topic_id".to_string(),
            )
        })?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT ke.id
                 FROM knowledge_entries ke
                 LEFT JOIN student_entry_state ses
                    ON ses.entry_id = ke.id AND ses.user_id = ?1
                 WHERE ke.topic_id = ?2 AND ke.status = 'active'
                 ORDER BY (COALESCE(ses.confusion_score, 0) + (10000 - COALESCE(ses.recall_strength, 0)) + COALESCE(ke.priority_score, 0)) DESC,
                          ke.title ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(stmt.query_map(
            params![student_id, topic_id, input.entry_count.max(1) as i64],
            |row| row.get::<_, i64>(0),
        ))
    }

    fn build_test_item(&self, entry_id: i64, test_mode: &str) -> EcoachResult<GlossaryTestItem> {
        let detail = self.get_entry_detail(None, entry_id, 8, 4)?;
        let entry_title = detail.entry.title.clone();
        let default_prompt = detail
            .definition_meta
            .as_ref()
            .and_then(|meta| {
                meta.short_definition
                    .clone()
                    .or(Some(meta.definition_text.clone()))
            })
            .or_else(|| detail.entry.short_text.clone())
            .or_else(|| detail.entry.simple_text.clone())
            .unwrap_or_else(|| format!("Explain {}", entry_title));

        let item = match test_mode {
            "formula_builder" => {
                let formula_meta = detail.formula_meta.clone().ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "formula_builder requires formula meta for entry {}",
                        entry_id
                    ))
                })?;
                GlossaryTestItem {
                    sequence_no: 0,
                    entry_id,
                    prompt_type: "formula_builder".to_string(),
                    prompt_text: formula_meta
                        .when_to_use
                        .clone()
                        .or(detail.entry.exam_text.clone())
                        .unwrap_or_else(|| format!("Rebuild the formula for {}", entry_title)),
                    expected_answer: Some(formula_meta.formula_expression.clone()),
                    options: formula_option_fragments(&formula_meta),
                    metadata: json!({
                        "formula_speech": formula_meta.formula_speech,
                        "when_not_to_use": formula_meta.when_not_to_use,
                    }),
                }
            }
            "context_recognition" => GlossaryTestItem {
                sequence_no: 0,
                entry_id,
                prompt_type: "context_recognition".to_string(),
                prompt_text: detail
                    .examples
                    .first()
                    .map(|example| example.example_text.clone())
                    .or_else(|| {
                        detail
                            .linked_questions
                            .first()
                            .and_then(|question| question.stem.clone())
                    })
                    .unwrap_or_else(|| default_prompt.clone()),
                expected_answer: Some(entry_title.clone()),
                options: Vec::new(),
                metadata: json!({}),
            },
            "confusion_duel" => {
                let confusion = detail.confusion_pairs.first().cloned().ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "confusion_duel requires a confusion pair for entry {}",
                        entry_id
                    ))
                })?;
                GlossaryTestItem {
                    sequence_no: 0,
                    entry_id,
                    prompt_type: "confusion_duel".to_string(),
                    prompt_text: confusion
                        .clue_to_distinguish
                        .clone()
                        .or(confusion.common_confusion_reason.clone())
                        .unwrap_or(confusion.distinction_explanation.clone()),
                    expected_answer: Some(entry_title.clone()),
                    options: vec![entry_title.clone(), confusion.paired_entry_title],
                    metadata: json!({}),
                }
            }
            "intruder_mode" => {
                let mapping = detail.neighbor_intruder.clone().ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "intruder_mode requires neighbor/intruder mapping for entry {}",
                        entry_id
                    ))
                })?;
                let options = self.resolve_entry_titles(
                    &mapping
                        .neighbors
                        .iter()
                        .chain(mapping.intruders.iter())
                        .copied()
                        .collect::<Vec<_>>(),
                )?;
                let intruder_title = self
                    .resolve_entry_titles(&mapping.intruders)
                    .map(|titles| titles.first().cloned().unwrap_or_default())?;
                GlossaryTestItem {
                    sequence_no: 0,
                    entry_id,
                    prompt_type: "intruder_mode".to_string(),
                    prompt_text: format!("Which one does not belong near {}?", entry_title),
                    expected_answer: Some(intruder_title),
                    options,
                    metadata: json!({}),
                }
            }
            "question_signal" => GlossaryTestItem {
                sequence_no: 0,
                entry_id,
                prompt_type: "question_signal".to_string(),
                prompt_text: detail
                    .linked_questions
                    .first()
                    .and_then(|question| question.stem.clone())
                    .unwrap_or_else(|| default_prompt.clone()),
                expected_answer: Some(entry_title.clone()),
                options: Vec::new(),
                metadata: json!({}),
            },
            "connection_map" => GlossaryTestItem {
                sequence_no: 0,
                entry_id,
                prompt_type: "connection_map".to_string(),
                prompt_text: format!("Name a directly related idea to {}.", entry_title),
                expected_answer: detail
                    .relations
                    .first()
                    .map(|relation| relation.related_entry_title.clone()),
                options: detail
                    .relations
                    .iter()
                    .take(4)
                    .map(|relation| relation.related_entry_title.clone())
                    .collect(),
                metadata: json!({}),
            },
            "fill_gap" => {
                let formula = detail
                    .formula_meta
                    .as_ref()
                    .map(|meta| meta.formula_expression.clone())
                    .unwrap_or_else(|| entry_title.clone());
                let expected = last_formula_token(&formula).unwrap_or_else(|| entry_title.clone());
                GlossaryTestItem {
                    sequence_no: 0,
                    entry_id,
                    prompt_type: "fill_gap".to_string(),
                    prompt_text: formula.replacen(&expected, "___", 1),
                    expected_answer: Some(expected),
                    options: Vec::new(),
                    metadata: json!({}),
                }
            }
            _ => GlossaryTestItem {
                sequence_no: 0,
                entry_id,
                prompt_type: test_mode.to_string(),
                prompt_text: default_prompt,
                expected_answer: Some(entry_title),
                options: Vec::new(),
                metadata: json!({}),
            },
        };

        Ok(item)
    }

    fn acceptable_answers_for_item(&self, item: &GlossaryTestItem) -> EcoachResult<Vec<String>> {
        let mut answers = Vec::new();
        if let Some(expected_answer) = item.expected_answer.clone() {
            answers.push(expected_answer);
        }
        let aliases = self.load_entry_aliases(item.entry_id)?;
        for alias in aliases {
            answers.push(alias.alias_text);
        }
        Ok(answers)
    }

    fn resolve_entry_titles(&self, entry_ids: &[i64]) -> EcoachResult<Vec<String>> {
        let mut titles = Vec::new();
        for entry_id in entry_ids {
            let title: Option<String> = self
                .conn
                .query_row(
                    "SELECT title FROM knowledge_entries WHERE id = ?1",
                    [entry_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(title) = title {
                titles.push(title);
            }
        }
        Ok(titles)
    }

    fn apply_test_outcome(
        &self,
        student_id: i64,
        entry_id: i64,
        is_correct: bool,
        test_mode: &str,
    ) -> EcoachResult<()> {
        self.apply_interaction_effects(
            student_id,
            entry_id,
            if is_correct { "test_pass" } else { "test_fail" },
        )?;
        let (recognition_delta, connection_delta, application_delta, retention_delta) =
            test_outcome_deltas(test_mode, is_correct);
        self.conn
            .execute(
                "UPDATE student_entry_state
             SET recognition_score = COALESCE(recognition_score, 0) + ?3,
                 connection_score = COALESCE(connection_score, 0) + ?4,
                 application_score = COALESCE(application_score, 0) + ?5,
                 retention_score = COALESCE(retention_score, 0) + ?6,
                 spaced_review_due_at = CASE
                    WHEN ?2 = 1 THEN datetime('now', '+7 day')
                    ELSE datetime('now', '+1 day')
                 END
             WHERE user_id = ?1 AND entry_id = ?7",
                params![
                    student_id,
                    is_correct as i64,
                    recognition_delta,
                    connection_delta,
                    application_delta,
                    retention_delta,
                    entry_id,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn refresh_glossary_test_session_scores(&self, session_id: i64) -> EcoachResult<()> {
        let (
            entry_count,
            attempt_count,
            recall_score,
            recognition_score,
            connection_score,
            application_score,
            retention_score,
        ): (i64, i64, f64, f64, f64, f64, f64) = self
            .conn
            .query_row(
                "SELECT
                    gts.entry_count,
                    COALESCE(COUNT(gta.id), 0),
                    COALESCE(AVG(gta.meaning_recall_bp), 0),
                    COALESCE(AVG(gta.word_recognition_bp), 0),
                    COALESCE(AVG(gta.relationship_understanding_bp), 0),
                    COALESCE(AVG(gta.context_transfer_bp), 0),
                    COALESCE(AVG(gta.formula_recall_bp), 0)
                 FROM glossary_test_sessions gts
                 LEFT JOIN glossary_test_attempts gta ON gta.test_session_id = gts.id
                 WHERE gts.id = ?1",
                [session_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let completion_rate_bp = if entry_count <= 0 {
            0
        } else {
            ((attempt_count * 10_000) / entry_count).clamp(0, 10_000)
        };
        self.conn
            .execute(
                "UPDATE glossary_test_sessions
             SET recall_score_bp = ?2,
                 recognition_score_bp = ?3,
                 connection_score_bp = ?4,
                 application_score_bp = ?5,
                 retention_score_bp = ?6,
                 completion_rate_bp = ?7
             WHERE id = ?1",
                params![
                    session_id,
                    recall_score.round() as i64,
                    recognition_score.round() as i64,
                    connection_score.round() as i64,
                    application_score.round() as i64,
                    retention_score.round() as i64,
                    completion_rate_bp,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn topic_name(&self, topic_id: i64) -> EcoachResult<String> {
        self.conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }
}

fn collect_rows<T, I>(rows: Result<I, rusqlite::Error>) -> EcoachResult<Vec<T>>
where
    I: Iterator<Item = Result<T, rusqlite::Error>>,
{
    let rows = rows.map_err(|err| EcoachError::Storage(err.to_string()))?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(out)
}

fn parse_json_value(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

fn parse_string_array(text: Option<String>) -> Vec<String> {
    match text {
        Some(text) if text.trim_start().starts_with('[') => {
            serde_json::from_str(&text).unwrap_or_default()
        }
        Some(text) => split_loose_terms(&Some(text)),
        None => Vec::new(),
    }
}

fn parse_i64_array(text: Option<String>) -> Vec<i64> {
    match text {
        Some(text) if text.trim_start().starts_with('[') => {
            serde_json::from_str(&text).unwrap_or_default()
        }
        Some(text) => text
            .split(',')
            .filter_map(|part| part.trim().parse::<i64>().ok())
            .collect(),
        None => Vec::new(),
    }
}

fn split_loose_terms(text: &Option<String>) -> Vec<String> {
    text.as_deref()
        .unwrap_or("")
        .split(|ch| matches!(ch, ',' | ';' | '\n' | '|'))
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn render_formula_speech(formula_expression: &str) -> String {
    formula_expression
        .replace("=", " equals ")
        .replace("/", " divided by ")
        .replace("*", " times ")
        .replace("+", " plus ")
        .replace("-", " minus ")
        .replace("^2", " squared ")
        .replace("^3", " cubed ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case(text: &str) -> String {
    text.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    let mut out = first.to_uppercase().collect::<String>();
                    out.push_str(chars.as_str());
                    out
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_search_query(query: &str) -> String {
    query
        .to_lowercase()
        .replace(['?', '.', ',', ':', ';', '!', '"', '\''], " ")
        .replace("what is", " ")
        .replace("explain", " ")
        .replace("play", " ")
        .replace("difference between", "compare ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn detect_query_intent(query: &str) -> String {
    if query.starts_with("compare ") || query.contains(" difference ") {
        "compare".to_string()
    } else if query.contains("formula") || query.contains('=') || query.contains('/') {
        "formula".to_string()
    } else if query.contains("audio") || query.contains("listen") {
        "audio".to_string()
    } else if query.contains("question") || query.contains("signal") {
        "question_bridge".to_string()
    } else {
        "lookup".to_string()
    }
}

fn score_text_match(text: &str, normalized_query: &str, base_weight: i64) -> i64 {
    if normalized_query.is_empty() || text.trim().is_empty() {
        return 0;
    }
    let haystack = normalize_search_query(text);
    if haystack.is_empty() {
        return 0;
    }
    if haystack == normalized_query {
        return base_weight;
    }
    if haystack.starts_with(normalized_query) {
        return base_weight - 250;
    }
    if haystack.contains(normalized_query) {
        return base_weight - 500;
    }
    let query_tokens = normalized_query.split_whitespace().collect::<Vec<_>>();
    let token_hits = query_tokens
        .iter()
        .filter(|token| haystack.contains(**token))
        .count() as i64;
    if token_hits > 0 {
        return (base_weight / 2) + token_hits * 180;
    }
    let distance = levenshtein_distance(&haystack, normalized_query);
    if distance <= 2 {
        return (base_weight / 2).saturating_sub(distance as i64 * 120);
    }
    0
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut costs = (0..=right_chars.len()).collect::<Vec<_>>();
    for (left_index, left_char) in left_chars.iter().enumerate() {
        let mut previous = costs[0];
        costs[0] = left_index + 1;
        for (right_index, right_char) in right_chars.iter().enumerate() {
            let temp = costs[right_index + 1];
            costs[right_index + 1] = if left_char == right_char {
                previous
            } else {
                1 + previous.min(costs[right_index]).min(costs[right_index + 1])
            };
            previous = temp;
        }
    }
    costs[right_chars.len()]
}

fn persisted_program_source(source_type: &str) -> &str {
    match source_type {
        "topic" => "topic",
        "bundle" => "bundle",
        "weakness" => "weakness_flow",
        "search" => "search_result",
        _ => "custom",
    }
}

fn normalize_test_mode(mode: &str) -> String {
    match mode {
        "recall"
        | "reverse_recall"
        | "audio_recall"
        | "formula_builder"
        | "context_recognition"
        | "confusion_duel"
        | "intruder_mode"
        | "question_signal"
        | "connection_map"
        | "fill_gap" => mode.to_string(),
        "relationship_hunt" => "connection_map".to_string(),
        "formula_reverse_recall" => "formula_builder".to_string(),
        "which_one_does_not_belong" => "intruder_mode".to_string(),
        other => other.to_string(),
    }
}

fn normalize_answer(text: &str) -> String {
    text.to_lowercase()
        .replace([' ', '\n', '\t'], "")
        .replace(['.', ',', ';', ':', '!', '?', '"', '\''], "")
}

fn formula_option_fragments(meta: &FormulaMeta) -> Vec<String> {
    if let Some(map) = meta.variables.as_object() {
        map.keys().cloned().collect()
    } else {
        meta.formula_expression
            .split(|ch: char| matches!(ch, '=' | '+' | '-' | '*' | '/' | '(' | ')' | '^'))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(ToString::to_string)
            .collect()
    }
}

fn last_formula_token(formula: &str) -> Option<String> {
    formula
        .split(|ch: char| matches!(ch, '=' | '+' | '-' | '*' | '/' | '(' | ')' | '^' | ' '))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .last()
        .map(ToString::to_string)
}

fn test_metric_scores(test_mode: &str, is_correct: bool) -> (i64, i64, i64, i64, i64, i64, i64) {
    let yes = if is_correct { 10_000 } else { 0 };
    match test_mode {
        "formula_builder" => (0, 0, yes, 0, 0, 0, yes / 2),
        "context_recognition" | "question_signal" => (yes / 2, yes, 0, yes, 0, 0, yes),
        "confusion_duel" | "intruder_mode" => (0, yes / 2, 0, yes / 2, yes, yes, 0),
        "connection_map" => (0, 0, 0, yes / 2, yes, 0, 0),
        "fill_gap" => (yes, yes / 2, yes / 2, 0, 0, 0, yes / 2),
        _ => (yes, yes, 0, yes / 2, yes / 2, 0, yes / 2),
    }
}

fn test_outcome_deltas(test_mode: &str, is_correct: bool) -> (i64, i64, i64, i64) {
    let scale = if is_correct { 600 } else { -150 };
    match test_mode {
        "formula_builder" => (100, 100, scale, 250),
        "context_recognition" | "question_signal" => (scale, 150, scale, 200),
        "confusion_duel" | "intruder_mode" => (200, 250, 100, 150),
        "connection_map" => (150, scale, 100, 100),
        _ => (scale, 100, 100, 200),
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use serde_json::json;

    use super::*;

    fn create_deep_glossary_schema(conn: &Connection) {
        conn.execute_batch(
            "
            CREATE TABLE topics (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
            CREATE TABLE knowledge_entries (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER,
                topic_id INTEGER,
                subtopic_id INTEGER,
                entry_type TEXT NOT NULL,
                title TEXT NOT NULL,
                canonical_name TEXT,
                slug TEXT,
                short_text TEXT,
                full_text TEXT,
                simple_text TEXT,
                technical_text TEXT,
                exam_text TEXT,
                importance_score INTEGER NOT NULL DEFAULT 0,
                difficulty_level INTEGER NOT NULL DEFAULT 0,
                grade_band TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now')),
                audio_available INTEGER NOT NULL DEFAULT 0,
                has_formula INTEGER NOT NULL DEFAULT 0,
                confusion_pair_count INTEGER NOT NULL DEFAULT 0,
                example_count INTEGER NOT NULL DEFAULT 0,
                misconception_count INTEGER NOT NULL DEFAULT 0,
                exam_relevance_score INTEGER NOT NULL DEFAULT 0,
                priority_score INTEGER NOT NULL DEFAULT 0,
                phonetic_text TEXT
            );
            CREATE TABLE entry_aliases (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL,
                alias_text TEXT NOT NULL,
                alias_type TEXT
            );
            CREATE TABLE entry_content_blocks (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL,
                block_type TEXT NOT NULL,
                order_index INTEGER NOT NULL DEFAULT 0,
                content_json TEXT NOT NULL DEFAULT '{}'
            );
            CREATE TABLE definition_meta (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL UNIQUE,
                definition_text TEXT NOT NULL,
                short_definition TEXT,
                real_world_meaning TEXT,
                non_examples TEXT,
                context_clues TEXT,
                pronunciation_text TEXT,
                formal_definition TEXT
            );
            CREATE TABLE formula_meta (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL UNIQUE,
                formula_expression TEXT NOT NULL,
                formula_speech TEXT,
                variables_json TEXT NOT NULL DEFAULT '[]',
                units_json TEXT,
                when_to_use TEXT,
                when_not_to_use TEXT,
                rearrangements_json TEXT,
                derivation_summary TEXT,
                formula_latex TEXT,
                assumptions_json TEXT,
                common_errors_json TEXT,
                worked_example_ids_json TEXT
            );
            CREATE TABLE concept_meta (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL UNIQUE,
                concept_explanation TEXT NOT NULL,
                intuition_summary TEXT,
                related_visual_keywords TEXT,
                misconception_signals TEXT,
                why_it_matters TEXT,
                mastery_indicators_json TEXT
            );
            CREATE TABLE entry_examples (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL,
                sequence_order INTEGER NOT NULL DEFAULT 0,
                example_text TEXT NOT NULL,
                context_type TEXT,
                difficulty_level INTEGER NOT NULL DEFAULT 0,
                worked_solution_text TEXT,
                is_exam_style INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE entry_misconceptions (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL,
                misconception_text TEXT NOT NULL,
                cause_explanation TEXT,
                correction_explanation TEXT,
                confusion_pair_entry_id INTEGER,
                misconception_source TEXT,
                severity_bp INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE knowledge_relations (
                id INTEGER PRIMARY KEY,
                from_entry_id INTEGER NOT NULL,
                to_entry_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0,
                explanation TEXT
            );
            CREATE TABLE knowledge_bundles (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                bundle_type TEXT NOT NULL,
                subject_id INTEGER,
                topic_id INTEGER,
                description TEXT,
                difficulty_level INTEGER NOT NULL DEFAULT 0,
                estimated_duration INTEGER DEFAULT 0,
                exam_relevance_score INTEGER NOT NULL DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );
            CREATE TABLE knowledge_bundle_items (
                id INTEGER PRIMARY KEY,
                bundle_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                item_role TEXT,
                sequence_order INTEGER NOT NULL DEFAULT 0,
                required INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE student_entry_state (
                user_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                familiarity_state TEXT NOT NULL DEFAULT 'unseen',
                mastery_score INTEGER NOT NULL DEFAULT 0,
                confusion_score INTEGER NOT NULL DEFAULT 0,
                recall_strength INTEGER NOT NULL DEFAULT 0,
                last_viewed_at TEXT,
                last_played_at TEXT,
                last_tested_at TEXT,
                review_due_at TEXT,
                open_count INTEGER NOT NULL DEFAULT 0,
                linked_wrong_answer_count INTEGER NOT NULL DEFAULT 0,
                recognition_score INTEGER NOT NULL DEFAULT 0,
                connection_score INTEGER NOT NULL DEFAULT 0,
                application_score INTEGER NOT NULL DEFAULT 0,
                retention_score INTEGER NOT NULL DEFAULT 0,
                test_count INTEGER NOT NULL DEFAULT 0,
                test_pass_count INTEGER NOT NULL DEFAULT 0,
                spaced_review_due_at TEXT,
                at_risk_threshold_date TEXT,
                UNIQUE(user_id, entry_id)
            );
            CREATE TABLE glossary_entry_mastery (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                mastery_state TEXT NOT NULL DEFAULT 'unseen',
                state_entry_date TEXT NOT NULL DEFAULT (datetime('now')),
                last_advanced_date TEXT,
                consecutive_correct INTEGER NOT NULL DEFAULT 0,
                at_risk_flag INTEGER NOT NULL DEFAULT 0,
                review_count INTEGER NOT NULL DEFAULT 0,
                test_count INTEGER NOT NULL DEFAULT 0,
                test_pass_count INTEGER NOT NULL DEFAULT 0,
                recognition_score_bp INTEGER NOT NULL DEFAULT 0,
                connection_score_bp INTEGER NOT NULL DEFAULT 0,
                application_score_bp INTEGER NOT NULL DEFAULT 0,
                retention_score_bp INTEGER NOT NULL DEFAULT 0,
                spaced_review_due_at TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(student_id, entry_id)
            );
            CREATE TABLE confusion_pairs (
                id INTEGER PRIMARY KEY,
                entry_id_1 INTEGER NOT NULL,
                entry_id_2 INTEGER NOT NULL,
                distinction_explanation TEXT NOT NULL,
                common_confusion_reason TEXT,
                clue_to_distinguish TEXT,
                example_sentence_1 TEXT,
                example_sentence_2 TEXT,
                confusion_recovery_bundle_id INTEGER,
                confusion_frequency_bp INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE neighbor_intruder_mappings (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL,
                neighbor_entry_ids_json TEXT NOT NULL DEFAULT '[]',
                intruder_entry_ids_json TEXT NOT NULL DEFAULT '[]'
            );
            CREATE TABLE questions (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                stem TEXT NOT NULL
            );
            CREATE TABLE question_glossary_links (
                id INTEGER PRIMARY KEY,
                question_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                link_source TEXT NOT NULL,
                link_reason TEXT,
                confidence_score INTEGER NOT NULL DEFAULT 0,
                is_primary INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE entry_audio_segments (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL,
                segment_type TEXT NOT NULL,
                script_text TEXT NOT NULL,
                duration_seconds INTEGER,
                teaching_mode TEXT NOT NULL DEFAULT 'standard'
            );
            CREATE TABLE glossary_search_index (
                id INTEGER PRIMARY KEY,
                entry_id INTEGER NOT NULL UNIQUE,
                title_tokens TEXT,
                alias_tokens TEXT,
                full_text_content TEXT,
                simple_text_content TEXT,
                formula_speech_text TEXT,
                misconception_text TEXT,
                topic_labels TEXT,
                bundle_labels TEXT,
                intent_keywords TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE glossary_interaction_events (
                id INTEGER PRIMARY KEY,
                student_id INTEGER,
                entry_id INTEGER,
                bundle_id INTEGER,
                question_id INTEGER,
                event_type TEXT NOT NULL,
                query_text TEXT,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE glossary_audio_programs (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                source_type TEXT NOT NULL,
                source_id INTEGER,
                teaching_mode TEXT NOT NULL,
                student_id INTEGER,
                total_duration_seconds INTEGER NOT NULL DEFAULT 0,
                item_count INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'ready'
            );
            CREATE TABLE glossary_audio_program_items (
                id INTEGER PRIMARY KEY,
                program_id INTEGER NOT NULL,
                sequence_no INTEGER NOT NULL,
                segment_id INTEGER,
                entry_id INTEGER,
                prompt_text TEXT,
                focus_reason TEXT,
                duration_seconds INTEGER
            );
            CREATE TABLE glossary_audio_queue_state (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL UNIQUE,
                current_program_id INTEGER,
                current_position INTEGER NOT NULL DEFAULT 0,
                is_playing INTEGER NOT NULL DEFAULT 0,
                playback_speed REAL NOT NULL DEFAULT 1.0,
                include_examples INTEGER NOT NULL DEFAULT 1,
                include_misconceptions INTEGER NOT NULL DEFAULT 1,
                last_played_at TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE glossary_test_sessions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                test_mode TEXT NOT NULL,
                topic_id INTEGER,
                bundle_id INTEGER,
                entry_count INTEGER NOT NULL DEFAULT 0,
                duration_seconds INTEGER,
                difficulty_level INTEGER NOT NULL DEFAULT 0,
                recall_score_bp INTEGER,
                recognition_score_bp INTEGER,
                connection_score_bp INTEGER,
                application_score_bp INTEGER,
                retention_score_bp INTEGER,
                confidence_score_bp INTEGER,
                completion_rate_bp INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE glossary_test_items (
                id INTEGER PRIMARY KEY,
                test_session_id INTEGER NOT NULL,
                sequence_no INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                prompt_type TEXT NOT NULL,
                prompt_text TEXT NOT NULL,
                expected_answer TEXT,
                options_json TEXT NOT NULL DEFAULT '[]',
                metadata_json TEXT NOT NULL DEFAULT '{}'
            );
            CREATE TABLE glossary_test_attempts (
                id INTEGER PRIMARY KEY,
                test_session_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                attempt_no INTEGER NOT NULL DEFAULT 1,
                test_mode TEXT NOT NULL,
                student_response TEXT,
                is_correct INTEGER NOT NULL DEFAULT 0,
                time_seconds INTEGER,
                meaning_recall_bp INTEGER,
                word_recognition_bp INTEGER,
                spelling_accuracy_bp INTEGER,
                formula_recall_bp INTEGER,
                concept_recognition_bp INTEGER,
                relationship_understanding_bp INTEGER,
                confusion_resistance_bp INTEGER,
                context_transfer_bp INTEGER
            );
            CREATE TABLE student_topic_states (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                mastery_score INTEGER NOT NULL DEFAULT 0,
                gap_score INTEGER NOT NULL DEFAULT 0,
                decay_risk INTEGER NOT NULL DEFAULT 0,
                trend_state TEXT NOT NULL DEFAULT 'stable',
                is_blocked INTEGER NOT NULL DEFAULT 0,
                next_review_at TEXT
            );
            CREATE TABLE wrong_answer_diagnoses (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                primary_diagnosis TEXT NOT NULL,
                recommended_action TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE contrast_pairs (
                id INTEGER PRIMARY KEY,
                left_entry_id INTEGER NOT NULL,
                right_entry_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                trap_strength INTEGER NOT NULL DEFAULT 0
            );
            ",
        )
        .expect("deep glossary schema");
    }

    fn seed_deep_glossary(conn: &Connection) {
        conn.execute_batch(
            "
            INSERT INTO topics (id, name) VALUES (1, 'Density');
            INSERT INTO knowledge_entries (
                id, subject_id, topic_id, entry_type, title, canonical_name, slug, short_text,
                full_text, simple_text, technical_text, exam_text, importance_score,
                difficulty_level, status, audio_available, has_formula, confusion_pair_count,
                example_count, misconception_count, exam_relevance_score, priority_score,
                phonetic_text
            ) VALUES
                (
                    1, 1, 1, 'definition', 'Density', 'Density', 'density',
                    'Mass per unit volume',
                    'Density measures how much mass is packed into a given volume.',
                    'How tightly matter is packed',
                    'Mass per unit volume of a substance.',
                    'Use density to compare how compact substances are.',
                    9200, 1200, 'active', 1, 0, 1, 1, 1, 9300, 9100, 'den-si-ty'
                ),
                (
                    2, 1, 1, 'formula', 'Density Formula', 'Density Formula', 'density-formula',
                    'density = mass / volume',
                    'The density formula divides mass by volume.',
                    'Take mass and divide by volume.',
                    'rho equals mass over volume.',
                    'The density formula appears in structured word problems.',
                    9000, 1300, 'active', 1, 1, 0, 1, 1, 9500, 9400, NULL
                ),
                (
                    3, 1, 1, 'concept', 'Mass', 'Mass', 'mass',
                    'Amount of matter',
                    'Mass tells you how much matter is present.',
                    'How much matter something has.',
                    'Mass is a measure of quantity of matter.',
                    'Mass is often paired with density and volume.',
                    8400, 1000, 'active', 0, 0, 0, 0, 0, 8600, 8000, NULL
                ),
                (
                    4, 1, 1, 'concept', 'Weight', 'Weight', 'weight',
                    'Force due to gravity',
                    'Weight changes with gravity and is not the same as mass.',
                    'How hard gravity pulls.',
                    'Weight is a force, unlike mass.',
                    'Mass vs weight is a classic confusion pair.',
                    8200, 1100, 'active', 0, 0, 1, 0, 1, 8500, 7800, NULL
                );
            INSERT INTO entry_aliases (id, entry_id, alias_text, alias_type) VALUES
                (1, 1, 'compactness', 'synonym'),
                (2, 2, 'rho equals m over v', 'speech');
            INSERT INTO entry_content_blocks (id, entry_id, block_type, order_index, content_json) VALUES
                (1, 1, 'warning', 1, '{\"title\":\"Mass is not weight\"}');
            INSERT INTO definition_meta (
                id, entry_id, definition_text, short_definition, real_world_meaning,
                non_examples, context_clues, pronunciation_text, formal_definition
            ) VALUES (
                1, 1, 'Mass per unit volume', 'Mass per unit volume',
                'How compact something is', 'weight', 'packed into a given volume',
                'den-si-ty', 'Density is the mass per unit volume of a substance'
            );
            INSERT INTO formula_meta (
                id, entry_id, formula_expression, formula_speech, variables_json, units_json,
                when_to_use, when_not_to_use, rearrangements_json, derivation_summary,
                formula_latex, assumptions_json, common_errors_json, worked_example_ids_json
            ) VALUES (
                1, 2, 'density = mass / volume', 'density equals mass divided by volume',
                '{\"mass\":\"m\",\"volume\":\"v\"}', '{\"density\":\"kg/m3\"}',
                'When mass and volume are known', 'When weight is given instead of mass',
                '[\"mass = density * volume\"]', 'Density compares compactness.',
                '\\\\rho = \\\\frac{m}{V}', '[\"uniform substance\"]',
                '[\"using weight instead of mass\"]', '[1]'
            );
            INSERT INTO concept_meta (
                id, entry_id, concept_explanation, intuition_summary,
                related_visual_keywords, misconception_signals, why_it_matters, mastery_indicators_json
            ) VALUES (
                1, 1, 'Density links mass and volume into one idea.',
                'Think packed tightly versus loosely.',
                'particles,compact,volume', 'same mass,same volume,packed', 'It helps interpret many material questions.',
                '[\"can compare substances\",\"can choose the right formula\"]'
            );
            INSERT INTO entry_examples (
                id, entry_id, sequence_order, example_text, context_type, difficulty_level, worked_solution_text, is_exam_style
            ) VALUES (
                1, 1, 1, 'A metal block with more mass in the same space has greater density.',
                'exam', 1200, 'Compare mass in equal volumes.', 1
            );
            INSERT INTO entry_examples (
                id, entry_id, sequence_order, example_text, context_type, difficulty_level, worked_solution_text, is_exam_style
            ) VALUES (
                2, 2, 1, 'A substance with mass 10 g and volume 2 cm3 has density 5 g/cm3.',
                'worked', 1300, '10 divided by 2 gives 5.', 1
            );
            INSERT INTO entry_misconceptions (
                id, entry_id, misconception_text, cause_explanation, correction_explanation,
                confusion_pair_entry_id, misconception_source, severity_bp
            ) VALUES (
                1, 1, 'Confusing density with weight', 'Weight feels more familiar',
                'Density depends on mass and volume, not gravity.', 4, 'curated', 8800
            );
            INSERT INTO knowledge_relations (id, from_entry_id, to_entry_id, relation_type, strength_score, explanation) VALUES
                (1, 1, 2, 'expressed_by_formula', 9300, 'Density is expressed by the density formula.'),
                (2, 1, 3, 'depends_on', 9000, 'You need mass to compute density.'),
                (3, 1, 4, 'confused_with', 8600, 'Students often confuse density with weight.');
            INSERT INTO knowledge_bundles (
                id, title, bundle_type, subject_id, topic_id, description, difficulty_level, estimated_duration, exam_relevance_score
            ) VALUES (
                1, 'Density Core Pack', 'core bundle', 1, 1, 'Definition, formula, and confusion repair for density.',
                1200, 8, 9400
            );
            INSERT INTO knowledge_bundle_items (id, bundle_id, entry_id, item_role, sequence_order, required) VALUES
                (1, 1, 1, 'anchor', 0, 1),
                (2, 1, 2, 'formula', 1, 1),
                (3, 1, 4, 'confusion', 2, 1);
            INSERT INTO student_entry_state (
                user_id, entry_id, familiarity_state, mastery_score, confusion_score, recall_strength,
                review_due_at, open_count, linked_wrong_answer_count, recognition_score,
                connection_score, application_score, retention_score
            ) VALUES
                (42, 1, 'seen', 3200, 7200, 1800, '2026-03-30T09:00:00Z', 2, 1, 1200, 1100, 900, 800),
                (42, 2, 'seen', 3400, 5100, 2200, NULL, 1, 1, 1500, 1200, 1000, 900);
            INSERT INTO glossary_entry_mastery (student_id, entry_id, mastery_state, at_risk_flag) VALUES
                (42, 1, 'at_risk', 1),
                (42, 2, 'understood', 0);
            INSERT INTO confusion_pairs (
                id, entry_id_1, entry_id_2, distinction_explanation, common_confusion_reason,
                clue_to_distinguish, example_sentence_1, example_sentence_2, confusion_frequency_bp
            ) VALUES (
                1, 1, 4, 'Density is mass per volume, while weight is gravitational force.',
                'Both are discussed in material questions',
                'Ask whether gravity matters.',
                'Oil is less dense than water.', 'Your weight changes on the Moon.', 9100
            );
            INSERT INTO neighbor_intruder_mappings (id, entry_id, neighbor_entry_ids_json, intruder_entry_ids_json) VALUES
                (1, 1, '[2,3]', '[4]');
            INSERT INTO questions (id, topic_id, stem) VALUES
                (10, 1, 'A block has mass 10 g and volume 2 cm3. Find its density.');
            INSERT INTO question_glossary_links (
                id, question_id, entry_id, relation_type, link_source, link_reason, confidence_score, is_primary
            ) VALUES
                (1, 10, 1, 'definition_support', 'manual', 'The question asks for density.', 9300, 1),
                (2, 10, 2, 'formula_support', 'manual', 'Use the density formula.', 9500, 1);
            INSERT INTO entry_audio_segments (id, entry_id, segment_type, script_text, duration_seconds, teaching_mode) VALUES
                (1, 1, 'definition', 'Density is mass per unit volume.', 12, 'standard'),
                (2, 1, 'misconception', 'Do not confuse density with weight.', 10, 'standard');
            INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, decay_risk, trend_state, is_blocked, next_review_at
            ) VALUES (42, 1, 3600, 7000, 6400, 'declining', 0, '2026-03-30T08:00:00Z');
            INSERT INTO wrong_answer_diagnoses (
                id, student_id, topic_id, primary_diagnosis, recommended_action, created_at
            ) VALUES (1, 42, 1, 'unit confusion', 'Re-state what density compares before computing.', '2026-03-29T11:00:00Z');
            INSERT INTO contrast_pairs (id, left_entry_id, right_entry_id, title, trap_strength) VALUES
                (1, 1, 4, 'Density vs weight', 8700);
            ",
        )
        .expect("seed deep glossary");
    }

    #[test]
    fn deep_glossary_entry_detail_and_grouped_search_work() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_deep_glossary_schema(&conn);
        seed_deep_glossary(&conn);

        let service = GlossaryService::new(&conn);
        let detail = service
            .get_entry_detail(Some(42), 1, 10, 10)
            .expect("entry detail");
        assert_eq!(detail.entry.title, "Density");
        assert!(!detail.aliases.is_empty());
        assert!(detail.definition_meta.is_some());
        assert!(!detail.examples.is_empty());
        assert!(!detail.misconceptions.is_empty());
        assert!(!detail.relations.is_empty());
        assert!(!detail.bundles.is_empty());
        assert!(!detail.linked_questions.is_empty());
        assert!(!detail.audio_segments.is_empty());
        assert!(detail.student_state.is_some());
        assert!(!detail.confusion_pairs.is_empty());

        let rebuilt = service.rebuild_search_index().expect("rebuild index");
        assert!(rebuilt >= 4);

        let response = service
            .search_catalog(
                &GlossarySearchInput {
                    query: "density".to_string(),
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
            .expect("grouped search");
        assert!(!response.groups.is_empty());
        assert!(
            response
                .groups
                .iter()
                .any(|group| group.group_key == "best_match")
        );
        assert!(
            response
                .groups
                .iter()
                .any(|group| group.group_key == "definitions")
        );
        assert!(
            response
                .groups
                .iter()
                .any(|group| group.group_key == "bundles")
        );
    }

    #[test]
    fn deep_glossary_queue_and_test_session_work() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        create_deep_glossary_schema(&conn);
        seed_deep_glossary(&conn);

        let service = GlossaryService::new(&conn);
        let queue = service
            .start_audio_queue(
                42,
                &StartGlossaryAudioQueueInput {
                    source_type: "entry".to_string(),
                    source_id: 1,
                    limit: 3,
                    teaching_mode: Some("standard".to_string()),
                    include_examples: true,
                    include_misconceptions: true,
                },
            )
            .expect("start queue");
        assert!(queue.current_program_id.is_some());
        assert!(queue.current_segment.is_some());

        let advanced = service.next_audio_queue(42).expect("advance queue");
        assert!(advanced.current_position >= 0);

        let session = service
            .create_glossary_test_session(
                42,
                &CreateGlossaryTestInput {
                    test_mode: "reverse_recall".to_string(),
                    topic_id: Some(1),
                    bundle_id: None,
                    entry_ids: vec![1],
                    entry_count: 1,
                    duration_seconds: Some(60),
                    difficulty_level: Some(1200),
                },
            )
            .expect("create glossary test");
        assert_eq!(session.items.len(), 1);

        let result = service
            .submit_glossary_test_attempt(
                42,
                session.session_id,
                &SubmitGlossaryTestAttemptInput {
                    entry_id: 1,
                    student_response: "Density".to_string(),
                    time_seconds: Some(8),
                },
            )
            .expect("submit attempt");
        assert!(result.is_correct);
        assert!(result.mastery_score > 3200);

        let refreshed = service
            .get_glossary_test_session(session.session_id)
            .expect("refresh session");
        assert!(refreshed.completion_rate_bp > 0);

        let event_id = service
            .record_interaction(&GlossaryInteractionInput {
                student_id: Some(42),
                entry_id: Some(1),
                bundle_id: Some(1),
                question_id: Some(10),
                event_type: "opened_entry".to_string(),
                query_text: Some("what is density".to_string()),
                metadata: json!({"source":"test"}),
            })
            .expect("record interaction");
        assert!(event_id > 0);
    }
}
