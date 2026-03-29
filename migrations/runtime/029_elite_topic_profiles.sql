CREATE TABLE IF NOT EXISTS elite_topic_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    precision_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    depth_score INTEGER NOT NULL DEFAULT 0,
    composure_score INTEGER NOT NULL DEFAULT 0,
    consistency_score INTEGER NOT NULL DEFAULT 0,
    trap_resistance_score INTEGER NOT NULL DEFAULT 0,
    domination_score INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'foundation'
        CHECK (status IN ('foundation', 'core', 'apex', 'legend')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_elite_topic_profiles_student ON elite_topic_profiles(student_id, subject_id);
CREATE INDEX IF NOT EXISTS idx_elite_topic_profiles_domination ON elite_topic_profiles(domination_score DESC);
