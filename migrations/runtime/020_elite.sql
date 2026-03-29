CREATE TABLE IF NOT EXISTS elite_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    eps_score INTEGER NOT NULL DEFAULT 0,
    tier TEXT NOT NULL DEFAULT 'foundation',
    precision_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    depth_score INTEGER NOT NULL DEFAULT 0,
    composure_score INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE TABLE IF NOT EXISTS elite_session_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    session_type TEXT NOT NULL,
    eps_delta INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
