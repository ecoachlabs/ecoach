-- Student link mastery: track understanding at the node_edge level.
-- Student representation strength: track per-format accuracy.
-- Family promotion/quarantine status column.

CREATE TABLE student_link_mastery (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    from_node_id INTEGER NOT NULL,
    to_node_id INTEGER NOT NULL,
    edge_type TEXT NOT NULL,
    mastery_score INTEGER NOT NULL DEFAULT 0,
    total_attempts INTEGER NOT NULL DEFAULT 0,
    correct_attempts INTEGER NOT NULL DEFAULT 0,
    last_seen_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, from_node_id, to_node_id, edge_type)
);

CREATE INDEX idx_student_link_mastery_student ON student_link_mastery(student_id);
CREATE INDEX idx_student_link_mastery_nodes ON student_link_mastery(from_node_id, to_node_id);

CREATE TABLE student_representation_strength (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    format_code TEXT NOT NULL,
    accuracy_bp INTEGER NOT NULL DEFAULT 5000,
    total_attempts INTEGER NOT NULL DEFAULT 0,
    correct_attempts INTEGER NOT NULL DEFAULT 0,
    last_seen_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, format_code)
);

CREATE INDEX idx_student_repr_strength_student ON student_representation_strength(student_id);
