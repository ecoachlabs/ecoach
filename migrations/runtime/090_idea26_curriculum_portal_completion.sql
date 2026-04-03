-- idea26: curriculum portal completion - canonical curriculum graph,
-- public snapshots, search aliases, resource links, and version governance.

-- ============================================================================
-- 1. Curriculum families and version governance
-- ============================================================================

CREATE TABLE IF NOT EXISTS curriculum_families (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    country_code TEXT NOT NULL DEFAULT 'GH',
    exam_board TEXT,
    education_stage TEXT,
    description TEXT,
    is_public INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

ALTER TABLE curriculum_versions ADD COLUMN curriculum_family_id INTEGER REFERENCES curriculum_families(id);
ALTER TABLE curriculum_versions ADD COLUMN effective_to TEXT;
ALTER TABLE curriculum_versions ADD COLUMN source_summary_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE curriculum_versions ADD COLUMN replaced_by_version_id INTEGER REFERENCES curriculum_versions(id);

CREATE INDEX IF NOT EXISTS idx_curriculum_versions_family
ON curriculum_versions(curriculum_family_id, status, published_at);

INSERT INTO curriculum_families (
    slug, name, country_code, exam_board, education_stage, description, is_public
)
SELECT DISTINCT
    lower(
        replace(
            replace(
                replace(
                    trim(COALESCE(version.name, 'curriculum-' || version.id)),
                    ' ', '-'
                ),
                '/', '-'
            ),
            '--', '-'
        )
    ) || '-' || version.id,
    COALESCE(version.name, 'Curriculum ' || version.id),
    COALESCE(version.country, 'GH'),
    version.exam_board,
    version.education_stage,
    'Auto-created curriculum family for version backfill.',
    CASE WHEN version.status = 'published' THEN 1 ELSE 0 END
FROM curriculum_versions version
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_families family
    WHERE family.slug = lower(
        replace(
            replace(
                replace(
                    trim(COALESCE(version.name, 'curriculum-' || version.id)),
                    ' ', '-'
                ),
                '/', '-'
            ),
            '--', '-'
        )
    ) || '-' || version.id
);

UPDATE curriculum_versions
SET curriculum_family_id = (
    SELECT family.id
    FROM curriculum_families family
    WHERE family.slug = lower(
        replace(
            replace(
                replace(
                    trim(COALESCE(curriculum_versions.name, 'curriculum-' || curriculum_versions.id)),
                    ' ', '-'
                ),
                '/', '-'
            ),
            '--', '-'
        )
    ) || '-' || curriculum_versions.id
)
WHERE curriculum_family_id IS NULL;

-- ============================================================================
-- 2. Canonical curriculum portal graph
-- ============================================================================

CREATE TABLE IF NOT EXISTS curriculum_subject_tracks (
    id INTEGER PRIMARY KEY,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id) ON DELETE CASCADE,
    legacy_subject_id INTEGER UNIQUE REFERENCES subjects(id),
    subject_code TEXT NOT NULL,
    subject_name TEXT NOT NULL,
    subject_slug TEXT NOT NULL,
    public_title TEXT NOT NULL,
    description TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(curriculum_version_id, subject_slug)
);

CREATE TABLE IF NOT EXISTS curriculum_levels (
    id INTEGER PRIMARY KEY,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id) ON DELETE CASCADE,
    level_code TEXT NOT NULL,
    level_name TEXT NOT NULL,
    stage_order INTEGER NOT NULL DEFAULT 0,
    public_title TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(curriculum_version_id, level_code)
);

CREATE TABLE IF NOT EXISTS curriculum_term_periods (
    id INTEGER PRIMARY KEY,
    level_id INTEGER NOT NULL REFERENCES curriculum_levels(id) ON DELETE CASCADE,
    term_code TEXT NOT NULL,
    term_name TEXT NOT NULL,
    sequence_no INTEGER NOT NULL DEFAULT 1,
    public_term TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(level_id, term_code)
);

CREATE TABLE IF NOT EXISTS curriculum_nodes (
    id INTEGER PRIMARY KEY,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id) ON DELETE CASCADE,
    subject_track_id INTEGER NOT NULL REFERENCES curriculum_subject_tracks(id) ON DELETE CASCADE,
    level_id INTEGER REFERENCES curriculum_levels(id),
    term_id INTEGER REFERENCES curriculum_term_periods(id),
    parent_node_id INTEGER REFERENCES curriculum_nodes(id) ON DELETE CASCADE,
    legacy_topic_id INTEGER UNIQUE REFERENCES topics(id),
    node_type TEXT NOT NULL
        CHECK (node_type IN (
            'strand', 'sub_strand', 'unit', 'topic', 'subtopic',
            'objective_group', 'concept_cluster', 'assessment_area'
        )),
    canonical_title TEXT NOT NULL,
    public_title TEXT NOT NULL,
    slug TEXT NOT NULL,
    official_text TEXT,
    public_summary TEXT,
    sequence_no INTEGER NOT NULL DEFAULT 0,
    depth INTEGER NOT NULL DEFAULT 0,
    estimated_weight INTEGER NOT NULL DEFAULT 5000,
    exam_relevance_score INTEGER NOT NULL DEFAULT 5000,
    difficulty_hint TEXT NOT NULL DEFAULT 'medium',
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'approved', 'published', 'archived')),
    review_status TEXT NOT NULL DEFAULT 'pending_review'
        CHECK (review_status IN ('pending_review', 'approved', 'rejected', 'superseded')),
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(curriculum_version_id, slug)
);

CREATE TABLE IF NOT EXISTS curriculum_node_objectives (
    id INTEGER PRIMARY KEY,
    curriculum_node_id INTEGER NOT NULL REFERENCES curriculum_nodes(id) ON DELETE CASCADE,
    legacy_learning_objective_id INTEGER UNIQUE REFERENCES learning_objectives(id),
    objective_text TEXT NOT NULL,
    simplified_text TEXT,
    cognitive_level TEXT,
    objective_type TEXT NOT NULL DEFAULT 'understanding'
        CHECK (objective_type IN (
            'knowledge', 'understanding', 'application',
            'reasoning', 'practical', 'communication'
        )),
    sequence_no INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    review_status TEXT NOT NULL DEFAULT 'pending_review'
        CHECK (review_status IN ('pending_review', 'approved', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS curriculum_concept_atoms (
    id INTEGER PRIMARY KEY,
    curriculum_node_id INTEGER NOT NULL REFERENCES curriculum_nodes(id) ON DELETE CASCADE,
    legacy_academic_node_id INTEGER UNIQUE REFERENCES academic_nodes(id),
    concept_type TEXT NOT NULL
        CHECK (concept_type IN (
            'concept', 'formula', 'keyword', 'competency', 'misconception', 'assessment_pattern'
        )),
    canonical_term TEXT NOT NULL,
    public_term TEXT,
    description TEXT,
    alias_group_id TEXT,
    review_status TEXT NOT NULL DEFAULT 'pending_review'
        CHECK (review_status IN ('pending_review', 'approved', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS curriculum_aliases (
    id INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL
        CHECK (entity_type IN ('family', 'subject_track', 'node', 'objective', 'concept_atom')),
    entity_id INTEGER NOT NULL,
    alias_text TEXT NOT NULL,
    alias_kind TEXT NOT NULL DEFAULT 'synonym'
        CHECK (alias_kind IN (
            'synonym', 'exam_phrase', 'teacher_phrase', 'student_phrase',
            'abbreviation', 'alternate_spelling'
        )),
    locale TEXT NOT NULL DEFAULT 'en',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entity_type, entity_id, alias_text, locale)
);

CREATE TABLE IF NOT EXISTS curriculum_relationships (
    id INTEGER PRIMARY KEY,
    from_entity_type TEXT NOT NULL
        CHECK (from_entity_type IN ('node', 'objective', 'concept_atom')),
    from_entity_id INTEGER NOT NULL,
    to_entity_type TEXT NOT NULL
        CHECK (to_entity_type IN ('node', 'objective', 'concept_atom')),
    to_entity_id INTEGER NOT NULL,
    relationship_type TEXT NOT NULL
        CHECK (relationship_type IN (
            'prerequisite', 'related', 'depends_on', 'assessed_by',
            'clarifies', 'confused_with', 'part_of', 'adjacent_to'
        )),
    strength_score INTEGER NOT NULL DEFAULT 5000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(from_entity_type, from_entity_id, to_entity_type, to_entity_id, relationship_type)
);

CREATE TABLE IF NOT EXISTS curriculum_resource_links (
    id INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL
        CHECK (entity_type IN ('node', 'objective', 'concept_atom')),
    entity_id INTEGER NOT NULL,
    resource_type TEXT NOT NULL
        CHECK (resource_type IN (
            'question', 'test', 'mission', 'lesson', 'note',
            'glossary', 'drill', 'video', 'game', 'past_exam_cluster'
        )),
    resource_id INTEGER NOT NULL,
    link_strength INTEGER NOT NULL DEFAULT 5000,
    source TEXT NOT NULL DEFAULT 'system',
    review_status TEXT NOT NULL DEFAULT 'approved'
        CHECK (review_status IN ('pending_review', 'approved', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entity_type, entity_id, resource_type, resource_id)
);

CREATE INDEX IF NOT EXISTS idx_curriculum_subject_tracks_version
ON curriculum_subject_tracks(curriculum_version_id, display_order, subject_name);
CREATE INDEX IF NOT EXISTS idx_curriculum_levels_version
ON curriculum_levels(curriculum_version_id, stage_order);
CREATE INDEX IF NOT EXISTS idx_curriculum_term_periods_level
ON curriculum_term_periods(level_id, sequence_no);
CREATE INDEX IF NOT EXISTS idx_curriculum_nodes_subject
ON curriculum_nodes(subject_track_id, sequence_no, public_title);
CREATE INDEX IF NOT EXISTS idx_curriculum_nodes_parent
ON curriculum_nodes(parent_node_id, sequence_no);
CREATE INDEX IF NOT EXISTS idx_curriculum_nodes_legacy_topic
ON curriculum_nodes(legacy_topic_id);
CREATE INDEX IF NOT EXISTS idx_curriculum_node_objectives_node
ON curriculum_node_objectives(curriculum_node_id, sequence_no);
CREATE INDEX IF NOT EXISTS idx_curriculum_concept_atoms_node
ON curriculum_concept_atoms(curriculum_node_id, concept_type);
CREATE INDEX IF NOT EXISTS idx_curriculum_aliases_lookup
ON curriculum_aliases(alias_text, entity_type);
CREATE INDEX IF NOT EXISTS idx_curriculum_relationships_from
ON curriculum_relationships(from_entity_type, from_entity_id, relationship_type);
CREATE INDEX IF NOT EXISTS idx_curriculum_relationships_to
ON curriculum_relationships(to_entity_type, to_entity_id, relationship_type);
CREATE INDEX IF NOT EXISTS idx_curriculum_resource_links_entity
ON curriculum_resource_links(entity_type, entity_id, resource_type);

-- ============================================================================
-- 3. Publication and snapshot governance
-- ============================================================================

CREATE TABLE IF NOT EXISTS curriculum_public_snapshots (
    id INTEGER PRIMARY KEY,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id) ON DELETE CASCADE,
    snapshot_kind TEXT NOT NULL DEFAULT 'portal'
        CHECK (snapshot_kind IN ('portal', 'brain_context', 'coverage')),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'live', 'archived', 'superseded')),
    snapshot_json TEXT NOT NULL DEFAULT '{}',
    generated_by_account_id INTEGER REFERENCES accounts(id),
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
    published_at TEXT
);

CREATE TABLE IF NOT EXISTS curriculum_publish_logs (
    id INTEGER PRIMARY KEY,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id) ON DELETE CASCADE,
    snapshot_id INTEGER REFERENCES curriculum_public_snapshots(id),
    action_type TEXT NOT NULL
        CHECK (action_type IN ('snapshot_generated', 'published', 'archived')),
    actor_account_id INTEGER REFERENCES accounts(id),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_curriculum_public_snapshots_version
ON curriculum_public_snapshots(curriculum_version_id, status, generated_at DESC);
CREATE INDEX IF NOT EXISTS idx_curriculum_publish_logs_version
ON curriculum_publish_logs(curriculum_version_id, created_at DESC);

-- ============================================================================
-- 4. Backfill canonical graph from existing curriculum records
-- ============================================================================

INSERT INTO curriculum_subject_tracks (
    curriculum_version_id, legacy_subject_id, subject_code, subject_name, subject_slug,
    public_title, description, display_order
)
SELECT
    subject.curriculum_version_id,
    subject.id,
    subject.code,
    subject.name,
    lower(
        replace(
            replace(
                replace(trim(COALESCE(subject.code, subject.name)), ' ', '-'),
                '/', '-'
            ),
            '--', '-'
        )
    ),
    subject.name,
    NULL,
    subject.display_order
FROM subjects subject
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_subject_tracks track
    WHERE track.legacy_subject_id = subject.id
);

INSERT INTO curriculum_levels (
    curriculum_version_id, level_code, level_name, stage_order, public_title
)
SELECT
    version.id,
    lower(
        replace(
            replace(
                trim(COALESCE(NULLIF(version.education_stage, ''), 'core')),
                ' ', '_'
            ),
            '/', '_'
        )
    ),
    COALESCE(NULLIF(version.education_stage, ''), 'Core'),
    1,
    COALESCE(NULLIF(version.education_stage, ''), 'Core')
FROM curriculum_versions version
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_levels level
    WHERE level.curriculum_version_id = version.id
);

INSERT INTO curriculum_term_periods (
    level_id, term_code, term_name, sequence_no, public_term
)
SELECT
    level.id,
    'all',
    'All Year',
    1,
    'All Year'
FROM curriculum_levels level
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_term_periods term
    WHERE term.level_id = level.id
);

INSERT INTO curriculum_nodes (
    curriculum_version_id, subject_track_id, level_id, term_id, parent_node_id, legacy_topic_id,
    node_type, canonical_title, public_title, slug, official_text, public_summary,
    sequence_no, depth, estimated_weight, exam_relevance_score, difficulty_hint,
    status, review_status, confidence_score
)
SELECT
    version.id,
    track.id,
    level.id,
    term.id,
    NULL,
    topic.id,
    CASE
        WHEN topic.node_type IN ('strand', 'sub_strand', 'topic', 'subtopic') THEN topic.node_type
        ELSE 'topic'
    END,
    topic.name,
    topic.name,
    lower(
        replace(
            replace(
                replace(trim(COALESCE(NULLIF(topic.code, ''), topic.name)), ' ', '-'),
                '/', '-'
            ),
            '--', '-'
        )
    ),
    topic.description,
    topic.description,
    topic.display_order,
    CASE WHEN topic.parent_topic_id IS NULL THEN 0 ELSE 1 END,
    topic.importance_weight,
    topic.exam_weight,
    COALESCE(NULLIF(topic.difficulty_band, ''), 'medium'),
    CASE WHEN version.status = 'published' THEN 'published' ELSE 'draft' END,
    CASE WHEN version.status = 'published' THEN 'approved' ELSE 'pending_review' END,
    7200
FROM topics topic
JOIN subjects subject
  ON subject.id = topic.subject_id
JOIN curriculum_versions version
  ON version.id = subject.curriculum_version_id
JOIN curriculum_subject_tracks track
  ON track.legacy_subject_id = subject.id
JOIN curriculum_levels level
  ON level.curriculum_version_id = version.id
JOIN curriculum_term_periods term
  ON term.level_id = level.id
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_nodes node
    WHERE node.legacy_topic_id = topic.id
);

UPDATE curriculum_nodes
SET parent_node_id = (
    SELECT parent_node.id
    FROM topics legacy_topic
    JOIN curriculum_nodes parent_node
      ON parent_node.legacy_topic_id = legacy_topic.parent_topic_id
    WHERE legacy_topic.id = curriculum_nodes.legacy_topic_id
)
WHERE legacy_topic_id IS NOT NULL
  AND parent_node_id IS NULL;

INSERT INTO curriculum_node_objectives (
    curriculum_node_id, legacy_learning_objective_id, objective_text, simplified_text,
    cognitive_level, objective_type, sequence_no, confidence_score, review_status
)
SELECT
    node.id,
    objective.id,
    objective.objective_text,
    objective.simplified_text,
    objective.cognitive_level,
    CASE
        WHEN objective.cognitive_level IN ('knowledge', 'understanding', 'application', 'reasoning') THEN objective.cognitive_level
        ELSE 'understanding'
    END,
    objective.display_order,
    7000,
    CASE WHEN version.status = 'published' THEN 'approved' ELSE 'pending_review' END
FROM learning_objectives objective
JOIN curriculum_nodes node
  ON node.legacy_topic_id = objective.topic_id
JOIN curriculum_versions version
  ON version.id = node.curriculum_version_id
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_node_objectives portal_objective
    WHERE portal_objective.legacy_learning_objective_id = objective.id
);

INSERT INTO curriculum_concept_atoms (
    curriculum_node_id, legacy_academic_node_id, concept_type, canonical_term,
    public_term, description, alias_group_id, review_status
)
SELECT
    node.id,
    legacy_node.id,
    CASE
        WHEN legacy_node.node_type = 'formula' THEN 'formula'
        WHEN legacy_node.node_type IN ('definition', 'concept', 'comparison', 'principle', 'rule', 'theorem', 'application', 'vocabulary') THEN 'concept'
        ELSE 'competency'
    END,
    legacy_node.canonical_title,
    COALESCE(legacy_node.short_label, legacy_node.canonical_title),
    COALESCE(NULLIF(legacy_node.description_simple, ''), legacy_node.core_meaning),
    NULL,
    CASE WHEN version.status = 'published' THEN 'approved' ELSE 'pending_review' END
FROM academic_nodes legacy_node
JOIN curriculum_nodes node
  ON node.legacy_topic_id = legacy_node.topic_id
JOIN curriculum_versions version
  ON version.id = node.curriculum_version_id
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_concept_atoms atom
    WHERE atom.legacy_academic_node_id = legacy_node.id
);

INSERT INTO curriculum_aliases (
    entity_type, entity_id, alias_text, alias_kind, locale
)
SELECT
    'node',
    node.id,
    topic.code,
    'abbreviation',
    'en'
FROM curriculum_nodes node
JOIN topics topic
  ON topic.id = node.legacy_topic_id
WHERE topic.code IS NOT NULL
  AND trim(topic.code) <> ''
  AND NOT EXISTS (
      SELECT 1
      FROM curriculum_aliases alias
      WHERE alias.entity_type = 'node'
        AND alias.entity_id = node.id
        AND alias.alias_text = topic.code
        AND alias.locale = 'en'
  );

INSERT INTO curriculum_relationships (
    from_entity_type, from_entity_id, to_entity_type, to_entity_id, relationship_type, strength_score
)
SELECT
    'node',
    child_node.id,
    'node',
    parent_node.id,
    'part_of',
    8000
FROM topics topic
JOIN curriculum_nodes child_node
  ON child_node.legacy_topic_id = topic.id
JOIN curriculum_nodes parent_node
  ON parent_node.legacy_topic_id = topic.parent_topic_id
WHERE topic.parent_topic_id IS NOT NULL
  AND NOT EXISTS (
      SELECT 1
      FROM curriculum_relationships relationship
      WHERE relationship.from_entity_type = 'node'
        AND relationship.from_entity_id = child_node.id
        AND relationship.to_entity_type = 'node'
        AND relationship.to_entity_id = parent_node.id
        AND relationship.relationship_type = 'part_of'
  );

INSERT INTO curriculum_relationships (
    from_entity_type, from_entity_id, to_entity_type, to_entity_id, relationship_type, strength_score
)
SELECT
    'node',
    from_node.id,
    'node',
    to_node.id,
    CASE
        WHEN edge.edge_type IN ('prerequisite', 'related', 'confused_with', 'dependent', 'part_of') THEN edge.edge_type
        WHEN edge.edge_type = 'soft_prerequisite' THEN 'depends_on'
        ELSE 'related'
    END,
    edge.strength_score
FROM node_edges edge
JOIN curriculum_nodes from_node
  ON edge.from_node_type = 'topic'
 AND from_node.legacy_topic_id = edge.from_node_id
JOIN curriculum_nodes to_node
  ON edge.to_node_type = 'topic'
 AND to_node.legacy_topic_id = edge.to_node_id
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_relationships relationship
    WHERE relationship.from_entity_type = 'node'
      AND relationship.from_entity_id = from_node.id
      AND relationship.to_entity_type = 'node'
      AND relationship.to_entity_id = to_node.id
      AND relationship.relationship_type = CASE
            WHEN edge.edge_type IN ('prerequisite', 'related', 'confused_with', 'dependent', 'part_of') THEN edge.edge_type
            WHEN edge.edge_type = 'soft_prerequisite' THEN 'depends_on'
            ELSE 'related'
        END
);

INSERT INTO curriculum_resource_links (
    entity_type, entity_id, resource_type, resource_id, link_strength, source, review_status
)
SELECT
    'node',
    node.id,
    'question',
    question.id,
    7600,
    'legacy_topic_sync',
    'approved'
FROM questions question
JOIN curriculum_nodes node
  ON node.legacy_topic_id = question.topic_id
WHERE NOT EXISTS (
    SELECT 1
    FROM curriculum_resource_links link
    WHERE link.entity_type = 'node'
      AND link.entity_id = node.id
      AND link.resource_type = 'question'
      AND link.resource_id = question.id
);
