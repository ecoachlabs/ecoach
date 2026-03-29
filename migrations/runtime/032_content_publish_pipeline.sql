CREATE TABLE IF NOT EXISTS content_publish_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_upload_id INTEGER NOT NULL REFERENCES curriculum_source_uploads(id) ON DELETE CASCADE,
    content_pack_id INTEGER REFERENCES content_packs(id) ON DELETE SET NULL,
    requested_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE SET NULL,
    topic_id INTEGER REFERENCES topics(id) ON DELETE SET NULL,
    target_version_label TEXT,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'gating', 'review_required', 'ready_to_publish', 'publishing', 'published', 'failed')),
    decision_trace_json TEXT NOT NULL DEFAULT '{}',
    artifact_summary_json TEXT NOT NULL DEFAULT '{}',
    published_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_publish_jobs_source
    ON content_publish_jobs(source_upload_id, status, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_content_publish_jobs_subject
    ON content_publish_jobs(subject_id, topic_id, status);

CREATE TABLE IF NOT EXISTS content_quality_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    publish_job_id INTEGER NOT NULL REFERENCES content_publish_jobs(id) ON DELETE CASCADE,
    report_type TEXT NOT NULL,
    status TEXT NOT NULL
        CHECK (status IN ('pass', 'warning', 'fail', 'needs_review')),
    confidence_score INTEGER NOT NULL DEFAULT 0,
    metrics_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_quality_reports_job
    ON content_quality_reports(publish_job_id, status, created_at DESC);
