use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, params};

use crate::models::{KnowledgeBundle, KnowledgeEntry, QuestionKnowledgeLink};

pub struct GlossaryService<'a> {
    conn: &'a Connection,
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
}
