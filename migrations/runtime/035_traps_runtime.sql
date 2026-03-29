ALTER TABLE contrast_pairs ADD COLUMN pair_code TEXT;
ALTER TABLE contrast_pairs ADD COLUMN subject_id INTEGER REFERENCES subjects(id);
ALTER TABLE contrast_pairs ADD COLUMN topic_id INTEGER REFERENCES topics(id);
ALTER TABLE contrast_pairs ADD COLUMN left_label TEXT;
ALTER TABLE contrast_pairs ADD COLUMN right_label TEXT;
ALTER TABLE contrast_pairs ADD COLUMN summary_text TEXT;
ALTER TABLE contrast_pairs ADD COLUMN difficulty_score INTEGER NOT NULL DEFAULT 5000;

ALTER TABLE contrast_evidence_atoms ADD COLUMN lane TEXT NOT NULL DEFAULT 'feature';
ALTER TABLE contrast_evidence_atoms ADD COLUMN explanation_text TEXT;
ALTER TABLE contrast_evidence_atoms ADD COLUMN difficulty_score INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE contrast_evidence_atoms ADD COLUMN is_speed_ready INTEGER NOT NULL DEFAULT 1;
ALTER TABLE contrast_evidence_atoms ADD COLUMN reveal_order INTEGER NOT NULL DEFAULT 1;

CREATE UNIQUE INDEX IF NOT EXISTS idx_contrast_pairs_pair_code
    ON contrast_pairs(pair_code)
    WHERE pair_code IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_contrast_pairs_subject_topic
    ON contrast_pairs(subject_id, topic_id);

CREATE INDEX IF NOT EXISTS idx_contrast_atoms_pair_lane
    ON contrast_evidence_atoms(pair_id, lane);

CREATE TABLE IF NOT EXISTS traps_rounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    atom_id INTEGER REFERENCES contrast_evidence_atoms(id) ON DELETE SET NULL,
    round_number INTEGER NOT NULL,
    mode TEXT NOT NULL
        CHECK (mode IN ('difference_drill', 'similarity_trap', 'know_the_difference', 'which_is_which', 'unmask')),
    lane TEXT NOT NULL DEFAULT 'feature',
    prompt_text TEXT NOT NULL,
    prompt_payload_json TEXT NOT NULL DEFAULT '{}',
    options_json TEXT NOT NULL DEFAULT '[]',
    correct_choice_code TEXT NOT NULL,
    correct_choice_label TEXT NOT NULL,
    selected_choice_code TEXT,
    selected_choice_label TEXT,
    is_correct INTEGER,
    response_time_ms INTEGER,
    timed_out INTEGER NOT NULL DEFAULT 0,
    skipped INTEGER NOT NULL DEFAULT 0,
    reveal_count INTEGER NOT NULL DEFAULT 1,
    max_reveal_count INTEGER NOT NULL DEFAULT 1,
    confusion_reason_code TEXT,
    confusion_reason_text TEXT,
    explanation_text TEXT,
    score_earned INTEGER NOT NULL DEFAULT 0,
    answered_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(game_session_id, round_number)
);

CREATE INDEX IF NOT EXISTS idx_traps_rounds_session
    ON traps_rounds(game_session_id, round_number);

CREATE INDEX IF NOT EXISTS idx_traps_rounds_pair
    ON traps_rounds(pair_id, mode);

CREATE TABLE IF NOT EXISTS student_contrast_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    fluency_bp INTEGER NOT NULL DEFAULT 0,
    confusion_score INTEGER NOT NULL DEFAULT 5000,
    difference_drill_bp INTEGER NOT NULL DEFAULT 0,
    similarity_trap_bp INTEGER NOT NULL DEFAULT 0,
    know_difference_bp INTEGER NOT NULL DEFAULT 0,
    which_is_which_bp INTEGER NOT NULL DEFAULT 0,
    unmask_bp INTEGER NOT NULL DEFAULT 0,
    rounds_played INTEGER NOT NULL DEFAULT 0,
    rounds_correct INTEGER NOT NULL DEFAULT 0,
    timed_out_count INTEGER NOT NULL DEFAULT 0,
    average_response_time_ms INTEGER NOT NULL DEFAULT 0,
    weakest_lane TEXT,
    last_mode TEXT,
    confusion_reason_profile_json TEXT NOT NULL DEFAULT '{}',
    last_practiced_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, pair_id)
);

CREATE INDEX IF NOT EXISTS idx_student_contrast_states_student
    ON student_contrast_states(student_id, confusion_score DESC);
