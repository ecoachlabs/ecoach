-- idea17 runtime completion: flexible entry blocks, interaction events,
-- and persisted glossary test items.

-- ============================================================================
-- 1. Flexible content blocks
-- ============================================================================

CREATE TABLE IF NOT EXISTS entry_content_blocks (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    block_type TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    content_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_entry_content_blocks_entry
    ON entry_content_blocks(entry_id, order_index);

-- ============================================================================
-- 2. Event stream for glossary interactions and analytics
-- ============================================================================

CREATE TABLE IF NOT EXISTS glossary_interaction_events (
    id INTEGER PRIMARY KEY,
    student_id INTEGER REFERENCES accounts(id),
    entry_id INTEGER REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    bundle_id INTEGER REFERENCES knowledge_bundles(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    query_text TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_glossary_events_student
    ON glossary_interaction_events(student_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_glossary_events_entry
    ON glossary_interaction_events(entry_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_glossary_events_type
    ON glossary_interaction_events(event_type, created_at DESC);

-- ============================================================================
-- 3. Persisted test prompts/items per glossary test session
-- ============================================================================

CREATE TABLE IF NOT EXISTS glossary_test_items (
    id INTEGER PRIMARY KEY,
    test_session_id INTEGER NOT NULL REFERENCES glossary_test_sessions(id) ON DELETE CASCADE,
    sequence_no INTEGER NOT NULL,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    prompt_type TEXT NOT NULL,
    prompt_text TEXT NOT NULL,
    expected_answer TEXT,
    options_json TEXT NOT NULL DEFAULT '[]',
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_glossary_test_items_session
    ON glossary_test_items(test_session_id, sequence_no);

-- ============================================================================
-- 4. Entry-level scoring fields for richer ranking
-- ============================================================================

ALTER TABLE knowledge_entries ADD COLUMN exam_relevance_score INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE knowledge_entries ADD COLUMN priority_score INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE knowledge_entries ADD COLUMN phonetic_text TEXT;
