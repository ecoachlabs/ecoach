CREATE TABLE IF NOT EXISTS student_skill_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    mastery_score INTEGER NOT NULL DEFAULT 0,
    gap_score INTEGER NOT NULL DEFAULT 10000,
    priority_score INTEGER NOT NULL DEFAULT 0,
    evidence_count INTEGER NOT NULL DEFAULT 0,
    total_attempts INTEGER NOT NULL DEFAULT 0,
    correct_attempts INTEGER NOT NULL DEFAULT 0,
    last_seen_at TEXT,
    last_correct_at TEXT,
    state TEXT NOT NULL DEFAULT 'unseen'
        CHECK (state IN ('unseen', 'emerging', 'functional', 'stable', 'exam_ready')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, node_id)
);

CREATE INDEX IF NOT EXISTS idx_student_skill_states_student ON student_skill_states(student_id);
CREATE INDEX IF NOT EXISTS idx_student_skill_states_node ON student_skill_states(node_id);
CREATE INDEX IF NOT EXISTS idx_student_skill_states_priority ON student_skill_states(priority_score DESC);
