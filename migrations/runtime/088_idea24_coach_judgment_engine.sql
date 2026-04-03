CREATE TABLE IF NOT EXISTS coach_evidence_ledger_entries (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    ledger_code TEXT NOT NULL,
    ledger_label TEXT NOT NULL,
    evidence_score INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'watch',
    summary TEXT NOT NULL,
    details_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_key, topic_key, ledger_code)
);

CREATE INDEX IF NOT EXISTS idx_coach_evidence_ledger_student
    ON coach_evidence_ledger_entries(student_id, subject_key, topic_key, evidence_score DESC);

CREATE TABLE IF NOT EXISTS coach_feature_activations (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    feature_code TEXT NOT NULL,
    feature_label TEXT NOT NULL,
    activation_priority_score INTEGER NOT NULL DEFAULT 0,
    urgency_score INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    readiness_guardrail TEXT NOT NULL DEFAULT 'monitor',
    rationale TEXT NOT NULL,
    payload_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_key, topic_key, feature_code)
);

CREATE INDEX IF NOT EXISTS idx_coach_feature_activations_student
    ON coach_feature_activations(student_id, subject_key, topic_key, activation_priority_score DESC);

CREATE TABLE IF NOT EXISTS coach_content_governor_snapshots (
    id INTEGER PRIMARY KEY,
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    readiness_score INTEGER NOT NULL DEFAULT 0,
    quality_score INTEGER NOT NULL DEFAULT 0,
    provenance_score INTEGER NOT NULL DEFAULT 0,
    contradiction_risk_score INTEGER NOT NULL DEFAULT 0,
    quality_state TEXT NOT NULL DEFAULT 'watch',
    blocking_issues_json TEXT NOT NULL DEFAULT '[]',
    evidence_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(subject_key, topic_key)
);

CREATE INDEX IF NOT EXISTS idx_coach_content_governor_topic
    ON coach_content_governor_snapshots(subject_key, topic_key, quality_score DESC);

CREATE TABLE IF NOT EXISTS coach_independence_reviews (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_key INTEGER NOT NULL DEFAULT 0,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE CASCADE,
    topic_key INTEGER NOT NULL DEFAULT 0,
    topic_id INTEGER REFERENCES topics(id) ON DELETE CASCADE,
    overall_score INTEGER NOT NULL DEFAULT 0,
    judgment_confidence_score INTEGER NOT NULL DEFAULT 0,
    independence_band TEXT NOT NULL DEFAULT 'fragile',
    biggest_risk TEXT NOT NULL,
    next_best_move TEXT NOT NULL,
    capability_json TEXT NOT NULL DEFAULT '[]',
    summary_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_key, topic_key)
);

CREATE INDEX IF NOT EXISTS idx_coach_independence_reviews_student
    ON coach_independence_reviews(student_id, subject_key, topic_key, overall_score DESC);
