CREATE TABLE IF NOT EXISTS mock_blueprints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    blueprint_type TEXT NOT NULL
        CHECK (blueprint_type IN ('repair_mock', 'recovery_mock', 'balanced_mock', 'coverage_mock', 'pressure_mock')),
    duration_minutes INTEGER,
    question_count INTEGER NOT NULL DEFAULT 0,
    readiness_score INTEGER NOT NULL DEFAULT 0,
    readiness_band TEXT NOT NULL DEFAULT 'developing',
    coverage_json TEXT NOT NULL DEFAULT '{}',
    quota_json TEXT NOT NULL DEFAULT '{}',
    compiled_question_ids_json TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'ready', 'compiled', 'used', 'stale')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_mock_blueprints_student
    ON mock_blueprints(student_id, subject_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS journey_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    target_exam TEXT,
    route_type TEXT NOT NULL DEFAULT 'mastery_route'
        CHECK (route_type IN ('mastery_route', 'repair_route', 'exam_route')),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'completed', 'stale')),
    current_station_code TEXT,
    route_summary_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_journey_routes_student
    ON journey_routes(student_id, subject_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS journey_stations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_id INTEGER NOT NULL REFERENCES journey_routes(id) ON DELETE CASCADE,
    station_code TEXT NOT NULL,
    title TEXT NOT NULL,
    topic_id INTEGER REFERENCES topics(id) ON DELETE SET NULL,
    sequence_no INTEGER NOT NULL,
    station_type TEXT NOT NULL
        CHECK (station_type IN ('foundation', 'repair', 'checkpoint', 'performance', 'review')),
    target_mastery_score INTEGER,
    target_accuracy_score INTEGER,
    target_readiness_score INTEGER,
    status TEXT NOT NULL DEFAULT 'locked'
        CHECK (status IN ('locked', 'available', 'active', 'completed', 'skipped')),
    entry_rule_json TEXT NOT NULL DEFAULT '{}',
    completion_rule_json TEXT NOT NULL DEFAULT '{}',
    evidence_json TEXT NOT NULL DEFAULT '{}',
    unlocked_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(route_id, station_code),
    UNIQUE(route_id, sequence_no)
);

CREATE INDEX IF NOT EXISTS idx_journey_stations_route
    ON journey_stations(route_id, sequence_no, status);
