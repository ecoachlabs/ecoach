use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp, ema_update, to_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::json;

use crate::models::{EliteProfile, EliteSessionBlueprint, EliteSessionScore, EliteTopicProfile};

pub struct EliteService<'a> {
    conn: &'a Connection,
}

impl<'a> EliteService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn upsert_profile(
        &self,
        student_id: i64,
        subject_id: i64,
        eps_score: i64,
        tier: &str,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO elite_profiles (student_id, subject_id, eps_score, tier)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(student_id, subject_id)
                 DO UPDATE SET eps_score = excluded.eps_score, tier = excluded.tier, updated_at = datetime('now')",
                params![student_id, subject_id, eps_score, tier],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn score_session(
        &self,
        student_id: i64,
        session_id: i64,
        session_class: &str,
    ) -> EcoachResult<EliteSessionScore> {
        let session = self
            .load_session(session_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("session {} not found", session_id)))?;
        if session.student_id != student_id {
            return Err(EcoachError::Validation(format!(
                "session {} does not belong to student {}",
                session_id, student_id
            )));
        }
        let subject_id = session.subject_id.ok_or_else(|| {
            EcoachError::Validation(format!("session {} has no subject_id", session_id))
        })?;

        let items = self.load_scored_items(session_id)?;
        if items.is_empty() {
            return Err(EcoachError::Validation(format!(
                "session {} has no scoreable items",
                session_id
            )));
        }

        let answered_count = items.len() as f64;
        let correct_count = items.iter().filter(|item| item.is_correct).count() as f64;
        let accuracy_score = to_bp(correct_count / answered_count);

        let precision_score =
            to_bp(items.iter().map(precision_component).sum::<f64>() / answered_count);
        let speed_score = to_bp(items.iter().map(speed_component).sum::<f64>() / answered_count);
        let depth_score = to_bp(items.iter().map(depth_component).sum::<f64>() / answered_count);
        let trap_resistance_score =
            to_bp(items.iter().map(trap_resistance_component).sum::<f64>() / answered_count);
        let composure_score = compute_composure_score(&items);
        let consistency_score = compute_consistency_score(&items);

        let eps_score = compute_eps_score(
            session_class,
            accuracy_score,
            precision_score,
            speed_score,
            depth_score,
            trap_resistance_score,
            composure_score,
            consistency_score,
        );
        let session_label = elite_label_from_score(eps_score).to_string();
        let debrief_text = elite_debrief_text(
            accuracy_score,
            precision_score,
            speed_score,
            depth_score,
            trap_resistance_score,
            composure_score,
        );
        let recommended_next_session = elite_recommendation(
            precision_score,
            speed_score,
            depth_score,
            trap_resistance_score,
            composure_score,
            consistency_score,
        )
        .to_string();

        let previous_profile = self.get_profile(student_id, subject_id)?;
        let eps_delta = previous_profile
            .as_ref()
            .map(|profile| eps_score as i64 - profile.eps_score as i64)
            .unwrap_or(eps_score as i64);
        let metadata = json!({
            "accuracy_score": accuracy_score,
            "precision_score": precision_score,
            "speed_score": speed_score,
            "depth_score": depth_score,
            "trap_resistance_score": trap_resistance_score,
            "composure_score": composure_score,
            "consistency_score": consistency_score,
            "item_count": items.len(),
            "recommended_next_session": recommended_next_session,
        });

        self.conn
            .execute(
                "INSERT INTO elite_session_records (student_id, subject_id, session_type, eps_delta, metadata_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    student_id,
                    subject_id,
                    session_class,
                    eps_delta,
                    serde_json::to_string(&metadata)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.update_profile_rollup(
            student_id,
            subject_id,
            eps_score,
            precision_score,
            speed_score,
            depth_score,
            composure_score,
        )?;
        self.update_topic_domination(student_id, subject_id, &items)?;

        Ok(EliteSessionScore {
            session_id,
            student_id,
            subject_id,
            session_class: session_class.to_string(),
            accuracy_score,
            precision_score,
            speed_score,
            depth_score,
            trap_resistance_score,
            composure_score,
            consistency_score,
            eps_score,
            session_label,
            debrief_text,
            recommended_next_session,
            metadata,
        })
    }

    pub fn get_profile(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Option<EliteProfile>> {
        self.conn
            .query_row(
                "SELECT student_id, subject_id, eps_score, tier, precision_score, speed_score,
                        depth_score, composure_score
                 FROM elite_profiles
                 WHERE student_id = ?1 AND subject_id = ?2",
                params![student_id, subject_id],
                |row| {
                    Ok(EliteProfile {
                        student_id: row.get(0)?,
                        subject_id: row.get(1)?,
                        eps_score: row.get(2)?,
                        tier: row.get(3)?,
                        precision_score: row.get(4)?,
                        speed_score: row.get(5)?,
                        depth_score: row.get(6)?,
                        composure_score: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn list_topic_domination(
        &self,
        student_id: i64,
        subject_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<EliteTopicProfile>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT etp.topic_id, t.name, etp.precision_score, etp.speed_score, etp.depth_score,
                        etp.composure_score, etp.consistency_score, etp.trap_resistance_score,
                        etp.domination_score, etp.status
                 FROM elite_topic_profiles etp
                 INNER JOIN topics t ON t.id = etp.topic_id
                 WHERE etp.student_id = ?1 AND etp.subject_id = ?2
                 ORDER BY etp.domination_score DESC, t.name ASC
                 LIMIT ?3",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id, limit as i64], |row| {
                Ok(EliteTopicProfile {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    precision_score: row.get(2)?,
                    speed_score: row.get(3)?,
                    depth_score: row.get(4)?,
                    composure_score: row.get(5)?,
                    consistency_score: row.get(6)?,
                    trap_resistance_score: row.get(7)?,
                    domination_score: row.get(8)?,
                    status: row.get(9)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut profiles = Vec::new();
        for row in rows {
            profiles.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(profiles)
    }

    pub fn build_session_blueprint(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<EliteSessionBlueprint> {
        let profile = self.get_profile(student_id, subject_id)?;
        let mut session_class = profile
            .as_ref()
            .map(|profile| {
                elite_recommendation(
                    profile.precision_score,
                    profile.speed_score,
                    profile.depth_score,
                    profile.precision_score.min(profile.depth_score),
                    profile.composure_score,
                    profile.composure_score.min(profile.speed_score),
                )
            })
            .unwrap_or("precision_lab")
            .to_string();

        let mut target_topic_ids = self.load_blueprint_topics(student_id, subject_id)?;
        let trap_signal =
            self.load_trap_blueprint_signal(student_id, subject_id, &target_topic_ids)?;
        if trap_signal.force_trapsense {
            session_class = "trapsense".to_string();
        }
        if let Some(topic_id) = trap_signal.topic_id {
            target_topic_ids.retain(|candidate| *candidate != topic_id);
            target_topic_ids.insert(0, topic_id);
            target_topic_ids.truncate(2);
        }
        let target_family_ids =
            self.load_blueprint_families(subject_id, &target_topic_ids, &session_class)?;
        let authoring_modes = authoring_modes_for_session_class(&session_class, &target_family_ids);
        let target_question_count = match session_class.as_str() {
            "endurance_track" => 16,
            "apex_mock" => 14,
            "depth_lab" => 12,
            _ => 10,
        };
        let rationale = if target_topic_ids.is_empty() {
            format!(
                "{} is the next elite lane because no weak-topic history exists yet; start from the strongest exam-facing families.",
                session_class
            )
        } else if let Some(reason) = trap_signal.rationale {
            format!(
                "{} now targets topics {:?} with families {:?} because {}.",
                session_class, target_topic_ids, target_family_ids, reason
            )
        } else {
            format!(
                "{} targets topics {:?} with families {:?} because those are the lowest-domination or highest-pressure surfaces in the current elite profile.",
                session_class, target_topic_ids, target_family_ids
            )
        };

        Ok(EliteSessionBlueprint {
            student_id,
            subject_id,
            session_class,
            target_topic_ids,
            target_family_ids,
            authoring_modes,
            target_question_count,
            rationale,
        })
    }

    fn update_profile_rollup(
        &self,
        student_id: i64,
        subject_id: i64,
        eps_score: BasisPoints,
        precision_score: BasisPoints,
        speed_score: BasisPoints,
        depth_score: BasisPoints,
        composure_score: BasisPoints,
    ) -> EcoachResult<()> {
        let existing = self.get_profile(student_id, subject_id)?;
        let alpha = 0.25;
        let rolled_eps = existing
            .as_ref()
            .map(|profile| ema_update(profile.eps_score, eps_score, alpha))
            .unwrap_or(eps_score);
        let rolled_precision = existing
            .as_ref()
            .map(|profile| ema_update(profile.precision_score, precision_score, alpha))
            .unwrap_or(precision_score);
        let rolled_speed = existing
            .as_ref()
            .map(|profile| ema_update(profile.speed_score, speed_score, alpha))
            .unwrap_or(speed_score);
        let rolled_depth = existing
            .as_ref()
            .map(|profile| ema_update(profile.depth_score, depth_score, alpha))
            .unwrap_or(depth_score);
        let rolled_composure = existing
            .as_ref()
            .map(|profile| ema_update(profile.composure_score, composure_score, alpha))
            .unwrap_or(composure_score);

        self.conn.execute(
            "INSERT INTO elite_profiles (
                student_id, subject_id, eps_score, tier, precision_score, speed_score, depth_score, composure_score
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(student_id, subject_id) DO UPDATE SET
                eps_score = excluded.eps_score,
                tier = excluded.tier,
                precision_score = excluded.precision_score,
                speed_score = excluded.speed_score,
                depth_score = excluded.depth_score,
                composure_score = excluded.composure_score,
                updated_at = datetime('now')",
            params![
                student_id,
                subject_id,
                rolled_eps,
                elite_tier_from_score(rolled_eps),
                rolled_precision,
                rolled_speed,
                rolled_depth,
                rolled_composure,
            ],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn update_topic_domination(
        &self,
        student_id: i64,
        subject_id: i64,
        items: &[EliteScoredItem],
    ) -> EcoachResult<()> {
        let mut topic_ids = items.iter().map(|item| item.topic_id).collect::<Vec<_>>();
        topic_ids.sort_unstable();
        topic_ids.dedup();

        for topic_id in topic_ids {
            let topic_items = items
                .iter()
                .filter(|item| item.topic_id == topic_id)
                .collect::<Vec<_>>();
            let count = topic_items.len().max(1) as f64;
            let precision_score = to_bp(
                topic_items
                    .iter()
                    .map(|item| precision_component(item))
                    .sum::<f64>()
                    / count,
            );
            let speed_score = to_bp(
                topic_items
                    .iter()
                    .map(|item| speed_component(item))
                    .sum::<f64>()
                    / count,
            );
            let depth_score = to_bp(
                topic_items
                    .iter()
                    .map(|item| depth_component(item))
                    .sum::<f64>()
                    / count,
            );
            let trap_resistance_score = to_bp(
                topic_items
                    .iter()
                    .map(|item| trap_resistance_component(item))
                    .sum::<f64>()
                    / count,
            );
            let composure_score = compute_composure_score_refs(&topic_items);
            let consistency_score = compute_consistency_score_refs(&topic_items);
            let domination_score = clamp_bp(
                (0.35 * precision_score as f64
                    + 0.15 * speed_score as f64
                    + 0.15 * depth_score as f64
                    + 0.10 * trap_resistance_score as f64
                    + 0.15 * composure_score as f64
                    + 0.10 * consistency_score as f64)
                    .round() as i64,
            ) as BasisPoints;

            self.conn.execute(
                "INSERT INTO elite_topic_profiles (
                    student_id, subject_id, topic_id, precision_score, speed_score, depth_score,
                    composure_score, consistency_score, trap_resistance_score, domination_score, status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(student_id, topic_id) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    precision_score = excluded.precision_score,
                    speed_score = excluded.speed_score,
                    depth_score = excluded.depth_score,
                    composure_score = excluded.composure_score,
                    consistency_score = excluded.consistency_score,
                    trap_resistance_score = excluded.trap_resistance_score,
                    domination_score = excluded.domination_score,
                    status = excluded.status,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    subject_id,
                    topic_id,
                    precision_score,
                    speed_score,
                    depth_score,
                    composure_score,
                    consistency_score,
                    trap_resistance_score,
                    domination_score,
                    elite_tier_from_score(domination_score),
                ],
            ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        Ok(())
    }

    fn load_session(&self, session_id: i64) -> EcoachResult<Option<EliteSessionHeader>> {
        self.conn
            .query_row(
                "SELECT id, student_id, subject_id
                 FROM sessions
                 WHERE id = ?1",
                [session_id],
                |row| {
                    Ok(EliteSessionHeader {
                        student_id: row.get(1)?,
                        subject_id: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_scored_items(&self, session_id: i64) -> EcoachResult<Vec<EliteScoredItem>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT
                si.question_id,
                q.topic_id,
                si.display_order,
                q.difficulty_level,
                q.estimated_time_seconds,
                COALESCE(si.response_time_ms, q.estimated_time_seconds * 1000),
                COALESCE(si.is_correct, 0),
                CASE WHEN qo.misconception_id IS NOT NULL THEN 1 ELSE 0 END
             FROM session_items si
             INNER JOIN questions q ON q.id = si.question_id
             LEFT JOIN question_options qo ON qo.id = si.selected_option_id
             WHERE si.session_id = ?1
               AND si.status = 'answered'
             ORDER BY si.display_order ASC, si.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([session_id], |row| {
                Ok(EliteScoredItem {
                    topic_id: row.get(1)?,
                    difficulty_level: row.get(3)?,
                    expected_time_seconds: row.get(4)?,
                    response_time_ms: row.get(5)?,
                    is_correct: row.get::<_, i64>(6)? == 1,
                    misconception_trap_hit: row.get::<_, i64>(7)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }

    fn load_blueprint_topics(&self, student_id: i64, subject_id: i64) -> EcoachResult<Vec<i64>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id
                 FROM elite_topic_profiles
                 WHERE student_id = ?1 AND subject_id = ?2
                 ORDER BY domination_score ASC, precision_score ASC, trap_resistance_score ASC
                 LIMIT 2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| row.get::<_, i64>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topics = Vec::new();
        for row in rows {
            topics.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }

        if topics.is_empty() {
            let mut fallback_statement = self
                .conn
                .prepare(
                    "SELECT sts.topic_id
                     FROM student_topic_states sts
                     INNER JOIN topics t ON t.id = sts.topic_id
                     WHERE sts.student_id = ?1 AND t.subject_id = ?2
                     ORDER BY sts.priority_score DESC, sts.gap_score DESC
                     LIMIT 2",
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            let fallback_rows = fallback_statement
                .query_map(params![student_id, subject_id], |row| row.get::<_, i64>(0))
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            for row in fallback_rows {
                topics.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
            }
        }

        Ok(topics)
    }

    fn load_blueprint_families(
        &self,
        subject_id: i64,
        topic_ids: &[i64],
        session_class: &str,
    ) -> EcoachResult<Vec<i64>> {
        let order_clause = if session_class == "trapsense" {
            "CASE COALESCE(qfh.health_status, 'warming')
                WHEN 'fragile' THEN 0
                WHEN 'warming' THEN 1
                WHEN 'active' THEN 2
                ELSE 3
             END ASC,
             COALESCE(qfa.replacement_score, 0) DESC,
             COALESCE(qfa.recurrence_score, 0) DESC,
             qf.id ASC"
        } else {
            "COALESCE(qfa.replacement_score, 0) DESC,
             COALESCE(qfa.recurrence_score, 0) DESC,
             CASE COALESCE(qfh.health_status, 'warming')
                 WHEN 'fragile' THEN 0
                 WHEN 'warming' THEN 1
                 WHEN 'active' THEN 2
                 ELSE 3
             END ASC,
             qf.id ASC"
        };
        if topic_ids.is_empty() {
            let sql = format!(
                "SELECT qf.id
                     FROM question_families qf
                     LEFT JOIN question_family_analytics qfa ON qfa.family_id = qf.id
                     LEFT JOIN question_family_health qfh ON qfh.family_id = qf.id
                     WHERE qf.subject_id = ?1
                     ORDER BY {}
                     LIMIT 3",
                order_clause
            );
            let mut statement = self
                .conn
                .prepare(&sql)
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            return collect_single_column(&mut statement, params![subject_id]);
        }

        let placeholders = topic_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT qf.id
             FROM question_families qf
             LEFT JOIN question_family_analytics qfa ON qfa.family_id = qf.id
             LEFT JOIN question_family_health qfh ON qfh.family_id = qf.id
             WHERE qf.subject_id = ?1 AND qf.topic_id IN ({})
             ORDER BY {}
             LIMIT 3",
            placeholders, order_clause
        );
        let mut params_vec: Vec<rusqlite::types::Value> = Vec::with_capacity(topic_ids.len() + 1);
        params_vec.push(subject_id.into());
        for topic_id in topic_ids {
            params_vec.push((*topic_id).into());
        }
        let mut statement = self
            .conn
            .prepare(&sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut families = Vec::new();
        for row in rows {
            families.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(families)
    }

    fn load_trap_blueprint_signal(
        &self,
        student_id: i64,
        subject_id: i64,
        topic_ids: &[i64],
    ) -> EcoachResult<TrapBlueprintSignal> {
        if !self.table_exists("student_contrast_states")? || !self.table_exists("contrast_pairs")? {
            return Ok(TrapBlueprintSignal::default());
        }

        let mut sql = "SELECT cp.topic_id, scs.confusion_score, scs.similarity_trap_bp,
                              scs.which_is_which_bp, scs.timed_out_count
                       FROM student_contrast_states scs
                       INNER JOIN contrast_pairs cp ON cp.id = scs.pair_id
                       WHERE scs.student_id = ?1 AND cp.subject_id = ?2"
            .to_string();
        let mut params_vec: Vec<rusqlite::types::Value> =
            vec![student_id.into(), subject_id.into()];
        if !topic_ids.is_empty() {
            let placeholders = topic_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            sql.push_str(&format!(" AND cp.topic_id IN ({placeholders})"));
            for topic_id in topic_ids {
                params_vec.push((*topic_id).into());
            }
        }
        sql.push_str(
            " ORDER BY scs.confusion_score DESC, scs.timed_out_count DESC, cp.topic_id ASC LIMIT 1",
        );

        let signal = self
            .conn
            .query_row(&sql, rusqlite::params_from_iter(params_vec.iter()), |row| {
                Ok(TrapBlueprintSignal {
                    topic_id: row.get(0)?,
                    force_trapsense: row.get::<_, i64>(1)? >= 6800
                        || row.get::<_, i64>(2)? < 5500
                        || row.get::<_, i64>(3)? < 5500
                        || row.get::<_, i64>(4)? >= 2,
                    rationale: Some(format!(
                        "recent trap evidence is still fragile for topic {} with confusion {} bp",
                        row.get::<_, Option<i64>>(0)?.unwrap_or_default(),
                        row.get::<_, i64>(1)?
                    )),
                })
            })
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Ok(signal.unwrap_or_default())
    }

    fn table_exists(&self, table_name: &str) -> EcoachResult<bool> {
        let exists = self
            .conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master
                    WHERE type = 'table' AND name = ?1
                 )",
                [table_name],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(exists == 1)
    }
}

struct EliteSessionHeader {
    student_id: i64,
    subject_id: Option<i64>,
}

#[derive(Default)]
struct TrapBlueprintSignal {
    topic_id: Option<i64>,
    force_trapsense: bool,
    rationale: Option<String>,
}

struct EliteScoredItem {
    topic_id: i64,
    difficulty_level: BasisPoints,
    expected_time_seconds: i64,
    response_time_ms: i64,
    is_correct: bool,
    misconception_trap_hit: bool,
}

fn precision_component(item: &EliteScoredItem) -> f64 {
    let accuracy = if item.is_correct { 1.0 } else { 0.0 };
    let trap_penalty = if item.misconception_trap_hit {
        0.35
    } else {
        0.0
    };
    (0.80 * accuracy + 0.20 * difficulty_factor(item) - trap_penalty).clamp(0.0, 1.0)
}

fn speed_component(item: &EliteScoredItem) -> f64 {
    let expected_ms = (item.expected_time_seconds.max(1) * 1000) as f64;
    let actual_ms = item.response_time_ms.max(1) as f64;
    let pace_ratio = (expected_ms / actual_ms).clamp(0.0, 1.2);
    let base = if item.is_correct {
        pace_ratio.min(1.0)
    } else {
        0.15
    };
    base.clamp(0.0, 1.0)
}

fn depth_component(item: &EliteScoredItem) -> f64 {
    let difficulty = difficulty_factor(item);
    let accuracy = if item.is_correct { 1.0 } else { 0.0 };
    (0.65 * accuracy + 0.35 * difficulty * accuracy).clamp(0.0, 1.0)
}

fn trap_resistance_component(item: &EliteScoredItem) -> f64 {
    if item.misconception_trap_hit {
        0.0
    } else if item.is_correct {
        1.0
    } else {
        0.35
    }
}

fn difficulty_factor(item: &EliteScoredItem) -> f64 {
    (item.difficulty_level as f64 / 10_000.0).clamp(0.0, 1.0)
}

fn compute_composure_score(items: &[EliteScoredItem]) -> BasisPoints {
    compute_composure_score_refs(&items.iter().collect::<Vec<_>>())
}

fn compute_composure_score_refs(items: &[&EliteScoredItem]) -> BasisPoints {
    let split_index = (items.len() / 2).max(1);
    let first_half = &items[..split_index];
    let second_half = &items[split_index..];
    let first_accuracy = average_accuracy(first_half);
    let second_accuracy = average_accuracy(second_half);
    let hold = (second_accuracy / first_accuracy.max(0.2)).clamp(0.0, 1.0);
    to_bp(0.55 * hold + 0.45 * second_accuracy)
}

fn compute_consistency_score(items: &[EliteScoredItem]) -> BasisPoints {
    compute_consistency_score_refs(&items.iter().collect::<Vec<_>>())
}

fn compute_consistency_score_refs(items: &[&EliteScoredItem]) -> BasisPoints {
    if items.is_empty() {
        return 0;
    }
    let window_size = (items.len() / 3).max(1);
    let mut accuracies = Vec::new();
    for chunk in items.chunks(window_size) {
        accuracies.push(average_accuracy(chunk));
    }
    let mean = accuracies.iter().sum::<f64>() / accuracies.len() as f64;
    let variance = accuracies
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / accuracies.len() as f64;
    let stability = (1.0 - variance.sqrt()).clamp(0.0, 1.0);
    to_bp(stability)
}

fn average_accuracy(items: &[&EliteScoredItem]) -> f64 {
    if items.is_empty() {
        return 0.0;
    }
    items.iter().filter(|item| item.is_correct).count() as f64 / items.len() as f64
}

fn compute_eps_score(
    session_class: &str,
    accuracy_score: BasisPoints,
    precision_score: BasisPoints,
    speed_score: BasisPoints,
    depth_score: BasisPoints,
    trap_resistance_score: BasisPoints,
    composure_score: BasisPoints,
    consistency_score: BasisPoints,
) -> BasisPoints {
    let weights = match session_class {
        "precision_lab" => (0.16, 0.32, 0.12, 0.08, 0.17, 0.08, 0.07),
        "elite_sprint" => (0.18, 0.10, 0.28, 0.06, 0.12, 0.16, 0.10),
        "depth_lab" => (0.18, 0.10, 0.08, 0.28, 0.10, 0.14, 0.12),
        "endurance_track" => (0.20, 0.12, 0.10, 0.10, 0.08, 0.16, 0.24),
        "perfect_run" => (0.22, 0.18, 0.12, 0.10, 0.12, 0.14, 0.12),
        "apex_mock" => (0.20, 0.16, 0.15, 0.10, 0.10, 0.15, 0.14),
        _ => (0.30, 0.20, 0.15, 0.10, 0.05, 0.10, 0.10),
    };

    clamp_bp(
        (weights.0 * accuracy_score as f64
            + weights.1 * precision_score as f64
            + weights.2 * speed_score as f64
            + weights.3 * depth_score as f64
            + weights.4 * trap_resistance_score as f64
            + weights.5 * composure_score as f64
            + weights.6 * consistency_score as f64)
            .round() as i64,
    ) as BasisPoints
}

fn elite_label_from_score(score: BasisPoints) -> &'static str {
    match score {
        0..=5499 => "building",
        5500..=6999 => "core",
        7000..=8499 => "apex",
        _ => "legend_run",
    }
}

fn elite_tier_from_score(score: BasisPoints) -> &'static str {
    match score {
        0..=5999 => "foundation",
        6000..=7499 => "core",
        7500..=8999 => "apex",
        _ => "legend",
    }
}

fn elite_debrief_text(
    accuracy_score: BasisPoints,
    precision_score: BasisPoints,
    speed_score: BasisPoints,
    depth_score: BasisPoints,
    trap_resistance_score: BasisPoints,
    composure_score: BasisPoints,
) -> String {
    if precision_score + 1500 < accuracy_score {
        "Knowledge is ahead of discipline right now. Precision loss is still leaking marks."
            .to_string()
    } else if speed_score + 2000 < accuracy_score {
        "Your knowledge is ahead of your timing. Speed is now the limiting factor.".to_string()
    } else if trap_resistance_score < 6000 {
        "Trap vulnerability is still costing you under close distractors and near-miss options."
            .to_string()
    } else if composure_score < 6500 {
        "Your late-session control dipped. Composure needs work so strong starts still finish cleanly.".to_string()
    } else if depth_score < 6500 {
        "You handled direct work well, but reasoning density is still below your best level."
            .to_string()
    } else {
        "Strong elite control. The session stayed clean across precision, pace, and pressure."
            .to_string()
    }
}

fn elite_recommendation(
    precision_score: BasisPoints,
    speed_score: BasisPoints,
    depth_score: BasisPoints,
    trap_resistance_score: BasisPoints,
    composure_score: BasisPoints,
    consistency_score: BasisPoints,
) -> &'static str {
    let mut pairs = [
        ("precision_lab", precision_score),
        ("elite_sprint", speed_score),
        ("depth_lab", depth_score),
        ("trapsense", trap_resistance_score),
        ("endurance_track", consistency_score.min(composure_score)),
    ];
    pairs.sort_by_key(|(_, score)| *score);
    pairs[0].0
}

fn authoring_modes_for_session_class(
    session_class: &str,
    target_family_ids: &[i64],
) -> Vec<String> {
    let mut modes = match session_class {
        "precision_lab" => vec!["rescue", "misconception_probe"],
        "elite_sprint" => vec!["isomorphic", "representation_shift"],
        "depth_lab" => vec!["stretch", "representation_shift"],
        "endurance_track" => vec!["isomorphic", "stretch"],
        "trapsense" => vec!["misconception_probe", "representation_shift"],
        "apex_mock" => vec!["representation_shift", "stretch"],
        _ => vec!["rescue", "isomorphic"],
    }
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    if !target_family_ids.is_empty() && !modes.iter().any(|mode| mode == "misconception_probe") {
        modes.push("misconception_probe".to_string());
    }
    modes
}

fn collect_single_column<P>(
    statement: &mut rusqlite::Statement<'_>,
    params: P,
) -> EcoachResult<Vec<i64>>
where
    P: rusqlite::Params,
{
    let rows = statement
        .query_map(params, |row| row.get::<_, i64>(0))
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let mut values = Vec::new();
    for row in rows {
        values.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn blueprint_uses_low_domination_topics_and_high_pressure_families() {
        let conn = Connection::open_in_memory().expect("in-memory db should open");
        seed_schema(&conn);
        seed_blueprint_fixture(&conn);

        let service = EliteService::new(&conn);
        let blueprint = service
            .build_session_blueprint(7, 1)
            .expect("blueprint should build");

        assert_eq!(blueprint.session_class, "trapsense");
        assert_eq!(blueprint.target_topic_ids.first().copied(), Some(100));
        assert_eq!(blueprint.target_family_ids.first().copied(), Some(900));
        assert!(
            blueprint
                .authoring_modes
                .iter()
                .any(|mode| mode == "misconception_probe")
        );
    }

    fn seed_schema(conn: &Connection) {
        for sql in [
            "CREATE TABLE elite_profiles (
                student_id INTEGER NOT NULL,
                subject_id INTEGER NOT NULL,
                eps_score INTEGER NOT NULL,
                tier TEXT NOT NULL,
                precision_score INTEGER NOT NULL,
                speed_score INTEGER NOT NULL,
                depth_score INTEGER NOT NULL,
                composure_score INTEGER NOT NULL,
                updated_at TEXT,
                UNIQUE(student_id, subject_id)
            )",
            "CREATE TABLE elite_topic_profiles (
                student_id INTEGER NOT NULL,
                subject_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                precision_score INTEGER NOT NULL,
                speed_score INTEGER NOT NULL,
                depth_score INTEGER NOT NULL,
                composure_score INTEGER NOT NULL,
                consistency_score INTEGER NOT NULL,
                trap_resistance_score INTEGER NOT NULL,
                domination_score INTEGER NOT NULL,
                status TEXT NOT NULL,
                updated_at TEXT,
                UNIQUE(student_id, topic_id)
            )",
            "CREATE TABLE student_topic_states (
                student_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                priority_score INTEGER NOT NULL,
                gap_score INTEGER NOT NULL
            )",
            "CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                name TEXT NOT NULL
            )",
            "CREATE TABLE question_families (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                topic_id INTEGER,
                family_code TEXT,
                family_name TEXT
            )",
            "CREATE TABLE question_family_analytics (
                family_id INTEGER NOT NULL,
                recurrence_score INTEGER NOT NULL,
                coappearance_score INTEGER NOT NULL,
                replacement_score INTEGER NOT NULL
            )",
            "CREATE TABLE question_family_health (
                family_id INTEGER NOT NULL,
                health_status TEXT NOT NULL
            )",
            "CREATE TABLE contrast_pairs (
                id INTEGER PRIMARY KEY,
                subject_id INTEGER NOT NULL,
                topic_id INTEGER
            )",
            "CREATE TABLE student_contrast_states (
                student_id INTEGER NOT NULL,
                pair_id INTEGER NOT NULL,
                confusion_score INTEGER NOT NULL,
                similarity_trap_bp INTEGER NOT NULL,
                which_is_which_bp INTEGER NOT NULL,
                timed_out_count INTEGER NOT NULL
            )",
        ] {
            conn.execute(sql, []).expect("schema statement should run");
        }
    }

    fn seed_blueprint_fixture(conn: &Connection) {
        conn.execute(
            "INSERT INTO elite_profiles (
                student_id, subject_id, eps_score, tier, precision_score, speed_score, depth_score, composure_score
             ) VALUES (7, 1, 7100, 'apex', 4200, 7100, 7600, 7300)",
            [],
        )
        .expect("elite profile should insert");

        for (topic_id, domination_score) in [(100_i64, 4100_i64), (200, 6800)] {
            conn.execute(
                "INSERT INTO topics (id, subject_id, name) VALUES (?1, 1, 'Topic')",
                params![topic_id],
            )
            .expect("topic should insert");
            conn.execute(
                "INSERT INTO elite_topic_profiles (
                    student_id, subject_id, topic_id, precision_score, speed_score, depth_score,
                    composure_score, consistency_score, trap_resistance_score, domination_score, status
                 ) VALUES (7, 1, ?1, 4200, 6500, 7000, 7200, 6900, 6100, ?2, 'core')",
                params![topic_id, domination_score],
            )
            .expect("elite topic profile should insert");
        }

        conn.execute(
            "INSERT INTO question_families (id, subject_id, topic_id, family_code, family_name)
             VALUES (900, 1, 100, 'ALG_TRAP', 'Algebra Trap')",
            [],
        )
        .expect("first family should insert");
        conn.execute(
            "INSERT INTO question_families (id, subject_id, topic_id, family_code, family_name)
             VALUES (901, 1, 200, 'ALG_SPEED', 'Algebra Speed')",
            [],
        )
        .expect("second family should insert");
        conn.execute(
            "INSERT INTO question_family_analytics (family_id, recurrence_score, coappearance_score, replacement_score)
             VALUES (900, 7800, 6400, 9300)",
            [],
        )
        .expect("first analytics should insert");
        conn.execute(
            "INSERT INTO question_family_analytics (family_id, recurrence_score, coappearance_score, replacement_score)
             VALUES (901, 8200, 5200, 4100)",
            [],
        )
        .expect("second analytics should insert");
        conn.execute(
            "INSERT INTO question_family_health (family_id, health_status) VALUES (900, 'fragile')",
            [],
        )
        .expect("first health should insert");
        conn.execute(
            "INSERT INTO question_family_health (family_id, health_status) VALUES (901, 'active')",
            [],
        )
        .expect("second health should insert");
        conn.execute(
            "INSERT INTO contrast_pairs (id, subject_id, topic_id) VALUES (300, 1, 100)",
            [],
        )
        .expect("contrast pair should insert");
        conn.execute(
            "INSERT INTO student_contrast_states (
                student_id, pair_id, confusion_score, similarity_trap_bp, which_is_which_bp, timed_out_count
             ) VALUES (7, 300, 7900, 4200, 5100, 2)",
            [],
        )
        .expect("contrast state should insert");
    }
}
