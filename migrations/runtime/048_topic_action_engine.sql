-- idea3 deep features: topic action sessions, stepped questions, topic proof, goal recommendations.

-- ============================================================================
-- 1. Topic action sessions (4 modes per topic)
-- ============================================================================

CREATE TABLE topic_action_sessions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    action_mode TEXT NOT NULL
        CHECK (action_mode IN ('learn', 'repair', 'revision', 'expert')),
    session_id INTEGER REFERENCES sessions(id),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'completed', 'abandoned')),
    diagnosis_json TEXT,
    repair_path_json TEXT,
    progress_score INTEGER NOT NULL DEFAULT 0,
    subtopics_total INTEGER NOT NULL DEFAULT 0,
    subtopics_completed INTEGER NOT NULL DEFAULT 0,
    mastery_at_start INTEGER NOT NULL DEFAULT 0,
    mastery_at_end INTEGER,
    symptom_input_json TEXT,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_topic_action_student ON topic_action_sessions(student_id, topic_id);
CREATE INDEX idx_topic_action_mode ON topic_action_sessions(action_mode);
CREATE INDEX idx_topic_action_status ON topic_action_sessions(status);

-- ============================================================================
-- 2. Stepped question attempts (Watch Me Solve / breakpoint detection)
-- ============================================================================

CREATE TABLE stepped_question_templates (
    id INTEGER PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id),
    total_steps INTEGER NOT NULL DEFAULT 1,
    steps_json TEXT NOT NULL DEFAULT '[]',
    subject_reasoning_type TEXT NOT NULL DEFAULT 'procedural'
        CHECK (subject_reasoning_type IN (
            'procedural', 'causal', 'interpretive', 'recall_chain', 'analytical'
        )),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_stepped_templates_question ON stepped_question_templates(question_id);

CREATE TABLE stepped_attempts (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    question_id INTEGER NOT NULL,
    template_id INTEGER NOT NULL REFERENCES stepped_question_templates(id),
    session_id INTEGER,
    total_steps INTEGER NOT NULL DEFAULT 1,
    steps_completed INTEGER NOT NULL DEFAULT 0,
    breakpoint_step INTEGER,
    breakpoint_reason TEXT,
    steps_data_json TEXT NOT NULL DEFAULT '[]',
    thinking_map_json TEXT,
    overall_correct INTEGER NOT NULL DEFAULT 0,
    response_time_ms INTEGER,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_stepped_attempts_student ON stepped_attempts(student_id);
CREATE INDEX idx_stepped_attempts_question ON stepped_attempts(question_id);

-- ============================================================================
-- 3. Topic proof certifications
-- ============================================================================

CREATE TABLE topic_proof_certifications (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    proof_tier TEXT NOT NULL DEFAULT 'not_ready'
        CHECK (proof_tier IN (
            'not_ready', 'emerging', 'functional', 'strong', 'certified', 'expert'
        )),
    accuracy_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    transfer_score INTEGER NOT NULL DEFAULT 0,
    variation_score INTEGER NOT NULL DEFAULT 0,
    pressure_score INTEGER NOT NULL DEFAULT 0,
    mistake_recurrence_score INTEGER NOT NULL DEFAULT 0,
    reasoning_score INTEGER NOT NULL DEFAULT 0,
    composite_score INTEGER NOT NULL DEFAULT 0,
    evidence_count INTEGER NOT NULL DEFAULT 0,
    last_assessed_at TEXT,
    certified_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE INDEX idx_topic_proof_student ON topic_proof_certifications(student_id, subject_id);
CREATE INDEX idx_topic_proof_tier ON topic_proof_certifications(proof_tier);

-- ============================================================================
-- 4. Goal recommendation profiles
-- ============================================================================

ALTER TABLE goal_targets ADD COLUMN goal_type TEXT NOT NULL DEFAULT 'exam_readiness';
ALTER TABLE goal_targets ADD COLUMN urgency_band TEXT NOT NULL DEFAULT 'structured';
ALTER TABLE goal_targets ADD COLUMN confidence_level TEXT;
ALTER TABLE goal_targets ADD COLUMN recommended_default_mode TEXT;
