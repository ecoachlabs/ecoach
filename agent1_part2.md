# eCoach — Agent 1 Part 2: Migrations 012-022 and All Rust Types
## Generated: 2026-03-29

---

# PART 2B: DATABASE SCHEMA — MIGRATIONS 012 TO 022

---

## 012_reporting.sql

```sql
-- 012_reporting.sql
-- Readiness claims, proofs, danger zones, reports, digests, parent snapshots, memos

CREATE TABLE readiness_claims (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    claim_type TEXT NOT NULL DEFAULT 'subject'
        CHECK (claim_type IN ('topic', 'subject', 'full_exam', 'custom_scope')),
    claimed_at TEXT NOT NULL DEFAULT (datetime('now')),
    claim_basis TEXT NOT NULL,                        -- JSON: {mastery_bp, coverage_bp, mock_score_bp, ...}
    readiness_score INTEGER NOT NULL DEFAULT 0,       -- basis points at time of claim
    predicted_exam_score INTEGER,                     -- basis points
    confidence_level INTEGER NOT NULL DEFAULT 0,      -- basis points
    scope_json TEXT NOT NULL DEFAULT '{}',            -- topics or nodes in scope
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'proven', 'failed', 'expired', 'withdrawn')),
    expires_at TEXT,
    proven_at TEXT,
    failed_at TEXT,
    failure_reason TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE readiness_proofs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    claim_id INTEGER NOT NULL REFERENCES readiness_claims(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    proof_type TEXT NOT NULL
        CHECK (proof_type IN (
            'mock_score', 'topic_mastery_gate', 'coverage_gate',
            'retention_check', 'speed_gate', 'consistency_gate',
            'misconception_clearance', 'timed_pressure_pass'
        )),
    proof_value INTEGER NOT NULL DEFAULT 0,           -- basis points
    threshold_required INTEGER NOT NULL DEFAULT 0,    -- basis points
    passed INTEGER NOT NULL DEFAULT 0,
    evidence_session_id INTEGER,                      -- FK to sessions if applicable
    evidence_json TEXT NOT NULL DEFAULT '{}',
    evaluated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE danger_zones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    zone_type TEXT NOT NULL
        CHECK (zone_type IN (
            'critical_gap', 'memory_decay', 'misconception_active',
            'coverage_hole', 'speed_collapse', 'consistency_failure',
            'exam_anxiety', 'prerequisite_block', 'inactivity'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    node_id INTEGER REFERENCES academic_nodes(id),
    severity TEXT NOT NULL DEFAULT 'watch'
        CHECK (severity IN ('watch', 'active', 'urgent', 'critical')),
    severity_score INTEGER NOT NULL DEFAULT 0,        -- basis points
    description TEXT NOT NULL,
    detail_json TEXT NOT NULL DEFAULT '{}',
    first_detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_confirmed_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT,
    resolution_note TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    is_acknowledged INTEGER NOT NULL DEFAULT 0,
    snooze_until TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE readiness_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    report_type TEXT NOT NULL
        CHECK (report_type IN ('weekly', 'checkpoint', 'pre_exam', 'post_mock', 'milestone', 'ad_hoc')),
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    overall_readiness_bp INTEGER NOT NULL DEFAULT 0,
    predicted_score_bp INTEGER,
    coverage_bp INTEGER NOT NULL DEFAULT 0,
    mastery_bp INTEGER NOT NULL DEFAULT 0,
    retention_bp INTEGER NOT NULL DEFAULT 0,
    mock_performance_bp INTEGER,
    speed_bp INTEGER,
    consistency_bp INTEGER,
    trend_direction TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_direction IN ('improving', 'stable', 'declining', 'critical')),
    active_danger_zone_count INTEGER NOT NULL DEFAULT 0,
    critical_gap_count INTEGER NOT NULL DEFAULT 0,
    sessions_completed INTEGER NOT NULL DEFAULT 0,
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    summary_text TEXT,
    coach_notes TEXT,
    report_data_json TEXT NOT NULL DEFAULT '{}',      -- full snapshot
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE weekly_digests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    week_start TEXT NOT NULL,
    week_end TEXT NOT NULL,
    sessions_count INTEGER NOT NULL DEFAULT 0,
    total_study_minutes INTEGER NOT NULL DEFAULT 0,
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    topics_touched INTEGER NOT NULL DEFAULT 0,
    topics_improved INTEGER NOT NULL DEFAULT 0,
    topics_declined INTEGER NOT NULL DEFAULT 0,
    new_mastery_unlocks INTEGER NOT NULL DEFAULT 0,
    streak_days INTEGER NOT NULL DEFAULT 0,
    best_session_score_bp INTEGER,
    worst_subject_id INTEGER REFERENCES subjects(id),
    best_subject_id INTEGER REFERENCES subjects(id),
    danger_zones_opened INTEGER NOT NULL DEFAULT 0,
    danger_zones_resolved INTEGER NOT NULL DEFAULT 0,
    readiness_delta_bp INTEGER NOT NULL DEFAULT 0,    -- change from last week
    highlight_text TEXT,
    digest_data_json TEXT NOT NULL DEFAULT '{}',
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, week_start)
);

CREATE TABLE parent_report_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    snapshot_type TEXT NOT NULL DEFAULT 'weekly'
        CHECK (snapshot_type IN ('weekly', 'monthly', 'term', 'pre_exam', 'alert_triggered')),
    period_label TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    plain_language_summary TEXT NOT NULL,
    status_signal TEXT NOT NULL DEFAULT 'ok'
        CHECK (status_signal IN ('ok', 'attention', 'urgent', 'critical')),
    readiness_level TEXT NOT NULL DEFAULT 'unknown'
        CHECK (readiness_level IN ('unknown', 'low', 'moderate', 'good', 'exam_ready')),
    recommended_actions TEXT,                         -- JSON array of action strings
    alert_count INTEGER NOT NULL DEFAULT 0,
    snapshot_data_json TEXT NOT NULL DEFAULT '{}',
    delivered_at TEXT,
    viewed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE memo_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    memo_type TEXT NOT NULL
        CHECK (memo_type IN (
            'coach_note', 'milestone', 'parent_alert',
            'system_flag', 'student_note', 'session_summary'
        )),
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    related_entity_type TEXT,                         -- 'topic', 'session', 'mock', etc.
    related_entity_id INTEGER,
    severity TEXT DEFAULT 'info'
        CHECK (severity IN ('info', 'watch', 'active', 'urgent', 'critical')),
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    expires_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_readiness_claims_student ON readiness_claims(student_id);
CREATE INDEX idx_readiness_claims_status ON readiness_claims(status);
CREATE INDEX idx_readiness_proofs_claim ON readiness_proofs(claim_id);
CREATE INDEX idx_readiness_proofs_student ON readiness_proofs(student_id);
CREATE INDEX idx_danger_zones_student ON danger_zones(student_id);
CREATE INDEX idx_danger_zones_active ON danger_zones(is_active, severity);
CREATE INDEX idx_danger_zones_topic ON danger_zones(topic_id);
CREATE INDEX idx_readiness_reports_student ON readiness_reports(student_id);
CREATE INDEX idx_readiness_reports_type ON readiness_reports(report_type);
CREATE INDEX idx_weekly_digests_student ON weekly_digests(student_id);
CREATE INDEX idx_weekly_digests_week ON weekly_digests(week_start);
CREATE INDEX idx_parent_report_snapshots_parent ON parent_report_snapshots(parent_id);
CREATE INDEX idx_parent_report_snapshots_student ON parent_report_snapshots(student_id);
CREATE INDEX idx_memo_records_student ON memo_records(student_id);
CREATE INDEX idx_memo_records_type ON memo_records(memo_type);
```

---

## 013_glossary.sql

```sql
-- 013_glossary.sql
-- Glossary entries, relations, bundles, bundle items, search index, question links, formulas

CREATE TABLE glossary_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_key TEXT NOT NULL UNIQUE,                   -- e.g. 'fraction_numerator'
    term TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    entry_type TEXT NOT NULL DEFAULT 'definition'
        CHECK (entry_type IN (
            'definition', 'concept', 'formula', 'procedure',
            'rule', 'theorem', 'example_term', 'abbreviation'
        )),
    -- Content layers
    plain_definition TEXT NOT NULL,
    extended_explanation TEXT,
    exam_tip TEXT,
    common_mistake TEXT,
    mnemonic TEXT,
    -- Media
    has_diagram INTEGER NOT NULL DEFAULT 0,
    diagram_path TEXT,
    has_audio INTEGER NOT NULL DEFAULT 0,
    audio_path TEXT,
    -- Classification
    cognitive_level TEXT
        CHECK (cognitive_level IN ('recall', 'understanding', 'application', 'analysis')),
    difficulty_level INTEGER NOT NULL DEFAULT 5000,   -- basis points
    grade_levels TEXT NOT NULL DEFAULT '[]',          -- JSON array: ['JHS1','JHS2','JHS3']
    -- Quality
    is_active INTEGER NOT NULL DEFAULT 1,
    is_exam_critical INTEGER NOT NULL DEFAULT 0,
    pack_id TEXT REFERENCES content_packs(pack_id),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE glossary_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    to_entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL
        CHECK (relation_type IN (
            'prerequisite', 'related', 'contrast', 'extends',
            'part_of', 'example_of', 'synonym', 'antonym'
        )),
    strength INTEGER NOT NULL DEFAULT 5000,           -- basis points
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(from_entry_id, to_entry_id, relation_type)
);

CREATE TABLE glossary_bundles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bundle_key TEXT NOT NULL UNIQUE,
    bundle_name TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    bundle_type TEXT NOT NULL DEFAULT 'topic_cluster'
        CHECK (bundle_type IN (
            'topic_cluster', 'exam_vocab', 'formula_set',
            'contrast_group', 'prerequisite_chain', 'custom'
        )),
    description TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    pack_id TEXT REFERENCES content_packs(pack_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE glossary_bundle_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bundle_id INTEGER NOT NULL REFERENCES glossary_bundles(id) ON DELETE CASCADE,
    entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_anchor INTEGER NOT NULL DEFAULT 0,             -- anchor term of the bundle
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(bundle_id, entry_id)
);

CREATE TABLE glossary_search_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    search_text TEXT NOT NULL,                        -- flattened searchable text
    tokens TEXT NOT NULL DEFAULT '[]',                -- JSON array of normalised tokens
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- FTS virtual table for glossary full-text search
CREATE VIRTUAL TABLE glossary_fts USING fts5(
    entry_id UNINDEXED,
    term,
    plain_definition,
    extended_explanation,
    exam_tip,
    content='glossary_entries',
    content_rowid='id'
);

CREATE TABLE question_glossary_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    link_type TEXT NOT NULL DEFAULT 'uses_term'
        CHECK (link_type IN (
            'uses_term', 'tests_term', 'requires_definition',
            'distractor_exploits', 'stem_contains'
        )),
    relevance_score INTEGER NOT NULL DEFAULT 5000,    -- basis points
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(question_id, entry_id, link_type)
);

CREATE TABLE formula_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    formula_key TEXT NOT NULL UNIQUE,
    formula_name TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    -- Representations
    latex_expression TEXT NOT NULL,
    plain_text_expression TEXT NOT NULL,
    speech_readable TEXT NOT NULL,                    -- "area equals length times width"
    worked_example TEXT,
    -- Variables
    variables_json TEXT NOT NULL DEFAULT '[]',        -- [{symbol, name, unit, description}]
    -- Classification
    formula_category TEXT NOT NULL DEFAULT 'core'
        CHECK (formula_category IN (
            'core', 'derived', 'shortcut', 'conversion',
            'definition_form', 'rearrangement', 'approximation'
        )),
    is_exam_sheet_formula INTEGER NOT NULL DEFAULT 0, -- appears on BECE formula sheet
    is_must_memorise INTEGER NOT NULL DEFAULT 0,
    difficulty_level INTEGER NOT NULL DEFAULT 5000,
    grade_levels TEXT NOT NULL DEFAULT '[]',
    -- Media
    has_diagram INTEGER NOT NULL DEFAULT 0,
    diagram_path TEXT,
    -- Quality
    is_active INTEGER NOT NULL DEFAULT 1,
    pack_id TEXT REFERENCES content_packs(pack_id),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_glossary_entries_subject ON glossary_entries(subject_id);
CREATE INDEX idx_glossary_entries_topic ON glossary_entries(topic_id);
CREATE INDEX idx_glossary_entries_type ON glossary_entries(entry_type);
CREATE INDEX idx_glossary_entries_active ON glossary_entries(is_active);
CREATE INDEX idx_glossary_relations_from ON glossary_relations(from_entry_id);
CREATE INDEX idx_glossary_relations_to ON glossary_relations(to_entry_id);
CREATE INDEX idx_glossary_bundle_items_bundle ON glossary_bundle_items(bundle_id);
CREATE INDEX idx_glossary_search_entry ON glossary_search_index(entry_id);
CREATE INDEX idx_question_glossary_links_question ON question_glossary_links(question_id);
CREATE INDEX idx_question_glossary_links_entry ON question_glossary_links(entry_id);
CREATE INDEX idx_formula_entries_subject ON formula_entries(subject_id);
CREATE INDEX idx_formula_entries_topic ON formula_entries(topic_id);
```

---

## 014_library.sql

```sql
-- 014_library.sql
-- Library items, shelves, shelf items, mistake bank, memory shelf, revision packs, library cards

CREATE TABLE library_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL
        CHECK (item_type IN (
            'saved_question', 'saved_explanation', 'saved_glossary_entry',
            'saved_formula', 'worked_example', 'coach_note',
            'custom_note', 'mistake_capture', 'highlight'
        )),
    title TEXT NOT NULL,
    body TEXT,
    source_type TEXT,                                 -- 'question', 'glossary', 'formula', etc.
    source_id INTEGER,                                -- FK to source entity
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    tags TEXT NOT NULL DEFAULT '[]',                  -- JSON array of tag strings
    is_starred INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    review_due_at TEXT,
    last_reviewed_at TEXT,
    review_count INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE library_shelves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    shelf_name TEXT NOT NULL,
    shelf_type TEXT NOT NULL DEFAULT 'custom'
        CHECK (shelf_type IN (
            'custom', 'mistakes', 'memory', 'formulas',
            'glossary', 'revision', 'starred', 'coach_picks'
        )),
    description TEXT,
    color_hex TEXT,
    icon_name TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_system INTEGER NOT NULL DEFAULT 0,             -- 1 = auto-created, not deletable
    is_archived INTEGER NOT NULL DEFAULT 0,
    item_count INTEGER NOT NULL DEFAULT 0,            -- denormalised for display
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, shelf_name)
);

CREATE TABLE shelf_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shelf_id INTEGER NOT NULL REFERENCES library_shelves(id) ON DELETE CASCADE,
    item_id INTEGER NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(shelf_id, item_id)
);

CREATE TABLE mistake_bank_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    question_id INTEGER NOT NULL REFERENCES questions(id),
    attempt_id INTEGER REFERENCES student_question_attempts(id),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    -- What went wrong
    error_type TEXT NOT NULL
        CHECK (error_type IN (
            'forgot_concept', 'misunderstood_wording', 'calculation_slip',
            'guessed', 'panic_under_time', 'chose_familiar_distractor',
            'wrong_first_step', 'incomplete_reasoning', 'misread_units',
            'careless_error'
        )),
    selected_option_id INTEGER REFERENCES question_options(id),
    correct_option_id INTEGER REFERENCES question_options(id),
    misconception_id INTEGER REFERENCES misconception_patterns(id),
    student_explanation TEXT,                         -- student's own note about why they got it wrong
    coach_diagnosis TEXT,
    repair_status TEXT NOT NULL DEFAULT 'unresolved'
        CHECK (repair_status IN (
            'unresolved', 'in_progress', 'repaired', 'relapsed', 'mastered'
        )),
    repair_attempts INTEGER NOT NULL DEFAULT 0,
    last_repair_attempt_at TEXT,
    resolved_at TEXT,
    priority_score INTEGER NOT NULL DEFAULT 5000,     -- basis points, higher = revisit sooner
    review_due_at TEXT,
    times_reviewed INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    library_item_id INTEGER REFERENCES library_items(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE memory_shelf_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL
        CHECK (item_type IN (
            'glossary_entry', 'formula', 'concept_summary',
            'mnemonic', 'procedure_steps', 'fact'
        )),
    source_id INTEGER NOT NULL,                       -- FK to glossary_entries or formula_entries
    source_type TEXT NOT NULL,
    title TEXT NOT NULL,
    memory_cue TEXT,                                  -- hint for recall
    memory_state TEXT NOT NULL DEFAULT 'fresh'
        CHECK (memory_state IN (
            'fresh', 'consolidating', 'stable', 'at_risk',
            'decaying', 'critical', 'forgotten', 'rescued',
            'reinforced', 'transferred', 'dormant', 'archived'
        )),
    memory_strength_bp INTEGER NOT NULL DEFAULT 5000,
    last_recall_at TEXT,
    next_review_at TEXT NOT NULL,
    review_interval_days REAL NOT NULL DEFAULT 1.0,
    easiness_factor REAL NOT NULL DEFAULT 2.5,        -- SM-2 / custom spaced rep factor
    consecutive_correct INTEGER NOT NULL DEFAULT 0,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    correct_reviews INTEGER NOT NULL DEFAULT 0,
    decay_rate_per_day REAL NOT NULL DEFAULT 0.1,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE revision_pack_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    pack_name TEXT NOT NULL,
    pack_type TEXT NOT NULL DEFAULT 'custom'
        CHECK (pack_type IN (
            'custom', 'topic_summary', 'exam_countdown',
            'weak_areas', 'coach_generated', 'mistake_compilation'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    item_type TEXT NOT NULL
        CHECK (item_type IN (
            'question', 'glossary_entry', 'formula',
            'concept_note', 'worked_example', 'trap_pair'
        )),
    item_id INTEGER NOT NULL,                         -- FK to respective table
    display_order INTEGER NOT NULL DEFAULT 0,
    is_completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    pack_label TEXT NOT NULL DEFAULT 'default',       -- group within a pack
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE library_card_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    total_items INTEGER NOT NULL DEFAULT 0,
    starred_items INTEGER NOT NULL DEFAULT 0,
    mistake_bank_size INTEGER NOT NULL DEFAULT 0,
    resolved_mistakes INTEGER NOT NULL DEFAULT 0,
    memory_shelf_size INTEGER NOT NULL DEFAULT 0,
    due_for_review_count INTEGER NOT NULL DEFAULT 0,
    revision_packs_count INTEGER NOT NULL DEFAULT 0,
    last_library_visit_at TEXT,
    most_visited_shelf_id INTEGER REFERENCES library_shelves(id),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

CREATE INDEX idx_library_items_student ON library_items(student_id);
CREATE INDEX idx_library_items_type ON library_items(item_type);
CREATE INDEX idx_library_items_topic ON library_items(topic_id);
CREATE INDEX idx_library_shelves_student ON library_shelves(student_id);
CREATE INDEX idx_shelf_items_shelf ON shelf_items(shelf_id);
CREATE INDEX idx_shelf_items_item ON shelf_items(item_id);
CREATE INDEX idx_mistake_bank_student ON mistake_bank_entries(student_id);
CREATE INDEX idx_mistake_bank_question ON mistake_bank_entries(question_id);
CREATE INDEX idx_mistake_bank_status ON mistake_bank_entries(repair_status);
CREATE INDEX idx_mistake_bank_topic ON mistake_bank_entries(topic_id);
CREATE INDEX idx_memory_shelf_student ON memory_shelf_entries(student_id);
CREATE INDEX idx_memory_shelf_state ON memory_shelf_entries(memory_state);
CREATE INDEX idx_memory_shelf_review ON memory_shelf_entries(next_review_at);
CREATE INDEX idx_revision_pack_student ON revision_pack_items(student_id);
```

---

## 015_games.sql

```sql
-- 015_games.sql
-- Game sessions, events, tug-of-war states, mindstack states, leaderboards, rewards

CREATE TABLE game_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    game_type TEXT NOT NULL
        CHECK (game_type IN ('tug_of_war', 'mindstack', 'speed_race', 'beat_yesterday', 'streak_challenge')),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'completed', 'abandoned', 'timed_out')),
    -- Timing
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    total_duration_ms INTEGER,
    -- Scoring
    initial_score INTEGER NOT NULL DEFAULT 0,
    final_score INTEGER,
    high_score INTEGER,
    score_delta INTEGER,                              -- final - initial
    -- Game-specific state reference
    state_snapshot_json TEXT NOT NULL DEFAULT '{}',
    -- Questions
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    streak_peak INTEGER NOT NULL DEFAULT 0,
    -- Rewards
    xp_earned INTEGER NOT NULL DEFAULT 0,
    badges_earned TEXT NOT NULL DEFAULT '[]',         -- JSON array
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE game_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    event_type TEXT NOT NULL
        CHECK (event_type IN (
            'question_answered', 'streak_gained', 'streak_broken',
            'power_used', 'level_up', 'score_change', 'pause',
            'resume', 'hint_used', 'tug_advance', 'tug_retreat',
            'stack_push', 'stack_collapse', 'time_warning', 'game_over'
        )),
    occurred_at TEXT NOT NULL DEFAULT (datetime('now')),
    question_id INTEGER REFERENCES questions(id),
    is_correct INTEGER,
    response_time_ms INTEGER,
    score_before INTEGER,
    score_after INTEGER,
    streak_before INTEGER,
    streak_after INTEGER,
    event_data_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE tug_of_war_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    -- Rope position: 0 = fully opponent side, 10000 = fully student side (basis points)
    rope_position_bp INTEGER NOT NULL DEFAULT 5000,
    student_pulls INTEGER NOT NULL DEFAULT 0,
    opponent_pulls INTEGER NOT NULL DEFAULT 0,
    current_streak INTEGER NOT NULL DEFAULT 0,
    opponent_streak INTEGER NOT NULL DEFAULT 0,
    -- Opponent model
    opponent_type TEXT NOT NULL DEFAULT 'adaptive_ai'
        CHECK (opponent_type IN ('adaptive_ai', 'historical_self', 'class_average', 'elite_target')),
    opponent_difficulty_bp INTEGER NOT NULL DEFAULT 5000,
    -- Round tracking
    current_round INTEGER NOT NULL DEFAULT 1,
    total_rounds INTEGER NOT NULL DEFAULT 10,
    rounds_won INTEGER NOT NULL DEFAULT 0,
    rounds_lost INTEGER NOT NULL DEFAULT 0,
    -- State
    phase TEXT NOT NULL DEFAULT 'in_progress'
        CHECK (phase IN ('in_progress', 'student_winning', 'opponent_winning', 'tiebreaker', 'complete')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE mindstack_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    -- Stack metaphor: answer correctly to add blocks, wrong = tower wobbles / collapses
    stack_height INTEGER NOT NULL DEFAULT 0,
    max_stack_achieved INTEGER NOT NULL DEFAULT 0,
    wobble_level INTEGER NOT NULL DEFAULT 0,          -- 0-3: 3 = about to collapse
    collapses INTEGER NOT NULL DEFAULT 0,
    -- Difficulty escalation
    current_difficulty_bp INTEGER NOT NULL DEFAULT 3000,
    difficulty_ceiling_bp INTEGER NOT NULL DEFAULT 9500,
    difficulty_step_bp INTEGER NOT NULL DEFAULT 500,
    -- Topics in current stack rotation
    active_topic_ids TEXT NOT NULL DEFAULT '[]',      -- JSON array
    -- State
    phase TEXT NOT NULL DEFAULT 'building'
        CHECK (phase IN ('building', 'wobbling', 'collapsed', 'complete', 'perfect_run')),
    is_perfect_run INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE game_leaderboards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    board_type TEXT NOT NULL
        CHECK (board_type IN ('all_time', 'weekly', 'subject', 'topic', 'game_type')),
    game_type TEXT,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    period_label TEXT,                                -- e.g. '2026-W13'
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    rank INTEGER NOT NULL DEFAULT 0,
    score INTEGER NOT NULL DEFAULT 0,
    sessions_count INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(board_type, game_type, subject_id, topic_id, period_label, student_id)
);

CREATE TABLE game_rewards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    session_id INTEGER REFERENCES game_sessions(id),
    reward_type TEXT NOT NULL
        CHECK (reward_type IN (
            'xp', 'badge', 'streak_shield', 'power_up',
            'title', 'milestone_unlock', 'bonus_hint'
        )),
    reward_key TEXT NOT NULL,                         -- e.g. 'badge_first_tug_win'
    reward_label TEXT NOT NULL,
    reward_value INTEGER NOT NULL DEFAULT 0,          -- XP amount or quantity
    reward_data_json TEXT NOT NULL DEFAULT '{}',
    earned_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_claimed INTEGER NOT NULL DEFAULT 0,
    claimed_at TEXT
);

CREATE INDEX idx_game_sessions_student ON game_sessions(student_id);
CREATE INDEX idx_game_sessions_type ON game_sessions(game_type);
CREATE INDEX idx_game_sessions_status ON game_sessions(status);
CREATE INDEX idx_game_events_session ON game_events(session_id);
CREATE INDEX idx_game_events_student ON game_events(student_id);
CREATE INDEX idx_game_events_type ON game_events(event_type);
CREATE INDEX idx_tug_states_session ON tug_of_war_states(session_id);
CREATE INDEX idx_mindstack_states_session ON mindstack_states(session_id);
CREATE INDEX idx_game_leaderboards_board ON game_leaderboards(board_type, game_type);
CREATE INDEX idx_game_rewards_student ON game_rewards(student_id);
CREATE INDEX idx_game_rewards_type ON game_rewards(reward_type);
```

---

## 016_past_papers.sql

```sql
-- 016_past_papers.sql
-- Past paper sources, parsed questions, recurrence records, examiner patterns, blueprints

CREATE TABLE past_paper_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    paper_code TEXT NOT NULL UNIQUE,                  -- e.g. 'BECE_MATH_2023_P1'
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    exam_board TEXT NOT NULL DEFAULT 'WAEC',
    exam_name TEXT NOT NULL DEFAULT 'BECE',
    exam_year INTEGER NOT NULL,
    paper_number INTEGER NOT NULL DEFAULT 1,
    paper_section TEXT,                               -- 'Objective', 'Essay', 'Theory'
    total_questions INTEGER NOT NULL DEFAULT 0,
    total_marks INTEGER NOT NULL DEFAULT 0,
    time_allowed_minutes INTEGER,
    source_document_path TEXT,
    ocr_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (ocr_status IN ('pending', 'processing', 'complete', 'failed', 'manual')),
    ingestion_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (ingestion_status IN ('pending', 'parsed', 'mapped', 'verified', 'active', 'retired')),
    ingested_at TEXT,
    verified_by TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE paper_questions_parsed (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES past_paper_sources(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id),     -- NULL until mapped to question bank
    position_in_paper INTEGER NOT NULL,
    section TEXT,
    raw_stem_text TEXT NOT NULL,
    raw_options_json TEXT,                            -- JSON array of {label, text}
    correct_option_label TEXT,
    marks INTEGER NOT NULL DEFAULT 1,
    -- Mapping metadata
    mapping_status TEXT NOT NULL DEFAULT 'unmapped'
        CHECK (mapping_status IN ('unmapped', 'auto_mapped', 'manual_mapped', 'conflict', 'new_question')),
    mapped_topic_id INTEGER REFERENCES topics(id),
    mapped_node_ids TEXT NOT NULL DEFAULT '[]',       -- JSON array
    mapping_confidence INTEGER NOT NULL DEFAULT 0,    -- basis points
    -- Classification
    difficulty_estimate INTEGER,                      -- basis points
    question_intent TEXT,
    representation_type TEXT
        CHECK (representation_type IN ('text', 'diagram', 'graph', 'table', 'mixed')),
    -- Intelligence
    intel_snapshot_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE question_recurrence_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER REFERENCES questions(id),
    family_id INTEGER REFERENCES question_families(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    node_id INTEGER REFERENCES academic_nodes(id),
    -- Frequency data
    first_seen_year INTEGER NOT NULL,
    last_seen_year INTEGER NOT NULL,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    years_appeared TEXT NOT NULL DEFAULT '[]',        -- JSON array of years
    papers_appeared TEXT NOT NULL DEFAULT '[]',       -- JSON array of paper_codes
    -- Probability
    recurrence_probability_bp INTEGER NOT NULL DEFAULT 0,
    trend_direction TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_direction IN ('increasing', 'stable', 'decreasing')),
    is_high_frequency INTEGER NOT NULL DEFAULT 0,     -- appeared 3+ times
    is_exam_staple INTEGER NOT NULL DEFAULT 0,        -- appeared 5+ times
    forecast_score_bp INTEGER NOT NULL DEFAULT 0,     -- from forecast formula
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE examiner_pattern_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    pattern_type TEXT NOT NULL
        CHECK (pattern_type IN (
            'topic_rotation', 'difficulty_cycle', 'question_format_preference',
            'section_weight', 'mark_distribution', 'concept_emphasis',
            'style_shift', 'surprise_pattern'
        )),
    description TEXT NOT NULL,
    evidence_years TEXT NOT NULL DEFAULT '[]',        -- JSON array
    strength_bp INTEGER NOT NULL DEFAULT 5000,        -- how reliable is this pattern
    pattern_data_json TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 1,
    last_validated_year INTEGER,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE blueprint_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    blueprint_version TEXT NOT NULL,
    blueprint_type TEXT NOT NULL DEFAULT 'predicted'
        CHECK (blueprint_type IN ('official', 'predicted', 'historical_average', 'custom')),
    exam_year_target INTEGER,
    -- Topic weights: JSON {topic_id: weight_bp}
    topic_weights_json TEXT NOT NULL DEFAULT '{}',
    -- Difficulty distribution: JSON {easy_bp, medium_bp, hard_bp}
    difficulty_distribution_json TEXT NOT NULL DEFAULT '{}',
    -- Format distribution: JSON {mcq_bp, short_answer_bp, ...}
    format_distribution_json TEXT NOT NULL DEFAULT '{}',
    total_questions INTEGER NOT NULL DEFAULT 0,
    total_marks INTEGER NOT NULL DEFAULT 0,
    confidence_bp INTEGER NOT NULL DEFAULT 5000,
    source_description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_past_paper_sources_subject ON past_paper_sources(subject_id);
CREATE INDEX idx_past_paper_sources_year ON past_paper_sources(exam_year);
CREATE INDEX idx_paper_questions_source ON paper_questions_parsed(source_id);
CREATE INDEX idx_paper_questions_question ON paper_questions_parsed(question_id);
CREATE INDEX idx_paper_questions_mapping ON paper_questions_parsed(mapping_status);
CREATE INDEX idx_question_recurrence_topic ON question_recurrence_records(topic_id);
CREATE INDEX idx_question_recurrence_family ON question_recurrence_records(family_id);
CREATE INDEX idx_question_recurrence_freq ON question_recurrence_records(is_high_frequency);
CREATE INDEX idx_examiner_patterns_subject ON examiner_pattern_records(subject_id);
CREATE INDEX idx_blueprint_records_subject ON blueprint_records(subject_id);
CREATE INDEX idx_blueprint_records_active ON blueprint_records(is_active);
```

---

## 017_traps.sql

```sql
-- 017_traps.sql
-- Contrast pairs, evidence atoms, trap drill sessions, trap attempts, difference mastery states

CREATE TABLE contrast_pairs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_key TEXT NOT NULL UNIQUE,                    -- e.g. 'mean_vs_median'
    concept_a_label TEXT NOT NULL,
    concept_b_label TEXT NOT NULL,
    -- Source entities
    entry_a_id INTEGER REFERENCES glossary_entries(id),
    entry_b_id INTEGER REFERENCES glossary_entries(id),
    node_a_id INTEGER REFERENCES academic_nodes(id),
    node_b_id INTEGER REFERENCES academic_nodes(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    -- Characterisation
    confusion_type TEXT NOT NULL DEFAULT 'conceptual'
        CHECK (confusion_type IN (
            'conceptual', 'definitional', 'procedural',
            'notational', 'contextual', 'surface_similarity'
        )),
    confusion_severity_bp INTEGER NOT NULL DEFAULT 5000,
    -- Content
    key_difference_plain TEXT NOT NULL,
    extended_contrast TEXT,
    exam_tip TEXT,
    -- Frequency
    trap_frequency_bp INTEGER NOT NULL DEFAULT 0,     -- how often this is tested as a trap
    is_exam_critical INTEGER NOT NULL DEFAULT 0,
    -- Quality
    is_active INTEGER NOT NULL DEFAULT 1,
    pack_id TEXT REFERENCES content_packs(pack_id),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE contrast_evidence_atoms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    atom_type TEXT NOT NULL
        CHECK (atom_type IN (
            'definition_contrast', 'example_contrast', 'formula_contrast',
            'visual_contrast', 'worked_example', 'common_error_scenario',
            'memory_hook', 'exam_question_sample'
        )),
    concept_side TEXT NOT NULL DEFAULT 'both'
        CHECK (concept_side IN ('a', 'b', 'both')),
    content_text TEXT NOT NULL,
    latex_expression TEXT,
    diagram_path TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE trap_drill_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    drill_mode TEXT NOT NULL DEFAULT 'standard'
        CHECK (drill_mode IN ('standard', 'timed', 'boss_set', 'exam_simulation')),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'completed', 'abandoned')),
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    duration_ms INTEGER,
    pairs_attempted INTEGER NOT NULL DEFAULT 0,
    pairs_correct INTEGER NOT NULL DEFAULT 0,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    pairs_targeted TEXT NOT NULL DEFAULT '[]',        -- JSON array of pair_ids
    performance_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE trap_attempt_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES trap_drill_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id),
    question_id INTEGER REFERENCES questions(id),
    attempt_type TEXT NOT NULL DEFAULT 'identify_difference'
        CHECK (attempt_type IN (
            'identify_difference', 'classify_concept',
            'complete_statement', 'select_correct_context', 'match_definition'
        )),
    is_correct INTEGER NOT NULL DEFAULT 0,
    response_time_ms INTEGER,
    selected_answer TEXT,
    correct_answer TEXT,
    hint_used INTEGER NOT NULL DEFAULT 0,
    confidence_level TEXT
        CHECK (confidence_level IN ('sure', 'not_sure', 'guessed')),
    error_note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE difference_mastery_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    mastery_bp INTEGER NOT NULL DEFAULT 0,
    confusion_risk_bp INTEGER NOT NULL DEFAULT 10000,
    attempts INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TEXT,
    last_correct_at TEXT,
    last_confused_at TEXT,
    confusion_count INTEGER NOT NULL DEFAULT 0,
    is_resolved INTEGER NOT NULL DEFAULT 0,
    resolved_at TEXT,
    next_review_at TEXT,
    review_interval_days REAL NOT NULL DEFAULT 1.0,
    state TEXT NOT NULL DEFAULT 'unseen'
        CHECK (state IN (
            'unseen', 'confused', 'partially_clear',
            'clear', 'automatic', 'mastered'
        )),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, pair_id)
);

CREATE INDEX idx_contrast_pairs_subject ON contrast_pairs(subject_id);
CREATE INDEX idx_contrast_pairs_topic ON contrast_pairs(topic_id);
CREATE INDEX idx_contrast_pairs_active ON contrast_pairs(is_active);
CREATE INDEX idx_contrast_evidence_pair ON contrast_evidence_atoms(pair_id);
CREATE INDEX idx_trap_drill_sessions_student ON trap_drill_sessions(student_id);
CREATE INDEX idx_trap_attempt_records_session ON trap_attempt_records(session_id);
CREATE INDEX idx_trap_attempt_records_pair ON trap_attempt_records(pair_id);
CREATE INDEX idx_difference_mastery_student ON difference_mastery_states(student_id);
CREATE INDEX idx_difference_mastery_pair ON difference_mastery_states(pair_id);
CREATE INDEX idx_difference_mastery_state ON difference_mastery_states(state);
```

---

## 018_intake.sql

```sql
-- 018_intake.sql
-- Intake documents, pages, segments, question candidates, processing jobs, OCR results

CREATE TABLE intake_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_key TEXT NOT NULL UNIQUE,
    document_name TEXT NOT NULL,
    document_type TEXT NOT NULL
        CHECK (document_type IN (
            'past_paper', 'textbook_chapter', 'curriculum_doc',
            'teacher_upload', 'student_upload', 'reference_material'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    source_description TEXT,
    file_path TEXT NOT NULL,
    file_format TEXT NOT NULL
        CHECK (file_format IN ('pdf', 'jpg', 'png', 'docx', 'txt', 'json')),
    file_size_bytes INTEGER NOT NULL DEFAULT 0,
    file_hash TEXT,
    page_count INTEGER NOT NULL DEFAULT 0,
    -- Provenance
    uploaded_by INTEGER REFERENCES accounts(id),
    upload_source TEXT NOT NULL DEFAULT 'admin'
        CHECK (upload_source IN ('admin', 'teacher', 'student', 'system', 'api')),
    -- Processing
    processing_status TEXT NOT NULL DEFAULT 'queued'
        CHECK (processing_status IN (
            'queued', 'ocr_pending', 'ocr_complete', 'parsing',
            'parsed', 'mapping', 'mapped', 'review_pending',
            'approved', 'rejected', 'failed'
        )),
    error_message TEXT,
    processing_started_at TEXT,
    processing_completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE intake_pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id) ON DELETE CASCADE,
    page_number INTEGER NOT NULL,
    image_path TEXT,
    raw_text TEXT,
    ocr_confidence_bp INTEGER,
    layout_json TEXT,                                 -- detected regions/blocks
    page_type TEXT
        CHECK (page_type IN (
            'questions', 'instructions', 'answer_key', 'blank',
            'diagram_only', 'mixed', 'unknown'
        )),
    processing_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (processing_status IN ('pending', 'ocr_done', 'parsed', 'failed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(document_id, page_number)
);

CREATE TABLE intake_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id INTEGER NOT NULL REFERENCES intake_pages(id) ON DELETE CASCADE,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id),
    segment_type TEXT NOT NULL
        CHECK (segment_type IN (
            'question_stem', 'option_a', 'option_b', 'option_c', 'option_d',
            'answer_key_entry', 'section_header', 'instruction',
            'diagram_region', 'table_region', 'formula_region', 'unknown'
        )),
    sequence_order INTEGER NOT NULL DEFAULT 0,
    raw_text TEXT,
    cleaned_text TEXT,
    bounding_box_json TEXT,                           -- {x, y, w, h} in page coords
    confidence_bp INTEGER NOT NULL DEFAULT 0,
    is_formula INTEGER NOT NULL DEFAULT 0,
    has_diagram INTEGER NOT NULL DEFAULT 0,
    linked_question_number INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE intake_question_candidates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id) ON DELETE CASCADE,
    question_number INTEGER NOT NULL,
    stem_text TEXT NOT NULL,
    option_a TEXT,
    option_b TEXT,
    option_c TEXT,
    option_d TEXT,
    correct_option TEXT,
    marks INTEGER NOT NULL DEFAULT 1,
    -- Classification candidates
    candidate_topic_id INTEGER REFERENCES topics(id),
    candidate_difficulty_bp INTEGER,
    candidate_intent TEXT,
    -- Status
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (review_status IN (
            'pending', 'approved', 'rejected',
            'duplicate', 'needs_edit', 'promoted'
        )),
    promoted_question_id INTEGER REFERENCES questions(id),
    reviewer_notes TEXT,
    auto_flags TEXT NOT NULL DEFAULT '[]',            -- JSON array of flag strings
    confidence_bp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE intake_processing_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id) ON DELETE CASCADE,
    job_type TEXT NOT NULL
        CHECK (job_type IN (
            'ocr', 'layout_parse', 'question_extract', 'topic_map',
            'difficulty_classify', 'quality_check', 'promote_questions'
        )),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'complete', 'failed', 'cancelled')),
    priority INTEGER NOT NULL DEFAULT 5,
    queued_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT,
    duration_ms INTEGER,
    worker_id TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    error_message TEXT,
    result_summary_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE ocr_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id) ON DELETE CASCADE,
    page_id INTEGER REFERENCES intake_pages(id),
    engine_name TEXT NOT NULL DEFAULT 'tesseract',
    engine_version TEXT,
    overall_confidence_bp INTEGER NOT NULL DEFAULT 0,
    character_count INTEGER NOT NULL DEFAULT 0,
    word_count INTEGER NOT NULL DEFAULT 0,
    detected_language TEXT NOT NULL DEFAULT 'en',
    raw_output_path TEXT,
    structured_output_json TEXT,
    processing_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_intake_documents_status ON intake_documents(processing_status);
CREATE INDEX idx_intake_documents_type ON intake_documents(document_type);
CREATE INDEX idx_intake_pages_document ON intake_pages(document_id);
CREATE INDEX idx_intake_segments_page ON intake_segments(page_id);
CREATE INDEX idx_intake_segments_document ON intake_segments(document_id);
CREATE INDEX idx_intake_candidates_document ON intake_question_candidates(document_id);
CREATE INDEX idx_intake_candidates_status ON intake_question_candidates(review_status);
CREATE INDEX idx_intake_jobs_document ON intake_processing_jobs(document_id);
CREATE INDEX idx_intake_jobs_status ON intake_processing_jobs(status, priority);
CREATE INDEX idx_ocr_results_document ON ocr_results(document_id);
```

---

## 019_premium.sql

```sql
-- 019_premium.sql
-- Premium strategy sessions, concierge actions, strategy documents, premium plans, elite prep

CREATE TABLE premium_strategy_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    session_label TEXT NOT NULL,
    session_type TEXT NOT NULL DEFAULT 'strategy_review'
        CHECK (session_type IN (
            'strategy_review', 'deep_diagnosis', 'exam_plan_build',
            'mock_debrief', 'crisis_intervention', 'milestone_review',
            'parent_brief', 'subject_deep_dive'
        )),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'completed', 'archived')),
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    -- Context snapshot at session start
    readiness_bp_at_start INTEGER,
    days_to_exam_at_start INTEGER,
    active_danger_zones_at_start INTEGER,
    -- Outputs
    strategy_document_id INTEGER,                     -- FK set after creation
    action_count INTEGER NOT NULL DEFAULT 0,
    actions_completed INTEGER NOT NULL DEFAULT 0,
    coach_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE premium_concierge_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES premium_strategy_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    action_type TEXT NOT NULL
        CHECK (action_type IN (
            'assign_topic_repair', 'schedule_mock', 'unlock_elite_drill',
            'flag_danger_zone', 'generate_revision_pack', 'send_parent_alert',
            'adjust_daily_plan', 'override_session_type', 'set_exam_countdown',
            'create_custom_mission', 'escalate_to_crisis'
        )),
    priority INTEGER NOT NULL DEFAULT 5,              -- 1 = highest
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    target_entity_type TEXT,                          -- 'topic', 'session', 'plan', etc.
    target_entity_id INTEGER,
    parameters_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'completed', 'skipped', 'failed')),
    due_by TEXT,
    completed_at TEXT,
    result_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE strategy_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES premium_strategy_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    document_type TEXT NOT NULL DEFAULT 'strategy_brief'
        CHECK (document_type IN (
            'strategy_brief', 'exam_game_plan', 'crisis_plan',
            'parent_summary', 'milestone_report', 'subject_breakdown'
        )),
    title TEXT NOT NULL,
    -- Sections stored as JSON: [{heading, body, priority}]
    sections_json TEXT NOT NULL DEFAULT '[]',
    plain_text_summary TEXT,
    readiness_snapshot_json TEXT NOT NULL DEFAULT '{}',
    action_items_json TEXT NOT NULL DEFAULT '[]',
    is_finalized INTEGER NOT NULL DEFAULT 0,
    finalized_at TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Add FK back-reference in premium_strategy_sessions
-- (SQLite doesn't support ALTER TABLE ADD CONSTRAINT, so handled in application layer)

CREATE TABLE premium_plan_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    plan_type TEXT NOT NULL DEFAULT 'weekly'
        CHECK (plan_type IN ('daily', 'weekly', 'exam_countdown', 'subject_blitz', 'custom')),
    label TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('draft', 'active', 'completed', 'superseded', 'cancelled')),
    starts_on TEXT NOT NULL,
    ends_on TEXT,
    -- Goals
    readiness_target_bp INTEGER,
    coverage_target_bp INTEGER,
    sessions_target INTEGER,
    mocks_target INTEGER,
    -- Actuals (updated as plan progresses)
    sessions_completed INTEGER NOT NULL DEFAULT 0,
    mocks_completed INTEGER NOT NULL DEFAULT 0,
    readiness_achieved_bp INTEGER,
    -- Plan definition
    plan_data_json TEXT NOT NULL DEFAULT '{}',
    generated_by TEXT NOT NULL DEFAULT 'coach'
        CHECK (generated_by IN ('coach', 'premium_concierge', 'admin', 'student')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE elite_prep_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    -- Elite progression
    current_tier TEXT NOT NULL DEFAULT 'foundation'
        CHECK (current_tier IN ('foundation', 'core', 'apex', 'legend')),
    tier_unlocked_at TEXT,
    -- Pillar scores (all basis points)
    precision_bp INTEGER NOT NULL DEFAULT 0,
    speed_bp INTEGER NOT NULL DEFAULT 0,
    depth_bp INTEGER NOT NULL DEFAULT 0,
    endurance_bp INTEGER NOT NULL DEFAULT 0,
    trap_sense_bp INTEGER NOT NULL DEFAULT 0,
    elegant_solve_bp INTEGER NOT NULL DEFAULT 0,
    pressure_bp INTEGER NOT NULL DEFAULT 0,
    -- Overall
    elite_composite_bp INTEGER NOT NULL DEFAULT 0,
    -- Sessions
    elite_sessions_count INTEGER NOT NULL DEFAULT 0,
    perfect_runs_count INTEGER NOT NULL DEFAULT 0,
    longest_clean_streak INTEGER NOT NULL DEFAULT 0,
    current_clean_streak INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE INDEX idx_premium_strategy_sessions_student ON premium_strategy_sessions(student_id);
CREATE INDEX idx_premium_concierge_actions_session ON premium_concierge_actions(session_id);
CREATE INDEX idx_premium_concierge_actions_status ON premium_concierge_actions(status);
CREATE INDEX idx_strategy_documents_session ON strategy_documents(session_id);
CREATE INDEX idx_premium_plan_records_student ON premium_plan_records(student_id);
CREATE INDEX idx_premium_plan_records_status ON premium_plan_records(status);
CREATE INDEX idx_elite_prep_records_student ON elite_prep_records(student_id);
```

---

## 020_elite.sql

```sql
-- 020_elite.sql
-- Elite scores, performance snapshots, snapshot components, missions, benchmarks

CREATE TABLE elite_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    session_id INTEGER,                               -- FK to sessions if from a session
    score_type TEXT NOT NULL
        CHECK (score_type IN (
            'sprint', 'gauntlet', 'precision_lab', 'depth_lab',
            'perfect_run', 'endurance_track', 'apex_mock'
        )),
    -- Raw performance
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    questions_correct INTEGER NOT NULL DEFAULT 0,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms INTEGER,
    fastest_correct_ms INTEGER,
    -- Elite metrics
    precision_score_bp INTEGER NOT NULL DEFAULT 0,
    speed_score_bp INTEGER NOT NULL DEFAULT 0,
    depth_score_bp INTEGER NOT NULL DEFAULT 0,
    endurance_score_bp INTEGER NOT NULL DEFAULT 0,
    trap_sense_score_bp INTEGER NOT NULL DEFAULT 0,
    pressure_score_bp INTEGER NOT NULL DEFAULT 0,
    composite_elite_bp INTEGER NOT NULL DEFAULT 0,
    -- Run integrity
    is_perfect_run INTEGER NOT NULL DEFAULT 0,
    hints_used INTEGER NOT NULL DEFAULT 0,
    errors_count INTEGER NOT NULL DEFAULT 0,
    -- Percentile (vs historical self and targets)
    personal_best_flag INTEGER NOT NULL DEFAULT 0,
    personal_best_prev_bp INTEGER,
    scored_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE elite_performance_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    snapshot_label TEXT NOT NULL,
    snapshot_period TEXT NOT NULL DEFAULT 'session'
        CHECK (snapshot_period IN ('session', 'daily', 'weekly', 'monthly', 'milestone')),
    -- Aggregate pillars
    precision_avg_bp INTEGER NOT NULL DEFAULT 0,
    speed_avg_bp INTEGER NOT NULL DEFAULT 0,
    depth_avg_bp INTEGER NOT NULL DEFAULT 0,
    endurance_avg_bp INTEGER NOT NULL DEFAULT 0,
    trap_sense_avg_bp INTEGER NOT NULL DEFAULT 0,
    elegant_solve_avg_bp INTEGER NOT NULL DEFAULT 0,
    pressure_avg_bp INTEGER NOT NULL DEFAULT 0,
    composite_avg_bp INTEGER NOT NULL DEFAULT 0,
    -- Progress
    sessions_in_period INTEGER NOT NULL DEFAULT 0,
    perfect_runs_in_period INTEGER NOT NULL DEFAULT 0,
    questions_in_period INTEGER NOT NULL DEFAULT 0,
    accuracy_in_period_bp INTEGER NOT NULL DEFAULT 0,
    trend_direction TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_direction IN ('improving', 'stable', 'declining')),
    snapped_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE eps_components (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER NOT NULL REFERENCES elite_performance_snapshots(id) ON DELETE CASCADE,
    pillar TEXT NOT NULL
        CHECK (pillar IN (
            'precision', 'speed', 'depth', 'endurance',
            'trap_sense', 'elegant_solve', 'pressure'
        )),
    score_bp INTEGER NOT NULL DEFAULT 0,
    delta_from_previous_bp INTEGER NOT NULL DEFAULT 0,
    percentile_rank INTEGER,                          -- 0-100 vs historical self
    contributing_sessions INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE elite_missions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    mission_type TEXT NOT NULL
        CHECK (mission_type IN (
            'hit_accuracy_target', 'achieve_perfect_run', 'beat_personal_best',
            'clear_trap_cluster', 'sustain_speed_under_pressure', 'complete_gauntlet',
            'reach_tier', 'master_elite_set', 'custom'
        )),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    -- Targets
    target_metric TEXT NOT NULL,                      -- e.g. 'precision_bp'
    target_value INTEGER NOT NULL,                    -- basis points
    current_value INTEGER NOT NULL DEFAULT 0,
    -- Status
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'completed', 'failed', 'expired', 'cancelled')),
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    deadline TEXT,
    completed_at TEXT,
    -- Reward
    xp_reward INTEGER NOT NULL DEFAULT 0,
    badge_reward TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE elite_benchmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    tier TEXT NOT NULL
        CHECK (tier IN ('foundation', 'core', 'apex', 'legend')),
    pillar TEXT NOT NULL
        CHECK (pillar IN (
            'precision', 'speed', 'depth', 'endurance',
            'trap_sense', 'elegant_solve', 'pressure', 'composite'
        )),
    benchmark_bp INTEGER NOT NULL,                    -- minimum score to be at this tier
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(subject_id, tier, pillar)
);

CREATE INDEX idx_elite_scores_student ON elite_scores(student_id);
CREATE INDEX idx_elite_scores_subject ON elite_scores(subject_id);
CREATE INDEX idx_elite_scores_type ON elite_scores(score_type);
CREATE INDEX idx_elite_perf_snapshots_student ON elite_performance_snapshots(student_id);
CREATE INDEX idx_eps_components_snapshot ON eps_components(snapshot_id);
CREATE INDEX idx_elite_missions_student ON elite_missions(student_id);
CREATE INDEX idx_elite_missions_status ON elite_missions(status);
CREATE INDEX idx_elite_benchmarks_subject ON elite_benchmarks(subject_id, tier);
```

---

## 021_event_log.sql

```sql
-- 021_event_log.sql
-- Append-only event sourcing table, outbox events, event consumers, domain event types

-- Core append-only event log: never UPDATE or DELETE rows in this table
CREATE TABLE event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL UNIQUE,                    -- UUID v4
    event_type TEXT NOT NULL,                         -- e.g. 'student.attempt.recorded'
    aggregate_type TEXT NOT NULL,                     -- 'student', 'session', 'topic_state', etc.
    aggregate_id TEXT NOT NULL,                       -- stringified entity id
    student_id INTEGER REFERENCES accounts(id),       -- denormalised for fast per-student replay
    sequence_number INTEGER NOT NULL DEFAULT 0,       -- monotonic per aggregate
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    recorded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- Payload
    payload TEXT NOT NULL DEFAULT '{}',               -- JSON event body
    -- Causality
    causation_id TEXT,                                -- event_id that caused this event
    correlation_id TEXT,                              -- trace / session correlation
    -- Schema
    schema_version INTEGER NOT NULL DEFAULT 1,
    -- Processing
    is_processed INTEGER NOT NULL DEFAULT 0,
    processed_at TEXT,
    -- Integrity: no UPDATE or DELETE allowed; enforced at application layer
    CHECK (json_valid(payload))
);

-- Transactional outbox for cross-boundary delivery
CREATE TABLE outbox_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_log_id INTEGER NOT NULL REFERENCES event_log(id),
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    destination TEXT NOT NULL,                        -- consumer name or 'broadcast'
    payload TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'delivered', 'failed', 'dead_lettered')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 5,
    last_attempt_at TEXT,
    next_attempt_at TEXT,
    delivered_at TEXT,
    error_message TEXT
);

CREATE TABLE event_consumers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    consumer_name TEXT NOT NULL UNIQUE,
    consumer_description TEXT,
    subscribed_event_types TEXT NOT NULL DEFAULT '[]', -- JSON array of event type patterns
    last_processed_event_id INTEGER REFERENCES event_log(id),
    last_processed_at TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE domain_event_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL UNIQUE,                  -- canonical name
    aggregate_type TEXT NOT NULL,
    description TEXT NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1,
    payload_schema_json TEXT NOT NULL DEFAULT '{}',   -- JSON Schema for payload validation
    is_active INTEGER NOT NULL DEFAULT 1,
    is_deprecated INTEGER NOT NULL DEFAULT 0,
    deprecated_at TEXT,
    superseded_by TEXT,                               -- event_type that replaces this
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_event_log_event_id ON event_log(event_id);
CREATE INDEX idx_event_log_aggregate ON event_log(aggregate_type, aggregate_id);
CREATE INDEX idx_event_log_student ON event_log(student_id);
CREATE INDEX idx_event_log_type ON event_log(event_type);
CREATE INDEX idx_event_log_occurred ON event_log(occurred_at);
CREATE INDEX idx_event_log_processed ON event_log(is_processed);
CREATE INDEX idx_outbox_events_status ON outbox_events(status, next_attempt_at);
CREATE INDEX idx_outbox_events_event ON outbox_events(event_log_id);
CREATE INDEX idx_domain_event_types_aggregate ON domain_event_types(aggregate_type);
```

---

## 022_system.sql

```sql
-- 022_system.sql
-- App settings, feature flags, migration history, sync state, background jobs, audit log

CREATE TABLE app_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    setting_key TEXT NOT NULL UNIQUE,
    setting_value TEXT NOT NULL,
    value_type TEXT NOT NULL DEFAULT 'string'
        CHECK (value_type IN ('string', 'integer', 'float', 'boolean', 'json')),
    category TEXT NOT NULL DEFAULT 'general'
        CHECK (category IN (
            'general', 'coach', 'ui', 'session', 'memory',
            'parent', 'notification', 'security', 'content', 'debug'
        )),
    description TEXT,
    is_user_configurable INTEGER NOT NULL DEFAULT 0,
    is_secret INTEGER NOT NULL DEFAULT 0,
    min_value TEXT,
    max_value TEXT,
    default_value TEXT NOT NULL,
    last_changed_at TEXT NOT NULL DEFAULT (datetime('now')),
    changed_by INTEGER REFERENCES accounts(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE feature_flags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    flag_key TEXT NOT NULL UNIQUE,                    -- e.g. 'elite_mode_enabled'
    flag_name TEXT NOT NULL,
    description TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 0,
    -- Targeting
    enabled_for_tiers TEXT NOT NULL DEFAULT '[]',     -- JSON: ['Standard','Premium','Elite']
    enabled_for_accounts TEXT NOT NULL DEFAULT '[]',  -- JSON: specific account IDs (override)
    rollout_percentage INTEGER NOT NULL DEFAULT 100,  -- 0-100
    -- Lifecycle
    category TEXT NOT NULL DEFAULT 'feature'
        CHECK (category IN ('feature', 'experiment', 'kill_switch', 'debug', 'migration')),
    expires_at TEXT,
    is_permanent INTEGER NOT NULL DEFAULT 0,
    changed_at TEXT NOT NULL DEFAULT (datetime('now')),
    changed_by INTEGER REFERENCES accounts(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE migration_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    migration_name TEXT NOT NULL UNIQUE,              -- e.g. '001_accounts.sql'
    migration_number INTEGER NOT NULL UNIQUE,
    checksum TEXT NOT NULL,                           -- SHA-256 of migration SQL
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    duration_ms INTEGER,
    applied_by TEXT NOT NULL DEFAULT 'system',
    rollback_sql TEXT,                                -- optional rollback script
    notes TEXT
);

CREATE TABLE sync_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sync_key TEXT NOT NULL UNIQUE,                    -- e.g. 'content_pack_sync', 'account_sync'
    sync_type TEXT NOT NULL
        CHECK (sync_type IN (
            'content_pack', 'account', 'settings',
            'feature_flags', 'analytics_upload', 'backup'
        )),
    last_sync_at TEXT,
    last_success_at TEXT,
    last_failure_at TEXT,
    last_error TEXT,
    sync_status TEXT NOT NULL DEFAULT 'idle'
        CHECK (sync_status IN ('idle', 'running', 'success', 'failed', 'paused')),
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    -- For incremental sync
    last_synced_cursor TEXT,                          -- timestamp or sequence number
    bytes_transferred INTEGER NOT NULL DEFAULT 0,
    records_synced INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE background_job_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_key TEXT NOT NULL,                            -- e.g. 'rebuild_fts_index'
    job_type TEXT NOT NULL
        CHECK (job_type IN (
            'index_rebuild', 'state_recompute', 'memory_decay_update',
            'report_generation', 'pack_install', 'ocr_process',
            'cleanup', 'export', 'snapshot', 'analytics'
        )),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'complete', 'failed', 'cancelled', 'skipped')),
    priority INTEGER NOT NULL DEFAULT 5,              -- 1 = highest
    scheduled_for TEXT,
    queued_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT,
    duration_ms INTEGER,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    last_error TEXT,
    parameters_json TEXT NOT NULL DEFAULT '{}',
    result_json TEXT NOT NULL DEFAULT '{}',
    triggered_by TEXT NOT NULL DEFAULT 'system'       -- 'system', 'user', 'schedule', 'event'
);

CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Who
    account_id INTEGER REFERENCES accounts(id),
    account_role TEXT,
    session_token_hash TEXT,
    -- What
    action TEXT NOT NULL,                             -- e.g. 'pin_changed', 'pack_installed'
    entity_type TEXT,
    entity_id TEXT,
    -- Detail
    old_value_json TEXT,
    new_value_json TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    -- Where / When
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ip_address TEXT,
    -- Integrity: rows should never be updated or deleted
    CHECK (json_valid(metadata_json))
);

CREATE INDEX idx_app_settings_category ON app_settings(category);
CREATE INDEX idx_feature_flags_enabled ON feature_flags(is_enabled);
CREATE INDEX idx_migration_history_number ON migration_history(migration_number);
CREATE INDEX idx_background_jobs_status ON background_job_records(status, priority);
CREATE INDEX idx_background_jobs_type ON background_job_records(job_type);
CREATE INDEX idx_audit_log_account ON audit_log(account_id);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_occurred ON audit_log(occurred_at);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
```

---

# PART 3: ALL RUST TYPES & ENUMS

All types live in `ecoach-substrate/src/`. The crate is `no_std`-compatible except where noted.
Add to `Cargo.toml`: `serde = { version = "1", features = ["derive"] }`.

---

## 3.1 Primitive Types

```rust
// ecoach-substrate/src/primitives.rs

use serde::{Deserialize, Serialize};

// ─── BasisPoints ─────────────────────────────────────────────────────────────

/// A fixed-point representation of a proportion in the range [0, 10_000].
/// 10_000 bp == 100 %. Used throughout for scores, weights, and thresholds.
pub type BasisPoints = u16;

/// Convert a floating-point proportion (0.0–1.0) to BasisPoints.
/// Values outside range are clamped.
#[inline]
pub fn to_bp(value: f64) -> BasisPoints {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as BasisPoints
}

/// Convert BasisPoints back to a floating-point proportion (0.0–1.0).
#[inline]
pub fn from_bp(bp: BasisPoints) -> f64 {
    bp as f64 / 10_000.0
}

// ─── ConfidenceScore ─────────────────────────────────────────────────────────

/// A learner's self-reported or inferred confidence, 0–100 (inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ConfidenceScore(u8);

impl ConfidenceScore {
    pub const MIN: ConfidenceScore = ConfidenceScore(0);
    pub const MAX: ConfidenceScore = ConfidenceScore(100);

    /// Create a ConfidenceScore, clamping to [0, 100].
    pub fn new(value: u8) -> Self {
        ConfidenceScore(value.min(100))
    }

    pub fn value(self) -> u8 {
        self.0
    }

    pub fn as_bp(self) -> BasisPoints {
        (self.0 as u16) * 100
    }
}

impl Default for ConfidenceScore {
    fn default() -> Self {
        ConfidenceScore(50)
    }
}

// ─── SeverityLevel ───────────────────────────────────────────────────────────

/// Severity classification used for danger zones, alerts, and issue escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityLevel {
    /// Below watch threshold — informational only.
    Low,
    /// Approaching a problem threshold — monitor.
    Watch,
    /// Confirmed problem — intervention warranted.
    Active,
    /// Serious, time-sensitive — prioritise immediately.
    Urgent,
    /// Exam-threatening — all-hands response required.
    Critical,
}

impl SeverityLevel {
    /// Minimum BasisPoints score that classifies as this level.
    pub const fn threshold_bp(self) -> BasisPoints {
        match self {
            SeverityLevel::Low => 0,
            SeverityLevel::Watch => 3_000,
            SeverityLevel::Active => 5_000,
            SeverityLevel::Urgent => 7_000,
            SeverityLevel::Critical => 8_500,
        }
    }

    /// Classify a danger score (basis points) into a SeverityLevel.
    pub fn from_score(score_bp: BasisPoints) -> Self {
        if score_bp >= SeverityLevel::Critical.threshold_bp() {
            SeverityLevel::Critical
        } else if score_bp >= SeverityLevel::Urgent.threshold_bp() {
            SeverityLevel::Urgent
        } else if score_bp >= SeverityLevel::Active.threshold_bp() {
            SeverityLevel::Active
        } else if score_bp >= SeverityLevel::Watch.threshold_bp() {
            SeverityLevel::Watch
        } else {
            SeverityLevel::Low
        }
    }
}

// ─── TrendDirection ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    /// Score is falling sharply and below a critical threshold.
    Critical,
}

// ─── Timestamps ──────────────────────────────────────────────────────────────

/// Unix epoch timestamp in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimestampMs(pub i64);

impl TimestampMs {
    pub fn value(self) -> i64 {
        self.0
    }
}

/// A duration expressed in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DurationMs(pub i64);

impl DurationMs {
    pub fn value(self) -> i64 {
        self.0
    }

    pub fn as_seconds(self) -> f64 {
        self.0 as f64 / 1_000.0
    }

    pub fn as_minutes(self) -> f64 {
        self.0 as f64 / 60_000.0
    }
}
```

---

## 3.2 Domain Enums

```rust
// ecoach-substrate/src/domain_enums.rs

use serde::{Deserialize, Serialize};

// ─── Role ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Student,
    Parent,
    Admin,
}

// ─── EntitlementTier ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementTier {
    Standard,
    Premium,
    Elite,
}

impl EntitlementTier {
    pub fn includes_premium(self) -> bool {
        matches!(self, EntitlementTier::Premium | EntitlementTier::Elite)
    }

    pub fn includes_elite(self) -> bool {
        matches!(self, EntitlementTier::Elite)
    }
}

// ─── CoachLifecycleState ─────────────────────────────────────────────────────

/// The 14-state machine governing what the coach is doing for a given student.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachLifecycleState {
    /// No active student — waiting for account selection.
    Dormant,
    /// Running initial calibration diagnostics.
    Calibrating,
    /// Setting the baseline mastery/coverage map after calibration.
    BaselineSetting,
    /// Building the study mission and weekly plan.
    MissionPlanning,
    /// Normal daily coaching in progress.
    ActiveTeaching,
    /// Executing targeted remediation for identified gaps.
    GapRepair,
    /// Running memory rescue scheduling (spaced-repetition emergency).
    MemoryRescue,
    /// Composing and running mock exams.
    MockOrchestrating,
    /// Collecting and verifying readiness proofs.
    ReadinessProofing,
    /// Emergency intervention: exam-threatening risk detected.
    CrisisIntervention,
    /// Final countdown mode: ≤ 7 days to exam.
    ExamCountdown,
    /// Post-exam analysis and debrief.
    PostExamReview,
    /// Generating and delivering parent briefing.
    ParentalBriefing,
    /// Background maintenance: index rebuild, state recompute, etc.
    SystemMaintenance,
}

// ─── MemoryState ─────────────────────────────────────────────────────────────

/// 12-state memory lifecycle for a learner's recall of a specific knowledge item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryState {
    /// Newly learned; not yet consolidated.
    Fresh,
    /// In the consolidation window (first 24-48 h).
    Consolidating,
    /// Well-reviewed; memory is solid.
    Stable,
    /// Overdue for review; at risk of decay.
    AtRisk,
    /// Noticeably weakened — recall slower or error-prone.
    Decaying,
    /// Below rescue threshold; near-forgotten.
    Critical,
    /// Measured as forgotten; needs reteaching.
    Forgotten,
    /// Was forgotten but successfully recalled after rescue session.
    Rescued,
    /// Deliberately reinforced above baseline.
    Reinforced,
    /// Knowledge transferred to novel contexts.
    Transferred,
    /// Intentionally parked (not in active rotation).
    Dormant,
    /// Removed from active review cycle (mastered long-term or out of scope).
    Archived,
}

// ─── MasteryState ────────────────────────────────────────────────────────────

/// 8-state mastery chain for a skill or topic.
/// States are ordered — a learner must pass through them sequentially.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MasteryState {
    /// Never attempted.
    Untested,
    /// First exposures — patterns beginning to appear.
    Emerging,
    /// Can answer sometimes but inconsistently.
    Unstable,
    /// Reliably correct in standard conditions.
    Functional,
    /// Consistent across multiple sessions and question types.
    Stable,
    /// Correct in novel/transfer contexts.
    Transferable,
    /// Performing at exam-passing standards under timed conditions.
    ExamReady,
    /// Comprehensive, automatic, durable knowledge.
    Mastered,
}

impl MasteryState {
    /// Minimum BasisPoints score required to be at this state.
    pub const fn min_score_bp(self) -> crate::primitives::BasisPoints {
        match self {
            MasteryState::Untested => 0,
            MasteryState::Emerging => 1_000,
            MasteryState::Unstable => 2_500,
            MasteryState::Functional => 4_000,
            MasteryState::Stable => 5_500,
            MasteryState::Transferable => 7_000,
            MasteryState::ExamReady => 8_500,
            MasteryState::Mastered => 9_000,
        }
    }

    /// Derive MasteryState from a raw basis-points score.
    pub fn from_score(score_bp: crate::primitives::BasisPoints) -> Self {
        if score_bp >= MasteryState::Mastered.min_score_bp() {
            MasteryState::Mastered
        } else if score_bp >= MasteryState::ExamReady.min_score_bp() {
            MasteryState::ExamReady
        } else if score_bp >= MasteryState::Transferable.min_score_bp() {
            MasteryState::Transferable
        } else if score_bp >= MasteryState::Stable.min_score_bp() {
            MasteryState::Stable
        } else if score_bp >= MasteryState::Functional.min_score_bp() {
            MasteryState::Functional
        } else if score_bp >= MasteryState::Unstable.min_score_bp() {
            MasteryState::Unstable
        } else if score_bp >= MasteryState::Emerging.min_score_bp() {
            MasteryState::Emerging
        } else {
            MasteryState::Untested
        }
    }
}

// ─── MockType ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MockType {
    /// Maximum realism — mirrors the most likely real exam.
    Forecast,
    /// Maximum diagnostic insight — designed to reveal gaps.
    Diagnostic,
    /// Closes known weak areas with targeted questions.
    Remediation,
    /// Full readiness proof at exam-standard conditions.
    FinalExam,
    /// Resilience training with hard, unexpected items.
    Shock,
    /// Mastery proof at elite performance standards.
    Wisdom,
}

// ─── JourneyPhase ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JourneyPhase {
    /// Fix major weaknesses and core foundational concepts.
    StabilizeFoundation,
    /// Cover main syllabus, build deep understanding.
    BuildCore,
    /// Attack weak topics and recurring mistakes.
    StrengthenWeakLinks,
    /// Timed questions, exam pressure, mixed-topic drills.
    ExamConditioning,
    /// Mocks, revision bursts, confidence repair, exam strategy.
    FinalReadiness,
}

// ─── RiseStage ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiseStage {
    /// Stop the bleeding; find root gaps; first wins; shame removal.
    Rescue,
    /// Make correct thinking repeatable; scaffolded; concept clusters.
    Stabilize,
    /// Speed and independence; timed drills; mixed topics; pressure mode.
    Accelerate,
    /// Outperform top students; trap questions; elite variants; speed + accuracy.
    Dominate,
}

// ─── KnowledgeGapType ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeGapType {
    /// Never encountered the concept at all.
    Content,
    /// Seen it but does not truly understand it.
    Understanding,
    /// Understands theory but cannot use it to solve questions.
    Application,
    /// Knows the idea but procedural steps are weak.
    Process,
    /// Can solve but too slowly for exam conditions.
    Speed,
    /// Knows the content but makes careless mistakes.
    Accuracy,
    /// Once knew it but has forgotten it.
    Retention,
    /// Solves familiar versions but fails novel/transfer forms.
    Transfer,
    /// Interference from a similar concept causing confusion.
    Interference,
    /// Does not know what they do not know.
    SelfAwareness,
}

// ─── QuestionIntent ──────────────────────────────────────────────────────────

/// The pedagogic purpose for which a question is selected or composed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionIntent {
    /// Find where the learner currently stands.
    Discovery,
    /// Probe a specific suspected weakness.
    Diagnosis,
    /// Verify improvement from a different angle.
    Confirmation,
    /// Broaden syllabus coverage.
    Coverage,
    /// Strengthen fragile recent gains.
    Reinforcement,
    /// Test whether a wrong mental model is still present.
    MisconceptionExposure,
    /// Check recall after time has passed.
    RetentionCheck,
    /// Check performance in a new context or wording.
    TransferCheck,
    /// Test smooth, low-effort performance.
    FluencyCheck,
    /// Test performance under time constraint.
    SpeedCheck,
    /// Replicate exam conditions faithfully.
    ExamSimulation,
    /// Rebuild confidence with achievable challenges.
    ConfidenceRepair,
}

// ─── LearnerArchetype ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearnerArchetype {
    /// Gaps are wide but effort is consistent — foundation-first progression.
    WeakButConsistent,
    /// Strong knowledge but avoids hard work — shorter, sharper, accountable.
    StrongButLazy,
    /// Arriving very late — high-yield triage mode.
    PanickingLastMinute,
    /// Believes they know more than they do — diagnostics + timed challenges.
    Overconfident,
    /// Has lost belief in their ability — confidence-first sequencing.
    Discouraged,
}

// ─── PressureLevel ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureLevel {
    /// No time pressure; supportive environment.
    Calm,
    /// Gentle pacing guidance; timer visible but not strict.
    GuidedTimed,
    /// Light time constraint; slight urgency.
    Mild,
    /// Noticeable time pressure; forces prioritisation.
    Moderate,
    /// Full exam-equivalent time pressure.
    ExamPressure,
    /// Compressed beyond exam norms — elite conditioning.
    ElitePressure,
}

// ─── ErrorType ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Did not remember the underlying concept.
    ForgotConcept,
    /// Misread or misinterpreted the question wording.
    MisunderstoodWording,
    /// Knew the method but made an arithmetic or algebraic slip.
    CalculationSlip,
    /// Selected an answer without real knowledge.
    Guessed,
    /// Pressure or time caused thinking to break down.
    PanicUnderTime,
    /// Chose the most familiar-looking option rather than the correct one.
    ChoseFamiliarDistractor,
    /// Took the right topic but wrong first step in the solution path.
    WrongFirstStep,
    /// Started correctly but reasoning was incomplete or unsupported.
    IncompleteReasoning,
    /// Misread the unit, scale, or a keyword in the question.
    MisreadUnits,
    /// Knew the concept but made an avoidable, inattentive error.
    CarelessError,
}
```

---

## 3.3 Key Structs

```rust
// ecoach-substrate/src/structs.rs

use serde::{Deserialize, Serialize};
use crate::primitives::{BasisPoints, TimestampMs};

// ─── DomainEvent ─────────────────────────────────────────────────────────────

/// A single fact that has occurred in the system, stored in the event_log table.
/// Payload is an opaque JSON string; consumers deserialise to their own type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    /// UUID v4; globally unique identifier for this event instance.
    pub event_id: String,
    /// Canonical event type string, e.g. `"student.attempt.recorded"`.
    pub event_type: String,
    /// Stringified identifier of the root entity this event belongs to.
    pub aggregate_id: String,
    /// Wall-clock time when the event occurred, in milliseconds since epoch.
    pub occurred_at: TimestampMs,
    /// JSON-encoded event payload. Structure varies by event_type.
    pub payload: String,
    /// Optional correlation identifier linking a chain of related events.
    pub trace_id: Option<String>,
    /// Optional causation: the event_id that triggered this one.
    pub causation_id: Option<String>,
}

// ─── LearnerState ────────────────────────────────────────────────────────────

/// The seven-dimensional model of a learner's current academic reality.
/// Each field is a domain-specific map or estimate; types are in their own modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerState {
    /// Per-topic mastery scores and mastery-state classification.
    pub knowledge_state: TopicMasteryMap,
    /// Stability and confidence signals per topic.
    pub confidence_state: TopicConfidenceMap,
    /// Active and suspected misconceptions and their resolution status.
    pub misconception_state: MisconceptionSet,
    /// Dependency-gap map: which weaknesses block other learning.
    pub dependency_state: DependencyGapMap,
    /// Exam-readiness estimate across all required topics.
    pub readiness_state: ReadinessEstimate,
    /// Syllabus coverage: which nodes have been touched and to what depth.
    pub coverage_state: SyllabusCoverageMap,
    /// Current trajectory: improving, plateau, or regressing.
    pub momentum_state: MomentumTrend,
}

// Placeholder map/set types — concrete definitions live in their own domain modules.
// Declared here so LearnerState compiles in the substrate crate.

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicMasteryMap(pub std::collections::HashMap<i64, BasisPoints>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicConfidenceMap(pub std::collections::HashMap<i64, BasisPoints>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MisconceptionSet(pub Vec<ActiveMisconception>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMisconception {
    pub misconception_id: i64,
    pub topic_id: i64,
    pub strength_bp: BasisPoints,
    pub last_triggered_at: Option<TimestampMs>,
    pub is_resolved: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGapMap(pub Vec<DependencyGap>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGap {
    pub blocked_topic_id: i64,
    pub blocking_topic_id: i64,
    pub gap_severity_bp: BasisPoints,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReadinessEstimate {
    pub overall_bp: BasisPoints,
    pub mastery_component_bp: BasisPoints,
    pub retention_component_bp: BasisPoints,
    pub mock_component_bp: BasisPoints,
    pub speed_component_bp: BasisPoints,
    pub coverage_component_bp: BasisPoints,
    pub consistency_component_bp: BasisPoints,
    pub penalty_bp: BasisPoints,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyllabusCoverageMap(pub std::collections::HashMap<i64, CoverageEntry>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageEntry {
    pub topic_id: i64,
    pub seen: bool,
    pub depth_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumTrend {
    pub direction: crate::domain_enums::TrendDirection,
    pub momentum_score_bp: BasisPoints,
    /// EMA of accuracy over recent sessions.
    pub accuracy_ema_bp: BasisPoints,
    /// EMA of session volume over recent days.
    pub volume_ema: f64,
    /// EMA of practice pace (questions/minute).
    pub pace_ema: f64,
}

impl Default for MomentumTrend {
    fn default() -> Self {
        MomentumTrend {
            direction: crate::domain_enums::TrendDirection::Stable,
            momentum_score_bp: 5_000,
            accuracy_ema_bp: 0,
            volume_ema: 0.0,
            pace_ema: 0.0,
        }
    }
}

// ─── RiseStudentScores ───────────────────────────────────────────────────────

/// Eight internal intelligence scores that drive the Rise (weakest-to-best) engine.
/// All fields are BasisPoints (0–10_000).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiseStudentScores {
    /// How intact are prerequisite / foundational concepts? Higher = stronger foundation.
    pub foundation_score: BasisPoints,
    /// Retention quality: how well does the learner keep what they learned?
    pub recall_score: BasisPoints,
    /// Speed: correct solutions within expected time windows.
    pub speed_score: BasisPoints,
    /// Accuracy: proportion of correct answers across attempts.
    pub accuracy_score: BasisPoints,
    /// Stability under pressure: how much does performance drop when timed or pressured?
    pub pressure_stability_score: BasisPoints,
    /// Density of recurring wrong patterns (normalised; lower = fewer misconceptions).
    pub misconception_density: BasisPoints,
    /// Trajectory: rising / flat / falling over recent sessions.
    pub momentum_score: BasisPoints,
    /// Readiness to advance to the next Rise stage.
    pub transformation_readiness: BasisPoints,
}

// ─── EvidenceWeight ──────────────────────────────────────────────────────────

/// Computes the effective evidential weight of a single question attempt,
/// accounting for hints, transfer context, and recall delay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceWeight {
    /// Base weight before any modifiers (typically 1.0 for a clean unaided attempt).
    pub base: f64,
    /// Multiplicative hint factor: each hint halves the weight (0.5^n).
    /// Pass 1.0 if no hints were used.
    pub hint_factor: f64,
    /// Bonus multiplier when the question was a transfer/novel-context variant.
    pub transfer_bonus: f64,
    /// Bonus multiplier when the question was answered after a deliberate delay
    /// (spaced-recall check).
    pub recall_delay_bonus: f64,
}

impl EvidenceWeight {
    /// Create an EvidenceWeight for an unaided, non-transfer, non-delayed attempt.
    pub fn standard() -> Self {
        EvidenceWeight {
            base: 1.0,
            hint_factor: 1.0,
            transfer_bonus: 1.0,
            recall_delay_bonus: 1.0,
        }
    }

    /// Create an EvidenceWeight applying n hints.
    pub fn with_hints(hint_count: u8) -> Self {
        let hint_factor = 0.5_f64.powi(hint_count as i32);
        EvidenceWeight {
            base: 1.0,
            hint_factor,
            transfer_bonus: 1.0,
            recall_delay_bonus: 1.0,
        }
    }

    /// Compute the final effective weight after all modifiers.
    ///
    /// Formula: `base × hint_factor × transfer_bonus × recall_delay_bonus`
    pub fn effective_weight(&self) -> f64 {
        self.base * self.hint_factor * self.transfer_bonus * self.recall_delay_bonus
    }

    /// Convert the effective weight to a BasisPoints integer for storage.
    pub fn as_bp(&self) -> BasisPoints {
        crate::primitives::to_bp(self.effective_weight().clamp(0.0, 1.0))
    }
}
```

---

## 3.4 Constants Module

```rust
// ecoach-substrate/src/thresholds.rs

/// All numeric thresholds used across eCoach engines.
/// Values are in BasisPoints (u16, range 0–10_000) unless noted.
pub mod thresholds {

    // ── Mastery Gates ────────────────────────────────────────────────────────

    /// Below this score a topic is blocked — prerequisites not met.
    pub const MASTERY_BLOCKED_BELOW: u16 = 4_000;

    /// Minimum score for Functional mastery (reliable in standard conditions).
    pub const MASTERY_FUNCTIONAL_MIN: u16 = 5_500;

    /// Minimum score for Stable mastery (consistent across sessions).
    pub const MASTERY_STABLE_MIN: u16 = 7_000;

    /// Minimum score for ExamReady mastery (timed, exam-standard performance).
    pub const MASTERY_EXAM_READY_MIN: u16 = 8_500;

    /// Score at which mastery is considered fully achieved.
    pub const MASTERY_FULL: u16 = 9_000;

    // ── Evidence Weights ─────────────────────────────────────────────────────

    /// Each hint reduces the attempt's evidential weight by this factor.
    /// With n hints: weight = base × HINT_WEIGHT_MULTIPLIER^n.
    pub const HINT_WEIGHT_MULTIPLIER: f64 = 0.5;

    /// Multiplier added when an attempt is a transfer (novel-context) variant.
    pub const TRANSFER_WEIGHT_BONUS: f64 = 1.30;

    /// Multiplier added when an attempt occurs after a deliberate recall delay.
    pub const DELAYED_RECALL_BONUS: f64 = 1.50;

    // ── Exponential Moving Average ────────────────────────────────────────────

    /// Alpha for the EMA smoothing formula: new_ema = alpha × value + (1 − alpha) × old_ema.
    /// 0.3 weights recent observations meaningfully while dampening noise.
    pub const EMA_ALPHA: f64 = 0.3;

    // ── Session Duration ─────────────────────────────────────────────────────

    /// Hard ceiling on a single coaching session (minutes).
    pub const MAX_SESSION_MINUTES: u32 = 90;

    /// Minimum useful session length (minutes).
    pub const MIN_SESSION_MINUTES: u32 = 10;

    /// Default session target when the learner has not customised their schedule.
    pub const DEFAULT_SESSION_MINUTES: u32 = 45;

    // ── Memory Decay ─────────────────────────────────────────────────────────

    /// Memory strength at or below which the rescue scheduler activates.
    pub const MEMORY_RESCUE_THRESHOLD_BP: u16 = 4_000;

    /// Memory strength at or below which the item is classified Critical.
    pub const MEMORY_CRITICAL_THRESHOLD_BP: u16 = 2_500;

    // ── Gap Priority ─────────────────────────────────────────────────────────

    /// Gap score (as a 0.0–1.0 fraction) above which a gap is classified Urgent.
    pub const GAP_URGENT_THRESHOLD: f64 = 0.70;

    /// Gap score above which a gap is classified Critical.
    pub const GAP_CRITICAL_THRESHOLD: f64 = 0.85;

    // ── PIN Lockout ───────────────────────────────────────────────────────────

    /// Number of consecutive failed PIN attempts before lockout is applied.
    pub const PIN_MAX_ATTEMPTS: u8 = 5;

    /// Duration of the PIN lockout period (minutes).
    pub const PIN_LOCKOUT_MINUTES: u32 = 5;

    // ── Readiness Score Weights ───────────────────────────────────────────────
    // Source: backend_supplement section 1.5 (full formula)

    pub const READINESS_WEIGHT_MASTERY: f64 = 0.45;
    pub const READINESS_WEIGHT_TIMED_PERFORMANCE: f64 = 0.20;
    pub const READINESS_WEIGHT_COVERAGE: f64 = 0.15;
    pub const READINESS_WEIGHT_CONSISTENCY: f64 = 0.10;
    pub const READINESS_WEIGHT_TREND: f64 = 0.10;

    // ── Momentum Score Weights ────────────────────────────────────────────────

    pub const MOMENTUM_WEIGHT_VOLUME: f64 = 0.35;
    pub const MOMENTUM_WEIGHT_ACCURACY: f64 = 0.40;
    pub const MOMENTUM_WEIGHT_PACE: f64 = 0.25;

    // ── Strain Score Weights ──────────────────────────────────────────────────

    pub const STRAIN_WEIGHT_ACCURACY_DROP: f64 = 0.30;
    pub const STRAIN_WEIGHT_COMPLETION_DROP: f64 = 0.20;
    pub const STRAIN_WEIGHT_HINT_SPIKE: f64 = 0.20;
    pub const STRAIN_WEIGHT_SKIP: f64 = 0.15;
    pub const STRAIN_WEIGHT_PACE_INSTABILITY: f64 = 0.15;

    // ── Weakness Scoring Weights ──────────────────────────────────────────────

    pub const WEAKNESS_WEIGHT_MASTERY_DEFICIT: f64 = 0.35;
    pub const WEAKNESS_WEIGHT_LINK_BREAKAGE: f64 = 0.20;
    pub const WEAKNESS_WEIGHT_MISCONCEPTION_PRESSURE: f64 = 0.15;
    pub const WEAKNESS_WEIGHT_REPRESENTATION_GAP: f64 = 0.10;
    pub const WEAKNESS_WEIGHT_TIMED_GAP: f64 = 0.10;
    pub const WEAKNESS_WEIGHT_GUESS_PENALTY: f64 = 0.05;
    pub const WEAKNESS_WEIGHT_RECENCY_DECAY: f64 = 0.05;

    // ── Forecast Score Weights ────────────────────────────────────────────────

    pub const FORECAST_WEIGHT_FREQUENCY: f64 = 0.25;
    pub const FORECAST_WEIGHT_RECENCY: f64 = 0.20;
    pub const FORECAST_WEIGHT_TREND: f64 = 0.15;
    pub const FORECAST_WEIGHT_BUNDLE_STRENGTH: f64 = 0.15;
    pub const FORECAST_WEIGHT_SYLLABUS_PRIORITY: f64 = 0.10;
    pub const FORECAST_WEIGHT_STYLE_REGIME_FIT: f64 = 0.10;
    pub const FORECAST_WEIGHT_EXAMINER_GOAL_FIT: f64 = 0.05;

    // ── Orchestration Anti-Repeat Penalty ────────────────────────────────────

    pub const ORCHESTRATION_ANTI_REPEAT_PENALTY: f64 = 0.25;

    // ── Danger Zone Thresholds ────────────────────────────────────────────────

    /// Days of inactivity before the inactivity danger zone opens.
    pub const INACTIVITY_DANGER_DAYS: u32 = 3;

    /// Days of inactivity before the danger zone escalates to Urgent.
    pub const INACTIVITY_URGENT_DAYS: u32 = 7;

    // ── Exam Countdown ────────────────────────────────────────────────────────

    /// Days to exam at which ExamCountdown coach state activates.
    pub const EXAM_COUNTDOWN_TRIGGER_DAYS: u32 = 7;

    /// Days to exam at which CrisisIntervention may be triggered if readiness is low.
    pub const CRISIS_INTERVENTION_DAYS_THRESHOLD: u32 = 14;

    /// Readiness score (bp) below which CrisisIntervention is triggered in final window.
    pub const CRISIS_READINESS_THRESHOLD_BP: u16 = 4_500;
}
```

---

## 3.5 Crate Root

```rust
// ecoach-substrate/src/lib.rs

pub mod primitives;
pub mod domain_enums;
pub mod structs;
pub mod thresholds;

// Re-export the most frequently used items at crate root.
pub use primitives::{
    to_bp, from_bp, BasisPoints, ConfidenceScore,
    SeverityLevel, TrendDirection, TimestampMs, DurationMs,
};
pub use domain_enums::{
    Role, EntitlementTier, CoachLifecycleState, MemoryState,
    MasteryState, MockType, JourneyPhase, RiseStage,
    KnowledgeGapType, QuestionIntent, LearnerArchetype,
    PressureLevel, ErrorType,
};
pub use structs::{
    DomainEvent, LearnerState, RiseStudentScores, EvidenceWeight,
    TopicMasteryMap, TopicConfidenceMap, MisconceptionSet,
    ActiveMisconception, DependencyGapMap, DependencyGap,
    ReadinessEstimate, SyllabusCoverageMap, CoverageEntry, MomentumTrend,
};
pub use thresholds::thresholds;
```
