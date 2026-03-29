use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: i64,
    pub curriculum_version_id: i64,
    pub code: String,
    pub name: String,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    pub id: i64,
    pub subject_id: i64,
    pub parent_topic_id: Option<i64>,
    pub code: Option<String>,
    pub name: String,
    pub node_type: String,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicNode {
    pub id: i64,
    pub topic_id: i64,
    pub node_type: String,
    pub canonical_title: String,
    pub core_meaning: Option<String>,
    pub exam_relevance_score: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSourceUpload {
    pub id: i64,
    pub uploader_account_id: i64,
    pub source_kind: String,
    pub title: String,
    pub source_path: Option<String>,
    pub country_code: Option<String>,
    pub exam_board: Option<String>,
    pub education_level: Option<String>,
    pub subject_code: Option<String>,
    pub academic_year: Option<String>,
    pub language_code: String,
    pub version_label: Option<String>,
    pub source_status: String,
    pub confidence_score: i64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumParseCandidate {
    pub id: i64,
    pub source_upload_id: i64,
    pub candidate_type: String,
    pub parent_candidate_id: Option<i64>,
    pub raw_label: String,
    pub normalized_label: Option<String>,
    pub payload: Value,
    pub confidence_score: i64,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumReviewTask {
    pub id: i64,
    pub source_upload_id: i64,
    pub candidate_id: Option<i64>,
    pub task_type: String,
    pub status: String,
    pub severity: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurriculumSourceReport {
    pub source_upload: CurriculumSourceUpload,
    pub candidates: Vec<CurriculumParseCandidate>,
    pub review_tasks: Vec<CurriculumReviewTask>,
}
