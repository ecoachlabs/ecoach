CREATE TABLE IF NOT EXISTS past_paper_sets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    exam_year INTEGER NOT NULL,
    paper_code TEXT,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS past_paper_question_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    paper_id INTEGER NOT NULL REFERENCES past_paper_sets(id) ON DELETE CASCADE,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    section_label TEXT,
    question_number TEXT
);

CREATE TABLE IF NOT EXISTS question_family_analytics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_id INTEGER NOT NULL REFERENCES question_families(id) ON DELETE CASCADE,
    recurrence_score INTEGER NOT NULL DEFAULT 0,
    coappearance_score INTEGER NOT NULL DEFAULT 0,
    replacement_score INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
