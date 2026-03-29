CREATE TABLE IF NOT EXISTS knowledge_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    subtopic_id INTEGER REFERENCES topics(id),
    entry_type TEXT NOT NULL,
    title TEXT NOT NULL,
    canonical_name TEXT,
    slug TEXT,
    short_text TEXT,
    full_text TEXT,
    simple_text TEXT,
    technical_text TEXT,
    exam_text TEXT,
    importance_score INTEGER NOT NULL DEFAULT 5000,
    difficulty_level INTEGER NOT NULL DEFAULT 5000,
    grade_band TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS entry_aliases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    alias_text TEXT NOT NULL,
    alias_type TEXT DEFAULT 'synonym'
);

CREATE TABLE IF NOT EXISTS knowledge_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    to_entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL,
    strength_score INTEGER NOT NULL DEFAULT 5000,
    explanation TEXT
);

CREATE TABLE IF NOT EXISTS knowledge_bundles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    bundle_type TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    description TEXT,
    difficulty_level INTEGER NOT NULL DEFAULT 5000,
    estimated_duration INTEGER DEFAULT 0,
    exam_relevance_score INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS knowledge_bundle_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bundle_id INTEGER NOT NULL REFERENCES knowledge_bundles(id) ON DELETE CASCADE,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    item_role TEXT NOT NULL,
    sequence_order INTEGER NOT NULL DEFAULT 0,
    required INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS student_entry_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES accounts(id),
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    familiarity_state TEXT NOT NULL DEFAULT 'unseen',
    mastery_score INTEGER NOT NULL DEFAULT 0,
    confusion_score INTEGER NOT NULL DEFAULT 0,
    recall_strength INTEGER NOT NULL DEFAULT 0,
    last_viewed_at TEXT,
    last_played_at TEXT,
    last_tested_at TEXT,
    review_due_at TEXT,
    open_count INTEGER NOT NULL DEFAULT 0,
    linked_wrong_answer_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(user_id, entry_id)
);
