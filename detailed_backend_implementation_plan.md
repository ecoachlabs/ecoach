# eCoach — Detailed Backend Implementation Plan
## Grounded in 38 Idea Files | Backend-Only | Rust + SQLite + Tauri

---

# TABLE OF CONTENTS

1. [Architectural Foundation](#1-architectural-foundation)
2. [Phase 0 — Foundations (Weeks 1-3)](#2-phase-0)
3. [Phase 1 — Core Primitives (Weeks 4-8)](#3-phase-1)
4. [Phase 2 — Essential Workflows (Weeks 9-14)](#4-phase-2)
5. [Phase 3 — Exam Simulation & Intelligence (Weeks 15-20)](#5-phase-3)
6. [Phase 4 — Hardening & Scale (Weeks 21-26)](#6-phase-4)
7. [Master Entity Reference](#7-master-entity-reference)
8. [Master State Machine Reference](#8-master-state-machine-reference)
9. [Master Scoring Formula Reference](#9-master-scoring-formula-reference)
10. [Master API Command Reference](#10-master-api-command-reference)

---

# 1. ARCHITECTURAL FOUNDATION

## 1.1 Technology Stack (Confirmed)

| Layer | Technology | Source |
|-------|-----------|--------|
| Backend language | Rust (stable) | idea1, idea35, idea38 |
| Desktop shell | Tauri v2 | idea1, idea35 |
| Frontend | Nuxt 3 (TypeScript) | idea1, idea35 |
| Database | SQLite 3 with WAL mode | idea1, idea2, idea6 |
| IPC | Tauri command boundary | idea1, idea35 |
| Score storage | Basis points as u16 (0-10000) | idea5 |
| Content delivery | Signed offline packs on local filesystem | idea1, idea2 |
| Background jobs | In-process Rust async (tokio) | Inferred from offline-first constraint |

## 1.2 Workspace Layout

```
ecoach-backend/
├── Cargo.toml                          # Workspace manifest
├── crates/
│   ├── ecoach-substrate/               # Shared types, enums, scoring wrappers, config, time utilities
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs                # BasisPoints, ConfidenceScore, SeverityLevel, etc.
│   │   │   ├── scoring.rs              # Score normalization, clamping, EMA helpers
│   │   │   ├── config.rs               # Threshold registry, tunable constants
│   │   │   ├── time.rs                 # Monotonic time, wall clock, duration helpers
│   │   │   ├── errors.rs               # Shared error types
│   │   │   └── events.rs               # DomainEvent trait, EventEnvelope
│   │   └── Cargo.toml
│   │
│   ├── ecoach-storage/                 # SQLite connection, migrations, repository traits
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── connection.rs           # Pool setup, WAL mode, pragmas
│   │   │   ├── migrations.rs           # Embedded migration runner
│   │   │   ├── repository.rs           # Generic CRUD trait
│   │   │   └── query.rs               # Query builder helpers
│   │   ├── migrations/
│   │   │   ├── 001_identity.sql
│   │   │   ├── 002_curriculum.sql
│   │   │   ├── 003_questions.sql
│   │   │   ├── 004_student_state.sql
│   │   │   ├── 005_sessions.sql
│   │   │   ├── 006_coach.sql
│   │   │   ├── 007_memory.sql
│   │   │   ├── 008_mock_centre.sql
│   │   │   ├── 009_knowledge_gap.sql
│   │   │   ├── 010_goals_calendar.sql
│   │   │   ├── 011_content_packs.sql
│   │   │   ├── 012_reporting.sql
│   │   │   ├── 013_glossary.sql
│   │   │   ├── 014_library.sql
│   │   │   ├── 015_games.sql
│   │   │   ├── 016_past_papers.sql
│   │   │   ├── 017_traps.sql
│   │   │   ├── 018_intake.sql
│   │   │   ├── 019_premium.sql
│   │   │   └── 020_elite.sql
│   │   └── Cargo.toml
│   │
│   ├── ecoach-identity/                # Users, PINs, roles, entitlements
│   ├── ecoach-curriculum/              # Academic truth graph
│   ├── ecoach-content/                 # Content OS, packs, artifacts, trust
│   ├── ecoach-questions/               # Question intelligence, families, generation
│   ├── ecoach-student-model/           # All student state
│   ├── ecoach-diagnostics/             # Diagnostic battery engine
│   ├── ecoach-coach-brain/             # State machine, planner, missions, interventions
│   ├── ecoach-memory/                  # Decay, recall, retention, interference
│   ├── ecoach-sessions/                # Session orchestrator, presence, time
│   ├── ecoach-mock-centre/             # Mock compilation, runtime, scoring
│   ├── ecoach-knowledge-gap/           # Gap scoring, repair, solidification
│   ├── ecoach-goals-calendar/          # Goals, exams, timeline, intensity
│   ├── ecoach-intake/                  # Document upload, OCR bridge, bundles
│   ├── ecoach-reporting/               # Readiness, memos, dashboards
│   ├── ecoach-library/                 # Content relationships, shelves
│   ├── ecoach-glossary/                # Knowledge entries, search, bundles
│   ├── ecoach-games/                   # MindStack, Tug of War, Traps
│   ├── ecoach-past-papers/             # Family mining, recurrence
│   ├── ecoach-premium/                 # Strategy engine, concierge
│   ├── ecoach-elite/                   # Elite mode, EPS scoring
│   └── ecoach-commands/                # Tauri command boundary
│       ├── src/
│       │   ├── lib.rs
│       │   ├── identity_commands.rs
│       │   ├── curriculum_commands.rs
│       │   ├── question_commands.rs
│       │   ├── student_commands.rs
│       │   ├── diagnostic_commands.rs
│       │   ├── coach_commands.rs
│       │   ├── session_commands.rs
│       │   ├── mock_commands.rs
│       │   ├── gap_commands.rs
│       │   ├── memory_commands.rs
│       │   ├── goals_commands.rs
│       │   ├── reporting_commands.rs
│       │   ├── library_commands.rs
│       │   ├── glossary_commands.rs
│       │   ├── game_commands.rs
│       │   ├── intake_commands.rs
│       │   └── admin_commands.rs
│       └── Cargo.toml
│
├── src-tauri/
│   ├── src/
│   │   └── main.rs                     # Tauri app entry, register all commands
│   ├── tauri.conf.json
│   └── Cargo.toml
│
└── tests/
    ├── fixtures/                       # Seed data for tests
    ├── integration/                    # Cross-crate integration tests
    └── benchmarks/                     # Performance benchmarks
```

## 1.3 Database Conventions

- Table naming: `snake_case`, singular noun where appropriate, prefixed by domain
- Primary keys: `id INTEGER PRIMARY KEY AUTOINCREMENT` (SQLite)
- UUIDs: stored as TEXT where cross-device sync is anticipated
- Timestamps: `created_at TEXT NOT NULL DEFAULT (datetime('now'))`, ISO 8601
- Scores: `INTEGER NOT NULL DEFAULT 0` (basis points 0-10000, representing 0.00-100.00)
- JSON columns: `TEXT NOT NULL DEFAULT '{}'` with application-level validation
- Foreign keys: `REFERENCES table(id) ON DELETE CASCADE` where appropriate
- Indexes: on all foreign keys, on frequently-queried columns

## 1.4 Event System Design

```rust
// Every domain event follows this shape
pub struct DomainEvent {
    pub event_id: String,           // UUID
    pub event_type: String,         // e.g., "answer.submitted", "mastery.updated"
    pub aggregate_id: String,       // e.g., student_id or session_id
    pub occurred_at: String,        // ISO 8601
    pub payload: serde_json::Value, // Event-specific data
    pub trace_id: String,           // For correlating cascading events
}
```

Events are written to an `event_log` table (append-only) and processed in-process by registered handlers. No external message broker.

---

# 2. PHASE 0 — FOUNDATIONS (Weeks 1-3)

## Week 1: Workspace, Database, Identity

### Task 0.1: Initialize Cargo workspace
- Create workspace `Cargo.toml` with all crate members
- Set up `ecoach-substrate` with shared types
- Set up `ecoach-storage` with SQLite connection pool (using `rusqlite` or `sqlx`)
- Configure WAL mode, foreign keys, journal size
- Implement embedded migration runner that applies `.sql` files in order
- **Test**: connection opens, WAL mode confirmed, migration applies cleanly

### Task 0.2: Substrate shared types
Define in `ecoach-substrate/src/types.rs`:

```rust
/// Score stored as basis points (0-10000 = 0.00% - 100.00%)
pub type BasisPoints = u16;

/// Convert float 0.0-1.0 to basis points
pub fn to_bp(score: f64) -> BasisPoints {
    (score.clamp(0.0, 1.0) * 10000.0).round() as u16
}

/// Convert basis points to float
pub fn from_bp(bp: BasisPoints) -> f64 {
    bp as f64 / 10000.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
pub enum Role {
    Student,
    Parent,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
pub enum EntitlementTier {
    Standard,
    Premium,
    Elite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityLevel {
    Low,      // 0-24
    Watch,    // 25-49
    Active,   // 50-69
    Urgent,   // 70-84
    Critical, // 85-100
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Critical,
}
```

### Task 0.3: Identity system (migration 001)

**Schema** (from idea3):

```sql
-- 001_identity.sql

CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_type TEXT NOT NULL CHECK (account_type IN ('student', 'parent', 'admin')),
    display_name TEXT NOT NULL,
    avatar_path TEXT,
    pin_hash TEXT NOT NULL,
    pin_salt TEXT NOT NULL,
    entitlement_tier TEXT NOT NULL DEFAULT 'standard'
        CHECK (entitlement_tier IN ('standard', 'premium', 'elite')),
    failed_pin_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,  -- ISO 8601 timestamp or NULL
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'archived')),
    first_run INTEGER NOT NULL DEFAULT 1,  -- boolean: needs onboarding
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_active_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE student_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    grade_level TEXT,
    curriculum_track TEXT,
    exam_target TEXT,        -- e.g., 'BECE 2026'
    exam_target_date TEXT,   -- ISO date
    age_band TEXT,
    preferred_subjects TEXT,  -- JSON array of subject IDs
    study_days_per_week INTEGER DEFAULT 5,
    daily_study_budget_minutes INTEGER DEFAULT 60,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE TABLE parent_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    display_preference TEXT DEFAULT 'standard',
    simplified_mode INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE TABLE parent_student_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    relationship_label TEXT DEFAULT 'parent',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(parent_account_id, student_account_id)
);

CREATE TABLE admin_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE INDEX idx_accounts_type ON accounts(account_type);
CREATE INDEX idx_student_profiles_account ON student_profiles(account_id);
CREATE INDEX idx_parent_links_parent ON parent_student_links(parent_account_id);
CREATE INDEX idx_parent_links_student ON parent_student_links(student_account_id);
```

**Identity service** (`ecoach-identity/src/`):

```rust
// services.rs
pub struct IdentityService { db: DbPool }

impl IdentityService {
    /// Create a new account with PIN
    pub async fn create_account(&self, input: CreateAccountInput) -> Result<Account, IdentityError>;

    /// Verify PIN, return account or increment failures
    pub async fn authenticate(&self, account_id: i64, pin: &str) -> Result<Account, IdentityError>;

    /// Switch active account (multi-user device)
    pub async fn switch_account(&self, account_id: i64) -> Result<Account, IdentityError>;

    /// Get all accounts for profile switcher screen
    pub async fn list_accounts(&self) -> Result<Vec<AccountSummary>, IdentityError>;

    /// Link parent to student
    pub async fn link_parent_student(&self, parent_id: i64, student_id: i64) -> Result<(), IdentityError>;

    /// Parent resets child PIN
    pub async fn reset_student_pin(&self, parent_id: i64, student_id: i64, new_pin: &str) -> Result<(), IdentityError>;

    /// Get children linked to parent
    pub async fn get_linked_students(&self, parent_id: i64) -> Result<Vec<AccountSummary>, IdentityError>;
}
```

**PIN rules** (from idea3):
- Student: 4-6 digits
- Parent: 6+ digits
- Admin: 6+ digits
- Lockout: 5 failed attempts → 5-minute cooldown
- Hash: argon2id with random salt
- **Test**: create account, authenticate, fail 5 times, verify lockout, wait, authenticate again

### Task 0.4: Tauri command boundary

```rust
// ecoach-commands/src/identity_commands.rs

#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    input: CreateAccountInput,
) -> Result<AccountDto, CommandError> {
    let account = state.identity_service.create_account(input).await?;
    Ok(AccountDto::from(account))
}

#[tauri::command]
pub async fn login_with_pin(
    state: State<'_, AppState>,
    account_id: i64,
    pin: String,
) -> Result<SessionDto, CommandError> {
    let account = state.identity_service.authenticate(account_id, &pin).await?;
    Ok(SessionDto::from(account))
}

#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountSummaryDto>, CommandError> {
    let accounts = state.identity_service.list_accounts().await?;
    Ok(accounts.into_iter().map(AccountSummaryDto::from).collect())
}
```

**DTO pattern** — never expose internal domain objects to frontend:
```rust
#[derive(Serialize)]
pub struct AccountSummaryDto {
    pub id: i64,
    pub display_name: String,
    pub account_type: String,
    pub avatar_path: Option<String>,
    pub status: String,
    pub needs_checkup: bool,       // first_run == 1
    pub last_active_label: String, // "Active today" / "Away 3 days" / etc.
}
```

## Week 2: Curriculum Graph

### Task 0.5: Curriculum schema (migration 002)

```sql
-- 002_curriculum.sql

CREATE TABLE curriculum_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,              -- e.g., "Ghana NaCCA JHS 2024"
    country TEXT NOT NULL DEFAULT 'GH',
    exam_board TEXT,                 -- e.g., 'WAEC'
    education_stage TEXT,            -- e.g., 'JHS'
    version_label TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'review', 'published', 'archived')),
    effective_from TEXT,
    published_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE subjects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    curriculum_version_id INTEGER NOT NULL REFERENCES curriculum_versions(id),
    code TEXT NOT NULL,              -- e.g., 'MATH', 'SCI', 'ENG', 'SOC'
    name TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE topics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    parent_topic_id INTEGER REFERENCES topics(id),  -- for strands/sub-strands
    code TEXT,
    name TEXT NOT NULL,
    description TEXT,
    node_type TEXT NOT NULL DEFAULT 'topic'
        CHECK (node_type IN ('strand', 'sub_strand', 'topic', 'subtopic')),
    display_order INTEGER NOT NULL DEFAULT 0,
    exam_weight INTEGER NOT NULL DEFAULT 5000,  -- basis points
    difficulty_band TEXT DEFAULT 'medium'
        CHECK (difficulty_band IN ('easy', 'medium', 'hard', 'advanced')),
    importance_weight INTEGER NOT NULL DEFAULT 5000,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE academic_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    node_type TEXT NOT NULL
        CHECK (node_type IN (
            'definition', 'concept', 'formula', 'procedure', 'comparison',
            'principle', 'rule', 'theorem', 'worked_pattern', 'application',
            'interpretation', 'diagram_spatial', 'proof_justification',
            'essay_structured', 'word_problem_translation', 'vocabulary',
            'symbol_notation'
        )),
    canonical_title TEXT NOT NULL,
    short_label TEXT,
    description_formal TEXT,
    description_simple TEXT,
    core_meaning TEXT,
    difficulty_band TEXT DEFAULT 'medium',
    exam_relevance_score INTEGER NOT NULL DEFAULT 5000,
    foundation_weight INTEGER NOT NULL DEFAULT 5000,
    is_active INTEGER NOT NULL DEFAULT 1,
    metadata_json TEXT NOT NULL DEFAULT '{}',  -- Type-specific extra fields
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE node_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_node_id INTEGER NOT NULL,     -- Can reference topics or academic_nodes
    from_node_type TEXT NOT NULL CHECK (from_node_type IN ('topic', 'academic_node')),
    to_node_id INTEGER NOT NULL,
    to_node_type TEXT NOT NULL CHECK (to_node_type IN ('topic', 'academic_node')),
    edge_type TEXT NOT NULL
        CHECK (edge_type IN (
            'prerequisite', 'soft_prerequisite', 'related', 'confused_with',
            'uses_formula', 'uses_procedure', 'has_example', 'has_non_example',
            'contrasts_with', 'is_applied_in', 'targets_misconception',
            'representation_of', 'dependent', 'part_of'
        )),
    strength_score INTEGER NOT NULL DEFAULT 5000,  -- basis points
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE misconception_patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id INTEGER REFERENCES academic_nodes(id),
    topic_id INTEGER REFERENCES topics(id),
    title TEXT NOT NULL,
    misconception_statement TEXT NOT NULL,
    cause_type TEXT
        CHECK (cause_type IN (
            'overgeneralization', 'memorization_without_understanding',
            'visual_confusion', 'language_confusion', 'step_confusion',
            'false_analogy', 'surface_similarity', 'incomplete_rule'
        )),
    wrong_answer_pattern TEXT,
    correction_hint TEXT,
    severity INTEGER NOT NULL DEFAULT 5000,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE learning_objectives (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    objective_text TEXT NOT NULL,
    simplified_text TEXT,
    cognitive_level TEXT
        CHECK (cognitive_level IN ('knowledge', 'understanding', 'application', 'reasoning', 'evaluation')),
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_topics_subject ON topics(subject_id);
CREATE INDEX idx_topics_parent ON topics(parent_topic_id);
CREATE INDEX idx_academic_nodes_topic ON academic_nodes(topic_id);
CREATE INDEX idx_academic_nodes_type ON academic_nodes(node_type);
CREATE INDEX idx_node_edges_from ON node_edges(from_node_id, from_node_type);
CREATE INDEX idx_node_edges_to ON node_edges(to_node_id, to_node_type);
CREATE INDEX idx_node_edges_type ON node_edges(edge_type);
CREATE INDEX idx_misconceptions_node ON misconception_patterns(node_id);
CREATE INDEX idx_misconceptions_topic ON misconception_patterns(topic_id);
```

### Task 0.6: Curriculum service

```rust
// ecoach-curriculum/src/services.rs

pub struct CurriculumService { db: DbPool }

impl CurriculumService {
    /// Get all subjects for a curriculum version
    pub async fn get_subjects(&self, curriculum_version_id: i64) -> Result<Vec<Subject>>;

    /// Get topic tree for a subject (hierarchical)
    pub async fn get_topic_tree(&self, subject_id: i64) -> Result<TopicTree>;

    /// Get full topic detail with nodes, prerequisites, misconceptions
    pub async fn get_topic_detail(&self, topic_id: i64) -> Result<TopicDetail>;

    /// Get prerequisites for a topic (transitive closure)
    pub async fn get_prerequisites(&self, topic_id: i64, depth: u32) -> Result<Vec<TopicSummary>>;

    /// Get topics confused with a given topic
    pub async fn get_confused_neighbors(&self, topic_id: i64) -> Result<Vec<TopicSummary>>;

    /// Search topics by name/keyword
    pub async fn search_topics(&self, query: &str, subject_id: Option<i64>) -> Result<Vec<TopicSummary>>;
}
```

## Week 3: Content Packs + First Sample Content

### Task 0.7: Content pack format

A content pack is a directory with this structure:
```
math-bece-2026-v1/
├── manifest.json           # Pack metadata, version, subject, checksums
├── curriculum/
│   ├── topics.json         # Topic definitions for this subject
│   ├── nodes.json          # Academic nodes
│   ├── edges.json          # Prerequisite/relationship edges
│   ├── misconceptions.json # Misconception patterns
│   └── objectives.json     # Learning objectives
├── questions/
│   ├── families.json       # Question family definitions
│   ├── questions.json      # Question instances with full metadata
│   └── intelligence.json   # 8-axis classification per question
├── content/
│   ├── explanations.json   # Explanation assets per node
│   ├── worked_examples.json
│   ├── glossary.json       # Knowledge entries
│   └── formulas.json       # Formula entries with LaTeX + plain + speech
├── assets/
│   ├── diagrams/           # Image files referenced by content
│   └── audio/              # Audio files for glossary/explanations
└── signature.json          # Pack signature for integrity verification
```

**manifest.json schema**:
```json
{
    "pack_id": "math-bece-2026-v1",
    "pack_version": "1.0.0",
    "subject_code": "MATH",
    "curriculum_version": "Ghana NaCCA JHS 2024",
    "exam_target": "BECE",
    "grade_levels": ["JHS1", "JHS2", "JHS3"],
    "topic_count": 42,
    "question_count": 850,
    "min_app_version": "0.1.0",
    "checksums": {
        "curriculum/topics.json": "sha256:abc123...",
        "questions/questions.json": "sha256:def456..."
    },
    "created_at": "2026-03-28T00:00:00Z",
    "author": "eCoach Content Team"
}
```

### Task 0.8: Pack installer service

```sql
-- 011_content_packs.sql

CREATE TABLE content_packs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pack_id TEXT NOT NULL UNIQUE,
    pack_version TEXT NOT NULL,
    subject_code TEXT NOT NULL,
    curriculum_version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'installing'
        CHECK (status IN ('installing', 'installed', 'active', 'failed', 'removed')),
    topic_count INTEGER NOT NULL DEFAULT 0,
    question_count INTEGER NOT NULL DEFAULT 0,
    install_path TEXT NOT NULL,
    manifest_json TEXT NOT NULL,
    installed_at TEXT NOT NULL DEFAULT (datetime('now')),
    activated_at TEXT,
    error_message TEXT
);
```

```rust
// ecoach-content/src/pack_service.rs

pub struct PackService { db: DbPool, pack_dir: PathBuf }

impl PackService {
    /// Install pack from directory path
    /// 1. Verify manifest + checksums
    /// 2. Parse all JSON files
    /// 3. Insert curriculum data (topics, nodes, edges, misconceptions)
    /// 4. Insert questions
    /// 5. Insert content (explanations, glossary, formulas)
    /// 6. Mark pack as active
    /// Rollback on any failure
    pub async fn install_pack(&self, pack_path: &Path) -> Result<PackInstallResult, PackError>;

    /// List installed packs
    pub async fn list_packs(&self) -> Result<Vec<PackSummary>>;

    /// Remove pack (mark as removed, optionally delete data)
    pub async fn remove_pack(&self, pack_id: &str) -> Result<()>;

    /// Rebuild content index after pack operations
    pub async fn rebuild_index(&self) -> Result<()>;
}
```

### Task 0.9: Create first sample content pack
- Mathematics for BECE
- 10 topics minimum: Number and Numeration, Fractions, Decimals, Percentages, Ratios, Algebra (basic), Geometry (shapes), Measurement (area, perimeter), Data Handling, Integers
- Each topic: 3-5 subtopics, 5-10 academic nodes, 3-5 misconceptions
- 20+ questions per topic (200+ total)
- Each question: stem, 4 options, correct answer, explanation, difficulty, topic/subtopic/skill tags, misconception tags on wrong options
- **This is the single most important non-code deliverable in Phase 0**

**Phase 0 Exit Criteria**:
- [ ] App launches, shows profile switcher
- [ ] Can create student/parent/admin accounts with PIN
- [ ] Can authenticate and switch between accounts
- [ ] Curriculum tree loads from installed pack
- [ ] Can browse subjects → topics → subtopics
- [ ] Sample Math pack installed with 200+ questions
- [ ] All tests pass

---

# 3. PHASE 1 — CORE PRIMITIVES (Weeks 4-8)

## Week 4: Question Entity + Student State Foundation

### Task 1.1: Question schema (migration 003)

```sql
-- 003_questions.sql

CREATE TABLE question_families (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_code TEXT NOT NULL,
    family_name TEXT NOT NULL,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    subtopic_id INTEGER REFERENCES topics(id),
    family_type TEXT NOT NULL DEFAULT 'recurring_pattern'
        CHECK (family_type IN ('recurring_pattern', 'worked_example_template',
            'misconception_cluster', 'exam_structure')),
    canonical_pattern TEXT,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE questions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    subtopic_id INTEGER REFERENCES topics(id),
    family_id INTEGER REFERENCES question_families(id),

    -- Content
    stem TEXT NOT NULL,
    question_format TEXT NOT NULL DEFAULT 'mcq'
        CHECK (question_format IN ('mcq', 'short_answer', 'numeric', 'true_false', 'matching', 'ordering')),
    explanation_text TEXT,

    -- Metadata
    difficulty_level INTEGER NOT NULL DEFAULT 5000,  -- basis points
    estimated_time_seconds INTEGER NOT NULL DEFAULT 30,
    marks INTEGER NOT NULL DEFAULT 1,
    source_type TEXT DEFAULT 'authored'
        CHECK (source_type IN ('past_question', 'authored', 'generated', 'teacher_upload')),
    source_ref TEXT,      -- e.g., "BECE 2023 Paper 1 Q5"
    exam_year INTEGER,

    -- Intelligence (8-axis classification from idea22)
    primary_knowledge_role TEXT,
    primary_cognitive_demand TEXT,
    primary_solve_pattern TEXT,
    primary_pedagogic_function TEXT,
    classification_confidence INTEGER DEFAULT 0,
    intelligence_snapshot TEXT DEFAULT '{}',  -- Full 8-axis JSON

    -- Skills
    primary_skill_id INTEGER REFERENCES academic_nodes(id),
    cognitive_level TEXT,

    -- Quality
    is_active INTEGER NOT NULL DEFAULT 1,
    pack_id TEXT REFERENCES content_packs(pack_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE question_options (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    option_label TEXT NOT NULL,    -- 'A', 'B', 'C', 'D'
    option_text TEXT NOT NULL,
    is_correct INTEGER NOT NULL DEFAULT 0,
    misconception_id INTEGER REFERENCES misconception_patterns(id),
    distractor_intent TEXT,        -- Why this wrong answer is tempting
    position INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE question_skill_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    contribution_weight INTEGER NOT NULL DEFAULT 10000,  -- basis points
    is_primary INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_questions_subject ON questions(subject_id);
CREATE INDEX idx_questions_topic ON questions(topic_id);
CREATE INDEX idx_questions_subtopic ON questions(subtopic_id);
CREATE INDEX idx_questions_family ON questions(family_id);
CREATE INDEX idx_questions_difficulty ON questions(difficulty_level);
CREATE INDEX idx_questions_active ON questions(is_active);
CREATE INDEX idx_question_options_question ON question_options(question_id);
CREATE INDEX idx_question_skill_links_question ON question_skill_links(question_id);
CREATE INDEX idx_question_skill_links_node ON question_skill_links(node_id);
```

### Task 1.2: Student state schema (migration 004)

```sql
-- 004_student_state.sql

CREATE TABLE student_topic_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),

    -- Mastery
    mastery_score INTEGER NOT NULL DEFAULT 0,        -- 0-10000 bp
    mastery_state TEXT NOT NULL DEFAULT 'unseen'
        CHECK (mastery_state IN (
            'unseen', 'exposed', 'emerging', 'partial', 'fragile',
            'stable', 'robust', 'exam_ready'
        )),

    -- Dimensions
    accuracy_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    retention_score INTEGER NOT NULL DEFAULT 0,
    transfer_score INTEGER NOT NULL DEFAULT 0,
    consistency_score INTEGER NOT NULL DEFAULT 0,

    -- Gap & Priority
    gap_score INTEGER NOT NULL DEFAULT 10000,        -- 10000 - mastery_score
    priority_score INTEGER NOT NULL DEFAULT 0,

    -- Trend
    trend_state TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_state IN ('improving', 'stable', 'fragile', 'declining', 'critical')),

    -- Fragility
    fragility_score INTEGER NOT NULL DEFAULT 0,
    pressure_collapse_index INTEGER NOT NULL DEFAULT 0,

    -- Evidence
    total_attempts INTEGER NOT NULL DEFAULT 0,
    correct_attempts INTEGER NOT NULL DEFAULT 0,
    recent_attempts_window INTEGER NOT NULL DEFAULT 0,
    recent_correct_window INTEGER NOT NULL DEFAULT 0,
    evidence_count INTEGER NOT NULL DEFAULT 0,

    -- Timing
    last_seen_at TEXT,
    last_correct_at TEXT,
    last_mastered_at TEXT,
    last_decline_at TEXT,

    -- Memory
    decay_risk INTEGER NOT NULL DEFAULT 0,
    next_review_at TEXT,
    memory_strength INTEGER NOT NULL DEFAULT 0,

    -- Flags
    is_blocked INTEGER NOT NULL DEFAULT 0,
    is_urgent INTEGER NOT NULL DEFAULT 0,
    is_exam_critical INTEGER NOT NULL DEFAULT 0,
    repair_priority INTEGER NOT NULL DEFAULT 0,

    -- Metadata
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(student_id, topic_id)
);

CREATE TABLE student_error_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),

    -- Error type intensities (0-10000 bp each)
    knowledge_gap_score INTEGER NOT NULL DEFAULT 0,
    conceptual_confusion_score INTEGER NOT NULL DEFAULT 0,
    recognition_failure_score INTEGER NOT NULL DEFAULT 0,
    execution_error_score INTEGER NOT NULL DEFAULT 0,
    carelessness_score INTEGER NOT NULL DEFAULT 0,
    pressure_breakdown_score INTEGER NOT NULL DEFAULT 0,
    expression_weakness_score INTEGER NOT NULL DEFAULT 0,
    speed_error_score INTEGER NOT NULL DEFAULT 0,

    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE TABLE student_question_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    session_id INTEGER,              -- FK to session table (created later)
    session_type TEXT,               -- 'practice', 'diagnostic', 'mock', 'gap_repair', etc.

    -- Attempt data
    attempt_number INTEGER NOT NULL DEFAULT 1,
    started_at TEXT NOT NULL,
    submitted_at TEXT,
    response_time_ms INTEGER,

    -- Response
    selected_option_id INTEGER REFERENCES question_options(id),
    answer_text TEXT,                -- For non-MCQ formats
    is_correct INTEGER NOT NULL DEFAULT 0,

    -- Behavioral signals
    confidence_level TEXT CHECK (confidence_level IN ('sure', 'not_sure', 'guessed')),
    hint_count INTEGER NOT NULL DEFAULT 0,
    changed_answer_count INTEGER NOT NULL DEFAULT 0,
    skipped INTEGER NOT NULL DEFAULT 0,
    timed_out INTEGER NOT NULL DEFAULT 0,

    -- Diagnostics
    error_type TEXT,
    misconception_triggered_id INTEGER REFERENCES misconception_patterns(id),
    support_level TEXT DEFAULT 'independent'
        CHECK (support_level IN ('independent', 'guided', 'heavily_guided')),

    -- Context
    was_timed INTEGER NOT NULL DEFAULT 0,
    was_transfer_variant INTEGER NOT NULL DEFAULT 0,
    was_retention_check INTEGER NOT NULL DEFAULT 0,
    was_mixed_context INTEGER NOT NULL DEFAULT 0,

    -- Evidence
    evidence_weight INTEGER NOT NULL DEFAULT 10000,  -- Reduced by hints, timing, etc.

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_student_topic_states_student ON student_topic_states(student_id);
CREATE INDEX idx_student_topic_states_topic ON student_topic_states(topic_id);
CREATE INDEX idx_student_topic_states_mastery ON student_topic_states(mastery_state);
CREATE INDEX idx_student_topic_states_priority ON student_topic_states(priority_score DESC);
CREATE INDEX idx_student_topic_states_review ON student_topic_states(next_review_at);
CREATE INDEX idx_student_error_profiles ON student_error_profiles(student_id, topic_id);
CREATE INDEX idx_attempts_student ON student_question_attempts(student_id);
CREATE INDEX idx_attempts_question ON student_question_attempts(question_id);
CREATE INDEX idx_attempts_session ON student_question_attempts(session_id);
CREATE INDEX idx_attempts_created ON student_question_attempts(created_at);
```

## Week 5: Answer Processing Pipeline

### Task 1.3: The core evidence loop

This is the most important backend workflow. Every answer flows through this pipeline:

```
Student submits answer
    → Validate submission
    → Score correctness
    → Classify error type (if wrong)
    → Compute evidence weight (reduce for hints, guessing, etc.)
    → Write student_question_attempt record
    → Update student_topic_state
        → Recalculate mastery_score
        → Recalculate gap_score, priority_score
        → Recalculate trend_state
        → Check for state transitions (exposed → emerging → partial, etc.)
    → Update student_error_profile
    → Check for misconception recurrence
    → Emit events for downstream consumers (memory, coach, reporting)
```

**Mastery formula** (from idea6, idea20, idea28):
```rust
/// Mastery = weighted composite of performance dimensions
/// Source: idea6 (simplified first-version), idea20, idea28
pub fn compute_mastery(state: &StudentTopicState) -> BasisPoints {
    let accuracy = from_bp(state.accuracy_score);
    let retention = from_bp(state.retention_score);
    let transfer = from_bp(state.transfer_score);
    let speed = from_bp(state.speed_score);
    let confidence = from_bp(state.confidence_score);
    let consistency = from_bp(state.consistency_score);

    let mastery = 0.35 * accuracy
        + 0.20 * retention
        + 0.15 * transfer
        + 0.15 * speed
        + 0.10 * confidence
        + 0.05 * consistency;

    to_bp(mastery)
}
```

**Priority formula** (from idea6):
```rust
/// Priority = how urgently this topic needs attention
/// Source: idea6
pub fn compute_priority(
    gap: f64,           // 0-1
    trend_risk: f64,    // 0-1 (declining = high)
    dependency_impact: f64, // 0-1 (blocks other topics = high)
    exam_weight: f64,   // 0-1
    recurrence: f64,    // 0-1 (same errors repeating)
    forgetting_risk: f64,
    misconception_penalty: f64,
) -> BasisPoints {
    let priority = 0.30 * gap
        + 0.20 * trend_risk
        + 0.15 * dependency_impact
        + 0.15 * exam_weight
        + 0.10 * recurrence
        + 0.05 * forgetting_risk
        + 0.05 * misconception_penalty;

    to_bp(priority)
}
```

**Error classification** (from idea11):
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorType {
    KnowledgeGap,           // Concept not known
    ConceptualConfusion,    // Concept confused with similar
    RecognitionFailure,     // Knew it but didn't recognize the form
    ExecutionError,         // Knew method but made procedural mistake
    Carelessness,           // Knew it, was capable, but slipped
    PressureBreakdown,      // Knew it untimed, failed under time pressure
    ExpressionWeakness,     // Knew it but couldn't articulate
    SpeedError,             // Correct thinking but too slow
    GuessingDetected,       // No real knowledge, picked randomly
    MisconceptionTriggered, // Specific false belief caused the error
}

/// Classify error type from attempt context
/// Source: idea11 diagnostic pipeline, idea18, idea29
pub fn classify_error(
    attempt: &QuestionAttempt,
    question: &Question,
    selected_option: &QuestionOption,
    student_state: &StudentTopicState,
) -> ErrorType {
    // Stage 1: Check if misconception triggered
    if let Some(misconception_id) = &selected_option.misconception_id {
        return ErrorType::MisconceptionTriggered;
    }

    // Stage 2: Check for guessing signals
    let guess_likelihood = compute_guess_likelihood(attempt);
    if guess_likelihood > 0.7 {
        return ErrorType::GuessingDetected;
    }

    // Stage 3: Check for pressure breakdown
    if attempt.was_timed && student_state.accuracy_score > 6000
        && attempt.response_time_ms.unwrap_or(0) < (question.estimated_time_seconds as i64 * 400) {
        return ErrorType::PressureBreakdown;
    }

    // Stage 4: Check for carelessness
    if student_state.mastery_score > 6500
        && attempt.response_time_ms.unwrap_or(0) < (question.estimated_time_seconds as i64 * 500)
        && attempt.hint_count == 0 {
        return ErrorType::Carelessness;
    }

    // Stage 5: Check for recognition failure
    if attempt.was_transfer_variant && !attempt.was_timed
        && student_state.accuracy_score > 5000 {
        return ErrorType::RecognitionFailure;
    }

    // Stage 6: Default to knowledge gap
    if student_state.mastery_score < 3000 {
        return ErrorType::KnowledgeGap;
    }

    // Stage 7: Conceptual confusion
    ErrorType::ConceptualConfusion
}
```

**Evidence weight computation** (from idea7, idea6):
```rust
/// Reduce evidence weight based on conditions
/// Source: idea7 (identical repeats get diminishing weight)
pub fn compute_evidence_weight(attempt: &QuestionAttempt) -> BasisPoints {
    let mut weight: f64 = 1.0;

    // Hints halve evidence weight (idea28)
    if attempt.hint_count > 0 {
        weight *= 0.5_f64.powi(attempt.hint_count.min(3) as i32);
    }

    // Guided support reduces weight
    match attempt.support_level.as_deref() {
        Some("guided") => weight *= 0.7,
        Some("heavily_guided") => weight *= 0.4,
        _ => {}
    }

    // Transfer evidence carries extra value (idea7)
    if attempt.was_transfer_variant {
        weight *= 1.3;
    }

    // Delayed retention check carries extra value
    if attempt.was_retention_check {
        weight *= 1.5;
    }

    // Low confidence correct = reduced certainty
    if attempt.is_correct == 1 && attempt.confidence_level.as_deref() == Some("guessed") {
        weight *= 0.5;
    }

    to_bp(weight.clamp(0.0, 2.0) / 2.0)  // Normalize to 0-10000
}
```

### Task 1.4: Student model update service

```rust
// ecoach-student-model/src/services.rs

pub struct StudentModelService { db: DbPool }

impl StudentModelService {
    /// Process a completed answer and update all student state
    /// This is the CORE PIPELINE of the entire system
    pub async fn process_answer(
        &self,
        student_id: i64,
        submission: AnswerSubmission,
    ) -> Result<AnswerProcessingResult, StudentModelError> {
        // 1. Load question + options
        let question = self.load_question(submission.question_id).await?;
        let selected_option = self.load_option(submission.selected_option_id).await?;

        // 2. Score correctness
        let is_correct = selected_option.is_correct;

        // 3. Classify error (if wrong)
        let topic_state = self.get_or_create_topic_state(student_id, question.topic_id).await?;
        let error_type = if !is_correct {
            Some(classify_error(&submission, &question, &selected_option, &topic_state))
        } else {
            None
        };

        // 4. Compute evidence weight
        let evidence_weight = compute_evidence_weight(&submission);

        // 5. Write attempt record
        let attempt = self.write_attempt(student_id, &submission, is_correct, &error_type, evidence_weight).await?;

        // 6. Update topic state
        let updated_state = self.update_topic_state(student_id, question.topic_id, &attempt).await?;

        // 7. Update error profile
        if let Some(ref et) = error_type {
            self.update_error_profile(student_id, question.topic_id, et).await?;
        }

        // 8. Check misconception recurrence
        if let Some(misconception_id) = selected_option.misconception_id {
            self.record_misconception_hit(student_id, misconception_id, question.topic_id).await?;
        }

        // 9. Emit domain events
        let events = vec![
            DomainEvent::new("answer.processed", &attempt),
            DomainEvent::new("topic_state.updated", &updated_state),
        ];

        Ok(AnswerProcessingResult {
            is_correct,
            error_type,
            explanation: question.explanation_text.clone(),
            selected_option_text: selected_option.option_text.clone(),
            correct_option_text: self.get_correct_option_text(question.id).await?,
            updated_mastery: updated_state.mastery_score,
            updated_gap: updated_state.gap_score,
            misconception_info: selected_option.distractor_intent.clone(),
        })
    }

    /// Recalculate topic state from recent evidence using EMA
    async fn update_topic_state(
        &self,
        student_id: i64,
        topic_id: i64,
        attempt: &QuestionAttempt,
    ) -> Result<StudentTopicState> {
        let mut state = self.get_or_create_topic_state(student_id, topic_id).await?;

        // EMA update for accuracy (alpha = 0.3 for recency weighting)
        let alpha = 0.3;
        let new_accuracy = if attempt.is_correct { 10000 } else { 0 };
        state.accuracy_score = ema_update(state.accuracy_score, new_accuracy, alpha);

        // Update speed score from response time
        if let Some(time_ms) = attempt.response_time_ms {
            let expected_ms = attempt.estimated_time_seconds.unwrap_or(30) * 1000;
            let speed = (expected_ms as f64 / time_ms.max(1) as f64).clamp(0.0, 1.0);
            state.speed_score = ema_update(state.speed_score, to_bp(speed), alpha);
        }

        // Update evidence counts
        state.total_attempts += 1;
        if attempt.is_correct { state.correct_attempts += 1; }
        state.evidence_count += 1;
        state.last_seen_at = Some(now_iso());
        if attempt.is_correct { state.last_correct_at = Some(now_iso()); }

        // Recompute mastery
        state.mastery_score = compute_mastery(&state);
        state.gap_score = 10000 - state.mastery_score;

        // Recompute priority
        state.priority_score = compute_priority_from_state(&state);

        // State transition check
        state.mastery_state = resolve_mastery_state(&state);

        // Trend update
        state.trend_state = compute_trend(&state);

        state.version += 1;
        state.updated_at = now_iso();

        self.save_topic_state(&state).await?;
        Ok(state)
    }
}

fn ema_update(old: BasisPoints, new: BasisPoints, alpha: f64) -> BasisPoints {
    let result = alpha * new as f64 + (1.0 - alpha) * old as f64;
    result.round() as BasisPoints
}
```

### Task 1.5: Mastery state machine (from idea28, idea29, idea32)

```rust
/// Topic mastery state transitions
/// Source: idea28 (Topic mastery state machine), idea29 (6 diagnostic dimensions)
pub fn resolve_mastery_state(state: &StudentTopicState) -> MasteryState {
    let m = from_bp(state.mastery_score);
    let f = from_bp(state.fragility_score);
    let e = state.evidence_count;

    match state.mastery_state {
        MasteryState::Unseen => {
            if e >= 1 { MasteryState::Exposed } else { MasteryState::Unseen }
        }
        MasteryState::Exposed => {
            if e >= 3 && m >= 0.25 { MasteryState::Emerging }
            else { MasteryState::Exposed }
        }
        MasteryState::Emerging => {
            if m >= 0.45 && e >= 8 { MasteryState::Partial }
            else if m < 0.15 && e >= 5 { MasteryState::Exposed }  // regression
            else { MasteryState::Emerging }
        }
        MasteryState::Partial => {
            if m >= 0.60 && f < 0.30 && e >= 15 { MasteryState::Fragile }
            else if m < 0.35 { MasteryState::Emerging }  // regression
            else { MasteryState::Partial }
        }
        MasteryState::Fragile => {
            if m >= 0.72 && f < 0.20 && e >= 25 { MasteryState::Stable }
            else if m < 0.50 || f > 0.50 { MasteryState::Partial }
            else { MasteryState::Fragile }
        }
        MasteryState::Stable => {
            if m >= 0.82 && f < 0.15 && from_bp(state.transfer_score) >= 0.65
                && from_bp(state.retention_score) >= 0.70 { MasteryState::Robust }
            else if m < 0.60 || f > 0.35 { MasteryState::Fragile }
            else { MasteryState::Stable }
        }
        MasteryState::Robust => {
            if m >= 0.90 && from_bp(state.pressure_collapse_index) < 0.15
                && from_bp(state.retention_score) >= 0.80 { MasteryState::ExamReady }
            else if m < 0.72 || f > 0.25 { MasteryState::Stable }
            else { MasteryState::Robust }
        }
        MasteryState::ExamReady => {
            if m < 0.80 || f > 0.20 { MasteryState::Robust }  // regression
            else { MasteryState::ExamReady }
        }
    }
}
```

## Week 6-7: Practice Session + Question Selection

### Task 1.6: Session schema (migration 005)

```sql
-- 005_sessions.sql

CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_type TEXT NOT NULL
        CHECK (session_type IN (
            'practice', 'diagnostic', 'mock', 'gap_repair', 'memory_review',
            'coach_mission', 'custom_test', 'elite', 'game', 'traps'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    topic_ids TEXT,                   -- JSON array of topic IDs

    -- Configuration
    question_count INTEGER,
    duration_minutes INTEGER,
    is_timed INTEGER NOT NULL DEFAULT 0,
    difficulty_preference TEXT DEFAULT 'adaptive',

    -- State
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'active', 'paused', 'completed', 'abandoned')),
    started_at TEXT,
    paused_at TEXT,
    completed_at TEXT,

    -- Results (populated on completion)
    total_questions INTEGER NOT NULL DEFAULT 0,
    answered_questions INTEGER NOT NULL DEFAULT 0,
    correct_questions INTEGER NOT NULL DEFAULT 0,
    accuracy_score INTEGER,
    avg_response_time_ms INTEGER,

    -- Metadata
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_sessions_student ON sessions(student_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_type ON sessions(session_type);
```

### Task 1.7: Question selection engine

```rust
// ecoach-questions/src/selection.rs

/// Select questions for a practice session
/// Source: idea14 (candidate_fit formula), idea9 (question factory)
pub struct QuestionSelector { db: DbPool }

impl QuestionSelector {
    pub async fn select_questions(
        &self,
        request: QuestionSelectionRequest,
    ) -> Result<Vec<SelectedQuestion>, SelectionError> {
        // 1. Build candidate pool from topic scope
        let candidates = self.get_candidate_pool(&request).await?;

        // 2. Score each candidate
        let mut scored: Vec<(Question, f64)> = candidates
            .into_iter()
            .map(|q| {
                let fit = self.compute_candidate_fit(&q, &request);
                (q, fit)
            })
            .collect();

        // 3. Sort by fit score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 4. Apply hard constraints (anti-repetition, topic quotas, difficulty quotas)
        let selected = self.apply_assembly_constraints(scored, &request);

        // 5. Order for session (warm start → core → pressure → finish strong)
        let ordered = self.apply_ordering_pattern(selected, &request);

        Ok(ordered)
    }

    /// Candidate fit formula (from idea14)
    fn compute_candidate_fit(&self, question: &Question, request: &QuestionSelectionRequest) -> f64 {
        let scope_match = self.score_scope_match(question, request);
        let difficulty_fit = self.score_difficulty_fit(question, request);
        let weakness_match = self.score_weakness_match(question, request);
        let variety_bonus = self.score_variety_bonus(question, request);
        let recency_penalty = self.score_recency_penalty(question, request);

        0.25 * scope_match
            + 0.20 * difficulty_fit
            + 0.20 * weakness_match
            + 0.15 * variety_bonus
            + 0.10 * (1.0 - recency_penalty)
            + 0.10 * self.score_timing_fit(question, request)
    }
}
```

### Task 1.8: Practice session commands

```rust
// ecoach-commands/src/session_commands.rs

#[tauri::command]
pub async fn start_practice_session(
    state: State<'_, AppState>,
    student_id: i64,
    subject_id: i64,
    topic_ids: Vec<i64>,
    question_count: Option<i32>,
    is_timed: Option<bool>,
) -> Result<PracticeSessionDto, CommandError> {
    // 1. Create session record
    // 2. Select questions
    // 3. Return first question + session metadata
}

#[tauri::command]
pub async fn submit_practice_answer(
    state: State<'_, AppState>,
    session_id: i64,
    question_id: i64,
    selected_option_id: i64,
    response_time_ms: i64,
    confidence_level: Option<String>,
) -> Result<AnswerResultDto, CommandError> {
    // 1. Delegate to StudentModelService.process_answer()
    // 2. Get next question (or session complete)
    // 3. Return result + next question
}

#[tauri::command]
pub async fn complete_practice_session(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<SessionSummaryDto, CommandError> {
    // 1. Finalize session stats
    // 2. Generate summary (topics covered, accuracy, weak areas, recommendations)
    // 3. Return DTO
}
```

## Week 8: Student Dashboard + Readiness

### Task 1.9: Dashboard service

```rust
// ecoach-reporting/src/dashboard.rs

pub struct DashboardService { db: DbPool }

impl DashboardService {
    /// Student home dashboard
    pub async fn get_student_dashboard(&self, student_id: i64) -> Result<StudentDashboardDto> {
        let profile = self.get_student_profile(student_id).await?;
        let subjects = self.get_enrolled_subjects(student_id).await?;

        let mut subject_summaries = Vec::new();
        for subject in &subjects {
            let topics = self.get_topic_states(student_id, subject.id).await?;
            let readiness = self.compute_subject_readiness(student_id, subject.id).await?;
            subject_summaries.push(SubjectSummaryDto {
                subject_id: subject.id,
                subject_name: subject.name.clone(),
                readiness_band: readiness.band,
                mastered_topic_count: topics.iter().filter(|t| t.mastery_state == "stable" || t.mastery_state == "robust" || t.mastery_state == "exam_ready").count(),
                weak_topic_count: topics.iter().filter(|t| from_bp(t.mastery_score) < 0.40).count(),
                total_topic_count: topics.len(),
                top_weakness: self.get_top_weakness(student_id, subject.id).await?,
                recent_trend: readiness.trend,
            });
        }

        Ok(StudentDashboardDto {
            student_name: profile.display_name,
            exam_target: profile.exam_target,
            days_to_exam: self.compute_days_to_exam(&profile),
            subject_summaries,
            overall_readiness_band: self.compute_overall_readiness(student_id).await?,
            today_recommendation: self.get_today_recommendation(student_id).await?,
            recent_activity: self.get_recent_activity(student_id, 7).await?,
        })
    }
}
```

**Readiness bands** (from idea12, idea14, idea34):
```rust
pub fn classify_readiness_band(score: BasisPoints) -> &'static str {
    match from_bp(score) {
        s if s >= 0.85 => "Exam Ready",
        s if s >= 0.70 => "Strong",
        s if s >= 0.55 => "Building",
        s if s >= 0.40 => "At Risk",
        _ => "Not Ready",
    }
}
```

**Phase 1 Exit Criteria**:
- [ ] Student can start a practice session on any installed topic
- [ ] Questions are selected adaptively (weakness-aware, variety-aware)
- [ ] Every answer updates mastery, gap, priority, error profile, trend
- [ ] State machine transitions work (unseen → exposed → emerging → ... → exam_ready)
- [ ] Dashboard shows subject readiness, weak topics, recent activity
- [ ] Error classification works for MCQ answers
- [ ] 200+ questions available in sample Math pack
- [ ] All scoring formulas match documented specifications
- [ ] All tests pass

---

# 4. PHASE 2 — ESSENTIAL WORKFLOWS (Weeks 9-14)

## Week 9-10: Diagnostic Engine

### Task 2.1: Diagnostic schema

```sql
-- Add to 005_sessions.sql or create separate migration

CREATE TABLE diagnostic_instances (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    session_mode TEXT NOT NULL DEFAULT 'standard'
        CHECK (session_mode IN ('quick', 'standard', 'deep')),
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'phase_1', 'phase_2', 'phase_3', 'phase_4', 'phase_5', 'completed', 'abandoned')),
    started_at TEXT,
    completed_at TEXT,
    result_json TEXT,     -- Full diagnostic result object
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE diagnostic_session_phases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    phase_number INTEGER NOT NULL,
    phase_type TEXT NOT NULL
        CHECK (phase_type IN ('broad_scan', 'adaptive_zoom', 'condition_testing', 'stability_recheck', 'confidence_snapshot')),
    status TEXT NOT NULL DEFAULT 'pending',
    question_count INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    completed_at TEXT,
    phase_result_json TEXT
);

CREATE TABLE diagnostic_item_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id),
    phase_id INTEGER NOT NULL REFERENCES diagnostic_session_phases(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    display_order INTEGER NOT NULL,

    -- Same fields as student_question_attempts plus:
    condition_type TEXT DEFAULT 'normal'
        CHECK (condition_type IN ('normal', 'timed', 'recall', 'recognition', 'transfer', 'stability')),
    sibling_group_id TEXT,

    -- Response
    started_at TEXT,
    submitted_at TEXT,
    response_time_ms INTEGER,
    selected_option_id INTEGER,
    is_correct INTEGER,
    confidence_level TEXT,
    changed_answer_count INTEGER DEFAULT 0,
    skipped INTEGER DEFAULT 0,
    timed_out INTEGER DEFAULT 0,

    evidence_weight INTEGER NOT NULL DEFAULT 10000,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Task 2.2: Diagnostic engine service

```rust
// ecoach-diagnostics/src/engine.rs

/// 5-phase diagnostic battery (from idea18, idea34)
pub struct DiagnosticEngine { db: DbPool, question_selector: QuestionSelector }

impl DiagnosticEngine {
    /// Start a diagnostic session
    pub async fn start_diagnostic(
        &self,
        student_id: i64,
        subject_id: i64,
        mode: DiagnosticMode,  // Quick/Standard/Deep
    ) -> Result<DiagnosticSession>;

    /// Get next question for current phase
    /// Implements adaptive branching logic (idea34 Phase 2 rules)
    pub async fn get_next_item(
        &self,
        diagnostic_id: i64,
    ) -> Result<Option<DiagnosticItemDto>>;

    /// Submit answer to diagnostic item
    pub async fn submit_item_answer(
        &self,
        diagnostic_id: i64,
        item_id: i64,
        submission: ItemSubmission,
    ) -> Result<ItemResultDto>;

    /// Complete diagnostic and generate full profile
    pub async fn complete_diagnostic(
        &self,
        diagnostic_id: i64,
    ) -> Result<DiagnosticResultDto>;

    /// Phase 1: Broad Scan (18-24 questions)
    /// - 2-3 questions per topic, spread across difficulty
    /// - After Phase 1, classify each topic: strong/weak/uncertain/insufficient
    async fn run_broad_scan(&self, diagnostic_id: i64, subject_id: i64) -> Result<Phase1Result>;

    /// Phase 2: Adaptive Zoom (12-20 questions)
    /// - Strong topics: 1 transfer + 1 timed, then stop
    /// - Weak topics: 3-5 subskill probes + misconception isolators
    /// - Uncertain: 2-3 clarification + 1 alternate format + 1 confidence check
    async fn run_adaptive_zoom(&self, diagnostic_id: i64, phase1: &Phase1Result) -> Result<Phase2Result>;

    /// Phase 3: Condition Testing (6-10 questions)
    /// - Timed mirrors of previously untimed items
    /// - Recognition vs recall pairs
    /// - Transfer variants
    async fn run_condition_testing(&self, diagnostic_id: i64, phase2: &Phase2Result) -> Result<Phase3Result>;

    /// Phase 4: Stability Recheck (4-8 questions)
    /// - Revisit concepts indirectly through sibling items
    async fn run_stability_recheck(&self, diagnostic_id: i64, phase3: &Phase3Result) -> Result<Phase4Result>;

    /// Phase 5: Confidence Snapshot (1-2 min)
    /// - Self-report on how student felt about different topics
    async fn run_confidence_snapshot(&self, diagnostic_id: i64) -> Result<Phase5Result>;

    /// Generate diagnostic result profile
    /// Scoring per topic: mastery, fluency, precision, pressure, flexibility, stability
    /// Classification: Secure/Strong-but-slow/Recognized-not-owned/Partial/Misconception-driven/Pressure-sensitive/Likely-guessed/Not-learned
    async fn generate_result(&self, diagnostic_id: i64) -> Result<DiagnosticResult>;
}
```

**Diagnostic result object** (from idea18, idea34):
```rust
pub struct DiagnosticResult {
    pub overall_readiness: BasisPoints,
    pub readiness_band: String,
    pub topic_results: Vec<TopicDiagnosticResult>,
    pub condition_deltas: ConditionDeltas,
    pub error_profile_summary: ErrorProfileSummary,
    pub misconception_register: Vec<MisconceptionFinding>,
    pub recommended_next_actions: Vec<RecommendedAction>,
    pub student_type_classification: String,  // "Careful Thinker", "Fragile Knower", etc.
}

pub struct TopicDiagnosticResult {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_score: BasisPoints,
    pub fluency_score: BasisPoints,
    pub precision_score: BasisPoints,
    pub pressure_score: BasisPoints,
    pub flexibility_score: BasisPoints,
    pub stability_score: BasisPoints,
    pub fragility_index: BasisPoints,
    pub pressure_collapse_index: BasisPoints,
    pub recognition_gap_index: BasisPoints,
    pub classification: String,  // Secure/Strong-but-slow/etc.
    pub weakness_type_primary: Option<String>,
    pub recommended_intervention: String,
}

pub struct ConditionDeltas {
    pub speed_accuracy_delta: i16,     // Calm accuracy - Speed accuracy
    pub calm_pressure_delta: i16,
    pub direct_variant_delta: i16,
    pub recall_recognition_delta: i16,
    pub early_late_delta: i16,
    pub confidence_correctness_delta: i16,
}
```

## Week 11-12: Coach Brain + Plan Engine

### Task 2.3: Coach schema (migration 006)

```sql
-- 006_coach.sql

CREATE TABLE coach_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    exam_target TEXT,
    exam_date TEXT,
    start_date TEXT NOT NULL,
    total_days INTEGER,
    daily_budget_minutes INTEGER NOT NULL DEFAULT 60,
    current_phase TEXT NOT NULL DEFAULT 'foundation'
        CHECK (current_phase IN ('foundation', 'strengthening', 'performance', 'consolidation', 'final_revision')),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('draft', 'active', 'stale', 'completed')),
    plan_data_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE coach_plan_days (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id INTEGER NOT NULL REFERENCES coach_plans(id) ON DELETE CASCADE,
    date TEXT NOT NULL,
    phase TEXT NOT NULL,
    target_minutes INTEGER NOT NULL DEFAULT 60,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'missed', 'partial')),
    carryover_minutes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE coach_missions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_day_id INTEGER REFERENCES coach_plan_days(id),
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    title TEXT NOT NULL,
    reason TEXT NOT NULL,               -- Why this mission today
    subject_id INTEGER REFERENCES subjects(id),
    primary_topic_id INTEGER REFERENCES topics(id),
    activity_type TEXT NOT NULL
        CHECK (activity_type IN (
            'learn', 'guided_practice', 'worked_example', 'review',
            'speed_drill', 'repair', 'checkpoint', 'mixed_test',
            'memory_reactivation', 'pressure_conditioning'
        )),
    target_minutes INTEGER NOT NULL DEFAULT 20,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'active', 'completed', 'skipped', 'deferred')),
    steps_json TEXT NOT NULL DEFAULT '[]',  -- MissionStep[]
    success_criteria_json TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE coach_topic_profiles (
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    mastery_estimate INTEGER NOT NULL DEFAULT 0,
    fragility_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    misconception_recurrence INTEGER NOT NULL DEFAULT 0,
    evidence_count INTEGER NOT NULL DEFAULT 0,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    last_seen_at TEXT,
    blocked_status INTEGER NOT NULL DEFAULT 0,
    repair_priority INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (student_id, topic_id)
);

CREATE TABLE coach_session_evidence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mission_id INTEGER REFERENCES coach_missions(id),
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    activity_type TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    accuracy INTEGER,
    timed_accuracy INTEGER,
    avg_latency_ms INTEGER,
    misconception_tags TEXT DEFAULT '[]',
    confidence_score INTEGER,
    completed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE coach_blockers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    reason TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'moderate',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);

CREATE INDEX idx_coach_plans_student ON coach_plans(student_id);
CREATE INDEX idx_coach_plan_days_plan ON coach_plan_days(plan_id);
CREATE INDEX idx_coach_plan_days_date ON coach_plan_days(date);
CREATE INDEX idx_coach_missions_student ON coach_missions(student_id);
CREATE INDEX idx_coach_missions_status ON coach_missions(status);
CREATE INDEX idx_coach_topic_profiles ON coach_topic_profiles(student_id, topic_id);
CREATE INDEX idx_coach_evidence_student ON coach_session_evidence(student_id);
```

### Task 2.4: Coach brain state machine (from idea20)

```rust
// ecoach-coach-brain/src/state_machine.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoachState {
    OnboardingRequired,
    SubjectSelectionRequired,
    ContentReadinessRequired,
    DiagnosticRequired,
    PlanGenerationRequired,
    ReadyForTodayMission,
    MissionInProgress,
    MissionReviewRequired,
    RepairRequired,
    BlockedOnTopic,
    PlanAdjustmentRequired,
    ReviewDay,
    ExamMode,
    StalledNoContent,
}

/// Resolve current coach state (strict evaluation order from idea20)
pub async fn resolve_coach_state(
    db: &DbPool,
    student_id: i64,
) -> Result<CoachStateResolution> {
    // 1. Check onboarding
    let account = get_account(db, student_id).await?;
    if account.first_run == 1 {
        return Ok(CoachStateResolution::new(CoachState::OnboardingRequired));
    }

    // 2. Check subjects selected
    let profile = get_student_profile(db, student_id).await?;
    if profile.preferred_subjects.is_empty() {
        return Ok(CoachStateResolution::new(CoachState::SubjectSelectionRequired));
    }

    // 3. Check content readiness
    let readiness = check_content_readiness(db, student_id).await?;
    if !readiness.is_ready() {
        return Ok(CoachStateResolution::new(CoachState::ContentReadinessRequired));
    }

    // 4. Check pending review
    let pending_review = has_pending_mission_review(db, student_id).await?;
    if pending_review {
        return Ok(CoachStateResolution::new(CoachState::MissionReviewRequired));
    }

    // 5. Check active mission
    let active_mission = get_active_mission(db, student_id).await?;
    if active_mission.is_some() {
        return Ok(CoachStateResolution::new(CoachState::MissionInProgress));
    }

    // 6. Check repair needed
    let repair_needed = check_repair_needed(db, student_id).await?;
    if repair_needed {
        return Ok(CoachStateResolution::new(CoachState::RepairRequired));
    }

    // 7. Check blocked topics
    let blocked = get_blocked_topics(db, student_id).await?;
    if !blocked.is_empty() {
        return Ok(CoachStateResolution::new(CoachState::BlockedOnTopic));
    }

    // 8. Check diagnostic completed
    let has_baseline = has_completed_diagnostic(db, student_id).await?;
    if !has_baseline {
        return Ok(CoachStateResolution::new(CoachState::DiagnosticRequired));
    }

    // 9. Check active plan
    let plan = get_active_plan(db, student_id).await?;
    if plan.is_none() {
        return Ok(CoachStateResolution::new(CoachState::PlanGenerationRequired));
    }

    // 10. Check plan validity
    let plan = plan.unwrap();
    if plan.status == "stale" {
        return Ok(CoachStateResolution::new(CoachState::PlanAdjustmentRequired));
    }

    // 11. Ready for today's mission
    Ok(CoachStateResolution::new(CoachState::ReadyForTodayMission))
}
```

### Task 2.5: Plan engine V1 (deterministic, from idea20)

```rust
// ecoach-coach-brain/src/plan_engine.rs

pub struct PlanEngine { db: DbPool }

impl PlanEngine {
    /// Generate a study plan from diagnostic evidence + exam date
    /// Source: idea20 plan engine V1
    pub async fn generate_plan(
        &self,
        student_id: i64,
        exam_date: &str,
        subjects: &[i64],
        daily_budget_minutes: i32,
    ) -> Result<CoachPlan> {
        // 1. Compute days remaining
        let days_remaining = compute_days_remaining(exam_date);

        // 2. Get topic states for all subjects
        let topic_states = self.get_all_topic_states(student_id, subjects).await?;

        // 3. Classify topics: critical_weak, weak, fragile, stable, strong
        let classified = classify_topics(&topic_states);

        // 4. Compute phase distribution
        let phases = compute_phases(days_remaining, &classified);
        // foundation: days 0-20%, strengthening: 20-50%, performance: 50-75%,
        // consolidation: 75-90%, final_revision: 90-100%

        // 5. Allocate daily time across subjects
        let subject_allocation = allocate_subject_time(
            daily_budget_minutes, subjects, &classified,
        );

        // 6. Generate plan days
        let plan_days = generate_plan_days(
            days_remaining, &phases, &subject_allocation, &classified,
        );

        // 7. Persist
        let plan = self.save_plan(student_id, exam_date, daily_budget_minutes, plan_days).await?;

        Ok(plan)
    }

    /// Generate today's missions from the plan
    pub async fn generate_today_missions(
        &self,
        student_id: i64,
    ) -> Result<Vec<CoachMission>> {
        let plan = self.get_active_plan(student_id).await?;
        let today = today_date();
        let plan_day = self.get_plan_day(&plan, &today).await?;
        let topic_profiles = self.get_topic_profiles(student_id).await?;
        let blockers = self.get_blockers(student_id).await?;

        let missions = self.compose_missions(
            student_id,
            &plan_day,
            &topic_profiles,
            &blockers,
        ).await?;

        Ok(missions)
    }
}
```

## Week 13-14: Memory Engine + Knowledge Gap + Parent Dashboard

### Task 2.6: Memory engine schema + service (migration 007)

```sql
-- 007_memory.sql

CREATE TABLE memory_states (
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),

    memory_state TEXT NOT NULL DEFAULT 'not_formed'
        CHECK (memory_state IN (
            'not_formed', 'emerging', 'fragile', 'stable', 'strong', 'slipping', 'collapsed'
        )),
    memory_strength INTEGER NOT NULL DEFAULT 0,
    recall_fluency INTEGER NOT NULL DEFAULT 0,
    decay_risk INTEGER NOT NULL DEFAULT 0,
    spacing_need_days INTEGER NOT NULL DEFAULT 7,

    last_successful_recall_at TEXT,
    last_failed_recall_at TEXT,
    next_reactivation_due_at TEXT,

    retrieval_success_count INTEGER NOT NULL DEFAULT 0,
    retrieval_failure_count INTEGER NOT NULL DEFAULT 0,

    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (student_id, topic_id)
);

CREATE TABLE memory_evidence_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    question_id INTEGER REFERENCES questions(id),
    session_id INTEGER REFERENCES sessions(id),

    retrieval_mode TEXT NOT NULL
        CHECK (retrieval_mode IN ('recognition', 'cued_recall', 'free_recall', 'application', 'transfer', 'explanation')),
    is_correct INTEGER NOT NULL,
    response_time_ms INTEGER,
    confidence_level TEXT,
    hint_used INTEGER NOT NULL DEFAULT 0,
    was_timed INTEGER NOT NULL DEFAULT 0,
    was_delayed INTEGER NOT NULL DEFAULT 0,   -- Time since last exposure
    delay_days INTEGER,

    evidence_weight INTEGER NOT NULL DEFAULT 10000,
    occurred_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE recheck_schedules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    scheduled_for TEXT NOT NULL,
    schedule_type TEXT NOT NULL
        CHECK (schedule_type IN ('initial', 'short_gap', 'medium_gap', 'long_gap', 'exam_proximity', 'relapse_followup')),
    priority_score INTEGER NOT NULL DEFAULT 5000,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'completed', 'missed', 'rescheduled')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE interference_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_a_id INTEGER NOT NULL REFERENCES topics(id),
    topic_b_id INTEGER NOT NULL REFERENCES topics(id),
    confusion_strength INTEGER NOT NULL DEFAULT 0,
    total_confusions INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'unrelated'
        CHECK (status IN ('unrelated', 'weak_overlap', 'watchlist', 'active', 'high_risk', 'severe_confusion', 'stabilized')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_a_id, topic_b_id)
);

CREATE INDEX idx_memory_states_student ON memory_states(student_id);
CREATE INDEX idx_memory_states_due ON memory_states(next_reactivation_due_at);
CREATE INDEX idx_memory_states_risk ON memory_states(decay_risk DESC);
CREATE INDEX idx_memory_evidence_student ON memory_evidence_events(student_id);
CREATE INDEX idx_recheck_schedules ON recheck_schedules(student_id, scheduled_for, status);
CREATE INDEX idx_interference_edges ON interference_edges(student_id);
```

### Task 2.7: Knowledge gap mode schema (migration 009)

```sql
-- 009_knowledge_gap.sql

CREATE TABLE gap_repair_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER REFERENCES subjects(id),
    plan_type TEXT NOT NULL DEFAULT 'daily'
        CHECK (plan_type IN ('daily', 'weekly', 'urgent', 'post_mock')),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('draft', 'active', 'completed', 'abandoned')),
    generated_reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE gap_repair_plan_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repair_plan_id INTEGER NOT NULL REFERENCES gap_repair_plans(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    priority_score_snapshot INTEGER NOT NULL,
    reason_code TEXT NOT NULL
        CHECK (reason_code IN ('critical_blocker', 'slipping', 'forgetting', 'exam_critical', 'recurring')),
    recommended_mode TEXT NOT NULL
        CHECK (recommended_mode IN ('reteach', 'guided_practice', 'transfer_drill', 'retention_check', 'speed_drill')),
    estimated_minutes INTEGER NOT NULL DEFAULT 15,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'completed', 'deferred')),
    display_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE solidification_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    plan_item_id INTEGER REFERENCES gap_repair_plan_items(id),
    session_mode TEXT NOT NULL
        CHECK (session_mode IN ('critical_repair', 'slipping_repair', 'retention_repair', 'misconception_repair')),
    status TEXT NOT NULL DEFAULT 'started'
        CHECK (status IN ('started', 'teaching', 'guided', 'proving', 'retesting', 'completed', 'failed')),
    starting_gap_score INTEGER,
    ending_gap_score INTEGER,
    success_flag INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);
```

### Task 2.8: Parent dashboard

```rust
// ecoach-reporting/src/parent_dashboard.rs

pub struct ParentDashboardService { db: DbPool }

impl ParentDashboardService {
    /// Get parent overview for all linked children
    pub async fn get_parent_dashboard(
        &self,
        parent_id: i64,
    ) -> Result<ParentDashboardDto> {
        let children = self.get_linked_students(parent_id).await?;
        let mut child_summaries = Vec::new();

        for child in children {
            let readiness = self.compute_child_readiness(child.id).await?;
            let risks = self.get_active_risks(child.id).await?;
            let recent_trend = self.compute_weekly_trend(child.id).await?;

            child_summaries.push(ChildSummaryDto {
                student_id: child.id,
                student_name: child.display_name,
                overall_readiness_band: readiness.band,
                subject_readiness: readiness.by_subject,
                top_risk: risks.first().map(|r| r.summary.clone()),
                weekly_trend: recent_trend,
                consistency_label: self.compute_consistency(child.id).await?,
                days_to_exam: readiness.days_to_exam,
                last_active: child.last_active_at,
            });
        }

        Ok(ParentDashboardDto {
            children: child_summaries,
        })
    }

    /// Plain-language insight for a specific child (from idea12)
    pub async fn get_child_insight(
        &self,
        child_id: i64,
    ) -> Result<Vec<ParentInsightDto>> {
        // Translate engine data into plain parent messages
        // e.g., "Victor's main challenge right now is fractions. He understands
        // the basics but struggles when fractions appear in word problems."
    }
}
```

**Phase 2 Exit Criteria**:
- [ ] Diagnostic battery works: 5 phases, adaptive branching, full profile output
- [ ] Coach brain state machine resolves correctly for all 14 states
- [ ] Plan engine generates deterministic study plans from diagnostic + exam date
- [ ] Daily missions are generated from plan
- [ ] Memory engine tracks decay risk and schedules reviews
- [ ] Knowledge gap mode: gap scoring, repair plans, solidification sessions
- [ ] Parent can see readiness, risks, and plain-language insights per child
- [ ] Full evidence pipeline: answer → mastery → memory → plan → mission → parent view

---

# 5. PHASE 3 — EXAM SIMULATION & INTELLIGENCE (Weeks 15-20)

## Key deliverables:

### Mock Centre (migration 008)
- Mock blueprint compiler (selects questions from families respecting exam blueprint quotas)
- Mock session runtime (strict timing, no hints, auto-save, pause/resume)
- Mock scoring engine (correct/incorrect, marks, topic breakdown)
- Post-mock diagnosis (where marks were lost, why, what to do next)
- Readiness update after mock (strongest predictor of exam performance)
- Mock review mode (question-by-question with explanations)

### Question Intelligence Engine
- 8-axis classification pipeline (rule-based layer first)
- Question family clustering (group questions testing same underlying skill)
- Misconception mapping (which wrong answers reveal which false beliefs)
- Classification stored per question in intelligence_snapshot JSON

### Customised Testing Engine
- Intent capture (test type, subjects, topics, days left, preparation mode)
- Scope resolution (expand topics into testable units)
- Blueprint generation (question count, difficulty distribution, ordering pattern)
- Session runner with adaptation (event-driven trigger checks after each answer)
- Result interpretation (readiness score, weakness map, recommendations)

### Goal & Calendar System (migration 010)
- Goal CRUD with hierarchy (north star → campaign → tactical → background)
- Calendar events (exams, mocks, class tests with dates and subjects)
- Preparation intensity phases (Build → Strengthen → Firm Up → Wrap Up → Perform)
- Dynamic replanning when dates change

### Past Paper Intelligence (migration 016)
- Question family mining from historical papers
- Recurrence analytics (which patterns appear repeatedly)
- Co-appearance and inverse-appearance relationships

**Phase 3 Exit Criteria**:
- [ ] Students can take realistic mock exams with strict timing
- [ ] Post-mock analysis shows exactly where and why marks were lost
- [ ] Questions carry 8-axis intelligence classification
- [ ] Custom tests can be generated for specific upcoming assessments
- [ ] Goals and exam dates drive preparation intensity
- [ ] Calendar changes trigger automatic strategy replanning

---

# 6. PHASE 4 — HARDENING & SCALE (Weeks 21-26)

## Key deliverables:

- **Elite Mode**: Tier system (Foundation → Core → Prime → Apex → Master → Legend), EPS scoring, baseline calibration, 7 session types
- **Beat Yesterday**: Daily growth engine, 3 axes (volume/accuracy/pace), momentum/strain scores, daily target generation
- **Games**: MindStack (Tetris-learning hybrid) with control permission system, answer-to-control mapping, board physics
- **Traps**: 5 confusion-resolution modes (Difference Drill, Similarity Trap, Know the Difference, Which Is Which, Unmask)
- **Glossary Lab**: Knowledge entries with relations, bundles, search, audio playback queue
- **Library Intelligence**: Content relationship engine, item states, auto-generated shelves, recommendations
- **Document Intake**: Basic upload → OCR bridge → text extraction → topic mapping → evidence creation
- **Backup/Restore**: Export database + packs to user-chosen location, restore from backup
- **Performance**: Query optimization, caching hot queries (topic states, dashboard), startup time under 3 seconds
- **Comprehensive tests**: 80%+ code coverage on scoring formulas, state machines, and evidence pipeline

---

# 7. MASTER ENTITY REFERENCE

| Entity | Table | Module | Phase |
|--------|-------|--------|-------|
| Account | accounts | identity | 0 |
| StudentProfile | student_profiles | identity | 0 |
| ParentProfile | parent_profiles | identity | 0 |
| CurriculumVersion | curriculum_versions | curriculum | 0 |
| Subject | subjects | curriculum | 0 |
| Topic | topics | curriculum | 0 |
| AcademicNode | academic_nodes | curriculum | 0 |
| NodeEdge | node_edges | curriculum | 0 |
| MisconceptionPattern | misconception_patterns | curriculum | 0 |
| ContentPack | content_packs | content | 0 |
| QuestionFamily | question_families | questions | 1 |
| Question | questions | questions | 1 |
| QuestionOption | question_options | questions | 1 |
| QuestionSkillLink | question_skill_links | questions | 1 |
| StudentTopicState | student_topic_states | student-model | 1 |
| StudentErrorProfile | student_error_profiles | student-model | 1 |
| StudentQuestionAttempt | student_question_attempts | student-model | 1 |
| Session | sessions | sessions | 1 |
| DiagnosticInstance | diagnostic_instances | diagnostics | 2 |
| CoachPlan | coach_plans | coach-brain | 2 |
| CoachMission | coach_missions | coach-brain | 2 |
| CoachTopicProfile | coach_topic_profiles | coach-brain | 2 |
| MemoryState | memory_states | memory | 2 |
| RecheckSchedule | recheck_schedules | memory | 2 |
| InterferenceEdge | interference_edges | memory | 2 |
| GapRepairPlan | gap_repair_plans | knowledge-gap | 2 |
| SolidificationSession | solidification_sessions | knowledge-gap | 2 |
| MockSession | mock_sessions | mock-centre | 3 |
| Goal | goals | goals-calendar | 3 |
| CalendarEvent | calendar_events | goals-calendar | 3 |

---

# 8. MASTER STATE MACHINE REFERENCE

| State Machine | States | Source |
|---------------|--------|--------|
| Topic Mastery | unseen → exposed → emerging → partial → fragile → stable → robust → exam_ready | idea28, idea29 |
| Coach Lifecycle | 14 states (onboarding → ... → ready_for_mission → mission_in_progress → ...) | idea20 |
| Memory | not_formed → emerging → fragile → stable → strong → slipping → collapsed | idea32, idea7 |
| Decay Status | stable → watchlist → fragile → decaying → collapsed | idea32 |
| Mock Session | created → ready → active → paused → submitting → scored → review_ready | idea1 |
| Diagnostic | created → phase_1 → phase_2 → phase_3 → phase_4 → phase_5 → completed | idea18, idea34 |
| Intervention | planned → delivered → verifying → provisionally_successful → securely_successful | idea24 |
| Goal | drafted → confirmed → active → paused/blocked/at_risk → completed | idea36 |
| Content Trust | raw → extracted → parsed → reviewed → trusted → published → deprecated | idea38 |
| Knowledge State | unknown → weak → fragile → repairing → stable → mastered → at_risk → declining | idea6 |

---

# 9. MASTER SCORING FORMULA REFERENCE

| Formula | Expression | Source |
|---------|-----------|--------|
| Mastery | 0.35*accuracy + 0.20*retention + 0.15*transfer + 0.15*speed + 0.10*confidence + 0.05*consistency | idea6, idea20 |
| Gap | 10000 - mastery | idea6 |
| Priority | 0.30*gap + 0.20*trend_risk + 0.15*dependency + 0.15*exam_weight + 0.10*recurrence + 0.05*forgetting + 0.05*misconception | idea6 |
| Fragility | StdDev(baseline, speed, precision, pressure, flex) normalized | idea18 |
| Pressure Collapse | max(0, baseline - pressure) + timeout_penalty + rushed_error_penalty | idea18 |
| Recognition Gap | direct_variant_score - disguised_variant_score | idea18 |
| Momentum (Beat Yesterday) | 0.35*volume_growth + 0.40*accuracy_growth + 0.25*pace_growth | idea4 |
| Strain | 0.30*accuracy_drop + 0.20*completion_drop + 0.20*hint_spike + 0.15*skip_spike + 0.15*pace_instability | idea4 |
| Elite EPS | Session-type-specific weighted composite of accuracy, precision, speed, depth, composure, consistency, independence, trap_resistance | idea5 |
| Memory RAS | 0.35*accuracy + 0.20*speed + 0.20*independence + 0.15*consistency + 0.10*confidence_alignment | idea7 |
| Memory DCS | 0.25*time_separated + 0.20*variant + 0.15*embedded_use + 0.15*interference_resistance + 0.15*recheck_stability + 0.10*relearning_efficiency | idea7 |
| Decay Risk | 0.22*time_decay + 0.18*stability_decay + 0.16*latency_decay + 0.14*support_decay + 0.15*mode_decay + 0.15*pressure_decay | idea32 |
| Intervention Priority | 0.25*decay + 0.20*dependency + 0.20*exam_urgency + 0.15*importance + 0.10*interference + 0.10*relapse | idea32 |
| Review Urgency | 0.28*decay + 0.20*dependency + 0.15*exam + 0.12*interference + 0.10*recovery + 0.08*pressure_gap + 0.07*importance | idea32 |
| Candidate Fit (Q selection) | 0.25*scope + 0.20*difficulty_fit + 0.20*weakness + 0.15*variety + 0.10*recency + 0.10*timing_fit | idea14 |
| Readiness | 0.35*accuracy + 0.15*coverage + 0.15*difficulty_handled + 0.10*speed + 0.10*consistency + 0.10*pressure_stability - 0.05*misconception_penalty | idea14 |

---

# 10. MASTER API COMMAND REFERENCE

## Phase 0 Commands (Week 1-3)
| Command | Input | Output |
|---------|-------|--------|
| `list_accounts` | — | `Vec<AccountSummaryDto>` |
| `create_account` | name, type, pin | `AccountDto` |
| `login_with_pin` | account_id, pin | `SessionDto` |
| `switch_account` | account_id | `SessionDto` |
| `link_parent_student` | parent_id, student_id | `()` |
| `get_subjects` | curriculum_version_id | `Vec<SubjectDto>` |
| `get_topic_tree` | subject_id | `TopicTreeDto` |
| `get_topic_detail` | topic_id | `TopicDetailDto` |
| `install_content_pack` | pack_path | `PackInstallResultDto` |
| `list_installed_packs` | — | `Vec<PackSummaryDto>` |

## Phase 1 Commands (Week 4-8)
| Command | Input | Output |
|---------|-------|--------|
| `start_practice_session` | student_id, subject_id, topic_ids, count, timed | `PracticeSessionDto` |
| `get_next_question` | session_id | `QuestionDto` |
| `submit_answer` | session_id, question_id, option_id, time_ms, confidence | `AnswerResultDto` |
| `complete_session` | session_id | `SessionSummaryDto` |
| `get_student_dashboard` | student_id | `StudentDashboardDto` |
| `get_topic_mastery` | student_id, topic_id | `TopicMasteryDto` |
| `get_readiness_overview` | student_id | `ReadinessOverviewDto` |

## Phase 2 Commands (Week 9-14)
| Command | Input | Output |
|---------|-------|--------|
| `start_diagnostic` | student_id, subject_id, mode | `DiagnosticSessionDto` |
| `get_diagnostic_next_item` | diagnostic_id | `DiagnosticItemDto` |
| `submit_diagnostic_item` | diagnostic_id, item_id, answer | `DiagnosticItemResultDto` |
| `complete_diagnostic` | diagnostic_id | `DiagnosticResultDto` |
| `get_coach_state` | student_id | `CoachStateDto` |
| `get_today_missions` | student_id | `Vec<MissionDto>` |
| `start_mission` | mission_id | `MissionSessionDto` |
| `complete_mission` | mission_id, evidence | `MissionResultDto` |
| `get_memory_overview` | student_id | `MemoryOverviewDto` |
| `get_knowledge_gap_overview` | student_id | `GapOverviewDto` |
| `start_solidification_session` | student_id, topic_id | `SolidificationSessionDto` |
| `get_parent_dashboard` | parent_id | `ParentDashboardDto` |
| `get_child_summary` | parent_id, child_id | `ChildSummaryDto` |

## Phase 3 Commands (Week 15-20)
| Command | Input | Output |
|---------|-------|--------|
| `get_mock_centre_snapshot` | student_id | `MockCentreDto` |
| `start_mock` | student_id, blueprint | `MockSessionDto` |
| `record_mock_answer` | mock_id, question_id, answer | `()` |
| `submit_mock` | mock_id | `MockResultDto` |
| `get_mock_review` | mock_id | `MockReviewDto` |
| `create_custom_test` | student_id, test_config | `CustomTestDto` |
| `create_goal` | student_id, goal_input | `GoalDto` |
| `add_calendar_event` | student_id, event_input | `CalendarEventDto` |
| `get_weekly_plan` | student_id | `WeeklyPlanDto` |
| `get_daily_plan` | student_id, date | `DailyPlanDto` |

---

*This plan is grounded in all 38 idea files. Every schema, formula, state machine, and API contract traces back to specific source documents. Implementation should proceed phase by phase, with each phase producing a working vertical slice that a student can actually use.*
