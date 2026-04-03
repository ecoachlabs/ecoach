ALTER TABLE availability_profiles ADD COLUMN ideal_session_minutes INTEGER NOT NULL DEFAULT 60;
ALTER TABLE availability_profiles ADD COLUMN split_sessions_allowed INTEGER NOT NULL DEFAULT 1;
ALTER TABLE availability_profiles ADD COLUMN max_split_sessions INTEGER NOT NULL DEFAULT 2;
ALTER TABLE availability_profiles ADD COLUMN min_break_minutes INTEGER NOT NULL DEFAULT 20;
ALTER TABLE availability_profiles ADD COLUMN trigger_mode TEXT NOT NULL DEFAULT 'hybrid'
    CHECK (trigger_mode IN ('manual', 'auto', 'hybrid'));
ALTER TABLE availability_profiles ADD COLUMN notification_lead_minutes INTEGER NOT NULL DEFAULT 10;
ALTER TABLE availability_profiles ADD COLUMN weekday_capacity_weight_bp INTEGER NOT NULL DEFAULT 10000;
ALTER TABLE availability_profiles ADD COLUMN weekend_capacity_weight_bp INTEGER NOT NULL DEFAULT 11500;
ALTER TABLE availability_profiles ADD COLUMN schedule_buffer_ratio_bp INTEGER NOT NULL DEFAULT 1500;
ALTER TABLE availability_profiles ADD COLUMN fatigue_start_minute INTEGER;
ALTER TABLE availability_profiles ADD COLUMN fatigue_end_minute INTEGER;
ALTER TABLE availability_profiles ADD COLUMN thinking_idle_grace_seconds INTEGER NOT NULL DEFAULT 180;
ALTER TABLE availability_profiles ADD COLUMN idle_confirmation_seconds INTEGER NOT NULL DEFAULT 120;
ALTER TABLE availability_profiles ADD COLUMN abandonment_seconds INTEGER NOT NULL DEFAULT 900;

CREATE TABLE IF NOT EXISTS exam_plan_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    anchor_date TEXT NOT NULL,
    exam_date TEXT NOT NULL,
    target_effective_minutes INTEGER NOT NULL,
    completed_effective_minutes INTEGER NOT NULL DEFAULT 0,
    remaining_effective_minutes INTEGER NOT NULL DEFAULT 0,
    available_study_days INTEGER NOT NULL DEFAULT 0,
    required_weekly_minutes INTEGER NOT NULL DEFAULT 0,
    protected_buffer_minutes INTEGER NOT NULL DEFAULT 0,
    buffer_consumed_minutes INTEGER NOT NULL DEFAULT 0,
    missed_debt_minutes INTEGER NOT NULL DEFAULT 0,
    bonus_credit_minutes INTEGER NOT NULL DEFAULT 0,
    pressure_score_bp INTEGER NOT NULL DEFAULT 0,
    feasibility_score_bp INTEGER NOT NULL DEFAULT 0,
    schedule_truth_score_bp INTEGER NOT NULL DEFAULT 0,
    plan_mode TEXT NOT NULL DEFAULT 'mastery'
        CHECK (plan_mode IN ('mastery', 'exam_performance', 'rescue')),
    auto_trigger_mode TEXT NOT NULL DEFAULT 'hybrid'
        CHECK (auto_trigger_mode IN ('manual', 'auto', 'hybrid')),
    explanation_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, exam_date)
);

CREATE TABLE IF NOT EXISTS schedule_ledger (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    ledger_date TEXT NOT NULL,
    scheduled_minutes INTEGER NOT NULL DEFAULT 0,
    completed_minutes INTEGER NOT NULL DEFAULT 0,
    effective_credit_minutes INTEGER NOT NULL DEFAULT 0,
    buffer_minutes_reserved INTEGER NOT NULL DEFAULT 0,
    buffer_minutes_consumed INTEGER NOT NULL DEFAULT 0,
    missed_minutes_debt INTEGER NOT NULL DEFAULT 0,
    bonus_minutes_credit INTEGER NOT NULL DEFAULT 0,
    pressure_score_bp INTEGER NOT NULL DEFAULT 0,
    feasibility_score_bp INTEGER NOT NULL DEFAULT 0,
    explanation_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, ledger_date)
);

CREATE TABLE IF NOT EXISTS time_session_blocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    plan_state_id INTEGER REFERENCES exam_plan_states(id) ON DELETE SET NULL,
    block_date TEXT NOT NULL,
    start_minute INTEGER,
    end_minute INTEGER,
    target_minutes INTEGER NOT NULL DEFAULT 0,
    session_type TEXT NOT NULL,
    objective_summary TEXT NOT NULL,
    focus_topic_ids_json TEXT NOT NULL DEFAULT '[]',
    trigger_mode TEXT NOT NULL DEFAULT 'manual'
        CHECK (trigger_mode IN ('manual', 'auto', 'hybrid')),
    fit_score_bp INTEGER NOT NULL DEFAULT 0,
    priority_score_bp INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'planned'
        CHECK (status IN ('planned', 'queued', 'triggered', 'started', 'completed', 'missed', 'skipped', 'replaced')),
    fallback_session_type TEXT,
    replacement_options_json TEXT NOT NULL DEFAULT '[]',
    created_by TEXT NOT NULL DEFAULT 'orchestrator',
    source_kind TEXT NOT NULL DEFAULT 'planned_window',
    explanation_text TEXT NOT NULL,
    linked_session_id INTEGER REFERENCES sessions(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS schedule_trigger_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    session_block_id INTEGER REFERENCES time_session_blocks(id) ON DELETE CASCADE,
    trigger_kind TEXT NOT NULL DEFAULT 'reminder'
        CHECK (trigger_kind IN ('launch', 'reminder', 'hybrid_prompt')),
    scheduled_for TEXT NOT NULL,
    lead_minutes INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'scheduled'
        CHECK (status IN ('scheduled', 'claimed', 'fired', 'dismissed', 'cancelled')),
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS session_presence_snapshots (
    session_id INTEGER PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    current_state TEXT NOT NULL DEFAULT 'launched_unengaged',
    current_segment_started_at TEXT,
    first_meaningful_at TEXT,
    last_meaningful_at TEXT,
    idle_started_at TEXT,
    idle_confirmed_at TEXT,
    interruption_started_at TEXT,
    gross_elapsed_ms INTEGER NOT NULL DEFAULT 0,
    active_engaged_ms INTEGER NOT NULL DEFAULT 0,
    passive_engaged_ms INTEGER NOT NULL DEFAULT 0,
    thinking_time_ms INTEGER NOT NULL DEFAULT 0,
    idle_time_ms INTEGER NOT NULL DEFAULT 0,
    interruption_time_ms INTEGER NOT NULL DEFAULT 0,
    counted_study_time_ms INTEGER NOT NULL DEFAULT 0,
    abandonment_risk_bp INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS session_presence_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    state_before TEXT,
    state_after TEXT NOT NULL,
    segment_duration_ms INTEGER NOT NULL DEFAULT 0,
    counted_credit_ms INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_exam_plan_states_student_subject
    ON exam_plan_states(student_id, subject_id, exam_date);
CREATE INDEX IF NOT EXISTS idx_schedule_ledger_student_subject_date
    ON schedule_ledger(student_id, subject_id, ledger_date);
CREATE INDEX IF NOT EXISTS idx_time_session_blocks_student_subject_date
    ON time_session_blocks(student_id, subject_id, block_date, status);
CREATE INDEX IF NOT EXISTS idx_schedule_trigger_jobs_due
    ON schedule_trigger_jobs(status, scheduled_for);
CREATE INDEX IF NOT EXISTS idx_schedule_trigger_jobs_student
    ON schedule_trigger_jobs(student_id, status);
CREATE INDEX IF NOT EXISTS idx_session_presence_events_session_time
    ON session_presence_events(session_id, occurred_at);
