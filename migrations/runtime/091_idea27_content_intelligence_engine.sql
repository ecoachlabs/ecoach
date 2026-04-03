-- idea27: Content intelligence engine completion.
-- Adds the durable source-registry, research, provenance, snapshot,
-- retrieval, and evaluation runtime needed for the always-improving
-- content intelligence loop.

ALTER TABLE curriculum_source_uploads ADD COLUMN canonical_uri TEXT;
ALTER TABLE curriculum_source_uploads ADD COLUMN publisher TEXT;
ALTER TABLE curriculum_source_uploads ADD COLUMN author TEXT;
ALTER TABLE curriculum_source_uploads ADD COLUMN publication_date TEXT;
ALTER TABLE curriculum_source_uploads ADD COLUMN license_type TEXT;
ALTER TABLE curriculum_source_uploads ADD COLUMN crawl_permission TEXT NOT NULL DEFAULT 'internal_only'
    CHECK (crawl_permission IN ('internal_only', 'allowlisted_only', 'manual_only', 'disabled'));
ALTER TABLE curriculum_source_uploads ADD COLUMN source_tier TEXT NOT NULL DEFAULT 'staged'
    CHECK (source_tier IN ('official', 'trusted', 'community', 'web', 'staged', 'blocked'));
ALTER TABLE curriculum_source_uploads ADD COLUMN trust_score_bp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE curriculum_source_uploads ADD COLUMN freshness_score_bp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE curriculum_source_uploads ADD COLUMN parse_status_detail TEXT NOT NULL DEFAULT 'uploaded';
ALTER TABLE curriculum_source_uploads ADD COLUMN allowlisted_domain INTEGER NOT NULL DEFAULT 0;
ALTER TABLE curriculum_source_uploads ADD COLUMN last_verified_at TEXT;
ALTER TABLE curriculum_source_uploads ADD COLUMN review_due_at TEXT;
ALTER TABLE curriculum_source_uploads ADD COLUMN stale_flag INTEGER NOT NULL DEFAULT 0;

UPDATE curriculum_source_uploads
SET source_tier = CASE
        WHEN source_kind IN ('curriculum', 'syllabus') THEN 'official'
        WHEN source_kind IN ('guide', 'textbook') THEN 'trusted'
        WHEN source_kind = 'web_source' THEN 'web'
        ELSE 'community'
    END,
    crawl_permission = CASE
        WHEN source_kind = 'web_source' THEN 'allowlisted_only'
        ELSE 'internal_only'
    END,
    trust_score_bp = CASE
        WHEN confidence_score > 0 THEN confidence_score
        WHEN source_kind IN ('curriculum', 'syllabus') THEN 9000
        WHEN source_kind IN ('guide', 'textbook') THEN 7800
        ELSE 6200
    END,
    freshness_score_bp = CASE
        WHEN academic_year IS NOT NULL AND academic_year <> '' THEN 7600
        WHEN source_kind = 'web_source' THEN 6800
        ELSE 7200
    END,
    parse_status_detail = source_status,
    allowlisted_domain = CASE
        WHEN source_kind <> 'web_source' THEN 1
        WHEN source_path IS NOT NULL AND source_path NOT LIKE '%http%' THEN 1
        ELSE 0
    END,
    last_verified_at = CASE
        WHEN source_status IN ('reviewed', 'published') THEN COALESCE(last_verified_at, created_at)
        ELSE last_verified_at
    END,
    review_due_at = COALESCE(
        review_due_at,
        CASE
            WHEN source_status IN ('reviewed', 'published') THEN datetime(created_at, '+90 day')
            ELSE NULL
        END
    ),
    stale_flag = CASE
        WHEN source_status = 'archived' THEN 1
        ELSE stale_flag
    END;

CREATE TABLE IF NOT EXISTS content_source_policies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    policy_name TEXT NOT NULL,
    scope_type TEXT NOT NULL DEFAULT 'global'
        CHECK (scope_type IN ('global', 'subject', 'topic', 'source_kind', 'domain')),
    scope_ref TEXT,
    source_kind TEXT,
    domain_pattern TEXT,
    access_mode TEXT NOT NULL DEFAULT 'allowlisted_only'
        CHECK (access_mode IN ('internal_only', 'allowlisted_only', 'manual_only', 'disabled')),
    trust_tier TEXT NOT NULL DEFAULT 'trusted'
        CHECK (trust_tier IN ('official', 'trusted', 'community', 'web', 'blocked')),
    freshness_window_days INTEGER NOT NULL DEFAULT 90,
    allow_crawl INTEGER NOT NULL DEFAULT 0,
    allow_publish INTEGER NOT NULL DEFAULT 1,
    notes TEXT,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'retired')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_source_policies_scope
    ON content_source_policies(scope_type, status);

INSERT INTO content_source_policies (
    policy_name, scope_type, source_kind, access_mode, trust_tier,
    freshness_window_days, allow_crawl, allow_publish, notes
) VALUES
    (
        'Official curriculum documents',
        'source_kind',
        'curriculum',
        'internal_only',
        'official',
        365,
        0,
        1,
        'Canonical internal curriculum uploads and official documents.'
    ),
    (
        'Approved syllabus uploads',
        'source_kind',
        'syllabus',
        'internal_only',
        'official',
        180,
        0,
        1,
        'High-trust syllabus and exam board materials.'
    ),
    (
        'Allowlisted web research',
        'source_kind',
        'web_source',
        'allowlisted_only',
        'web',
        45,
        1,
        0,
        'Web research must stay in staging until a publish decision approves it.'
    )
ON CONFLICT DO NOTHING;

CREATE TABLE IF NOT EXISTS content_source_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_upload_id INTEGER NOT NULL REFERENCES curriculum_source_uploads(id) ON DELETE CASCADE,
    topic_id INTEGER REFERENCES topics(id) ON DELETE SET NULL,
    concept_id INTEGER REFERENCES academic_nodes(id) ON DELETE SET NULL,
    section_title TEXT,
    raw_text TEXT NOT NULL,
    normalized_text TEXT,
    markdown_text TEXT,
    image_refs_json TEXT NOT NULL DEFAULT '[]',
    equation_refs_json TEXT NOT NULL DEFAULT '[]',
    page_range TEXT,
    checksum TEXT,
    semantic_hash TEXT,
    extraction_confidence_bp INTEGER NOT NULL DEFAULT 0,
    relevance_score_bp INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_source_segments_source
    ON content_source_segments(source_upload_id, topic_id);
CREATE INDEX IF NOT EXISTS idx_content_source_segments_hash
    ON content_source_segments(source_upload_id, semantic_hash);

ALTER TABLE content_acquisition_jobs ADD COLUMN allowed_source_classes_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE content_acquisition_jobs ADD COLUMN requested_asset_types_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE content_acquisition_jobs ADD COLUMN coverage_snapshot_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE content_acquisition_jobs ADD COLUMN mission_stage TEXT NOT NULL DEFAULT 'planned'
    CHECK (mission_stage IN (
        'planned', 'scouting', 'extracting', 'verifying',
        'synthesizing', 'ready_for_gate', 'published', 'failed'
    ));
ALTER TABLE content_acquisition_jobs ADD COLUMN planner_notes TEXT;

ALTER TABLE acquisition_evidence_candidates ADD COLUMN authority_score_bp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE acquisition_evidence_candidates ADD COLUMN relevance_score_bp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE acquisition_evidence_candidates ADD COLUMN source_tier TEXT NOT NULL DEFAULT 'staged'
    CHECK (source_tier IN ('official', 'trusted', 'community', 'web', 'staged', 'blocked'));
ALTER TABLE acquisition_evidence_candidates ADD COLUMN license_type TEXT;

CREATE TABLE IF NOT EXISTS content_research_missions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    acquisition_job_id INTEGER NOT NULL REFERENCES content_acquisition_jobs(id) ON DELETE CASCADE,
    gap_ticket_id INTEGER REFERENCES content_gap_tickets(id) ON DELETE SET NULL,
    source_upload_id INTEGER REFERENCES curriculum_source_uploads(id) ON DELETE SET NULL,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE SET NULL,
    topic_id INTEGER REFERENCES topics(id) ON DELETE SET NULL,
    mission_type TEXT NOT NULL DEFAULT 'gap_fill',
    mission_brief TEXT NOT NULL,
    allowed_source_classes_json TEXT NOT NULL DEFAULT '[]',
    requested_asset_types_json TEXT NOT NULL DEFAULT '[]',
    coverage_snapshot_json TEXT NOT NULL DEFAULT '{}',
    priority_bp INTEGER NOT NULL DEFAULT 5000,
    mission_stage TEXT NOT NULL DEFAULT 'planned'
        CHECK (mission_stage IN (
            'planned', 'scouting', 'extracting', 'verifying',
            'synthesizing', 'ready_for_gate', 'published', 'failed'
        )),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'review_required', 'completed', 'failed')),
    created_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_research_missions_topic
    ON content_research_missions(topic_id, status, mission_stage);

CREATE TABLE IF NOT EXISTS content_research_candidates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mission_id INTEGER NOT NULL REFERENCES content_research_missions(id) ON DELETE CASCADE,
    acquisition_candidate_id INTEGER REFERENCES acquisition_evidence_candidates(id) ON DELETE SET NULL,
    source_upload_id INTEGER REFERENCES curriculum_source_uploads(id) ON DELETE SET NULL,
    source_label TEXT NOT NULL,
    source_url TEXT,
    source_kind TEXT NOT NULL,
    title TEXT,
    snippet TEXT,
    source_tier TEXT NOT NULL DEFAULT 'staged'
        CHECK (source_tier IN ('official', 'trusted', 'community', 'web', 'staged', 'blocked')),
    authority_score_bp INTEGER NOT NULL DEFAULT 0,
    relevance_score_bp INTEGER NOT NULL DEFAULT 0,
    freshness_score_bp INTEGER NOT NULL DEFAULT 0,
    license_type TEXT,
    selected_for_verification INTEGER NOT NULL DEFAULT 0,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_research_candidates_mission
    ON content_research_candidates(mission_id, source_tier);

CREATE TABLE IF NOT EXISTS content_research_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mission_id INTEGER NOT NULL REFERENCES content_research_missions(id) ON DELETE CASCADE,
    stage TEXT NOT NULL,
    summary_text TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_research_events_mission
    ON content_research_events(mission_id, created_at DESC);

ALTER TABLE evidence_blocks ADD COLUMN source_upload_id INTEGER REFERENCES curriculum_source_uploads(id) ON DELETE SET NULL;
ALTER TABLE evidence_blocks ADD COLUMN source_segment_id INTEGER REFERENCES content_source_segments(id) ON DELETE SET NULL;
ALTER TABLE evidence_blocks ADD COLUMN contradiction_score_bp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE evidence_blocks ADD COLUMN provenance_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE evidence_blocks ADD COLUMN verified_at TEXT;

CREATE INDEX IF NOT EXISTS idx_evidence_blocks_segment
    ON evidence_blocks(source_segment_id, status);

CREATE TABLE IF NOT EXISTS content_publish_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    publish_job_id INTEGER REFERENCES content_publish_jobs(id) ON DELETE SET NULL,
    source_upload_id INTEGER REFERENCES curriculum_source_uploads(id) ON DELETE SET NULL,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE SET NULL,
    topic_id INTEGER REFERENCES topics(id) ON DELETE SET NULL,
    gate_name TEXT NOT NULL,
    decision_status TEXT NOT NULL
        CHECK (decision_status IN ('approved', 'preview', 'rejected', 'rollback')),
    decision_reason TEXT NOT NULL,
    decision_score_bp INTEGER NOT NULL DEFAULT 0,
    decision_trace_json TEXT NOT NULL DEFAULT '{}',
    snapshot_id INTEGER,
    decided_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_publish_decisions_topic
    ON content_publish_decisions(topic_id, decision_status, created_at DESC);

CREATE TABLE IF NOT EXISTS content_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE SET NULL,
    audience_type TEXT NOT NULL DEFAULT 'student'
        CHECK (audience_type IN ('student', 'coach', 'parent', 'admin')),
    snapshot_kind TEXT NOT NULL DEFAULT 'verified_topic_bundle'
        CHECK (snapshot_kind IN ('verified_topic_bundle', 'preview_topic_bundle', 'rollback_bundle')),
    label TEXT,
    status TEXT NOT NULL DEFAULT 'published'
        CHECK (status IN ('preview', 'published', 'rolled_back')),
    source_publish_job_id INTEGER REFERENCES content_publish_jobs(id) ON DELETE SET NULL,
    source_upload_id INTEGER REFERENCES curriculum_source_uploads(id) ON DELETE SET NULL,
    manifest_json TEXT NOT NULL DEFAULT '{}',
    fingerprint TEXT NOT NULL,
    item_count INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_snapshots_topic
    ON content_snapshots(topic_id, audience_type, is_active, created_at DESC);

CREATE TABLE IF NOT EXISTS content_snapshot_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER NOT NULL REFERENCES content_snapshots(id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    source_ref TEXT NOT NULL,
    item_type TEXT NOT NULL,
    title TEXT NOT NULL,
    body_markdown TEXT NOT NULL,
    citation_ref TEXT,
    quality_score_bp INTEGER NOT NULL DEFAULT 0,
    freshness_score_bp INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_snapshot_items_snapshot
    ON content_snapshot_items(snapshot_id, item_type, display_order);

CREATE TABLE IF NOT EXISTS content_retrieval_queries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE SET NULL,
    topic_id INTEGER REFERENCES topics(id) ON DELETE SET NULL,
    audience_type TEXT,
    query_text TEXT NOT NULL,
    filters_json TEXT NOT NULL DEFAULT '{}',
    snapshot_id INTEGER REFERENCES content_snapshots(id) ON DELETE SET NULL,
    result_count INTEGER NOT NULL DEFAULT 0,
    citation_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'served', 'empty', 'failed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_retrieval_queries_topic
    ON content_retrieval_queries(topic_id, created_at DESC);

CREATE TABLE IF NOT EXISTS content_retrieval_hits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_id INTEGER NOT NULL REFERENCES content_retrieval_queries(id) ON DELETE CASCADE,
    snapshot_item_id INTEGER REFERENCES content_snapshot_items(id) ON DELETE SET NULL,
    live_source_type TEXT,
    live_source_id INTEGER,
    rank_index INTEGER NOT NULL,
    score_bp INTEGER NOT NULL DEFAULT 0,
    citation_ref TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_retrieval_hits_query
    ON content_retrieval_hits(query_id, rank_index);

CREATE TABLE IF NOT EXISTS content_evaluation_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_id INTEGER REFERENCES content_retrieval_queries(id) ON DELETE SET NULL,
    topic_id INTEGER REFERENCES topics(id) ON DELETE SET NULL,
    metric_family TEXT NOT NULL,
    groundedness_bp INTEGER NOT NULL DEFAULT 0,
    relevance_bp INTEGER NOT NULL DEFAULT 0,
    correctness_bp INTEGER NOT NULL DEFAULT 0,
    completeness_bp INTEGER NOT NULL DEFAULT 0,
    utilization_bp INTEGER NOT NULL DEFAULT 0,
    notes_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_evaluation_runs_topic
    ON content_evaluation_runs(topic_id, created_at DESC);
