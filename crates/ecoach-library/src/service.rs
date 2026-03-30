use std::collections::HashSet;

use chrono::{DateTime, NaiveDateTime, Utc};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

use crate::models::{
    ContinueLearningCard, GeneratedLibraryShelf, LibraryHomeSnapshot, LibraryItem,
    LibraryShelfItem, RevisionPackItem, RevisionPackSummary, SaveLibraryItemInput,
    SavedQuestionCard, TeachActionPlan, TopicRelationshipHint,
};

pub struct LibraryService<'a> {
    conn: &'a Connection,
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
        let generated_shelves = self.build_generated_shelves(student_id, limit)?;

        Ok(LibraryHomeSnapshot {
            due_now_count: pending_review_count + fading_concept_count + untouched_saved_count,
            pending_review_count,
            fading_concept_count,
            untouched_saved_count,
            continue_card: self.get_continue_learning_card(student_id)?,
            generated_shelves,
            saved_questions: self.list_saved_questions(student_id, limit)?,
        })
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

        let (mastery_score, gap_score): (i64, i64) = self
            .conn
            .query_row(
                "SELECT mastery_score, gap_score
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((0, 10_000));

        let fragile_memory_count = self.count_scalar(
            "SELECT COUNT(*)
             FROM memory_states
             WHERE student_id = ?1
               AND topic_id = ?2
               AND memory_state IN ('fragile', 'at_risk', 'fading', 'collapsed', 'rebuilding')",
            params![student_id, topic_id],
        )?;

        let action_type = if gap_score >= 6500 {
            "reteach_foundation"
        } else if fragile_memory_count > 0 {
            "memory_reactivation"
        } else if mastery_score < 5500 {
            "guided_practice"
        } else {
            "exam_linking"
        }
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
        let primary_prompt = self.build_teach_prompt(
            &topic_name,
            &action_type,
            mastery_score,
            gap_score,
            fragile_memory_count,
            &linked_entry_titles,
            &target_node_titles,
        );

        Ok(TeachActionPlan {
            student_id,
            topic_id,
            topic_name,
            action_type,
            primary_prompt,
            linked_question_ids,
            linked_entry_ids,
            linked_entry_titles,
            target_node_titles,
            relationship_hints,
        })
    }

    pub fn list_topic_relationship_hints(
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
                    COALESCE(an_to.canonical_title, t_to.name)
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
                Ok(TopicRelationshipHint {
                    relation_type: row.get(0)?,
                    from_title: row.get(1)?,
                    to_title: row.get(2)?,
                    explanation: String::new(),
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
            if !hints.iter().any(|existing: &TopicRelationshipHint| {
                existing.relation_type == hint.relation_type
                    && existing.from_title == hint.from_title
                    && existing.to_title == hint.to_title
            }) {
                hints.push(hint);
            }
        }

        if hints.len() < limit {
            let remaining = limit.saturating_sub(hints.len());
            let mut knowledge_stmt = self
                .conn
                .prepare(
                    "SELECT
                        kr.relation_type,
                        from_ke.title,
                        to_ke.title
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
                    Ok(TopicRelationshipHint {
                        relation_type: row.get(0)?,
                        from_title: row.get(1)?,
                        to_title: row.get(2)?,
                        explanation: String::new(),
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
                if !hints.iter().any(|existing: &TopicRelationshipHint| {
                    existing.relation_type == hint.relation_type
                        && existing.from_title == hint.from_title
                        && existing.to_title == hint.to_title
                }) {
                    hints.push(hint);
                }
                if hints.len() >= limit {
                    break;
                }
            }
        }

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
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
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
