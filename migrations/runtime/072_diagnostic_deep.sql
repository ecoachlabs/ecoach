-- ideas 18, 33: Multi-dimensional diagnostic battery, adaptive diagnostic routing,
-- error typing taxonomy, endurance tracking, student profile classification.

-- ============================================================================
-- 1. Diagnostic session groups (8-session battery structure)
-- ============================================================================

CREATE TABLE diagnostic_session_groups (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    group_type TEXT NOT NULL DEFAULT 'standard'
        CHECK (group_type IN ('light', 'standard', 'deep', 'custom')),
    status TEXT NOT NULL DEFAULT 'in_progress'
        CHECK (status IN ('in_progress', 'completed', 'abandoned')),
    stages_completed_json TEXT NOT NULL DEFAULT '{}',
    profile_type TEXT,
    comparison_deltas_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX idx_diagnostic_groups_student ON diagnostic_session_groups(student_id);

-- ============================================================================
-- 2. Diagnostic session phases (per-phase performance)
-- ============================================================================

CREATE TABLE diagnostic_phase_results (
    id INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES diagnostic_session_groups(id),
    phase_type TEXT NOT NULL
        CHECK (phase_type IN (
            'baseline', 'speed', 'precision', 'pressure',
            'flexibility', 'root_cause', 'endurance', 'recovery',
            'misconception_probe', 'transfer', 'confidence_capture'
        )),
    session_id INTEGER REFERENCES sessions(id),
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    fluency_bp INTEGER NOT NULL DEFAULT 0,
    precision_bp INTEGER NOT NULL DEFAULT 0,
    pressure_bp INTEGER NOT NULL DEFAULT 0,
    flexibility_bp INTEGER NOT NULL DEFAULT 0,
    stability_bp INTEGER NOT NULL DEFAULT 0,
    early_segment_accuracy_bp INTEGER,
    middle_segment_accuracy_bp INTEGER,
    final_segment_accuracy_bp INTEGER,
    confidence_capture_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_diagnostic_phases_group ON diagnostic_phase_results(group_id);

-- ============================================================================
-- 3. Error typing taxonomy
-- ============================================================================

CREATE TABLE error_type_taxonomy (
    id INTEGER PRIMARY KEY,
    error_code TEXT NOT NULL UNIQUE,
    error_name TEXT NOT NULL,
    category TEXT NOT NULL
        CHECK (category IN (
            'conceptual', 'formula', 'substitution', 'arithmetic',
            'unit_sign', 'misread', 'distractor', 'rushed', 'blank',
            'pattern_confusion', 'partial_method', 'step_omission'
        )),
    description TEXT,
    failure_stage TEXT,
    diagnostic_signals_json TEXT,
    recovery_strategy TEXT
);

INSERT INTO error_type_taxonomy (error_code, error_name, category, failure_stage) VALUES
    ('conceptual_gap', 'Conceptual Understanding Gap', 'conceptual', 'concept_level'),
    ('formula_selection', 'Wrong Formula Selection', 'formula', 'formula_level'),
    ('formula_recall', 'Formula Recall Failure', 'formula', 'formula_level'),
    ('substitution_error', 'Substitution Error', 'substitution', 'substitution_level'),
    ('arithmetic_slip', 'Arithmetic Slip', 'arithmetic', 'arithmetic_level'),
    ('unit_sign_error', 'Unit/Sign Error', 'unit_sign', 'arithmetic_level'),
    ('misread_question', 'Misread Question', 'misread', 'interpretation_level'),
    ('distractor_attracted', 'Distractor Attraction', 'distractor', 'interpretation_level'),
    ('rushed_answer', 'Rushed Answer', 'rushed', 'execution_level'),
    ('blank_no_attempt', 'No Attempt', 'blank', 'concept_level'),
    ('pattern_confused', 'Pattern Confusion', 'pattern_confusion', 'concept_level'),
    ('partial_method', 'Partial Method Applied', 'partial_method', 'step_level'),
    ('step_omitted', 'Step Omission', 'step_omission', 'step_level');

-- ============================================================================
-- 4. Student profile classifier
-- ============================================================================

CREATE TABLE student_learning_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER,
    profile_type TEXT NOT NULL
        CHECK (profile_type IN (
            'sprinter', 'careful_thinker', 'fragile_knower',
            'pressure_collapser', 'formula_memorizer', 'execution_breaker',
            'recognition_gap', 'balanced', 'unclassified'
        )),
    confidence_bp INTEGER NOT NULL DEFAULT 5000,
    evidence_json TEXT,
    diagnostic_group_id INTEGER REFERENCES diagnostic_session_groups(id),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

-- ============================================================================
-- 5. Diagnostic comparative deltas
-- ============================================================================

CREATE TABLE diagnostic_deltas (
    id INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES diagnostic_session_groups(id),
    student_id INTEGER NOT NULL,
    topic_id INTEGER,
    speed_accuracy_delta_bp INTEGER,
    calm_pressure_delta_bp INTEGER,
    direct_variant_delta_bp INTEGER,
    recall_application_delta_bp INTEGER,
    formula_recall_use_delta_bp INTEGER,
    early_late_delta_bp INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_diagnostic_deltas_group ON diagnostic_deltas(group_id);

-- ============================================================================
-- 6. Learning dimensions (12-dimensional readiness profile)
-- ============================================================================

CREATE TABLE diagnostic_learning_dimensions (
    id INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES diagnostic_session_groups(id),
    student_id INTEGER NOT NULL,
    coverage_bp INTEGER NOT NULL DEFAULT 0,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    recall_strength_bp INTEGER NOT NULL DEFAULT 0,
    recognition_vs_production_bp INTEGER NOT NULL DEFAULT 5000,
    reasoning_depth_bp INTEGER NOT NULL DEFAULT 0,
    misconception_density_bp INTEGER NOT NULL DEFAULT 0,
    speed_bp INTEGER NOT NULL DEFAULT 0,
    pressure_response_bp INTEGER NOT NULL DEFAULT 0,
    transfer_ability_bp INTEGER NOT NULL DEFAULT 0,
    stability_bp INTEGER NOT NULL DEFAULT 0,
    confidence_calibration_bp INTEGER NOT NULL DEFAULT 5000,
    fatigue_pattern_bp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ============================================================================
-- 7. Extend diagnostic_topic_analytics
-- ============================================================================

ALTER TABLE diagnostic_topic_analytics ADD COLUMN endurance_score_bp INTEGER;
ALTER TABLE diagnostic_topic_analytics ADD COLUMN error_distribution_json TEXT;
ALTER TABLE diagnostic_topic_analytics ADD COLUMN weakness_type TEXT;
ALTER TABLE diagnostic_topic_analytics ADD COLUMN failure_stage TEXT;
