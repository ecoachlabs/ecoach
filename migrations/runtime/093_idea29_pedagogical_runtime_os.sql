-- idea29: canonical pedagogical runtime operating system
-- Topic teaching profiles, learning units, instructional objects,
-- learner-state planes, personalization snapshots, and turn-by-turn runtime.

CREATE TABLE IF NOT EXISTS learning_units (
    id INTEGER PRIMARY KEY,
    unit_key TEXT NOT NULL UNIQUE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    node_id INTEGER REFERENCES academic_nodes(id),
    title TEXT NOT NULL,
    content_type_primary TEXT NOT NULL,
    content_type_secondary_json TEXT NOT NULL DEFAULT '[]',
    representation_tags_json TEXT NOT NULL DEFAULT '[]',
    prerequisite_links_json TEXT NOT NULL DEFAULT '[]',
    mastery_evidence_json TEXT NOT NULL DEFAULT '[]',
    review_modes_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_learning_units_topic ON learning_units(topic_id);
CREATE INDEX IF NOT EXISTS idx_learning_units_subject ON learning_units(subject_id);

CREATE TABLE IF NOT EXISTS topic_teaching_profiles (
    id INTEGER PRIMARY KEY,
    topic_id INTEGER NOT NULL REFERENCES topics(id) UNIQUE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_name TEXT NOT NULL,
    primary_content_type TEXT NOT NULL,
    secondary_content_types_json TEXT NOT NULL DEFAULT '[]',
    representation_modes_json TEXT NOT NULL DEFAULT '[]',
    strategy_families_json TEXT NOT NULL DEFAULT '[]',
    drill_families_json TEXT NOT NULL DEFAULT '[]',
    mastery_evidence_json TEXT NOT NULL DEFAULT '[]',
    failure_signatures_json TEXT NOT NULL DEFAULT '[]',
    review_modes_json TEXT NOT NULL DEFAULT '[]',
    prerequisite_topic_ids_json TEXT NOT NULL DEFAULT '[]',
    learning_unit_count INTEGER NOT NULL DEFAULT 0,
    instructional_object_count INTEGER NOT NULL DEFAULT 0,
    freshness_score_bp INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_topic_teaching_profiles_subject
    ON topic_teaching_profiles(subject_id);

CREATE TABLE IF NOT EXISTS instructional_objects (
    id INTEGER PRIMARY KEY,
    object_key TEXT NOT NULL UNIQUE,
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    learning_unit_id INTEGER REFERENCES learning_units(id),
    object_type TEXT NOT NULL
        CHECK (object_type IN (
            'explanation_card', 'diagnostic_probe', 'repair_bundle', 'review_card',
            'worked_example', 'contrast_card', 'timed_burst', 'summary_card'
        )),
    pedagogical_purpose TEXT NOT NULL
        CHECK (pedagogical_purpose IN (
            'teach', 'clarify', 'retrieve', 'probe', 'contrast', 'repair',
            'transfer', 'integrate', 'assess', 'perform', 'summarize', 'reflect'
        )),
    title TEXT NOT NULL,
    content_type_primary TEXT NOT NULL,
    representation_mode TEXT,
    response_mode TEXT,
    strategy_families_json TEXT NOT NULL DEFAULT '[]',
    drill_families_json TEXT NOT NULL DEFAULT '[]',
    mastery_evidence_json TEXT NOT NULL DEFAULT '[]',
    supported_failure_signatures_json TEXT NOT NULL DEFAULT '[]',
    difficulty_bp INTEGER NOT NULL DEFAULT 5000,
    quality_score_bp INTEGER NOT NULL DEFAULT 5000,
    effectiveness_score_bp INTEGER NOT NULL DEFAULT 5000,
    source_ref TEXT,
    payload_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_instructional_objects_topic
    ON instructional_objects(topic_id, pedagogical_purpose);
CREATE INDEX IF NOT EXISTS idx_instructional_objects_unit
    ON instructional_objects(learning_unit_id);

CREATE TABLE IF NOT EXISTS learner_unit_states (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    learning_unit_id INTEGER NOT NULL REFERENCES learning_units(id),
    scope_key TEXT NOT NULL,
    presence_state TEXT NOT NULL DEFAULT 'missing',
    clarity_state TEXT NOT NULL DEFAULT 'boundary_blurred',
    retrieval_state TEXT NOT NULL DEFAULT 'cue_dependent',
    execution_state TEXT NOT NULL DEFAULT 'sequence_fractured',
    transfer_state TEXT NOT NULL DEFAULT 'surface_locked',
    performance_state TEXT NOT NULL DEFAULT 'pressure_sensitive',
    diagnostic_confidence_bp INTEGER NOT NULL DEFAULT 0,
    recent_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    delayed_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    mixed_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    timed_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    latency_score_bp INTEGER NOT NULL DEFAULT 0,
    hint_dependence_bp INTEGER NOT NULL DEFAULT 0,
    confidence_alignment_bp INTEGER NOT NULL DEFAULT 0,
    decay_risk_bp INTEGER NOT NULL DEFAULT 0,
    dominant_failure_signature TEXT,
    confusion_neighbors_json TEXT NOT NULL DEFAULT '[]',
    misconception_flags_json TEXT NOT NULL DEFAULT '[]',
    preferred_strategy_families_json TEXT NOT NULL DEFAULT '[]',
    failed_strategy_families_json TEXT NOT NULL DEFAULT '[]',
    last_review_mode TEXT,
    next_review_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, learning_unit_id, scope_key)
);

CREATE INDEX IF NOT EXISTS idx_learner_unit_states_student
    ON learner_unit_states(student_id, scope_key);
CREATE INDEX IF NOT EXISTS idx_learner_unit_states_unit
    ON learner_unit_states(learning_unit_id);

CREATE TABLE IF NOT EXISTS personalization_snapshots (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    scope_key TEXT NOT NULL UNIQUE,
    observed_profile_json TEXT NOT NULL DEFAULT '{}',
    derived_profile_json TEXT NOT NULL DEFAULT '{}',
    inferred_profile_json TEXT NOT NULL DEFAULT '{}',
    strategic_control_json TEXT NOT NULL DEFAULT '{}',
    recommendation_json TEXT NOT NULL DEFAULT '{}',
    confidence_score_bp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_personalization_snapshots_lookup
    ON personalization_snapshots(student_id, subject_id, topic_id);

CREATE TABLE IF NOT EXISTS runtime_teaching_turns (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER REFERENCES topics(id),
    learning_unit_id INTEGER REFERENCES learning_units(id),
    turn_index INTEGER NOT NULL,
    move_type TEXT NOT NULL,
    instructional_intention TEXT NOT NULL,
    success_condition TEXT NOT NULL,
    diagnostic_focus TEXT,
    support_level TEXT NOT NULL DEFAULT 'guided',
    pressure_level TEXT NOT NULL DEFAULT 'calm',
    representation_mode TEXT,
    selected_object_id INTEGER REFERENCES instructional_objects(id),
    selected_review_episode_id INTEGER REFERENCES review_episodes(id),
    local_state_json TEXT NOT NULL DEFAULT '{}',
    outcome_status TEXT,
    outcome_score_bp INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    UNIQUE(session_id, turn_index)
);

CREATE INDEX IF NOT EXISTS idx_runtime_teaching_turns_session
    ON runtime_teaching_turns(session_id, turn_index);

CREATE TABLE IF NOT EXISTS teaching_move_feedback (
    id INTEGER PRIMARY KEY,
    turn_id INTEGER REFERENCES runtime_teaching_turns(id) ON DELETE SET NULL,
    instructional_object_id INTEGER REFERENCES instructional_objects(id) ON DELETE SET NULL,
    review_episode_id INTEGER REFERENCES review_episodes(id) ON DELETE SET NULL,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_id INTEGER REFERENCES sessions(id) ON DELETE CASCADE,
    feedback_source TEXT NOT NULL
        CHECK (feedback_source IN ('attempt', 'resource_learning', 'session_close', 'manual')),
    move_type TEXT,
    learning_delta_bp INTEGER NOT NULL DEFAULT 0,
    retention_delta_bp INTEGER NOT NULL DEFAULT 0,
    transfer_delta_bp INTEGER NOT NULL DEFAULT 0,
    pressure_delta_bp INTEGER NOT NULL DEFAULT 0,
    confidence_delta_bp INTEGER NOT NULL DEFAULT 0,
    effectiveness_label TEXT,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_teaching_move_feedback_student
    ON teaching_move_feedback(student_id, feedback_source);
CREATE INDEX IF NOT EXISTS idx_teaching_move_feedback_turn
    ON teaching_move_feedback(turn_id);

ALTER TABLE review_episodes ADD COLUMN learning_unit_id INTEGER REFERENCES learning_units(id);
ALTER TABLE review_episodes ADD COLUMN failure_code TEXT REFERENCES failure_cause_taxonomy(failure_code);
ALTER TABLE review_episodes ADD COLUMN intervention_family TEXT;
ALTER TABLE review_episodes ADD COLUMN evidence_strength_bp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE review_episodes ADD COLUMN outcome_summary_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE review_episodes ADD COLUMN next_review_at TEXT;

CREATE INDEX IF NOT EXISTS idx_review_episodes_learning_unit
    ON review_episodes(learning_unit_id, status);
