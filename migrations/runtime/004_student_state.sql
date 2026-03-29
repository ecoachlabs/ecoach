CREATE TABLE IF NOT EXISTS student_topic_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    mastery_score INTEGER NOT NULL DEFAULT 0,
    mastery_state TEXT NOT NULL DEFAULT 'unseen'
        CHECK (mastery_state IN ('unseen', 'exposed', 'emerging', 'partial', 'fragile', 'stable', 'robust', 'exam_ready')),
    accuracy_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    retention_score INTEGER NOT NULL DEFAULT 0,
    transfer_score INTEGER NOT NULL DEFAULT 0,
    consistency_score INTEGER NOT NULL DEFAULT 0,
    gap_score INTEGER NOT NULL DEFAULT 10000,
    priority_score INTEGER NOT NULL DEFAULT 0,
    trend_state TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_state IN ('improving', 'stable', 'fragile', 'declining', 'critical')),
    fragility_score INTEGER NOT NULL DEFAULT 0,
    pressure_collapse_index INTEGER NOT NULL DEFAULT 0,
    total_attempts INTEGER NOT NULL DEFAULT 0,
    correct_attempts INTEGER NOT NULL DEFAULT 0,
    recent_attempts_window INTEGER NOT NULL DEFAULT 0,
    recent_correct_window INTEGER NOT NULL DEFAULT 0,
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_seen_at TEXT,
    last_correct_at TEXT,
    last_mastered_at TEXT,
    last_decline_at TEXT,
    decay_risk INTEGER NOT NULL DEFAULT 0,
    next_review_at TEXT,
    memory_strength INTEGER NOT NULL DEFAULT 0,
    is_blocked INTEGER NOT NULL DEFAULT 0,
    is_urgent INTEGER NOT NULL DEFAULT 0,
    is_exam_critical INTEGER NOT NULL DEFAULT 0,
    repair_priority INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE TABLE IF NOT EXISTS student_error_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    knowledge_gap_score INTEGER NOT NULL DEFAULT 0,
    conceptual_confusion_score INTEGER NOT NULL DEFAULT 0,
    recognition_failure_score INTEGER NOT NULL DEFAULT 0,
    execution_error_score INTEGER NOT NULL DEFAULT 0,
    carelessness_score INTEGER NOT NULL DEFAULT 0,
    pressure_breakdown_score INTEGER NOT NULL DEFAULT 0,
    expression_weakness_score INTEGER NOT NULL DEFAULT 0,
    speed_error_score INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE TABLE IF NOT EXISTS student_question_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    session_id INTEGER,
    session_type TEXT,
    attempt_number INTEGER NOT NULL DEFAULT 1,
    started_at TEXT NOT NULL,
    submitted_at TEXT,
    response_time_ms INTEGER,
    selected_option_id INTEGER REFERENCES question_options(id),
    answer_text TEXT,
    is_correct INTEGER NOT NULL DEFAULT 0,
    confidence_level TEXT CHECK (confidence_level IN ('sure', 'not_sure', 'guessed')),
    hint_count INTEGER NOT NULL DEFAULT 0,
    changed_answer_count INTEGER NOT NULL DEFAULT 0,
    skipped INTEGER NOT NULL DEFAULT 0,
    timed_out INTEGER NOT NULL DEFAULT 0,
    error_type TEXT,
    misconception_triggered_id INTEGER REFERENCES misconception_patterns(id),
    support_level TEXT DEFAULT 'independent'
        CHECK (support_level IN ('independent', 'guided', 'heavily_guided')),
    was_timed INTEGER NOT NULL DEFAULT 0,
    was_transfer_variant INTEGER NOT NULL DEFAULT 0,
    was_retention_check INTEGER NOT NULL DEFAULT 0,
    was_mixed_context INTEGER NOT NULL DEFAULT 0,
    evidence_weight INTEGER NOT NULL DEFAULT 10000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_student_topic_states_student ON student_topic_states(student_id);
CREATE INDEX IF NOT EXISTS idx_student_topic_states_topic ON student_topic_states(topic_id);
CREATE INDEX IF NOT EXISTS idx_student_topic_states_mastery ON student_topic_states(mastery_state);
CREATE INDEX IF NOT EXISTS idx_student_topic_states_priority ON student_topic_states(priority_score DESC);
CREATE INDEX IF NOT EXISTS idx_student_topic_states_review ON student_topic_states(next_review_at);
CREATE INDEX IF NOT EXISTS idx_student_error_profiles ON student_error_profiles(student_id, topic_id);
CREATE INDEX IF NOT EXISTS idx_attempts_student ON student_question_attempts(student_id);
CREATE INDEX IF NOT EXISTS idx_attempts_question ON student_question_attempts(question_id);
CREATE INDEX IF NOT EXISTS idx_attempts_session ON student_question_attempts(session_id);
CREATE INDEX IF NOT EXISTS idx_attempts_created ON student_question_attempts(created_at);
