-- ideas 31, 32: Memory decay defense, recall resilience, interference mapping,
-- decay detection profiles, pressure recall testing, recovery strategies.

-- ============================================================================
-- 1. Decay detection profiles (multi-signal decay assessment)
-- ============================================================================

CREATE TABLE decay_detection_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL,
    node_id INTEGER,
    decay_severity TEXT NOT NULL DEFAULT 'green'
        CHECK (decay_severity IN (
            'green', 'yellow', 'orange', 'red', 'black'
        )),
    stability_flag INTEGER NOT NULL DEFAULT 1,
    watchlist_flag INTEGER NOT NULL DEFAULT 0,
    fragile_flag INTEGER NOT NULL DEFAULT 0,
    decaying_flag INTEGER NOT NULL DEFAULT 0,
    collapsed_flag INTEGER NOT NULL DEFAULT 0,
    time_since_last_retrieval_hours INTEGER,
    successful_recall_count INTEGER NOT NULL DEFAULT 0,
    failed_recall_count INTEGER NOT NULL DEFAULT 0,
    recall_speed_trend TEXT,
    confidence_trend TEXT,
    hint_dependency_bp INTEGER NOT NULL DEFAULT 0,
    interference_weight_bp INTEGER NOT NULL DEFAULT 0,
    relapse_frequency INTEGER NOT NULL DEFAULT 0,
    decay_severity_bp INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id, node_id)
);

CREATE INDEX idx_decay_profiles_student ON decay_detection_profiles(student_id, decay_severity);

-- ============================================================================
-- 2. Memory strength profiles (recall type breakdown)
-- ============================================================================

CREATE TABLE memory_strength_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL,
    memory_strength_bp INTEGER NOT NULL DEFAULT 5000,
    recognition_only_bp INTEGER NOT NULL DEFAULT 0,
    cued_recall_bp INTEGER NOT NULL DEFAULT 0,
    free_recall_bp INTEGER NOT NULL DEFAULT 0,
    applied_recall_bp INTEGER NOT NULL DEFAULT 0,
    transferred_recall_bp INTEGER NOT NULL DEFAULT 0,
    pressured_recall_bp INTEGER NOT NULL DEFAULT 0,
    pressure_resistance_bp INTEGER NOT NULL DEFAULT 5000,
    interference_risk_bp INTEGER NOT NULL DEFAULT 0,
    last_three_outcomes_json TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX idx_memory_strength_student ON memory_strength_profiles(student_id);

-- ============================================================================
-- 3. Interference mapping (concept collision tracking)
-- ============================================================================

CREATE TABLE concept_interference_map (
    id INTEGER PRIMARY KEY,
    from_concept_id INTEGER NOT NULL,
    to_concept_id INTEGER NOT NULL,
    confusion_direction TEXT NOT NULL DEFAULT 'bidirectional'
        CHECK (confusion_direction IN ('a_to_b', 'b_to_a', 'bidirectional')),
    frequency_bp INTEGER NOT NULL DEFAULT 0,
    last_observed_at TEXT,
    separation_strategy TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(from_concept_id, to_concept_id)
);

CREATE INDEX idx_interference_from ON concept_interference_map(from_concept_id);

-- ============================================================================
-- 4. Recovery strategies (anti-forgetfulness loops)
-- ============================================================================

CREATE TABLE memory_recovery_strategies (
    id INTEGER PRIMARY KEY,
    strategy_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    loop_type TEXT NOT NULL
        CHECK (loop_type IN (
            'reactivate', 'separate_confusion', 'rebuild_structure',
            'pressure_stabilization', 'long_gap_defense',
            'cue_stripping', 'retrieval_strengthening'
        )),
    steps_json TEXT NOT NULL,
    description TEXT
);

INSERT INTO memory_recovery_strategies (strategy_code, display_name, loop_type, steps_json) VALUES
    ('reactivate_basic', 'Basic Reactivation', 'reactivate', '["recall_prompt","spaced_check","confirm"]'),
    ('confusion_separation', 'Confusion Separation', 'separate_confusion', '["identify_pair","contrast_drill","distinguish_test","spaced_verify"]'),
    ('structure_rebuild', 'Structure Rebuild', 'rebuild_structure', '["reteach_core","connect_related","practice_varied","verify_stable"]'),
    ('pressure_stabilize', 'Pressure Stabilization', 'pressure_stabilization', '["calm_recall","light_timer","moderate_timer","exam_speed"]'),
    ('long_gap_defense', 'Long Gap Defense', 'long_gap_defense', '["initial_probe","guided_recovery","independent_recall","delayed_recheck"]'),
    ('cue_strip', 'Cue Stripping', 'cue_stripping', '["heavy_cue_recall","moderate_cue","light_cue","no_cue_recall"]'),
    ('retrieval_strengthen', 'Retrieval Strengthening', 'retrieval_strengthening', '["varied_cue_practice","interleaved_recall","generation_practice","application_recall"]');

-- ============================================================================
-- 5. Decay priority scoring
-- ============================================================================

CREATE TABLE memory_decay_priorities (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL,
    importance_weight_bp INTEGER NOT NULL DEFAULT 5000,
    exam_relevance_weight_bp INTEGER NOT NULL DEFAULT 5000,
    decay_severity_weight_bp INTEGER NOT NULL DEFAULT 5000,
    recurrence_risk_weight_bp INTEGER NOT NULL DEFAULT 5000,
    dependency_weight_bp INTEGER NOT NULL DEFAULT 5000,
    composite_priority_bp INTEGER NOT NULL DEFAULT 0,
    recommended_strategy_id INTEGER REFERENCES memory_recovery_strategies(id),
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX idx_decay_priorities_student ON memory_decay_priorities(student_id, composite_priority_bp DESC);

-- ============================================================================
-- 6. Study decay profiles (learner-specific decay models - idea 30)
-- ============================================================================

CREATE TABLE study_decay_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    decay_curve_type TEXT NOT NULL DEFAULT 'exponential',
    half_life_hours INTEGER,
    rapid_forgetting_threshold_hours INTEGER,
    retention_curve_params_json TEXT,
    observed_data_points INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 7. Performance time-of-day profiles (idea 30)
-- ============================================================================

CREATE TABLE performance_time_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    time_performance_json TEXT NOT NULL DEFAULT '{}',
    best_performance_window TEXT,
    worst_performance_window TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 8. Schedule rebalancing log (idea 30)
-- ============================================================================

CREATE TABLE schedule_rebalancing_log (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    plan_id INTEGER,
    trigger_reason TEXT NOT NULL
        CHECK (trigger_reason IN (
            'missed_session', 'bonus_session', 'performance_change',
            'decay_detected', 'exam_date_change', 'manual_adjustment'
        )),
    hours_adjusted REAL,
    new_daily_load_minutes INTEGER,
    reasoning_json TEXT,
    rebalanced_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_rebalance_log_student ON schedule_rebalancing_log(student_id);
