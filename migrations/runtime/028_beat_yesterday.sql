CREATE TABLE IF NOT EXISTS beat_yesterday_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    current_stage TEXT NOT NULL DEFAULT 'rescue'
        CHECK (current_stage IN ('rescue', 'stabilize', 'accelerate', 'dominate')),
    current_mode TEXT NOT NULL DEFAULT 'volume_push'
        CHECK (current_mode IN ('volume_push', 'accuracy_repair', 'speed_lift', 'recovery_mode')),
    momentum_score INTEGER NOT NULL DEFAULT 5000,
    strain_score INTEGER NOT NULL DEFAULT 0,
    readiness_score INTEGER NOT NULL DEFAULT 0,
    recovery_need_score INTEGER NOT NULL DEFAULT 0,
    streak_days INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE TABLE IF NOT EXISTS beat_yesterday_daily_targets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    target_date TEXT NOT NULL,
    stage TEXT NOT NULL
        CHECK (stage IN ('rescue', 'stabilize', 'accelerate', 'dominate')),
    mode TEXT NOT NULL
        CHECK (mode IN ('volume_push', 'accuracy_repair', 'speed_lift', 'recovery_mode')),
    target_attempts INTEGER NOT NULL DEFAULT 0,
    target_correct INTEGER NOT NULL DEFAULT 0,
    target_avg_response_time_ms INTEGER,
    warm_start_minutes INTEGER NOT NULL DEFAULT 2,
    core_climb_minutes INTEGER NOT NULL DEFAULT 5,
    speed_burst_minutes INTEGER NOT NULL DEFAULT 1,
    finish_strong_minutes INTEGER NOT NULL DEFAULT 1,
    focus_topic_ids_json TEXT NOT NULL DEFAULT '[]',
    rationale_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'planned'
        CHECK (status IN ('planned', 'active', 'completed', 'skipped')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, target_date)
);

CREATE TABLE IF NOT EXISTS beat_yesterday_daily_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_id INTEGER REFERENCES beat_yesterday_daily_targets(id) ON DELETE SET NULL,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    summary_date TEXT NOT NULL,
    actual_attempts INTEGER NOT NULL DEFAULT 0,
    actual_correct INTEGER NOT NULL DEFAULT 0,
    actual_avg_response_time_ms INTEGER,
    beat_attempt_target INTEGER NOT NULL DEFAULT 0,
    beat_accuracy_target INTEGER NOT NULL DEFAULT 0,
    beat_pace_target INTEGER NOT NULL DEFAULT 0,
    momentum_score INTEGER NOT NULL DEFAULT 5000,
    strain_score INTEGER NOT NULL DEFAULT 0,
    recovery_mode_triggered INTEGER NOT NULL DEFAULT 0,
    summary_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, summary_date)
);

CREATE INDEX IF NOT EXISTS idx_beat_yesterday_profiles_student ON beat_yesterday_profiles(student_id);
CREATE INDEX IF NOT EXISTS idx_beat_yesterday_targets_student_date ON beat_yesterday_daily_targets(student_id, target_date);
CREATE INDEX IF NOT EXISTS idx_beat_yesterday_summaries_student_date ON beat_yesterday_daily_summaries(student_id, summary_date);
