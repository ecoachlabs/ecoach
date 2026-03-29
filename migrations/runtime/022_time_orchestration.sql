CREATE TABLE IF NOT EXISTS availability_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    timezone_name TEXT NOT NULL DEFAULT 'Africa/Accra',
    preferred_daily_minutes INTEGER NOT NULL DEFAULT 60,
    min_session_minutes INTEGER NOT NULL DEFAULT 15,
    max_session_minutes INTEGER NOT NULL DEFAULT 90,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

CREATE TABLE IF NOT EXISTS availability_windows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    weekday INTEGER NOT NULL CHECK (weekday BETWEEN 0 AND 6),
    start_minute INTEGER NOT NULL CHECK (start_minute BETWEEN 0 AND 1439),
    end_minute INTEGER NOT NULL CHECK (end_minute BETWEEN 1 AND 1440),
    is_preferred INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS availability_exceptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    exception_date TEXT NOT NULL,
    start_minute INTEGER,
    end_minute INTEGER,
    availability_mode TEXT NOT NULL DEFAULT 'blocked'
        CHECK (availability_mode IN ('blocked', 'reduced', 'extended')),
    minutes_delta INTEGER NOT NULL DEFAULT 0,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_availability_windows_student_day ON availability_windows(student_id, weekday);
CREATE INDEX IF NOT EXISTS idx_availability_exceptions_student_date ON availability_exceptions(student_id, exception_date);
