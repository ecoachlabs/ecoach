CREATE TABLE IF NOT EXISTS goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    parent_goal_id INTEGER REFERENCES goals(id),
    goal_type TEXT NOT NULL DEFAULT 'campaign'
        CHECK (goal_type IN ('north_star', 'campaign', 'tactical', 'background')),
    title TEXT NOT NULL,
    description TEXT,
    target_date TEXT,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('draft', 'active', 'completed', 'paused', 'archived')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS calendar_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    event_type TEXT NOT NULL
        CHECK (event_type IN ('exam', 'mock', 'class_test', 'assignment', 'milestone')),
    title TEXT NOT NULL,
    scheduled_for TEXT NOT NULL,
    preparation_profile_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_goals_student ON goals(student_id);
CREATE INDEX IF NOT EXISTS idx_calendar_events_student ON calendar_events(student_id);
