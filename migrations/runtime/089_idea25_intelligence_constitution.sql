CREATE TABLE IF NOT EXISTS coach_orchestration_runs (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER REFERENCES subjects(id),
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER REFERENCES topics(id),
    focus_layer TEXT NOT NULL,
    guardrail_status TEXT NOT NULL,
    final_action_type TEXT NOT NULL,
    final_route TEXT NOT NULL,
    overall_confidence_score INTEGER NOT NULL DEFAULT 5000,
    contradiction_count INTEGER NOT NULL DEFAULT 0,
    snapshot_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_key, topic_key)
);

CREATE INDEX IF NOT EXISTS idx_coach_orchestration_runs_student
    ON coach_orchestration_runs(student_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS coach_governance_checks (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER REFERENCES subjects(id),
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER REFERENCES topics(id),
    check_code TEXT NOT NULL,
    check_label TEXT NOT NULL,
    owner_engine_key TEXT NOT NULL,
    status TEXT NOT NULL,
    severity TEXT NOT NULL,
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    rationale TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_key, topic_key, check_code)
);

CREATE INDEX IF NOT EXISTS idx_coach_governance_checks_student
    ON coach_governance_checks(student_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS coach_arbitration_records (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER REFERENCES subjects(id),
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER REFERENCES topics(id),
    arbitration_code TEXT NOT NULL,
    winning_engine_key TEXT NOT NULL,
    losing_engine_keys_json TEXT NOT NULL DEFAULT '[]',
    authority_class TEXT NOT NULL,
    rationale TEXT NOT NULL,
    outcome_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_key, topic_key, arbitration_code)
);

CREATE INDEX IF NOT EXISTS idx_coach_arbitration_records_student
    ON coach_arbitration_records(student_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS coach_engine_health_snapshots (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER REFERENCES subjects(id),
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER REFERENCES topics(id),
    engine_key TEXT NOT NULL,
    engine_title TEXT NOT NULL,
    layer TEXT NOT NULL,
    health_status TEXT NOT NULL,
    health_score INTEGER NOT NULL DEFAULT 5000,
    rationale TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_key, topic_key, engine_key)
);

CREATE INDEX IF NOT EXISTS idx_coach_engine_health_snapshots_student
    ON coach_engine_health_snapshots(student_id, updated_at DESC);
