-- Distractor health tracking: per-option effectiveness metrics.

CREATE TABLE distractor_health (
    id INTEGER PRIMARY KEY,
    option_id INTEGER NOT NULL,
    question_id INTEGER NOT NULL,
    times_selected INTEGER NOT NULL DEFAULT 0,
    times_shown INTEGER NOT NULL DEFAULT 0,
    selection_rate_bp INTEGER NOT NULL DEFAULT 0,
    is_dead INTEGER NOT NULL DEFAULT 0,
    last_selected_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_distractor_health_question ON distractor_health(question_id);
CREATE INDEX idx_distractor_health_dead ON distractor_health(is_dead);
CREATE INDEX idx_distractor_health_option ON distractor_health(option_id);
