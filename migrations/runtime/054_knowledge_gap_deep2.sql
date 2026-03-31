-- idea6 deep gaps: solidification steps, skill state extensions,
-- subject aggregate, hidden blocker flags, intervention tracking.

-- ============================================================================
-- 1. Solidification session steps (stage-by-stage tracking)
-- ============================================================================

CREATE TABLE solidification_session_steps (
    id INTEGER PRIMARY KEY,
    solidification_session_id INTEGER NOT NULL REFERENCES solidification_sessions(id),
    step_type TEXT NOT NULL
        CHECK (step_type IN (
            'diagnose', 'explain', 'worked_example', 'guided_practice',
            'independent_check', 'variation_test', 'timed_check'
        )),
    display_order INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'skipped')),
    result_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_solidification_steps_session ON solidification_session_steps(solidification_session_id);

-- ============================================================================
-- 2. Extend student_skill_states with gap-mode-specific fields
-- ============================================================================

ALTER TABLE student_skill_states ADD COLUMN recurrence_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN dependency_impact_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN forgetting_risk_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN is_hidden_blocker INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN is_urgent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN is_exam_critical_gap INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN is_foundational_gap INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_skill_states ADD COLUMN repair_queue_rank INTEGER;
ALTER TABLE student_skill_states ADD COLUMN intervention_status TEXT;
ALTER TABLE student_skill_states ADD COLUMN intervention_started_at TEXT;
ALTER TABLE student_skill_states ADD COLUMN predicted_decay_at TEXT;

-- ============================================================================
-- 3. Subject-level gap aggregate (fast dashboard rendering)
-- ============================================================================

CREATE TABLE student_subject_gap_aggregate (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    coverage_score INTEGER NOT NULL DEFAULT 0,
    gap_score INTEGER NOT NULL DEFAULT 10000,
    priority_score INTEGER NOT NULL DEFAULT 0,
    critical_blockers_count INTEGER NOT NULL DEFAULT 0,
    slipping_skill_count INTEGER NOT NULL DEFAULT 0,
    unknown_skill_count INTEGER NOT NULL DEFAULT 0,
    weak_skill_count INTEGER NOT NULL DEFAULT 0,
    declining_skill_count INTEGER NOT NULL DEFAULT 0,
    mastered_skill_count INTEGER NOT NULL DEFAULT 0,
    fixed_this_week_count INTEGER NOT NULL DEFAULT 0,
    total_skill_count INTEGER NOT NULL DEFAULT 0,
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE INDEX idx_subject_gap_aggregate_student ON student_subject_gap_aggregate(student_id);

-- ============================================================================
-- 4. Extend student_topic_states with skill count breakdowns
-- ============================================================================

ALTER TABLE student_topic_states ADD COLUMN unknown_skill_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_topic_states ADD COLUMN weak_skill_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_topic_states ADD COLUMN declining_skill_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_topic_states ADD COLUMN critical_skill_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_topic_states ADD COLUMN mastered_skill_count INTEGER NOT NULL DEFAULT 0;
