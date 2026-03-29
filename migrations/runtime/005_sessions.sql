CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_type TEXT NOT NULL
        CHECK (session_type IN ('practice', 'diagnostic', 'mock', 'gap_repair', 'memory_review', 'coach_mission', 'custom_test', 'elite', 'game', 'traps')),
    subject_id INTEGER REFERENCES subjects(id),
    topic_ids TEXT NOT NULL DEFAULT '[]',
    question_count INTEGER,
    duration_minutes INTEGER,
    is_timed INTEGER NOT NULL DEFAULT 0,
    difficulty_preference TEXT DEFAULT 'adaptive',
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'active', 'paused', 'completed', 'abandoned')),
    started_at TEXT,
    paused_at TEXT,
    completed_at TEXT,
    total_questions INTEGER NOT NULL DEFAULT 0,
    answered_questions INTEGER NOT NULL DEFAULT 0,
    correct_questions INTEGER NOT NULL DEFAULT 0,
    accuracy_score INTEGER,
    avg_response_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS diagnostic_instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    session_mode TEXT NOT NULL DEFAULT 'standard'
        CHECK (session_mode IN ('quick', 'standard', 'deep')),
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'phase_1', 'phase_2', 'phase_3', 'phase_4', 'phase_5', 'completed', 'abandoned')),
    started_at TEXT,
    completed_at TEXT,
    result_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS diagnostic_session_phases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    phase_number INTEGER NOT NULL,
    phase_type TEXT NOT NULL
        CHECK (phase_type IN ('broad_scan', 'adaptive_zoom', 'condition_testing', 'stability_recheck', 'confidence_snapshot')),
    status TEXT NOT NULL DEFAULT 'pending',
    question_count INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    completed_at TEXT,
    phase_result_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS diagnostic_item_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id),
    phase_id INTEGER NOT NULL REFERENCES diagnostic_session_phases(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    display_order INTEGER NOT NULL,
    condition_type TEXT DEFAULT 'normal'
        CHECK (condition_type IN ('normal', 'timed', 'recall', 'recognition', 'transfer', 'stability')),
    sibling_group_id TEXT,
    started_at TEXT,
    submitted_at TEXT,
    response_time_ms INTEGER,
    selected_option_id INTEGER REFERENCES question_options(id),
    is_correct INTEGER,
    confidence_level TEXT,
    changed_answer_count INTEGER DEFAULT 0,
    skipped INTEGER DEFAULT 0,
    timed_out INTEGER DEFAULT 0,
    evidence_weight INTEGER NOT NULL DEFAULT 10000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_sessions_student ON sessions(student_id);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_type ON sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_diag_instances_student ON diagnostic_instances(student_id);
CREATE INDEX IF NOT EXISTS idx_diag_instances_status ON diagnostic_instances(status);
CREATE INDEX IF NOT EXISTS idx_diag_phases_diag ON diagnostic_session_phases(diagnostic_id);
CREATE INDEX IF NOT EXISTS idx_diag_attempts_diag ON diagnostic_item_attempts(diagnostic_id);
