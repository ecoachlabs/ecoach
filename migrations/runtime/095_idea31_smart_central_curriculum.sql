-- idea31: smart central curriculum completion
-- Adds source fidelity, node intelligence, cohort pinning, regeneration jobs,
-- and role-specific curriculum projections on top of the existing portal graph.

CREATE TABLE IF NOT EXISTS curriculum_node_citations (
    id INTEGER PRIMARY KEY,
    curriculum_node_id INTEGER NOT NULL REFERENCES curriculum_nodes(id) ON DELETE CASCADE,
    source_upload_id INTEGER REFERENCES curriculum_source_uploads(id) ON DELETE SET NULL,
    citation_kind TEXT NOT NULL DEFAULT 'official_text'
        CHECK (citation_kind IN (
            'official_text', 'page_reference', 'source_snippet', 'exemplar', 'comment'
        )),
    reference_code TEXT,
    source_file_label TEXT,
    source_page INTEGER,
    source_section TEXT,
    source_snippet TEXT,
    ocr_confidence_score INTEGER NOT NULL DEFAULT 0,
    parsing_confidence_score INTEGER NOT NULL DEFAULT 0,
    review_status TEXT NOT NULL DEFAULT 'pending_review'
        CHECK (review_status IN ('pending_review', 'approved', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_curriculum_node_citations_node
ON curriculum_node_citations(curriculum_node_id, citation_kind, source_page);

CREATE TABLE IF NOT EXISTS curriculum_node_exemplars (
    id INTEGER PRIMARY KEY,
    curriculum_node_id INTEGER NOT NULL REFERENCES curriculum_nodes(id) ON DELETE CASCADE,
    citation_id INTEGER REFERENCES curriculum_node_citations(id) ON DELETE SET NULL,
    exemplar_kind TEXT NOT NULL DEFAULT 'example'
        CHECK (exemplar_kind IN (
            'example', 'non_example', 'boundary_note', 'teacher_hint',
            'local_context_cue', 'misconception_alert', 'application_cue',
            'assessment_cue', 'scaffold_suggestion'
        )),
    raw_text TEXT NOT NULL,
    public_text TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    display_order INTEGER NOT NULL DEFAULT 0,
    review_status TEXT NOT NULL DEFAULT 'pending_review'
        CHECK (review_status IN ('pending_review', 'approved', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_curriculum_node_exemplars_node
ON curriculum_node_exemplars(curriculum_node_id, exemplar_kind, display_order);

CREATE TABLE IF NOT EXISTS curriculum_node_comments (
    id INTEGER PRIMARY KEY,
    curriculum_node_id INTEGER NOT NULL REFERENCES curriculum_nodes(id) ON DELETE CASCADE,
    citation_id INTEGER REFERENCES curriculum_node_citations(id) ON DELETE SET NULL,
    comment_type TEXT NOT NULL DEFAULT 'teacher_hint'
        CHECK (comment_type IN (
            'teacher_hint', 'scope_guard', 'local_context_note', 'misconception_alert',
            'application_note', 'assessment_note', 'scaffold_note', 'boundary_note'
        )),
    comment_text TEXT NOT NULL,
    public_text TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    display_order INTEGER NOT NULL DEFAULT 0,
    review_status TEXT NOT NULL DEFAULT 'pending_review'
        CHECK (review_status IN ('pending_review', 'approved', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_curriculum_node_comments_node
ON curriculum_node_comments(curriculum_node_id, comment_type, display_order);

CREATE TABLE IF NOT EXISTS curriculum_node_intelligence (
    id INTEGER PRIMARY KEY,
    curriculum_node_id INTEGER NOT NULL UNIQUE REFERENCES curriculum_nodes(id) ON DELETE CASCADE,
    friendly_topic_name TEXT,
    internal_subtopic_atoms_json TEXT NOT NULL DEFAULT '[]',
    knowledge_points_json TEXT NOT NULL DEFAULT '[]',
    skills_json TEXT NOT NULL DEFAULT '[]',
    cognitive_verb TEXT,
    expected_evidence_type TEXT,
    instructional_mode TEXT,
    assessment_mode TEXT,
    misconception_tags_json TEXT NOT NULL DEFAULT '[]',
    prerequisite_node_ids_json TEXT NOT NULL DEFAULT '[]',
    dependent_node_ids_json TEXT NOT NULL DEFAULT '[]',
    difficulty_ladder_json TEXT NOT NULL DEFAULT '[]',
    teaching_strategies_json TEXT NOT NULL DEFAULT '[]',
    question_families_json TEXT NOT NULL DEFAULT '[]',
    worked_example_templates_json TEXT NOT NULL DEFAULT '[]',
    memory_tags_json TEXT NOT NULL DEFAULT '[]',
    local_context_examples_json TEXT NOT NULL DEFAULT '[]',
    exam_mapping_json TEXT NOT NULL DEFAULT '{}',
    notes_json TEXT NOT NULL DEFAULT '{}',
    approval_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (approval_status IN ('draft', 'review', 'approved', 'superseded')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS curriculum_cohort_pins (
    id INTEGER PRIMARY KEY,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id) ON DELETE CASCADE,
    cohort_key TEXT NOT NULL,
    cohort_label TEXT NOT NULL,
    level_code TEXT,
    effective_from TEXT,
    effective_to TEXT,
    rollout_status TEXT NOT NULL DEFAULT 'active'
        CHECK (rollout_status IN ('draft', 'scheduled', 'active', 'retired')),
    pinned_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(curriculum_version_id, cohort_key)
);

CREATE INDEX IF NOT EXISTS idx_curriculum_cohort_pins_version
ON curriculum_cohort_pins(curriculum_version_id, rollout_status, cohort_key);

CREATE TABLE IF NOT EXISTS student_curriculum_assignments (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id) ON DELETE CASCADE,
    cohort_pin_id INTEGER REFERENCES curriculum_cohort_pins(id) ON DELETE SET NULL,
    assignment_source TEXT NOT NULL DEFAULT 'manual'
        CHECK (assignment_source IN ('manual', 'cohort', 'default')),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'superseded', 'archived')),
    notes TEXT,
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_student_curriculum_assignments_student
ON student_curriculum_assignments(student_id, status, assigned_at DESC);

CREATE TABLE IF NOT EXISTS curriculum_regeneration_jobs (
    id INTEGER PRIMARY KEY,
    base_version_id INTEGER REFERENCES curriculum_versions(id) ON DELETE SET NULL,
    compare_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id) ON DELETE CASCADE,
    affected_node_id INTEGER REFERENCES curriculum_nodes(id) ON DELETE SET NULL,
    entity_type TEXT NOT NULL DEFAULT 'node',
    entity_key TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'interpretive'
        CHECK (severity IN ('cosmetic', 'interpretive', 'structural', 'high_impact')),
    action_required TEXT NOT NULL DEFAULT 'review'
        CHECK (action_required IN (
            'refresh_metadata', 'review', 'regenerate', 'deprecate', 'resequence'
        )),
    resource_type TEXT NOT NULL DEFAULT 'mixed'
        CHECK (resource_type IN ('mixed', 'question', 'lesson', 'drill', 'diagnostic', 'note')),
    resource_count INTEGER NOT NULL DEFAULT 0,
    impact_summary TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'ready', 'running', 'completed', 'cancelled')),
    triggered_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_curriculum_regeneration_jobs_version
ON curriculum_regeneration_jobs(compare_version_id, status, severity, created_at DESC);
