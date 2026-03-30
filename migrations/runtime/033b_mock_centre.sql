CREATE TABLE IF NOT EXISTS mock_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'active', 'paused', 'time_up', 'completed', 'abandoned')),
    duration_minutes INTEGER NOT NULL DEFAULT 120,
    question_count INTEGER NOT NULL DEFAULT 0,
    paper_year TEXT,
    grade TEXT,
    percentage REAL,
    time_banked_seconds INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    resumed_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_mock_sessions_student ON mock_sessions(student_id, subject_id);
CREATE INDEX IF NOT EXISTS idx_mock_sessions_status ON mock_sessions(status);
CREATE INDEX IF NOT EXISTS idx_mock_sessions_grade ON mock_sessions(grade);
