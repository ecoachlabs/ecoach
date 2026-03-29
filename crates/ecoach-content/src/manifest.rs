use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackManifest {
    pub pack_id: String,
    pub pack_version: String,
    pub subject_code: String,
    pub curriculum_version: String,
    pub exam_target: Option<String>,
    pub grade_levels: Vec<String>,
    pub topic_count: i64,
    pub question_count: i64,
    pub min_app_version: Option<String>,
    #[serde(default)]
    pub checksums: BTreeMap<String, String>,
    pub created_at: Option<String>,
    pub author: Option<String>,
}
