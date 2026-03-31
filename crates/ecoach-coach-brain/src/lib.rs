pub mod evidence_engine;
pub mod goal_engine;
pub mod journey;
pub mod journey_adaptation;
pub mod plan_engine;
pub mod prerequisite_graph;
pub mod readiness_engine;
pub mod rise_mode_engine;
pub mod session_composer;
pub mod state_machine;
pub mod stepped_question_engine;
pub mod topic_action_engine;
pub mod topic_case;
pub mod topic_proof_engine;
pub mod velocity_engine;

pub use evidence_engine::{
    EvidenceEvent, EvidenceInterpretationEngine, LearnerMisconceptionSnapshot, MisconceptionStatus,
};
pub use goal_engine::{GoalEngine, GoalRecommendation, GoalType, UrgencyBand};
pub use stepped_question_engine::{
    SteppedAttemptResult, SteppedQuestionEngine, ThinkingMap,
};
pub use topic_action_engine::{
    TopicActionEngine, TopicActionMode, TopicActionSession, TopicActionSummary, TopicDiagnosis,
};
pub use topic_proof_engine::{ProofTier, TopicProofCertification, TopicProofEngine};
pub use journey::{JourneyRoute, JourneyRouteSnapshot, JourneyService, JourneyStation};
pub use journey_adaptation::{
    AdaptationResult, ConsistencySnapshot, DeadlinePressure, JourneyAdaptationEngine,
    KnowledgeMapNode, MoraleSignal, RouteMode,
};
pub use plan_engine::{CoachMissionMemory, PlanEngine, PlanRewriteResult};
pub use prerequisite_graph::{PrerequisiteGraph, PrerequisiteLink, ReentryProbeResult};
pub use velocity_engine::{CompressionAction, GoalFeasibility, VelocityEngine, VelocitySnapshot};
pub use readiness_engine::{ReadinessEngine, StudentReadinessSnapshot, TopicReadinessSlice};
pub use rise_mode_engine::{
    RiseModeEngine, RiseModeProfile, StageTransitionResult, TransformationStage,
};
pub use session_composer::{ComposedSession, QuestionIntent, SessionComposer, SessionSegment};
pub use state_machine::{
    CoachActionType, CoachNextAction, CoachStateResolution, ContentReadinessResolution,
    ContentReadinessStatus, LearnerJourneyState, assess_content_readiness, resolve_coach_state,
    resolve_next_coach_action,
};
pub use topic_case::{
    TopicCase, TopicCaseBlocker, TopicCaseDiagnosis, TopicCaseHypothesis, TopicCaseIntervention,
    build_topic_case, list_priority_topic_cases,
};
