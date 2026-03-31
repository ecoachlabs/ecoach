-- idea17: Glossary Lab — examples, misconceptions, formula metadata,
-- audio system, testing modes, mastery progression, confusion pairs,
-- search index, definition/concept metadata.

-- ============================================================================
-- 1. Entry examples (worked examples per knowledge entry)
-- ============================================================================

CREATE TABLE entry_examples (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    sequence_order INTEGER NOT NULL DEFAULT 0,
    example_text TEXT NOT NULL,
    context_type TEXT DEFAULT 'general',
    difficulty_level INTEGER NOT NULL DEFAULT 5000,
    worked_solution_text TEXT,
    is_exam_style INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_entry_examples_entry ON entry_examples(entry_id);

-- ============================================================================
-- 2. Entry misconceptions (common errors per knowledge entry)
-- ============================================================================

CREATE TABLE entry_misconceptions (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    misconception_text TEXT NOT NULL,
    cause_explanation TEXT,
    correction_explanation TEXT,
    confusion_pair_entry_id INTEGER REFERENCES knowledge_entries(id),
    misconception_source TEXT DEFAULT 'curated',
    severity_bp INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_entry_misconceptions_entry ON entry_misconceptions(entry_id);

-- ============================================================================
-- 3. Formula metadata
-- ============================================================================

CREATE TABLE formula_meta (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    formula_expression TEXT NOT NULL,
    formula_speech TEXT,
    variables_json TEXT NOT NULL DEFAULT '[]',
    units_json TEXT,
    when_to_use TEXT,
    when_not_to_use TEXT,
    rearrangements_json TEXT,
    derivation_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entry_id)
);

-- ============================================================================
-- 4. Definition metadata
-- ============================================================================

CREATE TABLE definition_meta (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    definition_text TEXT NOT NULL,
    short_definition TEXT,
    real_world_meaning TEXT,
    non_examples TEXT,
    context_clues TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entry_id)
);

-- ============================================================================
-- 5. Concept metadata
-- ============================================================================

CREATE TABLE concept_meta (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    concept_explanation TEXT NOT NULL,
    intuition_summary TEXT,
    related_visual_keywords TEXT,
    misconception_signals TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entry_id)
);

-- ============================================================================
-- 6. Audio segments (per-entry audio content)
-- ============================================================================

CREATE TABLE entry_audio_segments (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    segment_type TEXT NOT NULL DEFAULT 'definition'
        CHECK (segment_type IN (
            'definition', 'explanation', 'example', 'misconception',
            'transition', 'formula_speech', 'summary', 'test_prompt'
        )),
    script_text TEXT NOT NULL,
    duration_seconds INTEGER,
    teaching_mode TEXT NOT NULL DEFAULT 'standard'
        CHECK (teaching_mode IN ('simple', 'standard', 'technical', 'exam_focused')),
    is_auto_generated INTEGER NOT NULL DEFAULT 1,
    audio_asset_ref TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_audio_segments_entry ON entry_audio_segments(entry_id);

-- ============================================================================
-- 7. Audio programs (radio-style playback)
-- ============================================================================

CREATE TABLE glossary_audio_programs (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    source_type TEXT NOT NULL DEFAULT 'bundle'
        CHECK (source_type IN ('bundle', 'topic', 'search_result', 'weakness_flow', 'custom')),
    source_id INTEGER,
    teaching_mode TEXT NOT NULL DEFAULT 'standard',
    student_id INTEGER REFERENCES accounts(id),
    total_duration_seconds INTEGER NOT NULL DEFAULT 0,
    item_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'ready'
        CHECK (status IN ('generating', 'ready', 'playing', 'completed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE glossary_audio_program_items (
    id INTEGER PRIMARY KEY,
    program_id INTEGER NOT NULL REFERENCES glossary_audio_programs(id) ON DELETE CASCADE,
    sequence_no INTEGER NOT NULL,
    segment_id INTEGER REFERENCES entry_audio_segments(id),
    entry_id INTEGER REFERENCES knowledge_entries(id),
    prompt_text TEXT,
    focus_reason TEXT,
    duration_seconds INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_audio_items_program ON glossary_audio_program_items(program_id);

-- ============================================================================
-- 8. Audio queue state (per-student playback tracking)
-- ============================================================================

CREATE TABLE glossary_audio_queue_state (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    current_program_id INTEGER REFERENCES glossary_audio_programs(id),
    current_position INTEGER NOT NULL DEFAULT 0,
    is_playing INTEGER NOT NULL DEFAULT 0,
    playback_speed REAL NOT NULL DEFAULT 1.0,
    include_examples INTEGER NOT NULL DEFAULT 1,
    include_misconceptions INTEGER NOT NULL DEFAULT 1,
    last_played_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 9. Glossary test sessions
-- ============================================================================

CREATE TABLE glossary_test_sessions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    test_mode TEXT NOT NULL
        CHECK (test_mode IN (
            'recall', 'reverse_recall', 'audio_recall', 'formula_builder',
            'context_recognition', 'confusion_duel', 'intruder_mode',
            'question_signal', 'connection_map', 'speed_recall',
            'application_test', 'fill_gap', 'spot_mistake', 'exam_style'
        )),
    topic_id INTEGER,
    bundle_id INTEGER REFERENCES knowledge_bundles(id),
    entry_count INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER,
    difficulty_level INTEGER NOT NULL DEFAULT 5000,
    recall_score_bp INTEGER,
    recognition_score_bp INTEGER,
    connection_score_bp INTEGER,
    application_score_bp INTEGER,
    retention_score_bp INTEGER,
    confidence_score_bp INTEGER,
    completion_rate_bp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_glossary_tests_student ON glossary_test_sessions(student_id);

-- ============================================================================
-- 10. Glossary test attempts (per-entry attempts)
-- ============================================================================

CREATE TABLE glossary_test_attempts (
    id INTEGER PRIMARY KEY,
    test_session_id INTEGER NOT NULL REFERENCES glossary_test_sessions(id),
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id),
    attempt_no INTEGER NOT NULL DEFAULT 1,
    test_mode TEXT NOT NULL,
    student_response TEXT,
    is_correct INTEGER NOT NULL DEFAULT 0,
    time_seconds INTEGER,
    meaning_recall_bp INTEGER,
    word_recognition_bp INTEGER,
    spelling_accuracy_bp INTEGER,
    formula_recall_bp INTEGER,
    concept_recognition_bp INTEGER,
    relationship_understanding_bp INTEGER,
    confusion_resistance_bp INTEGER,
    context_transfer_bp INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_glossary_attempts_session ON glossary_test_attempts(test_session_id);

-- ============================================================================
-- 11. Glossary entry mastery progression
-- ============================================================================

CREATE TABLE glossary_entry_mastery (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id),
    mastery_state TEXT NOT NULL DEFAULT 'unseen'
        CHECK (mastery_state IN (
            'unseen', 'seen', 'explored', 'understood', 'recalled',
            'recognized', 'connected', 'applied', 'strong', 'mastered', 'at_risk'
        )),
    state_entry_date TEXT NOT NULL DEFAULT (datetime('now')),
    last_advanced_date TEXT,
    consecutive_correct INTEGER NOT NULL DEFAULT 0,
    at_risk_flag INTEGER NOT NULL DEFAULT 0,
    review_count INTEGER NOT NULL DEFAULT 0,
    test_count INTEGER NOT NULL DEFAULT 0,
    test_pass_count INTEGER NOT NULL DEFAULT 0,
    recognition_score_bp INTEGER NOT NULL DEFAULT 0,
    connection_score_bp INTEGER NOT NULL DEFAULT 0,
    application_score_bp INTEGER NOT NULL DEFAULT 0,
    retention_score_bp INTEGER NOT NULL DEFAULT 0,
    spaced_review_due_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, entry_id)
);

CREATE INDEX idx_glossary_mastery_student ON glossary_entry_mastery(student_id, mastery_state);

-- ============================================================================
-- 12. Confusion pairs
-- ============================================================================

CREATE TABLE confusion_pairs (
    id INTEGER PRIMARY KEY,
    entry_id_1 INTEGER NOT NULL REFERENCES knowledge_entries(id),
    entry_id_2 INTEGER NOT NULL REFERENCES knowledge_entries(id),
    distinction_explanation TEXT NOT NULL,
    common_confusion_reason TEXT,
    clue_to_distinguish TEXT,
    example_sentence_1 TEXT,
    example_sentence_2 TEXT,
    confusion_recovery_bundle_id INTEGER REFERENCES knowledge_bundles(id),
    confusion_frequency_bp INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entry_id_1, entry_id_2)
);

CREATE INDEX idx_confusion_pairs_1 ON confusion_pairs(entry_id_1);
CREATE INDEX idx_confusion_pairs_2 ON confusion_pairs(entry_id_2);

-- ============================================================================
-- 13. Neighbor/intruder mappings (for intruder mode testing)
-- ============================================================================

CREATE TABLE neighbor_intruder_mappings (
    id INTEGER PRIMARY KEY,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id),
    neighbor_entry_ids_json TEXT NOT NULL DEFAULT '[]',
    intruder_entry_ids_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entry_id)
);

-- ============================================================================
-- 14. Student entry state extensions
-- ============================================================================

ALTER TABLE student_entry_state ADD COLUMN recognition_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_entry_state ADD COLUMN connection_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_entry_state ADD COLUMN application_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_entry_state ADD COLUMN retention_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_entry_state ADD COLUMN test_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_entry_state ADD COLUMN test_pass_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE student_entry_state ADD COLUMN spaced_review_due_at TEXT;
ALTER TABLE student_entry_state ADD COLUMN at_risk_threshold_date TEXT;

-- ============================================================================
-- 15. Knowledge entries expansion
-- ============================================================================

ALTER TABLE knowledge_entries ADD COLUMN audio_available INTEGER NOT NULL DEFAULT 0;
ALTER TABLE knowledge_entries ADD COLUMN has_formula INTEGER NOT NULL DEFAULT 0;
ALTER TABLE knowledge_entries ADD COLUMN confusion_pair_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE knowledge_entries ADD COLUMN example_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE knowledge_entries ADD COLUMN misconception_count INTEGER NOT NULL DEFAULT 0;
