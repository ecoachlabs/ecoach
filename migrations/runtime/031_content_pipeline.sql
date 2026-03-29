CREATE TABLE IF NOT EXISTS curriculum_source_uploads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uploader_account_id INTEGER NOT NULL REFERENCES accounts(id),
    source_kind TEXT NOT NULL
        CHECK (source_kind IN (
            'curriculum', 'syllabus', 'guide', 'worksheet',
            'past_question', 'textbook', 'web_source'
        )),
    title TEXT NOT NULL,
    source_path TEXT,
    country_code TEXT,
    exam_board TEXT,
    education_level TEXT,
    subject_code TEXT,
    academic_year TEXT,
    language_code TEXT DEFAULT 'en',
    version_label TEXT,
    source_status TEXT NOT NULL DEFAULT 'uploaded'
        CHECK (source_status IN (
            'uploaded', 'parsed', 'review_required', 'reviewed',
            'published', 'archived', 'failed'
        )),
    confidence_score INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS curriculum_parse_candidates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_upload_id INTEGER NOT NULL REFERENCES curriculum_source_uploads(id) ON DELETE CASCADE,
    candidate_type TEXT NOT NULL
        CHECK (candidate_type IN (
            'subject', 'topic', 'subtopic', 'objective', 'skill',
            'keyword', 'formula', 'concept', 'dependency'
        )),
    parent_candidate_id INTEGER REFERENCES curriculum_parse_candidates(id),
    raw_label TEXT NOT NULL,
    normalized_label TEXT,
    payload_json TEXT NOT NULL DEFAULT '{}',
    confidence_score INTEGER NOT NULL DEFAULT 0,
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (review_status IN ('pending', 'approved', 'rejected', 'merged')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS curriculum_review_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_upload_id INTEGER NOT NULL REFERENCES curriculum_source_uploads(id) ON DELETE CASCADE,
    candidate_id INTEGER REFERENCES curriculum_parse_candidates(id) ON DELETE CASCADE,
    task_type TEXT NOT NULL
        CHECK (task_type IN (
            'normalization', 'duplicate_check', 'hierarchy_fix',
            'publish_gate', 'contradiction'
        )),
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_review', 'resolved')),
    severity TEXT NOT NULL DEFAULT 'medium'
        CHECK (severity IN ('low', 'medium', 'high')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE TABLE IF NOT EXISTS content_acquisition_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    intent_type TEXT NOT NULL
        CHECK (intent_type IN (
            'gap_fill', 'refresh', 'corroborate', 'example_hunt',
            'question_hunt', 'glossary_hunt'
        )),
    query_text TEXT NOT NULL,
    source_scope TEXT NOT NULL DEFAULT 'mixed'
        CHECK (source_scope IN ('internal', 'approved_web', 'mixed')),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'review_required', 'completed', 'failed')),
    result_summary_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS acquisition_evidence_candidates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER NOT NULL REFERENCES content_acquisition_jobs(id) ON DELETE CASCADE,
    source_label TEXT NOT NULL,
    source_url TEXT,
    source_kind TEXT NOT NULL
        CHECK (source_kind IN ('upload', 'web', 'internal', 'teacher_note')),
    title TEXT,
    snippet TEXT,
    extracted_payload_json TEXT NOT NULL DEFAULT '{}',
    quality_score INTEGER NOT NULL DEFAULT 0,
    freshness_score INTEGER NOT NULL DEFAULT 0,
    review_status TEXT NOT NULL DEFAULT 'staged'
        CHECK (review_status IN ('staged', 'approved', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_curriculum_source_uploads_status
    ON curriculum_source_uploads(source_status);
CREATE INDEX IF NOT EXISTS idx_curriculum_parse_candidates_source
    ON curriculum_parse_candidates(source_upload_id, candidate_type);
CREATE INDEX IF NOT EXISTS idx_curriculum_review_tasks_source
    ON curriculum_review_tasks(source_upload_id, status);
CREATE INDEX IF NOT EXISTS idx_content_acquisition_jobs_status
    ON content_acquisition_jobs(status, intent_type);
CREATE INDEX IF NOT EXISTS idx_content_acquisition_jobs_topic
    ON content_acquisition_jobs(topic_id);
CREATE INDEX IF NOT EXISTS idx_acquisition_candidates_job
    ON acquisition_evidence_candidates(job_id, review_status);
