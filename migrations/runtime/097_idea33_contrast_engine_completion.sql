CREATE TABLE IF NOT EXISTS contrast_pair_profiles (
    pair_id INTEGER PRIMARY KEY REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    left_profile_json TEXT NOT NULL DEFAULT '{}',
    right_profile_json TEXT NOT NULL DEFAULT '{}',
    shared_traits_json TEXT NOT NULL DEFAULT '[]',
    decisive_differences_json TEXT NOT NULL DEFAULT '[]',
    common_confusions_json TEXT NOT NULL DEFAULT '[]',
    trap_angles_json TEXT NOT NULL DEFAULT '[]',
    coverage_json TEXT NOT NULL DEFAULT '{}',
    generator_contract_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS contrast_concept_attributes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    concept_side TEXT NOT NULL CHECK (concept_side IN ('left', 'right')),
    lane TEXT NOT NULL,
    attribute_label TEXT NOT NULL,
    attribute_value TEXT NOT NULL,
    importance_weight_bp INTEGER NOT NULL DEFAULT 5000,
    difficulty_score INTEGER NOT NULL DEFAULT 5000,
    source_confidence_bp INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_contrast_concept_attributes_pair_side_lane
    ON contrast_concept_attributes(pair_id, concept_side, lane);

CREATE TABLE IF NOT EXISTS contrast_diagram_assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    concept_side TEXT CHECK (concept_side IN ('left', 'right', 'both', 'neither')),
    lane TEXT NOT NULL DEFAULT 'diagram',
    diagram_type TEXT NOT NULL DEFAULT 'reference',
    asset_ref TEXT NOT NULL,
    prompt_payload_json TEXT NOT NULL DEFAULT '{}',
    visual_clues_json TEXT NOT NULL DEFAULT '[]',
    decisive_visual_clue TEXT,
    trap_potential TEXT,
    usable_modes_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_contrast_diagram_assets_pair
    ON contrast_diagram_assets(pair_id, lane);

CREATE TABLE IF NOT EXISTS contrast_comparison_rows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    lane TEXT NOT NULL,
    compare_label TEXT NOT NULL,
    left_value TEXT NOT NULL,
    right_value TEXT NOT NULL,
    overlap_note TEXT,
    decisive_clue TEXT,
    teaching_note TEXT,
    diagram_asset_id INTEGER REFERENCES contrast_diagram_assets(id) ON DELETE SET NULL,
    display_order INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_contrast_comparison_rows_pair
    ON contrast_comparison_rows(pair_id, display_order, id);

CREATE TABLE IF NOT EXISTS contrast_misconception_reasons (
    code TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    category TEXT NOT NULL,
    modes_json TEXT NOT NULL DEFAULT '[]',
    display_order INTEGER NOT NULL DEFAULT 100,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS contrast_mode_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    mode TEXT NOT NULL
        CHECK (mode IN ('difference_drill', 'similarity_trap', 'know_the_difference', 'which_is_which', 'unmask')),
    source_atom_id INTEGER REFERENCES contrast_evidence_atoms(id) ON DELETE SET NULL,
    comparison_row_id INTEGER REFERENCES contrast_comparison_rows(id) ON DELETE SET NULL,
    diagram_asset_id INTEGER REFERENCES contrast_diagram_assets(id) ON DELETE SET NULL,
    prompt_type TEXT NOT NULL DEFAULT 'text_card',
    prompt_text TEXT NOT NULL,
    prompt_payload_json TEXT NOT NULL DEFAULT '{}',
    options_json TEXT NOT NULL DEFAULT '[]',
    correct_choice_code TEXT,
    correct_choice_label TEXT,
    difficulty_score INTEGER NOT NULL DEFAULT 5000,
    time_limit_seconds INTEGER,
    explanation_bundle_json TEXT NOT NULL DEFAULT '{}',
    misconception_reason_codes_json TEXT NOT NULL DEFAULT '[]',
    is_active INTEGER NOT NULL DEFAULT 1,
    display_order INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_contrast_mode_items_pair_mode
    ON contrast_mode_items(pair_id, mode, is_active, display_order, id);

ALTER TABLE contrast_evidence_atoms ADD COLUMN item_forms_json TEXT NOT NULL DEFAULT '["difference_drill","similarity_trap","know_the_difference","which_is_which","unmask"]';
ALTER TABLE contrast_evidence_atoms ADD COLUMN diagram_capable INTEGER NOT NULL DEFAULT 0;
ALTER TABLE contrast_evidence_atoms ADD COLUMN trap_angle TEXT;
ALTER TABLE contrast_evidence_atoms ADD COLUMN review_payload_json TEXT NOT NULL DEFAULT '{}';

ALTER TABLE traps_rounds ADD COLUMN mode_item_id INTEGER REFERENCES contrast_mode_items(id) ON DELETE SET NULL;
ALTER TABLE traps_rounds ADD COLUMN review_payload_json TEXT NOT NULL DEFAULT '{}';

INSERT OR IGNORE INTO contrast_misconception_reasons (
    code, label, category, modes_json, display_order, is_active
) VALUES
    ('sound_similar', 'They sound similar', 'naming_confusion', '["difference_drill","similarity_trap","which_is_which","unmask"]', 10, 1),
    ('feature_confusion', 'I know both terms but I mix up their features', 'feature_confusion', '["difference_drill","similarity_trap","which_is_which"]', 20, 1),
    ('definition_feature_gap', 'I know the definitions but not the features', 'definition_confusion', '["difference_drill","know_the_difference"]', 30, 1),
    ('example_confusion', 'I confused the examples', 'example_confusion', '["difference_drill","similarity_trap","know_the_difference","which_is_which"]', 40, 1),
    ('speed_rush', 'I rushed because of time', 'speed_pressure', '["difference_drill","similarity_trap","which_is_which","unmask"]', 50, 1),
    ('guessed', 'I guessed', 'guessing', '["difference_drill","similarity_trap","which_is_which","unmask"]', 60, 1),
    ('condition_omission', 'I forgot the condition', 'condition_confusion', '["difference_drill","similarity_trap","know_the_difference","which_is_which"]', 70, 1),
    ('diagram_misread', 'I misread the diagram clue', 'diagram_misreading', '["difference_drill","similarity_trap","know_the_difference","which_is_which","unmask"]', 80, 1),
    ('partial_understanding', 'I do not fully understand either term yet', 'incomplete_boundary', '["difference_drill","similarity_trap","know_the_difference","which_is_which","unmask"]', 90, 1),
    ('other', 'Other', 'other', '["difference_drill","similarity_trap","know_the_difference","which_is_which","unmask"]', 100, 1);
