-- idea2.txt deep model: learner misconception states, goal targets, evidence events,
-- progress velocity, extended skill states, expanded station types/statuses.

-- ============================================================================
-- 1. Learner misconception state tracking (per user, per misconception)
-- ============================================================================

CREATE TABLE learner_misconception_states (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    misconception_id INTEGER NOT NULL REFERENCES misconception_patterns(id),
    risk_score INTEGER NOT NULL DEFAULT 0,
    first_detected_at TEXT,
    last_detected_at TEXT,
    times_detected INTEGER NOT NULL DEFAULT 0,
    current_status TEXT NOT NULL DEFAULT 'dormant'
        CHECK (current_status IN (
            'dormant', 'suspected', 'active', 'reducing',
            'cleared_but_watch', 'cleared'
        )),
    cleared_confidence INTEGER NOT NULL DEFAULT 0,
    related_node_ids_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, misconception_id)
);

CREATE INDEX idx_learner_misconception_student ON learner_misconception_states(student_id, subject_id);
CREATE INDEX idx_learner_misconception_status ON learner_misconception_states(current_status);

-- ============================================================================
-- 2. Goal targets (exam goals that outlive individual routes)
-- ============================================================================

CREATE TABLE goal_targets (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    target_exam TEXT,
    target_score_band TEXT NOT NULL DEFAULT 'pass',
    target_readiness_type TEXT NOT NULL DEFAULT 'exam_ready'
        CHECK (target_readiness_type IN ('pass', 'strong', 'elite', 'deep_mastery')),
    exam_date TEXT,
    minimum_acceptable_goal TEXT,
    preferred_goal TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, is_active)
);

CREATE INDEX idx_goal_targets_student ON goal_targets(student_id, subject_id);

-- Link journey_routes to goal_targets
ALTER TABLE journey_routes ADD COLUMN goal_target_id INTEGER REFERENCES goal_targets(id);

-- ============================================================================
-- 3. Evidence events (interpreted meaning from each answer)
-- ============================================================================

CREATE TABLE evidence_events (
    id INTEGER PRIMARY KEY,
    attempt_id INTEGER NOT NULL REFERENCES student_question_attempts(id),
    student_id INTEGER NOT NULL,
    subject_id INTEGER NOT NULL,
    topic_id INTEGER,
    node_id INTEGER,
    skill_id INTEGER,
    station_id INTEGER,
    testing_reason TEXT,
    evidence_weight INTEGER NOT NULL DEFAULT 5000,
    mastery_delta INTEGER NOT NULL DEFAULT 0,
    stability_delta INTEGER NOT NULL DEFAULT 0,
    retention_delta INTEGER NOT NULL DEFAULT 0,
    transfer_delta INTEGER NOT NULL DEFAULT 0,
    timed_delta INTEGER NOT NULL DEFAULT 0,
    misconception_signal TEXT,
    hypothesis_result TEXT CHECK (hypothesis_result IN ('confirmed', 'challenged', 'neutral')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_evidence_events_student ON evidence_events(student_id, subject_id);
CREATE INDEX idx_evidence_events_attempt ON evidence_events(attempt_id);
CREATE INDEX idx_evidence_events_topic ON evidence_events(topic_id);
CREATE INDEX idx_evidence_events_created ON evidence_events(created_at);

-- ============================================================================
-- 4. Progress velocity (computed rates of improvement)
-- ============================================================================

CREATE TABLE progress_velocity (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    station_clearance_rate INTEGER NOT NULL DEFAULT 0,
    mastery_gain_rate INTEGER NOT NULL DEFAULT 0,
    retention_gain_rate INTEGER NOT NULL DEFAULT 0,
    mock_improvement_rate INTEGER NOT NULL DEFAULT 0,
    speed_gain_rate INTEGER NOT NULL DEFAULT 0,
    challenge_success_rate INTEGER NOT NULL DEFAULT 0,
    coverage_expansion_rate INTEGER NOT NULL DEFAULT 0,
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE INDEX idx_progress_velocity_student ON progress_velocity(student_id, subject_id);

-- ============================================================================
-- 5. Extend student_skill_states with idea2.txt fields
-- SQLite supports ALTER TABLE ADD COLUMN, so we add columns incrementally.
-- ============================================================================

ALTER TABLE student_skill_states ADD COLUMN historical_peak_level TEXT;
ALTER TABLE student_skill_states ADD COLUMN stability_score INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE student_skill_states ADD COLUMN retention_confidence INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE student_skill_states ADD COLUMN transfer_strength INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN timed_strength INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN coverage_depth INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN proof_tier TEXT NOT NULL DEFAULT 'weak';
ALTER TABLE student_skill_states ADD COLUMN node_status TEXT NOT NULL DEFAULT 'unseen';
ALTER TABLE student_skill_states ADD COLUMN reactivation_required INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN last_confirmed_at TEXT;
ALTER TABLE student_skill_states ADD COLUMN last_mastered_at TEXT;
ALTER TABLE student_skill_states ADD COLUMN review_due_at TEXT;
ALTER TABLE student_skill_states ADD COLUMN misconception_risk INTEGER NOT NULL DEFAULT 0;

-- ============================================================================
-- 6. Extend journey_stations with richer tracking
-- We cannot ALTER CHECK constraints in SQLite, but we CAN add columns.
-- The station_type and status values are enforced in Rust code, not just SQL.
-- New station_types: challenge, mini_mock, reactivation, readiness_gate
-- New statuses: improving, fragile, checkpoint_pending, challenge_pending,
--               mini_mock_pending, cleared, retained, at_risk, needs_reactivation
-- ============================================================================

ALTER TABLE journey_stations ADD COLUMN progress_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE journey_stations ADD COLUMN blocking_skill_ids_json TEXT;
ALTER TABLE journey_stations ADD COLUMN completion_confidence INTEGER NOT NULL DEFAULT 0;
ALTER TABLE journey_stations ADD COLUMN times_entered INTEGER NOT NULL DEFAULT 0;
ALTER TABLE journey_stations ADD COLUMN times_reactivated INTEGER NOT NULL DEFAULT 0;
ALTER TABLE journey_stations ADD COLUMN retention_checked_at TEXT;
ALTER TABLE journey_stations ADD COLUMN reactivation_due_at TEXT;

-- ============================================================================
-- 7. Plan control state (strategic control alongside route)
-- ============================================================================

CREATE TABLE plan_control_states (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    route_mode TEXT NOT NULL DEFAULT 'balanced',
    deadline_pressure_index INTEGER NOT NULL DEFAULT 0,
    work_remaining_score INTEGER NOT NULL DEFAULT 10000,
    goal_gap_score INTEGER NOT NULL DEFAULT 10000,
    mock_intensity_level TEXT NOT NULL DEFAULT 'standard',
    recall_intensity_level TEXT NOT NULL DEFAULT 'standard',
    proof_burden_profile TEXT NOT NULL DEFAULT 'standard',
    compression_level INTEGER NOT NULL DEFAULT 0,
    deepening_level INTEGER NOT NULL DEFAULT 0,
    feasibility_status TEXT NOT NULL DEFAULT 'feasible'
        CHECK (feasibility_status IN (
            'comfortable', 'feasible', 'challenging', 'at_risk', 'needs_adjustment'
        )),
    last_recalculated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE INDEX idx_plan_control_student ON plan_control_states(student_id, subject_id);
