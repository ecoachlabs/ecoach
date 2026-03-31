-- idea16: Library Intelligence Hub — state history, auto-shelf rules,
-- revision pack templates, cross-entity relationships, item actions,
-- exam hotspot tracking, library search dimensions.

-- ============================================================================
-- 1. Library item state history (track transitions)
-- ============================================================================

CREATE TABLE library_item_state_history (
    id INTEGER PRIMARY KEY,
    library_item_id INTEGER NOT NULL REFERENCES library_items(id),
    from_state TEXT,
    to_state TEXT NOT NULL,
    reason TEXT,
    changed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_library_state_history_item ON library_item_state_history(library_item_id);

-- ============================================================================
-- 2. Library item actions (track user interactions)
-- ============================================================================

CREATE TABLE library_item_actions (
    id INTEGER PRIMARY KEY,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    library_item_id INTEGER NOT NULL REFERENCES library_items(id),
    action_type TEXT NOT NULL
        CHECK (action_type IN (
            'opened', 'studied', 'tested', 'marked_weak', 'marked_mastered',
            'marked_exam_critical', 'marked_confusing', 'added_note',
            'asked_explanation', 'asked_worked_solution', 'added_to_pack',
            'shared', 'downloaded_offline', 'tested_self', 'turned_flashcard'
        )),
    context_json TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_library_actions_student ON library_item_actions(student_id);
CREATE INDEX idx_library_actions_item ON library_item_actions(library_item_id);

-- ============================================================================
-- 3. Library items expansion (richer metadata)
-- ============================================================================

ALTER TABLE library_items ADD COLUMN subject_id INTEGER;
ALTER TABLE library_items ADD COLUMN subtopic_id INTEGER;
ALTER TABLE library_items ADD COLUMN difficulty_bp INTEGER;
ALTER TABLE library_items ADD COLUMN exam_frequency_bp INTEGER;
ALTER TABLE library_items ADD COLUMN source TEXT;
ALTER TABLE library_items ADD COLUMN last_opened_at TEXT;
ALTER TABLE library_items ADD COLUMN open_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE library_items ADD COLUMN study_count INTEGER NOT NULL DEFAULT 0;

-- ============================================================================
-- 4. Shelf generation rules (configurable auto-shelf logic)
-- ============================================================================

CREATE TABLE shelf_generation_rules (
    id INTEGER PRIMARY KEY,
    shelf_type TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    generation_query TEXT NOT NULL,
    sort_field TEXT NOT NULL DEFAULT 'urgency_score',
    sort_direction TEXT NOT NULL DEFAULT 'DESC',
    max_items INTEGER NOT NULL DEFAULT 20,
    refresh_interval_minutes INTEGER NOT NULL DEFAULT 60,
    is_active INTEGER NOT NULL DEFAULT 1,
    priority_order INTEGER NOT NULL DEFAULT 10,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO shelf_generation_rules (shelf_type, display_name, description, generation_query, sort_field, max_items, priority_order) VALUES
    ('due_now', 'Due Now', 'Items needing immediate attention', 'memory_queue_items WHERE bucket IN (''due_today'',''emergency_recall'',''fading_now'')', 'priority_bp', 15, 1),
    ('memory_shelf', 'Memory Shelf', 'Concepts fading from memory', 'memory_queue_items WHERE bucket = ''fading_now''', 'decay_risk_bp', 10, 2),
    ('mistake_bank', 'Mistake Bank', 'Recent mistakes organized by pattern', 'wrong_answer_diagnoses WHERE created_at > datetime(''now'',''-7 days'')', 'created_at', 20, 3),
    ('saved_questions', 'Saved Questions', 'Questions you saved for later', 'library_items WHERE item_type = ''question'' AND state != ''mastered''', 'urgency_score', 30, 4),
    ('exam_hotspots', 'Exam Hotspots', 'Frequently tested, not yet mastered', 'family_recurrence_metrics WHERE recurrence_rate_bp > 6000', 'recurrence_rate_bp', 10, 5),
    ('near_wins', 'Almost There', 'Topics close to mastery', 'near_win_opportunities WHERE gap_bp < 2000', 'gap_bp', 8, 6),
    ('untouched_important', 'Untouched but Important', 'High-value items never reviewed', 'library_items WHERE open_count = 0 AND exam_frequency_bp > 5000', 'exam_frequency_bp', 10, 7),
    ('continue_learning', 'Continue Where You Stopped', 'Resume your last learning path', 'sessions WHERE status = ''paused'' OR status = ''active''', 'last_activity_at', 3, 8),
    ('weak_concepts', 'My Weak Concepts', 'Concepts needing most work', 'student_topic_states WHERE mastery_score < 4000', 'mastery_score', 15, 9),
    ('things_i_forget', 'Things I Keep Forgetting', 'Topics with repeated recall failures', 'memory_states WHERE memory_state IN (''fading'',''collapsed'',''at_risk'')', 'decay_risk', 10, 10);

-- ============================================================================
-- 5. Revision pack templates (auto-generation rules)
-- ============================================================================

CREATE TABLE revision_pack_templates (
    id INTEGER PRIMARY KEY,
    template_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    description TEXT,
    pack_type TEXT NOT NULL
        CHECK (pack_type IN (
            'weak_area', 'mock_exam_prep', 'formula_rescue',
            'last_minute', 'likely_exam', 'things_i_fail',
            'memory_rescue', 'confidence_builder', 'custom'
        )),
    selection_strategy TEXT NOT NULL DEFAULT 'weakness_weighted',
    default_item_count INTEGER NOT NULL DEFAULT 15,
    difficulty_profile TEXT NOT NULL DEFAULT 'balanced',
    topic_scope TEXT NOT NULL DEFAULT 'weak_topics',
    include_explanations INTEGER NOT NULL DEFAULT 1,
    include_worked_examples INTEGER NOT NULL DEFAULT 1,
    time_estimate_minutes INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO revision_pack_templates (template_code, display_name, description, pack_type, selection_strategy, default_item_count, difficulty_profile, topic_scope, time_estimate_minutes) VALUES
    ('weak_area', 'Weak Area Pack', 'Questions from your weakest topics', 'weak_area', 'weakness_weighted', 15, 'balanced', 'weak_topics', 30),
    ('mock_prep', 'Mock Exam Prep', 'Balanced exam-style preparation', 'mock_exam_prep', 'exam_balanced', 30, 'harder', 'all_topics', 60),
    ('formula_rescue', 'Formula Rescue', 'Formulas you keep forgetting', 'formula_rescue', 'decay_priority', 10, 'easier', 'formula_topics', 15),
    ('last_minute', 'Last-Minute Rescue', 'Highest-yield quick review', 'last_minute', 'score_impact', 10, 'balanced', 'high_yield', 15),
    ('likely_exam', 'Likely Exam Questions', 'Based on exam pattern intelligence', 'likely_exam', 'recurrence_weighted', 20, 'balanced', 'high_frequency', 40),
    ('things_i_fail', 'Things I Keep Failing', 'Your recurring mistakes as practice', 'things_i_fail', 'mistake_recurrence', 10, 'balanced', 'mistake_topics', 20),
    ('memory_rescue', 'Memory Rescue', 'Topics slipping from memory', 'memory_rescue', 'decay_priority', 12, 'easier', 'fading_topics', 25),
    ('confidence_build', 'Confidence Builder', 'Start easy, build up confidence', 'confidence_builder', 'confidence_ramp', 15, 'easier', 'near_mastery', 25);

-- ============================================================================
-- 6. Revision packs expansion
-- ============================================================================

ALTER TABLE revision_packs ADD COLUMN template_id INTEGER REFERENCES revision_pack_templates(id);
ALTER TABLE revision_packs ADD COLUMN pack_type TEXT;
ALTER TABLE revision_packs ADD COLUMN subject_id INTEGER;
ALTER TABLE revision_packs ADD COLUMN question_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE revision_packs ADD COLUMN estimated_minutes INTEGER;
ALTER TABLE revision_packs ADD COLUMN difficulty_profile TEXT;
ALTER TABLE revision_packs ADD COLUMN completion_rate_bp INTEGER NOT NULL DEFAULT 0;
ALTER TABLE revision_packs ADD COLUMN accuracy_bp INTEGER;
ALTER TABLE revision_packs ADD COLUMN status TEXT NOT NULL DEFAULT 'ready'
    CHECK (status IN ('generating', 'ready', 'in_progress', 'completed', 'expired'));

-- ============================================================================
-- 7. Cross-entity relationship mappings
-- ============================================================================

CREATE TABLE content_relationships (
    id INTEGER PRIMARY KEY,
    source_type TEXT NOT NULL
        CHECK (source_type IN (
            'topic', 'subtopic', 'concept', 'skill', 'question',
            'formula', 'definition', 'worked_example', 'question_family'
        )),
    source_id INTEGER NOT NULL,
    target_type TEXT NOT NULL
        CHECK (target_type IN (
            'topic', 'subtopic', 'concept', 'skill', 'question',
            'formula', 'definition', 'worked_example', 'question_family'
        )),
    target_id INTEGER NOT NULL,
    relationship_type TEXT NOT NULL
        CHECK (relationship_type IN (
            'prerequisite', 'builds_on', 'contrasts_with',
            'commonly_paired', 'exam_family', 'misconception_trap',
            'uses', 'explains', 'demonstrates', 'tests'
        )),
    strength_bp INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source_type, source_id, target_type, target_id, relationship_type)
);

CREATE INDEX idx_content_rels_source ON content_relationships(source_type, source_id);
CREATE INDEX idx_content_rels_target ON content_relationships(target_type, target_id);

-- ============================================================================
-- 8. Library tag taxonomy
-- ============================================================================

CREATE TABLE library_tag_definitions (
    id INTEGER PRIMARY KEY,
    tag_code TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    category TEXT NOT NULL
        CHECK (category IN ('state', 'priority', 'exam', 'learning', 'custom')),
    description TEXT,
    color_hint TEXT,
    is_system INTEGER NOT NULL DEFAULT 1
);

INSERT INTO library_tag_definitions (tag_code, display_name, category, description) VALUES
    ('exam_critical', 'Exam Critical', 'exam', 'High probability of appearing in exam'),
    ('keep_forgetting', 'Keep Forgetting', 'state', 'Topic with repeated recall failures'),
    ('careless_error', 'Careless Error Zone', 'learning', 'Area prone to careless mistakes'),
    ('needs_reteach', 'Needs Re-teaching', 'learning', 'Concept not understood properly'),
    ('nearly_mastered', 'Nearly Mastered', 'state', 'Close to full mastery'),
    ('high_yield', 'High Yield', 'exam', 'Worth many marks in exam'),
    ('confusing', 'Confusing', 'learning', 'Easy to mix up with similar concepts'),
    ('favourite', 'Favourite', 'custom', 'Student-marked as favourite'),
    ('revision_done', 'Revised', 'state', 'Completed revision cycle'),
    ('needs_practice', 'Needs Practice', 'learning', 'Theory understood but practice needed');

-- ============================================================================
-- 9. Library shelves expansion
-- ============================================================================

ALTER TABLE library_shelves ADD COLUMN description TEXT;
ALTER TABLE library_shelves ADD COLUMN icon_hint TEXT;
ALTER TABLE library_shelves ADD COLUMN item_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE library_shelves ADD COLUMN priority_order INTEGER NOT NULL DEFAULT 10;
ALTER TABLE library_shelves ADD COLUMN last_refreshed_at TEXT;
ALTER TABLE library_shelves ADD COLUMN rule_id INTEGER REFERENCES shelf_generation_rules(id);
