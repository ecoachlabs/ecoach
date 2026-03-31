-- idea15: Teach Mode, retention architecture, momentum tracking,
-- weakness lifecycle, memory queue, confidence calibration,
-- near-win detection, answer construction, speed profiles.

-- ============================================================================
-- 1. Teach mode content delivery (explanation variants per concept)
-- ============================================================================

CREATE TABLE teach_explanations (
    id INTEGER PRIMARY KEY,
    node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    explanation_level TEXT NOT NULL DEFAULT 'standard'
        CHECK (explanation_level IN (
            'beginner', 'standard', 'exam_ready', 'elite', 'repair'
        )),
    hero_summary TEXT,
    why_it_matters TEXT,
    simple_explanation TEXT,
    structured_breakdown_json TEXT,
    worked_examples_json TEXT,
    common_mistakes_json TEXT,
    exam_appearance_notes TEXT,
    pattern_recognition_tips TEXT,
    related_concepts_json TEXT,
    visual_asset_refs_json TEXT,
    subject_style TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(node_id, explanation_level)
);

CREATE INDEX idx_teach_explanations_node ON teach_explanations(node_id);

-- ============================================================================
-- 2. Teach mode micro-checks (inline verification questions)
-- ============================================================================

CREATE TABLE teach_micro_checks (
    id INTEGER PRIMARY KEY,
    explanation_id INTEGER NOT NULL REFERENCES teach_explanations(id),
    check_type TEXT NOT NULL DEFAULT 'true_false'
        CHECK (check_type IN (
            'true_false', 'fill_gap', 'step_ordering',
            'select_correct', 'spot_mistake', 'quick_apply'
        )),
    prompt TEXT NOT NULL,
    correct_answer TEXT NOT NULL,
    distractor_answers_json TEXT,
    explanation_if_wrong TEXT,
    position_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_micro_checks_explanation ON teach_micro_checks(explanation_id);

-- ============================================================================
-- 3. Weakness lifecycle tracking
-- ============================================================================

CREATE TABLE weakness_lifecycle (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL,
    subtopic_id INTEGER,
    weakness_type TEXT NOT NULL
        CHECK (weakness_type IN (
            'concept_gap', 'misconception', 'speed_weakness',
            'pressure_weakness', 'recall_fragility', 'application_failure',
            'careless_pattern', 'exam_technique'
        )),
    lifecycle_state TEXT NOT NULL DEFAULT 'detected'
        CHECK (lifecycle_state IN (
            'detected', 'analyzed', 'being_repaired', 'partially_improved',
            'stable_low_pressure', 'stable_mixed', 'stable_pressure', 'closed'
        )),
    severity_bp INTEGER NOT NULL DEFAULT 5000,
    detection_source TEXT,
    repair_attempts INTEGER NOT NULL DEFAULT 0,
    last_repair_at TEXT,
    evidence_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    closed_at TEXT,
    UNIQUE(student_id, topic_id, weakness_type)
);

CREATE INDEX idx_weakness_lifecycle_student ON weakness_lifecycle(student_id, lifecycle_state);

-- ============================================================================
-- 4. Memory queue buckets
-- ============================================================================

CREATE TABLE memory_queue_items (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL,
    node_id INTEGER,
    bucket TEXT NOT NULL DEFAULT 'due_today'
        CHECK (bucket IN (
            'due_today', 'fading_now', 'high_value',
            'recently_recovered', 'emergency_recall', 'reinforcement'
        )),
    priority_bp INTEGER NOT NULL DEFAULT 5000,
    decay_risk_bp INTEGER NOT NULL DEFAULT 0,
    exam_value_bp INTEGER NOT NULL DEFAULT 5000,
    last_recall_at TEXT,
    next_review_at TEXT,
    consecutive_successes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id, node_id)
);

CREATE INDEX idx_memory_queue_student_bucket ON memory_queue_items(student_id, bucket);

-- ============================================================================
-- 5. Momentum state tracking
-- ============================================================================

CREATE TABLE student_momentum (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    momentum_state TEXT NOT NULL DEFAULT 'building'
        CHECK (momentum_state IN (
            'building', 'strong', 'slipping', 'broken', 'comeback'
        )),
    current_streak_days INTEGER NOT NULL DEFAULT 0,
    best_streak_days INTEGER NOT NULL DEFAULT 0,
    consistency_7d_bp INTEGER NOT NULL DEFAULT 0,
    consistency_14d_bp INTEGER NOT NULL DEFAULT 0,
    consistency_30d_bp INTEGER NOT NULL DEFAULT 0,
    dropout_risk_bp INTEGER NOT NULL DEFAULT 0,
    last_session_date TEXT,
    days_since_last_session INTEGER NOT NULL DEFAULT 0,
    comeback_session_count INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 6. Confidence calibration responses
-- ============================================================================

CREATE TABLE confidence_responses (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_id INTEGER NOT NULL,
    question_id INTEGER NOT NULL,
    confidence_level TEXT NOT NULL
        CHECK (confidence_level IN (
            'guessed', 'unsure', 'somewhat_sure', 'confident', 'certain'
        )),
    was_correct INTEGER NOT NULL DEFAULT 0,
    calibration_category TEXT
        CHECK (calibration_category IN (
            'guessed_correct', 'shaky_correct', 'solid_correct',
            'overconfident_wrong', 'uncertain_wrong', 'guessed_wrong'
        )),
    response_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_confidence_responses_student ON confidence_responses(student_id);
CREATE INDEX idx_confidence_responses_session ON confidence_responses(session_id);

-- ============================================================================
-- 7. Confidence calibration profile (aggregated per student)
-- ============================================================================

CREATE TABLE student_confidence_profile (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER,
    overconfidence_rate_bp INTEGER NOT NULL DEFAULT 0,
    underconfidence_rate_bp INTEGER NOT NULL DEFAULT 0,
    guess_rate_bp INTEGER NOT NULL DEFAULT 0,
    calibration_accuracy_bp INTEGER NOT NULL DEFAULT 5000,
    confidence_reliability_bp INTEGER NOT NULL DEFAULT 5000,
    total_responses INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

-- ============================================================================
-- 8. Speed profiles per question family
-- ============================================================================

CREATE TABLE student_speed_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    family_id INTEGER NOT NULL REFERENCES question_families(id),
    avg_time_ms INTEGER NOT NULL DEFAULT 0,
    median_time_ms INTEGER NOT NULL DEFAULT 0,
    fastest_time_ms INTEGER,
    slowest_time_ms INTEGER,
    optimal_pace_ms INTEGER,
    rushing_threshold_ms INTEGER,
    overthinking_threshold_ms INTEGER,
    careless_error_rate_bp INTEGER NOT NULL DEFAULT 0,
    hesitation_rate_bp INTEGER NOT NULL DEFAULT 0,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, family_id)
);

CREATE INDEX idx_speed_profiles_student ON student_speed_profiles(student_id);

-- ============================================================================
-- 9. Near-win tracking (topics close to mastery)
-- ============================================================================

CREATE TABLE near_win_opportunities (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL,
    current_mastery_bp INTEGER NOT NULL DEFAULT 0,
    target_mastery_bp INTEGER NOT NULL DEFAULT 7500,
    gap_bp INTEGER NOT NULL DEFAULT 0,
    estimated_sessions_to_close INTEGER NOT NULL DEFAULT 1,
    estimated_minutes_to_close INTEGER,
    score_gain_if_closed_bp INTEGER NOT NULL DEFAULT 0,
    priority_rank INTEGER NOT NULL DEFAULT 0,
    opportunity_type TEXT NOT NULL DEFAULT 'nearly_mastered'
        CHECK (opportunity_type IN (
            'nearly_mastered', 'almost_closed_weakness',
            'one_more_session', 'quick_win', 'high_value_close'
        )),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX idx_near_wins_student ON near_win_opportunities(student_id, priority_rank);

-- ============================================================================
-- 10. Answer construction rubrics (mark scheme storage)
-- ============================================================================

CREATE TABLE answer_rubrics (
    id INTEGER PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id),
    rubric_type TEXT NOT NULL DEFAULT 'step_based'
        CHECK (rubric_type IN ('step_based', 'holistic', 'criterion')),
    total_marks INTEGER NOT NULL DEFAULT 1,
    steps_json TEXT NOT NULL DEFAULT '[]',
    mandatory_steps_json TEXT,
    full_answer_example TEXT,
    concise_answer_example TEXT,
    weak_answer_example TEXT,
    common_omissions_json TEXT,
    marking_notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_answer_rubrics_question ON answer_rubrics(question_id);

-- ============================================================================
-- 11. Step submissions for answer construction
-- ============================================================================

CREATE TABLE step_submissions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_id INTEGER NOT NULL,
    question_id INTEGER NOT NULL,
    rubric_id INTEGER NOT NULL REFERENCES answer_rubrics(id),
    submitted_steps_json TEXT NOT NULL DEFAULT '[]',
    matched_steps_json TEXT,
    omitted_steps_json TEXT,
    extra_steps_json TEXT,
    step_quality_bp INTEGER NOT NULL DEFAULT 0,
    marks_awarded INTEGER NOT NULL DEFAULT 0,
    marks_possible INTEGER NOT NULL DEFAULT 0,
    feedback_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_step_submissions_student ON step_submissions(student_id);

-- ============================================================================
-- 12. Mission urgency scoring (daily mission composition rules)
-- ============================================================================

CREATE TABLE mission_urgency_scores (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL,
    decay_urgency_bp INTEGER NOT NULL DEFAULT 0,
    weakness_urgency_bp INTEGER NOT NULL DEFAULT 0,
    exam_proximity_urgency_bp INTEGER NOT NULL DEFAULT 0,
    mistake_recurrence_urgency_bp INTEGER NOT NULL DEFAULT 0,
    score_impact_urgency_bp INTEGER NOT NULL DEFAULT 0,
    composite_urgency_bp INTEGER NOT NULL DEFAULT 0,
    estimated_minutes INTEGER,
    estimated_score_gain_bp INTEGER,
    recommended_mode TEXT,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX idx_mission_urgency_student ON mission_urgency_scores(student_id, composite_urgency_bp DESC);

-- ============================================================================
-- 13. Duel/competitive sessions
-- ============================================================================

CREATE TABLE duel_sessions (
    id INTEGER PRIMARY KEY,
    challenger_id INTEGER NOT NULL REFERENCES accounts(id),
    opponent_id INTEGER REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    topic_id INTEGER,
    duel_type TEXT NOT NULL DEFAULT 'topic_duel'
        CHECK (duel_type IN (
            'topic_duel', 'speed_challenge', 'accuracy_battle',
            'revenge_match', 'class_challenge'
        )),
    question_count INTEGER NOT NULL DEFAULT 10,
    time_limit_seconds INTEGER,
    challenger_score_bp INTEGER,
    opponent_score_bp INTEGER,
    winner_id INTEGER,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'expired', 'declined')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX idx_duels_challenger ON duel_sessions(challenger_id, status);
CREATE INDEX idx_duels_opponent ON duel_sessions(opponent_id, status);

-- ============================================================================
-- 14. Revenge queue (re-beat past mistakes)
-- ============================================================================

CREATE TABLE revenge_queue (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    original_session_id INTEGER,
    original_error_type TEXT,
    original_wrong_answer TEXT,
    attempts_to_beat INTEGER NOT NULL DEFAULT 0,
    is_beaten INTEGER NOT NULL DEFAULT 0,
    beaten_at TEXT,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, question_id)
);

CREATE INDEX idx_revenge_queue_student ON revenge_queue(student_id, is_beaten);

-- ============================================================================
-- 15. Tutor interaction log
-- ============================================================================

CREATE TABLE tutor_interactions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_id INTEGER,
    question_id INTEGER,
    topic_id INTEGER,
    interaction_type TEXT NOT NULL
        CHECK (interaction_type IN (
            'why_correct', 'why_wrong', 'explain_simply',
            'show_example', 'step_by_step', 'show_shortcut',
            'compare_options', 'pattern_hint', 'custom_question'
        )),
    prompt_text TEXT,
    response_text TEXT,
    context_json TEXT,
    was_helpful INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_tutor_interactions_student ON tutor_interactions(student_id);

-- ============================================================================
-- 16. Focus mode sessions
-- ============================================================================

ALTER TABLE sessions ADD COLUMN focus_mode INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD COLUMN focus_goal TEXT;
ALTER TABLE sessions ADD COLUMN break_schedule_json TEXT;
ALTER TABLE sessions ADD COLUMN ambient_profile TEXT;
