use std::collections::{BTreeMap, BTreeSet};

use ecoach_questions::{
    QuestionGenerationRequestInput, QuestionReactor, QuestionSelectionRequest, QuestionSelector,
    QuestionSlotSpec, QuestionVariantMode,
};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{ContentStrategyRegistry, ResourceReadinessService, TopicResourceReadiness};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicResourceIntelligenceSnapshot {
    pub subject_id: i64,
    pub subject_code: String,
    pub subject_name: String,
    pub topic_id: i64,
    pub topic_name: String,
    pub concept_atom_count: i64,
    pub indexed_resource_count: i64,
    pub evidence_block_count: i64,
    pub overall_coverage_bp: BasisPoints,
    pub question_coverage_bp: BasisPoints,
    pub teaching_coverage_bp: BasisPoints,
    pub assessment_coverage_bp: BasisPoints,
    pub confidence_coverage_bp: BasisPoints,
    pub coverage_color: String,
    pub active_gap_ticket_count: i64,
    pub missing_components: Vec<String>,
    pub ready_modes: Vec<String>,
    pub curriculum_interpretation_available: bool,
    pub official_document_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceObjectiveProfile {
    pub objective_code: String,
    pub objective_label: String,
    pub session_mode: String,
    pub learner_need: String,
    pub subject_id: Option<i64>,
    pub subject_code: Option<String>,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
    pub prompt_text: Option<String>,
    pub preferred_formats: Vec<String>,
    pub time_budget_minutes: i64,
    pub target_difficulty_bp: BasisPoints,
    pub curriculum_lens: Vec<String>,
    pub pedagogical_lens: Vec<String>,
    pub learner_lens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCandidate {
    pub resource_type: String,
    pub resource_id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub role_code: String,
    pub confidence_tier: String,
    pub intrinsic_quality_bp: BasisPoints,
    pub contextual_fitness_bp: BasisPoints,
    pub overall_score_bp: BasisPoints,
    pub rationale: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAssemblyStep {
    pub step_order: i64,
    pub role_code: String,
    pub title: String,
    pub purpose: String,
    pub estimated_minutes: i64,
    pub resource_type: Option<String>,
    pub resource_id: Option<i64>,
    pub generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicabilityOption {
    pub code: String,
    pub label: String,
    pub rationale: String,
    pub topic_id: Option<i64>,
    pub objective_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicabilityPrompt {
    pub check_id: Option<i64>,
    pub reason: String,
    pub prompt_text: String,
    pub options: Vec<ApplicabilityOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedGapArtifact {
    pub resource_type: String,
    pub resource_id: i64,
    pub artifact_id: Option<i64>,
    pub artifact_version_id: Option<i64>,
    pub role_code: String,
    pub title: String,
    pub confidence_tier: String,
    pub provenance_ref: String,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOrchestrationRequest {
    pub student_id: i64,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub prompt_text: Option<String>,
    pub objective_code: Option<String>,
    pub session_mode: Option<String>,
    pub preferred_formats: Vec<String>,
    pub time_budget_minutes: Option<i64>,
    pub target_difficulty_bp: Option<BasisPoints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceOrchestrationResult {
    pub run_id: i64,
    pub topic_snapshot: Option<TopicResourceIntelligenceSnapshot>,
    pub objective: ResourceObjectiveProfile,
    pub curriculum_context: Value,
    pub ambiguity_prompt: Option<ApplicabilityPrompt>,
    pub selected_candidates: Vec<ResourceCandidate>,
    pub generated_artifacts: Vec<GeneratedGapArtifact>,
    pub assembled_steps: Vec<ResourceAssemblyStep>,
    pub gap_ticket_ids: Vec<i64>,
    pub confidence_score_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceApplicabilityResolution {
    pub check_id: i64,
    pub selected_option_code: String,
    pub selected_option_label: String,
    pub follow_up: ResourceOrchestrationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordResourceLearningInput {
    pub run_id: i64,
    pub session_id: Option<i64>,
    pub outcome_status: String,
    pub usefulness_bp: BasisPoints,
    pub confidence_shift_bp: i64,
    pub speed_shift_bp: i64,
    pub accuracy_shift_bp: i64,
    pub learner_feedback: Value,
    pub still_struggling: Vec<String>,
    pub effective_resource_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLearningRecord {
    pub id: i64,
    pub run_id: i64,
    pub outcome_status: String,
    pub usefulness_bp: BasisPoints,
    pub created_at: String,
}

pub struct ResourceIntelligenceService<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone)]
struct TopicContext {
    subject_id: i64,
    subject_code: String,
    subject_name: String,
    topic_id: i64,
    topic_name: String,
    topic_description: Option<String>,
}

#[derive(Debug, Clone)]
struct NodeRow {
    id: i64,
    node_type: String,
    canonical_title: String,
    exam_relevance_score: i64,
    foundation_weight: i64,
    primary_content_type: Option<String>,
    preferred_strategies_json: Option<String>,
}

#[derive(Debug, Clone)]
struct IndexedResourceRow {
    resource_id: i64,
    resource_type: String,
    title: String,
    subtitle: Option<String>,
    topic_id: Option<i64>,
    difficulty_bp: i64,
    exam_relevance_bp: i64,
    teach_suitability_bp: i64,
    test_suitability_bp: i64,
    pressure_suitability_bp: i64,
    confidence_tier: String,
    teacher_verified: bool,
    student_success_rate_bp: Option<i64>,
    metadata: Value,
}

#[derive(Debug, Clone)]
struct TopicMatch {
    subject_id: i64,
    subject_code: String,
    topic_id: i64,
    topic_name: String,
    score: BasisPoints,
}

#[derive(Debug, Clone, Default)]
struct LearnerSignals {
    mastery_score: i64,
    gap_score: i64,
    fragility_score: i64,
    decay_risk: i64,
    wrong_answer_count: i64,
    recent_actions: Vec<String>,
}

#[derive(Debug, Clone)]
struct SelectedRole {
    role_code: String,
    purpose: String,
    estimated_minutes: i64,
    candidate: Option<ResourceCandidate>,
}

impl<'a> ResourceIntelligenceService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn sync_topic_resource_intelligence(
        &self,
        topic_id: i64,
    ) -> EcoachResult<TopicResourceIntelligenceSnapshot> {
        let topic = self.load_topic_context(topic_id)?;
        let nodes = self.load_topic_nodes(topic_id)?;
        self.ensure_curriculum_interpretation(&topic, &nodes)?;
        self.ensure_student_facing_curriculum(&topic, &nodes)?;
        self.sync_concept_atoms(&topic, &nodes)?;
        self.sync_evidence_blocks(&topic, &nodes)?;
        self.sync_resource_index(&topic, &nodes)?;
        self.sync_concept_coverage(&topic, &nodes)?;

        let readiness = ResourceReadinessService::new(self.conn)
            .get_topic_readiness(topic_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!("topic {} readiness missing", topic_id))
            })?;
        self.sync_topic_health(&topic, &readiness)?;
        self.refresh_gap_tickets(&topic, &nodes, &readiness)?;
        self.build_topic_snapshot(&topic, &readiness)
    }

    fn load_topic_context(&self, topic_id: i64) -> EcoachResult<TopicContext> {
        self.conn
            .query_row(
                "SELECT s.id, s.code, s.name, t.id, t.name, t.description
                 FROM topics t
                 INNER JOIN subjects s ON s.id = t.subject_id
                 WHERE t.id = ?1",
                [topic_id],
                |row| {
                    Ok(TopicContext {
                        subject_id: row.get(0)?,
                        subject_code: row.get(1)?,
                        subject_name: row.get(2)?,
                        topic_id: row.get(3)?,
                        topic_name: row.get(4)?,
                        topic_description: row.get(5)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_topic_nodes(&self, topic_id: i64) -> EcoachResult<Vec<NodeRow>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, node_type, canonical_title, exam_relevance_score,
                        foundation_weight, primary_content_type, preferred_strategies_json
                 FROM academic_nodes
                 WHERE topic_id = ?1 AND is_active = 1
                 ORDER BY foundation_weight DESC, exam_relevance_score DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], |row| {
                Ok(NodeRow {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    canonical_title: row.get(2)?,
                    exam_relevance_score: row.get(3)?,
                    foundation_weight: row.get(4)?,
                    primary_content_type: row.get(5)?,
                    preferred_strategies_json: row.get(6)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(nodes)
    }

    fn ensure_curriculum_interpretation(
        &self,
        topic: &TopicContext,
        nodes: &[NodeRow],
    ) -> EcoachResult<()> {
        let knowledge_points = nodes
            .iter()
            .map(|node| node.canonical_title.clone())
            .collect::<Vec<_>>();
        let skills_involved = nodes
            .iter()
            .map(|node| node.node_type.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let question_families = self.list_string_column(
            "SELECT family_name
             FROM question_families
             WHERE topic_id = ?1
             ORDER BY updated_at DESC, id ASC
             LIMIT 8",
            topic.topic_id,
        )?;
        let worked_example_templates = self.list_string_column(
            "SELECT title
             FROM knowledge_entries
             WHERE topic_id = ?1
               AND entry_type IN ('worked_example', 'example')
               AND status = 'active'
             ORDER BY importance_score DESC, id ASC
             LIMIT 8",
            topic.topic_id,
        )?;
        let prerequisite_nodes = self.list_prerequisite_titles(topic.topic_id)?;
        let cognitive_verb = self
            .conn
            .query_row(
                "SELECT cognitive_level
                 FROM learning_objectives
                 WHERE topic_id = ?1
                 ORDER BY display_order ASC, id ASC
                 LIMIT 1",
                [topic.topic_id],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .flatten()
            .unwrap_or_else(|| "understanding".to_string());
        let common_misconceptions = self.list_string_column(
            "SELECT title
             FROM misconception_patterns
             WHERE topic_id = ?1 AND is_active = 1
             ORDER BY severity DESC, id ASC
             LIMIT 8",
            topic.topic_id,
        )?;
        let teaching_strategies = self.merge_strategy_codes(nodes);
        let memory_tags = nodes
            .iter()
            .map(|node| {
                format!(
                    "{}:{}",
                    node.node_type,
                    normalize_phrase(&node.canonical_title)
                )
            })
            .take(8)
            .collect::<Vec<_>>();

        self.conn
            .execute(
                "INSERT INTO curriculum_interpretations (
                    topic_id, friendly_name, knowledge_points_json, skills_involved_json,
                    cognitive_verb, expected_evidence_type, common_misconceptions_json,
                    prerequisite_nodes_json, dependent_nodes_json, difficulty_ladder_json,
                    teaching_strategies_json, question_families_json,
                    worked_example_templates_json, memory_recall_tags_json,
                    local_context_examples_json, bece_mapping_json, approval_status,
                    updated_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, 'mixed', ?6, ?7, '[]', '[]',
                    ?8, ?9, ?10, ?11, '[]', ?12, 'draft', datetime('now')
                 )
                 ON CONFLICT(topic_id) DO UPDATE SET
                    friendly_name = excluded.friendly_name,
                    knowledge_points_json = excluded.knowledge_points_json,
                    skills_involved_json = excluded.skills_involved_json,
                    cognitive_verb = excluded.cognitive_verb,
                    common_misconceptions_json = excluded.common_misconceptions_json,
                    prerequisite_nodes_json = excluded.prerequisite_nodes_json,
                    teaching_strategies_json = excluded.teaching_strategies_json,
                    question_families_json = excluded.question_families_json,
                    worked_example_templates_json = excluded.worked_example_templates_json,
                    memory_recall_tags_json = excluded.memory_recall_tags_json,
                    bece_mapping_json = excluded.bece_mapping_json,
                    updated_at = datetime('now')",
                params![
                    topic.topic_id,
                    topic.topic_name,
                    to_json(&knowledge_points)?,
                    to_json(&skills_involved)?,
                    cognitive_verb,
                    to_json(&common_misconceptions)?,
                    to_json(&prerequisite_nodes)?,
                    to_json(&teaching_strategies)?,
                    to_json(&question_families)?,
                    to_json(&worked_example_templates)?,
                    to_json(&memory_tags)?,
                    json!({
                        "subject_code": topic.subject_code,
                        "topic_name": topic.topic_name,
                    })
                    .to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn ensure_student_facing_curriculum(
        &self,
        topic: &TopicContext,
        nodes: &[NodeRow],
    ) -> EcoachResult<()> {
        let what_to_know = nodes
            .iter()
            .take(3)
            .map(|node| node.canonical_title.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let what_to_do = self
            .list_string_column(
                "SELECT simplified_text
                 FROM learning_objectives
                 WHERE topic_id = ?1
                   AND simplified_text IS NOT NULL
                 ORDER BY display_order ASC, id ASC
                 LIMIT 3",
                topic.topic_id,
            )?
            .join("; ");
        let examples = self.list_string_column(
            "SELECT title
             FROM knowledge_entries
             WHERE topic_id = ?1
               AND entry_type IN ('example', 'worked_example', 'application')
               AND status = 'active'
             ORDER BY importance_score DESC, id ASC
             LIMIT 5",
            topic.topic_id,
        )?;
        self.conn
            .execute(
                "INSERT INTO student_facing_curriculum (
                    topic_id, simplified_name, simplified_description,
                    what_to_know, what_to_do, examples_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(topic_id) DO UPDATE SET
                    simplified_name = excluded.simplified_name,
                    simplified_description = excluded.simplified_description,
                    what_to_know = excluded.what_to_know,
                    what_to_do = excluded.what_to_do,
                    examples_json = excluded.examples_json",
                params![
                    topic.topic_id,
                    topic.topic_name,
                    topic.topic_description,
                    if what_to_know.is_empty() {
                        None::<String>
                    } else {
                        Some(what_to_know)
                    },
                    if what_to_do.is_empty() {
                        None::<String>
                    } else {
                        Some(what_to_do)
                    },
                    to_json(&examples)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn merge_strategy_codes(&self, nodes: &[NodeRow]) -> Vec<String> {
        let registry = ContentStrategyRegistry::core();
        let mut codes = BTreeSet::new();
        for node in nodes {
            let node_type = node
                .primary_content_type
                .as_deref()
                .unwrap_or(node.node_type.as_str());
            for strategy in &registry.strategies {
                if strategy.node_type == node_type {
                    for code in &strategy.strategy_families {
                        codes.insert(code.clone());
                    }
                }
            }
            if let Some(preferred_json) = &node.preferred_strategies_json {
                if let Ok(extra_codes) = serde_json::from_str::<Vec<String>>(preferred_json) {
                    for code in extra_codes {
                        codes.insert(code);
                    }
                }
            }
        }
        codes.into_iter().collect()
    }

    fn sync_concept_atoms(&self, topic: &TopicContext, nodes: &[NodeRow]) -> EcoachResult<()> {
        for node in nodes {
            let atom_type = map_node_to_atom_type(node);
            self.insert_atom_if_missing(
                node.id,
                atom_type,
                &node.canonical_title,
                "text",
                node.foundation_weight,
                node.exam_relevance_score,
            )?;
        }

        let mut objectives = self
            .conn
            .prepare(
                "SELECT objective_text, simplified_text
                 FROM learning_objectives
                 WHERE topic_id = ?1
                 ORDER BY display_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let objective_rows = objectives
            .query_map([topic.topic_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in objective_rows {
            let (objective_text, simplified_text) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let node_id = resolve_best_node_id(nodes, &objective_text);
            self.insert_atom_if_missing(
                node_id,
                "objective",
                &objective_text,
                "text",
                7_000,
                7_000,
            )?;
            if let Some(simple) = simplified_text {
                self.insert_atom_if_missing(
                    node_id,
                    "objective",
                    &simple,
                    "flashcard",
                    6_500,
                    6_500,
                )?;
            }
        }

        let mut misconception_stmt = self
            .conn
            .prepare(
                "SELECT node_id, title, misconception_statement, correction_hint, severity
                 FROM misconception_patterns
                 WHERE topic_id = ?1 AND is_active = 1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let misconception_rows = misconception_stmt
            .query_map([topic.topic_id], |row| {
                Ok((
                    row.get::<_, Option<i64>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in misconception_rows {
            let (node_id, title, statement, hint, severity) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let resolved_node_id = node_id.unwrap_or_else(|| resolve_best_node_id(nodes, &title));
            self.insert_atom_if_missing(
                resolved_node_id,
                "misconception",
                &statement,
                "diagnostic_mode",
                severity,
                severity,
            )?;
            if let Some(hint_text) = hint {
                self.insert_atom_if_missing(
                    resolved_node_id,
                    "hint",
                    &hint_text,
                    "teach_mode",
                    severity,
                    severity,
                )?;
            }
        }

        let mut entry_stmt = self
            .conn
            .prepare(
                "SELECT title, entry_type, short_text, full_text, importance_score, difficulty_level
                 FROM knowledge_entries
                 WHERE topic_id = ?1
                   AND status = 'active'",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let entry_rows = entry_stmt
            .query_map([topic.topic_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in entry_rows {
            let (title, entry_type, short_text, full_text, importance_score, difficulty_level) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let node_id = resolve_best_node_id(nodes, &title);
            let atom_type = map_entry_to_atom_type(&entry_type);
            if let Some(text) = short_text
                .as_deref()
                .filter(|value| !value.trim().is_empty())
            {
                self.insert_atom_if_missing(
                    node_id,
                    atom_type,
                    text,
                    entry_representation(&entry_type),
                    difficulty_level,
                    importance_score,
                )?;
            }
            if let Some(text) = full_text
                .as_deref()
                .filter(|value| !value.trim().is_empty())
            {
                self.insert_atom_if_missing(
                    node_id,
                    atom_type,
                    text,
                    "text",
                    difficulty_level,
                    importance_score,
                )?;
            }
        }

        let mut teach_stmt = self
            .conn
            .prepare(
                "SELECT node_id, hero_summary, why_it_matters, simple_explanation
                 FROM teach_explanations
                 WHERE node_id IN (
                    SELECT id FROM academic_nodes WHERE topic_id = ?1 AND is_active = 1
                 )",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let teach_rows = teach_stmt
            .query_map([topic.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in teach_rows {
            let (node_id, hero_summary, why_it_matters, simple_explanation) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            for text in [hero_summary, why_it_matters, simple_explanation]
                .into_iter()
                .flatten()
                .filter(|value| !value.trim().is_empty())
            {
                self.insert_atom_if_missing(
                    node_id,
                    "explanation",
                    &text,
                    "teach_mode",
                    7_200,
                    7_400,
                )?;
            }
        }

        Ok(())
    }

    fn sync_evidence_blocks(&self, topic: &TopicContext, nodes: &[NodeRow]) -> EcoachResult<()> {
        let curriculum_id = self
            .conn
            .query_row(
                "SELECT id
                 FROM curriculum_interpretations
                 WHERE topic_id = ?1",
                [topic.topic_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if let Some(interp_id) = curriculum_id {
            self.insert_evidence_if_missing(
                "curriculum_interpretation",
                Some(interp_id),
                topic.topic_id,
                Some(resolve_best_node_id(nodes, &topic.topic_name)),
                &format!("Curriculum interpretation exists for {}.", topic.topic_name),
                Some("Derived from topic nodes, learning objectives, and question families."),
                7_400,
                7_200,
                7_000,
                6_500,
                "verified",
            )?;
        }

        let mut entry_stmt = self
            .conn
            .prepare(
                "SELECT id, title, COALESCE(full_text, short_text, simple_text, exam_text)
                 FROM knowledge_entries
                 WHERE topic_id = ?1
                   AND status = 'active'",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let entry_rows = entry_stmt
            .query_map([topic.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in entry_rows {
            let (entry_id, title, text) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.insert_evidence_if_missing(
                "knowledge_entry",
                Some(entry_id),
                topic.topic_id,
                Some(resolve_best_node_id(nodes, &title)),
                &title,
                text.as_deref(),
                8_200,
                7_400,
                7_000,
                7_500,
                "verified",
            )?;
        }

        let official_document_count = self.count_official_documents(topic.topic_name.as_str())?;
        if official_document_count > 0 {
            self.insert_evidence_if_missing(
                "official_document",
                None,
                topic.topic_id,
                Some(resolve_best_node_id(nodes, &topic.topic_name)),
                &format!(
                    "{} has official supporting document coverage.",
                    topic.topic_name
                ),
                Some("Trusted document mining mentions this topic."),
                7_600,
                7_200,
                7_000,
                7_400,
                "verified",
            )?;
        }
        Ok(())
    }

    fn sync_resource_index(&self, topic: &TopicContext, nodes: &[NodeRow]) -> EcoachResult<()> {
        let content_type_map = self.load_content_type_registry()?;
        let mut question_stmt = self
            .conn
            .prepare(
                "SELECT id, subject_id, topic_id, family_id, stem, difficulty_level,
                        estimated_time_seconds, source_type, primary_content_type,
                        classification_confidence_bp, human_verified, exam_year
                 FROM questions
                 WHERE topic_id = ?1 AND is_active = 1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let question_rows = question_stmt
            .query_map([topic.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, Option<i64>>(11)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in question_rows {
            let (
                question_id,
                subject_id,
                topic_id,
                _family_id,
                stem,
                difficulty_level,
                estimated_time_seconds,
                source_type,
                primary_content_type,
                classification_confidence_bp,
                human_verified,
                exam_year,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let content_type_code = primary_content_type
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("application");
            let confidence_tier = question_confidence_tier(
                source_type.as_deref(),
                human_verified == 1,
                classification_confidence_bp.unwrap_or(0),
            );
            self.upsert_resource_index(
                question_id,
                "question",
                subject_id,
                Some(topic_id),
                None,
                Some(resolve_best_node_id(nodes, &stem)),
                content_type_map.get(content_type_code).copied(),
                difficulty_level,
                question_exam_relevance(source_type.as_deref(), exam_year),
                3_800,
                8_800,
                if estimated_time_seconds <= 60 {
                    8_400
                } else {
                    6_200
                },
                source_type.as_deref(),
                confidence_tier,
                human_verified == 1,
                None,
            )?;
        }

        let mut entry_stmt = self
            .conn
            .prepare(
                "SELECT id, subject_id, topic_id, subtopic_id, entry_type, title,
                        difficulty_level, importance_score
                 FROM knowledge_entries
                 WHERE topic_id = ?1
                   AND status = 'active'",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let entry_rows = entry_stmt
            .query_map([topic.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in entry_rows {
            let (
                entry_id,
                subject_id,
                topic_id,
                subtopic_id,
                entry_type,
                title,
                difficulty_level,
                importance_score,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let resource_type = if entry_type == "worked_example" || entry_type == "example" {
                "worked_example"
            } else {
                "glossary_entry"
            };
            let content_type_code = entry_content_type_code(&entry_type);
            self.upsert_resource_index(
                entry_id,
                resource_type,
                subject_id.unwrap_or(topic.subject_id),
                topic_id,
                subtopic_id,
                Some(resolve_best_node_id(nodes, &title)),
                content_type_map.get(content_type_code).copied(),
                difficulty_level,
                importance_score,
                glossary_teach_suitability(&entry_type),
                glossary_test_suitability(&entry_type),
                glossary_pressure_suitability(&entry_type),
                Some("knowledge_entry"),
                "syllabus_aligned",
                false,
                None,
            )?;
        }

        let mut teach_stmt = self
            .conn
            .prepare(
                "SELECT id, node_id, explanation_level
                 FROM teach_explanations
                 WHERE node_id IN (
                    SELECT id FROM academic_nodes WHERE topic_id = ?1 AND is_active = 1
                 )",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let teach_rows = teach_stmt
            .query_map([topic.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in teach_rows {
            let (teach_id, node_id, explanation_level) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let content_type_code = nodes
                .iter()
                .find(|node| node.id == node_id)
                .and_then(|node| node.primary_content_type.as_deref())
                .unwrap_or("concept");
            self.upsert_resource_index(
                teach_id,
                "teach_explanation",
                topic.subject_id,
                Some(topic.topic_id),
                None,
                Some(node_id),
                content_type_map.get(content_type_code).copied(),
                explanation_level_difficulty(&explanation_level),
                7_800,
                9_200,
                4_000,
                4_200,
                Some("teach_explanation"),
                "teacher_authored",
                true,
                None,
            )?;
        }

        let mut artifact_stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.artifact_type, a.subject_id, a.topic_id,
                        av.id, av.version_no, av.quality_score_bp
                 FROM artifacts a
                 INNER JOIN artifact_versions av ON av.id = a.current_version_id
                 WHERE a.topic_id = ?1
                   AND a.lifecycle_state IN ('approved', 'live')",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let artifact_rows = artifact_stmt
            .query_map([topic.topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        for row in artifact_rows {
            let (
                artifact_id,
                artifact_type,
                subject_id,
                topic_id,
                artifact_version_id,
                version_no,
                quality_score_bp,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let Some(resource_type) = artifact_resource_type(&artifact_type) else {
                continue;
            };
            self.upsert_resource_index(
                artifact_version_id,
                resource_type,
                subject_id.unwrap_or(topic.subject_id),
                topic_id,
                None,
                Some(resolve_best_node_id(nodes, &artifact_type)),
                None,
                5_200,
                quality_score_bp.unwrap_or(6_200),
                if resource_type == "note" {
                    8_000
                } else {
                    6_400
                },
                if resource_type == "drill" {
                    8_600
                } else {
                    5_200
                },
                if resource_type == "drill" {
                    8_000
                } else {
                    4_500
                },
                Some("generated_gap_fill"),
                "ai_generated",
                false,
                Some(json!({ "artifact_id": artifact_id, "version_no": version_no }).to_string()),
            )?;
        }

        Ok(())
    }

    fn sync_concept_coverage(&self, topic: &TopicContext, nodes: &[NodeRow]) -> EcoachResult<()> {
        for node in nodes {
            let definition_like =
                self.count_atoms_for_node(node.id, &["definition", "explanation", "concept"])?;
            let example_like = self.count_atoms_for_node(
                node.id,
                &["example", "counterexample", "application", "worked_example"],
            )?;
            let objective_like = self.count_atoms_for_node(node.id, &["objective"])?;
            let misconception_like =
                self.count_atoms_for_node(node.id, &["misconception", "hint"])?;
            let formula_like =
                self.count_atoms_for_node(node.id, &["formula", "rule", "theorem"])?;
            let question_count = self.count_questions_for_node(topic.topic_id, node.id)?;
            let teach_count = self.count_teach_resources_for_node(node.id)?;
            let validation_count = self.count_validation_evidence(topic.topic_id, node.id)?;
            let confidence_bp = self.compute_node_confidence(topic.topic_id, node.id)?;

            let knowledge_coverage_bp = clamp_bp(
                (if definition_like > 0 { 3_000 } else { 0 })
                    + (if example_like > 0 { 2_400 } else { 0 })
                    + (if objective_like > 0 { 1_600 } else { 0 })
                    + (if misconception_like > 0 { 1_500 } else { 0 })
                    + (if formula_like > 0 { 1_500 } else { 0 }),
            );
            let question_coverage_bp = coverage_from_count(question_count, 4);
            let teaching_coverage_bp = clamp_bp(
                (if teach_count > 0 { 6_500 } else { 0 })
                    + (if definition_like > 0 { 2_000 } else { 0 })
                    + (if example_like > 0 { 1_500 } else { 0 }),
            );
            let assessment_coverage_bp = clamp_bp(
                (question_coverage_bp as i64 * 70 / 100)
                    + (if example_like > 0 { 1_800 } else { 0 })
                    + (if misconception_like > 0 { 1_200 } else { 0 }),
            );
            let validation_coverage_bp = coverage_from_count(validation_count, 3);
            let overall_coverage_bp = clamp_bp(
                (knowledge_coverage_bp as i64
                    + question_coverage_bp as i64
                    + teaching_coverage_bp as i64
                    + assessment_coverage_bp as i64
                    + validation_coverage_bp as i64
                    + confidence_bp as i64)
                    / 6,
            );
            let coverage_color = coverage_color(overall_coverage_bp).to_string();

            self.conn
                .execute(
                    "INSERT INTO concept_coverage_scores (
                        node_id, knowledge_coverage_bp, question_coverage_bp,
                        teaching_coverage_bp, assessment_coverage_bp,
                        validation_coverage_bp, confidence_coverage_bp,
                        overall_coverage_bp, coverage_color, computed_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
                     ON CONFLICT(node_id) DO UPDATE SET
                        knowledge_coverage_bp = excluded.knowledge_coverage_bp,
                        question_coverage_bp = excluded.question_coverage_bp,
                        teaching_coverage_bp = excluded.teaching_coverage_bp,
                        assessment_coverage_bp = excluded.assessment_coverage_bp,
                        validation_coverage_bp = excluded.validation_coverage_bp,
                        confidence_coverage_bp = excluded.confidence_coverage_bp,
                        overall_coverage_bp = excluded.overall_coverage_bp,
                        coverage_color = excluded.coverage_color,
                        computed_at = datetime('now')",
                    params![
                        node.id,
                        knowledge_coverage_bp as i64,
                        question_coverage_bp as i64,
                        teaching_coverage_bp as i64,
                        assessment_coverage_bp as i64,
                        validation_coverage_bp as i64,
                        confidence_bp as i64,
                        overall_coverage_bp as i64,
                        coverage_color,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn sync_topic_health(
        &self,
        topic: &TopicContext,
        readiness: &TopicResourceReadiness,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO topic_health (
                    topic_id, package_completeness_bp, quality_score_bp,
                    live_health_state, missing_components_json, last_refresh_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
                 ON CONFLICT(topic_id) DO UPDATE SET
                    package_completeness_bp = excluded.package_completeness_bp,
                    quality_score_bp = excluded.quality_score_bp,
                    live_health_state = excluded.live_health_state,
                    missing_components_json = excluded.missing_components_json,
                    last_refresh_at = datetime('now')",
                params![
                    topic.topic_id,
                    readiness.readiness_score as i64,
                    compute_topic_quality(readiness) as i64,
                    live_health_from_readiness(readiness),
                    to_json(&readiness.missing_resources)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn orchestrate_resources(
        &self,
        request: ResourceOrchestrationRequest,
    ) -> EcoachResult<ResourceOrchestrationResult> {
        let objective = self.interpret_objective(&request)?;
        let topic_snapshot = objective
            .topic_id
            .map(|topic_id| self.sync_topic_resource_intelligence(topic_id))
            .transpose()?;
        let curriculum_context = self.load_curriculum_context(objective.topic_id)?;
        let recipe = self.build_recipe(&objective, topic_snapshot.as_ref());
        let ambiguity_prompt = self.detect_ambiguity(&objective, &request)?;
        let run_id = self.insert_run(
            &request,
            &objective,
            &curriculum_context,
            &recipe,
            ambiguity_prompt.is_some(),
        )?;

        if let Some(mut prompt) = ambiguity_prompt {
            let check_id = self.insert_applicability_check(run_id, &prompt)?;
            prompt.check_id = Some(check_id);
            return Ok(ResourceOrchestrationResult {
                run_id,
                topic_snapshot,
                objective,
                curriculum_context,
                ambiguity_prompt: Some(prompt),
                selected_candidates: Vec::new(),
                generated_artifacts: Vec::new(),
                assembled_steps: recipe
                    .iter()
                    .enumerate()
                    .map(|(index, role)| ResourceAssemblyStep {
                        step_order: index as i64 + 1,
                        role_code: role.role_code.clone(),
                        title: title_case_role(&role.role_code),
                        purpose: role.purpose.clone(),
                        estimated_minutes: role.estimated_minutes,
                        resource_type: None,
                        resource_id: None,
                        generated: false,
                    })
                    .collect(),
                gap_ticket_ids: Vec::new(),
                confidence_score_bp: 0,
            });
        }

        let learner_signals = self.load_learner_signals(request.student_id, objective.topic_id)?;
        let indexed_resources = self.collect_candidates(&objective, &learner_signals)?;
        let mut used_refs = BTreeSet::new();
        let mut selected_candidates = Vec::new();
        let mut generated_artifacts = Vec::new();
        let mut assembled_steps = Vec::new();
        let mut gap_ticket_ids = Vec::new();

        for (index, role) in recipe.iter().enumerate() {
            let selected = self.select_best_candidate_for_role(
                &indexed_resources,
                &objective,
                &learner_signals,
                role,
                &used_refs,
            )?;

            let (candidate, generated) = if let Some(candidate) = selected {
                used_refs.insert((candidate.resource_type.clone(), candidate.resource_id));
                (Some(candidate), None)
            } else if let Some(generated) = self.generate_gap_fill(&objective, role)? {
                let candidate = ResourceCandidate {
                    resource_type: generated.resource_type.clone(),
                    resource_id: generated.resource_id,
                    title: generated.title.clone(),
                    subtitle: Some(format!(
                        "Generated for {}",
                        title_case_role(&role.role_code)
                    )),
                    role_code: role.role_code.clone(),
                    confidence_tier: generated.confidence_tier.clone(),
                    intrinsic_quality_bp: 6_200,
                    contextual_fitness_bp: 7_100,
                    overall_score_bp: 6_700,
                    rationale: vec!["Generated to cover a missing role in the plan.".to_string()],
                    metadata: generated.content.clone(),
                };
                used_refs.insert((candidate.resource_type.clone(), candidate.resource_id));
                (Some(candidate), Some(generated))
            } else {
                (None, None)
            };

            if let Some(candidate) = candidate.clone() {
                selected_candidates.push(candidate.clone());
                assembled_steps.push(ResourceAssemblyStep {
                    step_order: index as i64 + 1,
                    role_code: role.role_code.clone(),
                    title: title_case_role(&role.role_code),
                    purpose: role.purpose.clone(),
                    estimated_minutes: role.estimated_minutes,
                    resource_type: Some(candidate.resource_type.clone()),
                    resource_id: Some(candidate.resource_id),
                    generated: generated.is_some(),
                });
            } else {
                let gap_ticket_id =
                    self.open_gap_ticket_for_role(objective.topic_id, &role.role_code)?;
                gap_ticket_ids.push(gap_ticket_id);
                assembled_steps.push(ResourceAssemblyStep {
                    step_order: index as i64 + 1,
                    role_code: role.role_code.clone(),
                    title: title_case_role(&role.role_code),
                    purpose: role.purpose.clone(),
                    estimated_minutes: role.estimated_minutes,
                    resource_type: None,
                    resource_id: None,
                    generated: false,
                });
            }

            if let Some(generated) = generated {
                generated_artifacts.push(generated);
            }
        }

        self.persist_candidates(run_id, &selected_candidates, &generated_artifacts)?;
        let confidence_score_bp =
            self.update_run_confidence(run_id, &selected_candidates, &generated_artifacts)?;

        Ok(ResourceOrchestrationResult {
            run_id,
            topic_snapshot,
            objective,
            curriculum_context,
            ambiguity_prompt: None,
            selected_candidates,
            generated_artifacts,
            assembled_steps,
            gap_ticket_ids,
            confidence_score_bp,
        })
    }

    pub fn confirm_applicability(
        &self,
        check_id: i64,
        selected_option_code: &str,
        response_text: Option<&str>,
    ) -> EcoachResult<ResourceApplicabilityResolution> {
        let (run_id, options_json) = self
            .conn
            .query_row(
                "SELECT run_id, options_json
                 FROM resource_applicability_checks
                 WHERE id = ?1",
                [check_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let options = parse_json_vec::<ApplicabilityOption>(&options_json)?;
        let option = options
            .iter()
            .find(|item| item.code == selected_option_code)
            .cloned()
            .ok_or_else(|| {
                EcoachError::Validation(format!(
                    "unknown applicability option '{}'",
                    selected_option_code
                ))
            })?;

        self.conn
            .execute(
                "UPDATE resource_applicability_checks
                 SET selected_option_code = ?2,
                     selected_option_label = ?3,
                     response_text = ?4,
                     resolved_at = datetime('now')
                 WHERE id = ?1",
                params![
                    check_id,
                    option.code,
                    option.label,
                    response_text.map(str::trim)
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE resource_orchestration_runs
                 SET ambiguity_status = 'resolved',
                     run_status = 'archived',
                     updated_at = datetime('now')
                 WHERE id = ?1",
                [run_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let (student_id, subject_id, request_text, request_payload_json) = self
            .conn
            .query_row(
                "SELECT student_id, subject_id, request_text, request_payload_json
                 FROM resource_orchestration_runs
                 WHERE id = ?1",
                [run_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut replay_request: ResourceOrchestrationRequest =
            serde_json::from_str(&request_payload_json)
                .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        replay_request.student_id = student_id;
        replay_request.subject_id = subject_id;
        replay_request.topic_id = option.topic_id;
        replay_request.objective_code = option
            .objective_code
            .clone()
            .or(replay_request.objective_code.clone());
        replay_request.prompt_text = request_text.or(replay_request.prompt_text.clone());

        let follow_up = self.orchestrate_resources(replay_request)?;

        Ok(ResourceApplicabilityResolution {
            check_id,
            selected_option_code: option.code,
            selected_option_label: option.label,
            follow_up,
        })
    }

    pub fn record_learning_outcome(
        &self,
        input: RecordResourceLearningInput,
    ) -> EcoachResult<ResourceLearningRecord> {
        self.conn
            .execute(
                "INSERT INTO resource_learning_events (
                    run_id, session_id, outcome_status, usefulness_bp,
                    confidence_shift_bp, speed_shift_bp, accuracy_shift_bp,
                    learner_feedback_json, still_struggling_json,
                    effective_resource_refs_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    input.run_id,
                    input.session_id,
                    input.outcome_status,
                    input.usefulness_bp as i64,
                    input.confidence_shift_bp,
                    input.speed_shift_bp,
                    input.accuracy_shift_bp,
                    to_json(&input.learner_feedback)?,
                    to_json(&input.still_struggling)?,
                    to_json(&input.effective_resource_refs)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let id = self.conn.last_insert_rowid();

        self.conn
            .execute(
                "UPDATE resource_orchestration_runs
                 SET run_status = 'evaluated',
                     updated_at = datetime('now')
                 WHERE id = ?1",
                [input.run_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.update_resource_success_rates(
            &input.effective_resource_refs,
            input.usefulness_bp,
            &input.outcome_status,
        )?;

        if !input.still_struggling.is_empty() {
            let topic_id = self
                .conn
                .query_row(
                    "SELECT topic_id
                     FROM resource_orchestration_runs
                     WHERE id = ?1",
                    [input.run_id],
                    |row| row.get::<_, Option<i64>>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .flatten();
            for struggle in &input.still_struggling {
                self.ensure_gap_ticket(
                    topic_id,
                    None,
                    "student_request",
                    json!({ "still_struggling": struggle }),
                    "high",
                    &["note", "question"],
                )?;
            }
        }

        self.conn
            .query_row(
                "SELECT id, run_id, outcome_status, usefulness_bp, created_at
                 FROM resource_learning_events
                 WHERE id = ?1",
                [id],
                |row| {
                    Ok(ResourceLearningRecord {
                        id: row.get(0)?,
                        run_id: row.get(1)?,
                        outcome_status: row.get(2)?,
                        usefulness_bp: row.get::<_, i64>(3)?.clamp(0, 10_000) as BasisPoints,
                        created_at: row.get(4)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn refresh_gap_tickets(
        &self,
        topic: &TopicContext,
        nodes: &[NodeRow],
        readiness: &TopicResourceReadiness,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE content_gap_tickets
                 SET status = 'resolved',
                     resolved_at = datetime('now')
                 WHERE topic_id = ?1
                   AND status IN ('open', 'in_progress')
                   AND trigger_type IN (
                        'coverage_scan',
                        'question_shortage',
                        'misconception_unaddressed',
                        'quality_issue'
                   )",
                [topic.topic_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for missing in &readiness.missing_resources {
            self.ensure_gap_ticket(
                Some(topic.topic_id),
                None,
                "coverage_scan",
                json!({ "missing_component": missing }),
                if missing.contains("question") || missing.contains("explanation") {
                    "high"
                } else {
                    "medium"
                },
                &required_assets_for_role(missing),
            )?;
        }

        for node in nodes {
            let coverage = self
                .conn
                .query_row(
                    "SELECT overall_coverage_bp
                     FROM concept_coverage_scores
                     WHERE node_id = ?1",
                    [node.id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .unwrap_or(0);
            if coverage < 4_200 {
                self.ensure_gap_ticket(
                    Some(topic.topic_id),
                    Some(node.id),
                    "quality_issue",
                    json!({
                        "node": node.canonical_title,
                        "coverage_bp": coverage,
                    }),
                    "medium",
                    &["note", "question"],
                )?;
            }
        }

        Ok(())
    }

    fn build_topic_snapshot(
        &self,
        topic: &TopicContext,
        readiness: &TopicResourceReadiness,
    ) -> EcoachResult<TopicResourceIntelligenceSnapshot> {
        let concept_atom_count = self.count_query(
            "SELECT COUNT(*)
             FROM concept_atoms ca
             INNER JOIN academic_nodes an ON an.id = ca.node_id
             WHERE an.topic_id = ?1",
            topic.topic_id,
        )?;
        let indexed_resource_count = self.count_query(
            "SELECT COUNT(*)
             FROM resource_metadata_index
             WHERE topic_id = ?1",
            topic.topic_id,
        )?;
        let evidence_block_count = self.count_query(
            "SELECT COUNT(*)
             FROM evidence_blocks
             WHERE topic_id = ?1",
            topic.topic_id,
        )?;
        let active_gap_ticket_count = self.count_query(
            "SELECT COUNT(*)
             FROM content_gap_tickets
             WHERE topic_id = ?1
               AND status IN ('open', 'in_progress')",
            topic.topic_id,
        )?;
        let (
            overall_coverage_bp,
            question_coverage_bp,
            teaching_coverage_bp,
            assessment_coverage_bp,
            confidence_coverage_bp,
            coverage_color,
        ) = self
            .conn
            .query_row(
                "SELECT
                        CAST(COALESCE(AVG(overall_coverage_bp), 0) AS INTEGER),
                        CAST(COALESCE(AVG(question_coverage_bp), 0) AS INTEGER),
                        CAST(COALESCE(AVG(teaching_coverage_bp), 0) AS INTEGER),
                        CAST(COALESCE(AVG(assessment_coverage_bp), 0) AS INTEGER),
                        CAST(COALESCE(AVG(confidence_coverage_bp), 0) AS INTEGER),
                        COALESCE(MAX(coverage_color), 'red')
                     FROM concept_coverage_scores ccs
                     INNER JOIN academic_nodes an ON an.id = ccs.node_id
                     WHERE an.topic_id = ?1",
                [topic.topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(TopicResourceIntelligenceSnapshot {
            subject_id: topic.subject_id,
            subject_code: topic.subject_code.clone(),
            subject_name: topic.subject_name.clone(),
            topic_id: topic.topic_id,
            topic_name: topic.topic_name.clone(),
            concept_atom_count,
            indexed_resource_count,
            evidence_block_count,
            overall_coverage_bp: overall_coverage_bp.clamp(0, 10_000) as BasisPoints,
            question_coverage_bp: question_coverage_bp.clamp(0, 10_000) as BasisPoints,
            teaching_coverage_bp: teaching_coverage_bp.clamp(0, 10_000) as BasisPoints,
            assessment_coverage_bp: assessment_coverage_bp.clamp(0, 10_000) as BasisPoints,
            confidence_coverage_bp: confidence_coverage_bp.clamp(0, 10_000) as BasisPoints,
            coverage_color,
            active_gap_ticket_count,
            missing_components: readiness.missing_resources.clone(),
            ready_modes: readiness.generation_modes.clone(),
            curriculum_interpretation_available: self.count_query(
                "SELECT COUNT(*)
                     FROM curriculum_interpretations
                     WHERE topic_id = ?1",
                topic.topic_id,
            )? > 0,
            official_document_count: self.count_official_documents_for_topic_id(topic.topic_id)?,
        })
    }

    fn interpret_objective(
        &self,
        request: &ResourceOrchestrationRequest,
    ) -> EcoachResult<ResourceObjectiveProfile> {
        let mut subject_id = request.subject_id;
        let mut subject_code = None;
        let mut topic_id = request.topic_id;
        let mut topic_name = None;

        if let Some(topic_value) = topic_id {
            let topic = self.load_topic_context(topic_value)?;
            subject_id = Some(topic.subject_id);
            subject_code = Some(topic.subject_code.clone());
            topic_name = Some(topic.topic_name.clone());
        } else if let Some(prompt_text) = request.prompt_text.as_deref() {
            let matches = self.find_topic_matches(prompt_text, subject_id, 3)?;
            if matches.len() == 1
                || matches
                    .first()
                    .map(|item| item.score >= 8_200)
                    .unwrap_or(false)
            {
                if let Some(best) = matches.first() {
                    subject_id = Some(best.subject_id);
                    subject_code = Some(best.subject_code.clone());
                    topic_id = Some(best.topic_id);
                    topic_name = Some(best.topic_name.clone());
                }
            }
        }

        if subject_code.is_none() {
            if let Some(subject_value) = subject_id {
                subject_code = self
                    .conn
                    .query_row(
                        "SELECT code
                         FROM subjects
                         WHERE id = ?1",
                        [subject_value],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
        }

        let objective_code = request.objective_code.clone().unwrap_or_else(|| {
            detect_objective_code(
                request.prompt_text.as_deref(),
                request.session_mode.as_deref(),
            )
        });
        let session_mode = request
            .session_mode
            .clone()
            .unwrap_or_else(|| session_mode_for_objective(&objective_code));
        let learner_need = learner_need_for_objective(&objective_code).to_string();
        let preferred_formats = if request.preferred_formats.is_empty() {
            default_formats_for_objective(&objective_code)
        } else {
            request.preferred_formats.clone()
        };
        let time_budget_minutes =
            request
                .time_budget_minutes
                .unwrap_or(match session_mode.as_str() {
                    "teach" => 28,
                    "exam" => 22,
                    "repair" => 24,
                    "confidence" => 20,
                    _ => 25,
                });
        let target_difficulty_bp =
            request
                .target_difficulty_bp
                .unwrap_or(match session_mode.as_str() {
                    "teach" => 4_600,
                    "confidence" => 4_200,
                    "exam" => 7_200,
                    "repair" => 5_200,
                    _ => 5_800,
                });

        Ok(ResourceObjectiveProfile {
            objective_code: objective_code.clone(),
            objective_label: title_case_role(&objective_code),
            session_mode: session_mode.clone(),
            learner_need,
            subject_id,
            subject_code,
            topic_id,
            topic_name,
            prompt_text: request.prompt_text.clone(),
            preferred_formats,
            time_budget_minutes,
            target_difficulty_bp,
            curriculum_lens: vec![
                "syllabus_alignment".to_string(),
                "coverage_balance".to_string(),
                "exam_relevance".to_string(),
            ],
            pedagogical_lens: default_pedagogical_lens(&session_mode),
            learner_lens: learner_lens_for_objective(&objective_code),
        })
    }

    fn detect_ambiguity(
        &self,
        objective: &ResourceObjectiveProfile,
        request: &ResourceOrchestrationRequest,
    ) -> EcoachResult<Option<ApplicabilityPrompt>> {
        if objective.topic_id.is_some() {
            return Ok(None);
        }

        let Some(prompt_text) = request.prompt_text.as_deref() else {
            return Ok(None);
        };
        let matches = self.find_topic_matches(prompt_text, request.subject_id, 4)?;
        if matches.len() < 2 {
            return Ok(None);
        }

        let top_score = matches[0].score;
        let second_score = matches[1].score;
        if top_score >= 8_400 && top_score.saturating_sub(second_score) > 1_000 {
            return Ok(None);
        }

        Ok(Some(ApplicabilityPrompt {
            check_id: None,
            reason: "The request could fit more than one topic.".to_string(),
            prompt_text: "Which topic best matches what you want to study?".to_string(),
            options: matches
                .into_iter()
                .take(3)
                .map(|item| ApplicabilityOption {
                    code: format!("topic:{}", item.topic_id),
                    label: item.topic_name,
                    rationale: format!(
                        "{} match from your wording and the curriculum spine.",
                        item.subject_code
                    ),
                    topic_id: Some(item.topic_id),
                    objective_code: request.objective_code.clone(),
                })
                .collect(),
        }))
    }

    fn load_curriculum_context(&self, topic_id: Option<i64>) -> EcoachResult<Value> {
        let Some(topic_id) = topic_id else {
            return Ok(json!({
                "scope": "subject",
                "curriculum_interpretation": null,
                "student_facing_curriculum": null,
                "topic_health": null,
            }));
        };

        let interpretation = self
            .conn
            .query_row(
                "SELECT friendly_name, knowledge_points_json, skills_involved_json,
                        common_misconceptions_json, prerequisite_nodes_json,
                        teaching_strategies_json, question_families_json,
                        worked_example_templates_json, memory_recall_tags_json
                 FROM curriculum_interpretations
                 WHERE topic_id = ?1",
                [topic_id],
                |row| {
                    Ok(json!({
                        "friendly_name": row.get::<_, Option<String>>(0)?,
                        "knowledge_points": parse_json_value(row.get::<_, Option<String>>(1)?.as_deref()),
                        "skills_involved": parse_json_value(row.get::<_, Option<String>>(2)?.as_deref()),
                        "common_misconceptions": parse_json_value(row.get::<_, Option<String>>(3)?.as_deref()),
                        "prerequisites": parse_json_value(row.get::<_, Option<String>>(4)?.as_deref()),
                        "teaching_strategies": parse_json_value(row.get::<_, Option<String>>(5)?.as_deref()),
                        "question_families": parse_json_value(row.get::<_, Option<String>>(6)?.as_deref()),
                        "worked_examples": parse_json_value(row.get::<_, Option<String>>(7)?.as_deref()),
                        "memory_tags": parse_json_value(row.get::<_, Option<String>>(8)?.as_deref()),
                    }))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(Value::Null);

        let student_facing = self
            .conn
            .query_row(
                "SELECT simplified_name, simplified_description, what_to_know,
                        what_to_do, examples_json
                 FROM student_facing_curriculum
                 WHERE topic_id = ?1",
                [topic_id],
                |row| {
                    Ok(json!({
                        "simplified_name": row.get::<_, String>(0)?,
                        "simplified_description": row.get::<_, Option<String>>(1)?,
                        "what_to_know": row.get::<_, Option<String>>(2)?,
                        "what_to_do": row.get::<_, Option<String>>(3)?,
                        "examples": parse_json_value(row.get::<_, Option<String>>(4)?.as_deref()),
                    }))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(Value::Null);

        let topic_health = self
            .conn
            .query_row(
                "SELECT package_completeness_bp, quality_score_bp, live_health_state, missing_components_json
                 FROM topic_health
                 WHERE topic_id = ?1",
                [topic_id],
                |row| {
                    Ok(json!({
                        "package_completeness_bp": row.get::<_, i64>(0)?,
                        "quality_score_bp": row.get::<_, i64>(1)?,
                        "live_health_state": row.get::<_, String>(2)?,
                        "missing_components": parse_json_value(row.get::<_, Option<String>>(3)?.as_deref()),
                    }))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(Value::Null);

        Ok(json!({
            "scope": "topic",
            "curriculum_interpretation": interpretation,
            "student_facing_curriculum": student_facing,
            "topic_health": topic_health,
        }))
    }

    fn load_learner_signals(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<LearnerSignals> {
        let Some(topic_id) = topic_id else {
            return Ok(LearnerSignals::default());
        };

        let topic_state = self
            .conn
            .query_row(
                "SELECT mastery_score, gap_score, fragility_score, decay_risk, mastery_state
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let wrong_answer_count = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM wrong_answer_diagnoses
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(0);
        let coverage_status = self
            .conn
            .query_row(
                "SELECT coverage_status
                 FROM curriculum_coverage_ledger
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut recent_actions = Vec::new();
        if let Some((_, _, _, _, mastery_state)) = &topic_state {
            recent_actions.push(format!("mastery:{}", mastery_state));
        }
        if let Some(coverage_status) = coverage_status {
            recent_actions.push(format!("coverage:{}", coverage_status));
        }
        if wrong_answer_count > 0 {
            recent_actions.push("wrong_answers_present".to_string());
        }

        Ok(match topic_state {
            Some((mastery_score, gap_score, fragility_score, decay_risk, _)) => LearnerSignals {
                mastery_score,
                gap_score,
                fragility_score,
                decay_risk,
                wrong_answer_count,
                recent_actions,
            },
            None => LearnerSignals {
                wrong_answer_count,
                recent_actions,
                ..LearnerSignals::default()
            },
        })
    }

    fn collect_candidates(
        &self,
        objective: &ResourceObjectiveProfile,
        learner_signals: &LearnerSignals,
    ) -> EcoachResult<Vec<IndexedResourceRow>> {
        let Some(subject_id) = objective.subject_id else {
            return Ok(Vec::new());
        };
        let mut resources = self.load_indexed_resources(subject_id, objective.topic_id)?;

        if let Some(topic_id) = objective.topic_id {
            let selected_questions =
                QuestionSelector::new(self.conn).select_questions(&QuestionSelectionRequest {
                    subject_id,
                    topic_ids: vec![topic_id],
                    family_ids: Vec::new(),
                    target_question_count: 6,
                    target_difficulty: Some(objective.target_difficulty_bp),
                    weakness_topic_ids: if learner_signals.gap_score >= 5_000 {
                        vec![topic_id]
                    } else {
                        Vec::new()
                    },
                    recently_seen_question_ids: Vec::new(),
                    timed: objective.session_mode == "exam",
                    diagnostic_stage: None,
                    condition_type: None,
                    require_confidence_prompt: false,
                    require_concept_guess_prompt: false,
                })?;

            for selected in selected_questions {
                if resources.iter().any(|item| {
                    item.resource_type == "question" && item.resource_id == selected.question.id
                }) {
                    continue;
                }

                resources.push(IndexedResourceRow {
                    resource_id: selected.question.id,
                    resource_type: "question".to_string(),
                    title: selected.question.stem.clone(),
                    subtitle: selected.question.explanation_text.clone(),
                    topic_id: Some(selected.question.topic_id),
                    difficulty_bp: selected.question.difficulty_level as i64,
                    exam_relevance_bp: 8_000,
                    teach_suitability_bp: 3_800,
                    test_suitability_bp: 8_900,
                    pressure_suitability_bp: if objective.session_mode == "exam" {
                        8_800
                    } else {
                        6_200
                    },
                    confidence_tier: "performance_tested".to_string(),
                    teacher_verified: false,
                    student_success_rate_bp: Some(
                        clamp_bp((selected.fit_score * 10_000.0) as i64) as i64
                    ),
                    metadata: json!({
                        "question_format": selected.question.question_format,
                        "selector_fit_bp": clamp_bp((selected.fit_score * 10_000.0) as i64),
                    }),
                });
            }
        }

        Ok(resources)
    }

    fn build_recipe(
        &self,
        objective: &ResourceObjectiveProfile,
        topic_snapshot: Option<&TopicResourceIntelligenceSnapshot>,
    ) -> Vec<SelectedRole> {
        let mut roles = Vec::new();
        match objective.session_mode.as_str() {
            "teach" => {
                roles.push(selected_role(
                    "concept_explainer",
                    "Build the concept clearly.",
                    8,
                ));
                roles.push(selected_role(
                    "worked_example",
                    "Show a fully worked pattern.",
                    8,
                ));
                roles.push(selected_role(
                    "accuracy_drill",
                    "Check the learner can apply it.",
                    7,
                ));
                roles.push(selected_role(
                    "mastery_validator",
                    "Confirm independent recall.",
                    5,
                ));
            }
            "repair" => {
                roles.push(selected_role(
                    "quick_refresher",
                    "Reactivate the core idea fast.",
                    5,
                ));
                roles.push(selected_role(
                    "misconception_corrector",
                    "Repair the likely error pattern.",
                    8,
                ));
                roles.push(selected_role(
                    "accuracy_drill",
                    "Stabilize the corrected path.",
                    8,
                ));
                roles.push(selected_role(
                    "mastery_validator",
                    "Check the repair holds.",
                    4,
                ));
            }
            "exam" => {
                roles.push(selected_role(
                    "quick_refresher",
                    "Recall the trigger cues.",
                    4,
                ));
                roles.push(selected_role(
                    "pressure_drill",
                    "Practice under exam pressure.",
                    10,
                ));
                roles.push(selected_role(
                    "mastery_validator",
                    "Validate exam-ready output.",
                    6,
                ));
            }
            "confidence" => {
                roles.push(selected_role(
                    "confidence_rebuilder",
                    "Start with a confidence-building win.",
                    8,
                ));
                roles.push(selected_role(
                    "worked_example",
                    "Model the successful pattern.",
                    7,
                ));
                roles.push(selected_role(
                    "accuracy_drill",
                    "Move into independent success.",
                    6,
                ));
            }
            _ => {
                roles.push(selected_role(
                    "quick_refresher",
                    "Refresh the main idea.",
                    5,
                ));
                roles.push(selected_role(
                    "worked_example",
                    "Anchor with one strong example.",
                    7,
                ));
                roles.push(selected_role(
                    "accuracy_drill",
                    "Practice retrieval and use.",
                    8,
                ));
            }
        }

        if objective
            .preferred_formats
            .iter()
            .any(|value| value.contains("formula"))
        {
            roles.insert(
                0,
                selected_role("formula_anchor", "Anchor the key formula or rule.", 5),
            );
        }
        if objective
            .preferred_formats
            .iter()
            .any(|value| value.contains("vocab") || value.contains("glossary"))
        {
            roles.insert(
                0,
                selected_role("vocabulary_repair", "Clarify the key terms first.", 5),
            );
        }
        if topic_snapshot
            .map(|snapshot| snapshot.overall_coverage_bp < 5_000)
            .unwrap_or(false)
            && !roles
                .iter()
                .any(|role| role.role_code == "concept_explainer")
        {
            roles.insert(
                0,
                selected_role(
                    "concept_explainer",
                    "Rebuild the topic spine before drills.",
                    8,
                ),
            );
        }

        let mut total_minutes = 0;
        let budget = objective.time_budget_minutes.max(10);
        roles
            .into_iter()
            .filter(|role| {
                if total_minutes >= budget {
                    return false;
                }
                total_minutes += role.estimated_minutes;
                true
            })
            .collect()
    }

    fn select_best_candidate_for_role(
        &self,
        indexed_resources: &[IndexedResourceRow],
        objective: &ResourceObjectiveProfile,
        learner_signals: &LearnerSignals,
        role: &SelectedRole,
        used_refs: &BTreeSet<(String, i64)>,
    ) -> EcoachResult<Option<ResourceCandidate>> {
        let mut best: Option<ResourceCandidate> = None;
        for resource in indexed_resources {
            if used_refs.contains(&(resource.resource_type.clone(), resource.resource_id)) {
                continue;
            }
            let (intrinsic_quality_bp, contextual_fitness_bp, overall_score_bp, rationale) =
                self.score_candidate(resource, objective, learner_signals, &role.role_code);
            if overall_score_bp < 4_400 {
                continue;
            }

            let candidate = ResourceCandidate {
                resource_type: resource.resource_type.clone(),
                resource_id: resource.resource_id,
                title: resource.title.clone(),
                subtitle: resource.subtitle.clone(),
                role_code: role.role_code.clone(),
                confidence_tier: resource.confidence_tier.clone(),
                intrinsic_quality_bp,
                contextual_fitness_bp,
                overall_score_bp,
                rationale,
                metadata: resource.metadata.clone(),
            };

            let replace = best
                .as_ref()
                .map(|current| current.overall_score_bp < candidate.overall_score_bp)
                .unwrap_or(true);
            if replace {
                best = Some(candidate);
            }
        }

        Ok(best)
    }

    fn generate_gap_fill(
        &self,
        objective: &ResourceObjectiveProfile,
        role: &SelectedRole,
    ) -> EcoachResult<Option<GeneratedGapArtifact>> {
        if objective.topic_id.is_none() || objective.subject_id.is_none() {
            return Ok(None);
        }

        match role.role_code.as_str() {
            "pressure_drill" | "accuracy_drill" | "mastery_validator" | "stretch_challenge" => self
                .generate_question_for_role(objective, role)
                .map(Some)
                .or_else(|_| self.generate_artifact_for_role(objective, role).map(Some)),
            _ => self.generate_artifact_for_role(objective, role).map(Some),
        }
    }

    fn generate_question_for_role(
        &self,
        objective: &ResourceObjectiveProfile,
        role: &SelectedRole,
    ) -> EcoachResult<GeneratedGapArtifact> {
        let subject_id = objective.subject_id.ok_or_else(|| {
            EcoachError::Validation("subject is required for question generation".to_string())
        })?;
        let topic_id = objective.topic_id.ok_or_else(|| {
            EcoachError::Validation("topic is required for question generation".to_string())
        })?;

        let variant_mode = match role.role_code.as_str() {
            "pressure_drill" => QuestionVariantMode::Stretch,
            "mastery_validator" => QuestionVariantMode::Adversary,
            _ => QuestionVariantMode::Rescue,
        };
        let slot_spec = QuestionSlotSpec {
            subject_id,
            topic_id: Some(topic_id),
            target_cognitive_demand: Some(match role.role_code.as_str() {
                "mastery_validator" => "reasoning".to_string(),
                "pressure_drill" => "application".to_string(),
                _ => "understanding".to_string(),
            }),
            target_question_format: Some(match role.role_code.as_str() {
                "mastery_validator" => "structured".to_string(),
                _ => "mcq".to_string(),
            }),
            max_generated_share: 9_000,
        };
        let reactor = QuestionReactor::new(self.conn);
        let request = reactor.create_generation_request(&QuestionGenerationRequestInput {
            slot_spec,
            family_id: None,
            source_question_id: None,
            request_kind: "resource_gap_fill".to_string(),
            variant_mode,
            requested_count: 1,
            rationale: Some(format!(
                "Generate {} for resource orchestration.",
                role.role_code
            )),
        })?;
        let generated = reactor.process_generation_request(request.id)?;
        let draft = generated.into_iter().next().ok_or_else(|| {
            EcoachError::NotFound("question generation returned no draft".to_string())
        })?;
        let node_id = self.load_topic_nodes(topic_id)?.first().map(|node| node.id);
        self.upsert_resource_index(
            draft.question.id,
            "question",
            subject_id,
            Some(topic_id),
            draft.question.subtopic_id,
            node_id,
            None,
            draft.question.difficulty_level as i64,
            8_000,
            3_600,
            8_900,
            8_400,
            Some("resource_gap_fill"),
            "ai_generated",
            false,
            Some(
                json!({ "request_id": request.id, "variant_mode": draft.variant_mode }).to_string(),
            ),
        )?;

        Ok(GeneratedGapArtifact {
            resource_type: "question".to_string(),
            resource_id: draft.question.id,
            artifact_id: None,
            artifact_version_id: None,
            role_code: role.role_code.clone(),
            title: draft.question.stem,
            confidence_tier: "ai_generated".to_string(),
            provenance_ref: format!("question_generation_request:{}", request.id),
            content: json!({
                "question_id": draft.question.id,
                "variant_mode": draft.variant_mode,
                "transform_summary": draft.transform_summary,
                "options": draft.options,
            }),
        })
    }

    fn generate_artifact_for_role(
        &self,
        objective: &ResourceObjectiveProfile,
        role: &SelectedRole,
    ) -> EcoachResult<GeneratedGapArtifact> {
        let subject_id = objective.subject_id.ok_or_else(|| {
            EcoachError::Validation("subject is required for artifact generation".to_string())
        })?;
        let topic_id = objective.topic_id.ok_or_else(|| {
            EcoachError::Validation("topic is required for artifact generation".to_string())
        })?;
        let atom_bundle = self.load_atom_bundle_for_generation(topic_id)?;
        let headline = match role.role_code.as_str() {
            "formula_anchor" => "Formula Anchor",
            "vocabulary_repair" => "Vocabulary Repair",
            "misconception_corrector" => "Misconception Repair Note",
            "worked_example" => "Worked Example",
            "quick_refresher" => "Quick Refresher",
            "confidence_rebuilder" => "Confidence Builder",
            _ => "Generated Lesson Note",
        };
        let artifact_type = match role.role_code.as_str() {
            "worked_example" => "worked_example",
            "pressure_drill" | "accuracy_drill" | "mastery_validator" => "drill",
            "formula_anchor" => "formula",
            _ => "explanation",
        };
        let resource_type = artifact_resource_type(artifact_type).unwrap_or("note");
        let content_json = json!({
            "title": format!(
                "{}: {}",
                headline,
                objective.topic_name.clone().unwrap_or_else(|| "Topic".to_string())
            ),
            "role_code": role.role_code,
            "objective_code": objective.objective_code,
            "summary": atom_bundle
                .first()
                .cloned()
                .unwrap_or_else(|| "Core topic summary".to_string()),
            "supporting_points": atom_bundle.iter().take(5).cloned().collect::<Vec<_>>(),
            "practice_prompts": [
                format!(
                    "Explain {} in your own words.",
                    objective.topic_name.clone().unwrap_or_else(|| "the topic".to_string())
                ),
                format!(
                    "What is the main trap in {}?",
                    objective.topic_name.clone().unwrap_or_else(|| "this topic".to_string())
                )
            ],
        });

        self.conn
            .execute(
                "INSERT INTO artifacts (
                    artifact_type, topic_id, subject_id, lifecycle_state, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, 'approved', datetime('now'), datetime('now'))",
                params![artifact_type, topic_id, subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let artifact_id = self.conn.last_insert_rowid();
        self.conn
            .execute(
                "INSERT INTO artifact_versions (
                    artifact_id, version_no, state, content_json, build_reason,
                    quality_score_bp, provenance_ref, created_at
                 ) VALUES (?1, 1, 'approved', ?2, 'resource_gap_fill', ?3, ?4, datetime('now'))",
                params![
                    artifact_id,
                    content_json.to_string(),
                    6_800,
                    format!("resource_orchestration:{}:{}", topic_id, role.role_code),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let artifact_version_id = self.conn.last_insert_rowid();
        self.conn
            .execute(
                "UPDATE artifacts
                 SET current_version_id = ?2,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![artifact_id, artifact_version_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO artifact_quality_reports (
                    artifact_version_id, structural_score_bp, academic_score_bp,
                    relevance_score_bp, clarity_score_bp, overall_score_bp, pass_fail, issues_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pass', '[]')",
                params![artifact_version_id, 6_900, 6_600, 7_000, 7_200, 6_800],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let node_id = self.load_topic_nodes(topic_id)?.first().map(|node| node.id);
        self.upsert_resource_index(
            artifact_version_id,
            resource_type,
            subject_id,
            Some(topic_id),
            None,
            node_id,
            None,
            5_200,
            6_600,
            if resource_type == "note" {
                8_200
            } else {
                6_400
            },
            if resource_type == "drill" {
                8_600
            } else {
                4_800
            },
            if resource_type == "drill" {
                8_000
            } else {
                4_000
            },
            Some("resource_gap_fill"),
            "ai_generated",
            false,
            Some(content_json.to_string()),
        )?;

        Ok(GeneratedGapArtifact {
            resource_type: resource_type.to_string(),
            resource_id: artifact_version_id,
            artifact_id: Some(artifact_id),
            artifact_version_id: Some(artifact_version_id),
            role_code: role.role_code.clone(),
            title: format!(
                "{}: {}",
                headline,
                objective
                    .topic_name
                    .clone()
                    .unwrap_or_else(|| "Topic".to_string())
            ),
            confidence_tier: "ai_generated".to_string(),
            provenance_ref: format!("artifact_version:{}", artifact_version_id),
            content: content_json,
        })
    }

    fn load_atom_bundle_for_generation(&self, topic_id: i64) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT ca.content_text
                 FROM concept_atoms ca
                 INNER JOIN academic_nodes an ON an.id = ca.node_id
                 WHERE an.topic_id = ?1
                 ORDER BY ca.exam_relevance DESC, ca.id ASC
                 LIMIT 8",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut bundle = Vec::new();
        for row in rows {
            bundle.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(bundle)
    }

    fn load_indexed_resources(
        &self,
        subject_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Vec<IndexedResourceRow>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT resource_id, resource_type, topic_id, difficulty_bp, exam_relevance_bp,
                        teach_suitability_bp, test_suitability_bp, pressure_suitability_bp,
                        COALESCE(confidence_tier, 'ai_generated'),
                        teacher_verified, student_success_rate_bp, source
                 FROM resource_metadata_index
                 WHERE subject_id = ?1
                   AND (?2 = 0 OR topic_id = ?3 OR topic_id IS NULL)
                 ORDER BY exam_relevance_bp DESC, difficulty_bp ASC, resource_id ASC
                 LIMIT 48",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![subject_id, if topic_id.is_some() { 1 } else { 0 }, topic_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                        row.get::<_, i64>(7)?,
                        row.get::<_, String>(8)?,
                        row.get::<_, i64>(9)?,
                        row.get::<_, Option<i64>>(10)?,
                        row.get::<_, Option<String>>(11)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut resources = Vec::new();
        for row in rows {
            let (
                resource_id,
                resource_type,
                resource_topic_id,
                difficulty_bp,
                exam_relevance_bp,
                teach_suitability_bp,
                test_suitability_bp,
                pressure_suitability_bp,
                confidence_tier,
                teacher_verified,
                student_success_rate_bp,
                source,
            ) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;

            let hydrated = match resource_type.as_str() {
                "question" => self
                    .conn
                    .query_row(
                        "SELECT stem, explanation_text, question_format, estimated_time_seconds
                         FROM questions
                         WHERE id = ?1",
                        [resource_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                json!({
                                    "question_format": row.get::<_, String>(2)?,
                                    "estimated_time_seconds": row.get::<_, i64>(3)?,
                                    "source": source,
                                }),
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                "worked_example" | "glossary_entry" => self
                    .conn
                    .query_row(
                        "SELECT title, short_text, entry_type
                         FROM knowledge_entries
                         WHERE id = ?1",
                        [resource_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                json!({ "entry_type": row.get::<_, String>(2)? }),
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                "teach_explanation" => self
                    .conn
                    .query_row(
                        "SELECT an.canonical_title, te.simple_explanation, te.explanation_level
                         FROM teach_explanations te
                         INNER JOIN academic_nodes an ON an.id = te.node_id
                         WHERE te.id = ?1",
                        [resource_id],
                        |row| {
                            Ok((
                                row.get::<_, String>(0)?,
                                row.get::<_, Option<String>>(1)?,
                                json!({ "explanation_level": row.get::<_, String>(2)? }),
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                "note" | "drill" | "flashcard" => self
                    .conn
                    .query_row(
                        "SELECT a.artifact_type, av.content_json
                         FROM artifact_versions av
                         INNER JOIN artifacts a ON a.id = av.artifact_id
                         WHERE av.id = ?1",
                        [resource_id],
                        |row| {
                            let artifact_type = row.get::<_, String>(0)?;
                            let content = row.get::<_, String>(1)?;
                            let json_value = parse_json_value(Some(content.as_str()));
                            let title = json_value
                                .get("title")
                                .and_then(Value::as_str)
                                .unwrap_or("Generated resource")
                                .to_string();
                            let subtitle = json_value
                                .get("summary")
                                .and_then(Value::as_str)
                                .map(str::to_string);
                            Ok((
                                title,
                                subtitle,
                                json!({ "artifact_type": artifact_type, "content": json_value }),
                            ))
                        },
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?,
                _ => None,
            };

            if let Some((title, subtitle, metadata)) = hydrated {
                resources.push(IndexedResourceRow {
                    resource_id,
                    resource_type,
                    title,
                    subtitle,
                    topic_id: resource_topic_id,
                    difficulty_bp,
                    exam_relevance_bp,
                    teach_suitability_bp,
                    test_suitability_bp,
                    pressure_suitability_bp,
                    confidence_tier,
                    teacher_verified: teacher_verified == 1,
                    student_success_rate_bp,
                    metadata,
                });
            }
        }

        Ok(resources)
    }

    fn score_candidate(
        &self,
        resource: &IndexedResourceRow,
        objective: &ResourceObjectiveProfile,
        learner_signals: &LearnerSignals,
        role_code: &str,
    ) -> (BasisPoints, BasisPoints, BasisPoints, Vec<String>) {
        let role_fit = role_fit_score(role_code, resource);
        let difficulty_fit =
            difficulty_fit_score(objective.target_difficulty_bp, resource.difficulty_bp);
        let learner_fit = learner_fit_score(role_code, resource, learner_signals);
        let mode_fit = match objective.session_mode.as_str() {
            "teach" => resource.teach_suitability_bp as BasisPoints,
            "exam" => resource.pressure_suitability_bp as BasisPoints,
            _ => resource.test_suitability_bp as BasisPoints,
        };
        let intrinsic_quality_bp = clamp_bp(
            (resource.exam_relevance_bp
                + confidence_tier_score(&resource.confidence_tier) as i64
                + resource.student_success_rate_bp.unwrap_or(6_000)
                + if resource.teacher_verified {
                    9_000
                } else {
                    5_500
                })
                / 4,
        );
        let contextual_fitness_bp = clamp_bp(
            (role_fit as i64 + difficulty_fit as i64 + learner_fit as i64 + mode_fit as i64) / 4,
        );
        let overall_score_bp = clamp_bp(
            (intrinsic_quality_bp as i64 * 45 / 100) + (contextual_fitness_bp as i64 * 55 / 100),
        );
        let mut rationale = vec![
            format!("Role fit {}.", role_fit),
            format!("Difficulty fit {}.", difficulty_fit),
            format!("Mode fit {}.", mode_fit),
        ];
        if let Some(topic_id) = objective.topic_id {
            if resource.topic_id == Some(topic_id) {
                rationale.push("Matches the requested topic directly.".to_string());
            }
        }
        if resource.teacher_verified {
            rationale.push("Teacher-verified or curated source.".to_string());
        }

        (
            intrinsic_quality_bp,
            contextual_fitness_bp,
            overall_score_bp,
            rationale,
        )
    }

    fn find_topic_matches(
        &self,
        text: &str,
        subject_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<TopicMatch>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT s.id, s.code, t.id, t.name, COALESCE(t.description, '')
                 FROM topics t
                 INNER JOIN subjects s ON s.id = t.subject_id
                 WHERE t.is_active = 1
                   AND (?1 IS NULL OR t.subject_id = ?1)
                 ORDER BY t.display_order ASC, t.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut matches = Vec::new();
        for row in rows {
            let (subject_id, subject_code, topic_id, topic_name, description) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let score = topic_match_score(text, &topic_name, &description);
            if score > 0 {
                matches.push(TopicMatch {
                    subject_id,
                    subject_code,
                    topic_id,
                    topic_name,
                    score,
                });
            }
        }
        matches.sort_by(|left, right| right.score.cmp(&left.score));
        matches.truncate(limit);
        Ok(matches)
    }

    fn insert_run(
        &self,
        request: &ResourceOrchestrationRequest,
        objective: &ResourceObjectiveProfile,
        curriculum_context: &Value,
        recipe: &[SelectedRole],
        needs_confirmation: bool,
    ) -> EcoachResult<i64> {
        let recipe_json = recipe
            .iter()
            .map(|role| {
                json!({
                    "role_code": role.role_code,
                    "purpose": role.purpose,
                    "estimated_minutes": role.estimated_minutes,
                })
            })
            .collect::<Vec<_>>();
        self.conn
            .execute(
                "INSERT INTO resource_orchestration_runs (
                    student_id, subject_id, topic_id, objective_code, session_mode,
                    request_text, request_payload_json, interpreted_objective_json,
                    curriculum_context_json, recipe_json, ambiguity_status, run_status,
                    confidence_score_bp, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 0, datetime('now'), datetime('now'))",
                params![
                    request.student_id,
                    objective.subject_id,
                    objective.topic_id,
                    objective.objective_code,
                    objective.session_mode,
                    request.prompt_text,
                    to_json(request)?,
                    to_json(objective)?,
                    curriculum_context.to_string(),
                    to_json(&recipe_json)?,
                    if needs_confirmation { "needs_confirmation" } else { "clear" },
                    if needs_confirmation { "needs_confirmation" } else { "composed" },
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    fn insert_applicability_check(
        &self,
        run_id: i64,
        prompt: &ApplicabilityPrompt,
    ) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO resource_applicability_checks (
                    run_id, prompt_text, options_json, created_at
                 ) VALUES (?1, ?2, ?3, datetime('now'))",
                params![run_id, prompt.prompt_text, to_json(&prompt.options)?],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    fn persist_candidates(
        &self,
        run_id: i64,
        selected_candidates: &[ResourceCandidate],
        generated_artifacts: &[GeneratedGapArtifact],
    ) -> EcoachResult<()> {
        let generated_refs = generated_artifacts
            .iter()
            .map(|item| (item.resource_type.clone(), item.resource_id))
            .collect::<BTreeSet<_>>();
        for (index, candidate) in selected_candidates.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO resource_orchestration_candidates (
                        run_id, resource_id, resource_type, role_code, title,
                        selection_rank, selected, generated, intrinsic_quality_bp,
                        contextual_fitness_bp, overall_score_bp, confidence_tier,
                        rationale_json, metadata_json, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'))",
                    params![
                        run_id,
                        candidate.resource_id,
                        candidate.resource_type,
                        candidate.role_code,
                        candidate.title,
                        index as i64 + 1,
                        if generated_refs.contains(&(candidate.resource_type.clone(), candidate.resource_id)) { 1 } else { 0 },
                        candidate.intrinsic_quality_bp as i64,
                        candidate.contextual_fitness_bp as i64,
                        candidate.overall_score_bp as i64,
                        candidate.confidence_tier,
                        to_json(&candidate.rationale)?,
                        candidate.metadata.to_string(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn update_run_confidence(
        &self,
        run_id: i64,
        selected_candidates: &[ResourceCandidate],
        generated_artifacts: &[GeneratedGapArtifact],
    ) -> EcoachResult<BasisPoints> {
        let base_confidence = if selected_candidates.is_empty() {
            0
        } else {
            clamp_bp(
                selected_candidates
                    .iter()
                    .map(|candidate| candidate.overall_score_bp as i64)
                    .sum::<i64>()
                    / selected_candidates.len() as i64,
            )
        };
        let confidence_score_bp = if generated_artifacts.is_empty() {
            base_confidence
        } else {
            base_confidence.saturating_sub(600)
        };
        self.conn
            .execute(
                "UPDATE resource_orchestration_runs
                 SET confidence_score_bp = ?2,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![run_id, confidence_score_bp as i64],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(confidence_score_bp)
    }

    fn open_gap_ticket_for_role(
        &self,
        topic_id: Option<i64>,
        role_code: &str,
    ) -> EcoachResult<i64> {
        let trigger_type = if role_code.contains("drill") || role_code == "mastery_validator" {
            "question_shortage"
        } else if role_code == "misconception_corrector" {
            "misconception_unaddressed"
        } else {
            "coverage_scan"
        };
        self.ensure_gap_ticket(
            topic_id,
            None,
            trigger_type,
            json!({ "role_code": role_code }),
            if role_code == "pressure_drill" || role_code == "mastery_validator" {
                "high"
            } else {
                "medium"
            },
            &required_assets_for_role(role_code),
        )
    }

    fn ensure_gap_ticket(
        &self,
        topic_id: Option<i64>,
        node_id: Option<i64>,
        trigger_type: &str,
        trigger_context: Value,
        severity: &str,
        required_asset_types: &[&str],
    ) -> EcoachResult<i64> {
        let required_assets_json = to_json(&required_asset_types)?;
        if let Some(existing_id) = self
            .conn
            .query_row(
                "SELECT id
                 FROM content_gap_tickets
                 WHERE topic_id IS ?1
                   AND node_id IS ?2
                   AND trigger_type = ?3
                   AND required_asset_types_json = ?4
                   AND status IN ('open', 'in_progress')
                 ORDER BY id DESC
                 LIMIT 1",
                params![topic_id, node_id, trigger_type, required_assets_json],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            return Ok(existing_id);
        }

        self.conn
            .execute(
                "INSERT INTO content_gap_tickets (
                    topic_id, node_id, trigger_type, trigger_context_json,
                    severity, required_asset_types_json, status, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'open', datetime('now'))",
                params![
                    topic_id,
                    node_id,
                    trigger_type,
                    trigger_context.to_string(),
                    severity,
                    required_assets_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    fn update_resource_success_rates(
        &self,
        effective_resource_refs: &[String],
        usefulness_bp: BasisPoints,
        outcome_status: &str,
    ) -> EcoachResult<()> {
        let status_bonus = match outcome_status {
            "improved" => 600,
            "partial" => 0,
            "failed" => -800,
            _ => 200,
        };
        for resource_ref in effective_resource_refs {
            if let Some((resource_type, resource_id)) = parse_resource_ref(resource_ref) {
                let existing = self
                    .conn
                    .query_row(
                        "SELECT student_success_rate_bp
                         FROM resource_metadata_index
                         WHERE resource_type = ?1 AND resource_id = ?2",
                        params![resource_type, resource_id],
                        |row| row.get::<_, Option<i64>>(0),
                    )
                    .optional()
                    .map_err(|err| EcoachError::Storage(err.to_string()))?
                    .flatten()
                    .unwrap_or(6_000);
                let updated = clamp_bp((existing + usefulness_bp as i64 + status_bonus) / 2);
                self.conn
                    .execute(
                        "UPDATE resource_metadata_index
                         SET student_success_rate_bp = ?3
                         WHERE resource_type = ?1 AND resource_id = ?2",
                        params![resource_type, resource_id, updated as i64],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
        }
        Ok(())
    }

    fn upsert_resource_index(
        &self,
        resource_id: i64,
        resource_type: &str,
        subject_id: i64,
        topic_id: Option<i64>,
        subtopic_id: Option<i64>,
        concept_id: Option<i64>,
        content_type_id: Option<i64>,
        difficulty_bp: i64,
        exam_relevance_bp: i64,
        teach_suitability_bp: i64,
        test_suitability_bp: i64,
        pressure_suitability_bp: i64,
        source: Option<&str>,
        confidence_tier: &str,
        teacher_verified: bool,
        _metadata_json: Option<String>,
    ) -> EcoachResult<()> {
        let concept_id = concept_id.filter(|value| *value > 0);
        self.conn
            .execute(
                "INSERT INTO resource_metadata_index (
                    resource_id, resource_type, subject_id, topic_id, subtopic_id,
                    concept_id, content_type_id, difficulty_bp, exam_relevance_bp,
                    teach_suitability_bp, test_suitability_bp, pressure_suitability_bp,
                    source, confidence_tier, teacher_verified, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, datetime('now'))
                 ON CONFLICT(resource_id, resource_type) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_id = excluded.topic_id,
                    subtopic_id = excluded.subtopic_id,
                    concept_id = excluded.concept_id,
                    content_type_id = excluded.content_type_id,
                    difficulty_bp = excluded.difficulty_bp,
                    exam_relevance_bp = excluded.exam_relevance_bp,
                    teach_suitability_bp = excluded.teach_suitability_bp,
                    test_suitability_bp = excluded.test_suitability_bp,
                    pressure_suitability_bp = excluded.pressure_suitability_bp,
                    source = excluded.source,
                    confidence_tier = excluded.confidence_tier,
                    teacher_verified = excluded.teacher_verified",
                params![
                    resource_id,
                    resource_type,
                    subject_id,
                    topic_id,
                    subtopic_id,
                    concept_id,
                    content_type_id,
                    difficulty_bp,
                    exam_relevance_bp,
                    teach_suitability_bp,
                    test_suitability_bp,
                    pressure_suitability_bp,
                    source,
                    confidence_tier,
                    if teacher_verified { 1 } else { 0 },
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn insert_atom_if_missing(
        &self,
        node_id: i64,
        atom_type: &str,
        content_text: &str,
        representation_type: &str,
        mastery_level: i64,
        exam_relevance: i64,
    ) -> EcoachResult<()> {
        if node_id <= 0 || content_text.trim().is_empty() {
            return Ok(());
        }
        let exists = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM concept_atoms
                 WHERE node_id = ?1 AND atom_type = ?2 AND content_text = ?3",
                params![node_id, atom_type, content_text],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists == 0 {
            self.conn
                .execute(
                    "INSERT INTO concept_atoms (
                        node_id, atom_type, content_text, representation_type,
                        mastery_level, exam_relevance, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
                    params![
                        node_id,
                        atom_type,
                        content_text,
                        representation_type,
                        mastery_level,
                        exam_relevance,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn insert_evidence_if_missing(
        &self,
        source_type: &str,
        source_id: Option<i64>,
        topic_id: i64,
        concept_id: Option<i64>,
        claim_text: &str,
        supporting_text: Option<&str>,
        extraction_confidence_bp: i64,
        corroboration_score_bp: i64,
        pedagogy_score_bp: i64,
        freshness_score_bp: i64,
        status: &str,
    ) -> EcoachResult<()> {
        let concept_id = concept_id.filter(|value| *value > 0);
        let exists = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM evidence_blocks
                 WHERE source_type = ?1
                   AND source_id IS ?2
                   AND topic_id = ?3
                   AND claim_text = ?4",
                params![source_type, source_id, topic_id, claim_text],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if exists == 0 {
            self.conn
                .execute(
                    "INSERT INTO evidence_blocks (
                        source_type, source_id, topic_id, concept_id, evidence_type,
                        claim_text, supporting_text, extraction_confidence_bp,
                        corroboration_score_bp, pedagogy_score_bp, freshness_score_bp,
                        final_quality_bp, status, created_at
                     ) VALUES (?1, ?2, ?3, ?4, 'claim', ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))",
                    params![
                        source_type,
                        source_id,
                        topic_id,
                        concept_id,
                        claim_text,
                        supporting_text,
                        extraction_confidence_bp,
                        corroboration_score_bp,
                        pedagogy_score_bp,
                        freshness_score_bp,
                        clamp_bp(
                            (extraction_confidence_bp
                                + corroboration_score_bp
                                + pedagogy_score_bp
                                + freshness_score_bp)
                                / 4
                        ) as i64,
                        status,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn load_content_type_registry(&self) -> EcoachResult<BTreeMap<String, i64>> {
        let mut statement = self
            .conn
            .prepare("SELECT content_type_code, id FROM content_type_registry")
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut map = BTreeMap::new();
        for row in rows {
            let (code, id) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            map.insert(code, id);
        }
        Ok(map)
    }

    fn list_prerequisite_titles(&self, topic_id: i64) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT COALESCE(an.canonical_title, t.name)
                 FROM node_edges ne
                 LEFT JOIN academic_nodes an
                    ON ne.from_node_type = 'academic_node' AND an.id = ne.from_node_id
                 LEFT JOIN topics t
                    ON ne.from_node_type = 'topic' AND t.id = ne.from_node_id
                 WHERE ne.to_node_type = 'topic'
                   AND ne.to_node_id = ?1
                   AND ne.edge_type IN ('prerequisite', 'soft_prerequisite')
                 ORDER BY 1 ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut values = Vec::new();
        for row in rows {
            values.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(values)
    }

    fn list_string_column(&self, sql: &str, topic_id: i64) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut values = Vec::new();
        for row in rows {
            values.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(values)
    }

    fn count_official_documents_for_topic_id(&self, topic_id: i64) -> EcoachResult<i64> {
        let topic_name = self
            .conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.count_official_documents(&topic_name)
    }

    fn count_official_documents(&self, topic_name: &str) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(DISTINCT du.id)
                 FROM document_uploads du
                 LEFT JOIN document_mining_outputs dmo ON dmo.document_id = du.id
                 WHERE du.source_category IN ('official', 'teacher', 'extracted')
                   AND (
                        COALESCE(dmo.topics_detected_json, '') LIKE ?1
                        OR COALESCE(dmo.concepts_needing_remediation_json, '') LIKE ?1
                        OR COALESCE(dmo.definitions_json, '') LIKE ?1
                   )",
                [format!("%{}%", topic_name)],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_query(&self, sql: &str, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(sql, [topic_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_atoms_for_node(&self, node_id: i64, atom_types: &[&str]) -> EcoachResult<i64> {
        if node_id <= 0 || atom_types.is_empty() {
            return Ok(0);
        }
        let placeholders = atom_types
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT COUNT(*)
             FROM concept_atoms
             WHERE node_id = ?1 AND atom_type IN ({})",
            placeholders
        );
        let mut values = Vec::with_capacity(atom_types.len() + 1);
        values.push(rusqlite::types::Value::from(node_id));
        for atom_type in atom_types {
            values.push(rusqlite::types::Value::from(atom_type.to_string()));
        }
        self.conn
            .query_row(&sql, rusqlite::params_from_iter(values.iter()), |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_questions_for_node(&self, topic_id: i64, node_id: i64) -> EcoachResult<i64> {
        let (canonical_title, node_type): (String, String) = self
            .conn
            .query_row(
                "SELECT canonical_title, node_type
                 FROM academic_nodes
                 WHERE id = ?1",
                [node_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or_else(|| ("".to_string(), "".to_string()));
        if canonical_title.is_empty() {
            return Ok(0);
        }
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM questions
                 WHERE topic_id = ?1
                   AND is_active = 1
                   AND (
                        stem LIKE ?2
                        OR COALESCE(primary_content_type, '') = ?3
                   )",
                params![topic_id, format!("%{}%", canonical_title), node_type],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_teach_resources_for_node(&self, node_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM teach_explanations WHERE node_id = ?1",
                [node_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_validation_evidence(&self, topic_id: i64, node_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM evidence_blocks
                 WHERE topic_id = ?1
                   AND status = 'verified'
                   AND (concept_id = ?2 OR concept_id IS NULL)",
                params![topic_id, node_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn compute_node_confidence(&self, topic_id: i64, node_id: i64) -> EcoachResult<BasisPoints> {
        let evidence_bp = self
            .conn
            .query_row(
                "SELECT CAST(COALESCE(AVG(final_quality_bp), 0) AS INTEGER)
                 FROM evidence_blocks
                 WHERE topic_id = ?1
                   AND (concept_id = ?2 OR concept_id IS NULL)",
                params![topic_id, node_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let resource_bp = self
            .conn
            .query_row(
                "SELECT CAST(COALESCE(AVG(
                    CASE confidence_tier
                        WHEN 'teacher_authored' THEN 9200
                        WHEN 'performance_tested' THEN 8600
                        WHEN 'syllabus_aligned' THEN 8000
                        WHEN 'partially_matched' THEN 6200
                        WHEN 'inferred' THEN 5600
                        ELSE 5000
                    END
                 ), 0) AS INTEGER)
                 FROM resource_metadata_index
                 WHERE topic_id = ?1
                   AND (concept_id = ?2 OR concept_id IS NULL)",
                params![topic_id, node_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(clamp_bp((evidence_bp + resource_bp) / 2))
    }
}

fn selected_role(role_code: &str, purpose: &str, estimated_minutes: i64) -> SelectedRole {
    SelectedRole {
        role_code: role_code.to_string(),
        purpose: purpose.to_string(),
        estimated_minutes,
        candidate: None,
    }
}

fn to_json<T: Serialize>(value: &T) -> EcoachResult<String> {
    serde_json::to_string(value).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_json_vec<T: for<'de> Deserialize<'de>>(raw: &str) -> EcoachResult<Vec<T>> {
    serde_json::from_str(raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_json_value(raw: Option<&str>) -> Value {
    raw.and_then(|value| serde_json::from_str::<Value>(value).ok())
        .unwrap_or(Value::Null)
}

fn normalize_phrase(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_tokens(value: &str) -> BTreeSet<String> {
    normalize_phrase(value)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

fn topic_match_score(query: &str, topic_name: &str, description: &str) -> BasisPoints {
    let query_norm = normalize_phrase(query);
    let topic_norm = normalize_phrase(topic_name);
    if query_norm.is_empty() || topic_norm.is_empty() {
        return 0;
    }
    let query_tokens = normalized_tokens(query);
    let topic_tokens = normalized_tokens(topic_name);
    let description_tokens = normalized_tokens(description);
    let overlap = query_tokens.intersection(&topic_tokens).count() as i64;
    let desc_overlap = query_tokens.intersection(&description_tokens).count() as i64;
    let exact_bonus = if query_norm.contains(&topic_norm) || topic_norm.contains(&query_norm) {
        4_500
    } else {
        0
    };
    clamp_bp(exact_bonus + overlap * 2_000 + desc_overlap * 700)
}

fn detect_objective_code(prompt_text: Option<&str>, session_mode: Option<&str>) -> String {
    let joined = format!(
        "{} {}",
        session_mode.unwrap_or_default(),
        prompt_text.unwrap_or_default()
    )
    .to_ascii_lowercase();
    if joined.contains("exam") || joined.contains("timed") || joined.contains("pressure") {
        "exam_prep".to_string()
    } else if joined.contains("confidence") {
        "confidence_rebuild".to_string()
    } else if joined.contains("teach") || joined.contains("understand") || joined.contains("learn")
    {
        "teach_topic".to_string()
    } else if joined.contains("repair") || joined.contains("mistake") || joined.contains("wrong") {
        "repair_gap".to_string()
    } else if joined.contains("formula") {
        "formula_anchor".to_string()
    } else if joined.contains("vocab") || joined.contains("term") || joined.contains("glossary") {
        "vocabulary_repair".to_string()
    } else {
        "guided_revision".to_string()
    }
}

fn session_mode_for_objective(objective_code: &str) -> String {
    match objective_code {
        "teach_topic" => "teach",
        "repair_gap" => "repair",
        "confidence_rebuild" => "confidence",
        "exam_prep" => "exam",
        _ => "adaptive",
    }
    .to_string()
}

fn learner_need_for_objective(objective_code: &str) -> &'static str {
    match objective_code {
        "teach_topic" => "understanding",
        "repair_gap" => "repair",
        "confidence_rebuild" => "confidence",
        "exam_prep" => "performance",
        "formula_anchor" => "formula_recall",
        "vocabulary_repair" => "term_clarity",
        _ => "mixed_revision",
    }
}

fn default_formats_for_objective(objective_code: &str) -> Vec<String> {
    match objective_code {
        "teach_topic" => vec!["teach_explanation", "worked_example", "question"],
        "repair_gap" => vec!["teach_explanation", "question", "drill"],
        "confidence_rebuild" => vec!["worked_example", "note", "question"],
        "exam_prep" => vec!["question", "drill", "quick_refresher"],
        "formula_anchor" => vec!["formula", "flashcard", "question"],
        "vocabulary_repair" => vec!["glossary", "flashcard", "question"],
        _ => vec!["worked_example", "question"],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn default_pedagogical_lens(session_mode: &str) -> Vec<String> {
    match session_mode {
        "teach" => vec!["direct_instruction", "worked_example_fading"],
        "repair" => vec!["error_analysis", "scaffolded_practice"],
        "exam" => vec!["pressure_conditioning", "interleaved_practice"],
        "confidence" => vec!["worked_example_fading", "self_explanation"],
        _ => vec!["spaced_retrieval", "self_explanation"],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn learner_lens_for_objective(objective_code: &str) -> Vec<String> {
    match objective_code {
        "confidence_rebuild" => vec!["confidence_support", "quick_win"],
        "repair_gap" => vec!["error_repair", "misconception_cleanup"],
        "exam_prep" => vec!["time_pressure", "exam_transfer"],
        _ => vec!["clarity", "progressive_independence"],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn title_case_role(code: &str) -> String {
    code.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn difficulty_fit_score(target_bp: BasisPoints, resource_difficulty_bp: i64) -> BasisPoints {
    let delta = (target_bp as i64 - resource_difficulty_bp).abs();
    clamp_bp(10_000 - delta.min(10_000))
}

fn learner_fit_score(
    role_code: &str,
    resource: &IndexedResourceRow,
    learner_signals: &LearnerSignals,
) -> BasisPoints {
    let mut score = 5_500;
    if learner_signals.gap_score >= 5_000 {
        score += if matches!(
            role_code,
            "concept_explainer" | "worked_example" | "misconception_corrector"
        ) {
            2_000
        } else {
            400
        };
    }
    if learner_signals.fragility_score >= 5_000 && role_code == "confidence_rebuilder" {
        score += 2_200;
    }
    if learner_signals.mastery_score >= 7_000 && role_code == "stretch_challenge" {
        score += 2_500;
    }
    if learner_signals.decay_risk >= 5_000 && role_code == "quick_refresher" {
        score += 1_800;
    }
    if learner_signals.wrong_answer_count > 0 && role_code == "misconception_corrector" {
        score += 2_000;
    }
    if resource.teacher_verified {
        score += 600;
    }
    clamp_bp(score)
}

fn role_fit_score(role_code: &str, resource: &IndexedResourceRow) -> BasisPoints {
    match (role_code, resource.resource_type.as_str()) {
        ("concept_explainer", "teach_explanation") => 9_400,
        ("concept_explainer", "glossary_entry") => 8_200,
        ("concept_explainer", "note") => 7_600,
        ("misconception_corrector", "teach_explanation") => 9_000,
        ("misconception_corrector", "note") => 8_000,
        ("quick_refresher", "flashcard") => 9_200,
        ("quick_refresher", "note") => 8_600,
        ("quick_refresher", "glossary_entry") => 8_200,
        ("worked_example", "worked_example") => 9_500,
        ("pressure_drill", "question") => 9_700,
        ("pressure_drill", "drill") => 9_200,
        ("accuracy_drill", "question") => 9_300,
        ("accuracy_drill", "drill") => 9_000,
        ("vocabulary_repair", "glossary_entry") => 9_700,
        ("vocabulary_repair", "flashcard") => 8_900,
        ("formula_anchor", "glossary_entry") => 9_200,
        ("formula_anchor", "flashcard") => 8_600,
        ("confidence_rebuilder", "worked_example") => 8_900,
        ("confidence_rebuilder", "teach_explanation") => 8_700,
        ("mastery_validator", "question") => 9_400,
        ("mastery_validator", "drill") => 8_700,
        ("stretch_challenge", "question") => 9_200,
        ("stretch_challenge", "drill") => 8_600,
        _ => 4_800,
    }
}

fn confidence_tier_score(confidence_tier: &str) -> BasisPoints {
    match confidence_tier {
        "teacher_authored" => 9_200,
        "performance_tested" => 8_800,
        "syllabus_aligned" => 8_200,
        "partially_matched" => 6_500,
        "inferred" => 5_800,
        _ => 5_200,
    }
}

fn parse_resource_ref(resource_ref: &str) -> Option<(String, i64)> {
    let (resource_type, id_raw) = resource_ref.split_once(':')?;
    let resource_id = id_raw.parse::<i64>().ok()?;
    Some((resource_type.to_string(), resource_id))
}

fn resolve_best_node_id(nodes: &[NodeRow], text: &str) -> i64 {
    nodes
        .iter()
        .max_by_key(|node| {
            topic_match_score(
                text,
                &node.canonical_title,
                node.primary_content_type.as_deref().unwrap_or_default(),
            )
        })
        .map(|node| node.id)
        .unwrap_or(0)
}

fn map_node_to_atom_type(node: &NodeRow) -> &str {
    match node.node_type.as_str() {
        "definition" => "definition",
        "concept" => "explanation",
        "formula" => "formula",
        "procedure" => "rule",
        "comparison" => "counterexample",
        "principle" => "rule",
        "rule" => "rule",
        "theorem" => "theorem",
        "worked_pattern" => "example",
        "application" => "application",
        "interpretation" => "explanation",
        "diagram_spatial" => "diagram_label",
        "proof_justification" => "derivation",
        "essay_structured" => "marking_point",
        "word_problem_translation" => "hint",
        "vocabulary" => "vocabulary",
        "symbol_notation" => "vocabulary",
        _ => "explanation",
    }
}

fn map_entry_to_atom_type(entry_type: &str) -> &str {
    match entry_type {
        "definition" => "definition",
        "formula" => "formula",
        "worked_example" | "example" => "example",
        "counterexample" => "counterexample",
        "application" => "application",
        "rule" => "rule",
        "theorem" => "theorem",
        _ => "explanation",
    }
}

fn entry_representation(entry_type: &str) -> &str {
    match entry_type {
        "definition" | "formula" => "flashcard",
        "worked_example" | "example" => "teach_mode",
        _ => "text",
    }
}

fn question_confidence_tier(
    source_type: Option<&str>,
    human_verified: bool,
    classification_confidence_bp: i64,
) -> &'static str {
    if human_verified {
        "teacher_authored"
    } else if matches!(source_type, Some("past_paper") | Some("official")) {
        "performance_tested"
    } else if classification_confidence_bp >= 7_000 {
        "syllabus_aligned"
    } else if classification_confidence_bp >= 5_000 {
        "partially_matched"
    } else {
        "ai_generated"
    }
}

fn question_exam_relevance(source_type: Option<&str>, exam_year: Option<i64>) -> i64 {
    let source_score = match source_type {
        Some("past_paper") => 9_200,
        Some("official") => 8_800,
        Some("generated") => 6_000,
        _ => 7_000,
    };
    let recency_bonus = exam_year
        .map(|year| ((year - 2018).max(0) * 150).min(1_200))
        .unwrap_or(0);
    clamp_bp(source_score + recency_bonus) as i64
}

fn entry_content_type_code(entry_type: &str) -> &str {
    match entry_type {
        "definition" => "definition",
        "formula" => "formula",
        "worked_example" | "example" => "worked_example",
        "rule" => "rule",
        "theorem" => "theorem",
        "application" => "application",
        _ => "concept",
    }
}

fn glossary_teach_suitability(entry_type: &str) -> i64 {
    match entry_type {
        "definition" => 8_600,
        "formula" => 8_200,
        "worked_example" | "example" => 8_900,
        _ => 7_000,
    }
}

fn glossary_test_suitability(entry_type: &str) -> i64 {
    match entry_type {
        "definition" => 6_600,
        "formula" => 7_800,
        "worked_example" | "example" => 7_200,
        _ => 6_400,
    }
}

fn glossary_pressure_suitability(entry_type: &str) -> i64 {
    match entry_type {
        "formula" => 8_000,
        "worked_example" | "example" => 6_600,
        _ => 5_600,
    }
}

fn explanation_level_difficulty(explanation_level: &str) -> i64 {
    match explanation_level {
        "foundation" => 4_200,
        "core" => 5_200,
        "exam" => 6_800,
        "advanced" => 7_800,
        _ => 5_400,
    }
}

fn artifact_resource_type(artifact_type: &str) -> Option<&'static str> {
    match artifact_type {
        "explanation" | "formula" => Some("note"),
        "worked_example" => Some("worked_example"),
        "drill" | "assessment" => Some("drill"),
        "question" => Some("question"),
        "glossary_entry" => Some("glossary_entry"),
        "audio_script" => Some("audio_segment"),
        _ => None,
    }
}

fn coverage_from_count(count: i64, target_count: i64) -> BasisPoints {
    if target_count <= 0 {
        return 0;
    }
    clamp_bp((count.min(target_count) * 10_000) / target_count)
}

fn coverage_color(overall_coverage_bp: BasisPoints) -> &'static str {
    match overall_coverage_bp {
        0..=3999 => "red",
        4000..=6499 => "amber",
        6500..=8499 => "green",
        _ => "blue",
    }
}

fn compute_topic_quality(readiness: &TopicResourceReadiness) -> BasisPoints {
    clamp_bp(
        (readiness.readiness_score as i64
            + coverage_from_count(readiness.question_count, 8) as i64
            + coverage_from_count(readiness.explanation_count, 3) as i64
            + coverage_from_count(readiness.worked_example_count, 3) as i64)
            / 4,
    )
}

fn live_health_from_readiness(readiness: &TopicResourceReadiness) -> &'static str {
    match readiness.readiness_score {
        0..=3999 => "incomplete",
        4000..=6499 => "thin",
        6500..=8499 => "healthy",
        _ => "strong",
    }
}

fn required_assets_for_role(role_code: &str) -> Vec<&'static str> {
    match role_code {
        "pressure_drill" | "accuracy_drill" | "mastery_validator" | "question_shortage" => {
            vec!["question", "drill"]
        }
        "misconception_corrector" | "misconception_unaddressed" => {
            vec!["note", "teach_explanation", "question"]
        }
        "formula_anchor" | "formula entries" => vec!["note", "flashcard", "question"],
        "vocabulary_repair" | "glossary definitions" => vec!["glossary_entry", "flashcard"],
        "worked_example" | "worked examples" => vec!["worked_example", "note"],
        "teach explanations" => vec!["teach_explanation", "note"],
        _ => vec!["note", "question"],
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    use super::*;
    use crate::PackService;

    #[test]
    fn sync_topic_resource_intelligence_builds_snapshot() {
        let conn = seeded_connection();
        let topic_id = lookup_topic_id(&conn, "FRA");

        let service = ResourceIntelligenceService::new(&conn);
        let snapshot = service
            .sync_topic_resource_intelligence(topic_id)
            .expect("topic intelligence snapshot should build");

        assert!(snapshot.concept_atom_count > 0);
        assert!(snapshot.indexed_resource_count > 0);
        assert!(snapshot.overall_coverage_bp > 0);
    }

    #[test]
    fn orchestrate_resources_and_record_learning_round_trip() {
        let conn = seeded_connection();
        let topic_id = lookup_topic_id(&conn, "FRA");
        let student_id = insert_student(&conn, "Akosua");
        let service = ResourceIntelligenceService::new(&conn);

        let result = service
            .orchestrate_resources(ResourceOrchestrationRequest {
                student_id,
                subject_id: Some(1),
                topic_id: Some(topic_id),
                prompt_text: Some("Help me repair fractions before the exam.".to_string()),
                objective_code: Some("repair_gap".to_string()),
                session_mode: None,
                preferred_formats: vec![],
                time_budget_minutes: Some(20),
                target_difficulty_bp: None,
            })
            .expect("resource orchestration should succeed");

        assert!(!result.assembled_steps.is_empty());
        assert!(result.selected_candidates.len() + result.generated_artifacts.len() > 0);

        let learning = service
            .record_learning_outcome(RecordResourceLearningInput {
                run_id: result.run_id,
                session_id: None,
                outcome_status: "improved".to_string(),
                usefulness_bp: 7_800,
                confidence_shift_bp: 600,
                speed_shift_bp: 200,
                accuracy_shift_bp: 500,
                learner_feedback: json!({ "note": "This helped." }),
                still_struggling: vec![],
                effective_resource_refs: result
                    .selected_candidates
                    .iter()
                    .take(1)
                    .map(|candidate| {
                        format!("{}:{}", candidate.resource_type, candidate.resource_id)
                    })
                    .collect(),
            })
            .expect("learning outcome should persist");

        assert_eq!(learning.run_id, result.run_id);
        assert_eq!(learning.outcome_status, "improved");
    }

    fn seeded_connection() -> Connection {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        run_runtime_migrations(&mut conn).expect("migrations should run");
        let service = PackService::new(&conn);
        service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        conn
    }

    fn lookup_topic_id(conn: &Connection, topic_code: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM topics WHERE code = ?1",
            [topic_code],
            |row| row.get::<_, i64>(0),
        )
        .expect("topic should exist")
    }

    fn insert_student(conn: &Connection, name: &str) -> i64 {
        conn.execute(
            "INSERT INTO accounts (
                account_type, display_name, pin_hash, pin_salt,
                entitlement_tier, status, first_run, created_at, updated_at
             ) VALUES ('student', ?1, 'hash', 'salt', 'standard', 'active', 0, datetime('now'), datetime('now'))",
            [name],
        )
        .expect("student account should insert");
        let student_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_profiles (account_id, created_at, updated_at)
             VALUES (?1, datetime('now'), datetime('now'))",
            [student_id],
        )
        .expect("student profile should insert");
        student_id
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
