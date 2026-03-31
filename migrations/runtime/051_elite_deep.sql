-- Elite Mode deep gaps: personal bests, badges, independence score.

CREATE TABLE elite_personal_bests (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    record_type TEXT NOT NULL,
    record_value INTEGER NOT NULL DEFAULT 0,
    record_context_json TEXT NOT NULL DEFAULT '{}',
    achieved_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, record_type)
);

CREATE INDEX idx_elite_personal_bests_student ON elite_personal_bests(student_id, subject_id);

CREATE TABLE elite_badges (
    id INTEGER PRIMARY KEY,
    badge_code TEXT NOT NULL UNIQUE,
    badge_name TEXT NOT NULL,
    description TEXT NOT NULL,
    tier_required TEXT,
    icon_name TEXT
);

CREATE TABLE elite_earned_badges (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL,
    badge_code TEXT NOT NULL,
    earned_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id, badge_code)
);

CREATE INDEX idx_elite_earned_badges_student ON elite_earned_badges(student_id, subject_id);

-- Seed elite badges
INSERT INTO elite_badges (badge_code, badge_name, description) VALUES
    ('precision_beast', 'Precision Beast', 'Maintained 90%+ precision for 5 sessions'),
    ('trap_hunter', 'Trap Hunter', 'Avoided all traps in 3 consecutive sessions'),
    ('speed_authority', 'Speed Authority', 'Achieved a personal speed record'),
    ('perfect_run', 'Perfect Run', 'Completed a full session with zero errors'),
    ('distinction_machine', 'Distinction Machine', 'Reached Apex tier'),
    ('no_hint_master', 'No-Hint Master', 'Completed 5 sessions without hints'),
    ('examiner_proof', 'Examiner-Proof', 'Scored 85%+ on trap-heavy and precision sessions'),
    ('endurance_king', 'Endurance King', 'Completed an endurance track with stable performance'),
    ('legend_status', 'Legend Status', 'Reached Legend tier');

-- Add independence score to elite profiles
ALTER TABLE elite_profiles ADD COLUMN independence_score INTEGER NOT NULL DEFAULT 5000;
ALTER TABLE elite_profiles ADD COLUMN endurance_score INTEGER NOT NULL DEFAULT 5000;
