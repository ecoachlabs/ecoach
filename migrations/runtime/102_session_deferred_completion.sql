ALTER TABLE sessions ADD COLUMN deferred_completion_state TEXT NOT NULL DEFAULT 'idle'
    CHECK (deferred_completion_state IN ('idle', 'pending', 'failed', 'processed'));
ALTER TABLE sessions ADD COLUMN deferred_completion_updated_at TEXT;
ALTER TABLE sessions ADD COLUMN deferred_completion_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD COLUMN deferred_completion_last_error TEXT;

CREATE INDEX IF NOT EXISTS idx_sessions_deferred_completion_state
    ON sessions(deferred_completion_state);

CREATE INDEX IF NOT EXISTS idx_sessions_student_deferred_completion
    ON sessions(student_id, deferred_completion_state);
