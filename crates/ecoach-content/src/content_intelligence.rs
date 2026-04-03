use std::collections::BTreeSet;

use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSourcePolicyInput {
    pub id: Option<i64>,
    pub policy_name: String,
    pub scope_type: String,
    pub scope_ref: Option<String>,
    pub source_kind: Option<String>,
    pub domain_pattern: Option<String>,
    pub access_mode: String,
    pub trust_tier: String,
    pub freshness_window_days: i64,
    pub allow_crawl: bool,
    pub allow_publish: bool,
    pub notes: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSourcePolicy {
    pub id: i64,
    pub policy_name: String,
    pub scope_type: String,
    pub scope_ref: Option<String>,
    pub source_kind: Option<String>,
    pub domain_pattern: Option<String>,
    pub access_mode: String,
    pub trust_tier: String,
    pub freshness_window_days: i64,
    pub allow_crawl: bool,
    pub allow_publish: bool,
    pub notes: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSourceProfileInput {
    pub canonical_uri: Option<String>,
    pub publisher: Option<String>,
    pub author: Option<String>,
    pub publication_date: Option<String>,
    pub license_type: Option<String>,
    pub crawl_permission: Option<String>,
    pub source_tier: Option<String>,
    pub trust_score_bp: Option<BasisPoints>,
    pub freshness_score_bp: Option<BasisPoints>,
    pub parse_status_detail: Option<String>,
    pub allowlisted_domain: Option<bool>,
    pub last_verified_at: Option<String>,
    pub review_due_at: Option<String>,
    pub stale_flag: Option<bool>,
    pub metadata_patch: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSourceRegistryEntry {
    pub id: i64,
    pub uploader_account_id: i64,
    pub source_kind: String,
    pub title: String,
    pub source_path: Option<String>,
    pub country_code: Option<String>,
    pub exam_board: Option<String>,
    pub education_level: Option<String>,
    pub subject_code: Option<String>,
    pub academic_year: Option<String>,
    pub language_code: String,
    pub version_label: Option<String>,
    pub source_status: String,
    pub confidence_score: BasisPoints,
    pub canonical_uri: Option<String>,
    pub publisher: Option<String>,
    pub author: Option<String>,
    pub publication_date: Option<String>,
    pub license_type: Option<String>,
    pub crawl_permission: String,
    pub source_tier: String,
    pub trust_score_bp: BasisPoints,
    pub freshness_score_bp: BasisPoints,
    pub parse_status_detail: String,
    pub allowlisted_domain: bool,
    pub last_verified_at: Option<String>,
    pub review_due_at: Option<String>,
    pub stale_flag: bool,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSourceSegmentInput {
    pub topic_id: Option<i64>,
    pub concept_id: Option<i64>,
    pub section_title: Option<String>,
    pub raw_text: String,
    pub normalized_text: Option<String>,
    pub markdown_text: Option<String>,
    pub image_refs: Value,
    pub equation_refs: Value,
    pub page_range: Option<String>,
    pub checksum: Option<String>,
    pub semantic_hash: Option<String>,
    pub extraction_confidence_bp: Option<BasisPoints>,
    pub relevance_score_bp: Option<BasisPoints>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSourceSegment {
    pub id: i64,
    pub source_upload_id: i64,
    pub topic_id: Option<i64>,
    pub concept_id: Option<i64>,
    pub section_title: Option<String>,
    pub raw_text: String,
    pub normalized_text: Option<String>,
    pub markdown_text: Option<String>,
    pub image_refs: Value,
    pub equation_refs: Value,
    pub page_range: Option<String>,
    pub checksum: Option<String>,
    pub semantic_hash: Option<String>,
    pub extraction_confidence_bp: BasisPoints,
    pub relevance_score_bp: BasisPoints,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResearchMissionInput {
    pub source_upload_id: Option<i64>,
    pub gap_ticket_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub mission_type: String,
    pub mission_brief: String,
    pub allowed_source_classes: Vec<String>,
    pub requested_asset_types: Vec<String>,
    pub coverage_snapshot: Value,
    pub priority_bp: Option<BasisPoints>,
    pub created_by_account_id: Option<i64>,
    pub planner_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResearchMission {
    pub id: i64,
    pub acquisition_job_id: i64,
    pub gap_ticket_id: Option<i64>,
    pub source_upload_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub mission_type: String,
    pub mission_brief: String,
    pub allowed_source_classes: Vec<String>,
    pub requested_asset_types: Vec<String>,
    pub coverage_snapshot: Value,
    pub priority_bp: BasisPoints,
    pub mission_stage: String,
    pub status: String,
    pub planner_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResearchCandidateInput {
    pub source_upload_id: Option<i64>,
    pub source_label: String,
    pub source_url: Option<String>,
    pub source_kind: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub source_tier: Option<String>,
    pub authority_score_bp: Option<BasisPoints>,
    pub relevance_score_bp: Option<BasisPoints>,
    pub freshness_score_bp: Option<BasisPoints>,
    pub license_type: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResearchCandidate {
    pub id: i64,
    pub mission_id: i64,
    pub acquisition_candidate_id: i64,
    pub source_upload_id: Option<i64>,
    pub source_label: String,
    pub source_url: Option<String>,
    pub source_kind: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub source_tier: String,
    pub authority_score_bp: BasisPoints,
    pub relevance_score_bp: BasisPoints,
    pub freshness_score_bp: BasisPoints,
    pub license_type: Option<String>,
    pub selected_for_verification: bool,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEvidenceRecordInput {
    pub mission_id: Option<i64>,
    pub research_candidate_id: Option<i64>,
    pub source_upload_id: Option<i64>,
    pub source_segment_id: Option<i64>,
    pub topic_id: i64,
    pub concept_id: Option<i64>,
    pub evidence_type: String,
    pub claim_text: String,
    pub supporting_text: Option<String>,
    pub extraction_confidence_bp: BasisPoints,
    pub corroboration_score_bp: BasisPoints,
    pub contradiction_score_bp: BasisPoints,
    pub pedagogy_score_bp: BasisPoints,
    pub freshness_score_bp: BasisPoints,
    pub provenance: Value,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEvidenceRecord {
    pub id: i64,
    pub source_type: String,
    pub source_id: Option<i64>,
    pub source_upload_id: Option<i64>,
    pub source_segment_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub concept_id: Option<i64>,
    pub evidence_type: String,
    pub claim_text: String,
    pub supporting_text: Option<String>,
    pub extraction_confidence_bp: BasisPoints,
    pub corroboration_score_bp: BasisPoints,
    pub contradiction_score_bp: BasisPoints,
    pub pedagogy_score_bp: BasisPoints,
    pub freshness_score_bp: BasisPoints,
    pub final_quality_bp: BasisPoints,
    pub status: String,
    pub verified_at: Option<String>,
    pub provenance: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPublishDecisionInput {
    pub publish_job_id: Option<i64>,
    pub source_upload_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub gate_name: String,
    pub decision_status: String,
    pub decision_reason: String,
    pub decision_score_bp: Option<BasisPoints>,
    pub decision_trace: Value,
    pub decided_by_account_id: Option<i64>,
    pub snapshot_audience: Option<String>,
    pub build_snapshot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPublishDecision {
    pub id: i64,
    pub publish_job_id: Option<i64>,
    pub source_upload_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub gate_name: String,
    pub decision_status: String,
    pub decision_reason: String,
    pub decision_score_bp: BasisPoints,
    pub snapshot_id: Option<i64>,
    pub decided_by_account_id: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSnapshotBuildInput {
    pub topic_id: i64,
    pub subject_id: Option<i64>,
    pub audience_type: String,
    pub snapshot_kind: Option<String>,
    pub label: Option<String>,
    pub status: Option<String>,
    pub source_publish_job_id: Option<i64>,
    pub source_upload_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSnapshot {
    pub id: i64,
    pub topic_id: i64,
    pub subject_id: Option<i64>,
    pub audience_type: String,
    pub snapshot_kind: String,
    pub label: Option<String>,
    pub status: String,
    pub source_publish_job_id: Option<i64>,
    pub source_upload_id: Option<i64>,
    pub manifest: Value,
    pub fingerprint: String,
    pub item_count: i64,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSnapshotItem {
    pub id: i64,
    pub snapshot_id: i64,
    pub source_type: String,
    pub source_ref: String,
    pub item_type: String,
    pub title: String,
    pub body_markdown: String,
    pub citation_ref: Option<String>,
    pub quality_score_bp: BasisPoints,
    pub freshness_score_bp: BasisPoints,
    pub metadata: Value,
    pub display_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRetrievalQueryInput {
    pub student_id: Option<i64>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub audience_type: Option<String>,
    pub query_text: String,
    pub content_types: Vec<String>,
    pub limit: Option<usize>,
    pub snapshot_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRetrievalHit {
    pub rank_index: i64,
    pub snapshot_item_id: Option<i64>,
    pub live_source_type: String,
    pub live_source_id: i64,
    pub item_type: String,
    pub title: String,
    pub excerpt: String,
    pub score_bp: BasisPoints,
    pub citation_ref: Option<String>,
    pub quality_score_bp: BasisPoints,
    pub freshness_score_bp: BasisPoints,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRetrievalResult {
    pub query_id: i64,
    pub used_snapshot_id: Option<i64>,
    pub hits: Vec<ContentRetrievalHit>,
    pub result_count: i64,
    pub citation_count: i64,
    pub citation_coverage_bp: BasisPoints,
    pub groundedness_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEvaluationRunInput {
    pub query_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub metric_family: String,
    pub groundedness_bp: BasisPoints,
    pub relevance_bp: BasisPoints,
    pub correctness_bp: BasisPoints,
    pub completeness_bp: BasisPoints,
    pub utilization_bp: BasisPoints,
    pub notes: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentEvaluationRun {
    pub id: i64,
    pub query_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub metric_family: String,
    pub groundedness_bp: BasisPoints,
    pub relevance_bp: BasisPoints,
    pub correctness_bp: BasisPoints,
    pub completeness_bp: BasisPoints,
    pub utilization_bp: BasisPoints,
    pub notes: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentIntelligenceOverview {
    pub topic_id: i64,
    pub topic_name: String,
    pub source_count: i64,
    pub segment_count: i64,
    pub evidence_count: i64,
    pub open_gap_ticket_count: i64,
    pub active_mission_count: i64,
    pub latest_snapshot_id: Option<i64>,
    pub latest_snapshot_status: Option<String>,
    pub published_asset_count: i64,
    pub retrieval_query_count: i64,
    pub evaluation_run_count: i64,
    pub stale_source_count: i64,
    pub average_quality_bp: BasisPoints,
}

#[derive(Debug, Clone)]
struct RetrievalCandidate {
    snapshot_item_id: Option<i64>,
    live_source_type: String,
    live_source_id: i64,
    item_type: String,
    title: String,
    body: String,
    citation_ref: Option<String>,
    quality_score_bp: BasisPoints,
    freshness_score_bp: BasisPoints,
    metadata: Value,
}

pub struct ContentIntelligenceService<'a> {
    conn: &'a Connection,
}

impl<'a> ContentIntelligenceService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn upsert_source_policy(
        &self,
        input: ContentSourcePolicyInput,
    ) -> EcoachResult<ContentSourcePolicy> {
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE content_source_policies
                     SET policy_name = ?2,
                         scope_type = ?3,
                         scope_ref = ?4,
                         source_kind = ?5,
                         domain_pattern = ?6,
                         access_mode = ?7,
                         trust_tier = ?8,
                         freshness_window_days = ?9,
                         allow_crawl = ?10,
                         allow_publish = ?11,
                         notes = ?12,
                         status = ?13,
                         updated_at = datetime('now')
                     WHERE id = ?1",
                    params![
                        id,
                        input.policy_name,
                        input.scope_type,
                        input.scope_ref,
                        input.source_kind,
                        input.domain_pattern,
                        input.access_mode,
                        input.trust_tier,
                        input.freshness_window_days,
                        if input.allow_crawl { 1 } else { 0 },
                        if input.allow_publish { 1 } else { 0 },
                        input.notes,
                        input.status,
                    ],
                )
                .map_err(storage_error)?;
            return self
                .get_source_policy(id)?
                .ok_or_else(|| EcoachError::NotFound(format!("source policy {} not found", id)));
        }

        self.conn
            .execute(
                "INSERT INTO content_source_policies (
                    policy_name, scope_type, scope_ref, source_kind, domain_pattern,
                    access_mode, trust_tier, freshness_window_days, allow_crawl,
                    allow_publish, notes, status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    input.policy_name,
                    input.scope_type,
                    input.scope_ref,
                    input.source_kind,
                    input.domain_pattern,
                    input.access_mode,
                    input.trust_tier,
                    input.freshness_window_days,
                    if input.allow_crawl { 1 } else { 0 },
                    if input.allow_publish { 1 } else { 0 },
                    input.notes,
                    input.status,
                ],
            )
            .map_err(storage_error)?;
        self.get_source_policy(self.conn.last_insert_rowid())?
            .ok_or_else(|| EcoachError::NotFound("inserted source policy missing".to_string()))
    }

    pub fn list_source_policies(
        &self,
        status: Option<&str>,
    ) -> EcoachResult<Vec<ContentSourcePolicy>> {
        let sql = if status.is_some() {
            "SELECT id, policy_name, scope_type, scope_ref, source_kind, domain_pattern,
                    access_mode, trust_tier, freshness_window_days, allow_crawl,
                    allow_publish, notes, status
             FROM content_source_policies
             WHERE status = ?1
             ORDER BY scope_type ASC, policy_name ASC"
        } else {
            "SELECT id, policy_name, scope_type, scope_ref, source_kind, domain_pattern,
                    access_mode, trust_tier, freshness_window_days, allow_crawl,
                    allow_publish, notes, status
             FROM content_source_policies
             ORDER BY scope_type ASC, policy_name ASC"
        };
        let mut stmt = self.conn.prepare(sql).map_err(storage_error)?;
        let rows = if let Some(status) = status {
            stmt.query_map([status], map_source_policy)
        } else {
            stmt.query_map([], map_source_policy)
        }
        .map_err(storage_error)?;
        collect_rows(rows)
    }

    pub fn save_source_profile(
        &self,
        source_upload_id: i64,
        input: ContentSourceProfileInput,
    ) -> EcoachResult<ContentSourceRegistryEntry> {
        let existing = self
            .get_source(source_upload_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("source upload {} not found", source_upload_id)))?;
        let merged_metadata = merge_json(existing.metadata, input.metadata_patch);
        self.conn
            .execute(
                "UPDATE curriculum_source_uploads
                 SET canonical_uri = ?2,
                     publisher = ?3,
                     author = ?4,
                     publication_date = ?5,
                     license_type = ?6,
                     crawl_permission = ?7,
                     source_tier = ?8,
                     trust_score_bp = ?9,
                     freshness_score_bp = ?10,
                     parse_status_detail = ?11,
                     allowlisted_domain = ?12,
                     last_verified_at = ?13,
                     review_due_at = ?14,
                     stale_flag = ?15,
                     metadata_json = ?16,
                     updated_at = datetime('now')
                 WHERE id = ?1",
                params![
                    source_upload_id,
                    input.canonical_uri.or(existing.canonical_uri),
                    input.publisher.or(existing.publisher),
                    input.author.or(existing.author),
                    input.publication_date.or(existing.publication_date),
                    input.license_type.or(existing.license_type),
                    input
                        .crawl_permission
                        .unwrap_or(existing.crawl_permission),
                    input.source_tier.unwrap_or(existing.source_tier),
                    input.trust_score_bp.unwrap_or(existing.trust_score_bp) as i64,
                    input
                        .freshness_score_bp
                        .unwrap_or(existing.freshness_score_bp) as i64,
                    input
                        .parse_status_detail
                        .unwrap_or(existing.parse_status_detail),
                    if input
                        .allowlisted_domain
                        .unwrap_or(existing.allowlisted_domain)
                    {
                        1
                    } else {
                        0
                    },
                    input.last_verified_at.or(existing.last_verified_at),
                    input.review_due_at.or(existing.review_due_at),
                    if input.stale_flag.unwrap_or(existing.stale_flag) {
                        1
                    } else {
                        0
                    },
                    to_json(&merged_metadata)?,
                ],
            )
            .map_err(storage_error)?;
        self.get_source(source_upload_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("source upload {} not found", source_upload_id)))
    }

    pub fn list_content_sources(
        &self,
        status: Option<&str>,
        source_kind: Option<&str>,
        limit: Option<usize>,
    ) -> EcoachResult<Vec<ContentSourceRegistryEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                        exam_board, education_level, subject_code, academic_year,
                        language_code, version_label, source_status, confidence_score,
                        canonical_uri, publisher, author, publication_date, license_type,
                        crawl_permission, source_tier, trust_score_bp, freshness_score_bp,
                        parse_status_detail, allowlisted_domain, last_verified_at,
                        review_due_at, stale_flag, metadata_json
                 FROM curriculum_source_uploads
                 ORDER BY updated_at DESC, id DESC",
            )
            .map_err(storage_error)?;
        let rows = stmt.query_map([], map_source_registry_entry).map_err(storage_error)?;
        let mut items = collect_rows(rows)?;
        if let Some(status) = status {
            items.retain(|item| item.source_status == status);
        }
        if let Some(source_kind) = source_kind {
            items.retain(|item| item.source_kind == source_kind);
        }
        if let Some(limit) = limit {
            items.truncate(limit);
        }
        Ok(items)
    }

    pub fn ingest_source_segments(
        &self,
        source_upload_id: i64,
        segments: Vec<ContentSourceSegmentInput>,
    ) -> EcoachResult<Vec<ContentSourceSegment>> {
        if self.get_source(source_upload_id)?.is_none() {
            return Err(EcoachError::NotFound(format!(
                "source upload {} not found",
                source_upload_id
            )));
        }
        let mut persisted = Vec::new();
        for segment in segments {
            let existing_id = if let Some(hash) = segment.semantic_hash.as_deref() {
                self.conn
                    .query_row(
                        "SELECT id
                         FROM content_source_segments
                         WHERE source_upload_id = ?1 AND semantic_hash = ?2
                         LIMIT 1",
                        params![source_upload_id, hash],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(storage_error)?
            } else {
                None
            };
            if let Some(existing_id) = existing_id {
                self.conn
                    .execute(
                        "UPDATE content_source_segments
                         SET topic_id = ?2,
                             concept_id = ?3,
                             section_title = ?4,
                             raw_text = ?5,
                             normalized_text = ?6,
                             markdown_text = ?7,
                             image_refs_json = ?8,
                             equation_refs_json = ?9,
                             page_range = ?10,
                             checksum = ?11,
                             semantic_hash = ?12,
                             extraction_confidence_bp = ?13,
                             relevance_score_bp = ?14,
                             metadata_json = ?15
                         WHERE id = ?1",
                        params![
                            existing_id,
                            segment.topic_id,
                            segment.concept_id,
                            segment.section_title,
                            segment.raw_text,
                            segment.normalized_text,
                            segment.markdown_text,
                            to_json(&segment.image_refs)?,
                            to_json(&segment.equation_refs)?,
                            segment.page_range,
                            segment.checksum,
                            segment.semantic_hash,
                            segment.extraction_confidence_bp.unwrap_or(7_000) as i64,
                            segment.relevance_score_bp.unwrap_or(6_500) as i64,
                            to_json(&segment.metadata)?,
                        ],
                    )
                    .map_err(storage_error)?;
                persisted.push(self.get_source_segment(existing_id)?.ok_or_else(|| {
                    EcoachError::NotFound(format!("segment {} not found", existing_id))
                })?);
                continue;
            }
            self.conn
                .execute(
                    "INSERT INTO content_source_segments (
                        source_upload_id, topic_id, concept_id, section_title, raw_text,
                        normalized_text, markdown_text, image_refs_json, equation_refs_json,
                        page_range, checksum, semantic_hash, extraction_confidence_bp,
                        relevance_score_bp, metadata_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                    params![
                        source_upload_id,
                        segment.topic_id,
                        segment.concept_id,
                        segment.section_title,
                        segment.raw_text,
                        segment.normalized_text,
                        segment.markdown_text,
                        to_json(&segment.image_refs)?,
                        to_json(&segment.equation_refs)?,
                        segment.page_range,
                        segment.checksum,
                        segment.semantic_hash,
                        segment.extraction_confidence_bp.unwrap_or(7_000) as i64,
                        segment.relevance_score_bp.unwrap_or(6_500) as i64,
                        to_json(&segment.metadata)?,
                    ],
                )
                .map_err(storage_error)?;
            persisted.push(self.get_source_segment(self.conn.last_insert_rowid())?.ok_or_else(
                || EcoachError::NotFound("inserted segment missing".to_string()),
            )?);
        }
        Ok(persisted)
    }

    pub fn plan_research_mission(
        &self,
        input: ContentResearchMissionInput,
    ) -> EcoachResult<ContentResearchMission> {
        let gap_ticket_id = match (input.gap_ticket_id, input.topic_id) {
            (Some(existing), _) => Some(existing),
            (None, Some(topic_id)) => {
                self.conn
                    .execute(
                        "INSERT INTO content_gap_tickets (
                            topic_id, trigger_type, trigger_context_json, severity,
                            required_asset_types_json, status, created_at
                         ) VALUES (?1, 'coverage_scan', ?2, 'medium', ?3, 'in_progress', datetime('now'))",
                        params![
                            topic_id,
                            to_json(&json!({
                                "mission_type": input.mission_type,
                                "mission_brief": input.mission_brief,
                            }))?,
                            to_json(&input.requested_asset_types)?,
                        ],
                    )
                    .map_err(storage_error)?;
                Some(self.conn.last_insert_rowid())
            }
            _ => None,
        };
        let intent_type = mission_type_to_intent(&input.mission_type);
        let source_scope = infer_source_scope(&input.allowed_source_classes);
        self.conn
            .execute(
                "INSERT INTO content_acquisition_jobs (
                    subject_id, topic_id, intent_type, query_text, source_scope,
                    status, result_summary_json, allowed_source_classes_json,
                    requested_asset_types_json, coverage_snapshot_json,
                    mission_stage, planner_notes
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 'queued', '{}', ?6, ?7, ?8, 'planned', ?9)",
                params![
                    input.subject_id,
                    input.topic_id,
                    intent_type,
                    input.mission_brief,
                    source_scope,
                    to_json(&input.allowed_source_classes)?,
                    to_json(&input.requested_asset_types)?,
                    to_json(&input.coverage_snapshot)?,
                    input.planner_notes,
                ],
            )
            .map_err(storage_error)?;
        let acquisition_job_id = self.conn.last_insert_rowid();
        self.conn
            .execute(
                "INSERT INTO content_research_missions (
                    acquisition_job_id, gap_ticket_id, source_upload_id, subject_id,
                    topic_id, mission_type, mission_brief, allowed_source_classes_json,
                    requested_asset_types_json, coverage_snapshot_json, priority_bp,
                    mission_stage, status, created_by_account_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'planned', 'queued', ?12)",
                params![
                    acquisition_job_id,
                    gap_ticket_id,
                    input.source_upload_id,
                    input.subject_id,
                    input.topic_id,
                    input.mission_type,
                    input.mission_brief,
                    to_json(&input.allowed_source_classes)?,
                    to_json(&input.requested_asset_types)?,
                    to_json(&input.coverage_snapshot)?,
                    input.priority_bp.unwrap_or(6_000) as i64,
                    input.created_by_account_id,
                ],
            )
            .map_err(storage_error)?;
        let mission_id = self.conn.last_insert_rowid();
        self.log_research_event(
            mission_id,
            "planned",
            "Research mission planned",
            &json!({
                "mission_type": input.mission_type,
                "allowed_source_classes": input.allowed_source_classes,
                "requested_asset_types": input.requested_asset_types,
            }),
        )?;
        self.get_research_mission(mission_id)?
            .ok_or_else(|| EcoachError::NotFound("inserted mission missing".to_string()))
    }

    pub fn add_research_candidate(
        &self,
        mission_id: i64,
        input: ContentResearchCandidateInput,
    ) -> EcoachResult<ContentResearchCandidate> {
        let mission = self
            .get_research_mission(mission_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("research mission {} not found", mission_id)))?;
        self.conn
            .execute(
                "INSERT INTO acquisition_evidence_candidates (
                    job_id, source_label, source_url, source_kind, title, snippet,
                    extracted_payload_json, quality_score, freshness_score, review_status,
                    authority_score_bp, relevance_score_bp, source_tier, license_type
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'staged', ?10, ?11, ?12, ?13)",
                params![
                    mission.acquisition_job_id,
                    input.source_label,
                    input.source_url,
                    input.source_kind,
                    input.title,
                    input.snippet,
                    to_json(&input.payload)?,
                    clamp_bp(
                        (input.authority_score_bp.unwrap_or(6_500) as i64
                            + input.relevance_score_bp.unwrap_or(6_500) as i64)
                            / 2
                    ) as i64,
                    input.freshness_score_bp.unwrap_or(6_000) as i64,
                    input.authority_score_bp.unwrap_or(6_500) as i64,
                    input.relevance_score_bp.unwrap_or(6_500) as i64,
                    input.source_tier.as_deref().unwrap_or("staged"),
                    input.license_type,
                ],
            )
            .map_err(storage_error)?;
        let acquisition_candidate_id = self.conn.last_insert_rowid();
        self.conn
            .execute(
                "INSERT INTO content_research_candidates (
                    mission_id, acquisition_candidate_id, source_upload_id, source_label,
                    source_url, source_kind, title, snippet, source_tier,
                    authority_score_bp, relevance_score_bp, freshness_score_bp,
                    license_type, payload_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    mission_id,
                    acquisition_candidate_id,
                    input.source_upload_id,
                    input.source_label,
                    input.source_url,
                    input.source_kind,
                    input.title,
                    input.snippet,
                    input.source_tier.as_deref().unwrap_or("staged"),
                    input.authority_score_bp.unwrap_or(6_500) as i64,
                    input.relevance_score_bp.unwrap_or(6_500) as i64,
                    input.freshness_score_bp.unwrap_or(6_000) as i64,
                    input.license_type,
                    to_json(&input.payload)?,
                ],
            )
            .map_err(storage_error)?;
        let candidate_id = self.conn.last_insert_rowid();
        self.conn
            .execute(
                "UPDATE content_research_missions
                 SET mission_stage = 'scouting',
                     status = 'running',
                     updated_at = datetime('now')
                 WHERE id = ?1",
                [mission_id],
            )
            .map_err(storage_error)?;
        self.conn
            .execute(
                "UPDATE content_acquisition_jobs
                 SET mission_stage = 'scouting',
                     status = 'running',
                     updated_at = datetime('now')
                 WHERE id = ?1",
                [mission.acquisition_job_id],
            )
            .map_err(storage_error)?;
        self.log_research_event(
            mission_id,
            "scouting",
            "Research candidate added",
            &json!({
                "candidate_id": candidate_id,
                "source_label": input.source_label,
            }),
        )?;
        self.get_research_candidate(candidate_id)?
            .ok_or_else(|| EcoachError::NotFound("inserted candidate missing".to_string()))
    }

    pub fn record_evidence(
        &self,
        input: ContentEvidenceRecordInput,
    ) -> EcoachResult<ContentEvidenceRecord> {
        let final_quality = clamp_bp(
            (input.extraction_confidence_bp as i64
                + input.corroboration_score_bp as i64
                + input.pedagogy_score_bp as i64
                + input.freshness_score_bp as i64
                + (10_000 - input.contradiction_score_bp as i64))
                / 5,
        );
        let source_type = if input.source_segment_id.is_some() {
            "source_segment"
        } else if input.source_upload_id.is_some() {
            "source_upload"
        } else if input.research_candidate_id.is_some() {
            "research_candidate"
        } else {
            "manual"
        };
        let source_id = input
            .source_upload_id
            .or(input.research_candidate_id)
            .or(input.source_segment_id);
        self.conn
            .execute(
                "INSERT INTO evidence_blocks (
                    source_type, source_id, source_upload_id, source_segment_id,
                    topic_id, concept_id, evidence_type, claim_text, supporting_text,
                    extraction_confidence_bp, corroboration_score_bp,
                    contradiction_score_bp, pedagogy_score_bp, freshness_score_bp,
                    final_quality_bp, status, provenance_json, verified_at, created_at
                 ) VALUES (
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, ?7, ?8, ?9,
                    ?10, ?11, ?12, ?13, ?14,
                    ?15, ?16, ?17,
                    CASE WHEN ?18 = 1 THEN datetime('now') ELSE NULL END,
                    datetime('now')
                 )",
                params![
                    source_type,
                    source_id,
                    input.source_upload_id,
                    input.source_segment_id,
                    input.topic_id,
                    input.concept_id,
                    input.evidence_type,
                    input.claim_text,
                    input.supporting_text,
                    input.extraction_confidence_bp as i64,
                    input.corroboration_score_bp as i64,
                    input.contradiction_score_bp as i64,
                    input.pedagogy_score_bp as i64,
                    input.freshness_score_bp as i64,
                    final_quality as i64,
                    if input.verified { "verified" } else { "pending" },
                    to_json(&input.provenance)?,
                    if input.verified { 1 } else { 0 },
                ],
            )
            .map_err(storage_error)?;
        let evidence_id = self.conn.last_insert_rowid();
        if let Some(candidate_id) = input.research_candidate_id {
            self.conn
                .execute(
                    "UPDATE content_research_candidates
                     SET selected_for_verification = 1
                     WHERE id = ?1",
                    [candidate_id],
                )
                .map_err(storage_error)?;
        }
        if let Some(mission_id) = input.mission_id {
            self.conn
                .execute(
                    "UPDATE content_research_missions
                     SET mission_stage = 'verifying',
                         status = 'running',
                         updated_at = datetime('now')
                     WHERE id = ?1",
                    [mission_id],
                )
                .map_err(storage_error)?;
            self.log_research_event(
                mission_id,
                "verifying",
                "Evidence recorded",
                &json!({ "evidence_id": evidence_id, "final_quality_bp": final_quality }),
            )?;
        }
        self.get_evidence_record(evidence_id)?
            .ok_or_else(|| EcoachError::NotFound("inserted evidence missing".to_string()))
    }

    pub fn record_publish_decision(
        &self,
        input: ContentPublishDecisionInput,
    ) -> EcoachResult<ContentPublishDecision> {
        let (topic_id, subject_id) =
            self.resolve_decision_scope(input.publish_job_id, input.topic_id, input.subject_id)?;
        self.conn
            .execute(
                "INSERT INTO content_publish_decisions (
                    publish_job_id, source_upload_id, subject_id, topic_id, gate_name,
                    decision_status, decision_reason, decision_score_bp, decision_trace_json,
                    decided_by_account_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    input.publish_job_id,
                    input.source_upload_id,
                    subject_id,
                    topic_id,
                    input.gate_name,
                    input.decision_status,
                    input.decision_reason,
                    input.decision_score_bp.unwrap_or(7_000) as i64,
                    to_json(&input.decision_trace)?,
                    input.decided_by_account_id,
                ],
            )
            .map_err(storage_error)?;
        let decision_id = self.conn.last_insert_rowid();
        let snapshot_id = if input.build_snapshot {
            let snapshot = self.build_topic_snapshot(ContentSnapshotBuildInput {
                topic_id: topic_id.ok_or_else(|| {
                    EcoachError::Validation("topic_id is required for snapshot publishing".to_string())
                })?,
                subject_id,
                audience_type: input
                    .snapshot_audience
                    .unwrap_or_else(|| "student".to_string()),
                snapshot_kind: Some(match input.decision_status.as_str() {
                    "preview" => "preview_topic_bundle".to_string(),
                    "rollback" => "rollback_bundle".to_string(),
                    _ => "verified_topic_bundle".to_string(),
                }),
                label: Some(format!("{} snapshot", input.gate_name)),
                status: Some(match input.decision_status.as_str() {
                    "preview" => "preview".to_string(),
                    "rollback" => "rolled_back".to_string(),
                    _ => "published".to_string(),
                }),
                source_publish_job_id: input.publish_job_id,
                source_upload_id: input.source_upload_id,
            })?;
            Some(snapshot.id)
        } else {
            None
        };
        if let Some(snapshot_id) = snapshot_id {
            self.conn
                .execute(
                    "UPDATE content_publish_decisions SET snapshot_id = ?2 WHERE id = ?1",
                    params![decision_id, snapshot_id],
                )
                .map_err(storage_error)?;
        }
        self.get_publish_decision(decision_id)?
            .ok_or_else(|| EcoachError::NotFound("inserted publish decision missing".to_string()))
    }

    pub fn build_topic_snapshot(
        &self,
        input: ContentSnapshotBuildInput,
    ) -> EcoachResult<ContentSnapshot> {
        let topic_name = self.topic_name(input.topic_id)?;
        let subject_id = if let Some(subject_id) = input.subject_id {
            Some(subject_id)
        } else {
            self.conn
                .query_row(
                    "SELECT subject_id FROM topics WHERE id = ?1",
                    [input.topic_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(storage_error)?
        };
        let mut items = self.load_snapshot_artifact_items(input.topic_id)?;
        items.extend(self.load_snapshot_evidence_items(input.topic_id)?);
        items.extend(self.load_snapshot_segment_items(input.topic_id, input.source_upload_id)?);
        if items.is_empty() {
            return Err(EcoachError::Validation(
                "cannot build a snapshot without artifacts, evidence, or source segments"
                    .to_string(),
            ));
        }

        self.conn
            .execute(
                "UPDATE content_snapshots
                 SET is_active = 0
                 WHERE topic_id = ?1 AND audience_type = ?2",
                params![input.topic_id, input.audience_type],
            )
            .map_err(storage_error)?;

        let manifest = json!({
            "topic_name": topic_name,
            "item_count": items.len(),
            "artifact_count": items.iter().filter(|item| item.live_source_type == "artifact").count(),
            "evidence_count": items.iter().filter(|item| item.live_source_type == "evidence").count(),
            "segment_count": items.iter().filter(|item| item.live_source_type == "segment").count(),
        });
        let fingerprint = format!(
            "topic:{}:{}:{}",
            input.topic_id,
            input.audience_type,
            items.len()
        );
        self.conn
            .execute(
                "INSERT INTO content_snapshots (
                    topic_id, subject_id, audience_type, snapshot_kind, label, status,
                    source_publish_job_id, source_upload_id, manifest_json, fingerprint,
                    item_count, is_active
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 1)",
                params![
                    input.topic_id,
                    subject_id,
                    input.audience_type,
                    input
                        .snapshot_kind
                        .unwrap_or_else(|| "verified_topic_bundle".to_string()),
                    input.label,
                    input.status.unwrap_or_else(|| "published".to_string()),
                    input.source_publish_job_id,
                    input.source_upload_id,
                    to_json(&manifest)?,
                    fingerprint,
                    items.len() as i64,
                ],
            )
            .map_err(storage_error)?;
        let snapshot_id = self.conn.last_insert_rowid();
        for (index, item) in items.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO content_snapshot_items (
                        snapshot_id, source_type, source_ref, item_type, title, body_markdown,
                        citation_ref, quality_score_bp, freshness_score_bp, metadata_json,
                        display_order
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        snapshot_id,
                        item.live_source_type,
                        item.live_source_id.to_string(),
                        item.item_type,
                        item.title,
                        item.body,
                        item.citation_ref,
                        item.quality_score_bp as i64,
                        item.freshness_score_bp as i64,
                        to_json(&item.metadata)?,
                        index as i64,
                    ],
                )
                .map_err(storage_error)?;
        }
        self.get_snapshot(snapshot_id)?
            .ok_or_else(|| EcoachError::NotFound("inserted snapshot missing".to_string()))
    }

    pub fn retrieve_content(
        &self,
        input: ContentRetrievalQueryInput,
    ) -> EcoachResult<ContentRetrievalResult> {
        let snapshot = self.resolve_active_snapshot(input.topic_id, input.audience_type.as_deref())?;
        self.conn
            .execute(
                "INSERT INTO content_retrieval_queries (
                    student_id, subject_id, topic_id, audience_type, query_text,
                    filters_json, snapshot_id, status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending')",
                params![
                    input.student_id,
                    input.subject_id,
                    input.topic_id,
                    input.audience_type,
                    input.query_text,
                    to_json(&json!({
                        "content_types": input.content_types,
                        "snapshot_only": input.snapshot_only,
                    }))?,
                    snapshot.as_ref().map(|item| item.id),
                ],
            )
            .map_err(storage_error)?;
        let query_id = self.conn.last_insert_rowid();

        let mut candidates = if let Some(snapshot) = snapshot.as_ref() {
            self.load_snapshot_retrieval_candidates(snapshot.id)?
        } else {
            Vec::new()
        };
        if candidates.is_empty() && !input.snapshot_only {
            candidates = self.load_live_retrieval_candidates(input.topic_id, input.subject_id)?;
        }

        let normalized_query = normalize_text(&input.query_text);
        let requested_types: BTreeSet<String> = input
            .content_types
            .iter()
            .map(|value| normalize_text(value))
            .collect();
        let limit = input.limit.unwrap_or(8);
        let mut hits = Vec::new();
        for candidate in candidates {
            if !requested_types.is_empty()
                && !requested_types.contains(&normalize_text(&candidate.item_type))
                && !candidate_matches_requested_types(&candidate.metadata, &requested_types)
            {
                continue;
            }
            let score = score_candidate(&normalized_query, &candidate, input.topic_id);
            if score == 0 {
                continue;
            }
            hits.push(ContentRetrievalHit {
                rank_index: 0,
                snapshot_item_id: candidate.snapshot_item_id,
                live_source_type: candidate.live_source_type,
                live_source_id: candidate.live_source_id,
                item_type: candidate.item_type,
                title: candidate.title,
                excerpt: excerpt(&candidate.body, 220),
                score_bp: score,
                citation_ref: candidate.citation_ref,
                quality_score_bp: candidate.quality_score_bp,
                freshness_score_bp: candidate.freshness_score_bp,
                metadata: candidate.metadata,
            });
        }
        hits.sort_by(|left, right| {
            right
                .score_bp
                .cmp(&left.score_bp)
                .then_with(|| right.quality_score_bp.cmp(&left.quality_score_bp))
                .then_with(|| left.title.cmp(&right.title))
        });
        hits.truncate(limit);

        for (index, hit) in hits.iter_mut().enumerate() {
            hit.rank_index = index as i64;
            self.conn
                .execute(
                    "INSERT INTO content_retrieval_hits (
                        query_id, snapshot_item_id, live_source_type, live_source_id,
                        rank_index, score_bp, citation_ref, metadata_json
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        query_id,
                        hit.snapshot_item_id,
                        hit.live_source_type,
                        hit.live_source_id,
                        hit.rank_index,
                        hit.score_bp as i64,
                        hit.citation_ref,
                        to_json(&hit.metadata)?,
                    ],
                )
                .map_err(storage_error)?;
        }

        let citation_count = hits.iter().filter(|hit| hit.citation_ref.is_some()).count() as i64;
        let citation_coverage_bp = if hits.is_empty() {
            0
        } else {
            clamp_bp((citation_count * 10_000) / hits.len() as i64)
        };
        let groundedness_bp = if hits.is_empty() {
            0
        } else {
            clamp_bp(
                hits.iter()
                    .map(|hit| hit.quality_score_bp as i64)
                    .sum::<i64>()
                    / hits.len() as i64,
            )
        };
        self.conn
            .execute(
                "UPDATE content_retrieval_queries
                 SET result_count = ?2,
                     citation_count = ?3,
                     status = ?4
                 WHERE id = ?1",
                params![
                    query_id,
                    hits.len() as i64,
                    citation_count,
                    if hits.is_empty() { "empty" } else { "served" },
                ],
            )
            .map_err(storage_error)?;

        Ok(ContentRetrievalResult {
            query_id,
            used_snapshot_id: snapshot.map(|item| item.id),
            result_count: hits.len() as i64,
            citation_count,
            citation_coverage_bp,
            groundedness_bp,
            hits,
        })
    }

    pub fn record_evaluation_run(
        &self,
        input: ContentEvaluationRunInput,
    ) -> EcoachResult<ContentEvaluationRun> {
        self.conn
            .execute(
                "INSERT INTO content_evaluation_runs (
                    query_id, topic_id, metric_family, groundedness_bp, relevance_bp,
                    correctness_bp, completeness_bp, utilization_bp, notes_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    input.query_id,
                    input.topic_id,
                    input.metric_family,
                    input.groundedness_bp as i64,
                    input.relevance_bp as i64,
                    input.correctness_bp as i64,
                    input.completeness_bp as i64,
                    input.utilization_bp as i64,
                    to_json(&input.notes)?,
                ],
            )
            .map_err(storage_error)?;
        self.get_evaluation_run(self.conn.last_insert_rowid())?
            .ok_or_else(|| EcoachError::NotFound("inserted evaluation missing".to_string()))
    }

    pub fn get_content_intelligence_overview(
        &self,
        topic_id: i64,
    ) -> EcoachResult<ContentIntelligenceOverview> {
        let topic_name = self.topic_name(topic_id)?;
        let source_count = self.count_query(
            "SELECT COUNT(DISTINCT source_upload_id)
             FROM content_source_segments
             WHERE topic_id = ?1",
            topic_id,
        )?;
        let segment_count = self.count_query(
            "SELECT COUNT(*) FROM content_source_segments WHERE topic_id = ?1",
            topic_id,
        )?;
        let evidence_count =
            self.count_query("SELECT COUNT(*) FROM evidence_blocks WHERE topic_id = ?1", topic_id)?;
        let open_gap_ticket_count = self.count_query(
            "SELECT COUNT(*)
             FROM content_gap_tickets
             WHERE topic_id = ?1
               AND status IN ('open', 'in_progress')",
            topic_id,
        )?;
        let active_mission_count = self.count_query(
            "SELECT COUNT(*)
             FROM content_research_missions
             WHERE topic_id = ?1
               AND status IN ('queued', 'running', 'review_required')",
            topic_id,
        )?;
        let published_asset_count = self.count_query(
            "SELECT COUNT(*)
             FROM artifacts
             WHERE topic_id = ?1
               AND lifecycle_state IN ('approved', 'live')",
            topic_id,
        )?;
        let retrieval_query_count = self.count_query(
            "SELECT COUNT(*) FROM content_retrieval_queries WHERE topic_id = ?1",
            topic_id,
        )?;
        let evaluation_run_count = self.count_query(
            "SELECT COUNT(*) FROM content_evaluation_runs WHERE topic_id = ?1",
            topic_id,
        )?;
        let stale_source_count = self.count_query(
            "SELECT COUNT(DISTINCT segments.source_upload_id)
             FROM content_source_segments segments
             INNER JOIN curriculum_source_uploads uploads
                ON uploads.id = segments.source_upload_id
             WHERE segments.topic_id = ?1
               AND uploads.stale_flag = 1",
            topic_id,
        )?;
        let average_quality_bp = self
            .conn
            .query_row(
                "SELECT AVG(final_quality_bp) FROM evidence_blocks WHERE topic_id = ?1",
                [topic_id],
                |row| row.get::<_, Option<f64>>(0),
            )
            .map_err(storage_error)?
            .map(|value| clamp_bp(value.round() as i64))
            .unwrap_or(0);
        let latest_snapshot = self
            .conn
            .query_row(
                "SELECT id, status
                 FROM content_snapshots
                 WHERE topic_id = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                [topic_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(ContentIntelligenceOverview {
            topic_id,
            topic_name,
            source_count,
            segment_count,
            evidence_count,
            open_gap_ticket_count,
            active_mission_count,
            latest_snapshot_id: latest_snapshot.as_ref().map(|(id, _)| *id),
            latest_snapshot_status: latest_snapshot.map(|(_, status)| status),
            published_asset_count,
            retrieval_query_count,
            evaluation_run_count,
            stale_source_count,
            average_quality_bp,
        })
    }

    fn get_source_policy(&self, policy_id: i64) -> EcoachResult<Option<ContentSourcePolicy>> {
        self.conn
            .query_row(
                "SELECT id, policy_name, scope_type, scope_ref, source_kind, domain_pattern,
                        access_mode, trust_tier, freshness_window_days, allow_crawl,
                        allow_publish, notes, status
                 FROM content_source_policies
                 WHERE id = ?1",
                [policy_id],
                map_source_policy,
            )
            .optional()
            .map_err(storage_error)
    }

    fn get_source(&self, source_upload_id: i64) -> EcoachResult<Option<ContentSourceRegistryEntry>> {
        self.conn
            .query_row(
                "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                        exam_board, education_level, subject_code, academic_year,
                        language_code, version_label, source_status, confidence_score,
                        canonical_uri, publisher, author, publication_date, license_type,
                        crawl_permission, source_tier, trust_score_bp, freshness_score_bp,
                        parse_status_detail, allowlisted_domain, last_verified_at,
                        review_due_at, stale_flag, metadata_json
                 FROM curriculum_source_uploads
                 WHERE id = ?1",
                [source_upload_id],
                map_source_registry_entry,
            )
            .optional()
            .map_err(storage_error)
    }

    fn get_source_segment(&self, segment_id: i64) -> EcoachResult<Option<ContentSourceSegment>> {
        self.conn
            .query_row(
                "SELECT id, source_upload_id, topic_id, concept_id, section_title, raw_text,
                        normalized_text, markdown_text, image_refs_json, equation_refs_json,
                        page_range, checksum, semantic_hash, extraction_confidence_bp,
                        relevance_score_bp, metadata_json, created_at
                 FROM content_source_segments
                 WHERE id = ?1",
                [segment_id],
                map_source_segment,
            )
            .optional()
            .map_err(storage_error)
    }

    fn get_research_mission(
        &self,
        mission_id: i64,
    ) -> EcoachResult<Option<ContentResearchMission>> {
        self.conn
            .query_row(
                "SELECT id, acquisition_job_id, gap_ticket_id, source_upload_id, subject_id,
                        topic_id, mission_type, mission_brief, allowed_source_classes_json,
                        requested_asset_types_json, coverage_snapshot_json, priority_bp,
                        mission_stage, status,
                        (SELECT planner_notes FROM content_acquisition_jobs jobs
                         WHERE jobs.id = content_research_missions.acquisition_job_id)
                 FROM content_research_missions
                 WHERE id = ?1",
                [mission_id],
                map_research_mission,
            )
            .optional()
            .map_err(storage_error)
    }

    fn get_research_candidate(
        &self,
        candidate_id: i64,
    ) -> EcoachResult<Option<ContentResearchCandidate>> {
        self.conn
            .query_row(
                "SELECT id, mission_id, acquisition_candidate_id, source_upload_id, source_label,
                        source_url, source_kind, title, snippet, source_tier,
                        authority_score_bp, relevance_score_bp, freshness_score_bp,
                        license_type, selected_for_verification, payload_json
                 FROM content_research_candidates
                 WHERE id = ?1",
                [candidate_id],
                map_research_candidate,
            )
            .optional()
            .map_err(storage_error)
    }

    fn get_evidence_record(
        &self,
        evidence_id: i64,
    ) -> EcoachResult<Option<ContentEvidenceRecord>> {
        self.conn
            .query_row(
                "SELECT id, source_type, source_id, source_upload_id, source_segment_id,
                        topic_id, concept_id, evidence_type, claim_text, supporting_text,
                        extraction_confidence_bp, corroboration_score_bp,
                        contradiction_score_bp, pedagogy_score_bp, freshness_score_bp,
                        final_quality_bp, status, verified_at, provenance_json
                 FROM evidence_blocks
                 WHERE id = ?1",
                [evidence_id],
                map_evidence_record,
            )
            .optional()
            .map_err(storage_error)
    }

    fn get_publish_decision(
        &self,
        decision_id: i64,
    ) -> EcoachResult<Option<ContentPublishDecision>> {
        self.conn
            .query_row(
                "SELECT id, publish_job_id, source_upload_id, subject_id, topic_id, gate_name,
                        decision_status, decision_reason, decision_score_bp, snapshot_id,
                        decided_by_account_id, created_at
                 FROM content_publish_decisions
                 WHERE id = ?1",
                [decision_id],
                map_publish_decision,
            )
            .optional()
            .map_err(storage_error)
    }

    fn get_snapshot(&self, snapshot_id: i64) -> EcoachResult<Option<ContentSnapshot>> {
        self.conn
            .query_row(
                "SELECT id, topic_id, subject_id, audience_type, snapshot_kind, label,
                        status, source_publish_job_id, source_upload_id, manifest_json,
                        fingerprint, item_count, is_active, created_at
                 FROM content_snapshots
                 WHERE id = ?1",
                [snapshot_id],
                map_snapshot,
            )
            .optional()
            .map_err(storage_error)
    }

    fn resolve_active_snapshot(
        &self,
        topic_id: Option<i64>,
        audience_type: Option<&str>,
    ) -> EcoachResult<Option<ContentSnapshot>> {
        let Some(topic_id) = topic_id else {
            return Ok(None);
        };
        let audience = audience_type.unwrap_or("student");
        self.conn
            .query_row(
                "SELECT id, topic_id, subject_id, audience_type, snapshot_kind, label,
                        status, source_publish_job_id, source_upload_id, manifest_json,
                        fingerprint, item_count, is_active, created_at
                 FROM content_snapshots
                 WHERE topic_id = ?1
                   AND audience_type = ?2
                   AND is_active = 1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                params![topic_id, audience],
                map_snapshot,
            )
            .optional()
            .map_err(storage_error)
    }

    fn get_evaluation_run(
        &self,
        evaluation_id: i64,
    ) -> EcoachResult<Option<ContentEvaluationRun>> {
        self.conn
            .query_row(
                "SELECT id, query_id, topic_id, metric_family, groundedness_bp, relevance_bp,
                        correctness_bp, completeness_bp, utilization_bp, notes_json, created_at
                 FROM content_evaluation_runs
                 WHERE id = ?1",
                [evaluation_id],
                map_evaluation_run,
            )
            .optional()
            .map_err(storage_error)
    }

    fn load_snapshot_artifact_items(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Vec<RetrievalCandidate>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.artifact_type, av.content_json, COALESCE(av.quality_score_bp, 7000)
                 FROM artifacts a
                 INNER JOIN artifact_versions av ON av.id = a.current_version_id
                 WHERE a.topic_id = ?1
                   AND a.lifecycle_state IN ('approved', 'live')
                 ORDER BY a.id ASC",
            )
            .map_err(storage_error)?;
        let rows = stmt
            .query_map([topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(storage_error)?;
        let mut items = Vec::new();
        for row in rows {
            let (artifact_id, artifact_type, content_json, quality_score) =
                row.map_err(storage_error)?;
            let parsed: Value = parse_json_text(&content_json).map_err(storage_error)?;
            items.push(RetrievalCandidate {
                snapshot_item_id: None,
                live_source_type: "artifact".to_string(),
                live_source_id: artifact_id,
                item_type: "artifact".to_string(),
                title: parsed["title"]
                    .as_str()
                    .unwrap_or("Generated artifact")
                    .to_string(),
                body: render_json_body(&parsed),
                citation_ref: Some(format!("artifact:{}", artifact_id)),
                quality_score_bp: clamp_bp(quality_score),
                freshness_score_bp: 7_200,
                metadata: json!({ "artifact_type": artifact_type, "topic_id": topic_id }),
            });
        }
        Ok(items)
    }

    fn load_snapshot_evidence_items(
        &self,
        topic_id: i64,
    ) -> EcoachResult<Vec<RetrievalCandidate>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, claim_text, supporting_text, final_quality_bp, freshness_score_bp,
                        source_upload_id, provenance_json
                 FROM evidence_blocks
                 WHERE topic_id = ?1
                 ORDER BY final_quality_bp DESC, id ASC",
            )
            .map_err(storage_error)?;
        let rows = stmt
            .query_map([topic_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(storage_error)?;
        let mut items = Vec::new();
        for row in rows {
            let (id, claim_text, supporting_text, quality, freshness, source_upload_id, provenance) =
                row.map_err(storage_error)?;
            let source_label = match source_upload_id {
                Some(source_upload_id) => self
                    .get_source(source_upload_id)?
                    .map(|source| source.title)
                    .unwrap_or_else(|| format!("Source {}", source_upload_id)),
                None => "Verified evidence".to_string(),
            };
            items.push(RetrievalCandidate {
                snapshot_item_id: None,
                live_source_type: "evidence".to_string(),
                live_source_id: id,
                item_type: "evidence".to_string(),
                title: source_label.clone(),
                body: supporting_text
                    .map(|supporting| format!("{}\n\n{}", claim_text, supporting))
                    .unwrap_or(claim_text),
                citation_ref: Some(format!("evidence:{}:{}", id, source_label)),
                quality_score_bp: clamp_bp(quality),
                freshness_score_bp: clamp_bp(freshness),
                metadata: parse_json_text(&provenance).map_err(storage_error)?,
            });
        }
        Ok(items)
    }

    fn load_snapshot_segment_items(
        &self,
        topic_id: i64,
        source_upload_id: Option<i64>,
    ) -> EcoachResult<Vec<RetrievalCandidate>> {
        let sql = if source_upload_id.is_some() {
            "SELECT segments.id, segments.source_upload_id, segments.section_title,
                    COALESCE(segments.markdown_text, segments.normalized_text, segments.raw_text),
                    segments.relevance_score_bp, uploads.title, uploads.freshness_score_bp,
                    segments.metadata_json
             FROM content_source_segments segments
             INNER JOIN curriculum_source_uploads uploads ON uploads.id = segments.source_upload_id
             WHERE segments.topic_id = ?1
               AND segments.source_upload_id = ?2
             ORDER BY segments.relevance_score_bp DESC, segments.id ASC"
        } else {
            "SELECT segments.id, segments.source_upload_id, segments.section_title,
                    COALESCE(segments.markdown_text, segments.normalized_text, segments.raw_text),
                    segments.relevance_score_bp, uploads.title, uploads.freshness_score_bp,
                    segments.metadata_json
             FROM content_source_segments segments
             INNER JOIN curriculum_source_uploads uploads ON uploads.id = segments.source_upload_id
             WHERE segments.topic_id = ?1
             ORDER BY segments.relevance_score_bp DESC, segments.id ASC"
        };
        let mut items = Vec::new();
        let mut stmt = self.conn.prepare(sql).map_err(storage_error)?;
        if let Some(source_upload_id) = source_upload_id {
            let rows = stmt
                .query_map(params![topic_id, source_upload_id], map_segment_row)
                .map_err(storage_error)?;
            for row in rows {
                let (segment_id, upload_id, section_title, body, relevance, source_title, freshness, metadata_json) =
                    row.map_err(storage_error)?;
                items.push(RetrievalCandidate {
                    snapshot_item_id: None,
                    live_source_type: "segment".to_string(),
                    live_source_id: segment_id,
                    item_type: "segment".to_string(),
                    title: section_title.unwrap_or_else(|| format!("{} segment", source_title)),
                    body,
                    citation_ref: Some(format!("source:{}:segment:{}", upload_id, segment_id)),
                    quality_score_bp: clamp_bp((relevance + 6_500) / 2),
                    freshness_score_bp: clamp_bp(freshness),
                    metadata: parse_json_text(&metadata_json).map_err(storage_error)?,
                });
            }
        } else {
            let rows = stmt
                .query_map([topic_id], map_segment_row)
                .map_err(storage_error)?;
            for row in rows {
                let (segment_id, upload_id, section_title, body, relevance, source_title, freshness, metadata_json) =
                    row.map_err(storage_error)?;
                items.push(RetrievalCandidate {
                    snapshot_item_id: None,
                    live_source_type: "segment".to_string(),
                    live_source_id: segment_id,
                    item_type: "segment".to_string(),
                    title: section_title.unwrap_or_else(|| format!("{} segment", source_title)),
                    body,
                    citation_ref: Some(format!("source:{}:segment:{}", upload_id, segment_id)),
                    quality_score_bp: clamp_bp((relevance + 6_500) / 2),
                    freshness_score_bp: clamp_bp(freshness),
                    metadata: parse_json_text(&metadata_json).map_err(storage_error)?,
                });
            }
        }
        Ok(items)
    }

    fn load_snapshot_retrieval_candidates(
        &self,
        snapshot_id: i64,
    ) -> EcoachResult<Vec<RetrievalCandidate>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source_type, source_ref, item_type, title, body_markdown,
                        citation_ref, quality_score_bp, freshness_score_bp, metadata_json
                 FROM content_snapshot_items
                 WHERE snapshot_id = ?1
                 ORDER BY display_order ASC, id ASC",
            )
            .map_err(storage_error)?;
        let rows = stmt
            .query_map([snapshot_id], |row| {
                Ok(RetrievalCandidate {
                    snapshot_item_id: Some(row.get::<_, i64>(0)?),
                    live_source_type: row.get::<_, String>(1)?,
                    live_source_id: row
                        .get::<_, String>(2)?
                        .parse::<i64>()
                        .unwrap_or_default(),
                    item_type: row.get::<_, String>(3)?,
                    title: row.get::<_, String>(4)?,
                    body: row.get::<_, String>(5)?,
                    citation_ref: row.get::<_, Option<String>>(6)?,
                    quality_score_bp: clamp_bp(row.get::<_, i64>(7)?),
                    freshness_score_bp: clamp_bp(row.get::<_, i64>(8)?),
                    metadata: parse_json_text(&row.get::<_, String>(9)?)?,
                })
            })
            .map_err(storage_error)?;
        collect_rows(rows)
    }

    fn load_live_retrieval_candidates(
        &self,
        topic_id: Option<i64>,
        subject_id: Option<i64>,
    ) -> EcoachResult<Vec<RetrievalCandidate>> {
        let mut items = Vec::new();
        if let Some(topic_id) = topic_id {
            items.extend(self.load_snapshot_artifact_items(topic_id)?);
            items.extend(self.load_snapshot_evidence_items(topic_id)?);
            items.extend(self.load_snapshot_segment_items(topic_id, None)?);
        } else if let Some(subject_id) = subject_id {
            let mut stmt = self
                .conn
                .prepare("SELECT id FROM topics WHERE subject_id = ?1 ORDER BY id ASC LIMIT 12")
                .map_err(storage_error)?;
            let rows = stmt
                .query_map([subject_id], |row| row.get::<_, i64>(0))
                .map_err(storage_error)?;
            for row in rows {
                let topic_id = row.map_err(storage_error)?;
                items.extend(self.load_snapshot_artifact_items(topic_id)?);
                items.extend(self.load_snapshot_evidence_items(topic_id)?);
            }
        }
        Ok(items)
    }

    fn resolve_decision_scope(
        &self,
        publish_job_id: Option<i64>,
        topic_id: Option<i64>,
        subject_id: Option<i64>,
    ) -> EcoachResult<(Option<i64>, Option<i64>)> {
        let Some(publish_job_id) = publish_job_id else {
            return Ok((topic_id, subject_id));
        };
        let job_scope = self
            .conn
            .query_row(
                "SELECT topic_id, subject_id FROM content_publish_jobs WHERE id = ?1",
                [publish_job_id],
                |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, Option<i64>>(1)?)),
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(|| EcoachError::NotFound(format!("publish job {} not found", publish_job_id)))?;
        Ok((topic_id.or(job_scope.0), subject_id.or(job_scope.1)))
    }

    fn log_research_event(
        &self,
        mission_id: i64,
        stage: &str,
        summary: &str,
        payload: &Value,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO content_research_events (
                    mission_id, stage, summary_text, payload_json
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![mission_id, stage, summary, to_json(payload)?],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    fn topic_name(&self, topic_id: i64) -> EcoachResult<String> {
        self.conn
            .query_row("SELECT name FROM topics WHERE id = ?1", [topic_id], |row| {
                row.get::<_, String>(0)
            })
            .map_err(storage_error)
    }

    fn count_query(&self, sql: &str, topic_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(sql, [topic_id], |row| row.get::<_, i64>(0))
            .map_err(storage_error)
    }
}

fn map_source_policy(row: &Row<'_>) -> rusqlite::Result<ContentSourcePolicy> {
    Ok(ContentSourcePolicy {
        id: row.get(0)?,
        policy_name: row.get(1)?,
        scope_type: row.get(2)?,
        scope_ref: row.get(3)?,
        source_kind: row.get(4)?,
        domain_pattern: row.get(5)?,
        access_mode: row.get(6)?,
        trust_tier: row.get(7)?,
        freshness_window_days: row.get(8)?,
        allow_crawl: row.get::<_, i64>(9)? == 1,
        allow_publish: row.get::<_, i64>(10)? == 1,
        notes: row.get(11)?,
        status: row.get(12)?,
    })
}

fn map_source_registry_entry(row: &Row<'_>) -> rusqlite::Result<ContentSourceRegistryEntry> {
    Ok(ContentSourceRegistryEntry {
        id: row.get(0)?,
        uploader_account_id: row.get(1)?,
        source_kind: row.get(2)?,
        title: row.get(3)?,
        source_path: row.get(4)?,
        country_code: row.get(5)?,
        exam_board: row.get(6)?,
        education_level: row.get(7)?,
        subject_code: row.get(8)?,
        academic_year: row.get(9)?,
        language_code: row.get(10)?,
        version_label: row.get(11)?,
        source_status: row.get(12)?,
        confidence_score: clamp_bp(row.get::<_, i64>(13)?),
        canonical_uri: row.get(14)?,
        publisher: row.get(15)?,
        author: row.get(16)?,
        publication_date: row.get(17)?,
        license_type: row.get(18)?,
        crawl_permission: row.get(19)?,
        source_tier: row.get(20)?,
        trust_score_bp: clamp_bp(row.get::<_, i64>(21)?),
        freshness_score_bp: clamp_bp(row.get::<_, i64>(22)?),
        parse_status_detail: row.get(23)?,
        allowlisted_domain: row.get::<_, i64>(24)? == 1,
        last_verified_at: row.get(25)?,
        review_due_at: row.get(26)?,
        stale_flag: row.get::<_, i64>(27)? == 1,
        metadata: parse_json_text(&row.get::<_, String>(28)?)?,
    })
}

fn map_source_segment(row: &Row<'_>) -> rusqlite::Result<ContentSourceSegment> {
    Ok(ContentSourceSegment {
        id: row.get(0)?,
        source_upload_id: row.get(1)?,
        topic_id: row.get(2)?,
        concept_id: row.get(3)?,
        section_title: row.get(4)?,
        raw_text: row.get(5)?,
        normalized_text: row.get(6)?,
        markdown_text: row.get(7)?,
        image_refs: parse_json_text(&row.get::<_, String>(8)?)?,
        equation_refs: parse_json_text(&row.get::<_, String>(9)?)?,
        page_range: row.get(10)?,
        checksum: row.get(11)?,
        semantic_hash: row.get(12)?,
        extraction_confidence_bp: clamp_bp(row.get::<_, i64>(13)?),
        relevance_score_bp: clamp_bp(row.get::<_, i64>(14)?),
        metadata: parse_json_text(&row.get::<_, String>(15)?)?,
        created_at: row.get(16)?,
    })
}

fn map_research_mission(row: &Row<'_>) -> rusqlite::Result<ContentResearchMission> {
    Ok(ContentResearchMission {
        id: row.get(0)?,
        acquisition_job_id: row.get(1)?,
        gap_ticket_id: row.get(2)?,
        source_upload_id: row.get(3)?,
        subject_id: row.get(4)?,
        topic_id: row.get(5)?,
        mission_type: row.get(6)?,
        mission_brief: row.get(7)?,
        allowed_source_classes: parse_json_text(&row.get::<_, String>(8)?)?,
        requested_asset_types: parse_json_text(&row.get::<_, String>(9)?)?,
        coverage_snapshot: parse_json_text(&row.get::<_, String>(10)?)?,
        priority_bp: clamp_bp(row.get::<_, i64>(11)?),
        mission_stage: row.get(12)?,
        status: row.get(13)?,
        planner_notes: row.get(14)?,
    })
}

fn map_research_candidate(row: &Row<'_>) -> rusqlite::Result<ContentResearchCandidate> {
    Ok(ContentResearchCandidate {
        id: row.get(0)?,
        mission_id: row.get(1)?,
        acquisition_candidate_id: row.get(2)?,
        source_upload_id: row.get(3)?,
        source_label: row.get(4)?,
        source_url: row.get(5)?,
        source_kind: row.get(6)?,
        title: row.get(7)?,
        snippet: row.get(8)?,
        source_tier: row.get(9)?,
        authority_score_bp: clamp_bp(row.get::<_, i64>(10)?),
        relevance_score_bp: clamp_bp(row.get::<_, i64>(11)?),
        freshness_score_bp: clamp_bp(row.get::<_, i64>(12)?),
        license_type: row.get(13)?,
        selected_for_verification: row.get::<_, i64>(14)? == 1,
        payload: parse_json_text(&row.get::<_, String>(15)?)?,
    })
}

fn map_evidence_record(row: &Row<'_>) -> rusqlite::Result<ContentEvidenceRecord> {
    Ok(ContentEvidenceRecord {
        id: row.get(0)?,
        source_type: row.get(1)?,
        source_id: row.get(2)?,
        source_upload_id: row.get(3)?,
        source_segment_id: row.get(4)?,
        topic_id: row.get(5)?,
        concept_id: row.get(6)?,
        evidence_type: row.get(7)?,
        claim_text: row.get(8)?,
        supporting_text: row.get(9)?,
        extraction_confidence_bp: clamp_bp(row.get::<_, i64>(10)?),
        corroboration_score_bp: clamp_bp(row.get::<_, i64>(11)?),
        contradiction_score_bp: clamp_bp(row.get::<_, i64>(12)?),
        pedagogy_score_bp: clamp_bp(row.get::<_, i64>(13)?),
        freshness_score_bp: clamp_bp(row.get::<_, i64>(14)?),
        final_quality_bp: clamp_bp(row.get::<_, i64>(15)?),
        status: row.get(16)?,
        verified_at: row.get(17)?,
        provenance: parse_json_text(&row.get::<_, String>(18)?)?,
    })
}

fn map_publish_decision(row: &Row<'_>) -> rusqlite::Result<ContentPublishDecision> {
    Ok(ContentPublishDecision {
        id: row.get(0)?,
        publish_job_id: row.get(1)?,
        source_upload_id: row.get(2)?,
        subject_id: row.get(3)?,
        topic_id: row.get(4)?,
        gate_name: row.get(5)?,
        decision_status: row.get(6)?,
        decision_reason: row.get(7)?,
        decision_score_bp: clamp_bp(row.get::<_, i64>(8)?),
        snapshot_id: row.get(9)?,
        decided_by_account_id: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn map_snapshot(row: &Row<'_>) -> rusqlite::Result<ContentSnapshot> {
    Ok(ContentSnapshot {
        id: row.get(0)?,
        topic_id: row.get(1)?,
        subject_id: row.get(2)?,
        audience_type: row.get(3)?,
        snapshot_kind: row.get(4)?,
        label: row.get(5)?,
        status: row.get(6)?,
        source_publish_job_id: row.get(7)?,
        source_upload_id: row.get(8)?,
        manifest: parse_json_text(&row.get::<_, String>(9)?)?,
        fingerprint: row.get(10)?,
        item_count: row.get(11)?,
        is_active: row.get::<_, i64>(12)? == 1,
        created_at: row.get(13)?,
    })
}

fn map_evaluation_run(row: &Row<'_>) -> rusqlite::Result<ContentEvaluationRun> {
    Ok(ContentEvaluationRun {
        id: row.get(0)?,
        query_id: row.get(1)?,
        topic_id: row.get(2)?,
        metric_family: row.get(3)?,
        groundedness_bp: clamp_bp(row.get::<_, i64>(4)?),
        relevance_bp: clamp_bp(row.get::<_, i64>(5)?),
        correctness_bp: clamp_bp(row.get::<_, i64>(6)?),
        completeness_bp: clamp_bp(row.get::<_, i64>(7)?),
        utilization_bp: clamp_bp(row.get::<_, i64>(8)?),
        notes: parse_json_text(&row.get::<_, String>(9)?)?,
        created_at: row.get(10)?,
    })
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&Row<'_>) -> rusqlite::Result<T>>,
) -> EcoachResult<Vec<T>> {
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(storage_error)?);
    }
    Ok(items)
}

fn parse_json_text<T>(raw: &str) -> rusqlite::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    })
}

fn to_json(value: &impl Serialize) -> EcoachResult<String> {
    serde_json::to_string(value).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn storage_error(err: rusqlite::Error) -> EcoachError {
    EcoachError::Storage(err.to_string())
}

fn map_segment_row(
    row: &Row<'_>,
) -> rusqlite::Result<(
    i64,
    i64,
    Option<String>,
    String,
    i64,
    String,
    i64,
    String,
)> {
    Ok((
        row.get::<_, i64>(0)?,
        row.get::<_, i64>(1)?,
        row.get::<_, Option<String>>(2)?,
        row.get::<_, String>(3)?,
        row.get::<_, i64>(4)?,
        row.get::<_, String>(5)?,
        row.get::<_, i64>(6)?,
        row.get::<_, String>(7)?,
    ))
}

fn merge_json(base: Value, patch: Option<Value>) -> Value {
    let Some(patch) = patch else {
        return base;
    };
    if !base.is_object() || !patch.is_object() {
        return patch;
    }
    let mut merged: Map<String, Value> = base.as_object().cloned().unwrap_or_default();
    for (key, value) in patch.as_object().cloned().unwrap_or_default() {
        merged.insert(key, value);
    }
    Value::Object(merged)
}

fn mission_type_to_intent(mission_type: &str) -> &'static str {
    match normalize_text(mission_type).as_str() {
        "refresh" => "refresh",
        "corroborate" => "corroborate",
        "example hunt" | "example_hunt" => "example_hunt",
        "question hunt" | "question_hunt" => "question_hunt",
        "glossary hunt" | "glossary_hunt" => "glossary_hunt",
        _ => "gap_fill",
    }
}

fn infer_source_scope(allowed_source_classes: &[String]) -> &'static str {
    let normalized: BTreeSet<String> = allowed_source_classes
        .iter()
        .map(|value| normalize_text(value))
        .collect();
    if normalized.is_empty() || normalized.contains("approved web") || normalized.contains("web") {
        "mixed"
    } else if normalized.contains("internal") || normalized.contains("upload") {
        "internal"
    } else {
        "approved_web"
    }
}

fn normalize_text(text: &str) -> String {
    text.to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character.is_ascii_whitespace() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_json_body(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(render_json_body)
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(object) => object
            .values()
            .map(render_json_body)
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn excerpt(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    trimmed.chars().take(max_chars).collect::<String>()
}

fn candidate_matches_requested_types(
    metadata: &Value,
    requested_types: &BTreeSet<String>,
) -> bool {
    requested_types.iter().any(|requested| {
        metadata
            .get("artifact_type")
            .and_then(Value::as_str)
            .map(normalize_text)
            .as_ref()
            == Some(requested)
            || metadata
                .get("content_type")
                .and_then(Value::as_str)
                .map(normalize_text)
                .as_ref()
                == Some(requested)
    })
}

fn score_candidate(
    normalized_query: &str,
    candidate: &RetrievalCandidate,
    topic_id: Option<i64>,
) -> BasisPoints {
    let terms: Vec<&str> = normalized_query
        .split_whitespace()
        .filter(|term| term.len() > 1)
        .collect();
    if terms.is_empty() {
        return clamp_bp(
            (candidate.quality_score_bp as i64 + candidate.freshness_score_bp as i64) / 2,
        );
    }
    let title_norm = normalize_text(&candidate.title);
    let body_norm = normalize_text(&candidate.body);
    let mut matched_terms = 0_i64;
    let mut title_bonus = 0_i64;
    for term in &terms {
        if title_norm.contains(term) || body_norm.contains(term) {
            matched_terms += 1;
        }
        if title_norm.contains(term) {
            title_bonus += 750;
        }
    }
    if matched_terms == 0 {
        return 0;
    }
    let lexical = matched_terms * 10_000 / terms.len() as i64;
    let topic_bonus = if topic_id.is_some()
        && candidate
            .metadata
            .get("topic_id")
            .and_then(Value::as_i64)
            == topic_id
    {
        800
    } else {
        0
    };
    clamp_bp(
        (lexical * 3 / 5)
            + (candidate.quality_score_bp as i64 / 4)
            + (candidate.freshness_score_bp as i64 / 8)
            + title_bonus.min(2_000)
            + topic_bonus,
    )
}
