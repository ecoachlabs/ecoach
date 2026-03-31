-- idea5 deep model: extended elite profiles, topic profiles, session runs,
-- debrief entities, recommendations, title awards, question metadata.

-- ============================================================================
-- 1. Extend elite_profiles with missing fields from idea5 spec
-- ============================================================================

ALTER TABLE elite_profiles ADD COLUMN elite_status TEXT NOT NULL DEFAULT 'active';
ALTER TABLE elite_profiles ADD COLUMN entry_tier TEXT;
ALTER TABLE elite_profiles ADD COLUMN eligibility_state TEXT NOT NULL DEFAULT 'eligible';
ALTER TABLE elite_profiles ADD COLUMN baseline_completed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_profiles ADD COLUMN accuracy_index INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_profiles ADD COLUMN consistency_index INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_profiles ADD COLUMN trap_resistance_index INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_profiles ADD COLUMN current_streak_days INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_profiles ADD COLUMN best_streak_days INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_profiles ADD COLUMN last_elite_session_at TEXT;
ALTER TABLE elite_profiles ADD COLUMN volatility_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_profiles ADD COLUMN burnout_risk_signal INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_profiles ADD COLUMN growth_momentum_signal TEXT NOT NULL DEFAULT 'stable';

-- ============================================================================
-- 2. Extend elite_topic_profiles with missing fields
-- ============================================================================

ALTER TABLE elite_topic_profiles ADD COLUMN endurance_index INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_topic_profiles ADD COLUMN independence_index INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_topic_profiles ADD COLUMN topic_eps_rolling INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_topic_profiles ADD COLUMN recent_decline_flag INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_topic_profiles ADD COLUMN last_practiced_at TEXT;
ALTER TABLE elite_topic_profiles ADD COLUMN last_record_at TEXT;
ALTER TABLE elite_topic_profiles ADD COLUMN dominant_flag INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_topic_profiles ADD COLUMN apex_ready_flag INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_topic_profiles ADD COLUMN full_domination_flag INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_topic_profiles ADD COLUMN decay_signal INTEGER NOT NULL DEFAULT 0;
ALTER TABLE elite_topic_profiles ADD COLUMN attempts_count INTEGER NOT NULL DEFAULT 0;

-- ============================================================================
-- 3. Elite session runs (dedicated runtime tracking)
-- ============================================================================

CREATE TABLE elite_session_runs (
    id INTEGER PRIMARY KEY,
    blueprint_id INTEGER,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    session_type TEXT NOT NULL,
    topic_scope_json TEXT NOT NULL DEFAULT '[]',
    pressure_level TEXT NOT NULL DEFAULT 'standard',
    total_questions INTEGER NOT NULL DEFAULT 0,
    answered_questions INTEGER NOT NULL DEFAULT 0,
    correct_questions INTEGER NOT NULL DEFAULT 0,
    hints_used INTEGER NOT NULL DEFAULT 0,
    skipped_questions INTEGER NOT NULL DEFAULT 0,
    perfect_run_alive INTEGER NOT NULL DEFAULT 1,
    record_alive_state INTEGER NOT NULL DEFAULT 0,
    current_streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'completed', 'abandoned')),
    final_eps INTEGER,
    final_grade TEXT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_elite_session_runs_student ON elite_session_runs(student_id, subject_id);

-- ============================================================================
-- 4. Elite question instances (per-question metadata in session)
-- ============================================================================

CREATE TABLE elite_question_instances (
    id INTEGER PRIMARY KEY,
    session_run_id INTEGER NOT NULL REFERENCES elite_session_runs(id),
    question_id INTEGER NOT NULL,
    position_index INTEGER NOT NULL,
    topic_id INTEGER,
    target_dimension TEXT,
    difficulty_band TEXT,
    pressure_modifier REAL NOT NULL DEFAULT 1.0,
    timer_seconds INTEGER,
    trap_intensity REAL NOT NULL DEFAULT 0.0,
    reasoning_density INTEGER NOT NULL DEFAULT 1,
    was_stabilizer INTEGER NOT NULL DEFAULT 0,
    was_boss_question INTEGER NOT NULL DEFAULT 0,
    -- Attempt data
    is_correct INTEGER,
    response_time_ms INTEGER,
    confidence_level REAL,
    hint_used INTEGER NOT NULL DEFAULT 0,
    changed_answer_count INTEGER NOT NULL DEFAULT 0,
    skip_used INTEGER NOT NULL DEFAULT 0,
    trap_hit_flag INTEGER NOT NULL DEFAULT 0,
    error_type TEXT,
    method_efficiency_score REAL,
    verification_flag INTEGER NOT NULL DEFAULT 0,
    pressure_state_at_submit REAL,
    streak_before INTEGER NOT NULL DEFAULT 0,
    streak_after INTEGER NOT NULL DEFAULT 0,
    -- Computed scores
    accuracy_outcome REAL,
    precision_outcome REAL,
    speed_outcome REAL,
    depth_outcome REAL,
    trap_outcome REAL,
    independence_outcome REAL,
    answered_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_elite_question_instances_run ON elite_question_instances(session_run_id);

-- ============================================================================
-- 5. Elite debriefs
-- ============================================================================

CREATE TABLE elite_debriefs (
    id INTEGER PRIMARY KEY,
    session_run_id INTEGER NOT NULL REFERENCES elite_session_runs(id),
    summary_text TEXT NOT NULL,
    strongest_dimension TEXT,
    weakest_dimension TEXT,
    primary_error_cluster TEXT,
    secondary_error_cluster TEXT,
    fatigue_pattern TEXT,
    pressure_pattern TEXT,
    recommendation_type TEXT,
    recommendation_payload_json TEXT NOT NULL DEFAULT '{}',
    record_events_json TEXT NOT NULL DEFAULT '[]',
    generated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_elite_debriefs_run ON elite_debriefs(session_run_id);

-- ============================================================================
-- 6. Elite recommendations
-- ============================================================================

CREATE TABLE elite_recommendations (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    source_context TEXT NOT NULL,
    recommendation_type TEXT NOT NULL,
    session_type TEXT,
    topic_scope_json TEXT,
    priority INTEGER NOT NULL DEFAULT 5,
    explanation TEXT NOT NULL,
    expected_duration_minutes INTEGER,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'consumed', 'expired', 'dismissed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at TEXT,
    consumed_at TEXT
);

CREATE INDEX idx_elite_recommendations_student ON elite_recommendations(student_id, subject_id);

-- ============================================================================
-- 7. Elite title awards (prestige)
-- ============================================================================

CREATE TABLE elite_title_awards (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    title_code TEXT NOT NULL,
    topic_id INTEGER,
    rationale TEXT,
    awarded_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, title_code)
);

CREATE INDEX idx_elite_title_awards_student ON elite_title_awards(student_id, subject_id);

-- Seed title definitions
INSERT INTO elite_badges (badge_code, badge_name, description) VALUES
    ('algebra_authority', 'Algebra Authority', 'Dominated Algebra at Apex level')
    ON CONFLICT(badge_code) DO NOTHING;
INSERT INTO elite_badges (badge_code, badge_name, description) VALUES
    ('precision_specialist', 'Precision Specialist', 'Maintained 90%+ precision across 10 sessions')
    ON CONFLICT(badge_code) DO NOTHING;
INSERT INTO elite_badges (badge_code, badge_name, description) VALUES
    ('speed_tactician', 'Speed Tactician', 'Consistently fast while maintaining accuracy')
    ON CONFLICT(badge_code) DO NOTHING;
INSERT INTO elite_badges (badge_code, badge_name, description) VALUES
    ('trap_breaker', 'Trap Breaker', 'Avoided traps in 5 consecutive trap-heavy sessions')
    ON CONFLICT(badge_code) DO NOTHING;
INSERT INTO elite_badges (badge_code, badge_name, description) VALUES
    ('apex_finisher', 'Apex Finisher', 'Completed an Apex Mock with distinction')
    ON CONFLICT(badge_code) DO NOTHING;
INSERT INTO elite_badges (badge_code, badge_name, description) VALUES
    ('endurance_steel', 'Endurance Steel', 'Maintained stable performance in long sessions')
    ON CONFLICT(badge_code) DO NOTHING;

-- ============================================================================
-- 8. Add elite question metadata to questions table
-- ============================================================================

ALTER TABLE questions ADD COLUMN precision_demand REAL;
ALTER TABLE questions ADD COLUMN speed_demand REAL;
ALTER TABLE questions ADD COLUMN depth_demand REAL;
ALTER TABLE questions ADD COLUMN trap_intensity REAL;
ALTER TABLE questions ADD COLUMN reasoning_density INTEGER;
ALTER TABLE questions ADD COLUMN elite_session_fit_tags TEXT;
ALTER TABLE questions ADD COLUMN usable_in_elite_mode INTEGER NOT NULL DEFAULT 0;
