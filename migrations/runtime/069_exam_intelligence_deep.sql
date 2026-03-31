-- idea13 deep gaps: paper sections, misconception master table,
-- missing edge types, matching levels, micro-skills.

-- ============================================================================
-- 1. Paper sections (structural parts of a paper)
-- ============================================================================

CREATE TABLE paper_sections (
    id INTEGER PRIMARY KEY,
    paper_set_id INTEGER NOT NULL REFERENCES past_paper_sets(id),
    section_label TEXT NOT NULL,
    section_type TEXT NOT NULL DEFAULT 'objective'
        CHECK (section_type IN ('objective', 'theory', 'structured', 'practical', 'mixed')),
    section_order INTEGER NOT NULL DEFAULT 1,
    instructions_raw TEXT,
    marks_allocated INTEGER,
    question_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_paper_sections_paper ON paper_sections(paper_set_id);

-- ============================================================================
-- 2. Misconception master records
-- ============================================================================

CREATE TABLE misconception_records (
    id INTEGER PRIMARY KEY,
    misconception_code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL DEFAULT 'conceptual'
        CHECK (category IN (
            'conceptual', 'procedural', 'linguistic', 'visual',
            'overgeneralization', 'surface_similarity', 'false_analogy',
            'incomplete_rule', 'step_confusion'
        )),
    symptoms_json TEXT,
    detection_rules_json TEXT,
    common_distractors_json TEXT,
    repair_strategies_json TEXT,
    subject_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_misconception_records_category ON misconception_records(category);

-- ============================================================================
-- 3. Question-to-misconception links
-- ============================================================================

CREATE TABLE question_misconception_links (
    id INTEGER PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id),
    misconception_id INTEGER NOT NULL REFERENCES misconception_records(id),
    exposure_score_bp INTEGER NOT NULL DEFAULT 5000,
    distractor_option_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(question_id, misconception_id)
);

CREATE INDEX idx_question_misconception_q ON question_misconception_links(question_id);
CREATE INDEX idx_question_misconception_m ON question_misconception_links(misconception_id);

-- ============================================================================
-- 4. Micro-skills (below subtopic level)
-- ============================================================================

CREATE TABLE micro_skills (
    id INTEGER PRIMARY KEY,
    subtopic_id INTEGER NOT NULL,
    skill_name TEXT NOT NULL,
    description TEXT,
    cognitive_skill_id INTEGER REFERENCES cognitive_skills(id),
    difficulty_bp INTEGER NOT NULL DEFAULT 5000,
    exam_frequency_bp INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_micro_skills_subtopic ON micro_skills(subtopic_id);

-- ============================================================================
-- 5. Question-to-section links
-- ============================================================================

ALTER TABLE past_paper_question_links ADD COLUMN section_id INTEGER REFERENCES paper_sections(id);

-- ============================================================================
-- 6. Expand family_relationship_edges with more edge types
-- ============================================================================

-- The CHECK constraint in 064 only allows 9 types. We need more.
-- SQLite doesn't support ALTER CHECK, so we add a view for validation.

-- Additional edge types tracked via a reference table:
CREATE TABLE relationship_edge_types (
    edge_type TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    description TEXT,
    is_directional INTEGER NOT NULL DEFAULT 1
);

INSERT INTO relationship_edge_types (edge_type, display_name, description, is_directional) VALUES
    ('co_appears_with', 'Co-Appears With', 'Families that frequently appear in same paper', 0),
    ('inverse_to', 'Inverse To', 'Families that suppress each other', 0),
    ('replaces', 'Replaces', 'New family taking over from old', 1),
    ('evolves_into', 'Evolves Into', 'Family mutating into new form', 1),
    ('shares_misconception', 'Shares Misconception', 'Families exposing same error pattern', 0),
    ('shares_cognitive_profile', 'Shares Cognitive Profile', 'Families testing same mental operations', 0),
    ('easier_variant', 'Easier Variant', 'Simpler version of same family', 1),
    ('harder_variant', 'Harder Variant', 'More complex version of same family', 1),
    ('precursor_to', 'Precursor To', 'Prerequisite family that must be mastered first', 1),
    ('exact_repeat_of', 'Exact Repeat Of', 'Essentially identical questions', 1),
    ('template_repeat_of', 'Template Repeat Of', 'Same structure with different surface details', 1),
    ('structurally_similar_to', 'Structurally Similar To', 'Same reasoning skeleton, different presentation', 0),
    ('belongs_to_same_family', 'Same Family', 'Members of same question family', 0);

-- ============================================================================
-- 7. Computation run tracking (audit trail for index computations)
-- ============================================================================

CREATE TABLE intelligence_computation_runs (
    id INTEGER PRIMARY KEY,
    computation_type TEXT NOT NULL
        CHECK (computation_type IN (
            'family_recurrence', 'co_appearance', 'inverse_appearance',
            'replacement_index', 'mutation_drift', 'paper_dna',
            'story_generation', 'student_overlay', 'full_pipeline'
        )),
    subject_id INTEGER,
    families_processed INTEGER NOT NULL DEFAULT 0,
    edges_created INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER,
    status TEXT NOT NULL DEFAULT 'completed',
    error_message TEXT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

-- ============================================================================
-- 8. Matching similarity scores (question pair similarity cache)
-- ============================================================================

CREATE TABLE question_similarity_scores (
    id INTEGER PRIMARY KEY,
    question_a_id INTEGER NOT NULL REFERENCES questions(id),
    question_b_id INTEGER NOT NULL REFERENCES questions(id),
    match_level INTEGER NOT NULL DEFAULT 0
        CHECK (match_level BETWEEN 0 AND 3),
    text_similarity_bp INTEGER NOT NULL DEFAULT 0,
    structure_similarity_bp INTEGER NOT NULL DEFAULT 0,
    cognitive_similarity_bp INTEGER NOT NULL DEFAULT 0,
    composite_similarity_bp INTEGER NOT NULL DEFAULT 0,
    algorithm_version TEXT NOT NULL DEFAULT 'v1',
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(question_a_id, question_b_id)
);

CREATE INDEX idx_question_similarity_a ON question_similarity_scores(question_a_id);
CREATE INDEX idx_question_similarity_b ON question_similarity_scores(question_b_id);
