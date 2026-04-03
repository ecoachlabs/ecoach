pub mod models;
pub mod service;

pub use models::{
    AcquisitionEvidenceCandidate, AcquisitionJobReport, BundleCoachApplicationResult, BundleFile,
    BundleInboxItem, BundleOcrPage, BundleOcrWorkspace, BundleProcessReport,
    BundleReviewNote, BundleReviewReflectionInput, BundleSharedPromotion,
    BundleConfirmationInput, CoachGoalSignal, ContentAcquisitionJob, ExtractedInsight,
    FollowUpRecommendation, PersonalAcademicVaultEntry, PersonalAcademicVaultSnapshot,
    SubmissionBundle, TopicActionSummary, UploadedPaperReviewSnapshot,
};
pub use service::IntakeService;
