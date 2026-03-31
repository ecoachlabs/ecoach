-- idea9: Question Factory deep model — concept primitives, representations,
-- examples, solution graphs, cognitive type taxonomy, question templates.

-- ============================================================================
-- 1. Concept primitives (decomposed knowledge atoms)
-- ============================================================================

CREATE TABLE concept_primitives (
    id INTEGER PRIMARY KEY,
    concept_node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    primitive_type TEXT NOT NULL
        CHECK (primitive_type IN (
            'definition', 'fact', 'rule', 'formula', 'law', 'condition',
            'cause', 'effect', 'step', 'clue', 'explanation_anchor',
            'boundary_case', 'memory_hook', 'application_hint'
        )),
    label TEXT,
    content TEXT NOT NULL,
    importance_weight REAL NOT NULL DEFAULT 1.0,
    difficulty_weight REAL NOT NULL DEFAULT 0.5,
    sequence_order INTEGER,
    metadata_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_concept_primitives_node ON concept_primitives(concept_node_id);
CREATE INDEX idx_concept_primitives_topic ON concept_primitives(topic_id);
CREATE INDEX idx_concept_primitives_type ON concept_primitives(primitive_type);

-- ============================================================================
-- 2. Representation records (different forms of same concept)
-- ============================================================================

CREATE TABLE representation_records (
    id INTEGER PRIMARY KEY,
    concept_node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    representation_type TEXT NOT NULL
        CHECK (representation_type IN (
            'text', 'equation', 'graph', 'table', 'diagram',
            'scenario', 'worked_example', 'audio_prompt'
        )),
    title TEXT,
    content TEXT NOT NULL,
    difficulty REAL NOT NULL DEFAULT 0.5,
    metadata_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_representation_records_node ON representation_records(concept_node_id);

-- ============================================================================
-- 3. Example records (examples and non-examples per concept)
-- ============================================================================

CREATE TABLE example_records (
    id INTEGER PRIMARY KEY,
    concept_node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    example_type TEXT NOT NULL
        CHECK (example_type IN (
            'canonical', 'tricky', 'disguised', 'real_world',
            'borderline', 'non_example'
        )),
    title TEXT,
    description TEXT NOT NULL,
    explanation TEXT,
    validity_conditions TEXT,
    similarity_score REAL NOT NULL DEFAULT 0.5,
    metadata_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_example_records_node ON example_records(concept_node_id);

-- ============================================================================
-- 4. Solution graphs (step-by-step process structures)
-- ============================================================================

CREATE TABLE solution_graphs (
    id INTEGER PRIMARY KEY,
    concept_node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    question_family_id INTEGER REFERENCES question_families(id),
    title TEXT NOT NULL,
    start_state_json TEXT NOT NULL DEFAULT '{}',
    nodes_json TEXT NOT NULL DEFAULT '[]',
    transitions_json TEXT NOT NULL DEFAULT '[]',
    invalid_transitions_json TEXT,
    terminal_states_json TEXT NOT NULL DEFAULT '[]',
    hint_anchors_json TEXT,
    explanation_anchors_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_solution_graphs_node ON solution_graphs(concept_node_id);
CREATE INDEX idx_solution_graphs_family ON solution_graphs(question_family_id);

-- ============================================================================
-- 5. Extend question_families with cognitive type and transformation rules
-- ============================================================================

ALTER TABLE question_families ADD COLUMN cognitive_type TEXT;
ALTER TABLE question_families ADD COLUMN evaluative_intent TEXT;
ALTER TABLE question_families ADD COLUMN transformation_rules_json TEXT;
ALTER TABLE question_families ADD COLUMN required_primitives_json TEXT;
ALTER TABLE question_families ADD COLUMN allowed_formats_json TEXT;
ALTER TABLE question_families ADD COLUMN scoring_mode TEXT;
ALTER TABLE question_families ADD COLUMN validation_rules_json TEXT;

-- ============================================================================
-- 6. Question templates (template-based generation)
-- ============================================================================

CREATE TABLE question_templates (
    id INTEGER PRIMARY KEY,
    question_family_id INTEGER NOT NULL REFERENCES question_families(id),
    format_type TEXT NOT NULL,
    template_text TEXT NOT NULL,
    slot_definitions_json TEXT NOT NULL DEFAULT '[]',
    slot_constraints_json TEXT,
    language_level TEXT,
    tone TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_question_templates_family ON question_templates(question_family_id);

-- ============================================================================
-- 7. Extend questions with cognitive type taxonomy
-- ============================================================================

ALTER TABLE questions ADD COLUMN cognitive_type TEXT;
ALTER TABLE questions ADD COLUMN evaluative_intent TEXT;
ALTER TABLE questions ADD COLUMN variant_signature TEXT;
ALTER TABLE questions ADD COLUMN mode_context TEXT;
ALTER TABLE questions ADD COLUMN generation_template_id INTEGER;

-- ============================================================================
-- 8. Extended student concept dimensions (for adaptive generation)
-- ============================================================================

ALTER TABLE student_skill_states ADD COLUMN recognition_strength INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN reconstruction_strength INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN reasoning_strength INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN pressure_tolerance INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE student_skill_states ADD COLUMN retention_decay_rate INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN confidence_calibration INTEGER NOT NULL DEFAULT 5000;
