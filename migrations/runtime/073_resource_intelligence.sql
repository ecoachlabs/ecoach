-- ideas 19, 21, 22, 27, 28: Resource intelligence, concept atoms,
-- content type taxonomy, question intelligence, evidence blocks.

-- ============================================================================
-- 1. Concept atoms (reusable content units)
-- ============================================================================

CREATE TABLE concept_atoms (
    id INTEGER PRIMARY KEY,
    node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    atom_type TEXT NOT NULL
        CHECK (atom_type IN (
            'definition', 'explanation', 'example', 'counterexample',
            'formula', 'derivation', 'application', 'misconception',
            'diagram_label', 'clue', 'hint', 'marking_point',
            'vocabulary', 'objective', 'rule', 'theorem'
        )),
    content_text TEXT NOT NULL,
    representation_type TEXT NOT NULL DEFAULT 'text'
        CHECK (representation_type IN (
            'text', 'audio', 'image', 'drill', 'quiz',
            'flashcard', 'teach_mode', 'diagnostic_mode'
        )),
    mastery_level INTEGER NOT NULL DEFAULT 5000,
    exam_relevance INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_concept_atoms_node ON concept_atoms(node_id, atom_type);

-- ============================================================================
-- 2. Concept coverage scores (readiness per concept)
-- ============================================================================

CREATE TABLE concept_coverage_scores (
    id INTEGER PRIMARY KEY,
    node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    knowledge_coverage_bp INTEGER NOT NULL DEFAULT 0,
    question_coverage_bp INTEGER NOT NULL DEFAULT 0,
    teaching_coverage_bp INTEGER NOT NULL DEFAULT 0,
    assessment_coverage_bp INTEGER NOT NULL DEFAULT 0,
    validation_coverage_bp INTEGER NOT NULL DEFAULT 0,
    confidence_coverage_bp INTEGER NOT NULL DEFAULT 0,
    overall_coverage_bp INTEGER NOT NULL DEFAULT 0,
    coverage_color TEXT NOT NULL DEFAULT 'red'
        CHECK (coverage_color IN ('red', 'amber', 'green', 'blue')),
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(node_id)
);

-- ============================================================================
-- 3. Content type registry
-- ============================================================================

CREATE TABLE content_type_registry (
    id INTEGER PRIMARY KEY,
    content_type_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    difficulty_band TEXT,
    exam_relevance_bp INTEGER NOT NULL DEFAULT 5000,
    foundation_weight INTEGER NOT NULL DEFAULT 5000
);

INSERT INTO content_type_registry (content_type_code, display_name) VALUES
    ('definition', 'Definition'),
    ('formula', 'Formula'),
    ('concept', 'Concept'),
    ('process', 'Process/Procedure'),
    ('sequence', 'Sequence/Steps'),
    ('comparison', 'Comparison'),
    ('essay_structure', 'Essay/Structured Response'),
    ('proof', 'Proof/Derivation'),
    ('diagram_spatial', 'Diagram/Spatial'),
    ('vocabulary', 'Vocabulary'),
    ('rule', 'Rule/Law'),
    ('theorem', 'Theorem'),
    ('application', 'Application/Real-World'),
    ('interpretation', 'Interpretation/Analysis'),
    ('worked_example', 'Worked Example');

-- ============================================================================
-- 4. Strategy family registry
-- ============================================================================

CREATE TABLE strategy_family_registry (
    id INTEGER PRIMARY KEY,
    strategy_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    pedagogy_basis TEXT,
    implementation_hints_json TEXT
);

INSERT INTO strategy_family_registry (strategy_code, display_name) VALUES
    ('direct_instruction', 'Direct Instruction'),
    ('scaffolded_practice', 'Scaffolded Practice'),
    ('contrast_drill', 'Contrast Drill'),
    ('spaced_retrieval', 'Spaced Retrieval'),
    ('interleaved_practice', 'Interleaved Practice'),
    ('worked_example_fading', 'Worked Example Fading'),
    ('error_analysis', 'Error Analysis'),
    ('self_explanation', 'Self-Explanation'),
    ('elaborative_interrogation', 'Elaborative Interrogation'),
    ('pressure_conditioning', 'Pressure Conditioning');

-- ============================================================================
-- 5. Content type teaching strategies (maps types → strategies)
-- ============================================================================

CREATE TABLE content_type_strategies (
    id INTEGER PRIMARY KEY,
    content_type_id INTEGER NOT NULL REFERENCES content_type_registry(id),
    strategy_id INTEGER NOT NULL REFERENCES strategy_family_registry(id),
    sequence_order INTEGER NOT NULL DEFAULT 1,
    effectiveness_bp INTEGER NOT NULL DEFAULT 5000,
    UNIQUE(content_type_id, strategy_id)
);

-- ============================================================================
-- 6. Resource metadata index (unified metadata for all resources)
-- ============================================================================

CREATE TABLE resource_metadata_index (
    id INTEGER PRIMARY KEY,
    resource_id INTEGER NOT NULL,
    resource_type TEXT NOT NULL
        CHECK (resource_type IN (
            'question', 'worked_example', 'glossary_entry', 'teach_explanation',
            'drill', 'audio_segment', 'note', 'intervention', 'flashcard'
        )),
    subject_id INTEGER,
    topic_id INTEGER,
    subtopic_id INTEGER,
    concept_id INTEGER,
    content_type_id INTEGER REFERENCES content_type_registry(id),
    cognitive_skill_id INTEGER REFERENCES cognitive_skills(id),
    difficulty_bp INTEGER NOT NULL DEFAULT 5000,
    exam_relevance_bp INTEGER NOT NULL DEFAULT 5000,
    teach_suitability_bp INTEGER NOT NULL DEFAULT 5000,
    test_suitability_bp INTEGER NOT NULL DEFAULT 5000,
    pressure_suitability_bp INTEGER NOT NULL DEFAULT 5000,
    source TEXT,
    confidence_tier TEXT DEFAULT 'ai_generated'
        CHECK (confidence_tier IN (
            'teacher_authored', 'syllabus_aligned', 'performance_tested',
            'ai_generated', 'partially_matched', 'inferred'
        )),
    teacher_verified INTEGER NOT NULL DEFAULT 0,
    student_success_rate_bp INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(resource_id, resource_type)
);

CREATE INDEX idx_resource_meta_topic ON resource_metadata_index(topic_id, resource_type);

-- ============================================================================
-- 7. Resource roles registry
-- ============================================================================

CREATE TABLE resource_roles_registry (
    id INTEGER PRIMARY KEY,
    role_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    when_to_use TEXT,
    success_criteria TEXT
);

INSERT INTO resource_roles_registry (role_code, display_name) VALUES
    ('concept_explainer', 'Concept Explainer'),
    ('misconception_corrector', 'Misconception Corrector'),
    ('quick_refresher', 'Quick Refresher'),
    ('worked_example', 'Worked Example'),
    ('pressure_drill', 'Pressure Drill'),
    ('accuracy_drill', 'Accuracy Drill'),
    ('vocabulary_repair', 'Vocabulary Repair'),
    ('comparison_builder', 'Comparison Builder'),
    ('formula_anchor', 'Formula Anchor'),
    ('confidence_rebuilder', 'Confidence Rebuilder'),
    ('stretch_challenge', 'Stretch Challenge'),
    ('mastery_validator', 'Mastery Validator');

-- ============================================================================
-- 8. Evidence blocks (normalized evidence from sources)
-- ============================================================================

CREATE TABLE evidence_blocks (
    id INTEGER PRIMARY KEY,
    source_type TEXT NOT NULL,
    source_id INTEGER,
    topic_id INTEGER,
    concept_id INTEGER,
    evidence_type TEXT NOT NULL DEFAULT 'claim',
    claim_text TEXT NOT NULL,
    supporting_text TEXT,
    extraction_confidence_bp INTEGER NOT NULL DEFAULT 5000,
    corroboration_score_bp INTEGER NOT NULL DEFAULT 0,
    pedagogy_score_bp INTEGER NOT NULL DEFAULT 5000,
    freshness_score_bp INTEGER NOT NULL DEFAULT 5000,
    final_quality_bp INTEGER NOT NULL DEFAULT 5000,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'verified', 'rejected', 'stale')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_evidence_blocks_topic ON evidence_blocks(topic_id);

-- ============================================================================
-- 9. Gap tickets (content gap detection)
-- ============================================================================

CREATE TABLE content_gap_tickets (
    id INTEGER PRIMARY KEY,
    topic_id INTEGER NOT NULL,
    node_id INTEGER,
    trigger_type TEXT NOT NULL
        CHECK (trigger_type IN (
            'student_request', 'coverage_scan', 'diagnostic_gap',
            'misconception_unaddressed', 'question_shortage', 'quality_issue'
        )),
    trigger_context_json TEXT,
    severity TEXT NOT NULL DEFAULT 'medium',
    required_asset_types_json TEXT,
    status TEXT NOT NULL DEFAULT 'open'
        CHECK (status IN ('open', 'in_progress', 'resolved', 'deferred')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX idx_gap_tickets_status ON content_gap_tickets(status);

-- ============================================================================
-- 10. Extend academic_nodes with content type
-- ============================================================================

ALTER TABLE academic_nodes ADD COLUMN primary_content_type TEXT;
ALTER TABLE academic_nodes ADD COLUMN representation_mode TEXT;
ALTER TABLE academic_nodes ADD COLUMN cognitive_action TEXT;
ALTER TABLE academic_nodes ADD COLUMN preferred_strategies_json TEXT;

-- ============================================================================
-- 11. Extend questions with classification metadata
-- ============================================================================

ALTER TABLE questions ADD COLUMN classification_source TEXT;
ALTER TABLE questions ADD COLUMN classification_confidence_bp INTEGER;
ALTER TABLE questions ADD COLUMN human_verified INTEGER NOT NULL DEFAULT 0;
ALTER TABLE questions ADD COLUMN review_status TEXT;
ALTER TABLE questions ADD COLUMN primary_content_type TEXT;
