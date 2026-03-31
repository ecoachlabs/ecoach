-- idea10: MindStack deep features — control permissions, morph system,
-- sub-modes, board state, run state, boss rounds, power-ups, TugOfWar zones.

-- ============================================================================
-- 1. Extend game_sessions with sub-mode and enriched state
-- ============================================================================

ALTER TABLE game_sessions ADD COLUMN sub_mode TEXT;
ALTER TABLE game_sessions ADD COLUMN difficulty_level TEXT NOT NULL DEFAULT 'focused';
ALTER TABLE game_sessions ADD COLUMN gravity_level INTEGER NOT NULL DEFAULT 1;
ALTER TABLE game_sessions ADD COLUMN mercy_mode_active INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN current_pressure_level INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN morphs_earned INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN morphs_used INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN panic_events INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN rescue_events INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN boss_rounds_cleared INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN pressure_accuracy_bp INTEGER NOT NULL DEFAULT 0;

-- ============================================================================
-- 2. Extend game_answer_events with control permissions and block context
-- ============================================================================

ALTER TABLE game_answer_events ADD COLUMN answer_quality TEXT;
ALTER TABLE game_answer_events ADD COLUMN movement_unlocked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN rotation_unlocked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN morph_unlocked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN slowdown_activated INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN block_shape TEXT;
ALTER TABLE game_answer_events ADD COLUMN morphed_to_shape TEXT;
ALTER TABLE game_answer_events ADD COLUMN board_danger_level INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN is_boss_round INTEGER NOT NULL DEFAULT 0;

-- ============================================================================
-- 3. MindStack power-ups
-- ============================================================================

CREATE TABLE mindstack_power_ups (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    power_up_type TEXT NOT NULL
        CHECK (power_up_type IN (
            'slow_time', 'shape_shift', 'smart_rotate', 'safe_hold',
            'retry_answer', 'misconception_shield', 'row_repair', 'stability_boost'
        )),
    uses_remaining INTEGER NOT NULL DEFAULT 1,
    earned_from TEXT,
    earned_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_mindstack_powerups_student ON mindstack_power_ups(student_id);

-- ============================================================================
-- 4. TugOfWar extended state
-- ============================================================================

ALTER TABLE game_sessions ADD COLUMN tug_zone TEXT;
ALTER TABLE game_sessions ADD COLUMN tug_pull_power INTEGER NOT NULL DEFAULT 100;
ALTER TABLE game_sessions ADD COLUMN tug_opponent_name TEXT;
ALTER TABLE game_sessions ADD COLUMN tug_misconception_target_id INTEGER;

-- ============================================================================
-- 5. MindStack run analytics (per-run learning insights)
-- ============================================================================

CREATE TABLE mindstack_run_analytics (
    id INTEGER PRIMARY KEY,
    game_session_id INTEGER NOT NULL REFERENCES game_sessions(id),
    student_id INTEGER NOT NULL,
    subject_id INTEGER NOT NULL,
    pressure_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    calm_accuracy_bp INTEGER NOT NULL DEFAULT 0,
    avg_answer_speed_ms INTEGER,
    timeout_count INTEGER NOT NULL DEFAULT 0,
    misconception_hit_count INTEGER NOT NULL DEFAULT 0,
    recovery_success_rate_bp INTEGER NOT NULL DEFAULT 0,
    collapse_cause TEXT,
    strongest_topic TEXT,
    weakest_topic TEXT,
    most_common_error_pattern TEXT,
    morph_efficiency_bp INTEGER NOT NULL DEFAULT 0,
    board_efficiency_bp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_mindstack_analytics_session ON mindstack_run_analytics(game_session_id);
CREATE INDEX idx_mindstack_analytics_student ON mindstack_run_analytics(student_id);
