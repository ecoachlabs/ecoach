-- idea12: Premium Academic Concierge Program — strategy memos, milestone reviews,
-- concierge responses, readiness profiles, premium intake, risk taxonomy expansion,
-- intervention lifecycle enrichment, parent communication ledger.

-- ============================================================================
-- 1. Risk flags expansion (add category, review scheduling, lifecycle states)
-- ============================================================================

ALTER TABLE risk_flags ADD COLUMN risk_category TEXT;
ALTER TABLE risk_flags ADD COLUMN risk_group TEXT;
ALTER TABLE risk_flags ADD COLUMN impact_description TEXT;
ALTER TABLE risk_flags ADD COLUMN evidence_json TEXT;
ALTER TABLE risk_flags ADD COLUMN review_at TEXT;
ALTER TABLE risk_flags ADD COLUMN linked_intervention_id INTEGER;
ALTER TABLE risk_flags ADD COLUMN trigger_rule TEXT;
ALTER TABLE risk_flags ADD COLUMN direction TEXT;

-- ============================================================================
-- 2. Intervention records expansion (lifecycle, classes, review scheduling)
-- ============================================================================

ALTER TABLE intervention_records ADD COLUMN intervention_class TEXT;
ALTER TABLE intervention_records ADD COLUMN objective TEXT;
ALTER TABLE intervention_records ADD COLUMN trigger_evidence TEXT;
ALTER TABLE intervention_records ADD COLUMN activation_date TEXT;
ALTER TABLE intervention_records ADD COLUMN review_date TEXT;
ALTER TABLE intervention_records ADD COLUMN outcome_summary TEXT;
ALTER TABLE intervention_records ADD COLUMN outcome_mastery_delta INTEGER;
ALTER TABLE intervention_records ADD COLUMN qualification_state TEXT NOT NULL DEFAULT 'qualified';
ALTER TABLE intervention_records ADD COLUMN session_pattern TEXT;
ALTER TABLE intervention_records ADD COLUMN success_criteria TEXT;
ALTER TABLE intervention_records ADD COLUMN progress_bp INTEGER NOT NULL DEFAULT 0;

-- ============================================================================
-- 3. Strategy memos expansion (structured sections)
-- ============================================================================

ALTER TABLE weekly_memos ADD COLUMN readiness_band TEXT;
ALTER TABLE weekly_memos ADD COLUMN days_to_exam INTEGER;
ALTER TABLE weekly_memos ADD COLUMN executive_summary TEXT;
ALTER TABLE weekly_memos ADD COLUMN improvements_json TEXT;
ALTER TABLE weekly_memos ADD COLUMN exposed_areas_json TEXT;
ALTER TABLE weekly_memos ADD COLUMN strategy_changes_json TEXT;
ALTER TABLE weekly_memos ADD COLUMN next_seven_days_json TEXT;
ALTER TABLE weekly_memos ADD COLUMN parent_note TEXT;
ALTER TABLE weekly_memos ADD COLUMN memo_type TEXT NOT NULL DEFAULT 'weekly';

-- ============================================================================
-- 4. Milestone reviews
-- ============================================================================

CREATE TABLE milestone_reviews (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    parent_id INTEGER REFERENCES accounts(id),
    review_type TEXT NOT NULL
        CHECK (review_type IN (
            'thirty_day', 'sixty_day', 'pre_mock', 'pre_exam', 'custom'
        )),
    review_date TEXT NOT NULL DEFAULT (datetime('now')),
    readiness_band TEXT NOT NULL,
    overall_trend TEXT NOT NULL,
    executive_position TEXT NOT NULL,
    subject_progression_json TEXT NOT NULL DEFAULT '[]',
    intervention_effectiveness_json TEXT,
    confirmed_strengths_json TEXT,
    unresolved_risks_json TEXT,
    strategic_adjustments TEXT,
    forecast_summary TEXT,
    parent_guidance TEXT,
    reviewer_type TEXT NOT NULL DEFAULT 'system'
        CHECK (reviewer_type IN ('system', 'strategist', 'specialist')),
    reviewer_name TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_milestone_reviews_student ON milestone_reviews(student_id);

-- ============================================================================
-- 5. Concierge responses
-- ============================================================================

CREATE TABLE concierge_responses (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    parent_id INTEGER NOT NULL REFERENCES accounts(id),
    question_family TEXT
        CHECK (question_family IN (
            'status', 'risk', 'strategy', 'forecast',
            'action', 'explanation', 'custom'
        )),
    parent_question TEXT NOT NULL,
    direct_answer TEXT NOT NULL,
    evidence_summary TEXT,
    academic_interpretation TEXT,
    current_action TEXT,
    expected_outcome TEXT,
    parent_action_needed TEXT,
    evidence_refs_json TEXT,
    strategy_state_snapshot_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_concierge_responses_student ON concierge_responses(student_id);
CREATE INDEX idx_concierge_responses_parent ON concierge_responses(parent_id);

-- ============================================================================
-- 6. Readiness profile snapshots (persistent for trending)
-- ============================================================================

CREATE TABLE readiness_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    snapshot_date TEXT NOT NULL DEFAULT (datetime('now')),
    overall_readiness_bp INTEGER NOT NULL DEFAULT 5000,
    overall_band TEXT NOT NULL DEFAULT 'building',
    knowledge_solidity_bp INTEGER NOT NULL DEFAULT 5000,
    application_strength_bp INTEGER NOT NULL DEFAULT 5000,
    reasoning_quality_bp INTEGER NOT NULL DEFAULT 5000,
    speed_under_pressure_bp INTEGER NOT NULL DEFAULT 5000,
    memory_stability_bp INTEGER NOT NULL DEFAULT 5000,
    confidence_resilience_bp INTEGER NOT NULL DEFAULT 5000,
    consistency_bp INTEGER NOT NULL DEFAULT 5000,
    exam_technique_bp INTEGER NOT NULL DEFAULT 5000,
    target_band TEXT,
    trajectory TEXT,
    interpretation TEXT,
    subject_readiness_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_readiness_profiles_student ON readiness_profiles(student_id, snapshot_date);

-- ============================================================================
-- 7. Premium intake (structured onboarding data)
-- ============================================================================

CREATE TABLE premium_intakes (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    parent_id INTEGER NOT NULL REFERENCES accounts(id),
    -- Student profile
    school_name TEXT,
    school_type TEXT,
    curriculum TEXT,
    exam_board TEXT,
    subjects_json TEXT NOT NULL DEFAULT '[]',
    -- Parent goals
    target_performance TEXT,
    target_school TEXT,
    priority_subjects_json TEXT,
    urgency_level TEXT
        CHECK (urgency_level IN ('low', 'moderate', 'high', 'urgent')),
    biggest_worry TEXT,
    success_definition TEXT,
    -- Learning history
    recent_results_json TEXT,
    known_strengths TEXT,
    known_weaknesses TEXT,
    avoided_subjects TEXT,
    previous_tutoring TEXT,
    consistency_pattern TEXT,
    pressure_behavior TEXT,
    -- Study logistics
    available_hours_per_week INTEGER,
    device_type TEXT,
    preferred_study_windows TEXT,
    internet_quality TEXT,
    supervision_level TEXT,
    -- Child mindset
    confidence_level TEXT,
    anxiety_level TEXT,
    attention_consistency TEXT,
    resilience_when_corrected TEXT,
    tendency_to_rush INTEGER NOT NULL DEFAULT 0,
    tendency_to_hesitate INTEGER NOT NULL DEFAULT 0,
    -- Status
    intake_status TEXT NOT NULL DEFAULT 'draft'
        CHECK (intake_status IN ('draft', 'submitted', 'reviewed', 'activated')),
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX idx_premium_intakes_student ON premium_intakes(student_id);

-- ============================================================================
-- 8. Parent communication ledger
-- ============================================================================

CREATE TABLE parent_communications (
    id INTEGER PRIMARY KEY,
    parent_id INTEGER NOT NULL REFERENCES accounts(id),
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    comm_type TEXT NOT NULL
        CHECK (comm_type IN (
            'risk_alert', 'strategy_shift', 'milestone_progress',
            'weekly_memo', 'milestone_review', 'concierge_response',
            'effort_update', 'readiness_update'
        )),
    priority INTEGER NOT NULL DEFAULT 3
        CHECK (priority BETWEEN 1 AND 5),
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    evidence_summary TEXT,
    linked_entity_type TEXT,
    linked_entity_id INTEGER,
    read_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_parent_comms_parent ON parent_communications(parent_id, created_at DESC);
CREATE INDEX idx_parent_comms_student ON parent_communications(student_id);

-- ============================================================================
-- 9. Strategy state persistence
-- ============================================================================

CREATE TABLE strategy_states (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    primary_focus TEXT,
    secondary_focus TEXT,
    focus_reason TEXT,
    expected_outcome TEXT,
    outcome_window_days INTEGER,
    mode_selection TEXT,
    intervention_sequence_json TEXT,
    revision_cadence TEXT,
    pressure_cadence TEXT,
    retest_schedule_json TEXT,
    subject_priority_json TEXT NOT NULL DEFAULT '[]',
    topic_priority_json TEXT NOT NULL DEFAULT '[]',
    escalation_recommendation TEXT,
    next_review_date TEXT,
    last_shift_date TEXT,
    last_shift_reason TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 10. Strategy timeline (history of shifts)
-- ============================================================================

CREATE TABLE strategy_timeline (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    shift_date TEXT NOT NULL DEFAULT (datetime('now')),
    shift_title TEXT NOT NULL,
    reason TEXT NOT NULL,
    evidence_snapshot TEXT,
    expected_result TEXT,
    actual_outcome TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_strategy_timeline_student ON strategy_timeline(student_id, shift_date DESC);

-- ============================================================================
-- 11. Parent dashboard expansion
-- ============================================================================

ALTER TABLE parent_dashboards ADD COLUMN confidence_band TEXT;
ALTER TABLE parent_dashboards ADD COLUMN consistency_band TEXT;
ALTER TABLE parent_dashboards ADD COLUMN trajectory TEXT;
ALTER TABLE parent_dashboards ADD COLUMN interpretation TEXT;
ALTER TABLE parent_dashboards ADD COLUMN intervention_intensity TEXT;
ALTER TABLE parent_dashboards ADD COLUMN strategy_summary_json TEXT;
ALTER TABLE parent_dashboards ADD COLUMN progress_highlights_json TEXT;
ALTER TABLE parent_dashboards ADD COLUMN exam_countdown_json TEXT;
ALTER TABLE parent_dashboards ADD COLUMN days_to_exam INTEGER;

-- ============================================================================
-- 12. Exam forecast center data
-- ============================================================================

ALTER TABLE parent_dashboards ADD COLUMN forecast_status TEXT;
ALTER TABLE parent_dashboards ADD COLUMN forecast_confidence TEXT;
ALTER TABLE parent_dashboards ADD COLUMN strongest_subjects_json TEXT;
ALTER TABLE parent_dashboards ADD COLUMN exposed_subjects_json TEXT;
ALTER TABLE parent_dashboards ADD COLUMN mark_loss_zones_json TEXT;
ALTER TABLE parent_dashboards ADD COLUMN intensity_recommendation TEXT;

-- ============================================================================
-- 13. Seed additional premium features
-- ============================================================================

INSERT OR IGNORE INTO premium_features (feature_key, display_name, tier_required, description) VALUES
    ('strategy_memos', 'Weekly Strategy Memos', 'premium', 'Structured weekly parent briefings with strategy changes'),
    ('milestone_reviews', 'Milestone Reviews', 'premium', 'Formal 30/60-day and pre-exam review reports'),
    ('concierge_qa', 'Concierge Q&A', 'premium', 'Parent intelligence desk for data-backed answers'),
    ('readiness_profiles', 'Readiness Profiles', 'premium', 'Persistent readiness tracking with historical trending'),
    ('premium_intake', 'White-Glove Intake', 'premium', 'Structured premium onboarding with deep child profiling'),
    ('risk_center', 'Risk Center', 'premium', 'Full risk taxonomy with trigger rules and lifecycle'),
    ('strategy_timeline', 'Strategy Timeline', 'premium', 'Visual history of preparation strategy evolution'),
    ('intervention_ledger', 'Intervention Ledger', 'premium', 'Detailed intervention lifecycle tracking'),
    ('exam_forecast', 'Exam Forecast Center', 'premium', 'Forward-looking readiness projection with scenarios'),
    ('parent_command_center', 'Parent Command Center', 'premium', 'Executive academic oversight dashboard'),
    ('communication_ledger', 'Communication Ledger', 'premium', 'Complete parent notification and briefing history');
