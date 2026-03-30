use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::readiness_engine::ReadinessEngine;
use crate::topic_case::{TopicCase, build_topic_case};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyRoute {
    pub id: i64,
    pub student_id: i64,
    pub subject_id: i64,
    pub target_exam: Option<String>,
    pub route_type: String,
    pub status: String,
    pub current_station_code: Option<String>,
    pub route_summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyStation {
    pub id: i64,
    pub route_id: i64,
    pub station_code: String,
    pub title: String,
    pub topic_id: Option<i64>,
    pub sequence_no: i64,
    pub station_type: String,
    pub target_mastery_score: Option<BasisPoints>,
    pub target_accuracy_score: Option<BasisPoints>,
    pub target_readiness_score: Option<BasisPoints>,
    pub status: String,
    pub entry_rule: Value,
    pub completion_rule: Value,
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JourneyRouteSnapshot {
    pub route: JourneyRoute,
    pub stations: Vec<JourneyStation>,
}

pub struct JourneyService<'a> {
    conn: &'a Connection,
}

impl<'a> JourneyService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn build_or_refresh_route(
        &self,
        student_id: i64,
        subject_id: i64,
        target_exam: Option<&str>,
    ) -> EcoachResult<JourneyRouteSnapshot> {
        let readiness =
            ReadinessEngine::new(self.conn).build_subject_readiness(student_id, subject_id)?;
        let topic_cases = self.load_topic_cases(student_id, subject_id)?;
        if topic_cases.is_empty() {
            return Err(EcoachError::NotFound(
                "no topics available to build a journey route".to_string(),
            ));
        }

        self.conn
            .execute(
                "UPDATE journey_routes
                 SET status = 'stale', updated_at = datetime('now')
                 WHERE student_id = ?1 AND subject_id = ?2 AND status = 'active'",
                params![student_id, subject_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let route_type = if readiness.blocked_topic_count > 0
            || readiness.due_review_count + readiness.due_memory_count >= 2
        {
            "repair_route"
        } else if target_exam.is_some()
            && (readiness.readiness_score >= 7_200
                || readiness.recommended_mock_blueprint.contains("timed"))
        {
            "exam_route"
        } else {
            "mastery_route"
        };
        let ordered_cases = order_route_cases(&topic_cases, &readiness);
        let route_summary_json = serde_json::to_string(&json!({
            "generated_from": "journey_service",
            "readiness_score": readiness.readiness_score,
            "readiness_band": readiness.readiness_band,
            "recommended_mock_blueprint": readiness.recommended_mock_blueprint,
            "topic_count": ordered_cases.len(),
            "blocked_topic_count": readiness.blocked_topic_count,
            "due_review_count": readiness.due_review_count,
            "due_memory_count": readiness.due_memory_count,
            "weak_topic_count": readiness.weak_topic_count,
            "route_intent": route_type,
            "top_hypotheses": ordered_cases
                .iter()
                .take(3)
                .map(|item| json!({
                    "topic_id": item.topic_id,
                    "topic_name": &item.topic_name,
                    "primary_hypothesis_code": &item.primary_hypothesis_code,
                    "recommended_mode": &item.recommended_intervention.mode,
                }))
                .collect::<Vec<_>>(),
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO journey_routes (
                    student_id, subject_id, target_exam, route_type, status, route_summary_json
                 ) VALUES (?1, ?2, ?3, ?4, 'active', ?5)",
                params![
                    student_id,
                    subject_id,
                    target_exam,
                    route_type,
                    route_summary_json
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let route_id = self.conn.last_insert_rowid();

        for (index, topic_case) in ordered_cases.iter().enumerate() {
            let station_code = format!("station_{:02}", index + 1);
            let station_type = station_type_for_case(
                topic_case,
                route_type,
                index == 0
                    && (readiness.due_review_count > 0 || readiness.due_memory_count > 0)
                    && is_review_station_candidate(topic_case),
            );
            let status = if index == 0 { "active" } else { "locked" };
            let entry_rule = if index == 0 {
                json!({
                    "entry": "start_immediately",
                    "primary_hypothesis_code": &topic_case.primary_hypothesis_code,
                    "requires_probe": topic_case.requires_probe,
                    "recommended_mode": &topic_case.recommended_intervention.mode,
                })
            } else {
                json!({
                    "requires_previous_station": format!("station_{:02}", index),
                    "primary_hypothesis_code": &topic_case.primary_hypothesis_code,
                    "requires_probe": topic_case.requires_probe,
                })
            };
            let completion_rule = json!({
                "target_mastery_score": clamp_bp(topic_case.mastery_score as i64 + 1500),
                "target_accuracy_score": target_accuracy_for_station(station_type, topic_case, index as i64),
                "target_readiness_score": readiness.readiness_score,
                "min_questions": min_questions_for_station(station_type),
                "proof_gaps": &topic_case.proof_gaps,
                "requires_delayed_recall": station_type == "review",
                "requires_timed_success": station_type == "performance",
                "recommended_mode": &topic_case.recommended_intervention.mode,
            });
            let evidence = json!({
                "topic_id": topic_case.topic_id,
                "topic_name": &topic_case.topic_name,
                "priority_score": topic_case.priority_score,
                "diagnosis_certainty": topic_case.diagnosis_certainty,
                "primary_hypothesis_code": &topic_case.primary_hypothesis_code,
                "active_hypotheses": &topic_case.active_hypotheses,
                "recommended_intervention": &topic_case.recommended_intervention,
                "open_questions": &topic_case.open_questions,
            });
            self.conn
                .execute(
                    "INSERT INTO journey_stations (
                        route_id, station_code, title, topic_id, sequence_no, station_type,
                        target_mastery_score, target_accuracy_score, target_readiness_score,
                        status, entry_rule_json, completion_rule_json, evidence_json, unlocked_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                                CASE WHEN ?10 = 'active' THEN datetime('now') ELSE NULL END)",
                    params![
                        route_id,
                        station_code,
                        station_title_for_case(topic_case, station_type),
                        topic_case.topic_id,
                        (index + 1) as i64,
                        station_type,
                        clamp_bp(topic_case.mastery_score as i64 + 1500),
                        target_accuracy_for_station(station_type, topic_case, index as i64),
                        readiness.readiness_score,
                        status,
                        serde_json::to_string(&entry_rule)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        serde_json::to_string(&completion_rule)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        serde_json::to_string(&evidence)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.conn
            .execute(
                "UPDATE journey_routes
                 SET current_station_code = 'station_01', updated_at = datetime('now')
                 WHERE id = ?1",
                [route_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.append_runtime_event(DomainEvent::new(
            "journey.route_built",
            route_id.to_string(),
            json!({
                "student_id": student_id,
                "subject_id": subject_id,
                "route_type": route_type,
                "station_count": ordered_cases.len(),
            }),
        ))?;

        self.get_route_snapshot(route_id)?
            .ok_or_else(|| EcoachError::NotFound("journey route was not created".to_string()))
    }

    pub fn get_active_route(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Option<JourneyRouteSnapshot>> {
        let route_id = self
            .conn
            .query_row(
                "SELECT id FROM journey_routes
                 WHERE student_id = ?1 AND subject_id = ?2 AND status = 'active'
                 ORDER BY id DESC LIMIT 1",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        match route_id {
            Some(route_id) => self.get_route_snapshot(route_id),
            None => Ok(None),
        }
    }

    pub fn complete_station(
        &self,
        station_id: i64,
        evidence: &Value,
    ) -> EcoachResult<JourneyRouteSnapshot> {
        let (route_id, sequence_no, station_code, station_type, completion_rule_json): (
            i64,
            i64,
            String,
            String,
            String,
        ) = self
            .conn
            .query_row(
                "SELECT route_id, sequence_no, station_code, station_type, completion_rule_json
                 FROM journey_stations
                 WHERE id = ?1",
                [station_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let evidence_json = serde_json::to_string(evidence)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let completion_rule = parse_json_value(4, &completion_rule_json)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let decision = evaluate_station_progression(&station_type, &completion_rule, evidence);
        if !decision.passed {
            self.conn
                .execute(
                    "UPDATE journey_stations
                     SET status = 'active',
                         evidence_json = ?1,
                         updated_at = datetime('now')
                     WHERE id = ?2",
                    params![evidence_json, station_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.append_runtime_event(DomainEvent::new(
                "journey.station_retry_required",
                route_id.to_string(),
                json!({
                    "station_id": station_id,
                    "station_code": station_code,
                    "station_type": station_type,
                    "missing_criteria": decision.missing_criteria,
                }),
            ))?;
            return self
                .get_route_snapshot(route_id)?
                .ok_or_else(|| EcoachError::NotFound("journey route not found".to_string()));
        }
        self.conn
            .execute(
                "UPDATE journey_stations
                 SET status = 'completed',
                     evidence_json = ?1,
                     completed_at = datetime('now'),
                     updated_at = datetime('now')
                 WHERE id = ?2",
                params![evidence_json, station_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let next_station = self
            .conn
            .query_row(
                "SELECT id, station_code
                 FROM journey_stations
                 WHERE route_id = ?1 AND sequence_no = ?2",
                params![route_id, sequence_no + 1],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if let Some((next_station_id, next_station_code)) = next_station {
            self.conn
                .execute(
                    "UPDATE journey_stations
                     SET status = 'active', unlocked_at = datetime('now'), updated_at = datetime('now')
                     WHERE id = ?1",
                    [next_station_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.conn
                .execute(
                    "UPDATE journey_routes
                     SET current_station_code = ?1, updated_at = datetime('now')
                     WHERE id = ?2",
                    params![next_station_code, route_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        } else {
            self.conn
                .execute(
                    "UPDATE journey_routes
                     SET status = 'completed', current_station_code = NULL, updated_at = datetime('now')
                     WHERE id = ?1",
                    [route_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        self.append_runtime_event(DomainEvent::new(
            "journey.station_completed",
            route_id.to_string(),
            json!({
                "station_id": station_id,
                "station_code": station_code,
                "station_type": station_type,
            }),
        ))?;

        self.get_route_snapshot(route_id)?
            .ok_or_else(|| EcoachError::NotFound("journey route not found".to_string()))
    }

    fn get_route_snapshot(&self, route_id: i64) -> EcoachResult<Option<JourneyRouteSnapshot>> {
        let route = self
            .conn
            .query_row(
                "SELECT id, student_id, subject_id, target_exam, route_type, status, current_station_code, route_summary_json
                 FROM journey_routes
                 WHERE id = ?1",
                [route_id],
                map_route,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let Some(route) = route else {
            return Ok(None);
        };

        let mut statement = self
            .conn
            .prepare(
                "SELECT id, route_id, station_code, title, topic_id, sequence_no, station_type,
                        target_mastery_score, target_accuracy_score, target_readiness_score,
                        status, entry_rule_json, completion_rule_json, evidence_json
                 FROM journey_stations
                 WHERE route_id = ?1
                 ORDER BY sequence_no ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([route_id], map_station)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut stations = Vec::new();
        for row in rows {
            stations.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        Ok(Some(JourneyRouteSnapshot { route, stations }))
    }

    fn load_topic_candidates(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<TopicCandidate>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT t.id, t.name,
                        COALESCE(sts.mastery_score, 0),
                        COALESCE(sts.gap_score, 10000),
                        COALESCE(sts.fragility_score, 0),
                        COALESCE(sts.priority_score, 0)
                 FROM topics t
                 LEFT JOIN student_topic_states sts
                    ON sts.topic_id = t.id
                   AND sts.student_id = ?1
                 WHERE t.subject_id = ?2
                 ORDER BY COALESCE(sts.priority_score, 0) DESC,
                          COALESCE(sts.gap_score, 10000) DESC,
                          t.display_order ASC,
                          t.id ASC
                 LIMIT 8",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                Ok(TopicCandidate {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    mastery_score: row.get(2)?,
                    gap_score: row.get(3)?,
                    fragility_score: row.get(4)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn load_topic_cases(&self, student_id: i64, subject_id: i64) -> EcoachResult<Vec<TopicCase>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.topic_id
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1 AND t.subject_id = ?2
                 ORDER BY sts.priority_score DESC,
                          sts.repair_priority DESC,
                          sts.gap_score DESC,
                          sts.topic_id ASC
                 LIMIT 8",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            let topic_id = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            items.push(build_topic_case(self.conn, student_id, topic_id)?);
        }
        Ok(items)
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
struct TopicCandidate {
    topic_id: i64,
    topic_name: String,
    mastery_score: BasisPoints,
    gap_score: BasisPoints,
    fragility_score: BasisPoints,
}

fn station_type_for_candidate(
    mastery_score: BasisPoints,
    gap_score: BasisPoints,
    fragility_score: BasisPoints,
) -> &'static str {
    if gap_score >= 7_500 || fragility_score >= 6_500 {
        "repair"
    } else if mastery_score < 4_500 {
        "foundation"
    } else if mastery_score < 7_500 {
        "checkpoint"
    } else {
        "performance"
    }
}

fn order_route_cases<'a>(
    topic_cases: &'a [TopicCase],
    readiness: &crate::readiness_engine::StudentReadinessSnapshot,
) -> Vec<&'a TopicCase> {
    let mut ordered = Vec::new();
    if readiness.due_review_count > 0 || readiness.due_memory_count > 0 {
        if let Some(review_case) = topic_cases
            .iter()
            .find(|item| is_review_station_candidate(item))
        {
            ordered.push(review_case);
        }
    }
    for topic_case in topic_cases {
        if ordered
            .iter()
            .any(|item| item.topic_id == topic_case.topic_id)
        {
            continue;
        }
        ordered.push(topic_case);
    }
    ordered
}

fn is_review_station_candidate(topic_case: &TopicCase) -> bool {
    matches!(topic_case.primary_hypothesis_code.as_str(), "memory_decay")
        || matches!(
            topic_case.memory_state.as_str(),
            "fragile" | "at_risk" | "fading" | "rebuilding" | "collapsed"
        )
}

fn station_type_for_case(
    topic_case: &TopicCase,
    route_type: &str,
    force_review: bool,
) -> &'static str {
    if force_review || is_review_station_candidate(topic_case) {
        "review"
    } else if topic_case.active_blocker.is_some()
        || topic_case.primary_hypothesis_code == "knowledge_gap" && topic_case.mastery_score < 4500
    {
        "foundation"
    } else if topic_case.active_blocker.is_some()
        || matches!(
            topic_case.primary_hypothesis_code.as_str(),
            "blocked_topic" | "conceptual_confusion" | "knowledge_gap"
        )
    {
        "repair"
    } else if topic_case.primary_hypothesis_code == "execution_drift" {
        "checkpoint"
    } else if route_type == "exam_route"
        || topic_case.primary_hypothesis_code == "pressure_collapse"
        || topic_case.pressure_collapse_index >= 5500
    {
        "performance"
    } else {
        station_type_for_candidate(
            topic_case.mastery_score,
            topic_case.gap_score,
            topic_case.fragility_score,
        )
    }
}

fn station_title_for_case(topic_case: &TopicCase, station_type: &str) -> String {
    match station_type {
        "review" => format!("Reactivate {}", topic_case.topic_name),
        "foundation" => format!("Rebuild {}", topic_case.topic_name),
        "repair" => format!("Repair {}", topic_case.topic_name),
        "checkpoint" => format!("Checkpoint {}", topic_case.topic_name),
        "performance" => format!("Perform Under Load: {}", topic_case.topic_name),
        _ => format!("{} Station", topic_case.topic_name),
    }
}

fn min_questions_for_station(station_type: &str) -> i64 {
    match station_type {
        "review" => 4,
        "foundation" | "repair" => 6,
        "performance" => 8,
        _ => 5,
    }
}

fn target_accuracy_for_station(
    station_type: &str,
    topic_case: &TopicCase,
    sequence_index: i64,
) -> BasisPoints {
    let base = match station_type {
        "review" => 7600,
        "performance" => 7200,
        "repair" | "foundation" => 6500,
        _ => 6800,
    };
    clamp_bp(base + sequence_index * 150 + (topic_case.diagnosis_certainty as i64 / 20))
}

#[derive(Debug)]
struct StationProgressDecision {
    passed: bool,
    missing_criteria: Vec<String>,
}

fn evaluate_station_progression(
    station_type: &str,
    completion_rule: &Value,
    evidence: &Value,
) -> StationProgressDecision {
    if evidence
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status.eq_ignore_ascii_case("passed"))
    {
        return StationProgressDecision {
            passed: true,
            missing_criteria: Vec::new(),
        };
    }

    let mut missing_criteria = Vec::new();
    let answered_questions = evidence
        .get("answered_questions")
        .or_else(|| evidence.get("attempt_count"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if let Some(min_questions) = completion_rule
        .get("min_questions")
        .and_then(Value::as_i64)
        .filter(|min_questions| answered_questions < *min_questions)
    {
        missing_criteria.push(format!(
            "requires at least {} questions but only {} were recorded",
            min_questions, answered_questions
        ));
    }

    let accuracy_score = evidence
        .get("accuracy_score")
        .or_else(|| evidence.get("timed_accuracy"))
        .and_then(Value::as_i64);
    if let (Some(target_accuracy_score), Some(actual_accuracy_score)) = (
        completion_rule
            .get("target_accuracy_score")
            .and_then(Value::as_i64),
        accuracy_score,
    ) {
        if actual_accuracy_score < target_accuracy_score {
            missing_criteria.push(format!(
                "accuracy {} is below target {}",
                actual_accuracy_score, target_accuracy_score
            ));
        }
    }

    if let (Some(target_mastery_score), Some(actual_mastery_score)) = (
        completion_rule
            .get("target_mastery_score")
            .and_then(Value::as_i64),
        evidence.get("mastery_score").and_then(Value::as_i64),
    ) {
        if actual_mastery_score < target_mastery_score {
            missing_criteria.push(format!(
                "mastery {} is below target {}",
                actual_mastery_score, target_mastery_score
            ));
        }
    }

    if let (Some(target_readiness_score), Some(actual_readiness_score)) = (
        completion_rule
            .get("target_readiness_score")
            .and_then(Value::as_i64),
        evidence.get("readiness_score").and_then(Value::as_i64),
    ) {
        if actual_readiness_score < target_readiness_score {
            missing_criteria.push(format!(
                "readiness {} is below target {}",
                actual_readiness_score, target_readiness_score
            ));
        }
    }

    if completion_rule
        .get("requires_delayed_recall")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        && !evidence
            .get("delayed_recall_passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        missing_criteria.push("delayed recall proof is still missing".to_string());
    }

    if completion_rule
        .get("requires_timed_success")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let timed_success = evidence
            .get("timed_success")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let timed_accuracy = evidence.get("timed_accuracy").and_then(Value::as_i64);
        let timed_target = completion_rule
            .get("target_accuracy_score")
            .and_then(Value::as_i64)
            .unwrap_or(7000);
        if !timed_success && timed_accuracy.unwrap_or(0) < timed_target {
            missing_criteria.push("timed success evidence is still missing".to_string());
        }
    }

    if missing_criteria.is_empty() && station_type == "review" && answered_questions == 0 {
        missing_criteria.push("review station needs at least one retrieval attempt".to_string());
    }

    StationProgressDecision {
        passed: missing_criteria.is_empty(),
        missing_criteria,
    }
}

fn map_route(row: &rusqlite::Row<'_>) -> rusqlite::Result<JourneyRoute> {
    let route_summary_json: String = row.get(7)?;
    let route_summary = serde_json::from_str::<Value>(&route_summary_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(err))
    })?;
    Ok(JourneyRoute {
        id: row.get(0)?,
        student_id: row.get(1)?,
        subject_id: row.get(2)?,
        target_exam: row.get(3)?,
        route_type: row.get(4)?,
        status: row.get(5)?,
        current_station_code: row.get(6)?,
        route_summary,
    })
}

fn map_station(row: &rusqlite::Row<'_>) -> rusqlite::Result<JourneyStation> {
    let entry_rule_json: String = row.get(11)?;
    let completion_rule_json: String = row.get(12)?;
    let evidence_json: String = row.get(13)?;
    Ok(JourneyStation {
        id: row.get(0)?,
        route_id: row.get(1)?,
        station_code: row.get(2)?,
        title: row.get(3)?,
        topic_id: row.get(4)?,
        sequence_no: row.get(5)?,
        station_type: row.get(6)?,
        target_mastery_score: row.get(7)?,
        target_accuracy_score: row.get(8)?,
        target_readiness_score: row.get(9)?,
        status: row.get(10)?,
        entry_rule: parse_json_value(11, &entry_rule_json)?,
        completion_rule: parse_json_value(12, &completion_rule_json)?,
        evidence: parse_json_value(13, &evidence_json)?,
    })
}

fn parse_json_value(column_index: usize, raw: &str) -> rusqlite::Result<Value> {
    serde_json::from_str::<Value>(raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            column_index,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn journey_service_builds_and_advances_route() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_ids = {
            let mut statement = conn
                .prepare("SELECT id FROM topics WHERE subject_id = ?1 ORDER BY id ASC LIMIT 2")
                .expect("statement should prepare");
            let rows = statement
                .query_map([subject_id], |row| row.get::<_, i64>(0))
                .expect("topic rows should query");
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row.expect("topic id should map"));
            }
            ids
        };
        for (index, topic_id) in topic_ids.iter().enumerate() {
            conn.execute(
                "INSERT INTO student_topic_states (
                    student_id, topic_id, mastery_score, gap_score, fragility_score, priority_score
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    1,
                    topic_id,
                    3000 + (index as i64 * 1500),
                    8000 - (index as i64 * 1000),
                    6000 - (index as i64 * 500),
                    9000 - (index as i64 * 1000),
                ],
            )
            .expect("topic state should insert");
        }

        let service = JourneyService::new(&conn);
        let snapshot = service
            .build_or_refresh_route(1, subject_id, Some("BECE"))
            .expect("route should build");
        assert_eq!(snapshot.route.student_id, 1);
        assert!(!snapshot.stations.is_empty());
        assert_eq!(snapshot.stations[0].status, "active");

        let updated = service
            .complete_station(snapshot.stations[0].id, &json!({ "status": "passed" }))
            .expect("station should complete");

        assert_eq!(updated.stations[0].status, "completed");
    }

    #[test]
    fn journey_service_prioritizes_review_station_when_memory_is_due() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_ids = {
            let mut statement = conn
                .prepare("SELECT id FROM topics WHERE subject_id = ?1 ORDER BY id ASC LIMIT 2")
                .expect("statement should prepare");
            let rows = statement
                .query_map([subject_id], |row| row.get::<_, i64>(0))
                .expect("topic rows should query");
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row.expect("topic id should map"));
            }
            ids
        };
        let priority_topic_id = topic_ids[0];
        let fading_topic_id = topic_ids[1];

        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, priority_score,
                pressure_collapse_index, memory_strength, decay_risk, evidence_count, repair_priority
             ) VALUES (1, ?1, 6400, 4200, 2800, 9800, 1800, 7200, 1800, 4, 2000)",
            [priority_topic_id],
        )
        .expect("priority topic state should insert");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, priority_score,
                pressure_collapse_index, memory_strength, decay_risk, evidence_count, repair_priority
             ) VALUES (1, ?1, 6100, 5100, 4300, 6200, 2200, 2600, 8400, 4, 2500)",
            [fading_topic_id],
        )
        .expect("fading topic state should insert");
        conn.execute(
            "INSERT INTO memory_states (
                student_id, topic_id, memory_state, memory_strength, recall_fluency, decay_risk, review_due_at
             ) VALUES (1, ?1, 'fading', 2400, 1800, 8600, datetime('now', '-1 day'))",
            [fading_topic_id],
        )
        .expect("memory state should insert");

        let service = JourneyService::new(&conn);
        let snapshot = service
            .build_or_refresh_route(1, subject_id, Some("BECE"))
            .expect("route should build");

        assert_eq!(snapshot.stations[0].topic_id, Some(fading_topic_id));
        assert_eq!(snapshot.stations[0].station_type, "review");
        assert_eq!(snapshot.stations[0].status, "active");
    }

    #[test]
    fn journey_station_does_not_advance_when_completion_rule_is_missed() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        seed_student(&conn);
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let subject_id: i64 = conn
            .query_row(
                "SELECT id FROM subjects WHERE code = 'MATH' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("subject should exist");
        let topic_ids = {
            let mut statement = conn
                .prepare("SELECT id FROM topics WHERE subject_id = ?1 ORDER BY id ASC LIMIT 2")
                .expect("statement should prepare");
            let rows = statement
                .query_map([subject_id], |row| row.get::<_, i64>(0))
                .expect("topic rows should query");
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row.expect("topic id should map"));
            }
            ids
        };
        for (index, topic_id) in topic_ids.iter().enumerate() {
            conn.execute(
                "INSERT INTO student_topic_states (
                    student_id, topic_id, mastery_score, gap_score, fragility_score, priority_score
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    1,
                    topic_id,
                    3000 + (index as i64 * 1500),
                    8000 - (index as i64 * 1000),
                    6000 - (index as i64 * 500),
                    9000 - (index as i64 * 1000),
                ],
            )
            .expect("topic state should insert");
        }

        let service = JourneyService::new(&conn);
        let snapshot = service
            .build_or_refresh_route(1, subject_id, Some("BECE"))
            .expect("route should build");
        let updated = service
            .complete_station(
                snapshot.stations[0].id,
                &json!({
                    "accuracy_score": 4200,
                    "answered_questions": 2,
                    "status": "needs_retry"
                }),
            )
            .expect("station should remain active");

        assert_eq!(updated.route.current_station_code.as_deref(), Some("station_01"));
        assert_eq!(updated.stations[0].status, "active");
        assert_eq!(updated.stations[1].status, "locked");
    }

    fn seed_student(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'student', 'Ama', 'hash', 'salt', 'active', 0)",
            [],
        )
        .expect("student should insert");
        conn.execute(
            "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
             VALUES (1, '[\"MATH\"]', 60)",
            [],
        )
        .expect("student profile should insert");
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
