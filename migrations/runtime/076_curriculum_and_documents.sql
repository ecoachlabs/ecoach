-- ideas 34, 36: Smart curriculum interpretations, document intelligence,
-- artifact lifecycle, content foundry extensions.

-- ============================================================================
-- 1. Curriculum interpretations (enrichment layer)
-- ============================================================================

CREATE TABLE curriculum_interpretations (
    id INTEGER PRIMARY KEY,
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    friendly_name TEXT,
    knowledge_points_json TEXT,
    skills_involved_json TEXT,
    cognitive_verb TEXT,
    expected_evidence_type TEXT,
    common_misconceptions_json TEXT,
    prerequisite_nodes_json TEXT,
    dependent_nodes_json TEXT,
    difficulty_ladder_json TEXT,
    teaching_strategies_json TEXT,
    question_families_json TEXT,
    worked_example_templates_json TEXT,
    memory_recall_tags_json TEXT,
    local_context_examples_json TEXT,
    bece_mapping_json TEXT,
    approval_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (approval_status IN ('draft', 'pending_review', 'approved', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(topic_id)
);

-- ============================================================================
-- 2. Curriculum coverage ledger (per-student topic coverage)
-- ============================================================================

CREATE TABLE curriculum_coverage_ledger (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    coverage_status TEXT NOT NULL DEFAULT 'not_introduced'
        CHECK (coverage_status IN (
            'not_introduced', 'introduced', 'taught', 'practiced',
            'assessed', 'mastered', 'unstable', 'decayed', 're_opened'
        )),
    coverage_timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX idx_coverage_ledger_student ON curriculum_coverage_ledger(student_id);

-- ============================================================================
-- 3. Student-facing curriculum (simplified view)
-- ============================================================================

CREATE TABLE student_facing_curriculum (
    id INTEGER PRIMARY KEY,
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    simplified_name TEXT NOT NULL,
    simplified_description TEXT,
    what_to_know TEXT,
    what_to_do TEXT,
    examples_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(topic_id)
);

-- ============================================================================
-- 4. Document uploads and intelligence mining
-- ============================================================================

CREATE TABLE document_uploads (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    uploader_id INTEGER NOT NULL REFERENCES accounts(id),
    file_name TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_size_bytes INTEGER,
    document_type TEXT NOT NULL DEFAULT 'academic_material'
        CHECK (document_type IN (
            'academic_material', 'assessment_evidence', 'instructional_guidance',
            'student_generated', 'report_card', 'mock_exam_result',
            'correction_sheet', 'teacher_notes', 'syllabus'
        )),
    source_category TEXT NOT NULL DEFAULT 'student'
        CHECK (source_category IN ('official', 'teacher', 'student', 'extracted')),
    trust_level TEXT NOT NULL DEFAULT 'low'
        CHECK (trust_level IN ('low', 'medium', 'high', 'verified')),
    parse_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (parse_status IN ('pending', 'parsing', 'parsed', 'failed', 'reviewed')),
    extracted_entities_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_document_uploads_student ON document_uploads(student_id);

-- ============================================================================
-- 5. Document mining outputs
-- ============================================================================

CREATE TABLE document_mining_outputs (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES document_uploads(id),
    topics_detected_json TEXT,
    weak_areas_json TEXT,
    teacher_concerns_json TEXT,
    concepts_needing_remediation_json TEXT,
    deadlines_json TEXT,
    formulas_json TEXT,
    definitions_json TEXT,
    question_patterns_json TEXT,
    marking_scheme_clues_json TEXT,
    scores_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_mining_outputs_document ON document_mining_outputs(document_id);

-- ============================================================================
-- 6. Document actions triggered
-- ============================================================================

CREATE TABLE document_actions (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES document_uploads(id),
    action_type TEXT NOT NULL
        CHECK (action_type IN (
            'create_goal', 'recommend_mission', 'build_test',
            'create_glossary_review', 'add_to_weakness_map',
            'schedule_intervention', 'notify_parent', 'attach_to_campaign'
        )),
    action_payload_json TEXT,
    triggered_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- 7. Artifact lifecycle (idea 26 - content foundry)
-- ============================================================================

CREATE TABLE artifacts (
    id INTEGER PRIMARY KEY,
    artifact_type TEXT NOT NULL
        CHECK (artifact_type IN (
            'question', 'explanation', 'worked_example', 'glossary_entry',
            'formula', 'drill', 'audio_script', 'diagram', 'assessment'
        )),
    topic_id INTEGER,
    subject_id INTEGER,
    family_id INTEGER,
    current_version_id INTEGER,
    lifecycle_state TEXT NOT NULL DEFAULT 'draft'
        CHECK (lifecycle_state IN (
            'draft', 'in_review', 'approved', 'live', 'deprecated', 'archived'
        )),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_artifacts_state ON artifacts(lifecycle_state);

CREATE TABLE artifact_versions (
    id INTEGER PRIMARY KEY,
    artifact_id INTEGER NOT NULL REFERENCES artifacts(id),
    version_no INTEGER NOT NULL DEFAULT 1,
    state TEXT NOT NULL DEFAULT 'draft',
    content_json TEXT NOT NULL,
    build_reason TEXT,
    quality_score_bp INTEGER,
    provenance_ref TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_artifact_versions_artifact ON artifact_versions(artifact_id);

CREATE TABLE artifact_quality_reports (
    id INTEGER PRIMARY KEY,
    artifact_version_id INTEGER NOT NULL REFERENCES artifact_versions(id),
    structural_score_bp INTEGER NOT NULL DEFAULT 0,
    academic_score_bp INTEGER NOT NULL DEFAULT 0,
    relevance_score_bp INTEGER NOT NULL DEFAULT 0,
    clarity_score_bp INTEGER NOT NULL DEFAULT 0,
    overall_score_bp INTEGER NOT NULL DEFAULT 0,
    pass_fail TEXT NOT NULL DEFAULT 'pending',
    issues_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- 8. Topic health (content completeness tracking)
-- ============================================================================

CREATE TABLE topic_health (
    id INTEGER PRIMARY KEY,
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    package_completeness_bp INTEGER NOT NULL DEFAULT 0,
    quality_score_bp INTEGER NOT NULL DEFAULT 0,
    live_health_state TEXT NOT NULL DEFAULT 'incomplete',
    missing_components_json TEXT,
    last_refresh_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(topic_id)
);
