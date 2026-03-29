CREATE TABLE IF NOT EXISTS coach_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    exam_target TEXT,
    exam_date TEXT,
    start_date TEXT NOT NULL,
    total_days INTEGER,
    daily_budget_minutes INTEGER NOT NULL DEFAULT 60,
    current_phase TEXT NOT NULL DEFAULT 'foundation'
        CHECK (current_phase IN ('foundation', 'strengthening', 'performance', 'consolidation', 'final_revision')),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('draft', 'active', 'stale', 'completed')),
    plan_data_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS coach_plan_days (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id INTEGER NOT NULL REFERENCES coach_plans(id) ON DELETE CASCADE,
    date TEXT NOT NULL,
    phase TEXT NOT NULL,
    target_minutes INTEGER NOT NULL DEFAULT 60,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'missed', 'partial')),
    carryover_minutes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS coach_missions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_day_id INTEGER REFERENCES coach_plan_days(id),
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    title TEXT NOT NULL,
    reason TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    primary_topic_id INTEGER REFERENCES topics(id),
    activity_type TEXT NOT NULL
        CHECK (activity_type IN ('learn', 'guided_practice', 'worked_example', 'review', 'speed_drill', 'repair', 'checkpoint', 'mixed_test', 'memory_reactivation', 'pressure_conditioning')),
    target_minutes INTEGER NOT NULL DEFAULT 20,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'skipped', 'deferred')),
    steps_json TEXT NOT NULL DEFAULT '[]',
    success_criteria_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS coach_topic_profiles (
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    mastery_estimate INTEGER NOT NULL DEFAULT 0,
    fragility_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    misconception_recurrence INTEGER NOT NULL DEFAULT 0,
    evidence_count INTEGER NOT NULL DEFAULT 0,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    last_seen_at TEXT,
    blocked_status INTEGER NOT NULL DEFAULT 0,
    repair_priority INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (student_id, topic_id)
);

CREATE TABLE IF NOT EXISTS coach_session_evidence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mission_id INTEGER REFERENCES coach_missions(id),
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    activity_type TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    accuracy INTEGER,
    timed_accuracy INTEGER,
    avg_latency_ms INTEGER,
    misconception_tags TEXT NOT NULL DEFAULT '[]',
    confidence_score INTEGER,
    completed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS coach_blockers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    reason TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'moderate',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_coach_plans_student ON coach_plans(student_id);
CREATE INDEX IF NOT EXISTS idx_coach_plan_days_plan ON coach_plan_days(plan_id);
CREATE INDEX IF NOT EXISTS idx_coach_plan_days_date ON coach_plan_days(date);
CREATE INDEX IF NOT EXISTS idx_coach_missions_student ON coach_missions(student_id);
CREATE INDEX IF NOT EXISTS idx_coach_missions_status ON coach_missions(status);
CREATE INDEX IF NOT EXISTS idx_coach_topic_profiles ON coach_topic_profiles(student_id, topic_id);
CREATE INDEX IF NOT EXISTS idx_coach_evidence_student ON coach_session_evidence(student_id);
