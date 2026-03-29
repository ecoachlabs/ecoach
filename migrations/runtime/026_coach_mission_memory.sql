CREATE TABLE IF NOT EXISTS coach_mission_memories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mission_id INTEGER NOT NULL REFERENCES coach_missions(id) ON DELETE CASCADE,
    plan_day_id INTEGER REFERENCES coach_plan_days(id),
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_id INTEGER REFERENCES sessions(id),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    mission_status TEXT NOT NULL DEFAULT 'completed'
        CHECK (mission_status IN ('completed', 'partial', 'repair_required', 'review_due')),
    attempt_count INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    accuracy_score INTEGER,
    avg_latency_ms INTEGER,
    misconception_tags TEXT NOT NULL DEFAULT '[]',
    review_due_at TEXT,
    next_action_type TEXT NOT NULL DEFAULT 'review_results',
    strategy_effect TEXT,
    summary_json TEXT NOT NULL DEFAULT '{}',
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (review_status IN ('pending', 'acknowledged')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(mission_id)
);

CREATE INDEX IF NOT EXISTS idx_coach_mission_memories_student
    ON coach_mission_memories(student_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_coach_mission_memories_review
    ON coach_mission_memories(student_id, review_status, review_due_at);
