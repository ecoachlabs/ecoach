CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_type TEXT NOT NULL CHECK (account_type IN ('student', 'parent', 'admin')),
    display_name TEXT NOT NULL,
    avatar_path TEXT,
    pin_hash TEXT NOT NULL,
    pin_salt TEXT NOT NULL,
    entitlement_tier TEXT NOT NULL DEFAULT 'standard'
        CHECK (entitlement_tier IN ('standard', 'premium', 'elite')),
    failed_pin_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'archived')),
    first_run INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_active_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS student_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    grade_level TEXT,
    curriculum_track TEXT,
    exam_target TEXT,
    exam_target_date TEXT,
    age_band TEXT,
    preferred_subjects TEXT NOT NULL DEFAULT '[]',
    study_days_per_week INTEGER DEFAULT 5,
    daily_study_budget_minutes INTEGER DEFAULT 60,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE TABLE IF NOT EXISTS parent_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    display_preference TEXT DEFAULT 'standard',
    simplified_mode INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE TABLE IF NOT EXISTS parent_student_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    relationship_label TEXT DEFAULT 'parent',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(parent_account_id, student_account_id)
);

CREATE TABLE IF NOT EXISTS admin_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE INDEX IF NOT EXISTS idx_accounts_type ON accounts(account_type);
CREATE INDEX IF NOT EXISTS idx_student_profiles_account ON student_profiles(account_id);
CREATE INDEX IF NOT EXISTS idx_parent_links_parent ON parent_student_links(parent_account_id);
CREATE INDEX IF NOT EXISTS idx_parent_links_student ON parent_student_links(student_account_id);
