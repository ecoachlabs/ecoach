use chrono::Utc;
use ecoach_substrate::{DomainEvent, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    CreateGapRepairPlanInput, GapDashboard, GapFeedItem, GapRepairFocus, GapRepairPlan,
    GapRepairPlanItem, GapScoreCard, GapSnapshotResult, GapTrendPoint, RepairItemStatus,
    SolidificationProgress, SolidificationSession,
};

// ── Gap severity thresholds ──
const GAP_CRITICAL_BP: i64 = 7500;
const GAP_HIGH_BP: i64 = 5500;
const GAP_MEDIUM_BP: i64 = 3500;

// ── Error profile weights for repair priority ──
const KNOWLEDGE_GAP_WEIGHT: f64 = 0.35;
const CONCEPTUAL_CONFUSION_WEIGHT: f64 = 0.25;
const RECOGNITION_FAILURE_WEIGHT: f64 = 0.20;
const EXECUTION_ERROR_WEIGHT: f64 = 0.20;

pub struct KnowledgeGapService<'a> {
    conn: &'a Connection,
}

impl<'a> KnowledgeGapService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // ── Gap scoring ──

    pub fn compute_gap_score_card(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<GapScoreCard> {
        // Get student_topic_state
        let (mastery_score, gap_score, _repair_priority): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT mastery_score, gap_score, repair_priority
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| EcoachError::NotFound(format!("topic state not found: {}", e)))?;

        let topic_name: String = self
            .conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get(0)
            })
            .map_err(|e| EcoachError::NotFound(format!("topic not found: {}", e)))?;

        // Get error profile
        let (kg, cc, rf, ee): (i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT knowledge_gap_score, conceptual_confusion_score,
                        recognition_failure_score, execution_error_score
                 FROM student_error_profiles
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap_or((0, 0, 0, 0));

        // Compute composite priority
        let composite_priority = (kg as f64 * KNOWLEDGE_GAP_WEIGHT
            + cc as f64 * CONCEPTUAL_CONFUSION_WEIGHT
            + rf as f64 * RECOGNITION_FAILURE_WEIGHT
            + ee as f64 * EXECUTION_ERROR_WEIGHT)
            .round() as i64;

        let effective_priority =
            clamp_bp(((gap_score as f64 * 0.5) + (composite_priority as f64 * 0.5)).round() as i64);

        let severity_label = match gap_score {
            s if s >= GAP_CRITICAL_BP => "critical",
            s if s >= GAP_HIGH_BP => "high",
            s if s >= GAP_MEDIUM_BP => "medium",
            _ => "low",
        }
        .to_string();

        // Check if repair plan exists
        let has_plan: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM gap_repair_plans
                 WHERE student_id = ?1 AND topic_id = ?2 AND status = 'active'",
                params![student_id, topic_id],
                |row| Ok(row.get::<_, i64>(0)? > 0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(GapScoreCard {
            student_id,
            topic_id,
            topic_name,
            gap_score: clamp_bp(gap_score),
            mastery_score: clamp_bp(mastery_score),
            knowledge_gap_score: clamp_bp(kg),
            conceptual_confusion_score: clamp_bp(cc),
            recognition_failure_score: clamp_bp(rf),
            execution_error_score: clamp_bp(ee),
            severity_label,
            repair_priority: effective_priority,
            has_active_repair_plan: has_plan,
        })
    }

    pub fn list_priority_gaps(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<GapScoreCard>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT topic_id FROM student_topic_states
                 WHERE student_id = ?1 AND gap_score > ?2
                 ORDER BY gap_score DESC, repair_priority DESC
                 LIMIT ?3",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let topic_ids: Vec<i64> = stmt
            .query_map(params![student_id, GAP_MEDIUM_BP, limit as i64], |row| {
                row.get(0)
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        let mut cards = Vec::new();
        for topic_id in topic_ids {
            cards.push(self.compute_gap_score_card(student_id, topic_id)?);
        }
        Ok(cards)
    }

    // ── Gap repair plans ──

    pub fn create_repair_plan(
        &self,
        input: &CreateGapRepairPlanInput,
    ) -> EcoachResult<GapRepairPlan> {
        if let Some(existing_plan_id) = self
            .conn
            .query_row(
                "SELECT id
                 FROM gap_repair_plans
                 WHERE student_id = ?1
                   AND topic_id = ?2
                   AND status = 'active'
                 ORDER BY updated_at DESC, id DESC
                 LIMIT 1",
                params![input.student_id, input.topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
        {
            return self.get_repair_plan(existing_plan_id);
        }

        let gap_card = self.compute_gap_score_card(input.student_id, input.topic_id)?;
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO gap_repair_plans (student_id, topic_id, status, priority_score, created_at, updated_at)
                 VALUES (?1, ?2, 'active', ?3, ?4, ?4)",
                params![input.student_id, input.topic_id, gap_card.repair_priority, now],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let plan_id = self.conn.last_insert_rowid();

        // Build repair items from academic nodes in the topic
        self.populate_repair_items(plan_id, input.topic_id, &gap_card)?;

        self.append_event(DomainEvent::new(
            "gap.repair_plan_created",
            plan_id.to_string(),
            json!({
                "student_id": input.student_id,
                "topic_id": input.topic_id,
                "gap_score": gap_card.gap_score,
                "priority": gap_card.repair_priority,
            }),
        ))?;

        self.get_repair_plan(plan_id)
    }

    fn populate_repair_items(
        &self,
        plan_id: i64,
        topic_id: i64,
        gap_card: &GapScoreCard,
    ) -> EcoachResult<()> {
        // Get academic nodes for this topic, ordered from foundation-first to advanced.
        // The curriculum schema does not currently store depth/sequence columns on academic_nodes,
        // so we derive a stable instructional order from prerequisite edges and node weights.
        let mut stmt = self
            .conn
            .prepare(
                "SELECT an.id
                 FROM academic_nodes an
                 WHERE an.topic_id = ?1
                   AND an.is_active = 1
                 ORDER BY
                    COALESCE((
                        SELECT COUNT(*)
                        FROM node_edges ne
                        WHERE ne.to_node_id = an.id
                          AND ne.to_node_type = 'academic_node'
                          AND ne.edge_type IN ('prerequisite', 'soft_prerequisite')
                    ), 0) ASC,
                    an.foundation_weight DESC,
                    an.exam_relevance_score DESC,
                    an.id ASC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let node_ids: Vec<i64> = stmt
            .query_map([topic_id], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        // Determine repair action type based on dominant error pattern
        let dominant_focus = self.dominant_focus_key(gap_card);
        let dominant_action = match dominant_focus.as_str() {
            "conceptual_confusion" => "clarify_confusion",
            "recognition_failure" => "drill_recognition",
            "execution_error" => "practice_with_scaffolding",
            _ => "teach_concept",
        };

        for (seq, node_id) in node_ids.iter().enumerate() {
            // Vary the repair action across the sequence
            let action = match seq % 3 {
                0 => dominant_action,
                1 => "practice_with_scaffolding",
                _ => "independent_drill",
            };
            let status = if seq == 0 { "active" } else { "pending" };

            self.conn
                .execute(
                    "INSERT INTO gap_repair_plan_items (plan_id, node_id, sequence_order, repair_action, status, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
                    params![plan_id, node_id, seq as i64, action, status],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        // If no academic nodes found, create a single topic-level item
        if node_ids.is_empty() {
            self.conn
                .execute(
                    "INSERT INTO gap_repair_plan_items (plan_id, node_id, sequence_order, repair_action, status, created_at)
                     VALUES (?1, NULL, 0, ?2, 'active', datetime('now'))",
                    params![plan_id, dominant_action],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        Ok(())
    }

    pub fn get_repair_plan(&self, plan_id: i64) -> EcoachResult<GapRepairPlan> {
        let (id, student_id, topic_id, status, priority, created_at, updated_at): (
            i64,
            i64,
            i64,
            String,
            i64,
            String,
            String,
        ) = self
            .conn
            .query_row(
                "SELECT id, student_id, topic_id, status, priority_score, created_at, updated_at
                 FROM gap_repair_plans WHERE id = ?1",
                [plan_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .map_err(|e| EcoachError::NotFound(format!("repair plan not found: {}", e)))?;

        let topic_name: Option<String> = self
            .conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let gap_card = self.compute_gap_score_card(student_id, topic_id).ok();
        let dominant_focus = gap_card
            .as_ref()
            .map(|card| self.dominant_focus_key(card))
            .unwrap_or_else(|| "knowledge_gap".to_string());
        let items = self.get_plan_items(plan_id, topic_id, &dominant_focus)?;

        let total = items.len() as i64;
        let completed = items.iter().filter(|i| i.status == "completed").count() as i64;
        let progress_percent = if total > 0 {
            to_bp(completed as f64 / total as f64)
        } else {
            0
        };
        let severity_label = gap_card
            .as_ref()
            .map(|card| card.severity_label.clone())
            .unwrap_or_else(|| "medium".to_string());
        let recommended_session_type = self.recommended_session_type(&dominant_focus).to_string();
        let rationale = gap_card
            .as_ref()
            .map(|card| self.plan_rationale(card))
            .unwrap_or_else(|| {
                "Repair sequence built from topic evidence; run the active item first.".to_string()
            });
        let focus_breakdown = gap_card
            .as_ref()
            .map(|card| self.focus_breakdown(card))
            .unwrap_or_default();

        Ok(GapRepairPlan {
            id,
            student_id,
            topic_id,
            topic_name,
            status,
            priority_score: clamp_bp(priority),
            severity_label,
            dominant_focus,
            recommended_session_type,
            rationale,
            focus_breakdown,
            items,
            progress_percent,
            created_at,
            updated_at,
        })
    }

    fn get_plan_items(
        &self,
        plan_id: i64,
        topic_id: i64,
        dominant_focus: &str,
    ) -> EcoachResult<Vec<GapRepairPlanItem>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT gpi.id, gpi.plan_id, gpi.node_id, an.canonical_title, an.node_type,
                        gpi.sequence_order, gpi.repair_action, gpi.status
                 FROM gap_repair_plan_items gpi
                 LEFT JOIN academic_nodes an ON an.id = gpi.node_id
                 WHERE gpi.plan_id = ?1
                 ORDER BY gpi.sequence_order ASC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([plan_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            let (
                id,
                plan_id,
                node_id,
                node_title,
                node_type,
                sequence_order,
                repair_action,
                status,
            ) = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
            let candidate_question_ids = self.list_candidate_question_ids(topic_id, node_id, 4)?;
            let misconception_titles = self.list_misconception_titles(topic_id, node_id, 3)?;
            let resource_titles =
                self.list_resource_titles(topic_id, &candidate_question_ids, 3)?;
            items.push(GapRepairPlanItem {
                id,
                plan_id,
                node_id,
                node_title: node_title.clone(),
                node_type: node_type.clone(),
                sequence_order,
                repair_action: repair_action.clone(),
                status,
                reason: self.item_reason(&repair_action, dominant_focus, node_title.as_deref()),
                target_outcome: self.item_target_outcome(
                    &repair_action,
                    node_type.as_deref(),
                    node_title.as_deref(),
                ),
                suggested_duration_minutes: self.suggested_duration_minutes(&repair_action),
                candidate_question_ids,
                misconception_titles,
                resource_titles,
            });
        }
        Ok(items)
    }

    pub fn advance_repair_item(
        &self,
        item_id: i64,
        new_status: RepairItemStatus,
    ) -> EcoachResult<GapRepairPlan> {
        let plan_id: i64 = self
            .conn
            .query_row(
                "SELECT plan_id FROM gap_repair_plan_items WHERE id = ?1",
                [item_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::NotFound(format!("repair item not found: {}", e)))?;

        self.conn
            .execute(
                "UPDATE gap_repair_plan_items SET status = ?1 WHERE id = ?2",
                params![new_status.as_str(), item_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        if matches!(new_status, RepairItemStatus::Active) {
            self.conn
                .execute(
                    "UPDATE gap_repair_plan_items
                     SET status = 'pending'
                     WHERE plan_id = ?1
                       AND id != ?2
                       AND status = 'active'",
                    params![plan_id, item_id],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        self.append_event(DomainEvent::new(
            "gap.repair_item_updated",
            item_id.to_string(),
            json!({
                "plan_id": plan_id,
                "status": new_status.as_str(),
            }),
        ))?;

        // If an item is finished, advance the plan to the next pending repair step.
        if matches!(
            new_status,
            RepairItemStatus::Completed | RepairItemStatus::Skipped
        ) {
            self.conn
                .execute(
                    "UPDATE gap_repair_plan_items SET status = 'active'
                     WHERE plan_id = ?1 AND status = 'pending'
                     AND sequence_order = (
                         SELECT MIN(sequence_order) FROM gap_repair_plan_items
                         WHERE plan_id = ?1 AND status = 'pending'
                     )",
                    [plan_id],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;

            // Check if all items complete => complete the plan
            let remaining: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM gap_repair_plan_items
                     WHERE plan_id = ?1 AND status IN ('pending', 'active')",
                    [plan_id],
                    |row| row.get(0),
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;

            if remaining == 0 {
                self.conn
                    .execute(
                        "UPDATE gap_repair_plans SET status = 'completed', updated_at = datetime('now') WHERE id = ?1",
                        [plan_id],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;

                self.append_event(DomainEvent::new(
                    "gap.repair_plan_completed",
                    plan_id.to_string(),
                    json!({}),
                ))?;
            } else {
                let active_item_id: Option<i64> = self
                    .conn
                    .query_row(
                        "SELECT id
                         FROM gap_repair_plan_items
                         WHERE plan_id = ?1 AND status = 'active'
                         ORDER BY sequence_order ASC
                         LIMIT 1",
                        [plan_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
                if let Some(active_item_id) = active_item_id {
                    self.append_event(DomainEvent::new(
                        "gap.repair_item_activated",
                        active_item_id.to_string(),
                        json!({
                            "plan_id": plan_id,
                        }),
                    ))?;
                }
            }
        }

        self.conn
            .execute(
                "UPDATE gap_repair_plans SET updated_at = datetime('now') WHERE id = ?1",
                [plan_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.get_repair_plan(plan_id)
    }

    pub fn list_active_repair_plans(&self, student_id: i64) -> EcoachResult<Vec<GapRepairPlan>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id FROM gap_repair_plans
                 WHERE student_id = ?1 AND status = 'active'
                 ORDER BY priority_score DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let plan_ids: Vec<i64> = stmt
            .query_map([student_id], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        let mut plans = Vec::new();
        for plan_id in plan_ids {
            plans.push(self.get_repair_plan(plan_id)?);
        }
        Ok(plans)
    }

    // ── Solidification sessions ──

    pub fn start_solidification_session(
        &self,
        student_id: i64,
        topic_id: i64,
        repair_plan_id: Option<i64>,
    ) -> EcoachResult<SolidificationSession> {
        let now = Utc::now().to_rfc3339();
        let subject_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT subject_id FROM topics WHERE id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .flatten();

        // Create underlying session
        let topic_ids_json = format!("[{}]", topic_id);
        self.conn
            .execute(
                "INSERT INTO sessions (
                    student_id, session_type, subject_id, topic_ids, difficulty_preference, status,
                    started_at, created_at, updated_at
                 ) VALUES (?1, 'gap_repair', ?2, ?3, 'adaptive', 'active', ?4, ?4, ?4)",
                params![student_id, subject_id, topic_ids_json, now],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let session_id = self.conn.last_insert_rowid();

        self.conn
            .execute(
                "INSERT INTO solidification_sessions (
                    student_id, topic_id, repair_plan_id, session_id, status, created_at
                 ) VALUES (?1, ?2, ?3, ?4, 'active', ?5)",
                params![student_id, topic_id, repair_plan_id, session_id, now],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let solid_id = self.conn.last_insert_rowid();
        let (repair_item_id, seeded_questions, estimated_minutes) =
            self.seed_solidification_session(session_id, topic_id, repair_plan_id)?;

        self.append_event(DomainEvent::new(
            "gap.solidification_started",
            solid_id.to_string(),
            json!({
                "student_id": student_id,
                "topic_id": topic_id,
                "repair_plan_id": repair_plan_id,
                "repair_item_id": repair_item_id,
                "seeded_question_count": seeded_questions,
                "estimated_minutes": estimated_minutes,
            }),
        ))?;

        self.get_solidification_session(solid_id)
    }

    pub fn complete_solidification_session(
        &self,
        solidification_id: i64,
    ) -> EcoachResult<SolidificationSession> {
        let now = Utc::now().to_rfc3339();
        let (student_id, topic_id, repair_plan_id, session_id): (
            i64,
            i64,
            Option<i64>,
            Option<i64>,
        ) = self
            .conn
            .query_row(
                "SELECT student_id, topic_id, repair_plan_id, session_id
                     FROM solidification_sessions
                     WHERE id = ?1",
                [solidification_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| {
                EcoachError::NotFound(format!("solidification session not found: {}", e))
            })?;
        let affected = self
            .conn
            .execute(
                "UPDATE solidification_sessions SET status = 'completed', completed_at = ?1
                 WHERE id = ?2 AND status = 'active'",
                params![now, solidification_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        if affected == 0 {
            return Err(EcoachError::Validation(
                "solidification session not active or not found".to_string(),
            ));
        }

        let repair_item_id = self.load_solidification_repair_item_id(solidification_id)?;
        let outcome = if let Some(sid) = session_id {
            Some(self.evaluate_solidification_session(sid)?)
        } else {
            None
        };

        if let Some(sid) = session_id {
            self.conn
                .execute(
                    "UPDATE sessions
                     SET status = 'completed',
                         completed_at = ?1,
                         answered_questions = ?2,
                         correct_questions = ?3,
                         accuracy_score = ?4,
                         avg_response_time_ms = ?5,
                         updated_at = ?1
                     WHERE id = ?6",
                    params![
                        now,
                        outcome.as_ref().map(|item| item.engaged_count).unwrap_or(0),
                        outcome.as_ref().map(|item| item.correct_count).unwrap_or(0),
                        outcome
                            .as_ref()
                            .map(|item| item.accuracy_score)
                            .unwrap_or(0),
                        outcome.as_ref().and_then(|item| item.avg_response_time_ms),
                        sid,
                    ],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        if let Some(outcome) = outcome.as_ref() {
            self.apply_solidification_outcome(
                student_id,
                topic_id,
                repair_plan_id,
                repair_item_id,
                outcome,
            )?;
            if let Some(session_id) = session_id {
                self.append_session_event(DomainEvent::new(
                    "session.interpreted",
                    session_id.to_string(),
                    json!({
                        "session_id": session_id,
                        "student_id": student_id,
                        "session_type": "gap_repair",
                        "status": "completed",
                        "next_action_hint": outcome.next_action_hint,
                        "interpretation_tags": outcome.interpretation_tags,
                        "repair_outcome": outcome.outcome_label,
                        "repair_item_id": repair_item_id,
                        "repair_plan_id": repair_plan_id,
                        "topic_summaries": [{
                            "topic_id": topic_id,
                            "accuracy_score": outcome.accuracy_score,
                            "answered_items": outcome.answered_count,
                            "skipped_items": outcome.skipped_count,
                            "correct_items": outcome.correct_count,
                            "total_items": outcome.total_count,
                            "dominant_error_type": outcome.dominant_signal,
                            "repair_outcome": outcome.outcome_label,
                        }],
                    }),
                ))?;
            }
        }

        self.append_event(DomainEvent::new(
            "gap.solidification_completed",
            solidification_id.to_string(),
            json!({
                "student_id": student_id,
                "topic_id": topic_id,
                "repair_plan_id": repair_plan_id,
                "session_id": session_id,
                "repair_item_id": repair_item_id,
                "outcome": outcome.as_ref().map(|item| item.outcome_label.clone()),
                "accuracy_score": outcome.as_ref().map(|item| item.accuracy_score),
            }),
        ))?;

        self.get_solidification_session(solidification_id)
    }

    pub fn get_solidification_session(&self, id: i64) -> EcoachResult<SolidificationSession> {
        self.conn
            .query_row(
                "SELECT id, student_id, topic_id, repair_plan_id, session_id, status, created_at, completed_at
                 FROM solidification_sessions WHERE id = ?1",
                [id],
                |row| {
                    Ok(SolidificationSession {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        repair_plan_id: row.get(3)?,
                        session_id: row.get(4)?,
                        status: row.get(5)?,
                        created_at: row.get(6)?,
                        completed_at: row.get(7)?,
                    })
                },
            )
            .map_err(|e| EcoachError::NotFound(format!("solidification session not found: {}", e)))
    }

    // ── Dashboard ──

    pub fn get_gap_dashboard(&self, student_id: i64) -> EcoachResult<GapDashboard> {
        let critical_gaps = self.list_priority_gaps(student_id, 10)?;
        let active_repairs = self.list_active_repair_plans(student_id)?;

        let (total_sol, completed_sol, active_sol): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT COUNT(*),
                        SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END),
                        SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END)
                 FROM solidification_sessions WHERE student_id = ?1",
                [student_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let topics_solidified: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT topic_id) FROM solidification_sessions
                 WHERE student_id = ?1 AND status = 'completed'",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(GapDashboard {
            student_id,
            critical_gaps,
            active_repairs,
            solidification_progress: SolidificationProgress {
                total_sessions: total_sol,
                completed_sessions: completed_sol,
                active_sessions: active_sol,
                topics_solidified,
            },
        })
    }

    // ── Internal ──

    fn seed_solidification_session(
        &self,
        session_id: i64,
        topic_id: i64,
        repair_plan_id: Option<i64>,
    ) -> EcoachResult<(Option<i64>, i64, i64)> {
        let repair_item: Option<(i64, Option<i64>, String)> = if let Some(plan_id) = repair_plan_id
        {
            self.conn
                .query_row(
                    "SELECT id, node_id, repair_action
                     FROM gap_repair_plan_items
                     WHERE plan_id = ?1
                     ORDER BY
                        CASE status
                            WHEN 'active' THEN 0
                            WHEN 'pending' THEN 1
                            WHEN 'skipped' THEN 2
                            ELSE 3
                        END,
                        sequence_order ASC
                     LIMIT 1",
                    [plan_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()
                .map_err(|e| EcoachError::Storage(e.to_string()))?
        } else {
            None
        };

        let (repair_item_id, node_id, repair_action) = if let Some(item) = repair_item {
            (Some(item.0), item.1, item.2)
        } else {
            (None, None, "practice_with_scaffolding".to_string())
        };
        let question_ids = self.list_candidate_question_ids(topic_id, node_id, 6)?;

        for (index, question_id) in question_ids.iter().enumerate() {
            let source_family_id: Option<i64> = self
                .conn
                .query_row(
                    "SELECT family_id FROM questions WHERE id = ?1",
                    [*question_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| EcoachError::Storage(e.to_string()))?
                .flatten();

            self.conn
                .execute(
                    "INSERT INTO session_items (
                        session_id, question_id, display_order, source_family_id, source_topic_id, status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, 'queued')",
                    params![
                        session_id,
                        question_id,
                        index as i64,
                        source_family_id,
                        topic_id,
                    ],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

        let question_count = question_ids.len() as i64;
        let estimated_minutes =
            self.suggested_duration_minutes(&repair_action) + question_count.saturating_sub(1) * 2;
        self.conn
            .execute(
                "UPDATE sessions
                 SET question_count = ?1,
                     total_questions = ?1,
                     duration_minutes = ?2,
                     updated_at = datetime('now')
                 WHERE id = ?3",
                params![question_count, estimated_minutes, session_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok((repair_item_id, question_count, estimated_minutes))
    }

    fn load_solidification_repair_item_id(
        &self,
        solidification_id: i64,
    ) -> EcoachResult<Option<i64>> {
        let payload_json: Option<String> = self
            .conn
            .query_row(
                "SELECT payload_json
                 FROM runtime_events
                 WHERE aggregate_kind = 'gap'
                   AND aggregate_id = ?1
                   AND event_type = 'gap.solidification_started'
                 ORDER BY occurred_at DESC, id DESC
                 LIMIT 1",
                [solidification_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .flatten();

        let Some(payload_json) = payload_json else {
            return Ok(None);
        };
        let payload: serde_json::Value = serde_json::from_str(&payload_json)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        Ok(payload
            .get("repair_item_id")
            .and_then(|value| value.as_i64()))
    }

    fn evaluate_solidification_session(
        &self,
        session_id: i64,
    ) -> EcoachResult<SolidificationOutcome> {
        let (total_count, answered_count, skipped_count, correct_count, avg_response_time_ms): (
            i64,
            i64,
            i64,
            i64,
            Option<i64>,
        ) = self
            .conn
            .query_row(
                "SELECT
                    COUNT(*),
                    COALESCE(SUM(CASE WHEN status = 'answered' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN status = 'skipped' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN is_correct = 1 THEN 1 ELSE 0 END), 0),
                    CAST(AVG(response_time_ms) AS INTEGER)
                 FROM session_items
                 WHERE session_id = ?1",
                [session_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let engaged_count = answered_count + skipped_count;
        let accuracy_score = if total_count > 0 {
            to_bp(correct_count as f64 / total_count as f64)
        } else {
            0
        };
        let coverage_score = if total_count > 0 {
            to_bp(engaged_count as f64 / total_count as f64)
        } else {
            0
        };
        let blended_score =
            ((accuracy_score as f64 * 0.75) + (coverage_score as f64 * 0.25)).round() as i64;

        let (outcome_label, next_action_hint, dominant_signal, interpretation_tags) =
            if total_count == 0 {
                (
                    "failed".to_string(),
                    "repair_retry".to_string(),
                    "repair_seed_missing".to_string(),
                    vec![
                        "gap_repair_failed".to_string(),
                        "gap_session_empty".to_string(),
                    ],
                )
            } else if blended_score >= 7600 && accuracy_score >= 7000 {
                (
                    "success".to_string(),
                    "stabilize_memory".to_string(),
                    "repair_success".to_string(),
                    vec![
                        "gap_repair_success".to_string(),
                        "ready_for_progression".to_string(),
                    ],
                )
            } else if blended_score >= 4500 {
                (
                    "mixed".to_string(),
                    "repair_retry".to_string(),
                    "repair_partial".to_string(),
                    vec!["gap_repair_mixed".to_string(), "needs_retry".to_string()],
                )
            } else {
                (
                    "failed".to_string(),
                    "reteach_before_retry".to_string(),
                    "repair_failed".to_string(),
                    vec![
                        "gap_repair_failed".to_string(),
                        "reteach_required".to_string(),
                    ],
                )
            };

        Ok(SolidificationOutcome {
            total_count,
            answered_count,
            skipped_count,
            engaged_count,
            correct_count,
            accuracy_score,
            coverage_score,
            avg_response_time_ms,
            outcome_label,
            next_action_hint,
            dominant_signal,
            interpretation_tags,
        })
    }

    fn apply_solidification_outcome(
        &self,
        student_id: i64,
        topic_id: i64,
        repair_plan_id: Option<i64>,
        repair_item_id: Option<i64>,
        outcome: &SolidificationOutcome,
    ) -> EcoachResult<()> {
        let repair_item_id = if repair_item_id.is_some() {
            repair_item_id
        } else if let Some(plan_id) = repair_plan_id {
            self.conn
                .query_row(
                    "SELECT id
                     FROM gap_repair_plan_items
                     WHERE plan_id = ?1
                       AND status IN ('active', 'pending')
                     ORDER BY
                        CASE status WHEN 'active' THEN 0 ELSE 1 END,
                        sequence_order ASC
                     LIMIT 1",
                    [plan_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(|e| EcoachError::Storage(e.to_string()))?
        } else {
            None
        };

        if let Some(item_id) = repair_item_id {
            match outcome.outcome_label.as_str() {
                "success" => {
                    let _ = self.advance_repair_item(item_id, RepairItemStatus::Completed)?;
                }
                "mixed" | "failed" => {
                    let _ = self.advance_repair_item(item_id, RepairItemStatus::Active)?;
                }
                _ => {}
            }
        }

        if let Some(plan_id) = repair_plan_id {
            let priority_delta = match outcome.outcome_label.as_str() {
                "success" => -1200,
                "mixed" => 500,
                "failed" => 1400,
                _ => 0,
            };
            if priority_delta != 0 {
                self.conn
                    .execute(
                        "UPDATE gap_repair_plans
                         SET priority_score = MIN(MAX(priority_score + ?1, 0), 10000),
                             updated_at = datetime('now')
                         WHERE id = ?2",
                        params![priority_delta, plan_id],
                    )
                    .map_err(|e| EcoachError::Storage(e.to_string()))?;
            }
        }

        let repair_priority_delta = match outcome.outcome_label.as_str() {
            "success" => -1500,
            "mixed" => 400,
            "failed" => 1600,
            _ => 0,
        };
        let urgent_flag = matches!(outcome.outcome_label.as_str(), "mixed" | "failed");
        self.conn
            .execute(
                "UPDATE student_topic_states
                 SET repair_priority = MIN(MAX(repair_priority + ?1, 0), 10000),
                     is_urgent = ?2,
                     last_decline_at = CASE
                         WHEN ?2 = 1 THEN datetime('now')
                         ELSE last_decline_at
                     END,
                     updated_at = datetime('now')
                 WHERE student_id = ?3
                   AND topic_id = ?4",
                params![
                    repair_priority_delta,
                    if urgent_flag { 1 } else { 0 },
                    student_id,
                    topic_id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.append_event(DomainEvent::new(
            "gap.solidification_evaluated",
            format!("{}:{}", student_id, topic_id),
            json!({
                "student_id": student_id,
                "topic_id": topic_id,
                "repair_plan_id": repair_plan_id,
                "repair_item_id": repair_item_id,
                "outcome": outcome.outcome_label,
                "accuracy_score": outcome.accuracy_score,
                "coverage_score": outcome.coverage_score,
                "next_action_hint": outcome.next_action_hint,
            }),
        ))?;

        Ok(())
    }

    fn append_session_event(&self, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, 'session', ?3, ?4, ?5, ?6)",
                params![
                    event.event_id,
                    event.event_type,
                    event.aggregate_id,
                    event.trace_id,
                    payload_json,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    fn append_event(&self, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, 'gap', ?3, ?4, ?5, ?6)",
                params![
                    event.event_id,
                    event.event_type,
                    event.aggregate_id,
                    event.trace_id,
                    payload_json,
                    event.occurred_at.to_rfc3339(),
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    fn dominant_focus_key(&self, gap_card: &GapScoreCard) -> String {
        let focus_scores = [
            ("knowledge_gap", gap_card.knowledge_gap_score),
            ("conceptual_confusion", gap_card.conceptual_confusion_score),
            ("recognition_failure", gap_card.recognition_failure_score),
            ("execution_error", gap_card.execution_error_score),
        ];

        focus_scores
            .into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(key, _)| key.to_string())
            .unwrap_or_else(|| "knowledge_gap".to_string())
    }

    fn focus_breakdown(&self, gap_card: &GapScoreCard) -> Vec<GapRepairFocus> {
        let mut focus = vec![
            GapRepairFocus {
                focus_key: "knowledge_gap".to_string(),
                score: gap_card.knowledge_gap_score,
                label: "Missing foundation".to_string(),
            },
            GapRepairFocus {
                focus_key: "conceptual_confusion".to_string(),
                score: gap_card.conceptual_confusion_score,
                label: "Concept confusion".to_string(),
            },
            GapRepairFocus {
                focus_key: "recognition_failure".to_string(),
                score: gap_card.recognition_failure_score,
                label: "Recognition failure".to_string(),
            },
            GapRepairFocus {
                focus_key: "execution_error".to_string(),
                score: gap_card.execution_error_score,
                label: "Execution weakness".to_string(),
            },
        ];
        focus.sort_by(|left, right| right.score.cmp(&left.score));
        focus
    }

    fn recommended_session_type(&self, dominant_focus: &str) -> &'static str {
        match dominant_focus {
            "conceptual_confusion" => "contrast_and_reteach",
            "recognition_failure" => "rapid_recognition_drill",
            "execution_error" => "worked_example_then_independent",
            _ => "teach_then_practice",
        }
    }

    fn plan_rationale(&self, gap_card: &GapScoreCard) -> String {
        format!(
            "This repair plan is prioritizing {} because the topic gap is {} and that error pattern is the strongest signal.",
            self.recommended_session_type(&self.dominant_focus_key(gap_card))
                .replace('_', " "),
            gap_card.severity_label
        )
    }

    fn item_reason(
        &self,
        repair_action: &str,
        dominant_focus: &str,
        node_title: Option<&str>,
    ) -> String {
        let node_label = node_title.unwrap_or("this topic");
        match repair_action {
            "teach_concept" => format!(
                "Start by reteaching {} because {} is the dominant gap signal.",
                node_label,
                dominant_focus.replace('_', " ")
            ),
            "clarify_confusion" => format!(
                "Use {} to separate closely-confused ideas before drilling.",
                node_label
            ),
            "drill_recognition" => format!(
                "Run quick recognition work on {} to improve pattern recall.",
                node_label
            ),
            "practice_with_scaffolding" => format!(
                "Practice {} with support so the learner can convert explanation into execution.",
                node_label
            ),
            "independent_drill" => format!(
                "Finish with independent questions on {} to verify the repair is holding.",
                node_label
            ),
            _ => format!("Repair {} with a focused intervention.", node_label),
        }
    }

    fn item_target_outcome(
        &self,
        repair_action: &str,
        node_type: Option<&str>,
        node_title: Option<&str>,
    ) -> String {
        let node_label = node_title.unwrap_or("the topic");
        let node_hint = node_type.unwrap_or("concept");
        match repair_action {
            "teach_concept" => format!(
                "Explain {} as a usable {} in the learner's own words.",
                node_label, node_hint
            ),
            "clarify_confusion" => format!(
                "Separate {} from its nearby confusions with clear contrasts.",
                node_label
            ),
            "drill_recognition" => format!(
                "Recognize {} quickly and accurately in mixed question sets.",
                node_label
            ),
            "practice_with_scaffolding" => {
                format!("Apply {} correctly with guided steps.", node_label)
            }
            "independent_drill" => format!(
                "Solve {} questions independently without prompts.",
                node_label
            ),
            _ => format!(
                "Strengthen {} enough to reduce repeated breakdowns.",
                node_label
            ),
        }
    }

    fn suggested_duration_minutes(&self, repair_action: &str) -> i64 {
        match repair_action {
            "teach_concept" | "clarify_confusion" => 15,
            "practice_with_scaffolding" => 12,
            "drill_recognition" => 10,
            "independent_drill" => 8,
            _ => 10,
        }
    }

    fn list_candidate_question_ids(
        &self,
        topic_id: i64,
        node_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<i64>> {
        let mut ids = Vec::new();
        if let Some(node_id) = node_id {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT q.id
                     FROM question_skill_links qsl
                     INNER JOIN questions q ON q.id = qsl.question_id
                     WHERE q.topic_id = ?1
                       AND qsl.node_id = ?2
                       AND q.is_active = 1
                     ORDER BY qsl.is_primary DESC, q.difficulty_level ASC, q.id ASC
                     LIMIT ?3",
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map(params![topic_id, node_id, limit as i64], |row| row.get(0))
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            for row in rows {
                ids.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
            }
        }

        if ids.len() < limit {
            let remaining = limit.saturating_sub(ids.len());
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT q.id
                     FROM questions q
                     WHERE q.topic_id = ?1
                       AND q.is_active = 1
                       AND (?2 IS NULL OR COALESCE(q.primary_skill_id, -1) != ?2)
                     ORDER BY q.difficulty_level ASC, q.id ASC
                     LIMIT ?3",
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map(params![topic_id, node_id, remaining as i64], |row| {
                    row.get(0)
                })
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            for row in rows {
                let question_id = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
                if !ids.contains(&question_id) {
                    ids.push(question_id);
                }
            }
        }

        Ok(ids)
    }

    fn list_misconception_titles(
        &self,
        topic_id: i64,
        node_id: Option<i64>,
        limit: usize,
    ) -> EcoachResult<Vec<String>> {
        let sql = if node_id.is_some() {
            "SELECT title
             FROM misconception_patterns
             WHERE is_active = 1 AND node_id = ?1
             ORDER BY severity DESC, id ASC
             LIMIT ?2"
        } else {
            "SELECT title
             FROM misconception_patterns
             WHERE is_active = 1 AND topic_id = ?1
             ORDER BY severity DESC, id ASC
             LIMIT ?2"
        };
        let anchor_id = node_id.unwrap_or(topic_id);
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![anchor_id, limit as i64], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        let mut titles = Vec::new();
        for row in rows {
            titles.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        if titles.is_empty() && node_id.is_some() {
            return self.list_misconception_titles(topic_id, None, limit);
        }
        Ok(titles)
    }

    fn list_resource_titles(
        &self,
        topic_id: i64,
        candidate_question_ids: &[i64],
        limit: usize,
    ) -> EcoachResult<Vec<String>> {
        let mut titles = Vec::new();
        for question_id in candidate_question_ids {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT ke.title
                     FROM question_glossary_links qgl
                     INNER JOIN knowledge_entries ke ON ke.id = qgl.entry_id
                     WHERE qgl.question_id = ?1
                     ORDER BY qgl.is_primary DESC, qgl.confidence_score DESC, ke.importance_score DESC, ke.id ASC
                     LIMIT ?2",
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map(params![question_id, limit as i64], |row| row.get(0))
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            for row in rows {
                let title: String = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
                if !titles.contains(&title) {
                    titles.push(title);
                }
                if titles.len() >= limit {
                    return Ok(titles);
                }
            }
        }

        if titles.len() < limit {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT title
                     FROM knowledge_entries
                     WHERE topic_id = ?1 AND status = 'active'
                     ORDER BY importance_score DESC, difficulty_level ASC, id ASC
                     LIMIT ?2",
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            let rows = stmt
                .query_map(params![topic_id, limit as i64], |row| row.get(0))
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            for row in rows {
                let title: String = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
                if !titles.contains(&title) {
                    titles.push(title);
                }
                if titles.len() >= limit {
                    break;
                }
            }
        }

        Ok(titles)
    }

    // -----------------------------------------------------------------------
    // Knowledge Gap Deep: snapshots, feed, aggregate breakdown
    // -----------------------------------------------------------------------

    /// Compute and persist a gap snapshot for trend visualization.
    pub fn capture_gap_snapshot(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<GapSnapshotResult> {
        // Count total skills for this subject
        let total_skills: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM student_skill_states sss
             INNER JOIN academic_nodes an ON an.id = sss.node_id
             INNER JOIN topics t ON t.id = an.topic_id
             WHERE sss.student_id = ?1 AND t.subject_id = ?2",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        if total_skills == 0 {
            // Fall back to topic-level
            return self.capture_topic_level_snapshot(student_id, subject_id);
        }

        // Count by state category
        let mastered: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM student_skill_states sss
             INNER JOIN academic_nodes an ON an.id = sss.node_id
             INNER JOIN topics t ON t.id = an.topic_id
             WHERE sss.student_id = ?1 AND t.subject_id = ?2 AND sss.mastery_score >= 8000",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        let weak: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM student_skill_states sss
             INNER JOIN academic_nodes an ON an.id = sss.node_id
             INNER JOIN topics t ON t.id = an.topic_id
             WHERE sss.student_id = ?1 AND t.subject_id = ?2
               AND sss.mastery_score >= 3000 AND sss.mastery_score < 6000",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        let unknown: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM student_skill_states sss
             INNER JOIN academic_nodes an ON an.id = sss.node_id
             INNER JOIN topics t ON t.id = an.topic_id
             WHERE sss.student_id = ?1 AND t.subject_id = ?2 AND sss.evidence_count < 3",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        let critical: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM student_skill_states sss
             INNER JOIN academic_nodes an ON an.id = sss.node_id
             INNER JOIN topics t ON t.id = an.topic_id
             WHERE sss.student_id = ?1 AND t.subject_id = ?2 AND sss.mastery_score < 3000 AND sss.evidence_count >= 3",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        let safe_total = total_skills.max(1);
        let total_gap = 100 - (mastered * 100 / safe_total);
        let unknown_pct = unknown * 100 / safe_total;
        let weak_pct = weak * 100 / safe_total;
        let critical_pct = critical * 100 / safe_total;
        let declining_pct = 0i64; // would need trend analysis
        let forgetting_pct = 0i64; // would need retention analysis

        self.conn.execute(
            "INSERT INTO gap_snapshots
                (student_id, subject_id, total_gap_percent, unknown_percent, weak_percent,
                 declining_percent, forgetting_percent, critical_percent,
                 total_skills, mastered_skills, critical_blockers)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                student_id, subject_id, total_gap, unknown_pct, weak_pct,
                declining_pct, forgetting_pct, critical_pct,
                total_skills, mastered, critical,
            ],
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(GapSnapshotResult {
            total_gap_percent: total_gap,
            unknown_percent: unknown_pct,
            weak_percent: weak_pct,
            declining_percent: declining_pct,
            forgetting_percent: forgetting_pct,
            critical_percent: critical_pct,
            total_skills,
            mastered_skills: mastered,
            critical_blockers: critical,
        })
    }

    fn capture_topic_level_snapshot(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<GapSnapshotResult> {
        let total: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM student_topic_states sts
             INNER JOIN topics t ON t.id = sts.topic_id
             WHERE sts.student_id = ?1 AND t.subject_id = ?2",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        let mastered: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM student_topic_states sts
             INNER JOIN topics t ON t.id = sts.topic_id
             WHERE sts.student_id = ?1 AND t.subject_id = ?2 AND sts.mastery_score >= 8000",
            params![student_id, subject_id], |row| row.get(0),
        ).unwrap_or(0);

        let safe = total.max(1);
        let gap = 100 - (mastered * 100 / safe);

        self.conn.execute(
            "INSERT INTO gap_snapshots
                (student_id, subject_id, total_gap_percent, total_skills, mastered_skills)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![student_id, subject_id, gap, total, mastered],
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(GapSnapshotResult {
            total_gap_percent: gap,
            unknown_percent: 0, weak_percent: 0, declining_percent: 0,
            forgetting_percent: 0, critical_percent: 0,
            total_skills: total, mastered_skills: mastered, critical_blockers: 0,
        })
    }

    /// Get gap trend over time.
    pub fn list_gap_trend(
        &self,
        student_id: i64,
        subject_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<GapTrendPoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT total_gap_percent, snapshot_at
             FROM gap_snapshots
             WHERE student_id = ?1 AND subject_id = ?2
             ORDER BY snapshot_at DESC LIMIT ?3",
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt.query_map(params![student_id, subject_id, limit as i64], |row| {
            Ok(GapTrendPoint {
                gap_percent: row.get(0)?,
                snapshot_at: row.get(1)?,
            })
        }).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut points = Vec::new();
        for row in rows { points.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?); }
        Ok(points)
    }

    /// Post a knowledge update feed item.
    pub fn post_gap_feed_item(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_id: Option<i64>,
        event_type: &str,
        message: &str,
        severity: &str,
    ) -> EcoachResult<()> {
        self.conn.execute(
            "INSERT INTO knowledge_update_feed
                (student_id, subject_id, topic_id, event_type, message, severity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![student_id, subject_id, topic_id, event_type, message, severity],
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Get unread feed items.
    pub fn list_gap_feed(
        &self,
        student_id: i64,
        subject_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<GapFeedItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, topic_id, event_type, message, severity, created_at
             FROM knowledge_update_feed
             WHERE student_id = ?1 AND subject_id = ?2
             ORDER BY created_at DESC LIMIT ?3",
        ).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt.query_map(params![student_id, subject_id, limit as i64], |row| {
            Ok(GapFeedItem {
                id: row.get(0)?,
                topic_id: row.get(1)?,
                event_type: row.get(2)?,
                message: row.get(3)?,
                severity: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows { items.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?); }
        Ok(items)
    }
}

struct SolidificationOutcome {
    total_count: i64,
    answered_count: i64,
    skipped_count: i64,
    engaged_count: i64,
    correct_count: i64,
    accuracy_score: u16,
    coverage_score: u16,
    avg_response_time_ms: Option<i64>,
    outcome_label: String,
    next_action_hint: String,
    dominant_signal: String,
    interpretation_tags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, params};
    use serde_json::json;

    use super::*;

    #[test]
    fn complete_solidification_session_advances_plan_on_success() {
        let conn = test_conn();
        seed_gap_topic(&conn, 10, "Fractions");
        seed_active_plan(&conn, 1, 1, 10, 11, 100);
        seed_gap_session(&conn, 31, 21, 1, 10, Some(1), Some(11));
        seed_question(&conn, 501, 10, Some(100));
        seed_question(&conn, 502, 10, Some(100));
        conn.execute(
            "INSERT INTO session_items (session_id, question_id, display_order, status, is_correct, response_time_ms)
             VALUES (21, 501, 0, 'answered', 1, 12000),
                    (21, 502, 1, 'answered', 1, 10000)",
            [],
        )
        .expect("session items should seed");

        let service = KnowledgeGapService::new(&conn);
        service
            .complete_solidification_session(31)
            .expect("solidification should complete");

        let plan_status: String = conn
            .query_row(
                "SELECT status FROM gap_repair_plans WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .expect("plan status should query");
        assert_eq!(plan_status, "completed");

        let item_status: String = conn
            .query_row(
                "SELECT status FROM gap_repair_plan_items WHERE id = 11",
                [],
                |row| row.get(0),
            )
            .expect("item status should query");
        assert_eq!(item_status, "completed");

        let session_rollup: (i64, i64, i64) = conn
            .query_row(
                "SELECT answered_questions, correct_questions, accuracy_score
                 FROM sessions WHERE id = 21",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("session rollup should query");
        assert_eq!(session_rollup.0, 2);
        assert_eq!(session_rollup.1, 2);
        assert_eq!(session_rollup.2, 10_000);

        let interpreted_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM runtime_events
                 WHERE aggregate_kind = 'session'
                   AND aggregate_id = '21'
                   AND event_type = 'session.interpreted'",
                [],
                |row| row.get(0),
            )
            .expect("session interpretation count should query");
        assert_eq!(interpreted_count, 1);
    }

    #[test]
    fn complete_solidification_session_keeps_plan_active_on_failure() {
        let conn = test_conn();
        seed_gap_topic(&conn, 10, "Fractions");
        seed_active_plan(&conn, 1, 1, 10, 11, 100);
        seed_gap_session(&conn, 31, 21, 1, 10, Some(1), Some(11));
        seed_question(&conn, 501, 10, Some(100));
        seed_question(&conn, 502, 10, Some(100));
        conn.execute(
            "INSERT INTO session_items (session_id, question_id, display_order, status, is_correct, response_time_ms)
             VALUES (21, 501, 0, 'answered', 0, 18000),
                    (21, 502, 1, 'skipped', 0, NULL)",
            [],
        )
        .expect("session items should seed");

        let service = KnowledgeGapService::new(&conn);
        service
            .complete_solidification_session(31)
            .expect("solidification should complete");

        let plan_status: String = conn
            .query_row(
                "SELECT status FROM gap_repair_plans WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .expect("plan status should query");
        assert_eq!(plan_status, "active");

        let item_status: String = conn
            .query_row(
                "SELECT status FROM gap_repair_plan_items WHERE id = 11",
                [],
                |row| row.get(0),
            )
            .expect("item status should query");
        assert_eq!(item_status, "active");

        let topic_repair_signal: (i64, i64) = conn
            .query_row(
                "SELECT repair_priority, is_urgent
                 FROM student_topic_states
                 WHERE student_id = 1 AND topic_id = 10",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("topic repair signal should query");
        assert!(topic_repair_signal.0 > 7000);
        assert_eq!(topic_repair_signal.1, 1);
    }

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        conn.execute_batch(
            "
            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER,
                name TEXT NOT NULL
            );
            CREATE TABLE gap_repair_plans (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                status TEXT NOT NULL,
                priority_score INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE gap_repair_plan_items (
                id INTEGER PRIMARY KEY,
                plan_id INTEGER NOT NULL,
                node_id INTEGER,
                sequence_order INTEGER NOT NULL DEFAULT 0,
                repair_action TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE solidification_sessions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                repair_plan_id INTEGER,
                session_id INTEGER,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT
            );
            CREATE TABLE sessions (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                session_type TEXT NOT NULL,
                status TEXT NOT NULL,
                answered_questions INTEGER NOT NULL DEFAULT 0,
                correct_questions INTEGER NOT NULL DEFAULT 0,
                accuracy_score INTEGER,
                avg_response_time_ms INTEGER,
                completed_at TEXT,
                updated_at TEXT
            );
            CREATE TABLE session_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                question_id INTEGER NOT NULL,
                display_order INTEGER NOT NULL,
                status TEXT NOT NULL,
                is_correct INTEGER,
                response_time_ms INTEGER
            );
            CREATE TABLE runtime_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL UNIQUE,
                event_type TEXT NOT NULL,
                aggregate_kind TEXT NOT NULL,
                aggregate_id TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                occurred_at TEXT NOT NULL
            );
            CREATE TABLE student_topic_states (
                id INTEGER PRIMARY KEY,
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                mastery_score INTEGER NOT NULL DEFAULT 0,
                gap_score INTEGER NOT NULL DEFAULT 0,
                repair_priority INTEGER NOT NULL DEFAULT 0,
                is_urgent INTEGER NOT NULL DEFAULT 0,
                last_decline_at TEXT,
                updated_at TEXT
            );
            CREATE TABLE academic_nodes (
                id INTEGER PRIMARY KEY,
                canonical_title TEXT,
                node_type TEXT
            );
            CREATE TABLE student_error_profiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                knowledge_gap_score INTEGER NOT NULL DEFAULT 0,
                conceptual_confusion_score INTEGER NOT NULL DEFAULT 0,
                recognition_failure_score INTEGER NOT NULL DEFAULT 0,
                execution_error_score INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE questions (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER NOT NULL,
                family_id INTEGER,
                difficulty_level INTEGER NOT NULL DEFAULT 5000,
                primary_skill_id INTEGER,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE question_skill_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                question_id INTEGER NOT NULL,
                node_id INTEGER NOT NULL,
                is_primary INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE misconception_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                node_id INTEGER,
                topic_id INTEGER,
                title TEXT NOT NULL,
                severity INTEGER NOT NULL DEFAULT 5000,
                is_active INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE knowledge_entries (
                id INTEGER PRIMARY KEY,
                topic_id INTEGER,
                title TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                importance_score INTEGER NOT NULL DEFAULT 5000,
                difficulty_level INTEGER NOT NULL DEFAULT 5000
            );
            CREATE TABLE question_glossary_links (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                question_id INTEGER NOT NULL,
                entry_id INTEGER NOT NULL,
                confidence_score INTEGER NOT NULL DEFAULT 5000,
                is_primary INTEGER NOT NULL DEFAULT 0
            );
            ",
        )
        .expect("schema should seed");
        conn
    }

    fn seed_gap_topic(conn: &Connection, topic_id: i64, topic_name: &str) {
        conn.execute(
            "INSERT INTO topics (id, subject_id, name) VALUES (?1, 1, ?2)",
            params![topic_id, topic_name],
        )
        .expect("topic should seed");
        conn.execute(
            "INSERT INTO student_topic_states (
                id, student_id, topic_id, mastery_score, gap_score, repair_priority, is_urgent, updated_at
             ) VALUES (1, 1, ?1, 3000, 7000, 7000, 1, datetime('now'))",
            [topic_id],
        )
        .expect("topic state should seed");
    }

    fn seed_active_plan(
        conn: &Connection,
        plan_id: i64,
        student_id: i64,
        topic_id: i64,
        item_id: i64,
        node_id: i64,
    ) {
        conn.execute(
            "INSERT INTO academic_nodes (id, canonical_title, node_type)
             VALUES (?1, 'Equivalent fractions', 'concept')",
            [node_id],
        )
        .expect("node should seed");
        conn.execute(
            "INSERT INTO gap_repair_plans (id, student_id, topic_id, status, priority_score)
             VALUES (?1, ?2, ?3, 'active', 7000)",
            params![plan_id, student_id, topic_id],
        )
        .expect("plan should seed");
        conn.execute(
            "INSERT INTO gap_repair_plan_items (id, plan_id, node_id, sequence_order, repair_action, status)
             VALUES (?1, ?2, ?3, 0, 'teach_concept', 'active')",
            params![item_id, plan_id, node_id],
        )
        .expect("plan item should seed");
    }

    fn seed_gap_session(
        conn: &Connection,
        solidification_id: i64,
        session_id: i64,
        student_id: i64,
        topic_id: i64,
        repair_plan_id: Option<i64>,
        repair_item_id: Option<i64>,
    ) {
        conn.execute(
            "INSERT INTO sessions (id, student_id, session_type, status, updated_at)
             VALUES (?1, ?2, 'gap_repair', 'active', datetime('now'))",
            params![session_id, student_id],
        )
        .expect("session should seed");
        conn.execute(
            "INSERT INTO solidification_sessions (id, student_id, topic_id, repair_plan_id, session_id, status)
             VALUES (?1, ?2, ?3, ?4, ?5, 'active')",
            params![solidification_id, student_id, topic_id, repair_plan_id, session_id],
        )
        .expect("solidification should seed");
        conn.execute(
            "INSERT INTO runtime_events (
                event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
             ) VALUES (
                'solid-start-1',
                'gap.solidification_started',
                'gap',
                ?1,
                'trace-gap-1',
                ?2,
                datetime('now')
             )",
            params![
                solidification_id.to_string(),
                json!({ "repair_item_id": repair_item_id }).to_string(),
            ],
        )
        .expect("solidification start event should seed");
    }

    fn seed_question(conn: &Connection, question_id: i64, topic_id: i64, node_id: Option<i64>) {
        conn.execute(
            "INSERT INTO questions (id, topic_id, primary_skill_id, is_active)
             VALUES (?1, ?2, ?3, 1)",
            params![question_id, topic_id, node_id],
        )
        .expect("question should seed");
        if let Some(node_id) = node_id {
            conn.execute(
                "INSERT INTO question_skill_links (question_id, node_id, is_primary)
                 VALUES (?1, ?2, 1)",
                params![question_id, node_id],
            )
            .expect("question skill link should seed");
        }
    }
}
