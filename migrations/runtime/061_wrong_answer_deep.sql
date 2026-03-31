-- idea11: Deep wrong answer analysis — extended diagnosis fields,
-- error persistence, error families, distractor analysis, mastery fragility,
-- intervention tracking, academic analyst data.

-- ============================================================================
-- 1. Extend wrong_answer_diagnoses with missing fields from idea11
-- ============================================================================

ALTER TABLE wrong_answer_diagnoses ADD COLUMN reasoning_family TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN recognition_visibility TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN persistence_state TEXT NOT NULL DEFAULT 'new';
ALTER TABLE wrong_answer_diagnoses ADD COLUMN severity_depth INTEGER NOT NULL DEFAULT 0;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN severity_breadth INTEGER NOT NULL DEFAULT 0;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN severity_recurrence INTEGER NOT NULL DEFAULT 0;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN severity_repairability INTEGER NOT NULL DEFAULT 0;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN severity_exam_risk INTEGER NOT NULL DEFAULT 0;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN first_wrong_step TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN likely_misconception TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN intervention_assigned TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN repair_outcome TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN response_time_ms INTEGER;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN confidence_rating TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN changed_answer_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN distractor_analysis_json TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN reasoning_reconstruction_json TEXT;
ALTER TABLE wrong_answer_diagnoses ADD COLUMN question_fingerprint TEXT;

-- ============================================================================
-- 2. Error families (separate from question families)
-- ============================================================================

CREATE TABLE error_families (
    id INTEGER PRIMARY KEY,
    family_code TEXT NOT NULL UNIQUE,
    family_name TEXT NOT NULL,
    error_class TEXT NOT NULL,
    description TEXT,
    common_triggers_json TEXT,
    repair_strategies_json TEXT,
    subject_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_error_families_class ON error_families(error_class);

-- ============================================================================
-- 3. Student error family tracking (recurring patterns)
-- ============================================================================

CREATE TABLE student_error_family_history (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    error_family_id INTEGER NOT NULL REFERENCES error_families(id),
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    first_detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    persistence_state TEXT NOT NULL DEFAULT 'new'
        CHECK (persistence_state IN (
            'new', 'emerging', 'recurring', 'entrenched',
            'repairing', 'resolved', 'relapsing'
        )),
    repair_attempts INTEGER NOT NULL DEFAULT 0,
    successful_repairs INTEGER NOT NULL DEFAULT 0,
    recovery_speed_ms INTEGER,
    relapse_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(student_id, error_family_id)
);

CREATE INDEX idx_student_error_family_student ON student_error_family_history(student_id);

-- ============================================================================
-- 4. Distractor analysis per wrong answer
-- ============================================================================

CREATE TABLE distractor_analysis_records (
    id INTEGER PRIMARY KEY,
    diagnosis_id INTEGER NOT NULL REFERENCES wrong_answer_diagnoses(id),
    option_id INTEGER NOT NULL,
    option_text TEXT,
    is_selected INTEGER NOT NULL DEFAULT 0,
    is_correct INTEGER NOT NULL DEFAULT 0,
    misconception_id INTEGER,
    distractor_intent TEXT,
    temptation_type TEXT
        CHECK (temptation_type IN (
            'keyword_match', 'half_true', 'extreme_version',
            'reversed_cause', 'lookalike', 'partial_procedure',
            'surface_similarity', 'common_error_result'
        )),
    why_attractive TEXT,
    why_wrong TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_distractor_analysis_diagnosis ON distractor_analysis_records(diagnosis_id);

-- ============================================================================
-- 5. Mastery fragility dimensions (5-dimensional)
-- ============================================================================

ALTER TABLE student_topic_states ADD COLUMN fragility_accuracy INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_topic_states ADD COLUMN fragility_stability INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_topic_states ADD COLUMN fragility_transferability INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_topic_states ADD COLUMN fragility_pressure_resistance INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_topic_states ADD COLUMN fragility_trap_resistance INTEGER NOT NULL DEFAULT 0;

-- ============================================================================
-- 6. Student vulnerability profiles (7 live profiles from idea11)
-- ============================================================================

CREATE TABLE student_vulnerability_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    concept_weakness_score INTEGER NOT NULL DEFAULT 0,
    misconception_density_score INTEGER NOT NULL DEFAULT 0,
    reasoning_weakness_score INTEGER NOT NULL DEFAULT 0,
    distractor_vulnerability_score INTEGER NOT NULL DEFAULT 0,
    pressure_vulnerability_score INTEGER NOT NULL DEFAULT 0,
    transfer_weakness_score INTEGER NOT NULL DEFAULT 0,
    recovery_speed_score INTEGER NOT NULL DEFAULT 0,
    relapse_rate_score INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE INDEX idx_vulnerability_profiles_student ON student_vulnerability_profiles(student_id);

-- ============================================================================
-- 7. Intervention tracking
-- ============================================================================

CREATE TABLE wrong_answer_interventions (
    id INTEGER PRIMARY KEY,
    diagnosis_id INTEGER NOT NULL REFERENCES wrong_answer_diagnoses(id),
    student_id INTEGER NOT NULL,
    intervention_type TEXT NOT NULL
        CHECK (intervention_type IN (
            'instant_repair', 'contrast_repair', 'step_repair',
            'misconception_repair', 'family_repair', 'prerequisite_repair',
            'pressure_repair', 'reflection_repair'
        )),
    status TEXT NOT NULL DEFAULT 'assigned'
        CHECK (status IN ('assigned', 'started', 'completed', 'failed', 'skipped')),
    repair_session_id INTEGER,
    outcome_mastery_delta INTEGER,
    outcome_notes TEXT,
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX idx_wrong_interventions_diagnosis ON wrong_answer_interventions(diagnosis_id);
CREATE INDEX idx_wrong_interventions_student ON wrong_answer_interventions(student_id);

-- ============================================================================
-- 8. Extend question_options with distractor profile
-- ============================================================================

ALTER TABLE question_options ADD COLUMN temptation_type TEXT;
ALTER TABLE question_options ADD COLUMN why_attractive TEXT;
ALTER TABLE question_options ADD COLUMN misconception_subtype TEXT;
ALTER TABLE question_options ADD COLUMN severity_if_chosen INTEGER;
