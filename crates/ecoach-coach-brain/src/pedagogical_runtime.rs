use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration, Utc};
use ecoach_content::{ContentStrategyRegistry, ContentTypeStrategy};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub struct PedagogicalRuntimeService<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTeachingProfile {
    pub topic_id: i64,
    pub subject_id: i64,
    pub topic_name: String,
    pub primary_content_type: String,
    pub secondary_content_types: Vec<String>,
    pub representation_modes: Vec<String>,
    pub strategy_families: Vec<String>,
    pub drill_families: Vec<String>,
    pub mastery_evidence: Vec<String>,
    pub failure_signatures: Vec<String>,
    pub review_modes: Vec<String>,
    pub prerequisite_topic_ids: Vec<i64>,
    pub learning_unit_count: i64,
    pub instructional_object_count: i64,
    pub freshness_score_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionalObjectEnvelope {
    pub id: i64,
    pub object_key: String,
    pub topic_id: i64,
    pub learning_unit_id: Option<i64>,
    pub object_type: String,
    pub pedagogical_purpose: String,
    pub title: String,
    pub content_type_primary: String,
    pub representation_mode: Option<String>,
    pub response_mode: Option<String>,
    pub strategy_families: Vec<String>,
    pub drill_families: Vec<String>,
    pub mastery_evidence: Vec<String>,
    pub supported_failure_signatures: Vec<String>,
    pub difficulty_bp: BasisPoints,
    pub quality_score_bp: BasisPoints,
    pub effectiveness_score_bp: BasisPoints,
    pub source_ref: Option<String>,
    pub payload: Value,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerUnitStateSnapshot {
    pub learning_unit_id: i64,
    pub scope_key: String,
    pub presence_state: String,
    pub clarity_state: String,
    pub retrieval_state: String,
    pub execution_state: String,
    pub transfer_state: String,
    pub performance_state: String,
    pub diagnostic_confidence_bp: BasisPoints,
    pub recent_accuracy_bp: BasisPoints,
    pub delayed_accuracy_bp: BasisPoints,
    pub mixed_accuracy_bp: BasisPoints,
    pub timed_accuracy_bp: BasisPoints,
    pub latency_score_bp: BasisPoints,
    pub hint_dependence_bp: BasisPoints,
    pub confidence_alignment_bp: BasisPoints,
    pub decay_risk_bp: BasisPoints,
    pub dominant_failure_signature: Option<String>,
    pub preferred_strategy_families: Vec<String>,
    pub failed_strategy_families: Vec<String>,
    pub last_review_mode: Option<String>,
    pub next_review_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizationSnapshot {
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_id: Option<i64>,
    pub scope_key: String,
    pub observed_profile: Value,
    pub derived_profile: Value,
    pub inferred_profile: Value,
    pub strategic_control: Value,
    pub recommendation: Value,
    pub confidence_score_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEpisodeSummary {
    pub id: i64,
    pub topic_id: Option<i64>,
    pub learning_unit_id: Option<i64>,
    pub review_mode: String,
    pub stated_purpose: Option<String>,
    pub status: String,
    pub failure_code: Option<String>,
    pub intervention_family: Option<String>,
    pub evidence_strength_bp: BasisPoints,
    pub next_review_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingTurnPlan {
    pub id: i64,
    pub session_id: i64,
    pub student_id: i64,
    pub topic_id: Option<i64>,
    pub learning_unit_id: Option<i64>,
    pub turn_index: i64,
    pub move_type: String,
    pub instructional_intention: String,
    pub success_condition: String,
    pub diagnostic_focus: Option<String>,
    pub support_level: String,
    pub pressure_level: String,
    pub representation_mode: Option<String>,
    pub selected_object_id: Option<i64>,
    pub selected_review_episode_id: Option<i64>,
    pub local_state: Value,
    pub outcome_status: Option<String>,
    pub outcome_score_bp: Option<BasisPoints>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingRuntimeSnapshot {
    pub session_id: i64,
    pub student_id: i64,
    pub subject_id: Option<i64>,
    pub session_type: String,
    pub topic_ids: Vec<i64>,
    pub topic_profile: Option<TopicTeachingProfile>,
    pub personalization: Option<PersonalizationSnapshot>,
    pub active_turn: Option<TeachingTurnPlan>,
    pub turns: Vec<TeachingTurnPlan>,
    pub active_review_episodes: Vec<ReviewEpisodeSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PedagogicalAttemptSignal {
    pub student_id: i64,
    pub session_id: i64,
    pub question_id: i64,
    pub response_time_ms: Option<i64>,
    pub confidence_level: Option<String>,
    pub hint_count: i64,
    pub was_timed: bool,
    pub was_transfer_variant: bool,
    pub was_retention_check: bool,
    pub was_mixed_context: bool,
    pub is_correct: bool,
    pub error_type: Option<String>,
    pub recommended_action: Option<String>,
}

#[derive(Debug, Clone)]
struct TopicContext {
    topic_id: i64,
    subject_id: i64,
    topic_name: String,
}

#[derive(Debug, Clone)]
struct LearningUnitSeed {
    unit_key: String,
    topic_id: i64,
    subject_id: i64,
    node_id: Option<i64>,
    title: String,
    content_type_primary: String,
    representation_tags: Vec<String>,
    prerequisite_links: Vec<String>,
    mastery_evidence: Vec<String>,
    review_modes: Vec<String>,
    strategy_families: Vec<String>,
    drill_families: Vec<String>,
    failure_signatures: Vec<String>,
    difficulty_bp: BasisPoints,
    quality_score_bp: BasisPoints,
}

#[derive(Debug, Clone)]
struct LearningUnitRow {
    id: i64,
    unit_key: String,
    title: String,
    content_type_primary: String,
    representation_tags: Vec<String>,
    mastery_evidence: Vec<String>,
    review_modes: Vec<String>,
}

#[derive(Debug, Clone)]
struct QuestionContext {
    subject_id: i64,
    topic_id: i64,
    primary_content_type: String,
}

#[derive(Debug, Clone)]
struct SessionContext {
    session_id: i64,
    student_id: i64,
    subject_id: Option<i64>,
    session_type: String,
    topic_ids: Vec<i64>,
}

#[derive(Debug, Clone)]
struct TopicTruthAggregate {
    total_attempts: i64,
    decay_risk_bp: BasisPoints,
}

impl<'a> PedagogicalRuntimeService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn sync_topic_teaching_profile(&self, topic_id: i64) -> EcoachResult<TopicTeachingProfile> {
        let topic = self.load_topic_context(topic_id)?;
        let registry = ContentStrategyRegistry::core();
        let units = self.load_topic_units(&topic, &registry)?;
        let prerequisite_topic_ids = self.list_prerequisite_topic_ids(topic_id)?;
        let question_count = self.count_rows(
            "SELECT COUNT(*) FROM questions WHERE topic_id = ?1",
            topic_id,
        )?;
        let knowledge_count = self.count_rows(
            "SELECT COUNT(*) FROM knowledge_entries WHERE topic_id = ?1",
            topic_id,
        )?;

        let mut type_counts = BTreeMap::<String, i64>::new();
        let mut representation_modes = BTreeSet::<String>::new();
        let mut strategy_families = BTreeSet::<String>::new();
        let mut drill_families = BTreeSet::<String>::new();
        let mut mastery_evidence = BTreeSet::<String>::new();
        let mut failure_signatures = BTreeSet::<String>::new();
        let mut review_modes = BTreeSet::<String>::new();
        let mut learning_unit_count = 0_i64;
        let mut instructional_object_count = 0_i64;

        for unit in &units {
            *type_counts
                .entry(unit.content_type_primary.clone())
                .or_default() += 1;
            representation_modes.extend(unit.representation_tags.iter().cloned());
            strategy_families.extend(unit.strategy_families.iter().cloned());
            drill_families.extend(unit.drill_families.iter().cloned());
            mastery_evidence.extend(unit.mastery_evidence.iter().cloned());
            failure_signatures.extend(unit.failure_signatures.iter().cloned());
            review_modes.extend(unit.review_modes.iter().cloned());

            let learning_unit_id = self.upsert_learning_unit(unit)?;
            learning_unit_count += 1;
            instructional_object_count += self.sync_instructional_objects_for_unit(
                learning_unit_id,
                unit,
                question_count,
                knowledge_count,
            )?;
        }

        let primary_content_type = type_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(content_type, _)| content_type)
            .unwrap_or_else(|| "concept".to_string());
        let secondary_content_types = units
            .iter()
            .map(|unit| unit.content_type_primary.clone())
            .filter(|item| item != &primary_content_type)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let freshness_score_bp = clamp_bp(
            3_500
                + learning_unit_count * 900
                + instructional_object_count * 220
                + question_count.min(12) * 120
                + knowledge_count.min(12) * 110,
        );

        self.conn
            .execute(
                "INSERT INTO topic_teaching_profiles (
                    topic_id, subject_id, topic_name, primary_content_type,
                    secondary_content_types_json, representation_modes_json,
                    strategy_families_json, drill_families_json, mastery_evidence_json,
                    failure_signatures_json, review_modes_json, prerequisite_topic_ids_json,
                    learning_unit_count, instructional_object_count, freshness_score_bp, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, datetime('now'))
                 ON CONFLICT(topic_id) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_name = excluded.topic_name,
                    primary_content_type = excluded.primary_content_type,
                    secondary_content_types_json = excluded.secondary_content_types_json,
                    representation_modes_json = excluded.representation_modes_json,
                    strategy_families_json = excluded.strategy_families_json,
                    drill_families_json = excluded.drill_families_json,
                    mastery_evidence_json = excluded.mastery_evidence_json,
                    failure_signatures_json = excluded.failure_signatures_json,
                    review_modes_json = excluded.review_modes_json,
                    prerequisite_topic_ids_json = excluded.prerequisite_topic_ids_json,
                    learning_unit_count = excluded.learning_unit_count,
                    instructional_object_count = excluded.instructional_object_count,
                    freshness_score_bp = excluded.freshness_score_bp,
                    updated_at = datetime('now')",
                params![
                    topic.topic_id,
                    topic.subject_id,
                    topic.topic_name,
                    primary_content_type,
                    to_json(&secondary_content_types)?,
                    to_json(&representation_modes.into_iter().collect::<Vec<_>>())?,
                    to_json(&strategy_families.into_iter().collect::<Vec<_>>())?,
                    to_json(&drill_families.into_iter().collect::<Vec<_>>())?,
                    to_json(&mastery_evidence.into_iter().collect::<Vec<_>>())?,
                    to_json(&failure_signatures.into_iter().collect::<Vec<_>>())?,
                    to_json(&review_modes.into_iter().collect::<Vec<_>>())?,
                    to_json(&prerequisite_topic_ids)?,
                    learning_unit_count,
                    instructional_object_count,
                    freshness_score_bp as i64,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.get_topic_teaching_profile(topic_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!(
                "topic teaching profile {} missing after sync",
                topic_id
            ))
        })
    }

    pub fn get_topic_teaching_profile(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Option<TopicTeachingProfile>> {
        self.conn
            .query_row(
                "SELECT topic_id, subject_id, topic_name, primary_content_type,
                        secondary_content_types_json, representation_modes_json,
                        strategy_families_json, drill_families_json, mastery_evidence_json,
                        failure_signatures_json, review_modes_json, prerequisite_topic_ids_json,
                        learning_unit_count, instructional_object_count, freshness_score_bp
                 FROM topic_teaching_profiles
                 WHERE topic_id = ?1",
                [topic_id],
                |row| {
                    Ok(TopicTeachingProfile {
                        topic_id: row.get(0)?,
                        subject_id: row.get(1)?,
                        topic_name: row.get(2)?,
                        primary_content_type: row.get(3)?,
                        secondary_content_types: parse_json_text(row.get::<_, String>(4)?)
                            .map_err(map_serialization_err)?,
                        representation_modes: parse_json_text(row.get::<_, String>(5)?)
                            .map_err(map_serialization_err)?,
                        strategy_families: parse_json_text(row.get::<_, String>(6)?)
                            .map_err(map_serialization_err)?,
                        drill_families: parse_json_text(row.get::<_, String>(7)?)
                            .map_err(map_serialization_err)?,
                        mastery_evidence: parse_json_text(row.get::<_, String>(8)?)
                            .map_err(map_serialization_err)?,
                        failure_signatures: parse_json_text(row.get::<_, String>(9)?)
                            .map_err(map_serialization_err)?,
                        review_modes: parse_json_text(row.get::<_, String>(10)?)
                            .map_err(map_serialization_err)?,
                        prerequisite_topic_ids: parse_json_text(row.get::<_, String>(11)?)
                            .map_err(map_serialization_err)?,
                        learning_unit_count: row.get(12)?,
                        instructional_object_count: row.get(13)?,
                        freshness_score_bp: clamp_bp(row.get::<_, i64>(14)?),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn list_instructional_objects(
        &self,
        topic_id: i64,
        pedagogical_purpose: Option<&str>,
        limit: usize,
    ) -> EcoachResult<Vec<InstructionalObjectEnvelope>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, object_key, topic_id, learning_unit_id, object_type, pedagogical_purpose,
                        title, content_type_primary, representation_mode, response_mode,
                        strategy_families_json, drill_families_json, mastery_evidence_json,
                        supported_failure_signatures_json, difficulty_bp, quality_score_bp,
                        effectiveness_score_bp, source_ref, payload_json, status
                 FROM instructional_objects
                 WHERE topic_id = ?1
                   AND (?2 IS NULL OR pedagogical_purpose = ?2)
                   AND status = 'active'
                 ORDER BY effectiveness_score_bp DESC, quality_score_bp DESC, id ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(
                params![topic_id, pedagogical_purpose, limit.max(1) as i64],
                |row| {
                    Ok(InstructionalObjectEnvelope {
                        id: row.get(0)?,
                        object_key: row.get(1)?,
                        topic_id: row.get(2)?,
                        learning_unit_id: row.get(3)?,
                        object_type: row.get(4)?,
                        pedagogical_purpose: row.get(5)?,
                        title: row.get(6)?,
                        content_type_primary: row.get(7)?,
                        representation_mode: row.get(8)?,
                        response_mode: row.get(9)?,
                        strategy_families: parse_json_text(row.get::<_, String>(10)?)
                            .map_err(map_serialization_err)?,
                        drill_families: parse_json_text(row.get::<_, String>(11)?)
                            .map_err(map_serialization_err)?,
                        mastery_evidence: parse_json_text(row.get::<_, String>(12)?)
                            .map_err(map_serialization_err)?,
                        supported_failure_signatures: parse_json_text(row.get::<_, String>(13)?)
                            .map_err(map_serialization_err)?,
                        difficulty_bp: clamp_bp(row.get::<_, i64>(14)?),
                        quality_score_bp: clamp_bp(row.get::<_, i64>(15)?),
                        effectiveness_score_bp: clamp_bp(row.get::<_, i64>(16)?),
                        source_ref: row.get(17)?,
                        payload: parse_json_value(row.get::<_, String>(18)?),
                        status: row.get(19)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    pub fn get_personalization_snapshot(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Option<PersonalizationSnapshot>> {
        let scope_key = personalization_scope_key(student_id, subject_id, topic_id);
        self.conn
            .query_row(
                "SELECT student_id, subject_id, topic_id, scope_key, observed_profile_json,
                        derived_profile_json, inferred_profile_json, strategic_control_json,
                        recommendation_json, confidence_score_bp
                 FROM personalization_snapshots
                 WHERE scope_key = ?1",
                [scope_key],
                |row| {
                    Ok(PersonalizationSnapshot {
                        student_id: row.get(0)?,
                        subject_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        scope_key: row.get(3)?,
                        observed_profile: parse_json_value(row.get::<_, String>(4)?),
                        derived_profile: parse_json_value(row.get::<_, String>(5)?),
                        inferred_profile: parse_json_value(row.get::<_, String>(6)?),
                        strategic_control: parse_json_value(row.get::<_, String>(7)?),
                        recommendation: parse_json_value(row.get::<_, String>(8)?),
                        confidence_score_bp: clamp_bp(row.get::<_, i64>(9)?),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn build_personalization_snapshot(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<PersonalizationSnapshot> {
        let scope_key = personalization_scope_key(student_id, subject_id, topic_id);
        let observed_profile = self.collect_observed_profile(student_id, subject_id, topic_id)?;
        let derived_profile = self.collect_derived_profile(student_id, subject_id, topic_id)?;
        let inferred_profile = infer_profile(&observed_profile, &derived_profile);
        let strategic_control =
            derive_strategic_control(&observed_profile, &derived_profile, &inferred_profile);
        let recommendation =
            derive_recommendation(&strategic_control, &inferred_profile, &derived_profile);
        let confidence_score_bp = confidence_from_profile(&observed_profile, &derived_profile);

        self.conn
            .execute(
                "INSERT INTO personalization_snapshots (
                    student_id, subject_id, topic_id, scope_key, observed_profile_json,
                    derived_profile_json, inferred_profile_json, strategic_control_json,
                    recommendation_json, confidence_score_bp, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))
                 ON CONFLICT(scope_key) DO UPDATE SET
                    observed_profile_json = excluded.observed_profile_json,
                    derived_profile_json = excluded.derived_profile_json,
                    inferred_profile_json = excluded.inferred_profile_json,
                    strategic_control_json = excluded.strategic_control_json,
                    recommendation_json = excluded.recommendation_json,
                    confidence_score_bp = excluded.confidence_score_bp,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    subject_id,
                    topic_id,
                    scope_key,
                    observed_profile.to_string(),
                    derived_profile.to_string(),
                    inferred_profile.to_string(),
                    strategic_control.to_string(),
                    recommendation.to_string(),
                    confidence_score_bp as i64,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(PersonalizationSnapshot {
            student_id,
            subject_id,
            topic_id,
            scope_key,
            observed_profile,
            derived_profile,
            inferred_profile,
            strategic_control,
            recommendation,
            confidence_score_bp,
        })
    }

    pub fn initialize_session_runtime(
        &self,
        session_id: i64,
    ) -> EcoachResult<TeachingRuntimeSnapshot> {
        let session = self.load_session_context(session_id)?;
        if self.session_turn_count(session_id)? > 0 {
            return self.get_session_runtime_snapshot(session_id);
        }

        let primary_topic_id = session.topic_ids.first().copied();
        let topic_profile = if let Some(topic_id) = primary_topic_id {
            Some(self.sync_topic_teaching_profile(topic_id)?)
        } else {
            None
        };
        let personalization = match (session.subject_id, primary_topic_id) {
            (Some(subject_id), topic_id) => Some(self.build_personalization_snapshot(
                session.student_id,
                subject_id,
                topic_id,
            )?),
            _ => None,
        };
        let (move_type, intention, success_condition, purpose) =
            initial_turn_spec(&session.session_type);
        let selected_object = if let Some(topic_id) = primary_topic_id {
            self.select_instructional_object(topic_id, None, purpose, None)?
        } else {
            None
        };
        self.insert_turn(
            session.session_id,
            session.student_id,
            primary_topic_id,
            None,
            move_type,
            intention,
            success_condition,
            None,
            support_level_from_snapshot(personalization.as_ref()),
            pressure_level_from_snapshot(personalization.as_ref()),
            selected_object
                .as_ref()
                .and_then(|object| object.representation_mode.clone()),
            selected_object.as_ref().map(|object| object.id),
            None,
            json!({
                "session_type": session.session_type,
                "topic_profile_ready": topic_profile.is_some(),
                "personalization_ready": personalization.is_some(),
            }),
        )?;

        self.get_session_runtime_snapshot(session_id)
    }

    pub fn get_session_runtime_snapshot(
        &self,
        session_id: i64,
    ) -> EcoachResult<TeachingRuntimeSnapshot> {
        let session = self.load_session_context(session_id)?;
        let turns = self.list_turns(session_id, 12)?;
        let active_turn = turns
            .iter()
            .find(|turn| turn.outcome_status.is_none())
            .cloned();
        let primary_topic_id = session.topic_ids.first().copied();
        let topic_profile = match primary_topic_id {
            Some(topic_id) => self.get_topic_teaching_profile(topic_id)?,
            None => None,
        };
        let personalization = match (session.subject_id, primary_topic_id) {
            (Some(subject_id), topic_id) => {
                self.get_personalization_snapshot(session.student_id, subject_id, topic_id)?
            }
            _ => None,
        };

        Ok(TeachingRuntimeSnapshot {
            session_id: session.session_id,
            student_id: session.student_id,
            subject_id: session.subject_id,
            session_type: session.session_type,
            topic_ids: session.topic_ids,
            topic_profile,
            personalization,
            active_turn,
            turns,
            active_review_episodes: self.list_review_episodes_for_session(session_id, 8)?,
        })
    }

    pub fn record_attempt_feedback(
        &self,
        signal: PedagogicalAttemptSignal,
    ) -> EcoachResult<TeachingRuntimeSnapshot> {
        let question = self.load_question_context(signal.question_id)?;
        let profile = self.sync_topic_teaching_profile(question.topic_id)?;
        let learning_unit =
            self.resolve_learning_unit(question.topic_id, &question.primary_content_type)?;
        let unit_state =
            self.upsert_learner_unit_state(&signal, &question, learning_unit.as_ref())?;
        let review_episode_id =
            self.record_review_episode(&signal, &question, learning_unit.as_ref(), &unit_state)?;

        if let Some(turn) = self.latest_turn(signal.session_id)? {
            if turn.outcome_status.is_none() {
                let outcome_status = if signal.is_correct {
                    "success"
                } else {
                    "repair_required"
                };
                self.conn
                    .execute(
                        "UPDATE runtime_teaching_turns
                         SET outcome_status = ?2,
                             outcome_score_bp = ?3,
                             selected_review_episode_id = COALESCE(selected_review_episode_id, ?4),
                             completed_at = datetime('now')
                         WHERE id = ?1",
                        params![
                            turn.id,
                            outcome_status,
                            unit_state.recent_accuracy_bp as i64,
                            review_episode_id
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
                self.record_turn_feedback(&turn, review_episode_id, &signal, &unit_state)?;
            }
        }

        let personalization = self.build_personalization_snapshot(
            signal.student_id,
            question.subject_id,
            Some(question.topic_id),
        )?;
        self.plan_next_turn(
            signal.session_id,
            question.topic_id,
            learning_unit.as_ref(),
            &signal,
            &personalization,
            &profile,
            review_episode_id,
        )?;

        self.get_session_runtime_snapshot(signal.session_id)
    }

    pub fn record_resource_learning_feedback(&self, run_id: i64) -> EcoachResult<()> {
        let event = self
            .conn
            .query_row(
                "SELECT ror.student_id, ror.subject_id, ror.topic_id, ror.session_mode,
                        rle.session_id, rle.outcome_status, rle.usefulness_bp,
                        rle.confidence_shift_bp, rle.speed_shift_bp, rle.accuracy_shift_bp
                 FROM resource_learning_events rle
                 INNER JOIN resource_orchestration_runs ror ON ror.id = rle.run_id
                 WHERE rle.run_id = ?1
                 ORDER BY rle.id DESC
                 LIMIT 1",
                [run_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, String>(5)?,
                        clamp_bp(row.get::<_, i64>(6)?),
                        row.get::<_, i64>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, i64>(9)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let Some((
            student_id,
            subject_id,
            topic_id,
            session_mode,
            session_id,
            outcome_status,
            usefulness_bp,
            confidence_shift_bp,
            speed_shift_bp,
            accuracy_shift_bp,
        )) = event
        else {
            return Ok(());
        };
        let Some(subject_id) = subject_id else {
            return Ok(());
        };
        let Some(topic_id) = topic_id else {
            return Ok(());
        };

        let _ = self.sync_topic_teaching_profile(topic_id)?;
        let learning_unit = self.resolve_learning_unit(topic_id, "concept")?;
        let purpose = resource_purpose(session_mode.as_deref());
        if let Some(object) = self.select_instructional_object(
            topic_id,
            learning_unit.as_ref().map(|item| item.id),
            purpose,
            None,
        )? {
            self.conn
                .execute(
                    "INSERT INTO teaching_move_feedback (
                        instructional_object_id, student_id, session_id, feedback_source,
                        move_type, learning_delta_bp, retention_delta_bp, transfer_delta_bp,
                        pressure_delta_bp, confidence_delta_bp, effectiveness_label, payload_json
                     ) VALUES (?1, ?2, ?3, 'resource_learning', ?4, ?5, 0, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        object.id,
                        student_id,
                        session_id,
                        purpose,
                        accuracy_shift_bp,
                        speed_shift_bp,
                        speed_shift_bp,
                        confidence_shift_bp,
                        if usefulness_bp >= 7000 {
                            "effective"
                        } else {
                            "mixed"
                        },
                        json!({
                            "run_id": run_id,
                            "outcome_status": outcome_status,
                            "usefulness_bp": usefulness_bp,
                        })
                        .to_string(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.apply_object_effectiveness(object.id, usefulness_bp)?;
        }
        let _ = self.build_personalization_snapshot(student_id, subject_id, Some(topic_id))?;
        Ok(())
    }

    fn load_topic_context(&self, topic_id: i64) -> EcoachResult<TopicContext> {
        self.conn
            .query_row(
                "SELECT id, subject_id, name
                 FROM topics
                 WHERE id = ?1",
                [topic_id],
                |row| {
                    Ok(TopicContext {
                        topic_id: row.get(0)?,
                        subject_id: row.get(1)?,
                        topic_name: row.get(2)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_topic_units(
        &self,
        topic: &TopicContext,
        registry: &ContentStrategyRegistry,
    ) -> EcoachResult<Vec<LearningUnitSeed>> {
        let prerequisite_links = self
            .list_prerequisite_topic_ids(topic.topic_id)?
            .into_iter()
            .map(|id| format!("topic:{}", id))
            .collect::<Vec<_>>();
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, canonical_title, primary_content_type, representation_mode,
                        preferred_strategies_json, foundation_weight, exam_relevance_score, node_type
                 FROM academic_nodes
                 WHERE topic_id = ?1 AND is_active = 1
                 ORDER BY foundation_weight DESC, exam_relevance_score DESC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic.topic_id], |row| {
                let content_type = row
                    .get::<_, Option<String>>(2)?
                    .filter(|value| !value.trim().is_empty())
                    .or_else(|| row.get::<_, Option<String>>(7).ok().flatten())
                    .unwrap_or_else(|| "concept".to_string());
                let content_type = normalize_content_type(&content_type);
                let preferred = parse_json_text::<String>(
                    row.get::<_, Option<String>>(4)?
                        .unwrap_or_else(|| "[]".to_string()),
                )
                .map_err(map_serialization_err)?;
                let strategy = strategy_for_content_type(registry, &content_type);
                let foundation_weight = row.get::<_, i64>(5)?;
                let exam_relevance_score = row.get::<_, i64>(6)?;
                let difficulty_bp = clamp_bp(
                    4_000
                        + foundation_weight.clamp(0, 100) * 18
                        + exam_relevance_score.clamp(0, 100) * 18,
                );
                let quality_score_bp = clamp_bp(
                    5_400
                        + foundation_weight.clamp(0, 100) * 10
                        + if preferred.is_empty() { 0 } else { 700 },
                );
                Ok(LearningUnitSeed {
                    unit_key: learning_unit_key(topic.topic_id, row.get::<_, i64>(0).ok()),
                    topic_id: topic.topic_id,
                    subject_id: topic.subject_id,
                    node_id: row.get::<_, i64>(0).ok(),
                    title: row.get(1)?,
                    content_type_primary: content_type,
                    representation_tags: vec![
                        row.get::<_, Option<String>>(3)?
                            .unwrap_or_else(|| "text".to_string()),
                    ],
                    prerequisite_links: prerequisite_links.clone(),
                    mastery_evidence: strategy
                        .map(|entry| entry.mastery_evidence.clone())
                        .unwrap_or_else(default_mastery_evidence),
                    review_modes: strategy
                        .map(|entry| vec![entry.review_mode.clone()])
                        .unwrap_or_else(|| vec!["structural".to_string()]),
                    strategy_families: merge_strategy_families(
                        strategy
                            .map(|entry| entry.strategy_families.clone())
                            .unwrap_or_default(),
                        preferred,
                    ),
                    drill_families: strategy
                        .map(|entry| entry.drill_families.clone())
                        .unwrap_or_default(),
                    failure_signatures: strategy
                        .map(|entry| entry.failure_modes.clone())
                        .unwrap_or_else(default_failure_signatures),
                    difficulty_bp,
                    quality_score_bp,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut units = collect_rows(rows)?;
        if units.is_empty() {
            let fallback_type = self
                .conn
                .query_row(
                    "SELECT COALESCE(primary_content_type, 'concept')
                     FROM questions
                     WHERE topic_id = ?1
                     ORDER BY id ASC
                     LIMIT 1",
                    [topic.topic_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?
                .unwrap_or_else(|| "concept".to_string());
            let content_type = normalize_content_type(&fallback_type);
            let strategy = strategy_for_content_type(registry, &content_type);
            units.push(LearningUnitSeed {
                unit_key: learning_unit_key(topic.topic_id, None),
                topic_id: topic.topic_id,
                subject_id: topic.subject_id,
                node_id: None,
                title: topic.topic_name.clone(),
                content_type_primary: content_type,
                representation_tags: vec!["text".to_string()],
                prerequisite_links,
                mastery_evidence: strategy
                    .map(|entry| entry.mastery_evidence.clone())
                    .unwrap_or_else(default_mastery_evidence),
                review_modes: strategy
                    .map(|entry| vec![entry.review_mode.clone()])
                    .unwrap_or_else(|| vec!["structural".to_string()]),
                strategy_families: strategy
                    .map(|entry| entry.strategy_families.clone())
                    .unwrap_or_default(),
                drill_families: strategy
                    .map(|entry| entry.drill_families.clone())
                    .unwrap_or_default(),
                failure_signatures: strategy
                    .map(|entry| entry.failure_modes.clone())
                    .unwrap_or_else(default_failure_signatures),
                difficulty_bp: 5_000,
                quality_score_bp: 5_600,
            });
        }
        Ok(units)
    }

    fn upsert_learning_unit(&self, seed: &LearningUnitSeed) -> EcoachResult<i64> {
        self.conn
            .execute(
                "INSERT INTO learning_units (
                    unit_key, subject_id, topic_id, node_id, title, content_type_primary,
                    content_type_secondary_json, representation_tags_json, prerequisite_links_json,
                    mastery_evidence_json, review_modes_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '[]', ?7, ?8, ?9, ?10, datetime('now'))
                 ON CONFLICT(unit_key) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_id = excluded.topic_id,
                    node_id = excluded.node_id,
                    title = excluded.title,
                    content_type_primary = excluded.content_type_primary,
                    representation_tags_json = excluded.representation_tags_json,
                    prerequisite_links_json = excluded.prerequisite_links_json,
                    mastery_evidence_json = excluded.mastery_evidence_json,
                    review_modes_json = excluded.review_modes_json,
                    updated_at = datetime('now')",
                params![
                    seed.unit_key,
                    seed.subject_id,
                    seed.topic_id,
                    seed.node_id,
                    seed.title,
                    seed.content_type_primary,
                    to_json(&seed.representation_tags)?,
                    to_json(&seed.prerequisite_links)?,
                    to_json(&seed.mastery_evidence)?,
                    to_json(&seed.review_modes)?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .query_row(
                "SELECT id FROM learning_units WHERE unit_key = ?1",
                [seed.unit_key.as_str()],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn sync_instructional_objects_for_unit(
        &self,
        learning_unit_id: i64,
        seed: &LearningUnitSeed,
        question_count: i64,
        knowledge_count: i64,
    ) -> EcoachResult<i64> {
        let specs = [
            ("teach", "explanation_card", "explain"),
            ("probe", "diagnostic_probe", "explain_back"),
            ("repair", "repair_bundle", "guided_practice"),
            ("retrieve", "review_card", "recall"),
        ];
        for (purpose, object_type, response_mode) in specs {
            let object_key = format!("{}:{}", seed.unit_key, purpose);
            let title = format!("{} {}", title_prefix_for_purpose(purpose), seed.title);
            let quality_score_bp = clamp_bp(
                seed.quality_score_bp as i64
                    + question_count.min(10) * 120
                    + knowledge_count.min(10) * 100,
            );
            self.conn
                .execute(
                    "INSERT INTO instructional_objects (
                        object_key, topic_id, learning_unit_id, object_type, pedagogical_purpose,
                        title, content_type_primary, representation_mode, response_mode,
                        strategy_families_json, drill_families_json, mastery_evidence_json,
                        supported_failure_signatures_json, difficulty_bp, quality_score_bp,
                        effectiveness_score_bp, source_ref, payload_json, status, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, 'active', datetime('now'))
                     ON CONFLICT(object_key) DO UPDATE SET
                        topic_id = excluded.topic_id,
                        learning_unit_id = excluded.learning_unit_id,
                        object_type = excluded.object_type,
                        pedagogical_purpose = excluded.pedagogical_purpose,
                        title = excluded.title,
                        content_type_primary = excluded.content_type_primary,
                        representation_mode = excluded.representation_mode,
                        response_mode = excluded.response_mode,
                        strategy_families_json = excluded.strategy_families_json,
                        drill_families_json = excluded.drill_families_json,
                        mastery_evidence_json = excluded.mastery_evidence_json,
                        supported_failure_signatures_json = excluded.supported_failure_signatures_json,
                        difficulty_bp = excluded.difficulty_bp,
                        quality_score_bp = excluded.quality_score_bp,
                        source_ref = excluded.source_ref,
                        payload_json = excluded.payload_json,
                        updated_at = datetime('now')",
                    params![
                        object_key,
                        seed.topic_id,
                        learning_unit_id,
                        object_type,
                        purpose,
                        title,
                        seed.content_type_primary,
                        seed.representation_tags.first().cloned(),
                        response_mode,
                        to_json(&seed.strategy_families)?,
                        to_json(&seed.drill_families)?,
                        to_json(&seed.mastery_evidence)?,
                        to_json(&seed.failure_signatures)?,
                        seed.difficulty_bp as i64,
                        quality_score_bp as i64,
                        clamp_bp((quality_score_bp as i64 + 5_000) / 2) as i64,
                        seed.unit_key.clone(),
                        json!({
                            "learning_unit_key": seed.unit_key,
                            "review_modes": seed.review_modes,
                            "failure_signatures": seed.failure_signatures,
                        })
                        .to_string(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(specs.len() as i64)
    }

    fn load_session_context(&self, session_id: i64) -> EcoachResult<SessionContext> {
        let (student_id, subject_id, session_type, topic_ids_json) = self
            .conn
            .query_row(
                "SELECT student_id, subject_id, session_type, COALESCE(topic_ids, '[]')
                 FROM sessions
                 WHERE id = ?1",
                [session_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Option<i64>>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut topic_ids = parse_json_text::<i64>(topic_ids_json)?;
        if topic_ids.is_empty() {
            let mut statement = self
                .conn
                .prepare(
                    "SELECT DISTINCT q.topic_id
                     FROM session_items si
                     INNER JOIN questions q ON q.id = si.question_id
                     WHERE si.session_id = ?1
                     ORDER BY q.topic_id ASC",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let rows = statement
                .query_map([session_id], |row| row.get::<_, i64>(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            topic_ids = collect_rows(rows)?;
        }
        Ok(SessionContext {
            session_id,
            student_id,
            subject_id,
            session_type,
            topic_ids,
        })
    }

    fn load_question_context(&self, question_id: i64) -> EcoachResult<QuestionContext> {
        self.conn
            .query_row(
                "SELECT subject_id, topic_id, COALESCE(primary_content_type, 'concept')
                 FROM questions
                 WHERE id = ?1",
                [question_id],
                |row| {
                    Ok(QuestionContext {
                        subject_id: row.get(0)?,
                        topic_id: row.get(1)?,
                        primary_content_type: normalize_content_type(&row.get::<_, String>(2)?),
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn resolve_learning_unit(
        &self,
        topic_id: i64,
        content_type_primary: &str,
    ) -> EcoachResult<Option<LearningUnitRow>> {
        let by_type = self
            .conn
            .query_row(
                "SELECT id, unit_key, title, content_type_primary, representation_tags_json,
                        mastery_evidence_json, review_modes_json
                 FROM learning_units
                 WHERE topic_id = ?1
                   AND content_type_primary = ?2
                 ORDER BY id ASC
                 LIMIT 1",
                params![topic_id, content_type_primary],
                |row| map_learning_unit_row(row),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if by_type.is_some() {
            return Ok(by_type);
        }
        self.conn
            .query_row(
                "SELECT id, unit_key, title, content_type_primary, representation_tags_json,
                        mastery_evidence_json, review_modes_json
                 FROM learning_units
                 WHERE topic_id = ?1
                 ORDER BY id ASC
                 LIMIT 1",
                [topic_id],
                |row| map_learning_unit_row(row),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn upsert_learner_unit_state(
        &self,
        signal: &PedagogicalAttemptSignal,
        question: &QuestionContext,
        learning_unit: Option<&LearningUnitRow>,
    ) -> EcoachResult<LearnerUnitStateSnapshot> {
        let Some(learning_unit) = learning_unit else {
            return Err(EcoachError::NotFound(format!(
                "learning unit missing for topic {}",
                question.topic_id
            )));
        };
        let scope_key = format!("topic:{}", question.topic_id);
        let existing =
            self.load_learner_unit_state(signal.student_id, learning_unit.id, &scope_key)?;
        let topic_truth = self.load_topic_truth(signal.student_id, question.topic_id)?;
        let failure_code = failure_code_for_signal(signal);
        let review_mode = review_mode_for_signal(signal, failure_code.as_deref(), learning_unit);
        let recent_accuracy_bp = ema_basis_points(
            existing
                .as_ref()
                .map(|item| item.recent_accuracy_bp)
                .unwrap_or(5_000),
            correctness_bp(signal.is_correct),
            0.45,
        );
        let delayed_accuracy_bp = if signal.was_retention_check {
            ema_basis_points(
                existing
                    .as_ref()
                    .map(|item| item.delayed_accuracy_bp)
                    .unwrap_or(5_000),
                correctness_bp(signal.is_correct),
                0.55,
            )
        } else {
            existing
                .as_ref()
                .map(|item| item.delayed_accuracy_bp)
                .unwrap_or(5_000)
        };
        let mixed_accuracy_bp = if signal.was_transfer_variant || signal.was_mixed_context {
            ema_basis_points(
                existing
                    .as_ref()
                    .map(|item| item.mixed_accuracy_bp)
                    .unwrap_or(5_000),
                correctness_bp(signal.is_correct),
                0.5,
            )
        } else {
            existing
                .as_ref()
                .map(|item| item.mixed_accuracy_bp)
                .unwrap_or(5_000)
        };
        let timed_accuracy_bp = if signal.was_timed {
            ema_basis_points(
                existing
                    .as_ref()
                    .map(|item| item.timed_accuracy_bp)
                    .unwrap_or(5_000),
                correctness_bp(signal.is_correct),
                0.5,
            )
        } else {
            existing
                .as_ref()
                .map(|item| item.timed_accuracy_bp)
                .unwrap_or(5_000)
        };
        let state = LearnerUnitStateSnapshot {
            learning_unit_id: learning_unit.id,
            scope_key: scope_key.clone(),
            presence_state: presence_state_for_signal(signal, existing.as_ref()),
            clarity_state: clarity_state_for_signal(signal, failure_code.as_deref()),
            retrieval_state: retrieval_state_for_signal(signal, existing.as_ref()),
            execution_state: execution_state_for_signal(signal, failure_code.as_deref()),
            transfer_state: transfer_state_for_signal(signal),
            performance_state: performance_state_for_signal(signal, failure_code.as_deref()),
            diagnostic_confidence_bp: clamp_bp(
                3_500
                    + topic_truth.total_attempts.min(10) * 450
                    + if failure_code.is_some() { 900 } else { 0 },
            ),
            recent_accuracy_bp,
            delayed_accuracy_bp,
            mixed_accuracy_bp,
            timed_accuracy_bp,
            latency_score_bp: speed_score_from_response_time(signal.response_time_ms),
            hint_dependence_bp: ema_basis_points(
                existing
                    .as_ref()
                    .map(|item| item.hint_dependence_bp)
                    .unwrap_or(3_000),
                hint_dependence_score(signal.hint_count),
                0.5,
            ),
            confidence_alignment_bp: ema_basis_points(
                existing
                    .as_ref()
                    .map(|item| item.confidence_alignment_bp)
                    .unwrap_or(5_000),
                confidence_alignment_score(signal.confidence_level.as_deref(), signal.is_correct),
                0.45,
            ),
            decay_risk_bp: topic_truth.decay_risk_bp,
            dominant_failure_signature: failure_code.clone(),
            preferred_strategy_families: if signal.is_correct {
                learning_unit.review_modes.iter().take(1).cloned().collect()
            } else {
                learning_unit
                    .mastery_evidence
                    .iter()
                    .take(1)
                    .cloned()
                    .collect()
            },
            failed_strategy_families: if signal.is_correct {
                existing
                    .as_ref()
                    .map(|item| item.failed_strategy_families.clone())
                    .unwrap_or_default()
            } else {
                vec![recommended_strategy_family(
                    &review_mode,
                    failure_code.as_deref(),
                )]
            },
            last_review_mode: Some(review_mode.clone()),
            next_review_at: Some(next_review_at_for_mode(&review_mode, signal.is_correct)),
        };

        self.conn
            .execute(
                "INSERT INTO learner_unit_states (
                    student_id, learning_unit_id, scope_key, presence_state, clarity_state,
                    retrieval_state, execution_state, transfer_state, performance_state,
                    diagnostic_confidence_bp, recent_accuracy_bp, delayed_accuracy_bp,
                    mixed_accuracy_bp, timed_accuracy_bp, latency_score_bp, hint_dependence_bp,
                    confidence_alignment_bp, decay_risk_bp, dominant_failure_signature,
                    confusion_neighbors_json, misconception_flags_json,
                    preferred_strategy_families_json, failed_strategy_families_json,
                    last_review_mode, next_review_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, '[]', '[]', ?20, ?21, ?22, ?23, datetime('now'))
                 ON CONFLICT(student_id, learning_unit_id, scope_key) DO UPDATE SET
                    presence_state = excluded.presence_state,
                    clarity_state = excluded.clarity_state,
                    retrieval_state = excluded.retrieval_state,
                    execution_state = excluded.execution_state,
                    transfer_state = excluded.transfer_state,
                    performance_state = excluded.performance_state,
                    diagnostic_confidence_bp = excluded.diagnostic_confidence_bp,
                    recent_accuracy_bp = excluded.recent_accuracy_bp,
                    delayed_accuracy_bp = excluded.delayed_accuracy_bp,
                    mixed_accuracy_bp = excluded.mixed_accuracy_bp,
                    timed_accuracy_bp = excluded.timed_accuracy_bp,
                    latency_score_bp = excluded.latency_score_bp,
                    hint_dependence_bp = excluded.hint_dependence_bp,
                    confidence_alignment_bp = excluded.confidence_alignment_bp,
                    decay_risk_bp = excluded.decay_risk_bp,
                    dominant_failure_signature = excluded.dominant_failure_signature,
                    preferred_strategy_families_json = excluded.preferred_strategy_families_json,
                    failed_strategy_families_json = excluded.failed_strategy_families_json,
                    last_review_mode = excluded.last_review_mode,
                    next_review_at = excluded.next_review_at,
                    updated_at = datetime('now')",
                params![
                    signal.student_id,
                    learning_unit.id,
                    scope_key,
                    state.presence_state,
                    state.clarity_state,
                    state.retrieval_state,
                    state.execution_state,
                    state.transfer_state,
                    state.performance_state,
                    state.diagnostic_confidence_bp as i64,
                    state.recent_accuracy_bp as i64,
                    state.delayed_accuracy_bp as i64,
                    state.mixed_accuracy_bp as i64,
                    state.timed_accuracy_bp as i64,
                    state.latency_score_bp as i64,
                    state.hint_dependence_bp as i64,
                    state.confidence_alignment_bp as i64,
                    state.decay_risk_bp as i64,
                    state.dominant_failure_signature,
                    to_json(&state.preferred_strategy_families)?,
                    to_json(&state.failed_strategy_families)?,
                    state.last_review_mode,
                    state.next_review_at,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(state)
    }

    fn record_review_episode(
        &self,
        signal: &PedagogicalAttemptSignal,
        question: &QuestionContext,
        learning_unit: Option<&LearningUnitRow>,
        unit_state: &LearnerUnitStateSnapshot,
    ) -> EcoachResult<Option<i64>> {
        let Some(learning_unit) = learning_unit else {
            return Ok(None);
        };
        let review_mode_code = unit_state
            .last_review_mode
            .clone()
            .unwrap_or_else(|| "structural".to_string());
        let review_mode_id = self.lookup_review_mode_id(&review_mode_code)?;
        let intervention_family = recommended_strategy_family(
            &review_mode_code,
            unit_state.dominant_failure_signature.as_deref(),
        );
        self.conn
            .execute(
                "INSERT INTO review_episodes (
                    student_id, topic_id, review_mode_id, session_id, stated_purpose, status,
                    scheduled_for_at, completed_at, learning_unit_id, failure_code,
                    intervention_family, evidence_strength_bp, outcome_summary_json, next_review_at,
                    created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 'completed', datetime('now'), datetime('now'), ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'))",
                params![
                    signal.student_id,
                    question.topic_id,
                    review_mode_id,
                    signal.session_id,
                    signal
                        .recommended_action
                        .clone()
                        .unwrap_or_else(|| review_purpose_for_mode(&review_mode_code).to_string()),
                    learning_unit.id,
                    unit_state.dominant_failure_signature,
                    intervention_family,
                    unit_state.diagnostic_confidence_bp as i64,
                    json!({
                        "is_correct": signal.is_correct,
                        "error_type": signal.error_type,
                        "review_mode": review_mode_code,
                    })
                    .to_string(),
                    unit_state.next_review_at,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let completed_id = self.conn.last_insert_rowid();

        if let Some(next_review_at) = &unit_state.next_review_at {
            if !signal.is_correct
                || signal.was_timed
                || signal.was_transfer_variant
                || signal.was_retention_check
            {
                self.conn
                    .execute(
                        "INSERT INTO review_episodes (
                            student_id, topic_id, review_mode_id, session_id, stated_purpose, status,
                            scheduled_for_at, learning_unit_id, failure_code, intervention_family,
                            evidence_strength_bp, outcome_summary_json, next_review_at, created_at
                         ) VALUES (?1, ?2, ?3, ?4, ?5, 'scheduled', ?6, ?7, ?8, ?9, ?10, '{}', ?6, datetime('now'))",
                        params![
                            signal.student_id,
                            question.topic_id,
                            review_mode_id,
                            signal.session_id,
                            review_purpose_for_mode(&review_mode_code),
                            next_review_at,
                            learning_unit.id,
                            unit_state.dominant_failure_signature,
                            intervention_family,
                            unit_state.diagnostic_confidence_bp as i64,
                        ],
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?;
            }
        }

        Ok(Some(completed_id))
    }

    fn plan_next_turn(
        &self,
        session_id: i64,
        topic_id: i64,
        learning_unit: Option<&LearningUnitRow>,
        signal: &PedagogicalAttemptSignal,
        personalization: &PersonalizationSnapshot,
        profile: &TopicTeachingProfile,
        review_episode_id: Option<i64>,
    ) -> EcoachResult<TeachingTurnPlan> {
        let (move_type, purpose, intention, success_condition, diagnostic_focus) =
            next_turn_spec(signal, personalization, profile);
        let selected_object = self.select_instructional_object(
            topic_id,
            learning_unit.map(|item| item.id),
            purpose,
            signal.error_type.as_deref(),
        )?;
        let turn_id = self.insert_turn(
            session_id,
            signal.student_id,
            Some(topic_id),
            learning_unit.map(|item| item.id),
            move_type,
            intention,
            success_condition,
            diagnostic_focus.as_deref(),
            support_level_from_snapshot(Some(personalization)),
            pressure_level_from_snapshot(Some(personalization)),
            selected_object
                .as_ref()
                .and_then(|object| object.representation_mode.clone()),
            selected_object.as_ref().map(|object| object.id),
            review_episode_id,
            json!({
                "dominant_failure_signature": unit_state_signature(signal),
                "profile_primary_content_type": profile.primary_content_type,
                "purpose": purpose,
            }),
        )?;
        self.turn_by_id(turn_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("turn {} missing after insert", turn_id)))
    }

    fn insert_turn(
        &self,
        session_id: i64,
        student_id: i64,
        topic_id: Option<i64>,
        learning_unit_id: Option<i64>,
        move_type: &str,
        instructional_intention: &str,
        success_condition: &str,
        diagnostic_focus: Option<&str>,
        support_level: String,
        pressure_level: String,
        representation_mode: Option<String>,
        selected_object_id: Option<i64>,
        selected_review_episode_id: Option<i64>,
        local_state: Value,
    ) -> EcoachResult<i64> {
        let turn_index = self.session_turn_count(session_id)? + 1;
        self.conn
            .execute(
                "INSERT INTO runtime_teaching_turns (
                    session_id, student_id, topic_id, learning_unit_id, turn_index, move_type,
                    instructional_intention, success_condition, diagnostic_focus,
                    support_level, pressure_level, representation_mode, selected_object_id,
                    selected_review_episode_id, local_state_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    session_id,
                    student_id,
                    topic_id,
                    learning_unit_id,
                    turn_index,
                    move_type,
                    instructional_intention,
                    success_condition,
                    diagnostic_focus,
                    support_level,
                    pressure_level,
                    representation_mode,
                    selected_object_id,
                    selected_review_episode_id,
                    local_state.to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    fn latest_turn(&self, session_id: i64) -> EcoachResult<Option<TeachingTurnPlan>> {
        self.conn
            .query_row(
                "SELECT id, session_id, student_id, topic_id, learning_unit_id, turn_index,
                        move_type, instructional_intention, success_condition, diagnostic_focus,
                        support_level, pressure_level, representation_mode, selected_object_id,
                        selected_review_episode_id, local_state_json, outcome_status,
                        outcome_score_bp, created_at, completed_at
                 FROM runtime_teaching_turns
                 WHERE session_id = ?1
                 ORDER BY turn_index DESC
                 LIMIT 1",
                [session_id],
                |row| map_turn(row),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn turn_by_id(&self, turn_id: i64) -> EcoachResult<Option<TeachingTurnPlan>> {
        self.conn
            .query_row(
                "SELECT id, session_id, student_id, topic_id, learning_unit_id, turn_index,
                        move_type, instructional_intention, success_condition, diagnostic_focus,
                        support_level, pressure_level, representation_mode, selected_object_id,
                        selected_review_episode_id, local_state_json, outcome_status,
                        outcome_score_bp, created_at, completed_at
                 FROM runtime_teaching_turns
                 WHERE id = ?1",
                [turn_id],
                |row| map_turn(row),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_turns(&self, session_id: i64, limit: usize) -> EcoachResult<Vec<TeachingTurnPlan>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, session_id, student_id, topic_id, learning_unit_id, turn_index,
                        move_type, instructional_intention, success_condition, diagnostic_focus,
                        support_level, pressure_level, representation_mode, selected_object_id,
                        selected_review_episode_id, local_state_json, outcome_status,
                        outcome_score_bp, created_at, completed_at
                 FROM runtime_teaching_turns
                 WHERE session_id = ?1
                 ORDER BY turn_index DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![session_id, limit.max(1) as i64], |row| {
                map_turn(row)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_review_episodes_for_session(
        &self,
        session_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<ReviewEpisodeSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT re.id, re.topic_id, re.learning_unit_id, rm.mode_code,
                        re.stated_purpose, re.status, re.failure_code,
                        re.intervention_family, re.evidence_strength_bp,
                        re.next_review_at, re.created_at
                 FROM review_episodes re
                 INNER JOIN review_modes rm ON rm.id = re.review_mode_id
                 WHERE re.session_id = ?1
                 ORDER BY re.id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![session_id, limit.max(1) as i64], |row| {
                Ok(ReviewEpisodeSummary {
                    id: row.get(0)?,
                    topic_id: row.get(1)?,
                    learning_unit_id: row.get(2)?,
                    review_mode: row.get(3)?,
                    stated_purpose: row.get(4)?,
                    status: row.get(5)?,
                    failure_code: row.get(6)?,
                    intervention_family: row.get(7)?,
                    evidence_strength_bp: clamp_bp(row.get::<_, i64>(8)?),
                    next_review_at: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn record_turn_feedback(
        &self,
        turn: &TeachingTurnPlan,
        review_episode_id: Option<i64>,
        signal: &PedagogicalAttemptSignal,
        unit_state: &LearnerUnitStateSnapshot,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO teaching_move_feedback (
                    turn_id, instructional_object_id, review_episode_id, student_id, session_id,
                    feedback_source, move_type, learning_delta_bp, retention_delta_bp,
                    transfer_delta_bp, pressure_delta_bp, confidence_delta_bp,
                    effectiveness_label, payload_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 'attempt', ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    turn.id,
                    turn.selected_object_id,
                    review_episode_id,
                    signal.student_id,
                    signal.session_id,
                    turn.move_type,
                    unit_state.recent_accuracy_bp as i64 - 5000,
                    unit_state.delayed_accuracy_bp as i64 - 5000,
                    unit_state.mixed_accuracy_bp as i64 - 5000,
                    unit_state.timed_accuracy_bp as i64 - 5000,
                    unit_state.confidence_alignment_bp as i64 - 5000,
                    if signal.is_correct {
                        "effective"
                    } else {
                        "repair_needed"
                    },
                    json!({
                        "is_correct": signal.is_correct,
                        "error_type": signal.error_type,
                        "hint_count": signal.hint_count,
                    })
                    .to_string(),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if let Some(object_id) = turn.selected_object_id {
            self.apply_object_effectiveness(object_id, unit_state.recent_accuracy_bp)?;
        }
        Ok(())
    }

    fn apply_object_effectiveness(
        &self,
        object_id: i64,
        signal_score_bp: BasisPoints,
    ) -> EcoachResult<()> {
        let current = self
            .conn
            .query_row(
                "SELECT effectiveness_score_bp
                 FROM instructional_objects
                 WHERE id = ?1",
                [object_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or(5_000);
        let updated = ema_basis_points(clamp_bp(current), signal_score_bp, 0.3);
        self.conn
            .execute(
                "UPDATE instructional_objects
                 SET effectiveness_score_bp = ?2,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![object_id, updated as i64],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn select_instructional_object(
        &self,
        topic_id: i64,
        learning_unit_id: Option<i64>,
        purpose: &str,
        failure_code: Option<&str>,
    ) -> EcoachResult<Option<InstructionalObjectEnvelope>> {
        let mut objects = self.list_instructional_objects(topic_id, Some(purpose), 12)?;
        objects.sort_by_key(|object| {
            let base = -(object.effectiveness_score_bp as i64 + object.quality_score_bp as i64);
            let unit_bonus =
                if learning_unit_id.is_some() && object.learning_unit_id == learning_unit_id {
                    -1_000
                } else {
                    0
                };
            let failure_bonus = if failure_code
                .map(|failure| {
                    object
                        .supported_failure_signatures
                        .iter()
                        .any(|item| item == failure)
                })
                .unwrap_or(false)
            {
                -1_500
            } else {
                0
            };
            base + unit_bonus + failure_bonus
        });
        Ok(objects.into_iter().next())
    }

    fn load_learner_unit_state(
        &self,
        student_id: i64,
        learning_unit_id: i64,
        scope_key: &str,
    ) -> EcoachResult<Option<LearnerUnitStateSnapshot>> {
        self.conn
            .query_row(
                "SELECT learning_unit_id, scope_key, presence_state, clarity_state, retrieval_state,
                        execution_state, transfer_state, performance_state,
                        diagnostic_confidence_bp, recent_accuracy_bp, delayed_accuracy_bp,
                        mixed_accuracy_bp, timed_accuracy_bp, latency_score_bp,
                        hint_dependence_bp, confidence_alignment_bp, decay_risk_bp,
                        dominant_failure_signature, preferred_strategy_families_json,
                        failed_strategy_families_json, last_review_mode, next_review_at
                 FROM learner_unit_states
                 WHERE student_id = ?1 AND learning_unit_id = ?2 AND scope_key = ?3",
                params![student_id, learning_unit_id, scope_key],
                |row| {
                    Ok(LearnerUnitStateSnapshot {
                        learning_unit_id: row.get(0)?,
                        scope_key: row.get(1)?,
                        presence_state: row.get(2)?,
                        clarity_state: row.get(3)?,
                        retrieval_state: row.get(4)?,
                        execution_state: row.get(5)?,
                        transfer_state: row.get(6)?,
                        performance_state: row.get(7)?,
                        diagnostic_confidence_bp: clamp_bp(row.get::<_, i64>(8)?),
                        recent_accuracy_bp: clamp_bp(row.get::<_, i64>(9)?),
                        delayed_accuracy_bp: clamp_bp(row.get::<_, i64>(10)?),
                        mixed_accuracy_bp: clamp_bp(row.get::<_, i64>(11)?),
                        timed_accuracy_bp: clamp_bp(row.get::<_, i64>(12)?),
                        latency_score_bp: clamp_bp(row.get::<_, i64>(13)?),
                        hint_dependence_bp: clamp_bp(row.get::<_, i64>(14)?),
                        confidence_alignment_bp: clamp_bp(row.get::<_, i64>(15)?),
                        decay_risk_bp: clamp_bp(row.get::<_, i64>(16)?),
                        dominant_failure_signature: row.get(17)?,
                        preferred_strategy_families: parse_json_text(row.get::<_, String>(18)?)
                            .map_err(map_serialization_err)?,
                        failed_strategy_families: parse_json_text(row.get::<_, String>(19)?)
                            .map_err(map_serialization_err)?,
                        last_review_mode: row.get(20)?,
                        next_review_at: row.get(21)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_topic_truth(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<TopicTruthAggregate> {
        self.conn
            .query_row(
                "SELECT COALESCE(total_attempts, 0), COALESCE(decay_risk, 0)
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| {
                    Ok(TopicTruthAggregate {
                        total_attempts: row.get(0)?,
                        decay_risk_bp: clamp_bp(row.get::<_, i64>(1)?),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .ok_or_else(|| EcoachError::NotFound(format!("topic truth missing for {}", topic_id)))
    }

    fn lookup_review_mode_id(&self, mode_code: &str) -> EcoachResult<i64> {
        if let Some(mode_id) = self
            .conn
            .query_row(
                "SELECT id
                 FROM review_modes
                 WHERE mode_code = ?1",
                [mode_code],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
        {
            return Ok(mode_id);
        }

        let display_name = title_case_code(mode_code);
        self.conn
            .execute(
                "INSERT INTO review_modes (
                    mode_code, display_name, description, testing_purpose_json, typical_behaviors_json
                 ) VALUES (?1, ?2, ?3, '[]', '[]')
                 ON CONFLICT(mode_code) DO NOTHING",
                params![
                    mode_code,
                    display_name.clone(),
                    format!("Runtime-generated review mode for {}", display_name.to_lowercase()),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .query_row(
                "SELECT id
                 FROM review_modes
                 WHERE mode_code = ?1",
                [mode_code],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn collect_observed_profile(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Value> {
        let topic_filter = if let Some(topic_id) = topic_id {
            format!(" AND q.topic_id = {}", topic_id)
        } else {
            String::new()
        };
        let sql = format!(
            "SELECT COUNT(*),
                    CAST(COALESCE(AVG(CASE WHEN sqa.is_correct = 1 THEN 10000 ELSE 0 END), 0) AS INTEGER),
                    CAST(COALESCE(AVG(COALESCE(sqa.response_time_ms, 0)), 0) AS INTEGER),
                    COALESCE(SUM(CASE WHEN sqa.hint_count > 0 THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN sqa.was_timed = 1 THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN sqa.was_transfer_variant = 1 OR sqa.was_mixed_context = 1 THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN sqa.was_retention_check = 1 THEN 1 ELSE 0 END), 0)
             FROM student_question_attempts sqa
             INNER JOIN questions q ON q.id = sqa.question_id
             WHERE sqa.student_id = ?1
               AND q.subject_id = ?2{}",
            topic_filter
        );
        let attempts = self
            .conn
            .query_row(&sql, params![student_id, subject_id], |row| {
                Ok(json!({
                    "attempt_count": row.get::<_, i64>(0)?,
                    "accuracy_bp": clamp_bp(row.get::<_, i64>(1)?),
                    "avg_response_time_ms": row.get::<_, i64>(2)?,
                    "hinted_attempts": row.get::<_, i64>(3)?,
                    "timed_attempts": row.get::<_, i64>(4)?,
                    "transfer_attempts": row.get::<_, i64>(5)?,
                    "retention_attempts": row.get::<_, i64>(6)?,
                }))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(json!({
            "attempts": attempts,
            "weakest_representations": self.list_representation_strengths(student_id)?,
            "top_failures": self.list_failure_breakdown(student_id, subject_id, topic_id)?,
        }))
    }

    fn collect_derived_profile(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Value> {
        let topic_filter = if let Some(topic_id) = topic_id {
            format!(" AND sts.topic_id = {}", topic_id)
        } else {
            String::new()
        };
        let sql = format!(
            "SELECT CAST(COALESCE(AVG(sts.mastery_score), 0) AS INTEGER),
                    CAST(COALESCE(AVG(sts.gap_score), 0) AS INTEGER),
                    CAST(COALESCE(AVG(sts.fragility_score), 0) AS INTEGER),
                    CAST(COALESCE(AVG(sts.pressure_collapse_index), 0) AS INTEGER),
                    CAST(COALESCE(AVG(sts.retention_score), 0) AS INTEGER),
                    CAST(COALESCE(AVG(sts.transfer_score), 0) AS INTEGER),
                    CAST(COALESCE(AVG(sts.confidence_score), 0) AS INTEGER),
                    COALESCE(SUM(sts.total_attempts), 0)
             FROM student_topic_states sts
             INNER JOIN topics t ON t.id = sts.topic_id
             WHERE sts.student_id = ?1
               AND t.subject_id = ?2{}",
            topic_filter
        );
        let core = self
            .conn
            .query_row(&sql, params![student_id, subject_id], |row| {
                Ok(json!({
                    "mastery_bp": clamp_bp(row.get::<_, i64>(0)?),
                    "gap_bp": clamp_bp(row.get::<_, i64>(1)?),
                    "fragility_bp": clamp_bp(row.get::<_, i64>(2)?),
                    "pressure_bp": clamp_bp(row.get::<_, i64>(3)?),
                    "retention_bp": clamp_bp(row.get::<_, i64>(4)?),
                    "transfer_bp": clamp_bp(row.get::<_, i64>(5)?),
                    "confidence_bp": clamp_bp(row.get::<_, i64>(6)?),
                    "total_attempts": row.get::<_, i64>(7)?,
                }))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(json!({
            "core": core,
            "feedback_patterns": self.list_feedback_patterns(student_id, topic_id)?,
        }))
    }

    fn list_representation_strengths(&self, student_id: i64) -> EcoachResult<Vec<Value>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT format_code, accuracy_bp, total_attempts
                 FROM student_representation_strength
                 WHERE student_id = ?1
                 ORDER BY accuracy_bp ASC, total_attempts DESC, format_code ASC
                 LIMIT 6",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([student_id], |row| {
                Ok(json!({
                    "format_code": row.get::<_, String>(0)?,
                    "accuracy_bp": clamp_bp(row.get::<_, i64>(1)?),
                    "total_attempts": row.get::<_, i64>(2)?,
                }))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_failure_breakdown(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Vec<Value>> {
        let topic_filter = if let Some(topic_id) = topic_id {
            format!(" AND wad.topic_id = {}", topic_id)
        } else {
            String::new()
        };
        let sql = format!(
            "SELECT wad.error_type, COUNT(*)
             FROM wrong_answer_diagnoses wad
             INNER JOIN topics t ON t.id = wad.topic_id
             WHERE wad.student_id = ?1
               AND t.subject_id = ?2{}
             GROUP BY wad.error_type
             ORDER BY COUNT(*) DESC, wad.error_type ASC
             LIMIT 5",
            topic_filter
        );
        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                Ok(json!({
                    "error_type": row.get::<_, String>(0)?,
                    "count": row.get::<_, i64>(1)?,
                }))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_feedback_patterns(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<Vec<Value>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT io.strategy_families_json, AVG(tmf.learning_delta_bp), COUNT(*)
                 FROM teaching_move_feedback tmf
                 INNER JOIN instructional_objects io ON io.id = tmf.instructional_object_id
                 WHERE tmf.student_id = ?1
                   AND (?2 IS NULL OR io.topic_id = ?2)
                 GROUP BY io.id, io.strategy_families_json
                 ORDER BY AVG(tmf.learning_delta_bp) DESC, COUNT(*) DESC
                 LIMIT 8",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, topic_id], |row| {
                let avg_delta = row.get::<_, Option<f64>>(1)?.unwrap_or(0.0);
                Ok(json!({
                    "strategy_families": parse_json_text::<String>(row.get::<_, String>(0)?)
                        .map_err(map_serialization_err)?,
                    "avg_learning_delta_bp": avg_delta,
                    "sample_count": row.get::<_, i64>(2)?,
                }))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn list_prerequisite_topic_ids(&self, topic_id: i64) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT an_from.topic_id
                 FROM node_edges ne
                 INNER JOIN academic_nodes an_from ON an_from.id = ne.from_node_id
                 INNER JOIN academic_nodes an_to ON an_to.id = ne.to_node_id
                 WHERE an_to.topic_id = ?1
                   AND an_from.topic_id <> an_to.topic_id
                   AND ne.edge_type IN ('prerequisite', 'soft_prerequisite', 'depends_on')
                 ORDER BY an_from.topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([topic_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn count_rows(&self, sql: &str, value: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(sql, [value], |row| row.get(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn session_turn_count(&self, session_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM runtime_teaching_turns
                 WHERE session_id = ?1",
                [session_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }
}

fn map_learning_unit_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LearningUnitRow> {
    Ok(LearningUnitRow {
        id: row.get(0)?,
        unit_key: row.get(1)?,
        title: row.get(2)?,
        content_type_primary: row.get(3)?,
        representation_tags: parse_json_text(row.get::<_, String>(4)?)
            .map_err(map_serialization_err)?,
        mastery_evidence: parse_json_text(row.get::<_, String>(5)?)
            .map_err(map_serialization_err)?,
        review_modes: parse_json_text(row.get::<_, String>(6)?).map_err(map_serialization_err)?,
    })
}

fn map_turn(row: &rusqlite::Row<'_>) -> rusqlite::Result<TeachingTurnPlan> {
    Ok(TeachingTurnPlan {
        id: row.get(0)?,
        session_id: row.get(1)?,
        student_id: row.get(2)?,
        topic_id: row.get(3)?,
        learning_unit_id: row.get(4)?,
        turn_index: row.get(5)?,
        move_type: row.get(6)?,
        instructional_intention: row.get(7)?,
        success_condition: row.get(8)?,
        diagnostic_focus: row.get(9)?,
        support_level: row.get(10)?,
        pressure_level: row.get(11)?,
        representation_mode: row.get(12)?,
        selected_object_id: row.get(13)?,
        selected_review_episode_id: row.get(14)?,
        local_state: parse_json_value(row.get::<_, String>(15)?),
        outcome_status: row.get(16)?,
        outcome_score_bp: row.get::<_, Option<i64>>(17)?.map(clamp_bp),
        created_at: row.get(18)?,
        completed_at: row.get(19)?,
    })
}

fn parse_json_text<T: for<'de> Deserialize<'de>>(raw: String) -> EcoachResult<Vec<T>> {
    serde_json::from_str(&raw).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_json_value(raw: String) -> Value {
    serde_json::from_str(&raw).unwrap_or(Value::Null)
}

fn to_json<T: Serialize>(value: &T) -> EcoachResult<String> {
    serde_json::to_string(value).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> EcoachResult<Vec<T>> {
    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(out)
}

fn map_serialization_err(err: EcoachError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
}

fn normalize_content_type(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "process" | "sequence" | "worked_procedure" => "procedure".to_string(),
        "diagram" | "spatial" | "diagram_spatial_understanding" => "interpretation".to_string(),
        "word_problem_translation" | "word_problem" => "application".to_string(),
        "" => "concept".to_string(),
        other => other.to_string(),
    }
}

fn learning_unit_key(topic_id: i64, node_id: Option<i64>) -> String {
    match node_id {
        Some(node_id) => format!("topic:{}:node:{}", topic_id, node_id),
        None => format!("topic:{}:core", topic_id),
    }
}

fn strategy_for_content_type<'a>(
    registry: &'a ContentStrategyRegistry,
    content_type: &str,
) -> Option<&'a ContentTypeStrategy> {
    registry
        .strategies
        .iter()
        .find(|entry| entry.node_type == content_type)
}

fn merge_strategy_families(
    mut registry_strategies: Vec<String>,
    preferred_strategies: Vec<String>,
) -> Vec<String> {
    for item in preferred_strategies {
        if !registry_strategies.iter().any(|existing| existing == &item) {
            registry_strategies.push(item);
        }
    }
    registry_strategies
}

fn default_failure_signatures() -> Vec<String> {
    vec![
        "partial_understanding".to_string(),
        "memory_decay".to_string(),
        "cue_dependence".to_string(),
    ]
}

fn default_mastery_evidence() -> Vec<String> {
    vec![
        "clean_explanation".to_string(),
        "independent_recall".to_string(),
        "stable_transfer".to_string(),
    ]
}

fn title_prefix_for_purpose(purpose: &str) -> &'static str {
    match purpose {
        "teach" => "Teach",
        "probe" => "Probe",
        "repair" => "Repair",
        _ => "Review",
    }
}

fn title_case_code(value: &str) -> String {
    value
        .split('_')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => {
                    let mut word = first.to_uppercase().collect::<String>();
                    word.push_str(chars.as_str());
                    word
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn personalization_scope_key(student_id: i64, subject_id: i64, topic_id: Option<i64>) -> String {
    match topic_id {
        Some(topic_id) => format!(
            "student:{}:subject:{}:topic:{}",
            student_id, subject_id, topic_id
        ),
        None => format!("student:{}:subject:{}", student_id, subject_id),
    }
}

fn correctness_bp(is_correct: bool) -> BasisPoints {
    if is_correct { 10_000 } else { 0 }
}

fn ema_basis_points(current: BasisPoints, signal: BasisPoints, alpha: f64) -> BasisPoints {
    clamp_bp(((current as f64 * (1.0 - alpha)) + (signal as f64 * alpha)).round() as i64)
}

fn speed_score_from_response_time(response_time_ms: Option<i64>) -> BasisPoints {
    match response_time_ms.unwrap_or(20_000) {
        value if value <= 6_000 => 9_300,
        value if value <= 12_000 => 8_100,
        value if value <= 20_000 => 6_700,
        value if value <= 35_000 => 5_200,
        _ => 3_700,
    }
}

fn hint_dependence_score(hint_count: i64) -> BasisPoints {
    clamp_bp(2_000 + hint_count.clamp(0, 4) * 1_900)
}

fn confidence_alignment_score(confidence_level: Option<&str>, is_correct: bool) -> BasisPoints {
    let normalized = confidence_level.unwrap_or("").trim().to_ascii_lowercase();
    match (normalized.as_str(), is_correct) {
        ("certain" | "confident" | "sure", true) => 8_800,
        ("certain" | "confident" | "sure", false) => 2_200,
        ("guessed" | "guess" | "unsure" | "not_sure", true) => 6_200,
        ("guessed" | "guess" | "unsure" | "not_sure", false) => 7_200,
        _ => 5_000,
    }
}

fn failure_code_for_signal(signal: &PedagogicalAttemptSignal) -> Option<String> {
    if signal.is_correct {
        return None;
    }
    if signal.was_retention_check {
        return Some("memory_decay".to_string());
    }
    if signal.was_timed && matches!(signal.error_type.as_deref(), Some("pressure_breakdown")) {
        return Some("pressure_collapse".to_string());
    }
    match signal.error_type.as_deref() {
        Some("knowledge_gap") => Some("true_understanding_gap".to_string()),
        Some("conceptual_confusion") | Some("misconception_triggered") => {
            Some("concept_confusion".to_string())
        }
        Some("recognition_failure") => Some("partial_understanding".to_string()),
        Some("execution_error") => Some("step_omission".to_string()),
        Some("pressure_breakdown") => Some("pressure_collapse".to_string()),
        Some("expression_weakness") => Some("expression_bottleneck".to_string()),
        Some("speed_error") => Some("slow_processing".to_string()),
        Some("guessing_detected") => Some("cue_dependence".to_string()),
        _ if signal.was_transfer_variant || signal.was_mixed_context => {
            Some("translation_failure".to_string())
        }
        _ if signal.was_timed => Some("timed_collapse".to_string()),
        _ => Some("partial_understanding".to_string()),
    }
}

fn review_mode_for_signal(
    signal: &PedagogicalAttemptSignal,
    failure_code: Option<&str>,
    learning_unit: &LearningUnitRow,
) -> String {
    if signal.was_retention_check {
        return "delayed_retrieval".to_string();
    }
    if signal.was_timed && !signal.is_correct {
        return "pressure".to_string();
    }
    if signal.was_transfer_variant || signal.was_mixed_context {
        return "transfer".to_string();
    }
    if matches!(failure_code, Some("concept_confusion")) {
        return "contrast".to_string();
    }
    learning_unit
        .review_modes
        .first()
        .cloned()
        .unwrap_or_else(|| {
            if signal.is_correct {
                "immediate_reinforcement".to_string()
            } else {
                "structural".to_string()
            }
        })
}

fn next_review_at_for_mode(review_mode: &str, is_correct: bool) -> String {
    let hours = match (review_mode, is_correct) {
        ("immediate_reinforcement", true) => 18,
        ("delayed_retrieval", true) => 28,
        ("contrast", true) => 22,
        ("transfer", true) => 26,
        ("pressure", true) => 24,
        ("pressure", false) => 10,
        ("contrast", false) => 8,
        ("transfer", false) => 12,
        ("delayed_retrieval", false) => 8,
        _ if is_correct => 20,
        _ => 6,
    };
    (Utc::now() + Duration::hours(hours)).to_rfc3339()
}

fn presence_state_for_signal(
    signal: &PedagogicalAttemptSignal,
    existing: Option<&LearnerUnitStateSnapshot>,
) -> String {
    if signal.is_correct {
        return "productive".to_string();
    }
    if existing
        .map(|item| item.recent_accuracy_bp >= 5_500)
        .unwrap_or(false)
    {
        "recognition_only".to_string()
    } else {
        "latent".to_string()
    }
}

fn clarity_state_for_signal(
    signal: &PedagogicalAttemptSignal,
    failure_code: Option<&str>,
) -> String {
    if signal.is_correct {
        return "clear".to_string();
    }
    match failure_code {
        Some("concept_confusion") => "neighbor_contaminated",
        Some("step_omission") => "fragmented",
        Some("translation_failure") => "distorted",
        Some("true_understanding_gap") => "boundary_blurred",
        _ => "fragmented",
    }
    .to_string()
}

fn retrieval_state_for_signal(
    signal: &PedagogicalAttemptSignal,
    existing: Option<&LearnerUnitStateSnapshot>,
) -> String {
    if signal.was_retention_check && !signal.is_correct {
        return "decaying".to_string();
    }
    if signal.hint_count > 0 {
        return "cue_dependent".to_string();
    }
    if signal.is_correct && signal.was_retention_check {
        return "stable".to_string();
    }
    existing
        .map(|item| item.retrieval_state.clone())
        .unwrap_or_else(|| "fragile_retrieval".to_string())
}

fn execution_state_for_signal(
    signal: &PedagogicalAttemptSignal,
    failure_code: Option<&str>,
) -> String {
    if signal.is_correct {
        return "procedurally_sound".to_string();
    }
    match failure_code {
        Some("step_omission") => "step_omission",
        Some("translation_failure") => "translation_failed",
        Some("partial_understanding") => "selection_failed",
        _ => "sequence_fractured",
    }
    .to_string()
}

fn transfer_state_for_signal(signal: &PedagogicalAttemptSignal) -> String {
    if signal.was_transfer_variant || signal.was_mixed_context {
        if signal.is_correct {
            "transfer_ready".to_string()
        } else {
            "representation_locked".to_string()
        }
    } else {
        "surface_locked".to_string()
    }
}

fn performance_state_for_signal(
    signal: &PedagogicalAttemptSignal,
    failure_code: Option<&str>,
) -> String {
    if signal.was_timed && !signal.is_correct {
        return if matches!(failure_code, Some("slow_processing")) {
            "speed_limited".to_string()
        } else {
            "pressure_sensitive".to_string()
        };
    }
    if matches!(failure_code, Some("expression_bottleneck")) {
        return "expression_bottlenecked".to_string();
    }
    if signal.is_correct && signal.was_timed {
        return "robust".to_string();
    }
    "pressure_sensitive".to_string()
}

fn review_purpose_for_mode(review_mode: &str) -> &'static str {
    match review_mode {
        "contrast" => "Separate the confusable neighbors cleanly.",
        "transfer" => "Check whether the idea survives a changed context.",
        "pressure" => "Test whether the idea collapses under time.",
        "delayed_retrieval" => "Check whether the idea survived a time gap.",
        _ => "Stabilize the core understanding before pushing forward.",
    }
}

fn recommended_strategy_family(review_mode: &str, failure_code: Option<&str>) -> String {
    match (review_mode, failure_code) {
        ("contrast", _) | (_, Some("concept_confusion")) => "contrast".to_string(),
        ("pressure", _) | (_, Some("pressure_collapse")) | (_, Some("timed_collapse")) => {
            "timed_performance".to_string()
        }
        (_, Some("translation_failure")) => "translation".to_string(),
        (_, Some("step_omission")) => "faded_example".to_string(),
        ("delayed_retrieval", _) | (_, Some("memory_decay")) => "retrieval".to_string(),
        _ => "semantic_unpacking".to_string(),
    }
}

fn infer_profile(observed: &Value, derived: &Value) -> Value {
    let attempts = observed["attempts"]["attempt_count"].as_i64().unwrap_or(0);
    let hinted = observed["attempts"]["hinted_attempts"]
        .as_i64()
        .unwrap_or(0);
    let weakest_repr = observed["weakest_representations"]
        .as_array()
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or(Value::Null);
    let gap_bp = derived["core"]["gap_bp"].as_i64().unwrap_or(0);
    let fragility_bp = derived["core"]["fragility_bp"].as_i64().unwrap_or(0);
    let pressure_bp = derived["core"]["pressure_bp"].as_i64().unwrap_or(0);
    let transfer_bp = derived["core"]["transfer_bp"].as_i64().unwrap_or(0);

    json!({
        "cue_dependent": attempts > 0 && hinted * 3 >= attempts,
        "pressure_sensitive": pressure_bp >= 5500,
        "representation_locked": weakest_repr["accuracy_bp"].as_i64().unwrap_or(5000) <= 4200,
        "transfer_not_ready": transfer_bp <= 5000,
        "confidence_fragile": derived["core"]["confidence_bp"].as_i64().unwrap_or(0) <= 4500,
        "repair_priority": if gap_bp >= 6000 || fragility_bp >= 5500 { "high" } else if gap_bp >= 4500 { "medium" } else { "low" },
    })
}

fn derive_strategic_control(observed: &Value, derived: &Value, inferred: &Value) -> Value {
    let gap_bp = derived["core"]["gap_bp"].as_i64().unwrap_or(0);
    let pressure_sensitive = inferred["pressure_sensitive"].as_bool().unwrap_or(false);
    let cue_dependent = inferred["cue_dependent"].as_bool().unwrap_or(false);
    let transfer_not_ready = inferred["transfer_not_ready"].as_bool().unwrap_or(false);
    let avg_response_time_ms = observed["attempts"]["avg_response_time_ms"]
        .as_i64()
        .unwrap_or(20_000);

    json!({
        "support_level": if gap_bp >= 6000 || cue_dependent { "guided" } else { "independent" },
        "pressure_level": if pressure_sensitive { "calm" } else if avg_response_time_ms > 20000 { "light" } else { "moderate" },
        "preferred_review_mode": if pressure_sensitive { "pressure" } else if transfer_not_ready { "transfer" } else if cue_dependent { "delayed_retrieval" } else { "structural" },
        "next_campaign_phase": if gap_bp >= 6000 { "repair" } else if pressure_sensitive { "stabilize" } else { "advance" },
    })
}

fn derive_recommendation(strategic_control: &Value, inferred: &Value, derived: &Value) -> Value {
    let next_phase = strategic_control["next_campaign_phase"]
        .as_str()
        .unwrap_or("repair");
    let gap_bp = derived["core"]["gap_bp"].as_i64().unwrap_or(0);
    json!({
        "headline": match next_phase {
            "repair" => "Target the exact weak pattern before adding more content.",
            "stabilize" => "Hold accuracy steady, then reintroduce pressure slowly.",
            _ => "Push into transfer and mixed conditions without losing accuracy.",
        },
        "next_move_family": strategic_control["preferred_review_mode"],
        "avoid_early_timing": inferred["pressure_sensitive"],
        "urgency_bp": clamp_bp(gap_bp),
    })
}

fn confidence_from_profile(observed: &Value, derived: &Value) -> BasisPoints {
    clamp_bp(
        3_500
            + observed["attempts"]["attempt_count"]
                .as_i64()
                .unwrap_or(0)
                .min(12)
                * 300
            + derived["core"]["total_attempts"]
                .as_i64()
                .unwrap_or(0)
                .min(15)
                * 220,
    )
}

fn support_level_from_snapshot(snapshot: Option<&PersonalizationSnapshot>) -> String {
    snapshot
        .and_then(|item| item.strategic_control["support_level"].as_str())
        .unwrap_or("guided")
        .to_string()
}

fn pressure_level_from_snapshot(snapshot: Option<&PersonalizationSnapshot>) -> String {
    snapshot
        .and_then(|item| item.strategic_control["pressure_level"].as_str())
        .unwrap_or("calm")
        .to_string()
}

fn initial_turn_spec(
    session_type: &str,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match session_type {
        "coach_mission" => (
            "probe",
            "activate prior knowledge around the mission target",
            "Surface the real weak edge before spending the block.",
            "probe",
        ),
        "mock" | "custom_test" => (
            "ask_for_recall",
            "settle the learner into exam mode with one clean first move",
            "Start with a controlled success, then widen the pressure.",
            "retrieve",
        ),
        _ => (
            "ask_for_recall",
            "warm the learner up with low-noise retrieval",
            "Get one stable response before escalating.",
            "retrieve",
        ),
    }
}

fn next_turn_spec(
    signal: &PedagogicalAttemptSignal,
    personalization: &PersonalizationSnapshot,
    profile: &TopicTeachingProfile,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Option<String>,
) {
    let failure_code = failure_code_for_signal(signal);
    if !signal.is_correct {
        if matches!(failure_code.as_deref(), Some("concept_confusion")) {
            return (
                "contrast",
                "contrast",
                "separate the confusable concepts",
                "Learner rejects the trap and states the boundary cleanly.",
                failure_code,
            );
        }
        if matches!(
            failure_code.as_deref(),
            Some("pressure_collapse") | Some("timed_collapse")
        ) {
            return (
                "remove_timing",
                "repair",
                "reduce pressure and rebuild the move calmly",
                "Learner solves accurately before timing comes back.",
                failure_code,
            );
        }
        if matches!(failure_code.as_deref(), Some("step_omission")) {
            return (
                "show_worked_step",
                "repair",
                "repair the exact missing step",
                "Learner completes the setup independently.",
                failure_code,
            );
        }
        return (
            "clarify",
            "teach",
            "repair the core meaning before testing again",
            "Learner explains the idea in their own words.",
            failure_code,
        );
    }
    if personalization.inferred_profile["transfer_not_ready"]
        .as_bool()
        .unwrap_or(false)
    {
        return (
            "switch_representation",
            "transfer",
            "force the idea to survive a different representation",
            "Learner handles the variation without new prompting.",
            Some("transfer_readiness".to_string()),
        );
    }
    if personalization.inferred_profile["pressure_sensitive"]
        .as_bool()
        .unwrap_or(false)
        && profile.review_modes.iter().any(|item| item == "pressure")
    {
        return (
            "introduce_light_timing",
            "perform",
            "add a small amount of timing without breaking accuracy",
            "Learner stays accurate inside a lighter time box.",
            Some("pressure_readiness".to_string()),
        );
    }
    (
        "ask_for_explanation",
        "probe",
        "check whether the success was real understanding, not lucky selection",
        "Learner explains why the move works.",
        None,
    )
}

fn resource_purpose(session_mode: Option<&str>) -> &'static str {
    match session_mode.unwrap_or_default() {
        "repair" => "repair",
        "exam" => "perform",
        "teach" => "teach",
        _ => "retrieve",
    }
}

fn unit_state_signature(signal: &PedagogicalAttemptSignal) -> Option<String> {
    failure_code_for_signal(signal)
}
