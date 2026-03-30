-- Mock Centre extensions: mock types, blueprint-driven assembly, sections, per-question timing.

ALTER TABLE mock_sessions ADD COLUMN mock_type TEXT NOT NULL DEFAULT 'forecast';
ALTER TABLE mock_sessions ADD COLUMN blueprint_id INTEGER;
ALTER TABLE mock_sessions ADD COLUMN section_count INTEGER;

CREATE TABLE mock_sections (
    id INTEGER PRIMARY KEY,
    mock_session_id INTEGER NOT NULL REFERENCES mock_sessions(id),
    section_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    question_count INTEGER NOT NULL,
    time_limit_seconds INTEGER,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX idx_mock_sections_session ON mock_sections(mock_session_id);

ALTER TABLE session_items ADD COLUMN section_id INTEGER;
ALTER TABLE session_items ADD COLUMN confidence_level TEXT;
