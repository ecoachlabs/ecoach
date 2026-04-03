-- idea21: objective-driven academic resource orchestration runtime,
-- applicability confirmation, and outcome learning.

CREATE TABLE IF NOT EXISTS resource_orchestration_runs (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    objective_code TEXT NOT NULL,
    session_mode TEXT NOT NULL DEFAULT 'adaptive',
    request_text TEXT,
    request_payload_json TEXT NOT NULL DEFAULT '{}',
    interpreted_objective_json TEXT NOT NULL DEFAULT '{}',
    curriculum_context_json TEXT NOT NULL DEFAULT '{}',
    recipe_json TEXT NOT NULL DEFAULT '[]',
    ambiguity_status TEXT NOT NULL DEFAULT 'clear'
        CHECK (ambiguity_status IN ('clear', 'needs_confirmation', 'resolved')),
    run_status TEXT NOT NULL DEFAULT 'composed'
        CHECK (run_status IN ('composed', 'needs_confirmation', 'evaluated', 'archived')),
    confidence_score_bp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_resource_runs_student
    ON resource_orchestration_runs(student_id, created_at DESC);

CREATE TABLE IF NOT EXISTS resource_orchestration_candidates (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES resource_orchestration_runs(id) ON DELETE CASCADE,
    resource_id INTEGER,
    resource_type TEXT NOT NULL,
    role_code TEXT,
    title TEXT NOT NULL,
    selection_rank INTEGER NOT NULL DEFAULT 0,
    selected INTEGER NOT NULL DEFAULT 0,
    generated INTEGER NOT NULL DEFAULT 0,
    intrinsic_quality_bp INTEGER NOT NULL DEFAULT 0,
    contextual_fitness_bp INTEGER NOT NULL DEFAULT 0,
    overall_score_bp INTEGER NOT NULL DEFAULT 0,
    confidence_tier TEXT,
    rationale_json TEXT NOT NULL DEFAULT '[]',
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_resource_candidates_run
    ON resource_orchestration_candidates(run_id, selected DESC, overall_score_bp DESC);

CREATE TABLE IF NOT EXISTS resource_applicability_checks (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES resource_orchestration_runs(id) ON DELETE CASCADE,
    prompt_text TEXT NOT NULL,
    options_json TEXT NOT NULL DEFAULT '[]',
    selected_option_code TEXT,
    selected_option_label TEXT,
    response_text TEXT,
    resolved_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_resource_checks_run
    ON resource_applicability_checks(run_id, created_at DESC);

CREATE TABLE IF NOT EXISTS resource_learning_events (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES resource_orchestration_runs(id) ON DELETE CASCADE,
    session_id INTEGER REFERENCES sessions(id),
    outcome_status TEXT NOT NULL DEFAULT 'observed'
        CHECK (outcome_status IN ('observed', 'improved', 'partial', 'failed')),
    usefulness_bp INTEGER NOT NULL DEFAULT 0,
    confidence_shift_bp INTEGER NOT NULL DEFAULT 0,
    speed_shift_bp INTEGER NOT NULL DEFAULT 0,
    accuracy_shift_bp INTEGER NOT NULL DEFAULT 0,
    learner_feedback_json TEXT NOT NULL DEFAULT '{}',
    still_struggling_json TEXT NOT NULL DEFAULT '[]',
    effective_resource_refs_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_resource_learning_run
    ON resource_learning_events(run_id, created_at DESC);
