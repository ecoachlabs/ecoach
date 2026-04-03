CREATE TABLE IF NOT EXISTS account_entitlement_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    previous_tier TEXT NOT NULL
        CHECK (previous_tier IN ('standard', 'premium', 'elite')),
    new_tier TEXT NOT NULL
        CHECK (new_tier IN ('standard', 'premium', 'elite')),
    changed_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    note TEXT,
    previous_status TEXT NOT NULL DEFAULT 'active'
        CHECK (previous_status IN ('active', 'inactive', 'archived')),
    new_status TEXT NOT NULL DEFAULT 'active'
        CHECK (new_status IN ('active', 'inactive', 'archived')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_account_entitlement_events_account
    ON account_entitlement_events(account_id, created_at DESC);

CREATE TABLE IF NOT EXISTS bundle_shared_promotions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bundle_id INTEGER NOT NULL REFERENCES submission_bundles(id) ON DELETE CASCADE,
    source_upload_id INTEGER REFERENCES curriculum_source_uploads(id) ON DELETE SET NULL,
    requested_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    promotion_status TEXT NOT NULL DEFAULT 'queued'
        CHECK (promotion_status IN ('queued', 'approved', 'rejected', 'published')),
    promotion_summary_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_bundle_shared_promotions_bundle
    ON bundle_shared_promotions(bundle_id);
CREATE INDEX IF NOT EXISTS idx_bundle_shared_promotions_status
    ON bundle_shared_promotions(promotion_status, created_at DESC);

CREATE TABLE IF NOT EXISTS content_source_governance_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_upload_id INTEGER NOT NULL REFERENCES curriculum_source_uploads(id) ON DELETE CASCADE,
    source_status TEXT NOT NULL
        CHECK (source_status IN (
            'uploaded', 'parsed', 'review_required', 'reviewed',
            'published', 'archived', 'failed'
        )),
    decided_by_account_id INTEGER REFERENCES accounts(id) ON DELETE SET NULL,
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_content_source_governance_events_source
    ON content_source_governance_events(source_upload_id, created_at DESC);

CREATE TABLE IF NOT EXISTS areal_profiles (
    student_id INTEGER PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    tone_style TEXT NOT NULL DEFAULT 'steady',
    motivation_style TEXT NOT NULL DEFAULT 'coach',
    urgency_style TEXT NOT NULL DEFAULT 'adaptive',
    explanation_style TEXT NOT NULL DEFAULT 'step_by_step',
    narrative_enabled INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS areal_guidance_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    coach_state TEXT NOT NULL,
    current_mode TEXT NOT NULL,
    headline TEXT NOT NULL,
    summary TEXT NOT NULL,
    route_key TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_areal_guidance_events_student
    ON areal_guidance_events(student_id, created_at DESC);
