ALTER TABLE diagnostic_session_phases
    ADD COLUMN phase_code TEXT;

ALTER TABLE diagnostic_session_phases
    ADD COLUMN phase_title TEXT;

ALTER TABLE diagnostic_session_phases
    ADD COLUMN condition_profile_json TEXT NOT NULL DEFAULT '{}';

ALTER TABLE diagnostic_session_phases
    ADD COLUMN time_limit_seconds INTEGER;
