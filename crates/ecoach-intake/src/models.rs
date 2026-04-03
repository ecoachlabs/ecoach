use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachGoalSignal {
    pub signal_key: String,
    pub title: String,
    pub summary: String,
    pub priority: String,
    pub confidence_band: String,
    pub supporting_topics: Vec<String>,
    pub source_document_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicActionSummary {
    pub topic_label: String,
    pub summary: String,
    pub priority: String,
    pub confidence_band: String,
    pub recommended_actions: Vec<String>,
    pub weakness_signals: Vec<String>,
    pub source_document_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpRecommendation {
    pub audience: String,
    pub recommendation_key: String,
    pub summary: String,
    pub priority: String,
    pub confidence_band: String,
    pub topic_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionBundle {
    pub id: i64,
    pub student_id: i64,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInboxItem {
    pub bundle: SubmissionBundle,
    pub confirmation_state: String,
    pub coach_application_status: String,
    pub review_priority: String,
    pub needs_confirmation: bool,
    pub detected_subjects: Vec<String>,
    pub detected_topics: Vec<String>,
    pub summary_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleConfirmationInput {
    pub confirmation_state: String,
    pub note: Option<String>,
    pub topic_overrides: Vec<String>,
    pub document_role_overrides: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleReviewReflectionInput {
    pub question_ref: String,
    pub topic_label: Option<String>,
    pub review_side: String,
    pub reflection_kind: String,
    pub reflection_text: String,
    pub recommended_action: Option<String>,
    pub severity_bp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleReviewNote {
    pub id: i64,
    pub bundle_id: i64,
    pub question_ref: String,
    pub topic_label: Option<String>,
    pub review_side: String,
    pub reflection_kind: String,
    pub reflection_text: String,
    pub recommended_action: Option<String>,
    pub severity_bp: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedReviewItem {
    pub question_ref: String,
    pub topic_label: Option<String>,
    pub alignment_confidence: String,
    pub weakness_signals: Vec<String>,
    pub coach_explanation: String,
    pub recommended_actions: Vec<String>,
    pub reflections: Vec<BundleReviewNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedPaperReviewSnapshot {
    pub bundle: SubmissionBundle,
    pub coach_impact_summary: Value,
    pub parent_summary: Vec<String>,
    pub items: Vec<UploadedReviewItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleCoachApplicationResult {
    pub bundle_id: i64,
    pub coach_application_status: String,
    pub created_goal_ids: Vec<i64>,
    pub updated_topic_labels: Vec<String>,
    pub parent_alert_count: i64,
    pub question_environment_profile: Value,
    pub summary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleOcrPage {
    pub file_id: i64,
    pub file_name: String,
    pub document_role: String,
    pub page_number: i64,
    pub label: String,
    pub confidence_score: i64,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleOcrWorkspace {
    pub bundle: SubmissionBundle,
    pub review_priority: String,
    pub files_with_ocr: i64,
    pub recovered_file_count: i64,
    pub pages: Vec<BundleOcrPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSharedPromotion {
    pub id: i64,
    pub bundle_id: i64,
    pub source_upload_id: Option<i64>,
    pub requested_by_account_id: Option<i64>,
    pub promotion_status: String,
    pub promotion_summary: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAcademicVaultEntry {
    pub bundle: SubmissionBundle,
    pub bundle_kind: String,
    pub review_priority: String,
    pub confirmation_state: String,
    pub coach_application_status: String,
    pub detected_subjects: Vec<String>,
    pub detected_topics: Vec<String>,
    pub summary_points: Vec<String>,
    pub file_count: i64,
    pub files: Vec<BundleFile>,
    pub promotion: Option<BundleSharedPromotion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAcademicVaultSnapshot {
    pub student_id: i64,
    pub total_bundle_count: i64,
    pub pending_review_count: i64,
    pub coach_applied_count: i64,
    pub promoted_bundle_count: i64,
    pub active_topics: Vec<String>,
    pub bundles: Vec<PersonalAcademicVaultEntry>,
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
    pub detected_topics: Vec<String>,
    pub detected_dates: Vec<String>,
    pub question_like_file_count: i64,
    pub answer_like_file_count: i64,
    pub ocr_candidate_file_count: i64,
    pub ocr_recovered_file_count: i64,
    pub layout_recovered_file_count: i64,
    pub estimated_question_count: i64,
    pub estimated_answer_count: i64,
    pub reconstructed_document_count: i64,
    pub paired_assessment_document_count: i64,
    pub reconstruction_confidence_score: i64,
    pub extracted_question_block_count: i64,
    pub aligned_question_pair_count: i64,
    pub high_confidence_alignment_count: i64,
    pub medium_confidence_alignment_count: i64,
    pub low_confidence_alignment_count: i64,
    pub score_signal_count: i64,
    pub remark_signal_count: i64,
    pub needs_confirmation: bool,
    pub unresolved_alignment_count: i64,
    pub review_priority: String,
    pub reconstruction_confidence_band: String,
    pub bundle_kind: String,
    pub detected_document_roles: Vec<String>,
    pub weakness_signals: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub review_reasons: Vec<String>,
    pub coach_goal_signals: Vec<CoachGoalSignal>,
    pub topic_action_summaries: Vec<TopicActionSummary>,
    pub follow_up_recommendations: Vec<FollowUpRecommendation>,
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
