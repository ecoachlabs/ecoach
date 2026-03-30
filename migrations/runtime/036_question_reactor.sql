CREATE TABLE IF NOT EXISTS question_generation_requests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    family_id INTEGER REFERENCES question_families(id),
    source_question_id INTEGER REFERENCES questions(id),
    request_kind TEXT NOT NULL DEFAULT 'variant'
        CHECK (request_kind IN ('variant', 'slot_fill', 'diagnostic_probe', 'remediation_fill')),
    variant_mode TEXT NOT NULL
        CHECK (variant_mode IN ('isomorphic', 'representation_shift', 'misconception_probe', 'rescue', 'stretch')),
    requested_count INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'processing', 'completed', 'failed')),
    constraints_json TEXT NOT NULL DEFAULT '{}',
    rationale TEXT,
    generated_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS question_lineage_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL UNIQUE REFERENCES questions(id) ON DELETE CASCADE,
    family_id INTEGER REFERENCES question_families(id),
    lineage_key TEXT NOT NULL UNIQUE,
    node_role TEXT NOT NULL DEFAULT 'instance'
        CHECK (node_role IN ('seed', 'instance', 'variant', 'diagnostic_probe', 'repair_variant')),
    origin_kind TEXT NOT NULL DEFAULT 'authored'
        CHECK (origin_kind IN ('authored', 'past_question', 'generated', 'teacher_upload')),
    fingerprint_text TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS question_lineage_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_node_id INTEGER NOT NULL REFERENCES question_lineage_nodes(id) ON DELETE CASCADE,
    to_node_id INTEGER NOT NULL REFERENCES question_lineage_nodes(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL DEFAULT 'generated_from'
        CHECK (relation_type IN ('generated_from', 'variant_of', 'representation_shift', 'difficulty_ladder', 'misconception_probe', 'repair_variant')),
    transform_mode TEXT,
    rationale TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(from_node_id, to_node_id, relation_type)
);

CREATE TABLE IF NOT EXISTS question_transform_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id INTEGER REFERENCES question_generation_requests(id) ON DELETE SET NULL,
    family_id INTEGER REFERENCES question_families(id),
    source_question_id INTEGER NOT NULL REFERENCES questions(id),
    generated_question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    variant_mode TEXT NOT NULL,
    transform_summary TEXT NOT NULL,
    transform_payload TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS question_family_health (
    family_id INTEGER PRIMARY KEY REFERENCES question_families(id) ON DELETE CASCADE,
    total_instances INTEGER NOT NULL DEFAULT 0,
    generated_instances INTEGER NOT NULL DEFAULT 0,
    active_instances INTEGER NOT NULL DEFAULT 0,
    recent_attempts INTEGER NOT NULL DEFAULT 0,
    recent_correct_attempts INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms INTEGER NOT NULL DEFAULT 0,
    misconception_hit_count INTEGER NOT NULL DEFAULT 0,
    freshness_score INTEGER NOT NULL DEFAULT 0,
    calibration_score INTEGER NOT NULL DEFAULT 0,
    quality_score INTEGER NOT NULL DEFAULT 0,
    health_status TEXT NOT NULL DEFAULT 'warming'
        CHECK (health_status IN ('missing', 'warming', 'active', 'gold', 'fragile')),
    last_generated_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_generation_requests_status ON question_generation_requests(status);
CREATE INDEX IF NOT EXISTS idx_generation_requests_family ON question_generation_requests(family_id);
CREATE INDEX IF NOT EXISTS idx_lineage_nodes_family ON question_lineage_nodes(family_id);
CREATE INDEX IF NOT EXISTS idx_lineage_edges_from ON question_lineage_edges(from_node_id);
CREATE INDEX IF NOT EXISTS idx_lineage_edges_to ON question_lineage_edges(to_node_id);
CREATE INDEX IF NOT EXISTS idx_transform_log_request ON question_transform_log(request_id);
CREATE INDEX IF NOT EXISTS idx_transform_log_family ON question_transform_log(family_id);
