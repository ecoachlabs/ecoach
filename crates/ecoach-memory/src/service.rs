use chrono::{Duration, Utc};
use ecoach_substrate::{DomainEvent, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{
    DecayBatchResult, InterferenceEdge, MemoryDashboard, MemoryReviewQueueItem, MemoryState,
    MemoryStateRecord, RecallMode, RecheckItem, RecordMemoryEvidenceInput, TopicMemorySummary,
};

// ── Decay model constants ──
// Ebbinghaus-inspired with modifications for learning context

const BASE_DECAY_RATE_BP_PER_DAY: i64 = 350;
const INTERFERENCE_DECAY_MULTIPLIER: f64 = 1.6;
const COLLAPSE_THRESHOLD: i64 = 500;
const ACCESSIBLE_THRESHOLD: i64 = 4000;
const ANCHORING_THRESHOLD: i64 = 6500;
const CONFIRMED_THRESHOLD: i64 = 8000;
const LOCKED_IN_THRESHOLD: i64 = 9200;
const INITIAL_REVIEW_HOURS: i64 = 24;
const REVIEW_INTERVAL_MULTIPLIER: f64 = 2.0;
const MAX_REVIEW_INTERVAL_DAYS: i64 = 60;
const CORRECT_RECALL_GAIN: i64 = 800;
const FAILED_RECALL_LOSS: i64 = 1200;
const FREE_RECALL_BONUS: i64 = 400;

pub struct MemoryService<'a> {
    conn: &'a Connection,
}

impl<'a> MemoryService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    // ── Record evidence and update memory state ──

    pub fn record_evidence(
        &self,
        input: &RecordMemoryEvidenceInput,
    ) -> EcoachResult<MemoryStateRecord> {
        let now = Utc::now().to_rfc3339();

        // Insert evidence event
        self.conn
            .execute(
                "INSERT INTO memory_evidence_events (
                    student_id, node_id, topic_id, recall_mode, cue_level, delay_bucket,
                    interference_detected, was_correct, confidence_level, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    input.student_id,
                    input.node_id,
                    input.topic_id,
                    input.recall_mode.as_str(),
                    input.cue_level.as_str(),
                    input.delay_bucket,
                    if input.interference_detected { 1 } else { 0 },
                    if input.was_correct { 1 } else { 0 },
                    input.confidence_level,
                    now,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Get or create the memory state record
        let existing =
            self.get_memory_state_by_node(input.student_id, input.topic_id, input.node_id)?;

        match existing {
            Some(record) => self.update_memory_from_evidence(record, input),
            None => self.create_initial_memory_state(input),
        }
    }

    fn create_initial_memory_state(
        &self,
        input: &RecordMemoryEvidenceInput,
    ) -> EcoachResult<MemoryStateRecord> {
        let now = Utc::now();
        let initial_state = if input.was_correct {
            MemoryState::Encoded
        } else {
            MemoryState::Seen
        };
        let initial_strength: u16 = if input.was_correct { 3000 } else { 1000 };
        let initial_fluency: u16 = if input.was_correct { 2000 } else { 500 };
        let review_due = now + Duration::hours(INITIAL_REVIEW_HOURS);

        self.conn
            .execute(
                "INSERT INTO memory_states (
                    student_id, topic_id, node_id, memory_state, memory_strength,
                    recall_fluency, decay_risk, review_due_at, last_recalled_at,
                    created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
                params![
                    input.student_id,
                    input.topic_id,
                    input.node_id,
                    initial_state.as_str(),
                    initial_strength,
                    initial_fluency,
                    0i64,
                    review_due.to_rfc3339(),
                    now.to_rfc3339(),
                    now.to_rfc3339(),
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let record_id = self.conn.last_insert_rowid();

        // Schedule first review
        self.schedule_recheck(
            input.student_id,
            input.node_id,
            &review_due.to_rfc3339(),
            "spaced_review",
        )?;

        self.append_event(DomainEvent::new(
            "memory.state_created",
            record_id.to_string(),
            json!({
                "student_id": input.student_id,
                "state": initial_state.as_str(),
                "strength": initial_strength,
            }),
        ))?;

        self.get_memory_state(record_id)
    }

    fn update_memory_from_evidence(
        &self,
        record: MemoryStateRecord,
        input: &RecordMemoryEvidenceInput,
    ) -> EcoachResult<MemoryStateRecord> {
        let now = Utc::now();
        let current_state =
            MemoryState::from_str(&record.memory_state).unwrap_or(MemoryState::Seen);

        let str_i64 = record.memory_strength as i64;
        let base_delta: i64 = if input.was_correct {
            CORRECT_RECALL_GAIN
        } else {
            -FAILED_RECALL_LOSS
        };
        let recall_bonus: i64 =
            if input.was_correct && matches!(input.recall_mode, RecallMode::FreeRecall) {
                FREE_RECALL_BONUS
            } else {
                0
            };
        let interference_penalty: i64 = if input.interference_detected && !input.was_correct {
            -400
        } else {
            0
        };
        let new_strength = clamp_bp(str_i64 + base_delta + recall_bonus + interference_penalty);

        let fluency_signal: f64 = if input.was_correct { 10000.0 } else { 0.0 };
        let new_fluency =
            clamp_bp((record.recall_fluency as f64 * 0.7 + fluency_signal * 0.3).round() as i64);

        // Determine next state
        let next_state = self.resolve_next_memory_state(current_state, new_strength, input);

        // Compute next review interval (expanding on success, contracting on failure)
        let interval_hours = if input.was_correct {
            let base = INITIAL_REVIEW_HOURS as f64 * REVIEW_INTERVAL_MULTIPLIER;
            let factor = (new_strength as f64 / 5000.0).min(4.0);
            (base * factor).round() as i64
        } else {
            INITIAL_REVIEW_HOURS
        };
        let capped_hours = interval_hours.min(MAX_REVIEW_INTERVAL_DAYS * 24);
        let next_review = now + Duration::hours(capped_hours);

        // Compute decay risk
        let decay_risk =
            self.compute_decay_risk(new_strength, &next_state, input.interference_detected);

        self.conn
            .execute(
                "UPDATE memory_states
                 SET memory_state = ?1, memory_strength = ?2, recall_fluency = ?3,
                     decay_risk = ?4, review_due_at = ?5, last_recalled_at = ?6, updated_at = ?6
                 WHERE id = ?7",
                params![
                    next_state.as_str(),
                    new_strength,
                    new_fluency,
                    decay_risk,
                    next_review.to_rfc3339(),
                    now.to_rfc3339(),
                    record.id,
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        // Schedule the next review
        self.schedule_recheck(
            record.student_id,
            record.node_id,
            &next_review.to_rfc3339(),
            "spaced_review",
        )?;

        self.append_event(DomainEvent::new(
            "memory.state_updated",
            record.id.to_string(),
            json!({
                "previous_state": current_state.as_str(),
                "next_state": next_state.as_str(),
                "previous_strength": record.memory_strength,
                "next_strength": new_strength,
                "was_correct": input.was_correct,
            }),
        ))?;

        self.get_memory_state(record.id)
    }

    fn resolve_next_memory_state(
        &self,
        current: MemoryState,
        strength_bp: u16,
        input: &RecordMemoryEvidenceInput,
    ) -> MemoryState {
        let strength = strength_bp as i64;
        if strength < COLLAPSE_THRESHOLD {
            return MemoryState::Collapsed;
        }

        // On correct recall, promote through the state machine
        if input.was_correct {
            match current {
                MemoryState::Seen | MemoryState::Collapsed => MemoryState::Encoded,
                MemoryState::Encoded if strength >= ACCESSIBLE_THRESHOLD => MemoryState::Accessible,
                MemoryState::Encoded => MemoryState::Encoded,
                MemoryState::Accessible if strength >= ANCHORING_THRESHOLD => {
                    MemoryState::Anchoring
                }
                MemoryState::Accessible => MemoryState::Accessible,
                MemoryState::Fragile | MemoryState::Fading | MemoryState::AtRisk => {
                    MemoryState::Rebuilding
                }
                MemoryState::Rebuilding if strength >= ACCESSIBLE_THRESHOLD => {
                    MemoryState::Recovered
                }
                MemoryState::Rebuilding => MemoryState::Rebuilding,
                MemoryState::Recovered if strength >= ANCHORING_THRESHOLD => MemoryState::Anchoring,
                MemoryState::Recovered => MemoryState::Recovered,
                MemoryState::Anchoring if strength >= CONFIRMED_THRESHOLD => MemoryState::Confirmed,
                MemoryState::Anchoring => MemoryState::Anchoring,
                MemoryState::Confirmed if strength >= LOCKED_IN_THRESHOLD => MemoryState::LockedIn,
                MemoryState::Confirmed => MemoryState::Confirmed,
                MemoryState::LockedIn => MemoryState::LockedIn,
            }
        } else {
            // On failed recall, demote
            match current {
                MemoryState::LockedIn | MemoryState::Confirmed => MemoryState::AtRisk,
                MemoryState::Anchoring | MemoryState::Recovered => MemoryState::Fragile,
                MemoryState::Accessible => MemoryState::Fragile,
                MemoryState::AtRisk => MemoryState::Fading,
                MemoryState::Fragile => MemoryState::Fading,
                MemoryState::Fading => {
                    if strength < COLLAPSE_THRESHOLD {
                        MemoryState::Collapsed
                    } else {
                        MemoryState::Fading
                    }
                }
                other => other,
            }
        }
    }

    fn compute_decay_risk(&self, strength_bp: u16, state: &MemoryState, interference: bool) -> u16 {
        let strength = strength_bp as i64;
        let base_risk = match state {
            MemoryState::LockedIn => 500,
            MemoryState::Confirmed => 1000,
            MemoryState::Anchoring => 2000,
            MemoryState::Accessible | MemoryState::Recovered => 3000,
            MemoryState::Fragile | MemoryState::Rebuilding => 5500,
            MemoryState::AtRisk => 7000,
            MemoryState::Fading => 8500,
            MemoryState::Encoded => 4000,
            MemoryState::Seen => 6000,
            MemoryState::Collapsed => 10000,
        };

        let strength_factor = ((10000 - strength) as f64 / 10000.0 * 2000.0).round() as i64;
        let interference_add = if interference { 1000 } else { 0 };
        clamp_bp(base_risk + strength_factor + interference_add)
    }

    // ── Decay processing (batch job) ──

    pub fn process_decay_batch(&self, limit: usize) -> EcoachResult<DecayBatchResult> {
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        // Find overdue memory items
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, student_id, topic_id, node_id, memory_state, memory_strength,
                        recall_fluency, decay_risk, review_due_at
                 FROM memory_states
                 WHERE review_due_at IS NOT NULL AND review_due_at < ?1
                   AND memory_state NOT IN ('collapsed', 'locked_in')
                 ORDER BY review_due_at ASC
                 LIMIT ?2",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        struct OverdueItem {
            id: i64,
            student_id: i64,
            node_id: Option<i64>,
            memory_state: String,
            memory_strength: i64,
            review_due_at: String,
        }

        let items: Vec<OverdueItem> = stmt
            .query_map(params![now_str, limit as i64], |row| {
                Ok(OverdueItem {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    node_id: row.get(3)?,
                    memory_state: row.get(4)?,
                    memory_strength: row.get(5)?,
                    review_due_at: row.get(8)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        let mut result = DecayBatchResult {
            items_processed: items.len(),
            items_decayed: 0,
            items_collapsed: 0,
            new_rechecks_scheduled: 0,
        };

        for item in &items {
            // Calculate overdue days
            let overdue_days =
                if let Ok(due) = chrono::DateTime::parse_from_rfc3339(&item.review_due_at) {
                    let diff = now.signed_duration_since(due.with_timezone(&Utc));
                    diff.num_days().max(1)
                } else {
                    1
                };

            // Check interference from neighbouring nodes
            let interference = if let Some(node_id) = item.node_id {
                self.has_active_interference(item.student_id, node_id)?
            } else {
                false
            };

            let decay_multiplier = if interference {
                INTERFERENCE_DECAY_MULTIPLIER
            } else {
                1.0
            };

            let decay_amount = (BASE_DECAY_RATE_BP_PER_DAY as f64
                * overdue_days as f64
                * decay_multiplier)
                .round() as i64;

            let new_strength = (item.memory_strength - decay_amount).max(0);

            let current_state =
                MemoryState::from_str(&item.memory_state).unwrap_or(MemoryState::Seen);

            let next_state = if new_strength < COLLAPSE_THRESHOLD {
                result.items_collapsed += 1;
                MemoryState::Collapsed
            } else {
                result.items_decayed += 1;
                match current_state {
                    MemoryState::Accessible | MemoryState::Anchoring | MemoryState::Recovered => {
                        MemoryState::Fragile
                    }
                    MemoryState::Confirmed => MemoryState::AtRisk,
                    MemoryState::Fragile | MemoryState::AtRisk => MemoryState::Fading,
                    other => other,
                }
            };

            let decay_risk =
                self.compute_decay_risk(clamp_bp(new_strength), &next_state, interference);

            // Re-schedule review sooner for decayed items
            let next_review = now + Duration::hours(INITIAL_REVIEW_HOURS);

            self.conn
                .execute(
                    "UPDATE memory_states
                     SET memory_state = ?1, memory_strength = ?2, decay_risk = ?3,
                         review_due_at = ?4, updated_at = ?5
                     WHERE id = ?6",
                    params![
                        next_state.as_str(),
                        new_strength,
                        decay_risk,
                        next_review.to_rfc3339(),
                        now_str,
                        item.id,
                    ],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;

            // Mark old recheck as missed, schedule new one
            self.conn
                .execute(
                    "UPDATE recheck_schedules SET status = 'missed' WHERE student_id = ?1 AND node_id IS ?2 AND status = 'pending'",
                    params![item.student_id, item.node_id],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;

            self.schedule_recheck(
                item.student_id,
                item.node_id,
                &next_review.to_rfc3339(),
                "decay_recovery",
            )?;
            result.new_rechecks_scheduled += 1;

            self.append_event(DomainEvent::new(
                "memory.decayed",
                item.id.to_string(),
                json!({
                    "student_id": item.student_id,
                    "node_id": item.node_id,
                    "overdue_days": overdue_days,
                    "interference": interference,
                    "previous_state": current_state.as_str(),
                    "next_state": next_state.as_str(),
                    "next_review_at": next_review.to_rfc3339(),
                    "new_strength": clamp_bp(new_strength),
                }),
            ))?;
        }

        Ok(result)
    }

    // ── Interference detection ──

    fn has_active_interference(&self, student_id: i64, node_id: i64) -> EcoachResult<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM interference_edges ie
                 INNER JOIN memory_states ms
                     ON ms.student_id = ?1
                    AND (
                         (ie.from_node_id = ?2 AND ms.node_id = ie.to_node_id)
                         OR
                         (ie.to_node_id = ?2 AND ms.node_id = ie.from_node_id)
                    )
                 WHERE (ie.from_node_id = ?2 OR ie.to_node_id = ?2)
                   AND ie.strength_score > 3000
                   AND ms.memory_state IN ('fragile', 'fading', 'at_risk', 'collapsed')",
                params![student_id, node_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(count > 0)
    }

    pub fn record_interference(
        &self,
        from_node_id: i64,
        to_node_id: i64,
        strength: u16,
    ) -> EcoachResult<InterferenceEdge> {
        let now = Utc::now().to_rfc3339();
        let existing_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id
                 FROM interference_edges
                 WHERE from_node_id = ?1 AND to_node_id = ?2",
                params![from_node_id, to_node_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let edge_id = if let Some(edge_id) = existing_id {
            self.conn
                .execute(
                    "UPDATE interference_edges
                     SET strength_score = ?1, last_seen_at = ?2
                     WHERE id = ?3",
                    params![strength, now, edge_id],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            edge_id
        } else {
            self.conn
                .execute(
                    "INSERT INTO interference_edges (from_node_id, to_node_id, strength_score, last_seen_at, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![from_node_id, to_node_id, strength, now],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
            self.conn.last_insert_rowid()
        };

        self.append_event(DomainEvent::new(
            "memory.interference_recorded",
            edge_id.to_string(),
            json!({
                "from_node_id": from_node_id,
                "to_node_id": to_node_id,
                "strength_score": strength,
            }),
        ))?;

        Ok(InterferenceEdge {
            id: edge_id,
            from_node_id,
            to_node_id,
            strength_score: strength,
            last_seen_at: Some(now),
        })
    }

    pub fn get_interference_edges(&self, node_id: i64) -> EcoachResult<Vec<InterferenceEdge>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, from_node_id, to_node_id, strength_score, last_seen_at
                 FROM interference_edges
                 WHERE from_node_id = ?1 OR to_node_id = ?1
                 ORDER BY strength_score DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([node_id], |row| {
                Ok(InterferenceEdge {
                    id: row.get(0)?,
                    from_node_id: row.get(1)?,
                    to_node_id: row.get(2)?,
                    strength_score: row.get(3)?,
                    last_seen_at: row.get(4)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut edges = Vec::new();
        for row in rows {
            edges.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(edges)
    }

    // ── Review scheduling ──

    pub fn get_due_rechecks(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<RecheckItem>> {
        let now = Utc::now().to_rfc3339();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT rs.id, rs.student_id, rs.node_id, rs.due_at, rs.schedule_type, rs.status,
                        ms.memory_state, ms.memory_strength, ms.decay_risk,
                        t.name AS topic_name, an.canonical_title AS node_title
                 FROM recheck_schedules rs
                 LEFT JOIN memory_states ms
                     ON ms.student_id = rs.student_id AND ms.node_id IS rs.node_id
                 LEFT JOIN academic_nodes an ON an.id = rs.node_id
                 LEFT JOIN topics t ON t.id = ms.topic_id
                 WHERE rs.student_id = ?1 AND rs.status = 'pending' AND rs.due_at <= ?2
                 ORDER BY rs.due_at ASC
                 LIMIT ?3",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, now, limit as i64], |row| {
                Ok(RecheckItem {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    node_id: row.get(2)?,
                    due_at: row.get(3)?,
                    schedule_type: row.get(4)?,
                    status: row.get(5)?,
                    memory_state: row.get(6)?,
                    memory_strength: row.get(7)?,
                    decay_risk: row.get(8)?,
                    topic_name: row.get(9)?,
                    node_title: row.get(10)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(items)
    }

    pub fn complete_recheck(&self, recheck_id: i64) -> EcoachResult<()> {
        let recheck: Option<(i64, Option<i64>, String, String)> = self
            .conn
            .query_row(
                "SELECT student_id, node_id, schedule_type, due_at
                 FROM recheck_schedules
                 WHERE id = ?1",
                [recheck_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let Some((student_id, node_id, schedule_type, due_at)) = recheck else {
            return Err(EcoachError::NotFound(
                "recheck not found or already completed".to_string(),
            ));
        };

        let affected = self
            .conn
            .execute(
                "UPDATE recheck_schedules SET status = 'completed', completed_at = datetime('now') WHERE id = ?1 AND status = 'pending'",
                [recheck_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        if affected == 0 {
            return Err(EcoachError::NotFound(
                "recheck not found or already completed".to_string(),
            ));
        }

        self.append_event(DomainEvent::new(
            "memory.recheck_completed",
            recheck_id.to_string(),
            json!({
                "student_id": student_id,
                "node_id": node_id,
                "schedule_type": schedule_type,
                "due_at": due_at,
            }),
        ))?;
        Ok(())
    }

    fn schedule_recheck(
        &self,
        student_id: i64,
        node_id: Option<i64>,
        due_at: &str,
        schedule_type: &str,
    ) -> EcoachResult<()> {
        // Cancel any existing pending rechecks for this node
        self.conn
            .execute(
                "UPDATE recheck_schedules SET status = 'cancelled'
                 WHERE student_id = ?1 AND node_id IS ?2 AND status = 'pending'",
                params![student_id, node_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO recheck_schedules (student_id, node_id, due_at, schedule_type, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, 'pending', datetime('now'))",
                params![student_id, node_id, due_at, schedule_type],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    // ── Queries ──

    pub fn get_memory_state(&self, id: i64) -> EcoachResult<MemoryStateRecord> {
        self.conn
            .query_row(
                "SELECT id, student_id, topic_id, node_id, memory_state, memory_strength,
                        recall_fluency, decay_risk, review_due_at, last_recalled_at,
                        created_at, updated_at
                 FROM memory_states WHERE id = ?1",
                [id],
                |row| {
                    Ok(MemoryStateRecord {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        node_id: row.get(3)?,
                        memory_state: row.get(4)?,
                        memory_strength: row.get(5)?,
                        recall_fluency: row.get(6)?,
                        decay_risk: row.get(7)?,
                        review_due_at: row.get(8)?,
                        last_recalled_at: row.get(9)?,
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .map_err(|e| EcoachError::NotFound(format!("memory state not found: {}", e)))
    }

    fn get_memory_state_by_node(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        node_id: Option<i64>,
    ) -> EcoachResult<Option<MemoryStateRecord>> {
        self.conn
            .query_row(
                "SELECT id, student_id, topic_id, node_id, memory_state, memory_strength,
                        recall_fluency, decay_risk, review_due_at, last_recalled_at,
                        created_at, updated_at
                 FROM memory_states
                 WHERE student_id = ?1 AND topic_id IS ?2 AND node_id IS ?3",
                params![student_id, topic_id, node_id],
                |row| {
                    Ok(MemoryStateRecord {
                        id: row.get(0)?,
                        student_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        node_id: row.get(3)?,
                        memory_state: row.get(4)?,
                        memory_strength: row.get(5)?,
                        recall_fluency: row.get(6)?,
                        decay_risk: row.get(7)?,
                        review_due_at: row.get(8)?,
                        last_recalled_at: row.get(9)?,
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))
    }

    pub fn list_memory_states_for_student(
        &self,
        student_id: i64,
        filter_state: Option<&str>,
        limit: usize,
    ) -> EcoachResult<Vec<MemoryStateRecord>> {
        let sql = if filter_state.is_some() {
            "SELECT id, student_id, topic_id, node_id, memory_state, memory_strength,
                    recall_fluency, decay_risk, review_due_at, last_recalled_at,
                    created_at, updated_at
             FROM memory_states
             WHERE student_id = ?1 AND memory_state = ?2
             ORDER BY decay_risk DESC
             LIMIT ?3"
        } else {
            "SELECT id, student_id, topic_id, node_id, memory_state, memory_strength,
                    recall_fluency, decay_risk, review_due_at, last_recalled_at,
                    created_at, updated_at
             FROM memory_states
             WHERE student_id = ?1 AND (?2 IS NULL OR memory_state = ?2)
             ORDER BY decay_risk DESC
             LIMIT ?3"
        };

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, filter_state, limit as i64], |row| {
                Ok(MemoryStateRecord {
                    id: row.get(0)?,
                    student_id: row.get(1)?,
                    topic_id: row.get(2)?,
                    node_id: row.get(3)?,
                    memory_state: row.get(4)?,
                    memory_strength: row.get(5)?,
                    recall_fluency: row.get(6)?,
                    decay_risk: row.get(7)?,
                    review_due_at: row.get(8)?,
                    last_recalled_at: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(items)
    }

    // ── Dashboard ──

    pub fn build_review_queue(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<MemoryReviewQueueItem>> {
        let now = Utc::now().to_rfc3339();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    ms.id,
                    ms.student_id,
                    ms.topic_id,
                    t.name,
                    ms.node_id,
                    an.canonical_title,
                    ms.memory_state,
                    ms.memory_strength,
                    ms.decay_risk,
                    ms.review_due_at,
                    rs.schedule_type,
                    COALESCE((
                        SELECT COUNT(*)
                        FROM interference_edges ie
                        WHERE (ie.from_node_id = ms.node_id OR ie.to_node_id = ms.node_id)
                          AND ie.strength_score >= 3000
                    ), 0) AS interference_count
                 FROM memory_states ms
                 LEFT JOIN topics t ON t.id = ms.topic_id
                 LEFT JOIN academic_nodes an ON an.id = ms.node_id
                 LEFT JOIN recheck_schedules rs
                     ON rs.student_id = ms.student_id
                    AND rs.node_id IS ms.node_id
                    AND rs.status = 'pending'
                 WHERE ms.student_id = ?1
                 ORDER BY
                    CASE
                        WHEN ms.review_due_at IS NOT NULL AND ms.review_due_at <= ?2 THEN 0
                        WHEN ms.memory_state IN ('collapsed', 'fading', 'at_risk', 'fragile', 'rebuilding') THEN 1
                        ELSE 2
                    END,
                    ms.decay_risk DESC,
                    ms.memory_strength ASC,
                    ms.id DESC
                 LIMIT ?3",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, now, limit as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, u16>(7)?,
                    row.get::<_, u16>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, i64>(11)?,
                ))
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            let (
                memory_state_id,
                student_id,
                topic_id,
                topic_name,
                node_id,
                node_title,
                memory_state,
                memory_strength,
                decay_risk,
                due_at,
                schedule_type,
                interference_count,
            ) = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
            let is_due = due_at.as_ref().is_some_and(|due| due <= &now);
            let priority_score = self.review_priority_score(
                &memory_state,
                memory_strength,
                decay_risk,
                is_due,
                interference_count,
            );

            items.push(MemoryReviewQueueItem {
                memory_state_id,
                student_id,
                topic_id,
                topic_name,
                node_id,
                node_title,
                action_type: self.review_action_for_state(
                    &memory_state,
                    is_due,
                    interference_count,
                ),
                schedule_type: schedule_type
                    .unwrap_or_else(|| self.schedule_type_for_state(&memory_state)),
                memory_state,
                priority_score,
                memory_strength,
                decay_risk,
                due_at,
                interference_count,
            });
        }
        Ok(items)
    }

    pub fn list_topic_summaries(
        &self,
        student_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<TopicMemorySummary>> {
        let now = Utc::now().to_rfc3339();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                    ms.topic_id,
                    t.name,
                    COUNT(*),
                    SUM(CASE WHEN ms.memory_state IN ('accessible', 'anchoring', 'confirmed', 'locked_in', 'recovered') THEN 1 ELSE 0 END),
                    SUM(CASE WHEN ms.memory_state IN ('fragile', 'at_risk', 'fading', 'rebuilding') THEN 1 ELSE 0 END),
                    SUM(CASE WHEN ms.memory_state = 'collapsed' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN ms.review_due_at IS NOT NULL AND ms.review_due_at <= ?2 THEN 1 ELSE 0 END),
                    CAST(COALESCE(AVG(ms.memory_strength), 0) AS INTEGER),
                    MIN(ms.review_due_at)
                 FROM memory_states ms
                 INNER JOIN topics t ON t.id = ms.topic_id
                 WHERE ms.student_id = ?1
                   AND ms.topic_id IS NOT NULL
                 GROUP BY ms.topic_id, t.name
                 ORDER BY
                    SUM(CASE WHEN ms.review_due_at IS NOT NULL AND ms.review_due_at <= ?2 THEN 1 ELSE 0 END) DESC,
                    SUM(CASE WHEN ms.memory_state IN ('fragile', 'at_risk', 'fading', 'rebuilding') THEN 1 ELSE 0 END) DESC,
                    CAST(COALESCE(AVG(ms.memory_strength), 0) AS INTEGER) ASC,
                    t.name ASC
                 LIMIT ?3",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, now, limit as i64], |row| {
                Ok(TopicMemorySummary {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    total_items: row.get(2)?,
                    healthy_items: row.get(3)?,
                    fragile_items: row.get(4)?,
                    collapsed_items: row.get(5)?,
                    overdue_reviews: row.get(6)?,
                    average_strength: row.get::<_, i64>(7)?.clamp(0, 10_000) as u16,
                    next_review_due: row.get(8)?,
                    recommended_action: String::new(),
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut summaries = Vec::new();
        for row in rows {
            let mut summary = row.map_err(|e| EcoachError::Storage(e.to_string()))?;
            summary.recommended_action = self.topic_action_for_summary(&summary).to_string();
            summaries.push(summary);
        }
        Ok(summaries)
    }

    pub fn get_memory_dashboard(&self, student_id: i64) -> EcoachResult<MemoryDashboard> {
        let now = Utc::now().to_rfc3339();

        let stats: (i64, i64, i64, i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT
                    COUNT(*),
                    SUM(CASE WHEN memory_state IN ('accessible','anchoring','confirmed','locked_in','recovered') THEN 1 ELSE 0 END),
                    SUM(CASE WHEN memory_state = 'at_risk' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN memory_state = 'fading' THEN 1 ELSE 0 END),
                    SUM(CASE WHEN memory_state = 'collapsed' THEN 1 ELSE 0 END),
                    CAST(COALESCE(AVG(memory_strength), 0) AS INTEGER),
                    SUM(CASE WHEN review_due_at IS NOT NULL AND review_due_at < ?2 AND memory_state != 'collapsed' THEN 1 ELSE 0 END)
                 FROM memory_states WHERE student_id = ?1",
                params![student_id, now],
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
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let next_review_due: Option<String> = self
            .conn
            .query_row(
                "SELECT MIN(review_due_at) FROM memory_states
                 WHERE student_id = ?1 AND memory_state != 'collapsed' AND review_due_at IS NOT NULL",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| EcoachError::Storage(e.to_string()))?
            .flatten();

        Ok(MemoryDashboard {
            student_id,
            total_items: stats.0,
            healthy_count: stats.1,
            at_risk_count: stats.2,
            fading_count: stats.3,
            collapsed_count: stats.4,
            overdue_reviews: stats.6,
            average_strength: clamp_bp(stats.5),
            next_review_due,
        })
    }

    // ── Internal ──

    fn review_priority_score(
        &self,
        memory_state: &str,
        memory_strength: u16,
        decay_risk: u16,
        is_due: bool,
        interference_count: i64,
    ) -> u16 {
        let state_bonus = match memory_state {
            "collapsed" => 2200,
            "fading" => 1800,
            "at_risk" => 1500,
            "fragile" | "rebuilding" => 1000,
            _ => 0,
        };
        let due_bonus = if is_due { 1800 } else { 0 };
        let low_strength_bonus = ((10_000 - memory_strength as i64) / 5).max(0);
        let interference_bonus = (interference_count.min(4) * 350).max(0);
        clamp_bp(
            decay_risk as i64 + state_bonus + due_bonus + low_strength_bonus + interference_bonus,
        )
    }

    fn review_action_for_state(
        &self,
        memory_state: &str,
        is_due: bool,
        interference_count: i64,
    ) -> String {
        if interference_count > 0 && matches!(memory_state, "fragile" | "fading" | "at_risk") {
            return "interference_repair".to_string();
        }
        if is_due && matches!(memory_state, "confirmed" | "locked_in") {
            return "retention_check".to_string();
        }
        match memory_state {
            "collapsed" => "rebuild_foundation",
            "fading" | "at_risk" => "urgent_recall_repair",
            "fragile" | "rebuilding" => "guided_reinforcement",
            "encoded" | "accessible" => "spaced_review",
            "anchoring" | "recovered" => "stabilize_memory",
            _ => "maintenance_review",
        }
        .to_string()
    }

    fn schedule_type_for_state(&self, memory_state: &str) -> String {
        match memory_state {
            "collapsed" | "fading" | "at_risk" => "decay_recovery",
            "fragile" | "rebuilding" => "repair_review",
            _ => "spaced_review",
        }
        .to_string()
    }

    fn topic_action_for_summary(&self, summary: &TopicMemorySummary) -> &'static str {
        if summary.collapsed_items > 0 {
            "rebuild_foundation"
        } else if summary.overdue_reviews > 0 {
            "run_due_reviews"
        } else if summary.fragile_items > 0 {
            "stabilize_fragile_nodes"
        } else if summary.average_strength < 5000 {
            "reinforce_topic"
        } else {
            "maintain_retention"
        }
    }

    fn append_event(&self, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| EcoachError::Serialization(e.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, 'memory', ?3, ?4, ?5, ?6)",
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
}
