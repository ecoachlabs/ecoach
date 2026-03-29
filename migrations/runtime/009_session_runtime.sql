ALTER TABLE sessions ADD COLUMN active_item_index INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD COLUMN last_activity_at TEXT;
ALTER TABLE sessions ADD COLUMN resume_token TEXT;

CREATE TABLE IF NOT EXISTS session_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    question_id INTEGER NOT NULL REFERENCES questions(id),
    display_order INTEGER NOT NULL,
    source_family_id INTEGER REFERENCES question_families(id),
    source_topic_id INTEGER REFERENCES topics(id),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'presented', 'answered', 'skipped')),
    selected_option_id INTEGER REFERENCES question_options(id),
    answer_state_json TEXT NOT NULL DEFAULT '{}',
    flagged INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    answered_at TEXT,
    response_time_ms INTEGER,
    is_correct INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(session_id, display_order)
);

CREATE INDEX IF NOT EXISTS idx_session_items_session ON session_items(session_id);
CREATE INDEX IF NOT EXISTS idx_session_items_question ON session_items(question_id);
CREATE INDEX IF NOT EXISTS idx_session_items_status ON session_items(status);
