-- idea14: Custom Test Intelligence — preparation blueprints, test type profiles,
-- session runtime adaptation, mode-specific policies, test readiness scoring,
-- preparation context, confidence management.

-- ============================================================================
-- 1. Custom test blueprints (preparation plan before question selection)
-- ============================================================================

CREATE TABLE custom_test_blueprints (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    test_type TEXT NOT NULL
        CHECK (test_type IN (
            'class_test', 'midterm', 'mock_exam', 'terminal_exam',
            'revision_quiz', 'teacher_surprise', 'bece_wassce',
            'quick_quiz', 'end_of_term', 'custom'
        )),
    mode TEXT NOT NULL DEFAULT 'likely_questions'
        CHECK (mode IN (
            'likely_questions', 'realistic_simulation', 'pressure_mode',
            'fix_weak_areas', 'confidence_build', 'teach_through_test',
            'last_minute_rescue', 'teacher_style_prep'
        )),
    topic_ids_json TEXT NOT NULL DEFAULT '[]',
    days_until_test INTEGER,
    target_score_bp INTEGER,
    session_archetype TEXT NOT NULL DEFAULT 'mixed_mastery_check',
    question_count INTEGER NOT NULL DEFAULT 20,
    duration_minutes INTEGER,
    difficulty_preference TEXT NOT NULL DEFAULT 'balanced'
        CHECK (difficulty_preference IN ('easier', 'balanced', 'harder')),
    -- Resolved context
    urgency_band TEXT,
    expanded_subtopics_json TEXT,
    weakness_areas_json TEXT,
    -- Session composition
    topic_distribution_json TEXT NOT NULL DEFAULT '{}',
    difficulty_distribution_json TEXT NOT NULL DEFAULT '{}',
    cognitive_distribution_json TEXT NOT NULL DEFAULT '{}',
    ordering_pattern TEXT NOT NULL DEFAULT 'warm_test_trap_finish',
    -- Policies
    feedback_policy TEXT NOT NULL DEFAULT 'end_of_session'
        CHECK (feedback_policy IN ('instant', 'end_of_session', 'hidden_until_completion')),
    hint_policy TEXT NOT NULL DEFAULT 'reduced'
        CHECK (hint_policy IN ('allowed', 'reduced', 'disabled')),
    adaptation_policy TEXT NOT NULL DEFAULT 'light'
        CHECK (adaptation_policy IN ('none', 'light', 'moderate', 'aggressive')),
    pressure_profile TEXT NOT NULL DEFAULT 'medium'
        CHECK (pressure_profile IN ('low', 'medium', 'high', 'exam_authentic', 'tense')),
    -- Compiled output
    compiled_question_ids_json TEXT,
    session_id INTEGER REFERENCES sessions(id),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'compiled', 'active', 'completed', 'abandoned')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX idx_custom_blueprints_student ON custom_test_blueprints(student_id, status);

-- ============================================================================
-- 2. Test type profiles (behavior matrix per test type)
-- ============================================================================

CREATE TABLE test_type_profiles (
    id INTEGER PRIMARY KEY,
    test_type TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    default_question_count INTEGER NOT NULL DEFAULT 20,
    default_duration_minutes INTEGER,
    default_difficulty TEXT NOT NULL DEFAULT 'balanced',
    default_feedback_policy TEXT NOT NULL DEFAULT 'end_of_session',
    default_hint_policy TEXT NOT NULL DEFAULT 'reduced',
    default_adaptation_policy TEXT NOT NULL DEFAULT 'light',
    default_pressure_profile TEXT NOT NULL DEFAULT 'medium',
    default_ordering_pattern TEXT NOT NULL DEFAULT 'warm_test_trap_finish',
    scope_width TEXT NOT NULL DEFAULT 'moderate',
    recall_weight_bp INTEGER NOT NULL DEFAULT 3000,
    application_weight_bp INTEGER NOT NULL DEFAULT 4000,
    reasoning_weight_bp INTEGER NOT NULL DEFAULT 3000,
    trap_density TEXT NOT NULL DEFAULT 'moderate',
    description TEXT
);

INSERT INTO test_type_profiles (test_type, display_name, default_question_count, default_duration_minutes, default_difficulty, default_feedback_policy, default_hint_policy, default_pressure_profile, default_ordering_pattern, scope_width, recall_weight_bp, application_weight_bp, reasoning_weight_bp, trap_density, description) VALUES
    ('class_test', 'Class Test', 15, 30, 'balanced', 'end_of_session', 'reduced', 'medium', 'warm_test_trap_finish', 'narrow', 4000, 4000, 2000, 'low', 'Recent topics, teacher-style, direct questions'),
    ('midterm', 'Midterm Test', 30, 60, 'balanced', 'end_of_session', 'reduced', 'medium', 'warm_test_trap_finish', 'moderate', 3000, 4000, 3000, 'moderate', 'Broader coverage, cross-topic connections'),
    ('mock_exam', 'Mock Exam', 50, 120, 'harder', 'hidden_until_completion', 'disabled', 'exam_authentic', 'simulated_paper', 'wide', 2500, 3500, 4000, 'high', 'Full exam simulation with realistic pressure'),
    ('terminal_exam', 'End of Term Exam', 40, 90, 'balanced', 'hidden_until_completion', 'disabled', 'high', 'simulated_paper', 'wide', 3000, 3500, 3500, 'moderate', 'Comprehensive term-end assessment'),
    ('revision_quiz', 'Revision Quiz', 10, 15, 'easier', 'instant', 'allowed', 'low', 'warm_test_trap_finish', 'narrow', 5000, 3000, 2000, 'low', 'Quick check on recent learning'),
    ('teacher_surprise', 'Teacher Surprise Test', 10, 20, 'balanced', 'end_of_session', 'reduced', 'high', 'warm_test_trap_finish', 'narrow', 4000, 4000, 2000, 'moderate', 'Unexpected test on recent material'),
    ('bece_wassce', 'BECE/WASSCE Prep', 60, 150, 'harder', 'hidden_until_completion', 'disabled', 'exam_authentic', 'simulated_paper', 'wide', 2000, 3500, 4500, 'high', 'National exam full simulation'),
    ('quick_quiz', 'Quick Quiz', 5, 10, 'easier', 'instant', 'allowed', 'low', 'warm_test_trap_finish', 'narrow', 5000, 3000, 2000, 'low', 'Fast knowledge check'),
    ('end_of_term', 'End of Term', 40, 90, 'balanced', 'hidden_until_completion', 'disabled', 'high', 'simulated_paper', 'wide', 3000, 3500, 3500, 'moderate', 'End-of-term comprehensive assessment'),
    ('custom', 'Custom Test', 20, NULL, 'balanced', 'end_of_session', 'reduced', 'medium', 'warm_test_trap_finish', 'moderate', 3000, 4000, 3000, 'moderate', 'Fully customized by student');

-- ============================================================================
-- 3. Session runtime adaptation state (live monitoring)
-- ============================================================================

CREATE TABLE session_adaptation_state (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    current_streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    error_streak INTEGER NOT NULL DEFAULT 0,
    same_misconception_streak INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms INTEGER NOT NULL DEFAULT 0,
    confidence_drop_count INTEGER NOT NULL DEFAULT 0,
    stabilizer_questions_inserted INTEGER NOT NULL DEFAULT 0,
    difficulty_adjustments INTEGER NOT NULL DEFAULT 0,
    current_difficulty_offset INTEGER NOT NULL DEFAULT 0,
    adaptation_events_json TEXT NOT NULL DEFAULT '[]',
    fatigue_indicator_bp INTEGER NOT NULL DEFAULT 0,
    pressure_response_bp INTEGER NOT NULL DEFAULT 5000,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(session_id)
);

-- ============================================================================
-- 4. Custom test results (enriched outcome)
-- ============================================================================

CREATE TABLE custom_test_results (
    id INTEGER PRIMARY KEY,
    blueprint_id INTEGER NOT NULL REFERENCES custom_test_blueprints(id),
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    raw_score_bp INTEGER NOT NULL DEFAULT 0,
    adjusted_readiness_bp INTEGER NOT NULL DEFAULT 0,
    readiness_band TEXT,
    topic_strength_json TEXT NOT NULL DEFAULT '{}',
    misconception_profile_json TEXT,
    timing_profile_json TEXT,
    pressure_response_profile_json TEXT,
    careless_error_count INTEGER NOT NULL DEFAULT 0,
    endurance_drop_bp INTEGER NOT NULL DEFAULT 0,
    speed_band TEXT,
    next_recommended_action TEXT,
    next_focus_topics_json TEXT,
    next_mode_recommendation TEXT,
    interpretation_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_custom_results_student ON custom_test_results(student_id);
CREATE INDEX idx_custom_results_blueprint ON custom_test_results(blueprint_id);

-- ============================================================================
-- 5. Test readiness per test type (student readiness by assessment type)
-- ============================================================================

CREATE TABLE student_test_readiness (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    test_type TEXT NOT NULL,
    readiness_bp INTEGER NOT NULL DEFAULT 5000,
    readiness_band TEXT NOT NULL DEFAULT 'building',
    topic_readiness_json TEXT NOT NULL DEFAULT '{}',
    last_assessed_at TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    trend TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, test_type)
);

CREATE INDEX idx_test_readiness_student ON student_test_readiness(student_id);

-- ============================================================================
-- 6. Extend sessions table with custom test fields
-- ============================================================================

ALTER TABLE sessions ADD COLUMN blueprint_id INTEGER;
ALTER TABLE sessions ADD COLUMN test_type TEXT;
ALTER TABLE sessions ADD COLUMN mode TEXT;
ALTER TABLE sessions ADD COLUMN feedback_policy TEXT;
ALTER TABLE sessions ADD COLUMN hint_policy TEXT;
ALTER TABLE sessions ADD COLUMN pressure_profile TEXT;
ALTER TABLE sessions ADD COLUMN ordering_pattern TEXT;

-- ============================================================================
-- 7. Question fit scoring weights (configurable per mode)
-- ============================================================================

CREATE TABLE question_fit_weights (
    id INTEGER PRIMARY KEY,
    mode TEXT NOT NULL UNIQUE,
    topic_match_weight INTEGER NOT NULL DEFAULT 2500,
    test_type_fit_weight INTEGER NOT NULL DEFAULT 2000,
    mode_fit_weight INTEGER NOT NULL DEFAULT 1500,
    weakness_match_weight INTEGER NOT NULL DEFAULT 1500,
    recency_weight INTEGER NOT NULL DEFAULT 1000,
    timing_fit_weight INTEGER NOT NULL DEFAULT 500,
    realism_fit_weight INTEGER NOT NULL DEFAULT 500,
    variety_bonus_weight INTEGER NOT NULL DEFAULT 500
);

INSERT INTO question_fit_weights (mode, topic_match_weight, test_type_fit_weight, mode_fit_weight, weakness_match_weight, recency_weight, timing_fit_weight, realism_fit_weight, variety_bonus_weight) VALUES
    ('likely_questions', 2500, 2000, 1500, 1000, 1500, 500, 500, 500),
    ('realistic_simulation', 2000, 2500, 1000, 1000, 500, 1000, 1500, 500),
    ('pressure_mode', 2000, 1500, 2000, 1000, 500, 1500, 1000, 500),
    ('fix_weak_areas', 1500, 1000, 1500, 3000, 500, 500, 500, 1000),
    ('confidence_build', 2500, 1500, 1500, 1500, 1000, 500, 500, 500),
    ('teach_through_test', 2000, 1000, 2000, 1500, 1000, 500, 500, 1500),
    ('last_minute_rescue', 2000, 1500, 1500, 2000, 1500, 500, 500, 500),
    ('teacher_style_prep', 2000, 2500, 1500, 1000, 1500, 500, 500, 500);
