-- idea9 deep: per-family payload structures, difficulty profiles,
-- generation logging, concept review queue.

-- ============================================================================
-- 1. Multi-dimensional difficulty profiles per question
-- ============================================================================

ALTER TABLE questions ADD COLUMN difficulty_profile_json TEXT;

-- ============================================================================
-- 2. Concept-level review queue (for question factory scheduling)
-- ============================================================================

CREATE TABLE concept_review_queue (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    concept_node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    review_type TEXT NOT NULL
        CHECK (review_type IN (
            'retention_check', 'recovery', 'fluency',
            'misconception_repair', 'transfer_check',
            'reconstruction', 'pressure_check'
        )),
    due_at TEXT NOT NULL,
    priority_score INTEGER NOT NULL DEFAULT 50,
    source_reason TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'completed', 'expired', 'dismissed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_concept_review_student ON concept_review_queue(student_id);
CREATE INDEX idx_concept_review_due ON concept_review_queue(due_at);

-- ============================================================================
-- 3. Question generation log (observability)
-- ============================================================================

CREATE TABLE question_generation_log (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL,
    concept_node_id INTEGER,
    question_family_id INTEGER,
    cognitive_type TEXT,
    evaluative_intent TEXT,
    mode_context TEXT,
    difficulty_profile_json TEXT,
    variant_signature TEXT,
    generated_question_id INTEGER,
    learner_state_snapshot_json TEXT,
    generation_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_gen_log_student ON question_generation_log(student_id);
CREATE INDEX idx_gen_log_created ON question_generation_log(created_at);

-- ============================================================================
-- 4. Family-specific payload columns on question generation requests
-- ============================================================================

ALTER TABLE question_generation_requests ADD COLUMN cognitive_type TEXT;
ALTER TABLE question_generation_requests ADD COLUMN evaluative_intent TEXT;
ALTER TABLE question_generation_requests ADD COLUMN mode_context TEXT;
ALTER TABLE question_generation_requests ADD COLUMN difficulty_profile_json TEXT;
ALTER TABLE question_generation_requests ADD COLUMN target_primitive_ids_json TEXT;
ALTER TABLE question_generation_requests ADD COLUMN misconception_target_id INTEGER;
