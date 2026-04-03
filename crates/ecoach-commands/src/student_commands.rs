use ecoach_student_model::{LearnerTruthSnapshot, StudentModelService, StudentTopicState};
use serde::{Deserialize, Serialize};

use crate::{error::CommandError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerTruthDto {
    pub student_id: i64,
    pub student_name: String,
    pub overall_mastery_score: i64,
    pub overall_readiness_band: String,
    pub pending_review_count: i64,
    pub due_memory_count: i64,
    pub topic_count: usize,
    pub skill_count: usize,
    pub memory_count: usize,
    pub diagnosis_count: usize,
}

impl From<LearnerTruthSnapshot> for LearnerTruthDto {
    fn from(v: LearnerTruthSnapshot) -> Self {
        Self {
            student_id: v.student_id,
            student_name: v.student_name,
            overall_mastery_score: v.overall_mastery_score as i64,
            overall_readiness_band: v.overall_readiness_band,
            pending_review_count: v.pending_review_count,
            due_memory_count: v.due_memory_count,
            topic_count: v.topic_summaries.len(),
            skill_count: v.skill_summaries.len(),
            memory_count: v.memory_summaries.len(),
            diagnosis_count: v.recent_diagnoses.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStateDto {
    pub topic_id: i64,
    pub mastery_score: i64,
    pub mastery_state: String,
    pub accuracy_score: i64,
    pub speed_score: i64,
    pub confidence_score: i64,
    pub retention_score: i64,
    pub gap_score: i64,
    pub priority_score: i64,
    pub trend_state: String,
    pub fragility_score: i64,
    pub pressure_collapse_index: i64,
    pub total_attempts: i64,
    pub correct_attempts: i64,
    pub memory_strength: i64,
}

impl From<StudentTopicState> for TopicStateDto {
    fn from(v: StudentTopicState) -> Self {
        Self {
            topic_id: v.topic_id,
            mastery_score: v.mastery_score as i64,
            mastery_state: v.mastery_state.as_str().to_string(),
            accuracy_score: v.accuracy_score as i64,
            speed_score: v.speed_score as i64,
            confidence_score: v.confidence_score as i64,
            retention_score: v.retention_score as i64,
            gap_score: v.gap_score as i64,
            priority_score: v.priority_score as i64,
            trend_state: v.trend_state,
            fragility_score: v.fragility_score as i64,
            pressure_collapse_index: v.pressure_collapse_index as i64,
            total_attempts: v.total_attempts,
            correct_attempts: v.correct_attempts,
            memory_strength: v.memory_strength as i64,
        }
    }
}

pub fn get_learner_truth(
    state: &AppState,
    student_id: i64,
) -> Result<LearnerTruthDto, CommandError> {
    state.with_connection(|conn| {
        let service = StudentModelService::new(conn);
        let snapshot = service.get_learner_truth_snapshot(student_id)?;
        Ok(LearnerTruthDto::from(snapshot))
    })
}

// ── idea3 instant-gratification features ──

/// Academic MRI / instant scan: run diagnostic and return instant insight summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicScanResult {
    pub student_id: i64,
    pub overall_readiness_band: String,
    pub strongest_topic: Option<String>,
    pub weakest_topic: Option<String>,
    pub top_score_blockers: Vec<String>,
    pub readiness_score: i64,
    pub study_days_last_14: i64,
    pub recommended_next_action: String,
}

pub fn get_academic_scan(
    state: &AppState,
    student_id: i64,
) -> Result<AcademicScanResult, CommandError> {
    state.with_connection(|conn| {
        let truth = StudentModelService::new(conn).get_learner_truth_snapshot(student_id)?;

        let strongest = truth
            .topic_summaries
            .iter()
            .max_by_key(|t| t.mastery_score)
            .map(|t| t.topic_name.clone());
        let weakest = truth
            .topic_summaries
            .iter()
            .min_by_key(|t| t.mastery_score)
            .map(|t| t.topic_name.clone());

        // Top score blockers from recent diagnoses
        let blockers: Vec<String> = truth
            .recent_diagnoses
            .iter()
            .take(3)
            .map(|d| d.primary_diagnosis.clone())
            .collect();

        let recommended = if truth.pending_review_count > 0 {
            "Complete pending reviews before new content".into()
        } else if truth.overall_mastery_score < 5000 {
            "Start with a focused repair session on your weakest topic".into()
        } else {
            "Continue your study path and take a mini mock this week".into()
        };

        // Study consistency
        let study_days: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM study_consistency
                 WHERE student_id = ?1 AND study_date >= date('now', '-14 days')",
                [student_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(AcademicScanResult {
            student_id,
            overall_readiness_band: truth.overall_readiness_band,
            strongest_topic: strongest,
            weakest_topic: weakest,
            top_score_blockers: blockers,
            readiness_score: truth.overall_mastery_score as i64,
            study_days_last_14: study_days,
            recommended_next_action: recommended,
        })
    })
}

/// "What changed this week" summary for parents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyChangeDto {
    pub student_id: i64,
    pub student_name: String,
    pub improvements: Vec<String>,
    pub concerns: Vec<String>,
    pub recommended_actions: Vec<String>,
}

pub fn get_weekly_change_summary(
    state: &AppState,
    student_id: i64,
) -> Result<WeeklyChangeDto, CommandError> {
    state.with_connection(|conn| {
        let truth = StudentModelService::new(conn).get_learner_truth_snapshot(student_id)?;

        let mut improvements = Vec::new();
        let mut concerns = Vec::new();

        for topic in &truth.topic_summaries {
            if topic.mastery_state == "robust" || topic.mastery_state == "exam_ready" {
                improvements.push(format!("{} is improving", topic.topic_name));
            } else if topic.mastery_state == "fragile" || topic.mastery_state == "exposed" {
                concerns.push(format!("{} needs attention", topic.topic_name));
            }
        }

        // Check consistency
        let study_days: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM study_consistency
                 WHERE student_id = ?1 AND study_date >= date('now', '-7 days')",
                [student_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if study_days >= 5 {
            improvements.push("Strong study consistency this week".into());
        } else if study_days <= 2 {
            concerns.push("Study activity has been low this week".into());
        }

        let mut actions = Vec::new();
        if !concerns.is_empty() {
            actions.push("Focus on the areas flagged as needing attention".into());
        }
        if study_days <= 2 {
            actions.push("Encourage at least one short session today".into());
        }
        if improvements.is_empty() && concerns.is_empty() {
            actions.push("Maintain current study pace".into());
        }

        Ok(WeeklyChangeDto {
            student_id,
            student_name: truth.student_name,
            improvements,
            concerns,
            recommended_actions: actions,
        })
    })
}

/// Attention-needed queue for parents: which children need help most.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionNeededItem {
    pub student_id: i64,
    pub student_name: String,
    pub urgency: String,
    pub reason: String,
    pub recommended_action: String,
}

pub fn get_attention_needed_queue(
    state: &AppState,
    parent_id: i64,
) -> Result<Vec<AttentionNeededItem>, CommandError> {
    state.with_connection(|conn| {
        let identity = ecoach_identity::IdentityService::new(conn);
        let students = identity.get_linked_students(parent_id)?;

        let mut items = Vec::new();
        for student in &students {
            // Check for risk flags
            let risk_count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM risk_flags
                     WHERE student_id = ?1 AND status = 'active'",
                    [student.id],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            // Check inactivity
            let days_since_activity: i64 = conn
                .query_row(
                    "SELECT COALESCE(CAST(julianday('now') - julianday(MAX(study_date)) AS INTEGER), 99)
                     FROM study_consistency WHERE student_id = ?1",
                    [student.id],
                    |row| row.get(0),
                )
                .unwrap_or(99);

            if risk_count > 0 {
                items.push(AttentionNeededItem {
                    student_id: student.id,
                    student_name: student.display_name.clone(),
                    urgency: "high".into(),
                    reason: format!("{} active risk flag(s) detected", risk_count),
                    recommended_action: "Review risk details and focus on weak topics".into(),
                });
            } else if days_since_activity >= 5 {
                items.push(AttentionNeededItem {
                    student_id: student.id,
                    student_name: student.display_name.clone(),
                    urgency: "medium".into(),
                    reason: format!("No study activity for {} days", days_since_activity),
                    recommended_action: "Encourage a short return session to rebuild momentum".into(),
                });
            }
        }

        items.sort_by(|a, b| {
            let urgency_order = |u: &str| match u {
                "high" => 0,
                "medium" => 1,
                _ => 2,
            };
            urgency_order(&a.urgency).cmp(&urgency_order(&b.urgency))
        });

        Ok(items)
    })
}
