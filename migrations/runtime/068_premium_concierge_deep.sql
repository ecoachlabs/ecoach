-- idea12 deep gaps: missing strategy_state fields, intervention enrichment,
-- risk trigger rules table, escalation rules, readiness calculation weights.

-- ============================================================================
-- 1. Strategy states missing fields
-- ============================================================================

ALTER TABLE strategy_states ADD COLUMN intensity_level TEXT NOT NULL DEFAULT 'moderate';
ALTER TABLE strategy_states ADD COLUMN parent_explanation TEXT;

-- ============================================================================
-- 2. Intervention records enrichment
-- ============================================================================

ALTER TABLE intervention_records ADD COLUMN triggered_by_rule TEXT;
ALTER TABLE intervention_records ADD COLUMN remediation_logic_json TEXT;
ALTER TABLE intervention_records ADD COLUMN monitoring_signals_json TEXT;
ALTER TABLE intervention_records ADD COLUMN escalation_threshold_bp INTEGER;
ALTER TABLE intervention_records ADD COLUMN expected_improvement_bp INTEGER;

-- ============================================================================
-- 3. Risk trigger rules (configurable detection conditions)
-- ============================================================================

CREATE TABLE risk_trigger_rules (
    id INTEGER PRIMARY KEY,
    rule_code TEXT NOT NULL UNIQUE,
    rule_name TEXT NOT NULL,
    risk_category TEXT NOT NULL,
    risk_subtype TEXT NOT NULL,
    default_severity TEXT NOT NULL DEFAULT 'medium',
    description TEXT NOT NULL,
    condition_logic_json TEXT NOT NULL,
    min_evidence_count INTEGER NOT NULL DEFAULT 2,
    cooldown_hours INTEGER NOT NULL DEFAULT 24,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO risk_trigger_rules (rule_code, rule_name, risk_category, risk_subtype, default_severity, description, condition_logic_json, min_evidence_count) VALUES
    ('memory_decay', 'Memory Decay Detection', 'knowledge', 'topic_decay', 'medium', 'Flag when previously stable topic falls below retention threshold across two spaced checks', '{"check":"memory_state","condition":"was_stable_now_fading","threshold_checks":2}', 2),
    ('slow_completion', 'Slow Completion Detection', 'performance', 'slow_completion', 'medium', 'Flag when untimed accuracy stable but timed performance falls below expected band', '{"check":"timed_vs_untimed_gap","threshold_bp":2000}', 3),
    ('misconception_recurrence', 'Misconception Recurrence', 'reasoning', 'misconception_recurrence', 'high', 'Flag when same reasoning error occurs across multiple items within same question family', '{"check":"same_error_across_family","min_occurrences":3}', 3),
    ('plateau', 'Progress Plateau', 'strategic', 'plateau', 'medium', 'Flag when intervention active but expected movement not occurring within review window', '{"check":"intervention_stall","review_window_days":7,"min_expected_movement_bp":500}', 1),
    ('confidence_instability', 'Confidence Instability', 'confidence', 'fragile_confidence', 'medium', 'Flag when accuracy drops sharply after one or two wrong answers in same session', '{"check":"accuracy_drop_after_errors","drop_threshold_bp":3000,"trigger_errors":2}', 2),
    ('avoidance', 'Subject Avoidance', 'behavioral', 'subject_avoidance', 'medium', 'Flag when student repeatedly delays or under-engages with high-priority subject', '{"check":"engagement_gap","days_threshold":5,"priority_threshold_bp":7000}', 1),
    ('careless_spike', 'Careless Error Spike', 'performance', 'careless_error_spike', 'medium', 'Flag when careless errors increase significantly in recent sessions', '{"check":"careless_rate_increase","baseline_window_days":14,"spike_threshold_bp":2000}', 3),
    ('overconfidence', 'Overconfidence Pattern', 'confidence', 'overconfidence', 'medium', 'Flag when student frequently confident but wrong', '{"check":"confidence_wrong_rate","threshold_bp":3000,"min_responses":10}', 10),
    ('pressure_collapse', 'Pressure Collapse', 'performance', 'time_pressure_collapse', 'high', 'Flag when timed performance significantly worse than untimed', '{"check":"timed_accuracy_drop","threshold_bp":2500}', 3),
    ('recovery_lag', 'Recovery Lag', 'confidence', 'recovery_lag', 'medium', 'Flag when performance stays low after poor session without bouncing back', '{"check":"post_failure_recovery","recovery_window_sessions":3,"min_expected_recovery_bp":1500}', 3);

-- ============================================================================
-- 4. Escalation rules (when to involve human strategist)
-- ============================================================================

CREATE TABLE escalation_rules (
    id INTEGER PRIMARY KEY,
    rule_code TEXT NOT NULL UNIQUE,
    rule_name TEXT NOT NULL,
    description TEXT NOT NULL,
    trigger_condition_json TEXT NOT NULL,
    escalation_type TEXT NOT NULL DEFAULT 'strategist_review'
        CHECK (escalation_type IN (
            'strategist_review', 'subject_specialist', 'parent_briefing',
            'intensified_schedule', 'pre_exam_support', 'urgency_check'
        )),
    priority INTEGER NOT NULL DEFAULT 3,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO escalation_rules (rule_code, rule_name, description, trigger_condition_json, escalation_type, priority) VALUES
    ('progress_stall', 'Progress Stalls', 'Student shows flat improvement despite repeated intervention', '{"check":"flat_improvement","window_days":14,"min_interventions":2}', 'strategist_review', 2),
    ('persistent_risk', 'Risk Persists', 'Major weakness remains active beyond defined threshold', '{"check":"risk_age","severity":"high","max_days":21}', 'strategist_review', 2),
    ('high_stakes_near', 'High Stakes Near', 'Exam date close and readiness below target', '{"check":"exam_proximity_gap","max_days":30,"readiness_gap_bp":2000}', 'pre_exam_support', 1),
    ('unusual_pattern', 'Unusual Pattern', 'Student shows abnormal inconsistency or mixed signals', '{"check":"variance_spike","consistency_drop_bp":3000}', 'strategist_review', 3),
    ('parent_concern', 'Parent Concern', 'Parent expresses serious concern through concierge', '{"check":"parent_flag","question_family":"risk"}', 'parent_briefing', 2),
    ('behavior_factor', 'Behavioral Factor', 'Avoidance, low confidence, or panic affecting academic progress', '{"check":"behavioral_risk_active","min_risks":2}', 'strategist_review', 3);

-- ============================================================================
-- 5. Escalation log
-- ============================================================================

CREATE TABLE escalation_events (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    rule_id INTEGER REFERENCES escalation_rules(id),
    escalation_type TEXT NOT NULL,
    trigger_evidence_json TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'acknowledged', 'in_review', 'resolved', 'dismissed')),
    assigned_to TEXT,
    resolution_notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX idx_escalation_events_student ON escalation_events(student_id, status);

-- ============================================================================
-- 6. Readiness dimension weights (configurable)
-- ============================================================================

CREATE TABLE readiness_dimension_weights (
    id INTEGER PRIMARY KEY,
    context TEXT NOT NULL DEFAULT 'default' UNIQUE,
    knowledge_weight INTEGER NOT NULL DEFAULT 1500,
    application_weight INTEGER NOT NULL DEFAULT 1500,
    reasoning_weight INTEGER NOT NULL DEFAULT 1500,
    speed_weight INTEGER NOT NULL DEFAULT 1000,
    memory_weight INTEGER NOT NULL DEFAULT 1500,
    confidence_weight INTEGER NOT NULL DEFAULT 1000,
    consistency_weight INTEGER NOT NULL DEFAULT 1000,
    exam_technique_weight INTEGER NOT NULL DEFAULT 500
);

INSERT INTO readiness_dimension_weights (context) VALUES ('default');
INSERT INTO readiness_dimension_weights (context, speed_weight, exam_technique_weight, reasoning_weight)
    VALUES ('pre_exam', 1500, 1000, 1500);

-- ============================================================================
-- 7. Weekly memo generation tracking
-- ============================================================================

ALTER TABLE weekly_memos ADD COLUMN generation_status TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE weekly_memos ADD COLUMN auto_generated INTEGER NOT NULL DEFAULT 0;
ALTER TABLE weekly_memos ADD COLUMN sent_to_parent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE weekly_memos ADD COLUMN sent_at TEXT;
ALTER TABLE weekly_memos ADD COLUMN parent_comm_id INTEGER;

-- ============================================================================
-- 8. Risk flags additional indexes for performance
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_risk_flags_category ON risk_flags(risk_category, status);
CREATE INDEX IF NOT EXISTS idx_risk_flags_review ON risk_flags(review_at);

-- ============================================================================
-- 9. Concierge suggested prompts (context-aware)
-- ============================================================================

CREATE TABLE concierge_prompt_templates (
    id INTEGER PRIMARY KEY,
    question_family TEXT NOT NULL,
    prompt_text TEXT NOT NULL,
    context_condition TEXT,
    priority_order INTEGER NOT NULL DEFAULT 10,
    is_active INTEGER NOT NULL DEFAULT 1
);

INSERT INTO concierge_prompt_templates (question_family, prompt_text, priority_order) VALUES
    ('status', 'How is my child really doing?', 1),
    ('status', 'Are we on track for our target?', 2),
    ('risk', 'What is the biggest risk right now?', 1),
    ('risk', 'What are we most worried about?', 2),
    ('strategy', 'What changed this week?', 1),
    ('strategy', 'Why are you focusing on this topic?', 2),
    ('forecast', 'Are we on track for BECE?', 1),
    ('forecast', 'What is the predicted readiness?', 2),
    ('action', 'What should my child focus on this weekend?', 1),
    ('action', 'What is the most important thing to do today?', 2),
    ('explanation', 'Why did performance drop this week?', 1),
    ('explanation', 'What is causing the drop in science?', 2);
