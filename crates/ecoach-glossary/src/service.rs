use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, params};

use crate::models::{
    GlossaryAudioProgram, GlossaryAudioSegment, KnowledgeBundle, KnowledgeEntry,
    QuestionKnowledgeLink,
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
    ) -> GlossaryAudioProgram {
        let mut segments = Vec::new();
        let intro_prompt = entry_sources
            .first()
            .map(|entry| {
                format!(
                    "Listen for the exact meaning of {}, then compare it against how it appears in the problem.",
                    entry.title
                )
            })
            .unwrap_or_else(|| "Listen for the core idea and recall where it applies.".to_string());
        segments.push(GlossaryAudioSegment {
            sequence_no: 1,
            segment_type: "intro".to_string(),
            title: "Orientation".to_string(),
            script_text: format!(
                "{}. This audio program focuses on the key ideas you should be able to recognize, explain, and recall under exam pressure.",
                program_title
            ),
            entry_id: None,
            prompt_text: Some(intro_prompt),
            duration_seconds: 20,
        });

        for (index, entry) in entry_sources.iter().enumerate() {
            let segment_type = match entry.entry_type.as_str() {
                "definition" => "definition",
                "formula" => "formula",
                "worked_example" => "worked_example",
                _ => "explanation",
            };
            let prompt_text = match entry.entry_type.as_str() {
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
            segments.push(GlossaryAudioSegment {
                sequence_no: (index + 2) as i64,
                segment_type: segment_type.to_string(),
                title: entry.title.clone(),
                script_text: condense_script(&entry.narrative_text),
                entry_id: Some(entry.id),
                prompt_text,
                duration_seconds: estimate_duration_seconds(&entry.narrative_text),
            });
        }

        segments.push(GlossaryAudioSegment {
            sequence_no: (segments.len() + 1) as i64,
            segment_type: "recall_check".to_string(),
            title: "Recall Check".to_string(),
            script_text: "Now stop the audio and say the main idea, the usual trap, and the safest response pattern before you continue.".to_string(),
            entry_id: None,
            prompt_text: Some("Name the key idea, the trap, and the correct response path.".to_string()),
            duration_seconds: 18,
        });

        GlossaryAudioProgram {
            program_title,
            source_type: source_type.to_string(),
            topic_id,
            question_id,
            bundle_ids: bundles.iter().map(|bundle| bundle.id).collect(),
            entry_ids: entry_sources.iter().map(|entry| entry.id).collect(),
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
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
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
