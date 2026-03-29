use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{
    Question, QuestionIntelligenceLink, QuestionIntelligenceProfile, QuestionIntelligenceQuery,
    QuestionOption,
};

pub struct QuestionService<'a> {
    conn: &'a Connection,
}

impl<'a> QuestionService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_question(&self, question_id: i64) -> EcoachResult<Option<Question>> {
        self.conn
            .query_row(
                "SELECT id, subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                        explanation_text, difficulty_level, estimated_time_seconds, marks, primary_skill_id
                 FROM questions
                 WHERE id = ?1 AND is_active = 1",
                [question_id],
                map_question,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_question_profile(
        &self,
        question_id: i64,
    ) -> EcoachResult<Option<QuestionIntelligenceProfile>> {
        let Some(question) = self.get_question(question_id)? else {
            return Ok(None);
        };
        let links = self.list_intelligence_links(question_id)?;
        Ok(Some(QuestionIntelligenceProfile { question, links }))
    }

    pub fn list_intelligence_links(
        &self,
        question_id: i64,
    ) -> EcoachResult<Vec<QuestionIntelligenceLink>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT qil.axis_code, qil.concept_code, qit.display_name,
                        qil.confidence_score, qil.is_primary
                 FROM question_intelligence_links qil
                 INNER JOIN question_intelligence_taxonomy qit
                    ON qit.axis_code = qil.axis_code
                   AND qit.concept_code = qil.concept_code
                 WHERE qil.question_id = ?1
                 ORDER BY qil.axis_code ASC, qil.is_primary DESC, qil.confidence_score DESC, qit.display_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([question_id], |row| {
                Ok(QuestionIntelligenceLink {
                    axis_code: row.get(0)?,
                    concept_code: row.get(1)?,
                    display_name: row.get(2)?,
                    confidence_score: row.get::<_, i64>(3)?.clamp(0, 10_000) as u16,
                    is_primary: row.get::<_, i64>(4)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut links = Vec::new();
        for row in rows {
            links.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(links)
    }

    pub fn find_questions_by_intelligence(
        &self,
        query: &QuestionIntelligenceQuery,
    ) -> EcoachResult<Vec<Question>> {
        let topic_filter_enabled = query.topic_id.is_some();
        let subject_filter_enabled = query.subject_id.is_some();
        let limit = query.limit.max(1) as i64;
        let mut statement = self
            .conn
            .prepare(
                "SELECT q.id, q.subject_id, q.topic_id, q.subtopic_id, q.family_id, q.stem,
                        q.question_format, q.explanation_text, q.difficulty_level,
                        q.estimated_time_seconds, q.marks, q.primary_skill_id
                 FROM question_intelligence_links qil
                 INNER JOIN questions q ON q.id = qil.question_id
                 WHERE qil.axis_code = ?1
                   AND qil.concept_code = ?2
                   AND q.is_active = 1
                   AND (?3 = 0 OR q.subject_id = ?4)
                   AND (?5 = 0 OR q.topic_id = ?6)
                 ORDER BY qil.is_primary DESC, qil.confidence_score DESC, q.id ASC
                 LIMIT ?7",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map(
                params![
                    query.axis_code,
                    query.concept_code,
                    if subject_filter_enabled { 1 } else { 0 },
                    query.subject_id.unwrap_or_default(),
                    if topic_filter_enabled { 1 } else { 0 },
                    query.topic_id.unwrap_or_default(),
                    limit,
                ],
                map_question,
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(questions)
    }

    pub fn list_options(&self, question_id: i64) -> EcoachResult<Vec<QuestionOption>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, question_id, option_label, option_text, is_correct,
                        misconception_id, distractor_intent, position
                 FROM question_options
                 WHERE question_id = ?1
                 ORDER BY position ASC, option_label ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([question_id], |row| {
                Ok(QuestionOption {
                    id: row.get(0)?,
                    question_id: row.get(1)?,
                    option_label: row.get(2)?,
                    option_text: row.get(3)?,
                    is_correct: row.get::<_, i64>(4)? == 1,
                    misconception_id: row.get(5)?,
                    distractor_intent: row.get(6)?,
                    position: row.get(7)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut options = Vec::new();
        for row in rows {
            options.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(options)
    }

    pub fn get_option(&self, option_id: i64) -> EcoachResult<Option<QuestionOption>> {
        self.conn
            .query_row(
                "SELECT id, question_id, option_label, option_text, is_correct,
                        misconception_id, distractor_intent, position
                 FROM question_options WHERE id = ?1",
                [option_id],
                |row| {
                    Ok(QuestionOption {
                        id: row.get(0)?,
                        question_id: row.get(1)?,
                        option_label: row.get(2)?,
                        option_text: row.get(3)?,
                        is_correct: row.get::<_, i64>(4)? == 1,
                        misconception_id: row.get(5)?,
                        distractor_intent: row.get(6)?,
                        position: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_correct_option_text(&self, question_id: i64) -> EcoachResult<Option<String>> {
        self.conn
            .query_row(
                "SELECT option_text FROM question_options WHERE question_id = ?1 AND is_correct = 1 LIMIT 1",
                [question_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn insert_question(
        &self,
        subject_id: i64,
        topic_id: i64,
        stem: &str,
        question_format: &str,
        difficulty_level: i64,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO questions (
                    subject_id, topic_id, stem, question_format, difficulty_level, estimated_time_seconds
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 30)",
                params![subject_id, topic_id, stem, question_format, difficulty_level],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }
}

fn map_question(row: &rusqlite::Row<'_>) -> rusqlite::Result<Question> {
    Ok(Question {
        id: row.get(0)?,
        subject_id: row.get(1)?,
        topic_id: row.get(2)?,
        subtopic_id: row.get(3)?,
        family_id: row.get(4)?,
        stem: row.get(5)?,
        question_format: row.get(6)?,
        explanation_text: row.get(7)?,
        difficulty_level: row.get(8)?,
        estimated_time_seconds: row.get(9)?,
        marks: row.get(10)?,
        primary_skill_id: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn question_profile_returns_normalized_intelligence_links() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = QuestionService::new(&conn);
        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");

        let profile = service
            .get_question_profile(question_id)
            .expect("question profile should load")
            .expect("question profile should exist");

        assert!(
            profile
                .links
                .iter()
                .any(|link| link.axis_code == "knowledge_role" && link.is_primary)
        );
        assert!(
            profile
                .links
                .iter()
                .any(|link| link.axis_code == "question_family")
        );
        assert!(
            profile
                .links
                .iter()
                .any(|link| link.axis_code == "misconception_exposure")
        );
    }

    #[test]
    fn can_find_questions_by_normalized_intelligence_axis() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = QuestionService::new(&conn);
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");

        let questions = service
            .find_questions_by_intelligence(&QuestionIntelligenceQuery {
                axis_code: "knowledge_role".to_string(),
                concept_code: "worked_example".to_string(),
                subject_id: Some(subject_id),
                topic_id: None,
                limit: 10,
            })
            .expect("intelligence query should work");

        assert_eq!(questions.len(), 1);
        assert!(questions[0].stem.contains("simplest form"));
    }

    fn open_test_database() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        conn
    }

    fn sample_pack_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate directory should have workspace parent")
            .parent()
            .expect("workspace root should exist")
            .join("packs")
            .join("math-bece-sample")
    }
}
