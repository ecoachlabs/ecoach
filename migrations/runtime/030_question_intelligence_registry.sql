CREATE TABLE IF NOT EXISTS question_intelligence_axes (
    axis_code TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    description TEXT
);

INSERT OR IGNORE INTO question_intelligence_axes (axis_code, display_name, description) VALUES
    ('knowledge_role', 'Knowledge Role', 'What kind of knowledge artifact the question is centered on.'),
    ('cognitive_demand', 'Cognitive Demand', 'What kind of thinking the question demands.'),
    ('solve_pattern', 'Solve Pattern', 'How the learner typically solves the question.'),
    ('pedagogic_function', 'Pedagogic Function', 'What learning job the question performs.'),
    ('content_grain', 'Content Grain', 'What curriculum grain the question is operating at.'),
    ('question_family', 'Question Family', 'What recurring family or pattern the question belongs to.'),
    ('misconception_exposure', 'Misconception Exposure', 'What misconception or error pattern the question is likely to expose.');

CREATE TABLE IF NOT EXISTS question_intelligence_taxonomy (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    axis_code TEXT NOT NULL REFERENCES question_intelligence_axes(axis_code),
    concept_code TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    parent_code TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(axis_code, concept_code)
);

CREATE TABLE IF NOT EXISTS question_intelligence_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    axis_code TEXT NOT NULL REFERENCES question_intelligence_axes(axis_code),
    concept_code TEXT NOT NULL,
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    is_primary INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL DEFAULT 'pack'
        CHECK (source IN ('pack', 'system', 'review')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(question_id, axis_code, concept_code)
);

CREATE INDEX IF NOT EXISTS idx_question_intelligence_taxonomy_axis
    ON question_intelligence_taxonomy(axis_code, concept_code);
CREATE INDEX IF NOT EXISTS idx_question_intelligence_links_question
    ON question_intelligence_links(question_id);
CREATE INDEX IF NOT EXISTS idx_question_intelligence_links_axis
    ON question_intelligence_links(axis_code, concept_code);
