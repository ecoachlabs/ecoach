CREATE TABLE IF NOT EXISTS game_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    game_type TEXT NOT NULL,
    session_state TEXT NOT NULL DEFAULT 'created',
    score INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS game_answer_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id),
    was_correct INTEGER NOT NULL DEFAULT 0,
    effect_type TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
