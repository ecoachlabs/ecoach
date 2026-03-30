-- Forecast engine: probabilistic exam blueprint generation from past paper patterns.

CREATE TABLE forecast_snapshots (
    id INTEGER PRIMARY KEY,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    total_papers_analyzed INTEGER NOT NULL DEFAULT 0,
    year_range_start INTEGER,
    year_range_end INTEGER,
    blueprint_json TEXT NOT NULL DEFAULT '{}',
    pattern_profile_json TEXT NOT NULL DEFAULT '{}',
    confidence_score INTEGER NOT NULL DEFAULT 0,
    computed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_forecast_snapshots_subject ON forecast_snapshots(subject_id);
CREATE INDEX idx_forecast_snapshots_computed ON forecast_snapshots(computed_at);

CREATE TABLE forecast_topic_scores (
    id INTEGER PRIMARY KEY,
    snapshot_id INTEGER NOT NULL REFERENCES forecast_snapshots(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    frequency_score INTEGER NOT NULL DEFAULT 0,
    recency_score INTEGER NOT NULL DEFAULT 0,
    trend_score INTEGER NOT NULL DEFAULT 0,
    bundle_strength INTEGER NOT NULL DEFAULT 0,
    syllabus_priority INTEGER NOT NULL DEFAULT 0,
    style_regime_fit INTEGER NOT NULL DEFAULT 0,
    examiner_goal_fit INTEGER NOT NULL DEFAULT 0,
    composite_score INTEGER NOT NULL DEFAULT 0,
    uncertainty_band TEXT NOT NULL DEFAULT 'medium'
);

CREATE INDEX idx_forecast_topic_scores_snapshot ON forecast_topic_scores(snapshot_id);
CREATE INDEX idx_forecast_topic_scores_topic ON forecast_topic_scores(topic_id);
CREATE INDEX idx_forecast_topic_scores_composite ON forecast_topic_scores(composite_score);

CREATE TABLE forecast_format_scores (
    id INTEGER PRIMARY KEY,
    snapshot_id INTEGER NOT NULL REFERENCES forecast_snapshots(id),
    format_code TEXT NOT NULL,
    probability_score INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_forecast_format_scores_snapshot ON forecast_format_scores(snapshot_id);

CREATE TABLE forecast_difficulty_scores (
    id INTEGER PRIMARY KEY,
    snapshot_id INTEGER NOT NULL REFERENCES forecast_snapshots(id),
    difficulty_band TEXT NOT NULL,
    probability_score INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_forecast_difficulty_scores_snapshot ON forecast_difficulty_scores(snapshot_id);

CREATE TABLE forecast_bundles (
    id INTEGER PRIMARY KEY,
    snapshot_id INTEGER NOT NULL REFERENCES forecast_snapshots(id),
    bundle_key TEXT NOT NULL,
    topic_ids_json TEXT NOT NULL DEFAULT '[]',
    co_occurrence_count INTEGER NOT NULL DEFAULT 0,
    strength_score INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_forecast_bundles_snapshot ON forecast_bundles(snapshot_id);
