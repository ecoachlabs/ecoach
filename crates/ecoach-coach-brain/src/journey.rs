use ecoach_substrate::{BasisPoints, DomainEvent, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::readiness_engine::ReadinessEngine;

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
        let topic_candidates = self.load_topic_candidates(student_id, subject_id)?;
        if topic_candidates.is_empty() {
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

        let route_type = if readiness.blocked_topic_count > 0 {
            "repair_route"
        } else if readiness.readiness_score >= 7_800 {
            "exam_route"
        } else {
            "mastery_route"
        };
        let route_summary_json = serde_json::to_string(&json!({
            "generated_from": "journey_service",
            "readiness_score": readiness.readiness_score,
            "readiness_band": readiness.readiness_band,
            "recommended_mock_blueprint": readiness.recommended_mock_blueprint,
            "topic_count": topic_candidates.len(),
        }))
        .map_err(|err| EcoachError::Serialization(err.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO journey_routes (
                    student_id, subject_id, target_exam, route_type, status, route_summary_json
                 ) VALUES (?1, ?2, ?3, ?4, 'active', ?5)",
                params![student_id, subject_id, target_exam, route_type, route_summary_json],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let route_id = self.conn.last_insert_rowid();

        for (index, candidate) in topic_candidates.iter().enumerate() {
            let station_code = format!("station_{:02}", index + 1);
            let station_type = station_type_for_candidate(
                candidate.mastery_score,
                candidate.gap_score,
                candidate.fragility_score,
            );
            let status = if index == 0 { "active" } else { "locked" };
            let entry_rule = if index == 0 {
                json!({ "entry": "start_immediately" })
            } else {
                json!({ "requires_previous_station": format!("station_{:02}", index) })
            };
            let completion_rule = json!({
                "target_mastery_score": clamp_bp(candidate.mastery_score as i64 + 1500),
                "target_accuracy_score": clamp_bp(6500 + (index as i64 * 300)),
                "min_questions": if station_type == "repair" { 6 } else { 4 },
            });
            self.conn
                .execute(
                    "INSERT INTO journey_stations (
                        route_id, station_code, title, topic_id, sequence_no, station_type,
                        target_mastery_score, target_accuracy_score, target_readiness_score,
                        status, entry_rule_json, completion_rule_json, evidence_json, unlocked_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, '{}',
                               CASE WHEN ?10 = 'active' THEN datetime('now') ELSE NULL END)",
                    params![
                        route_id,
                        station_code,
                        format!("{} Station", candidate.topic_name),
                        candidate.topic_id,
                        (index + 1) as i64,
                        station_type,
                        clamp_bp(candidate.mastery_score as i64 + 1500),
                        clamp_bp(6500 + (index as i64 * 300)),
                        readiness.readiness_score,
                        status,
                        serde_json::to_string(&entry_rule)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        serde_json::to_string(&completion_rule)
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
                "station_count": topic_candidates.len(),
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
        let (route_id, sequence_no, station_code): (i64, i64, String) = self
            .conn
            .query_row(
                "SELECT route_id, sequence_no, station_code FROM journey_stations WHERE id = ?1",
                [station_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let evidence_json = serde_json::to_string(evidence)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
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
                    priority_score: row.get(5)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
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
    priority_score: BasisPoints,
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

fn map_route(row: &rusqlite::Row<'_>) -> rusqlite::Result<JourneyRoute> {
    let route_summary_json: String = row.get(7)?;
    let route_summary = serde_json::from_str::<Value>(&route_summary_json).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            7,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
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
