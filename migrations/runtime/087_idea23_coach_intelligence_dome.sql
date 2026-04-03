CREATE TABLE IF NOT EXISTS coach_intent_goal_register (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    goal_code TEXT NOT NULL,
    goal_label TEXT NOT NULL,
    target_state TEXT NOT NULL,
    current_score INTEGER NOT NULL DEFAULT 0,
    target_score INTEGER NOT NULL DEFAULT 10000,
    tension_score INTEGER NOT NULL DEFAULT 0,
    evidence_confidence INTEGER NOT NULL DEFAULT 0,
    urgency_rank INTEGER NOT NULL DEFAULT 0,
    priority_rank INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    details_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, goal_code)
);

CREATE INDEX IF NOT EXISTS idx_coach_intent_goal_register_student
    ON coach_intent_goal_register(student_id, priority_rank);

CREATE TABLE IF NOT EXISTS coach_tension_map (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    tension_code TEXT NOT NULL,
    tension_label TEXT NOT NULL,
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER,
    severity_score INTEGER NOT NULL DEFAULT 0,
    desired_state TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    evidence_summary TEXT NOT NULL,
    recommended_response TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, tension_code, topic_key)
);

CREATE INDEX IF NOT EXISTS idx_coach_tension_map_student
    ON coach_tension_map(student_id, severity_score DESC);

CREATE TABLE IF NOT EXISTS coach_system_health_snapshots (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    plan_credibility_score INTEGER NOT NULL DEFAULT 0,
    uncertainty_score INTEGER NOT NULL DEFAULT 0,
    intervention_effectiveness_score INTEGER NOT NULL DEFAULT 0,
    recovery_readiness_score INTEGER NOT NULL DEFAULT 0,
    relational_stability_score INTEGER NOT NULL DEFAULT 0,
    motivation_score INTEGER NOT NULL DEFAULT 0,
    resilience_score INTEGER NOT NULL DEFAULT 0,
    snapshot_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_coach_system_health_snapshots_student
    ON coach_system_health_snapshots(student_id, created_at DESC);

CREATE TABLE IF NOT EXISTS coach_intervention_effectiveness (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER,
    intervention_family TEXT NOT NULL,
    times_used INTEGER NOT NULL DEFAULT 0,
    success_rate_score INTEGER NOT NULL DEFAULT 0,
    avg_gain_score INTEGER NOT NULL DEFAULT 0,
    last_outcome TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_key, intervention_family)
);

CREATE INDEX IF NOT EXISTS idx_coach_intervention_effectiveness_student
    ON coach_intervention_effectiveness(student_id, success_rate_score DESC);

CREATE TABLE IF NOT EXISTS coach_topic_strategies (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    strategy_mode TEXT NOT NULL,
    teaching_modes_json TEXT NOT NULL DEFAULT '[]',
    concept_rank_json TEXT NOT NULL DEFAULT '[]',
    fallback_route_json TEXT NOT NULL DEFAULT '[]',
    primary_hypothesis_code TEXT,
    plan_confidence_score INTEGER NOT NULL DEFAULT 0,
    explanation_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_coach_topic_strategies_student
    ON coach_topic_strategies(student_id, topic_id);

CREATE TABLE IF NOT EXISTS coach_uncertainty_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER,
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER,
    uncertainty_score INTEGER NOT NULL DEFAULT 0,
    false_mastery_risk INTEGER NOT NULL DEFAULT 0,
    information_gain_score INTEGER NOT NULL DEFAULT 0,
    evidence_needed_json TEXT NOT NULL DEFAULT '[]',
    counterfactuals_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_key, topic_key)
);

CREATE INDEX IF NOT EXISTS idx_coach_uncertainty_profiles_student
    ON coach_uncertainty_profiles(student_id, uncertainty_score DESC);

CREATE TABLE IF NOT EXISTS concept_interference_graph (
    id INTEGER PRIMARY KEY,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_a_id INTEGER NOT NULL REFERENCES topics(id),
    topic_b_id INTEGER NOT NULL REFERENCES topics(id),
    edge_type TEXT NOT NULL,
    confusion_risk_score INTEGER NOT NULL DEFAULT 0,
    pressure_amplifier_score INTEGER NOT NULL DEFAULT 0,
    spacing_recommendation_days INTEGER NOT NULL DEFAULT 0,
    best_response_mode TEXT NOT NULL,
    evidence_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(subject_id, topic_a_id, topic_b_id, edge_type)
);

CREATE INDEX IF NOT EXISTS idx_concept_interference_graph_subject
    ON concept_interference_graph(subject_id, confusion_risk_score DESC);

CREATE TABLE IF NOT EXISTS learner_interference_cases (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_a_id INTEGER NOT NULL REFERENCES topics(id),
    topic_b_id INTEGER NOT NULL REFERENCES topics(id),
    interference_type TEXT NOT NULL,
    severity_score INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    response_mode TEXT NOT NULL,
    regression_audit_due INTEGER NOT NULL DEFAULT 0,
    evidence_summary TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_a_id, topic_b_id, interference_type)
);

CREATE INDEX IF NOT EXISTS idx_learner_interference_cases_student
    ON learner_interference_cases(student_id, severity_score DESC);

CREATE TABLE IF NOT EXISTS coach_surprise_event_runs (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    event_code TEXT NOT NULL,
    event_label TEXT NOT NULL,
    purpose TEXT NOT NULL,
    readiness_state TEXT NOT NULL,
    resilience_score INTEGER NOT NULL DEFAULT 0,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_coach_surprise_event_runs_student
    ON coach_surprise_event_runs(student_id, created_at DESC);

CREATE TABLE IF NOT EXISTS coach_reflection_cycles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER,
    cycle_stage TEXT NOT NULL,
    prior_strategy TEXT,
    outcome_signal TEXT,
    revision_reason TEXT NOT NULL,
    reopened_case INTEGER NOT NULL DEFAULT 0,
    follow_up_action TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_coach_reflection_cycles_student
    ON coach_reflection_cycles(student_id, created_at DESC);
