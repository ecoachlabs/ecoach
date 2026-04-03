-- idea22 follow-up: make family analytics safely upsertable.

CREATE UNIQUE INDEX IF NOT EXISTS idx_question_family_analytics_family_unique
    ON question_family_analytics(family_id);
