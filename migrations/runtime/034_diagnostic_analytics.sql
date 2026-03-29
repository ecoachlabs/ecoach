CREATE TABLE IF NOT EXISTS diagnostic_topic_analytics (
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    mastery_score INTEGER NOT NULL DEFAULT 0,
    fluency_score INTEGER NOT NULL DEFAULT 0,
    precision_score INTEGER NOT NULL DEFAULT 0,
    pressure_score INTEGER NOT NULL DEFAULT 0,
    flexibility_score INTEGER NOT NULL DEFAULT 0,
    stability_score INTEGER NOT NULL DEFAULT 0,
    classification TEXT NOT NULL,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    recommended_action TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (diagnostic_id, topic_id)
);

CREATE TABLE IF NOT EXISTS diagnostic_root_cause_hypotheses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    hypothesis_code TEXT NOT NULL,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    recommended_action TEXT NOT NULL,
    evidence_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(diagnostic_id, topic_id, hypothesis_code)
);

CREATE INDEX IF NOT EXISTS idx_diagnostic_topic_analytics_diagnostic
    ON diagnostic_topic_analytics(diagnostic_id, topic_id);

CREATE INDEX IF NOT EXISTS idx_diagnostic_root_cause_hypotheses_diagnostic
    ON diagnostic_root_cause_hypotheses(diagnostic_id, topic_id, confidence_score DESC);
