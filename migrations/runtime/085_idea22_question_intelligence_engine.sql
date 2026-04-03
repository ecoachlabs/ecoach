-- idea22: Question Intelligence Engine runtime profile, review workflow,
-- family membership, misconception mapping, and reclassification support.

CREATE UNIQUE INDEX IF NOT EXISTS idx_question_families_code_unique
    ON question_families(family_code);

CREATE TABLE IF NOT EXISTS question_intelligence_profiles (
    id INTEGER PRIMARY KEY,
    question_id INTEGER NOT NULL UNIQUE REFERENCES questions(id) ON DELETE CASCADE,
    taxonomy_version TEXT NOT NULL DEFAULT 'qi_taxonomy_v1',
    classification_version TEXT NOT NULL DEFAULT 'qi_engine_v1',
    family_engine_version TEXT NOT NULL DEFAULT 'qi_family_v1',
    classification_source TEXT NOT NULL DEFAULT 'rules'
        CHECK (classification_source IN (
            'pack', 'rules', 'model', 'hybrid', 'review', 'reactor', 'foundry'
        )),
    machine_confidence_bp INTEGER NOT NULL DEFAULT 0,
    primary_knowledge_role TEXT,
    primary_cognitive_demand TEXT,
    primary_solve_pattern TEXT,
    primary_pedagogic_function TEXT,
    primary_content_grain TEXT,
    question_family_id INTEGER REFERENCES question_families(id),
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (review_status IN (
            'pending', 'approved', 'rejected', 'needs_review',
            'taxonomy_gap', 'family_unresolved', 'overridden'
        )),
    review_reason TEXT,
    reviewer_id TEXT,
    reviewed_at TEXT,
    needs_reclassification INTEGER NOT NULL DEFAULT 0,
    snapshot_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_qi_profiles_review
    ON question_intelligence_profiles(review_status, machine_confidence_bp);

CREATE TABLE IF NOT EXISTS question_misconceptions (
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    misconception_code TEXT NOT NULL,
    confidence_score_bp INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL DEFAULT 'system'
        CHECK (source IN ('pack', 'system', 'review')),
    PRIMARY KEY (question_id, misconception_code)
);

CREATE INDEX IF NOT EXISTS idx_question_misconceptions_code
    ON question_misconceptions(misconception_code, confidence_score_bp DESC);

CREATE TABLE IF NOT EXISTS question_family_members (
    question_family_id INTEGER NOT NULL REFERENCES question_families(id) ON DELETE CASCADE,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    role_in_family TEXT NOT NULL DEFAULT 'member'
        CHECK (role_in_family IN ('canonical', 'variant', 'edge_case', 'inverse_pair', 'member')),
    similarity_score_bp INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (question_family_id, question_id)
);

CREATE INDEX IF NOT EXISTS idx_question_family_members_question
    ON question_family_members(question_id);

CREATE TABLE IF NOT EXISTS question_intelligence_reviews (
    id INTEGER PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    reviewer_id TEXT NOT NULL,
    action_code TEXT NOT NULL
        CHECK (action_code IN (
            'approve', 'reject', 'override', 'mark_taxonomy_gap',
            'mark_family_unresolved', 'send_for_reclassification'
        )),
    previous_review_status TEXT,
    new_review_status TEXT NOT NULL,
    note TEXT,
    previous_snapshot_json TEXT,
    new_snapshot_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_qi_reviews_question
    ON question_intelligence_reviews(question_id, created_at DESC);

CREATE TABLE IF NOT EXISTS question_reclassification_queue (
    id INTEGER PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    trigger_reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'processing', 'completed', 'failed')),
    requested_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    processed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_qi_reclass_status
    ON question_reclassification_queue(status, created_at);

INSERT OR IGNORE INTO question_intelligence_taxonomy (axis_code, concept_code, display_name, description) VALUES
    ('knowledge_role', 'definition', 'Definition', 'Tests a definition or direct statement of meaning.'),
    ('knowledge_role', 'key_concept', 'Key Concept', 'Centres on a core conceptual idea.'),
    ('knowledge_role', 'principle', 'Principle', 'Tests a principle or law-like statement.'),
    ('knowledge_role', 'formula_recall', 'Formula Recall', 'Requires remembering or selecting a formula.'),
    ('knowledge_role', 'formula_derivation', 'Formula Derivation', 'Requires deriving or justifying a formula.'),
    ('knowledge_role', 'worked_example', 'Worked Example', 'Uses a worked pattern or modeled solution.'),
    ('knowledge_role', 'procedure', 'Procedure', 'Tests a method or ordered process.'),
    ('knowledge_role', 'explanation', 'Explanation', 'Requires explaining an idea or phenomenon.'),
    ('knowledge_role', 'comparison', 'Comparison', 'Requires comparing two ideas or cases.'),
    ('knowledge_role', 'classification', 'Classification', 'Requires sorting into the right class or category.'),
    ('knowledge_role', 'application', 'Application Scenario', 'Applies knowledge in context.'),
    ('cognitive_demand', 'recognition', 'Recognition', 'Recognize the correct item or pattern.'),
    ('cognitive_demand', 'recall', 'Recall', 'Recall from memory.'),
    ('cognitive_demand', 'comprehension', 'Comprehension', 'Explain or interpret meaning.'),
    ('cognitive_demand', 'application', 'Application', 'Apply knowledge directly.'),
    ('cognitive_demand', 'analysis', 'Analysis', 'Break down and analyze.'),
    ('cognitive_demand', 'inference', 'Inference', 'Infer from evidence or conditions.'),
    ('cognitive_demand', 'justification', 'Justification', 'Provide a reasoned justification.'),
    ('solve_pattern', 'direct_retrieval', 'Direct Retrieval', 'Solved mainly by remembering.'),
    ('solve_pattern', 'substitute_and_solve', 'Substitute And Solve', 'Substitute into a rule or formula and solve.'),
    ('solve_pattern', 'pattern_spotting', 'Pattern Spotting', 'Solved by noticing a recurring pattern.'),
    ('solve_pattern', 'multi_step_reasoning', 'Multi-step Reasoning', 'Requires multiple connected steps.'),
    ('solve_pattern', 'proof_chain', 'Proof Chain', 'Requires linked proof or justification steps.'),
    ('solve_pattern', 'graph_or_table_reading', 'Graph Or Table Reading', 'Solved through chart or table interpretation.'),
    ('pedagogic_function', 'classification', 'Classification', 'Basic classification check.'),
    ('pedagogic_function', 'foundation_check', 'Foundation Check', 'Checks foundational understanding.'),
    ('pedagogic_function', 'misconception_diagnosis', 'Misconception Diagnosis', 'Designed to expose a misconception.'),
    ('pedagogic_function', 'exam_pattern_familiarization', 'Exam Pattern Familiarization', 'Builds familiarity with exam patterns.'),
    ('pedagogic_function', 'transfer_check', 'Transfer Check', 'Checks transfer into a new context.'),
    ('pedagogic_function', 'speed_build', 'Speed Build', 'Supports faster response under pressure.'),
    ('content_grain', 'topic', 'Topic', 'Operates at topic level.'),
    ('content_grain', 'concept', 'Concept', 'Targets a concept node.'),
    ('content_grain', 'micro_concept', 'Micro Concept', 'Targets a narrow sub-concept or step.'),
    ('content_grain', 'skill', 'Skill', 'Targets a skill or method.'),
    ('content_grain', 'formula', 'Formula', 'Targets a formula or symbolic relation.'),
    ('question_family', 'direct_variation_family', 'Direct Variation Family', 'Recurring direct variation pattern.'),
    ('question_family', 'worked_example_template', 'Worked Example Template', 'Recurring worked-example template.'),
    ('question_family', 'reasoning_family', 'Reasoning Family', 'Recurring reasoning pattern family.'),
    ('question_family', 'misconception_cluster', 'Misconception Cluster', 'Recurring misconception-heavy family.'),
    ('question_family', 'inverse_pair_family', 'Inverse Pair Family', 'Inverse or contrast pair family.'),
    ('misconception_exposure', 'wrong_inverse_operation', 'Wrong Inverse Operation', 'Confuses the inverse step needed to solve.'),
    ('misconception_exposure', 'surface_pattern_copy', 'Surface Pattern Copy', 'Copies surface pattern without meaning.'),
    ('misconception_exposure', 'concept_label_confusion', 'Concept Label Confusion', 'Confuses related concept labels.'),
    ('misconception_exposure', 'formula_selection_error', 'Formula Selection Error', 'Chooses the wrong formula or rule.'),
    ('misconception_exposure', 'process_vs_definition_confusion', 'Process Vs Definition Confusion', 'Confuses a process with a definition.');
