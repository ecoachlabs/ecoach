-- idea16 completion: explicit library notes, stronger personal attachments,
-- and additional generated shelf coverage.

-- ============================================================================
-- 1. Library item attachments (goals / upcoming tests)
-- ============================================================================

ALTER TABLE library_items ADD COLUMN goal_id INTEGER REFERENCES goals(id);
ALTER TABLE library_items ADD COLUMN calendar_event_id INTEGER REFERENCES calendar_events(id);

CREATE INDEX idx_library_items_goal ON library_items(goal_id);
CREATE INDEX idx_library_items_calendar_event ON library_items(calendar_event_id);

-- ============================================================================
-- 2. Typed library notes
-- ============================================================================

CREATE TABLE library_notes (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    library_item_id INTEGER REFERENCES library_items(id) ON DELETE CASCADE,
    topic_id INTEGER REFERENCES topics(id),
    note_type TEXT NOT NULL
        CHECK (note_type IN (
            'general', 'own_explanation', 'shortcut', 'memory_hook',
            'teacher_tip', 'trap_warning', 'formula_hint',
            'confusion_note', 'wording_warning'
        )),
    title TEXT,
    note_text TEXT NOT NULL,
    context_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_library_notes_student ON library_notes(student_id, created_at DESC);
CREATE INDEX idx_library_notes_item ON library_notes(library_item_id);
CREATE INDEX idx_library_notes_topic ON library_notes(topic_id);

-- ============================================================================
-- 3. Additional generated shelf rules
-- ============================================================================

INSERT INTO shelf_generation_rules (
    shelf_type,
    display_name,
    description,
    generation_query,
    sort_field,
    max_items,
    priority_order
) VALUES
    (
        'teach_me_again',
        'Teach Me Again',
        'Topics that need another teaching pass before they become stable',
        'student_topic_states WHERE gap_score > 5000 OR decay_risk > 5000',
        'gap_score',
        8,
        11
    ),
    (
        'formula_bank',
        'Formula Bank',
        'High-value formulas and rules that should stay easy to retrieve',
        'academic_nodes WHERE node_type = ''formula''',
        'exam_relevance_score',
        10,
        12
    ),
    (
        'concept_chains',
        'Concept Chains',
        'Prerequisite or build-on chains that explain what unlocks a topic',
        'content_relationships WHERE relationship_type IN (''prerequisite'',''builds_on'')',
        'strength_bp',
        8,
        13
    );
