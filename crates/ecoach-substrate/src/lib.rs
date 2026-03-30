pub mod config;
pub mod engine_registry;
pub mod errors;
pub mod events;
pub mod fabrics;
pub mod scoring;
pub mod time;
pub mod types;

pub use config::ThresholdRegistry;
pub use engine_registry::{EngineContract, EngineRegistry};
pub use errors::{EcoachError, EcoachResult};
pub use events::DomainEvent;
pub use fabrics::{
    FabricConsumerTarget, FabricEvidenceRecord, FabricOrchestrationSummary, FabricSignal,
    LearnerEvidenceFabric,
};
pub use scoring::{clamp_bp, ema_update, from_bp, to_bp};
pub use time::now_utc;
pub use types::{AccountType, BasisPoints, EntitlementTier, Role};
