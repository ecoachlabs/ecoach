use chrono::Utc;
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::state_machine::{ContentReadinessStatus, assess_content_readiness};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicReadinessSlice {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_score: BasisPoints,
    pub gap_score: BasisPoints,
    pub fragility_score: BasisPoints,
    pub memory_strength: BasisPoints,
    pub question_count: i64,
    pub blocked: bool,
    pub topic_readiness_score: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentReadinessSnapshot {
    pub student_id: i64,
    pub subject_id: i64,
    pub readiness_score: BasisPoints,
    pub readiness_band: String,
    pub content_status: ContentReadinessStatus,
    pub blocked_topic_count: i64,
    pub due_review_count: i64,
    pub due_memory_count: i64,
    pub weak_topic_count: i64,
    pub topic_coverage_ratio: BasisPoints,
    pub recommended_mock_blueprint: String,
    pub plan_rewrite_needed: bool,
    pub topic_slices: Vec<TopicReadinessSlice>,
}

pub struct ReadinessEngine<'a> {
    conn: &'a Connection,
}

impl<'a> ReadinessEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn build_subject_readiness(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<StudentReadinessSnapshot> {
        let content = assess_content_readiness(self.conn, student_id)?;
        let topic_slices = self.load_topic_slices(student_id, subject_id)?;
        let blocked_topic_count = topic_slices.iter().filter(|item| item.blocked).count() as i64;
        let weak_topic_count = topic_slices
            .iter()
            .filter(|item| item.topic_readiness_score < 5_500)
            .count() as i64;
        let due_review_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM coach_mission_memories cmm
                 WHERE cmm.student_id = ?1
                   AND cmm.review_status = 'pending'
                   AND (
                        cmm.subject_id = ?2
                        OR cmm.topic_id IN (SELECT id FROM topics WHERE subject_id = ?2)
                   )",
                params![student_id, subject_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let due_memory_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM memory_states ms
                 WHERE ms.student_id = ?1
                   AND ms.review_due_at IS NOT NULL
                   AND ms.review_due_at <= ?2
                   AND ms.topic_id IN (SELECT id FROM topics WHERE subject_id = ?3)",
                params![student_id, Utc::now().to_rfc3339(), subject_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let total_topics = topic_slices.len() as i64;
        let topics_with_questions = topic_slices
            .iter()
            .filter(|item| item.question_count > 0)
            .count() as i64;
        let topic_coverage_ratio = if total_topics > 0 {
            clamp_bp((topics_with_questions * 10_000) / total_topics)
        } else {
            0
        };

        let base_readiness = if topic_slices.is_empty() {
            0
        } else {
            clamp_bp(
                topic_slices
                    .iter()
                    .map(|item| item.topic_readiness_score as i64)
                    .sum::<i64>()
                    / topic_slices.len() as i64,
            )
        };
        let penalty = blocked_topic_count * 700
            + due_review_count * 350
            + due_memory_count * 250
            + weak_topic_count * 200;
        let readiness_score = clamp_bp(base_readiness as i64 - penalty);
        let readiness_band = readiness_band(readiness_score).to_string();
        let recommended_mock_blueprint = recommend_mock_blueprint(
            readiness_score,
            blocked_topic_count,
            due_review_count,
            due_memory_count,
            weak_topic_count,
            topic_coverage_ratio,
        )
        .to_string();
        let plan_rewrite_needed = blocked_topic_count > 0
            || due_review_count >= 2
            || due_memory_count >= 3
            || readiness_score < 4_800;

        Ok(StudentReadinessSnapshot {
            student_id,
            subject_id,
            readiness_score,
            readiness_band,
            content_status: content.status,
            blocked_topic_count,
            due_review_count,
            due_memory_count,
            weak_topic_count,
            topic_coverage_ratio,
            recommended_mock_blueprint,
            plan_rewrite_needed,
            topic_slices,
        })
    }

    fn load_topic_slices(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<TopicReadinessSlice>> {
        let mut statement = self
            .conn
            .prepare(
                "WITH question_counts AS (
                    SELECT topic_id, COUNT(*) AS question_count
                    FROM questions
                    WHERE is_active = 1
                    GROUP BY topic_id
                 ),
                 active_blockers AS (
                    SELECT DISTINCT topic_id
                    FROM coach_blockers
                    WHERE student_id = ?1
                      AND resolved_at IS NULL
                 )
                 SELECT t.id, t.name,
                        COALESCE(sts.mastery_score, 0),
                        COALESCE(sts.gap_score, 10000),
                        COALESCE(sts.fragility_score, 0),
                        COALESCE(sts.memory_strength, 0),
                        COALESCE(qc.question_count, 0),
                        ab.topic_id IS NOT NULL
                 FROM topics t
                 LEFT JOIN student_topic_states sts
                    ON sts.topic_id = t.id
                   AND sts.student_id = ?1
                 LEFT JOIN question_counts qc
                    ON qc.topic_id = t.id
                 LEFT JOIN active_blockers ab
                    ON ab.topic_id = t.id
                 WHERE t.subject_id = ?2
                 ORDER BY COALESCE(sts.priority_score, 0) DESC, COALESCE(sts.gap_score, 10000) DESC, t.display_order ASC, t.id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, subject_id], |row| {
                let mastery_score: BasisPoints = row.get(2)?;
                let gap_score: BasisPoints = row.get(3)?;
                let fragility_score: BasisPoints = row.get(4)?;
                let memory_strength: BasisPoints = row.get(5)?;
                let question_count: i64 = row.get(6)?;
                let blocked = row.get::<_, i64>(7)? == 1;
                let topic_readiness_score = compute_topic_readiness(
                    mastery_score,
                    gap_score,
                    fragility_score,
                    memory_strength,
                    question_count,
                    blocked,
                );
                Ok(TopicReadinessSlice {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    mastery_score,
                    gap_score,
                    fragility_score,
                    memory_strength,
                    question_count,
                    blocked,
                    topic_readiness_score,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut items = Vec::new();
        for row in rows {
            items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(items)
    }
}

fn compute_topic_readiness(
    mastery_score: BasisPoints,
    gap_score: BasisPoints,
    fragility_score: BasisPoints,
    memory_strength: BasisPoints,
    question_count: i64,
    blocked: bool,
) -> BasisPoints {
    let content_bonus = if question_count > 0 { 900 } else { -1500 };
    let blocked_penalty = if blocked { 1400 } else { 0 };
    clamp_bp(
        (mastery_score as i64 * 45 / 100)
            + ((10_000 - gap_score as i64) * 20 / 100)
            + ((10_000 - fragility_score as i64) * 15 / 100)
            + (memory_strength as i64 * 20 / 100)
            + content_bonus
            - blocked_penalty,
    )
}

fn readiness_band(score: BasisPoints) -> &'static str {
    match score {
        0..=2999 => "critical",
        3000..=4999 => "fragile",
        5000..=6799 => "developing",
        6800..=8199 => "progressing",
        _ => "ready",
    }
}

fn recommend_mock_blueprint(
    readiness_score: BasisPoints,
    blocked_topic_count: i64,
    due_review_count: i64,
    due_memory_count: i64,
    weak_topic_count: i64,
    topic_coverage_ratio: BasisPoints,
) -> &'static str {
    if blocked_topic_count > 0 || weak_topic_count >= 3 {
        "repair_mock"
    } else if due_review_count > 0 || due_memory_count > 0 {
        "recovery_mock"
    } else if topic_coverage_ratio < 6_000 {
        "coverage_mock"
    } else if readiness_score >= 7_800 {
        "pressure_mock"
    } else {
        "balanced_mock"
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
    fn readiness_engine_builds_subject_snapshot() {
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
        let topic_id: i64 = conn
            .query_row("SELECT id FROM topics ORDER BY id ASC LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("topic should exist");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, gap_score, fragility_score, memory_strength, priority_score
             ) VALUES (?1, ?2, 5200, 6000, 2800, 5600, 9000)",
            params![1, topic_id],
        )
        .expect("topic state should insert");

        let snapshot = ReadinessEngine::new(&conn)
            .build_subject_readiness(1, subject_id)
            .expect("snapshot should build");

        assert_eq!(snapshot.student_id, 1);
        assert_eq!(snapshot.subject_id, subject_id);
        assert!(!snapshot.topic_slices.is_empty());
        assert!(snapshot.topic_coverage_ratio > 0);
        assert!(!snapshot.recommended_mock_blueprint.is_empty());
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
