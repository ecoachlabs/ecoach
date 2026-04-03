CREATE TABLE IF NOT EXISTS ic_topic_evidence_points (
    evidence_id TEXT PRIMARY KEY,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    source_type TEXT NOT NULL,
    source_id INTEGER,
    timed INTEGER NOT NULL DEFAULT 0,
    transfer INTEGER NOT NULL DEFAULT 0,
    mixed_context INTEGER NOT NULL DEFAULT 0,
    delayed_recall INTEGER NOT NULL DEFAULT 0,
    retrieval INTEGER NOT NULL DEFAULT 0,
    hints_used INTEGER NOT NULL DEFAULT 0,
    correctness INTEGER NOT NULL DEFAULT 0,
    latency_ms INTEGER,
    representation_type TEXT,
    misconception_tags_json TEXT NOT NULL DEFAULT '[]',
    weight INTEGER NOT NULL DEFAULT 0,
    owner_engine_key TEXT NOT NULL DEFAULT 'topic',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(learner_id, source_type, source_id)
);

CREATE INDEX IF NOT EXISTS idx_ic_topic_evidence_topic_time
    ON ic_topic_evidence_points(learner_id, topic_id, created_at DESC);

CREATE TABLE IF NOT EXISTS ic_topic_teaching (
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    decision_id TEXT NOT NULL,
    dominant_hypothesis TEXT,
    co_causes_json TEXT NOT NULL DEFAULT '[]',
    teaching_mode TEXT,
    entry_point TEXT,
    mastery_state TEXT,
    false_mastery_score INTEGER,
    bottleneck_concept_id INTEGER,
    evidence_spine_json TEXT NOT NULL DEFAULT '{}',
    proof_contract_json TEXT NOT NULL DEFAULT '{}',
    strategy_file_json TEXT NOT NULL DEFAULT '{}',
    delayed_recall_required INTEGER NOT NULL DEFAULT 0,
    confidence_bundle_json TEXT NOT NULL DEFAULT '{}',
    owner_engine_key TEXT NOT NULL DEFAULT 'topic',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (learner_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_ic_topic_teaching_subject
    ON ic_topic_teaching(learner_id, subject_id);

CREATE TABLE IF NOT EXISTS ic_intervention_history (
    intervention_id TEXT PRIMARY KEY,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    intervention_family TEXT NOT NULL,
    outcome_state TEXT NOT NULL,
    gain_score INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    trigger_reason TEXT,
    evidence_json TEXT NOT NULL DEFAULT '{}',
    owner_engine_key TEXT NOT NULL DEFAULT 'topic',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(learner_id, topic_id, intervention_family, outcome_state)
);

CREATE INDEX IF NOT EXISTS idx_ic_intervention_history_topic_time
    ON ic_intervention_history(learner_id, topic_id, created_at DESC);

CREATE TABLE IF NOT EXISTS ic_coverage_classification (
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    bucket TEXT NOT NULL,
    base_bucket TEXT NOT NULL,
    override_applied INTEGER NOT NULL DEFAULT 0,
    gap_types_json TEXT NOT NULL DEFAULT '[]',
    session_demand INTEGER,
    review_due INTEGER NOT NULL DEFAULT 0,
    urgency_score INTEGER,
    owner_engine_key TEXT NOT NULL DEFAULT 'coverage',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (learner_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_ic_coverage_classification_subject
    ON ic_coverage_classification(learner_id, subject_id, bucket);

CREATE TABLE IF NOT EXISTS ic_coverage_snapshots (
    snapshot_id TEXT PRIMARY KEY,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    portfolio_json TEXT NOT NULL DEFAULT '{}',
    gap_map_json TEXT NOT NULL DEFAULT '{}',
    review_obligations_json TEXT NOT NULL DEFAULT '[]',
    time_budget_json TEXT NOT NULL DEFAULT '{}',
    trajectory_json TEXT NOT NULL DEFAULT '{}',
    recommendations_json TEXT NOT NULL DEFAULT '[]',
    owner_engine_key TEXT NOT NULL DEFAULT 'coverage',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_ic_coverage_snapshots_subject
    ON ic_coverage_snapshots(learner_id, subject_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS ic_sequencing_decisions (
    decision_id TEXT PRIMARY KEY,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    chosen_model TEXT NOT NULL,
    model_scores_json TEXT NOT NULL DEFAULT '{}',
    alternatives_json TEXT NOT NULL DEFAULT '[]',
    pivot_conditions_json TEXT NOT NULL DEFAULT '[]',
    stability_report_json TEXT NOT NULL DEFAULT '{}',
    lock_recommendation_json TEXT,
    owner_engine_key TEXT NOT NULL DEFAULT 'sequencing',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_ic_sequence_decisions_subject
    ON ic_sequencing_decisions(learner_id, subject_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS ic_sequencing_decision_topics (
    decision_id TEXT NOT NULL REFERENCES ic_sequencing_decisions(decision_id) ON DELETE CASCADE,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    priority_score INTEGER,
    moved_for_interference INTEGER NOT NULL DEFAULT 0,
    notes_json TEXT,
    PRIMARY KEY (decision_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_ic_sequence_topics_decision
    ON ic_sequencing_decision_topics(decision_id, position);

CREATE TABLE IF NOT EXISTS ic_timing_decisions (
    decision_id TEXT PRIMARY KEY,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    action_type TEXT NOT NULL,
    action_scope TEXT NOT NULL,
    scheduled_for TEXT,
    current_phase TEXT,
    rationale_json TEXT NOT NULL DEFAULT '{}',
    source_engine TEXT NOT NULL,
    consumed INTEGER NOT NULL DEFAULT 0,
    owner_engine_key TEXT NOT NULL DEFAULT 'timing',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_ic_timing_decisions_due
    ON ic_timing_decisions(learner_id, subject_id, scheduled_for, action_type);

CREATE INDEX IF NOT EXISTS idx_ic_timing_decisions_topic
    ON ic_timing_decisions(learner_id, topic_id, action_type);

CREATE TABLE IF NOT EXISTS ic_risk_assessments (
    assessment_id TEXT PRIMARY KEY,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    scope TEXT NOT NULL,
    risk_code TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    risk_score INTEGER NOT NULL DEFAULT 0,
    protection_policy_json TEXT NOT NULL DEFAULT '{}',
    rationale_json TEXT NOT NULL DEFAULT '{}',
    owner_engine_key TEXT NOT NULL DEFAULT 'risk',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_ic_risk_scope_subject
    ON ic_risk_assessments(learner_id, subject_id, scope, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_ic_risk_topic
    ON ic_risk_assessments(learner_id, topic_id, scope, updated_at DESC);

CREATE TABLE IF NOT EXISTS ic_adaptation_log (
    adaptation_id TEXT PRIMARY KEY,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    mode TEXT NOT NULL,
    trigger_reason TEXT NOT NULL,
    what_changed_json TEXT NOT NULL DEFAULT '[]',
    previous_strategy_json TEXT NOT NULL DEFAULT '{}',
    new_strategy_json TEXT NOT NULL DEFAULT '{}',
    tension_at_time_json TEXT NOT NULL DEFAULT '{}',
    owner_engine_key TEXT NOT NULL DEFAULT 'adaptation',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_ic_adaptation_log_subject
    ON ic_adaptation_log(learner_id, subject_id, created_at DESC);

CREATE TABLE IF NOT EXISTS ic_learner_state (
    learner_id INTEGER PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    readiness_state TEXT NOT NULL,
    preferred_families_json TEXT NOT NULL DEFAULT '[]',
    avoided_families_json TEXT NOT NULL DEFAULT '[]',
    avg_successful_session_minutes INTEGER,
    pressure_response_json TEXT NOT NULL DEFAULT '{}',
    learns_from_mistakes_score INTEGER,
    confidence_scaffolding_need INTEGER,
    mode_preferences_json TEXT NOT NULL DEFAULT '{}',
    evidence_confidence_json TEXT NOT NULL DEFAULT '{}',
    owner_engine_key TEXT NOT NULL DEFAULT 'adaptation',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_ic_learner_state_updated
    ON ic_learner_state(updated_at DESC);

CREATE TABLE IF NOT EXISTS ic_engine_cycle_audit (
    cycle_id TEXT PRIMARY KEY,
    trace_id TEXT NOT NULL,
    trigger TEXT NOT NULL,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    engines_run_json TEXT NOT NULL DEFAULT '[]',
    changed_engines_json TEXT NOT NULL DEFAULT '[]',
    conflicts_resolved_json TEXT NOT NULL DEFAULT '[]',
    planner_summary_json TEXT,
    owner_engine_key TEXT NOT NULL DEFAULT 'constitution',
    started_at TEXT NOT NULL,
    finished_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ic_outbox_events (
    outbox_id TEXT PRIMARY KEY,
    cycle_id TEXT NOT NULL REFERENCES ic_engine_cycle_audit(cycle_id) ON DELETE CASCADE,
    emitted_by TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    owner_engine_key TEXT NOT NULL DEFAULT 'constitution',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    delivered_at TEXT
);

CREATE TABLE IF NOT EXISTS ic_snapshot_invalidation (
    invalidation_id TEXT PRIMARY KEY,
    learner_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    snapshot_kind TEXT NOT NULL,
    reason TEXT NOT NULL,
    owner_engine_key TEXT NOT NULL DEFAULT 'constitution',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);
