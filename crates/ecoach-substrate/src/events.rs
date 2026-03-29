use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub event_id: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub trace_id: String,
}

impl DomainEvent {
    pub fn new(
        event_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        let event_id = uuid::Uuid::new_v4().to_string();
        let trace_id = uuid::Uuid::new_v4().to_string();
        Self {
            event_id,
            event_type: event_type.into(),
            aggregate_id: aggregate_id.into(),
            occurred_at: Utc::now(),
            payload,
            trace_id,
        }
    }
}
