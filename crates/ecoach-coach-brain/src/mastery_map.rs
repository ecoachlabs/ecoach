use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::prerequisite_graph::PrerequisiteGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteryMapNode {
    pub student_id: i64,
    pub subject_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_percentage_bp: BasisPoints,
    pub stability_state: String,
    pub is_blocked: bool,
    pub blocked_by_topic_ids: Vec<i64>,
    pub is_high_yield: bool,
    pub exam_risk_bp: BasisPoints,
    pub dependency_count: i64,
    pub dependent_count: i64,
    pub score_impact_bp: BasisPoints,
    pub last_activity_at: Option<String>,
    pub updated_at: String,
}

pub struct MasteryMapService<'a> {
    conn: &'a Connection,
}

impl<'a> MasteryMapService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn refresh_mastery_map(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<MasteryMapNode>> {
        let prerequisite_graph = PrerequisiteGraph::new(self.conn);
        let prerequisite_links = prerequisite_graph.get_topic_prerequisites(subject_id)?;
        let incoming = build_incoming_map(&prerequisite_links);
        let outgoing = build_outgoing_map(&prerequisite_links);
        let state_map = self.load_state_snapshots(student_id, subject_id)?;
        let rows = self.load_topic_rollup_rows(student_id, subject_id)?;

        let mut nodes = Vec::new();
        for row in rows {
            let blocked_by_topic_ids =
                determine_blocked_by_topics(row.topic_id, &incoming, &state_map);
            let dependency_count = incoming
                .get(&row.topic_id)
                .map(|items| items.len() as i64)
                .unwrap_or(0);
            let dependent_count = outgoing
                .get(&row.topic_id)
                .map(|items| items.len() as i64)
                .unwrap_or(0);
            let attempt_accuracy_bp = if row.attempt_count > 0 {
                clamp_bp((row.correct_count * 10_000) / row.attempt_count)
            } else {
                0
            };
            let step_quality_bp = if row.step_marks_possible > 0 {
                clamp_bp((row.step_marks_awarded * 10_000) / row.step_marks_possible)
            } else {
                0
            };
            let mastery_percentage_bp = combine_mastery_signals(
                row.mastery_score,
                row.retention_score,
                row.confidence_score,
                row.speed_score,
                row.consistency_score,
                attempt_accuracy_bp,
                step_quality_bp,
            );
            let stability_state = derive_stability_state(
                mastery_percentage_bp,
                row.gap_score,
                row.fragility_score,
                row.evidence_count,
                row.attempt_count,
                row.step_count,
                !blocked_by_topic_ids.is_empty() || row.is_blocked == 1,
            );
            let is_high_yield = row.exam_weight >= 7000
                || dependent_count >= 2
                || row.question_count >= 8
                || mastery_percentage_bp >= 8000;
            let exam_risk_bp = compute_exam_risk(
                mastery_percentage_bp,
                row.gap_score,
                row.fragility_score,
                dependency_count,
                blocked_by_topic_ids.len() as i64,
            );
            let score_impact_bp = compute_score_impact(
                row.exam_weight,
                row.importance_weight,
                dependent_count,
                row.question_count,
            );
            let last_activity_at = latest_timestamp(
                row.last_seen_at.as_deref(),
                row.last_correct_at.as_deref(),
                row.last_mastered_at.as_deref(),
                row.attempt_last_activity.as_deref(),
                row.step_last_activity.as_deref(),
            );
            let updated_at = chrono::Utc::now().to_rfc3339();
            let blocked_by_json = serde_json::to_string(&blocked_by_topic_ids)
                .map_err(|err| EcoachError::Serialization(err.to_string()))?;

            self.conn
                .execute(
                    "INSERT INTO mastery_map_nodes (
                        student_id, topic_id, subject_id, mastery_percentage_bp,
                        stability_state, is_blocked, blocked_by_json, is_high_yield,
                        exam_risk_bp, dependency_count, dependent_count, score_impact_bp,
                        last_activity_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                     ON CONFLICT(student_id, topic_id) DO UPDATE SET
                        subject_id = ?3,
                        mastery_percentage_bp = ?4,
                        stability_state = ?5,
                        is_blocked = ?6,
                        blocked_by_json = ?7,
                        is_high_yield = ?8,
                        exam_risk_bp = ?9,
                        dependency_count = ?10,
                        dependent_count = ?11,
                        score_impact_bp = ?12,
                        last_activity_at = ?13,
                        updated_at = ?14",
                    params![
                        student_id,
                        row.topic_id,
                        subject_id,
                        mastery_percentage_bp as i64,
                        stability_state,
                        if blocked_by_topic_ids.is_empty() && row.is_blocked == 0 {
                            0
                        } else {
                            1
                        },
                        blocked_by_json,
                        if is_high_yield { 1 } else { 0 },
                        exam_risk_bp as i64,
                        dependency_count,
                        dependent_count,
                        score_impact_bp as i64,
                        last_activity_at,
                        updated_at.clone(),
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            nodes.push(MasteryMapNode {
                student_id,
                subject_id,
                topic_id: row.topic_id,
                topic_name: row.topic_name,
                mastery_percentage_bp,
                stability_state,
                is_blocked: !blocked_by_topic_ids.is_empty() || row.is_blocked == 1,
                blocked_by_topic_ids,
                is_high_yield,
                exam_risk_bp,
                dependency_count,
                dependent_count,
                score_impact_bp,
                last_activity_at,
                updated_at,
            });
        }

        Ok(nodes)
    }

    pub fn get_mastery_map(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<MasteryMapNode>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT mmn.topic_id, t.name, mmn.mastery_percentage_bp, mmn.stability_state,
                        mmn.is_blocked, mmn.blocked_by_json, mmn.is_high_yield, mmn.exam_risk_bp,
                        mmn.dependency_count, mmn.dependent_count, mmn.score_impact_bp,
                        mmn.last_activity_at, mmn.updated_at
                 FROM mastery_map_nodes mmn
                 INNER JOIN topics t ON t.id = mmn.topic_id
                 WHERE mmn.student_id = ?1 AND mmn.subject_id = ?2
                 ORDER BY mmn.score_impact_bp DESC, mmn.exam_risk_bp DESC, mmn.topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                let blocked_by_json: Option<String> = row.get(5)?;
                Ok(MasteryMapNode {
                    student_id,
                    subject_id,
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    mastery_percentage_bp: clamp_bp(row.get::<_, i64>(2)?),
                    stability_state: row.get(3)?,
                    is_blocked: row.get::<_, i64>(4)? == 1,
                    blocked_by_topic_ids: serde_json::from_str(
                        blocked_by_json.as_deref().unwrap_or("[]"),
                    )
                    .unwrap_or_default(),
                    is_high_yield: row.get::<_, i64>(6)? == 1,
                    exam_risk_bp: clamp_bp(row.get::<_, i64>(7)?),
                    dependency_count: row.get(8)?,
                    dependent_count: row.get(9)?,
                    score_impact_bp: clamp_bp(row.get::<_, i64>(10)?),
                    last_activity_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(nodes)
    }

    fn load_topic_rollup_rows(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<TopicRollupRow>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT t.id, t.name, COALESCE(t.exam_weight, 5000), COALESCE(t.importance_weight, 5000),
                        COALESCE(sts.mastery_score, 0), COALESCE(sts.gap_score, 10000),
                        COALESCE(sts.fragility_score, 0), COALESCE(sts.confidence_score, 0),
                        COALESCE(sts.retention_score, 0), COALESCE(sts.speed_score, 0),
                        COALESCE(sts.consistency_score, 0), COALESCE(sts.evidence_count, 0),
                        COALESCE(sts.total_attempts, 0), COALESCE(sts.correct_attempts, 0),
                        COALESCE(sts.is_blocked, 0), COALESCE(sts.is_urgent, 0),
                        COALESCE(sts.is_exam_critical, 0), sts.last_seen_at,
                        sts.last_correct_at, sts.last_mastered_at,
                        COALESCE(qc.question_count, 0),
                        COALESCE(at.attempt_count, 0), COALESCE(at.correct_count, 0),
                        at.last_activity_at,
                        COALESCE(ss.step_count, 0), COALESCE(ss.marks_awarded, 0),
                        COALESCE(ss.marks_possible, 0), ss.last_activity_at
                 FROM topics t
                 LEFT JOIN student_topic_states sts
                    ON sts.topic_id = t.id AND sts.student_id = ?1
                 LEFT JOIN (
                    SELECT topic_id, COUNT(*) AS question_count
                    FROM questions
                    WHERE is_active = 1
                    GROUP BY topic_id
                 ) qc ON qc.topic_id = t.id
                 LEFT JOIN (
                    SELECT q.topic_id, COUNT(*) AS attempt_count,
                           SUM(CASE WHEN a.is_correct = 1 THEN 1 ELSE 0 END) AS correct_count,
                           MAX(COALESCE(a.submitted_at, a.created_at)) AS last_activity_at
                    FROM student_question_attempts a
                    INNER JOIN questions q ON q.id = a.question_id
                    WHERE a.student_id = ?1
                    GROUP BY q.topic_id
                 ) at ON at.topic_id = t.id
                 LEFT JOIN (
                    SELECT q.topic_id, COUNT(*) AS step_count,
                           SUM(ss.marks_awarded) AS marks_awarded,
                           SUM(ss.marks_possible) AS marks_possible,
                           MAX(ss.created_at) AS last_activity_at
                    FROM step_submissions ss
                    INNER JOIN questions q ON q.id = ss.question_id
                    WHERE ss.student_id = ?1
                    GROUP BY q.topic_id
                 ) ss ON ss.topic_id = t.id
                 WHERE t.subject_id = ?2 AND t.is_active = 1
                 ORDER BY t.display_order ASC, t.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                Ok(TopicRollupRow {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    exam_weight: row.get(2)?,
                    importance_weight: row.get(3)?,
                    mastery_score: row.get(4)?,
                    gap_score: row.get(5)?,
                    fragility_score: row.get(6)?,
                    confidence_score: row.get(7)?,
                    retention_score: row.get(8)?,
                    speed_score: row.get(9)?,
                    consistency_score: row.get(10)?,
                    evidence_count: row.get(11)?,
                    state_total_attempts: row.get(12)?,
                    state_correct_attempts: row.get(13)?,
                    is_blocked: row.get(14)?,
                    is_urgent: row.get(15)?,
                    is_exam_critical: row.get(16)?,
                    last_seen_at: row.get(17)?,
                    last_correct_at: row.get(18)?,
                    last_mastered_at: row.get(19)?,
                    question_count: row.get(20)?,
                    attempt_count: row.get(21)?,
                    correct_count: row.get(22)?,
                    attempt_last_activity: row.get(23)?,
                    step_count: row.get(24)?,
                    step_marks_awarded: row.get(25)?,
                    step_marks_possible: row.get(26)?,
                    step_last_activity: row.get(27)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut topics = Vec::new();
        for row in rows {
            topics.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(topics)
    }

    fn load_state_snapshots(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<HashMap<i64, TopicStateSnapshot>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.topic_id, sts.mastery_score, sts.gap_score, sts.fragility_score
                 FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1 AND t.subject_id = ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    TopicStateSnapshot {
                        mastery_score: row.get(1)?,
                        gap_score: row.get(2)?,
                        fragility_score: row.get(3)?,
                    },
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut states = HashMap::new();
        for row in rows {
            let (topic_id, snapshot) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            states.insert(topic_id, snapshot);
        }
        Ok(states)
    }
}

#[derive(Debug, Clone)]
struct TopicRollupRow {
    topic_id: i64,
    topic_name: String,
    exam_weight: i64,
    importance_weight: i64,
    mastery_score: i64,
    gap_score: i64,
    fragility_score: i64,
    confidence_score: i64,
    retention_score: i64,
    speed_score: i64,
    consistency_score: i64,
    evidence_count: i64,
    state_total_attempts: i64,
    state_correct_attempts: i64,
    is_blocked: i64,
    is_urgent: i64,
    is_exam_critical: i64,
    last_seen_at: Option<String>,
    last_correct_at: Option<String>,
    last_mastered_at: Option<String>,
    question_count: i64,
    attempt_count: i64,
    correct_count: i64,
    attempt_last_activity: Option<String>,
    step_count: i64,
    step_marks_awarded: i64,
    step_marks_possible: i64,
    step_last_activity: Option<String>,
}

#[derive(Debug, Clone)]
struct TopicStateSnapshot {
    mastery_score: i64,
    gap_score: i64,
    fragility_score: i64,
}

fn build_incoming_map(
    links: &[crate::prerequisite_graph::PrerequisiteLink],
) -> HashMap<i64, Vec<i64>> {
    let mut incoming: HashMap<i64, Vec<i64>> = HashMap::new();
    for link in links {
        incoming
            .entry(link.to_topic_id)
            .or_default()
            .push(link.from_topic_id);
    }
    incoming
}

fn build_outgoing_map(
    links: &[crate::prerequisite_graph::PrerequisiteLink],
) -> HashMap<i64, Vec<i64>> {
    let mut outgoing: HashMap<i64, Vec<i64>> = HashMap::new();
    for link in links {
        outgoing
            .entry(link.from_topic_id)
            .or_default()
            .push(link.to_topic_id);
    }
    outgoing
}

fn determine_blocked_by_topics(
    topic_id: i64,
    incoming: &HashMap<i64, Vec<i64>>,
    state_map: &HashMap<i64, TopicStateSnapshot>,
) -> Vec<i64> {
    let Some(prereqs) = incoming.get(&topic_id) else {
        return Vec::new();
    };

    let mut blocked = Vec::new();
    for prereq_topic_id in prereqs {
        if let Some(snapshot) = state_map.get(prereq_topic_id) {
            if snapshot.mastery_score < 4500
                || snapshot.gap_score >= 6500
                || snapshot.fragility_score >= 6500
            {
                blocked.push(*prereq_topic_id);
            }
        } else {
            blocked.push(*prereq_topic_id);
        }
    }
    blocked
}

fn combine_mastery_signals(
    mastery_score: i64,
    retention_score: i64,
    confidence_score: i64,
    speed_score: i64,
    consistency_score: i64,
    attempt_accuracy_bp: BasisPoints,
    step_quality_bp: BasisPoints,
) -> BasisPoints {
    clamp_bp(
        mastery_score * 45 / 100
            + retention_score * 15 / 100
            + confidence_score * 10 / 100
            + speed_score * 10 / 100
            + consistency_score * 10 / 100
            + attempt_accuracy_bp as i64 * 5 / 100
            + step_quality_bp as i64 * 5 / 100,
    )
}

fn derive_stability_state(
    mastery_percentage_bp: BasisPoints,
    gap_score: i64,
    fragility_score: i64,
    evidence_count: i64,
    attempt_count: i64,
    step_count: i64,
    blocked: bool,
) -> String {
    if evidence_count == 0 && attempt_count == 0 && step_count == 0 {
        return "unseen".to_string();
    }
    if mastery_percentage_bp < 2000 {
        "started".to_string()
    } else if blocked || gap_score >= 6500 || fragility_score >= 6500 {
        "fragile".to_string()
    } else if mastery_percentage_bp < 5500 {
        "building".to_string()
    } else if mastery_percentage_bp < 7500 {
        "stable".to_string()
    } else if mastery_percentage_bp < 9000 {
        "strong".to_string()
    } else {
        "mastered".to_string()
    }
}

fn compute_exam_risk(
    mastery_percentage_bp: BasisPoints,
    gap_score: i64,
    fragility_score: i64,
    dependency_count: i64,
    blocked_by_count: i64,
) -> BasisPoints {
    clamp_bp(
        (10_000 - mastery_percentage_bp as i64) * 25 / 100
            + gap_score * 30 / 100
            + fragility_score * 25 / 100
            + dependency_count * 180
            + blocked_by_count * 900,
    )
}

fn compute_score_impact(
    exam_weight: i64,
    importance_weight: i64,
    dependent_count: i64,
    question_count: i64,
) -> BasisPoints {
    clamp_bp(
        exam_weight * 35 / 100
            + importance_weight * 25 / 100
            + dependent_count * 900
            + question_count * 60,
    )
}

fn latest_timestamp<'a>(
    last_seen_at: Option<&'a str>,
    last_correct_at: Option<&'a str>,
    last_mastered_at: Option<&'a str>,
    attempt_last_activity: Option<&'a str>,
    step_last_activity: Option<&'a str>,
) -> Option<String> {
    let values = [
        last_seen_at,
        last_correct_at,
        last_mastered_at,
        attempt_last_activity,
        step_last_activity,
    ];
    values
        .into_iter()
        .flatten()
        .max()
        .map(|item| item.to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn mastery_map_refreshes_and_persists_rollups() {
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
                .expect("topics should query");
            let mut ids = Vec::new();
            for row in rows {
                ids.push(row.expect("topic id should map"));
            }
            ids
        };
        let prereq_topic_id = topic_ids[0];
        let dependent_topic_id = topic_ids[1];
        let prereq_node_id = seed_academic_node(&conn, prereq_topic_id, "Prereq node");
        let dependent_node_id = seed_academic_node(&conn, dependent_topic_id, "Dependent node");

        conn.execute(
            "INSERT INTO node_edges (
                from_node_id, from_node_type, to_node_id, to_node_type, edge_type, strength_score
             ) VALUES (?1, 'academic_node', ?2, 'academic_node', 'prerequisite', 8000)",
            params![prereq_node_id, dependent_node_id],
        )
        .expect("prerequisite edge should insert");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, confidence_score,
                retention_score, speed_score, consistency_score, evidence_count, total_attempts,
                correct_attempts, is_blocked, is_urgent
             ) VALUES (1, ?1, 2500, 7600, 6900, 2400, 2600, 2200, 2100, 3, 8, 2, 0, 1)",
            [prereq_topic_id],
        )
        .expect("prereq state should insert");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, confidence_score,
                retention_score, speed_score, consistency_score, evidence_count, total_attempts,
                correct_attempts, is_blocked
             ) VALUES (1, ?1, 7100, 4200, 1900, 6800, 6400, 5100, 5000, 6, 14, 10, 0)",
            [dependent_topic_id],
        )
        .expect("dependent state should insert");
        conn.execute(
            "INSERT INTO student_question_attempts (
                student_id, question_id, started_at, submitted_at, is_correct, created_at
             ) VALUES (1, ?1, datetime('now', '-1 day'), datetime('now', '-1 day'), 1, datetime('now', '-1 day'))",
            [seed_question_for_topic(&conn, dependent_topic_id)],
        )
        .expect("attempt should insert");

        let service = MasteryMapService::new(&conn);
        let nodes = service
            .refresh_mastery_map(1, subject_id)
            .expect("mastery map should refresh");

        assert_eq!(nodes.len(), 2);
        let blocked_node = nodes
            .iter()
            .find(|node| node.topic_id == dependent_topic_id)
            .expect("dependent node should exist");
        assert!(blocked_node.is_blocked);
        assert!(!blocked_node.blocked_by_topic_ids.is_empty());

        let loaded = service
            .get_mastery_map(1, subject_id)
            .expect("mastery map should load");
        assert_eq!(loaded.len(), 2);
    }

    fn seed_student(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
             VALUES (1, 'student', 'Ada', 'hash', 'salt', 'active', 0)",
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

    fn seed_question_for_topic(conn: &Connection, topic_id: i64) -> i64 {
        let subject_id: i64 = conn
            .query_row(
                "SELECT subject_id FROM topics WHERE id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .expect("topic should exist");
        conn.execute(
            "INSERT INTO questions (
                subject_id, topic_id, stem, question_format, marks, is_active
             ) VALUES (?1, ?2, 'Explain the idea', 'short_answer', 1, 1)",
            params![subject_id, topic_id],
        )
        .expect("question should insert");
        conn.last_insert_rowid()
    }

    fn seed_academic_node(conn: &Connection, topic_id: i64, title: &str) -> i64 {
        conn.execute(
            "INSERT INTO academic_nodes (
                topic_id, node_type, canonical_title, short_label, description_formal,
                description_simple, core_meaning, difficulty_band, exam_relevance_score,
                foundation_weight, is_active, metadata_json
             ) VALUES (?1, 'concept', ?2, ?3, ?4, ?5, ?6, 'medium', 6000, 6000, 1, '{}')",
            params![
                topic_id,
                title,
                format!("{} short", title),
                format!("{} formal", title),
                format!("{} simple", title),
                format!("{} meaning", title),
            ],
        )
        .expect("academic node should insert");
        conn.last_insert_rowid()
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
