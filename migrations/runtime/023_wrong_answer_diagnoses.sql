CREATE TABLE IF NOT EXISTS wrong_answer_diagnoses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    session_id INTEGER REFERENCES sessions(id),
    misconception_id INTEGER REFERENCES misconception_patterns(id),
    error_type TEXT NOT NULL,
    primary_diagnosis TEXT NOT NULL,
    secondary_diagnosis TEXT,
    severity TEXT NOT NULL
        CHECK (severity IN ('low', 'medium', 'high')),
    diagnosis_summary TEXT NOT NULL,
    recommended_action TEXT NOT NULL,
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_wrong_answer_diagnoses_student ON wrong_answer_diagnoses(student_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_wrong_answer_diagnoses_topic ON wrong_answer_diagnoses(topic_id);
CREATE INDEX IF NOT EXISTS idx_wrong_answer_diagnoses_question ON wrong_answer_diagnoses(question_id);
