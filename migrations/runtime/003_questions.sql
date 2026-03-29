CREATE TABLE IF NOT EXISTS question_families (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_code TEXT NOT NULL,
    family_name TEXT NOT NULL,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    subtopic_id INTEGER REFERENCES topics(id),
    family_type TEXT NOT NULL DEFAULT 'recurring_pattern'
        CHECK (family_type IN ('recurring_pattern', 'worked_example_template', 'misconception_cluster', 'exam_structure')),
    canonical_pattern TEXT,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS questions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    subtopic_id INTEGER REFERENCES topics(id),
    family_id INTEGER REFERENCES question_families(id),
    stem TEXT NOT NULL,
    question_format TEXT NOT NULL DEFAULT 'mcq'
        CHECK (question_format IN ('mcq', 'short_answer', 'numeric', 'true_false', 'matching', 'ordering')),
    explanation_text TEXT,
    difficulty_level INTEGER NOT NULL DEFAULT 5000,
    estimated_time_seconds INTEGER NOT NULL DEFAULT 30,
    marks INTEGER NOT NULL DEFAULT 1,
    source_type TEXT DEFAULT 'authored'
        CHECK (source_type IN ('past_question', 'authored', 'generated', 'teacher_upload')),
    source_ref TEXT,
    exam_year INTEGER,
    primary_knowledge_role TEXT,
    primary_cognitive_demand TEXT,
    primary_solve_pattern TEXT,
    primary_pedagogic_function TEXT,
    classification_confidence INTEGER DEFAULT 0,
    intelligence_snapshot TEXT NOT NULL DEFAULT '{}',
    primary_skill_id INTEGER REFERENCES academic_nodes(id),
    cognitive_level TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    pack_id TEXT REFERENCES content_packs(pack_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS question_options (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    option_label TEXT NOT NULL,
    option_text TEXT NOT NULL,
    is_correct INTEGER NOT NULL DEFAULT 0,
    misconception_id INTEGER REFERENCES misconception_patterns(id),
    distractor_intent TEXT,
    position INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS question_skill_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    contribution_weight INTEGER NOT NULL DEFAULT 10000,
    is_primary INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_questions_subject ON questions(subject_id);
CREATE INDEX IF NOT EXISTS idx_questions_topic ON questions(topic_id);
CREATE INDEX IF NOT EXISTS idx_questions_subtopic ON questions(subtopic_id);
CREATE INDEX IF NOT EXISTS idx_questions_family ON questions(family_id);
CREATE INDEX IF NOT EXISTS idx_questions_difficulty ON questions(difficulty_level);
CREATE INDEX IF NOT EXISTS idx_questions_active ON questions(is_active);
CREATE INDEX IF NOT EXISTS idx_question_options_question ON question_options(question_id);
CREATE INDEX IF NOT EXISTS idx_question_skill_links_question ON question_skill_links(question_id);
CREATE INDEX IF NOT EXISTS idx_question_skill_links_node ON question_skill_links(node_id);
