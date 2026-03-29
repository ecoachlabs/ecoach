CREATE TABLE IF NOT EXISTS parent_dashboards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_account_id INTEGER NOT NULL REFERENCES accounts(id),
    student_account_id INTEGER NOT NULL REFERENCES accounts(id),
    readiness_band TEXT,
    risk_summary_json TEXT NOT NULL DEFAULT '[]',
    trend_summary_json TEXT NOT NULL DEFAULT '[]',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(parent_account_id, student_account_id)
);

CREATE TABLE IF NOT EXISTS weekly_memos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    audience TEXT NOT NULL DEFAULT 'parent'
        CHECK (audience IN ('student', 'parent', 'admin')),
    week_start TEXT NOT NULL,
    memo_body TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
