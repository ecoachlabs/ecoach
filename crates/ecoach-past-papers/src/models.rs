use ecoach_substrate::BasisPoints;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperSet {
    pub id: i64,
    pub subject_id: i64,
    pub exam_year: i64,
    pub paper_code: Option<String>,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperSetSummary {
    pub paper_id: i64,
    pub exam_year: i64,
    pub title: String,
    pub question_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastPaperFamilyAnalytics {
    pub family_id: i64,
    pub family_code: String,
    pub family_name: String,
    pub topic_id: Option<i64>,
    pub recurrence_score: BasisPoints,
    pub coappearance_score: BasisPoints,
    pub replacement_score: BasisPoints,
    pub paper_count: i64,
    pub last_seen_year: Option<i64>,
}
