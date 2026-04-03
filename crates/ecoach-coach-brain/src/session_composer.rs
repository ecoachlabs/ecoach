use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::{CanonicalIntelligenceStore, journey_adaptation::RouteMode};

// ---------------------------------------------------------------------------
// Question intent: why each question is being shown
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionIntent {
    Discovery,
    Coverage,
    Repair,
    Confirmation,
    MisconceptionProbe,
    Reinforcement,
    Transfer,
    Retention,
    Pressure,
    Recovery,
    MiniMock,
    Challenge,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;
    use std::path::PathBuf;

    #[test]
    fn session_composer_prefers_canonical_review_timing_actions() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        seed_student(&conn);

        conn.execute(
            "INSERT INTO ic_timing_decisions (
                decision_id, learner_id, subject_id, topic_id, action_type, action_scope,
                scheduled_for, current_phase, rationale_json, source_engine, consumed,
                owner_engine_key, updated_at
             ) VALUES (
                'timing-review', 1, 1, NULL, 'delayed_recall', 'subject',
                datetime('now'), 'review', '{}', 'timing', 0,
                'timing', datetime('now')
             )",
            [],
        )
        .expect("timing decision should insert");

        let session =
            SessionComposer::new(&conn).compose_session(1, 1, "foundation", RouteMode::Balanced, 40)
                .expect("session should compose");

        assert_eq!(session.route_mode, RouteMode::Balanced);
        assert_eq!(session.segments[0].segment_mode, "spaced_recall");
    }

    #[test]
    fn session_composer_prefers_canonical_risk_and_adaptation_inputs() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        seed_student(&conn);

        conn.execute(
            "INSERT INTO ic_adaptation_log (
                adaptation_id, learner_id, subject_id, topic_id, mode, trigger_reason,
                what_changed_json, previous_strategy_json, new_strategy_json,
                tension_at_time_json, owner_engine_key, created_at, updated_at
             ) VALUES (
                'adaptation-1', 1, 1, NULL, 'reactivation', 'absence_warning',
                '[]', '{}', '{}', '{}', 'adaptation', datetime('now'), datetime('now')
             )",
            [],
        )
        .expect("adaptation log should insert");
        conn.execute(
            "INSERT INTO ic_risk_assessments (
                assessment_id, learner_id, subject_id, topic_id, scope, risk_code,
                risk_level, risk_score, protection_policy_json, rationale_json,
                owner_engine_key, updated_at
             ) VALUES (
                'risk-1', 1, 1, NULL, 'subject', 'coach_readiness',
                'critical', 9000, '{}', '{}', 'risk', datetime('now')
             )",
            [],
        )
        .expect("risk assessment should insert");

        let session =
            SessionComposer::new(&conn).compose_session(1, 1, "foundation", RouteMode::Balanced, 40)
                .expect("session should compose");

        assert_eq!(session.route_mode, RouteMode::Rescue);
        assert_eq!(session.segments[0].segment_mode, "triage");
    }

    fn seed_student(conn: &Connection) {
        conn.execute(
            "INSERT INTO accounts (
                id, account_type, display_name, pin_hash, pin_salt, entitlement_tier
             ) VALUES (1, 'student', 'Ama', 'hash', 'salt', 'standard')",
            [],
        )
        .expect("student account should insert");
        conn.execute(
            "INSERT INTO student_profiles (
                account_id, exam_target, exam_target_date, preferred_subjects
             ) VALUES (1, 'BECE', date('now', '+90 day'), '[\"MTH\"]')",
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

impl QuestionIntent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Discovery => "discovery",
            Self::Coverage => "coverage",
            Self::Repair => "repair",
            Self::Confirmation => "confirmation",
            Self::MisconceptionProbe => "misconception_probe",
            Self::Reinforcement => "reinforcement",
            Self::Transfer => "transfer",
            Self::Retention => "retention",
            Self::Pressure => "pressure",
            Self::Recovery => "recovery",
            Self::MiniMock => "mini_mock",
            Self::Challenge => "challenge",
        }
    }
}

// ---------------------------------------------------------------------------
// Session segment: a piece of a composed session
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSegment {
    pub segment_order: i64,
    pub segment_mode: String,
    pub segment_label: String,
    pub question_count: usize,
    pub duration_minutes: i64,
    pub question_intents: Vec<QuestionIntent>,
}

/// A composed session plan with mixed segments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposedSession {
    pub session_label: String,
    pub total_questions: usize,
    pub total_duration_minutes: i64,
    pub segments: Vec<SessionSegment>,
    pub route_mode: RouteMode,
    pub purpose: String,
}

// ---------------------------------------------------------------------------
// Session composer
// ---------------------------------------------------------------------------

pub struct SessionComposer<'a> {
    conn: &'a Connection,
}

impl<'a> SessionComposer<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Compose a mixed-mode session based on route mode, station type, and learner state.
    pub fn compose_session(
        &self,
        student_id: i64,
        subject_id: i64,
        station_type: &str,
        route_mode: RouteMode,
        daily_budget_minutes: i64,
    ) -> EcoachResult<ComposedSession> {
        let budget = daily_budget_minutes.max(15).min(120);
        let canonical_inputs = CanonicalIntelligenceStore::new(self.conn).resolve_session_inputs(
            student_id,
            subject_id,
            station_type,
            route_mode,
        )?;
        let effective_station_type = canonical_inputs.station_type.as_str();
        let effective_route_mode = canonical_inputs.route_mode;

        let segments = match (effective_route_mode, effective_station_type) {
            // Deep mastery: thorough learning + practice + review
            (RouteMode::DeepMastery, "foundation") => self.deep_foundation_segments(budget),
            (RouteMode::DeepMastery, _) => self.deep_mastery_segments(budget),

            // Balanced: standard mix
            (RouteMode::Balanced, "foundation") => self.balanced_foundation_segments(budget),
            (RouteMode::Balanced, "repair") => self.repair_segments(budget),
            (RouteMode::Balanced, "checkpoint") => self.checkpoint_segments(budget),
            (RouteMode::Balanced, "challenge") => self.challenge_segments(budget),
            (RouteMode::Balanced, "mini_mock") => self.mini_mock_segments(budget),
            (RouteMode::Balanced, "reactivation") => self.reactivation_segments(budget),
            (RouteMode::Balanced, "readiness_gate") => self.readiness_gate_segments(budget),
            (RouteMode::Balanced, "review") => self.review_segments(budget),
            (RouteMode::Balanced, _) => self.balanced_performance_segments(budget),

            // High yield: exam-focused, fast-paced
            (RouteMode::HighYield, _) => self.high_yield_segments(budget),

            // Rescue: triage mode, score movers only
            (RouteMode::Rescue, _) => self.rescue_segments(budget),

            // Reactivation: bring back fading knowledge
            (RouteMode::Reactivation, _) => self.reactivation_segments(budget),
        };

        let total_questions: usize = segments.iter().map(|s| s.question_count).sum();
        let total_duration: i64 = segments.iter().map(|s| s.duration_minutes).sum();

        let purpose = match effective_route_mode {
            RouteMode::DeepMastery => "Build deep understanding with thorough practice",
            RouteMode::Balanced => "Balanced preparation across concepts",
            RouteMode::HighYield => "Focus on highest exam-impact areas",
            RouteMode::Rescue => "Emergency triage for exam readiness",
            RouteMode::Reactivation => "Revive fading knowledge from prior sessions",
        };

        Ok(ComposedSession {
            session_label: format!("{} session", effective_route_mode.as_str()),
            total_questions,
            total_duration_minutes: total_duration,
            segments,
            route_mode: effective_route_mode,
            purpose: purpose.into(),
        })
    }

    /// Record a composed session's segments to the database.
    pub fn persist_composition(
        &self,
        session_id: i64,
        composition: &ComposedSession,
    ) -> EcoachResult<()> {
        for segment in &composition.segments {
            let intent_json =
                serde_json::to_string(&segment.question_intents).unwrap_or_else(|_| "[]".into());

            self.conn
                .execute(
                    "INSERT INTO session_composition
                        (session_id, segment_order, segment_mode, segment_label,
                         question_count, duration_minutes, intent_profile_json)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        session_id,
                        segment.segment_order,
                        segment.segment_mode,
                        segment.segment_label,
                        segment.question_count as i64,
                        segment.duration_minutes,
                        intent_json,
                    ],
                )
                .map_err(|e| EcoachError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    /// Tag a session item with its question intent.
    pub fn tag_question_intent(
        &self,
        session_id: i64,
        question_id: i64,
        intent: QuestionIntent,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "UPDATE session_items SET question_intent = ?1
                 WHERE session_id = ?2 AND question_id = ?3",
                params![intent.as_str(), session_id, question_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Segment templates by mode
    // -----------------------------------------------------------------------

    fn deep_foundation_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "concept_refresh".into(),
                segment_label: "Concept Refresh".into(),
                question_count: 3,
                duration_minutes: (budget * 15 / 100).max(3),
                question_intents: vec![QuestionIntent::Discovery],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "guided_practice".into(),
                segment_label: "Guided Practice".into(),
                question_count: 5,
                duration_minutes: (budget * 35 / 100).max(5),
                question_intents: vec![QuestionIntent::Coverage, QuestionIntent::Repair],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "misconception_check".into(),
                segment_label: "Misconception Check".into(),
                question_count: 3,
                duration_minutes: (budget * 20 / 100).max(3),
                question_intents: vec![QuestionIntent::MisconceptionProbe],
            },
            SessionSegment {
                segment_order: 4,
                segment_mode: "confirmation".into(),
                segment_label: "Confirmation".into(),
                question_count: 4,
                duration_minutes: (budget * 20 / 100).max(3),
                question_intents: vec![QuestionIntent::Confirmation, QuestionIntent::Transfer],
            },
            SessionSegment {
                segment_order: 5,
                segment_mode: "recall_close".into(),
                segment_label: "Recall Close".into(),
                question_count: 2,
                duration_minutes: (budget * 10 / 100).max(2),
                question_intents: vec![QuestionIntent::Retention],
            },
        ]
    }

    fn deep_mastery_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "warm_up".into(),
                segment_label: "Warm Up".into(),
                question_count: 3,
                duration_minutes: (budget * 10 / 100).max(2),
                question_intents: vec![QuestionIntent::Retention, QuestionIntent::Confirmation],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "core_practice".into(),
                segment_label: "Core Practice".into(),
                question_count: 6,
                duration_minutes: (budget * 40 / 100).max(8),
                question_intents: vec![QuestionIntent::Coverage, QuestionIntent::Transfer],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "challenge".into(),
                segment_label: "Challenge".into(),
                question_count: 3,
                duration_minutes: (budget * 25 / 100).max(5),
                question_intents: vec![QuestionIntent::Pressure, QuestionIntent::Challenge],
            },
            SessionSegment {
                segment_order: 4,
                segment_mode: "review".into(),
                segment_label: "Error Review".into(),
                question_count: 3,
                duration_minutes: (budget * 25 / 100).max(5),
                question_intents: vec![QuestionIntent::Repair, QuestionIntent::Reinforcement],
            },
        ]
    }

    fn balanced_foundation_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "learn".into(),
                segment_label: "Learn".into(),
                question_count: 4,
                duration_minutes: (budget * 25 / 100).max(5),
                question_intents: vec![QuestionIntent::Discovery, QuestionIntent::Coverage],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "practice".into(),
                segment_label: "Practice".into(),
                question_count: 5,
                duration_minutes: (budget * 35 / 100).max(7),
                question_intents: vec![QuestionIntent::Coverage, QuestionIntent::Confirmation],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "fix_mistakes".into(),
                segment_label: "Fix Mistakes".into(),
                question_count: 3,
                duration_minutes: (budget * 25 / 100).max(5),
                question_intents: vec![QuestionIntent::Repair, QuestionIntent::MisconceptionProbe],
            },
            SessionSegment {
                segment_order: 4,
                segment_mode: "recall".into(),
                segment_label: "Quick Recall".into(),
                question_count: 3,
                duration_minutes: (budget * 15 / 100).max(3),
                question_intents: vec![QuestionIntent::Retention],
            },
        ]
    }

    fn repair_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "diagnosis".into(),
                segment_label: "Spot the Gap".into(),
                question_count: 3,
                duration_minutes: (budget * 20 / 100).max(3),
                question_intents: vec![
                    QuestionIntent::Discovery,
                    QuestionIntent::MisconceptionProbe,
                ],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "targeted_repair".into(),
                segment_label: "Targeted Repair".into(),
                question_count: 6,
                duration_minutes: (budget * 50 / 100).max(10),
                question_intents: vec![QuestionIntent::Repair, QuestionIntent::Reinforcement],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "confirm_fix".into(),
                segment_label: "Confirm Fix".into(),
                question_count: 4,
                duration_minutes: (budget * 30 / 100).max(5),
                question_intents: vec![QuestionIntent::Confirmation, QuestionIntent::Transfer],
            },
        ]
    }

    fn checkpoint_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "mixed_assessment".into(),
                segment_label: "Checkpoint Assessment".into(),
                question_count: 10,
                duration_minutes: (budget * 70 / 100).max(15),
                question_intents: vec![
                    QuestionIntent::Confirmation,
                    QuestionIntent::Transfer,
                    QuestionIntent::Pressure,
                    QuestionIntent::MisconceptionProbe,
                ],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "review".into(),
                segment_label: "Review Results".into(),
                question_count: 0,
                duration_minutes: (budget * 30 / 100).max(5),
                question_intents: vec![],
            },
        ]
    }

    fn review_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "spaced_recall".into(),
                segment_label: "Memory Recall".into(),
                question_count: 5,
                duration_minutes: (budget * 40 / 100).max(5),
                question_intents: vec![QuestionIntent::Retention, QuestionIntent::Reinforcement],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "transfer_check".into(),
                segment_label: "Transfer Check".into(),
                question_count: 3,
                duration_minutes: (budget * 30 / 100).max(5),
                question_intents: vec![QuestionIntent::Transfer],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "quick_repair".into(),
                segment_label: "Quick Repair".into(),
                question_count: 2,
                duration_minutes: (budget * 30 / 100).max(5),
                question_intents: vec![QuestionIntent::Repair],
            },
        ]
    }

    fn balanced_performance_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "practice".into(),
                segment_label: "Core Practice".into(),
                question_count: 5,
                duration_minutes: (budget * 35 / 100).max(5),
                question_intents: vec![QuestionIntent::Coverage, QuestionIntent::Confirmation],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "timed_drill".into(),
                segment_label: "Timed Drill".into(),
                question_count: 5,
                duration_minutes: (budget * 35 / 100).max(5),
                question_intents: vec![QuestionIntent::Pressure, QuestionIntent::Transfer],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "recall".into(),
                segment_label: "Recall & Review".into(),
                question_count: 3,
                duration_minutes: (budget * 30 / 100).max(5),
                question_intents: vec![QuestionIntent::Retention, QuestionIntent::Repair],
            },
        ]
    }

    fn high_yield_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "formula_sprint".into(),
                segment_label: "Formula Sprint".into(),
                question_count: 4,
                duration_minutes: (budget * 20 / 100).max(3),
                question_intents: vec![QuestionIntent::Retention, QuestionIntent::Confirmation],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "exam_practice".into(),
                segment_label: "Exam-Style Practice".into(),
                question_count: 6,
                duration_minutes: (budget * 45 / 100).max(10),
                question_intents: vec![QuestionIntent::Pressure, QuestionIntent::Coverage],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "weakness_kill".into(),
                segment_label: "Weakness Kill".into(),
                question_count: 4,
                duration_minutes: (budget * 25 / 100).max(5),
                question_intents: vec![QuestionIntent::Repair, QuestionIntent::MisconceptionProbe],
            },
            SessionSegment {
                segment_order: 4,
                segment_mode: "mini_mock".into(),
                segment_label: "Mini Mock".into(),
                question_count: 3,
                duration_minutes: (budget * 10 / 100).max(3),
                question_intents: vec![QuestionIntent::MiniMock],
            },
        ]
    }

    fn rescue_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "triage".into(),
                segment_label: "Triage: Score Movers".into(),
                question_count: 6,
                duration_minutes: (budget * 50 / 100).max(8),
                question_intents: vec![QuestionIntent::Repair, QuestionIntent::Coverage],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "rapid_confirm".into(),
                segment_label: "Rapid Confirm".into(),
                question_count: 4,
                duration_minutes: (budget * 30 / 100).max(5),
                question_intents: vec![QuestionIntent::Confirmation, QuestionIntent::Pressure],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "memory_lock".into(),
                segment_label: "Memory Lock".into(),
                question_count: 3,
                duration_minutes: (budget * 20 / 100).max(3),
                question_intents: vec![QuestionIntent::Retention],
            },
        ]
    }

    fn reactivation_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "recall_probe".into(),
                segment_label: "Recall Probe".into(),
                question_count: 5,
                duration_minutes: (budget * 35 / 100).max(5),
                question_intents: vec![QuestionIntent::Retention, QuestionIntent::Discovery],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "repair_fading".into(),
                segment_label: "Repair Fading".into(),
                question_count: 4,
                duration_minutes: (budget * 35 / 100).max(5),
                question_intents: vec![QuestionIntent::Repair, QuestionIntent::Reinforcement],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "confirm_revival".into(),
                segment_label: "Confirm Revival".into(),
                question_count: 3,
                duration_minutes: (budget * 30 / 100).max(5),
                question_intents: vec![QuestionIntent::Confirmation, QuestionIntent::Transfer],
            },
        ]
    }

    fn challenge_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "pressure_warm_up".into(),
                segment_label: "Pressure Warm-Up".into(),
                question_count: 3,
                duration_minutes: (budget * 15 / 100).max(3),
                question_intents: vec![QuestionIntent::Pressure, QuestionIntent::Confirmation],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "challenge_core".into(),
                segment_label: "Challenge Core".into(),
                question_count: 6,
                duration_minutes: (budget * 40 / 100).max(8),
                question_intents: vec![QuestionIntent::Challenge, QuestionIntent::Transfer],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "high_pressure".into(),
                segment_label: "High Pressure Round".into(),
                question_count: 4,
                duration_minutes: (budget * 30 / 100).max(5),
                question_intents: vec![QuestionIntent::Pressure, QuestionIntent::Challenge],
            },
            SessionSegment {
                segment_order: 4,
                segment_mode: "recovery_review".into(),
                segment_label: "Recovery Review".into(),
                question_count: 2,
                duration_minutes: (budget * 15 / 100).max(3),
                question_intents: vec![QuestionIntent::Recovery, QuestionIntent::Reinforcement],
            },
        ]
    }

    fn mini_mock_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "timed_mixed".into(),
                segment_label: "Timed Mixed Assessment".into(),
                question_count: 8,
                duration_minutes: (budget * 55 / 100).max(10),
                question_intents: vec![
                    QuestionIntent::MiniMock,
                    QuestionIntent::Coverage,
                    QuestionIntent::Pressure,
                    QuestionIntent::Transfer,
                ],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "gap_spotlight".into(),
                segment_label: "Gap Spotlight".into(),
                question_count: 4,
                duration_minutes: (budget * 25 / 100).max(5),
                question_intents: vec![QuestionIntent::MisconceptionProbe, QuestionIntent::Repair],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "score_review".into(),
                segment_label: "Score Review".into(),
                question_count: 0,
                duration_minutes: (budget * 20 / 100).max(3),
                question_intents: vec![],
            },
        ]
    }

    fn readiness_gate_segments(&self, budget: i64) -> Vec<SessionSegment> {
        vec![
            SessionSegment {
                segment_order: 1,
                segment_mode: "recall_check".into(),
                segment_label: "Recall Check".into(),
                question_count: 4,
                duration_minutes: (budget * 20 / 100).max(4),
                question_intents: vec![QuestionIntent::Retention, QuestionIntent::Confirmation],
            },
            SessionSegment {
                segment_order: 2,
                segment_mode: "application".into(),
                segment_label: "Application Round".into(),
                question_count: 5,
                duration_minutes: (budget * 30 / 100).max(6),
                question_intents: vec![QuestionIntent::Transfer, QuestionIntent::Coverage],
            },
            SessionSegment {
                segment_order: 3,
                segment_mode: "pressure_gate".into(),
                segment_label: "Pressure Gate".into(),
                question_count: 4,
                duration_minutes: (budget * 25 / 100).max(5),
                question_intents: vec![QuestionIntent::Pressure, QuestionIntent::Challenge],
            },
            SessionSegment {
                segment_order: 4,
                segment_mode: "transfer_proof".into(),
                segment_label: "Transfer Proof".into(),
                question_count: 3,
                duration_minutes: (budget * 25 / 100).max(5),
                question_intents: vec![
                    QuestionIntent::Transfer,
                    QuestionIntent::MisconceptionProbe,
                ],
            },
        ]
    }
}
