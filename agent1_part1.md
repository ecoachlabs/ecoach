# backend_implement_agent1 — PART 1 & PART 2A

---

## PART 1: ARCHITECTURE & WORKSPACE

### 1.1 Technology Stack

| Layer | Technology | Source |
|-------|-----------|--------|
| Backend language | Rust (stable) | idea1, idea35, idea38 |
| Desktop shell | Tauri v2 | idea1, idea35 |
| Frontend | Nuxt 3 (TypeScript) | idea1, idea35 |
| Database | SQLite 3 with WAL mode | idea1, idea2, idea6 |
| IPC | Tauri command boundary (only interface) | idea1, idea35 |
| Score storage | BasisPoints as u16 (0-10000 = 0.00-100.00%) | idea5 |
| Content delivery | Signed offline content packs on local filesystem | idea1, idea2 |
| Background jobs | In-process Rust async (tokio) | offline-first constraint |
| Auth | argon2id PIN hashing | idea3 |
| Migrations | Embedded SQL runner (sqlx migrate) | architectural decision |

### 1.2 Cargo Workspace Layout (22 Crates)

```
ecoach-backend/
├── Cargo.toml                          # Workspace manifest
├── crates/
│   ├── ecoach-substrate/               # Shared types, enums, scoring, config, time
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs                # BasisPoints, ConfidenceScore, SeverityLevel
│   │   │   ├── scoring.rs              # to_bp, from_bp, ema_update, score normalization
│   │   │   ├── config.rs               # Threshold registry, tunable constants
│   │   │   ├── time.rs                 # Monotonic time, wall clock, duration helpers
│   │   │   ├── errors.rs               # Shared error types
│   │   │   └── events.rs               # DomainEvent trait, EventEnvelope
│   │   └── Cargo.toml
│   │
│   ├── ecoach-storage/                 # SQLite connection pool, migrations, repository traits
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── connection.rs           # Pool setup, WAL mode, foreign_keys ON, pragmas
│   │   │   ├── migrations.rs           # Embedded migration runner
│   │   │   ├── repository.rs           # Generic CRUD trait
│   │   │   └── query.rs                # Query builder helpers
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
│   │   │   ├── 020_elite.sql
│   │   │   ├── 021_event_log.sql
│   │   │   └── 022_system.sql
│   │   └── Cargo.toml
│   │
│   ├── ecoach-identity/                # Users, PINs, roles, entitlements, lockout
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── services.rs             # IdentityService
│   │   │   ├── pin.rs                  # argon2id hash/verify, lockout logic
│   │   │   └── models.rs               # Account, StudentProfile, ParentProfile
│   │   └── Cargo.toml
│   │
│   ├── ecoach-curriculum/              # Academic truth graph, topics, nodes, edges
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── services.rs             # CurriculumService
│   │   │   ├── tree.rs                 # Topic tree builder
│   │   │   └── models.rs               # Subject, Topic, AcademicNode, NodeEdge
│   │   └── Cargo.toml
│   │
│   ├── ecoach-content/                 # Content packs, pack installer, artifact management
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pack_service.rs         # install_pack, list_packs, remove_pack
│   │   │   ├── manifest.rs             # Manifest parsing, checksum verification
│   │   │   └── models.rs               # ContentPack, PackManifest
│   │   └── Cargo.toml
│   │
│   ├── ecoach-questions/               # Question intelligence, families, selection engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── selection.rs            # QuestionSelector, candidate_fit formula
│   │   │   ├── intelligence.rs         # 8-axis classification pipeline
│   │   │   ├── families.rs             # Family clustering, mutation operators
│   │   │   └── models.rs               # Question, QuestionOption, QuestionFamily
│   │   └── Cargo.toml
│   │
│   ├── ecoach-student-model/           # All student state, evidence pipeline, mastery
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── services.rs             # StudentModelService.process_answer()
│   │   │   ├── mastery.rs              # compute_mastery, resolve_mastery_state
│   │   │   ├── error_classification.rs # classify_error, ErrorType
│   │   │   ├── evidence.rs             # compute_evidence_weight, EMA
│   │   │   └── models.rs               # StudentTopicState, StudentErrorProfile
│   │   └── Cargo.toml
│   │
│   ├── ecoach-diagnostics/             # Diagnostic battery engine, 5-phase, DNA profile
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── engine.rs               # DiagnosticEngine, 5-phase battery
│   │   │   ├── phases.rs               # BroadScan, AdaptiveZoom, ConditionTesting, etc.
│   │   │   └── models.rs               # DiagnosticInstance, DiagnosticResult
│   │   └── Cargo.toml
│   │
│   ├── ecoach-coach-brain/             # 14-state machine, plan engine, missions, interventions
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── state_machine.rs        # resolve_coach_state, CoachState enum
│   │   │   ├── plan_engine.rs          # generate_plan, generate_today_missions
│   │   │   ├── mission_composer.rs     # compose_missions from plan_day
│   │   │   └── models.rs               # CoachPlan, CoachMission, CoachBlocker
│   │   └── Cargo.toml
│   │
│   ├── ecoach-memory/                  # Decay model, recall scheduling, interference
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── decay.rs                # Decay functions, memory_strength updates
│   │   │   ├── scheduler.rs            # RecheckScheduler, next_review_at logic
│   │   │   └── models.rs               # MemoryState, RecheckSchedule, InterferenceEdge
│   │   └── Cargo.toml
│   │
│   ├── ecoach-sessions/                # Session orchestrator, attempt capture, time tracking
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── orchestrator.rs         # start_session, submit_answer, end_session
│   │   │   └── models.rs               # Session
│   │   └── Cargo.toml
│   │
│   ├── ecoach-mock-centre/             # Mock compilation, runtime, forecast, post-analysis
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── compiler.rs             # MockSelect formula, blueprint assembly
│   │   │   ├── forecast.rs             # ForecastScore formula, band classification
│   │   │   ├── analyzer.rs             # Post-mock 8-section analysis
│   │   │   └── models.rs               # MockSession, MockBlueprint
│   │   └── Cargo.toml
│   │
│   ├── ecoach-knowledge-gap/           # Gap scoring, repair plans, solidification
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── detector.rs             # Gap detection, priority scoring
│   │   │   ├── repair.rs               # GapRepairPlan generator, repair session
│   │   │   └── models.rs               # GapRepairPlan, SolidificationSession
│   │   └── Cargo.toml
│   │
│   ├── ecoach-goals-calendar/          # Goals, exams, timeline, intensity
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── goals.rs                # Goal CRUD, hierarchy management
│   │   │   ├── calendar.rs             # CalendarEvent, exam dates, preparation phases
│   │   │   └── models.rs               # Goal, CalendarEvent
│   │   └── Cargo.toml
│   │
│   ├── ecoach-intake/                  # Document upload, OCR bridge, question extraction
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── upload.rs               # File upload, mime detection
│   │   │   ├── ocr_bridge.rs           # OCR adapter (Tesseract or cloud)
│   │   │   └── models.rs               # IntakeDocument, ExtractedQuestion
│   │   └── Cargo.toml
│   │
│   ├── ecoach-reporting/               # Readiness, memos, dashboards, parent alerts
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── dashboard.rs            # StudentDashboardService
│   │   │   ├── parent_dashboard.rs     # ParentDashboardService
│   │   │   ├── readiness.rs            # ReadinessEngine, danger zones
│   │   │   └── alerts.rs               # ParentAlertEngine, 9 trigger conditions
│   │   └── Cargo.toml
│   │
│   ├── ecoach-library/                 # Content relationships, shelves, mistake bank
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── library.rs              # LibraryService, shelf management
│   │   │   ├── glossary.rs             # GlossaryService, search, entry clusters
│   │   │   ├── mistake_bank.rs         # MistakeBankService
│   │   │   └── models.rs               # LibraryItem, LibraryShelf, GlossaryEntry
│   │   └── Cargo.toml
│   │
│   ├── ecoach-games/                   # MindStack, Tug of War, Traps
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mindstack.rs            # Tetris-learning engine
│   │   │   ├── tug_of_war.rs           # Rope mechanics, streak power-ups
│   │   │   └── traps.rs                # 5-mode confusion-resolution game
│   │   └── Cargo.toml
│   │
│   ├── ecoach-past-papers/             # Family mining, recurrence analytics
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── miner.rs                # PastPaperMiner, question extraction
│   │   │   └── models.rs               # PastPaper, PaperQuestion
│   │   └── Cargo.toml
│   │
│   ├── ecoach-premium/                 # Strategy engine, 6-layer concierge
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── concierge.rs            # PremiumConcierge, 6 layers
│   │   │   └── alerts.rs               # 9 premium alert conditions
│   │   └── Cargo.toml
│   │
│   ├── ecoach-elite/                   # Elite mode, EPS scoring, tier progression
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── mode.rs                 # EliteMode, 7 pillars, 6 session types
│   │   │   └── eps.rs                  # ElitePerformanceScore computation
│   │   └── Cargo.toml
│   │
│   └── ecoach-commands/                # Tauri command boundary — ONLY frontend interface
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
│   │   └── main.rs                     # Tauri app entry, AppState, register all commands
│   ├── tauri.conf.json
│   └── Cargo.toml
│
└── tests/
    ├── fixtures/                       # Seed data for tests
    ├── integration/                    # Cross-crate integration tests
    └── benchmarks/                     # Performance benchmarks
```

### 1.3 Database Conventions

| Convention | Rule |
|-----------|------|
| Table naming | `snake_case`, prefixed by domain |
| Primary keys | `id INTEGER PRIMARY KEY AUTOINCREMENT` |
| UUIDs | Stored as `TEXT` where cross-device sync anticipated |
| Timestamps | `created_at TEXT NOT NULL DEFAULT (datetime('now'))` — ISO 8601 |
| Scores | `INTEGER NOT NULL DEFAULT 0` — basis points 0-10000 |
| JSON columns | `TEXT NOT NULL DEFAULT '{}'` with application-level validation |
| Foreign keys | `REFERENCES table(id) ON DELETE CASCADE` where appropriate |
| Indexes | On all FKs, on frequently-queried columns |
| WAL mode | `PRAGMA journal_mode = WAL` on every connection open |
| Foreign keys | `PRAGMA foreign_keys = ON` on every connection open |

### 1.4 Event System Design

```rust
/// Every domain event follows this shape — append-only log, no broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub event_id: String,           // UUID v4
    pub event_type: String,         // e.g., "answer.submitted", "mastery.updated"
    pub aggregate_id: String,       // e.g., student_id or session_id
    pub aggregate_type: String,     // e.g., "student", "session"
    pub occurred_at: String,        // ISO 8601
    pub payload: serde_json::Value, // Event-specific data
    pub trace_id: String,           // For correlating cascading events
    pub causation_id: Option<String>, // event_id that caused this event
    pub version: u32,               // Schema version for this event type
}
```

Events are written to `event_log` table (append-only) and processed in-process by registered handlers. No external message broker is ever used.

### 1.5 Two-Plane Architecture

**Learner Runtime Plane** (local SQLite, offline-first):
- Owns learner truth, session execution, diagnostics, planning, memory, coach state
- Runs entirely on the student device
- In-process tokio background jobs for decay, scheduling, alerts
- Source of truth for all learner state

**Content OS / Foundry Plane** (central, content operations):
- OCR/parsing, artifact build, quality gating, pack signing, delivery
- Produces signed offline content packs consumed by Runtime Plane
- Foundry is source of truth for approved curriculum/content artifacts

Both planes are modular monoliths sharing contracts. No microservice mesh.

### 1.6 AppState struct (Tauri)

```rust
// src-tauri/src/main.rs

pub struct AppState {
    pub db: DbPool,
    pub identity_service: IdentityService,
    pub curriculum_service: CurriculumService,
    pub student_model_service: StudentModelService,
    pub diagnostic_engine: DiagnosticEngine,
    pub coach_brain: CoachBrainService,
    pub session_orchestrator: SessionOrchestrator,
    pub mock_service: MockCentreService,
    pub memory_service: MemoryService,
    pub gap_service: KnowledgeGapService,
    pub goals_service: GoalsCalendarService,
    pub reporting_service: ReportingService,
    pub library_service: LibraryService,
    pub glossary_service: GlossaryService,
    pub game_service: GameService,
    pub intake_service: IntakeService,
    pub pack_service: PackService,
    pub content_readiness: ContentReadinessService,
}

pub fn build_app_state(db: DbPool) -> AppState {
    AppState {
        db: db.clone(),
        identity_service: IdentityService::new(db.clone()),
        curriculum_service: CurriculumService::new(db.clone()),
        // ... all services initialized with shared DbPool
    }
}
```

### 1.7 Error Handling Pattern

```rust
/// Unified error type for all Tauri commands
/// Serializes to tagged JSON for frontend
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum CommandError {
    NotFound(String),
    Unauthorized(String),
    ValidationError(String),
    DatabaseError(String),
    BusinessRuleViolation(String),
    ContentNotReady(String),
    StateConflict(String),
    InternalError(String),
}

impl From<sqlx::Error> for CommandError {
    fn from(e: sqlx::Error) -> Self {
        CommandError::DatabaseError(e.to_string())
    }
}

// Domain errors use thiserror
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Account not found: {0}")]
    NotFound(i64),
    #[error("Invalid PIN")]
    InvalidPin,
    #[error("Account locked until {0}")]
    Locked(String),
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
}

// Command converts domain error to CommandError
impl From<IdentityError> for CommandError {
    fn from(e: IdentityError) -> Self {
        match e {
            IdentityError::NotFound(_) => CommandError::NotFound(e.to_string()),
            IdentityError::InvalidPin => CommandError::Unauthorized(e.to_string()),
            IdentityError::Locked(_) => CommandError::BusinessRuleViolation(e.to_string()),
            IdentityError::Db(_) => CommandError::DatabaseError(e.to_string()),
        }
    }
}
```

---

## PART 2A: DATABASE SCHEMA — MIGRATIONS 001 TO 011

### 001_identity.sql

```sql
-- Identity system: accounts, profiles, parent-student links
-- Source: idea3 (full identity spec)

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
    locked_until TEXT,          -- ISO 8601 timestamp or NULL
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'archived')),
    first_run INTEGER NOT NULL DEFAULT 1,  -- 1 = needs onboarding
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_active_at TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE student_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    grade_level TEXT,
    curriculum_track TEXT,
    exam_target TEXT,           -- e.g., 'BECE 2026'
    exam_target_date TEXT,      -- ISO date
    age_band TEXT,
    preferred_subjects TEXT,    -- JSON array of subject IDs
    study_days_per_week INTEGER DEFAULT 5,
    daily_study_budget_minutes INTEGER DEFAULT 60,
    class_level TEXT,
    learning_style TEXT,
    preferred_study_time TEXT,
    diagnostic_complete INTEGER NOT NULL DEFAULT 0,
    plan_generated INTEGER NOT NULL DEFAULT 0,
    onboarding_complete INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE TABLE parent_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    display_preference TEXT DEFAULT 'standard',
    simplified_mode INTEGER NOT NULL DEFAULT 0,
    email TEXT,
    phone TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE TABLE parent_student_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    relationship_label TEXT DEFAULT 'parent',   -- 'parent', 'guardian', 'teacher'
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(parent_account_id, student_account_id)
);

CREATE TABLE admin_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id)
);

CREATE TABLE parent_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    alert_type TEXT NOT NULL CHECK (alert_type IN (
        'inactivity', 'decline', 'exam_near', 'mock_overdue',
        'subject_lagging', 'concept_failing', 'memory_slippage',
        'misconception_recurring', 'false_confidence'
    )),
    message TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'watch'
        CHECK (severity IN ('info', 'watch', 'active', 'urgent', 'critical')),
    is_read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_accounts_type ON accounts(account_type);
CREATE INDEX idx_accounts_status ON accounts(status);
CREATE INDEX idx_student_profiles_account ON student_profiles(account_id);
CREATE INDEX idx_parent_profiles_account ON parent_profiles(account_id);
CREATE INDEX idx_parent_links_parent ON parent_student_links(parent_account_id);
CREATE INDEX idx_parent_links_student ON parent_student_links(student_account_id);
CREATE INDEX idx_parent_alerts_parent ON parent_alerts(parent_account_id);
CREATE INDEX idx_parent_alerts_student ON parent_alerts(student_account_id);
CREATE INDEX idx_parent_alerts_read ON parent_alerts(is_read);
```

### 002_curriculum.sql

```sql
-- Curriculum graph: versions, subjects, topics, academic nodes, edges, misconceptions
-- Source: idea2, idea13, idea22

CREATE TABLE curriculum_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                 -- e.g., "Ghana NaCCA JHS 2024"
    country TEXT NOT NULL DEFAULT 'GH',
    exam_board TEXT,                    -- e.g., 'WAEC'
    education_stage TEXT,               -- e.g., 'JHS'
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
    code TEXT NOT NULL,                 -- 'MATH', 'SCI', 'ENG', 'SOC'
    name TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE topics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    parent_topic_id INTEGER REFERENCES topics(id),   -- for strands/sub-strands
    code TEXT,
    name TEXT NOT NULL,
    description TEXT,
    node_type TEXT NOT NULL DEFAULT 'topic'
        CHECK (node_type IN ('strand', 'sub_strand', 'topic', 'subtopic')),
    display_order INTEGER NOT NULL DEFAULT 0,
    exam_weight INTEGER NOT NULL DEFAULT 5000,       -- basis points
    difficulty_band TEXT DEFAULT 'medium'
        CHECK (difficulty_band IN ('easy', 'medium', 'hard', 'advanced')),
    importance_weight INTEGER NOT NULL DEFAULT 5000,
    is_active INTEGER NOT NULL DEFAULT 1,
    is_exam_critical INTEGER NOT NULL DEFAULT 0,
    estimated_mastery_hours INTEGER DEFAULT 3,
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
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE node_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_node_id INTEGER NOT NULL,
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
    strength_score INTEGER NOT NULL DEFAULT 5000,
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

CREATE INDEX idx_subjects_curriculum ON subjects(curriculum_version_id);
CREATE INDEX idx_topics_subject ON topics(subject_id);
CREATE INDEX idx_topics_parent ON topics(parent_topic_id);
CREATE INDEX idx_topics_active ON topics(is_active);
CREATE INDEX idx_academic_nodes_topic ON academic_nodes(topic_id);
CREATE INDEX idx_academic_nodes_type ON academic_nodes(node_type);
CREATE INDEX idx_node_edges_from ON node_edges(from_node_id, from_node_type);
CREATE INDEX idx_node_edges_to ON node_edges(to_node_id, to_node_type);
CREATE INDEX idx_node_edges_type ON node_edges(edge_type);
CREATE INDEX idx_misconceptions_node ON misconception_patterns(node_id);
CREATE INDEX idx_misconceptions_topic ON misconception_patterns(topic_id);
CREATE INDEX idx_objectives_topic ON learning_objectives(topic_id);
```

### 003_questions.sql

```sql
-- Questions, options, intelligence classification, families, skill links
-- Source: idea9 (60 question types), idea22 (8-axis classification)

CREATE TABLE question_families (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_code TEXT NOT NULL,
    family_name TEXT NOT NULL,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    subtopic_id INTEGER REFERENCES topics(id),
    family_type TEXT NOT NULL DEFAULT 'recurring_pattern'
        CHECK (family_type IN (
            'recurring_pattern', 'worked_example_template',
            'misconception_cluster', 'exam_structure'
        )),
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
    source_ref TEXT,        -- e.g., "BECE 2023 Paper 1 Q5"
    exam_year INTEGER,

    -- Intelligence: 8-axis classification (idea22)
    primary_knowledge_role TEXT,
    primary_cognitive_demand TEXT,
    primary_solve_pattern TEXT,
    primary_pedagogic_function TEXT,
    classification_confidence INTEGER DEFAULT 0,
    intelligence_snapshot TEXT DEFAULT '{}',   -- Full 8-axis JSON

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
    option_label TEXT NOT NULL,     -- 'A', 'B', 'C', 'D'
    option_text TEXT NOT NULL,
    is_correct INTEGER NOT NULL DEFAULT 0,
    misconception_id INTEGER REFERENCES misconception_patterns(id),
    distractor_intent TEXT,         -- Why this wrong answer is tempting
    position INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE question_skill_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    node_id INTEGER NOT NULL REFERENCES academic_nodes(id),
    contribution_weight INTEGER NOT NULL DEFAULT 10000,
    is_primary INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE question_misconception_hits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    misconception_id INTEGER NOT NULL REFERENCES misconception_patterns(id),
    hit_count INTEGER NOT NULL DEFAULT 1,
    last_hit_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, question_id, misconception_id)
);

CREATE INDEX idx_questions_subject ON questions(subject_id);
CREATE INDEX idx_questions_topic ON questions(topic_id);
CREATE INDEX idx_questions_family ON questions(family_id);
CREATE INDEX idx_questions_difficulty ON questions(difficulty_level);
CREATE INDEX idx_questions_active ON questions(is_active);
CREATE INDEX idx_question_options_question ON question_options(question_id);
CREATE INDEX idx_question_skill_links_question ON question_skill_links(question_id);
CREATE INDEX idx_question_skill_links_node ON question_skill_links(node_id);
CREATE INDEX idx_misconception_hits_student ON question_misconception_hits(student_id);
```

### 004_student_state.sql

```sql
-- All student state: mastery, error profiles, attempt history
-- Source: idea6 (mastery formula), idea11 (error types), idea28 (evidence weights)

CREATE TABLE student_topic_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),

    -- Mastery (8-state machine)
    mastery_score INTEGER NOT NULL DEFAULT 0,       -- 0-10000 bp
    mastery_state TEXT NOT NULL DEFAULT 'unseen'
        CHECK (mastery_state IN (
            'unseen', 'exposed', 'emerging', 'partial', 'fragile',
            'stable', 'robust', 'exam_ready'
        )),

    -- 6 Mastery dimensions (all bp)
    accuracy_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    confidence_score INTEGER NOT NULL DEFAULT 0,
    retention_score INTEGER NOT NULL DEFAULT 0,
    transfer_score INTEGER NOT NULL DEFAULT 0,
    consistency_score INTEGER NOT NULL DEFAULT 0,

    -- Gap & Priority
    gap_score INTEGER NOT NULL DEFAULT 10000,       -- 10000 - mastery_score
    priority_score INTEGER NOT NULL DEFAULT 0,

    -- Trend
    trend_state TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_state IN ('improving', 'stable', 'fragile', 'declining', 'critical')),

    -- Fragility
    fragility_score INTEGER NOT NULL DEFAULT 0,
    pressure_collapse_index INTEGER NOT NULL DEFAULT 0,

    -- Evidence counts
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

    -- Versioning
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
    guessing_detected_score INTEGER NOT NULL DEFAULT 0,
    misconception_triggered_score INTEGER NOT NULL DEFAULT 0,

    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_id)
);

CREATE TABLE student_question_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    session_id INTEGER,                 -- FK to sessions (migration 005)
    session_type TEXT,

    -- Attempt data
    attempt_number INTEGER NOT NULL DEFAULT 1,
    started_at TEXT NOT NULL,
    submitted_at TEXT,
    response_time_ms INTEGER,

    -- Response
    selected_option_id INTEGER REFERENCES question_options(id),
    answer_text TEXT,
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

    -- Context flags
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
CREATE INDEX idx_student_topic_states_blocked ON student_topic_states(is_blocked);
CREATE INDEX idx_student_error_profiles ON student_error_profiles(student_id, topic_id);
CREATE INDEX idx_attempts_student ON student_question_attempts(student_id);
CREATE INDEX idx_attempts_question ON student_question_attempts(question_id);
CREATE INDEX idx_attempts_session ON student_question_attempts(session_id);
CREATE INDEX idx_attempts_created ON student_question_attempts(created_at);
CREATE INDEX idx_attempts_correct ON student_question_attempts(student_id, is_correct);
```

### 005_sessions.sql

```sql
-- Session lifecycle and diagnostic instances
-- Source: idea5, idea18, idea34

CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    session_type TEXT NOT NULL
        CHECK (session_type IN (
            'practice', 'diagnostic', 'mock', 'gap_repair', 'memory_review',
            'coach_mission', 'custom_test', 'elite', 'game', 'traps', 'past_papers'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    topic_ids TEXT,                     -- JSON array of topic IDs
    question_ids TEXT,                  -- JSON array of question IDs (ordered)

    -- Configuration
    question_count INTEGER,
    duration_minutes INTEGER,
    is_timed INTEGER NOT NULL DEFAULT 0,
    difficulty_preference TEXT DEFAULT 'adaptive',

    -- State
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'active', 'paused', 'completed', 'abandoned')),
    current_question_index INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    paused_at TEXT,
    completed_at TEXT,

    -- Results (populated on completion)
    total_questions INTEGER NOT NULL DEFAULT 0,
    answered_questions INTEGER NOT NULL DEFAULT 0,
    correct_questions INTEGER NOT NULL DEFAULT 0,
    accuracy_score INTEGER,
    avg_response_time_ms INTEGER,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

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
    result_json TEXT,                   -- Full diagnostic result object
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE diagnostic_session_phases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_id INTEGER NOT NULL REFERENCES diagnostic_instances(id) ON DELETE CASCADE,
    phase_number INTEGER NOT NULL,
    phase_type TEXT NOT NULL
        CHECK (phase_type IN (
            'broad_scan', 'adaptive_zoom', 'condition_testing',
            'stability_recheck', 'confidence_snapshot'
        )),
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
    condition_type TEXT DEFAULT 'normal'
        CHECK (condition_type IN ('normal', 'timed', 'recall', 'recognition', 'transfer', 'stability')),
    sibling_group_id TEXT,
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

CREATE INDEX idx_sessions_student ON sessions(student_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_type ON sessions(session_type);
CREATE INDEX idx_diagnostic_instances_student ON diagnostic_instances(student_id);
CREATE INDEX idx_diagnostic_phases_diagnostic ON diagnostic_session_phases(diagnostic_id);
CREATE INDEX idx_diagnostic_item_attempts_diag ON diagnostic_item_attempts(diagnostic_id);
```

### 006_coach.sql

```sql
-- Coach brain: plans, missions, topic profiles, blockers
-- Source: idea20 (coach state machine), idea5 (session composer)

CREATE TABLE coach_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    exam_target TEXT,
    exam_date TEXT,
    start_date TEXT NOT NULL,
    total_days INTEGER,
    daily_budget_minutes INTEGER NOT NULL DEFAULT 60,
    current_phase TEXT NOT NULL DEFAULT 'foundation'
        CHECK (current_phase IN (
            'foundation', 'strengthening', 'performance', 'consolidation', 'final_revision'
        )),
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
    steps_json TEXT NOT NULL DEFAULT '[]',
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

CREATE TABLE coach_blockers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    reason TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'moderate'
        CHECK (severity IN ('mild', 'moderate', 'severe', 'critical')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
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

CREATE INDEX idx_coach_plans_student ON coach_plans(student_id);
CREATE INDEX idx_coach_plan_days_plan ON coach_plan_days(plan_id);
CREATE INDEX idx_coach_plan_days_date ON coach_plan_days(date);
CREATE INDEX idx_coach_missions_student ON coach_missions(student_id);
CREATE INDEX idx_coach_missions_status ON coach_missions(status);
CREATE INDEX idx_coach_missions_day ON coach_missions(plan_day_id);
CREATE INDEX idx_coach_topic_profiles ON coach_topic_profiles(student_id, topic_id);
CREATE INDEX idx_coach_blockers_student ON coach_blockers(student_id);
CREATE INDEX idx_coach_evidence_student ON coach_session_evidence(student_id);
```

### 007_memory.sql

```sql
-- Memory engine: decay, recall scheduling, interference
-- Source: idea7 (retention), idea32 (decay types), idea27 (memory model)

CREATE TABLE memory_states (
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),

    memory_state TEXT NOT NULL DEFAULT 'not_formed'
        CHECK (memory_state IN (
            'not_formed', 'emerging', 'fragile', 'stable', 'strong', 'slipping', 'collapsed'
        )),
    memory_strength INTEGER NOT NULL DEFAULT 0,     -- 0-10000 bp
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
        CHECK (retrieval_mode IN (
            'recognition', 'cued_recall', 'free_recall',
            'application', 'transfer', 'explanation'
        )),
    is_correct INTEGER NOT NULL,
    response_time_ms INTEGER,
    confidence_level TEXT,
    hint_used INTEGER NOT NULL DEFAULT 0,
    was_timed INTEGER NOT NULL DEFAULT 0,
    was_delayed INTEGER NOT NULL DEFAULT 0,
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
        CHECK (schedule_type IN (
            'initial', 'short_gap', 'medium_gap', 'long_gap',
            'exam_proximity', 'relapse_followup'
        )),
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
        CHECK (status IN (
            'unrelated', 'weak_overlap', 'watchlist', 'active',
            'high_risk', 'severe_confusion', 'stabilized'
        )),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, topic_a_id, topic_b_id)
);

CREATE INDEX idx_memory_states_student ON memory_states(student_id);
CREATE INDEX idx_memory_states_due ON memory_states(next_reactivation_due_at);
CREATE INDEX idx_memory_states_risk ON memory_states(decay_risk DESC);
CREATE INDEX idx_memory_evidence_student ON memory_evidence_events(student_id);
CREATE INDEX idx_recheck_schedules_student ON recheck_schedules(student_id, scheduled_for, status);
CREATE INDEX idx_interference_edges_student ON interference_edges(student_id);
```

### 008_mock_centre.sql

```sql
-- Mock exam compilation, runtime sessions, blueprints, past papers
-- Source: idea1 (mock centre), idea14 (forecast), idea16 (past papers)

CREATE TABLE mock_blueprints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    blueprint_type TEXT NOT NULL DEFAULT 'standard'
        CHECK (blueprint_type IN ('standard', 'forecast_driven', 'diagnostic_driven', 'custom')),
    total_questions INTEGER NOT NULL DEFAULT 40,
    time_limit_minutes INTEGER NOT NULL DEFAULT 60,
    difficulty_distribution TEXT NOT NULL DEFAULT '{}',  -- JSON {easy:%, medium:%, hard:%}
    topic_weights TEXT NOT NULL DEFAULT '{}',            -- JSON {topic_id: weight_bp}
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE mock_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    blueprint_id INTEGER REFERENCES mock_blueprints(id),
    session_id INTEGER REFERENCES sessions(id),
    mock_type TEXT NOT NULL DEFAULT 'standard'
        CHECK (mock_type IN ('forecast', 'diagnostic', 'remediation', 'final_exam', 'shock', 'wisdom')),
    status TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'active', 'paused', 'completed', 'abandoned')),
    started_at TEXT,
    completed_at TEXT,
    raw_score INTEGER,
    percentage_score INTEGER,
    predicted_grade TEXT,
    readiness_movement INTEGER,         -- bp change in readiness after mock
    analysis_json TEXT,                 -- Full 8-section post-mock analysis
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE mock_question_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mock_session_id INTEGER NOT NULL REFERENCES mock_sessions(id),
    question_id INTEGER NOT NULL REFERENCES questions(id),
    display_order INTEGER NOT NULL,
    response_time_ms INTEGER,
    selected_option_id INTEGER REFERENCES question_options(id),
    is_correct INTEGER,
    marks_awarded INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE past_paper_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exam_board TEXT NOT NULL,
    exam_name TEXT NOT NULL,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    exam_year INTEGER NOT NULL,
    exam_month TEXT,
    paper_number INTEGER DEFAULT 1,
    max_marks INTEGER,
    duration_minutes INTEGER,
    status TEXT NOT NULL DEFAULT 'imported'
        CHECK (status IN ('imported', 'classified', 'linked', 'active')),
    source_file TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE past_paper_questions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    past_paper_id INTEGER NOT NULL REFERENCES past_paper_sources(id),
    question_number INTEGER NOT NULL,
    marks INTEGER NOT NULL DEFAULT 1,
    linked_question_id INTEGER REFERENCES questions(id),
    stem TEXT NOT NULL,
    options_json TEXT,
    correct_answer TEXT,
    topic_id INTEGER REFERENCES topics(id),
    difficulty_estimate INTEGER DEFAULT 5000,
    recurrence_count INTEGER NOT NULL DEFAULT 1,    -- How many papers this pattern appears in
    last_appeared_year INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE forecast_blueprints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    forecast_scores_json TEXT NOT NULL DEFAULT '{}',  -- topic_id → ForecastScore
    predicted_exam_score INTEGER,                      -- bp
    confidence_band TEXT,
    high_probability_topics TEXT DEFAULT '[]',        -- JSON arrays
    surprise_risk_topics TEXT DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_mock_sessions_student ON mock_sessions(student_id);
CREATE INDEX idx_mock_sessions_status ON mock_sessions(status);
CREATE INDEX idx_mock_question_log_session ON mock_question_log(mock_session_id);
CREATE INDEX idx_past_paper_sources_subject ON past_paper_sources(subject_id);
CREATE INDEX idx_past_paper_sources_year ON past_paper_sources(exam_year);
CREATE INDEX idx_past_paper_questions_paper ON past_paper_questions(past_paper_id);
CREATE INDEX idx_forecast_blueprints_student ON forecast_blueprints(student_id);
```

### 009_knowledge_gap.sql

```sql
-- Knowledge gap mode: repair plans, solidification sessions
-- Source: idea6 (gap types), idea11 (interventions)

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
        CHECK (reason_code IN (
            'critical_blocker', 'slipping', 'forgetting',
            'exam_critical', 'recurring', 'misconception_family'
        )),
    recommended_mode TEXT NOT NULL
        CHECK (recommended_mode IN (
            'reteach', 'guided_practice', 'transfer_drill',
            'retention_check', 'speed_drill', 'misconception_repair',
            'prerequisite_repair', 'pressure_repair'
        )),
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
        CHECK (session_mode IN (
            'critical_repair', 'slipping_repair', 'retention_repair', 'misconception_repair'
        )),
    status TEXT NOT NULL DEFAULT 'started'
        CHECK (status IN ('started', 'teaching', 'guided', 'proving', 'retesting', 'completed', 'failed')),
    starting_gap_score INTEGER,
    ending_gap_score INTEGER,
    success_flag INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT
);

CREATE TABLE gap_evidence_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    gap_type TEXT NOT NULL CHECK (gap_type IN (
        'content', 'understanding', 'application', 'process', 'speed',
        'accuracy', 'retention', 'transfer', 'interference', 'self_awareness'
    )),
    severity_score INTEGER NOT NULL DEFAULT 5000,
    evidence_source TEXT,               -- 'session', 'diagnostic', 'mock'
    recorded_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_gap_repair_plans_student ON gap_repair_plans(student_id);
CREATE INDEX idx_gap_repair_items_plan ON gap_repair_plan_items(repair_plan_id);
CREATE INDEX idx_solidification_student ON solidification_sessions(student_id);
CREATE INDEX idx_gap_evidence_student ON gap_evidence_log(student_id, topic_id);
```

### 010_goals_calendar.sql

```sql
-- Goals, exam dates, preparation phases, intensity settings
-- Source: idea2 (journey setup), idea25 (time orchestration)

CREATE TABLE exam_goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    exam_name TEXT NOT NULL,            -- e.g., 'BECE 2026'
    exam_date TEXT NOT NULL,
    target_grade TEXT,
    target_score_bp INTEGER,
    subjects_json TEXT NOT NULL DEFAULT '[]',  -- JSON array of subject IDs
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'achieved', 'missed', 'cancelled')),
    goal_level TEXT NOT NULL DEFAULT 'north_star'
        CHECK (goal_level IN ('north_star', 'current_campaign', 'active_tactical', 'background')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE preparation_phases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    goal_id INTEGER NOT NULL REFERENCES exam_goals(id) ON DELETE CASCADE,
    phase_name TEXT NOT NULL
        CHECK (phase_name IN ('build', 'strengthen', 'firm_up', 'wrap_up', 'perform')),
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    intensity_level TEXT NOT NULL DEFAULT 'balanced'
        CHECK (intensity_level IN ('relaxed', 'balanced', 'intense')),
    focus_description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE calendar_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    goal_id INTEGER REFERENCES exam_goals(id),
    event_type TEXT NOT NULL
        CHECK (event_type IN ('exam', 'mock_test', 'class_test', 'revision_day', 'break', 'milestone')),
    event_name TEXT NOT NULL,
    event_date TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    notes TEXT,
    is_confirmed INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE study_intensity_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) UNIQUE,
    path_intensity TEXT NOT NULL DEFAULT 'balanced'
        CHECK (path_intensity IN ('relaxed', 'balanced', 'intense')),
    daily_minutes_target INTEGER NOT NULL DEFAULT 60,
    study_days_per_week INTEGER NOT NULL DEFAULT 5,
    preferred_study_time TEXT,          -- 'morning', 'afternoon', 'evening', 'mixed'
    blackout_dates TEXT DEFAULT '[]',   -- JSON array of ISO dates
    weekly_pattern TEXT DEFAULT '{}',   -- JSON: {mon:60, tue:45, ...}
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE milestone_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    goal_id INTEGER REFERENCES exam_goals(id),
    milestone_type TEXT NOT NULL
        CHECK (milestone_type IN (
            'topic_mastered', 'mock_completed', 'gap_closed',
            'phase_complete', 'readiness_band_upgrade', 'streak_achieved'
        )),
    description TEXT NOT NULL,
    achieved_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_exam_goals_student ON exam_goals(student_id);
CREATE INDEX idx_exam_goals_status ON exam_goals(status);
CREATE INDEX idx_preparation_phases_goal ON preparation_phases(goal_id);
CREATE INDEX idx_calendar_events_student ON calendar_events(student_id);
CREATE INDEX idx_calendar_events_date ON calendar_events(event_date);
CREATE INDEX idx_milestone_records_student ON milestone_records(student_id);
```

### 011_content_packs.sql

```sql
-- Offline content pack registry, install lifecycle, knowledge atoms
-- Source: idea1 (content packs), idea2 (offline-first)

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
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pack_install_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pack_id TEXT NOT NULL REFERENCES content_packs(pack_id),
    action TEXT NOT NULL
        CHECK (action IN (
            'started', 'manifest_verified', 'extracted', 'parsed',
            'inserted_curriculum', 'inserted_questions', 'inserted_content',
            'activated', 'failed', 'rolled_back'
        )),
    status TEXT NOT NULL DEFAULT 'in_progress'
        CHECK (status IN ('in_progress', 'success', 'error')),
    message TEXT,
    step_number INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER,
    error_detail TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE knowledge_atoms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pack_id TEXT NOT NULL REFERENCES content_packs(pack_id),
    concept_id TEXT NOT NULL,
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    atom_type TEXT NOT NULL
        CHECK (atom_type IN (
            'definition', 'formula', 'procedure', 'principle', 'rule',
            'worked_example', 'interpretation', 'vocabulary'
        )),
    title TEXT NOT NULL,
    plain_text TEXT,
    formal_text TEXT,
    latex_form TEXT,
    audio_transcript TEXT,
    related_concepts TEXT DEFAULT '[]',
    difficulty_band TEXT DEFAULT 'medium',
    exam_relevance_score INTEGER NOT NULL DEFAULT 5000,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_content_packs_status ON content_packs(status);
CREATE INDEX idx_content_packs_subject ON content_packs(subject_code);
CREATE INDEX idx_pack_install_log_pack ON pack_install_log(pack_id);
CREATE INDEX idx_knowledge_atoms_pack ON knowledge_atoms(pack_id);
CREATE INDEX idx_knowledge_atoms_topic ON knowledge_atoms(topic_id);
CREATE INDEX idx_knowledge_atoms_type ON knowledge_atoms(atom_type);
```

---

*End of Part 1 (Architecture & Workspace) and Part 2A (Migrations 001–011)*
