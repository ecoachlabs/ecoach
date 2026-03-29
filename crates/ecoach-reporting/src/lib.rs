pub mod dashboard;
pub mod parent;

pub use dashboard::{DashboardService, StudentDashboard, SubjectSummary};
pub use parent::{
    ParentDashboardSnapshot, ParentInsightService, ParentRiskSummary, ParentStudentSummary,
};
