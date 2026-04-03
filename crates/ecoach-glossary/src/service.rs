use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, params};

use crate::models::{
    GlossaryAudioProgram, GlossaryAudioSegment, KnowledgeBundle, KnowledgeBundleSequenceItem,
    KnowledgeEntry, QuestionKnowledgeLink,
};

pub struct GlossaryService<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone)]
struct AudioEntrySource {
    id: i64,
    title: String,
    entry_type: String,
    topic_id: Option<i64>,
    narrative_text: String,
    focus_reason: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct AudioProgramContext {
    teaching_mode: String,
    listener_signals: Vec<String>,
    contrast_titles: Vec<String>,
    recommended_bundles: Vec<KnowledgeBundleSequenceItem>,
    review_entry_ids: Vec<i64>,
    review_entry_titles: Vec<String>,
    relationship_review_prompts: Vec<String>,
}

impl<'a> GlossaryService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn search_entries(&self, query: &str) -> EcoachResult<Vec<KnowledgeEntry>> {
        let like = format!("%{}%", query);
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, entry_type, short_text, topic_id
             FROM knowledge_entries
             WHERE title LIKE ?1 OR short_text LIKE ?1 OR simple_text LIKE ?1
             ORDER BY importance_score DESC, title ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([like], |row| {
                Ok(KnowledgeEntry {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    entry_type: row.get(2)?,
                    short_text: row.get(3)?,
                    topic_id: row.get(4)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(out)
    }

    pub fn create_entry(
        &self,
        title: &str,
        entry_type: &str,
        short_text: Option<&str>,
        topic_id: Option<i64>,
    ) -> EcoachResult<i64> {
        self.conn.execute(
            "INSERT INTO knowledge_entries (title, entry_type, short_text, topic_id) VALUES (?1, ?2, ?3, ?4)",
            params![title, entry_type, short_text, topic_id],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn create_bundle(
        &self,
        title: &str,
        bundle_type: &str,
        topic_id: Option<i64>,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO knowledge_bundles (title, bundle_type, topic_id) VALUES (?1, ?2, ?3)",
                params![title, bundle_type, topic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_bundles_for_topic(&self, topic_id: i64) -> EcoachResult<Vec<KnowledgeBundle>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, bundle_type, topic_id FROM knowledge_bundles WHERE topic_id = ?1 ORDER BY title ASC"
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([topic_id], |row| {
                Ok(KnowledgeBundle {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    bundle_type: row.get(2)?,
                    topic_id: row.get(3)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(out)
    }

    pub fn list_bundle_sequence_for_topic(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<KnowledgeBundleSequenceItem>> {
        self.load_bundle_sequence_for_topic(student_id, topic_id, limit)
    }

    pub fn link_question_entry(
        &self,
        question_id: i64,
        entry_id: i64,
        relation_type: &str,
        link_source: &str,
        link_reason: Option<&str>,
        confidence_score: i64,
        is_primary: bool,
    ) -> EcoachResult<()> {
        self.conn.execute(
            "INSERT INTO question_glossary_links (
                question_id, entry_id, relation_type, link_source, link_reason, confidence_score, is_primary
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(question_id, entry_id) DO UPDATE SET
                relation_type = excluded.relation_type,
                link_source = excluded.link_source,
                link_reason = excluded.link_reason,
                confidence_score = excluded.confidence_score,
                is_primary = excluded.is_primary,
                updated_at = datetime('now')",
            params![
                question_id,
                entry_id,
                relation_type,
                link_source,
                link_reason,
                confidence_score,
                is_primary as i64,
            ],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn list_entries_for_question(
        &self,
        question_id: i64,
    ) -> EcoachResult<Vec<QuestionKnowledgeLink>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                qgl.question_id,
                ke.id,
                ke.title,
                ke.entry_type,
                ke.short_text,
                ke.topic_id,
                qgl.relation_type,
                qgl.link_source,
                qgl.link_reason,
                qgl.confidence_score,
                qgl.is_primary
             FROM question_glossary_links qgl
             INNER JOIN knowledge_entries ke ON ke.id = qgl.entry_id
             WHERE qgl.question_id = ?1
             ORDER BY qgl.is_primary DESC, qgl.confidence_score DESC, ke.importance_score DESC, ke.title ASC"
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map([question_id], |row| {
                Ok(QuestionKnowledgeLink {
                    question_id: row.get(0)?,
                    entry_id: row.get(1)?,
                    title: row.get(2)?,
                    entry_type: row.get(3)?,
                    short_text: row.get(4)?,
                    topic_id: row.get(5)?,
                    relation_type: row.get(6)?,
                    link_source: row.get(7)?,
                    link_reason: row.get(8)?,
                    confidence_score: row.get(9)?,
                    is_primary: row.get::<_, i64>(10)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(out)
    }

    pub fn build_audio_program_for_topic(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<GlossaryAudioProgram> {
        let topic_name: String = self
            .conn
            .query_row(
                "SELECT name FROM topics WHERE id = ?1 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let bundles = self.list_bundles_for_topic(topic_id)?;
        let entry_sources = self.load_audio_sources_for_topic(topic_id, limit)?;
        if entry_sources.is_empty() {
            return Err(EcoachError::Validation(format!(
                "no glossary entries are available to build an audio program for topic {}",
                topic_id
            )));
        }

        Ok(self.compose_audio_program(
            format!("{} audio coach", topic_name),
            "topic",
            Some(topic_id),
            None,
            &bundles,
            &entry_sources,
            &AudioProgramContext::default(),
        ))
    }

    pub fn build_personalized_audio_program_for_topic(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<GlossaryAudioProgram> {
        let topic_name: String = self
            .conn
            .query_row(
                "SELECT name FROM topics WHERE id = ?1 LIMIT 1",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let bundles = self.list_bundles_for_topic(topic_id)?;
        let entry_sources =
            self.load_personalized_audio_sources_for_topic(student_id, topic_id, limit)?;
        if entry_sources.is_empty() {
            return Err(EcoachError::Validation(format!(
                "no glossary entries are available to build a personalized audio program for topic {}",
                topic_id
            )));
        }
        let context = self.build_audio_program_context(student_id, Some(topic_id))?;

        Ok(self.compose_audio_program(
            format!("{} guided audio coach", topic_name),
            "topic_personalized",
            Some(topic_id),
            None,
            &bundles,
            &entry_sources,
            &context,
        ))
    }

    pub fn build_audio_program_for_question(
        &self,
        question_id: i64,
        limit: usize,
    ) -> EcoachResult<GlossaryAudioProgram> {
        let question_stem: String = self
            .conn
            .query_row(
                "SELECT stem FROM questions WHERE id = ?1 LIMIT 1",
                [question_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let links = self.list_entries_for_question(question_id)?;
        if links.is_empty() {
            return Err(EcoachError::Validation(format!(
                "question {} has no linked glossary entries",
                question_id
            )));
        }

        let mut entry_sources = Vec::new();
        for link in links.into_iter().take(limit.max(1)) {
            entry_sources.push(self.load_audio_source(link.entry_id)?);
        }
        let topic_id = entry_sources.iter().find_map(|entry| entry.topic_id);
        let bundles = if let Some(topic_id) = topic_id {
            self.list_bundles_for_topic(topic_id)?
        } else {
            Vec::new()
        };

        Ok(self.compose_audio_program(
            format!(
                "Question repair audio: {}",
                truncate_title(&question_stem, 56)
            ),
            "question_repair",
            topic_id,
            Some(question_id),
            &bundles,
            &entry_sources,
            &AudioProgramContext::default(),
        ))
    }

    pub fn build_personalized_audio_program_for_question(
        &self,
        student_id: i64,
        question_id: i64,
        limit: usize,
    ) -> EcoachResult<GlossaryAudioProgram> {
        let (question_stem, question_topic_id): (String, Option<i64>) = self
            .conn
            .query_row(
                "SELECT stem, topic_id FROM questions WHERE id = ?1 LIMIT 1",
                [question_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let entry_sources =
            self.load_personalized_audio_sources_for_question(student_id, question_id, limit)?;
        if entry_sources.is_empty() {
            return Err(EcoachError::Validation(format!(
                "question {} has no linked glossary entries",
                question_id
            )));
        }
        let topic_id = entry_sources
            .iter()
            .find_map(|entry| entry.topic_id)
            .or(question_topic_id);
        let bundles = if let Some(topic_id) = topic_id {
            self.list_bundles_for_topic(topic_id)?
        } else {
            Vec::new()
        };
        let context = self.build_audio_program_context(student_id, topic_id)?;

        Ok(self.compose_audio_program(
            format!(
                "Question repair audio: {}",
                truncate_title(&question_stem, 56)
            ),
            "question_repair_personalized",
            topic_id,
            Some(question_id),
            &bundles,
            &entry_sources,
            &context,
        ))
    }

    fn compose_audio_program(
        &self,
        program_title: String,
        source_type: &str,
        topic_id: Option<i64>,
        question_id: Option<i64>,
        bundles: &[KnowledgeBundle],
        entry_sources: &[AudioEntrySource],
        context: &AudioProgramContext,
    ) -> GlossaryAudioProgram {
        let mut segments = Vec::new();
        let intro_prompt = context
            .listener_signals
            .first()
            .cloned()
            .or_else(|| {
                entry_sources.first().map(|entry| {
                    format!(
                        "Listen for the exact meaning of {}, then compare it against how it appears in the problem.",
                        entry.title
                    )
                })
            })
            .unwrap_or_else(|| "Listen for the core idea and recall where it applies.".to_string());
        segments.push(GlossaryAudioSegment {
            sequence_no: 1,
            segment_type: "intro".to_string(),
            title: "Orientation".to_string(),
            script_text: if context.teaching_mode.is_empty() {
                format!(
                    "{}. This audio program focuses on the key ideas you should be able to recognize, explain, and recall under exam pressure.",
                    program_title
                )
            } else {
                format!(
                    "{}. This is a {} audio program focused on the ideas you need to recognize, explain, and recall under exam pressure.",
                    program_title,
                    context.teaching_mode.replace('_', " ")
                )
            },
            entry_id: None,
            prompt_text: Some(intro_prompt),
            focus_reason: None,
            duration_seconds: 20,
        });

        if !context.listener_signals.is_empty() {
            let script_text = context.listener_signals.join(" ");
            segments.push(GlossaryAudioSegment {
                sequence_no: (segments.len() + 1) as i64,
                segment_type: "learner_focus".to_string(),
                title: "Why This Matters Right Now".to_string(),
                script_text: script_text.clone(),
                entry_id: None,
                prompt_text: Some(
                    "Pause and name the risk, gap, or recall problem you are trying to repair."
                        .to_string(),
                ),
                focus_reason: Some("Derived from current learner signals.".to_string()),
                duration_seconds: estimate_duration_seconds(&script_text),
            });
        }

        if !context.contrast_titles.is_empty() {
            let script_text = format!(
                "Keep these contrasts clean while you listen: {}. Say the boundary between them before you continue.",
                context.contrast_titles.join("; ")
            );
            segments.push(GlossaryAudioSegment {
                sequence_no: (segments.len() + 1) as i64,
                segment_type: "contrast_trap".to_string(),
                title: "Contrast and Trap Check".to_string(),
                script_text: script_text.clone(),
                entry_id: None,
                prompt_text: Some(
                    "Pause and explain how these ideas differ before returning to the problem."
                        .to_string(),
                ),
                focus_reason: Some(
                    "Derived from contrast and confusion relationships.".to_string(),
                ),
                duration_seconds: estimate_duration_seconds(&script_text),
            });
        }

        if !context.recommended_bundles.is_empty() {
            let bundle_script = context
                .recommended_bundles
                .iter()
                .map(|bundle| format!("{}: {}", bundle.title, bundle.focus_reason))
                .collect::<Vec<_>>()
                .join(" ");
            segments.push(GlossaryAudioSegment {
                sequence_no: (segments.len() + 1) as i64,
                segment_type: "bundle_route".to_string(),
                title: "Bundle Route".to_string(),
                script_text: bundle_script.clone(),
                entry_id: None,
                prompt_text: Some(
                    "Pause and name the bundle or explanation set you should return to first."
                        .to_string(),
                ),
                focus_reason: Some(
                    "Derived from bundle sequencing and learner entry state.".to_string(),
                ),
                duration_seconds: estimate_duration_seconds(&bundle_script),
            });
        }

        if !context.review_entry_titles.is_empty() {
            let review_script = format!(
                "Return to these glossary anchors before practice: {}.",
                context.review_entry_titles.join(", ")
            );
            segments.push(GlossaryAudioSegment {
                sequence_no: (segments.len() + 1) as i64,
                segment_type: "review_loop".to_string(),
                title: "Review Loop".to_string(),
                script_text: review_script.clone(),
                entry_id: None,
                prompt_text: Some(
                    "Pause and retrieve each anchor from memory before you continue.".to_string(),
                ),
                focus_reason: Some(
                    "Derived from review-due and weak-recall glossary entries.".to_string(),
                ),
                duration_seconds: estimate_duration_seconds(&review_script),
            });
        }

        if !context.relationship_review_prompts.is_empty() {
            let relationship_script = context.relationship_review_prompts.join(" ");
            segments.push(GlossaryAudioSegment {
                sequence_no: (segments.len() + 1) as i64,
                segment_type: "relationship_review".to_string(),
                title: "Relationship Review".to_string(),
                script_text: relationship_script.clone(),
                entry_id: None,
                prompt_text: Some(
                    "Pause and say the relationship or contrast before solving.".to_string(),
                ),
                focus_reason: Some(
                    "Derived from glossary relationships and contrast pairs.".to_string(),
                ),
                duration_seconds: estimate_duration_seconds(&relationship_script),
            });
        }

        for entry in entry_sources {
            let segment_type = match entry.entry_type.as_str() {
                "definition" => "definition",
                "formula" => "formula",
                "worked_example" => "worked_example",
                _ => "explanation",
            };
            let base_prompt = match entry.entry_type.as_str() {
                "definition" => Some(format!(
                    "Pause and restate the meaning of {} in your own words.",
                    entry.title
                )),
                "formula" => Some(format!(
                    "Pause and say when the {} formula is safe to use.",
                    entry.title
                )),
                "worked_example" => Some(format!(
                    "Pause and recall the decision points in the {} example.",
                    entry.title
                )),
                _ => Some(format!(
                    "Pause and explain why {} matters before solving.",
                    entry.title
                )),
            };
            let prompt_text = match (&base_prompt, &entry.focus_reason) {
                (Some(prompt), Some(reason)) => Some(format!("{} {}", prompt, reason)),
                (Some(prompt), None) => Some(prompt.clone()),
                (None, Some(reason)) => Some(reason.clone()),
                (None, None) => None,
            };
            segments.push(GlossaryAudioSegment {
                sequence_no: (segments.len() + 1) as i64,
                segment_type: segment_type.to_string(),
                title: entry.title.clone(),
                script_text: condense_script(&entry.narrative_text),
                entry_id: Some(entry.id),
                prompt_text,
                focus_reason: entry.focus_reason.clone(),
                duration_seconds: estimate_duration_seconds(&entry.narrative_text),
            });
        }

        segments.push(GlossaryAudioSegment {
            sequence_no: (segments.len() + 1) as i64,
            segment_type: "recall_check".to_string(),
            title: "Recall Check".to_string(),
            script_text: if context.teaching_mode == "repair" {
                "Now stop the audio and say the main idea, the trap you are repairing, and the safest response pattern before you continue.".to_string()
            } else if context.teaching_mode == "reactivation" {
                "Now stop the audio and retrieve the main idea from memory, then name the safest response pattern before you continue.".to_string()
            } else {
                "Now stop the audio and say the main idea, the usual trap, and the safest response pattern before you continue.".to_string()
            },
            entry_id: None,
            prompt_text: Some("Name the key idea, the trap, and the correct response path.".to_string()),
            focus_reason: None,
            duration_seconds: 18,
        });

        GlossaryAudioProgram {
            program_title,
            source_type: source_type.to_string(),
            teaching_mode: if context.teaching_mode.is_empty() {
                "standard".to_string()
            } else {
                context.teaching_mode.clone()
            },
            topic_id,
            question_id,
            bundle_ids: bundles.iter().map(|bundle| bundle.id).collect(),
            recommended_bundles: context.recommended_bundles.clone(),
            entry_ids: entry_sources.iter().map(|entry| entry.id).collect(),
            listener_signals: context.listener_signals.clone(),
            contrast_titles: context.contrast_titles.clone(),
            review_entry_ids: context.review_entry_ids.clone(),
            review_entry_titles: context.review_entry_titles.clone(),
            relationship_review_prompts: context.relationship_review_prompts.clone(),
            segments,
        }
    }

    fn load_audio_sources_for_topic(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<AudioEntrySource>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    id,
                    title,
                    entry_type,
                    topic_id,
                    COALESCE(exam_text, technical_text, full_text, short_text, simple_text, title)
                 FROM knowledge_entries
                 WHERE topic_id = ?1
                   AND status = 'active'
                 ORDER BY importance_score DESC, difficulty_level ASC, title ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![topic_id, limit.max(1) as i64], |row| {
                Ok(AudioEntrySource {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    entry_type: row.get(2)?,
                    topic_id: row.get(3)?,
                    narrative_text: row.get(4)?,
                    focus_reason: None,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(entries)
    }

    fn load_audio_source(&self, entry_id: i64) -> EcoachResult<AudioEntrySource> {
        self.conn
            .query_row(
                "SELECT
                    id,
                    title,
                    entry_type,
                    topic_id,
                    COALESCE(exam_text, technical_text, full_text, short_text, simple_text, title)
                 FROM knowledge_entries
                 WHERE id = ?1
                 LIMIT 1",
                [entry_id],
                |row| {
                    Ok(AudioEntrySource {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        entry_type: row.get(2)?,
                        topic_id: row.get(3)?,
                        narrative_text: row.get(4)?,
                        focus_reason: None,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_personalized_audio_sources_for_topic(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<AudioEntrySource>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    ke.id,
                    ke.title,
                    ke.entry_type,
                    ke.topic_id,
                    COALESCE(ke.exam_text, ke.technical_text, ke.full_text, ke.short_text, ke.simple_text, ke.title),
                    COALESCE(ses.confusion_score, 0),
                    COALESCE(ses.linked_wrong_answer_count, 0),
                    COALESCE(ses.recall_strength, 0),
                    CASE WHEN ses.review_due_at IS NOT NULL THEN 1 ELSE 0 END
                 FROM knowledge_entries ke
                 LEFT JOIN student_entry_state ses
                    ON ses.entry_id = ke.id
                   AND ses.user_id = ?1
                 WHERE ke.topic_id = ?2
                   AND ke.status = 'active'
                 ORDER BY
                    CASE WHEN ses.review_due_at IS NOT NULL THEN 0 ELSE 1 END,
                    COALESCE(ses.confusion_score, 0) DESC,
                    COALESCE(ses.linked_wrong_answer_count, 0) DESC,
                    COALESCE(ses.recall_strength, 0) ASC,
                    ke.importance_score DESC,
                    ke.difficulty_level ASC,
                    ke.title ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![student_id, topic_id, limit.max(1) as i64], |row| {
                let confusion_score: i64 = row.get(5)?;
                let linked_wrong_answer_count: i64 = row.get(6)?;
                let recall_strength: i64 = row.get(7)?;
                let review_due = row.get::<_, i64>(8)? == 1;
                Ok(AudioEntrySource {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    entry_type: row.get(2)?,
                    topic_id: row.get(3)?,
                    narrative_text: row.get(4)?,
                    focus_reason: audio_focus_reason(
                        confusion_score,
                        linked_wrong_answer_count,
                        recall_strength,
                        review_due,
                        None,
                    ),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(entries)
    }

    fn load_personalized_audio_sources_for_question(
        &self,
        student_id: i64,
        question_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<AudioEntrySource>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    ke.id,
                    ke.title,
                    ke.entry_type,
                    ke.topic_id,
                    COALESCE(ke.exam_text, ke.technical_text, ke.full_text, ke.short_text, ke.simple_text, ke.title),
                    COALESCE(ses.confusion_score, 0),
                    COALESCE(ses.linked_wrong_answer_count, 0),
                    COALESCE(ses.recall_strength, 0),
                    CASE WHEN ses.review_due_at IS NOT NULL THEN 1 ELSE 0 END,
                    qgl.link_reason
                 FROM question_glossary_links qgl
                 INNER JOIN knowledge_entries ke ON ke.id = qgl.entry_id
                 LEFT JOIN student_entry_state ses
                    ON ses.entry_id = ke.id
                   AND ses.user_id = ?1
                 WHERE qgl.question_id = ?2
                 ORDER BY
                    qgl.is_primary DESC,
                    CASE WHEN ses.review_due_at IS NOT NULL THEN 0 ELSE 1 END,
                    COALESCE(ses.confusion_score, 0) DESC,
                    qgl.confidence_score DESC,
                    ke.importance_score DESC,
                    ke.title ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(
                params![student_id, question_id, limit.max(1) as i64],
                |row| {
                    let confusion_score: i64 = row.get(5)?;
                    let linked_wrong_answer_count: i64 = row.get(6)?;
                    let recall_strength: i64 = row.get(7)?;
                    let review_due = row.get::<_, i64>(8)? == 1;
                    let link_reason: Option<String> = row.get(9)?;
                    Ok(AudioEntrySource {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        entry_type: row.get(2)?,
                        topic_id: row.get(3)?,
                        narrative_text: row.get(4)?,
                        focus_reason: audio_focus_reason(
                            confusion_score,
                            linked_wrong_answer_count,
                            recall_strength,
                            review_due,
                            link_reason.as_deref(),
                        ),
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(entries)
    }

    fn load_bundle_sequence_for_topic(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<KnowledgeBundleSequenceItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    kb.id,
                    kb.title,
                    kb.bundle_type,
                    SUM(CASE WHEN ses.review_due_at IS NOT NULL THEN 1 ELSE 0 END) AS due_review_count,
                    MAX(COALESCE(ses.confusion_score, 0)) AS max_confusion_score,
                    SUM(CASE WHEN COALESCE(ses.recall_strength, 10000) <= 3500 THEN 1 ELSE 0 END) AS weak_recall_count
                 FROM knowledge_bundles kb
                 LEFT JOIN knowledge_bundle_items kbi ON kbi.bundle_id = kb.id
                 LEFT JOIN student_entry_state ses
                    ON ses.entry_id = kbi.entry_id
                   AND ses.user_id = ?1
                 WHERE kb.topic_id = ?2
                 GROUP BY kb.id, kb.title, kb.bundle_type, kb.exam_relevance_score, kb.difficulty_level
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
            .query_map(params![student_id, topic_id, limit.max(1) as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut bundles = Vec::new();
        for (index, row) in rows.enumerate() {
            let (
                bundle_id,
                title,
                bundle_type,
                due_review_count,
                max_confusion_score,
                weak_recall_count,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let (focus_entry_ids, focus_entry_titles) =
                self.list_bundle_focus_entries(student_id, bundle_id, 3)?;
            let focus_reason = if due_review_count > 0 {
                format!(
                    "{} review-due entry(s) make {} the first return loop.",
                    due_review_count, title
                )
            } else if max_confusion_score >= 4500 {
                format!(
                    "{} contains the strongest confusion hotspot for this learner.",
                    title
                )
            } else if weak_recall_count > 0 {
                format!(
                    "{} is the cleanest bundle for restoring recall before practice.",
                    title
                )
            } else {
                format!(
                    "{} is the next best structured bundle for this topic.",
                    title
                )
            };

            bundles.push(KnowledgeBundleSequenceItem {
                bundle_id,
                title,
                bundle_type,
                sequence_order: (index + 1) as i64,
                focus_reason,
                due_review_count,
                focus_entry_ids,
                focus_entry_titles,
            });
        }

        Ok(bundles)
    }

    fn list_bundle_focus_entries(
        &self,
        student_id: i64,
        bundle_id: i64,
        limit: usize,
    ) -> EcoachResult<(Vec<i64>, Vec<String>)> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    ke.id,
                    ke.title
                 FROM knowledge_bundle_items kbi
                 INNER JOIN knowledge_entries ke ON ke.id = kbi.entry_id
                 LEFT JOIN student_entry_state ses
                    ON ses.entry_id = ke.id
                   AND ses.user_id = ?1
                 WHERE kbi.bundle_id = ?2
                 ORDER BY
                    CASE WHEN ses.review_due_at IS NOT NULL THEN 0 ELSE 1 END,
                    COALESCE(ses.confusion_score, 0) DESC,
                    COALESCE(ses.linked_wrong_answer_count, 0) DESC,
                    COALESCE(ses.recall_strength, 10000) ASC,
                    kbi.sequence_order ASC,
                    ke.title ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, bundle_id, limit as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut ids = Vec::new();
        let mut titles = Vec::new();
        for row in rows {
            let (entry_id, title) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            ids.push(entry_id);
            titles.push(title);
        }
        Ok((ids, titles))
    }

    fn list_review_entries_for_topic(
        &self,
        student_id: i64,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<(Vec<i64>, Vec<String>)> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    ke.id,
                    ke.title
                 FROM knowledge_entries ke
                 INNER JOIN student_entry_state ses
                    ON ses.entry_id = ke.id
                   AND ses.user_id = ?1
                 WHERE ke.topic_id = ?2
                   AND (
                        ses.review_due_at IS NOT NULL
                        OR ses.confusion_score >= 4500
                        OR ses.linked_wrong_answer_count > 0
                        OR ses.recall_strength <= 3500
                   )
                 ORDER BY
                    CASE WHEN ses.review_due_at IS NOT NULL THEN 0 ELSE 1 END,
                    ses.confusion_score DESC,
                    ses.linked_wrong_answer_count DESC,
                    ses.recall_strength ASC,
                    ke.importance_score DESC,
                    ke.title ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, topic_id, limit as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut ids = Vec::new();
        let mut titles = Vec::new();
        for row in rows {
            let (entry_id, title) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            ids.push(entry_id);
            titles.push(title);
        }
        Ok((ids, titles))
    }

    fn list_relationship_review_prompts(
        &self,
        topic_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<String>> {
        let mut prompts = Vec::new();

        let mut contrast_stmt = self
            .conn
            .prepare(
                "SELECT cp.title
                 FROM contrast_pairs cp
                 INNER JOIN knowledge_entries left_ke ON left_ke.id = cp.left_entry_id
                 INNER JOIN knowledge_entries right_ke ON right_ke.id = cp.right_entry_id
                 WHERE left_ke.topic_id = ?1 OR right_ke.topic_id = ?1
                 ORDER BY cp.trap_strength DESC, cp.id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = contrast_stmt
            .query_map(params![topic_id, limit as i64], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            push_unique_signal(
                &mut prompts,
                format!(
                    "Say the difference inside {} before you move into questions.",
                    row.map_err(|err| EcoachError::Storage(err.to_string()))?
                ),
            );
        }

        if prompts.len() < limit {
            let remaining = limit.saturating_sub(prompts.len());
            let mut relation_stmt = self
                .conn
                .prepare(
                    "SELECT relation_type, from_ke.title, to_ke.title
                     FROM knowledge_relations kr
                     INNER JOIN knowledge_entries from_ke ON from_ke.id = kr.from_entry_id
                     INNER JOIN knowledge_entries to_ke ON to_ke.id = kr.to_entry_id
                     WHERE (from_ke.topic_id = ?1 OR to_ke.topic_id = ?1)
                       AND kr.relation_type IN ('prerequisite', 'dependent', 'confused_with', 'contrasts_with', 'related')
                     ORDER BY kr.strength_score DESC, kr.id ASC
                     LIMIT ?2",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = relation_stmt
                .query_map(params![topic_id, remaining as i64], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                let (relation_type, from_title, to_title) =
                    row.map_err(|err| EcoachError::Storage(err.to_string()))?;
                let prompt = match relation_type.as_str() {
                    "prerequisite" | "dependent" => format!(
                        "Name why {} must be secure before {}.",
                        from_title, to_title
                    ),
                    "confused_with" | "contrasts_with" => format!(
                        "State the clean boundary between {} and {}.",
                        from_title, to_title
                    ),
                    _ => format!("Explain how {} connects to {}.", from_title, to_title),
                };
                push_unique_signal(&mut prompts, prompt);
                if prompts.len() >= limit {
                    break;
                }
            }
        }

        Ok(prompts)
    }

    fn build_audio_program_context(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<AudioProgramContext> {
        let Some(topic_id) = topic_id else {
            return Ok(AudioProgramContext::default());
        };

        let state = self
            .conn
            .query_row(
                "SELECT mastery_score, gap_score, decay_risk, trend_state, is_blocked, next_review_at
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, i64>(4)? == 1,
                        row.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
            .ok();

        let teaching_mode = if let Some((
            mastery_score,
            gap_score,
            decay_risk,
            trend_state,
            is_blocked,
            next_review_at,
        )) = &state
        {
            if *is_blocked || *gap_score >= 6500 {
                "repair".to_string()
            } else if *decay_risk >= 6000 || next_review_at.is_some() {
                "reactivation".to_string()
            } else if *mastery_score < 6500
                || matches!(trend_state.as_str(), "fragile" | "declining" | "critical")
            {
                "guided_practice".to_string()
            } else {
                "exam_bridge".to_string()
            }
        } else {
            "standard".to_string()
        };

        let mut listener_signals = Vec::new();
        if let Some((
            mastery_score,
            gap_score,
            decay_risk,
            trend_state,
            is_blocked,
            next_review_at,
        )) = state
        {
            if is_blocked || gap_score >= 6500 {
                listener_signals.push(
                    "Gap pressure is high here, so start from the exact meaning and rebuild the safest path before chasing speed."
                        .to_string(),
                );
            }
            if decay_risk >= 6000 || next_review_at.is_some() {
                listener_signals.push(
                    "This topic needs recall reactivation, so retrieve the idea from memory before you try fresh questions."
                        .to_string(),
                );
            }
            if matches!(trend_state.as_str(), "fragile" | "declining" | "critical")
                || mastery_score < 5000
            {
                listener_signals.push(
                    "Stay on clear, supported explanations first, then remove support only after the method feels stable."
                        .to_string(),
                );
            }
        }

        let mut stmt = self
            .conn
            .prepare(
                "SELECT primary_diagnosis, recommended_action
                 FROM wrong_answer_diagnoses
                 WHERE student_id = ?1
                   AND topic_id = ?2
                 ORDER BY created_at DESC, id DESC
                 LIMIT 2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![student_id, topic_id], |row| {
                let diagnosis: String = row.get(0)?;
                let action: String = row.get(1)?;
                Ok(format!(
                    "Recent errors point to {}. Keep this response pattern in mind: {}.",
                    diagnosis, action
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            push_unique_signal(
                &mut listener_signals,
                row.map_err(|err| EcoachError::Storage(err.to_string()))?,
            );
        }

        let recommended_bundles = self.load_bundle_sequence_for_topic(student_id, topic_id, 3)?;
        let (review_entry_ids, review_entry_titles) =
            self.list_review_entries_for_topic(student_id, topic_id, 3)?;
        let contrast_titles = self.list_audio_contrast_titles(topic_id, 2)?;
        let relationship_review_prompts = self.list_relationship_review_prompts(topic_id, 2)?;

        Ok(AudioProgramContext {
            teaching_mode,
            listener_signals,
            contrast_titles,
            recommended_bundles,
            review_entry_ids,
            review_entry_titles,
            relationship_review_prompts,
        })
    }

    fn list_audio_contrast_titles(&self, topic_id: i64, limit: usize) -> EcoachResult<Vec<String>> {
        let mut titles = Vec::new();

        let mut contrast_stmt = self
            .conn
            .prepare(
                "SELECT cp.title
                 FROM contrast_pairs cp
                 INNER JOIN knowledge_entries left_ke ON left_ke.id = cp.left_entry_id
                 INNER JOIN knowledge_entries right_ke ON right_ke.id = cp.right_entry_id
                 WHERE left_ke.topic_id = ?1 OR right_ke.topic_id = ?1
                 ORDER BY cp.trap_strength DESC, cp.id ASC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = contrast_stmt
            .query_map(params![topic_id, limit as i64], |row| row.get(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in rows {
            push_unique_signal(
                &mut titles,
                row.map_err(|err| EcoachError::Storage(err.to_string()))?,
            );
        }

        if titles.len() < limit {
            let remaining = limit.saturating_sub(titles.len());
            let mut relation_stmt = self
                .conn
                .prepare(
                    "SELECT from_ke.title, to_ke.title
                     FROM knowledge_relations kr
                     INNER JOIN knowledge_entries from_ke ON from_ke.id = kr.from_entry_id
                     INNER JOIN knowledge_entries to_ke ON to_ke.id = kr.to_entry_id
                     WHERE (from_ke.topic_id = ?1 OR to_ke.topic_id = ?1)
                       AND kr.relation_type IN ('confused_with', 'contrasts_with')
                     ORDER BY kr.strength_score DESC, kr.id ASC
                     LIMIT ?2",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = relation_stmt
                .query_map(params![topic_id, remaining as i64], |row| {
                    let from_title: String = row.get(0)?;
                    let to_title: String = row.get(1)?;
                    Ok(format!("{} vs {}", from_title, to_title))
                })
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in rows {
                push_unique_signal(
                    &mut titles,
                    row.map_err(|err| EcoachError::Storage(err.to_string()))?,
                );
                if titles.len() >= limit {
                    break;
                }
            }
        }

        Ok(titles)
    }
}

fn condense_script(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn estimate_duration_seconds(text: &str) -> i64 {
    let word_count = text.split_whitespace().count() as i64;
    ((word_count * 60) / 140).clamp(12, 75)
}

fn truncate_title(text: &str, max_len: usize) -> String {
    let trimmed = condense_script(text);
    if trimmed.chars().count() <= max_len {
        trimmed
    } else {
        let clipped = trimmed
            .chars()
            .take(max_len.saturating_sub(3))
            .collect::<String>();
        format!("{}...", clipped)
    }
}

fn audio_focus_reason(
    confusion_score: i64,
    linked_wrong_answer_count: i64,
    recall_strength: i64,
    review_due: bool,
    link_reason: Option<&str>,
) -> Option<String> {
    if review_due {
        Some("Review is due here, so retrieve the idea before moving on.".to_string())
    } else if confusion_score >= 4500 {
        Some(
            "This idea shows elevated confusion risk, so listen for the exact boundary."
                .to_string(),
        )
    } else if linked_wrong_answer_count > 0 {
        Some(
            "This idea has been tied to recent wrong answers, so keep the repair cue active."
                .to_string(),
        )
    } else if recall_strength > 0 && recall_strength <= 3500 {
        Some(
            "Recall strength is still low here, so restate the idea before applying it."
                .to_string(),
        )
    } else {
        link_reason.map(|reason| format!("Focus here because {}.", reason.trim_end_matches('.')))
    }
}

fn push_unique_signal(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

mod deep;

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::GlossaryService;

    #[test]
    fn personalized_topic_audio_uses_learner_signals_and_contrasts() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "
            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE knowledge_entries (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                entry_type TEXT NOT NULL,
                title TEXT NOT NULL,
                short_text TEXT,
                full_text TEXT,
                technical_text TEXT,
                exam_text TEXT,
                simple_text TEXT,
                importance_score INTEGER NOT NULL DEFAULT 0,
                difficulty_level INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'active'
            );
            CREATE TABLE knowledge_bundles (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                bundle_type TEXT NOT NULL,
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
            CREATE TABLE knowledge_relations (
                id INTEGER PRIMARY KEY,
                from_entry_id INTEGER NOT NULL,
                to_entry_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                strength_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE questions (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                stem TEXT NOT NULL
            );
            CREATE TABLE question_glossary_links (
                question_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                relation_type TEXT NOT NULL,
                link_source TEXT NOT NULL,
                link_reason TEXT,
                confidence_score INTEGER NOT NULL DEFAULT 0,
                is_primary INTEGER NOT NULL DEFAULT 0
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
            "INSERT INTO knowledge_entries (
                id, topic_id, entry_type, title, short_text, exam_text, importance_score, difficulty_level, status
             ) VALUES (
                101, 1, 'definition', 'Completing the square', 'Convert to a perfect square',
                'Completing the square lets you rewrite a quadratic into a safer solved form.',
                9000, 1500, 'active'
             )",
            [],
        )
        .expect("entry one");
        conn.execute(
            "INSERT INTO knowledge_entries (
                id, topic_id, entry_type, title, short_text, exam_text, importance_score, difficulty_level, status
             ) VALUES (
                102, 1, 'formula', 'Quadratic formula', 'Use the discriminant carefully',
                'Use the quadratic formula when factorisation is unreliable, and check the discriminant before you substitute.',
                8600, 1800, 'active'
             )",
            [],
        )
        .expect("entry two");
        conn.execute(
            "INSERT INTO knowledge_bundles (id, title, bundle_type, topic_id, exam_relevance_score, difficulty_level)
             VALUES (1, 'Quadratic repair pack', 'repair', 1, 9200, 1400)",
            [],
        )
        .expect("bundle");
        conn.execute(
            "INSERT INTO knowledge_bundle_items (id, bundle_id, entry_id, sequence_order)
             VALUES
                (1, 1, 101, 0),
                (2, 1, 102, 1)",
            [],
        )
        .expect("bundle items");
        conn.execute(
            "INSERT INTO student_entry_state (
                user_id, entry_id, confusion_score, recall_strength, linked_wrong_answer_count, review_due_at
             ) VALUES (42, 101, 7200, 1800, 2, '2026-03-29T09:00:00Z')",
            [],
        )
        .expect("entry state one");
        conn.execute(
            "INSERT INTO student_entry_state (
                user_id, entry_id, confusion_score, recall_strength, linked_wrong_answer_count, review_due_at
             ) VALUES (42, 102, 3000, 2200, 1, NULL)",
            [],
        )
        .expect("entry state two");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, decay_risk, trend_state, is_blocked, next_review_at
             ) VALUES (42, 1, 3400, 7100, 6400, 'declining', 0, '2026-03-30T08:00:00Z')",
            [],
        )
        .expect("topic state");
        conn.execute(
            "INSERT INTO wrong_answer_diagnoses (
                id, student_id, topic_id, primary_diagnosis, recommended_action, created_at
             ) VALUES (
                1, 42, 1, 'sign confusion', 'Rebuild the sign pattern before you substitute', '2026-03-29T11:00:00Z'
             )",
            [],
        )
        .expect("diagnosis");
        conn.execute(
            "INSERT INTO contrast_pairs (id, left_entry_id, right_entry_id, title, trap_strength)
             VALUES (1, 101, 102, 'Completing the square vs quadratic formula', 9100)",
            [],
        )
        .expect("contrast pair");
        conn.execute(
            "INSERT INTO knowledge_relations (id, from_entry_id, to_entry_id, relation_type, strength_score)
             VALUES (1, 101, 102, 'contrasts_with', 8900)",
            [],
        )
        .expect("knowledge relation");

        let program = GlossaryService::new(&conn)
            .build_personalized_audio_program_for_topic(42, 1, 3)
            .expect("personalized audio");

        assert_eq!(program.source_type, "topic_personalized");
        assert_eq!(program.teaching_mode, "repair");
        assert!(!program.listener_signals.is_empty());
        assert!(
            program
                .contrast_titles
                .iter()
                .any(|title| title.contains("Completing the square"))
        );
        assert_eq!(program.recommended_bundles.len(), 1);
        assert_eq!(program.recommended_bundles[0].bundle_id, 1);
        assert!(program.recommended_bundles[0].due_review_count >= 1);
        assert!(program.review_entry_ids.contains(&101));
        assert!(
            program
                .relationship_review_prompts
                .iter()
                .any(|prompt| prompt.contains("difference") || prompt.contains("boundary"))
        );
        assert!(
            program
                .segments
                .iter()
                .any(|segment| segment.segment_type == "learner_focus")
        );
        assert!(
            program
                .segments
                .iter()
                .any(|segment| segment.segment_type == "contrast_trap")
        );
        assert!(
            program
                .segments
                .iter()
                .any(|segment| segment.segment_type == "bundle_route")
        );
        assert!(
            program
                .segments
                .iter()
                .any(|segment| segment.segment_type == "review_loop")
        );
        assert!(
            program
                .segments
                .iter()
                .any(|segment| segment.segment_type == "relationship_review")
        );
        assert!(
            program
                .segments
                .iter()
                .filter_map(|segment| segment.focus_reason.as_ref())
                .any(|reason| reason.contains("Review is due") || reason.contains("confusion"))
        );
    }
}
