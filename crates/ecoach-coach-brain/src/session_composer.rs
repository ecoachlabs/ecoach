use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::journey_adaptation::RouteMode;

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

        let segments = match (route_mode, station_type) {
            // Deep mastery: thorough learning + practice + review
            (RouteMode::DeepMastery, "foundation") => self.deep_foundation_segments(budget),
            (RouteMode::DeepMastery, _) => self.deep_mastery_segments(budget),

            // Balanced: standard mix
            (RouteMode::Balanced, "foundation") => self.balanced_foundation_segments(budget),
            (RouteMode::Balanced, "repair") => self.repair_segments(budget),
            (RouteMode::Balanced, "checkpoint") => self.checkpoint_segments(budget),
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

        let purpose = match route_mode {
            RouteMode::DeepMastery => "Build deep understanding with thorough practice",
            RouteMode::Balanced => "Balanced preparation across concepts",
            RouteMode::HighYield => "Focus on highest exam-impact areas",
            RouteMode::Rescue => "Emergency triage for exam readiness",
            RouteMode::Reactivation => "Revive fading knowledge from prior sessions",
        };

        Ok(ComposedSession {
            session_label: format!("{} session", route_mode.as_str()),
            total_questions,
            total_duration_minutes: total_duration,
            segments,
            route_mode,
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
            let intent_json = serde_json::to_string(&segment.question_intents)
                .unwrap_or_else(|_| "[]".into());

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
                question_intents: vec![QuestionIntent::Discovery, QuestionIntent::MisconceptionProbe],
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
}
