-- idea18 runtime completion: richer diagnostic telemetry, skill analytics,
-- recommendation persistence, and audience-facing reporting payloads.

ALTER TABLE diagnostic_item_attempts ADD COLUMN first_focus_at TEXT;
ALTER TABLE diagnostic_item_attempts ADD COLUMN first_input_at TEXT;
ALTER TABLE diagnostic_item_attempts ADD COLUMN concept_guess TEXT;
ALTER TABLE diagnostic_item_attempts ADD COLUMN final_answer_json TEXT;
ALTER TABLE diagnostic_item_attempts ADD COLUMN raw_interaction_log_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE diagnostic_item_attempts ADD COLUMN hesitation_flags_json TEXT NOT NULL DEFAULT '{}';

CREATE TABLE diagnostic_skill_results (
    id INTEGER PRIMARY KEY,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    skill_kind TEXT NOT NULL
        CHECK (skill_kind IN ('micro_skill', 'academic_node', 'derived')),
    skill_id INTEGER,
    skill_key TEXT NOT NULL,
    skill_name TEXT NOT NULL,
    skill_type TEXT NOT NULL,
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    baseline_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    precision_score INTEGER NOT NULL DEFAULT 0,
    pressure_score INTEGER NOT NULL DEFAULT 0,
    flex_score INTEGER NOT NULL DEFAULT 0,
    root_cause_score INTEGER NOT NULL DEFAULT 0,
    endurance_score INTEGER NOT NULL DEFAULT 0,
    recovery_score INTEGER NOT NULL DEFAULT 0,
    mastery_score INTEGER NOT NULL DEFAULT 0,
    fragility_index INTEGER NOT NULL DEFAULT 0,
    pressure_collapse_index INTEGER NOT NULL DEFAULT 0,
    recognition_gap_index INTEGER NOT NULL DEFAULT 0,
    formula_recall_use_delta INTEGER NOT NULL DEFAULT 0,
    stability_score INTEGER NOT NULL DEFAULT 0,
    mastery_state TEXT NOT NULL,
    weakness_type_primary TEXT NOT NULL,
    weakness_type_secondary TEXT,
    recommended_intervention TEXT NOT NULL,
    evidence_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(diagnostic_id, skill_key)
);

CREATE INDEX idx_diagnostic_skill_results_diagnostic
    ON diagnostic_skill_results(diagnostic_id, topic_id, mastery_score);
CREATE INDEX idx_diagnostic_skill_results_student
    ON diagnostic_skill_results(student_id, topic_id);

CREATE TABLE diagnostic_recommendations (
    id INTEGER PRIMARY KEY,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    category TEXT NOT NULL,
    action_code TEXT NOT NULL,
    title TEXT NOT NULL,
    rationale TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    target_kind TEXT,
    target_ref TEXT,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_diagnostic_recommendations_diagnostic
    ON diagnostic_recommendations(diagnostic_id, priority DESC, id ASC);

CREATE TABLE diagnostic_audience_reports (
    id INTEGER PRIMARY KEY,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    audience TEXT NOT NULL
        CHECK (audience IN ('student', 'teacher', 'parent')),
    headline TEXT NOT NULL,
    narrative TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(diagnostic_id, audience)
);

CREATE INDEX idx_diagnostic_audience_reports_diagnostic
    ON diagnostic_audience_reports(diagnostic_id, audience);
