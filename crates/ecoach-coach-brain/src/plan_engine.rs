use chrono::{Days, NaiveDate, Utc};
use ecoach_substrate::{DomainEvent, EcoachError, EcoachResult};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachMissionMemory {
    pub id: i64,
    pub mission_id: i64,
    pub plan_day_id: Option<i64>,
    pub student_id: i64,
    pub session_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub mission_status: String,
    pub attempt_count: i64,
    pub correct_count: i64,
    pub accuracy_score: Option<i64>,
    pub avg_latency_ms: Option<i64>,
    pub misconception_tags: Vec<String>,
    pub review_due_at: Option<String>,
    pub next_action_type: String,
    pub strategy_effect: Option<String>,
    pub summary_json: String,
    pub review_status: String,
}

pub struct PlanEngine<'a> {
    conn: &'a Connection,
}

impl<'a> PlanEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn generate_plan(
        &self,
        student_id: i64,
        exam_target: &str,
        exam_date: &str,
        daily_budget_minutes: i64,
    ) -> EcoachResult<i64> {
        let today = Utc::now().date_naive();
        let exam = NaiveDate::parse_from_str(exam_date, "%Y-%m-%d")
            .map_err(|err| EcoachError::Validation(err.to_string()))?;
        let total_days = (exam - today).num_days().max(1);
        let phase = phase_for_remaining_days(total_days);
        let selected_subjects = self.load_selected_subjects(student_id)?;
        let plan_data_json = serde_json::to_string(&serde_json::json!({
            "selected_subjects": selected_subjects,
            "generated_at": today.to_string(),
            "phase_model": "days_to_exam",
            "daily_budget_minutes": daily_budget_minutes,
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO coach_plans (
                    student_id, exam_target, exam_date, start_date, total_days,
                    daily_budget_minutes, current_phase, status, plan_data_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', ?8)",
                params![
                    student_id,
                    exam_target,
                    exam_date,
                    today.to_string(),
                    total_days,
                    daily_budget_minutes,
                    phase,
                    plan_data_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let plan_id = self.conn.last_insert_rowid();

        for offset in 0..total_days {
            let plan_date = today
                .checked_add_days(Days::new(offset as u64))
                .ok_or_else(|| {
                    EcoachError::Validation("failed to compute plan date".to_string())
                })?;
            let remaining_days = total_days - offset;
            let day_phase = phase_for_remaining_days(remaining_days);
            let day_target_minutes = if is_review_day(offset) {
                (daily_budget_minutes / 2).max(15)
            } else {
                daily_budget_minutes
            };
            let day_status = if offset == 0 { "active" } else { "pending" };

            self.conn
                .execute(
                    "INSERT INTO coach_plan_days (plan_id, date, phase, target_minutes, status)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        plan_id,
                        plan_date.to_string(),
                        if is_review_day(offset) {
                            "review_day"
                        } else {
                            day_phase
                        },
                        day_target_minutes,
                        day_status,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        self.append_runtime_event(DomainEvent::new(
            "plan.generated",
            plan_id.to_string(),
            serde_json::json!({
                "student_id": student_id,
                "exam_target": exam_target,
                "exam_date": exam_date,
                "phase": phase,
                "daily_budget_minutes": daily_budget_minutes,
                "plan_day_count": total_days,
                "selected_subjects": selected_subjects,
            }),
        ))?;

        Ok(plan_id)
    }

    pub fn generate_today_mission(&self, student_id: i64) -> EcoachResult<i64> {
        let today = Utc::now().date_naive().to_string();
        let (plan_day_id, plan_day_phase, target_minutes) =
            self.ensure_active_plan_day(student_id, &today)?;
        let topic_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT topic_id FROM student_topic_states
                 WHERE student_id = ?1
                 ORDER BY priority_score DESC, gap_score DESC
                 LIMIT 1",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let topic_id = topic_id
            .ok_or_else(|| EcoachError::NotFound("no prioritized topic found".to_string()))?;
        let subject_id: i64 = self
            .conn
            .query_row(
                "SELECT subject_id FROM topics WHERE id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let activity_type = self.resolve_activity_type(student_id, topic_id, &plan_day_phase)?;
        let mission_title = mission_title_for_activity(activity_type);

        self.conn
            .execute(
                "INSERT INTO coach_missions (
                    plan_day_id, student_id, title, reason, subject_id, primary_topic_id, activity_type, target_minutes, status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending')",
                params![
                    plan_day_id,
                    student_id,
                    mission_title,
                    format!("Planned {} mission for the active coach day", activity_type),
                    subject_id,
                    topic_id,
                    activity_type,
                    target_minutes,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mission_id = self.conn.last_insert_rowid();
        self.append_runtime_event(DomainEvent::new(
            "mission.generated",
            mission_id.to_string(),
            serde_json::json!({
                "student_id": student_id,
                "subject_id": subject_id,
                "topic_id": topic_id,
                "activity_type": activity_type,
                "plan_day_id": plan_day_id,
                "plan_day_phase": plan_day_phase,
            }),
        ))?;

        Ok(mission_id)
    }

    pub fn start_mission(&self, mission_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE coach_missions
                 SET status = 'active'
                 WHERE id = ?1 AND status IN ('pending', 'deferred')",
                [mission_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.append_runtime_event(DomainEvent::new(
            "mission.started",
            mission_id.to_string(),
            serde_json::json!({ "mission_id": mission_id }),
        ))?;
        Ok(())
    }

    pub fn complete_mission_from_session(
        &self,
        mission_id: i64,
        session_id: Option<i64>,
    ) -> EcoachResult<CoachMissionMemory> {
        let mission = self.load_mission_context(mission_id)?;
        let (attempt_count, correct_count, accuracy_score, avg_latency_ms, misconception_tags) =
            self.load_session_outcome(session_id)?;
        let prior_evidence_count =
            self.count_prior_coach_evidence(mission.student_id, mission.topic_id)?;
        let mission_status =
            derive_mission_status(accuracy_score, attempt_count, prior_evidence_count);
        let review_due_at = derive_review_due_at(accuracy_score);
        let next_action_type = derive_next_action_type(&mission_status);
        let strategy_effect = derive_strategy_effect(accuracy_score, prior_evidence_count);
        let summary_json = serde_json::to_string(&serde_json::json!({
            "mission_title": mission.title,
            "reason": mission.reason,
            "activity_type": mission.activity_type,
            "attempt_count": attempt_count,
            "correct_count": correct_count,
            "accuracy_score": accuracy_score,
            "avg_latency_ms": avg_latency_ms,
            "misconception_tags": misconception_tags,
            "next_action_type": next_action_type,
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let misconception_tags_json = serde_json::to_string(&misconception_tags)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE coach_missions
                 SET status = 'completed', completed_at = datetime('now')
                 WHERE id = ?1",
                [mission_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "UPDATE coach_plan_days
                 SET status = CASE WHEN ?2 > 0 THEN 'completed' ELSE 'partial' END
                 WHERE id = ?1",
                params![mission.plan_day_id, attempt_count],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO coach_session_evidence (
                    mission_id, student_id, subject_id, topic_id, activity_type, attempt_count,
                    correct_count, accuracy, avg_latency_ms, misconception_tags, completed_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, datetime('now'))",
                params![
                    mission_id,
                    mission.student_id,
                    mission.subject_id,
                    mission.topic_id,
                    mission.activity_type,
                    attempt_count,
                    correct_count,
                    accuracy_score,
                    avg_latency_ms,
                    misconception_tags_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.sync_coach_topic_profile(
            mission.student_id,
            mission.topic_id,
            attempt_count,
            misconception_tags.len() as i64,
            &mission_status,
        )?;
        self.update_blockers(mission.student_id, mission.topic_id, &mission_status)?;

        self.conn
            .execute(
                "INSERT INTO coach_mission_memories (
                    mission_id, plan_day_id, student_id, session_id, subject_id, topic_id,
                    mission_status, attempt_count, correct_count, accuracy_score, avg_latency_ms,
                    misconception_tags, review_due_at, next_action_type, strategy_effect,
                    summary_json, review_status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 'pending')
                 ON CONFLICT(mission_id) DO UPDATE SET
                    session_id = excluded.session_id,
                    mission_status = excluded.mission_status,
                    attempt_count = excluded.attempt_count,
                    correct_count = excluded.correct_count,
                    accuracy_score = excluded.accuracy_score,
                    avg_latency_ms = excluded.avg_latency_ms,
                    misconception_tags = excluded.misconception_tags,
                    review_due_at = excluded.review_due_at,
                    next_action_type = excluded.next_action_type,
                    strategy_effect = excluded.strategy_effect,
                    summary_json = excluded.summary_json,
                    review_status = 'pending',
                    updated_at = datetime('now')",
                params![
                    mission_id,
                    mission.plan_day_id,
                    mission.student_id,
                    session_id,
                    mission.subject_id,
                    mission.topic_id,
                    mission_status,
                    attempt_count,
                    correct_count,
                    accuracy_score,
                    avg_latency_ms,
                    misconception_tags_json,
                    review_due_at,
                    next_action_type,
                    strategy_effect,
                    summary_json,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let memory_id = self.conn.last_insert_rowid();
        self.append_runtime_event(DomainEvent::new(
            "mission.completed",
            mission_id.to_string(),
            serde_json::json!({
                "mission_id": mission_id,
                "session_id": session_id,
                "mission_status": mission_status,
                "review_due_at": review_due_at,
                "next_action_type": next_action_type,
            }),
        ))?;
        self.append_runtime_event(DomainEvent::new(
            "review.scheduled",
            mission_id.to_string(),
            serde_json::json!({
                "memory_id": memory_id,
                "review_due_at": review_due_at,
                "student_id": mission.student_id,
            }),
        ))?;

        self.get_pending_mission_review(mission.student_id)?
            .ok_or_else(|| EcoachError::NotFound("mission memory was not persisted".to_string()))
    }

    pub fn get_pending_mission_review(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<CoachMissionMemory>> {
        self.conn
            .query_row(
                "SELECT id, mission_id, plan_day_id, student_id, session_id, subject_id, topic_id,
                        mission_status, attempt_count, correct_count, accuracy_score, avg_latency_ms,
                        misconception_tags, review_due_at, next_action_type, strategy_effect,
                        summary_json, review_status
                 FROM coach_mission_memories
                 WHERE student_id = ?1 AND review_status = 'pending'
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                [student_id],
                map_coach_mission_memory,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn acknowledge_mission_review(&self, memory_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE coach_mission_memories
                 SET review_status = 'acknowledged', updated_at = datetime('now')
                 WHERE id = ?1",
                [memory_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn load_selected_subjects(&self, student_id: i64) -> EcoachResult<Vec<String>> {
        let raw: Option<String> = self
            .conn
            .query_row(
                "SELECT preferred_subjects FROM student_profiles WHERE account_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        serde_json::from_str::<Vec<String>>(raw.as_deref().unwrap_or("[]"))
            .map_err(|err| EcoachError::Serialization(err.to_string()))
    }

    fn ensure_active_plan_day(
        &self,
        student_id: i64,
        today: &str,
    ) -> EcoachResult<(i64, String, i64)> {
        let active_day = self
            .conn
            .query_row(
                "SELECT cpd.id, cpd.phase, cpd.target_minutes
                 FROM coach_plan_days cpd
                 INNER JOIN coach_plans cp ON cp.id = cpd.plan_id
                 WHERE cp.student_id = ?1
                   AND cp.status IN ('active', 'stale')
                   AND cpd.date = ?2
                 ORDER BY cpd.id DESC
                 LIMIT 1",
                params![student_id, today],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let (plan_day_id, phase, target_minutes) = active_day.ok_or_else(|| {
            EcoachError::NotFound("no active coach plan day for today".to_string())
        })?;
        self.conn
            .execute(
                "UPDATE coach_plan_days SET status = 'active' WHERE id = ?1 AND status = 'pending'",
                [plan_day_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok((plan_day_id, phase, target_minutes))
    }

    fn resolve_activity_type(
        &self,
        student_id: i64,
        topic_id: i64,
        plan_day_phase: &str,
    ) -> EcoachResult<&'static str> {
        let (mastery_score, speed_score, fragility_score): (i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT mastery_score, speed_score, fragility_score
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((0, 0, 0));

        let activity = if mastery_score < 4500 || fragility_score > 6000 {
            "repair"
        } else {
            match plan_day_phase {
                "foundation" => "learn",
                "strengthening" => "guided_practice",
                "performance" => {
                    if speed_score < 4500 {
                        "speed_drill"
                    } else {
                        "mixed_test"
                    }
                }
                "consolidation" | "review_day" => "review",
                "final_revision" => "pressure_conditioning",
                _ => "guided_practice",
            }
        };

        Ok(activity)
    }

    fn load_mission_context(&self, mission_id: i64) -> EcoachResult<MissionContext> {
        self.conn
            .query_row(
                "SELECT plan_day_id, student_id, title, reason, subject_id, primary_topic_id, activity_type
                 FROM coach_missions
                 WHERE id = ?1",
                [mission_id],
                |row| {
                    Ok(MissionContext {
                        plan_day_id: row.get(0)?,
                        student_id: row.get(1)?,
                        title: row.get(2)?,
                        reason: row.get(3)?,
                        subject_id: row.get(4)?,
                        topic_id: row.get(5)?,
                        activity_type: row.get(6)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_session_outcome(
        &self,
        session_id: Option<i64>,
    ) -> EcoachResult<(i64, i64, Option<i64>, Option<i64>, Vec<String>)> {
        let Some(session_id) = session_id else {
            return Ok((0, 0, None, None, Vec::new()));
        };

        let (attempt_count, correct_count, accuracy_score, avg_latency_ms): (
            i64,
            i64,
            Option<i64>,
            Option<i64>,
        ) = self
            .conn
            .query_row(
                "SELECT answered_questions, correct_questions, accuracy_score, avg_response_time_ms
                 FROM sessions
                 WHERE id = ?1",
                [session_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut statement = self
            .conn
            .prepare(
                "SELECT DISTINCT COALESCE(mp.title, sqa.error_type)
                 FROM student_question_attempts sqa
                 LEFT JOIN misconception_patterns mp ON mp.id = sqa.misconception_triggered_id
                 WHERE sqa.session_id = ?1
                   AND COALESCE(mp.title, sqa.error_type) IS NOT NULL
                 ORDER BY COALESCE(mp.title, sqa.error_type) ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([session_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut misconception_tags = Vec::new();
        for row in rows {
            misconception_tags.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        Ok((
            attempt_count,
            correct_count,
            accuracy_score,
            avg_latency_ms,
            misconception_tags,
        ))
    }

    fn count_prior_coach_evidence(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<i64> {
        let Some(topic_id) = topic_id else {
            return Ok(0);
        };
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM coach_session_evidence WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn sync_coach_topic_profile(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        attempt_count: i64,
        misconception_count: i64,
        mission_status: &str,
    ) -> EcoachResult<()> {
        let Some(topic_id) = topic_id else {
            return Ok(());
        };
        let (mastery_estimate, fragility_score, speed_score, updated_at): (
            i64,
            i64,
            i64,
            Option<String>,
        ) = self
            .conn
            .query_row(
                "SELECT mastery_score, fragility_score, speed_score, updated_at
                 FROM student_topic_states
                 WHERE student_id = ?1 AND topic_id = ?2",
                params![student_id, topic_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?
            .unwrap_or((0, 0, 0, None));

        self.conn
            .execute(
                "INSERT INTO coach_topic_profiles (
                    student_id, topic_id, mastery_estimate, fragility_score, speed_score,
                    misconception_recurrence, evidence_count, attempt_count, last_seen_at,
                    blocked_status, repair_priority, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, datetime('now'), ?8, ?9, COALESCE(?10, datetime('now')))
                 ON CONFLICT(student_id, topic_id) DO UPDATE SET
                    mastery_estimate = excluded.mastery_estimate,
                    fragility_score = excluded.fragility_score,
                    speed_score = excluded.speed_score,
                    misconception_recurrence = coach_topic_profiles.misconception_recurrence + excluded.misconception_recurrence,
                    evidence_count = coach_topic_profiles.evidence_count + 1,
                    attempt_count = coach_topic_profiles.attempt_count + excluded.attempt_count,
                    last_seen_at = datetime('now'),
                    blocked_status = excluded.blocked_status,
                    repair_priority = excluded.repair_priority,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    topic_id,
                    mastery_estimate,
                    fragility_score,
                    speed_score,
                    misconception_count,
                    attempt_count,
                    if mission_status == "repair_required" { 1 } else { 0 },
                    if mission_status == "repair_required" { 9000 } else { 3000 },
                    updated_at,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(())
    }

    fn update_blockers(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        mission_status: &str,
    ) -> EcoachResult<()> {
        let Some(topic_id) = topic_id else {
            return Ok(());
        };

        if mission_status == "repair_required" {
            self.conn
                .execute(
                    "INSERT INTO coach_blockers (student_id, topic_id, reason, severity)
                     VALUES (?1, ?2, 'repeated low mission accuracy', 'high')",
                    params![student_id, topic_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        } else {
            self.conn
                .execute(
                    "UPDATE coach_blockers
                     SET resolved_at = datetime('now')
                     WHERE student_id = ?1 AND topic_id = ?2 AND resolved_at IS NULL",
                    params![student_id, topic_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn append_runtime_event(&self, event: DomainEvent) -> EcoachResult<()> {
        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO runtime_events (
                    event_id, event_type, aggregate_kind, aggregate_id, trace_id, payload_json, occurred_at
                 ) VALUES (?1, ?2, 'coach', ?3, ?4, ?5, ?6)",
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

#[derive(Debug)]
struct MissionContext {
    plan_day_id: Option<i64>,
    student_id: i64,
    title: String,
    reason: String,
    subject_id: Option<i64>,
    topic_id: Option<i64>,
    activity_type: String,
}

fn phase_for_remaining_days(total_days: i64) -> &'static str {
    if total_days > 90 {
        "foundation"
    } else if total_days > 45 {
        "strengthening"
    } else if total_days > 21 {
        "performance"
    } else if total_days > 7 {
        "consolidation"
    } else {
        "final_revision"
    }
}

fn derive_mission_status(
    accuracy_score: Option<i64>,
    attempt_count: i64,
    prior_evidence_count: i64,
) -> &'static str {
    match (accuracy_score, attempt_count) {
        (_, 0) => "partial",
        (Some(score), _) if score < 4000 && prior_evidence_count >= 1 => "repair_required",
        (Some(score), _) if score < 6500 => "review_due",
        _ => "completed",
    }
}

fn derive_review_due_at(accuracy_score: Option<i64>) -> Option<String> {
    let today = Utc::now().date_naive();
    let due_date = match accuracy_score {
        Some(score) if score < 4000 => today.checked_add_days(Days::new(1)),
        Some(score) if score < 6500 => today.checked_add_days(Days::new(3)),
        Some(_) => today.checked_add_days(Days::new(7)),
        None => today.checked_add_days(Days::new(2)),
    }?;
    Some(due_date.to_string())
}

fn derive_next_action_type(mission_status: &str) -> &'static str {
    match mission_status {
        "repair_required" => "start_repair",
        "review_due" => "review_results",
        "partial" => "resume_mission",
        _ => "start_today_mission",
    }
}

fn derive_strategy_effect(
    accuracy_score: Option<i64>,
    prior_evidence_count: i64,
) -> Option<String> {
    let effect = match accuracy_score {
        Some(score) if score < 4000 && prior_evidence_count >= 1 => "escalate_to_repair",
        Some(score) if score < 6500 => "schedule_short_review",
        Some(_) => "unlock_next_planned_step",
        None => "collect_more_evidence",
    };
    Some(effect.to_string())
}

fn map_coach_mission_memory(row: &rusqlite::Row<'_>) -> rusqlite::Result<CoachMissionMemory> {
    let misconception_json: String = row.get(12)?;
    let misconception_tags =
        serde_json::from_str::<Vec<String>>(&misconception_json).unwrap_or_default();

    Ok(CoachMissionMemory {
        id: row.get(0)?,
        mission_id: row.get(1)?,
        plan_day_id: row.get(2)?,
        student_id: row.get(3)?,
        session_id: row.get(4)?,
        subject_id: row.get(5)?,
        topic_id: row.get(6)?,
        mission_status: row.get(7)?,
        attempt_count: row.get(8)?,
        correct_count: row.get(9)?,
        accuracy_score: row.get(10)?,
        avg_latency_ms: row.get(11)?,
        misconception_tags,
        review_due_at: row.get(13)?,
        next_action_type: row.get(14)?,
        strategy_effect: row.get(15)?,
        summary_json: row.get(16)?,
        review_status: row.get(17)?,
    })
}

fn is_review_day(offset: i64) -> bool {
    offset > 0 && (offset + 1) % 7 == 0
}

fn mission_title_for_activity(activity_type: &str) -> &'static str {
    match activity_type {
        "learn" => "Foundation Learning Mission",
        "guided_practice" => "Guided Practice Mission",
        "speed_drill" => "Speed Conversion Mission",
        "mixed_test" => "Mixed Performance Mission",
        "review" => "Review and Recovery Mission",
        "pressure_conditioning" => "Pressure Conditioning Mission",
        _ => "Priority Repair Mission",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn generate_plan_creates_multiday_schedule_and_today_mission() {
        let conn = open_test_database();
        let pack_service = PackService::new(&conn);
        pack_service
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");

        conn.execute(
            "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES ('student', 'Ada', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        let student_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (?1, '[\"MATH\"]', 60)",
            [student_id],
        )
        .expect("student profile should insert");
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        conn.execute(
            "INSERT INTO student_topic_states (student_id, topic_id, mastery_score, gap_score, priority_score, fragility_score, speed_score)
             VALUES (?1, ?2, 3000, 8500, 9000, 6500, 4000)",
            params![student_id, topic_id],
        )
        .expect("topic state should insert");

        let engine = PlanEngine::new(&conn);
        let exam_date = Utc::now()
            .date_naive()
            .checked_add_days(Days::new(14))
            .expect("future date should exist")
            .to_string();
        let plan_id = engine
            .generate_plan(student_id, "BECE", &exam_date, 60)
            .expect("plan should generate");
        let mission_id = engine
            .generate_today_mission(student_id)
            .expect("mission should generate");
        engine
            .start_mission(mission_id)
            .expect("mission should start cleanly");

        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let question_id: i64 = conn
            .query_row(
                "SELECT id FROM questions ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("question should exist");
        let misconception_id: i64 = conn
            .query_row(
                "SELECT id FROM misconception_patterns ORDER BY id ASC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("misconception should exist");
        let session_topic_ids = format!("[{}]", topic_id);
        conn.execute(
            "INSERT INTO sessions (
                student_id, session_type, subject_id, topic_ids, question_count, total_questions,
                is_timed, status, started_at, completed_at, answered_questions, correct_questions,
                accuracy_score, avg_response_time_ms
             ) VALUES (?1, 'coach_mission', ?2, ?3, 2, 2, 0, 'completed', datetime('now'), datetime('now'), 2, 1, 5000, 15000)",
            params![student_id, subject_id, session_topic_ids],
        )
        .expect("session should insert");
        let session_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO student_question_attempts (
                student_id, question_id, session_id, session_type, attempt_number, started_at, submitted_at,
                response_time_ms, is_correct, error_type, misconception_triggered_id
             ) VALUES (?1, ?2, ?3, 'coach_mission', 1, datetime('now'), datetime('now'), 18000, 0, 'misconception_triggered', ?4)",
            params![student_id, question_id, session_id, misconception_id],
        )
        .expect("attempt should insert");
        conn.execute(
            "INSERT INTO student_question_attempts (
                student_id, question_id, session_id, session_type, attempt_number, started_at, submitted_at,
                response_time_ms, is_correct
             ) VALUES (?1, ?2, ?3, 'coach_mission', 2, datetime('now'), datetime('now'), 12000, 1)",
            params![student_id, question_id, session_id],
        )
        .expect("second attempt should insert");
        let mission_memory = engine
            .complete_mission_from_session(mission_id, Some(session_id))
            .expect("mission memory should persist");

        let plan_day_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM coach_plan_days WHERE plan_id = ?1",
                [plan_id],
                |row| row.get(0),
            )
            .expect("plan day count should query");
        let mission_plan_day_id: i64 = conn
            .query_row(
                "SELECT plan_day_id FROM coach_missions WHERE id = ?1",
                [mission_id],
                |row| row.get(0),
            )
            .expect("mission should belong to a plan day");
        let pending_review_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM coach_mission_memories WHERE student_id = ?1 AND review_status = 'pending'",
                [student_id],
                |row| row.get(0),
            )
            .expect("mission review count should query");

        assert!(plan_day_count >= 14);
        assert!(mission_plan_day_id > 0);
        assert_eq!(mission_memory.review_status, "pending");
        assert_eq!(pending_review_count, 1);
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
