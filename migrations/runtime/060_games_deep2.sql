-- idea10 deep gaps: missing control fields, board state, per-block cycle tracking,
-- rank progression, skill tracks, tug of war pull mechanics, mode unlocks.

-- ============================================================================
-- 1. Missing control permission fields on game_answer_events
-- ============================================================================

ALTER TABLE game_answer_events ADD COLUMN soft_drop_unlocked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN hard_drop_unlocked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN hold_unlocked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN rescue_active INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_answer_events ADD COLUMN answer_speed_band TEXT;
ALTER TABLE game_answer_events ADD COLUMN question_type_tag TEXT;
ALTER TABLE game_answer_events ADD COLUMN placement_quality_bp INTEGER;

-- ============================================================================
-- 2. Extended game_sessions with missing progress fields
-- ============================================================================

ALTER TABLE game_sessions ADD COLUMN current_wave INTEGER NOT NULL DEFAULT 1;
ALTER TABLE game_sessions ADD COLUMN blocks_spawned INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN blocks_locked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN current_combo INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN control_streak INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN timer_band TEXT NOT NULL DEFAULT 'normal';
ALTER TABLE game_sessions ADD COLUMN grid_state_json TEXT;
ALTER TABLE game_sessions ADD COLUMN active_block_json TEXT;
ALTER TABLE game_sessions ADD COLUMN next_blocks_json TEXT;
ALTER TABLE game_sessions ADD COLUMN hold_block_json TEXT;
ALTER TABLE game_sessions ADD COLUMN hole_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE game_sessions ADD COLUMN board_stability_index INTEGER NOT NULL DEFAULT 10000;
ALTER TABLE game_sessions ADD COLUMN topic_performance_map_json TEXT;
ALTER TABLE game_sessions ADD COLUMN fragile_mastery_flags_json TEXT;

-- ============================================================================
-- 3. Per-block question cycle tracking
-- ============================================================================

CREATE TABLE mindstack_block_cycles (
    id INTEGER PRIMARY KEY,
    game_session_id INTEGER NOT NULL REFERENCES game_sessions(id),
    cycle_number INTEGER NOT NULL,
    block_shape TEXT NOT NULL,
    morphed_to TEXT,
    question_id INTEGER,
    question_presented_at TEXT,
    answer_deadline_at TEXT,
    answer_submitted_at TEXT,
    answer_quality TEXT,
    cycle_state TEXT NOT NULL DEFAULT 'spawn_pending'
        CHECK (cycle_state IN (
            'spawn_pending', 'spawned_locked', 'answer_window_open',
            'answer_resolved', 'permissions_applied', 'placement_window',
            'block_locked', 'board_resolved', 'cycle_complete'
        )),
    control_permissions_json TEXT NOT NULL DEFAULT '{}',
    placement_quality_bp INTEGER,
    lines_cleared_this_cycle INTEGER NOT NULL DEFAULT 0,
    danger_level_at_lock INTEGER NOT NULL DEFAULT 0,
    was_boss_round INTEGER NOT NULL DEFAULT 0,
    power_up_used TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_block_cycles_session ON mindstack_block_cycles(game_session_id);

-- ============================================================================
-- 4. Rank progression
-- ============================================================================

CREATE TABLE mindstack_ranks (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    current_rank TEXT NOT NULL DEFAULT 'trainee'
        CHECK (current_rank IN (
            'trainee', 'controller', 'stack_tactician',
            'pressure_reader', 'morph_adept', 'mindstack_master'
        )),
    total_runs INTEGER NOT NULL DEFAULT 0,
    total_lines_cleared INTEGER NOT NULL DEFAULT 0,
    best_score INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    best_survival_seconds INTEGER NOT NULL DEFAULT 0,
    pressure_accuracy_best_bp INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

CREATE INDEX idx_mindstack_ranks_student ON mindstack_ranks(student_id);

-- ============================================================================
-- 5. MindStack badges
-- ============================================================================

CREATE TABLE mindstack_badges (
    id INTEGER PRIMARY KEY,
    badge_code TEXT NOT NULL UNIQUE,
    badge_name TEXT NOT NULL,
    description TEXT NOT NULL,
    condition_type TEXT NOT NULL,
    condition_threshold INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE mindstack_earned_badges (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    badge_code TEXT NOT NULL,
    earned_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, badge_code)
);

INSERT INTO mindstack_badges (badge_code, badge_name, description, condition_type, condition_threshold)
VALUES
    ('fraction_grip', 'Fraction Grip', 'Dominated fractions under pressure', 'topic_mastery', 1),
    ('grammar_control', 'Grammar Control', 'Stable grammar accuracy in MindStack', 'topic_mastery', 1),
    ('science_stabilizer', 'Science Stabilizer', 'Consistent science answers under falling blocks', 'topic_mastery', 1),
    ('morph_adept_badge', 'Morph Adept', 'Used 20 morphs effectively', 'morph_count', 20),
    ('pressure_reader_badge', 'Pressure Reader', 'Maintained 80%+ accuracy at high danger', 'pressure_accuracy', 8000),
    ('comeback_king', 'Comeback King', 'Recovered from 3+ panic events in one run', 'panic_recovery', 3),
    ('perfect_stack', 'Perfect Stack', 'Cleared 10 lines without any holes', 'perfect_lines', 10),
    ('streak_flame', 'Streak Flame', 'Achieved a 10+ correct answer streak', 'streak', 10);

-- ============================================================================
-- 6. MindStack skill tracks
-- ============================================================================

CREATE TABLE mindstack_skill_tracks (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    pressure_accuracy_bp INTEGER NOT NULL DEFAULT 5000,
    response_speed_bp INTEGER NOT NULL DEFAULT 5000,
    recovery_strength_bp INTEGER NOT NULL DEFAULT 5000,
    control_efficiency_bp INTEGER NOT NULL DEFAULT 5000,
    panic_resilience_bp INTEGER NOT NULL DEFAULT 5000,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

-- ============================================================================
-- 7. TugOfWar pull mechanics
-- ============================================================================

CREATE TABLE tug_of_war_pull_events (
    id INTEGER PRIMARY KEY,
    game_session_id INTEGER NOT NULL REFERENCES game_sessions(id),
    round_number INTEGER NOT NULL,
    question_id INTEGER,
    question_difficulty TEXT,
    pull_strength INTEGER NOT NULL DEFAULT 100,
    pull_direction TEXT NOT NULL DEFAULT 'player',
    was_correct INTEGER NOT NULL DEFAULT 0,
    streak_at_pull INTEGER NOT NULL DEFAULT 0,
    rope_position_after INTEGER NOT NULL DEFAULT 0,
    zone_after TEXT,
    was_power_pull INTEGER NOT NULL DEFAULT 0,
    was_boss_round INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_tug_pull_events_session ON tug_of_war_pull_events(game_session_id);

-- ============================================================================
-- 8. Control permission matrix (configurable)
-- ============================================================================

CREATE TABLE mindstack_control_matrix (
    id INTEGER PRIMARY KEY,
    difficulty_level TEXT NOT NULL,
    answer_quality TEXT NOT NULL,
    movement_allowed INTEGER NOT NULL DEFAULT 0,
    rotation_allowed INTEGER NOT NULL DEFAULT 0,
    soft_drop_allowed INTEGER NOT NULL DEFAULT 0,
    hard_drop_allowed INTEGER NOT NULL DEFAULT 0,
    morph_allowed INTEGER NOT NULL DEFAULT 0,
    hold_allowed INTEGER NOT NULL DEFAULT 0,
    slowdown_allowed INTEGER NOT NULL DEFAULT 0,
    rescue_allowed INTEGER NOT NULL DEFAULT 0,
    gravity_modifier REAL NOT NULL DEFAULT 1.0,
    score_multiplier REAL NOT NULL DEFAULT 1.0
);

-- Seed beginner matrix
INSERT INTO mindstack_control_matrix (difficulty_level, answer_quality, movement_allowed, rotation_allowed, soft_drop_allowed, hard_drop_allowed, morph_allowed, hold_allowed, slowdown_allowed, rescue_allowed, gravity_modifier, score_multiplier)
VALUES
    ('calm', 'wrong', 1, 0, 0, 0, 0, 0, 0, 0, 1.2, 0.5),
    ('calm', 'correct', 1, 1, 1, 0, 0, 0, 0, 0, 1.0, 1.0),
    ('calm', 'fast_correct', 1, 1, 1, 1, 0, 0, 1, 0, 0.8, 1.5),
    ('calm', 'streak_correct', 1, 1, 1, 1, 1, 0, 1, 0, 0.7, 2.0),
    ('focused', 'wrong', 0, 0, 0, 0, 0, 0, 0, 0, 1.5, 0.3),
    ('focused', 'correct', 1, 1, 1, 0, 0, 0, 0, 0, 1.0, 1.0),
    ('focused', 'fast_correct', 1, 1, 1, 1, 0, 0, 1, 0, 0.9, 1.3),
    ('focused', 'streak_correct', 1, 1, 1, 1, 1, 1, 1, 0, 0.8, 1.8),
    ('intense', 'wrong', 0, 0, 0, 0, 0, 0, 0, 0, 1.8, 0.2),
    ('intense', 'correct', 1, 1, 0, 0, 0, 0, 0, 0, 1.0, 1.0),
    ('intense', 'fast_correct', 1, 1, 1, 1, 0, 0, 1, 0, 0.9, 1.5),
    ('intense', 'streak_correct', 1, 1, 1, 1, 1, 1, 1, 1, 0.7, 2.5);

-- ============================================================================
-- 9. Mode unlock tracking
-- ============================================================================

CREATE TABLE mindstack_mode_unlocks (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    mode_type TEXT NOT NULL,
    is_unlocked INTEGER NOT NULL DEFAULT 0,
    unlocked_at TEXT,
    unlock_reason TEXT,
    UNIQUE(student_id, mode_type)
);

-- Default: classic_control always unlocked
INSERT INTO mindstack_mode_unlocks (student_id, mode_type, is_unlocked, unlock_reason)
SELECT id, 'classic_control', 1, 'default' FROM accounts WHERE account_type = 'student';
