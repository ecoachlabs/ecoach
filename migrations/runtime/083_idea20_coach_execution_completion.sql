-- idea20: coach execution completion
-- Adds explicit daily activity planning plus mission/session linkage so the
-- coach hub can expose a real roadmap and mission runtime.

CREATE TABLE IF NOT EXISTS coach_plan_activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_day_id INTEGER NOT NULL REFERENCES coach_plan_days(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    activity_type TEXT NOT NULL,
    target_minutes INTEGER NOT NULL DEFAULT 0,
    sequence_order INTEGER NOT NULL DEFAULT 1,
    target_outcome_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'blocked', 'deferred', 'skipped')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_coach_plan_activities_day
ON coach_plan_activities(plan_day_id, sequence_order);

CREATE INDEX IF NOT EXISTS idx_coach_plan_activities_status
ON coach_plan_activities(status, activity_type);

ALTER TABLE coach_missions ADD COLUMN session_id INTEGER REFERENCES sessions(id);
ALTER TABLE coach_missions ADD COLUMN question_ids_json TEXT NOT NULL DEFAULT '[]';
