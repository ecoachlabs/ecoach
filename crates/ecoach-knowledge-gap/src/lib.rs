pub mod models;
pub mod service;

pub use models::{
    CreateGapRepairPlanInput, GapDashboard, GapFeedItem, GapRepairFocus, GapRepairPlan,
    GapRepairPlanItem, GapScoreCard, GapSnapshotResult, GapTrendPoint, RepairItemStatus,
    SolidificationProgress, SolidificationSession,
};
pub use service::KnowledgeGapService;
