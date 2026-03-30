-- Forecast calibration: backtest accuracy tracking.
-- Curriculum crosswalk: map legacy syllabus labels to current topic IDs.

CREATE TABLE forecast_calibration_runs (
    id INTEGER PRIMARY KEY,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    holdout_year INTEGER NOT NULL,
    brier_score INTEGER NOT NULL DEFAULT 0,
    coverage_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    coefficient_adjustments_json TEXT,
    topics_evaluated INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_forecast_calibration_subject ON forecast_calibration_runs(subject_id);

CREATE TABLE curriculum_crosswalk (
    id INTEGER PRIMARY KEY,
    legacy_label TEXT NOT NULL,
    legacy_source TEXT,
    current_topic_id INTEGER REFERENCES topics(id),
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_curriculum_crosswalk_label ON curriculum_crosswalk(legacy_label);
CREATE INDEX idx_curriculum_crosswalk_topic ON curriculum_crosswalk(current_topic_id);
