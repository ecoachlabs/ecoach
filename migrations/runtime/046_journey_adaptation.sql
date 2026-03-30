-- Journey adaptation: route modes, deadline pressure, consistency tracking,
-- session composition, question intent, knowledge map heat state.

-- Route mode column on journey_routes
ALTER TABLE journey_routes ADD COLUMN route_mode TEXT NOT NULL DEFAULT 'balanced';
ALTER TABLE journey_routes ADD COLUMN deadline_pressure_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE journey_routes ADD COLUMN exam_date TEXT;
ALTER TABLE journey_routes ADD COLUMN daily_budget_minutes INTEGER NOT NULL DEFAULT 30;

-- Expanded station types
ALTER TABLE journey_stations ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;

-- Study consistency tracking
CREATE TABLE study_consistency (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    study_date TEXT NOT NULL,
    sessions_completed INTEGER NOT NULL DEFAULT 0,
    total_minutes INTEGER NOT NULL DEFAULT 0,
    questions_answered INTEGER NOT NULL DEFAULT 0,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, study_date)
);

CREATE INDEX idx_study_consistency_student ON study_consistency(student_id, subject_id);
CREATE INDEX idx_study_consistency_date ON study_consistency(study_date);

-- Session composition: what modes were mixed in a session
CREATE TABLE session_composition (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES sessions(id),
    segment_order INTEGER NOT NULL,
    segment_mode TEXT NOT NULL,
    segment_label TEXT,
    question_count INTEGER NOT NULL DEFAULT 0,
    duration_minutes INTEGER NOT NULL DEFAULT 0,
    intent_profile_json TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_session_composition_session ON session_composition(session_id);

-- Question intent: why each question was selected
ALTER TABLE session_items ADD COLUMN question_intent TEXT;

-- Knowledge map node heat (per student per topic)
CREATE TABLE knowledge_map_heat (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    heat_label TEXT NOT NULL DEFAULT 'unseen',
    mastery_heat INTEGER NOT NULL DEFAULT 0,
    stability_heat INTEGER NOT NULL DEFAULT 0,
    misconception_heat INTEGER NOT NULL DEFAULT 0,
    coverage_heat INTEGER NOT NULL DEFAULT 0,
    momentum_heat INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX idx_knowledge_map_heat_student ON knowledge_map_heat(student_id);

-- Morale feedback signals
CREATE TABLE morale_signals (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    signal_type TEXT NOT NULL,
    message TEXT NOT NULL,
    context_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_morale_signals_student ON morale_signals(student_id, subject_id);
