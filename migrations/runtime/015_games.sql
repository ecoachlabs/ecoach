CREATE TABLE IF NOT EXISTS game_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    game_type TEXT NOT NULL CHECK (game_type IN ('mindstack', 'tug_of_war', 'traps')),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    session_state TEXT NOT NULL DEFAULT 'created'
        CHECK (session_state IN ('created', 'active', 'paused', 'completed', 'abandoned')),
    score INTEGER NOT NULL DEFAULT 0,
    rounds_total INTEGER NOT NULL DEFAULT 10,
    rounds_played INTEGER NOT NULL DEFAULT 0,
    streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    topic_ids_json TEXT NOT NULL DEFAULT '[]',
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_game_sessions_student ON game_sessions(student_id, game_type);
CREATE INDEX IF NOT EXISTS idx_game_sessions_state ON game_sessions(session_state);

CREATE TABLE IF NOT EXISTS game_answer_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id),
    selected_option_id INTEGER,
    was_correct INTEGER NOT NULL DEFAULT 0,
    response_time_ms INTEGER,
    points_earned INTEGER NOT NULL DEFAULT 0,
    streak_at_answer INTEGER NOT NULL DEFAULT 0,
    misconception_triggered INTEGER NOT NULL DEFAULT 0,
    effect_type TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_game_answer_events_session ON game_answer_events(game_session_id);
