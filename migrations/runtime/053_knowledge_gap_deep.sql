-- Knowledge Gap Mode deep features: snapshots, feed items, aggregate breakdown.

-- Gap snapshots for trend visualization
CREATE TABLE gap_snapshots (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    total_gap_percent INTEGER NOT NULL DEFAULT 0,
    unknown_percent INTEGER NOT NULL DEFAULT 0,
    weak_percent INTEGER NOT NULL DEFAULT 0,
    declining_percent INTEGER NOT NULL DEFAULT 0,
    forgetting_percent INTEGER NOT NULL DEFAULT 0,
    critical_percent INTEGER NOT NULL DEFAULT 0,
    total_skills INTEGER NOT NULL DEFAULT 0,
    mastered_skills INTEGER NOT NULL DEFAULT 0,
    critical_blockers INTEGER NOT NULL DEFAULT 0,
    recently_fixed INTEGER NOT NULL DEFAULT 0,
    snapshot_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_gap_snapshots_student ON gap_snapshots(student_id, subject_id);
CREATE INDEX idx_gap_snapshots_date ON gap_snapshots(snapshot_at);

-- Knowledge update feed items (live gap change notifications)
CREATE TABLE knowledge_update_feed (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    topic_id INTEGER,
    event_type TEXT NOT NULL,
    message TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'info',
    context_json TEXT NOT NULL DEFAULT '{}',
    read_flag INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_knowledge_feed_student ON knowledge_update_feed(student_id, subject_id);
CREATE INDEX idx_knowledge_feed_unread ON knowledge_update_feed(read_flag);
