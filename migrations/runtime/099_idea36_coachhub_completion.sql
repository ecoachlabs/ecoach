-- idea36: CoachHub goal intelligence, evidence inbox, uploaded review, and
-- question-environment application layer.

ALTER TABLE goals ADD COLUMN goal_category TEXT NOT NULL DEFAULT 'preparation';
ALTER TABLE goals ADD COLUMN subject_id INTEGER REFERENCES subjects(id);
ALTER TABLE goals ADD COLUMN topics_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE goals ADD COLUMN urgency_level TEXT NOT NULL DEFAULT 'normal';
ALTER TABLE goals ADD COLUMN start_date TEXT;
ALTER TABLE goals ADD COLUMN deadline TEXT;
ALTER TABLE goals ADD COLUMN exam_id INTEGER REFERENCES academic_calendar_events(id);
ALTER TABLE goals ADD COLUMN confidence_score_bp INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE goals ADD COLUMN parent_priority_flag INTEGER NOT NULL DEFAULT 0;
ALTER TABLE goals ADD COLUMN suggested_weekly_effort_minutes INTEGER;
ALTER TABLE goals ADD COLUMN current_momentum_bp INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE goals ADD COLUMN blocked_reason TEXT;
ALTER TABLE goals ADD COLUMN goal_signal_key TEXT;
ALTER TABLE goals ADD COLUMN source_bundle_id INTEGER REFERENCES submission_bundles(id);
ALTER TABLE goals ADD COLUMN merged_goal_ids_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE goals ADD COLUMN metadata_json TEXT NOT NULL DEFAULT '{}';

CREATE INDEX IF NOT EXISTS idx_goals_student_state_level
    ON goals(student_id, goal_state, goal_level, coach_priority_bp DESC);
CREATE INDEX IF NOT EXISTS idx_goals_student_bundle
    ON goals(student_id, source_bundle_id);

ALTER TABLE submission_bundles ADD COLUMN confirmation_state TEXT NOT NULL DEFAULT 'not_needed';
ALTER TABLE submission_bundles ADD COLUMN coach_application_status TEXT NOT NULL DEFAULT 'pending';
ALTER TABLE submission_bundles ADD COLUMN coach_application_summary_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE submission_bundles ADD COLUMN last_reviewed_at TEXT;
ALTER TABLE submission_bundles ADD COLUMN last_applied_at TEXT;

CREATE INDEX IF NOT EXISTS idx_submission_bundles_student_inbox
    ON submission_bundles(student_id, status, confirmation_state, coach_application_status);

CREATE TABLE IF NOT EXISTS submission_bundle_review_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bundle_id INTEGER NOT NULL REFERENCES submission_bundles(id) ON DELETE CASCADE,
    question_ref TEXT NOT NULL,
    topic_label TEXT,
    review_side TEXT NOT NULL DEFAULT 'student',
    reflection_kind TEXT NOT NULL,
    reflection_text TEXT NOT NULL,
    recommended_action TEXT,
    severity_bp INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_bundle_review_notes_bundle
    ON submission_bundle_review_notes(bundle_id, created_at DESC);

CREATE TABLE IF NOT EXISTS student_question_environment_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    teacher_style TEXT NOT NULL DEFAULT 'balanced',
    directness_profile TEXT NOT NULL DEFAULT 'mixed',
    answer_depth_expectation TEXT NOT NULL DEFAULT 'balanced',
    objective_vs_structured_balance TEXT NOT NULL DEFAULT 'mixed',
    typical_difficulty TEXT NOT NULL DEFAULT 'medium',
    mark_loss_patterns_json TEXT NOT NULL DEFAULT '[]',
    environment_signals_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE INDEX IF NOT EXISTS idx_question_environment_profiles_student
    ON student_question_environment_profiles(student_id, subject_id);
