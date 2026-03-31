-- idea7 deep: proof dimensions, subject templates, extended memory state fields.

-- ============================================================================
-- 1. Proof dimension tracking per learner per skill
-- ============================================================================

CREATE TABLE memory_proof_dimensions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    node_id INTEGER NOT NULL,
    dimension_name TEXT NOT NULL
        CHECK (dimension_name IN (
            'independent_recall', 'delayed_recall', 'variant_transfer',
            'embedded_use', 'interference_resistance', 'explanation_reasoning',
            'representation_shift', 'sequence_relation', 'speed_fluency'
        )),
    status TEXT NOT NULL DEFAULT 'none'
        CHECK (status IN ('none', 'partial', 'satisfied')),
    evidence_count INTEGER NOT NULL DEFAULT 0,
    best_recent_score INTEGER NOT NULL DEFAULT 0,
    last_satisfied_at TEXT,
    UNIQUE(student_id, node_id, dimension_name)
);

CREATE INDEX idx_memory_proof_student ON memory_proof_dimensions(student_id, node_id);

-- ============================================================================
-- 2. Subject proof templates
-- ============================================================================

CREATE TABLE memory_proof_templates (
    id INTEGER PRIMARY KEY,
    subject_id INTEGER REFERENCES subjects(id),
    skill_type TEXT NOT NULL,
    template_name TEXT NOT NULL,
    required_dimensions_json TEXT NOT NULL DEFAULT '[]',
    minimum_time_separated_successes INTEGER NOT NULL DEFAULT 1,
    minimum_distinct_variants INTEGER NOT NULL DEFAULT 1,
    minimum_embedded_successes INTEGER NOT NULL DEFAULT 0,
    maximum_allowed_interference_failures INTEGER NOT NULL DEFAULT 2,
    minimum_speed_score INTEGER NOT NULL DEFAULT 0,
    confirmation_threshold INTEGER NOT NULL DEFAULT 70,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed subject templates
INSERT INTO memory_proof_templates
    (subject_id, skill_type, template_name, required_dimensions_json,
     minimum_time_separated_successes, minimum_distinct_variants)
VALUES
    (NULL, 'procedure', 'Math Procedure',
     '["independent_recall","variant_transfer","delayed_recall","embedded_use"]', 1, 1),
    (NULL, 'concept', 'Science Concept',
     '["independent_recall","explanation_reasoning","representation_shift","variant_transfer","delayed_recall","interference_resistance"]', 1, 1),
    (NULL, 'fact', 'Theory Fact/Relation',
     '["independent_recall","explanation_reasoning","sequence_relation","interference_resistance","delayed_recall"]', 1, 1);

-- ============================================================================
-- 3. Extend memory_states with deeper fields
-- ============================================================================

ALTER TABLE memory_states ADD COLUMN retrieval_access_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_states ADD COLUMN durability_confidence_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_states ADD COLUMN interference_risk_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_states ADD COLUMN relapse_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_states ADD COLUMN last_relapse_at TEXT;
ALTER TABLE memory_states ADD COLUMN last_independent_recall_at TEXT;
ALTER TABLE memory_states ADD COLUMN proof_template_id INTEGER;
ALTER TABLE memory_states ADD COLUMN time_separated_success_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_states ADD COLUMN variant_success_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_states ADD COLUMN embedded_use_success_count INTEGER NOT NULL DEFAULT 0;

-- ============================================================================
-- 4. Extend memory_evidence_events with diagnostic role
-- ============================================================================

ALTER TABLE memory_evidence_events ADD COLUMN diagnostic_role TEXT;
ALTER TABLE memory_evidence_events ADD COLUMN is_time_separated INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN is_mixed_context INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memory_evidence_events ADD COLUMN representation_mode TEXT;
ALTER TABLE memory_evidence_events ADD COLUMN variant_type TEXT;
