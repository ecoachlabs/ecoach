use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::topic_action_engine::TopicActionMode;

/// Goal & Timeline Engine: shapes recommendations based on learner's goal + time remaining.
pub struct GoalEngine<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalType {
    FoundationBuilding,
    WeaknessRepair,
    RevisionRefresh,
    ExamReadiness,
    ScoreImprovement,
    TopPerformance,
    ConfidenceRecovery,
    MasteryAcceleration,
}

impl GoalType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FoundationBuilding => "foundation_building",
            Self::WeaknessRepair => "weakness_repair",
            Self::RevisionRefresh => "revision_refresh",
            Self::ExamReadiness => "exam_readiness",
            Self::ScoreImprovement => "score_improvement",
            Self::TopPerformance => "top_performance",
            Self::ConfidenceRecovery => "confidence_recovery",
            Self::MasteryAcceleration => "mastery_acceleration",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "foundation_building" => Self::FoundationBuilding,
            "weakness_repair" => Self::WeaknessRepair,
            "revision_refresh" => Self::RevisionRefresh,
            "score_improvement" => Self::ScoreImprovement,
            "top_performance" => Self::TopPerformance,
            "confidence_recovery" => Self::ConfidenceRecovery,
            "mastery_acceleration" => Self::MasteryAcceleration,
            _ => Self::ExamReadiness,
        }
    }

    pub fn default_topic_mode(self) -> TopicActionMode {
        match self {
            Self::FoundationBuilding => TopicActionMode::Learn,
            Self::WeaknessRepair => TopicActionMode::Repair,
            Self::RevisionRefresh => TopicActionMode::Revision,
            Self::ConfidenceRecovery => TopicActionMode::Repair,
            Self::ExamReadiness | Self::ScoreImprovement => TopicActionMode::Repair,
            Self::TopPerformance | Self::MasteryAcceleration => TopicActionMode::Expert,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UrgencyBand {
    LongRange,
    Structured,
    Focused,
    Intensive,
    LastMile,
}

impl UrgencyBand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LongRange => "long_range",
            Self::Structured => "structured",
            Self::Focused => "focused",
            Self::Intensive => "intensive",
            Self::LastMile => "last_mile",
        }
    }

    pub fn from_days(days: i64) -> Self {
        match days {
            90.. => Self::LongRange,
            45..=89 => Self::Structured,
            21..=44 => Self::Focused,
            8..=20 => Self::Intensive,
            _ => Self::LastMile,
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::LongRange => "Long-range preparation",
            Self::Structured => "Structured preparation",
            Self::Focused => "Focused preparation",
            Self::Intensive => "Intensive revision",
            Self::LastMile => "Last-mile exam mode",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalRecommendation {
    pub goal_type: String,
    pub urgency_band: String,
    pub urgency_label: String,
    pub days_remaining: i64,
    pub default_topic_mode: String,
    pub recommended_actions: Vec<String>,
    pub focus_subjects: Vec<String>,
    pub session_style: String,
}

impl<'a> GoalEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Save goal and exam date during onboarding.
    pub fn save_goal(
        &self,
        student_id: i64,
        subject_id: i64,
        goal_type: &str,
        target_exam: Option<&str>,
        exam_date: Option<&str>,
        confidence_level: Option<&str>,
    ) -> EcoachResult<i64> {
        let days = exam_date
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .map(|exam| {
                let today = chrono::Utc::now().date_naive();
                (exam - today).num_days().max(0)
            })
            .unwrap_or(90);

        let urgency = UrgencyBand::from_days(days);
        let goal = GoalType::from_str(goal_type);

        // Deactivate previous goals for this subject
        self.conn
            .execute(
                "UPDATE goal_targets SET is_active = 0
                 WHERE student_id = ?1 AND subject_id = ?2 AND is_active = 1",
                params![student_id, subject_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO goal_targets
                    (student_id, subject_id, target_exam, exam_date, goal_type,
                     urgency_band, confidence_level, recommended_default_mode)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    student_id, subject_id, target_exam, exam_date,
                    goal_type, urgency.as_str(), confidence_level,
                    goal.default_topic_mode().as_str(),
                ],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Generate goal-shaped recommendations.
    pub fn get_recommendation(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<GoalRecommendation> {
        let (goal_type_str, exam_date, urgency_str, confidence): (String, Option<String>, String, Option<String>) = self
            .conn
            .query_row(
                "SELECT goal_type, exam_date, urgency_band, confidence_level
                 FROM goal_targets
                 WHERE student_id = ?1 AND subject_id = ?2 AND is_active = 1
                 ORDER BY created_at DESC LIMIT 1",
                params![student_id, subject_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|e| EcoachError::NotFound(format!("no active goal for student {student_id}: {e}")))?;

        let goal = GoalType::from_str(&goal_type_str);
        let urgency = UrgencyBand::from_days(
            exam_date
                .as_ref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
                .map(|exam| {
                    let today = chrono::Utc::now().date_naive();
                    (exam - today).num_days().max(0)
                })
                .unwrap_or(90),
        );
        let days = exam_date
            .as_ref()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .map(|exam| (exam - chrono::Utc::now().date_naive()).num_days().max(0))
            .unwrap_or(90);

        // Load weak subjects
        let weak_subjects = self.load_weak_subjects(student_id)?;

        // Build recommendations based on goal + urgency
        let mut actions = Vec::new();
        let session_style;

        match (goal, urgency) {
            (GoalType::FoundationBuilding, _) | (_, UrgencyBand::LongRange) => {
                actions.push("Start with 'Teach Me This Topic' on your weakest areas".into());
                actions.push("Take your time to build strong understanding".into());
                actions.push("No rush — focus on comprehension over speed".into());
                session_style = "deep_learning";
            }
            (GoalType::WeaknessRepair, _) | (GoalType::ScoreImprovement, _) => {
                actions.push("Use 'Fix My Weakness' on topics costing you the most marks".into());
                actions.push("Focus on the 3 biggest score blockers first".into());
                actions.push("Take a mini mock after each repair cycle".into());
                session_style = "targeted_repair";
            }
            (GoalType::RevisionRefresh, _) => {
                actions.push("Use 'Revise This Topic' to refresh key concepts".into());
                actions.push("Focus on recall and formula review".into());
                actions.push("Quick sessions are better than long ones".into());
                session_style = "quick_revision";
            }
            (GoalType::ConfidenceRecovery, _) => {
                actions.push("Start with a Quick Win session to rebuild momentum".into());
                actions.push("Use Comeback Mode after any difficult result".into());
                actions.push("Focus on visible improvement, not perfect scores".into());
                session_style = "confidence_building";
            }
            (GoalType::TopPerformance, _) | (GoalType::MasteryAcceleration, _) => {
                actions.push("Use 'Make Me An Expert' to push your limits".into());
                actions.push("Challenge yourself with harder questions and speed rounds".into());
                actions.push("Take pressure mocks to simulate exam conditions".into());
                session_style = "elite_challenge";
            }
            (_, UrgencyBand::LastMile) => {
                actions.push("Focus only on high-yield revision and repair".into());
                actions.push("Take one mock per day if possible".into());
                actions.push("Review your top 3 mistake patterns".into());
                session_style = "last_mile";
            }
            (_, UrgencyBand::Intensive) => {
                actions.push("Prioritize weak topics and formula recall".into());
                actions.push("Take a timed mini mock every 2-3 days".into());
                actions.push("Cut low-value exploration — focus on score movers".into());
                session_style = "intensive_revision";
            }
            _ => {
                actions.push("Continue balanced study across your subjects".into());
                actions.push("Mix repair sessions with revision and mini mocks".into());
                session_style = "balanced";
            }
        }

        Ok(GoalRecommendation {
            goal_type: goal_type_str,
            urgency_band: urgency.as_str().into(),
            urgency_label: urgency.display_label().into(),
            days_remaining: days,
            default_topic_mode: goal.default_topic_mode().as_str().into(),
            recommended_actions: actions,
            focus_subjects: weak_subjects,
            session_style: session_style.into(),
        })
    }

    fn load_weak_subjects(&self, student_id: i64) -> EcoachResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.name FROM student_topic_states sts
                 INNER JOIN topics t ON t.id = sts.topic_id
                 INNER JOIN subjects s ON s.id = t.subject_id
                 WHERE sts.student_id = ?1 AND sts.gap_score >= 5000
                 GROUP BY s.id
                 ORDER BY AVG(sts.gap_score) DESC
                 LIMIT 3",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([student_id], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut subjects = Vec::new();
        for row in rows {
            subjects.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(subjects)
    }
}
