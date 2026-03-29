CREATE TABLE IF NOT EXISTS curriculum_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    country TEXT NOT NULL DEFAULT 'GH',
    exam_board TEXT,
    education_stage TEXT,
    version_label TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'review', 'published', 'archived')),
    effective_from TEXT,
    published_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS subjects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id),
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS topics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    parent_topic_id INTEGER REFERENCES topics(id),
    code TEXT,
    name TEXT NOT NULL,
    description TEXT,
    node_type TEXT NOT NULL DEFAULT 'topic'
        CHECK (node_type IN ('strand', 'sub_strand', 'topic', 'subtopic')),
    display_order INTEGER NOT NULL DEFAULT 0,
    exam_weight INTEGER NOT NULL DEFAULT 5000,
    difficulty_band TEXT DEFAULT 'medium'
        CHECK (difficulty_band IN ('easy', 'medium', 'hard', 'advanced')),
    importance_weight INTEGER NOT NULL DEFAULT 5000,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS academic_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    node_type TEXT NOT NULL
        CHECK (node_type IN (
            'definition', 'concept', 'formula', 'procedure', 'comparison',
            'principle', 'rule', 'theorem', 'worked_pattern', 'application',
            'interpretation', 'diagram_spatial', 'proof_justification',
            'essay_structured', 'word_problem_translation', 'vocabulary',
            'symbol_notation'
        )),
    canonical_title TEXT NOT NULL,
    short_label TEXT,
    description_formal TEXT,
    description_simple TEXT,
    core_meaning TEXT,
    difficulty_band TEXT DEFAULT 'medium',
    exam_relevance_score INTEGER NOT NULL DEFAULT 5000,
    foundation_weight INTEGER NOT NULL DEFAULT 5000,
    is_active INTEGER NOT NULL DEFAULT 1,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS node_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_node_id INTEGER NOT NULL,
    from_node_type TEXT NOT NULL CHECK (from_node_type IN ('topic', 'academic_node')),
    to_node_id INTEGER NOT NULL,
    to_node_type TEXT NOT NULL CHECK (to_node_type IN ('topic', 'academic_node')),
    edge_type TEXT NOT NULL
        CHECK (edge_type IN (
            'prerequisite', 'soft_prerequisite', 'related', 'confused_with',
            'uses_formula', 'uses_procedure', 'has_example', 'has_non_example',
            'contrasts_with', 'is_applied_in', 'targets_misconception',
            'representation_of', 'dependent', 'part_of'
        )),
    strength_score INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS misconception_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id INTEGER REFERENCES academic_nodes(id),
    topic_id INTEGER REFERENCES topics(id),
    title TEXT NOT NULL,
    misconception_statement TEXT NOT NULL,
    cause_type TEXT
        CHECK (cause_type IN (
            'overgeneralization', 'memorization_without_understanding',
            'visual_confusion', 'language_confusion', 'step_confusion',
            'false_analogy', 'surface_similarity', 'incomplete_rule'
        )),
    wrong_answer_pattern TEXT,
    correction_hint TEXT,
    severity INTEGER NOT NULL DEFAULT 5000,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS learning_objectives (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    objective_text TEXT NOT NULL,
    simplified_text TEXT,
    cognitive_level TEXT
        CHECK (cognitive_level IN ('knowledge', 'understanding', 'application', 'reasoning', 'evaluation')),
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_topics_subject ON topics(subject_id);
CREATE INDEX IF NOT EXISTS idx_topics_parent ON topics(parent_topic_id);
CREATE INDEX IF NOT EXISTS idx_academic_nodes_topic ON academic_nodes(topic_id);
CREATE INDEX IF NOT EXISTS idx_academic_nodes_type ON academic_nodes(node_type);
CREATE INDEX IF NOT EXISTS idx_node_edges_from ON node_edges(from_node_id, from_node_type);
CREATE INDEX IF NOT EXISTS idx_node_edges_to ON node_edges(to_node_id, to_node_type);
CREATE INDEX IF NOT EXISTS idx_node_edges_type ON node_edges(edge_type);
CREATE INDEX IF NOT EXISTS idx_misconceptions_node ON misconception_patterns(node_id);
CREATE INDEX IF NOT EXISTS idx_misconceptions_topic ON misconception_patterns(topic_id);
