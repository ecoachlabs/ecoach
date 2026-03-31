-- idea15 remaining gaps: exam strategy mode, mastery map rollups,
-- comeback flow choreography.

-- ============================================================================
-- 1. Exam strategy profiles (time distribution & tactics)
-- ============================================================================

CREATE TABLE exam_strategy_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    total_exam_minutes INTEGER NOT NULL DEFAULT 120,
    section_time_allocation_json TEXT NOT NULL DEFAULT '{}',
    skip_return_effectiveness_bp INTEGER NOT NULL DEFAULT 5000,
    recheck_value_bp INTEGER NOT NULL DEFAULT 5000,
    rushing_error_rate_bp INTEGER NOT NULL DEFAULT 0,
    overthinking_time_loss_bp INTEGER NOT NULL DEFAULT 0,
    optimal_pace_seconds_per_question INTEGER,
    best_section_order_json TEXT,
    mark_maximization_strategy TEXT,
    when_blank_strategy TEXT,
    elimination_effectiveness_bp INTEGER NOT NULL DEFAULT 5000,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

-- ============================================================================
-- 2. Exam strategy training sessions
-- ============================================================================

CREATE TABLE exam_strategy_sessions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_id INTEGER REFERENCES sessions(id),
    strategy_type TEXT NOT NULL
        CHECK (strategy_type IN (
            'time_distribution', 'skip_return', 'elimination_practice',
            'mark_maximization', 'section_pacing', 'pressure_management',
            'answer_prioritization', 'full_strategy_drill'
        )),
    subject_id INTEGER NOT NULL,
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    questions_skipped INTEGER NOT NULL DEFAULT 0,
    questions_returned_to INTEGER NOT NULL DEFAULT 0,
    marks_gained_from_return INTEGER NOT NULL DEFAULT 0,
    time_wasted_seconds INTEGER NOT NULL DEFAULT 0,
    optimal_time_used_bp INTEGER NOT NULL DEFAULT 5000,
    strategy_score_bp INTEGER NOT NULL DEFAULT 0,
    insights_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_strategy_sessions_student ON exam_strategy_sessions(student_id);

-- ============================================================================
-- 3. Mastery map rollups (computed syllabus tree state)
-- ============================================================================

CREATE TABLE mastery_map_nodes (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    subject_id INTEGER NOT NULL,
    mastery_percentage_bp INTEGER NOT NULL DEFAULT 0,
    stability_state TEXT NOT NULL DEFAULT 'unknown'
        CHECK (stability_state IN (
            'unknown', 'unseen', 'started', 'fragile',
            'building', 'stable', 'strong', 'mastered'
        )),
    is_blocked INTEGER NOT NULL DEFAULT 0,
    blocked_by_json TEXT,
    is_high_yield INTEGER NOT NULL DEFAULT 0,
    exam_risk_bp INTEGER NOT NULL DEFAULT 0,
    dependency_count INTEGER NOT NULL DEFAULT 0,
    dependent_count INTEGER NOT NULL DEFAULT 0,
    score_impact_bp INTEGER NOT NULL DEFAULT 0,
    last_activity_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX idx_mastery_map_student ON mastery_map_nodes(student_id, subject_id);

-- ============================================================================
-- 4. Comeback flows (graceful re-entry choreography)
-- ============================================================================

CREATE TABLE comeback_flows (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    trigger_reason TEXT NOT NULL
        CHECK (trigger_reason IN (
            'missed_days', 'broken_momentum', 'long_absence',
            'failed_session', 'exam_date_passed', 'subject_change'
        )),
    days_inactive INTEGER NOT NULL DEFAULT 0,
    flow_steps_json TEXT NOT NULL DEFAULT '[]',
    current_step INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'completed', 'abandoned')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX idx_comeback_flows_student ON comeback_flows(student_id, status);

-- ============================================================================
-- 5. Comeback flow templates (pre-authored re-entry sequences)
-- ============================================================================

CREATE TABLE comeback_flow_templates (
    id INTEGER PRIMARY KEY,
    template_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    trigger_condition TEXT NOT NULL,
    steps_json TEXT NOT NULL,
    estimated_minutes INTEGER NOT NULL DEFAULT 15,
    description TEXT
);

INSERT INTO comeback_flow_templates (template_code, display_name, trigger_condition, steps_json, estimated_minutes) VALUES
    ('short_absence', 'Quick Return', 'missed 1-3 days', '["memory_rescue_5min","near_win_session","momentum_reset"]', 15),
    ('medium_absence', 'Gentle Re-entry', 'missed 4-7 days', '["memory_rescue_10min","weakness_check","confidence_builder","near_win_session"]', 25),
    ('long_absence', 'Full Recovery', 'missed 8+ days', '["diagnostic_mini","memory_rescue_15min","weakness_repair","confidence_rebuild","new_plan_generation"]', 40),
    ('post_failure', 'After Bad Session', 'failed session', '["confidence_stabilizer","easier_practice","near_win","positive_reflection"]', 15),
    ('exam_recovery', 'Post-Exam Reset', 'after exam', '["reflection","new_target_setting","fresh_diagnostic","new_journey"]', 30);
