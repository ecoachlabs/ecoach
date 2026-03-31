-- idea17 deep gaps: formula_meta expansion, definition_meta expansion,
-- concept_meta expansion, entry search index.

-- ============================================================================
-- 1. Formula meta expansion
-- ============================================================================

ALTER TABLE formula_meta ADD COLUMN formula_latex TEXT;
ALTER TABLE formula_meta ADD COLUMN assumptions_json TEXT;
ALTER TABLE formula_meta ADD COLUMN common_errors_json TEXT;
ALTER TABLE formula_meta ADD COLUMN worked_example_ids_json TEXT;

-- ============================================================================
-- 2. Definition meta expansion
-- ============================================================================

ALTER TABLE definition_meta ADD COLUMN pronunciation_text TEXT;
ALTER TABLE definition_meta ADD COLUMN formal_definition TEXT;

-- ============================================================================
-- 3. Concept meta expansion
-- ============================================================================

ALTER TABLE concept_meta ADD COLUMN why_it_matters TEXT;
ALTER TABLE concept_meta ADD COLUMN mastery_indicators_json TEXT;

-- ============================================================================
-- 4. Entry search index (for efficient multi-field search)
-- ============================================================================

CREATE TABLE glossary_search_index (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    title_tokens TEXT,
    alias_tokens TEXT,
    full_text_content TEXT,
    simple_text_content TEXT,
    formula_speech_text TEXT,
    misconception_text TEXT,
    topic_labels TEXT,
    bundle_labels TEXT,
    intent_keywords TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entry_id)
);

CREATE INDEX idx_glossary_search_entry ON glossary_search_index(entry_id);
