-- Beat Yesterday deep gaps: state machine, missing scores, badges, weekly reviews,
-- error classification, streak logic, question bucket tracking.

-- ============================================================================
-- 1. Extend beat_yesterday_profiles with missing scores and state
-- ============================================================================

ALTER TABLE beat_yesterday_profiles ADD COLUMN climb_state TEXT NOT NULL DEFAULT 'entry';
ALTER TABLE beat_yesterday_profiles ADD COLUMN speed_readiness_score INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_profiles ADD COLUMN confidence_score INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE beat_yesterday_profiles ADD COLUMN recovery_need_score_v2 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_profiles ADD COLUMN momentum_trend TEXT NOT NULL DEFAULT 'steady';
ALTER TABLE beat_yesterday_profiles ADD COLUMN growth_quality TEXT NOT NULL DEFAULT 'unknown';

-- ============================================================================
-- 2. Extend beat_yesterday_daily_summaries with error classification + growth quality
-- ============================================================================

ALTER TABLE beat_yesterday_daily_summaries ADD COLUMN concept_gap_errors INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_daily_summaries ADD COLUMN careless_errors INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_daily_summaries ADD COLUMN time_pressure_errors INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_daily_summaries ADD COLUMN misread_errors INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_daily_summaries ADD COLUMN guessed_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_daily_summaries ADD COLUMN incomplete_reasoning_errors INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_daily_summaries ADD COLUMN repeated_misconception_errors INTEGER NOT NULL DEFAULT 0;
ALTER TABLE beat_yesterday_daily_summaries ADD COLUMN growth_quality TEXT NOT NULL DEFAULT 'unknown';

-- ============================================================================
-- 3. Badge system
-- ============================================================================

CREATE TABLE beat_yesterday_badge_definitions (
    id INTEGER PRIMARY KEY,
    badge_code TEXT NOT NULL UNIQUE,
    badge_name TEXT NOT NULL,
    description TEXT NOT NULL,
    condition_type TEXT NOT NULL,
    condition_threshold INTEGER NOT NULL DEFAULT 1,
    icon_name TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE beat_yesterday_earned_badges (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    badge_code TEXT NOT NULL,
    earned_at TEXT NOT NULL DEFAULT (datetime('now')),
    context_json TEXT NOT NULL DEFAULT '{}',
    UNIQUE(student_id, subject_id, badge_code)
);

CREATE INDEX idx_by_earned_badges_student ON beat_yesterday_earned_badges(student_id, subject_id);

-- Seed badge definitions
INSERT INTO beat_yesterday_badge_definitions (badge_code, badge_name, description, condition_type, condition_threshold)
VALUES
    ('first_beat', 'First Beat', 'Beat yesterday for the first time', 'beat_count', 1),
    ('three_day_climb', '3-Day Climb', 'Beat yesterday 3 days in a row', 'streak', 3),
    ('five_day_climb', '5-Day Climb', 'Beat yesterday 5 days in a row', 'streak', 5),
    ('accuracy_lift', 'Accuracy Lift', 'Improved accuracy 3 days in a row', 'accuracy_streak', 3),
    ('speed_breakthrough', 'Speed Breakthrough', 'Achieved a personal speed record', 'speed_record', 1),
    ('no_quit_week', 'No Quit Week', 'Completed every session for 7 days', 'completion_streak', 7),
    ('recovery_comeback', 'Recovery Comeback', 'Returned from recovery mode and improved', 'recovery_return', 1),
    ('balanced_growth', 'Balanced Growth', 'Beat all 3 targets in one day', 'triple_beat', 1),
    ('consistency_builder', 'Consistency Builder', 'Studied 10 days in 14', 'study_consistency', 10);

-- ============================================================================
-- 4. Weekly review summaries
-- ============================================================================

CREATE TABLE beat_yesterday_weekly_reviews (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    week_start TEXT NOT NULL,
    week_end TEXT NOT NULL,
    sessions_completed INTEGER NOT NULL DEFAULT 0,
    avg_attempts_per_day INTEGER NOT NULL DEFAULT 0,
    avg_correctness_bp INTEGER NOT NULL DEFAULT 0,
    avg_pace_ms INTEGER NOT NULL DEFAULT 0,
    biggest_win TEXT,
    biggest_challenge TEXT,
    consistency_streak INTEGER NOT NULL DEFAULT 0,
    momentum_trend TEXT NOT NULL DEFAULT 'steady',
    next_week_primary_focus TEXT,
    next_week_secondary_focus TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, week_start)
);

CREATE INDEX idx_by_weekly_reviews_student ON beat_yesterday_weekly_reviews(student_id, subject_id);
