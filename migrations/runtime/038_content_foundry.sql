CREATE TABLE IF NOT EXISTS topic_package_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    package_state TEXT NOT NULL DEFAULT 'unseeded'
        CHECK (package_state IN (
            'unseeded', 'foundation_seeded', 'partially_supported',
            'content_building', 'quality_weak', 'quality_mixed',
            'quality_stable', 'quality_strong', 'needs_refresh',
            'under_revision', 'live_strong', 'retired'
        )),
    live_health_state TEXT NOT NULL DEFAULT 'unseeded'
        CHECK (live_health_state IN (
            'unseeded', 'foundation_seeded', 'partially_supported',
            'content_building', 'quality_weak', 'quality_mixed',
            'quality_stable', 'quality_strong', 'needs_refresh',
            'under_revision', 'live_strong', 'retired'
        )),
    resource_readiness_score INTEGER NOT NULL DEFAULT 0,
    completeness_score INTEGER NOT NULL DEFAULT 0,
    quality_score INTEGER NOT NULL DEFAULT 0,
    evidence_score INTEGER NOT NULL DEFAULT 0,
    source_support_count INTEGER NOT NULL DEFAULT 0,
    contrast_pair_count INTEGER NOT NULL DEFAULT 0,
    publishable_artifact_count INTEGER NOT NULL DEFAULT 0,
    published_artifact_count INTEGER NOT NULL DEFAULT 0,
    missing_components_json TEXT NOT NULL DEFAULT '[]',
    recommended_jobs_json TEXT NOT NULL DEFAULT '[]',
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(subject_id, topic_id)
);

CREATE INDEX IF NOT EXISTS idx_topic_package_snapshots_subject
    ON topic_package_snapshots(subject_id, completeness_score DESC);

CREATE INDEX IF NOT EXISTS idx_topic_package_snapshots_topic
    ON topic_package_snapshots(topic_id, live_health_state);
