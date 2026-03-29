CREATE TABLE IF NOT EXISTS contrast_pairs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    left_entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    right_entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    trap_strength INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS contrast_evidence_atoms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    ownership_type TEXT NOT NULL
        CHECK (ownership_type IN ('left_only', 'right_only', 'both', 'neither')),
    atom_text TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
