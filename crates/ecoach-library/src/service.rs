use std::collections::HashSet;

use chrono::{DateTime, NaiveDateTime, Utc};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    ContinueLearningCard, GeneratedLibraryShelf, LearningPathStep, LibraryHomeSnapshot,
    LibraryItem, LibraryShelfItem, PersonalizedLearningPath, RevisionPackItem, RevisionPackSummary,
    SaveLibraryItemInput, SavedQuestionCard, TeachActionPlan, TeachActionStep,
    TopicRelationshipHint,
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
            },
        )
    }

    pub fn save_item_with_metadata(
        &self,
        student_id: i64,
        input: &SaveLibraryItemInput,
    ) -> EcoachResult<i64> {
        let tags_json = serde_json::to_string(&input.tags)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO library_items (
                    student_id, item_type, item_ref_id, state, tags_json, note_text, topic_id, urgency_score
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    student_id,
                    input.item_type,
                    input.item_ref_id,
                    input.state,
                    tags_json,
                    input.note_text,
                    input.topic_id,
                    input.urgency_score,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_item_metadata(
        &self,
        library_item_id: i64,
        state: &str,
        tags: &[String],
        note_text: Option<&str>,
        urgency_score: BasisPoints,
    ) -> EcoachResult<()> {
        let tags_json = serde_json::to_string(tags)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "UPDATE library_items
                 SET state = ?1,
                     tags_json = ?2,
                     note_text = ?3,
                     urgency_score = ?4,
                     updated_at = datetime('now')
                 WHERE id = ?5",
                params![state, tags_json, note_text, urgency_score, library_item_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
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
                    urgency_score
                 FROM library_items
                 WHERE student_id = ?1
                 ORDER BY urgency_score DESC, updated_at DESC, id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut rows = statement
            .query([student_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            let tags_json: String = row
                .get(5)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            items.push(LibraryItem {
                id: row
                    .get(0)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                student_id: row
                    .get(1)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                item_type: row
                    .get(2)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                item_ref_id: row
                    .get(3)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                state: row
                    .get(4)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                tags: parse_tags(&tags_json)?,
                note_text: row
                    .get(6)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                topic_id: row
                    .get(7)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                urgency_score: row
                    .get(8)
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
            });
        }
        Ok(items)
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
            self.conn
                .execute(
                    "INSERT INTO library_shelves (student_id, shelf_type, title, generated)
                     VALUES (?1, ?2, ?3, 1)",
                    params![student_id, shelf.shelf_type, shelf.title],
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
        let weak_topics = self.list_weak_topic_shelf_items(student_id, 3)?;
        let topic_ids: Vec<i64> = weak_topics
            .iter()
            .filter_map(|item| item.item_ref_id)
            .collect();
        if topic_ids.is_empty() {
            return Err(EcoachError::Validation(
                "cannot build a revision pack without weak-topic evidence".to_string(),
            ));
        }

        let mut selected_question_ids = Vec::new();
        let mut seen_question_ids = HashSet::new();

        for topic_id in &topic_ids {
            let saved_ids =
                self.list_saved_question_ids_for_topic(student_id, *topic_id, question_limit)?;
            for question_id in saved_ids {
                if seen_question_ids.insert(question_id) {
                    selected_question_ids.push(question_id);
                    if selected_question_ids.len() >= question_limit {
                        break;
                    }
                }
            }
            if selected_question_ids.len() >= question_limit {
                break;
            }
        }

        if selected_question_ids.len() < question_limit {
            for topic_id in &topic_ids {
                let remaining = question_limit - selected_question_ids.len();
                let candidate_ids =
                    self.list_active_question_ids_for_topic(*topic_id, remaining)?;
                for question_id in candidate_ids {
                    if seen_question_ids.insert(question_id) {
                        selected_question_ids.push(question_id);
                        if selected_question_ids.len() >= question_limit {
                            break;
                        }
                    }
                }
                if selected_question_ids.len() >= question_limit {
                    break;
                }
            }
        }

        if selected_question_ids.is_empty() {
            return Err(EcoachError::Validation(
                "no questions were available to build the revision pack".to_string(),
            ));
        }

        let metadata = json!({
            "pack_strategy": "weak_topic_recovery",
            "topic_ids": topic_ids,
            "question_count": selected_question_ids.len(),
        });
        self.conn
            .execute(
                "INSERT INTO revision_packs (student_id, title, source_type, metadata_json)
                 VALUES (?1, ?2, 'generated_weak_topic', ?3)",
                params![
                    student_id,
                    title,
                    serde_json::to_string(&metadata)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let pack_id = self.conn.last_insert_rowid();

        for (index, question_id) in selected_question_ids.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO revision_pack_items (
                        pack_id, item_type, item_ref_id, sequence_order, required, metadata_json
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
            source_type: Some("generated_weak_topic".to_string()),
            topic_ids,
            question_count: selected_question_ids.len() as i64,
            created_at: parse_datetime(&created_at_raw)?,
        })
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

        let memory_shelf = GeneratedLibraryShelf {
            shelf_id: None,
            shelf_type: "memory_shelf".to_string(),
            title: "Memory Shelf".to_string(),
            generated: true,
            items: self.list_memory_shelf_items(student_id, limit)?,
        };
        if !memory_shelf.items.is_empty() {
            shelves.push(memory_shelf);
        }

        let mistake_bank = GeneratedLibraryShelf {
            shelf_id: None,
            shelf_type: "mistake_bank".to_string(),
            title: "Mistake Bank".to_string(),
            generated: true,
            items: self.list_mistake_bank_items(student_id, limit)?,
        };
        if !mistake_bank.items.is_empty() {
            shelves.push(mistake_bank);
        }

        let weak_topics = GeneratedLibraryShelf {
            shelf_id: None,
            shelf_type: "weak_topics".to_string(),
            title: "Weak Topics".to_string(),
            generated: true,
            items: self.list_weak_topic_shelf_items(student_id, limit)?,
        };
        if !weak_topics.items.is_empty() {
            shelves.push(weak_topics);
        }

        let saved_questions = GeneratedLibraryShelf {
            shelf_id: None,
            shelf_type: "saved_questions".to_string(),
            title: "Saved Questions".to_string(),
            generated: true,
            items: self
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
                    }),
                })
                .collect(),
        };
        if !saved_questions.items.is_empty() {
            shelves.push(saved_questions);
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

        Ok(GeneratedLibraryShelf {
            shelf_id: None,
            shelf_type: "due_now".to_string(),
            title: "Due Now".to_string(),
            generated: true,
            items,
        })
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
}

fn parse_tags(raw: &str) -> EcoachResult<Vec<String>> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_value(raw: &str) -> EcoachResult<Value> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
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

    use super::LibraryService;

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
}
