pub mod models;
pub mod service;

pub use models::{
    AcquisitionEvidenceCandidate, AcquisitionJobReport, BundleCoachApplicationResult,
    BundleConfirmationInput, BundleFile, BundleInboxItem, BundleOcrPage, BundleOcrWorkspace,
    BundleProcessReport, BundleReviewNote, BundleReviewReflectionInput, BundleSharedPromotion,
    CoachGoalSignal, ContentAcquisitionJob, ExtractedInsight, FollowUpRecommendation,
    PersonalAcademicVaultEntry, PersonalAcademicVaultSnapshot, SubmissionBundle,
    TopicActionSummary, UploadedPaperReviewSnapshot,
};
pub use service::{IntakeService, RecoveredText, RecoveredTextPage, extract_text_from_file};
