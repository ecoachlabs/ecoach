-- idea34: Academic DNA runtime blueprint and intervention prescription layer.

CREATE TABLE IF NOT EXISTS diagnostic_subject_blueprints (
    subject_id INTEGER PRIMARY KEY REFERENCES subjects(id) ON DELETE CASCADE,
    blueprint_code TEXT NOT NULL,
    subject_name TEXT NOT NULL,
    session_modes_json TEXT NOT NULL DEFAULT '{}',
    stage_rules_json TEXT NOT NULL DEFAULT '{}',
    item_family_mix_json TEXT NOT NULL DEFAULT '[]',
    routing_contract_json TEXT NOT NULL DEFAULT '{}',
    report_contract_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS diagnostic_item_routing_profiles (
    question_id INTEGER PRIMARY KEY REFERENCES questions(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    family_id INTEGER REFERENCES question_families(id) ON DELETE SET NULL,
    item_family TEXT NOT NULL,
    recognition_suitable INTEGER NOT NULL DEFAULT 0,
    recall_suitable INTEGER NOT NULL DEFAULT 0,
    transfer_suitable INTEGER NOT NULL DEFAULT 0,
    timed_suitable INTEGER NOT NULL DEFAULT 0,
    confidence_prompt TEXT NOT NULL DEFAULT 'sure_not_sure_guessed',
    recommended_stages_json TEXT NOT NULL DEFAULT '[]',
    sibling_variant_modes_json TEXT NOT NULL DEFAULT '[]',
    routing_notes_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_diagnostic_item_routing_subject
    ON diagnostic_item_routing_profiles(subject_id, topic_id, item_family);

CREATE TABLE IF NOT EXISTS intervention_mode_library (
    mode_code TEXT PRIMARY KEY,
    mode_name TEXT NOT NULL,
    mode_family TEXT NOT NULL,
    objective TEXT NOT NULL,
    entry_rules_json TEXT NOT NULL DEFAULT '[]',
    contraindications_json TEXT NOT NULL DEFAULT '[]',
    success_signals_json TEXT NOT NULL DEFAULT '[]',
    next_modes_json TEXT NOT NULL DEFAULT '[]',
    report_translation TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS diagnostic_problem_cause_fix_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    topic_name TEXT NOT NULL,
    problem_summary TEXT NOT NULL,
    cause_summary TEXT NOT NULL,
    fix_summary TEXT NOT NULL,
    confidence_score_bp INTEGER NOT NULL DEFAULT 0,
    impact_score_bp INTEGER NOT NULL DEFAULT 0,
    unlock_summary TEXT,
    evidence_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(diagnostic_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_diagnostic_problem_cards_diagnostic
    ON diagnostic_problem_cause_fix_cards(diagnostic_id, confidence_score_bp DESC);

CREATE TABLE IF NOT EXISTS diagnostic_intervention_prescriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    topic_name TEXT NOT NULL,
    primary_mode_code TEXT NOT NULL,
    support_mode_code TEXT,
    recheck_mode_code TEXT,
    mode_chain_json TEXT NOT NULL DEFAULT '[]',
    contraindications_json TEXT NOT NULL DEFAULT '[]',
    success_signals_json TEXT NOT NULL DEFAULT '[]',
    confidence_score_bp INTEGER NOT NULL DEFAULT 0,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(diagnostic_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_diagnostic_intervention_prescriptions_diagnostic
    ON diagnostic_intervention_prescriptions(diagnostic_id, confidence_score_bp DESC);

INSERT OR REPLACE INTO intervention_mode_library (
    mode_code, mode_name, mode_family, objective, entry_rules_json,
    contraindications_json, success_signals_json, next_modes_json,
    report_translation, sort_order, is_active
) VALUES
    (
        'concept_rebuild',
        'Concept Rebuild',
        'concept_repair',
        'Rebuild the meaning structure before more practice volume.',
        '["knowledge_gap","weak recall","weak transfer"]',
        '["timed_only_fix","advanced pressure work"]',
        '["explains the core idea","answers one direct item independently"]',
        '["compare_contrast_drill","guided_transfer_drill"]',
        'Rebuild the core idea with slower, guided repair first.',
        10,
        1
    ),
    (
        'compare_contrast_drill',
        'Compare-Contrast Drill',
        'misconception_repair',
        'Separate easily-confused ideas with explicit why-this-not-that reasoning.',
        '["conceptual_confusion","misconception pattern","label confusion"]',
        '["heavy timed pressure before meaning is clear"]',
        '["can explain the decisive difference","stops mixing paired ideas"]',
        '["guided_transfer_drill","stability_recheck_cycle"]',
        'Use side-by-side comparisons until the confusion stops.',
        20,
        1
    ),
    (
        'example_non_example_drill',
        'Example vs Non-Example Drill',
        'concept_repair',
        'Clarify category boundaries using valid and invalid cases.',
        '["boundary confusion","classification weakness"]',
        '["high-speed drilling"]',
        '["correctly sorts examples and non-examples"]',
        '["compare_contrast_drill","guided_transfer_drill"]',
        'Clarify what counts and what does not count.',
        30,
        1
    ),
    (
        'misconception_correction_set',
        'Misconception Correction Set',
        'misconception_repair',
        'Directly attack a persistent wrong belief and rebuild the correct model.',
        '["high-confidence wrong answers","known misconception trigger"]',
        '["untagged random drilling"]',
        '["identifies the old wrong rule","states the corrected rule"]',
        '["stability_recheck_cycle","guided_transfer_drill"]',
        'Correct a specific wrong belief before moving on.',
        40,
        1
    ),
    (
        'error_diagnosis_drill',
        'Error Diagnosis Drill',
        'metacognition',
        'Make the learner spot and explain what went wrong in worked attempts.',
        '["execution drift","careless loss","procedure slips"]',
        '["concept overload while the steps are still unstable"]',
        '["can name the exact error","avoids the same step failure twice"]',
        '["guided_worked_step_repair","stability_recheck_cycle"]',
        'Slow down and diagnose the exact mistake pattern.',
        50,
        1
    ),
    (
        'translation_scaffold',
        'Translation Scaffold',
        'application',
        'Translate the concept across forms before testing harder transfer.',
        '["representation gap","wording dependence","application hesitation"]',
        '["hard transfer before one supported translation succeeds"]',
        '["maps the idea across two forms"]',
        '["guided_transfer_drill","stability_recheck_cycle"]',
        'Bridge the idea across different forms before harder transfer.',
        60,
        1
    ),
    (
        'guided_transfer_drill',
        'Guided Transfer Drill',
        'transfer',
        'Test whether the idea survives a changed context with support.',
        '["transfer weakness","fragile direct success"]',
        '["heavy timing before transfer is stable"]',
        '["solves a changed-form sibling correctly"]',
        '["stability_recheck_cycle","pressure_ladder"]',
        'Test whether understanding survives a new form of the same idea.',
        70,
        1
    ),
    (
        'recall_probe',
        'Recall Probe',
        'retrieval',
        'Check whether the learner can pull the idea from memory without cues.',
        '["memory decay","cue dependence","weak retrieval"]',
        '["long timed bursts before recall returns"]',
        '["retrieves the idea without hints"]',
        '["stability_recheck_cycle","secure_zone_reinforcement"]',
        'Strengthen memory retrieval before harder mixed practice.',
        80,
        1
    ),
    (
        'stability_recheck_cycle',
        'Stability Recheck Cycle',
        'stability',
        'Confirm that success repeats across fresh items and short delays.',
        '["fragile success","single-hit understanding"]',
        '["premature exit after one correct answer"]',
        '["repeats success on fresh siblings","holds accuracy after a small gap"]',
        '["guided_transfer_drill","pressure_ladder","secure_zone_reinforcement"]',
        'Confirm that the improvement is stable, not accidental.',
        90,
        1
    ),
    (
        'fluency_burst',
        'Fluency Burst',
        'fluency',
        'Tight, short repetitions that improve speed after the concept is stable.',
        '["slow but correct","pressure mostly speed-driven"]',
        '["core meaning still weak","active misconception"]',
        '["response time shrinks while accuracy stays stable"]',
        '["pressure_ladder","secure_zone_reinforcement"]',
        'Build faster execution only after understanding is steady.',
        100,
        1
    ),
    (
        'pressure_ladder',
        'Pressure Ladder',
        'pressure',
        'Reintroduce timing in layers once calm success is reliable.',
        '["pressure collapse","timed-only breakdown"]',
        '["knowledge gap","fresh conceptual confusion"]',
        '["timed delta shrinks","confidence remains steady under time"]',
        '["stability_recheck_cycle","secure_zone_reinforcement"]',
        'Rebuild timed confidence gradually instead of forcing full pressure immediately.',
        110,
        1
    ),
    (
        'guided_worked_step_repair',
        'Guided Worked-Step Repair',
        'procedure',
        'Repair procedure and sequencing with explicit step visibility.',
        '["execution drift","procedural instability"]',
        '["random untargeted repetition"]',
        '["can complete the full sequence without step loss"]',
        '["error_diagnosis_drill","stability_recheck_cycle"]',
        'Repair the worked process step by step.',
        120,
        1
    ),
    (
        'confidence_reflection_check',
        'Confidence Reflection Check',
        'calibration',
        'Align certainty with reality so overconfidence and underconfidence are visible.',
        '["confidence distortion","guessed_correct streak","high-confidence wrong"]',
        '["timed escalation while confidence is distorted"]',
        '["confidence labels become more calibrated"]',
        '["misconception_correction_set","stability_recheck_cycle"]',
        'Calibrate confidence so the learner can trust the signal again.',
        130,
        1
    ),
    (
        'mixed_root_repair_set',
        'Mixed Root Repair Set',
        'integrated_repair',
        'Combine two repair angles when the weakness comes from more than one cause.',
        '["multi-cause weakness","uncertain primary cause"]',
        '["single-mode repetition when evidence is mixed"]',
        '["one cause becomes clearly dominant","the mixed error rate drops"]',
        '["guided_transfer_drill","stability_recheck_cycle"]',
        'Use a blended repair plan when the weakness is not coming from one cause alone.',
        140,
        1
    ),
    (
        'secure_zone_reinforcement',
        'Secure Zone Reinforcement',
        'confidence_build',
        'Preserve and reinforce the learner’s strong zones while repair happens elsewhere.',
        '["strong-but-fragile gains","morale support needed"]',
        '["using easy wins to avoid the real weak area"]',
        '["recent secure successes remain visible"]',
        '["guided_transfer_drill","pressure_ladder"]',
        'Keep solid areas active so the report and plan are not weakness-only.',
        150,
        1
    );
