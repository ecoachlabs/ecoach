-- idea7 deep engine gaps: memory sessions, extended recheck fields,
-- question diagnostic metadata, recovery path tracking.

-- ============================================================================
-- 1. Dedicated memory sessions
-- ============================================================================

CREATE TABLE memory_sessions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    session_id INTEGER REFERENCES sessions(id),
    mode_type TEXT NOT NULL DEFAULT 'scan'
        CHECK (mode_type IN (
            'scan', 'rescue_burst', 'deep_repair', 'chain_repair',
            'recall_builder', 'fluency_drill'
        )),
    primary_skill_ids_json TEXT NOT NULL DEFAULT '[]',
    secondary_skill_ids_json TEXT,
    session_goal TEXT,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'completed', 'abandoned')),
    skills_promoted_count INTEGER NOT NULL DEFAULT 0,
    skills_regressed_count INTEGER NOT NULL DEFAULT 0,
    skills_flagged_count INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms INTEGER,
    avg_accuracy_bp INTEGER,
    memory_gain_score INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_memory_sessions_student ON memory_sessions(student_id, subject_id);

-- ============================================================================
-- 2. Extend recheck_schedules with missing fields
-- ============================================================================

ALTER TABLE recheck_schedules ADD COLUMN target_proof_dimension TEXT;
ALTER TABLE recheck_schedules ADD COLUMN priority_score INTEGER NOT NULL DEFAULT 50;

-- ============================================================================
-- 3. Recovery path tracking on memory_states
-- ============================================================================

ALTER TABLE memory_states ADD COLUMN recovery_path_type TEXT;
ALTER TABLE memory_states ADD COLUMN recovery_started_at TEXT;
ALTER TABLE memory_states ADD COLUMN last_state_transition_at TEXT;

-- ============================================================================
-- 4. Evidence tier and diagnostic supports on memory_evidence_events
-- ============================================================================

ALTER TABLE memory_evidence_events ADD COLUMN evidence_tier TEXT;
ALTER TABLE memory_evidence_events ADD COLUMN supports_independent_recall INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN supports_variant_transfer INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN supports_delayed_recall INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN supports_embedded_use INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN supports_interference_resistance INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN supports_explanation_reasoning INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN supports_representation_shift INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN supports_sequence_relation INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN supports_speed_fluency INTEGER NOT NULL DEFAULT 0;
