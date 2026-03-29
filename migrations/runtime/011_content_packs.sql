CREATE TABLE IF NOT EXISTS content_packs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pack_id TEXT NOT NULL UNIQUE,
    pack_version TEXT NOT NULL,
    subject_code TEXT NOT NULL,
    curriculum_version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'installing'
        CHECK (status IN ('installing', 'installed', 'active', 'failed', 'removed')),
    topic_count INTEGER NOT NULL DEFAULT 0,
    question_count INTEGER NOT NULL DEFAULT 0,
    install_path TEXT NOT NULL,
    manifest_json TEXT NOT NULL,
    installed_at TEXT NOT NULL DEFAULT (datetime('now')),
    activated_at TEXT,
    error_message TEXT
);
