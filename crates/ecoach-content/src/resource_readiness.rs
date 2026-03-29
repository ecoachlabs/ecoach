use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicResourceReadiness {
    pub subject_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub node_count: i64,
    pub objective_count: i64,
    pub misconception_count: i64,
    pub node_edge_count: i64,
    pub question_family_count: i64,
    pub question_count: i64,
    pub explanation_count: i64,
    pub glossary_count: i64,
    pub formula_count: i64,
    pub worked_example_count: i64,
    pub readiness_score: BasisPoints,
    pub missing_resources: Vec<String>,
    pub generation_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectResourceReadiness {
    pub subject_id: i64,
    pub subject_code: String,
    pub subject_name: String,
    pub average_readiness_score: BasisPoints,
    pub strong_topic_count: i64,
    pub thin_topic_count: i64,
    pub missing_resource_topics: i64,
    pub topics: Vec<TopicResourceReadiness>,
}

pub struct ResourceReadinessService<'a> {
    conn: &'a Connection,
}

impl<'a> ResourceReadinessService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_topic_readiness(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Option<TopicResourceReadiness>> {
        let topic = self
            .conn
            .query_row(
                "SELECT t.subject_id, t.id, t.name
                 FROM topics t
                 WHERE t.id = ?1 AND t.is_active = 1",
                [topic_id],
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

        let Some((subject_id, topic_id, topic_name)) = topic else {
            return Ok(None);
        };

        let node_count = self.count_by_query(
            "SELECT COUNT(*) FROM academic_nodes WHERE topic_id = ?1 AND is_active = 1",
            topic_id,
        )?;
        let objective_count = self.count_by_query(
            "SELECT COUNT(*) FROM learning_objectives WHERE topic_id = ?1",
            topic_id,
        )?;
        let misconception_count = self.count_by_query(
            "SELECT COUNT(*) FROM misconception_patterns WHERE topic_id = ?1 AND is_active = 1",
            topic_id,
        )?;
        let node_edge_count = self.count_by_query(
            "SELECT COUNT(*) FROM node_edges
             WHERE (from_node_type = 'topic' AND from_node_id = ?1)
                OR (to_node_type = 'topic' AND to_node_id = ?1)",
            topic_id,
        )?;
        let question_family_count = self.count_by_query(
            "SELECT COUNT(*) FROM question_families WHERE topic_id = ?1",
            topic_id,
        )?;
        let question_count = self.count_by_query(
            "SELECT COUNT(*) FROM questions WHERE topic_id = ?1 AND is_active = 1",
            topic_id,
        )?;
        let explanation_count = self.count_entry_type(topic_id, "explanation")?;
        let glossary_count = self.count_entry_type(topic_id, "definition")?;
        let formula_count = self.count_entry_type(topic_id, "formula")?;
        let worked_example_count = self.count_entry_type(topic_id, "worked_example")?;

        let missing_resources = build_missing_resources(
            node_count,
            objective_count,
            misconception_count,
            node_edge_count,
            question_family_count,
            question_count,
            explanation_count,
            glossary_count,
            formula_count,
            worked_example_count,
        );
        let generation_modes = build_generation_modes(
            node_count,
            misconception_count,
            question_family_count,
            question_count,
            explanation_count,
            glossary_count,
            formula_count,
            worked_example_count,
            node_edge_count,
        );
        let readiness_score = compute_readiness_score(
            node_count,
            objective_count,
            misconception_count,
            node_edge_count,
            question_family_count,
            question_count,
            explanation_count,
            glossary_count,
            formula_count,
            worked_example_count,
        );

        Ok(Some(TopicResourceReadiness {
            subject_id,
            topic_id,
            topic_name,
            node_count,
            objective_count,
            misconception_count,
            node_edge_count,
            question_family_count,
            question_count,
            explanation_count,
            glossary_count,
            formula_count,
            worked_example_count,
            readiness_score,
            missing_resources,
            generation_modes,
        }))
    }

    pub fn get_subject_readiness(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Option<SubjectResourceReadiness>> {
        let subject = self
            .conn
            .query_row(
                "SELECT id, code, name
                 FROM subjects
                 WHERE id = ?1 AND is_active = 1",
                [subject_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some((subject_id, subject_code, subject_name)) = subject else {
            return Ok(None);
        };

        let mut statement = self
            .conn
            .prepare(
                "SELECT id
                 FROM topics
                 WHERE subject_id = ?1 AND is_active = 1
                 ORDER BY display_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topics = Vec::new();
        for row in rows {
            let topic_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            if let Some(topic_readiness) = self.get_topic_readiness(topic_id)? {
                topics.push(topic_readiness);
            }
        }

        let average_readiness_score = if topics.is_empty() {
            0
        } else {
            clamp_bp(
                (topics
                    .iter()
                    .map(|topic| topic.readiness_score as i64)
                    .sum::<i64>()
                    / topics.len() as i64)
                    .clamp(0, 10_000),
            )
        };
        let strong_topic_count = topics
            .iter()
            .filter(|topic| topic.readiness_score >= 7000)
            .count() as i64;
        let thin_topic_count = topics
            .iter()
            .filter(|topic| topic.readiness_score < 5500)
            .count() as i64;
        let missing_resource_topics = topics
            .iter()
            .filter(|topic| !topic.missing_resources.is_empty())
            .count() as i64;

        Ok(Some(SubjectResourceReadiness {
            subject_id,
            subject_code,
            subject_name,
            average_readiness_score,
            strong_topic_count,
            thin_topic_count,
            missing_resource_topics,
            topics,
        }))
    }

    fn count_by_query(&self, sql: &str, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(sql, [topic_id], |row| row.get(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_entry_type(&self, topic_id: i64, entry_type: &str) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM knowledge_entries
                 WHERE topic_id = ?1
                   AND entry_type = ?2
                   AND status = 'active'",
                params![topic_id, entry_type],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }
}

fn compute_readiness_score(
    node_count: i64,
    objective_count: i64,
    misconception_count: i64,
    node_edge_count: i64,
    question_family_count: i64,
    question_count: i64,
    explanation_count: i64,
    glossary_count: i64,
    formula_count: i64,
    worked_example_count: i64,
) -> BasisPoints {
    let mut score = 0;
    if node_count > 0 {
        score += 1500;
    }
    if objective_count > 0 {
        score += 900;
    }
    if misconception_count > 0 {
        score += 1200;
    }
    if node_edge_count > 0 {
        score += 800;
    }
    if question_family_count > 0 {
        score += 1300;
    }
    score += ((question_count.min(8) as f64 / 8.0) * 2200.0).round() as i64;
    if explanation_count > 0 {
        score += 700;
    }
    if glossary_count > 0 {
        score += 600;
    }
    if formula_count > 0 {
        score += 400;
    }
    if worked_example_count > 0 {
        score += 400;
    }

    clamp_bp(score)
}

fn build_missing_resources(
    node_count: i64,
    objective_count: i64,
    misconception_count: i64,
    node_edge_count: i64,
    question_family_count: i64,
    question_count: i64,
    explanation_count: i64,
    glossary_count: i64,
    formula_count: i64,
    worked_example_count: i64,
) -> Vec<String> {
    let mut missing = Vec::new();
    if node_count == 0 {
        missing.push("concept_atoms_missing".to_string());
    }
    if objective_count == 0 {
        missing.push("learning_objectives_missing".to_string());
    }
    if misconception_count == 0 {
        missing.push("misconception_map_missing".to_string());
    }
    if node_edge_count == 0 {
        missing.push("knowledge_graph_edges_missing".to_string());
    }
    if question_family_count == 0 {
        missing.push("question_family_missing".to_string());
    }
    if question_count == 0 {
        missing.push("question_bank_missing".to_string());
    }
    if explanation_count == 0 && glossary_count == 0 {
        missing.push("explanation_layer_missing".to_string());
    }
    if formula_count == 0 {
        missing.push("formula_support_missing".to_string());
    }
    if worked_example_count == 0 {
        missing.push("worked_example_missing".to_string());
    }
    missing
}

fn build_generation_modes(
    node_count: i64,
    misconception_count: i64,
    question_family_count: i64,
    question_count: i64,
    explanation_count: i64,
    glossary_count: i64,
    formula_count: i64,
    worked_example_count: i64,
    node_edge_count: i64,
) -> Vec<String> {
    let mut modes = Vec::new();
    if node_count > 0 && (explanation_count > 0 || glossary_count > 0) {
        modes.push("teach_mode".to_string());
    }
    if question_count > 0 {
        modes.push("assessment_mode".to_string());
    }
    if question_family_count > 0 && node_count > 0 {
        modes.push("question_generation_seed".to_string());
    }
    if misconception_count > 0 {
        modes.push("misconception_repair".to_string());
    }
    if worked_example_count > 0 {
        modes.push("worked_example_support".to_string());
    }
    if formula_count > 0 {
        modes.push("formula_support".to_string());
    }
    if node_edge_count > 0 {
        modes.push("adaptive_bundle_composition".to_string());
    }
    modes
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    use crate::PackService;

    use super::*;

    #[test]
    fn topic_resource_readiness_scores_installed_pack_content() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = ResourceReadinessService::new(&conn);
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        let readiness = service
            .get_topic_readiness(topic_id)
            .expect("topic readiness should query")
            .expect("topic readiness should exist");

        assert!(readiness.readiness_score > 0);
        assert!(
            readiness
                .generation_modes
                .contains(&"assessment_mode".to_string())
        );
        assert!(
            readiness
                .generation_modes
                .contains(&"teach_mode".to_string())
        );
    }

    #[test]
    fn subject_readiness_rolls_up_topic_scores() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        let service = ResourceReadinessService::new(&conn);
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let readiness = service
            .get_subject_readiness(subject_id)
            .expect("subject readiness should query")
            .expect("subject readiness should exist");

        assert_eq!(readiness.subject_code, "MATH");
        assert_eq!(readiness.topics.len(), 2);
        assert!(readiness.average_readiness_score > 0);
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
