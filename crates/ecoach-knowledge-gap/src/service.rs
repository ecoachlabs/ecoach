use chrono::Utc;
use ecoach_substrate::{DomainEvent, EcoachError, EcoachResult, clamp_bp, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    CreateGapRepairPlanInput, GapDashboard, GapRepairFocus, GapRepairPlan, GapRepairPlanItem,
    GapScoreCard, RepairItemStatus, SolidificationProgress, SolidificationSession,
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
        let dominant_action = if gap_card.knowledge_gap_score >= gap_card.conceptual_confusion_score
            && gap_card.knowledge_gap_score >= gap_card.recognition_failure_score
        {
            "teach_concept"
        } else if gap_card.conceptual_confusion_score >= gap_card.recognition_failure_score {
            "clarify_confusion"
        } else {
            "drill_recognition"
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

        // If item completed, activate the next pending item
        if matches!(new_status, RepairItemStatus::Completed) {
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

        // Create underlying session
        let topic_ids_json = format!("[{}]", topic_id);
        self.conn
            .execute(
                "INSERT INTO sessions (
                    student_id, session_type, topic_ids, difficulty_preference, status,
                    created_at, updated_at
                 ) VALUES (?1, 'gap_repair', ?2, 'adaptive', 'active', ?3, ?3)",
                params![student_id, topic_ids_json, now],
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

        self.append_event(DomainEvent::new(
            "gap.solidification_started",
            solid_id.to_string(),
            json!({
                "student_id": student_id,
                "topic_id": topic_id,
                "repair_plan_id": repair_plan_id,
            }),
        ))?;

        self.get_solidification_session(solid_id)
    }

    pub fn complete_solidification_session(
        &self,
        solidification_id: i64,
    ) -> EcoachResult<SolidificationSession> {
        let now = Utc::now().to_rfc3339();
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

        // Also complete the underlying session
        let session_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT session_id FROM solidification_sessions WHERE id = ?1",
                [solidification_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .flatten();

        if let Some(sid) = session_id {
            self.conn
                .execute(
                    "UPDATE sessions SET status = 'completed', completed_at = ?1, updated_at = ?1 WHERE id = ?2",
                    params![now, sid],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }

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
}
