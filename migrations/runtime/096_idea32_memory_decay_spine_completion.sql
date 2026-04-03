-- idea32: Complete the memory-decay spine with first-class knowledge units,
-- retrieval attempts, student knowledge states, interventions, persisted
-- review scheduling, interference analytics, pressure profiles, and job logs.

CREATE TABLE IF NOT EXISTS knowledge_units (
    id INTEGER PRIMARY KEY,
    node_id INTEGER,
    subject_id INTEGER,
    topic_id INTEGER,
    subtopic_id INTEGER,
    title TEXT NOT NULL,
    canonical_label TEXT NOT NULL,
    description TEXT,
    unit_type TEXT NOT NULL DEFAULT 'concept',
    difficulty_bp INTEGER NOT NULL DEFAULT 5000,
    importance_weight_bp INTEGER NOT NULL DEFAULT 5000,
    dependency_weight_bp INTEGER NOT NULL DEFAULT 5000,
    confusion_proneness_bp INTEGER NOT NULL DEFAULT 0,
    exam_frequency_weight_bp INTEGER NOT NULL DEFAULT 5000,
    canonical_representations_json TEXT NOT NULL DEFAULT '{}',
    tags_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(node_id),
    UNIQUE(topic_id, canonical_label)
);

CREATE INDEX IF NOT EXISTS idx_knowledge_units_topic ON knowledge_units(topic_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_units_subject ON knowledge_units(subject_id);

CREATE TABLE IF NOT EXISTS knowledge_unit_edges (
    id INTEGER PRIMARY KEY,
    source_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    target_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    edge_type TEXT NOT NULL,
    weight_bp INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_unit_id, target_unit_id, edge_type)
);

CREATE INDEX IF NOT EXISTS idx_knowledge_unit_edges_source
ON knowledge_unit_edges(source_unit_id, edge_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_unit_edges_target
ON knowledge_unit_edges(target_unit_id, edge_type);

CREATE TABLE IF NOT EXISTS student_knowledge_states (
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    node_id INTEGER,
    topic_id INTEGER,
    memory_state TEXT NOT NULL DEFAULT 'unexposed'
        CHECK (memory_state IN (
            'unexposed', 'exposed', 'familiar', 'recognizable',
            'cued_recallable', 'free_recallable', 'applicable',
            'transferable', 'pressure_stable', 'durable'
        )),
    state_confidence_bp INTEGER NOT NULL DEFAULT 0,
    state_updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    decay_status TEXT NOT NULL DEFAULT 'stable'
        CHECK (decay_status IN ('stable', 'watchlist', 'fragile', 'decaying', 'collapsed')),
    decay_risk_score INTEGER NOT NULL DEFAULT 0,
    recall_profile_json TEXT NOT NULL DEFAULT '{}',
    support_dependency_score INTEGER NOT NULL DEFAULT 0,
    confidence_calibration_score INTEGER NOT NULL DEFAULT 0,
    latency_score INTEGER NOT NULL DEFAULT 0,
    resilience_score INTEGER NOT NULL DEFAULT 0,
    primary_failure_mode TEXT,
    secondary_failure_mode TEXT,
    interference_risk_score INTEGER NOT NULL DEFAULT 0,
    downstream_risk_score INTEGER NOT NULL DEFAULT 0,
    exposure_count INTEGER NOT NULL DEFAULT 0,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    last_seen_at TEXT,
    last_attempt_at TEXT,
    last_success_at TEXT,
    last_free_recall_success_at TEXT,
    last_application_success_at TEXT,
    last_pressure_success_at TEXT,
    current_intervention_plan_id INTEGER,
    next_review_at TEXT,
    review_urgency_score INTEGER,
    flags_json TEXT NOT NULL DEFAULT '[]',
    explanation_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(student_id, knowledge_unit_id)
);

CREATE INDEX IF NOT EXISTS idx_sks_next_review_at ON student_knowledge_states(next_review_at);
CREATE INDEX IF NOT EXISTS idx_sks_decay_risk ON student_knowledge_states(decay_risk_score);
CREATE INDEX IF NOT EXISTS idx_sks_student_review
ON student_knowledge_states(student_id, next_review_at);
CREATE INDEX IF NOT EXISTS idx_sks_student_state
ON student_knowledge_states(student_id, memory_state);

CREATE TABLE IF NOT EXISTS retrieval_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    session_id INTEGER,
    question_id INTEGER,
    mode TEXT NOT NULL
        CHECK (mode IN (
            'recognition', 'cued_recall', 'free_recall',
            'application', 'transfer', 'pressure'
        )),
    format TEXT NOT NULL
        CHECK (format IN (
            'mcq', 'typed_short', 'typed_long', 'oral', 'drag_sort',
            'match', 'step_order', 'worked_solution',
            'reverse_identification', 'unknown'
        )),
    timed INTEGER NOT NULL DEFAULT 0,
    time_limit_ms INTEGER,
    response_time_ms INTEGER,
    first_commit_time_ms INTEGER,
    correctness TEXT NOT NULL
        CHECK (correctness IN ('correct', 'partially_correct', 'incorrect')),
    raw_score_bp INTEGER NOT NULL DEFAULT 0,
    confidence_self_report_bp INTEGER,
    hints_used INTEGER NOT NULL DEFAULT 0,
    hint_strength_bp INTEGER,
    options_visible INTEGER NOT NULL DEFAULT 0,
    formula_bank_visible INTEGER NOT NULL DEFAULT 0,
    answer_text TEXT,
    expected_node_id INTEGER,
    intruding_node_id INTEGER,
    switched_answer INTEGER NOT NULL DEFAULT 0,
    guess_likelihood_bp INTEGER,
    freeze_marker INTEGER NOT NULL DEFAULT 0,
    hesitation_score_bp INTEGER,
    derived_tags_json TEXT NOT NULL DEFAULT '[]',
    attempt_key TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(attempt_key)
);

CREATE INDEX IF NOT EXISTS idx_ra_student_unit_time
ON retrieval_attempts(student_id, knowledge_unit_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ra_session ON retrieval_attempts(session_id);
CREATE INDEX IF NOT EXISTS idx_ra_question ON retrieval_attempts(question_id);
CREATE INDEX IF NOT EXISTS idx_ra_timed
ON retrieval_attempts(student_id, timed, created_at DESC);

ALTER TABLE interference_edges ADD COLUMN student_id INTEGER;
ALTER TABLE interference_edges ADD COLUMN source_knowledge_unit_id INTEGER;
ALTER TABLE interference_edges ADD COLUMN target_knowledge_unit_id INTEGER;
ALTER TABLE interference_edges ADD COLUMN confusion_strength INTEGER NOT NULL DEFAULT 0;
ALTER TABLE interference_edges ADD COLUMN directionality TEXT NOT NULL DEFAULT 'source_to_target';
ALTER TABLE interference_edges ADD COLUMN timed_confusion_strength INTEGER NOT NULL DEFAULT 0;
ALTER TABLE interference_edges ADD COLUMN calm_confusion_strength INTEGER NOT NULL DEFAULT 0;
ALTER TABLE interference_edges ADD COLUMN total_confusions INTEGER NOT NULL DEFAULT 0;
ALTER TABLE interference_edges ADD COLUMN context_tags_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE interference_edges ADD COLUMN status TEXT NOT NULL DEFAULT 'watchlist';
ALTER TABLE interference_edges ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));

CREATE UNIQUE INDEX IF NOT EXISTS idx_interference_student_units
ON interference_edges(student_id, source_knowledge_unit_id, target_knowledge_unit_id);
CREATE INDEX IF NOT EXISTS idx_interference_student_status
ON interference_edges(student_id, status, confusion_strength DESC);

CREATE TABLE IF NOT EXISTS interference_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    source_knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    target_knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS contrast_pair_status (
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    source_knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    target_knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    status TEXT NOT NULL DEFAULT 'unseen'
        CHECK (status IN ('unseen', 'watchlist', 'active', 'stabilized')),
    last_drill_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(student_id, source_knowledge_unit_id, target_knowledge_unit_id)
);

CREATE TABLE IF NOT EXISTS intervention_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    family TEXT NOT NULL,
    reason TEXT NOT NULL,
    primary_failure_mode TEXT NOT NULL,
    target_state TEXT NOT NULL,
    steps_json TEXT NOT NULL,
    retest_plan_json TEXT NOT NULL,
    estimated_difficulty_bp INTEGER NOT NULL DEFAULT 5000,
    estimated_duration_min INTEGER NOT NULL DEFAULT 5,
    priority_score INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'abandoned')),
    completed_step_count INTEGER NOT NULL DEFAULT 0,
    total_step_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_intervention_student_status
ON intervention_plans(student_id, status, priority_score DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_intervention_active_unit
ON intervention_plans(student_id, knowledge_unit_id)
WHERE status IN ('pending', 'active');

CREATE TABLE IF NOT EXISTS intervention_step_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id INTEGER NOT NULL REFERENCES intervention_plans(id),
    step_code TEXT NOT NULL,
    outcome TEXT NOT NULL,
    successful INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS intervention_outcomes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id INTEGER NOT NULL REFERENCES intervention_plans(id),
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    outcome TEXT NOT NULL,
    success_flag INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS intervention_history_rollups (
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    total_plans INTEGER NOT NULL DEFAULT 0,
    successful_plans INTEGER NOT NULL DEFAULT 0,
    failed_plans INTEGER NOT NULL DEFAULT 0,
    last_outcome TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(student_id, knowledge_unit_id)
);

CREATE TABLE IF NOT EXISTS review_schedule_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    due_at TEXT NOT NULL,
    urgency_score INTEGER NOT NULL,
    recommended_mode TEXT NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled'
        CHECK (status IN ('scheduled', 'done', 'missed', 'rescheduled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_review_due
ON review_schedule_items(student_id, due_at, status);
CREATE INDEX IF NOT EXISTS idx_review_urgency
ON review_schedule_items(student_id, urgency_score DESC);

CREATE TABLE IF NOT EXISTS knowledge_state_transitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    from_state TEXT NOT NULL,
    to_state TEXT NOT NULL,
    reason TEXT NOT NULL,
    evidence_snapshot_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS memory_engine_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    student_id INTEGER,
    knowledge_unit_id INTEGER,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_memory_engine_events_student
ON memory_engine_events(student_id, created_at DESC);

CREATE TABLE IF NOT EXISTS memory_engine_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_type TEXT NOT NULL
        CHECK (job_type IN (
            'recompute_knowledge_state', 'plan_intervention',
            'schedule_review', 'nightly_decay_scan',
            'interference_graph_decay'
        )),
    student_id INTEGER,
    knowledge_unit_id INTEGER,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'completed', 'failed')),
    payload_json TEXT NOT NULL DEFAULT '{}',
    idempotency_key TEXT,
    error_message TEXT,
    scheduled_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    finished_at TEXT,
    UNIQUE(job_type, idempotency_key)
);

CREATE TABLE IF NOT EXISTS pressure_profiles (
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    calm_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    timed_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    pressure_gap_score INTEGER NOT NULL DEFAULT 0,
    switch_risk_score INTEGER NOT NULL DEFAULT 0,
    freeze_risk_score INTEGER NOT NULL DEFAULT 0,
    pressure_state TEXT NOT NULL DEFAULT 'neutral'
        CHECK (pressure_state IN ('neutral', 'watchlist', 'vulnerable', 'stable')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY(student_id, knowledge_unit_id)
);

CREATE TABLE IF NOT EXISTS pressure_attempt_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    retrieval_attempt_id INTEGER NOT NULL REFERENCES retrieval_attempts(id),
    timed INTEGER NOT NULL DEFAULT 0,
    pressure_gap_score INTEGER NOT NULL DEFAULT 0,
    switch_risk_score INTEGER NOT NULL DEFAULT 0,
    freeze_risk_score INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS pressure_transition_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    knowledge_unit_id INTEGER NOT NULL REFERENCES knowledge_units(id),
    from_state TEXT NOT NULL,
    to_state TEXT NOT NULL,
    pressure_gap_score INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
