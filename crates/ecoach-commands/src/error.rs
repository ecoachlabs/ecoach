use ecoach_substrate::EcoachError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<EcoachError> for CommandError {
    fn from(value: EcoachError) -> Self {
        let (code, message) = match value {
            EcoachError::Validation(message) => ("validation_error", message),
            EcoachError::Storage(message) => ("storage_error", message),
            EcoachError::NotFound(message) => ("not_found", message),
            EcoachError::Unauthorized(message) => ("unauthorized", message),
            EcoachError::Serialization(message) => ("serialization_error", message),
            EcoachError::Unsupported(message) => ("unsupported", message),
        };
        Self {
            code: code.to_string(),
            message,
        }
    }
}
