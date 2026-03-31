-- idea13: Past Paper Intelligence Engine — paper DNA, family relationship graph,
-- co-appearance/inverse/replacement edges, student-family performance overlays,
-- story summaries, deep question instance metadata, cognitive fingerprints.

-- ============================================================================
-- 1. Paper DNA profiles (per-paper intelligence summary)
-- ============================================================================

CREATE TABLE paper_dna (
    id INTEGER PRIMARY KEY,
    paper_set_id INTEGER NOT NULL REFERENCES past_paper_sets(id),
    topic_distribution_json TEXT NOT NULL DEFAULT '[]',
    cognitive_balance_json TEXT NOT NULL DEFAULT '{}',
    format_signature TEXT,
    difficulty_profile_json TEXT,
    trap_density_curve_json TEXT,
    recall_vs_reasoning_ratio INTEGER NOT NULL DEFAULT 5000,
    novelty_score INTEGER NOT NULL DEFAULT 5000,
    separator_questions_json TEXT,
    confidence_builder_questions_json TEXT,
    high_discriminator_questions_json TEXT,
    dominant_families_json TEXT NOT NULL DEFAULT '[]',
    absent_common_families_json TEXT,
    substitution_patterns_json TEXT,
    story_summary TEXT,
    pacing_curve_json TEXT,
    computed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX idx_paper_dna_paper ON paper_dna(paper_set_id);

-- ============================================================================
-- 2. Family relationship edges (co-appearance, inverse, replacement, evolution)
-- ============================================================================

CREATE TABLE family_relationship_edges (
    id INTEGER PRIMARY KEY,
    source_family_id INTEGER NOT NULL REFERENCES question_families(id),
    target_family_id INTEGER NOT NULL REFERENCES question_families(id),
    edge_type TEXT NOT NULL
        CHECK (edge_type IN (
            'co_appears_with', 'inverse_to', 'replaces',
            'evolves_into', 'shares_misconception', 'shares_cognitive_profile',
            'easier_variant', 'harder_variant', 'precursor_to'
        )),
    strength_score INTEGER NOT NULL DEFAULT 5000,
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    support_count INTEGER NOT NULL DEFAULT 0,
    time_window_start TEXT,
    time_window_end TEXT,
    evidence_json TEXT,
    algorithm_version TEXT NOT NULL DEFAULT 'v1',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_family_id, target_family_id, edge_type)
);

CREATE INDEX idx_family_edges_source ON family_relationship_edges(source_family_id, edge_type);
CREATE INDEX idx_family_edges_target ON family_relationship_edges(target_family_id, edge_type);

-- ============================================================================
-- 3. Family recurrence metrics (persistent computed scores)
-- ============================================================================

CREATE TABLE family_recurrence_metrics (
    id INTEGER PRIMARY KEY,
    family_id INTEGER NOT NULL REFERENCES question_families(id),
    subject_id INTEGER NOT NULL,
    total_papers_in_window INTEGER NOT NULL DEFAULT 0,
    papers_appeared INTEGER NOT NULL DEFAULT 0,
    recurrence_rate_bp INTEGER NOT NULL DEFAULT 0,
    family_density_bp INTEGER NOT NULL DEFAULT 0,
    persistence_score_bp INTEGER NOT NULL DEFAULT 0,
    dormancy_max_years INTEGER NOT NULL DEFAULT 0,
    last_appearance_year INTEGER,
    first_appearance_year INTEGER,
    peak_years_json TEXT,
    mutation_trend TEXT,
    current_relevance_bp INTEGER NOT NULL DEFAULT 5000,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(family_id, subject_id)
);

CREATE INDEX idx_family_recurrence_subject ON family_recurrence_metrics(subject_id, recurrence_rate_bp DESC);

-- ============================================================================
-- 4. Deep question instance metadata (per past-paper question)
-- ============================================================================

ALTER TABLE past_paper_question_links ADD COLUMN position_index INTEGER;
ALTER TABLE past_paper_question_links ADD COLUMN mark_value INTEGER;
ALTER TABLE past_paper_question_links ADD COLUMN estimated_time_seconds INTEGER;
ALTER TABLE past_paper_question_links ADD COLUMN trap_density_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE past_paper_question_links ADD COLUMN story_role TEXT;
ALTER TABLE past_paper_question_links ADD COLUMN mutation_class TEXT;
ALTER TABLE past_paper_question_links ADD COLUMN surface_difficulty_score INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE past_paper_question_links ADD COLUMN canonical_pattern_id INTEGER;
ALTER TABLE past_paper_question_links ADD COLUMN cognitive_fingerprint_json TEXT;
ALTER TABLE past_paper_question_links ADD COLUMN examiner_intent TEXT;
ALTER TABLE past_paper_question_links ADD COLUMN misconception_exposure_json TEXT;

-- ============================================================================
-- 5. Canonical patterns (normalized question structures)
-- ============================================================================

CREATE TABLE canonical_patterns (
    id INTEGER PRIMARY KEY,
    pattern_name TEXT NOT NULL,
    pattern_signature TEXT NOT NULL UNIQUE,
    normalized_template TEXT,
    solution_template TEXT,
    answer_schema TEXT,
    context_signature TEXT,
    complexity_score INTEGER NOT NULL DEFAULT 5000,
    parent_family_id INTEGER REFERENCES question_families(id),
    instance_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_canonical_patterns_family ON canonical_patterns(parent_family_id);

-- ============================================================================
-- 6. Student-family performance overlay
-- ============================================================================

CREATE TABLE student_family_performance (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    family_id INTEGER NOT NULL REFERENCES question_families(id),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    accuracy_rate_bp INTEGER NOT NULL DEFAULT 0,
    confidence_calibration_bp INTEGER NOT NULL DEFAULT 5000,
    average_latency_ms INTEGER,
    first_step_quality_bp INTEGER,
    classical_form_accuracy_bp INTEGER,
    mutated_form_accuracy_bp INTEGER,
    trap_fall_rate_bp INTEGER NOT NULL DEFAULT 0,
    misconception_distribution_json TEXT,
    recovery_progress_bp INTEGER NOT NULL DEFAULT 0,
    last_attempted_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, family_id)
);

CREATE INDEX idx_student_family_perf_student ON student_family_performance(student_id);

-- ============================================================================
-- 7. Family story summaries
-- ============================================================================

CREATE TABLE family_stories (
    id INTEGER PRIMARY KEY,
    family_id INTEGER NOT NULL REFERENCES question_families(id),
    story_type TEXT NOT NULL DEFAULT 'family'
        CHECK (story_type IN ('family', 'paper', 'exam_trend', 'question')),
    headline TEXT NOT NULL,
    narrative TEXT NOT NULL,
    evidence_json TEXT,
    insights_json TEXT,
    recommendation TEXT,
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(family_id, story_type)
);

CREATE INDEX idx_family_stories_type ON family_stories(story_type);

-- ============================================================================
-- 8. Inverse appearance index pairs (precomputed)
-- ============================================================================

CREATE TABLE inverse_appearance_pairs (
    id INTEGER PRIMARY KEY,
    family_a_id INTEGER NOT NULL REFERENCES question_families(id),
    family_b_id INTEGER NOT NULL REFERENCES question_families(id),
    iai_score_bp INTEGER NOT NULL DEFAULT 0,
    directional_a_suppresses_b_bp INTEGER NOT NULL DEFAULT 0,
    directional_b_suppresses_a_bp INTEGER NOT NULL DEFAULT 0,
    support_papers INTEGER NOT NULL DEFAULT 0,
    strongest_years_json TEXT,
    likely_explanation TEXT,
    is_mutual INTEGER NOT NULL DEFAULT 0,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(family_a_id, family_b_id)
);

CREATE INDEX idx_inverse_pairs_a ON inverse_appearance_pairs(family_a_id);
CREATE INDEX idx_inverse_pairs_b ON inverse_appearance_pairs(family_b_id);

-- ============================================================================
-- 9. Replacement trails
-- ============================================================================

CREATE TABLE family_replacement_trails (
    id INTEGER PRIMARY KEY,
    old_family_id INTEGER NOT NULL REFERENCES question_families(id),
    new_family_id INTEGER NOT NULL REFERENCES question_families(id),
    replacement_index_bp INTEGER NOT NULL DEFAULT 0,
    iai_component_bp INTEGER NOT NULL DEFAULT 0,
    chrono_shift_bp INTEGER NOT NULL DEFAULT 0,
    topic_overlap_bp INTEGER NOT NULL DEFAULT 0,
    cognitive_overlap_bp INTEGER NOT NULL DEFAULT 0,
    role_overlap_bp INTEGER NOT NULL DEFAULT 0,
    decline_start_year INTEGER,
    rise_start_year INTEGER,
    evidence_json TEXT,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(old_family_id, new_family_id)
);

CREATE INDEX idx_replacement_trails_old ON family_replacement_trails(old_family_id);

-- ============================================================================
-- 10. Extend question_family_analytics with deeper metrics
-- ============================================================================

ALTER TABLE question_family_analytics ADD COLUMN persistence_score_bp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE question_family_analytics ADD COLUMN dormancy_years INTEGER NOT NULL DEFAULT 0;
ALTER TABLE question_family_analytics ADD COLUMN mutation_drift_score_bp INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE question_family_analytics ADD COLUMN cognitive_fingerprint_json TEXT;
ALTER TABLE question_family_analytics ADD COLUMN top_misconceptions_json TEXT;
ALTER TABLE question_family_analytics ADD COLUMN story_headline TEXT;

-- ============================================================================
-- 11. Past paper sets expansion
-- ============================================================================

ALTER TABLE past_paper_sets ADD COLUMN duration_minutes INTEGER;
ALTER TABLE past_paper_sets ADD COLUMN total_marks INTEGER;
ALTER TABLE past_paper_sets ADD COLUMN total_questions INTEGER;
ALTER TABLE past_paper_sets ADD COLUMN paper_type TEXT;
ALTER TABLE past_paper_sets ADD COLUMN region_or_board TEXT;
ALTER TABLE past_paper_sets ADD COLUMN difficulty_estimate INTEGER;
ALTER TABLE past_paper_sets ADD COLUMN ingestion_status TEXT NOT NULL DEFAULT 'complete';

-- ============================================================================
-- 12. Cognitive skills reference table
-- ============================================================================

CREATE TABLE cognitive_skills (
    id INTEGER PRIMARY KEY,
    skill_name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    description TEXT,
    measurement_rules_json TEXT
);

INSERT INTO cognitive_skills (skill_name, category, description) VALUES
    ('recall', 'basic', 'Direct retrieval of memorized facts or procedures'),
    ('recognition', 'basic', 'Identifying familiar concepts in context'),
    ('interpretation', 'intermediate', 'Extracting meaning from data, text, or diagrams'),
    ('comparison', 'intermediate', 'Analyzing similarities and differences'),
    ('classification', 'intermediate', 'Categorizing items based on properties'),
    ('estimation', 'intermediate', 'Approximating quantities or outcomes'),
    ('computation', 'basic', 'Performing mathematical calculations'),
    ('symbolic_translation', 'advanced', 'Converting between verbal and symbolic forms'),
    ('method_selection', 'advanced', 'Choosing the correct approach to solve'),
    ('multi_step_reasoning', 'advanced', 'Chaining multiple operations logically'),
    ('elimination', 'intermediate', 'Ruling out incorrect options systematically'),
    ('justification', 'advanced', 'Providing evidence-based explanations'),
    ('inference', 'advanced', 'Drawing conclusions from incomplete information'),
    ('error_detection', 'advanced', 'Identifying mistakes in given work'),
    ('application', 'advanced', 'Using knowledge in novel real-world contexts');

-- ============================================================================
-- 13. Examiner intents reference table
-- ============================================================================

CREATE TABLE examiner_intents (
    id INTEGER PRIMARY KEY,
    intent_name TEXT NOT NULL UNIQUE,
    description TEXT,
    indicators_json TEXT
);

INSERT INTO examiner_intents (intent_name, description) VALUES
    ('foundation_check', 'Verify basic knowledge is intact'),
    ('confidence_builder', 'Easy question to build momentum'),
    ('speed_filter', 'Tests ability to perform quickly'),
    ('trap_placement', 'Deliberately deceptive question'),
    ('separator', 'Distinguishes average from strong students'),
    ('depth_check', 'Tests deeper understanding beyond surface'),
    ('transfer_check', 'Tests ability to apply knowledge in new context'),
    ('integration', 'Requires combining multiple concepts'),
    ('precision_check', 'Tests careful attention to detail'),
    ('careless_error_detector', 'Designed to catch rushing or sloppy work');
