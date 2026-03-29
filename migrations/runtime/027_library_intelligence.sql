ALTER TABLE library_items ADD COLUMN note_text TEXT;
ALTER TABLE library_items ADD COLUMN topic_id INTEGER REFERENCES topics(id);
ALTER TABLE library_items ADD COLUMN urgency_score INTEGER NOT NULL DEFAULT 5000;

CREATE TABLE IF NOT EXISTS library_shelf_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shelf_id INTEGER NOT NULL REFERENCES library_shelves(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    item_ref_id INTEGER,
    title TEXT NOT NULL,
    subtitle TEXT,
    reason TEXT NOT NULL,
    rank_score INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    sequence_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS revision_pack_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pack_id INTEGER NOT NULL REFERENCES revision_packs(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    item_ref_id INTEGER NOT NULL,
    sequence_order INTEGER NOT NULL DEFAULT 0,
    required INTEGER NOT NULL DEFAULT 1,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_library_items_student_state ON library_items(student_id, state);
CREATE INDEX IF NOT EXISTS idx_library_items_topic ON library_items(topic_id);
CREATE INDEX IF NOT EXISTS idx_library_shelf_items_shelf ON library_shelf_items(shelf_id);
CREATE INDEX IF NOT EXISTS idx_library_shelf_items_type ON library_shelf_items(item_type);
CREATE INDEX IF NOT EXISTS idx_revision_pack_items_pack ON revision_pack_items(pack_id);
