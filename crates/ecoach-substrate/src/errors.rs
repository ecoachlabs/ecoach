use thiserror::Error;

pub type EcoachResult<T> = Result<T, EcoachError>;

#[derive(Debug, Error)]
pub enum EcoachError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("unsupported operation: {0}")]
    Unsupported(String),
}
