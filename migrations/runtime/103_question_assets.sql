-- Migration 103: question_assets
--
-- Image attachments for past-paper questions. Scope tells us WHERE on
-- the question the image belongs:
--   'stem'        — inline with the question text
--   'option'      — attached to a specific option (scope_ref = option id)
--   'explanation' — inline with the model answer / marking guide
--
-- Bytes are stored directly in SQLite BLOB. Past-paper diagrams are
-- typically <500 KB each; for a 20-year × 5-subject bank that's ~100 MB
-- of images — well inside SQLite's comfortable range, and lets the
-- single ecoach.db file remain the full portable unit of content.

CREATE TABLE IF NOT EXISTS question_assets (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id   INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    scope         TEXT NOT NULL
        CHECK (scope IN ('stem', 'option', 'explanation')),
    scope_ref     INTEGER,
    mime_type     TEXT NOT NULL,
    byte_size     INTEGER NOT NULL,
    data          BLOB NOT NULL,
    position      INTEGER NOT NULL DEFAULT 0,
    alt_text      TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_question_assets_question
    ON question_assets(question_id);

CREATE INDEX IF NOT EXISTS idx_question_assets_scope
    ON question_assets(question_id, scope);
