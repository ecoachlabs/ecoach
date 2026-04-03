use crate::types::BasisPoints;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdRegistry {
    pub mastery_exam_ready: BasisPoints,
    pub mastery_stable: BasisPoints,
    pub readiness_exam_ready: BasisPoints,
    pub readiness_building: BasisPoints,
}

impl Default for ThresholdRegistry {
    fn default() -> Self {
        Self {
            mastery_exam_ready: 9_000,
            mastery_stable: 7_200,
            readiness_exam_ready: 8_500,
            readiness_building: 5_500,
        }
    }
}
