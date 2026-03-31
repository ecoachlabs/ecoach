-- Rise Mode: Last-to-First transformation engine.
-- 4-stage journey for weakest students: Rescue → Stabilize → Accelerate → Dominate.

CREATE TABLE rise_mode_profiles (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    current_stage TEXT NOT NULL DEFAULT 'rescue'
        CHECK (current_stage IN ('rescue', 'stabilize', 'accelerate', 'dominate', 'completed')),
    foundation_score INTEGER NOT NULL DEFAULT 0,
    recall_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    accuracy_score INTEGER NOT NULL DEFAULT 0,
    pressure_stability_score INTEGER NOT NULL DEFAULT 0,
    misconception_density_score INTEGER NOT NULL DEFAULT 0,
    momentum_score INTEGER NOT NULL DEFAULT 0,
    transformation_readiness_score INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    weakness_map_json TEXT NOT NULL DEFAULT '{}',
    recovery_plan_json TEXT NOT NULL DEFAULT '{}',
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    stage_entered_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE INDEX idx_rise_mode_student ON rise_mode_profiles(student_id, subject_id);
CREATE INDEX idx_rise_mode_stage ON rise_mode_profiles(current_stage);
