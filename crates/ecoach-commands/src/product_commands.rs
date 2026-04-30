use std::collections::BTreeSet;

use chrono::Utc;
use ecoach_coach_brain::{
    CoachActionType, CoachBrainTrigger, CoachJudgmentEngine, TeacherlessCapabilityReview,
    evaluate_coach_brain,
};
use ecoach_identity::IdentityService;
use ecoach_intake::IntakeService;
use ecoach_reporting::strategy::load_strategy_summary;
use ecoach_reporting::{
    AdminOversightService, AdminOversightSnapshot, DashboardService, ParentDashboardSnapshot,
    ParentInsightService, ParentStudentSummary,
};
use ecoach_student_model::{LearnerTruthSnapshot, StudentModelService};
use ecoach_substrate::{EcoachError, LearnerEvidenceFabric, ThresholdRegistry};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{error::CommandError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachDirectiveDto {
    pub directive_type: String,
    pub audience: String,
    pub title: String,
    pub summary: String,
    pub priority: String,
    pub primary_action: String,
    pub supporting_signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightCardDto {
    pub card_key: String,
    pub audience: String,
    pub title: String,
    pub summary: String,
    pub tone: String,
    pub metric_label: Option<String>,
    pub metric_value: Option<i64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendationDto {
    pub recommendation_key: String,
    pub audience: String,
    pub label: String,
    pub summary: String,
    pub route_key: String,
    pub urgency: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceExplanationDto {
    pub audience: String,
    pub headline: String,
    pub summary: String,
    pub supporting_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistryEntryDto {
    pub signal_key: String,
    pub capability_key: String,
    pub tool_key: String,
    pub tool_label: String,
    pub route_key: String,
    pub intended_audience: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistryDto {
    pub generated_at: String,
    pub mappings: Vec<CapabilityRegistryEntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticBandDto {
    pub registry_key: String,
    pub label: String,
    pub min_bp: Option<i64>,
    pub max_bp: Option<i64>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceFreshnessRuleDto {
    pub label: String,
    pub min_days: i64,
    pub max_days: Option<i64>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticsRegistryDto {
    pub generated_at: String,
    pub thresholds: ThresholdRegistry,
    pub readiness_bands: Vec<SemanticBandDto>,
    pub mastery_bands: Vec<SemanticBandDto>,
    pub confidence_bands: Vec<SemanticBandDto>,
    pub severity_bands: Vec<SemanticBandDto>,
    pub freshness_rules: Vec<EvidenceFreshnessRuleDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentProductSurfaceDto {
    pub student_id: i64,
    pub student_name: String,
    pub generated_at: String,
    pub overall_readiness_band: String,
    pub directives: Vec<CoachDirectiveDto>,
    pub insight_cards: Vec<InsightCardDto>,
    pub action_recommendations: Vec<ActionRecommendationDto>,
    pub audience_explanations: Vec<AudienceExplanationDto>,
    pub capability_reviews: Vec<TeacherlessCapabilityReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentProductSurfaceDto {
    pub parent_id: i64,
    pub parent_name: String,
    pub generated_at: String,
    pub student_focus: Option<ParentStudentSummary>,
    pub directives: Vec<CoachDirectiveDto>,
    pub insight_cards: Vec<InsightCardDto>,
    pub action_recommendations: Vec<ActionRecommendationDto>,
    pub audience_explanations: Vec<AudienceExplanationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHealthTopicIssueDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub average_quality_bp: i64,
    pub evaluation_run_count: i64,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHealthReadModelDto {
    pub generated_at: String,
    pub source_count: i64,
    pub stale_source_count: i64,
    pub overdue_source_review_count: i64,
    pub active_mission_count: i64,
    pub mission_review_required_count: i64,
    pub blocked_publish_count: i64,
    pub preview_publish_count: i64,
    pub average_quality_bp: i64,
    pub low_quality_topics: Vec<ContentHealthTopicIssueDto>,
    pub action_recommendations: Vec<ActionRecommendationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminProductSurfaceDto {
    pub admin_id: i64,
    pub admin_name: String,
    pub generated_at: String,
    pub oversight: AdminOversightSnapshot,
    pub content_health: ContentHealthReadModelDto,
    pub directives: Vec<CoachDirectiveDto>,
    pub insight_cards: Vec<InsightCardDto>,
    pub action_recommendations: Vec<ActionRecommendationDto>,
    pub audience_explanations: Vec<AudienceExplanationDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedEntitlementAccountDto {
    pub account_id: i64,
    pub display_name: String,
    pub account_type: String,
    pub entitlement_tier: String,
    pub status: String,
    pub last_active_at: Option<String>,
    pub linked_student_ids: Vec<i64>,
    pub feature_unlocks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementAuditEntryDto {
    pub id: i64,
    pub account_id: i64,
    pub changed_by_account_id: Option<i64>,
    pub previous_tier: String,
    pub new_tier: String,
    pub previous_status: String,
    pub new_status: String,
    pub reason: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementManagerSnapshotDto {
    pub generated_at: String,
    pub premium_count: i64,
    pub elite_count: i64,
    pub inactive_count: i64,
    pub accounts: Vec<ManagedEntitlementAccountDto>,
    pub audit_log: Vec<EntitlementAuditEntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicVaultFileDto {
    pub file_id: i64,
    pub file_name: String,
    pub file_kind: String,
    pub document_role: Option<String>,
    pub document_origin: Option<String>,
    pub ocr_status: String,
    pub layout_status: String,
    pub layout_kind: Option<String>,
    pub detected_topics: Vec<String>,
    pub question_patterns: Vec<String>,
    pub review_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcademicVaultBundleDto {
    pub bundle_id: i64,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub confirmation_state: String,
    pub coach_application_status: String,
    pub review_priority: String,
    pub bundle_kind: String,
    pub shared_promotion_status: Option<String>,
    pub detected_subjects: Vec<String>,
    pub detected_topics: Vec<String>,
    pub weakness_signals: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub summary_points: Vec<String>,
    pub files: Vec<AcademicVaultFileDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAcademicVaultDto {
    pub student_id: i64,
    pub student_name: String,
    pub generated_at: String,
    pub active_topics: Vec<String>,
    pub recurring_mistake_signals: Vec<String>,
    pub teacher_style_signals: Vec<String>,
    pub urgent_review_bundle_count: i64,
    pub ocr_attention_count: i64,
    pub pending_shared_promotion_count: i64,
    pub bundles: Vec<AcademicVaultBundleDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARealGuidanceEventDto {
    pub student_id: Option<i64>,
    pub student_name: Option<String>,
    pub headline: String,
    pub summary: String,
    pub route_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARealSurfaceDto {
    pub student_id: i64,
    pub student_name: String,
    pub generated_at: String,
    pub current_mode: String,
    pub tone_style: String,
    pub motivation_style: String,
    pub urgency_style: String,
    pub explanation_style: String,
    pub headline: String,
    pub summary: String,
    pub next_route_key: String,
    pub active_signals: Vec<String>,
    pub upload_context: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub recent_guidance: Vec<ARealGuidanceEventDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARealProfileInputDto {
    pub student_id: i64,
    pub tone_style: Option<String>,
    pub motivation_style: Option<String>,
    pub urgency_style: Option<String>,
    pub explanation_style: Option<String>,
    pub narrative_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARealProfileDto {
    pub student_id: i64,
    pub tone_style: String,
    pub motivation_style: String,
    pub urgency_style: String,
    pub explanation_style: String,
    pub narrative_enabled: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceGovernanceObjectDto {
    pub source_upload_id: i64,
    pub title: String,
    pub source_kind: String,
    pub source_status: String,
    pub parse_status_detail: Option<String>,
    pub source_tier: Option<String>,
    pub trust_score_bp: i64,
    pub stale_flag: bool,
    pub review_due_at: Option<String>,
    pub unresolved_review_count: i64,
    pub publish_decision_count: i64,
    pub last_governance_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrRecoveryObjectDto {
    pub bundle_id: i64,
    pub student_id: i64,
    pub student_name: String,
    pub file_id: i64,
    pub file_name: String,
    pub document_role: Option<String>,
    pub ocr_status: String,
    pub layout_status: String,
    pub layout_kind: Option<String>,
    pub review_priority: String,
    pub review_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedPromotionDto {
    pub id: i64,
    pub bundle_id: i64,
    pub source_upload_id: Option<i64>,
    pub requested_by_account_id: Option<i64>,
    pub promotion_status: String,
    pub promotion_summary: Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperAdminControlTowerDto {
    pub admin_id: i64,
    pub admin_name: String,
    pub generated_at: String,
    pub entitlement_manager: EntitlementManagerSnapshotDto,
    pub content_health: ContentHealthReadModelDto,
    pub source_objects: Vec<SourceGovernanceObjectDto>,
    pub ocr_recovery_objects: Vec<OcrRecoveryObjectDto>,
    pub pending_shared_promotions: Vec<SharedPromotionDto>,
    pub recent_areal_guidance: Vec<ARealGuidanceEventDto>,
    pub action_recommendations: Vec<ActionRecommendationDto>,
}

pub fn get_capability_registry() -> Result<CapabilityRegistryDto, CommandError> {
    Ok(CapabilityRegistryDto {
        generated_at: Utc::now().to_rfc3339(),
        mappings: build_capability_registry(),
    })
}

pub fn get_semantics_registry() -> Result<SemanticsRegistryDto, CommandError> {
    Ok(build_semantics_registry())
}

pub fn get_student_product_surface(
    state: &AppState,
    student_id: i64,
) -> Result<StudentProductSurfaceDto, CommandError> {
    state.with_connection(|conn| {
        let student_model = StudentModelService::new(conn);
        let truth = student_model.get_learner_truth_snapshot(student_id)?;
        let evidence_fabric = student_model.get_learner_evidence_fabric(student_id, 6)?;
        let dashboard = DashboardService::new(conn).get_student_dashboard(student_id)?;
        let coach_output =
            evaluate_coach_brain(conn, student_id, CoachBrainTrigger::ManualRefresh, 14)?;
        let judgment =
            CoachJudgmentEngine::new(conn).build_judgment_snapshot(student_id, None, None)?;
        let strategy = load_strategy_summary(conn, student_id)?;

        Ok(StudentProductSurfaceDto {
            student_id: truth.student_id,
            student_name: truth.student_name.clone(),
            generated_at: Utc::now().to_rfc3339(),
            overall_readiness_band: dashboard.overall_readiness_band.clone(),
            directives: build_student_directives(&truth, &dashboard, &coach_output, &judgment),
            insight_cards: build_student_insight_cards(
                &truth,
                &dashboard,
                &judgment,
                strategy.as_ref(),
            ),
            action_recommendations: build_student_action_recommendations(
                &truth,
                &coach_output.next_action,
                &judgment,
                strategy.as_ref(),
            ),
            audience_explanations: build_student_audience_explanations(
                &truth,
                &dashboard,
                &judgment,
                &evidence_fabric,
            ),
            capability_reviews: judgment.capability_reviews,
        })
    })
}

pub fn get_parent_product_surface(
    state: &AppState,
    parent_id: i64,
    student_id: Option<i64>,
) -> Result<ParentProductSurfaceDto, CommandError> {
    state.with_connection(|conn| {
        let service = ParentInsightService::new(conn);
        let dashboard = service.build_parent_dashboard(parent_id)?;
        let student_focus = select_parent_student(&dashboard, student_id)?;

        let (directives, insight_cards, action_recommendations, audience_explanations) =
            if let Some(student_summary) = student_focus.as_ref() {
                (
                    build_parent_directives(student_summary),
                    build_parent_insight_cards(student_summary),
                    build_parent_action_recommendations(student_summary),
                    build_parent_audience_explanations(student_summary),
                )
            } else {
                (
                    vec![CoachDirectiveDto {
                        directive_type: "monitor_household".to_string(),
                        audience: "parent".to_string(),
                        title: "No linked student selected".to_string(),
                        summary: "Link a learner or choose a child to unlock translated oversight cards.".to_string(),
                        priority: "medium".to_string(),
                        primary_action: "open_parent_dashboard".to_string(),
                        supporting_signals: vec!["parent_link_required".to_string()],
                    }],
                    Vec::new(),
                    Vec::new(),
                    vec![AudienceExplanationDto {
                        audience: "parent".to_string(),
                        headline: "Parent view is ready once a learner is linked.".to_string(),
                        summary: "The product keeps the parent surface translated and action-focused instead of exposing raw engine records.".to_string(),
                        supporting_points: vec![
                            "Link a learner account to see readiness, risks, and next actions.".to_string(),
                        ],
                    }],
                )
            };

        Ok(ParentProductSurfaceDto {
            parent_id: dashboard.parent_id,
            parent_name: dashboard.parent_name,
            generated_at: dashboard.generated_at,
            student_focus,
            directives,
            insight_cards,
            action_recommendations,
            audience_explanations,
        })
    })
}

pub fn get_content_health_read_model(
    state: &AppState,
) -> Result<ContentHealthReadModelDto, CommandError> {
    state.with_connection(|conn| build_content_health_read_model(conn).map_err(Into::into))
}

pub fn get_admin_product_surface(
    state: &AppState,
    admin_id: i64,
) -> Result<AdminProductSurfaceDto, CommandError> {
    state.with_connection(|conn| {
        let oversight =
            AdminOversightService::new(conn).build_admin_oversight_snapshot(admin_id)?;
        let content_health = build_content_health_read_model(conn)?;
        Ok(AdminProductSurfaceDto {
            admin_id: oversight.admin_id,
            admin_name: oversight.admin_name.clone(),
            generated_at: oversight.generated_at.clone(),
            directives: build_admin_directives(&oversight, &content_health),
            insight_cards: build_admin_insight_cards(&oversight, &content_health),
            action_recommendations: build_admin_action_recommendations(&oversight, &content_health),
            audience_explanations: build_admin_audience_explanations(&oversight, &content_health),
            oversight,
            content_health,
        })
    })
}

pub fn get_entitlement_manager_snapshot(
    state: &AppState,
    limit: Option<usize>,
) -> Result<EntitlementManagerSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        build_entitlement_manager_snapshot(conn, limit.unwrap_or(24)).map_err(Into::into)
    })
}

pub fn get_personal_academic_vault(
    state: &AppState,
    student_id: i64,
    limit: Option<usize>,
) -> Result<PersonalAcademicVaultDto, CommandError> {
    state.with_connection(|conn| {
        build_personal_academic_vault(conn, student_id, limit.unwrap_or(8)).map_err(Into::into)
    })
}

pub fn get_areal_surface(
    state: &AppState,
    student_id: i64,
) -> Result<ARealSurfaceDto, CommandError> {
    state.with_connection(|conn| build_areal_surface(conn, student_id).map_err(Into::into))
}

pub fn save_areal_profile(
    state: &AppState,
    input: ARealProfileInputDto,
) -> Result<ARealProfileDto, CommandError> {
    state.with_connection(|conn| upsert_areal_profile(conn, input).map_err(Into::into))
}

pub fn get_super_admin_control_tower(
    state: &AppState,
    admin_id: i64,
    limit: Option<usize>,
) -> Result<SuperAdminControlTowerDto, CommandError> {
    state.with_connection(|conn| {
        build_super_admin_control_tower(conn, admin_id, limit.unwrap_or(12)).map_err(Into::into)
    })
}

fn select_parent_student(
    dashboard: &ParentDashboardSnapshot,
    student_id: Option<i64>,
) -> Result<Option<ParentStudentSummary>, EcoachError> {
    if dashboard.students.is_empty() {
        return Ok(None);
    }
    if let Some(student_id) = student_id {
        dashboard
            .students
            .iter()
            .find(|student| student.student_id == student_id)
            .cloned()
            .map(Some)
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "student {} is not linked to parent {}",
                    student_id, dashboard.parent_id
                ))
            })
    } else {
        Ok(dashboard.students.first().cloned())
    }
}

fn build_student_directives(
    truth: &LearnerTruthSnapshot,
    dashboard: &ecoach_reporting::StudentDashboard,
    coach_output: &ecoach_coach_brain::CoachBrainOutput,
    judgment: &ecoach_coach_brain::CoachJudgmentSnapshot,
) -> Vec<CoachDirectiveDto> {
    let mut directives = vec![CoachDirectiveDto {
        directive_type: directive_type_for_action(coach_output.next_action.action_type),
        audience: "student".to_string(),
        title: coach_output.next_action.title.clone(),
        summary: coach_output.next_action.subtitle.clone(),
        priority: directive_priority_for_state(coach_output.next_action.action_type),
        primary_action: coach_output.next_action.route.clone(),
        supporting_signals: vec![
            format!("state:{}", coach_output.state.state.as_str()),
            format!("independence:{}", judgment.independence_band),
        ],
    }];

    if truth.pending_review_count > 0 {
        directives.push(CoachDirectiveDto {
            directive_type: "queue_review".to_string(),
            audience: "student".to_string(),
            title: "Resolve recent evidence before adding more work".to_string(),
            summary: format!(
                "{} review items are still open, so the coach is protecting you from shallow progress.",
                truth.pending_review_count
            ),
            priority: "high".to_string(),
            primary_action: "review_queue".to_string(),
            supporting_signals: vec!["pending_review".to_string()],
        });
    }
    if truth.due_memory_count > 0 {
        directives.push(CoachDirectiveDto {
            directive_type: "insert_reactivation_now".to_string(),
            audience: "student".to_string(),
            title: "Run a memory rescue block".to_string(),
            summary: format!(
                "{} memory items are due, so the coach wants recall protection before new content.",
                truth.due_memory_count
            ),
            priority: "high".to_string(),
            primary_action: "memory_rescue".to_string(),
            supporting_signals: vec!["memory_due".to_string()],
        });
    }
    if matches!(
        dashboard.overall_readiness_band.as_str(),
        "At Risk" | "Not Ready"
    ) {
        directives.push(CoachDirectiveDto {
            directive_type: "show_fragility_warning".to_string(),
            audience: "student".to_string(),
            title: "Current readiness is still fragile".to_string(),
            summary: format!(
                "{} readiness means the coach is prioritizing recovery before acceleration.",
                dashboard.overall_readiness_band
            ),
            priority: "high".to_string(),
            primary_action: "coach_hub".to_string(),
            supporting_signals: vec!["readiness_gap".to_string()],
        });
    }
    if judgment
        .biggest_risk
        .to_ascii_lowercase()
        .contains("foundation")
    {
        directives.push(CoachDirectiveDto {
            directive_type: "rescue_foundation_first".to_string(),
            audience: "student".to_string(),
            title: "Repair the base before pushing ahead".to_string(),
            summary: judgment.biggest_risk.clone(),
            priority: "high".to_string(),
            primary_action: "repair_path".to_string(),
            supporting_signals: vec!["foundation_risk".to_string()],
        });
    }
    if truth
        .recent_diagnoses
        .iter()
        .any(|item| item.primary_diagnosis.contains("confusion"))
    {
        directives.push(CoachDirectiveDto {
            directive_type: "run_contrast_block".to_string(),
            audience: "student".to_string(),
            title: "Run a contrast drill".to_string(),
            summary: "The coach has spotted interference-like errors and wants a clean discrimination block.".to_string(),
            priority: "medium".to_string(),
            primary_action: "contrast_mode".to_string(),
            supporting_signals: vec!["concept_confusion".to_string()],
        });
    }
    directives
}

fn build_student_insight_cards(
    truth: &LearnerTruthSnapshot,
    dashboard: &ecoach_reporting::StudentDashboard,
    judgment: &ecoach_coach_brain::CoachJudgmentSnapshot,
    strategy: Option<&ecoach_reporting::ReportingStrategySummary>,
) -> Vec<InsightCardDto> {
    let mut cards = vec![
        InsightCardDto {
            card_key: "readiness_snapshot".to_string(),
            audience: "student".to_string(),
            title: "Readiness snapshot".to_string(),
            summary: format!(
                "Your current readiness sits at `{}` and the coach is optimizing the next move around that state.",
                dashboard.overall_readiness_band
            ),
            tone: "steady".to_string(),
            metric_label: Some("overall_mastery_bp".to_string()),
            metric_value: Some(truth.overall_mastery_score as i64),
            tags: vec!["readiness".to_string(), "coachhub".to_string()],
        },
        InsightCardDto {
            card_key: "risk_translation".to_string(),
            audience: "student".to_string(),
            title: "What the coach thinks is most fragile".to_string(),
            summary: judgment.biggest_risk.clone(),
            tone: "protective".to_string(),
            metric_label: Some("judgment_confidence_bp".to_string()),
            metric_value: Some(judgment.judgment_confidence_score as i64),
            tags: vec!["risk".to_string(), "translation".to_string()],
        },
        InsightCardDto {
            card_key: "evidence_depth".to_string(),
            audience: "student".to_string(),
            title: "Evidence depth".to_string(),
            summary: format!(
                "The learner model is currently backed by {} recent diagnoses and {} memory signals.",
                truth.recent_diagnoses.len(),
                truth.memory_summaries.len()
            ),
            tone: "transparent".to_string(),
            metric_label: Some("pending_review_count".to_string()),
            metric_value: Some(truth.pending_review_count),
            tags: vec!["evidence".to_string()],
        },
    ];
    if let Some(strategy) = strategy {
        cards.push(InsightCardDto {
            card_key: "strategy_mode".to_string(),
            audience: "student".to_string(),
            title: "Current coach mode".to_string(),
            summary: format!(
                "The reporting layer currently sees this learner in `{}` mode with focus on {}.",
                strategy.strategy_mode,
                if strategy.priority_topics.is_empty() {
                    "stability and follow-through".to_string()
                } else {
                    strategy.priority_topics.join(", ")
                }
            ),
            tone: "directive".to_string(),
            metric_label: Some("overall_readiness_score".to_string()),
            metric_value: Some(strategy.overall_readiness_score.into()),
            tags: vec!["strategy".to_string()],
        });
    }
    cards
}

fn build_student_action_recommendations(
    truth: &LearnerTruthSnapshot,
    next_action: &ecoach_coach_brain::CoachNextAction,
    judgment: &ecoach_coach_brain::CoachJudgmentSnapshot,
    strategy: Option<&ecoach_reporting::ReportingStrategySummary>,
) -> Vec<ActionRecommendationDto> {
    let mut actions = vec![ActionRecommendationDto {
        recommendation_key: "primary_coach_move".to_string(),
        audience: "student".to_string(),
        label: next_action.title.clone(),
        summary: next_action.subtitle.clone(),
        route_key: next_action.route.clone(),
        urgency: directive_priority_for_state(next_action.action_type),
        rationale: judgment.next_best_move.clone(),
    }];
    if truth.due_memory_count > 0 {
        actions.push(ActionRecommendationDto {
            recommendation_key: "memory_rescue".to_string(),
            audience: "student".to_string(),
            label: "Run a short recall rescue".to_string(),
            summary:
                "Use retrieval before more exposure so the coach does not build on forgetting."
                    .to_string(),
            route_key: "memory_dashboard".to_string(),
            urgency: "high".to_string(),
            rationale: "Due memory evidence is already active.".to_string(),
        });
    }
    if truth.pending_review_count > 0 {
        actions.push(ActionRecommendationDto {
            recommendation_key: "review_recent_evidence".to_string(),
            audience: "student".to_string(),
            label: "Close the review loop".to_string(),
            summary:
                "Review recent marked evidence so the model can update with cleaner confidence."
                    .to_string(),
            route_key: "evidence_inbox".to_string(),
            urgency: "high".to_string(),
            rationale: "Pending review items are blocking clearer planning.".to_string(),
        });
    }
    if let Some(strategy) = strategy {
        actions.push(ActionRecommendationDto {
            recommendation_key: "follow_strategy_mode".to_string(),
            audience: "student".to_string(),
            label: format!("Follow {} mode", strategy.strategy_mode),
            summary: "The reporting layer has already translated the learner state into a clear strategic mode.".to_string(),
            route_key: "strategy_summary".to_string(),
            urgency: "medium".to_string(),
            rationale: format!(
                "Priority topics: {}",
                if strategy.priority_topics.is_empty() {
                    "stability".to_string()
                } else {
                    strategy.priority_topics.join(", ")
                }
            ),
        });
    }
    actions
}

fn build_student_audience_explanations(
    truth: &LearnerTruthSnapshot,
    dashboard: &ecoach_reporting::StudentDashboard,
    judgment: &ecoach_coach_brain::CoachJudgmentSnapshot,
    evidence_fabric: &LearnerEvidenceFabric,
) -> Vec<AudienceExplanationDto> {
    vec![
        AudienceExplanationDto {
            audience: "student".to_string(),
            headline: "The coach is trying to protect real progress, not just activity."
                .to_string(),
            summary: format!(
                "Your current state is `{}`, with {} pending reviews and {} due memory checks, so the system is prioritizing stability over speed.",
                dashboard.overall_readiness_band,
                truth.pending_review_count,
                truth.due_memory_count
            ),
            supporting_points: vec![
                judgment.biggest_risk.clone(),
                format!("Next best move: {}", judgment.next_best_move),
            ],
        },
        AudienceExplanationDto {
            audience: "parent".to_string(),
            headline: "The system is translating learner evidence into a safer next step."
                .to_string(),
            summary: format!(
                "The learner model currently has {} evidence records and is using that to avoid pushing fragile understanding too early.",
                evidence_fabric.evidence_records.len()
            ),
            supporting_points: vec![
                "Parent outputs should stay translated rather than exposing raw diagnostics."
                    .to_string(),
                format!(
                    "Current readiness band: {}",
                    dashboard.overall_readiness_band
                ),
            ],
        },
        AudienceExplanationDto {
            audience: "admin".to_string(),
            headline: "This surface is composed from the learner truth graph plus coach policy."
                .to_string(),
            summary: format!(
                "Judgment confidence is {} bp and the active explanation is `{}`.",
                judgment.judgment_confidence_score, judgment.next_best_move
            ),
            supporting_points: vec![
                format!("Signals available: {}", evidence_fabric.signals.len()),
                format!("Capability reviews: {}", judgment.capability_reviews.len()),
            ],
        },
    ]
}

fn build_parent_directives(student_summary: &ParentStudentSummary) -> Vec<CoachDirectiveDto> {
    let mut directives = vec![CoachDirectiveDto {
        directive_type: "parent_attention_summary".to_string(),
        audience: "parent".to_string(),
        title: format!(
            "{} needs translated oversight",
            student_summary.student_name
        ),
        summary: student_summary.weekly_memo.clone(),
        priority: if matches!(
            student_summary.overall_readiness_band.as_str(),
            "At Risk" | "Not Ready"
        ) {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        primary_action: "parent_dashboard".to_string(),
        supporting_signals: student_summary
            .active_risks
            .iter()
            .map(|risk| risk.title.clone())
            .collect(),
    }];
    if !student_summary.active_risks.is_empty() {
        directives.push(CoachDirectiveDto {
            directive_type: "parent_alert_candidate".to_string(),
            audience: "parent".to_string(),
            title: "A coach follow-through may need home support".to_string(),
            summary: "The parent layer is highlighting only the issues that need practical support, not raw engine detail.".to_string(),
            priority: "high".to_string(),
            primary_action: "parent_actions".to_string(),
            supporting_signals: student_summary
                .active_risks
                .iter()
                .map(|risk| risk.severity.clone())
                .collect(),
        });
    }
    directives
}

fn build_parent_insight_cards(student_summary: &ParentStudentSummary) -> Vec<InsightCardDto> {
    vec![
        InsightCardDto {
            card_key: "parent_readiness".to_string(),
            audience: "parent".to_string(),
            title: "Child readiness".to_string(),
            summary: format!(
                "{} is currently in the `{}` readiness band.",
                student_summary.student_name, student_summary.overall_readiness_band
            ),
            tone: "translated".to_string(),
            metric_label: Some("subject_count".to_string()),
            metric_value: Some(student_summary.subject_summaries.len() as i64),
            tags: vec!["readiness".to_string()],
        },
        InsightCardDto {
            card_key: "parent_risks".to_string(),
            audience: "parent".to_string(),
            title: "What needs attention".to_string(),
            summary: if student_summary.active_risks.is_empty() {
                "No active high-signal risks are currently demanding parent action.".to_string()
            } else {
                student_summary
                    .active_risks
                    .iter()
                    .map(|risk| risk.title.clone())
                    .collect::<Vec<_>>()
                    .join("; ")
            },
            tone: "careful".to_string(),
            metric_label: Some("active_risk_count".to_string()),
            metric_value: Some(student_summary.active_risks.len() as i64),
            tags: vec!["risk".to_string()],
        },
    ]
}

fn build_parent_action_recommendations(
    student_summary: &ParentStudentSummary,
) -> Vec<ActionRecommendationDto> {
    let mut actions = student_summary
        .recommendations
        .iter()
        .map(|item| ActionRecommendationDto {
            recommendation_key: normalize_key(item),
            audience: "parent".to_string(),
            label: item.clone(),
            summary: item.clone(),
            route_key: "parent_dashboard".to_string(),
            urgency: if matches!(
                student_summary.overall_readiness_band.as_str(),
                "At Risk" | "Not Ready"
            ) {
                "high".to_string()
            } else {
                "medium".to_string()
            },
            rationale:
                "The parent layer only surfaces translated actions that can improve follow-through."
                    .to_string(),
        })
        .collect::<Vec<_>>();
    if actions.is_empty() {
        actions.push(ActionRecommendationDto {
            recommendation_key: "stay_available".to_string(),
            audience: "parent".to_string(),
            label: "Stay available for follow-through".to_string(),
            summary: "The current signal is stable enough that support mainly means consistency and encouragement.".to_string(),
            route_key: "parent_dashboard".to_string(),
            urgency: "low".to_string(),
            rationale: "No strong parent intervention is currently required.".to_string(),
        });
    }
    actions
}

fn build_parent_audience_explanations(
    student_summary: &ParentStudentSummary,
) -> Vec<AudienceExplanationDto> {
    vec![
        AudienceExplanationDto {
            audience: "parent".to_string(),
            headline: "This summary is translated into practical parent language.".to_string(),
            summary: format!(
                "The system is not showing raw engine metrics. It is showing what matters most for {} right now.",
                student_summary.student_name
            ),
            supporting_points: vec![
                format!("Readiness band: {}", student_summary.overall_readiness_band),
                format!("Weekly memo: {}", student_summary.weekly_memo),
            ],
        },
        AudienceExplanationDto {
            audience: "student".to_string(),
            headline: "The parent view is aligned with the student model.".to_string(),
            summary: "Parent explanations are intentionally gentler and more action-focused than the internal coach view.".to_string(),
            supporting_points: vec![
                "Students see direct coaching; parents see translated oversight.".to_string(),
            ],
        },
        AudienceExplanationDto {
            audience: "admin".to_string(),
            headline: "Parent-facing translation stays filtered.".to_string(),
            summary: "This keeps the trust layer clean and prevents parents from receiving raw engine noise.".to_string(),
            supporting_points: vec![
                format!("Active parent-facing risks: {}", student_summary.active_risks.len()),
            ],
        },
    ]
}

fn build_content_health_read_model(
    conn: &Connection,
) -> Result<ContentHealthReadModelDto, EcoachError> {
    let (source_count, stale_source_count, overdue_source_review_count, average_source_quality): (
        i64,
        i64,
        i64,
        f64,
    ) = conn
        .query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN stale_flag = 1 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN review_due_at IS NOT NULL AND review_due_at <= datetime('now') THEN 1 ELSE 0 END), 0),
                COALESCE(AVG((confidence_score + trust_score_bp + freshness_score_bp) / 3.0), 0)
             FROM curriculum_source_uploads",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let (active_mission_count, mission_review_required_count): (i64, i64) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(CASE WHEN status IN ('queued', 'running', 'review_required') THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'review_required' THEN 1 ELSE 0 END), 0)
             FROM content_research_missions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let (blocked_publish_count, preview_publish_count): (i64, i64) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(CASE WHEN decision_status IN ('rejected', 'rollback') THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN decision_status = 'preview' THEN 1 ELSE 0 END), 0)
             FROM content_publish_decisions",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let average_quality_bp: i64 = conn
        .query_row(
            "SELECT COALESCE(AVG((groundedness_bp + relevance_bp + correctness_bp + completeness_bp + utilization_bp) / 5.0), 0)
             FROM content_evaluation_runs",
            [],
            |row| row.get::<_, f64>(0),
        )
        .map(|value| value.round() as i64)
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut low_quality_statement = conn
        .prepare(
            "SELECT
                t.id,
                t.name,
                AVG((cer.groundedness_bp + cer.relevance_bp + cer.correctness_bp + cer.completeness_bp + cer.utilization_bp) / 5.0) AS average_quality,
                COUNT(*)
             FROM content_evaluation_runs cer
             JOIN topics t ON t.id = cer.topic_id
             GROUP BY t.id, t.name
             ORDER BY average_quality ASC, COUNT(*) DESC, t.name ASC
             LIMIT 5",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = low_quality_statement
        .query_map([], |row| {
            let average_quality = row.get::<_, f64>(2)?.round() as i64;
            Ok(ContentHealthTopicIssueDto {
                topic_id: row.get(0)?,
                topic_name: row.get(1)?,
                average_quality_bp: average_quality,
                evaluation_run_count: row.get(3)?,
                severity: if average_quality < 5_500 {
                    "critical".to_string()
                } else if average_quality < 6_500 {
                    "high".to_string()
                } else {
                    "medium".to_string()
                },
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut low_quality_topics = Vec::new();
    for row in rows {
        let issue = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
        if issue.average_quality_bp < 7_000 {
            low_quality_topics.push(issue);
        }
    }

    let mut action_recommendations = Vec::new();
    if stale_source_count > 0 {
        action_recommendations.push(ActionRecommendationDto {
            recommendation_key: "refresh_stale_sources".to_string(),
            audience: "admin".to_string(),
            label: "Refresh stale source registry entries".to_string(),
            summary: "Some content sources are stale, so trust and freshness should be reviewed before the next publish cycle.".to_string(),
            route_key: "content_source_registry".to_string(),
            urgency: "high".to_string(),
            rationale: format!("{} source entries are currently stale.", stale_source_count),
        });
    }
    if mission_review_required_count > 0 {
        action_recommendations.push(ActionRecommendationDto {
            recommendation_key: "review_content_missions".to_string(),
            audience: "admin".to_string(),
            label: "Review gated research missions".to_string(),
            summary: "Research missions are waiting for human attention before they can become trusted content.".to_string(),
            route_key: "content_research_board".to_string(),
            urgency: "high".to_string(),
            rationale: format!(
                "{} mission(s) are waiting in review_required.",
                mission_review_required_count
            ),
        });
    }
    if blocked_publish_count > 0 {
        action_recommendations.push(ActionRecommendationDto {
            recommendation_key: "inspect_publish_rejections".to_string(),
            audience: "admin".to_string(),
            label: "Inspect rejected or rolled-back publish decisions".to_string(),
            summary: "The control plane has recorded publish failures that should feed back into source policy or quality governance.".to_string(),
            route_key: "content_publish_decisions".to_string(),
            urgency: "medium".to_string(),
            rationale: format!(
                "{} publish decisions are currently rejected or rolled back.",
                blocked_publish_count
            ),
        });
    }
    if !low_quality_topics.is_empty() {
        action_recommendations.push(ActionRecommendationDto {
            recommendation_key: "audit_low_quality_topics".to_string(),
            audience: "admin".to_string(),
            label: "Audit low-quality topic bundles".to_string(),
            summary: "The content health layer has detected topics whose evaluation quality is drifting below a healthy threshold.".to_string(),
            route_key: "content_health".to_string(),
            urgency: "medium".to_string(),
            rationale: format!(
                "{} topics are currently below the preferred quality floor.",
                low_quality_topics.len()
            ),
        });
    }

    Ok(ContentHealthReadModelDto {
        generated_at: Utc::now().to_rfc3339(),
        source_count,
        stale_source_count,
        overdue_source_review_count,
        active_mission_count,
        mission_review_required_count,
        blocked_publish_count,
        preview_publish_count,
        average_quality_bp: average_quality_bp.max(average_source_quality.round() as i64),
        low_quality_topics,
        action_recommendations,
    })
}

fn build_admin_directives(
    oversight: &AdminOversightSnapshot,
    content_health: &ContentHealthReadModelDto,
) -> Vec<CoachDirectiveDto> {
    let mut directives = vec![CoachDirectiveDto {
        directive_type: "admin_watchlist".to_string(),
        audience: "admin".to_string(),
        title: "Super-admin control plane summary".to_string(),
        summary: format!(
            "{} students are critical and {} households need attention.",
            oversight.critical_students, oversight.households_needing_attention
        ),
        priority: if oversight.critical_students > 0 {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        primary_action: "admin_oversight".to_string(),
        supporting_signals: vec![
            format!("critical_students:{}", oversight.critical_students),
            format!(
                "households_needing_attention:{}",
                oversight.households_needing_attention
            ),
        ],
    }];
    if content_health.stale_source_count > 0
        || content_health.mission_review_required_count > 0
        || content_health.blocked_publish_count > 0
    {
        directives.push(CoachDirectiveDto {
            directive_type: "content_governance_watch".to_string(),
            audience: "admin".to_string(),
            title: "Content OS governance needs attention".to_string(),
            summary: "The content control plane has stale sources, queued research, or blocked publish signals that should be reviewed.".to_string(),
            priority: "high".to_string(),
            primary_action: "content_health".to_string(),
            supporting_signals: vec![
                format!("stale_sources:{}", content_health.stale_source_count),
                format!(
                    "review_required_missions:{}",
                    content_health.mission_review_required_count
                ),
                format!("blocked_publish:{}", content_health.blocked_publish_count),
            ],
        });
    }
    directives
}

fn build_admin_insight_cards(
    oversight: &AdminOversightSnapshot,
    content_health: &ContentHealthReadModelDto,
) -> Vec<InsightCardDto> {
    vec![
        InsightCardDto {
            card_key: "admin_student_pressure".to_string(),
            audience: "admin".to_string(),
            title: "Student pressure overview".to_string(),
            summary: format!(
                "The oversight layer currently tracks {} total students with {} active interventions.",
                oversight.total_students, oversight.active_interventions
            ),
            tone: "operational".to_string(),
            metric_label: Some("critical_students".to_string()),
            metric_value: Some(oversight.critical_students as i64),
            tags: vec!["ops".to_string(), "readiness".to_string()],
        },
        InsightCardDto {
            card_key: "admin_content_health".to_string(),
            audience: "admin".to_string(),
            title: "Content health".to_string(),
            summary: format!(
                "{} sources, {} stale, average quality {} bp.",
                content_health.source_count,
                content_health.stale_source_count,
                content_health.average_quality_bp
            ),
            tone: "operational".to_string(),
            metric_label: Some("active_mission_count".to_string()),
            metric_value: Some(content_health.active_mission_count),
            tags: vec!["content_os".to_string(), "governance".to_string()],
        },
    ]
}

fn build_admin_action_recommendations(
    oversight: &AdminOversightSnapshot,
    content_health: &ContentHealthReadModelDto,
) -> Vec<ActionRecommendationDto> {
    let mut actions = content_health.action_recommendations.clone();
    if oversight.critical_students > 0 {
        actions.insert(
            0,
            ActionRecommendationDto {
                recommendation_key: "triage_critical_students".to_string(),
                audience: "admin".to_string(),
                label: "Triage critical learners first".to_string(),
                summary: "The admin surface is ranking pressure by real readiness and risk signals, not just raw scores.".to_string(),
                route_key: "admin_oversight".to_string(),
                urgency: "high".to_string(),
                rationale: format!(
                    "{} critical learners are currently on the watchlist.",
                    oversight.critical_students
                ),
            },
        );
    }
    actions
}

fn build_admin_audience_explanations(
    oversight: &AdminOversightSnapshot,
    content_health: &ContentHealthReadModelDto,
) -> Vec<AudienceExplanationDto> {
    vec![
        AudienceExplanationDto {
            audience: "admin".to_string(),
            headline: "This admin surface is intentionally operational.".to_string(),
            summary: "It combines learner readiness oversight with content-system governance so super-admin work stays tied to real student outcomes.".to_string(),
            supporting_points: vec![
                format!("Critical students: {}", oversight.critical_students),
                format!("Stale sources: {}", content_health.stale_source_count),
            ],
        },
        AudienceExplanationDto {
            audience: "parent".to_string(),
            headline: "Parent outputs stay translated even when admin views go deeper.".to_string(),
            summary: "The control plane can see pressure and governance detail without leaking raw operational noise into the parent surface.".to_string(),
            supporting_points: vec![
                "Parents receive curated summaries; admins receive operational truth.".to_string(),
            ],
        },
        AudienceExplanationDto {
            audience: "student".to_string(),
            headline: "Students feel the intervention, not the control plane.".to_string(),
            summary: "Admin governance improves the quality of the coaching experience without exposing internal tuning mechanics.".to_string(),
            supporting_points: vec![
                "This keeps student experience clean while maintaining oversight.".to_string(),
            ],
        },
    ]
}

fn build_capability_registry() -> Vec<CapabilityRegistryEntryDto> {
    vec![
        CapabilityRegistryEntryDto {
            signal_key: "knowledge_gap".to_string(),
            capability_key: "repair_foundation".to_string(),
            tool_key: "teach_mode".to_string(),
            tool_label: "Teach mode and repair lesson".to_string(),
            route_key: "teach_mode".to_string(),
            intended_audience: "student".to_string(),
            explanation: "Route clean knowledge gaps into direct explanation and structured repair instead of random extra practice.".to_string(),
        },
        CapabilityRegistryEntryDto {
            signal_key: "conceptual_confusion".to_string(),
            capability_key: "disambiguate_similar_concepts".to_string(),
            tool_key: "contrast_mode".to_string(),
            tool_label: "Contrast drill / Traps mode".to_string(),
            route_key: "contrast_mode".to_string(),
            intended_audience: "student".to_string(),
            explanation: "Confusion states should map to contrast-first tools that separate nearby concepts before speed work resumes.".to_string(),
        },
        CapabilityRegistryEntryDto {
            signal_key: "recognition_failure".to_string(),
            capability_key: "rebuild_recall_path".to_string(),
            tool_key: "memory_rescue".to_string(),
            tool_label: "Memory rescue and recall checks".to_string(),
            route_key: "memory_dashboard".to_string(),
            intended_audience: "student".to_string(),
            explanation: "Recognition-only strength should trigger recall-oriented rescue rather than more recognition practice.".to_string(),
        },
        CapabilityRegistryEntryDto {
            signal_key: "pressure_breakdown".to_string(),
            capability_key: "stabilize_under_time".to_string(),
            tool_key: "timed_ladder".to_string(),
            tool_label: "Timed ladder and mock pressure probes".to_string(),
            route_key: "mock_centre".to_string(),
            intended_audience: "student".to_string(),
            explanation: "Pressure issues should map into controlled timing and resilience probes instead of more untimed comfort work.".to_string(),
        },
        CapabilityRegistryEntryDto {
            signal_key: "expression_weakness".to_string(),
            capability_key: "upgrade_written_method".to_string(),
            tool_key: "answer_construction_lab".to_string(),
            tool_label: "Answer Construction Lab".to_string(),
            route_key: "answer_construction".to_string(),
            intended_audience: "student".to_string(),
            explanation: "Expression problems need structured written-response coaching, not just more multiple choice.".to_string(),
        },
        CapabilityRegistryEntryDto {
            signal_key: "low_confidence_evidence".to_string(),
            capability_key: "verify_uploaded_evidence".to_string(),
            tool_key: "evidence_inbox".to_string(),
            tool_label: "Evidence Inbox and confirmation flow".to_string(),
            route_key: "evidence_inbox".to_string(),
            intended_audience: "coach".to_string(),
            explanation: "Weak evidence confidence should map to confirmation and review tools before major plan changes are committed.".to_string(),
        },
        CapabilityRegistryEntryDto {
            signal_key: "readiness_gap".to_string(),
            capability_key: "rebalance_plan".to_string(),
            tool_key: "weekly_planner".to_string(),
            tool_label: "Weekly plan generator".to_string(),
            route_key: "weekly_plan".to_string(),
            intended_audience: "student".to_string(),
            explanation: "Readiness gaps should translate into plan composition changes that target the missing component, not generic harder work.".to_string(),
        },
        CapabilityRegistryEntryDto {
            signal_key: "content_governance".to_string(),
            capability_key: "protect_trusted_content".to_string(),
            tool_key: "content_health".to_string(),
            tool_label: "Content health and publish governance".to_string(),
            route_key: "content_health".to_string(),
            intended_audience: "admin".to_string(),
            explanation: "Governance signals should route into source policy, evaluation, and publish-decision tooling rather than student-facing flows.".to_string(),
        },
    ]
}

fn build_semantics_registry() -> SemanticsRegistryDto {
    SemanticsRegistryDto {
        generated_at: Utc::now().to_rfc3339(),
        thresholds: ThresholdRegistry::default(),
        readiness_bands: vec![
            SemanticBandDto {
                registry_key: "critical".to_string(),
                label: "Critical".to_string(),
                min_bp: None,
                max_bp: Some(3_999),
                explanation: "The learner is not ready and the coach should prioritize rescue over acceleration.".to_string(),
            },
            SemanticBandDto {
                registry_key: "fragile".to_string(),
                label: "Fragile".to_string(),
                min_bp: Some(4_000),
                max_bp: Some(5_499),
                explanation: "The learner has partial traction but the state is still unstable.".to_string(),
            },
            SemanticBandDto {
                registry_key: "building".to_string(),
                label: "Building".to_string(),
                min_bp: Some(5_500),
                max_bp: Some(8_499),
                explanation: "The learner is progressing, but protection and targeted repair still matter.".to_string(),
            },
            SemanticBandDto {
                registry_key: "exam_ready".to_string(),
                label: "Exam Ready".to_string(),
                min_bp: Some(8_500),
                max_bp: Some(10_000),
                explanation: "The learner is demonstrating a strong enough state for exam-facing execution.".to_string(),
            },
        ],
        mastery_bands: vec![
            SemanticBandDto {
                registry_key: "exposed".to_string(),
                label: "Exposed".to_string(),
                min_bp: None,
                max_bp: Some(3_999),
                explanation: "The concept has been seen but cannot yet be trusted.".to_string(),
            },
            SemanticBandDto {
                registry_key: "fragile".to_string(),
                label: "Fragile".to_string(),
                min_bp: Some(4_000),
                max_bp: Some(7_199),
                explanation: "The concept is partially learned but can still collapse.".to_string(),
            },
            SemanticBandDto {
                registry_key: "stable".to_string(),
                label: "Stable".to_string(),
                min_bp: Some(7_200),
                max_bp: Some(8_999),
                explanation: "The learner is reliably handling the concept in normal conditions.".to_string(),
            },
            SemanticBandDto {
                registry_key: "exam_ready".to_string(),
                label: "Exam Ready".to_string(),
                min_bp: Some(9_000),
                max_bp: Some(10_000),
                explanation: "The concept is operating at the highest governed mastery threshold.".to_string(),
            },
        ],
        confidence_bands: vec![
            SemanticBandDto {
                registry_key: "low".to_string(),
                label: "Low confidence".to_string(),
                min_bp: None,
                max_bp: Some(4_999),
                explanation: "The system should avoid high-impact decisions until more evidence lands.".to_string(),
            },
            SemanticBandDto {
                registry_key: "medium".to_string(),
                label: "Medium confidence".to_string(),
                min_bp: Some(5_000),
                max_bp: Some(7_499),
                explanation: "The signal is usable, but contradiction checks still matter.".to_string(),
            },
            SemanticBandDto {
                registry_key: "high".to_string(),
                label: "High confidence".to_string(),
                min_bp: Some(7_500),
                max_bp: Some(10_000),
                explanation: "The system can safely let this evidence drive stronger behavior changes.".to_string(),
            },
        ],
        severity_bands: vec![
            SemanticBandDto {
                registry_key: "low".to_string(),
                label: "Low".to_string(),
                min_bp: None,
                max_bp: Some(3_999),
                explanation: "Monitor the issue without overreacting.".to_string(),
            },
            SemanticBandDto {
                registry_key: "medium".to_string(),
                label: "Medium".to_string(),
                min_bp: Some(4_000),
                max_bp: Some(6_999),
                explanation: "The issue needs visible response but not crisis handling.".to_string(),
            },
            SemanticBandDto {
                registry_key: "high".to_string(),
                label: "High".to_string(),
                min_bp: Some(7_000),
                max_bp: Some(8_999),
                explanation: "The issue should alter planning or coaching behavior quickly.".to_string(),
            },
            SemanticBandDto {
                registry_key: "critical".to_string(),
                label: "Critical".to_string(),
                min_bp: Some(9_000),
                max_bp: Some(10_000),
                explanation: "The system should escalate and protect the learner from false progress.".to_string(),
            },
        ],
        freshness_rules: vec![
            EvidenceFreshnessRuleDto {
                label: "fresh".to_string(),
                min_days: 0,
                max_days: Some(7),
                explanation: "Fresh evidence should be weighted most strongly in planning.".to_string(),
            },
            EvidenceFreshnessRuleDto {
                label: "recent".to_string(),
                min_days: 8,
                max_days: Some(21),
                explanation: "Recent evidence is still actionable but should be checked for confirmation.".to_string(),
            },
            EvidenceFreshnessRuleDto {
                label: "aging".to_string(),
                min_days: 22,
                max_days: Some(45),
                explanation: "Aging evidence should influence planning with caution unless repeated.".to_string(),
            },
            EvidenceFreshnessRuleDto {
                label: "stale".to_string(),
                min_days: 46,
                max_days: None,
                explanation: "Stale evidence should not dominate new decisions without refresh.".to_string(),
            },
        ],
    }
}

fn directive_type_for_action(action_type: CoachActionType) -> String {
    match action_type {
        CoachActionType::ContinueOnboarding => "continue_onboarding",
        CoachActionType::SelectSubjects => "select_subjects",
        CoachActionType::ResolveContent => "resolve_content",
        CoachActionType::StartDiagnostic => "schedule_probe",
        CoachActionType::GeneratePlan => "build_plan",
        CoachActionType::StartTodayMission => "start_today_mission",
        CoachActionType::ResumeMission => "resume_session",
        CoachActionType::ReviewResults => "review_results",
        CoachActionType::StartRepair => "rescue_foundation_first",
        CoachActionType::AdjustPlan => "adjust_plan",
        CoachActionType::ViewOverview => "view_overview",
    }
    .to_string()
}

fn directive_priority_for_state(action_type: CoachActionType) -> String {
    match action_type {
        CoachActionType::StartRepair | CoachActionType::ResolveContent => "high",
        CoachActionType::StartDiagnostic | CoachActionType::AdjustPlan => "medium",
        _ => "medium",
    }
    .to_string()
}

fn normalize_key(value: &str) -> String {
    value.to_ascii_lowercase().replace(' ', "_")
}

trait JourneyStateLabel {
    fn as_str(&self) -> &'static str;
}

impl JourneyStateLabel for ecoach_coach_brain::LearnerJourneyState {
    fn as_str(&self) -> &'static str {
        match self {
            ecoach_coach_brain::LearnerJourneyState::OnboardingRequired => "onboarding_required",
            ecoach_coach_brain::LearnerJourneyState::SubjectSelectionRequired => {
                "subject_selection_required"
            }
            ecoach_coach_brain::LearnerJourneyState::ContentReadinessRequired => {
                "content_readiness_required"
            }
            ecoach_coach_brain::LearnerJourneyState::DiagnosticRequired => "diagnostic_required",
            ecoach_coach_brain::LearnerJourneyState::PlanGenerationRequired => {
                "plan_generation_required"
            }
            ecoach_coach_brain::LearnerJourneyState::ReadyForTodayMission => {
                "ready_for_today_mission"
            }
            ecoach_coach_brain::LearnerJourneyState::MissionInProgress => "mission_in_progress",
            ecoach_coach_brain::LearnerJourneyState::MissionReviewRequired => {
                "mission_review_required"
            }
            ecoach_coach_brain::LearnerJourneyState::RepairRequired => "repair_required",
            ecoach_coach_brain::LearnerJourneyState::BlockedOnTopic => "blocked_on_topic",
            ecoach_coach_brain::LearnerJourneyState::PlanAdjustmentRequired => {
                "plan_adjustment_required"
            }
            ecoach_coach_brain::LearnerJourneyState::ReviewDay => "review_day",
            ecoach_coach_brain::LearnerJourneyState::ExamMode => "exam_mode",
            ecoach_coach_brain::LearnerJourneyState::StalledNoContent => "stalled_no_content",
        }
    }
}

#[derive(Debug, Clone)]
struct BundleVaultRow {
    bundle_id: i64,
    title: String,
    status: String,
    created_at: String,
    updated_at: String,
    confirmation_state: String,
    coach_application_status: String,
    shared_promotion_status: Option<String>,
}

#[derive(Debug, Clone)]
struct ARealProfileRow {
    tone_style: String,
    motivation_style: String,
    urgency_style: String,
    explanation_style: String,
    narrative_enabled: bool,
}

fn build_entitlement_manager_snapshot(
    conn: &Connection,
    limit: usize,
) -> Result<EntitlementManagerSnapshotDto, EcoachError> {
    let mut statement = conn
        .prepare(
            "SELECT id, account_type, display_name, entitlement_tier, status, last_active_at
             FROM accounts
             ORDER BY datetime(updated_at) DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map(params![limit.max(1) as i64], |row| {
            Ok(ManagedEntitlementAccountDto {
                account_id: row.get(0)?,
                account_type: row.get(1)?,
                display_name: row.get(2)?,
                entitlement_tier: row.get(3)?,
                status: row.get(4)?,
                last_active_at: row.get(5)?,
                linked_student_ids: Vec::new(),
                feature_unlocks: Vec::new(),
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut accounts = Vec::new();
    let mut premium_count = 0i64;
    let mut elite_count = 0i64;
    let mut inactive_count = 0i64;
    for row in rows {
        let mut account = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
        if account.entitlement_tier == "premium" {
            premium_count += 1;
        }
        if account.entitlement_tier == "elite" {
            elite_count += 1;
        }
        if account.status != "active" {
            inactive_count += 1;
        }
        if account.account_type == "parent" {
            account.linked_student_ids = load_linked_student_ids(conn, account.account_id)?;
        }
        account.feature_unlocks =
            feature_unlocks_for(&account.account_type, &account.entitlement_tier);
        accounts.push(account);
    }

    let audit_log = IdentityService::new(conn)
        .list_entitlement_audit_entries(limit.min(12))?
        .into_iter()
        .map(|entry| EntitlementAuditEntryDto {
            id: entry.id,
            account_id: entry.account_id,
            changed_by_account_id: entry.changed_by_account_id,
            previous_tier: entry.previous_tier,
            new_tier: entry.new_tier,
            previous_status: entry.previous_status,
            new_status: entry.new_status,
            reason: entry.reason,
            created_at: entry.created_at,
        })
        .collect();

    Ok(EntitlementManagerSnapshotDto {
        generated_at: Utc::now().to_rfc3339(),
        premium_count,
        elite_count,
        inactive_count,
        accounts,
        audit_log,
    })
}

fn build_personal_academic_vault(
    conn: &Connection,
    student_id: i64,
    limit: usize,
) -> Result<PersonalAcademicVaultDto, EcoachError> {
    let student_name: String = conn
        .query_row(
            "SELECT display_name FROM accounts WHERE id = ?1",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let bundle_rows = load_recent_bundle_rows(conn, student_id, limit)?;
    let intake = IntakeService::new(conn);
    let inbox = intake.list_bundle_inbox(student_id, limit.max(1) * 2)?;

    let mut active_topics = BTreeSet::new();
    let mut recurring_mistake_signals = BTreeSet::new();
    let mut teacher_style_signals = BTreeSet::new();
    let mut urgent_review_bundle_count = 0i64;
    let mut ocr_attention_count = 0i64;
    let mut pending_shared_promotion_count = 0i64;
    let mut bundles = Vec::new();

    for row in bundle_rows {
        let report = intake.get_bundle_report(row.bundle_id)?;
        let files = intake.list_bundle_files(row.bundle_id)?;
        let insights = intake.list_bundle_insights(row.bundle_id)?;
        let inbox_item = inbox.iter().find(|item| item.bundle.id == row.bundle_id);
        let review_priority = inbox_item
            .map(|item| item.review_priority.clone())
            .unwrap_or_else(|| report.review_priority.clone());
        let summary_points = inbox_item
            .map(|item| item.summary_points.clone())
            .unwrap_or_else(|| {
                report
                    .recommended_actions
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
            });

        if review_priority == "high" || report.needs_confirmation {
            urgent_review_bundle_count += 1;
        }
        if matches!(
            row.shared_promotion_status.as_deref(),
            Some("queued" | "approved")
        ) {
            pending_shared_promotion_count += 1;
        }

        for topic in &report.detected_topics {
            active_topics.insert(topic.clone());
        }
        for weakness in &report.weakness_signals {
            recurring_mistake_signals.insert(weakness.clone());
        }

        let mut file_dtos = Vec::new();
        for file in files {
            let insight_payload = insights
                .iter()
                .filter(|insight| insight.insight_type == "file_reconstruction")
                .find(|insight| {
                    insight.payload.get("file_id").and_then(Value::as_i64) == Some(file.id)
                        || insight.payload.get("file_name").and_then(Value::as_str)
                            == Some(file.file_name.as_str())
                })
                .map(|insight| &insight.payload);
            let dto = build_vault_file_dto(&file, insight_payload);
            if ocr_object_needs_attention(&dto) {
                ocr_attention_count += 1;
            }
            if dto.document_origin.as_deref() == Some("teacher_provided")
                || matches!(
                    dto.document_role.as_deref(),
                    Some("teacher_handout" | "question_paper" | "corrected_script")
                )
            {
                for pattern in &dto.question_patterns {
                    teacher_style_signals.insert(pattern.clone());
                }
            }
            file_dtos.push(dto);
        }

        bundles.push(AcademicVaultBundleDto {
            bundle_id: row.bundle_id,
            title: row.title,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            confirmation_state: row.confirmation_state,
            coach_application_status: row.coach_application_status,
            review_priority,
            bundle_kind: report.bundle_kind,
            shared_promotion_status: row.shared_promotion_status,
            detected_subjects: report.detected_subjects,
            detected_topics: report.detected_topics,
            weakness_signals: report.weakness_signals,
            recommended_actions: report.recommended_actions,
            summary_points,
            files: file_dtos,
        });
    }

    Ok(PersonalAcademicVaultDto {
        student_id,
        student_name,
        generated_at: Utc::now().to_rfc3339(),
        active_topics: active_topics.into_iter().take(12).collect(),
        recurring_mistake_signals: recurring_mistake_signals.into_iter().take(12).collect(),
        teacher_style_signals: teacher_style_signals.into_iter().take(12).collect(),
        urgent_review_bundle_count,
        ocr_attention_count,
        pending_shared_promotion_count,
        bundles,
    })
}

fn build_areal_surface(conn: &Connection, student_id: i64) -> Result<ARealSurfaceDto, EcoachError> {
    let profile = ensure_areal_profile(conn, student_id)?;
    let truth = StudentModelService::new(conn).get_learner_truth_snapshot(student_id)?;
    let dashboard = DashboardService::new(conn).get_student_dashboard(student_id)?;
    let coach_output =
        evaluate_coach_brain(conn, student_id, CoachBrainTrigger::ManualRefresh, 14)?;
    let vault = build_personal_academic_vault(conn, student_id, 3)?;

    let entitlement_tier: String = conn
        .query_row(
            "SELECT entitlement_tier FROM accounts WHERE id = ?1",
            [student_id],
            |row| row.get(0),
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let current_mode = areal_mode_for(
        &entitlement_tier,
        &dashboard.overall_readiness_band,
        truth.pending_review_count,
        truth.due_memory_count,
        !vault.bundles.is_empty(),
        &coach_output.next_action.route,
    );
    let headline = if current_mode == "upload_interpreter" && !vault.bundles.is_empty() {
        "I turned your recent schoolwork into a plan".to_string()
    } else {
        coach_output.next_action.title.clone()
    };
    let summary = build_areal_summary(
        &dashboard.overall_readiness_band,
        &coach_output.next_action.subtitle,
        &vault,
        profile.narrative_enabled,
    );

    conn.execute(
        "INSERT INTO areal_guidance_events (
            student_id, coach_state, current_mode, headline, summary, route_key
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            student_id,
            coach_output.state.state.as_str(),
            current_mode,
            headline,
            summary,
            coach_output.next_action.route,
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut active_signals = vec![format!("readiness:{}", dashboard.overall_readiness_band)];
    if truth.pending_review_count > 0 {
        active_signals.push(format!("pending_review:{}", truth.pending_review_count));
    }
    if truth.due_memory_count > 0 {
        active_signals.push(format!("memory_due:{}", truth.due_memory_count));
    }
    if let Some(first_signal) = vault.recurring_mistake_signals.first() {
        active_signals.push(format!("recurring:{}", first_signal));
    }

    let mut upload_context = Vec::new();
    if let Some(bundle) = vault.bundles.first() {
        upload_context.extend(bundle.summary_points.iter().take(2).cloned());
        upload_context.extend(bundle.detected_topics.iter().take(2).cloned());
    }

    let mut recommended_actions = dedup_strings(
        std::iter::once(coach_output.next_action.route.clone())
            .chain(
                vault
                    .bundles
                    .first()
                    .into_iter()
                    .flat_map(|bundle| bundle.recommended_actions.iter().cloned().take(3)),
            )
            .collect(),
    );
    if recommended_actions.is_empty() {
        recommended_actions.push("coach_hub".to_string());
    }

    Ok(ARealSurfaceDto {
        student_id,
        student_name: truth.student_name,
        generated_at: Utc::now().to_rfc3339(),
        current_mode,
        tone_style: profile.tone_style,
        motivation_style: profile.motivation_style,
        urgency_style: profile.urgency_style,
        explanation_style: profile.explanation_style,
        headline,
        summary,
        next_route_key: coach_output.next_action.route,
        active_signals,
        upload_context,
        recommended_actions,
        recent_guidance: load_recent_areal_guidance(conn, Some(student_id), 5)?,
    })
}

fn build_super_admin_control_tower(
    conn: &Connection,
    admin_id: i64,
    limit: usize,
) -> Result<SuperAdminControlTowerDto, EcoachError> {
    let oversight = AdminOversightService::new(conn).build_admin_oversight_snapshot(admin_id)?;
    let content_health = build_content_health_read_model(conn)?;
    let entitlement_manager = build_entitlement_manager_snapshot(conn, limit.max(8))?;
    let source_objects = load_source_governance_objects(conn, limit)?;
    let ocr_recovery_objects = load_ocr_recovery_objects(conn, limit)?;
    let pending_shared_promotions = load_shared_promotions(conn, limit)?;
    let recent_areal_guidance = load_recent_areal_guidance(conn, None, limit)?;

    let mut action_recommendations = content_health.action_recommendations.clone();
    if source_objects
        .iter()
        .any(|item| item.unresolved_review_count > 0)
    {
        push_action_recommendation(
            &mut action_recommendations,
            ActionRecommendationDto {
                recommendation_key: "review_source_queue".to_string(),
                audience: "admin".to_string(),
                label: "Clear source review queue".to_string(),
                summary: "Content ingestion is ahead of governance, so unresolved review work should be cleared before more publishing.".to_string(),
                route_key: "content_foundry".to_string(),
                urgency: "high".to_string(),
                rationale: "idea38 requires super-admin approval and refinement before trust promotion.".to_string(),
            },
        );
    }
    if !ocr_recovery_objects.is_empty() {
        push_action_recommendation(
            &mut action_recommendations,
            ActionRecommendationDto {
                recommendation_key: "triage_ocr_queue".to_string(),
                audience: "admin".to_string(),
                label: "Triage OCR recovery".to_string(),
                summary: "Personal uploads still need OCR or layout attention before they can cleanly feed either the student model or the shared corpus.".to_string(),
                route_key: "personal_vault_review".to_string(),
                urgency: "high".to_string(),
                rationale: "idea38 treats OCR and layout recovery as first-class operating objects.".to_string(),
            },
        );
    }
    if !pending_shared_promotions.is_empty() {
        push_action_recommendation(
            &mut action_recommendations,
            ActionRecommendationDto {
                recommendation_key: "review_shared_promotions".to_string(),
                audience: "admin".to_string(),
                label: "Review personal-to-shared promotions".to_string(),
                summary: "Student materials are waiting for abstraction and approval before entering the shared content operating system.".to_string(),
                route_key: "content_governance".to_string(),
                urgency: "medium".to_string(),
                rationale: "idea38 explicitly keeps personal academic vault data separate from the shared corpus until reviewed.".to_string(),
            },
        );
    }
    if entitlement_manager.inactive_count > 0 {
        push_action_recommendation(
            &mut action_recommendations,
            ActionRecommendationDto {
                recommendation_key: "audit_entitlements".to_string(),
                audience: "admin".to_string(),
                label: "Audit access policy changes".to_string(),
                summary: "There are non-active or recently changed accounts, so the control tower should verify premium and elite policies are intentional.".to_string(),
                route_key: "entitlement_manager".to_string(),
                urgency: "medium".to_string(),
                rationale: "idea38 defines premium and elite as policy layers applied through the same shared system.".to_string(),
            },
        );
    }

    Ok(SuperAdminControlTowerDto {
        admin_id: oversight.admin_id,
        admin_name: oversight.admin_name,
        generated_at: Utc::now().to_rfc3339(),
        entitlement_manager,
        content_health,
        source_objects,
        ocr_recovery_objects,
        pending_shared_promotions,
        recent_areal_guidance,
        action_recommendations,
    })
}

fn load_recent_bundle_rows(
    conn: &Connection,
    student_id: i64,
    limit: usize,
) -> Result<Vec<BundleVaultRow>, EcoachError> {
    let mut statement = conn
        .prepare(
            "SELECT b.id, b.title, b.status, b.created_at, b.updated_at,
                    b.confirmation_state, b.coach_application_status,
                    (
                        SELECT promotion_status
                        FROM bundle_shared_promotions p
                        WHERE p.bundle_id = b.id
                        ORDER BY datetime(p.updated_at) DESC, p.id DESC
                        LIMIT 1
                    ) AS shared_promotion_status
             FROM submission_bundles b
             WHERE b.student_id = ?1
             ORDER BY datetime(b.updated_at) DESC, b.id DESC
             LIMIT ?2",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map(params![student_id, limit.max(1) as i64], |row| {
            Ok(BundleVaultRow {
                bundle_id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                confirmation_state: row.get(5)?,
                coach_application_status: row.get(6)?,
                shared_promotion_status: row.get(7)?,
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut bundles = Vec::new();
    for row in rows {
        bundles.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(bundles)
}

fn build_vault_file_dto(
    file: &ecoach_intake::BundleFile,
    insight_payload: Option<&Value>,
) -> AcademicVaultFileDto {
    let document_role = insight_payload
        .and_then(|payload| payload.get("document_role"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let document_origin = insight_payload
        .and_then(|payload| payload.get("document_origin"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let ocr_status = insight_payload
        .and_then(|payload| payload.pointer("/ocr_recovery/status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let layout_status = insight_payload
        .and_then(|payload| payload.pointer("/layout_recovery/status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let layout_kind = insight_payload
        .and_then(|payload| payload.pointer("/layout_recovery/kind"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let detected_topics = insight_payload
        .map(|payload| {
            let from_pointer =
                json_strings_at_pointer(payload, "/document_intelligence/detected_topics");
            if from_pointer.is_empty() {
                json_strings_from_key(payload, "detected_topics")
            } else {
                from_pointer
            }
        })
        .unwrap_or_default();
    let question_patterns = insight_payload
        .map(|payload| {
            let from_pointer =
                json_strings_at_pointer(payload, "/document_intelligence/question_patterns");
            if from_pointer.is_empty() {
                json_strings_from_key(payload, "question_patterns")
            } else {
                from_pointer
            }
        })
        .unwrap_or_default();
    let review_reasons = insight_payload
        .map(|payload| json_strings_from_key(payload, "review_reasons"))
        .unwrap_or_default();

    AcademicVaultFileDto {
        file_id: file.id,
        file_name: file.file_name.clone(),
        file_kind: file.file_kind.clone(),
        document_role,
        document_origin,
        ocr_status,
        layout_status,
        layout_kind,
        detected_topics,
        question_patterns,
        review_reasons,
    }
}

fn ocr_object_needs_attention(item: &AcademicVaultFileDto) -> bool {
    if matches!(
        item.ocr_status.as_str(),
        "required" | "pending_ocr" | "limited" | "unknown"
    ) {
        return true;
    }
    item.review_reasons.iter().any(|reason| {
        matches!(
            reason.as_str(),
            "ocr_required" | "low_ocr_confidence" | "weak_question_layout" | "weak_answer_layout"
        )
    })
}

fn ensure_areal_profile(
    conn: &Connection,
    student_id: i64,
) -> Result<ARealProfileRow, EcoachError> {
    conn.execute(
        "INSERT INTO areal_profiles (student_id) VALUES (?1)
         ON CONFLICT(student_id) DO NOTHING",
        [student_id],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;

    conn.query_row(
        "SELECT tone_style, motivation_style, urgency_style, explanation_style, narrative_enabled
         FROM areal_profiles
         WHERE student_id = ?1",
        [student_id],
        |row| {
            Ok(ARealProfileRow {
                tone_style: row.get(0)?,
                motivation_style: row.get(1)?,
                urgency_style: row.get(2)?,
                explanation_style: row.get(3)?,
                narrative_enabled: row.get::<_, i64>(4)? == 1,
            })
        },
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))
}

fn upsert_areal_profile(
    conn: &Connection,
    input: ARealProfileInputDto,
) -> Result<ARealProfileDto, EcoachError> {
    let existing = ensure_areal_profile(conn, input.student_id)?;
    conn.execute(
        "UPDATE areal_profiles
         SET tone_style = ?2,
             motivation_style = ?3,
             urgency_style = ?4,
             explanation_style = ?5,
             narrative_enabled = ?6,
             updated_at = datetime('now')
         WHERE student_id = ?1",
        params![
            input.student_id,
            input.tone_style.unwrap_or(existing.tone_style),
            input.motivation_style.unwrap_or(existing.motivation_style),
            input.urgency_style.unwrap_or(existing.urgency_style),
            input
                .explanation_style
                .unwrap_or(existing.explanation_style),
            if input
                .narrative_enabled
                .unwrap_or(existing.narrative_enabled)
            {
                1
            } else {
                0
            },
        ],
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))?;

    conn.query_row(
        "SELECT student_id, tone_style, motivation_style, urgency_style, explanation_style,
                narrative_enabled, updated_at
         FROM areal_profiles
         WHERE student_id = ?1",
        [input.student_id],
        |row| {
            Ok(ARealProfileDto {
                student_id: row.get(0)?,
                tone_style: row.get(1)?,
                motivation_style: row.get(2)?,
                urgency_style: row.get(3)?,
                explanation_style: row.get(4)?,
                narrative_enabled: row.get::<_, i64>(5)? == 1,
                updated_at: row.get(6)?,
            })
        },
    )
    .map_err(|err| EcoachError::Storage(err.to_string()))
}

fn load_recent_areal_guidance(
    conn: &Connection,
    student_id: Option<i64>,
    limit: usize,
) -> Result<Vec<ARealGuidanceEventDto>, EcoachError> {
    let sql = if student_id.is_some() {
        "SELECT e.student_id, a.display_name, e.headline, e.summary, e.route_key, e.created_at
         FROM areal_guidance_events e
         JOIN accounts a ON a.id = e.student_id
         WHERE e.student_id = ?1
         ORDER BY e.id DESC
         LIMIT ?2"
    } else {
        "SELECT e.student_id, a.display_name, e.headline, e.summary, e.route_key, e.created_at
         FROM areal_guidance_events e
         JOIN accounts a ON a.id = e.student_id
         ORDER BY e.id DESC
         LIMIT ?1"
    };
    let mut statement = conn
        .prepare(sql)
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mapper = |row: &rusqlite::Row<'_>| -> rusqlite::Result<ARealGuidanceEventDto> {
        Ok(ARealGuidanceEventDto {
            student_id: Some(row.get(0)?),
            student_name: Some(row.get(1)?),
            headline: row.get(2)?,
            summary: row.get(3)?,
            route_key: row.get(4)?,
            created_at: row.get(5)?,
        })
    };

    let rows = if let Some(student_id) = student_id {
        statement
            .query_map(params![student_id, limit.max(1) as i64], mapper)
            .map_err(|err| EcoachError::Storage(err.to_string()))?
    } else {
        statement
            .query_map(params![limit.max(1) as i64], mapper)
            .map_err(|err| EcoachError::Storage(err.to_string()))?
    };

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(items)
}

fn load_source_governance_objects(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SourceGovernanceObjectDto>, EcoachError> {
    let mut statement = conn
        .prepare(
            "SELECT s.id, s.title, s.source_kind, s.source_status, s.parse_status_detail,
                    s.source_tier, s.trust_score_bp, s.stale_flag, s.review_due_at,
                    COALESCE((
                        SELECT COUNT(*)
                        FROM curriculum_review_tasks t
                        WHERE t.source_upload_id = s.id AND t.status != 'resolved'
                    ), 0) AS unresolved_review_count,
                    COALESCE((
                        SELECT COUNT(*)
                        FROM content_publish_decisions d
                        WHERE d.source_upload_id = s.id
                    ), 0) AS publish_decision_count,
                    (
                        SELECT note
                        FROM content_source_governance_events g
                        WHERE g.source_upload_id = s.id
                        ORDER BY g.id DESC
                        LIMIT 1
                    ) AS last_governance_note
             FROM curriculum_source_uploads s
             ORDER BY COALESCE(datetime(s.review_due_at), datetime(s.created_at)) ASC, s.id DESC
             LIMIT ?1",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map(params![limit.max(1) as i64], |row| {
            Ok(SourceGovernanceObjectDto {
                source_upload_id: row.get(0)?,
                title: row.get(1)?,
                source_kind: row.get(2)?,
                source_status: row.get(3)?,
                parse_status_detail: row.get(4)?,
                source_tier: row.get(5)?,
                trust_score_bp: row.get(6)?,
                stale_flag: row.get::<_, i64>(7)? == 1,
                review_due_at: row.get(8)?,
                unresolved_review_count: row.get(9)?,
                publish_decision_count: row.get(10)?,
                last_governance_note: row.get(11)?,
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut objects = Vec::new();
    for row in rows {
        objects.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(objects)
}

fn load_ocr_recovery_objects(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<OcrRecoveryObjectDto>, EcoachError> {
    let mut statement = conn
        .prepare(
            "SELECT ei.bundle_id, sb.student_id, a.display_name, ei.payload_json
             FROM extracted_insights ei
             JOIN submission_bundles sb ON sb.id = ei.bundle_id
             JOIN accounts a ON a.id = sb.student_id
             WHERE ei.insight_type = 'file_reconstruction'
             ORDER BY ei.id DESC
             LIMIT ?1",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map(params![limit.max(1) as i64 * 6], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut seen = BTreeSet::new();
    let mut objects = Vec::new();
    for row in rows {
        let (bundle_id, student_id, student_name, payload_json) =
            row.map_err(|err| EcoachError::Storage(err.to_string()))?;
        let payload = serde_json::from_str::<Value>(&payload_json)
            .map_err(|err| EcoachError::Serialization(err.to_string()))?;
        let file_id = match payload.get("file_id").and_then(Value::as_i64) {
            Some(value) => value,
            None => continue,
        };
        if !seen.insert((bundle_id, file_id)) {
            continue;
        }

        let item = OcrRecoveryObjectDto {
            bundle_id,
            student_id,
            student_name,
            file_id,
            file_name: payload
                .get("file_name")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            document_role: payload
                .get("document_role")
                .and_then(Value::as_str)
                .map(str::to_string),
            ocr_status: payload
                .pointer("/ocr_recovery/status")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            layout_status: payload
                .pointer("/layout_recovery/status")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            layout_kind: payload
                .pointer("/layout_recovery/kind")
                .and_then(Value::as_str)
                .map(str::to_string),
            review_priority: payload
                .get("review_priority")
                .and_then(Value::as_str)
                .unwrap_or("medium")
                .to_string(),
            review_reasons: json_strings_from_key(&payload, "review_reasons"),
        };

        if ocr_object_needs_attention(&AcademicVaultFileDto {
            file_id: item.file_id,
            file_name: item.file_name.clone(),
            file_kind: "bundle_file".to_string(),
            document_role: item.document_role.clone(),
            document_origin: None,
            ocr_status: item.ocr_status.clone(),
            layout_status: item.layout_status.clone(),
            layout_kind: item.layout_kind.clone(),
            detected_topics: Vec::new(),
            question_patterns: Vec::new(),
            review_reasons: item.review_reasons.clone(),
        }) {
            objects.push(item);
        }
        if objects.len() >= limit.max(1) {
            break;
        }
    }

    Ok(objects)
}

fn load_shared_promotions(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<SharedPromotionDto>, EcoachError> {
    let mut statement = conn
        .prepare(
            "SELECT id, bundle_id, source_upload_id, requested_by_account_id, promotion_status,
                    promotion_summary_json, created_at, updated_at
             FROM bundle_shared_promotions
             WHERE promotion_status != 'published'
             ORDER BY datetime(updated_at) DESC, id DESC
             LIMIT ?1",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map(params![limit.max(1) as i64], |row| {
            let summary_json: String = row.get(5)?;
            Ok(SharedPromotionDto {
                id: row.get(0)?,
                bundle_id: row.get(1)?,
                source_upload_id: row.get(2)?,
                requested_by_account_id: row.get(3)?,
                promotion_status: row.get(4)?,
                promotion_summary: serde_json::from_str(&summary_json).unwrap_or(Value::Null),
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut promotions = Vec::new();
    for row in rows {
        promotions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(promotions)
}

fn load_linked_student_ids(conn: &Connection, parent_id: i64) -> Result<Vec<i64>, EcoachError> {
    let mut statement = conn
        .prepare(
            "SELECT student_account_id
             FROM parent_student_links
             WHERE parent_account_id = ?1
             ORDER BY student_account_id ASC",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
    let rows = statement
        .query_map([parent_id], |row| row.get(0))
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    let mut ids = Vec::new();
    for row in rows {
        ids.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
    }
    Ok(ids)
}

fn feature_unlocks_for(account_type: &str, entitlement_tier: &str) -> Vec<String> {
    let mut features = match account_type {
        "student" => vec![
            "learning_workspace".to_string(),
            "personal_vault".to_string(),
            "coach_mode".to_string(),
            "dna_diagnostics".to_string(),
            "memory_tools".to_string(),
        ],
        "parent" => vec![
            "oversight_dashboard".to_string(),
            "readiness_views".to_string(),
            "alerts".to_string(),
            "weekly_reports".to_string(),
        ],
        "admin" => vec![
            "content_operating_system".to_string(),
            "quality_governance".to_string(),
            "curriculum_manager".to_string(),
            "ingestion_pipeline".to_string(),
            "publish_pipeline".to_string(),
            "entitlement_manager".to_string(),
        ],
        _ => Vec::new(),
    };

    match (account_type, entitlement_tier) {
        ("student", "elite") => features.extend([
            "advanced_prep".to_string(),
            "harder_pathways".to_string(),
            "deeper_diagnostics".to_string(),
            "stronger_coach_behavior".to_string(),
        ]),
        ("parent", "premium") | ("parent", "elite") => features.extend([
            "live_progress_sync".to_string(),
            "strategy_updates".to_string(),
            "exam_countdown_intelligence".to_string(),
            "concierge_monitoring".to_string(),
        ]),
        ("student", "premium") => features.push("richer_reports".to_string()),
        _ => {}
    }

    dedup_strings(features)
}

fn areal_mode_for(
    entitlement_tier: &str,
    readiness_band: &str,
    pending_review_count: i64,
    due_memory_count: i64,
    has_recent_uploads: bool,
    next_route_key: &str,
) -> String {
    if has_recent_uploads && next_route_key.contains("review") {
        return "upload_interpreter".to_string();
    }
    if pending_review_count > 0
        || due_memory_count > 0
        || matches!(readiness_band, "At Risk" | "Not Ready")
    {
        return "repair_and_stabilize".to_string();
    }
    if entitlement_tier == "elite"
        && (next_route_key.contains("exam") || next_route_key.contains("pressure"))
    {
        return "elite_push".to_string();
    }
    "steady_progress".to_string()
}

fn build_areal_summary(
    readiness_band: &str,
    next_action_summary: &str,
    vault: &PersonalAcademicVaultDto,
    narrative_enabled: bool,
) -> String {
    if !narrative_enabled {
        return next_action_summary.to_string();
    }

    let mut parts = vec![format!(
        "Your current readiness band is {}, so the coach is keeping the next move focused.",
        readiness_band
    )];
    if let Some(bundle) = vault.bundles.first() {
        if !bundle.detected_topics.is_empty() {
            parts.push(format!(
                "Your latest uploaded work is centered on {}.",
                bundle
                    .detected_topics
                    .iter()
                    .take(2)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !bundle.summary_points.is_empty() {
            parts.push(bundle.summary_points[0].clone());
        }
    }
    parts.push(next_action_summary.to_string());
    parts.join(" ")
}

fn json_strings_from_key(payload: &Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn json_strings_at_pointer(payload: &Value, pointer: &str) -> Vec<String> {
    payload
        .pointer(pointer)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn dedup_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for value in values {
        if seen.insert(value.clone()) {
            result.push(value);
        }
    }
    result
}

fn push_action_recommendation(
    items: &mut Vec<ActionRecommendationDto>,
    recommendation: ActionRecommendationDto,
) {
    if items
        .iter()
        .any(|item| item.recommendation_key == recommendation.recommendation_key)
    {
        return;
    }
    items.push(recommendation);
}

#[cfg(test)]
mod tests {
    use ecoach_identity::CreateAccountInput;
    use ecoach_substrate::{AccountType, EntitlementTier};

    use crate::{identity_commands, state::AppState};

    use super::*;

    #[test]
    fn semantics_registry_exposes_readiness_and_confidence_bands() {
        let registry = build_semantics_registry();
        assert!(!registry.readiness_bands.is_empty());
        assert!(
            registry
                .confidence_bands
                .iter()
                .any(|band| band.registry_key == "high")
        );
        assert_eq!(registry.thresholds.mastery_exam_ready, 9_000);
    }

    #[test]
    fn capability_registry_maps_confusion_into_product_tools() {
        let registry = build_capability_registry();
        assert!(registry.iter().any(|item| {
            item.signal_key == "conceptual_confusion" && item.tool_key == "contrast_mode"
        }));
        assert!(
            registry
                .iter()
                .any(|item| item.signal_key == "content_governance")
        );
    }

    #[test]
    fn student_product_surface_translates_core_backend_state() {
        let state = AppState::in_memory().expect("in-memory state should build");
        let account = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Esi".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("student should create");

        let surface = get_student_product_surface(&state, account.id)
            .expect("student surface should compose");

        assert_eq!(surface.student_id, account.id);
        assert!(!surface.directives.is_empty());
        assert!(
            surface
                .audience_explanations
                .iter()
                .any(|item| item.audience == "parent")
        );
    }

    #[test]
    fn areal_surface_and_control_tower_are_available_through_product_layer() {
        let state = AppState::in_memory().expect("in-memory state should build");
        let student = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Yaw".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("student should create");
        let admin = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Admin,
                display_name: "Ops".to_string(),
                pin: "2468".to_string(),
                entitlement_tier: EntitlementTier::Elite,
            },
        )
        .expect("admin should create");

        let saved_profile = save_areal_profile(
            &state,
            ARealProfileInputDto {
                student_id: student.id,
                tone_style: Some("calm".to_string()),
                motivation_style: Some("mentor".to_string()),
                urgency_style: Some("adaptive".to_string()),
                explanation_style: Some("exam_focused".to_string()),
                narrative_enabled: Some(true),
            },
        )
        .expect("profile should save");
        let areal = get_areal_surface(&state, student.id).expect("areal surface should load");
        let tower =
            get_super_admin_control_tower(&state, admin.id, Some(8)).expect("tower should load");

        assert_eq!(saved_profile.student_id, student.id);
        assert_eq!(areal.student_id, student.id);
        assert_eq!(areal.tone_style, "calm");
        assert_eq!(tower.admin_id, admin.id);
        assert!(tower.entitlement_manager.accounts.len() >= 2);
    }
}
