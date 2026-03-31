-- idea14 deep gaps: preparation requests, session question events,
-- question metadata extensions for fit scoring, scope profiles.

-- ============================================================================
-- 1. Preparation requests (initial user context capture)
-- ============================================================================

CREATE TABLE preparation_requests (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    test_type TEXT NOT NULL,
    mode TEXT NOT NULL,
    subject_id INTEGER NOT NULL,
    topic_ids_json TEXT NOT NULL DEFAULT '[]',
    days_until_test INTEGER,
    target_score_bp INTEGER,
    difficulty_preference TEXT NOT NULL DEFAULT 'balanced',
    teacher_style_hint TEXT,
    session_length_minutes INTEGER,
    confidence_signal TEXT,
    blueprint_id INTEGER REFERENCES custom_test_blueprints(id),
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'resolved', 'compiled', 'abandoned')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_prep_requests_student ON preparation_requests(student_id, status);

-- ============================================================================
-- 2. Session question events (per-question runtime tracking)
-- ============================================================================

CREATE TABLE session_question_events (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    question_id INTEGER NOT NULL,
    question_index INTEGER NOT NULL,
    presented_at TEXT NOT NULL DEFAULT (datetime('now')),
    answered_at TEXT,
    response_time_ms INTEGER,
    is_correct INTEGER,
    selected_option_id INTEGER,
    confidence_signal TEXT,
    misconception_matched TEXT,
    error_cause TEXT,
    difficulty_at_presentation INTEGER,
    was_stabilizer INTEGER NOT NULL DEFAULT 0,
    was_adapted INTEGER NOT NULL DEFAULT 0,
    adaptation_reason TEXT,
    fit_score_bp INTEGER
);

CREATE INDEX idx_session_q_events_session ON session_question_events(session_id);

-- ============================================================================
-- 3. Question metadata extensions for fit scoring
-- ============================================================================

ALTER TABLE questions ADD COLUMN micro_skill_id INTEGER;
ALTER TABLE questions ADD COLUMN exam_style_fit_bp INTEGER;
ALTER TABLE questions ADD COLUMN pressure_suitability_bp INTEGER;
ALTER TABLE questions ADD COLUMN estimated_seconds INTEGER;
ALTER TABLE questions ADD COLUMN revision_value_bp INTEGER;
ALTER TABLE questions ADD COLUMN realism_score_bp INTEGER;

-- ============================================================================
-- 4. Student scope profiles (scoped mastery cache)
-- ============================================================================

CREATE TABLE student_scope_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    scope_hash TEXT NOT NULL,
    scope_topic_ids_json TEXT NOT NULL,
    mastery_by_topic_json TEXT NOT NULL DEFAULT '{}',
    speed_by_family_json TEXT NOT NULL DEFAULT '{}',
    common_misconceptions_json TEXT NOT NULL DEFAULT '[]',
    confidence_by_topic_json TEXT NOT NULL DEFAULT '{}',
    forgetting_risk_json TEXT NOT NULL DEFAULT '{}',
    candidate_question_count INTEGER NOT NULL DEFAULT 0,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, scope_hash)
);

CREATE INDEX idx_scope_profiles_student ON student_scope_profiles(student_id);

-- ============================================================================
-- 5. Session adaptation event log (structured events)
-- ============================================================================

CREATE TABLE session_adaptation_events (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    event_type TEXT NOT NULL
        CHECK (event_type IN (
            'misconception_streak', 'confidence_drop', 'slow_accurate',
            'fast_sloppy', 'overperformance', 'difficulty_adjusted',
            'stabilizer_inserted', 'question_reordered', 'repair_triggered',
            'pace_adjusted'
        )),
    trigger_details_json TEXT,
    action_taken TEXT,
    question_index_at_trigger INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_adapt_events_session ON session_adaptation_events(session_id);

-- ============================================================================
-- 6. Extend custom_test_blueprints with missing fields
-- ============================================================================

ALTER TABLE custom_test_blueprints ADD COLUMN resolved_scope_json TEXT;
ALTER TABLE custom_test_blueprints ADD COLUMN candidate_pool_size INTEGER;
ALTER TABLE custom_test_blueprints ADD COLUMN student_scope_profile_id INTEGER;

-- ============================================================================
-- 7. Extend session_adaptation_state with missing offset
-- ============================================================================

ALTER TABLE session_adaptation_state ADD COLUMN current_misconception_family TEXT;
ALTER TABLE session_adaptation_state ADD COLUMN careless_error_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE session_adaptation_state ADD COLUMN overperformance_streak INTEGER NOT NULL DEFAULT 0;
