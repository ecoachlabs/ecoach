use crate::{
    CanonicalIntelligenceStore, CoachIntelligenceDomeService, CoachJudgmentEngine,
    CoachJudgmentSnapshot, CoachNextAction, ComposedSession, ContentReadinessResolution, RouteMode,
    SessionComposer, SystemHealthSnapshot, UncertaintyProfile, assess_content_readiness,
    resolve_coach_state, resolve_next_coach_action,
};
use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, EngineRegistry, clamp_bp};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachGovernanceCheck {
    pub check_code: String,
    pub owner_engine_key: String,
    pub status: String,
    pub severity: String,
    pub confidence_score: BasisPoints,
    pub rationale: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachArbitrationRecord {
    pub arbitration_code: String,
    pub winning_engine_key: String,
    pub losing_engine_keys: Vec<String>,
    pub authority_class: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalEngineHealth {
    pub engine_key: String,
    pub engine_title: String,
    pub layer: String,
    pub health_status: String,
    pub health_score: BasisPoints,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStageSnapshot {
    pub stage_code: String,
    pub owner_engine_key: String,
    pub status: String,
    pub summary: String,
    pub confidence_score: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachOrchestrationSnapshot {
    pub student_id: i64,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub overall_confidence_score: BasisPoints,
    pub guardrail_status: String,
    pub next_action: CoachNextAction,
    pub runtime_registry: EngineRegistry,
    pub constitutional_registry: EngineRegistry,
    pub uncertainty_profile: Option<UncertaintyProfile>,
    pub system_health: Option<SystemHealthSnapshot>,
    pub governance_checks: Vec<CoachGovernanceCheck>,
    pub arbitrations: Vec<CoachArbitrationRecord>,
    pub engine_health: Vec<ConstitutionalEngineHealth>,
    pub orchestration_stages: Vec<OrchestrationStageSnapshot>,
    pub suggested_session: Option<ComposedSession>,
    pub judgment: Option<CoachJudgmentSnapshot>,
}

pub struct CoachConstitutionService<'a> {
    conn: &'a Connection,
}

impl<'a> CoachConstitutionService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn build_orchestration_snapshot(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<CoachOrchestrationSnapshot> {
        let runtime_registry = EngineRegistry::core_runtime();
        let constitutional_registry = EngineRegistry::constitutional_runtime();
        let content = assess_content_readiness(self.conn, student_id)?;
        let state = resolve_coach_state(self.conn, student_id)?;
        let base_action = resolve_next_coach_action(self.conn, student_id)?;
        let dome = CoachIntelligenceDomeService::new(self.conn)
            .build_intelligence_dome(student_id, subject_id, topic_id)
            .ok();
        let judgment = CoachJudgmentEngine::new(self.conn)
            .build_judgment_snapshot(student_id, subject_id, topic_id)
            .ok();
        let resolved_subject_id = subject_id
            .or_else(|| judgment.as_ref().and_then(|item| item.subject_id))
            .or_else(|| {
                dome.as_ref()
                    .and_then(|item| item.topic_strategy.as_ref().map(|row| row.subject_id))
            });
        let resolved_topic_id = topic_id
            .or_else(|| judgment.as_ref().and_then(|item| item.topic_id))
            .or_else(|| {
                dome.as_ref()
                    .and_then(|item| item.topic_strategy.as_ref().map(|row| row.topic_id))
            });

        let checks = build_checks(
            &content,
            state.reason.as_deref(),
            dome.as_ref(),
            judgment.as_ref(),
        );
        let next_action = apply_guardrails(&base_action, &checks, dome.as_ref());
        let guardrail_status = derive_guardrail_status(&checks);
        let overall_confidence_score = average_bp([
            judgment
                .as_ref()
                .map(|item| item.judgment_confidence_score)
                .unwrap_or(4_800),
            dome.as_ref()
                .and_then(|item| item.uncertainty_profile.as_ref())
                .map(|item| 10_000 - item.uncertainty_score)
                .unwrap_or(5_000),
            if matches!(content.status, crate::ContentReadinessStatus::Ready) {
                7_200
            } else {
                4_300
            },
        ]);
        let arbitrations = build_arbitrations(&checks);
        let suggested_session = self.suggest_session(
            student_id,
            resolved_subject_id,
            &next_action,
            judgment.as_ref(),
        )?;
        let attempt_count = self.count_attempts(student_id)?;
        let engine_health = build_engine_health(
            &constitutional_registry,
            &checks,
            judgment.as_ref(),
            suggested_session.as_ref(),
            attempt_count,
        );
        let orchestration_stages = build_stages(
            attempt_count,
            judgment.is_some(),
            suggested_session.is_some(),
            &guardrail_status,
            overall_confidence_score,
        );

        let snapshot = CoachOrchestrationSnapshot {
            student_id,
            subject_id: resolved_subject_id,
            topic_id: resolved_topic_id,
            overall_confidence_score,
            guardrail_status,
            next_action,
            runtime_registry,
            constitutional_registry,
            uncertainty_profile: dome
                .as_ref()
                .and_then(|item| item.uncertainty_profile.clone()),
            system_health: dome
                .as_ref()
                .map(|item| item.intent_core.system_health.clone()),
            governance_checks: checks,
            arbitrations,
            engine_health,
            orchestration_stages,
            suggested_session,
            judgment,
        };
        self.persist_snapshot(&snapshot)?;
        CanonicalIntelligenceStore::new(self.conn)
            .record_orchestration_cycle("constitution_snapshot", &snapshot)?;
        Ok(snapshot)
    }

    fn suggest_session(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        action: &CoachNextAction,
        judgment: Option<&CoachJudgmentSnapshot>,
    ) -> EcoachResult<Option<ComposedSession>> {
        let Some(subject_id) = subject_id else {
            return Ok(None);
        };
        let station = match action.action_type {
            crate::CoachActionType::StartRepair => "repair",
            crate::CoachActionType::ReviewResults => "review",
            crate::CoachActionType::StartDiagnostic | crate::CoachActionType::GeneratePlan => {
                "foundation"
            }
            crate::CoachActionType::StartTodayMission
            | crate::CoachActionType::ResumeMission
            | crate::CoachActionType::AdjustPlan
            | crate::CoachActionType::ViewOverview => "checkpoint",
            _ => return Ok(None),
        };
        let route = match action.action_type {
            crate::CoachActionType::StartRepair => RouteMode::Rescue,
            crate::CoachActionType::ReviewResults => RouteMode::Reactivation,
            _ if judgment
                .map(|item| item.biggest_risk.contains("exam"))
                .unwrap_or(false) =>
            {
                RouteMode::HighYield
            }
            _ => RouteMode::Balanced,
        };
        SessionComposer::new(self.conn)
            .compose_session(
                student_id,
                subject_id,
                station,
                route,
                action.estimated_minutes.unwrap_or(30).clamp(15, 60),
            )
            .map(Some)
    }

    fn count_attempts(&self, student_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM student_question_attempts WHERE student_id = ?1",
                [student_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn persist_snapshot(&self, snapshot: &CoachOrchestrationSnapshot) -> EcoachResult<()> {
        let subject_key = snapshot.subject_id.unwrap_or(0);
        let topic_key = snapshot.topic_id.unwrap_or(0);
        self.conn
            .execute(
                "INSERT INTO coach_orchestration_runs (
                student_id, subject_key, subject_id, topic_key, topic_id,
                focus_layer, guardrail_status, final_action_type, final_route,
                overall_confidence_score, contradiction_count, snapshot_json, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, 'governance', ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'))
             ON CONFLICT(student_id, subject_key, topic_key) DO UPDATE SET
                subject_id = excluded.subject_id,
                topic_id = excluded.topic_id,
                guardrail_status = excluded.guardrail_status,
                final_action_type = excluded.final_action_type,
                final_route = excluded.final_route,
                overall_confidence_score = excluded.overall_confidence_score,
                contradiction_count = excluded.contradiction_count,
                snapshot_json = excluded.snapshot_json,
                updated_at = datetime('now')",
                params![
                    snapshot.student_id,
                    subject_key,
                    snapshot.subject_id,
                    topic_key,
                    snapshot.topic_id,
                    snapshot.guardrail_status,
                    action_type_code(&snapshot.next_action),
                    snapshot.next_action.route,
                    snapshot.overall_confidence_score,
                    snapshot
                        .governance_checks
                        .iter()
                        .filter(|check| check.check_code == "contradiction_check"
                            && check.status != "clear")
                        .count() as i64,
                    serde_json::to_string(snapshot)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        for check in &snapshot.governance_checks {
            self.conn
                .execute(
                    "INSERT INTO coach_governance_checks (
                    student_id, subject_key, subject_id, topic_key, topic_id,
                    check_code, check_label, owner_engine_key, status, severity,
                    confidence_score, rationale, payload_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))
                 ON CONFLICT(student_id, subject_key, topic_key, check_code) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_id = excluded.topic_id,
                    owner_engine_key = excluded.owner_engine_key,
                    status = excluded.status,
                    severity = excluded.severity,
                    confidence_score = excluded.confidence_score,
                    rationale = excluded.rationale,
                    payload_json = excluded.payload_json,
                    updated_at = datetime('now')",
                    params![
                        snapshot.student_id,
                        subject_key,
                        snapshot.subject_id,
                        topic_key,
                        snapshot.topic_id,
                        check.check_code,
                        check.owner_engine_key,
                        check.status,
                        check.severity,
                        check.confidence_score,
                        check.rationale,
                        serde_json::to_string(&check.payload)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        for arbitration in &snapshot.arbitrations {
            self.conn
                .execute(
                    "INSERT INTO coach_arbitration_records (
                    student_id, subject_key, subject_id, topic_key, topic_id,
                    arbitration_code, winning_engine_key, losing_engine_keys_json,
                    authority_class, rationale, outcome_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, '{}', datetime('now'))
                 ON CONFLICT(student_id, subject_key, topic_key, arbitration_code) DO UPDATE SET
                    winning_engine_key = excluded.winning_engine_key,
                    losing_engine_keys_json = excluded.losing_engine_keys_json,
                    authority_class = excluded.authority_class,
                    rationale = excluded.rationale,
                    updated_at = datetime('now')",
                    params![
                        snapshot.student_id,
                        subject_key,
                        snapshot.subject_id,
                        topic_key,
                        snapshot.topic_id,
                        arbitration.arbitration_code,
                        arbitration.winning_engine_key,
                        serde_json::to_string(&arbitration.losing_engine_keys)
                            .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                        arbitration.authority_class,
                        arbitration.rationale,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        for health in &snapshot.engine_health {
            self.conn
                .execute(
                    "INSERT INTO coach_engine_health_snapshots (
                    student_id, subject_key, subject_id, topic_key, topic_id,
                    engine_key, engine_title, layer, health_status, health_score,
                    rationale, payload_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, '{}', datetime('now'))
                 ON CONFLICT(student_id, subject_key, topic_key, engine_key) DO UPDATE SET
                    engine_title = excluded.engine_title,
                    layer = excluded.layer,
                    health_status = excluded.health_status,
                    health_score = excluded.health_score,
                    rationale = excluded.rationale,
                    updated_at = datetime('now')",
                    params![
                        snapshot.student_id,
                        subject_key,
                        snapshot.subject_id,
                        topic_key,
                        snapshot.topic_id,
                        health.engine_key,
                        health.engine_title,
                        health.layer,
                        health.health_status,
                        health.health_score,
                        health.rationale,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }

        Ok(())
    }
}

fn build_checks(
    content: &ContentReadinessResolution,
    state_reason: Option<&str>,
    dome: Option<&crate::CoachIntelligenceDomeSnapshot>,
    judgment: Option<&CoachJudgmentSnapshot>,
) -> Vec<CoachGovernanceCheck> {
    let uncertainty = dome
        .and_then(|item| item.uncertainty_profile.as_ref())
        .map(|item| item.uncertainty_score)
        .unwrap_or(5_200);
    let contradiction = judgment
        .and_then(|item| item.content_governor.as_ref())
        .map(|item| item.contradiction_risk_score)
        .unwrap_or(2_800)
        .max(
            dome.map(|item| {
                item.interference_cases
                    .iter()
                    .map(|case| case.severity_score)
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0),
        );
    let confidence_score = average_bp([
        judgment
            .map(|item| item.judgment_confidence_score)
            .unwrap_or(4_800),
        10_000 - uncertainty,
    ]);
    vec![
        CoachGovernanceCheck {
            check_code: "confidence_gate".to_string(),
            owner_engine_key: "confidence_gate".to_string(),
            status: if confidence_score < 5_500 {
                "downgraded"
            } else {
                "clear"
            }
            .to_string(),
            severity: severity(if confidence_score < 5_500 {
                7_200
            } else {
                3_200
            }),
            confidence_score,
            rationale: if confidence_score < 5_500 {
                "The coach should probe before taking a harder forward action.".to_string()
            } else {
                "Confidence is strong enough for normal progression.".to_string()
            },
            payload: json!({ "uncertainty_score": uncertainty }),
        },
        CoachGovernanceCheck {
            check_code: "contradiction_check".to_string(),
            owner_engine_key: "consistency_validator".to_string(),
            status: if contradiction >= 6_500 {
                "blocking"
            } else if contradiction >= 5_200 {
                "warning"
            } else {
                "clear"
            }
            .to_string(),
            severity: severity(contradiction),
            confidence_score: clamp_bp(i64::from(10_000 - contradiction / 2)),
            rationale: if contradiction >= 6_500 {
                "Interference and contradiction risk are too high for an aggressive move."
                    .to_string()
            } else {
                "No major contradiction is stopping the coach right now.".to_string()
            },
            payload: json!({ "contradiction_risk_score": contradiction }),
        },
        CoachGovernanceCheck {
            check_code: "policy_guardrail".to_string(),
            owner_engine_key: "policy_guardrail".to_string(),
            status: if !matches!(content.status, crate::ContentReadinessStatus::Ready) {
                "blocking"
            } else {
                "clear"
            }
            .to_string(),
            severity: if matches!(content.status, crate::ContentReadinessStatus::Ready) {
                "low".to_string()
            } else {
                "high".to_string()
            },
            confidence_score: if matches!(content.status, crate::ContentReadinessStatus::Ready) {
                7_000
            } else {
                8_200
            },
            rationale: state_reason
                .unwrap_or("The guardrail is only allowing moves that fit readiness and doctrine.")
                .to_string(),
            payload: json!({ "content_status": format!("{:?}", content.status) }),
        },
    ]
}

fn apply_guardrails(
    action: &CoachNextAction,
    checks: &[CoachGovernanceCheck],
    dome: Option<&crate::CoachIntelligenceDomeSnapshot>,
) -> CoachNextAction {
    if checks
        .iter()
        .any(|check| check.check_code == "policy_guardrail" && check.status == "blocking")
        && !matches!(action.action_type, crate::CoachActionType::ResolveContent)
    {
        return CoachNextAction {
            state: action.state,
            action_type: crate::CoachActionType::ResolveContent,
            title: "Resolve blocked content before advancing".to_string(),
            subtitle: "The governance layer is holding progression until readiness is safe."
                .to_string(),
            estimated_minutes: Some(8),
            route: "/coach/content".to_string(),
            context: json!({ "governed_from": action.route }),
        };
    }
    if checks
        .iter()
        .any(|check| check.check_code == "confidence_gate" && check.status == "downgraded")
        && matches!(
            action.action_type,
            crate::CoachActionType::StartTodayMission
                | crate::CoachActionType::ResumeMission
                | crate::CoachActionType::AdjustPlan
                | crate::CoachActionType::ViewOverview
        )
    {
        return CoachNextAction {
            state: action.state,
            action_type: crate::CoachActionType::StartRepair,
            title: "Run a probe-first repair loop".to_string(),
            subtitle: dome
                .and_then(|item| item.uncertainty_profile.as_ref())
                .map(|item| {
                    format!(
                        "The coach wants stronger evidence first: {}.",
                        item.evidence_needed.join(", ")
                    )
                })
                .unwrap_or_else(|| "The coach wants safer evidence before advancing.".to_string()),
            estimated_minutes: Some(20),
            route: "/coach/repair".to_string(),
            context: json!({ "governed_from": action.route }),
        };
    }
    action.clone()
}

fn build_arbitrations(checks: &[CoachGovernanceCheck]) -> Vec<CoachArbitrationRecord> {
    vec![
        CoachArbitrationRecord {
            arbitration_code: "progression_vs_probe".to_string(),
            winning_engine_key: if checks
                .iter()
                .any(|check| check.check_code == "confidence_gate" && check.status == "downgraded")
            {
                "confidence_gate".to_string()
            } else {
                "teaching_strategy".to_string()
            },
            losing_engine_keys: vec!["session_composer".to_string()],
            authority_class: "constitutional".to_string(),
            rationale: "The orchestration loop chooses whether to advance or probe first."
                .to_string(),
        },
        CoachArbitrationRecord {
            arbitration_code: "safety_vs_speed".to_string(),
            winning_engine_key: if checks
                .iter()
                .any(|check| check.check_code == "policy_guardrail" && check.status == "blocking")
            {
                "policy_guardrail".to_string()
            } else {
                "risk_engine".to_string()
            },
            losing_engine_keys: vec![
                "content_selection".to_string(),
                "session_composer".to_string(),
            ],
            authority_class: "blocking".to_string(),
            rationale: "Safety and doctrinal readiness outrank execution speed.".to_string(),
        },
    ]
}

fn build_engine_health(
    registry: &EngineRegistry,
    checks: &[CoachGovernanceCheck],
    judgment: Option<&CoachJudgmentSnapshot>,
    suggested_session: Option<&ComposedSession>,
    attempt_count: i64,
) -> Vec<ConstitutionalEngineHealth> {
    vec![
        health(
            registry,
            "response_evidence",
            if attempt_count > 0 { "healthy" } else { "thin" },
            clamp_bp(4_000 + attempt_count * 1_000),
            "Response evidence coverage is feeding the orchestration loop.",
        ),
        health(
            registry,
            "hypothesis_competition",
            if judgment.is_some() {
                "healthy"
            } else {
                "warming_up"
            },
            judgment
                .map(|item| item.judgment_confidence_score)
                .unwrap_or(4_700),
            "Hypothesis confidence is available for decision-making.",
        ),
        health(
            registry,
            "session_composer",
            if suggested_session.is_some() {
                "healthy"
            } else {
                "idle"
            },
            if suggested_session.is_some() {
                7_500
            } else {
                4_500
            },
            "Execution-design coverage for the governed next move.",
        ),
        health(
            registry,
            "confidence_gate",
            if checks
                .iter()
                .any(|check| check.check_code == "confidence_gate" && check.status == "downgraded")
            {
                "guarding"
            } else {
                "healthy"
            },
            checks
                .iter()
                .find(|check| check.check_code == "confidence_gate")
                .map(|check| check.confidence_score)
                .unwrap_or(5_000),
            "Confidence-gate quality for the current decision horizon.",
        ),
    ]
}

fn build_stages(
    attempt_count: i64,
    has_judgment: bool,
    has_session: bool,
    guardrail_status: &str,
    overall_confidence_score: BasisPoints,
) -> Vec<OrchestrationStageSnapshot> {
    vec![
        stage(
            "observe",
            "response_evidence",
            if attempt_count > 0 { "ready" } else { "sparse" },
            format!(
                "{} attempts are available for evidence ingestion.",
                attempt_count
            ),
            clamp_bp(4_000 + attempt_count * 900),
        ),
        stage(
            "diagnose",
            "hypothesis_competition",
            if has_judgment { "ready" } else { "warming_up" },
            "The coach has built a diagnosis layer for this horizon.".to_string(),
            if has_judgment { 7_000 } else { 4_700 },
        ),
        stage(
            "decide",
            "decision_arbitration",
            if guardrail_status == "clear" {
                "green"
            } else {
                "guarded"
            },
            format!("Governance completed with `{}` status.", guardrail_status),
            overall_confidence_score,
        ),
        stage(
            "compose",
            "session_composer",
            if has_session { "ready" } else { "idle" },
            if has_session {
                "A governed session blueprint is ready.".to_string()
            } else {
                "No session blueprint is needed for this move.".to_string()
            },
            if has_session { 7_400 } else { 4_500 },
        ),
        stage(
            "govern",
            "consistency_validator",
            if guardrail_status == "blocking" {
                "blocking"
            } else {
                "complete"
            },
            "Contradictions, confidence, and policy checks have all run.".to_string(),
            overall_confidence_score,
        ),
    ]
}

fn health(
    registry: &EngineRegistry,
    key: &str,
    status: &str,
    score: BasisPoints,
    rationale: &str,
) -> ConstitutionalEngineHealth {
    let contract = registry.find_engine(key);
    ConstitutionalEngineHealth {
        engine_key: key.to_string(),
        engine_title: contract
            .map(|item| item.title.clone())
            .unwrap_or_else(|| key.to_string()),
        layer: contract
            .map(|item| item.layer.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        health_status: status.to_string(),
        health_score: clamp_bp(i64::from(score)),
        rationale: rationale.to_string(),
    }
}

fn stage(
    code: &str,
    owner: &str,
    status: &str,
    summary: String,
    score: BasisPoints,
) -> OrchestrationStageSnapshot {
    OrchestrationStageSnapshot {
        stage_code: code.to_string(),
        owner_engine_key: owner.to_string(),
        status: status.to_string(),
        summary,
        confidence_score: clamp_bp(i64::from(score)),
    }
}

fn average_bp(values: impl IntoIterator<Item = BasisPoints>) -> BasisPoints {
    let mut total = 0_i64;
    let mut count = 0_i64;
    for value in values {
        total += i64::from(value);
        count += 1;
    }
    if count == 0 {
        0
    } else {
        clamp_bp(total / count)
    }
}

fn derive_guardrail_status(checks: &[CoachGovernanceCheck]) -> String {
    if checks.iter().any(|check| check.status == "blocking") {
        "blocking".to_string()
    } else if checks
        .iter()
        .any(|check| check.status == "warning" || check.status == "downgraded")
    {
        "guarded".to_string()
    } else {
        "clear".to_string()
    }
}

fn severity(score: BasisPoints) -> String {
    if score >= 7_000 {
        "high".to_string()
    } else if score >= 5_000 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

fn action_type_code(action: &CoachNextAction) -> &'static str {
    match action.action_type {
        crate::CoachActionType::ContinueOnboarding => "continue_onboarding",
        crate::CoachActionType::SelectSubjects => "select_subjects",
        crate::CoachActionType::ResolveContent => "resolve_content",
        crate::CoachActionType::StartDiagnostic => "start_diagnostic",
        crate::CoachActionType::GeneratePlan => "generate_plan",
        crate::CoachActionType::StartTodayMission => "start_today_mission",
        crate::CoachActionType::ResumeMission => "resume_mission",
        crate::CoachActionType::ReviewResults => "review_results",
        crate::CoachActionType::StartRepair => "start_repair",
        crate::CoachActionType::AdjustPlan => "adjust_plan",
        crate::CoachActionType::ViewOverview => "view_overview",
    }
}
