CREATE TABLE IF NOT EXISTS question_glossary_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL DEFAULT 'repair_support'
        CHECK (relation_type IN (
            'repair_support',
            'definition_support',
            'worked_example_support',
            'formula_support',
            'revision_anchor'
        )),
    link_source TEXT NOT NULL DEFAULT 'pack_inference'
        CHECK (link_source IN ('pack_inference', 'manual', 'diagnosis', 'planner')),
    link_reason TEXT,
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    is_primary INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(question_id, entry_id)
);

CREATE INDEX IF NOT EXISTS idx_question_glossary_links_question
    ON question_glossary_links(question_id);

CREATE INDEX IF NOT EXISTS idx_question_glossary_links_entry
    ON question_glossary_links(entry_id);
