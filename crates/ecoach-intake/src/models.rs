use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionBundle {
    pub id: i64,
    pub student_id: i64,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleFile {
    pub id: i64,
    pub bundle_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub mime_type: Option<String>,
    pub file_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedInsight {
    pub id: i64,
    pub bundle_id: i64,
    pub insight_type: String,
    pub payload: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleProcessReport {
    pub bundle: SubmissionBundle,
    pub files: Vec<BundleFile>,
    pub insights: Vec<ExtractedInsight>,
    pub detected_subjects: Vec<String>,
    pub detected_exam_years: Vec<i64>,
    pub question_like_file_count: i64,
    pub answer_like_file_count: i64,
    pub ocr_candidate_file_count: i64,
    pub layout_recovered_file_count: i64,
    pub estimated_question_count: i64,
    pub estimated_answer_count: i64,
    pub reconstructed_document_count: i64,
    pub bundle_kind: String,
    pub detected_document_roles: Vec<String>,
    pub review_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAcquisitionJob {
    pub id: i64,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub intent_type: String,
    pub query_text: String,
    pub source_scope: String,
    pub status: String,
    pub result_summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionEvidenceCandidate {
    pub id: i64,
    pub job_id: i64,
    pub source_label: String,
    pub source_url: Option<String>,
    pub source_kind: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub extracted_payload: Value,
    pub quality_score: i64,
    pub freshness_score: i64,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquisitionJobReport {
    pub job: ContentAcquisitionJob,
    pub candidates: Vec<AcquisitionEvidenceCandidate>,
}
