CREATE TABLE IF NOT EXISTS academic_calendar_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    legacy_calendar_event_id INTEGER REFERENCES calendar_events(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    event_type TEXT NOT NULL
        CHECK (event_type IN (
            'quiz', 'class_test', 'mock', 'exam', 'final_exam', 'assignment',
            'project', 'milestone', 'title_defense', 'review_window', 'school_event'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    scheduled_date TEXT NOT NULL,
    start_time TEXT,
    end_time TEXT,
    term TEXT,
    academic_year TEXT,
    importance_bp INTEGER NOT NULL DEFAULT 5000,
    scope TEXT NOT NULL DEFAULT 'focused'
        CHECK (scope IN ('focused', 'mixed', 'broad')),
    linked_topic_ids_json TEXT NOT NULL DEFAULT '[]',
    preparation_window_days INTEGER NOT NULL DEFAULT 14,
    review_window_days INTEGER NOT NULL DEFAULT 7,
    status TEXT NOT NULL DEFAULT 'scheduled'
        CHECK (status IN ('scheduled', 'completed', 'postponed', 'cancelled')),
    result_after_event TEXT,
    coach_priority_weight_bp INTEGER NOT NULL DEFAULT 5000,
    expected_weight_bp INTEGER NOT NULL DEFAULT 5000,
    timed_performance_weight_bp INTEGER NOT NULL DEFAULT 5000,
    coverage_mode TEXT NOT NULL DEFAULT 'mixed'
        CHECK (coverage_mode IN ('focused', 'mixed', 'broad')),
    source TEXT NOT NULL DEFAULT 'manual',
    last_strategy_snapshot_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_academic_calendar_events_student
    ON academic_calendar_events(student_id, scheduled_date);
CREATE INDEX IF NOT EXISTS idx_academic_calendar_events_status
    ON academic_calendar_events(student_id, status, scheduled_date);

CREATE TABLE IF NOT EXISTS reminder_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mission_id INTEGER REFERENCES coach_missions(id) ON DELETE CASCADE,
    academic_event_id INTEGER REFERENCES academic_calendar_events(id) ON DELETE CASCADE,
    reminder_type TEXT NOT NULL
        CHECK (reminder_type IN (
            'session_upcoming', 'session_start', 'session_overdue', 'recovery_prompt',
            'defense_due', 'review_due', 'exam_countdown', 'parent_alert'
        )),
    scheduled_time TEXT NOT NULL,
    audience TEXT NOT NULL
        CHECK (audience IN ('student', 'parent', 'both', 'coach')),
    status TEXT NOT NULL DEFAULT 'scheduled'
        CHECK (status IN ('scheduled', 'sent', 'acknowledged', 'skipped', 'cancelled', 'expired')),
    escalation_level INTEGER NOT NULL DEFAULT 0,
    message TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    sent_at TEXT,
    acknowledged_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_reminder_schedules_learner
    ON reminder_schedules(learner_id, scheduled_time);
CREATE INDEX IF NOT EXISTS idx_reminder_schedules_status
    ON reminder_schedules(learner_id, status, scheduled_time);

CREATE TABLE IF NOT EXISTS engagement_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mission_id INTEGER REFERENCES coach_missions(id) ON DELETE SET NULL,
    session_id INTEGER REFERENCES sessions(id) ON DELETE SET NULL,
    reminder_schedule_id INTEGER REFERENCES reminder_schedules(id) ON DELETE SET NULL,
    academic_event_id INTEGER REFERENCES academic_calendar_events(id) ON DELETE SET NULL,
    session_state TEXT NOT NULL
        CHECK (session_state IN (
            'scheduled', 'started', 'completed', 'partially_completed',
            'paused', 'rescheduled', 'missed', 'excused'
        )),
    started_at TEXT,
    ended_at TEXT,
    completion_percent INTEGER NOT NULL DEFAULT 0,
    missed_reason TEXT,
    source TEXT NOT NULL DEFAULT 'coach',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_engagement_events_learner
    ON engagement_events(learner_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_engagement_events_state
    ON engagement_events(learner_id, session_state, created_at DESC);

CREATE TABLE IF NOT EXISTS engagement_risk_profiles (
    learner_id INTEGER PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    risk_level TEXT NOT NULL DEFAULT 'low'
        CHECK (risk_level IN ('low', 'medium', 'high', 'critical')),
    risk_score_bp INTEGER NOT NULL DEFAULT 0,
    consecutive_misses INTEGER NOT NULL DEFAULT 0,
    recent_partial_sessions INTEGER NOT NULL DEFAULT 0,
    last_session_state TEXT,
    next_recovery_action TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS parent_access_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    visibility_mode TEXT NOT NULL DEFAULT 'standard'
        CHECK (visibility_mode IN ('overview', 'standard', 'full')),
    reminders_enabled INTEGER NOT NULL DEFAULT 1,
    alerts_enabled INTEGER NOT NULL DEFAULT 1,
    feedback_enabled INTEGER NOT NULL DEFAULT 1,
    can_excuse_sessions INTEGER NOT NULL DEFAULT 1,
    preferred_channel TEXT NOT NULL DEFAULT 'in_app'
        CHECK (preferred_channel IN ('in_app', 'email', 'sms')),
    quiet_hours_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(parent_id, learner_id)
);

CREATE TABLE IF NOT EXISTS parent_feedback_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    parent_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    category TEXT NOT NULL
        CHECK (category IN (
            'schedule', 'fatigue', 'motivation', 'behavior',
            'wellbeing', 'logistics', 'performance', 'other'
        )),
    message TEXT NOT NULL,
    interpreted_signal TEXT NOT NULL,
    urgency TEXT NOT NULL DEFAULT 'low'
        CHECK (urgency IN ('low', 'medium', 'high', 'urgent')),
    suggested_support_action TEXT,
    visible_strategy_change TEXT,
    status TEXT NOT NULL DEFAULT 'new'
        CHECK (status IN ('new', 'reviewed', 'applied', 'archived')),
    submitted_at TEXT NOT NULL DEFAULT (datetime('now')),
    applied_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_parent_feedback_learner
    ON parent_feedback_records(learner_id, submitted_at DESC);

CREATE TABLE IF NOT EXISTS parent_alert_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    parent_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    trigger_type TEXT NOT NULL,
    severity TEXT NOT NULL
        CHECK (severity IN ('info', 'watch', 'high', 'urgent')),
    message TEXT NOT NULL,
    action_required TEXT,
    status TEXT NOT NULL DEFAULT 'open'
        CHECK (status IN ('open', 'acknowledged', 'resolved', 'dismissed')),
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    acknowledged_at TEXT,
    resolved_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_parent_alerts_parent
    ON parent_alert_records(parent_id, status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_parent_alerts_learner
    ON parent_alert_records(learner_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS strategy_adjustment_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    source TEXT NOT NULL
        CHECK (source IN ('calendar', 'feedback', 'engagement', 'performance', 'title')),
    old_strategy_snapshot_json TEXT NOT NULL DEFAULT '{}',
    new_strategy_snapshot_json TEXT NOT NULL DEFAULT '{}',
    visible_message_student TEXT,
    visible_message_parent TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_strategy_adjustment_logs_learner
    ON strategy_adjustment_logs(learner_id, created_at DESC);

CREATE TABLE IF NOT EXISTS coach_title_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    title_name TEXT NOT NULL,
    state TEXT NOT NULL
        CHECK (state IN (
            'candidate', 'earned', 'active', 'defense_due', 'defended',
            'narrowly_defended', 'contested', 'dormant', 'reclaimed'
        )),
    earned_at TEXT,
    last_defended_at TEXT,
    next_defense_due_at TEXT,
    coach_note TEXT,
    evidence_snapshot_json TEXT NOT NULL DEFAULT '{}',
    reclaim_plan_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id, title_name)
);

CREATE INDEX IF NOT EXISTS idx_coach_title_states_student
    ON coach_title_states(student_id, state, next_defense_due_at);

CREATE TABLE IF NOT EXISTS coach_title_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title_state_id INTEGER NOT NULL REFERENCES coach_title_states(id) ON DELETE CASCADE,
    previous_state TEXT,
    new_state TEXT NOT NULL,
    reason TEXT NOT NULL,
    snapshot_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_coach_title_history_title
    ON coach_title_history(title_state_id, created_at DESC);

CREATE TABLE IF NOT EXISTS title_defense_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title_state_id INTEGER NOT NULL REFERENCES coach_title_states(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    composition_json TEXT NOT NULL DEFAULT '{}',
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    outcome TEXT NOT NULL DEFAULT 'pending'
        CHECK (outcome IN ('pending', 'defended', 'narrowly_defended', 'contested')),
    evaluation_json TEXT NOT NULL DEFAULT '{}',
    coach_note TEXT
);

CREATE INDEX IF NOT EXISTS idx_title_defense_runs_title
    ON title_defense_runs(title_state_id, started_at DESC);

CREATE TABLE IF NOT EXISTS coach_badge_awards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    badge_name TEXT NOT NULL,
    badge_family TEXT NOT NULL
        CHECK (badge_family IN ('breakthrough', 'recovery', 'performance', 'discipline', 'title')),
    reason TEXT NOT NULL,
    related_session_id INTEGER REFERENCES sessions(id) ON DELETE SET NULL,
    related_title_state_id INTEGER REFERENCES coach_title_states(id) ON DELETE SET NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    awarded_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_coach_badge_awards_student
    ON coach_badge_awards(student_id, awarded_at DESC);
