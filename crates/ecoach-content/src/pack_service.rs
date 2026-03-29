use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use ecoach_substrate::{DomainEvent, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::manifest::PackManifest;

#[derive(Debug, Clone)]
pub struct PackInstallResult {
    pub pack_id: String,
    pub pack_version: String,
    pub install_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PackSummary {
    pub pack_id: String,
    pub pack_version: String,
    pub subject_code: String,
    pub status: String,
}

#[derive(Debug, Default, Clone, Copy)]
struct PackImportCounts {
    topic_count: i64,
    question_count: i64,
    knowledge_entry_count: i64,
}

#[derive(Debug)]
struct QuestionKnowledgeQuestion {
    id: i64,
    topic_id: i64,
    stem: String,
    explanation_text: Option<String>,
    primary_skill_title: Option<String>,
}

#[derive(Debug)]
struct QuestionKnowledgeEntry {
    id: i64,
    topic_id: Option<i64>,
    entry_type: String,
    title: String,
    canonical_name: Option<String>,
    short_text: Option<String>,
    importance_score: i64,
    aliases: Vec<String>,
}

#[derive(Debug, Clone)]
struct ScoredKnowledgeLink {
    entry_id: i64,
    relation_type: &'static str,
    confidence_score: i64,
    link_reason: String,
    importance_score: i64,
    same_topic: bool,
}

#[derive(Debug, Deserialize)]
struct TopicRecord {
    code: String,
    name: String,
    #[serde(default = "default_topic_node_type")]
    node_type: String,
    #[serde(default)]
    parent_code: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    display_order: Option<i64>,
    #[serde(default)]
    exam_weight: Option<i64>,
    #[serde(default)]
    difficulty_band: Option<String>,
    #[serde(default)]
    importance_weight: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AcademicNodeRecord {
    topic_code: String,
    canonical_title: String,
    node_type: String,
    #[serde(default)]
    short_label: Option<String>,
    #[serde(default)]
    description_formal: Option<String>,
    #[serde(default)]
    description_simple: Option<String>,
    #[serde(default)]
    core_meaning: Option<String>,
    #[serde(default)]
    difficulty_band: Option<String>,
    #[serde(default)]
    exam_relevance_score: Option<i64>,
    #[serde(default)]
    foundation_weight: Option<i64>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct NodeEdgeRecord {
    #[serde(default)]
    from_topic_code: Option<String>,
    #[serde(default)]
    from_node_title: Option<String>,
    #[serde(default)]
    from_node_topic_code: Option<String>,
    #[serde(default)]
    to_topic_code: Option<String>,
    #[serde(default)]
    to_node_title: Option<String>,
    #[serde(default)]
    to_node_topic_code: Option<String>,
    edge_type: String,
    #[serde(default)]
    strength_score: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MisconceptionRecord {
    topic_code: String,
    title: String,
    misconception_statement: String,
    #[serde(default)]
    node_title: Option<String>,
    #[serde(default)]
    cause_type: Option<String>,
    #[serde(default)]
    wrong_answer_pattern: Option<String>,
    #[serde(default)]
    correction_hint: Option<String>,
    #[serde(default)]
    severity: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ObjectiveRecord {
    topic_code: String,
    objective_text: String,
    #[serde(default)]
    simplified_text: Option<String>,
    #[serde(default)]
    cognitive_level: Option<String>,
    #[serde(default)]
    display_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct QuestionFamilyRecord {
    family_code: String,
    family_name: String,
    #[serde(default)]
    topic_code: Option<String>,
    #[serde(default)]
    subtopic_code: Option<String>,
    #[serde(default = "default_family_type")]
    family_type: String,
    #[serde(default)]
    canonical_pattern: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QuestionRecord {
    stem: String,
    topic_code: String,
    #[serde(default = "default_question_format")]
    question_format: String,
    #[serde(default)]
    subtopic_code: Option<String>,
    #[serde(default)]
    family_code: Option<String>,
    #[serde(default)]
    explanation_text: Option<String>,
    #[serde(default)]
    difficulty_level: Option<i64>,
    #[serde(default)]
    estimated_time_seconds: Option<i64>,
    #[serde(default)]
    marks: Option<i64>,
    #[serde(default)]
    source_type: Option<String>,
    #[serde(default)]
    source_ref: Option<String>,
    #[serde(default)]
    exam_year: Option<i64>,
    #[serde(default)]
    primary_skill_title: Option<String>,
    #[serde(default)]
    cognitive_level: Option<String>,
    #[serde(default)]
    options: Vec<QuestionOptionRecord>,
    #[serde(default)]
    skill_titles: Vec<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct QuestionOptionRecord {
    #[serde(default)]
    option_label: Option<String>,
    option_text: String,
    #[serde(default)]
    is_correct: Option<bool>,
    #[serde(default)]
    misconception_title: Option<String>,
    #[serde(default)]
    distractor_intent: Option<String>,
    #[serde(default)]
    position: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct QuestionIntelligenceRecord {
    question_index: usize,
    #[serde(default)]
    primary_knowledge_role: Option<String>,
    #[serde(default)]
    primary_cognitive_demand: Option<String>,
    #[serde(default)]
    primary_solve_pattern: Option<String>,
    #[serde(default)]
    primary_pedagogic_function: Option<String>,
    #[serde(default)]
    classification_confidence: Option<i64>,
    #[serde(default)]
    knowledge_roles: Vec<String>,
    #[serde(default)]
    cognitive_demands: Vec<String>,
    #[serde(default)]
    solve_patterns: Vec<String>,
    #[serde(default)]
    pedagogic_functions: Vec<String>,
    #[serde(default)]
    content_grains: Vec<String>,
    #[serde(default)]
    misconception_exposures: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct KnowledgeEntryRecord {
    title: String,
    #[serde(default)]
    topic_code: Option<String>,
    #[serde(default)]
    subtopic_code: Option<String>,
    #[serde(default)]
    entry_type: Option<String>,
    #[serde(default)]
    canonical_name: Option<String>,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    short_text: Option<String>,
    #[serde(default)]
    full_text: Option<String>,
    #[serde(default)]
    simple_text: Option<String>,
    #[serde(default)]
    technical_text: Option<String>,
    #[serde(default)]
    exam_text: Option<String>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    importance_score: Option<i64>,
    #[serde(default)]
    difficulty_level: Option<i64>,
    #[serde(default)]
    grade_band: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ContrastPairRecord {
    pair_code: String,
    title: String,
    topic_code: String,
    left_entry_title: String,
    right_entry_title: String,
    #[serde(default)]
    left_label: Option<String>,
    #[serde(default)]
    right_label: Option<String>,
    #[serde(default)]
    summary_text: Option<String>,
    #[serde(default)]
    trap_strength: Option<i64>,
    #[serde(default)]
    difficulty_score: Option<i64>,
    #[serde(default)]
    atoms: Vec<ContrastAtomRecord>,
}

#[derive(Debug, Deserialize)]
struct ContrastAtomRecord {
    ownership_type: String,
    atom_text: String,
    #[serde(default)]
    lane: Option<String>,
    #[serde(default)]
    explanation_text: Option<String>,
    #[serde(default)]
    difficulty_score: Option<i64>,
    #[serde(default)]
    is_speed_ready: Option<bool>,
    #[serde(default)]
    reveal_order: Option<i64>,
}

pub struct PackService<'a> {
    conn: &'a Connection,
}

impl<'a> PackService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn install_pack(&self, pack_path: &Path) -> EcoachResult<PackInstallResult> {
        let manifest = self.load_manifest(pack_path)?;
        self.validate_pack_shape(pack_path)?;

        let manifest_json = serde_json::to_string(&manifest)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let install_path = pack_path.display().to_string();

        self.upsert_pack_record(
            &manifest,
            &manifest_json,
            &install_path,
            "installing",
            None,
            None,
        )?;
        self.begin_transaction()?;

        let install_result = (|| -> EcoachResult<PackInstallResult> {
            let curriculum_version_id = self.upsert_curriculum_version(&manifest)?;
            let subject_id = self.upsert_subject(curriculum_version_id, &manifest)?;

            self.clear_subject_data(subject_id)?;

            let import_counts = self.import_pack_data(pack_path, subject_id, &manifest)?;
            self.validate_manifest_counts(&manifest, import_counts)?;
            self.mark_pack_active(&manifest, &manifest_json, &install_path, import_counts)?;

            Ok(PackInstallResult {
                pack_id: manifest.pack_id.clone(),
                pack_version: manifest.pack_version.clone(),
                install_path: pack_path.to_path_buf(),
            })
        })();

        match install_result {
            Ok(result) => {
                self.commit_transaction()?;
                self.append_runtime_event(DomainEvent::new(
                    "pack.installed",
                    result.pack_id.clone(),
                    serde_json::json!({
                        "pack_id": result.pack_id,
                        "pack_version": result.pack_version,
                        "install_path": result.install_path.display().to_string(),
                    }),
                ))?;
                Ok(result)
            }
            Err(err) => {
                let _ = self.rollback_transaction();
                let _ = self.mark_pack_failed(&manifest.pack_id, &err);
                Err(err)
            }
        }
    }

    pub fn list_packs(&self) -> EcoachResult<Vec<PackSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT pack_id, pack_version, subject_code, status
                 FROM content_packs
                 WHERE status != 'removed'
                 ORDER BY installed_at DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([], |row| {
                Ok(PackSummary {
                    pack_id: row.get(0)?,
                    pack_version: row.get(1)?,
                    subject_code: row.get(2)?,
                    status: row.get(3)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut packs = Vec::new();
        for row in rows {
            packs.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(packs)
    }

    fn import_pack_data(
        &self,
        pack_path: &Path,
        subject_id: i64,
        manifest: &PackManifest,
    ) -> EcoachResult<PackImportCounts> {
        let topics: Vec<TopicRecord> =
            self.read_json_file(&pack_path.join("curriculum/topics.json"))?;
        let nodes: Vec<AcademicNodeRecord> =
            self.read_json_file(&pack_path.join("curriculum/nodes.json"))?;
        let edges: Vec<NodeEdgeRecord> =
            self.read_json_file(&pack_path.join("curriculum/edges.json"))?;
        let misconceptions: Vec<MisconceptionRecord> =
            self.read_json_file(&pack_path.join("curriculum/misconceptions.json"))?;
        let objectives: Vec<ObjectiveRecord> =
            self.read_json_file(&pack_path.join("curriculum/objectives.json"))?;
        let families: Vec<QuestionFamilyRecord> =
            self.read_json_file(&pack_path.join("questions/families.json"))?;
        let questions: Vec<QuestionRecord> =
            self.read_json_file(&pack_path.join("questions/questions.json"))?;
        let intelligence: Vec<QuestionIntelligenceRecord> =
            self.read_json_file(&pack_path.join("questions/intelligence.json"))?;
        let explanations: Vec<KnowledgeEntryRecord> =
            self.read_json_file(&pack_path.join("content/explanations.json"))?;
        let glossary: Vec<KnowledgeEntryRecord> =
            self.read_json_file(&pack_path.join("content/glossary.json"))?;
        let formulas: Vec<KnowledgeEntryRecord> =
            self.read_json_file(&pack_path.join("content/formulas.json"))?;
        let worked_examples: Vec<KnowledgeEntryRecord> =
            self.read_json_file(&pack_path.join("content/worked_examples.json"))?;
        let contrast_pairs: Vec<ContrastPairRecord> =
            self.read_optional_json_file(&pack_path.join("content/contrast_pairs.json"))?;

        let topic_ids = self.insert_topics(subject_id, &topics)?;
        let node_ids = self.insert_academic_nodes(&topic_ids, &nodes)?;
        self.insert_node_edges(&topic_ids, &node_ids, &edges)?;
        let misconception_ids =
            self.insert_misconceptions(&topic_ids, &node_ids, &misconceptions)?;
        self.insert_learning_objectives(&topic_ids, &objectives)?;
        let family_ids = self.insert_question_families(subject_id, &topic_ids, &families)?;
        self.insert_questions(
            subject_id,
            &manifest.pack_id,
            &topic_ids,
            &node_ids,
            &misconception_ids,
            &family_ids,
            &questions,
            &intelligence,
        )?;

        let mut knowledge_entry_count = 0;
        knowledge_entry_count +=
            self.insert_knowledge_entries(subject_id, &topic_ids, &explanations, "explanation")?;
        knowledge_entry_count +=
            self.insert_knowledge_entries(subject_id, &topic_ids, &glossary, "definition")?;
        knowledge_entry_count +=
            self.insert_knowledge_entries(subject_id, &topic_ids, &formulas, "formula")?;
        knowledge_entry_count += self.insert_knowledge_entries(
            subject_id,
            &topic_ids,
            &worked_examples,
            "worked_example",
        )?;
        self.insert_contrast_profiles(subject_id, &topic_ids, &contrast_pairs)?;
        self.link_questions_to_knowledge(subject_id)?;

        Ok(PackImportCounts {
            topic_count: topics.len() as i64,
            question_count: questions.len() as i64,
            knowledge_entry_count,
        })
    }

    fn insert_topics(
        &self,
        subject_id: i64,
        topics: &[TopicRecord],
    ) -> EcoachResult<BTreeMap<String, i64>> {
        let mut topic_ids = BTreeMap::new();

        for (index, topic) in topics.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO topics (
                        subject_id, parent_topic_id, code, name, description, node_type,
                        display_order, exam_weight, difficulty_band, importance_weight
                    ) VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        subject_id,
                        topic.code,
                        topic.name,
                        topic.description,
                        topic.node_type,
                        topic.display_order.unwrap_or(index as i64),
                        topic.exam_weight.unwrap_or(5000),
                        topic.difficulty_band.as_deref().unwrap_or("medium"),
                        topic.importance_weight.unwrap_or(5000),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            topic_ids.insert(topic.code.clone(), self.conn.last_insert_rowid());
        }

        for topic in topics {
            if let Some(parent_code) = &topic.parent_code {
                let topic_id = *topic_ids.get(&topic.code).ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "unknown topic code during parent linking: {}",
                        topic.code
                    ))
                })?;
                let parent_topic_id = *topic_ids.get(parent_code).ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "topic {} references unknown parent topic code {}",
                        topic.code, parent_code
                    ))
                })?;

                self.conn
                    .execute(
                        "UPDATE topics SET parent_topic_id = ?1, updated_at = datetime('now') WHERE id = ?2",
                        params![parent_topic_id, topic_id],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
        }

        Ok(topic_ids)
    }

    fn insert_academic_nodes(
        &self,
        topic_ids: &BTreeMap<String, i64>,
        nodes: &[AcademicNodeRecord],
    ) -> EcoachResult<BTreeMap<String, i64>> {
        let mut node_ids = BTreeMap::new();

        for node in nodes {
            let topic_id = *topic_ids.get(&node.topic_code).ok_or_else(|| {
                EcoachError::Validation(format!(
                    "academic node {} references unknown topic code {}",
                    node.canonical_title, node.topic_code
                ))
            })?;

            let metadata_json = serde_json::to_string(&node.extra)
                .map_err(|err| EcoachError::Serialization(err.to_string()))?;

            self.conn
                .execute(
                    "INSERT INTO academic_nodes (
                        topic_id, node_type, canonical_title, short_label, description_formal,
                        description_simple, core_meaning, difficulty_band, exam_relevance_score,
                        foundation_weight, metadata_json
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        topic_id,
                        node.node_type,
                        node.canonical_title,
                        node.short_label,
                        node.description_formal,
                        node.description_simple,
                        node.core_meaning,
                        node.difficulty_band.as_deref().unwrap_or("medium"),
                        node.exam_relevance_score.unwrap_or(5000),
                        node.foundation_weight.unwrap_or(5000),
                        metadata_json,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            node_ids.insert(
                scoped_key(&node.topic_code, &node.canonical_title),
                self.conn.last_insert_rowid(),
            );
        }

        Ok(node_ids)
    }

    fn insert_node_edges(
        &self,
        topic_ids: &BTreeMap<String, i64>,
        node_ids: &BTreeMap<String, i64>,
        edges: &[NodeEdgeRecord],
    ) -> EcoachResult<()> {
        for edge in edges {
            let (from_node_id, from_node_type) = resolve_edge_endpoint(
                topic_ids,
                node_ids,
                edge.from_topic_code.as_deref(),
                edge.from_node_title.as_deref(),
                edge.from_node_topic_code.as_deref(),
                "from",
            )?;
            let (to_node_id, to_node_type) = resolve_edge_endpoint(
                topic_ids,
                node_ids,
                edge.to_topic_code.as_deref(),
                edge.to_node_title.as_deref(),
                edge.to_node_topic_code.as_deref(),
                "to",
            )?;

            self.conn
                .execute(
                    "INSERT INTO node_edges (
                        from_node_id, from_node_type, to_node_id, to_node_type, edge_type, strength_score
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        from_node_id,
                        from_node_type,
                        to_node_id,
                        to_node_type,
                        edge.edge_type,
                        edge.strength_score.unwrap_or(5000),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn insert_misconceptions(
        &self,
        topic_ids: &BTreeMap<String, i64>,
        node_ids: &BTreeMap<String, i64>,
        misconceptions: &[MisconceptionRecord],
    ) -> EcoachResult<BTreeMap<String, i64>> {
        let mut misconception_ids = BTreeMap::new();

        for misconception in misconceptions {
            let topic_id = *topic_ids.get(&misconception.topic_code).ok_or_else(|| {
                EcoachError::Validation(format!(
                    "misconception {} references unknown topic code {}",
                    misconception.title, misconception.topic_code
                ))
            })?;
            let node_id = if let Some(node_title) = &misconception.node_title {
                Some(resolve_node_id(
                    node_ids,
                    &misconception.topic_code,
                    node_title,
                )?)
            } else {
                None
            };

            self.conn
                .execute(
                    "INSERT INTO misconception_patterns (
                        node_id, topic_id, title, misconception_statement, cause_type,
                        wrong_answer_pattern, correction_hint, severity
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        node_id,
                        topic_id,
                        misconception.title,
                        misconception.misconception_statement,
                        misconception.cause_type,
                        misconception.wrong_answer_pattern,
                        misconception.correction_hint,
                        misconception.severity.unwrap_or(5000),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            misconception_ids.insert(
                scoped_key(&misconception.topic_code, &misconception.title),
                self.conn.last_insert_rowid(),
            );
        }

        Ok(misconception_ids)
    }

    fn insert_learning_objectives(
        &self,
        topic_ids: &BTreeMap<String, i64>,
        objectives: &[ObjectiveRecord],
    ) -> EcoachResult<()> {
        for (index, objective) in objectives.iter().enumerate() {
            let topic_id = *topic_ids.get(&objective.topic_code).ok_or_else(|| {
                EcoachError::Validation(format!(
                    "learning objective references unknown topic code {}",
                    objective.topic_code
                ))
            })?;

            self.conn
                .execute(
                    "INSERT INTO learning_objectives (
                        topic_id, objective_text, simplified_text, cognitive_level, display_order
                    ) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        topic_id,
                        objective.objective_text,
                        objective.simplified_text,
                        objective.cognitive_level,
                        objective.display_order.unwrap_or(index as i64),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn insert_question_families(
        &self,
        subject_id: i64,
        topic_ids: &BTreeMap<String, i64>,
        families: &[QuestionFamilyRecord],
    ) -> EcoachResult<BTreeMap<String, i64>> {
        let mut family_ids = BTreeMap::new();

        for family in families {
            let topic_id = match family.topic_code.as_deref() {
                Some(topic_code) => Some(*topic_ids.get(topic_code).ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "question family {} references unknown topic code {}",
                        family.family_code, topic_code
                    ))
                })?),
                None => None,
            };
            let subtopic_id = match family.subtopic_code.as_deref() {
                Some(topic_code) => Some(*topic_ids.get(topic_code).ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "question family {} references unknown subtopic code {}",
                        family.family_code, topic_code
                    ))
                })?),
                None => None,
            };

            self.conn
                .execute(
                    "INSERT INTO question_families (
                        family_code, family_name, subject_id, topic_id, subtopic_id, family_type,
                        canonical_pattern, description
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        family.family_code,
                        family.family_name,
                        subject_id,
                        topic_id,
                        subtopic_id,
                        family.family_type,
                        family.canonical_pattern,
                        family.description,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            family_ids.insert(family.family_code.clone(), self.conn.last_insert_rowid());
        }

        Ok(family_ids)
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_questions(
        &self,
        subject_id: i64,
        pack_id: &str,
        topic_ids: &BTreeMap<String, i64>,
        node_ids: &BTreeMap<String, i64>,
        misconception_ids: &BTreeMap<String, i64>,
        family_ids: &BTreeMap<String, i64>,
        questions: &[QuestionRecord],
        intelligence: &[QuestionIntelligenceRecord],
    ) -> EcoachResult<()> {
        let intelligence_by_index: BTreeMap<usize, &QuestionIntelligenceRecord> = intelligence
            .iter()
            .map(|item| (item.question_index, item))
            .collect();

        for (index, question) in questions.iter().enumerate() {
            let topic_id = *topic_ids.get(&question.topic_code).ok_or_else(|| {
                EcoachError::Validation(format!(
                    "question {} references unknown topic code {}",
                    index, question.topic_code
                ))
            })?;
            let subtopic_id = match question.subtopic_code.as_deref() {
                Some(topic_code) => Some(*topic_ids.get(topic_code).ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "question {} references unknown subtopic code {}",
                        index, topic_code
                    ))
                })?),
                None => None,
            };
            let family_id = match question.family_code.as_deref() {
                Some(family_code) => Some(*family_ids.get(family_code).ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "question {} references unknown family code {}",
                        index, family_code
                    ))
                })?),
                None => None,
            };
            let intelligence_record = intelligence_by_index.get(&index).copied();
            let primary_skill_id = match question.primary_skill_title.as_deref() {
                Some(node_title) => {
                    Some(resolve_node_id(node_ids, &question.topic_code, node_title)?)
                }
                None => None,
            };
            let intelligence_snapshot = build_question_snapshot(question, intelligence_record)?;

            self.conn
                .execute(
                    "INSERT INTO questions (
                        subject_id, topic_id, subtopic_id, family_id, stem, question_format,
                        explanation_text, difficulty_level, estimated_time_seconds, marks,
                        source_type, source_ref, exam_year, primary_knowledge_role,
                        primary_cognitive_demand, primary_solve_pattern,
                        primary_pedagogic_function, classification_confidence,
                        intelligence_snapshot, primary_skill_id, cognitive_level, pack_id
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
                    params![
                        subject_id,
                        topic_id,
                        subtopic_id,
                        family_id,
                        question.stem,
                        question.question_format,
                        question.explanation_text,
                        question.difficulty_level.unwrap_or(5000),
                        question.estimated_time_seconds.unwrap_or(30),
                        question.marks.unwrap_or(1),
                        question.source_type.as_deref().unwrap_or("authored"),
                        question.source_ref,
                        question.exam_year,
                        intelligence_record.and_then(|item| item.primary_knowledge_role.as_deref()),
                        intelligence_record.and_then(|item| item.primary_cognitive_demand.as_deref()),
                        intelligence_record.and_then(|item| item.primary_solve_pattern.as_deref()),
                        intelligence_record.and_then(|item| item.primary_pedagogic_function.as_deref()),
                        intelligence_record
                            .and_then(|item| item.classification_confidence)
                            .unwrap_or_default(),
                        intelligence_snapshot,
                        primary_skill_id,
                        question.cognitive_level,
                        pack_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            let question_id = self.conn.last_insert_rowid();

            self.insert_question_options(question_id, question, misconception_ids)?;
            self.insert_question_skill_links(question_id, question, node_ids)?;
            self.insert_question_intelligence_links(
                question_id,
                question,
                intelligence_record,
                family_id,
            )?;
        }

        Ok(())
    }

    fn insert_question_options(
        &self,
        question_id: i64,
        question: &QuestionRecord,
        misconception_ids: &BTreeMap<String, i64>,
    ) -> EcoachResult<()> {
        for (index, option) in question.options.iter().enumerate() {
            let misconception_id = match option.misconception_title.as_deref() {
                Some(title) => Some(resolve_misconception_id(
                    misconception_ids,
                    &question.topic_code,
                    title,
                )?),
                None => None,
            };

            self.conn
                .execute(
                    "INSERT INTO question_options (
                        question_id, option_label, option_text, is_correct, misconception_id,
                        distractor_intent, position
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        question_id,
                        option
                            .option_label
                            .clone()
                            .unwrap_or_else(|| default_option_label(index)),
                        option.option_text,
                        option.is_correct.unwrap_or(false) as i64,
                        misconception_id,
                        option.distractor_intent,
                        option.position.unwrap_or(index as i64),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn insert_question_skill_links(
        &self,
        question_id: i64,
        question: &QuestionRecord,
        node_ids: &BTreeMap<String, i64>,
    ) -> EcoachResult<()> {
        for (index, skill_title) in question.skill_titles.iter().enumerate() {
            let node_id = resolve_node_id(node_ids, &question.topic_code, skill_title)?;
            let is_primary = question
                .primary_skill_title
                .as_deref()
                .map(|primary| normalize_key(primary) == normalize_key(skill_title))
                .unwrap_or(index == 0);

            self.conn
                .execute(
                    "INSERT INTO question_skill_links (
                        question_id, node_id, contribution_weight, is_primary
                    ) VALUES (?1, ?2, ?3, ?4)",
                    params![question_id, node_id, 10000, is_primary as i64],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn insert_question_intelligence_links(
        &self,
        question_id: i64,
        question: &QuestionRecord,
        intelligence: Option<&QuestionIntelligenceRecord>,
        family_id: Option<i64>,
    ) -> EcoachResult<()> {
        let Some(intelligence) = intelligence else {
            if let Some(family_code) = question.family_code.as_deref() {
                self.upsert_question_intelligence_link(
                    question_id,
                    "question_family",
                    family_code,
                    family_code,
                    5000,
                    family_id.is_some(),
                )?;
            }
            return Ok(());
        };

        let classification_confidence = intelligence.classification_confidence.unwrap_or(5000);
        self.insert_axis_values(
            question_id,
            "knowledge_role",
            &intelligence.knowledge_roles,
            intelligence.primary_knowledge_role.as_deref(),
            classification_confidence,
        )?;
        self.insert_axis_values(
            question_id,
            "cognitive_demand",
            &intelligence.cognitive_demands,
            intelligence.primary_cognitive_demand.as_deref(),
            classification_confidence,
        )?;
        self.insert_axis_values(
            question_id,
            "solve_pattern",
            &intelligence.solve_patterns,
            intelligence.primary_solve_pattern.as_deref(),
            classification_confidence,
        )?;
        self.insert_axis_values(
            question_id,
            "pedagogic_function",
            &intelligence.pedagogic_functions,
            intelligence.primary_pedagogic_function.as_deref(),
            classification_confidence,
        )?;

        let mut content_grains = intelligence.content_grains.clone();
        if content_grains.is_empty() {
            content_grains.push("topic".to_string());
        }
        self.insert_axis_values(
            question_id,
            "content_grain",
            &content_grains,
            content_grains.first().map(|value| value.as_str()),
            classification_confidence,
        )?;

        if let Some(family_code) = question.family_code.as_deref() {
            self.upsert_question_intelligence_link(
                question_id,
                "question_family",
                family_code,
                family_code,
                classification_confidence,
                family_id.is_some(),
            )?;
        }

        for misconception in &intelligence.misconception_exposures {
            self.upsert_question_intelligence_link(
                question_id,
                "misconception_exposure",
                misconception,
                misconception,
                classification_confidence,
                false,
            )?;
        }

        Ok(())
    }

    fn insert_axis_values(
        &self,
        question_id: i64,
        axis_code: &str,
        values: &[String],
        primary_value: Option<&str>,
        confidence_score: i64,
    ) -> EcoachResult<()> {
        let mut emitted = BTreeSet::new();
        for raw_value in values {
            let normalized = normalize_taxonomy_value(raw_value);
            if !emitted.insert(normalized.clone()) {
                continue;
            }
            self.upsert_question_intelligence_link(
                question_id,
                axis_code,
                &normalized,
                raw_value,
                confidence_score,
                primary_value
                    .map(|value| normalize_taxonomy_value(value) == normalized)
                    .unwrap_or(false),
            )?;
        }

        if let Some(primary_value) = primary_value {
            let normalized_primary = normalize_taxonomy_value(primary_value);
            if emitted.insert(normalized_primary.clone()) {
                self.upsert_question_intelligence_link(
                    question_id,
                    axis_code,
                    &normalized_primary,
                    primary_value,
                    confidence_score,
                    true,
                )?;
            }
        }

        Ok(())
    }

    fn upsert_question_intelligence_link(
        &self,
        question_id: i64,
        axis_code: &str,
        concept_code: &str,
        display_name: &str,
        confidence_score: i64,
        is_primary: bool,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO question_intelligence_taxonomy (
                    axis_code, concept_code, display_name
                 ) VALUES (?1, ?2, ?3)
                 ON CONFLICT(axis_code, concept_code) DO UPDATE SET
                    display_name = excluded.display_name,
                    updated_at = datetime('now')",
                params![axis_code, concept_code, display_name],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO question_intelligence_links (
                    question_id, axis_code, concept_code, confidence_score, is_primary, source
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 'pack')
                 ON CONFLICT(question_id, axis_code, concept_code) DO UPDATE SET
                    confidence_score = excluded.confidence_score,
                    is_primary = MAX(question_intelligence_links.is_primary, excluded.is_primary)",
                params![
                    question_id,
                    axis_code,
                    concept_code,
                    confidence_score.clamp(0, 10_000),
                    if is_primary { 1 } else { 0 },
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_knowledge_entries(
        &self,
        subject_id: i64,
        topic_ids: &BTreeMap<String, i64>,
        entries: &[KnowledgeEntryRecord],
        default_entry_type: &str,
    ) -> EcoachResult<i64> {
        let mut inserted = 0;

        for entry in entries {
            let topic_id = match entry.topic_code.as_deref() {
                Some(topic_code) => Some(*topic_ids.get(topic_code).ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "knowledge entry {} references unknown topic code {}",
                        entry.title, topic_code
                    ))
                })?),
                None => None,
            };
            let subtopic_id = match entry.subtopic_code.as_deref() {
                Some(topic_code) => Some(*topic_ids.get(topic_code).ok_or_else(|| {
                    EcoachError::Validation(format!(
                        "knowledge entry {} references unknown subtopic code {}",
                        entry.title, topic_code
                    ))
                })?),
                None => None,
            };
            let full_text = entry.full_text.clone().or_else(|| entry.body.clone());

            self.conn
                .execute(
                    "INSERT INTO knowledge_entries (
                        subject_id, topic_id, subtopic_id, entry_type, title, canonical_name,
                        slug, short_text, full_text, simple_text, technical_text, exam_text,
                        importance_score, difficulty_level, grade_band, status
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                    params![
                        subject_id,
                        topic_id,
                        subtopic_id,
                        entry.entry_type.as_deref().unwrap_or(default_entry_type),
                        entry.title,
                        entry.canonical_name.as_deref().unwrap_or(&entry.title),
                        entry.slug.clone().unwrap_or_else(|| slugify(&entry.title)),
                        entry.short_text,
                        full_text,
                        entry.simple_text,
                        entry.technical_text,
                        entry.exam_text,
                        entry.importance_score.unwrap_or(5000),
                        entry.difficulty_level.unwrap_or(5000),
                        entry.grade_band,
                        entry.status.as_deref().unwrap_or("active"),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            let entry_id = self.conn.last_insert_rowid();

            for alias in &entry.aliases {
                self.conn
                    .execute(
                        "INSERT INTO entry_aliases (entry_id, alias_text, alias_type) VALUES (?1, ?2, 'synonym')",
                        params![entry_id, alias],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }

            inserted += 1;
        }

        Ok(inserted)
    }

    fn link_questions_to_knowledge(&self, subject_id: i64) -> EcoachResult<()> {
        let questions = self.load_question_knowledge_questions(subject_id)?;
        if questions.is_empty() {
            return Ok(());
        }

        let entries = self.load_question_knowledge_entries(subject_id)?;
        if entries.is_empty() {
            return Ok(());
        }

        self.conn
            .execute(
                "DELETE FROM question_glossary_links
                 WHERE question_id IN (SELECT id FROM questions WHERE subject_id = ?1)",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for question in &questions {
            let selected_links = select_knowledge_links_for_question(question, &entries);
            for (index, link) in selected_links.iter().enumerate() {
                self.conn
                    .execute(
                        "INSERT INTO question_glossary_links (
                            question_id, entry_id, relation_type, link_source, link_reason, confidence_score, is_primary
                         ) VALUES (?1, ?2, ?3, 'pack_inference', ?4, ?5, ?6)
                         ON CONFLICT(question_id, entry_id) DO UPDATE SET
                            relation_type = excluded.relation_type,
                            link_source = excluded.link_source,
                            link_reason = excluded.link_reason,
                            confidence_score = excluded.confidence_score,
                            is_primary = excluded.is_primary,
                            updated_at = datetime('now')",
                        params![
                            question.id,
                            link.entry_id,
                            link.relation_type,
                            link.link_reason,
                            link.confidence_score,
                            (index == 0) as i64,
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
        }

        Ok(())
    }

    fn insert_contrast_profiles(
        &self,
        subject_id: i64,
        topic_ids: &BTreeMap<String, i64>,
        contrast_pairs: &[ContrastPairRecord],
    ) -> EcoachResult<()> {
        if contrast_pairs.is_empty() {
            return Ok(());
        }

        let knowledge_entries = self.load_subject_knowledge_entries_by_title(subject_id)?;

        for pair in contrast_pairs {
            let topic_id = *topic_ids.get(&pair.topic_code).ok_or_else(|| {
                EcoachError::Validation(format!(
                    "contrast pair {} references unknown topic code {}",
                    pair.title, pair.topic_code
                ))
            })?;
            let left_entry_id = resolve_knowledge_entry_title(
                &knowledge_entries,
                &pair.left_entry_title,
                &pair.topic_code,
            )?;
            let right_entry_id = resolve_knowledge_entry_title(
                &knowledge_entries,
                &pair.right_entry_title,
                &pair.topic_code,
            )?;

            self.conn
                .execute(
                    "INSERT INTO contrast_pairs (
                        left_entry_id, right_entry_id, title, trap_strength, pair_code,
                        subject_id, topic_id, left_label, right_label, summary_text, difficulty_score
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        left_entry_id,
                        right_entry_id,
                        pair.title,
                        pair.trap_strength.unwrap_or(6500),
                        pair.pair_code,
                        subject_id,
                        topic_id,
                        pair.left_label
                            .as_deref()
                            .unwrap_or(pair.left_entry_title.as_str()),
                        pair.right_label
                            .as_deref()
                            .unwrap_or(pair.right_entry_title.as_str()),
                        pair.summary_text,
                        pair.difficulty_score.unwrap_or(5000),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            let pair_id = self.conn.last_insert_rowid();

            for atom in &pair.atoms {
                validate_contrast_ownership(&atom.ownership_type, &pair.title)?;
                self.conn
                    .execute(
                        "INSERT INTO contrast_evidence_atoms (
                            pair_id, ownership_type, atom_text, lane, explanation_text,
                            difficulty_score, is_speed_ready, reveal_order
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![
                            pair_id,
                            atom.ownership_type,
                            atom.atom_text,
                            atom.lane.as_deref().unwrap_or("feature"),
                            atom.explanation_text,
                            atom.difficulty_score.unwrap_or(5000),
                            atom.is_speed_ready.unwrap_or(true) as i64,
                            atom.reveal_order.unwrap_or(1),
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
        }

        Ok(())
    }

    fn load_subject_knowledge_entries_by_title(
        &self,
        subject_id: i64,
    ) -> EcoachResult<BTreeMap<String, i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT ke.id, t.code, ke.title
                 FROM knowledge_entries ke
                 LEFT JOIN topics t ON t.id = ke.topic_id
                 WHERE ke.subject_id = ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut entries = BTreeMap::new();
        for row in rows {
            let (entry_id, topic_code, title) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let title_key = normalize_title_key(&title);
            entries.insert(title_key.clone(), entry_id);
            if let Some(topic_code) = topic_code {
                entries.insert(scoped_key(&topic_code, &title), entry_id);
                entries.insert(scoped_key(&topic_code, &title_key), entry_id);
            }
        }

        Ok(entries)
    }

    fn load_question_knowledge_questions(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Vec<QuestionKnowledgeQuestion>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                    q.id,
                    q.topic_id,
                    q.stem,
                    q.explanation_text,
                    n.canonical_title
                 FROM questions q
                 LEFT JOIN academic_nodes n ON n.id = q.primary_skill_id
                 WHERE q.subject_id = ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([subject_id], |row| {
                Ok(QuestionKnowledgeQuestion {
                    id: row.get(0)?,
                    topic_id: row.get(1)?,
                    stem: row.get(2)?,
                    explanation_text: row.get(3)?,
                    primary_skill_title: row.get(4)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut questions = Vec::new();
        for row in rows {
            questions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(questions)
    }

    fn load_question_knowledge_entries(
        &self,
        subject_id: i64,
    ) -> EcoachResult<Vec<QuestionKnowledgeEntry>> {
        let mut entry_statement = self
            .conn
            .prepare(
                "SELECT id, topic_id, entry_type, title, canonical_name, short_text, importance_score
                 FROM knowledge_entries
                 WHERE subject_id = ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let entry_rows = entry_statement
            .query_map([subject_id], |row| {
                Ok(QuestionKnowledgeEntry {
                    id: row.get(0)?,
                    topic_id: row.get(1)?,
                    entry_type: row.get(2)?,
                    title: row.get(3)?,
                    canonical_name: row.get(4)?,
                    short_text: row.get(5)?,
                    importance_score: row.get(6)?,
                    aliases: Vec::new(),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut entries = Vec::new();
        for row in entry_rows {
            entries.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        let mut alias_statement = self
            .conn
            .prepare(
                "SELECT ea.entry_id, ea.alias_text
                 FROM entry_aliases ea
                 INNER JOIN knowledge_entries ke ON ke.id = ea.entry_id
                 WHERE ke.subject_id = ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let alias_rows = alias_statement
            .query_map([subject_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut aliases_by_entry = BTreeMap::<i64, Vec<String>>::new();
        for row in alias_rows {
            let (entry_id, alias) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            aliases_by_entry.entry(entry_id).or_default().push(alias);
        }

        for entry in &mut entries {
            entry.aliases = aliases_by_entry.remove(&entry.id).unwrap_or_default();
        }

        Ok(entries)
    }

    fn clear_subject_data(&self, subject_id: i64) -> EcoachResult<()> {
        let topic_ids =
            self.collect_ids("SELECT id FROM topics WHERE subject_id = ?1", [subject_id])?;
        let node_ids = self.collect_ids(
            "SELECT id FROM academic_nodes WHERE topic_id IN (SELECT id FROM topics WHERE subject_id = ?1)",
            [subject_id],
        )?;

        self.conn
            .execute(
                "DELETE FROM question_options WHERE question_id IN (SELECT id FROM questions WHERE subject_id = ?1)",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM question_skill_links WHERE question_id IN (SELECT id FROM questions WHERE subject_id = ?1)",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM question_glossary_links WHERE question_id IN (SELECT id FROM questions WHERE subject_id = ?1)",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute("DELETE FROM questions WHERE subject_id = ?1", [subject_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM question_families WHERE subject_id = ?1",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM knowledge_bundles WHERE subject_id = ?1",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "DELETE FROM knowledge_entries WHERE subject_id = ?1",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for topic_id in &topic_ids {
            self.conn
                .execute(
                    "DELETE FROM node_edges
                     WHERE (from_node_type = 'topic' AND from_node_id = ?1)
                        OR (to_node_type = 'topic' AND to_node_id = ?1)",
                    [topic_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        for node_id in &node_ids {
            self.conn
                .execute(
                    "DELETE FROM node_edges
                     WHERE (from_node_type = 'academic_node' AND from_node_id = ?1)
                        OR (to_node_type = 'academic_node' AND to_node_id = ?1)",
                    [node_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.conn
            .execute(
                "DELETE FROM misconception_patterns
                 WHERE topic_id IN (SELECT id FROM topics WHERE subject_id = ?1)
                    OR node_id IN (SELECT id FROM academic_nodes WHERE topic_id IN (SELECT id FROM topics WHERE subject_id = ?1))",
                [subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute("DELETE FROM topics WHERE subject_id = ?1", [subject_id])
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn collect_ids<P>(&self, sql: &str, params: P) -> EcoachResult<Vec<i64>>
    where
        P: rusqlite::Params,
    {
        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params, |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(ids)
    }

    fn upsert_curriculum_version(&self, manifest: &PackManifest) -> EcoachResult<i64> {
        let existing_id = self
            .conn
            .query_row(
                "SELECT id FROM curriculum_versions WHERE version_label = ?1 LIMIT 1",
                [manifest.curriculum_version.as_str()],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let education_stage = if manifest.grade_levels.is_empty() {
            manifest.exam_target.clone()
        } else {
            Some(manifest.grade_levels.join(","))
        };

        if let Some(id) = existing_id {
            self.conn
                .execute(
                    "UPDATE curriculum_versions
                     SET name = ?1,
                         education_stage = ?2,
                         status = 'published',
                         published_at = COALESCE(published_at, datetime('now')),
                         updated_at = datetime('now')
                     WHERE id = ?3",
                    params![manifest.curriculum_version, education_stage, id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            Ok(id)
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_versions (
                        name, country, exam_board, education_stage, version_label, status, published_at
                    ) VALUES (?1, 'GH', NULL, ?2, ?3, 'published', datetime('now'))",
                    params![manifest.curriculum_version, education_stage, manifest.curriculum_version],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    fn upsert_subject(
        &self,
        curriculum_version_id: i64,
        manifest: &PackManifest,
    ) -> EcoachResult<i64> {
        let existing_id = self
            .conn
            .query_row(
                "SELECT id FROM subjects WHERE curriculum_version_id = ?1 AND code = ?2 LIMIT 1",
                params![curriculum_version_id, manifest.subject_code],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let subject_name = subject_name_from_code(&manifest.subject_code);

        if let Some(id) = existing_id {
            self.conn
                .execute(
                    "UPDATE subjects
                     SET name = ?1, is_active = 1, updated_at = datetime('now')
                     WHERE id = ?2",
                    params![subject_name, id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            Ok(id)
        } else {
            self.conn
                .execute(
                    "INSERT INTO subjects (curriculum_version_id, code, name, display_order, is_active)
                     VALUES (?1, ?2, ?3, 0, 1)",
                    params![curriculum_version_id, manifest.subject_code, subject_name],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    fn validate_manifest_counts(
        &self,
        manifest: &PackManifest,
        counts: PackImportCounts,
    ) -> EcoachResult<()> {
        if counts.topic_count != manifest.topic_count {
            return Err(EcoachError::Validation(format!(
                "manifest topic_count {} does not match imported topic_count {}",
                manifest.topic_count, counts.topic_count
            )));
        }

        if counts.question_count != manifest.question_count {
            return Err(EcoachError::Validation(format!(
                "manifest question_count {} does not match imported question_count {}",
                manifest.question_count, counts.question_count
            )));
        }

        Ok(())
    }

    fn upsert_pack_record(
        &self,
        manifest: &PackManifest,
        manifest_json: &str,
        install_path: &str,
        status: &str,
        activated_at: Option<&str>,
        error_message: Option<&str>,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO content_packs (
                    pack_id, pack_version, subject_code, curriculum_version, status,
                    topic_count, question_count, install_path, manifest_json, activated_at, error_message
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(pack_id) DO UPDATE SET
                    pack_version = excluded.pack_version,
                    subject_code = excluded.subject_code,
                    curriculum_version = excluded.curriculum_version,
                    status = excluded.status,
                    topic_count = excluded.topic_count,
                    question_count = excluded.question_count,
                    install_path = excluded.install_path,
                    manifest_json = excluded.manifest_json,
                    installed_at = datetime('now'),
                    activated_at = excluded.activated_at,
                    error_message = excluded.error_message",
                params![
                    manifest.pack_id,
                    manifest.pack_version,
                    manifest.subject_code,
                    manifest.curriculum_version,
                    status,
                    manifest.topic_count,
                    manifest.question_count,
                    install_path,
                    manifest_json,
                    activated_at,
                    error_message,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn mark_pack_active(
        &self,
        manifest: &PackManifest,
        manifest_json: &str,
        install_path: &str,
        counts: PackImportCounts,
    ) -> EcoachResult<()> {
        let _ = counts.knowledge_entry_count;
        self.upsert_pack_record(
            manifest,
            manifest_json,
            install_path,
            "active",
            Some(""),
            None,
        )?;
        self.conn
            .execute(
                "UPDATE content_packs
                 SET topic_count = ?1,
                     question_count = ?2,
                     status = 'active',
                     activated_at = datetime('now'),
                     error_message = NULL
                 WHERE pack_id = ?3",
                params![counts.topic_count, counts.question_count, manifest.pack_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn mark_pack_failed(&self, pack_id: &str, err: &EcoachError) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE content_packs
                 SET status = 'failed',
                     error_message = ?1,
                     activated_at = NULL
                 WHERE pack_id = ?2",
                params![err.to_string(), pack_id],
            )
            .map_err(|storage_err| EcoachError::Storage(storage_err.to_string()))?;
        Ok(())
    }

    fn load_manifest(&self, pack_path: &Path) -> EcoachResult<PackManifest> {
        let manifest_path = pack_path.join("manifest.json");
        let raw = fs::read_to_string(&manifest_path).map_err(|err| {
            EcoachError::Storage(format!(
                "failed to read {}: {}",
                manifest_path.display(),
                err
            ))
        })?;
        serde_json::from_str(&raw).map_err(|err| EcoachError::Serialization(err.to_string()))
    }

    fn read_json_file<T: DeserializeOwned>(&self, file_path: &Path) -> EcoachResult<T> {
        let raw = fs::read_to_string(file_path).map_err(|err| {
            EcoachError::Storage(format!("failed to read {}: {}", file_path.display(), err))
        })?;
        serde_json::from_str(&raw).map_err(|err| {
            EcoachError::Serialization(format!("failed to parse {}: {}", file_path.display(), err))
        })
    }

    fn read_optional_json_file<T>(&self, file_path: &Path) -> EcoachResult<T>
    where
        T: DeserializeOwned + Default,
    {
        if !file_path.exists() {
            return Ok(T::default());
        }

        self.read_json_file(file_path)
    }

    fn validate_pack_shape(&self, pack_path: &Path) -> EcoachResult<()> {
        let required_paths = [
            "manifest.json",
            "curriculum",
            "curriculum/topics.json",
            "curriculum/nodes.json",
            "curriculum/edges.json",
            "curriculum/misconceptions.json",
            "curriculum/objectives.json",
            "questions",
            "questions/families.json",
            "questions/questions.json",
            "questions/intelligence.json",
            "content",
            "content/explanations.json",
            "content/glossary.json",
            "content/formulas.json",
            "content/worked_examples.json",
        ];

        for required in required_paths {
            let target = pack_path.join(required);
            if !target.exists() {
                return Err(EcoachError::Validation(format!(
                    "pack is missing required path: {}",
                    target.display()
                )));
            }
        }

        Ok(())
    }

    fn begin_transaction(&self) -> EcoachResult<()> {
        self.conn
            .execute_batch("BEGIN IMMEDIATE TRANSACTION;")
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn commit_transaction(&self) -> EcoachResult<()> {
        self.conn
            .execute_batch("COMMIT;")
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn rollback_transaction(&self) -> EcoachResult<()> {
        self.conn
            .execute_batch("ROLLBACK;")
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn append_runtime_event(&self, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, 'content_pack', ?3, ?4, ?5, ?6)",
                params![
                    event.event_id,
                    event.event_type,
                    event.aggregate_id,
                    event.trace_id,
                    payload_json,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }
}

fn resolve_edge_endpoint(
    topic_ids: &BTreeMap<String, i64>,
    node_ids: &BTreeMap<String, i64>,
    topic_code: Option<&str>,
    node_title: Option<&str>,
    node_topic_code: Option<&str>,
    direction: &str,
) -> EcoachResult<(i64, &'static str)> {
    if let Some(topic_code) = topic_code {
        let topic_id = *topic_ids.get(topic_code).ok_or_else(|| {
            EcoachError::Validation(format!(
                "{} edge endpoint references unknown topic code {}",
                direction, topic_code
            ))
        })?;
        return Ok((topic_id, "topic"));
    }

    if let Some(node_title) = node_title {
        let scoped_topic_code = node_topic_code.ok_or_else(|| {
            EcoachError::Validation(format!(
                "{} edge endpoint for node {} is missing node_topic_code",
                direction, node_title
            ))
        })?;
        let node_id = resolve_node_id(node_ids, scoped_topic_code, node_title)?;
        return Ok((node_id, "academic_node"));
    }

    Err(EcoachError::Validation(format!(
        "{} edge endpoint must provide either topic_code or node_title",
        direction
    )))
}

fn resolve_node_id(
    node_ids: &BTreeMap<String, i64>,
    topic_code: &str,
    node_title: &str,
) -> EcoachResult<i64> {
    node_ids
        .get(&scoped_key(topic_code, node_title))
        .copied()
        .ok_or_else(|| {
            EcoachError::Validation(format!(
                "unknown academic node {} in topic {}",
                node_title, topic_code
            ))
        })
}

fn resolve_misconception_id(
    misconception_ids: &BTreeMap<String, i64>,
    topic_code: &str,
    title: &str,
) -> EcoachResult<i64> {
    misconception_ids
        .get(&scoped_key(topic_code, title))
        .copied()
        .ok_or_else(|| {
            EcoachError::Validation(format!(
                "unknown misconception {} in topic {}",
                title, topic_code
            ))
        })
}

fn build_question_snapshot(
    question: &QuestionRecord,
    intelligence: Option<&QuestionIntelligenceRecord>,
) -> EcoachResult<String> {
    let mut snapshot = serde_json::Map::new();
    snapshot.insert(
        "topic_code".to_string(),
        Value::String(question.topic_code.clone()),
    );

    if let Some(subtopic_code) = &question.subtopic_code {
        snapshot.insert(
            "subtopic_code".to_string(),
            Value::String(subtopic_code.clone()),
        );
    }

    if !question.extra.is_empty() {
        snapshot.insert(
            "pack_fields".to_string(),
            serde_json::to_value(&question.extra)
                .map_err(|err| EcoachError::Serialization(err.to_string()))?,
        );
    }

    if let Some(intelligence) = intelligence {
        snapshot.insert(
            "intelligence".to_string(),
            serde_json::to_value(intelligence)
                .map_err(|err| EcoachError::Serialization(err.to_string()))?,
        );
    }

    serde_json::to_string(&snapshot).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn scoped_key(scope: &str, value: &str) -> String {
    format!("{}::{}", normalize_key(scope), normalize_key(value))
}

fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_title_key(value: &str) -> String {
    normalize_key(value)
}

fn validate_contrast_ownership(ownership_type: &str, pair_title: &str) -> EcoachResult<()> {
    match ownership_type {
        "left_only" | "right_only" | "both" | "neither" => Ok(()),
        _ => Err(EcoachError::Validation(format!(
            "contrast pair {} contains unsupported ownership type {}",
            pair_title, ownership_type
        ))),
    }
}

fn resolve_knowledge_entry_title(
    knowledge_entries: &BTreeMap<String, i64>,
    title: &str,
    topic_code: &str,
) -> EcoachResult<i64> {
    let scoped = scoped_key(topic_code, title);
    if let Some(entry_id) = knowledge_entries.get(&scoped) {
        return Ok(*entry_id);
    }

    let normalized = normalize_title_key(title);
    if let Some(entry_id) = knowledge_entries.get(&scoped_key(topic_code, &normalized)) {
        return Ok(*entry_id);
    }

    knowledge_entries.get(&normalized).copied().ok_or_else(|| {
        EcoachError::Validation(format!(
            "contrast pair references unknown knowledge entry title {} in topic {}",
            title, topic_code
        ))
    })
}

fn normalize_taxonomy_value(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace(|ch: char| !ch.is_ascii_alphanumeric(), "_")
        .split('_')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

fn subject_name_from_code(code: &str) -> String {
    match code {
        "MATH" => "Mathematics".to_string(),
        "ENG" => "English Language".to_string(),
        "SCI" => "Integrated Science".to_string(),
        "SOC" => "Social Studies".to_string(),
        "ICT" => "Information and Communication Technology".to_string(),
        _ => code.to_string(),
    }
}

fn default_option_label(index: usize) -> String {
    let label = ((index % 26) as u8 + b'A') as char;
    label.to_string()
}

fn default_topic_node_type() -> String {
    "topic".to_string()
}

fn default_family_type() -> String {
    "recurring_pattern".to_string()
}

fn default_question_format() -> String {
    "mcq".to_string()
}

fn select_knowledge_links_for_question(
    question: &QuestionKnowledgeQuestion,
    entries: &[QuestionKnowledgeEntry],
) -> Vec<ScoredKnowledgeLink> {
    let mut scored_links = Vec::new();

    for entry in entries {
        if let Some(link) = score_question_knowledge_link(question, entry) {
            scored_links.push(link);
        }
    }

    scored_links.sort_by(|left, right| {
        right
            .confidence_score
            .cmp(&left.confidence_score)
            .then_with(|| right.same_topic.cmp(&left.same_topic))
            .then_with(|| right.importance_score.cmp(&left.importance_score))
            .then_with(|| left.entry_id.cmp(&right.entry_id))
    });

    let mut selected: Vec<ScoredKnowledgeLink> = scored_links
        .iter()
        .filter(|link| link.confidence_score >= 5000)
        .take(3)
        .cloned()
        .collect();

    if selected.is_empty() {
        if let Some(fallback) = scored_links
            .iter()
            .find(|link| link.same_topic)
            .cloned()
            .or_else(|| scored_links.first().cloned())
        {
            selected.push(fallback);
        }
    }

    selected
}

fn score_question_knowledge_link(
    question: &QuestionKnowledgeQuestion,
    entry: &QuestionKnowledgeEntry,
) -> Option<ScoredKnowledgeLink> {
    let same_topic = entry.topic_id == Some(question.topic_id);
    let question_text = format!(
        "{} {}",
        question.stem,
        question.explanation_text.as_deref().unwrap_or_default()
    );
    let question_tokens = normalized_tokens(&question_text);
    let skill_tokens =
        normalized_tokens(question.primary_skill_title.as_deref().unwrap_or_default());
    let entry_tokens = knowledge_entry_tokens(entry);

    let mut score = 0;
    let mut reasons = Vec::new();

    if same_topic {
        score += 4000;
        reasons.push("shared_topic");
    }

    let skill_overlap = overlap_count(&skill_tokens, &entry_tokens);
    if skill_overlap > 0 {
        score += 3000 + (skill_overlap as i64 * 700).min(2100);
        reasons.push("skill_overlap");
    }

    let question_overlap = overlap_count(&question_tokens, &entry_tokens);
    if question_overlap > 0 {
        score += 1000 + (question_overlap as i64 * 250).min(1500);
        reasons.push("language_overlap");
    }

    if skill_title_matches_entry(question.primary_skill_title.as_deref(), entry) {
        score += 2500;
        reasons.push("title_match");
    }

    match entry.entry_type.as_str() {
        "definition" if skill_overlap > 0 || question_overlap > 0 => {
            score += 1000;
            reasons.push("definition_support");
        }
        "worked_example" if same_topic => {
            score += 750;
            reasons.push("worked_example_support");
        }
        "formula" if skill_overlap > 0 || question_overlap > 0 => {
            score += 750;
            reasons.push("formula_support");
        }
        _ => {}
    }

    if score == 0 {
        return None;
    }

    Some(ScoredKnowledgeLink {
        entry_id: entry.id,
        relation_type: relation_type_for_entry(entry.entry_type.as_str()),
        confidence_score: score.min(10000),
        link_reason: reasons.join("+"),
        importance_score: entry.importance_score,
        same_topic,
    })
}

fn relation_type_for_entry(entry_type: &str) -> &'static str {
    match entry_type {
        "definition" => "definition_support",
        "worked_example" => "worked_example_support",
        "formula" => "formula_support",
        _ => "repair_support",
    }
}

fn skill_title_matches_entry(skill_title: Option<&str>, entry: &QuestionKnowledgeEntry) -> bool {
    let Some(skill_title) = skill_title else {
        return false;
    };
    let skill_phrase = normalize_phrase(skill_title);
    if skill_phrase.is_empty() {
        return false;
    }

    knowledge_entry_phrases(entry).into_iter().any(|phrase| {
        !phrase.is_empty()
            && (phrase == skill_phrase
                || phrase.contains(&skill_phrase)
                || skill_phrase.contains(&phrase))
    })
}

fn knowledge_entry_tokens(entry: &QuestionKnowledgeEntry) -> BTreeSet<String> {
    let mut tokens = normalized_tokens(&entry.title);

    if let Some(canonical_name) = &entry.canonical_name {
        tokens.extend(normalized_tokens(canonical_name));
    }

    if let Some(short_text) = &entry.short_text {
        tokens.extend(normalized_tokens(short_text));
    }

    for alias in &entry.aliases {
        tokens.extend(normalized_tokens(alias));
    }

    tokens
}

fn knowledge_entry_phrases(entry: &QuestionKnowledgeEntry) -> Vec<String> {
    let mut phrases = Vec::new();
    phrases.push(normalize_phrase(&entry.title));

    if let Some(canonical_name) = &entry.canonical_name {
        phrases.push(normalize_phrase(canonical_name));
    }

    for alias in &entry.aliases {
        phrases.push(normalize_phrase(alias));
    }

    phrases
}

fn overlap_count(left: &BTreeSet<String>, right: &BTreeSet<String>) -> usize {
    left.intersection(right).count()
}

fn normalized_tokens(value: &str) -> BTreeSet<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter_map(|token| normalize_token(token))
        .collect()
}

fn normalize_phrase(value: &str) -> String {
    normalized_tokens(value)
        .into_iter()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_token(token: &str) -> Option<String> {
    let lower = token.trim().to_ascii_lowercase();
    if lower.len() < 2 {
        return None;
    }

    let singular = if lower.ends_with('s') && lower.len() > 3 {
        lower.trim_end_matches('s').to_string()
    } else {
        lower
    };

    if singular.is_empty() {
        None
    } else {
        Some(singular)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_glossary::GlossaryService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn installs_sample_pack_into_runtime_tables() {
        let conn = open_test_database();
        let service = PackService::new(&conn);
        let pack_path = sample_pack_path();

        service
            .install_pack(&pack_path)
            .expect("sample pack should install");

        let status: String = conn
            .query_row(
                "SELECT status FROM content_packs WHERE pack_id = 'math-bece-sample-v1'",
                [],
                |row| row.get(0),
            )
            .expect("pack status should exist");
        let topic_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM topics", [], |row| row.get(0))
            .expect("topic count should be queryable");
        let question_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM questions", [], |row| row.get(0))
            .expect("question count should be queryable");
        let knowledge_entry_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_entries", [], |row| {
                row.get(0)
            })
            .expect("knowledge entry count should be queryable");
        let subject_name: String = conn
            .query_row("SELECT name FROM subjects WHERE code = 'MATH'", [], |row| {
                row.get(0)
            })
            .expect("subject should exist");
        let runtime_event_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events WHERE aggregate_kind = 'content_pack' AND aggregate_id = 'math-bece-sample-v1'",
                [],
                |row| row.get(0),
            )
            .expect("runtime event count should be queryable");
        let glossary_link_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM question_glossary_links", [], |row| {
                row.get(0)
            })
            .expect("question glossary link count should be queryable");
        let first_question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("first question id should exist");
        let glossary_service = GlossaryService::new(&conn);
        let linked_entries = glossary_service
            .list_entries_for_question(first_question_id)
            .expect("linked glossary entries should be retrievable");
        let contrast_pair_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM contrast_pairs", [], |row| row.get(0))
            .expect("contrast pair count should be queryable");
        let contrast_atom_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM contrast_evidence_atoms", [], |row| {
                row.get(0)
            })
            .expect("contrast atom count should be queryable");

        assert_eq!(status, "active");
        assert_eq!(topic_count, 2);
        assert_eq!(question_count, 2);
        assert_eq!(knowledge_entry_count, 4);
        assert_eq!(subject_name, "Mathematics");
        assert_eq!(runtime_event_count, 1);
        assert!(glossary_link_count >= 2);
        assert!(!linked_entries.is_empty());
        assert!(linked_entries[0].is_primary);
        assert_eq!(contrast_pair_count, 1);
        assert_eq!(contrast_atom_count, 6);
    }

    #[test]
    fn reinstalling_sample_pack_replaces_subject_slice_without_duplication() {
        let conn = open_test_database();
        let service = PackService::new(&conn);
        let pack_path = sample_pack_path();

        service
            .install_pack(&pack_path)
            .expect("first install should succeed");
        service
            .install_pack(&pack_path)
            .expect("reinstall should also succeed");

        let topic_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM topics", [], |row| row.get(0))
            .expect("topic count should be queryable");
        let question_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM questions", [], |row| row.get(0))
            .expect("question count should be queryable");
        let family_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM question_families", [], |row| {
                row.get(0)
            })
            .expect("family count should be queryable");

        assert_eq!(topic_count, 2);
        assert_eq!(question_count, 2);
        assert_eq!(family_count, 1);
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
