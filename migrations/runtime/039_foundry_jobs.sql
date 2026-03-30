CREATE TABLE IF NOT EXISTS foundry_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_type TEXT NOT NULL
        CHECK (job_type IN (
            'source_review_job', 'duplicate_resolution_job', 'source_approval_job',
            'curriculum_enrichment_job', 'misconception_build_job',
            'question_generation_job', 'note_build_job', 'formula_pack_build_job',
            'worked_example_build_job', 'source_acquisition_job',
            'contrast_build_job', 'publish_job', 'publish_activation_job',
            'quality_review_job'
        )),
    trigger_type TEXT NOT NULL,
    target_type TEXT NOT NULL
        CHECK (target_type IN ('source_upload', 'topic_package', 'subject_package', 'publish_job')),
    target_id INTEGER NOT NULL,
    subject_id INTEGER REFERENCES subjects(id) ON DELETE SET NULL,
    topic_id INTEGER REFERENCES topics(id) ON DELETE SET NULL,
    priority INTEGER NOT NULL DEFAULT 5000,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'blocked', 'completed', 'failed', 'cancelled')),
    dependency_refs_json TEXT NOT NULL DEFAULT '[]',
    payload_json TEXT NOT NULL DEFAULT '{}',
    result_summary_json TEXT NOT NULL DEFAULT '{}',
    retry_count INTEGER NOT NULL DEFAULT 0,
    failure_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_foundry_jobs_status
    ON foundry_jobs(status, priority DESC, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_foundry_jobs_target
    ON foundry_jobs(target_type, target_id, status);

CREATE INDEX IF NOT EXISTS idx_foundry_jobs_subject_topic
    ON foundry_jobs(subject_id, topic_id, status);
