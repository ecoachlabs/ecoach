CREATE TABLE IF NOT EXISTS question_graph_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    to_question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL
        CHECK (relation_type IN (
            'same_skill',
            'same_family',
            'representation_shift',
            'isomorphic_cluster',
            'difficulty_ladder',
            'misconception_pair',
            'contrast_pair',
            'near_duplicate'
        )),
    similarity_score INTEGER NOT NULL DEFAULT 0,
    rationale TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(from_question_id, to_question_id, relation_type)
);

CREATE TABLE IF NOT EXISTS reactor_candidate_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id INTEGER NOT NULL REFERENCES question_generation_requests(id) ON DELETE CASCADE,
    source_question_id INTEGER NOT NULL REFERENCES questions(id),
    candidate_stem TEXT NOT NULL,
    candidate_fingerprint TEXT NOT NULL,
    matched_question_id INTEGER REFERENCES questions(id),
    similarity_score INTEGER NOT NULL DEFAULT 0,
    novelty_score INTEGER NOT NULL DEFAULT 0,
    anti_repeat_penalty INTEGER NOT NULL DEFAULT 0,
    decision_score INTEGER NOT NULL DEFAULT 0,
    decision TEXT NOT NULL
        CHECK (decision IN ('accepted', 'rejected_near_duplicate', 'rejected_exact_duplicate')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_question_graph_edges_from ON question_graph_edges(from_question_id);
CREATE INDEX IF NOT EXISTS idx_question_graph_edges_to ON question_graph_edges(to_question_id);
CREATE INDEX IF NOT EXISTS idx_question_graph_edges_relation ON question_graph_edges(relation_type);
CREATE INDEX IF NOT EXISTS idx_reactor_candidate_scores_request ON reactor_candidate_scores(request_id);
CREATE INDEX IF NOT EXISTS idx_reactor_candidate_scores_match ON reactor_candidate_scores(matched_question_id);
