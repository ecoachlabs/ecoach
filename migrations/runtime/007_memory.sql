CREATE TABLE IF NOT EXISTS memory_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER REFERENCES topics(id),
    node_id INTEGER REFERENCES academic_nodes(id),
    memory_state TEXT NOT NULL DEFAULT 'seen'
        CHECK (memory_state IN ('seen', 'encoded', 'accessible', 'fragile', 'anchoring', 'confirmed', 'locked_in', 'at_risk', 'fading', 'rebuilding', 'recovered', 'collapsed')),
    memory_strength INTEGER NOT NULL DEFAULT 0,
    recall_fluency INTEGER NOT NULL DEFAULT 0,
    decay_risk INTEGER NOT NULL DEFAULT 0,
    review_due_at TEXT,
    last_recalled_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS memory_evidence_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    node_id INTEGER REFERENCES academic_nodes(id),
    topic_id INTEGER REFERENCES topics(id),
    recall_mode TEXT,
    cue_level TEXT,
    delay_bucket TEXT,
    interference_detected INTEGER NOT NULL DEFAULT 0,
    was_correct INTEGER NOT NULL DEFAULT 0,
    confidence_level TEXT,
    evidence_weight INTEGER NOT NULL DEFAULT 10000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS recheck_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    node_id INTEGER REFERENCES academic_nodes(id),
    due_at TEXT NOT NULL,
    schedule_type TEXT NOT NULL DEFAULT 'spaced_review',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'completed', 'missed', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS interference_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    to_node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    strength_score INTEGER NOT NULL DEFAULT 5000,
    last_seen_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS gap_repair_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('draft', 'active', 'completed', 'abandoned')),
    priority_score INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS gap_repair_plan_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id INTEGER NOT NULL REFERENCES gap_repair_plans(id) ON DELETE CASCADE,
    node_id INTEGER REFERENCES academic_nodes(id),
    sequence_order INTEGER NOT NULL DEFAULT 0,
    repair_action TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'skipped')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS solidification_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    repair_plan_id INTEGER REFERENCES gap_repair_plans(id),
    session_id INTEGER REFERENCES sessions(id),
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'active', 'completed', 'abandoned')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_memory_states_student ON memory_states(student_id);
CREATE INDEX IF NOT EXISTS idx_memory_states_review_due ON memory_states(review_due_at);
CREATE INDEX IF NOT EXISTS idx_memory_events_student ON memory_evidence_events(student_id);
CREATE INDEX IF NOT EXISTS idx_recheck_schedules_due ON recheck_schedules(due_at);
CREATE INDEX IF NOT EXISTS idx_gap_repair_plans_student ON gap_repair_plans(student_id);
