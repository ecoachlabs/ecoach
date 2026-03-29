CREATE TABLE IF NOT EXISTS risk_flags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER REFERENCES topics(id),
    severity TEXT NOT NULL
        CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'monitoring', 'resolved')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_risk_flags_student ON risk_flags(student_id, status);
CREATE INDEX IF NOT EXISTS idx_risk_flags_severity ON risk_flags(severity, status);

CREATE TABLE IF NOT EXISTS intervention_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    risk_flag_id INTEGER REFERENCES risk_flags(id),
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'review', 'resolved', 'escalated')),
    summary_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_intervention_records_student ON intervention_records(student_id, status);

CREATE TABLE IF NOT EXISTS premium_features (
    feature_key TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    tier_required TEXT NOT NULL DEFAULT 'premium'
        CHECK (tier_required IN ('standard', 'premium', 'elite')),
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS premium_feature_flags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_key TEXT NOT NULL REFERENCES premium_features(feature_key),
    student_id INTEGER REFERENCES accounts(id),
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_premium_feature_flags_unique
    ON premium_feature_flags(feature_key, COALESCE(student_id, -1));

-- Seed default premium features
INSERT OR IGNORE INTO premium_features (feature_key, display_name, tier_required, description) VALUES
    ('risk_detection', 'Automatic Risk Detection', 'premium', 'Auto-detect weak topics and inactivity risks'),
    ('intervention_engine', 'Intervention Engine', 'premium', 'Create and track targeted intervention plans'),
    ('risk_dashboard', 'Risk Dashboard', 'premium', 'Visual risk overview with severity breakdown'),
    ('advanced_parent_digest', 'Advanced Parent Digest', 'premium', 'Detailed parent-facing risk and progress reports'),
    ('concierge_planning', 'Concierge Study Planning', 'premium', 'Personalized study strategy recommendations'),
    ('elite_sessions', 'Elite Performance Sessions', 'elite', 'Precision labs, sprints, depth labs, endurance tracks'),
    ('elite_leaderboard', 'Elite Leaderboard', 'elite', 'Competitive ranking across elite session types'),
    ('elite_topic_domination', 'Topic Domination Tracking', 'elite', 'Per-topic elite performance breakdown');
