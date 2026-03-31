-- ideas 20, 23-25, 29, 35: Coach lifecycle state machine, review modes,
-- goal hierarchy, decision traces, learner digital twin extensions.

-- ============================================================================
-- 1. Coach lifecycle state (canonical state machine)
-- ============================================================================

CREATE TABLE coach_lifecycle_states (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    current_state TEXT NOT NULL DEFAULT 'onboarding_required'
        CHECK (current_state IN (
            'onboarding_required', 'subject_selection_required',
            'diagnostic_required', 'content_readiness_required',
            'plan_generation_required', 'ready_for_today_mission',
            'mission_in_progress', 'mission_review_required',
            'repair_required', 'blocked_on_topic',
            'plan_adjustment_required', 'review_day',
            'exam_mode', 'stalled_no_content'
        )),
    blocking_reason TEXT,
    state_entered_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 2. Coach next actions (computed recommended action)
-- ============================================================================

CREATE TABLE coach_next_actions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    action_type TEXT NOT NULL,
    title TEXT NOT NULL,
    subtitle TEXT,
    estimated_minutes INTEGER,
    route TEXT,
    context_json TEXT,
    urgency_bp INTEGER NOT NULL DEFAULT 5000,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 3. Coach content readiness (gate before missions)
-- ============================================================================

CREATE TABLE coach_content_readiness (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    readiness_status TEXT NOT NULL DEFAULT 'checking'
        CHECK (readiness_status IN (
            'ready', 'no_subjects_selected', 'no_packs_installed',
            'no_topics_available', 'topics_no_questions',
            'insufficient_coverage', 'checking'
        )),
    selected_subjects_json TEXT,
    installed_packs_json TEXT,
    topic_count INTEGER NOT NULL DEFAULT 0,
    question_count INTEGER NOT NULL DEFAULT 0,
    failure_reason TEXT,
    checked_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 4. Recovery states and paths
-- ============================================================================

CREATE TABLE coach_recovery_states (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    state_type TEXT NOT NULL
        CHECK (state_type IN (
            'no_content_installed', 'no_questions_for_topic',
            'plan_generation_failed', 'session_interrupted',
            'insufficient_evidence', 'study_budget_exhausted',
            'topic_blocked_awaiting_repair', 'exam_date_changed'
        )),
    recovery_action TEXT,
    resolved INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX idx_recovery_states_student ON coach_recovery_states(student_id, resolved);

CREATE TABLE coach_recovery_paths (
    id INTEGER PRIMARY KEY,
    weakness_pattern TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    repair_flow_json TEXT NOT NULL,
    description TEXT
);

INSERT INTO coach_recovery_paths (weakness_pattern, display_name, repair_flow_json) VALUES
    ('repeated_misconception', 'Misconception Repair Flow', '["diagnose","contrast_drill","spaced_recheck","confirm"]'),
    ('low_accuracy', 'Accuracy Recovery Flow', '["simplify","scaffolded_practice","gradual_release","verify"]'),
    ('fragile_knowledge', 'Knowledge Stabilization Flow', '["reteach","multi_representation","interleaved_practice","pressure_test"]'),
    ('speed_collapse', 'Speed Recovery Flow', '["untimed_practice","gradual_timing","speed_drill","timed_verify"]'),
    ('pressure_sensitivity', 'Pressure Desensitization Flow', '["calm_practice","light_pressure","moderate_pressure","exam_pressure"]');

-- ============================================================================
-- 5. Review modes and episodes (idea 29)
-- ============================================================================

CREATE TABLE review_modes (
    id INTEGER PRIMARY KEY,
    mode_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    testing_purpose_json TEXT,
    typical_behaviors_json TEXT
);

INSERT INTO review_modes (mode_code, display_name, description) VALUES
    ('immediate_reinforcement', 'Immediate Reinforcement', 'Quick review right after learning'),
    ('structural', 'Structural Review', 'Deep review of concept structure'),
    ('delayed_retrieval', 'Delayed Retrieval', 'Spaced recall after time gap'),
    ('contrast', 'Contrast Review', 'Compare similar/confusable concepts'),
    ('transfer', 'Transfer Review', 'Apply knowledge in new contexts'),
    ('pressure', 'Pressure Review', 'Recall under time pressure');

CREATE TABLE review_episodes (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER,
    concept_id INTEGER,
    review_mode_id INTEGER NOT NULL REFERENCES review_modes(id),
    session_id INTEGER REFERENCES sessions(id),
    stated_purpose TEXT,
    status TEXT NOT NULL DEFAULT 'scheduled'
        CHECK (status IN ('scheduled', 'in_progress', 'completed', 'skipped')),
    scheduled_for_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_review_episodes_student ON review_episodes(student_id, status);

-- ============================================================================
-- 6. Failure cause taxonomy (idea 29)
-- ============================================================================

CREATE TABLE failure_cause_taxonomy (
    id INTEGER PRIMARY KEY,
    failure_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    diagnostic_signals_json TEXT,
    recovery_pathway TEXT
);

INSERT INTO failure_cause_taxonomy (failure_code, display_name) VALUES
    ('true_understanding_gap', 'True Understanding Gap'),
    ('partial_understanding', 'Partial Understanding'),
    ('concept_confusion', 'Concept Confusion'),
    ('step_omission', 'Step Omission'),
    ('pressure_collapse', 'Pressure Collapse'),
    ('reading_failure', 'Reading/Interpretation Failure'),
    ('translation_failure', 'Verbal-to-Symbolic Translation Failure'),
    ('memory_decay', 'Memory Decay'),
    ('cue_dependence', 'Cue-Dependent Recall'),
    ('slow_processing', 'Slow Processing'),
    ('timed_collapse', 'Timed Performance Collapse'),
    ('expression_bottleneck', 'Expression/Writing Bottleneck');

-- ============================================================================
-- 7. Goal hierarchy (idea 35)
-- ============================================================================

ALTER TABLE goals ADD COLUMN goal_level TEXT NOT NULL DEFAULT 'tactical';
ALTER TABLE goals ADD COLUMN goal_state TEXT NOT NULL DEFAULT 'active';
ALTER TABLE goals ADD COLUMN coach_priority_bp INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE goals ADD COLUMN evidence_sources_json TEXT;
ALTER TABLE goals ADD COLUMN dependency_goals_json TEXT;
ALTER TABLE goals ADD COLUMN risk_level TEXT;
ALTER TABLE goals ADD COLUMN completion_criteria_json TEXT;
ALTER TABLE goals ADD COLUMN momentum_state TEXT;

-- ============================================================================
-- 8. Decision traces (governance/audit - ideas 24, 25)
-- ============================================================================

CREATE TABLE decision_traces (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    engine_name TEXT NOT NULL,
    decision_type TEXT NOT NULL,
    input_summary_json TEXT NOT NULL,
    output_summary_json TEXT NOT NULL,
    reasoning_json TEXT,
    confidence_bp INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_decision_traces_student ON decision_traces(student_id, engine_name);

-- ============================================================================
-- 9. Coach mission extensions
-- ============================================================================

ALTER TABLE coach_missions ADD COLUMN review_mode TEXT;
ALTER TABLE coach_missions ADD COLUMN diagnostic_intent TEXT;
ALTER TABLE coach_missions ADD COLUMN confidence_label TEXT;
ALTER TABLE coach_missions ADD COLUMN mission_source TEXT;

-- ============================================================================
-- 10. Session extensions for coach tracking
-- ============================================================================

ALTER TABLE sessions ADD COLUMN coach_mode_entered_at TEXT;
ALTER TABLE sessions ADD COLUMN coach_mode_exited_at TEXT;
ALTER TABLE sessions ADD COLUMN active_study_time_ms INTEGER;
ALTER TABLE sessions ADD COLUMN idle_time_ms INTEGER;

-- ============================================================================
-- 11. Extend coach_plans with time orchestration
-- ============================================================================

ALTER TABLE coach_plans ADD COLUMN initial_target_hours INTEGER;
ALTER TABLE coach_plans ADD COLUMN decay_model_type TEXT;
ALTER TABLE coach_plans ADD COLUMN rebalance_frequency_days INTEGER;
ALTER TABLE coach_plans ADD COLUMN last_rebalance_at TEXT;
