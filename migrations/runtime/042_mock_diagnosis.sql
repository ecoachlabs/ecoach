-- Deep post-mock diagnosis: weakness scores, broken links, misconception hits,
-- representation gaps, timing/confidence diagnosis, predicted exam score.

CREATE TABLE mock_deep_diagnoses (
    id INTEGER PRIMARY KEY,
    mock_session_id INTEGER NOT NULL REFERENCES mock_sessions(id),
    student_id INTEGER NOT NULL,
    subject_id INTEGER NOT NULL,
    predicted_exam_score INTEGER,
    predicted_exam_range_low INTEGER,
    predicted_exam_range_high INTEGER,
    weakness_scores_json TEXT NOT NULL DEFAULT '[]',
    broken_links_json TEXT NOT NULL DEFAULT '[]',
    misconception_hits_json TEXT NOT NULL DEFAULT '[]',
    representation_gaps_json TEXT NOT NULL DEFAULT '[]',
    timing_diagnosis_json TEXT NOT NULL DEFAULT '{}',
    confidence_diagnosis_json TEXT NOT NULL DEFAULT '{}',
    action_plan_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_mock_deep_diagnoses_session ON mock_deep_diagnoses(mock_session_id);
CREATE INDEX idx_mock_deep_diagnoses_student ON mock_deep_diagnoses(student_id);
