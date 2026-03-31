-- idea11 deep gaps: recognition visibility on questions, per-class evidence scores
-- on diagnoses, concept possession score, diagnostic pipeline output fields.

-- ============================================================================
-- 1. Recognition visibility level on questions (not just diagnoses)
-- ============================================================================

ALTER TABLE questions ADD COLUMN recognition_visibility TEXT;

-- ============================================================================
-- 2. Per-error-class evidence scores on wrong_answer_diagnoses
-- (output of the 10-stage diagnostic pipeline)
-- ============================================================================

ALTER TABLE wrong_answer_diagnoses ADD COLUMN knowledge_absence_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN recognition_failure_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN interpretation_failure_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN application_failure_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN reasoning_breakdown_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN execution_slip_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN state_amplifier_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN prerequisite_failure_score INTEGER;

-- ============================================================================
-- 3. Concept possession score and diagnostic confidence
-- ============================================================================

ALTER TABLE wrong_answer_diagnoses ADD COLUMN concept_possession_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN diagnostic_confidence_score INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN amplifier_type TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN root_factor TEXT;

-- ============================================================================
-- 4. Student recognition strength per topic
-- (separate from mastery — tracks concept identification ability)
-- ============================================================================

ALTER TABLE student_topic_states ADD COLUMN recognition_strength INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE student_topic_states ADD COLUMN explicit_form_accuracy_bp INTEGER;
ALTER TABLE student_topic_states ADD COLUMN disguised_form_accuracy_bp INTEGER;
ALTER TABLE student_topic_states ADD COLUMN recognition_gap_bp INTEGER;
