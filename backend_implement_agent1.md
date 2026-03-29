# eCoach Backend Implementation Plan — backend_implement_agent1
## The Definitive Synthesis: Architecture, Schema, Types, Formulas, Commands, and Delivery

**Generated:** 2026-03-29  
**Sources:** 38 idea files, detailed_backend_implementation_plan.md, backend_supplement_missing_details.md, backend_implement_agent2.md

---

## TABLE OF CONTENTS

### PART 1: ARCHITECTURE & WORKSPACE
- 1.1 Technology Stack
- 1.2 Cargo Workspace Layout (22 Crates)
- 1.3 Database Conventions
- 1.4 Event System Design
- 1.5 Two-Plane Architecture
- 1.6 AppState and Initialization
- 1.7 Error Handling Pattern

### PART 2: DATABASE SCHEMA — ALL 22 MIGRATIONS

#### PART 2A: Migrations 001–011
- 001_identity.sql — Accounts, student profiles, parent alerts
- 002_curriculum.sql — Topics, academic nodes, edges, misconception patterns
- 003_questions.sql — Questions, options, difficulty axes
- 004_student_state.sql — Mastery dimensions, EMA, trend tracking
- 005_sessions.sql — Learning sessions, diagnostic instances, phases
- 006_coach.sql — Coach plans, missions, topic profiles, blockers
- 007_memory.sql — Memory states, evidence events, recheck schedules
- 008_mock_centre.sql — Mock sessions, past paper sources, forecast blueprints
- 009_knowledge_gap.sql — Gap repair plans, solidification sessions
- 010_goals_calendar.sql — Exam goals, preparation phases, calendar events
- 011_content_packs.sql — Content packs, install log, knowledge atoms

#### PART 2B: Migrations 012–022
- 012_reporting.sql — Readiness claims, proofs, reports, parent snapshots
- 013_glossary.sql — Knowledge entries, FTS5 search, student entry state
- 014_library.sql — Library items, shelves, revision packs, mistake bank
- 015_games.sql — MindStack, Tug of War, Contrast, Traps sessions
- 016_past_papers.sql — Past papers, question families, DNA profiles
- 017_traps.sql — Traps game sessions, items, confusion pairs
- 018_intake.sql — Intake documents, extractions, processing jobs
- 019_premium.sql — Premium profiles, child readiness, interventions
- 020_elite.sql — Elite profiles, sessions, performance dimensions
- 021_event_log.sql — Append-only event log, outbox, consumers
- 022_system.sql — App settings, feature flags, migration history, audit log

### PART 3: RUST TYPE SYSTEM
- 3.1 primitives.rs — BasisPoints, ConfidenceScore, SeverityLevel, TimestampMs
- 3.2 domain_enums.rs — All 13 domain enums incl. 14-state CoachLifecycleState, 12-state MemoryState, 8-state MasteryState
- 3.3 structs.rs — DomainEvent, LearnerState, RiseStudentScores, EvidenceWeight
- 3.4 thresholds.rs — All constants: mastery gates, EMA_ALPHA, session durations, PIN lockout
- 3.5 lib.rs — Module declarations and re-exports

### PART 4: SCORING FORMULAS (16 Functions)
- 4.1 ForecastScore (7 components)
- 4.2 MockOrchestrationScore (8 components)
- 4.3 MockSelectionScore (8 components)
- 4.4 WeaknessScore (7 components)
- 4.5 ReadinessScore + JourneyReadinessScore
- 4.6 PredictedExamScore (blueprint-weighted)
- 4.7 MomentumScore (volume 0.35, accuracy 0.40, pace 0.25)
- 4.8 StrainScore (5 components)
- 4.9 MSI Score (6 components)
- 4.10 update_mastery_ema + compute_evidence_weight
- 4.11 GapPriorityScore (6 components)
- 4.12 MemoryDecay (per-state rates) + memory_promote/demote
- 4.13 compute_full_evidence_weight (hints, guided, transfer, delayed recall)
- 4.14 ElitePerformanceScore (5 configurable dimensions)
- 4.15 rise_stage_gate + rise_next_stage

### PART 5: BUSINESS RULES & CONSTANTS
- 5.1 MasteryBand thresholds
- 5.2 Evidence weighting table
- 5.3 Memory scheduling rules
- 5.4 Session rules
- 5.5 Coach state machine transition table (all 14 states)
- 5.6 Mock orchestration policies (6)
- 5.7 Readiness proof rules (7 checks)
- 5.8 Parent alert triggers (7 conditions)
- 5.9 Game rules (MindStack + Tug of War)
- 5.10 Elite Mode entry gate

### PART 6: TAURI COMMAND API (16 Modules)
- identity, curriculum, question, student, diagnostic, coach, session
- mock, gap, memory, goals, reporting, glossary, library, game, intake, admin

### PART 7: 26-WEEK PHASED DELIVERY PLAN
- Phase 1 (Weeks 1-6): Core infrastructure
- Phase 2 (Weeks 7-13): Learning engine
- Phase 3 (Weeks 14-20): Advanced features
- Phase 4 (Weeks 21-26): Polish and launch

### PART 8: MASTER REFERENCE
- 22-crate responsibility table
- All 6 state machine references
- 60 question types in 8 families
- Post-mock 8 sections
- 7 journey engines
- 4 BECE subjects
- Diagnostic battery spec (3 modes, 7 stages, 12 dimensions)
- Intelligence Constitution engine registry (6 domains)

---

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


---

# eCoach — Agent 1 Part 2: Migrations 012-022 and All Rust Types
## Generated: 2026-03-29

---

# PART 2B: DATABASE SCHEMA — MIGRATIONS 012 TO 022

---

## 012_reporting.sql

```sql
-- 012_reporting.sql
-- Readiness claims, proofs, danger zones, reports, digests, parent snapshots, memos

CREATE TABLE readiness_claims (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    claim_type TEXT NOT NULL DEFAULT 'subject'
        CHECK (claim_type IN ('topic', 'subject', 'full_exam', 'custom_scope')),
    claimed_at TEXT NOT NULL DEFAULT (datetime('now')),
    claim_basis TEXT NOT NULL,                        -- JSON: {mastery_bp, coverage_bp, mock_score_bp, ...}
    readiness_score INTEGER NOT NULL DEFAULT 0,       -- basis points at time of claim
    predicted_exam_score INTEGER,                     -- basis points
    confidence_level INTEGER NOT NULL DEFAULT 0,      -- basis points
    scope_json TEXT NOT NULL DEFAULT '{}',            -- topics or nodes in scope
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'proven', 'failed', 'expired', 'withdrawn')),
    expires_at TEXT,
    proven_at TEXT,
    failed_at TEXT,
    failure_reason TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE readiness_proofs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    claim_id INTEGER NOT NULL REFERENCES readiness_claims(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    proof_type TEXT NOT NULL
        CHECK (proof_type IN (
            'mock_score', 'topic_mastery_gate', 'coverage_gate',
            'retention_check', 'speed_gate', 'consistency_gate',
            'misconception_clearance', 'timed_pressure_pass'
        )),
    proof_value INTEGER NOT NULL DEFAULT 0,           -- basis points
    threshold_required INTEGER NOT NULL DEFAULT 0,    -- basis points
    passed INTEGER NOT NULL DEFAULT 0,
    evidence_session_id INTEGER,                      -- FK to sessions if applicable
    evidence_json TEXT NOT NULL DEFAULT '{}',
    evaluated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE danger_zones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    zone_type TEXT NOT NULL
        CHECK (zone_type IN (
            'critical_gap', 'memory_decay', 'misconception_active',
            'coverage_hole', 'speed_collapse', 'consistency_failure',
            'exam_anxiety', 'prerequisite_block', 'inactivity'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    node_id INTEGER REFERENCES academic_nodes(id),
    severity TEXT NOT NULL DEFAULT 'watch'
        CHECK (severity IN ('watch', 'active', 'urgent', 'critical')),
    severity_score INTEGER NOT NULL DEFAULT 0,        -- basis points
    description TEXT NOT NULL,
    detail_json TEXT NOT NULL DEFAULT '{}',
    first_detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_confirmed_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT,
    resolution_note TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    is_acknowledged INTEGER NOT NULL DEFAULT 0,
    snooze_until TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE readiness_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    report_type TEXT NOT NULL
        CHECK (report_type IN ('weekly', 'checkpoint', 'pre_exam', 'post_mock', 'milestone', 'ad_hoc')),
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    overall_readiness_bp INTEGER NOT NULL DEFAULT 0,
    predicted_score_bp INTEGER,
    coverage_bp INTEGER NOT NULL DEFAULT 0,
    mastery_bp INTEGER NOT NULL DEFAULT 0,
    retention_bp INTEGER NOT NULL DEFAULT 0,
    mock_performance_bp INTEGER,
    speed_bp INTEGER,
    consistency_bp INTEGER,
    trend_direction TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_direction IN ('improving', 'stable', 'declining', 'critical')),
    active_danger_zone_count INTEGER NOT NULL DEFAULT 0,
    critical_gap_count INTEGER NOT NULL DEFAULT 0,
    sessions_completed INTEGER NOT NULL DEFAULT 0,
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    summary_text TEXT,
    coach_notes TEXT,
    report_data_json TEXT NOT NULL DEFAULT '{}',      -- full snapshot
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE weekly_digests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    week_start TEXT NOT NULL,
    week_end TEXT NOT NULL,
    sessions_count INTEGER NOT NULL DEFAULT 0,
    total_study_minutes INTEGER NOT NULL DEFAULT 0,
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    topics_touched INTEGER NOT NULL DEFAULT 0,
    topics_improved INTEGER NOT NULL DEFAULT 0,
    topics_declined INTEGER NOT NULL DEFAULT 0,
    new_mastery_unlocks INTEGER NOT NULL DEFAULT 0,
    streak_days INTEGER NOT NULL DEFAULT 0,
    best_session_score_bp INTEGER,
    worst_subject_id INTEGER REFERENCES subjects(id),
    best_subject_id INTEGER REFERENCES subjects(id),
    danger_zones_opened INTEGER NOT NULL DEFAULT 0,
    danger_zones_resolved INTEGER NOT NULL DEFAULT 0,
    readiness_delta_bp INTEGER NOT NULL DEFAULT 0,    -- change from last week
    highlight_text TEXT,
    digest_data_json TEXT NOT NULL DEFAULT '{}',
    generated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, week_start)
);

CREATE TABLE parent_report_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    snapshot_type TEXT NOT NULL DEFAULT 'weekly'
        CHECK (snapshot_type IN ('weekly', 'monthly', 'term', 'pre_exam', 'alert_triggered')),
    period_label TEXT NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    plain_language_summary TEXT NOT NULL,
    status_signal TEXT NOT NULL DEFAULT 'ok'
        CHECK (status_signal IN ('ok', 'attention', 'urgent', 'critical')),
    readiness_level TEXT NOT NULL DEFAULT 'unknown'
        CHECK (readiness_level IN ('unknown', 'low', 'moderate', 'good', 'exam_ready')),
    recommended_actions TEXT,                         -- JSON array of action strings
    alert_count INTEGER NOT NULL DEFAULT 0,
    snapshot_data_json TEXT NOT NULL DEFAULT '{}',
    delivered_at TEXT,
    viewed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE memo_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    memo_type TEXT NOT NULL
        CHECK (memo_type IN (
            'coach_note', 'milestone', 'parent_alert',
            'system_flag', 'student_note', 'session_summary'
        )),
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    related_entity_type TEXT,                         -- 'topic', 'session', 'mock', etc.
    related_entity_id INTEGER,
    severity TEXT DEFAULT 'info'
        CHECK (severity IN ('info', 'watch', 'active', 'urgent', 'critical')),
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    expires_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_readiness_claims_student ON readiness_claims(student_id);
CREATE INDEX idx_readiness_claims_status ON readiness_claims(status);
CREATE INDEX idx_readiness_proofs_claim ON readiness_proofs(claim_id);
CREATE INDEX idx_readiness_proofs_student ON readiness_proofs(student_id);
CREATE INDEX idx_danger_zones_student ON danger_zones(student_id);
CREATE INDEX idx_danger_zones_active ON danger_zones(is_active, severity);
CREATE INDEX idx_danger_zones_topic ON danger_zones(topic_id);
CREATE INDEX idx_readiness_reports_student ON readiness_reports(student_id);
CREATE INDEX idx_readiness_reports_type ON readiness_reports(report_type);
CREATE INDEX idx_weekly_digests_student ON weekly_digests(student_id);
CREATE INDEX idx_weekly_digests_week ON weekly_digests(week_start);
CREATE INDEX idx_parent_report_snapshots_parent ON parent_report_snapshots(parent_id);
CREATE INDEX idx_parent_report_snapshots_student ON parent_report_snapshots(student_id);
CREATE INDEX idx_memo_records_student ON memo_records(student_id);
CREATE INDEX idx_memo_records_type ON memo_records(memo_type);
```

---

## 013_glossary.sql

```sql
-- 013_glossary.sql
-- Glossary entries, relations, bundles, bundle items, search index, question links, formulas

CREATE TABLE glossary_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_key TEXT NOT NULL UNIQUE,                   -- e.g. 'fraction_numerator'
    term TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    entry_type TEXT NOT NULL DEFAULT 'definition'
        CHECK (entry_type IN (
            'definition', 'concept', 'formula', 'procedure',
            'rule', 'theorem', 'example_term', 'abbreviation'
        )),
    -- Content layers
    plain_definition TEXT NOT NULL,
    extended_explanation TEXT,
    exam_tip TEXT,
    common_mistake TEXT,
    mnemonic TEXT,
    -- Media
    has_diagram INTEGER NOT NULL DEFAULT 0,
    diagram_path TEXT,
    has_audio INTEGER NOT NULL DEFAULT 0,
    audio_path TEXT,
    -- Classification
    cognitive_level TEXT
        CHECK (cognitive_level IN ('recall', 'understanding', 'application', 'analysis')),
    difficulty_level INTEGER NOT NULL DEFAULT 5000,   -- basis points
    grade_levels TEXT NOT NULL DEFAULT '[]',          -- JSON array: ['JHS1','JHS2','JHS3']
    -- Quality
    is_active INTEGER NOT NULL DEFAULT 1,
    is_exam_critical INTEGER NOT NULL DEFAULT 0,
    pack_id TEXT REFERENCES content_packs(pack_id),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE glossary_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    to_entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL
        CHECK (relation_type IN (
            'prerequisite', 'related', 'contrast', 'extends',
            'part_of', 'example_of', 'synonym', 'antonym'
        )),
    strength INTEGER NOT NULL DEFAULT 5000,           -- basis points
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(from_entry_id, to_entry_id, relation_type)
);

CREATE TABLE glossary_bundles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bundle_key TEXT NOT NULL UNIQUE,
    bundle_name TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    bundle_type TEXT NOT NULL DEFAULT 'topic_cluster'
        CHECK (bundle_type IN (
            'topic_cluster', 'exam_vocab', 'formula_set',
            'contrast_group', 'prerequisite_chain', 'custom'
        )),
    description TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    pack_id TEXT REFERENCES content_packs(pack_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE glossary_bundle_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bundle_id INTEGER NOT NULL REFERENCES glossary_bundles(id) ON DELETE CASCADE,
    entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_anchor INTEGER NOT NULL DEFAULT 0,             -- anchor term of the bundle
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(bundle_id, entry_id)
);

CREATE TABLE glossary_search_index (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    search_text TEXT NOT NULL,                        -- flattened searchable text
    tokens TEXT NOT NULL DEFAULT '[]',                -- JSON array of normalised tokens
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- FTS virtual table for glossary full-text search
CREATE VIRTUAL TABLE glossary_fts USING fts5(
    entry_id UNINDEXED,
    term,
    plain_definition,
    extended_explanation,
    exam_tip,
    content='glossary_entries',
    content_rowid='id'
);

CREATE TABLE question_glossary_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    entry_id INTEGER NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    link_type TEXT NOT NULL DEFAULT 'uses_term'
        CHECK (link_type IN (
            'uses_term', 'tests_term', 'requires_definition',
            'distractor_exploits', 'stem_contains'
        )),
    relevance_score INTEGER NOT NULL DEFAULT 5000,    -- basis points
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(question_id, entry_id, link_type)
);

CREATE TABLE formula_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    formula_key TEXT NOT NULL UNIQUE,
    formula_name TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    -- Representations
    latex_expression TEXT NOT NULL,
    plain_text_expression TEXT NOT NULL,
    speech_readable TEXT NOT NULL,                    -- "area equals length times width"
    worked_example TEXT,
    -- Variables
    variables_json TEXT NOT NULL DEFAULT '[]',        -- [{symbol, name, unit, description}]
    -- Classification
    formula_category TEXT NOT NULL DEFAULT 'core'
        CHECK (formula_category IN (
            'core', 'derived', 'shortcut', 'conversion',
            'definition_form', 'rearrangement', 'approximation'
        )),
    is_exam_sheet_formula INTEGER NOT NULL DEFAULT 0, -- appears on BECE formula sheet
    is_must_memorise INTEGER NOT NULL DEFAULT 0,
    difficulty_level INTEGER NOT NULL DEFAULT 5000,
    grade_levels TEXT NOT NULL DEFAULT '[]',
    -- Media
    has_diagram INTEGER NOT NULL DEFAULT 0,
    diagram_path TEXT,
    -- Quality
    is_active INTEGER NOT NULL DEFAULT 1,
    pack_id TEXT REFERENCES content_packs(pack_id),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_glossary_entries_subject ON glossary_entries(subject_id);
CREATE INDEX idx_glossary_entries_topic ON glossary_entries(topic_id);
CREATE INDEX idx_glossary_entries_type ON glossary_entries(entry_type);
CREATE INDEX idx_glossary_entries_active ON glossary_entries(is_active);
CREATE INDEX idx_glossary_relations_from ON glossary_relations(from_entry_id);
CREATE INDEX idx_glossary_relations_to ON glossary_relations(to_entry_id);
CREATE INDEX idx_glossary_bundle_items_bundle ON glossary_bundle_items(bundle_id);
CREATE INDEX idx_glossary_search_entry ON glossary_search_index(entry_id);
CREATE INDEX idx_question_glossary_links_question ON question_glossary_links(question_id);
CREATE INDEX idx_question_glossary_links_entry ON question_glossary_links(entry_id);
CREATE INDEX idx_formula_entries_subject ON formula_entries(subject_id);
CREATE INDEX idx_formula_entries_topic ON formula_entries(topic_id);
```

---

## 014_library.sql

```sql
-- 014_library.sql
-- Library items, shelves, shelf items, mistake bank, memory shelf, revision packs, library cards

CREATE TABLE library_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL
        CHECK (item_type IN (
            'saved_question', 'saved_explanation', 'saved_glossary_entry',
            'saved_formula', 'worked_example', 'coach_note',
            'custom_note', 'mistake_capture', 'highlight'
        )),
    title TEXT NOT NULL,
    body TEXT,
    source_type TEXT,                                 -- 'question', 'glossary', 'formula', etc.
    source_id INTEGER,                                -- FK to source entity
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    tags TEXT NOT NULL DEFAULT '[]',                  -- JSON array of tag strings
    is_starred INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    review_due_at TEXT,
    last_reviewed_at TEXT,
    review_count INTEGER NOT NULL DEFAULT 0,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE library_shelves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    shelf_name TEXT NOT NULL,
    shelf_type TEXT NOT NULL DEFAULT 'custom'
        CHECK (shelf_type IN (
            'custom', 'mistakes', 'memory', 'formulas',
            'glossary', 'revision', 'starred', 'coach_picks'
        )),
    description TEXT,
    color_hex TEXT,
    icon_name TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_system INTEGER NOT NULL DEFAULT 0,             -- 1 = auto-created, not deletable
    is_archived INTEGER NOT NULL DEFAULT 0,
    item_count INTEGER NOT NULL DEFAULT 0,            -- denormalised for display
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, shelf_name)
);

CREATE TABLE shelf_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    shelf_id INTEGER NOT NULL REFERENCES library_shelves(id) ON DELETE CASCADE,
    item_id INTEGER NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    display_order INTEGER NOT NULL DEFAULT 0,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(shelf_id, item_id)
);

CREATE TABLE mistake_bank_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    question_id INTEGER NOT NULL REFERENCES questions(id),
    attempt_id INTEGER REFERENCES student_question_attempts(id),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    -- What went wrong
    error_type TEXT NOT NULL
        CHECK (error_type IN (
            'forgot_concept', 'misunderstood_wording', 'calculation_slip',
            'guessed', 'panic_under_time', 'chose_familiar_distractor',
            'wrong_first_step', 'incomplete_reasoning', 'misread_units',
            'careless_error'
        )),
    selected_option_id INTEGER REFERENCES question_options(id),
    correct_option_id INTEGER REFERENCES question_options(id),
    misconception_id INTEGER REFERENCES misconception_patterns(id),
    student_explanation TEXT,                         -- student's own note about why they got it wrong
    coach_diagnosis TEXT,
    repair_status TEXT NOT NULL DEFAULT 'unresolved'
        CHECK (repair_status IN (
            'unresolved', 'in_progress', 'repaired', 'relapsed', 'mastered'
        )),
    repair_attempts INTEGER NOT NULL DEFAULT 0,
    last_repair_attempt_at TEXT,
    resolved_at TEXT,
    priority_score INTEGER NOT NULL DEFAULT 5000,     -- basis points, higher = revisit sooner
    review_due_at TEXT,
    times_reviewed INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    library_item_id INTEGER REFERENCES library_items(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE memory_shelf_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL
        CHECK (item_type IN (
            'glossary_entry', 'formula', 'concept_summary',
            'mnemonic', 'procedure_steps', 'fact'
        )),
    source_id INTEGER NOT NULL,                       -- FK to glossary_entries or formula_entries
    source_type TEXT NOT NULL,
    title TEXT NOT NULL,
    memory_cue TEXT,                                  -- hint for recall
    memory_state TEXT NOT NULL DEFAULT 'fresh'
        CHECK (memory_state IN (
            'fresh', 'consolidating', 'stable', 'at_risk',
            'decaying', 'critical', 'forgotten', 'rescued',
            'reinforced', 'transferred', 'dormant', 'archived'
        )),
    memory_strength_bp INTEGER NOT NULL DEFAULT 5000,
    last_recall_at TEXT,
    next_review_at TEXT NOT NULL,
    review_interval_days REAL NOT NULL DEFAULT 1.0,
    easiness_factor REAL NOT NULL DEFAULT 2.5,        -- SM-2 / custom spaced rep factor
    consecutive_correct INTEGER NOT NULL DEFAULT 0,
    total_reviews INTEGER NOT NULL DEFAULT 0,
    correct_reviews INTEGER NOT NULL DEFAULT 0,
    decay_rate_per_day REAL NOT NULL DEFAULT 0.1,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE revision_pack_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    pack_name TEXT NOT NULL,
    pack_type TEXT NOT NULL DEFAULT 'custom'
        CHECK (pack_type IN (
            'custom', 'topic_summary', 'exam_countdown',
            'weak_areas', 'coach_generated', 'mistake_compilation'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    item_type TEXT NOT NULL
        CHECK (item_type IN (
            'question', 'glossary_entry', 'formula',
            'concept_note', 'worked_example', 'trap_pair'
        )),
    item_id INTEGER NOT NULL,                         -- FK to respective table
    display_order INTEGER NOT NULL DEFAULT 0,
    is_completed INTEGER NOT NULL DEFAULT 0,
    completed_at TEXT,
    pack_label TEXT NOT NULL DEFAULT 'default',       -- group within a pack
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE library_card_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    total_items INTEGER NOT NULL DEFAULT 0,
    starred_items INTEGER NOT NULL DEFAULT 0,
    mistake_bank_size INTEGER NOT NULL DEFAULT 0,
    resolved_mistakes INTEGER NOT NULL DEFAULT 0,
    memory_shelf_size INTEGER NOT NULL DEFAULT 0,
    due_for_review_count INTEGER NOT NULL DEFAULT 0,
    revision_packs_count INTEGER NOT NULL DEFAULT 0,
    last_library_visit_at TEXT,
    most_visited_shelf_id INTEGER REFERENCES library_shelves(id),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);

CREATE INDEX idx_library_items_student ON library_items(student_id);
CREATE INDEX idx_library_items_type ON library_items(item_type);
CREATE INDEX idx_library_items_topic ON library_items(topic_id);
CREATE INDEX idx_library_shelves_student ON library_shelves(student_id);
CREATE INDEX idx_shelf_items_shelf ON shelf_items(shelf_id);
CREATE INDEX idx_shelf_items_item ON shelf_items(item_id);
CREATE INDEX idx_mistake_bank_student ON mistake_bank_entries(student_id);
CREATE INDEX idx_mistake_bank_question ON mistake_bank_entries(question_id);
CREATE INDEX idx_mistake_bank_status ON mistake_bank_entries(repair_status);
CREATE INDEX idx_mistake_bank_topic ON mistake_bank_entries(topic_id);
CREATE INDEX idx_memory_shelf_student ON memory_shelf_entries(student_id);
CREATE INDEX idx_memory_shelf_state ON memory_shelf_entries(memory_state);
CREATE INDEX idx_memory_shelf_review ON memory_shelf_entries(next_review_at);
CREATE INDEX idx_revision_pack_student ON revision_pack_items(student_id);
```

---

## 015_games.sql

```sql
-- 015_games.sql
-- Game sessions, events, tug-of-war states, mindstack states, leaderboards, rewards

CREATE TABLE game_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    game_type TEXT NOT NULL
        CHECK (game_type IN ('tug_of_war', 'mindstack', 'speed_race', 'beat_yesterday', 'streak_challenge')),
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'completed', 'abandoned', 'timed_out')),
    -- Timing
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    total_duration_ms INTEGER,
    -- Scoring
    initial_score INTEGER NOT NULL DEFAULT 0,
    final_score INTEGER,
    high_score INTEGER,
    score_delta INTEGER,                              -- final - initial
    -- Game-specific state reference
    state_snapshot_json TEXT NOT NULL DEFAULT '{}',
    -- Questions
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    streak_peak INTEGER NOT NULL DEFAULT 0,
    -- Rewards
    xp_earned INTEGER NOT NULL DEFAULT 0,
    badges_earned TEXT NOT NULL DEFAULT '[]',         -- JSON array
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE game_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    event_type TEXT NOT NULL
        CHECK (event_type IN (
            'question_answered', 'streak_gained', 'streak_broken',
            'power_used', 'level_up', 'score_change', 'pause',
            'resume', 'hint_used', 'tug_advance', 'tug_retreat',
            'stack_push', 'stack_collapse', 'time_warning', 'game_over'
        )),
    occurred_at TEXT NOT NULL DEFAULT (datetime('now')),
    question_id INTEGER REFERENCES questions(id),
    is_correct INTEGER,
    response_time_ms INTEGER,
    score_before INTEGER,
    score_after INTEGER,
    streak_before INTEGER,
    streak_after INTEGER,
    event_data_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE tug_of_war_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    -- Rope position: 0 = fully opponent side, 10000 = fully student side (basis points)
    rope_position_bp INTEGER NOT NULL DEFAULT 5000,
    student_pulls INTEGER NOT NULL DEFAULT 0,
    opponent_pulls INTEGER NOT NULL DEFAULT 0,
    current_streak INTEGER NOT NULL DEFAULT 0,
    opponent_streak INTEGER NOT NULL DEFAULT 0,
    -- Opponent model
    opponent_type TEXT NOT NULL DEFAULT 'adaptive_ai'
        CHECK (opponent_type IN ('adaptive_ai', 'historical_self', 'class_average', 'elite_target')),
    opponent_difficulty_bp INTEGER NOT NULL DEFAULT 5000,
    -- Round tracking
    current_round INTEGER NOT NULL DEFAULT 1,
    total_rounds INTEGER NOT NULL DEFAULT 10,
    rounds_won INTEGER NOT NULL DEFAULT 0,
    rounds_lost INTEGER NOT NULL DEFAULT 0,
    -- State
    phase TEXT NOT NULL DEFAULT 'in_progress'
        CHECK (phase IN ('in_progress', 'student_winning', 'opponent_winning', 'tiebreaker', 'complete')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE mindstack_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    -- Stack metaphor: answer correctly to add blocks, wrong = tower wobbles / collapses
    stack_height INTEGER NOT NULL DEFAULT 0,
    max_stack_achieved INTEGER NOT NULL DEFAULT 0,
    wobble_level INTEGER NOT NULL DEFAULT 0,          -- 0-3: 3 = about to collapse
    collapses INTEGER NOT NULL DEFAULT 0,
    -- Difficulty escalation
    current_difficulty_bp INTEGER NOT NULL DEFAULT 3000,
    difficulty_ceiling_bp INTEGER NOT NULL DEFAULT 9500,
    difficulty_step_bp INTEGER NOT NULL DEFAULT 500,
    -- Topics in current stack rotation
    active_topic_ids TEXT NOT NULL DEFAULT '[]',      -- JSON array
    -- State
    phase TEXT NOT NULL DEFAULT 'building'
        CHECK (phase IN ('building', 'wobbling', 'collapsed', 'complete', 'perfect_run')),
    is_perfect_run INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE game_leaderboards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    board_type TEXT NOT NULL
        CHECK (board_type IN ('all_time', 'weekly', 'subject', 'topic', 'game_type')),
    game_type TEXT,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    period_label TEXT,                                -- e.g. '2026-W13'
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    rank INTEGER NOT NULL DEFAULT 0,
    score INTEGER NOT NULL DEFAULT 0,
    sessions_count INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(board_type, game_type, subject_id, topic_id, period_label, student_id)
);

CREATE TABLE game_rewards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    session_id INTEGER REFERENCES game_sessions(id),
    reward_type TEXT NOT NULL
        CHECK (reward_type IN (
            'xp', 'badge', 'streak_shield', 'power_up',
            'title', 'milestone_unlock', 'bonus_hint'
        )),
    reward_key TEXT NOT NULL,                         -- e.g. 'badge_first_tug_win'
    reward_label TEXT NOT NULL,
    reward_value INTEGER NOT NULL DEFAULT 0,          -- XP amount or quantity
    reward_data_json TEXT NOT NULL DEFAULT '{}',
    earned_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_claimed INTEGER NOT NULL DEFAULT 0,
    claimed_at TEXT
);

CREATE INDEX idx_game_sessions_student ON game_sessions(student_id);
CREATE INDEX idx_game_sessions_type ON game_sessions(game_type);
CREATE INDEX idx_game_sessions_status ON game_sessions(status);
CREATE INDEX idx_game_events_session ON game_events(session_id);
CREATE INDEX idx_game_events_student ON game_events(student_id);
CREATE INDEX idx_game_events_type ON game_events(event_type);
CREATE INDEX idx_tug_states_session ON tug_of_war_states(session_id);
CREATE INDEX idx_mindstack_states_session ON mindstack_states(session_id);
CREATE INDEX idx_game_leaderboards_board ON game_leaderboards(board_type, game_type);
CREATE INDEX idx_game_rewards_student ON game_rewards(student_id);
CREATE INDEX idx_game_rewards_type ON game_rewards(reward_type);
```

---

## 016_past_papers.sql

```sql
-- 016_past_papers.sql
-- Past paper sources, parsed questions, recurrence records, examiner patterns, blueprints

CREATE TABLE past_paper_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    paper_code TEXT NOT NULL UNIQUE,                  -- e.g. 'BECE_MATH_2023_P1'
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    exam_board TEXT NOT NULL DEFAULT 'WAEC',
    exam_name TEXT NOT NULL DEFAULT 'BECE',
    exam_year INTEGER NOT NULL,
    paper_number INTEGER NOT NULL DEFAULT 1,
    paper_section TEXT,                               -- 'Objective', 'Essay', 'Theory'
    total_questions INTEGER NOT NULL DEFAULT 0,
    total_marks INTEGER NOT NULL DEFAULT 0,
    time_allowed_minutes INTEGER,
    source_document_path TEXT,
    ocr_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (ocr_status IN ('pending', 'processing', 'complete', 'failed', 'manual')),
    ingestion_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (ingestion_status IN ('pending', 'parsed', 'mapped', 'verified', 'active', 'retired')),
    ingested_at TEXT,
    verified_by TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE paper_questions_parsed (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL REFERENCES past_paper_sources(id) ON DELETE CASCADE,
    question_id INTEGER REFERENCES questions(id),     -- NULL until mapped to question bank
    position_in_paper INTEGER NOT NULL,
    section TEXT,
    raw_stem_text TEXT NOT NULL,
    raw_options_json TEXT,                            -- JSON array of {label, text}
    correct_option_label TEXT,
    marks INTEGER NOT NULL DEFAULT 1,
    -- Mapping metadata
    mapping_status TEXT NOT NULL DEFAULT 'unmapped'
        CHECK (mapping_status IN ('unmapped', 'auto_mapped', 'manual_mapped', 'conflict', 'new_question')),
    mapped_topic_id INTEGER REFERENCES topics(id),
    mapped_node_ids TEXT NOT NULL DEFAULT '[]',       -- JSON array
    mapping_confidence INTEGER NOT NULL DEFAULT 0,    -- basis points
    -- Classification
    difficulty_estimate INTEGER,                      -- basis points
    question_intent TEXT,
    representation_type TEXT
        CHECK (representation_type IN ('text', 'diagram', 'graph', 'table', 'mixed')),
    -- Intelligence
    intel_snapshot_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE question_recurrence_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER REFERENCES questions(id),
    family_id INTEGER REFERENCES question_families(id),
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    node_id INTEGER REFERENCES academic_nodes(id),
    -- Frequency data
    first_seen_year INTEGER NOT NULL,
    last_seen_year INTEGER NOT NULL,
    occurrence_count INTEGER NOT NULL DEFAULT 1,
    years_appeared TEXT NOT NULL DEFAULT '[]',        -- JSON array of years
    papers_appeared TEXT NOT NULL DEFAULT '[]',       -- JSON array of paper_codes
    -- Probability
    recurrence_probability_bp INTEGER NOT NULL DEFAULT 0,
    trend_direction TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_direction IN ('increasing', 'stable', 'decreasing')),
    is_high_frequency INTEGER NOT NULL DEFAULT 0,     -- appeared 3+ times
    is_exam_staple INTEGER NOT NULL DEFAULT 0,        -- appeared 5+ times
    forecast_score_bp INTEGER NOT NULL DEFAULT 0,     -- from forecast formula
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE examiner_pattern_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    pattern_type TEXT NOT NULL
        CHECK (pattern_type IN (
            'topic_rotation', 'difficulty_cycle', 'question_format_preference',
            'section_weight', 'mark_distribution', 'concept_emphasis',
            'style_shift', 'surprise_pattern'
        )),
    description TEXT NOT NULL,
    evidence_years TEXT NOT NULL DEFAULT '[]',        -- JSON array
    strength_bp INTEGER NOT NULL DEFAULT 5000,        -- how reliable is this pattern
    pattern_data_json TEXT NOT NULL DEFAULT '{}',
    is_active INTEGER NOT NULL DEFAULT 1,
    last_validated_year INTEGER,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE blueprint_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    blueprint_version TEXT NOT NULL,
    blueprint_type TEXT NOT NULL DEFAULT 'predicted'
        CHECK (blueprint_type IN ('official', 'predicted', 'historical_average', 'custom')),
    exam_year_target INTEGER,
    -- Topic weights: JSON {topic_id: weight_bp}
    topic_weights_json TEXT NOT NULL DEFAULT '{}',
    -- Difficulty distribution: JSON {easy_bp, medium_bp, hard_bp}
    difficulty_distribution_json TEXT NOT NULL DEFAULT '{}',
    -- Format distribution: JSON {mcq_bp, short_answer_bp, ...}
    format_distribution_json TEXT NOT NULL DEFAULT '{}',
    total_questions INTEGER NOT NULL DEFAULT 0,
    total_marks INTEGER NOT NULL DEFAULT 0,
    confidence_bp INTEGER NOT NULL DEFAULT 5000,
    source_description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_past_paper_sources_subject ON past_paper_sources(subject_id);
CREATE INDEX idx_past_paper_sources_year ON past_paper_sources(exam_year);
CREATE INDEX idx_paper_questions_source ON paper_questions_parsed(source_id);
CREATE INDEX idx_paper_questions_question ON paper_questions_parsed(question_id);
CREATE INDEX idx_paper_questions_mapping ON paper_questions_parsed(mapping_status);
CREATE INDEX idx_question_recurrence_topic ON question_recurrence_records(topic_id);
CREATE INDEX idx_question_recurrence_family ON question_recurrence_records(family_id);
CREATE INDEX idx_question_recurrence_freq ON question_recurrence_records(is_high_frequency);
CREATE INDEX idx_examiner_patterns_subject ON examiner_pattern_records(subject_id);
CREATE INDEX idx_blueprint_records_subject ON blueprint_records(subject_id);
CREATE INDEX idx_blueprint_records_active ON blueprint_records(is_active);
```

---

## 017_traps.sql

```sql
-- 017_traps.sql
-- Contrast pairs, evidence atoms, trap drill sessions, trap attempts, difference mastery states

CREATE TABLE contrast_pairs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_key TEXT NOT NULL UNIQUE,                    -- e.g. 'mean_vs_median'
    concept_a_label TEXT NOT NULL,
    concept_b_label TEXT NOT NULL,
    -- Source entities
    entry_a_id INTEGER REFERENCES glossary_entries(id),
    entry_b_id INTEGER REFERENCES glossary_entries(id),
    node_a_id INTEGER REFERENCES academic_nodes(id),
    node_b_id INTEGER REFERENCES academic_nodes(id),
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    -- Characterisation
    confusion_type TEXT NOT NULL DEFAULT 'conceptual'
        CHECK (confusion_type IN (
            'conceptual', 'definitional', 'procedural',
            'notational', 'contextual', 'surface_similarity'
        )),
    confusion_severity_bp INTEGER NOT NULL DEFAULT 5000,
    -- Content
    key_difference_plain TEXT NOT NULL,
    extended_contrast TEXT,
    exam_tip TEXT,
    -- Frequency
    trap_frequency_bp INTEGER NOT NULL DEFAULT 0,     -- how often this is tested as a trap
    is_exam_critical INTEGER NOT NULL DEFAULT 0,
    -- Quality
    is_active INTEGER NOT NULL DEFAULT 1,
    pack_id TEXT REFERENCES content_packs(pack_id),
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE contrast_evidence_atoms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    atom_type TEXT NOT NULL
        CHECK (atom_type IN (
            'definition_contrast', 'example_contrast', 'formula_contrast',
            'visual_contrast', 'worked_example', 'common_error_scenario',
            'memory_hook', 'exam_question_sample'
        )),
    concept_side TEXT NOT NULL DEFAULT 'both'
        CHECK (concept_side IN ('a', 'b', 'both')),
    content_text TEXT NOT NULL,
    latex_expression TEXT,
    diagram_path TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE trap_drill_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    drill_mode TEXT NOT NULL DEFAULT 'standard'
        CHECK (drill_mode IN ('standard', 'timed', 'boss_set', 'exam_simulation')),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'completed', 'abandoned')),
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    duration_ms INTEGER,
    pairs_attempted INTEGER NOT NULL DEFAULT 0,
    pairs_correct INTEGER NOT NULL DEFAULT 0,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    pairs_targeted TEXT NOT NULL DEFAULT '[]',        -- JSON array of pair_ids
    performance_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE trap_attempt_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES trap_drill_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id),
    question_id INTEGER REFERENCES questions(id),
    attempt_type TEXT NOT NULL DEFAULT 'identify_difference'
        CHECK (attempt_type IN (
            'identify_difference', 'classify_concept',
            'complete_statement', 'select_correct_context', 'match_definition'
        )),
    is_correct INTEGER NOT NULL DEFAULT 0,
    response_time_ms INTEGER,
    selected_answer TEXT,
    correct_answer TEXT,
    hint_used INTEGER NOT NULL DEFAULT 0,
    confidence_level TEXT
        CHECK (confidence_level IN ('sure', 'not_sure', 'guessed')),
    error_note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE difference_mastery_states (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    pair_id INTEGER NOT NULL REFERENCES contrast_pairs(id) ON DELETE CASCADE,
    mastery_bp INTEGER NOT NULL DEFAULT 0,
    confusion_risk_bp INTEGER NOT NULL DEFAULT 10000,
    attempts INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TEXT,
    last_correct_at TEXT,
    last_confused_at TEXT,
    confusion_count INTEGER NOT NULL DEFAULT 0,
    is_resolved INTEGER NOT NULL DEFAULT 0,
    resolved_at TEXT,
    next_review_at TEXT,
    review_interval_days REAL NOT NULL DEFAULT 1.0,
    state TEXT NOT NULL DEFAULT 'unseen'
        CHECK (state IN (
            'unseen', 'confused', 'partially_clear',
            'clear', 'automatic', 'mastered'
        )),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, pair_id)
);

CREATE INDEX idx_contrast_pairs_subject ON contrast_pairs(subject_id);
CREATE INDEX idx_contrast_pairs_topic ON contrast_pairs(topic_id);
CREATE INDEX idx_contrast_pairs_active ON contrast_pairs(is_active);
CREATE INDEX idx_contrast_evidence_pair ON contrast_evidence_atoms(pair_id);
CREATE INDEX idx_trap_drill_sessions_student ON trap_drill_sessions(student_id);
CREATE INDEX idx_trap_attempt_records_session ON trap_attempt_records(session_id);
CREATE INDEX idx_trap_attempt_records_pair ON trap_attempt_records(pair_id);
CREATE INDEX idx_difference_mastery_student ON difference_mastery_states(student_id);
CREATE INDEX idx_difference_mastery_pair ON difference_mastery_states(pair_id);
CREATE INDEX idx_difference_mastery_state ON difference_mastery_states(state);
```

---

## 018_intake.sql

```sql
-- 018_intake.sql
-- Intake documents, pages, segments, question candidates, processing jobs, OCR results

CREATE TABLE intake_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_key TEXT NOT NULL UNIQUE,
    document_name TEXT NOT NULL,
    document_type TEXT NOT NULL
        CHECK (document_type IN (
            'past_paper', 'textbook_chapter', 'curriculum_doc',
            'teacher_upload', 'student_upload', 'reference_material'
        )),
    subject_id INTEGER REFERENCES subjects(id),
    source_description TEXT,
    file_path TEXT NOT NULL,
    file_format TEXT NOT NULL
        CHECK (file_format IN ('pdf', 'jpg', 'png', 'docx', 'txt', 'json')),
    file_size_bytes INTEGER NOT NULL DEFAULT 0,
    file_hash TEXT,
    page_count INTEGER NOT NULL DEFAULT 0,
    -- Provenance
    uploaded_by INTEGER REFERENCES accounts(id),
    upload_source TEXT NOT NULL DEFAULT 'admin'
        CHECK (upload_source IN ('admin', 'teacher', 'student', 'system', 'api')),
    -- Processing
    processing_status TEXT NOT NULL DEFAULT 'queued'
        CHECK (processing_status IN (
            'queued', 'ocr_pending', 'ocr_complete', 'parsing',
            'parsed', 'mapping', 'mapped', 'review_pending',
            'approved', 'rejected', 'failed'
        )),
    error_message TEXT,
    processing_started_at TEXT,
    processing_completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE intake_pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id) ON DELETE CASCADE,
    page_number INTEGER NOT NULL,
    image_path TEXT,
    raw_text TEXT,
    ocr_confidence_bp INTEGER,
    layout_json TEXT,                                 -- detected regions/blocks
    page_type TEXT
        CHECK (page_type IN (
            'questions', 'instructions', 'answer_key', 'blank',
            'diagram_only', 'mixed', 'unknown'
        )),
    processing_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (processing_status IN ('pending', 'ocr_done', 'parsed', 'failed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(document_id, page_number)
);

CREATE TABLE intake_segments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id INTEGER NOT NULL REFERENCES intake_pages(id) ON DELETE CASCADE,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id),
    segment_type TEXT NOT NULL
        CHECK (segment_type IN (
            'question_stem', 'option_a', 'option_b', 'option_c', 'option_d',
            'answer_key_entry', 'section_header', 'instruction',
            'diagram_region', 'table_region', 'formula_region', 'unknown'
        )),
    sequence_order INTEGER NOT NULL DEFAULT 0,
    raw_text TEXT,
    cleaned_text TEXT,
    bounding_box_json TEXT,                           -- {x, y, w, h} in page coords
    confidence_bp INTEGER NOT NULL DEFAULT 0,
    is_formula INTEGER NOT NULL DEFAULT 0,
    has_diagram INTEGER NOT NULL DEFAULT 0,
    linked_question_number INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE intake_question_candidates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id) ON DELETE CASCADE,
    question_number INTEGER NOT NULL,
    stem_text TEXT NOT NULL,
    option_a TEXT,
    option_b TEXT,
    option_c TEXT,
    option_d TEXT,
    correct_option TEXT,
    marks INTEGER NOT NULL DEFAULT 1,
    -- Classification candidates
    candidate_topic_id INTEGER REFERENCES topics(id),
    candidate_difficulty_bp INTEGER,
    candidate_intent TEXT,
    -- Status
    review_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (review_status IN (
            'pending', 'approved', 'rejected',
            'duplicate', 'needs_edit', 'promoted'
        )),
    promoted_question_id INTEGER REFERENCES questions(id),
    reviewer_notes TEXT,
    auto_flags TEXT NOT NULL DEFAULT '[]',            -- JSON array of flag strings
    confidence_bp INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE intake_processing_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id) ON DELETE CASCADE,
    job_type TEXT NOT NULL
        CHECK (job_type IN (
            'ocr', 'layout_parse', 'question_extract', 'topic_map',
            'difficulty_classify', 'quality_check', 'promote_questions'
        )),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'complete', 'failed', 'cancelled')),
    priority INTEGER NOT NULL DEFAULT 5,
    queued_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT,
    duration_ms INTEGER,
    worker_id TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    error_message TEXT,
    result_summary_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE ocr_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES intake_documents(id) ON DELETE CASCADE,
    page_id INTEGER REFERENCES intake_pages(id),
    engine_name TEXT NOT NULL DEFAULT 'tesseract',
    engine_version TEXT,
    overall_confidence_bp INTEGER NOT NULL DEFAULT 0,
    character_count INTEGER NOT NULL DEFAULT 0,
    word_count INTEGER NOT NULL DEFAULT 0,
    detected_language TEXT NOT NULL DEFAULT 'en',
    raw_output_path TEXT,
    structured_output_json TEXT,
    processing_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_intake_documents_status ON intake_documents(processing_status);
CREATE INDEX idx_intake_documents_type ON intake_documents(document_type);
CREATE INDEX idx_intake_pages_document ON intake_pages(document_id);
CREATE INDEX idx_intake_segments_page ON intake_segments(page_id);
CREATE INDEX idx_intake_segments_document ON intake_segments(document_id);
CREATE INDEX idx_intake_candidates_document ON intake_question_candidates(document_id);
CREATE INDEX idx_intake_candidates_status ON intake_question_candidates(review_status);
CREATE INDEX idx_intake_jobs_document ON intake_processing_jobs(document_id);
CREATE INDEX idx_intake_jobs_status ON intake_processing_jobs(status, priority);
CREATE INDEX idx_ocr_results_document ON ocr_results(document_id);
```

---

## 019_premium.sql

```sql
-- 019_premium.sql
-- Premium strategy sessions, concierge actions, strategy documents, premium plans, elite prep

CREATE TABLE premium_strategy_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    session_label TEXT NOT NULL,
    session_type TEXT NOT NULL DEFAULT 'strategy_review'
        CHECK (session_type IN (
            'strategy_review', 'deep_diagnosis', 'exam_plan_build',
            'mock_debrief', 'crisis_intervention', 'milestone_review',
            'parent_brief', 'subject_deep_dive'
        )),
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'completed', 'archived')),
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    -- Context snapshot at session start
    readiness_bp_at_start INTEGER,
    days_to_exam_at_start INTEGER,
    active_danger_zones_at_start INTEGER,
    -- Outputs
    strategy_document_id INTEGER,                     -- FK set after creation
    action_count INTEGER NOT NULL DEFAULT 0,
    actions_completed INTEGER NOT NULL DEFAULT 0,
    coach_summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE premium_concierge_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES premium_strategy_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    action_type TEXT NOT NULL
        CHECK (action_type IN (
            'assign_topic_repair', 'schedule_mock', 'unlock_elite_drill',
            'flag_danger_zone', 'generate_revision_pack', 'send_parent_alert',
            'adjust_daily_plan', 'override_session_type', 'set_exam_countdown',
            'create_custom_mission', 'escalate_to_crisis'
        )),
    priority INTEGER NOT NULL DEFAULT 5,              -- 1 = highest
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    target_entity_type TEXT,                          -- 'topic', 'session', 'plan', etc.
    target_entity_id INTEGER,
    parameters_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'in_progress', 'completed', 'skipped', 'failed')),
    due_by TEXT,
    completed_at TEXT,
    result_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE strategy_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES premium_strategy_sessions(id) ON DELETE CASCADE,
    student_id INTEGER NOT NULL REFERENCES accounts(id),
    document_type TEXT NOT NULL DEFAULT 'strategy_brief'
        CHECK (document_type IN (
            'strategy_brief', 'exam_game_plan', 'crisis_plan',
            'parent_summary', 'milestone_report', 'subject_breakdown'
        )),
    title TEXT NOT NULL,
    -- Sections stored as JSON: [{heading, body, priority}]
    sections_json TEXT NOT NULL DEFAULT '[]',
    plain_text_summary TEXT,
    readiness_snapshot_json TEXT NOT NULL DEFAULT '{}',
    action_items_json TEXT NOT NULL DEFAULT '[]',
    is_finalized INTEGER NOT NULL DEFAULT 0,
    finalized_at TEXT,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Add FK back-reference in premium_strategy_sessions
-- (SQLite doesn't support ALTER TABLE ADD CONSTRAINT, so handled in application layer)

CREATE TABLE premium_plan_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    plan_type TEXT NOT NULL DEFAULT 'weekly'
        CHECK (plan_type IN ('daily', 'weekly', 'exam_countdown', 'subject_blitz', 'custom')),
    label TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('draft', 'active', 'completed', 'superseded', 'cancelled')),
    starts_on TEXT NOT NULL,
    ends_on TEXT,
    -- Goals
    readiness_target_bp INTEGER,
    coverage_target_bp INTEGER,
    sessions_target INTEGER,
    mocks_target INTEGER,
    -- Actuals (updated as plan progresses)
    sessions_completed INTEGER NOT NULL DEFAULT 0,
    mocks_completed INTEGER NOT NULL DEFAULT 0,
    readiness_achieved_bp INTEGER,
    -- Plan definition
    plan_data_json TEXT NOT NULL DEFAULT '{}',
    generated_by TEXT NOT NULL DEFAULT 'coach'
        CHECK (generated_by IN ('coach', 'premium_concierge', 'admin', 'student')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE elite_prep_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    -- Elite progression
    current_tier TEXT NOT NULL DEFAULT 'foundation'
        CHECK (current_tier IN ('foundation', 'core', 'apex', 'legend')),
    tier_unlocked_at TEXT,
    -- Pillar scores (all basis points)
    precision_bp INTEGER NOT NULL DEFAULT 0,
    speed_bp INTEGER NOT NULL DEFAULT 0,
    depth_bp INTEGER NOT NULL DEFAULT 0,
    endurance_bp INTEGER NOT NULL DEFAULT 0,
    trap_sense_bp INTEGER NOT NULL DEFAULT 0,
    elegant_solve_bp INTEGER NOT NULL DEFAULT 0,
    pressure_bp INTEGER NOT NULL DEFAULT 0,
    -- Overall
    elite_composite_bp INTEGER NOT NULL DEFAULT 0,
    -- Sessions
    elite_sessions_count INTEGER NOT NULL DEFAULT 0,
    perfect_runs_count INTEGER NOT NULL DEFAULT 0,
    longest_clean_streak INTEGER NOT NULL DEFAULT 0,
    current_clean_streak INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id, subject_id)
);

CREATE INDEX idx_premium_strategy_sessions_student ON premium_strategy_sessions(student_id);
CREATE INDEX idx_premium_concierge_actions_session ON premium_concierge_actions(session_id);
CREATE INDEX idx_premium_concierge_actions_status ON premium_concierge_actions(status);
CREATE INDEX idx_strategy_documents_session ON strategy_documents(session_id);
CREATE INDEX idx_premium_plan_records_student ON premium_plan_records(student_id);
CREATE INDEX idx_premium_plan_records_status ON premium_plan_records(status);
CREATE INDEX idx_elite_prep_records_student ON elite_prep_records(student_id);
```

---

## 020_elite.sql

```sql
-- 020_elite.sql
-- Elite scores, performance snapshots, snapshot components, missions, benchmarks

CREATE TABLE elite_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    session_id INTEGER,                               -- FK to sessions if from a session
    score_type TEXT NOT NULL
        CHECK (score_type IN (
            'sprint', 'gauntlet', 'precision_lab', 'depth_lab',
            'perfect_run', 'endurance_track', 'apex_mock'
        )),
    -- Raw performance
    questions_attempted INTEGER NOT NULL DEFAULT 0,
    questions_correct INTEGER NOT NULL DEFAULT 0,
    accuracy_bp INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms INTEGER,
    fastest_correct_ms INTEGER,
    -- Elite metrics
    precision_score_bp INTEGER NOT NULL DEFAULT 0,
    speed_score_bp INTEGER NOT NULL DEFAULT 0,
    depth_score_bp INTEGER NOT NULL DEFAULT 0,
    endurance_score_bp INTEGER NOT NULL DEFAULT 0,
    trap_sense_score_bp INTEGER NOT NULL DEFAULT 0,
    pressure_score_bp INTEGER NOT NULL DEFAULT 0,
    composite_elite_bp INTEGER NOT NULL DEFAULT 0,
    -- Run integrity
    is_perfect_run INTEGER NOT NULL DEFAULT 0,
    hints_used INTEGER NOT NULL DEFAULT 0,
    errors_count INTEGER NOT NULL DEFAULT 0,
    -- Percentile (vs historical self and targets)
    personal_best_flag INTEGER NOT NULL DEFAULT 0,
    personal_best_prev_bp INTEGER,
    scored_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE elite_performance_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    snapshot_label TEXT NOT NULL,
    snapshot_period TEXT NOT NULL DEFAULT 'session'
        CHECK (snapshot_period IN ('session', 'daily', 'weekly', 'monthly', 'milestone')),
    -- Aggregate pillars
    precision_avg_bp INTEGER NOT NULL DEFAULT 0,
    speed_avg_bp INTEGER NOT NULL DEFAULT 0,
    depth_avg_bp INTEGER NOT NULL DEFAULT 0,
    endurance_avg_bp INTEGER NOT NULL DEFAULT 0,
    trap_sense_avg_bp INTEGER NOT NULL DEFAULT 0,
    elegant_solve_avg_bp INTEGER NOT NULL DEFAULT 0,
    pressure_avg_bp INTEGER NOT NULL DEFAULT 0,
    composite_avg_bp INTEGER NOT NULL DEFAULT 0,
    -- Progress
    sessions_in_period INTEGER NOT NULL DEFAULT 0,
    perfect_runs_in_period INTEGER NOT NULL DEFAULT 0,
    questions_in_period INTEGER NOT NULL DEFAULT 0,
    accuracy_in_period_bp INTEGER NOT NULL DEFAULT 0,
    trend_direction TEXT NOT NULL DEFAULT 'stable'
        CHECK (trend_direction IN ('improving', 'stable', 'declining')),
    snapped_at TEXT NOT NULL DEFAULT (datetime('now')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE eps_components (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    snapshot_id INTEGER NOT NULL REFERENCES elite_performance_snapshots(id) ON DELETE CASCADE,
    pillar TEXT NOT NULL
        CHECK (pillar IN (
            'precision', 'speed', 'depth', 'endurance',
            'trap_sense', 'elegant_solve', 'pressure'
        )),
    score_bp INTEGER NOT NULL DEFAULT 0,
    delta_from_previous_bp INTEGER NOT NULL DEFAULT 0,
    percentile_rank INTEGER,                          -- 0-100 vs historical self
    contributing_sessions INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE elite_missions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER REFERENCES subjects(id),
    mission_type TEXT NOT NULL
        CHECK (mission_type IN (
            'hit_accuracy_target', 'achieve_perfect_run', 'beat_personal_best',
            'clear_trap_cluster', 'sustain_speed_under_pressure', 'complete_gauntlet',
            'reach_tier', 'master_elite_set', 'custom'
        )),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    -- Targets
    target_metric TEXT NOT NULL,                      -- e.g. 'precision_bp'
    target_value INTEGER NOT NULL,                    -- basis points
    current_value INTEGER NOT NULL DEFAULT 0,
    -- Status
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'completed', 'failed', 'expired', 'cancelled')),
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    deadline TEXT,
    completed_at TEXT,
    -- Reward
    xp_reward INTEGER NOT NULL DEFAULT 0,
    badge_reward TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE elite_benchmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    tier TEXT NOT NULL
        CHECK (tier IN ('foundation', 'core', 'apex', 'legend')),
    pillar TEXT NOT NULL
        CHECK (pillar IN (
            'precision', 'speed', 'depth', 'endurance',
            'trap_sense', 'elegant_solve', 'pressure', 'composite'
        )),
    benchmark_bp INTEGER NOT NULL,                    -- minimum score to be at this tier
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(subject_id, tier, pillar)
);

CREATE INDEX idx_elite_scores_student ON elite_scores(student_id);
CREATE INDEX idx_elite_scores_subject ON elite_scores(subject_id);
CREATE INDEX idx_elite_scores_type ON elite_scores(score_type);
CREATE INDEX idx_elite_perf_snapshots_student ON elite_performance_snapshots(student_id);
CREATE INDEX idx_eps_components_snapshot ON eps_components(snapshot_id);
CREATE INDEX idx_elite_missions_student ON elite_missions(student_id);
CREATE INDEX idx_elite_missions_status ON elite_missions(status);
CREATE INDEX idx_elite_benchmarks_subject ON elite_benchmarks(subject_id, tier);
```

---

## 021_event_log.sql

```sql
-- 021_event_log.sql
-- Append-only event sourcing table, outbox events, event consumers, domain event types

-- Core append-only event log: never UPDATE or DELETE rows in this table
CREATE TABLE event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL UNIQUE,                    -- UUID v4
    event_type TEXT NOT NULL,                         -- e.g. 'student.attempt.recorded'
    aggregate_type TEXT NOT NULL,                     -- 'student', 'session', 'topic_state', etc.
    aggregate_id TEXT NOT NULL,                       -- stringified entity id
    student_id INTEGER REFERENCES accounts(id),       -- denormalised for fast per-student replay
    sequence_number INTEGER NOT NULL DEFAULT 0,       -- monotonic per aggregate
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    recorded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- Payload
    payload TEXT NOT NULL DEFAULT '{}',               -- JSON event body
    -- Causality
    causation_id TEXT,                                -- event_id that caused this event
    correlation_id TEXT,                              -- trace / session correlation
    -- Schema
    schema_version INTEGER NOT NULL DEFAULT 1,
    -- Processing
    is_processed INTEGER NOT NULL DEFAULT 0,
    processed_at TEXT,
    -- Integrity: no UPDATE or DELETE allowed; enforced at application layer
    CHECK (json_valid(payload))
);

-- Transactional outbox for cross-boundary delivery
CREATE TABLE outbox_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_log_id INTEGER NOT NULL REFERENCES event_log(id),
    event_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    destination TEXT NOT NULL,                        -- consumer name or 'broadcast'
    payload TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processing', 'delivered', 'failed', 'dead_lettered')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 5,
    last_attempt_at TEXT,
    next_attempt_at TEXT,
    delivered_at TEXT,
    error_message TEXT
);

CREATE TABLE event_consumers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    consumer_name TEXT NOT NULL UNIQUE,
    consumer_description TEXT,
    subscribed_event_types TEXT NOT NULL DEFAULT '[]', -- JSON array of event type patterns
    last_processed_event_id INTEGER REFERENCES event_log(id),
    last_processed_at TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE domain_event_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL UNIQUE,                  -- canonical name
    aggregate_type TEXT NOT NULL,
    description TEXT NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1,
    payload_schema_json TEXT NOT NULL DEFAULT '{}',   -- JSON Schema for payload validation
    is_active INTEGER NOT NULL DEFAULT 1,
    is_deprecated INTEGER NOT NULL DEFAULT 0,
    deprecated_at TEXT,
    superseded_by TEXT,                               -- event_type that replaces this
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_event_log_event_id ON event_log(event_id);
CREATE INDEX idx_event_log_aggregate ON event_log(aggregate_type, aggregate_id);
CREATE INDEX idx_event_log_student ON event_log(student_id);
CREATE INDEX idx_event_log_type ON event_log(event_type);
CREATE INDEX idx_event_log_occurred ON event_log(occurred_at);
CREATE INDEX idx_event_log_processed ON event_log(is_processed);
CREATE INDEX idx_outbox_events_status ON outbox_events(status, next_attempt_at);
CREATE INDEX idx_outbox_events_event ON outbox_events(event_log_id);
CREATE INDEX idx_domain_event_types_aggregate ON domain_event_types(aggregate_type);
```

---

## 022_system.sql

```sql
-- 022_system.sql
-- App settings, feature flags, migration history, sync state, background jobs, audit log

CREATE TABLE app_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    setting_key TEXT NOT NULL UNIQUE,
    setting_value TEXT NOT NULL,
    value_type TEXT NOT NULL DEFAULT 'string'
        CHECK (value_type IN ('string', 'integer', 'float', 'boolean', 'json')),
    category TEXT NOT NULL DEFAULT 'general'
        CHECK (category IN (
            'general', 'coach', 'ui', 'session', 'memory',
            'parent', 'notification', 'security', 'content', 'debug'
        )),
    description TEXT,
    is_user_configurable INTEGER NOT NULL DEFAULT 0,
    is_secret INTEGER NOT NULL DEFAULT 0,
    min_value TEXT,
    max_value TEXT,
    default_value TEXT NOT NULL,
    last_changed_at TEXT NOT NULL DEFAULT (datetime('now')),
    changed_by INTEGER REFERENCES accounts(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE feature_flags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    flag_key TEXT NOT NULL UNIQUE,                    -- e.g. 'elite_mode_enabled'
    flag_name TEXT NOT NULL,
    description TEXT,
    is_enabled INTEGER NOT NULL DEFAULT 0,
    -- Targeting
    enabled_for_tiers TEXT NOT NULL DEFAULT '[]',     -- JSON: ['Standard','Premium','Elite']
    enabled_for_accounts TEXT NOT NULL DEFAULT '[]',  -- JSON: specific account IDs (override)
    rollout_percentage INTEGER NOT NULL DEFAULT 100,  -- 0-100
    -- Lifecycle
    category TEXT NOT NULL DEFAULT 'feature'
        CHECK (category IN ('feature', 'experiment', 'kill_switch', 'debug', 'migration')),
    expires_at TEXT,
    is_permanent INTEGER NOT NULL DEFAULT 0,
    changed_at TEXT NOT NULL DEFAULT (datetime('now')),
    changed_by INTEGER REFERENCES accounts(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE migration_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    migration_name TEXT NOT NULL UNIQUE,              -- e.g. '001_accounts.sql'
    migration_number INTEGER NOT NULL UNIQUE,
    checksum TEXT NOT NULL,                           -- SHA-256 of migration SQL
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    duration_ms INTEGER,
    applied_by TEXT NOT NULL DEFAULT 'system',
    rollback_sql TEXT,                                -- optional rollback script
    notes TEXT
);

CREATE TABLE sync_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sync_key TEXT NOT NULL UNIQUE,                    -- e.g. 'content_pack_sync', 'account_sync'
    sync_type TEXT NOT NULL
        CHECK (sync_type IN (
            'content_pack', 'account', 'settings',
            'feature_flags', 'analytics_upload', 'backup'
        )),
    last_sync_at TEXT,
    last_success_at TEXT,
    last_failure_at TEXT,
    last_error TEXT,
    sync_status TEXT NOT NULL DEFAULT 'idle'
        CHECK (sync_status IN ('idle', 'running', 'success', 'failed', 'paused')),
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    -- For incremental sync
    last_synced_cursor TEXT,                          -- timestamp or sequence number
    bytes_transferred INTEGER NOT NULL DEFAULT 0,
    records_synced INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE background_job_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_key TEXT NOT NULL,                            -- e.g. 'rebuild_fts_index'
    job_type TEXT NOT NULL
        CHECK (job_type IN (
            'index_rebuild', 'state_recompute', 'memory_decay_update',
            'report_generation', 'pack_install', 'ocr_process',
            'cleanup', 'export', 'snapshot', 'analytics'
        )),
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'complete', 'failed', 'cancelled', 'skipped')),
    priority INTEGER NOT NULL DEFAULT 5,              -- 1 = highest
    scheduled_for TEXT,
    queued_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT,
    duration_ms INTEGER,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    last_error TEXT,
    parameters_json TEXT NOT NULL DEFAULT '{}',
    result_json TEXT NOT NULL DEFAULT '{}',
    triggered_by TEXT NOT NULL DEFAULT 'system'       -- 'system', 'user', 'schedule', 'event'
);

CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Who
    account_id INTEGER REFERENCES accounts(id),
    account_role TEXT,
    session_token_hash TEXT,
    -- What
    action TEXT NOT NULL,                             -- e.g. 'pin_changed', 'pack_installed'
    entity_type TEXT,
    entity_id TEXT,
    -- Detail
    old_value_json TEXT,
    new_value_json TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    -- Where / When
    occurred_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    ip_address TEXT,
    -- Integrity: rows should never be updated or deleted
    CHECK (json_valid(metadata_json))
);

CREATE INDEX idx_app_settings_category ON app_settings(category);
CREATE INDEX idx_feature_flags_enabled ON feature_flags(is_enabled);
CREATE INDEX idx_migration_history_number ON migration_history(migration_number);
CREATE INDEX idx_background_jobs_status ON background_job_records(status, priority);
CREATE INDEX idx_background_jobs_type ON background_job_records(job_type);
CREATE INDEX idx_audit_log_account ON audit_log(account_id);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_occurred ON audit_log(occurred_at);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id);
```

---

# PART 3: ALL RUST TYPES & ENUMS

All types live in `ecoach-substrate/src/`. The crate is `no_std`-compatible except where noted.
Add to `Cargo.toml`: `serde = { version = "1", features = ["derive"] }`.

---

## 3.1 Primitive Types

```rust
// ecoach-substrate/src/primitives.rs

use serde::{Deserialize, Serialize};

// ─── BasisPoints ─────────────────────────────────────────────────────────────

/// A fixed-point representation of a proportion in the range [0, 10_000].
/// 10_000 bp == 100 %. Used throughout for scores, weights, and thresholds.
pub type BasisPoints = u16;

/// Convert a floating-point proportion (0.0–1.0) to BasisPoints.
/// Values outside range are clamped.
#[inline]
pub fn to_bp(value: f64) -> BasisPoints {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as BasisPoints
}

/// Convert BasisPoints back to a floating-point proportion (0.0–1.0).
#[inline]
pub fn from_bp(bp: BasisPoints) -> f64 {
    bp as f64 / 10_000.0
}

// ─── ConfidenceScore ─────────────────────────────────────────────────────────

/// A learner's self-reported or inferred confidence, 0–100 (inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ConfidenceScore(u8);

impl ConfidenceScore {
    pub const MIN: ConfidenceScore = ConfidenceScore(0);
    pub const MAX: ConfidenceScore = ConfidenceScore(100);

    /// Create a ConfidenceScore, clamping to [0, 100].
    pub fn new(value: u8) -> Self {
        ConfidenceScore(value.min(100))
    }

    pub fn value(self) -> u8 {
        self.0
    }

    pub fn as_bp(self) -> BasisPoints {
        (self.0 as u16) * 100
    }
}

impl Default for ConfidenceScore {
    fn default() -> Self {
        ConfidenceScore(50)
    }
}

// ─── SeverityLevel ───────────────────────────────────────────────────────────

/// Severity classification used for danger zones, alerts, and issue escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeverityLevel {
    /// Below watch threshold — informational only.
    Low,
    /// Approaching a problem threshold — monitor.
    Watch,
    /// Confirmed problem — intervention warranted.
    Active,
    /// Serious, time-sensitive — prioritise immediately.
    Urgent,
    /// Exam-threatening — all-hands response required.
    Critical,
}

impl SeverityLevel {
    /// Minimum BasisPoints score that classifies as this level.
    pub const fn threshold_bp(self) -> BasisPoints {
        match self {
            SeverityLevel::Low => 0,
            SeverityLevel::Watch => 3_000,
            SeverityLevel::Active => 5_000,
            SeverityLevel::Urgent => 7_000,
            SeverityLevel::Critical => 8_500,
        }
    }

    /// Classify a danger score (basis points) into a SeverityLevel.
    pub fn from_score(score_bp: BasisPoints) -> Self {
        if score_bp >= SeverityLevel::Critical.threshold_bp() {
            SeverityLevel::Critical
        } else if score_bp >= SeverityLevel::Urgent.threshold_bp() {
            SeverityLevel::Urgent
        } else if score_bp >= SeverityLevel::Active.threshold_bp() {
            SeverityLevel::Active
        } else if score_bp >= SeverityLevel::Watch.threshold_bp() {
            SeverityLevel::Watch
        } else {
            SeverityLevel::Low
        }
    }
}

// ─── TrendDirection ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    /// Score is falling sharply and below a critical threshold.
    Critical,
}

// ─── Timestamps ──────────────────────────────────────────────────────────────

/// Unix epoch timestamp in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimestampMs(pub i64);

impl TimestampMs {
    pub fn value(self) -> i64 {
        self.0
    }
}

/// A duration expressed in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DurationMs(pub i64);

impl DurationMs {
    pub fn value(self) -> i64 {
        self.0
    }

    pub fn as_seconds(self) -> f64 {
        self.0 as f64 / 1_000.0
    }

    pub fn as_minutes(self) -> f64 {
        self.0 as f64 / 60_000.0
    }
}
```

---

## 3.2 Domain Enums

```rust
// ecoach-substrate/src/domain_enums.rs

use serde::{Deserialize, Serialize};

// ─── Role ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Student,
    Parent,
    Admin,
}

// ─── EntitlementTier ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntitlementTier {
    Standard,
    Premium,
    Elite,
}

impl EntitlementTier {
    pub fn includes_premium(self) -> bool {
        matches!(self, EntitlementTier::Premium | EntitlementTier::Elite)
    }

    pub fn includes_elite(self) -> bool {
        matches!(self, EntitlementTier::Elite)
    }
}

// ─── CoachLifecycleState ─────────────────────────────────────────────────────

/// The 14-state machine governing what the coach is doing for a given student.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoachLifecycleState {
    /// No active student — waiting for account selection.
    Dormant,
    /// Running initial calibration diagnostics.
    Calibrating,
    /// Setting the baseline mastery/coverage map after calibration.
    BaselineSetting,
    /// Building the study mission and weekly plan.
    MissionPlanning,
    /// Normal daily coaching in progress.
    ActiveTeaching,
    /// Executing targeted remediation for identified gaps.
    GapRepair,
    /// Running memory rescue scheduling (spaced-repetition emergency).
    MemoryRescue,
    /// Composing and running mock exams.
    MockOrchestrating,
    /// Collecting and verifying readiness proofs.
    ReadinessProofing,
    /// Emergency intervention: exam-threatening risk detected.
    CrisisIntervention,
    /// Final countdown mode: ≤ 7 days to exam.
    ExamCountdown,
    /// Post-exam analysis and debrief.
    PostExamReview,
    /// Generating and delivering parent briefing.
    ParentalBriefing,
    /// Background maintenance: index rebuild, state recompute, etc.
    SystemMaintenance,
}

// ─── MemoryState ─────────────────────────────────────────────────────────────

/// 12-state memory lifecycle for a learner's recall of a specific knowledge item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryState {
    /// Newly learned; not yet consolidated.
    Fresh,
    /// In the consolidation window (first 24-48 h).
    Consolidating,
    /// Well-reviewed; memory is solid.
    Stable,
    /// Overdue for review; at risk of decay.
    AtRisk,
    /// Noticeably weakened — recall slower or error-prone.
    Decaying,
    /// Below rescue threshold; near-forgotten.
    Critical,
    /// Measured as forgotten; needs reteaching.
    Forgotten,
    /// Was forgotten but successfully recalled after rescue session.
    Rescued,
    /// Deliberately reinforced above baseline.
    Reinforced,
    /// Knowledge transferred to novel contexts.
    Transferred,
    /// Intentionally parked (not in active rotation).
    Dormant,
    /// Removed from active review cycle (mastered long-term or out of scope).
    Archived,
}

// ─── MasteryState ────────────────────────────────────────────────────────────

/// 8-state mastery chain for a skill or topic.
/// States are ordered — a learner must pass through them sequentially.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MasteryState {
    /// Never attempted.
    Untested,
    /// First exposures — patterns beginning to appear.
    Emerging,
    /// Can answer sometimes but inconsistently.
    Unstable,
    /// Reliably correct in standard conditions.
    Functional,
    /// Consistent across multiple sessions and question types.
    Stable,
    /// Correct in novel/transfer contexts.
    Transferable,
    /// Performing at exam-passing standards under timed conditions.
    ExamReady,
    /// Comprehensive, automatic, durable knowledge.
    Mastered,
}

impl MasteryState {
    /// Minimum BasisPoints score required to be at this state.
    pub const fn min_score_bp(self) -> crate::primitives::BasisPoints {
        match self {
            MasteryState::Untested => 0,
            MasteryState::Emerging => 1_000,
            MasteryState::Unstable => 2_500,
            MasteryState::Functional => 4_000,
            MasteryState::Stable => 5_500,
            MasteryState::Transferable => 7_000,
            MasteryState::ExamReady => 8_500,
            MasteryState::Mastered => 9_000,
        }
    }

    /// Derive MasteryState from a raw basis-points score.
    pub fn from_score(score_bp: crate::primitives::BasisPoints) -> Self {
        if score_bp >= MasteryState::Mastered.min_score_bp() {
            MasteryState::Mastered
        } else if score_bp >= MasteryState::ExamReady.min_score_bp() {
            MasteryState::ExamReady
        } else if score_bp >= MasteryState::Transferable.min_score_bp() {
            MasteryState::Transferable
        } else if score_bp >= MasteryState::Stable.min_score_bp() {
            MasteryState::Stable
        } else if score_bp >= MasteryState::Functional.min_score_bp() {
            MasteryState::Functional
        } else if score_bp >= MasteryState::Unstable.min_score_bp() {
            MasteryState::Unstable
        } else if score_bp >= MasteryState::Emerging.min_score_bp() {
            MasteryState::Emerging
        } else {
            MasteryState::Untested
        }
    }
}

// ─── MockType ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MockType {
    /// Maximum realism — mirrors the most likely real exam.
    Forecast,
    /// Maximum diagnostic insight — designed to reveal gaps.
    Diagnostic,
    /// Closes known weak areas with targeted questions.
    Remediation,
    /// Full readiness proof at exam-standard conditions.
    FinalExam,
    /// Resilience training with hard, unexpected items.
    Shock,
    /// Mastery proof at elite performance standards.
    Wisdom,
}

// ─── JourneyPhase ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JourneyPhase {
    /// Fix major weaknesses and core foundational concepts.
    StabilizeFoundation,
    /// Cover main syllabus, build deep understanding.
    BuildCore,
    /// Attack weak topics and recurring mistakes.
    StrengthenWeakLinks,
    /// Timed questions, exam pressure, mixed-topic drills.
    ExamConditioning,
    /// Mocks, revision bursts, confidence repair, exam strategy.
    FinalReadiness,
}

// ─── RiseStage ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiseStage {
    /// Stop the bleeding; find root gaps; first wins; shame removal.
    Rescue,
    /// Make correct thinking repeatable; scaffolded; concept clusters.
    Stabilize,
    /// Speed and independence; timed drills; mixed topics; pressure mode.
    Accelerate,
    /// Outperform top students; trap questions; elite variants; speed + accuracy.
    Dominate,
}

// ─── KnowledgeGapType ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeGapType {
    /// Never encountered the concept at all.
    Content,
    /// Seen it but does not truly understand it.
    Understanding,
    /// Understands theory but cannot use it to solve questions.
    Application,
    /// Knows the idea but procedural steps are weak.
    Process,
    /// Can solve but too slowly for exam conditions.
    Speed,
    /// Knows the content but makes careless mistakes.
    Accuracy,
    /// Once knew it but has forgotten it.
    Retention,
    /// Solves familiar versions but fails novel/transfer forms.
    Transfer,
    /// Interference from a similar concept causing confusion.
    Interference,
    /// Does not know what they do not know.
    SelfAwareness,
}

// ─── QuestionIntent ──────────────────────────────────────────────────────────

/// The pedagogic purpose for which a question is selected or composed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionIntent {
    /// Find where the learner currently stands.
    Discovery,
    /// Probe a specific suspected weakness.
    Diagnosis,
    /// Verify improvement from a different angle.
    Confirmation,
    /// Broaden syllabus coverage.
    Coverage,
    /// Strengthen fragile recent gains.
    Reinforcement,
    /// Test whether a wrong mental model is still present.
    MisconceptionExposure,
    /// Check recall after time has passed.
    RetentionCheck,
    /// Check performance in a new context or wording.
    TransferCheck,
    /// Test smooth, low-effort performance.
    FluencyCheck,
    /// Test performance under time constraint.
    SpeedCheck,
    /// Replicate exam conditions faithfully.
    ExamSimulation,
    /// Rebuild confidence with achievable challenges.
    ConfidenceRepair,
}

// ─── LearnerArchetype ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearnerArchetype {
    /// Gaps are wide but effort is consistent — foundation-first progression.
    WeakButConsistent,
    /// Strong knowledge but avoids hard work — shorter, sharper, accountable.
    StrongButLazy,
    /// Arriving very late — high-yield triage mode.
    PanickingLastMinute,
    /// Believes they know more than they do — diagnostics + timed challenges.
    Overconfident,
    /// Has lost belief in their ability — confidence-first sequencing.
    Discouraged,
}

// ─── PressureLevel ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureLevel {
    /// No time pressure; supportive environment.
    Calm,
    /// Gentle pacing guidance; timer visible but not strict.
    GuidedTimed,
    /// Light time constraint; slight urgency.
    Mild,
    /// Noticeable time pressure; forces prioritisation.
    Moderate,
    /// Full exam-equivalent time pressure.
    ExamPressure,
    /// Compressed beyond exam norms — elite conditioning.
    ElitePressure,
}

// ─── ErrorType ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Did not remember the underlying concept.
    ForgotConcept,
    /// Misread or misinterpreted the question wording.
    MisunderstoodWording,
    /// Knew the method but made an arithmetic or algebraic slip.
    CalculationSlip,
    /// Selected an answer without real knowledge.
    Guessed,
    /// Pressure or time caused thinking to break down.
    PanicUnderTime,
    /// Chose the most familiar-looking option rather than the correct one.
    ChoseFamiliarDistractor,
    /// Took the right topic but wrong first step in the solution path.
    WrongFirstStep,
    /// Started correctly but reasoning was incomplete or unsupported.
    IncompleteReasoning,
    /// Misread the unit, scale, or a keyword in the question.
    MisreadUnits,
    /// Knew the concept but made an avoidable, inattentive error.
    CarelessError,
}
```

---

## 3.3 Key Structs

```rust
// ecoach-substrate/src/structs.rs

use serde::{Deserialize, Serialize};
use crate::primitives::{BasisPoints, TimestampMs};

// ─── DomainEvent ─────────────────────────────────────────────────────────────

/// A single fact that has occurred in the system, stored in the event_log table.
/// Payload is an opaque JSON string; consumers deserialise to their own type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    /// UUID v4; globally unique identifier for this event instance.
    pub event_id: String,
    /// Canonical event type string, e.g. `"student.attempt.recorded"`.
    pub event_type: String,
    /// Stringified identifier of the root entity this event belongs to.
    pub aggregate_id: String,
    /// Wall-clock time when the event occurred, in milliseconds since epoch.
    pub occurred_at: TimestampMs,
    /// JSON-encoded event payload. Structure varies by event_type.
    pub payload: String,
    /// Optional correlation identifier linking a chain of related events.
    pub trace_id: Option<String>,
    /// Optional causation: the event_id that triggered this one.
    pub causation_id: Option<String>,
}

// ─── LearnerState ────────────────────────────────────────────────────────────

/// The seven-dimensional model of a learner's current academic reality.
/// Each field is a domain-specific map or estimate; types are in their own modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerState {
    /// Per-topic mastery scores and mastery-state classification.
    pub knowledge_state: TopicMasteryMap,
    /// Stability and confidence signals per topic.
    pub confidence_state: TopicConfidenceMap,
    /// Active and suspected misconceptions and their resolution status.
    pub misconception_state: MisconceptionSet,
    /// Dependency-gap map: which weaknesses block other learning.
    pub dependency_state: DependencyGapMap,
    /// Exam-readiness estimate across all required topics.
    pub readiness_state: ReadinessEstimate,
    /// Syllabus coverage: which nodes have been touched and to what depth.
    pub coverage_state: SyllabusCoverageMap,
    /// Current trajectory: improving, plateau, or regressing.
    pub momentum_state: MomentumTrend,
}

// Placeholder map/set types — concrete definitions live in their own domain modules.
// Declared here so LearnerState compiles in the substrate crate.

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicMasteryMap(pub std::collections::HashMap<i64, BasisPoints>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicConfidenceMap(pub std::collections::HashMap<i64, BasisPoints>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MisconceptionSet(pub Vec<ActiveMisconception>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMisconception {
    pub misconception_id: i64,
    pub topic_id: i64,
    pub strength_bp: BasisPoints,
    pub last_triggered_at: Option<TimestampMs>,
    pub is_resolved: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGapMap(pub Vec<DependencyGap>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGap {
    pub blocked_topic_id: i64,
    pub blocking_topic_id: i64,
    pub gap_severity_bp: BasisPoints,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReadinessEstimate {
    pub overall_bp: BasisPoints,
    pub mastery_component_bp: BasisPoints,
    pub retention_component_bp: BasisPoints,
    pub mock_component_bp: BasisPoints,
    pub speed_component_bp: BasisPoints,
    pub coverage_component_bp: BasisPoints,
    pub consistency_component_bp: BasisPoints,
    pub penalty_bp: BasisPoints,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyllabusCoverageMap(pub std::collections::HashMap<i64, CoverageEntry>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoverageEntry {
    pub topic_id: i64,
    pub seen: bool,
    pub depth_bp: BasisPoints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumTrend {
    pub direction: crate::domain_enums::TrendDirection,
    pub momentum_score_bp: BasisPoints,
    /// EMA of accuracy over recent sessions.
    pub accuracy_ema_bp: BasisPoints,
    /// EMA of session volume over recent days.
    pub volume_ema: f64,
    /// EMA of practice pace (questions/minute).
    pub pace_ema: f64,
}

impl Default for MomentumTrend {
    fn default() -> Self {
        MomentumTrend {
            direction: crate::domain_enums::TrendDirection::Stable,
            momentum_score_bp: 5_000,
            accuracy_ema_bp: 0,
            volume_ema: 0.0,
            pace_ema: 0.0,
        }
    }
}

// ─── RiseStudentScores ───────────────────────────────────────────────────────

/// Eight internal intelligence scores that drive the Rise (weakest-to-best) engine.
/// All fields are BasisPoints (0–10_000).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiseStudentScores {
    /// How intact are prerequisite / foundational concepts? Higher = stronger foundation.
    pub foundation_score: BasisPoints,
    /// Retention quality: how well does the learner keep what they learned?
    pub recall_score: BasisPoints,
    /// Speed: correct solutions within expected time windows.
    pub speed_score: BasisPoints,
    /// Accuracy: proportion of correct answers across attempts.
    pub accuracy_score: BasisPoints,
    /// Stability under pressure: how much does performance drop when timed or pressured?
    pub pressure_stability_score: BasisPoints,
    /// Density of recurring wrong patterns (normalised; lower = fewer misconceptions).
    pub misconception_density: BasisPoints,
    /// Trajectory: rising / flat / falling over recent sessions.
    pub momentum_score: BasisPoints,
    /// Readiness to advance to the next Rise stage.
    pub transformation_readiness: BasisPoints,
}

// ─── EvidenceWeight ──────────────────────────────────────────────────────────

/// Computes the effective evidential weight of a single question attempt,
/// accounting for hints, transfer context, and recall delay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceWeight {
    /// Base weight before any modifiers (typically 1.0 for a clean unaided attempt).
    pub base: f64,
    /// Multiplicative hint factor: each hint halves the weight (0.5^n).
    /// Pass 1.0 if no hints were used.
    pub hint_factor: f64,
    /// Bonus multiplier when the question was a transfer/novel-context variant.
    pub transfer_bonus: f64,
    /// Bonus multiplier when the question was answered after a deliberate delay
    /// (spaced-recall check).
    pub recall_delay_bonus: f64,
}

impl EvidenceWeight {
    /// Create an EvidenceWeight for an unaided, non-transfer, non-delayed attempt.
    pub fn standard() -> Self {
        EvidenceWeight {
            base: 1.0,
            hint_factor: 1.0,
            transfer_bonus: 1.0,
            recall_delay_bonus: 1.0,
        }
    }

    /// Create an EvidenceWeight applying n hints.
    pub fn with_hints(hint_count: u8) -> Self {
        let hint_factor = 0.5_f64.powi(hint_count as i32);
        EvidenceWeight {
            base: 1.0,
            hint_factor,
            transfer_bonus: 1.0,
            recall_delay_bonus: 1.0,
        }
    }

    /// Compute the final effective weight after all modifiers.
    ///
    /// Formula: `base × hint_factor × transfer_bonus × recall_delay_bonus`
    pub fn effective_weight(&self) -> f64 {
        self.base * self.hint_factor * self.transfer_bonus * self.recall_delay_bonus
    }

    /// Convert the effective weight to a BasisPoints integer for storage.
    pub fn as_bp(&self) -> BasisPoints {
        crate::primitives::to_bp(self.effective_weight().clamp(0.0, 1.0))
    }
}
```

---

## 3.4 Constants Module

```rust
// ecoach-substrate/src/thresholds.rs

/// All numeric thresholds used across eCoach engines.
/// Values are in BasisPoints (u16, range 0–10_000) unless noted.
pub mod thresholds {

    // ── Mastery Gates ────────────────────────────────────────────────────────

    /// Below this score a topic is blocked — prerequisites not met.
    pub const MASTERY_BLOCKED_BELOW: u16 = 4_000;

    /// Minimum score for Functional mastery (reliable in standard conditions).
    pub const MASTERY_FUNCTIONAL_MIN: u16 = 5_500;

    /// Minimum score for Stable mastery (consistent across sessions).
    pub const MASTERY_STABLE_MIN: u16 = 7_000;

    /// Minimum score for ExamReady mastery (timed, exam-standard performance).
    pub const MASTERY_EXAM_READY_MIN: u16 = 8_500;

    /// Score at which mastery is considered fully achieved.
    pub const MASTERY_FULL: u16 = 9_000;

    // ── Evidence Weights ─────────────────────────────────────────────────────

    /// Each hint reduces the attempt's evidential weight by this factor.
    /// With n hints: weight = base × HINT_WEIGHT_MULTIPLIER^n.
    pub const HINT_WEIGHT_MULTIPLIER: f64 = 0.5;

    /// Multiplier added when an attempt is a transfer (novel-context) variant.
    pub const TRANSFER_WEIGHT_BONUS: f64 = 1.30;

    /// Multiplier added when an attempt occurs after a deliberate recall delay.
    pub const DELAYED_RECALL_BONUS: f64 = 1.50;

    // ── Exponential Moving Average ────────────────────────────────────────────

    /// Alpha for the EMA smoothing formula: new_ema = alpha × value + (1 − alpha) × old_ema.
    /// 0.3 weights recent observations meaningfully while dampening noise.
    pub const EMA_ALPHA: f64 = 0.3;

    // ── Session Duration ─────────────────────────────────────────────────────

    /// Hard ceiling on a single coaching session (minutes).
    pub const MAX_SESSION_MINUTES: u32 = 90;

    /// Minimum useful session length (minutes).
    pub const MIN_SESSION_MINUTES: u32 = 10;

    /// Default session target when the learner has not customised their schedule.
    pub const DEFAULT_SESSION_MINUTES: u32 = 45;

    // ── Memory Decay ─────────────────────────────────────────────────────────

    /// Memory strength at or below which the rescue scheduler activates.
    pub const MEMORY_RESCUE_THRESHOLD_BP: u16 = 4_000;

    /// Memory strength at or below which the item is classified Critical.
    pub const MEMORY_CRITICAL_THRESHOLD_BP: u16 = 2_500;

    // ── Gap Priority ─────────────────────────────────────────────────────────

    /// Gap score (as a 0.0–1.0 fraction) above which a gap is classified Urgent.
    pub const GAP_URGENT_THRESHOLD: f64 = 0.70;

    /// Gap score above which a gap is classified Critical.
    pub const GAP_CRITICAL_THRESHOLD: f64 = 0.85;

    // ── PIN Lockout ───────────────────────────────────────────────────────────

    /// Number of consecutive failed PIN attempts before lockout is applied.
    pub const PIN_MAX_ATTEMPTS: u8 = 5;

    /// Duration of the PIN lockout period (minutes).
    pub const PIN_LOCKOUT_MINUTES: u32 = 5;

    // ── Readiness Score Weights ───────────────────────────────────────────────
    // Source: backend_supplement section 1.5 (full formula)

    pub const READINESS_WEIGHT_MASTERY: f64 = 0.45;
    pub const READINESS_WEIGHT_TIMED_PERFORMANCE: f64 = 0.20;
    pub const READINESS_WEIGHT_COVERAGE: f64 = 0.15;
    pub const READINESS_WEIGHT_CONSISTENCY: f64 = 0.10;
    pub const READINESS_WEIGHT_TREND: f64 = 0.10;

    // ── Momentum Score Weights ────────────────────────────────────────────────

    pub const MOMENTUM_WEIGHT_VOLUME: f64 = 0.35;
    pub const MOMENTUM_WEIGHT_ACCURACY: f64 = 0.40;
    pub const MOMENTUM_WEIGHT_PACE: f64 = 0.25;

    // ── Strain Score Weights ──────────────────────────────────────────────────

    pub const STRAIN_WEIGHT_ACCURACY_DROP: f64 = 0.30;
    pub const STRAIN_WEIGHT_COMPLETION_DROP: f64 = 0.20;
    pub const STRAIN_WEIGHT_HINT_SPIKE: f64 = 0.20;
    pub const STRAIN_WEIGHT_SKIP: f64 = 0.15;
    pub const STRAIN_WEIGHT_PACE_INSTABILITY: f64 = 0.15;

    // ── Weakness Scoring Weights ──────────────────────────────────────────────

    pub const WEAKNESS_WEIGHT_MASTERY_DEFICIT: f64 = 0.35;
    pub const WEAKNESS_WEIGHT_LINK_BREAKAGE: f64 = 0.20;
    pub const WEAKNESS_WEIGHT_MISCONCEPTION_PRESSURE: f64 = 0.15;
    pub const WEAKNESS_WEIGHT_REPRESENTATION_GAP: f64 = 0.10;
    pub const WEAKNESS_WEIGHT_TIMED_GAP: f64 = 0.10;
    pub const WEAKNESS_WEIGHT_GUESS_PENALTY: f64 = 0.05;
    pub const WEAKNESS_WEIGHT_RECENCY_DECAY: f64 = 0.05;

    // ── Forecast Score Weights ────────────────────────────────────────────────

    pub const FORECAST_WEIGHT_FREQUENCY: f64 = 0.25;
    pub const FORECAST_WEIGHT_RECENCY: f64 = 0.20;
    pub const FORECAST_WEIGHT_TREND: f64 = 0.15;
    pub const FORECAST_WEIGHT_BUNDLE_STRENGTH: f64 = 0.15;
    pub const FORECAST_WEIGHT_SYLLABUS_PRIORITY: f64 = 0.10;
    pub const FORECAST_WEIGHT_STYLE_REGIME_FIT: f64 = 0.10;
    pub const FORECAST_WEIGHT_EXAMINER_GOAL_FIT: f64 = 0.05;

    // ── Orchestration Anti-Repeat Penalty ────────────────────────────────────

    pub const ORCHESTRATION_ANTI_REPEAT_PENALTY: f64 = 0.25;

    // ── Danger Zone Thresholds ────────────────────────────────────────────────

    /// Days of inactivity before the inactivity danger zone opens.
    pub const INACTIVITY_DANGER_DAYS: u32 = 3;

    /// Days of inactivity before the danger zone escalates to Urgent.
    pub const INACTIVITY_URGENT_DAYS: u32 = 7;

    // ── Exam Countdown ────────────────────────────────────────────────────────

    /// Days to exam at which ExamCountdown coach state activates.
    pub const EXAM_COUNTDOWN_TRIGGER_DAYS: u32 = 7;

    /// Days to exam at which CrisisIntervention may be triggered if readiness is low.
    pub const CRISIS_INTERVENTION_DAYS_THRESHOLD: u32 = 14;

    /// Readiness score (bp) below which CrisisIntervention is triggered in final window.
    pub const CRISIS_READINESS_THRESHOLD_BP: u16 = 4_500;
}
```

---

## 3.5 Crate Root

```rust
// ecoach-substrate/src/lib.rs

pub mod primitives;
pub mod domain_enums;
pub mod structs;
pub mod thresholds;

// Re-export the most frequently used items at crate root.
pub use primitives::{
    to_bp, from_bp, BasisPoints, ConfidenceScore,
    SeverityLevel, TrendDirection, TimestampMs, DurationMs,
};
pub use domain_enums::{
    Role, EntitlementTier, CoachLifecycleState, MemoryState,
    MasteryState, MockType, JourneyPhase, RiseStage,
    KnowledgeGapType, QuestionIntent, LearnerArchetype,
    PressureLevel, ErrorType,
};
pub use structs::{
    DomainEvent, LearnerState, RiseStudentScores, EvidenceWeight,
    TopicMasteryMap, TopicConfidenceMap, MisconceptionSet,
    ActiveMisconception, DependencyGapMap, DependencyGap,
    ReadinessEstimate, SyllabusCoverageMap, CoverageEntry, MomentumTrend,
};
pub use thresholds::thresholds;
```


---

# eCoach Backend Implementation Plan — Part 4 & Part 5
## Scoring Formulas as Rust Code + Business Rules & Constants
## Generated: 2026-03-29

---

# PART 4: ALL SCORING FORMULAS AS RUST CODE

All scores use the `BasisPoints` type alias (`type BasisPoints = i32`, range 0–10000 representing 0.0–1.0). Helper functions `to_bp(f: f64) -> BasisPoints` and `from_bp(bp: BasisPoints) -> f64` convert between the two representations.

```rust
/// Shared type definitions used across all scoring functions
pub type BasisPoints = i32;

/// Convert a float in [0.0, 1.0] to basis points [0, 10000]
#[inline]
pub fn to_bp(v: f64) -> BasisPoints {
    (v.clamp(0.0, 1.0) * 10_000.0).round() as BasisPoints
}

/// Convert basis points [0, 10000] to float [0.0, 1.0]
#[inline]
pub fn from_bp(bp: BasisPoints) -> f64 {
    bp as f64 / 10_000.0
}
```

---

## 4.1 ForecastScore

Estimates the probability that a given topic/question unit `u` will appear on the next BECE paper. Used by the Mock Centre to build Forecast mock sets and produce the Blueprint.

**Formula:**
```
ForecastScore(u) =
    0.25 × Frequency(u)
  + 0.20 × Recency(u)
  + 0.15 × Trend(u)
  + 0.15 × BundleStrength(u)
  + 0.10 × SyllabusPriority(u)
  + 0.10 × StyleRegimeFit(u)
  + 0.05 × ExaminerGoalFit(u)
```

**Output Bands:**
- High: score ≥ 0.70 (≥ 7000 bp)
- Medium: 0.45–0.69 (4500–6999 bp)
- Surprise Risk: 0.30–0.44 (3000–4499 bp)
- Uncertain: < 0.30 (< 3000 bp)

```rust
/// Probability band for a forecast score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForecastBand {
    /// Highly likely to appear on next exam (≥ 7000 bp)
    High,
    /// Moderately likely (4500–6999 bp)
    Medium,
    /// Possible but unlikely; worth including for resilience (3000–4499 bp)
    SurpriseRisk,
    /// Insufficient signal; do not rely on this appearing (< 3000 bp)
    Uncertain,
}

impl ForecastBand {
    pub fn from_score(score: BasisPoints) -> Self {
        match score {
            s if s >= 7_000 => Self::High,
            s if s >= 4_500 => Self::Medium,
            s if s >= 3_000 => Self::SurpriseRisk,
            _               => Self::Uncertain,
        }
    }
}

/// Inputs for the ForecastScore computation.
/// All component values are normalised to [0.0, 1.0].
pub struct ForecastInputs {
    /// How often this unit has appeared across past papers (normalised frequency).
    pub frequency: f64,
    /// How recently it appeared — more recent = higher value.
    pub recency: f64,
    /// Direction of appearance trend over the last N papers (rising trend = 1.0).
    pub trend: f64,
    /// Strength of the question bundle / topic cluster it belongs to.
    pub bundle_strength: f64,
    /// Weight this topic carries in the official syllabus.
    pub syllabus_priority: f64,
    /// Fit with the identified examiner style regime for this paper cycle.
    pub style_regime_fit: f64,
    /// Fit with inferred examiner goal for this sitting.
    pub examiner_goal_fit: f64,
}

/// Compute the ForecastScore for a topic/question unit.
///
/// # Source
/// idea1.txt §1.1; backend_supplement §1.1
///
/// # Returns
/// `(score_bp, band)` — raw score in basis points and its categorical band.
pub fn forecast_score(inputs: &ForecastInputs) -> (BasisPoints, ForecastBand) {
    let score = 0.25 * inputs.frequency
        + 0.20 * inputs.recency
        + 0.15 * inputs.trend
        + 0.15 * inputs.bundle_strength
        + 0.10 * inputs.syllabus_priority
        + 0.10 * inputs.style_regime_fit
        + 0.05 * inputs.examiner_goal_fit;

    let bp = to_bp(score);
    let band = ForecastBand::from_score(bp);
    (bp, band)
}
```

---

## 4.2 MockOrchestrationScore

Scores every candidate topic/question for inclusion in a mock session. A high score means the item should be prioritised in the next mock. The anti-repeat penalty subtracts weight when an item was recently seen.

**Formula:**
```
score =
    0.25 × weakness
  + 0.20 × coverage_gap
  + 0.20 × misconception_pressure
  + 0.15 × spaced_due
  + 0.10 × exam_weight
  + 0.10 × info_value
  + 0.05 × variety_bonus
  − 0.25 × anti_repeat_penalty
```

```rust
/// Inputs for MockOrchestrationScore.
/// All values normalised to [0.0, 1.0].
pub struct MockOrchestrationInputs {
    /// Student weakness score for this topic (see §4.4).
    pub weakness: f64,
    /// How uncovered this topic is in recent mocks.
    pub coverage_gap: f64,
    /// Active misconception pressure on this topic.
    pub misconception_pressure: f64,
    /// Spaced-repetition urgency: is a recheck overdue?
    pub spaced_due: f64,
    /// Exam blueprint weight for this topic.
    pub exam_weight: f64,
    /// Diagnostic information value: how much would a result tell us?
    pub info_value: f64,
    /// Variety bonus: reward for adding representation/topic diversity.
    pub variety_bonus: f64,
    /// Penalty for repeating the same topic/question family too soon.
    pub anti_repeat_penalty: f64,
}

/// Compute the mock orchestration priority score for a candidate topic/item.
///
/// # Source
/// idea1.txt §1.2; backend_supplement §1.2
///
/// # Returns
/// Score in basis points. Negative values are clamped to 0.
pub fn mock_orchestration_score(inputs: &MockOrchestrationInputs) -> BasisPoints {
    let raw = 0.25 * inputs.weakness
        + 0.20 * inputs.coverage_gap
        + 0.20 * inputs.misconception_pressure
        + 0.15 * inputs.spaced_due
        + 0.10 * inputs.exam_weight
        + 0.10 * inputs.info_value
        + 0.05 * inputs.variety_bonus
        - 0.25 * inputs.anti_repeat_penalty;

    to_bp(raw.max(0.0))
}
```

---

## 4.3 MockSelectionScore

Scores individual questions (not whole topics) for selection into a compiled mock paper. Complements MockOrchestrationScore by operating at question granularity.

**Formula:**
```
MockSelect(f) =
    0.30 × BlueprintFit(f)
  + 0.20 × DiagnosticNeed(f)
  + 0.15 × CoverageNeed(f)
  + 0.10 × InfoValue(f)
  + 0.10 × RepresentationNeed(f)
  + 0.10 × Variety(f)
  + 0.05 × SurpriseRisk(f)
  − 0.25 × AntiRepeat(f)
```

```rust
/// Inputs for MockSelectionScore, evaluated per question candidate `f`.
/// All values normalised to [0.0, 1.0].
pub struct MockSelectionInputs {
    /// How well this question fits the exam blueprint target distribution.
    pub blueprint_fit: f64,
    /// How much diagnostic signal this question would produce for this student.
    pub diagnostic_need: f64,
    /// How much this question covers a currently under-represented area.
    pub coverage_need: f64,
    /// Raw informational value (uncertainty reduction about student state).
    pub info_value: f64,
    /// Need for this representation type (text/diagram/graph/table) in the mock.
    pub representation_need: f64,
    /// Question variety contribution relative to already-selected questions.
    pub variety: f64,
    /// Surprise risk: this is a low-frequency but plausible exam question.
    pub surprise_risk: f64,
    /// Penalty for selecting a question too similar to a recent attempt.
    pub anti_repeat: f64,
}

/// Compute the selection score for a question candidate in mock compilation.
///
/// # Source
/// idea1.txt §1.3; backend_supplement §1.3
pub fn mock_selection_score(inputs: &MockSelectionInputs) -> BasisPoints {
    let raw = 0.30 * inputs.blueprint_fit
        + 0.20 * inputs.diagnostic_need
        + 0.15 * inputs.coverage_need
        + 0.10 * inputs.info_value
        + 0.10 * inputs.representation_need
        + 0.10 * inputs.variety
        + 0.05 * inputs.surprise_risk
        - 0.25 * inputs.anti_repeat;

    to_bp(raw.max(0.0))
}
```

---

## 4.4 WeaknessScore

Quantifies how weak a student is on a specific topic `t`. Higher score = higher urgency to address the topic.

**Formula:**
```
Weakness(t) =
    0.35 × (1 − Mastery_t)
  + 0.20 × LinkBreakage_t
  + 0.15 × MisconceptionPressure_t
  + 0.10 × RepresentationGap_t
  + 0.10 × TimedGap_t
  + 0.05 × GuessPenalty_t
  + 0.05 × RecencyDecay_t
```

```rust
/// Inputs for WeaknessScore on topic `t`.
/// All values normalised to [0.0, 1.0].
pub struct WeaknessInputs {
    /// Current mastery score for this topic (used as `1 - mastery` internally).
    pub mastery: f64,
    /// Degree to which prerequisite concept links are broken for this topic.
    pub link_breakage: f64,
    /// Intensity of active misconceptions on this topic.
    pub misconception_pressure: f64,
    /// Gap in the student's ability across different question representations.
    pub representation_gap: f64,
    /// Performance gap when the question is timed vs untimed.
    pub timed_gap: f64,
    /// Penalty for detected guessing patterns on this topic.
    pub guess_penalty: f64,
    /// Decay signal: how long since meaningful correct engagement.
    pub recency_decay: f64,
}

/// Compute the weakness score for topic `t`.
///
/// # Source
/// idea1.txt §1.4; backend_supplement §1.4
///
/// # Returns
/// Score in basis points: 10000 = maximally weak, 0 = no weakness detected.
pub fn weakness_score(inputs: &WeaknessInputs) -> BasisPoints {
    let raw = 0.35 * (1.0 - inputs.mastery)
        + 0.20 * inputs.link_breakage
        + 0.15 * inputs.misconception_pressure
        + 0.10 * inputs.representation_gap
        + 0.10 * inputs.timed_gap
        + 0.05 * inputs.guess_penalty
        + 0.05 * inputs.recency_decay;

    to_bp(raw)
}
```

---

## 4.5 ReadinessScore

Estimates overall exam readiness. Penalties are subtracted for known high-risk conditions.

**Formula:**
```
Readiness =
    0.45 × Mastery
  + 0.20 × TimedPerformance
  + 0.15 × Coverage
  + 0.10 × Consistency
  + 0.10 × Trend

Penalties applied for:
  - Weak critical topics (exam-critical topic with mastery < 4000 bp)
  - Recurring mistakes (same error class seen ≥ 3 times)
  - Missed sessions (> 3 consecutive days inactive)
  - Exam anxiety signals (pressure collapse index elevated)
  - Low recent trend (last 3-session accuracy declining)
```

```rust
/// Raw component inputs for ReadinessScore.
/// All values normalised to [0.0, 1.0].
pub struct ReadinessInputs {
    /// Weighted average mastery across all exam-relevant topics.
    pub mastery: f64,
    /// Performance accuracy and speed under timed conditions.
    pub timed_performance: f64,
    /// Percentage of syllabus topics with at least functional mastery (≥ 5500 bp).
    pub coverage: f64,
    /// Consistency of performance across sessions and question types.
    pub consistency: f64,
    /// Directional improvement trend over the last N sessions.
    pub trend: f64,
    /// Number of exam-critical topics below mastery gate (4000 bp).
    pub critical_topic_failures: u32,
    /// Whether recurring mistakes are detected (same class ≥ 3 times).
    pub has_recurring_mistakes: bool,
    /// Number of consecutive inactive days.
    pub consecutive_inactive_days: u32,
    /// Whether exam anxiety / pressure collapse pattern is elevated.
    pub has_exam_anxiety: bool,
    /// Whether the 3-session recent accuracy trend is declining.
    pub recent_trend_declining: bool,
}

/// Penalty magnitudes (in raw float units, not basis points).
const PENALTY_CRITICAL_TOPIC: f64 = 0.05;   // per failing critical topic, capped at 0.20
const PENALTY_RECURRING_MISTAKES: f64 = 0.04;
const PENALTY_INACTIVITY: f64 = 0.03;        // applied when inactive_days >= 3
const PENALTY_EXAM_ANXIETY: f64 = 0.04;
const PENALTY_RECENT_DECLINE: f64 = 0.03;

/// Compute the ReadinessScore.
///
/// # Source
/// idea1.txt §1.5; idea2.txt §2.6; backend_supplement §1.5, §2.6
///
/// # Returns
/// Score in basis points. Represents overall exam readiness.
pub fn readiness_score(inputs: &ReadinessInputs) -> BasisPoints {
    let base = 0.45 * inputs.mastery
        + 0.20 * inputs.timed_performance
        + 0.15 * inputs.coverage
        + 0.10 * inputs.consistency
        + 0.10 * inputs.trend;

    let critical_penalty = (inputs.critical_topic_failures as f64 * PENALTY_CRITICAL_TOPIC)
        .min(0.20);
    let recurring_penalty = if inputs.has_recurring_mistakes {
        PENALTY_RECURRING_MISTAKES
    } else {
        0.0
    };
    let inactivity_penalty = if inputs.consecutive_inactive_days >= 3 {
        PENALTY_INACTIVITY
    } else {
        0.0
    };
    let anxiety_penalty = if inputs.has_exam_anxiety {
        PENALTY_EXAM_ANXIETY
    } else {
        0.0
    };
    let trend_penalty = if inputs.recent_trend_declining {
        PENALTY_RECENT_DECLINE
    } else {
        0.0
    };

    let total_penalty = critical_penalty
        + recurring_penalty
        + inactivity_penalty
        + anxiety_penalty
        + trend_penalty;

    to_bp((base - total_penalty).max(0.0))
}

/// Journey-mode variant of ReadinessScore (from idea2.txt §2.6).
/// Weights differ from the Mock Centre version.
///
/// Readiness(Journey) =
///     0.25 × topic_mastery
///   + 0.20 × retention
///   + 0.20 × mock_performance
///   + 0.15 × speed
///   + 0.10 × syllabus_coverage
///   + 0.10 × consistency
///   − penalties (same penalty set as ReadinessScore)
pub struct JourneyReadinessInputs {
    pub topic_mastery: f64,
    pub retention: f64,
    pub mock_performance: f64,
    pub speed: f64,
    pub syllabus_coverage: f64,
    pub consistency: f64,
    pub critical_topic_failures: u32,
    pub has_recurring_mistakes: bool,
    pub consecutive_inactive_days: u32,
    pub has_exam_anxiety: bool,
    pub recent_trend_declining: bool,
}

/// Compute the Journey-mode readiness score (§4.14 alias).
pub fn journey_readiness_score(inputs: &JourneyReadinessInputs) -> BasisPoints {
    let base = 0.25 * inputs.topic_mastery
        + 0.20 * inputs.retention
        + 0.20 * inputs.mock_performance
        + 0.15 * inputs.speed
        + 0.10 * inputs.syllabus_coverage
        + 0.10 * inputs.consistency;

    let critical_penalty = (inputs.critical_topic_failures as f64 * PENALTY_CRITICAL_TOPIC)
        .min(0.20);
    let recurring_penalty = if inputs.has_recurring_mistakes { PENALTY_RECURRING_MISTAKES } else { 0.0 };
    let inactivity_penalty = if inputs.consecutive_inactive_days >= 3 { PENALTY_INACTIVITY } else { 0.0 };
    let anxiety_penalty = if inputs.has_exam_anxiety { PENALTY_EXAM_ANXIETY } else { 0.0 };
    let trend_penalty = if inputs.recent_trend_declining { PENALTY_RECENT_DECLINE } else { 0.0 };

    let total_penalty = critical_penalty + recurring_penalty + inactivity_penalty
        + anxiety_penalty + trend_penalty;

    to_bp((base - total_penalty).max(0.0))
}
```

---

## 4.6 PredictedExamScore

Projects the student's expected BECE score as a weighted product across all blueprint targets. Each topic `k` contributes its blueprint weight multiplied by the student's mastery on that topic, adjusted for timing readiness, memory retention, and misconception immunity.

**Formula:**
```
PredictedExamScore =
    ∑(k ∈ BlueprintTargets)
        BlueprintWeight_k × Mastery_k × TimingFactor_k
        × RetentionFactor_k × MisconceptionImmunity_k
```

```rust
/// A single topic's contribution data for PredictedExamScore.
pub struct BlueprintTopicFactors {
    /// Fraction of the exam blueprint this topic represents (sums to 1.0 across all k).
    pub blueprint_weight: f64,
    /// Student's mastery on this topic, normalised to [0.0, 1.0].
    pub mastery: f64,
    /// Readiness to perform under timed exam conditions (0.0 = not ready, 1.0 = fully ready).
    pub timing_factor: f64,
    /// Retention stability — how well knowledge will hold at exam time (0.0–1.0).
    pub retention_factor: f64,
    /// Immunity to known misconceptions on this topic (0.0 = high risk, 1.0 = immune).
    pub misconception_immunity: f64,
}

/// Compute the predicted exam score from blueprint topic factors.
///
/// # Source
/// idea1.txt §1.6; backend_supplement §1.6, §25
///
/// # Returns
/// Predicted score in basis points (0 = 0% predicted, 10000 = 100% predicted).
///
/// # Panics
/// Does not panic; returns 0 if `topics` is empty.
pub fn predicted_exam_score(topics: &[BlueprintTopicFactors]) -> BasisPoints {
    let sum: f64 = topics.iter().map(|t| {
        t.blueprint_weight
            * t.mastery
            * t.timing_factor
            * t.retention_factor
            * t.misconception_immunity
    }).sum();

    to_bp(sum)
}
```

---

## 4.7 MomentumScore

Measures the student's learning momentum across a recent session window. Used by Rise Mode to detect whether the student is rising, plateauing, or declining.

**Formula:**
```
MomentumScore = 0.35 × volume + 0.40 × accuracy + 0.25 × pace
```

```rust
/// Inputs for MomentumScore over a recent session window.
/// All values normalised to [0.0, 1.0].
pub struct MomentumInputs {
    /// Volume of meaningful attempts completed (normalised against target volume).
    pub volume: f64,
    /// Accuracy rate across the window (correct / total attempts).
    pub accuracy: f64,
    /// Pace: average speed relative to expected time-per-question (faster = higher).
    pub pace: f64,
}

/// Compute the MomentumScore.
///
/// # Source
/// idea4.txt §4.6; backend_supplement §4.6, §25
pub fn momentum_score(inputs: &MomentumInputs) -> BasisPoints {
    let raw = 0.35 * inputs.volume
        + 0.40 * inputs.accuracy
        + 0.25 * inputs.pace;

    to_bp(raw)
}
```

---

## 4.8 StrainScore

Detects cognitive overload or burnout signals within a session. A high strain score means the student is struggling beyond their productive zone and may need a break or de-escalation.

**Formula:**
```
StrainScore =
    0.30 × accuracy_drop
  + 0.20 × completion_drop
  + 0.20 × hint_spike
  + 0.15 × skip
  + 0.15 × pace_instability
```

```rust
/// Inputs for StrainScore within a session window.
/// All values normalised to [0.0, 1.0].
pub struct StrainInputs {
    /// Drop in accuracy compared to the student's recent baseline.
    pub accuracy_drop: f64,
    /// Drop in question completion rate (more questions abandoned mid-session).
    pub completion_drop: f64,
    /// Spike in hint usage relative to the student's usual rate.
    pub hint_spike: f64,
    /// Rate of skipped questions this session.
    pub skip: f64,
    /// Instability in answer pace (high variance in response times).
    pub pace_instability: f64,
}

/// Compute the StrainScore.
///
/// # Source
/// idea4.txt §4.7; backend_supplement §4.7, §25
pub fn strain_score(inputs: &StrainInputs) -> BasisPoints {
    let raw = 0.30 * inputs.accuracy_drop
        + 0.20 * inputs.completion_drop
        + 0.20 * inputs.hint_spike
        + 0.15 * inputs.skip
        + 0.15 * inputs.pace_instability;

    to_bp(raw)
}
```

---

## 4.9 MSI — Multi-Source Intelligence Score

Aggregates six independent evidence signals into a single memory-strength index for a knowledge unit. Used by Memory Mode to decide scheduling urgency.

**Formula:**
```
MSI =
    0.30 × Accuracy
  + 0.15 × Speed
  + 0.20 × Retention
  + 0.15 × Variability
  + 0.10 × Interference
  + 0.10 × Consistency
```

Where:
- `Accuracy` = recent accuracy rate (last N attempts)
- `Speed` = normalised recall latency (lower latency = higher score)
- `Retention` = decay resistance over time
- `Variability` = variant/transfer performance
- `Interference` = independence from hints and cues
- `Consistency` = stable performance across contexts

```rust
/// Inputs for the MSI (Multi-Source Intelligence) score.
/// All values normalised to [0.0, 1.0]; higher = better memory on that dimension.
pub struct MsiInputs {
    /// Recent accuracy across last N attempts on this knowledge unit.
    pub accuracy: f64,
    /// Recall speed — inverse of normalised latency (fast = high).
    pub speed: f64,
    /// Retention over time — how well the concept resists decay.
    pub retention: f64,
    /// Transfer/variant performance — performance across changed question forms.
    pub variability: f64,
    /// Independence — performance without hints or prompts.
    pub interference: f64,
    /// Consistency — performance stability across sessions and contexts.
    pub consistency: f64,
}

/// Compute the MSI score for a knowledge unit.
///
/// # Source
/// idea7.txt §7.1; backend_supplement §7.1, §25
pub fn msi_score(inputs: &MsiInputs) -> BasisPoints {
    let raw = 0.30 * inputs.accuracy
        + 0.15 * inputs.speed
        + 0.20 * inputs.retention
        + 0.15 * inputs.variability
        + 0.10 * inputs.interference
        + 0.10 * inputs.consistency;

    to_bp(raw)
}
```

---

## 4.10 EMA — Exponentially Weighted Moving Average Mastery Update

All mastery updates use an EMA so that new evidence has high impact but the total score is smoothed against noise. Evidence weight is reduced by hints, boosted by transfer contexts and delayed recall.

**Formulas:**
```
// EMA update
new_score = alpha × new_evidence + (1 - alpha) × old_score   [alpha = 0.3]

// Evidence weight
effective_weight = base_weight
    × (0.5 ^ hint_count)
    × (1.30 if transfer_context else 1.0)
    × (1.50 if delayed_recall else 1.0)
    × (0.60 if repeat_same_day else 1.0)

// Weighted evidence input
weighted_evidence = raw_score × effective_weight
```

```rust
/// Alpha for the EMA update (smoothing factor).
pub const EMA_ALPHA: f64 = 0.3;

/// Compute the effective evidence weight for a single attempt.
///
/// # Source
/// idea7.txt; idea28.txt; detailed_backend_plan §evidence weight computation
///
/// # Parameters
/// - `hint_count`: number of hints used (each halves the weight)
/// - `is_transfer_context`: question tested transfer to a new context
/// - `is_delayed_recall`: attempted ≥ 24 hours after last exposure
/// - `is_repeat_same_day`: same question attempted again within the same day
pub fn compute_evidence_weight(
    hint_count: u32,
    is_transfer_context: bool,
    is_delayed_recall: bool,
    is_repeat_same_day: bool,
) -> f64 {
    let mut weight: f64 = 1.0;

    // Each hint halves the evidence weight (cap at 3 halvings = 0.125 minimum from hints)
    if hint_count > 0 {
        weight *= 0.5_f64.powi(hint_count.min(3) as i32);
    }

    // Transfer context: knowing it in a new form is stronger evidence
    if is_transfer_context {
        weight *= 1.30;
    }

    // Delayed recall: recalling after ≥ 24h gap signals durable memory
    if is_delayed_recall {
        weight *= 1.50;
    }

    // Same-day repeat: diminishing returns on repeated attempts the same day
    if is_repeat_same_day {
        weight *= 0.60;
    }

    weight.clamp(0.05, 2.0)
}

/// Apply an EMA update to a mastery score using new evidence.
///
/// # Source
/// idea28.txt; detailed_backend_plan §1.3 (evidence weight computation)
///
/// # Parameters
/// - `old_bp`: current mastery score in basis points
/// - `new_score`: raw score from the new attempt, normalised to [0.0, 1.0]
/// - `weight`: effective evidence weight from `compute_evidence_weight`
///
/// # Returns
/// Updated mastery score in basis points.
pub fn update_mastery_ema(old_bp: BasisPoints, new_score: f64, weight: f64) -> BasisPoints {
    let old_score = from_bp(old_bp);
    // Weight the incoming evidence
    let weighted_evidence = new_score * weight;
    // Normalise weighted evidence back to [0,1] (weight may exceed 1.0 for boosted evidence)
    let effective_input = weighted_evidence.clamp(0.0, 1.0);
    // EMA blend
    let new_mastery = EMA_ALPHA * effective_input + (1.0 - EMA_ALPHA) * old_score;
    to_bp(new_mastery)
}
```

---

## 4.11 GapPriorityScore

Ranks knowledge gaps by how urgently they need to be repaired. A higher score means the gap should be addressed before others.

**Formula:**
```
GapPriority =
    0.30 × dependency_block
  + 0.20 × recency_decay
  + 0.15 × exam_weight
  + 0.15 × misconception_density
  + 0.10 × repair_effort
  + 0.10 × confidence_gap
```

```rust
/// Inputs for GapPriorityScore.
/// All values normalised to [0.0, 1.0].
pub struct GapPriorityInputs {
    /// Degree to which this gap blocks downstream prerequisite topics.
    pub dependency_block: f64,
    /// How rapidly mastery on this topic is decaying (recency decay pressure).
    pub recency_decay: f64,
    /// Weight of this topic in the exam blueprint.
    pub exam_weight: f64,
    /// Density of active misconceptions on this topic.
    pub misconception_density: f64,
    /// Estimated repair effort (higher = more effort needed; inverted for priority).
    /// Note: this term adds priority because harder repairs need earlier scheduling.
    pub repair_effort: f64,
    /// Gap between student's confidence and actual performance on this topic.
    pub confidence_gap: f64,
}

/// Compute the gap priority score.
///
/// # Source
/// idea6.txt; backend_supplement §5.1
pub fn gap_priority_score(inputs: &GapPriorityInputs) -> BasisPoints {
    let raw = 0.30 * inputs.dependency_block
        + 0.20 * inputs.recency_decay
        + 0.15 * inputs.exam_weight
        + 0.15 * inputs.misconception_density
        + 0.10 * inputs.repair_effort
        + 0.10 * inputs.confidence_gap;

    to_bp(raw)
}
```

---

## 4.12 Memory Decay Function

Models the exponential decay of memory strength over time. Decay rate is governed by the current `MemoryState` of the knowledge unit.

**Formula:**
```
MemoryStrength(t) = initial_strength × e^(−decay_rate × days_since_last_review)

Decay rates by memory state:
  Fresh          → 0.05
  Consolidating  → 0.08
  Stable         → 0.03
  AtRisk         → 0.15
```

```rust
/// Memory states that govern the decay rate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryState {
    /// Recently learned; not yet consolidated. Decay rate: 0.05 / day.
    Fresh,
    /// In the consolidation window. Decay rate: 0.08 / day.
    Consolidating,
    /// Durable, well-rehearsed memory. Decay rate: 0.03 / day.
    Stable,
    /// Showing signs of degradation. Decay rate: 0.15 / day.
    AtRisk,
    /// Actively collapsing — requires immediate rescue.
    Decaying,
    /// Fully collapsed — no recall without full reteach.
    Collapsed,
}

impl MemoryState {
    /// Return the daily exponential decay rate for this memory state.
    pub fn decay_rate(&self) -> f64 {
        match self {
            Self::Fresh         => 0.05,
            Self::Consolidating => 0.08,
            Self::Stable        => 0.03,
            Self::AtRisk        => 0.15,
            Self::Decaying      => 0.25,
            Self::Collapsed     => 0.50,
        }
    }
}

/// Compute the decayed memory strength after `days_since_review` days.
///
/// # Source
/// idea32.txt; backend_supplement §16; idea7.txt §7.5
///
/// # Parameters
/// - `initial_strength_bp`: memory strength at the time of last successful review (basis points)
/// - `memory_state`: current classification of this memory unit
/// - `days_since_last_review`: elapsed days since last confirmed successful recall
///
/// # Returns
/// Decayed memory strength in basis points.
pub fn memory_decay(
    initial_strength_bp: BasisPoints,
    memory_state: &MemoryState,
    days_since_last_review: f64,
) -> BasisPoints {
    let initial = from_bp(initial_strength_bp);
    let decay_rate = memory_state.decay_rate();
    let decayed = initial * (-decay_rate * days_since_last_review).exp();
    to_bp(decayed)
}
```

---

## 4.13 EvidenceWeight Computation

Full evidence weight formula incorporating all modifiers. Returns a normalised weight suitable for use in EMA mastery updates.

**Formula:**
```
effective_weight =
    base_weight
    × (0.5 ^ hint_count)
    × (1.30 if transfer_context)
    × (1.50 if delayed_recall ≥ 24h)
    × (0.60 if repeat_same_day)

For support level:
    "guided"        → × 0.70
    "heavily_guided" → × 0.40

For confidence-correct guesses:
    correct + confidence == "guessed" → × 0.50
```

```rust
/// Support level during an attempt, affecting evidence weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportLevel {
    Independent,
    Guided,
    HeavilyGuided,
}

/// Full evidence weight inputs for a single attempt.
pub struct EvidenceWeightInputs {
    pub hint_count: u32,
    pub support_level: SupportLevel,
    pub is_transfer_context: bool,
    /// True if ≥ 24 hours elapsed since last exposure to this topic.
    pub is_delayed_recall: bool,
    pub is_repeat_same_day: bool,
    /// True if the student answered correctly but self-reported as "guessed".
    pub is_correct_but_guessed: bool,
}

/// Compute the full effective evidence weight for an attempt.
///
/// # Source
/// idea7.txt; idea28.txt; detailed_backend_plan §1.3
///
/// # Returns
/// Weight as a plain float; caller divides by this or uses directly in EMA.
/// Clamped to [0.05, 2.0].
pub fn compute_full_evidence_weight(inputs: &EvidenceWeightInputs) -> f64 {
    let mut weight: f64 = 1.0;

    // Hint penalty: each hint halves weight, maximum 3 halvings
    if inputs.hint_count > 0 {
        weight *= 0.5_f64.powi(inputs.hint_count.min(3) as i32);
    }

    // Support level penalty
    match inputs.support_level {
        SupportLevel::Guided        => weight *= 0.70,
        SupportLevel::HeavilyGuided => weight *= 0.40,
        SupportLevel::Independent   => {}
    }

    // Transfer context boost
    if inputs.is_transfer_context {
        weight *= 1.30;
    }

    // Delayed recall boost
    if inputs.is_delayed_recall {
        weight *= 1.50;
    }

    // Same-day repeat diminishment
    if inputs.is_repeat_same_day {
        weight *= 0.60;
    }

    // Guessed-but-correct penalty
    if inputs.is_correct_but_guessed {
        weight *= 0.50;
    }

    weight.clamp(0.05, 2.0)
}
```

---

## 4.14 JourneyReadinessScore

See §4.5 — the `journey_readiness_score` function is defined there. It uses different weights from the Mock Centre ReadinessScore:

```
JourneyReadiness =
    0.25 × topic_mastery
  + 0.20 × retention
  + 0.20 × mock_performance
  + 0.15 × speed
  + 0.10 × syllabus_coverage
  + 0.10 × consistency
  − penalties
```

The `journey_readiness_score` function defined in §4.5 is the canonical implementation.

---

## 4.15 ElitePerformanceScore (EPS)

Produces a composite Elite Performance Score from the five core elite dimensions. Used to rank and benchmark elite-tier students.

**Formula:**
```
EPS = weighted_average(
    speed,
    accuracy_under_pressure,
    transfer_ability,
    novelty_handling,
    misconception_immunity
)
```

Default weights are equal (0.20 each) but can be configured per subject or mission type.

```rust
/// Inputs for the Elite Performance Score.
/// All values normalised to [0.0, 1.0].
pub struct EpsInputs {
    /// Speed performance — time-to-correct relative to elite benchmark.
    pub speed: f64,
    /// Accuracy under pressure conditions (timed, high-stakes).
    pub accuracy_under_pressure: f64,
    /// Transfer ability — performance on novel question forms.
    pub transfer_ability: f64,
    /// Novelty handling — performance on new/unseen question types.
    pub novelty_handling: f64,
    /// Immunity to active misconceptions on this topic set.
    pub misconception_immunity: f64,
}

/// Optional custom weights for EPS dimensions (must sum to 1.0).
pub struct EpsWeights {
    pub speed: f64,
    pub accuracy_under_pressure: f64,
    pub transfer_ability: f64,
    pub novelty_handling: f64,
    pub misconception_immunity: f64,
}

impl Default for EpsWeights {
    fn default() -> Self {
        Self {
            speed: 0.20,
            accuracy_under_pressure: 0.20,
            transfer_ability: 0.20,
            novelty_handling: 0.20,
            misconception_immunity: 0.20,
        }
    }
}

/// Compute the Elite Performance Score.
///
/// # Source
/// idea5.txt; backend_supplement §6
///
/// # Parameters
/// - `inputs`: performance dimension values
/// - `weights`: dimension weights (use `EpsWeights::default()` for equal weighting)
pub fn elite_performance_score(inputs: &EpsInputs, weights: &EpsWeights) -> BasisPoints {
    let raw = weights.speed * inputs.speed
        + weights.accuracy_under_pressure * inputs.accuracy_under_pressure
        + weights.transfer_ability * inputs.transfer_ability
        + weights.novelty_handling * inputs.novelty_handling
        + weights.misconception_immunity * inputs.misconception_immunity;

    to_bp(raw)
}
```

---

## 4.16 RiseTransformationReadiness — Stage Gate Checks

Each Rise Mode stage transition requires meeting a pair of score thresholds. The gate function returns `true` only when ALL required conditions are satisfied.

**Thresholds:**
```
Rescue → Stabilize:
    foundation_score ≥ 4500 AND misconception_density < 3500

Stabilize → Accelerate:
    recall_score ≥ 6000 AND accuracy_score ≥ 6500

Accelerate → Dominate:
    speed_score ≥ 7500 AND pressure_stability_score ≥ 7000
```

```rust
/// Rise Mode transformation stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiseStage {
    /// Stop the bleeding; find root gaps; build first wins.
    Rescue,
    /// Make correct thinking repeatable; scaffolded concept clusters.
    Stabilize,
    /// Speed + independence; timed drills; mixed topics; pressure mode.
    Accelerate,
    /// Outperform top students; trap questions; elite variants; speed + accuracy.
    Dominate,
}

/// The student's current Rise Mode score profile (all in basis points).
pub struct RiseScores {
    pub foundation_score: BasisPoints,
    pub recall_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub accuracy_score: BasisPoints,
    pub pressure_stability_score: BasisPoints,
    pub misconception_density: BasisPoints,
    pub momentum_score: BasisPoints,
    pub transformation_readiness: BasisPoints,
}

/// Check whether the student is ready to advance to the next Rise stage.
///
/// # Source
/// idea4.txt; backend_supplement §4.1, §4.5
///
/// # Returns
/// `true` if all threshold conditions for the current → next transition are met.
pub fn rise_stage_gate(current_stage: &RiseStage, scores: &RiseScores) -> bool {
    match current_stage {
        RiseStage::Rescue => {
            // Gate: foundation_score ≥ 4500 AND misconception_density < 3500
            scores.foundation_score >= 4_500
                && scores.misconception_density < 3_500
        }
        RiseStage::Stabilize => {
            // Gate: recall_score ≥ 6000 AND accuracy_score ≥ 6500
            scores.recall_score >= 6_000
                && scores.accuracy_score >= 6_500
        }
        RiseStage::Accelerate => {
            // Gate: speed_score ≥ 7500 AND pressure_stability_score ≥ 7000
            scores.speed_score >= 7_500
                && scores.pressure_stability_score >= 7_000
        }
        RiseStage::Dominate => {
            // Already at the final stage; no further transition.
            false
        }
    }
}

/// Return the next Rise stage given the current one, if a transition exists.
pub fn rise_next_stage(current: &RiseStage) -> Option<RiseStage> {
    match current {
        RiseStage::Rescue     => Some(RiseStage::Stabilize),
        RiseStage::Stabilize  => Some(RiseStage::Accelerate),
        RiseStage::Accelerate => Some(RiseStage::Dominate),
        RiseStage::Dominate   => None,
    }
}
```

---

# PART 5: ALL BUSINESS RULES & CONSTANTS

---

## 5.1 Mastery Gate Rules

Mastery is stored and compared as `BasisPoints` (0–10000). A student **cannot progress** to a dependent topic until the prerequisite topic meets the minimum gate threshold.

```rust
/// Named mastery bands with their basis-point boundaries.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MasteryBand {
    /// 0–3999: Blocked. Must resolve prerequisites before progressing.
    Blocked,
    /// 4000–5499: Emerging. Limited progression allowed; no dependency unlock.
    Emerging,
    /// 5500–6999: Functional. Normal progression; dependency topics may unlock.
    Functional,
    /// 7000–8499: Stable. Confident performance; exam-weight topics can be tackled.
    Stable,
    /// 8500–8999: ExamReady. Suitable for final exam conditioning and mock inclusion.
    ExamReady,
    /// 9000–10000: Mastered. Full elite access permitted.
    Mastered,
}

impl MasteryBand {
    pub fn classify(mastery_bp: BasisPoints) -> Self {
        match mastery_bp {
            m if m >= 9_000 => Self::Mastered,
            m if m >= 8_500 => Self::ExamReady,
            m if m >= 7_000 => Self::Stable,
            m if m >= 5_500 => Self::Functional,
            m if m >= 4_000 => Self::Emerging,
            _               => Self::Blocked,
        }
    }

    /// Returns true if the student may advance to dependent topics at this band.
    pub fn allows_progression(&self) -> bool {
        !matches!(self, Self::Blocked | Self::Emerging)
    }

    /// Returns true if the topic is accessible for standard practice sessions.
    pub fn allows_standard_session(&self) -> bool {
        !matches!(self, Self::Blocked)
    }
}

/// Hard progression gate: mastery below this threshold blocks dependent topics entirely.
pub const MASTERY_GATE_MINIMUM: BasisPoints = 4_000;

/// Gate for dependency unlock (must reach Functional to unlock next topic).
pub const MASTERY_GATE_DEPENDENCY_UNLOCK: BasisPoints = 5_500;

/// Gate for mock inclusion (topic should not appear in mock if below this).
pub const MASTERY_GATE_MOCK_INCLUSION: BasisPoints = 4_000;

/// Gate for exam readiness claim on a topic.
pub const MASTERY_GATE_EXAM_READY: BasisPoints = 8_500;

/// Gate for elite mode access per topic.
pub const MASTERY_GATE_ELITE: BasisPoints = 8_500;
```

---

## 5.2 Evidence Weighting Rules

These rules are applied whenever computing the weight of a new attempt before it enters the EMA mastery update pipeline. All rules compose multiplicatively.

| Rule | Multiplier |
|------|-----------|
| Baseline (first attempt) | 1.00 |
| Each hint used | × 0.50 per hint (max 3 halvings → min 0.125 from hints alone) |
| Guided support level | × 0.70 |
| Heavily guided support level | × 0.40 |
| Transfer context (new form/wording) | × 1.30 |
| Delayed recall (≥ 24h since last exposure) | × 1.50 |
| Same-day repeat attempt on same question | × 0.60 |
| Correct answer + "guessed" confidence | × 0.50 |

**Caps:**
- Final effective weight is clamped to [0.05, 2.0] before use.
- A single attempt cannot produce a weight of 0.0 (the system always registers some evidence).

**Rule: Minimum Evidence Before Mastery Update**
- At least 3 attempts must have been recorded before any mastery band transition is applied.
- A single successful attempt NEVER grants a mastery upgrade by itself.
- See §5.4 for the full session minimum rules.

```rust
pub const EVIDENCE_MIN_ATTEMPTS_FOR_UPDATE: u32 = 3;
pub const EVIDENCE_HINT_FACTOR: f64 = 0.50;
pub const EVIDENCE_GUIDED_FACTOR: f64 = 0.70;
pub const EVIDENCE_HEAVILY_GUIDED_FACTOR: f64 = 0.40;
pub const EVIDENCE_TRANSFER_BOOST: f64 = 1.30;
pub const EVIDENCE_DELAYED_RECALL_BOOST: f64 = 1.50;
pub const EVIDENCE_SAME_DAY_REPEAT_FACTOR: f64 = 0.60;
pub const EVIDENCE_GUESSED_CORRECT_FACTOR: f64 = 0.50;
pub const EVIDENCE_WEIGHT_MIN: f64 = 0.05;
pub const EVIDENCE_WEIGHT_MAX: f64 = 2.00;
```

---

## 5.3 Memory Scheduling Rules

The memory scheduling system assigns every knowledge unit to a review state and computes the `next_review_at` timestamp accordingly.

**Review Intervals by Memory State:**

| Memory State | Review Interval | Action |
|-------------|----------------|--------|
| Fresh | 1 day | Schedule normal review |
| Consolidating | 3 days | Schedule normal review |
| Stable | 7 days | Schedule maintenance review |
| AtRisk | 2 days | Schedule rescue review |
| Decaying | Today (0 days) | Schedule immediate rescue |
| Collapsed | Today (0 days) | Block session; require rescue before other topics |

**Spaced Recheck Ladder (after successful recall at each step):**
```
1 day → 3 days → 7 days → 14 days → 30 days
```

**Promotion and Demotion:**
- After **successful recall**: promote the unit one memory state level up.
- After **failed recall**: demote the unit one memory state level down and schedule rescue.
- After **failed recall at Critical/Collapsed**: block the session until rescue is completed.

```rust
/// Days until next review for a given memory state.
/// Returns `0` for states requiring immediate action.
pub fn next_review_days(state: &MemoryState) -> u32 {
    match state {
        MemoryState::Fresh         => 1,
        MemoryState::Consolidating => 3,
        MemoryState::Stable        => 7,
        MemoryState::AtRisk        => 2,
        MemoryState::Decaying      => 0,
        MemoryState::Collapsed     => 0,
    }
}

/// Promote a memory state after successful recall.
pub fn memory_promote(state: &MemoryState) -> MemoryState {
    match state {
        MemoryState::Collapsed     => MemoryState::AtRisk,
        MemoryState::Decaying      => MemoryState::AtRisk,
        MemoryState::AtRisk        => MemoryState::Fresh,
        MemoryState::Fresh         => MemoryState::Consolidating,
        MemoryState::Consolidating => MemoryState::Stable,
        MemoryState::Stable        => MemoryState::Stable, // Already at top
    }
}

/// Demote a memory state after failed recall.
pub fn memory_demote(state: &MemoryState) -> MemoryState {
    match state {
        MemoryState::Stable        => MemoryState::Consolidating,
        MemoryState::Consolidating => MemoryState::AtRisk,
        MemoryState::Fresh         => MemoryState::AtRisk,
        MemoryState::AtRisk        => MemoryState::Decaying,
        MemoryState::Decaying      => MemoryState::Collapsed,
        MemoryState::Collapsed     => MemoryState::Collapsed, // Cannot go lower
    }
}

/// Returns true if this memory state requires the session to be blocked
/// until a rescue sequence is completed.
pub fn requires_session_block(state: &MemoryState) -> bool {
    matches!(state, MemoryState::Collapsed)
}
```

---

## 5.4 Session Rules

These rules govern every study session across all modes.

| Rule | Value |
|------|-------|
| Minimum attempts before any mastery update | 3 |
| No mastery from single success | Hard rule — one correct answer never triggers upgrade |
| Maximum session duration | 90 minutes |
| Session must be formally closed to count time | If client crashes, time is NOT counted |
| Abandoned session — time | NOT counted toward daily study time |
| Abandoned session — attempts | Still recorded (evidence is preserved) |
| Minimum sessions spread for readiness claim | ≥ 3 separate sessions on the topic |
| Maximum active questions in session queue | 20 |

```rust
pub const SESSION_MIN_ATTEMPTS: u32 = 3;
pub const SESSION_MAX_DURATION_MINUTES: u32 = 90;
pub const SESSION_MIN_SESSIONS_FOR_READINESS: u32 = 3;
pub const SESSION_MAX_QUEUE_SIZE: u32 = 20;

/// A session is only eligible for time accounting if it has been formally closed.
/// `is_formally_closed` must be set to true by the session end handler.
/// Abandoned sessions preserve attempts but contribute 0 minutes to study time.
pub fn session_time_is_countable(is_formally_closed: bool) -> bool {
    is_formally_closed
}
```

---

## 5.5 Coach State Machine — All 14 States and Valid Transitions

The `CoachLifecycleState` drives every screen the learner sees. Transitions are triggered by specific events and can only occur along valid edges.

**States:**

| # | State | Meaning |
|---|-------|---------|
| 1 | `OnboardingRequired` | Account created but profile setup not complete |
| 2 | `SubjectSelectionRequired` | Profile done; no subjects selected yet |
| 3 | `DiagnosticRequired` | Subjects selected; baseline diagnostic not yet run |
| 4 | `ContentReadinessRequired` | Diagnostic done; content packs not installed |
| 5 | `PlanGenerationRequired` | Content ready; study plan not yet generated |
| 6 | `ReadyForTodayMission` | Plan exists; awaiting student to start today's mission |
| 7 | `MissionInProgress` | Student is actively working a session |
| 8 | `MissionReviewRequired` | Session completed; post-session analysis not yet shown |
| 9 | `RepairRequired` | Coach has detected a topic that must be repaired before continuing |
| 10 | `BlockedOnTopic` | Student's mastery < 4000 bp on a prerequisite; cannot advance |
| 11 | `PlanAdjustmentRequired` | Performance has diverged enough to require plan rebalancing |
| 12 | `ReviewDay` | It is a scheduled spaced-repetition review day |
| 13 | `ExamMode` | Exam is ≤ 14 days away; final conditioning active |
| 14 | `StalledNoContent` | No content packs available and no network to fetch them |

**Valid Transitions Table:**

| From State | → To State | Trigger Condition |
|-----------|-----------|------------------|
| `OnboardingRequired` | `SubjectSelectionRequired` | Onboarding profile steps completed |
| `SubjectSelectionRequired` | `DiagnosticRequired` | At least one subject selected |
| `DiagnosticRequired` | `ContentReadinessRequired` | Baseline diagnostic session completed |
| `ContentReadinessRequired` | `PlanGenerationRequired` | Minimum content pack installed (SufficiencyLevel ≥ Amber) |
| `ContentReadinessRequired` | `StalledNoContent` | No content packs available and cannot be fetched |
| `PlanGenerationRequired` | `ReadyForTodayMission` | Study plan generated and approved |
| `ReadyForTodayMission` | `MissionInProgress` | Student starts today's session |
| `ReadyForTodayMission` | `ReviewDay` | Scheduler determines today is a review day |
| `ReadyForTodayMission` | `ExamMode` | Exam date is ≤ 14 days away |
| `MissionInProgress` | `MissionReviewRequired` | Session formally closed |
| `MissionInProgress` | `ReadyForTodayMission` | Session abandoned (attempts saved, time not counted) |
| `MissionReviewRequired` | `ReadyForTodayMission` | Post-session review acknowledged by student |
| `MissionReviewRequired` | `RepairRequired` | Coach detects critical failure requiring immediate repair |
| `MissionReviewRequired` | `BlockedOnTopic` | Topic mastery < 4000 bp on a prerequisite topic |
| `MissionReviewRequired` | `PlanAdjustmentRequired` | Performance drift exceeds replan threshold |
| `RepairRequired` | `ReadyForTodayMission` | Repair sequence completed; mastery ≥ 4000 bp |
| `BlockedOnTopic` | `ReadyForTodayMission` | Prerequisite mastery restored to ≥ 4000 bp |
| `PlanAdjustmentRequired` | `ReadyForTodayMission` | Plan rebalanced and accepted |
| `ReviewDay` | `MissionInProgress` | Student starts review session |
| `ReviewDay` | `ReadyForTodayMission` | Review session completed |
| `ExamMode` | `MissionInProgress` | Student starts final conditioning session |
| `ExamMode` | `ReadyForTodayMission` | Exam conditioning session completed |
| `StalledNoContent` | `ContentReadinessRequired` | Content pack installed |
| Any state | `ExamMode` | Exam date becomes ≤ 14 days away (override transition) |

```rust
impl CoachLifecycleState {
    /// Determine if a transition to `next` is a valid edge from `self`.
    pub fn can_transition_to(&self, next: &CoachLifecycleState) -> bool {
        use CoachLifecycleState::*;
        matches!(
            (self, next),
            (OnboardingRequired,       SubjectSelectionRequired)
            | (SubjectSelectionRequired,  DiagnosticRequired)
            | (DiagnosticRequired,        ContentReadinessRequired)
            | (ContentReadinessRequired,  PlanGenerationRequired)
            | (ContentReadinessRequired,  StalledNoContent)
            | (PlanGenerationRequired,    ReadyForTodayMission)
            | (ReadyForTodayMission,      MissionInProgress)
            | (ReadyForTodayMission,      ReviewDay)
            | (ReadyForTodayMission,      ExamMode)
            | (MissionInProgress,         MissionReviewRequired)
            | (MissionInProgress,         ReadyForTodayMission)
            | (MissionReviewRequired,     ReadyForTodayMission)
            | (MissionReviewRequired,     RepairRequired)
            | (MissionReviewRequired,     BlockedOnTopic)
            | (MissionReviewRequired,     PlanAdjustmentRequired)
            | (RepairRequired,            ReadyForTodayMission)
            | (BlockedOnTopic,            ReadyForTodayMission)
            | (PlanAdjustmentRequired,    ReadyForTodayMission)
            | (ReviewDay,                 MissionInProgress)
            | (ReviewDay,                 ReadyForTodayMission)
            | (ExamMode,                  MissionInProgress)
            | (ExamMode,                  ReadyForTodayMission)
            | (StalledNoContent,          ContentReadinessRequired)
            // Global override: any state can transition to ExamMode when exam ≤ 14 days
            | (_, ExamMode)
        )
    }
}
```

---

## 5.6 Mock Orchestration Rules — 6 Policies

These policies are applied in order during mock compilation. A question/topic must pass all applicable policies to be included in a mock.

### Policy 1: Fix Prerequisites First
- A topic **must not appear in any mock** if its prerequisite topics have mastery < 5000 bp (Emerging band).
- The mock engine checks the full prerequisite chain, not just immediate parents.
- Exception: `MockType::Diagnostic` bypasses this rule to allow gap detection.

```rust
pub const MOCK_PREREQUISITE_GATE: BasisPoints = 5_000;

/// Returns true if a topic is eligible for mock inclusion based on prerequisite mastery.
/// `prerequisite_masteries`: the mastery scores of all prerequisite topics in the chain.
/// `mock_type`: Diagnostic mocks bypass this gate.
pub fn mock_prerequisite_gate_passes(
    prerequisite_masteries: &[BasisPoints],
    mock_type: &MockType,
) -> bool {
    if matches!(mock_type, MockType::Diagnostic) {
        return true;
    }
    prerequisite_masteries.iter().all(|&m| m >= MOCK_PREREQUISITE_GATE)
}
```

### Policy 2: Prioritize Unresolved Misconceptions
- Any topic with active misconception tags has its `MockOrchestrationScore` multiplied by a 1.25× misconception urgency boost.
- "Active" means the misconception has been triggered ≥ 2 times in the last 10 attempts and has not been resolved.

```rust
pub const MISCONCEPTION_URGENCY_MULTIPLIER: f64 = 1.25;
pub const MISCONCEPTION_ACTIVE_TRIGGER_THRESHOLD: u32 = 2;
pub const MISCONCEPTION_ACTIVE_WINDOW: u32 = 10; // last N attempts
```

### Policy 3: Enforce Coverage
- Within any 3-mock window, every syllabus strand must be represented at least once.
- A strand that has been absent from the last 3 mocks receives a `coverage_gap` boost of 0.30 (added to its MockOrchestrationScore input).

```rust
pub const COVERAGE_ENFORCEMENT_WINDOW: u32 = 3; // mocks
pub const COVERAGE_GAP_BOOST: f64 = 0.30;        // added to coverage_gap input when triggered
```

### Policy 4: Rotate Representations
- No more than 3 consecutive questions of the same representation type (text / diagram / graph / table) may appear within a single mock.
- When the limit is reached, the next question must use a different representation type.

```rust
pub const MAX_CONSECUTIVE_SAME_REPRESENTATION: u32 = 3;
```

### Policy 5: Increase Exam Simulation Near Exam
- When ≤ 14 days remain until the exam date, the default mock type switches to `MockType::FinalExam`.
- `FinalExam` mocks use 100% blueprint-matching question selection, full time pressure, and no scaffolding.

```rust
pub const EXAM_MODE_TRIGGER_DAYS: i64 = 14;

/// Returns the recommended mock type given days remaining until exam.
pub fn recommended_mock_type(days_to_exam: i64) -> MockType {
    if days_to_exam <= EXAM_MODE_TRIGGER_DAYS {
        MockType::FinalExam
    } else {
        MockType::Forecast
    }
}
```

### Policy 6: Block Repetitive Similarity (Anti-Repeat)
- A question that shares the same `question_family_id` as a question already attempted in the current mock receives a 0.25× `anti_repeat_penalty` applied to its MockSelectionScore.
- Questions attempted in the **immediately preceding mock session** also receive a reduced anti-repeat penalty of 0.15×.

```rust
pub const ANTI_REPEAT_SAME_MOCK_PENALTY: f64 = 0.25;
pub const ANTI_REPEAT_PREV_MOCK_PENALTY: f64 = 0.15;
```

---

## 5.7 Readiness Proof Rules

A readiness claim is a formal statement that the student is prepared to sit the exam. It requires passing ALL of the following checks.

| Check | Threshold |
|-------|-----------|
| Minimum attempts per topic | ≥ 15 attempts on the topic |
| Minimum session spread per topic | Attempts spread across ≥ 3 separate sessions |
| Timed evidence requirement | ≥ 5 timed attempts per topic |
| Danger zone check | NO exam-critical topic with mastery < 4000 bp |
| Coverage threshold | ≥ 80% of syllabus topics at ≥ Functional mastery (5500 bp) |
| No critical danger zones | All topics flagged `is_exam_critical = true` must have mastery ≥ 5500 bp |
| Mock requirement | ≥ 1 full mock completed (any `MockType`) |

```rust
pub const READINESS_MIN_ATTEMPTS_PER_TOPIC: u32 = 15;
pub const READINESS_MIN_SESSIONS_PER_TOPIC: u32 = 3;
pub const READINESS_MIN_TIMED_ATTEMPTS: u32 = 5;
pub const READINESS_MIN_COVERAGE_PERCENT: f64 = 0.80;   // 80% of topics
pub const READINESS_MIN_COVERAGE_MASTERY: BasisPoints = 5_500; // Functional band
pub const READINESS_CRITICAL_TOPIC_GATE: BasisPoints = 5_500;
pub const READINESS_MIN_MOCKS_COMPLETED: u32 = 1;

/// Result of a readiness proof check.
#[derive(Debug)]
pub struct ReadinessProofResult {
    pub passed: bool,
    pub failures: Vec<ReadinessProofFailure>,
}

#[derive(Debug)]
pub enum ReadinessProofFailure {
    InsufficientAttempts { topic_id: i64, actual: u32 },
    InsufficientSessionSpread { topic_id: i64, actual: u32 },
    InsufficientTimedAttempts { topic_id: i64, actual: u32 },
    DangerZonePresent { topic_id: i64, mastery: BasisPoints },
    CoverageBelow80 { actual_percent: f64 },
    CriticalTopicBelow { topic_id: i64, mastery: BasisPoints },
    NoMockCompleted,
}
```

---

## 5.8 Parent Alert Trigger Rules

The Parent Alert Engine evaluates these rules after every session completion and on a daily scheduled check. Alerts are generated when any condition is met. Alerts are deduplicated within a 24-hour window (same alert type + same student = only one alert per day).

| Alert Type | Trigger Condition | Severity |
|-----------|-------------------|---------|
| `inactivity` | No study for ≥ 3 consecutive days | `watch` |
| `decline` | Mastery dropped ≥ 1000 bp in any subject over a 7-day window | `urgent` |
| `exam_near` | ≤ 14 days to exam date AND readiness score < 6500 bp | `urgent` |
| `mock_overdue` | ≥ 14 days since last mock AND exam date < 60 days away | `watch` |
| `subject_lagging` | One subject ≥ 2000 bp below the student's average mastery across subjects | `watch` |
| `misconception_repeat` | Same concept failed ≥ 3 times in last 5 attempts | `watch` |
| `false_confidence` | High-confidence wrong answers on exam-critical topic in last session | `watch` |

```rust
pub const ALERT_INACTIVITY_DAYS: u32 = 3;
pub const ALERT_DECLINE_BP_THRESHOLD: BasisPoints = 1_000;
pub const ALERT_DECLINE_WINDOW_DAYS: u32 = 7;
pub const ALERT_EXAM_NEAR_DAYS: i64 = 14;
pub const ALERT_EXAM_NEAR_READINESS_GATE: BasisPoints = 6_500;
pub const ALERT_MOCK_OVERDUE_DAYS: u32 = 14;
pub const ALERT_MOCK_OVERDUE_EXAM_WINDOW_DAYS: i64 = 60;
pub const ALERT_SUBJECT_LAG_BP: BasisPoints = 2_000;
pub const ALERT_MISCONCEPTION_REPEAT_COUNT: u32 = 3;
pub const ALERT_MISCONCEPTION_REPEAT_WINDOW: u32 = 5; // last N attempts
```

**Alert Deduplication Rule:**
- If an alert of the same `alert_type` for the same `student_account_id` was created within the last 24 hours, skip creating a new one.
- Parent sees a read/unread flag; unread alerts display a badge count.

---

## 5.9 Game Engine Rules

### 5.9.1 MindStack (Tetris) — Scoring and Progression Rules

MindStack maps academic performance to Tetris mechanics. The quality of each answer determines how much control the player has over the falling block.

**Control Level by Answer Quality:**

| Answer Quality | Lateral Movement | Rotation | Reshape | Gravity |
|---------------|-----------------|---------|--------|--------|
| No mastery (wrong, no hints) | None | None | None | Fast (normal) |
| Partial (wrong + hint, or low-confidence correct) | 1–2 moves | None | None | Normal speed |
| Good (correct, independent) | Full | Full | None | Normal speed |
| Excellent (correct + fast + streak) | Full | Full | 1 reshape option | Slowed |

**Question-to-Power Mapping:**

| Question Type Answered Correctly | Power Unlocked |
|---------------------------------|----------------|
| Recall | Unlock lateral movement |
| Concept understanding | Unlock rotation |
| Reasoning | Unlock reshaping |
| Fast answer streak (3+ correct fast) | Temporary gravity slowdown + board-clear bomb + shield |

**Streak Bonuses:**

| Streak | Bonus |
|--------|-------|
| 3 consecutive correct | Stronger pull / score multiplier activated |
| 5 consecutive correct | "Overdrive" state — gravity halved, 2× score multiplier |

**Mercy Rules (Preventing Instant Loss):**
1. One wrong answer does not immediately lose the game — the block still falls but with reduced control.
2. The player may recover full control on the very next correct answer.
3. Early game levels have a grace zone (wider board clearing threshold).
4. Streak forgiveness: 4 correct + 1 miss does not cancel the streak multiplier; requires 2 misses in a row.
5. Rescue tokens are available (limited per session): Stabilize Block, Retry Answer, Slow Gravity.

**Session Score Formula:**
```rust
/// MindStack session score computation.
///
/// `base_points` = questions answered correctly × base_points_per_question
/// `streak_multiplier` = 1.0 + (0.1 × streak_length), capped at 2.0
/// `level_multiplier` = 1.0 + (0.05 × current_level)
/// `time_bonus` = max(0, target_time_seconds - actual_time_seconds) × 2
pub fn mindstack_session_score(
    correct_answers: u32,
    base_points_per_question: u32,
    streak_length: u32,
    current_level: u32,
    time_bonus: u32,
) -> u64 {
    let base = (correct_answers * base_points_per_question) as f64;
    let streak_multiplier = (1.0 + 0.1 * streak_length as f64).min(2.0);
    let level_multiplier = 1.0 + 0.05 * current_level as f64;
    ((base * streak_multiplier * level_multiplier) as u64) + time_bonus as u64
}
```

---

### 5.9.2 Tug of War (MindPull) — Scoring and Progression Rules

Two-player or player-vs-AI competitive academic game. Correct answers pull the rope toward the player's side; wrong answers let it slip back.

**Rope Zones:**

| Zone | Description | Effect |
|------|-------------|--------|
| `NeutralZone` | Starting position | Neither side has advantage |
| `PressureZone` | Opponent has slight advantage | Player must answer faster |
| `RecoveryZone` | Player has slight advantage | Maintain with correct answers |
| `VictoryZone` | Player has decisive advantage | One more correct = win |
| `CollapseZone` | Opponent has decisive advantage | One more wrong = loss |

**Pull Mechanics:**

| Answer Outcome | Rope Movement |
|---------------|---------------|
| Correct (normal) | +1 unit toward player |
| Correct (3-answer streak) | +2 units toward player |
| Correct (5-answer streak "Overdrive") | +3 units toward player ("Overdrive Pull") |
| Wrong | −1 unit (rope slips toward opponent) |
| Timed out | −1 unit |

**Power-Ups (available during game):**

| Power-Up | Effect |
|---------|--------|
| Freeze Slip | Next wrong answer does not move the rope |
| Double Pull | Next correct answer counts as 2 pulls |
| Time Shield | Adds 3 extra seconds to next question timer |
| Hint Rope | Uses a small clue; if answered correctly, only +0.5 units (reduced reward) |
| Misconception Scan | Reveals which distractor is the trap; reduces pull reward by 20% if used |

**Win Condition:** Rope reaches `VictoryZone` end boundary. Default rope length = 10 units. Win requires pulling 5+ units from centre.

**Session Score Formula:**
```rust
/// Tug of War session score.
/// Combines accuracy, speed, streak performance, and power-up efficiency.
///
/// `net_pulls` = total pulls toward player minus slips
/// `avg_response_ms` = average response time in milliseconds
/// `streak_best` = best consecutive-correct streak in session
/// `power_ups_used` = number of power-ups consumed (subtracts from score)
pub fn tug_of_war_session_score(
    net_pulls: i32,
    correct_count: u32,
    total_count: u32,
    streak_best: u32,
    power_ups_used: u32,
) -> BasisPoints {
    if total_count == 0 {
        return 0;
    }
    let accuracy = correct_count as f64 / total_count as f64;
    let pull_contribution = (net_pulls.max(0) as f64 / 10.0).min(1.0); // normalise to rope length
    let streak_bonus = (streak_best as f64 * 0.02).min(0.20);
    let power_penalty = (power_ups_used as f64 * 0.02).min(0.10);

    let raw = 0.50 * accuracy + 0.30 * pull_contribution + streak_bonus - power_penalty;
    to_bp(raw.max(0.0))
}
```

---

## 5.10 Elite Mode Gate Rules

Elite Mode is the highest-intensity practice tier. Access requires meeting both an entitlement and a performance threshold.

### Entry Requirements

| Requirement | Threshold |
|-------------|-----------|
| Entitlement tier | Must be `EntitlementTier::Elite` (premium subscriber) |
| Mastery breadth | mastery ≥ 8500 bp (ExamReady band) across ≥ 70% of enrolled topics |
| Entry path A (hard gate) | Both conditions above must be true simultaneously |
| Entry path B (self-select) | Any student may attempt Elite; the system calibrates them to `EliteTier::Foundation` with a warning displayed |

```rust
/// Entitlement tiers for feature gating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementTier {
    Free,
    Standard,
    Elite,
}

pub const ELITE_MASTERY_GATE: BasisPoints = 8_500;
pub const ELITE_TOPIC_COVERAGE_PERCENT: f64 = 0.70; // 70% of enrolled topics

/// Check if a student qualifies for Elite Mode (hard gate).
///
/// `topic_masteries`: mastery scores for all enrolled topics.
pub fn elite_mode_gate_passes(
    entitlement: &EntitlementTier,
    topic_masteries: &[BasisPoints],
) -> bool {
    if !matches!(entitlement, EntitlementTier::Elite) {
        return false;
    }
    if topic_masteries.is_empty() {
        return false;
    }
    let qualifying = topic_masteries.iter()
        .filter(|&&m| m >= ELITE_MASTERY_GATE)
        .count();
    let coverage = qualifying as f64 / topic_masteries.len() as f64;
    coverage >= ELITE_TOPIC_COVERAGE_PERCENT
}
```

### EPS Scoring and Benchmarking

- The `ElitePerformanceScore` (see §4.15) is computed after every Elite session.
- Benchmarks are maintained per subject as percentile bands (Top 10% / Top 25% / Top 50%).
- A student must maintain EPS ≥ 7000 bp across 3 consecutive elite sessions to be promoted to the next `EliteTier`.

```rust
pub const EPS_PROMOTION_GATE: BasisPoints = 7_000;
pub const EPS_PROMOTION_SESSIONS_REQUIRED: u32 = 3;

/// Elite tier levels within Elite Mode itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EliteTier {
    /// Accuracy discipline, cleaner thinking, light pressure.
    Foundation,
    /// Mixed challenge, trap handling, faster reasoning.
    Core,
    /// Ruthless precision, sustained perfection, exam-grade intensity.
    Apex,
    /// Perfect streaks, no-hint runs, timed boss sets, elite rankings.
    Legend,
}
```

### Elite Daily Missions

- Every Elite-tier student is assigned a **daily high-stakes challenge** from the mission pool.
- Missions are selected by `EPS + MockSelectionScore` joint ranking.
- Completing a daily mission awards a `legend_point`; failing deducts one.
- Badges awarded for milestone legend-point totals (see §6.7 of the supplement).
- Elite mission types include: Perfect Run (one error collapses run), Speed Authority (all correct under target time), Trap Hunter (all trap questions answered correctly), Examiner-Proof (all past-question-pattern questions correct).

```rust
pub const ELITE_LEGEND_POINTS_AWARDED: i32 = 1;
pub const ELITE_LEGEND_POINTS_DEDUCTED: i32 = -1;

/// Elite badge thresholds (cumulative legend points).
pub const BADGE_PRECISION_BEAST: i32 = 10;
pub const BADGE_TRAP_HUNTER: i32 = 25;
pub const BADGE_SPEED_AUTHORITY: i32 = 50;
pub const BADGE_PERFECT_RUN: i32 = 75;
pub const BADGE_DISTINCTION_MACHINE: i32 = 100;
pub const BADGE_NO_HINT_MASTER: i32 = 150;
pub const BADGE_EXAMINER_PROOF: i32 = 200;
```

---

## Summary Constants Reference

```rust
// ── Mastery bands ────────────────────────────────────────────────
pub const MASTERY_BLOCKED_MAX:     BasisPoints = 3_999;
pub const MASTERY_EMERGING_MIN:    BasisPoints = 4_000;
pub const MASTERY_EMERGING_MAX:    BasisPoints = 5_499;
pub const MASTERY_FUNCTIONAL_MIN:  BasisPoints = 5_500;
pub const MASTERY_FUNCTIONAL_MAX:  BasisPoints = 6_999;
pub const MASTERY_STABLE_MIN:      BasisPoints = 7_000;
pub const MASTERY_STABLE_MAX:      BasisPoints = 8_499;
pub const MASTERY_EXAM_READY_MIN:  BasisPoints = 8_500;
pub const MASTERY_EXAM_READY_MAX:  BasisPoints = 8_999;
pub const MASTERY_MASTERED_MIN:    BasisPoints = 9_000;

// ── EMA ───────────────────────────────────────────────────────────
pub const EMA_ALPHA: f64 = 0.3;

// ── Rise stage gates ─────────────────────────────────────────────
pub const RISE_GATE_RESCUE_FOUNDATION:    BasisPoints = 4_500;
pub const RISE_GATE_RESCUE_MISCONCEPTION: BasisPoints = 3_500; // must be BELOW
pub const RISE_GATE_STABILIZE_RECALL:     BasisPoints = 6_000;
pub const RISE_GATE_STABILIZE_ACCURACY:   BasisPoints = 6_500;
pub const RISE_GATE_ACCELERATE_SPEED:     BasisPoints = 7_500;
pub const RISE_GATE_ACCELERATE_PRESSURE:  BasisPoints = 7_000;

// ── Coach state / session ────────────────────────────────────────
pub const TOPIC_BLOCK_ACCURACY_THRESHOLD:   f64 = 0.40;  // < 40% after 2 sessions → block
pub const TOPIC_UNBLOCK_ACCURACY_THRESHOLD: f64 = 0.60;  // ≥ 60% → unblock
pub const MISCONCEPTION_FORCE_REPAIR_COUNT: u32 = 3;     // recurs 3 times → queue repair
pub const SESSION_MAX_DURATION_MINUTES:     u32 = 90;
pub const SESSION_MIN_ATTEMPTS:             u32 = 3;

// ── Forecasting ───────────────────────────────────────────────────
pub const FORECAST_HIGH_BAND:     BasisPoints = 7_000;
pub const FORECAST_MEDIUM_BAND:   BasisPoints = 4_500;
pub const FORECAST_SURPRISE_BAND: BasisPoints = 3_000;

// ── Readiness ─────────────────────────────────────────────────────
pub const READINESS_COVERAGE_THRESHOLD:  f64         = 0.80;
pub const READINESS_MIN_MOCK_COUNT:      u32         = 1;
pub const READINESS_EXAM_NEAR_DAYS:      i64         = 14;
pub const READINESS_CRITICAL_TOPIC_GATE: BasisPoints = 4_000;

// ── Parent alerts ─────────────────────────────────────────────────
pub const ALERT_INACTIVITY_DAYS:          u32         = 3;
pub const ALERT_DECLINE_THRESHOLD:        BasisPoints = 1_000;
pub const ALERT_EXAM_NEAR_READINESS_GATE: BasisPoints = 6_500;
pub const ALERT_SUBJECT_LAG_THRESHOLD:    BasisPoints = 2_000;
```

---

*End of Part 4 and Part 5.*


---

# eCoach — Backend Implementation Plan: Parts 6, 7, and 8
## Agent 1 Output | Tauri Commands, Phased Delivery, Master Reference

---

# PART 6: ALL TAURI COMMANDS — FULL API SURFACE

## Shared Error Type

All command modules return `Result<T, CommandError>`. The error type is defined once in `ecoach-commands/src/errors.rs` and re-exported by every command module.

```rust
// ecoach-commands/src/errors.rs

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum CommandError {
    NotFound(String),
    Unauthorized(String),
    ValidationError(String),
    DatabaseError(String),
    BusinessRuleViolation(String),
    InternalError(String),
}

impl From<IdentityError> for CommandError { ... }
impl From<StudentModelError> for CommandError { ... }
impl From<CoachBrainError> for CommandError { ... }
// one From impl per domain error type
```

## Command Pattern (Standard)

Every command follows this shape:

```rust
#[tauri::command]
pub async fn command_name(
    state: tauri::State<'_, AppState>,
    input: InputDto,
) -> Result<OutputDto, CommandError> {
    let service = state.service_name();
    let result = service.method(input.into()).await
        .map_err(CommandError::from)?;
    Ok(result.into())
}
```

`AppState` holds `Arc`-wrapped service handles. All services are injected at startup in `src-tauri/src/main.rs`.

---

## 6.1 identity_commands.rs

**File:** `ecoach-commands/src/identity_commands.rs`
**Owns:** Account lifecycle, PIN auth, parent-student linking

### Input / Output DTOs

```rust
#[derive(Deserialize)]
pub struct CreateAccountInput {
    pub account_type: String,       // "student" | "parent" | "admin"
    pub display_name: String,
    pub pin: String,                // raw PIN, hashed in service
    pub avatar_path: Option<String>,
    pub entitlement_tier: Option<String>,
}

#[derive(Serialize)]
pub struct AccountDto {
    pub id: i64,
    pub account_type: String,
    pub display_name: String,
    pub avatar_path: Option<String>,
    pub entitlement_tier: String,
    pub status: String,
    pub first_run: bool,
    pub created_at: String,
    pub last_active_at: Option<String>,
}

#[derive(Serialize)]
pub struct SessionDto {
    pub account_id: i64,
    pub display_name: String,
    pub account_type: String,
    pub entitlement_tier: String,
    pub session_token: String,      // short-lived in-memory token
    pub authenticated_at: String,
}

#[derive(Serialize)]
pub struct AccountSummaryDto {
    pub id: i64,
    pub display_name: String,
    pub account_type: String,
    pub avatar_path: Option<String>,
    pub entitlement_tier: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct UpdateProfileInput {
    pub display_name: Option<String>,
    pub avatar_path: Option<String>,
    pub grade_level: Option<String>,
    pub exam_target: Option<String>,
    pub exam_target_date: Option<String>,
    pub daily_study_budget_minutes: Option<i32>,
    pub study_days_per_week: Option<i32>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn create_account(
    state: State<'_, AppState>,
    input: CreateAccountInput,
) -> Result<AccountDto, CommandError>

#[tauri::command]
pub async fn login_with_pin(
    state: State<'_, AppState>,
    account_id: i64,
    pin: String,
) -> Result<SessionDto, CommandError>

#[tauri::command]
pub async fn list_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<AccountSummaryDto>, CommandError>

#[tauri::command]
pub async fn switch_account(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<AccountDto, CommandError>

#[tauri::command]
pub async fn link_parent_student(
    state: State<'_, AppState>,
    parent_id: i64,
    student_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn reset_student_pin(
    state: State<'_, AppState>,
    parent_id: i64,
    student_id: i64,
    new_pin: String,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn get_linked_students(
    state: State<'_, AppState>,
    parent_id: i64,
) -> Result<Vec<AccountSummaryDto>, CommandError>

#[tauri::command]
pub async fn update_account_profile(
    state: State<'_, AppState>,
    account_id: i64,
    input: UpdateProfileInput,
) -> Result<AccountDto, CommandError>
```

**Service delegation:** `IdentityService` in `ecoach-identity`

---

## 6.2 curriculum_commands.rs

**File:** `ecoach-commands/src/curriculum_commands.rs`
**Owns:** Read-only curriculum graph queries; no writes (curriculum is pack-installed)

### DTOs

```rust
#[derive(Serialize)]
pub struct SubjectDto {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub curriculum_version_id: i64,
    pub topic_count: i32,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct TopicDto {
    pub id: i64,
    pub subject_id: i64,
    pub name: String,
    pub node_id: String,        // e.g., "B7/JHS1.1.1"
    pub depth_level: i32,
    pub parent_topic_id: Option<i64>,
    pub exam_weight: i32,       // basis points
    pub prerequisite_ids: Vec<i64>,
    pub order_index: i32,
}

#[derive(Serialize)]
pub struct TopicTreeDto {
    pub subject_id: i64,
    pub nodes: Vec<TopicTreeNode>,
}

#[derive(Serialize)]
pub struct TopicTreeNode {
    pub id: i64,
    pub name: String,
    pub node_id: String,
    pub depth_level: i32,
    pub children: Vec<TopicTreeNode>,
    pub skill_atom_count: i32,
}

#[derive(Serialize)]
pub struct SkillAtomDto {
    pub id: i64,
    pub topic_id: i64,
    pub name: String,
    pub atom_type: String,
    pub cognitive_verb: String,
    pub difficulty_band: String,
    pub prerequisite_atom_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct CurriculumVersionDto {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub exam_board: String,
    pub effective_year: String,
    pub is_active: bool,
    pub subject_count: i32,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_subjects(
    state: State<'_, AppState>,
    curriculum_version_id: i64,
) -> Result<Vec<SubjectDto>, CommandError>

#[tauri::command]
pub async fn get_topics(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<Vec<TopicDto>, CommandError>

#[tauri::command]
pub async fn get_topic_tree(
    state: State<'_, AppState>,
    subject_id: i64,
) -> Result<TopicTreeDto, CommandError>

#[tauri::command]
pub async fn get_skill_atoms(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<Vec<SkillAtomDto>, CommandError>

#[tauri::command]
pub async fn get_curriculum_versions(
    state: State<'_, AppState>,
) -> Result<Vec<CurriculumVersionDto>, CommandError>
```

**Service delegation:** `CurriculumService` in `ecoach-curriculum`

---

## 6.3 session_commands.rs

**File:** `ecoach-commands/src/session_commands.rs`
**Owns:** Session lifecycle, attempt submission, session history

### DTOs

```rust
#[derive(Deserialize)]
pub struct StartSessionInput {
    pub account_id: i64,
    pub session_type: String,       // "practice"|"diagnostic"|"mock"|"gap_repair"|"memory_review"|"coach_mission"|"elite"|"game"|"traps"
    pub subject_id: Option<i64>,
    pub topic_ids: Option<Vec<i64>>,
    pub question_count: Option<i32>,
    pub duration_minutes: Option<i32>,
    pub is_timed: Option<bool>,
    pub difficulty_preference: Option<String>,
}

#[derive(Serialize)]
pub struct SessionDto {
    pub id: i64,
    pub account_id: i64,
    pub session_type: String,
    pub status: String,
    pub question_count: i32,
    pub first_question: Option<QuestionDto>,
    pub started_at: String,
    pub duration_minutes: Option<i32>,
    pub is_timed: bool,
}

#[derive(Deserialize)]
pub struct SubmitAttemptInput {
    pub question_id: i64,
    pub selected_option_id: Option<i64>,
    pub open_response_text: Option<String>,
    pub response_time_ms: i64,
    pub confidence_level: Option<String>,   // "sure"|"not_sure"|"guessed"
    pub hint_count: i32,
}

#[derive(Serialize)]
pub struct AttemptResultDto {
    pub attempt_id: i64,
    pub is_correct: bool,
    pub correct_option_id: Option<i64>,
    pub correct_option_text: String,
    pub explanation: Option<String>,
    pub error_type: Option<String>,
    pub misconception_info: Option<String>,
    pub updated_mastery_bp: i32,
    pub updated_gap_bp: i32,
    pub next_question: Option<QuestionDto>,
    pub session_complete: bool,
}

#[derive(Serialize)]
pub struct SessionSummaryDto {
    pub id: i64,
    pub session_type: String,
    pub total_questions: i32,
    pub correct_questions: i32,
    pub accuracy_bp: i32,
    pub avg_response_time_ms: i64,
    pub duration_seconds: i64,
    pub completed_at: String,
    pub topic_results: Vec<TopicResultDto>,
    pub coach_message: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionEventInput {
    pub event_type: String,     // "paused"|"resumed"|"hint_requested"|"question_skipped"
    pub payload_json: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn start_session(
    state: State<'_, AppState>,
    input: StartSessionInput,
) -> Result<SessionDto, CommandError>

#[tauri::command]
pub async fn submit_attempt(
    state: State<'_, AppState>,
    session_id: i64,
    input: SubmitAttemptInput,
) -> Result<AttemptResultDto, CommandError>

#[tauri::command]
pub async fn end_session(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<SessionSummaryDto, CommandError>

#[tauri::command]
pub async fn get_session_history(
    state: State<'_, AppState>,
    account_id: i64,
    limit: u32,
) -> Result<Vec<SessionSummaryDto>, CommandError>

#[tauri::command]
pub async fn record_session_event(
    state: State<'_, AppState>,
    session_id: i64,
    input: SessionEventInput,
) -> Result<(), CommandError>
```

**Service delegation:** `SessionOrchestrator` in `ecoach-sessions`; calls `StudentModelService.process_answer()` on each attempt

---

## 6.4 student_commands.rs

**File:** `ecoach-commands/src/student_commands.rs`
**Owns:** Student state reads — mastery, readiness, momentum

### DTOs

```rust
#[derive(Serialize)]
pub struct StudentStateDto {
    pub account_id: i64,
    pub overall_readiness_bp: i32,
    pub overall_mastery_bp: i32,
    pub streak_days: i32,
    pub total_sessions: i32,
    pub total_study_minutes: i64,
    pub exam_target: Option<String>,
    pub exam_date: Option<String>,
    pub days_to_exam: Option<i32>,
    pub coach_lifecycle_state: String,
    pub momentum_direction: String,
}

#[derive(Serialize)]
pub struct MasteryMapDto {
    pub account_id: i64,
    pub subjects: Vec<SubjectMasteryDto>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct SubjectMasteryDto {
    pub subject_id: i64,
    pub subject_name: String,
    pub mastery_bp: i32,
    pub gap_bp: i32,
    pub coverage_bp: i32,
    pub topics: Vec<TopicMasteryDto>,
}

#[derive(Serialize)]
pub struct TopicMasteryDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub mastery_state: String,
    pub mastery_bp: i32,
    pub gap_bp: i32,
    pub priority_bp: i32,
    pub accuracy_bp: i32,
    pub speed_bp: i32,
    pub retention_bp: i32,
    pub trend_direction: String,
    pub misconception_count: i32,
    pub last_seen_at: Option<String>,
}

#[derive(Serialize)]
pub struct ReadinessReportDto {
    pub account_id: i64,
    pub overall_readiness_bp: i32,
    pub predicted_exam_score_bp: i32,
    pub readiness_band: String,         // "not_ready"|"approaching"|"ready"|"strong"
    pub mastery_component_bp: i32,
    pub timed_performance_bp: i32,
    pub coverage_bp: i32,
    pub consistency_bp: i32,
    pub trend_bp: i32,
    pub critical_gaps: Vec<GapSummaryDto>,
    pub strongest_topics: Vec<String>,
    pub weakest_topics: Vec<String>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct MomentumDto {
    pub account_id: i64,
    pub momentum_score_bp: i32,
    pub direction: String,
    pub volume_bp: i32,
    pub accuracy_bp: i32,
    pub pace_bp: i32,
    pub streak_days: i32,
    pub sessions_this_week: i32,
    pub last_active_at: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_student_state(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<StudentStateDto, CommandError>

#[tauri::command]
pub async fn get_mastery_map(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MasteryMapDto, CommandError>

#[tauri::command]
pub async fn get_topic_mastery(
    state: State<'_, AppState>,
    account_id: i64,
    topic_id: i64,
) -> Result<TopicMasteryDto, CommandError>

#[tauri::command]
pub async fn get_readiness_report(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<ReadinessReportDto, CommandError>

#[tauri::command]
pub async fn get_momentum(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MomentumDto, CommandError>
```

**Service delegation:** `StudentModelService` in `ecoach-student-model`

---

## 6.5 coach_commands.rs

**File:** `ecoach-commands/src/coach_commands.rs`
**Owns:** Coach brain state, next action, missions, interventions

### DTOs

```rust
#[derive(Serialize)]
pub struct CoachStateDto {
    pub account_id: i64,
    pub lifecycle_state: String,            // CoachLifecycleState variant
    pub content_readiness_status: String,
    pub next_action: String,                // NextCoachAction variant
    pub active_mission_id: Option<i64>,
    pub intervention_count: i32,
    pub plan_exists: bool,
    pub plan_adherence_bp: i32,
    pub last_updated_at: String,
}

#[derive(Serialize)]
pub struct NextActionDto {
    pub action_type: String,
    pub title: String,
    pub description: String,
    pub cta_label: String,
    pub urgency: String,
    pub payload_json: Option<String>,
}

#[derive(Serialize)]
pub struct MissionDto {
    pub id: i64,
    pub account_id: i64,
    pub mission_type: String,
    pub title: String,
    pub description: String,
    pub subject_id: Option<i64>,
    pub topic_ids: Vec<i64>,
    pub estimated_minutes: i32,
    pub status: String,
    pub items: Vec<MissionItemDto>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct MissionItemDto {
    pub id: i64,
    pub item_type: String,
    pub description: String,
    pub is_completed: bool,
    pub order_index: i32,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_coach_state(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<CoachStateDto, CommandError>

#[tauri::command]
pub async fn get_next_action(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<NextActionDto, CommandError>

#[tauri::command]
pub async fn get_current_mission(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MissionDto, CommandError>

#[tauri::command]
pub async fn dismiss_intervention(
    state: State<'_, AppState>,
    intervention_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn acknowledge_coach_event(
    state: State<'_, AppState>,
    event_id: i64,
) -> Result<(), CommandError>
```

**Service delegation:** `CoachBrainService` in `ecoach-coach-brain`

---

## 6.6 diagnostic_commands.rs

**File:** `ecoach-commands/src/diagnostic_commands.rs`
**Owns:** Academic DNA test, problem cards, diagnostic report

### DTOs

```rust
#[derive(Deserialize)]
pub struct DiagnosticInput {
    pub mode: String,                   // "light"|"standard"|"deep"
    pub subject_ids: Vec<i64>,
    pub include_timed_layer: bool,
    pub include_transfer_layer: bool,
}

#[derive(Serialize)]
pub struct DiagnosticSessionDto {
    pub id: i64,
    pub account_id: i64,
    pub mode: String,
    pub total_estimated_items: i32,
    pub estimated_minutes: i32,
    pub current_stage: String,
    pub first_question: QuestionDto,
    pub started_at: String,
}

#[derive(Deserialize)]
pub struct DiagnosticResponseInput {
    pub question_id: i64,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: i64,
    pub confidence_level: String,       // "sure"|"not_sure"|"guessed"
    pub answer_changed: bool,
    pub change_count: i32,
    pub hint_used: bool,
}

#[derive(Serialize)]
pub struct DiagnosticResultDto {
    pub is_correct: bool,
    pub next_question: Option<QuestionDto>,
    pub stage_transition: Option<String>,
    pub diagnostic_complete: bool,
    pub items_remaining: i32,
}

#[derive(Serialize)]
pub struct DiagnosticReportDto {
    pub account_id: i64,
    pub generated_at: String,
    pub overall_profile: String,
    pub dimensions: DiagnosticDimensionsDto,
    pub subject_reports: Vec<SubjectDiagnosticDto>,
    pub problem_cards: Vec<ProblemCardDto>,
    pub recommended_focus_topics: Vec<i64>,
    pub archetype: String,
}

#[derive(Serialize)]
pub struct DiagnosticDimensionsDto {
    pub coverage_bp: i32,
    pub accuracy_bp: i32,
    pub recall_strength_bp: i32,
    pub reasoning_depth_bp: i32,
    pub misconception_density_bp: i32,
    pub speed_bp: i32,
    pub pressure_response_bp: i32,
    pub transfer_ability_bp: i32,
    pub stability_bp: i32,
    pub confidence_calibration_bp: i32,
    pub fatigue_pattern_bp: i32,
}

#[derive(Serialize)]
pub struct ProblemCardDto {
    pub id: i64,
    pub account_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub problem_type: String,
    pub severity: String,
    pub description: String,
    pub evidence_summary: String,
    pub recommended_intervention: String,
    pub created_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn start_diagnostic(
    state: State<'_, AppState>,
    account_id: i64,
    input: DiagnosticInput,
) -> Result<DiagnosticSessionDto, CommandError>

#[tauri::command]
pub async fn submit_diagnostic_response(
    state: State<'_, AppState>,
    diagnostic_id: i64,
    input: DiagnosticResponseInput,
) -> Result<DiagnosticResultDto, CommandError>

#[tauri::command]
pub async fn get_diagnostic_report(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<DiagnosticReportDto, CommandError>

#[tauri::command]
pub async fn get_problem_cards(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<ProblemCardDto>, CommandError>
```

**Service delegation:** `DiagnosticBatteryService` in `ecoach-diagnostics`

---

## 6.7 mock_commands.rs

**File:** `ecoach-commands/src/mock_commands.rs`
**Owns:** Mock compilation, runtime, post-mock analysis, forecast

### DTOs

```rust
#[derive(Deserialize)]
pub struct CompileMockInput {
    pub mock_type: String,      // "forecast"|"diagnostic"|"remediation"|"final_exam"|"shock"|"wisdom"
    pub subject_ids: Vec<i64>,
    pub question_count_per_subject: i32,
    pub duration_minutes: i32,
    pub use_forecast_weights: bool,
}

#[derive(Serialize)]
pub struct MockInstanceDto {
    pub id: i64,
    pub account_id: i64,
    pub mock_type: String,
    pub total_questions: i32,
    pub duration_minutes: i32,
    pub subjects_included: Vec<String>,
    pub forecast_coverage_bp: i32,
    pub compiled_at: String,
}

#[derive(Serialize)]
pub struct MockSessionDto {
    pub id: i64,
    pub mock_instance_id: i64,
    pub status: String,
    pub first_question: QuestionDto,
    pub total_questions: i32,
    pub time_limit_seconds: i64,
    pub started_at: String,
}

#[derive(Deserialize)]
pub struct MockAttemptInput {
    pub question_id: i64,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: i64,
    pub confidence_level: Option<String>,
}

#[derive(Serialize)]
pub struct MockAttemptResultDto {
    pub next_question: Option<QuestionDto>,
    pub mock_complete: bool,
    pub questions_remaining: i32,
    pub time_remaining_seconds: i64,
}

#[derive(Serialize)]
pub struct MockAnalysisDto {
    pub mock_instance_id: i64,
    pub overall_score_bp: i32,
    pub predicted_bece_band: String,
    pub readiness_movement_bp: i32,
    // Section 1 — Overall summary
    pub confidence_level: String,
    pub timing_assessment: String,
    // Section 2 — Subject/topic performance
    pub subject_results: Vec<MockSubjectResultDto>,
    // Section 3 — Link-level diagnosis
    pub broken_concept_links: Vec<String>,
    pub prerequisite_failures: Vec<String>,
    // Section 4 — Misconception diagnosis
    pub misconceptions_detected: Vec<MockMisconceptionDto>,
    // Section 5 — Representation diagnosis
    pub representation_breakdown: MockRepresentationDto,
    // Section 6 — Timing diagnosis
    pub timing_breakdown: MockTimingDto,
    // Section 7 — Confidence diagnosis
    pub confidence_breakdown: MockConfidenceDto,
    // Section 8 — Action plan
    pub repair_now: Vec<String>,
    pub drill_topics: Vec<String>,
    pub review_topics: Vec<String>,
    pub next_mock_recommended_date: Option<String>,
}

#[derive(Serialize)]
pub struct MockSummaryDto {
    pub mock_instance_id: i64,
    pub mock_type: String,
    pub score_bp: i32,
    pub completed_at: String,
    pub question_count: i32,
    pub accuracy_bp: i32,
}

#[derive(Serialize)]
pub struct ForecastReportDto {
    pub account_id: i64,
    pub high_probability_topics: Vec<ForecastTopicDto>,
    pub medium_probability_topics: Vec<ForecastTopicDto>,
    pub surprise_risk_topics: Vec<ForecastTopicDto>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct ForecastTopicDto {
    pub topic_id: i64,
    pub topic_name: String,
    pub forecast_score_bp: i32,
    pub probability_band: String,
    pub frequency_score_bp: i32,
    pub recency_score_bp: i32,
    pub student_mastery_bp: i32,
}
```

### Commands

```rust
#[tauri::command]
pub async fn compile_mock(
    state: State<'_, AppState>,
    account_id: i64,
    input: CompileMockInput,
) -> Result<MockInstanceDto, CommandError>

#[tauri::command]
pub async fn start_mock(
    state: State<'_, AppState>,
    mock_instance_id: i64,
) -> Result<MockSessionDto, CommandError>

#[tauri::command]
pub async fn submit_mock_attempt(
    state: State<'_, AppState>,
    mock_session_id: i64,
    input: MockAttemptInput,
) -> Result<MockAttemptResultDto, CommandError>

#[tauri::command]
pub async fn end_mock(
    state: State<'_, AppState>,
    mock_session_id: i64,
) -> Result<MockAnalysisDto, CommandError>

#[tauri::command]
pub async fn get_mock_history(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<MockSummaryDto>, CommandError>

#[tauri::command]
pub async fn get_mock_analysis(
    state: State<'_, AppState>,
    mock_instance_id: i64,
) -> Result<MockAnalysisDto, CommandError>

#[tauri::command]
pub async fn get_forecast_report(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<ForecastReportDto, CommandError>
```

**Service delegation:** `MockCentreService` in `ecoach-mock-centre`

---

## 6.8 gap_commands.rs

**File:** `ecoach-commands/src/gap_commands.rs`
**Owns:** Gap map, gap priority, gap repair sessions

### DTOs

```rust
#[derive(Serialize)]
pub struct GapMapDto {
    pub account_id: i64,
    pub coverage_bp: i32,
    pub total_gap_bp: i32,
    pub critical_gap_count: i32,
    pub hidden_gap_count: i32,
    pub fixed_this_month: i32,
    pub subjects: Vec<SubjectGapDto>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct SubjectGapDto {
    pub subject_id: i64,
    pub subject_name: String,
    pub gap_bp: i32,
    pub critical_gaps: Vec<GapDto>,
}

#[derive(Serialize)]
pub struct GapDto {
    pub id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub gap_type: String,       // KnowledgeGapType variant
    pub severity_bp: i32,
    pub rank: i32,
    pub error_types: Vec<String>,
    pub misconception_tags: Vec<String>,
    pub last_seen_at: Option<String>,
    pub attempts: i32,
    pub correct_rate_bp: i32,
}

#[derive(Serialize)]
pub struct GapRepairSessionDto {
    pub id: i64,
    pub account_id: i64,
    pub gap_id: i64,
    pub topic_name: String,
    pub gap_type: String,
    pub repair_strategy: String,
    pub first_question: QuestionDto,
    pub total_planned_items: i32,
    pub started_at: String,
}

#[derive(Serialize)]
pub struct RepairResultDto {
    pub is_correct: bool,
    pub explanation: String,
    pub repair_stage: String,
    pub next_question: Option<QuestionDto>,
    pub session_complete: bool,
    pub mastery_delta_bp: i32,
}

#[derive(Serialize)]
pub struct GapRepairSummaryDto {
    pub repair_session_id: i64,
    pub gap_id: i64,
    pub items_completed: i32,
    pub accuracy_bp: i32,
    pub mastery_change_bp: i32,
    pub gap_status_after: String,
    pub next_repair_recommended_at: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_gap_map(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<GapMapDto, CommandError>

#[tauri::command]
pub async fn get_gap_priority_list(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<GapDto>, CommandError>

#[tauri::command]
pub async fn start_gap_repair(
    state: State<'_, AppState>,
    account_id: i64,
    gap_id: i64,
) -> Result<GapRepairSessionDto, CommandError>

#[tauri::command]
pub async fn submit_gap_repair_attempt(
    state: State<'_, AppState>,
    repair_session_id: i64,
    input: SubmitAttemptInput,
) -> Result<RepairResultDto, CommandError>

#[tauri::command]
pub async fn close_gap_repair(
    state: State<'_, AppState>,
    repair_session_id: i64,
) -> Result<GapRepairSummaryDto, CommandError>
```

**Service delegation:** `KnowledgeGapService` in `ecoach-knowledge-gap`

---

## 6.9 memory_commands.rs

**File:** `ecoach-commands/src/memory_commands.rs`
**Owns:** Memory shelf, rescue queue, recall drills, memory health

### DTOs

```rust
#[derive(Serialize)]
pub struct MemoryShelfDto {
    pub account_id: i64,
    pub total_tracked: i32,
    pub locked_in_count: i32,
    pub stable_count: i32,
    pub vulnerable_count: i32,
    pub fading_count: i32,
    pub critical_count: i32,
    pub items: Vec<MemoryItemDto>,
}

#[derive(Serialize)]
pub struct MemoryItemDto {
    pub memory_state_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub memory_state: String,               // MemoryState variant
    pub msi_score_bp: i32,                  // Memory Strength Index
    pub decay_risk_bp: i32,
    pub last_recall_at: Option<String>,
    pub next_review_at: Option<String>,
    pub recall_success_rate_bp: i32,
    pub hint_dependency_bp: i32,
}

#[derive(Serialize)]
pub struct MemoryRescueItemDto {
    pub memory_state_id: i64,
    pub topic_id: i64,
    pub topic_name: String,
    pub urgency: String,
    pub decay_severity: String,
    pub recommended_session_type: String,
    pub estimated_minutes: i32,
}

#[derive(Deserialize)]
pub struct RecallInput {
    pub response_text: Option<String>,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: i64,
    pub confidence_level: String,
}

#[derive(Serialize)]
pub struct RecallResultDto {
    pub is_correct: bool,
    pub new_memory_state: String,
    pub msi_delta_bp: i32,
    pub next_review_at: String,
    pub recovery_stage: String,
    pub encouragement: Option<String>,
}

#[derive(Serialize)]
pub struct MemoryHealthDto {
    pub account_id: i64,
    pub overall_health_bp: i32,
    pub at_risk_count: i32,
    pub collapsed_count: i32,
    pub stable_or_above_count: i32,
    pub upcoming_reviews: Vec<MemoryRescueItemDto>,
    pub generated_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_memory_shelf(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MemoryShelfDto, CommandError>

#[tauri::command]
pub async fn get_rescue_queue(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<MemoryRescueItemDto>, CommandError>

#[tauri::command]
pub async fn submit_recall_attempt(
    state: State<'_, AppState>,
    memory_state_id: i64,
    input: RecallInput,
) -> Result<RecallResultDto, CommandError>

#[tauri::command]
pub async fn get_memory_health(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<MemoryHealthDto, CommandError>
```

**Service delegation:** `MemoryEngine` in `ecoach-memory`

---

## 6.10 goals_commands.rs

**File:** `ecoach-commands/src/goals_commands.rs`
**Owns:** Exam goals, weekly plans, daily schedules, readiness timeline

### DTOs

```rust
#[derive(Deserialize)]
pub struct ExamGoalInput {
    pub exam_target: String,            // e.g., "BECE 2026"
    pub exam_date: String,              // ISO date
    pub subject_ids: Vec<i64>,
    pub target_grade: Option<String>,
    pub daily_study_budget_minutes: i32,
    pub study_days_per_week: i32,
    pub path_intensity: String,         // "relaxed"|"balanced"|"intense"
}

#[derive(Serialize)]
pub struct ExamGoalDto {
    pub id: i64,
    pub account_id: i64,
    pub exam_target: String,
    pub exam_date: String,
    pub days_remaining: i32,
    pub subject_ids: Vec<i64>,
    pub target_grade: Option<String>,
    pub daily_budget_minutes: i32,
    pub study_days_per_week: i32,
    pub state: String,
    pub confidence_bp: i32,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct WeeklyPlanDto {
    pub account_id: i64,
    pub week_start: String,
    pub week_end: String,
    pub total_planned_minutes: i32,
    pub days: Vec<DayPlanDto>,
    pub focus_topics: Vec<String>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct DayPlanDto {
    pub date: String,
    pub subject_focus: Option<String>,
    pub planned_minutes: i32,
    pub session_types: Vec<String>,
    pub target_outcomes: Vec<String>,
    pub status: String,
}

#[derive(Serialize)]
pub struct DailyScheduleDto {
    pub date: String,
    pub account_id: i64,
    pub available_minutes: i32,
    pub sessions: Vec<ScheduledSessionDto>,
    pub total_planned_minutes: i32,
    pub coach_note: Option<String>,
}

#[derive(Serialize)]
pub struct ScheduledSessionDto {
    pub session_type: String,
    pub subject_name: String,
    pub topic_names: Vec<String>,
    pub estimated_minutes: i32,
    pub priority: String,
    pub reason: String,
}

#[derive(Deserialize)]
pub struct UpdateGoalInput {
    pub state: Option<String>,
    pub exam_date: Option<String>,
    pub daily_budget_minutes: Option<i32>,
    pub path_intensity: Option<String>,
}

#[derive(Serialize)]
pub struct ReadinessTimelineDto {
    pub account_id: i64,
    pub current_readiness_bp: i32,
    pub projected_exam_readiness_bp: i32,
    pub projected_date_to_ready: Option<String>,
    pub exam_date: Option<String>,
    pub milestones: Vec<ReadinessMilestoneDto>,
    pub on_track: bool,
    pub risk_level: String,
}

#[derive(Serialize)]
pub struct ReadinessMilestoneDto {
    pub label: String,
    pub target_date: String,
    pub target_readiness_bp: i32,
    pub achieved: bool,
}
```

### Commands

```rust
#[tauri::command]
pub async fn set_exam_goal(
    state: State<'_, AppState>,
    account_id: i64,
    input: ExamGoalInput,
) -> Result<ExamGoalDto, CommandError>

#[tauri::command]
pub async fn get_weekly_plan(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<WeeklyPlanDto, CommandError>

#[tauri::command]
pub async fn get_daily_schedule(
    state: State<'_, AppState>,
    account_id: i64,
    date: String,
) -> Result<DailyScheduleDto, CommandError>

#[tauri::command]
pub async fn update_goal(
    state: State<'_, AppState>,
    goal_id: i64,
    input: UpdateGoalInput,
) -> Result<ExamGoalDto, CommandError>

#[tauri::command]
pub async fn get_readiness_timeline(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<ReadinessTimelineDto, CommandError>
```

**Service delegation:** `GoalsCalendarService` in `ecoach-goals-calendar`

---

## 6.11 reporting_commands.rs

**File:** `ecoach-commands/src/reporting_commands.rs`
**Owns:** Parent dashboard, child reports, alerts, digests, printable reports

### DTOs

```rust
#[derive(Serialize)]
pub struct ParentDashboardDto {
    pub parent_account_id: i64,
    pub children: Vec<ChildOverviewDto>,
    pub total_alerts: i32,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct ChildOverviewDto {
    pub student_id: i64,
    pub display_name: String,
    pub academic_status: String,
    pub recent_activity_at: Option<String>,
    pub strongest_subject: Option<String>,
    pub weakest_subject: Option<String>,
    pub overall_readiness_bp: i32,
    pub streak_days: i32,
    pub unread_alerts: i32,
    pub trend_direction: String,
}

#[derive(Serialize)]
pub struct ChildReportDto {
    pub student_id: i64,
    pub display_name: String,
    pub performance_summary: PerformanceSummaryDto,
    pub activity_history: ActivityHistoryDto,
    pub attention_needed: Vec<ParentAlertDto>,
    pub subject_breakdown: Vec<SubjectMasteryDto>,
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct PerformanceSummaryDto {
    pub avg_score_trend_bp: i32,
    pub improvement_direction: String,
    pub test_frequency_per_week: f32,
    pub mock_count: i32,
    pub readiness_level: String,
    pub sessions_this_week: i32,
    pub total_study_minutes: i64,
}

#[derive(Serialize)]
pub struct ActivityHistoryDto {
    pub study_days: Vec<String>,
    pub subjects_studied: Vec<String>,
    pub tests_taken: i32,
    pub time_spent_minutes: i64,
    pub abandoned_sessions: i32,
    pub milestones_reached: Vec<String>,
}

#[derive(Serialize)]
pub struct ParentAlertDto {
    pub id: i64,
    pub alert_type: String,
    pub student_name: String,
    pub message: String,
    pub severity: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct WeeklyDigestDto {
    pub account_id: i64,
    pub week_start: String,
    pub week_end: String,
    pub sessions_completed: i32,
    pub study_minutes: i64,
    pub accuracy_this_week_bp: i32,
    pub topics_covered: Vec<String>,
    pub mastery_gains: Vec<MasteryGainDto>,
    pub top_achievement: Option<String>,
    pub areas_to_improve: Vec<String>,
}

#[derive(Serialize)]
pub struct MasteryGainDto {
    pub topic_name: String,
    pub old_state: String,
    pub new_state: String,
}

#[derive(Serialize)]
pub struct PrintableReportDto {
    pub student_id: i64,
    pub display_name: String,
    pub report_date: String,
    pub sections_json: String,      // JSON blob for PDF rendering
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_parent_dashboard(
    state: State<'_, AppState>,
    parent_account_id: i64,
) -> Result<ParentDashboardDto, CommandError>

#[tauri::command]
pub async fn get_parent_child_report(
    state: State<'_, AppState>,
    parent_id: i64,
    student_id: i64,
) -> Result<ChildReportDto, CommandError>

#[tauri::command]
pub async fn get_parent_alerts(
    state: State<'_, AppState>,
    parent_id: i64,
) -> Result<Vec<ParentAlertDto>, CommandError>

#[tauri::command]
pub async fn mark_alert_read(
    state: State<'_, AppState>,
    alert_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn get_weekly_digest(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<WeeklyDigestDto, CommandError>

#[tauri::command]
pub async fn generate_printable_report(
    state: State<'_, AppState>,
    student_id: i64,
) -> Result<PrintableReportDto, CommandError>
```

**Service delegation:** `ReportingService` in `ecoach-reporting`

---

## 6.12 glossary_commands.rs

**File:** `ecoach-commands/src/glossary_commands.rs`
**Owns:** Glossary search and retrieval

### DTOs

```rust
#[derive(Serialize)]
pub struct GlossaryEntryDto {
    pub id: i64,
    pub term: String,
    pub plain_explanation: String,
    pub exam_explanation: Option<String>,
    pub formal_definition: Option<String>,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub examples: Vec<String>,
    pub related_term_ids: Vec<i64>,
    pub audio_path: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn search_glossary(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<GlossaryEntryDto>, CommandError>

#[tauri::command]
pub async fn get_glossary_entry(
    state: State<'_, AppState>,
    entry_id: i64,
) -> Result<GlossaryEntryDto, CommandError>

#[tauri::command]
pub async fn get_related_entries(
    state: State<'_, AppState>,
    entry_id: i64,
) -> Result<Vec<GlossaryEntryDto>, CommandError>

#[tauri::command]
pub async fn get_topic_glossary(
    state: State<'_, AppState>,
    topic_id: i64,
) -> Result<Vec<GlossaryEntryDto>, CommandError>
```

**Service delegation:** `GlossaryService` in `ecoach-glossary`

---

## 6.13 library_commands.rs

**File:** `ecoach-commands/src/library_commands.rs`
**Owns:** Personal shelves, mistake bank, revision packs

### DTOs

```rust
#[derive(Serialize)]
pub struct ShelfDto {
    pub id: i64,
    pub account_id: i64,
    pub shelf_type: String,
    pub name: String,
    pub item_count: i32,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct LibraryItemDto {
    pub id: i64,
    pub shelf_id: i64,
    pub item_type: String,
    pub title: String,
    pub reference_id: i64,
    pub state: String,
    pub saved_at: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct SaveItemInput {
    pub shelf_id: i64,
    pub item_type: String,
    pub reference_id: i64,
    pub title: String,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct MistakeBankEntryDto {
    pub id: i64,
    pub account_id: i64,
    pub attempt_id: i64,
    pub question_id: i64,
    pub question_text: String,
    pub error_type: String,
    pub misconception: Option<String>,
    pub saved_at: String,
    pub review_count: i32,
}

#[derive(Deserialize)]
pub struct CreateRevisionPackInput {
    pub name: String,
    pub topic_ids: Vec<i64>,
    pub include_mistakes: bool,
    pub include_weak_topics: bool,
    pub question_count: i32,
}

#[derive(Serialize)]
pub struct RevisionPackDto {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub question_count: i32,
    pub topic_names: Vec<String>,
    pub created_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_library_shelves(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<ShelfDto>, CommandError>

#[tauri::command]
pub async fn get_shelf_items(
    state: State<'_, AppState>,
    shelf_id: i64,
) -> Result<Vec<LibraryItemDto>, CommandError>

#[tauri::command]
pub async fn save_to_library(
    state: State<'_, AppState>,
    account_id: i64,
    input: SaveItemInput,
) -> Result<LibraryItemDto, CommandError>

#[tauri::command]
pub async fn add_to_mistake_bank(
    state: State<'_, AppState>,
    account_id: i64,
    attempt_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn get_mistake_bank(
    state: State<'_, AppState>,
    account_id: i64,
) -> Result<Vec<MistakeBankEntryDto>, CommandError>

#[tauri::command]
pub async fn create_revision_pack(
    state: State<'_, AppState>,
    account_id: i64,
    input: CreateRevisionPackInput,
) -> Result<RevisionPackDto, CommandError>
```

**Service delegation:** `LibraryService` in `ecoach-library`

---

## 6.14 game_commands.rs

**File:** `ecoach-commands/src/game_commands.rs`
**Owns:** Game sessions for MindStack (Tetris) and Tug of War (MindPull)

### DTOs

```rust
#[derive(Deserialize)]
pub struct StartGameInput {
    pub game_type: String,          // "mindstack"|"tug_of_war"|"traps"
    pub subject_id: Option<i64>,
    pub topic_ids: Option<Vec<i64>>,
    pub difficulty: Option<String>,
}

#[derive(Serialize)]
pub struct GameSessionDto {
    pub id: i64,
    pub account_id: i64,
    pub game_type: String,
    pub status: String,
    pub initial_state_json: String, // game-specific state (board, rope position, etc.)
    pub first_question: QuestionDto,
    pub started_at: String,
}

#[derive(Deserialize)]
pub struct GameActionInput {
    pub action_type: String,        // "answer"|"use_powerup"|"skip"
    pub question_id: Option<i64>,
    pub selected_option_id: Option<i64>,
    pub response_time_ms: Option<i64>,
    pub powerup_type: Option<String>,
}

#[derive(Serialize)]
pub struct GameStateDto {
    pub game_session_id: i64,
    pub state_json: String,         // updated board/rope/score state
    pub is_correct: Option<bool>,
    pub next_question: Option<QuestionDto>,
    pub game_over: bool,
    pub score: i32,
    pub combo_count: i32,
}

#[derive(Serialize)]
pub struct GameResultDto {
    pub game_session_id: i64,
    pub game_type: String,
    pub final_score: i32,
    pub accuracy_bp: i32,
    pub max_combo: i32,
    pub topics_covered: Vec<String>,
    pub mastery_reinforced: Vec<String>,
    pub completed_at: String,
}

#[derive(Serialize)]
pub struct LeaderboardDto {
    pub game_type: String,
    pub entries: Vec<LeaderboardEntryDto>,
    pub player_rank: Option<i32>,
    pub player_best_score: Option<i32>,
}

#[derive(Serialize)]
pub struct LeaderboardEntryDto {
    pub rank: i32,
    pub display_name: String,
    pub score: i32,
    pub achieved_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn start_game(
    state: State<'_, AppState>,
    account_id: i64,
    input: StartGameInput,
) -> Result<GameSessionDto, CommandError>

#[tauri::command]
pub async fn submit_game_action(
    state: State<'_, AppState>,
    game_session_id: i64,
    input: GameActionInput,
) -> Result<GameStateDto, CommandError>

#[tauri::command]
pub async fn end_game(
    state: State<'_, AppState>,
    game_session_id: i64,
) -> Result<GameResultDto, CommandError>

#[tauri::command]
pub async fn get_game_leaderboard(
    state: State<'_, AppState>,
    game_type: String,
) -> Result<LeaderboardDto, CommandError>
```

**Service delegation:** `GameEngineService` in `ecoach-games`

---

## 6.15 intake_commands.rs

**File:** `ecoach-commands/src/intake_commands.rs`
**Owns:** Document upload portal, OCR bridge, candidate question review

### DTOs

```rust
#[derive(Deserialize)]
pub struct UploadDocumentInput {
    pub file_path: String,          // absolute local path
    pub document_type: String,      // "homework"|"class_notes"|"class_tests"|"assignments"|"teacher_handouts"|"revision_sheets"|"report_cards"|"exam_papers"|"textbook_snapshots"|"worksheets"
    pub subject_id: Option<i64>,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct IntakeDocumentDto {
    pub id: i64,
    pub account_id: i64,
    pub document_type: String,
    pub file_name: String,
    pub status: String,             // "queued"|"processing"|"parsed"|"review_pending"|"completed"|"failed"
    pub created_at: String,
}

#[derive(Serialize)]
pub struct IntakeStatusDto {
    pub document_id: i64,
    pub status: String,
    pub progress_percent: i32,
    pub candidate_question_count: i32,
    pub approved_count: i32,
    pub rejected_count: i32,
    pub pending_count: i32,
    pub error_message: Option<String>,
}
```

### Commands

```rust
#[tauri::command]
pub async fn upload_document(
    state: State<'_, AppState>,
    account_id: i64,
    input: UploadDocumentInput,
) -> Result<IntakeDocumentDto, CommandError>

#[tauri::command]
pub async fn get_intake_status(
    state: State<'_, AppState>,
    document_id: i64,
) -> Result<IntakeStatusDto, CommandError>

#[tauri::command]
pub async fn approve_intake_question(
    state: State<'_, AppState>,
    candidate_id: i64,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn reject_intake_question(
    state: State<'_, AppState>,
    candidate_id: i64,
    reason: String,
) -> Result<(), CommandError>
```

**Service delegation:** `IntakeService` in `ecoach-intake`

---

## 6.16 admin_commands.rs

**File:** `ecoach-commands/src/admin_commands.rs`
**Owns:** System status, migration checks, audit log, feature flags, content packs

### DTOs

```rust
#[derive(Serialize)]
pub struct SystemStatusDto {
    pub db_version: i32,
    pub db_size_bytes: i64,
    pub wal_mode: bool,
    pub migration_count: i32,
    pub content_pack_count: i32,
    pub account_count: i32,
    pub uptime_seconds: i64,
    pub build_version: String,
}

#[derive(Serialize)]
pub struct MigrationStatusDto {
    pub applied_count: i32,
    pub pending_count: i32,
    pub latest_applied: String,
    pub pending_migrations: Vec<String>,
    pub healthy: bool,
}

#[derive(Serialize)]
pub struct AuditEntryDto {
    pub id: i64,
    pub event_type: String,
    pub aggregate_id: String,
    pub account_id: Option<i64>,
    pub payload_json: String,
    pub occurred_at: String,
}

#[derive(Serialize)]
pub struct ContentPackStatusDto {
    pub id: i64,
    pub pack_name: String,
    pub version: String,
    pub subject_names: Vec<String>,
    pub question_count: i32,
    pub installed_at: String,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct ContentPackInstallDto {
    pub pack_id: i64,
    pub pack_name: String,
    pub version: String,
    pub subjects_installed: Vec<String>,
    pub question_count: i32,
    pub status: String,
    pub installed_at: String,
}
```

### Commands

```rust
#[tauri::command]
pub async fn get_system_status(
    state: State<'_, AppState>,
) -> Result<SystemStatusDto, CommandError>

#[tauri::command]
pub async fn run_migration_check(
    state: State<'_, AppState>,
) -> Result<MigrationStatusDto, CommandError>

#[tauri::command]
pub async fn get_audit_log(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<AuditEntryDto>, CommandError>

#[tauri::command]
pub async fn toggle_feature_flag(
    state: State<'_, AppState>,
    flag: String,
    enabled: bool,
) -> Result<(), CommandError>

#[tauri::command]
pub async fn get_content_pack_status(
    state: State<'_, AppState>,
) -> Result<Vec<ContentPackStatusDto>, CommandError>

#[tauri::command]
pub async fn install_content_pack(
    state: State<'_, AppState>,
    pack_path: String,
) -> Result<ContentPackInstallDto, CommandError>
```

**Service delegation:** `AdminService` in `ecoach-commands` (thin admin layer using `ecoach-storage` and `ecoach-content` directly)

---

# PART 7: PHASED DELIVERY PLAN

## Reading Key

Each week entry lists: **Tasks**, **Migrations applied**, **Commands exposed**, **Test criteria**.

---

## Phase 0 — Foundations (Weeks 1–3)

### Week 1: Workspace, Database, Identity

**Tasks:**
- Initialize Cargo workspace with all 22 crate stubs
- Set up `ecoach-substrate`: `BasisPoints`, `Role`, `EntitlementTier`, `SeverityLevel`, `TrendDirection`, `DomainEvent`, `EventEnvelope`
- Set up `ecoach-storage`: SQLite pool via `sqlx`, WAL mode pragma, foreign keys pragma, embedded migration runner
- Write and apply migration 001 (`accounts`, `student_profiles`, `parent_profiles`, `parent_student_links`, `admin_profiles`)
- Implement `IdentityService`: `create_account`, `verify_pin`, `list_accounts`, `link_parent_student`, `reset_student_pin`
- Register Tauri app shell; expose first command group

**Migrations applied:** `001_identity.sql`

**Commands exposed:**
- `create_account`, `login_with_pin`, `list_accounts`, `switch_account`, `link_parent_student`, `reset_student_pin`, `get_linked_students`, `update_account_profile`

**Test criteria:**
- Migration applies without errors; WAL mode confirmed via `PRAGMA journal_mode`
- Account created, PIN verified, wrong PIN increments `failed_pin_attempts`
- Parent-student link created and queryable
- All 8 identity commands return valid DTOs from a seeded fixture database

---

### Week 2: Curriculum Graph

**Tasks:**
- Write and apply migrations 002 (`curriculum_versions`, `subjects`, `strands`, `sub_strands`, `content_standards`, `indicators`, `topics`, `skill_atoms`, `curriculum_node_relations`)
- Write and apply migration 003 (`questions`, `question_options`, `question_families`, `question_intelligence_profiles`)
- Implement `CurriculumService`: all 5 read queries
- Implement `QuestionRepository`: `get_by_topic`, `get_by_family`, `get_candidate_pool`
- Seed test data: 2 subjects (Mathematics, Integrated Science), 5 topics each, 50 questions total

**Migrations applied:** `002_curriculum.sql`, `003_questions.sql`

**Commands exposed:**
- `get_curriculum_versions`, `get_subjects`, `get_topics`, `get_topic_tree`, `get_skill_atoms`

**Test criteria:**
- Topic tree query returns correct nested structure for each subject
- Question candidate pool returns questions filtered by topic scope
- `get_topic_tree` completes under 50ms with 100 topics

---

### Week 3: Student State Schema + Coach Init

**Tasks:**
- Write and apply migration 004 (`student_topic_states`, `student_error_profiles`, `student_question_attempts`)
- Write and apply migration 005 (`sessions`, `session_events`, `session_questions`)
- Implement `CoachBrainService` init: `resolve_lifecycle_state`, `get_next_action` (Phase 0 states only: `OnboardingRequired`, `SubjectSelectionRequired`, `DiagnosticRequired`, `ContentReadinessRequired`, `PlanGenerationRequired`)
- Implement `StudentModelService` skeleton: `get_or_create_topic_state`
- Expose basic Tauri command boundary; register all command modules in `main.rs`

**Migrations applied:** `004_student_state.sql`, `005_sessions.sql`

**Commands exposed:**
- `get_coach_state`, `get_next_action`
- `get_student_state` (initial stub — returns zeroed state)

**Test criteria:**
- New student account triggers `OnboardingRequired` lifecycle state
- After subject selection, state advances to `DiagnosticRequired`
- `get_student_state` returns valid DTO without panicking on empty state

---

## Phase 1 — Core Primitives (Weeks 4–8)

### Week 4: Coach Brain State Machine

**Tasks:**
- Write and apply migration 006 (`coach_state`, `missions`, `mission_items`, `plan_days`, `interventions`, `coach_events`)
- Implement all 14 `CoachLifecycleState` transitions with full guard conditions (see section 8.2)
- Implement `PlanEngine` v1: `generate_daily_plan`, `generate_weekly_plan`, `mark_day_complete`
- Implement `MissionFactory`: `create_mission_from_state`, `get_active_mission`
- Wire `CoachBrainService.recompute_after_session()` — called at session end

**Migrations applied:** `006_coach.sql`

**Commands exposed:**
- `get_current_mission`, `dismiss_intervention`, `acknowledge_coach_event`
- `get_weekly_plan`, `get_daily_schedule`

**Test criteria:**
- All 14 state transitions exercised in unit tests with guard conditions
- Plan generation produces a valid 7-day plan given exam date + subject list
- Mission created correctly for `ReadyForTodayMission` state

---

### Week 5: Memory Engine

**Tasks:**
- Write and apply migration 007 (`student_memory_states`, `memory_review_queue`, `recall_attempts`)
- Implement `MemoryEngine`: `compute_msi`, `detect_decay`, `schedule_next_review`, `process_recall_attempt`
- Implement all 12 `MemoryState` transitions
- Implement spaced repetition scheduler: intervals 1d → 3d → 7d → 14d → 30d
- Implement 6 decay signal detectors
- Wire memory recompute into `submit_attempt` hot path

**Migrations applied:** `007_memory.sql`

**Commands exposed:**
- `get_memory_shelf`, `get_rescue_queue`, `submit_recall_attempt`, `get_memory_health`

**Test criteria:**
- MSI formula produces correct values for known input vectors
- Decay detection triggers correctly after simulated time gap
- Review scheduler produces correct next-review dates for all 5 interval steps
- MemoryState transitions tested: `Unformed` → `DurableMastery`, then regression to `AtRisk`

---

### Week 6: Session Runtime + Answer Pipeline

**Tasks:**
- Implement `SessionOrchestrator`: `start_session`, `end_session`, `pause_session`, `resume_session`
- Implement `StudentModelService.process_answer()` — the core evidence loop (classify error, compute evidence weight, update topic state, emit events)
- Implement `QuestionSelector` with candidate fit formula
- Implement `MasteryState` machine (8 states, full transition logic)
- Implement EMA update for accuracy, speed, retention, confidence scores

**Migrations applied:** (none; uses existing migrations)

**Commands exposed:**
- `start_session`, `submit_attempt`, `end_session`, `get_session_history`, `record_session_event`

**Test criteria:**
- Submit 10 correct answers → mastery score increases monotonically
- Submit 5 wrong answers with misconception tags → error profile updated
- Evidence weight halved when `hint_count > 0`
- Session complete event triggers coach brain recompute
- `classify_error` returns correct type for each of the 10 error type scenarios

---

### Week 7: Knowledge Gap Engine

**Tasks:**
- Write and apply migration 009 (`knowledge_gaps`, `gap_repair_sessions`, `gap_repair_attempts`)
- Implement `KnowledgeGapService`: `compute_gap_map`, `score_gap`, `rank_gaps`, `start_repair`, `process_repair_attempt`, `close_repair`
- Implement all 10 `KnowledgeGapType` classifications
- Implement 7 gap discovery methods (passive detection wired into `process_answer`)
- Implement gap scoring formula

**Migrations applied:** `009_knowledge_gap.sql`

**Commands exposed:**
- `get_gap_map`, `get_gap_priority_list`, `start_gap_repair`, `submit_gap_repair_attempt`, `close_gap_repair`

**Test criteria:**
- Gap map correctly identifies critical vs hidden gaps from fixture data
- Gap ranks correlate with severity scores
- Repair session routes to correct intervention for each gap type
- Passive gap detection fires correctly on wrong answer submission

---

### Week 8: Goals and Calendar

**Tasks:**
- Write and apply migration 010 (`goals`, `exam_calendar`, `schedule_ledger`, `availability_profiles`)
- Implement `GoalsCalendarService`: `set_exam_goal`, `generate_weekly_plan`, `get_daily_schedule`, `get_readiness_timeline`
- Implement deadline pressure engine: total days → weekly load → urgency level
- Implement 3-layer schedule structure (Macro / Rolling Active / Daily)
- Implement `ReadinessTimelineProjector`: project readiness trajectory to exam date

**Migrations applied:** `010_goals_calendar.sql`

**Commands exposed:**
- `set_exam_goal`, `get_weekly_plan`, `get_daily_schedule`, `update_goal`, `get_readiness_timeline`

**Test criteria:**
- Exam goal set → weekly plan generated with correct number of study days
- Readiness timeline projects future milestones correctly
- Daily schedule respects `daily_study_budget_minutes` constraint
- Edge case: exam date in 7 days → plan intensity escalates correctly

---

## Phase 2 — Essential Workflows (Weeks 9–14)

### Week 9: Journey Mode

**Tasks:**
- Implement 7 Journey Engines (Starting Point, Deadline Pressure, Curriculum Decomposition, Path Sequencing, Session Composer, Adaptation, Exam Readiness)
- Implement all 5 `JourneyPhase` transitions
- Implement session structure templates (standard 5-part and stronger-learner 4-part)
- Wire journey sessions into `start_session` with `session_type = "coach_mission"`
- Implement `AdaptationEngine`: post-session adjustments to plan

**Migrations applied:** (none; uses 004–010)

**Commands exposed:** (all session + coach commands fully functional for journey mode)

**Test criteria:**
- Full journey session flow: start → submit answers → end → coach recomputes → next mission generated
- Path sequencing correctly orders foundation-first vs high-yield-first based on archetype
- Adaptation engine modifies plan after session with accuracy < 40%

---

### Week 10: Rise Mode

**Tasks:**
- Implement `RiseEngine`: `compute_rise_student_scores`, `select_rise_stage`, `build_rise_session`
- Implement all 4 `RiseStage` transitions with entry criteria
- Implement 8 internal intelligence scores (foundation, recall, speed, accuracy, pressure_stability, misconception_density, momentum, transformation_readiness)
- Implement Momentum score and Strain score formulas
- Implement `PressureLevel` ladder (6 levels)
- Implement error-type → intervention mapping (10 mappings)

**Migrations applied:** (none new)

**Commands exposed:** (journey + session commands support Rise mode sessions)

**Test criteria:**
- Student starting in `Rescue` stage with foundation_score < 0.3 routes to reteach intervention
- Momentum score formula produces correct values for known vectors
- Stage transition from `Rescue` to `Stabilize` requires transformation_readiness >= threshold

---

### Week 11: Mock Centre — Forecast + Compilation

**Tasks:**
- Write and apply migration 008 (`mock_instances`, `mock_sessions`, `mock_attempts`, `mock_analysis`)
- Implement `ForecastEngine`: compute `ForecastScore` using all 7 components
- Implement `MockCompiler`: `compile_mock` for all 6 `MockType` variants
- Implement Mock Orchestration formula (7 components + anti-repeat penalty)
- Implement Mock Selection formula (8 components)

**Migrations applied:** `008_mock_centre.sql`

**Commands exposed:**
- `compile_mock`, `start_mock`, `get_forecast_report`, `get_mock_history`

**Test criteria:**
- Forecast scores computed correctly for seed topic frequency data
- Mock compiled with correct question distribution per subject
- Anti-repeat penalty prevents re-selection of recently seen questions

---

### Week 12: Mock Analysis — 8-Section Report

**Tasks:**
- Implement `MockAnalyzer`: `analyze_mock_session`, producing all 8 sections
- Section 1: overall summary (score, predicted BECE range, timing, readiness movement)
- Section 2: subject/topic performance table
- Section 3: broken concept links, prerequisite failures
- Section 4: misconception diagnosis with counts
- Section 5: representation breakdown (text/diagram/graph/table)
- Section 6: timing diagnosis (slow-correct, fast-careless, pacing collapse)
- Section 7: confidence diagnosis (correct-unsure, wrong-confident, guessing rate)
- Section 8: action plan generation

**Migrations applied:** (none; extends 008)

**Commands exposed:**
- `submit_mock_attempt`, `end_mock`, `get_mock_analysis`

**Test criteria:**
- All 8 sections populated for a completed 40-question mock
- Predicted BECE band computed from weighted mastery formula
- Action plan contains at least one "repair_now" topic for mock with accuracy < 60%

---

### Week 13: Parent Dashboard + Alerts Engine

**Tasks:**
- Write and apply migration 012 (`parent_alerts`, `report_snapshots`, `recommendations`)
- Implement `ReportingService`: `get_parent_dashboard`, `get_parent_child_report`, `generate_weekly_digest`
- Implement `ParentAlertEngine`: detect all 9 auto-alert conditions (inactivity, decline, exam near, etc.)
- Implement internal → parent translation logic (plain-language summaries)
- Implement `PrintableReportGenerator`

**Migrations applied:** `012_reporting.sql`

**Commands exposed:**
- `get_parent_dashboard`, `get_parent_child_report`, `get_parent_alerts`, `mark_alert_read`, `get_weekly_digest`, `generate_printable_report`

**Test criteria:**
- Alert generated when student has 5+ inactive days
- Alert generated when score trend is −8% over 3 assessments
- Parent dashboard `children` array correctly reflects all linked students
- Plain-language translation produces human-readable summary (fixture test)

---

### Week 14: Content Packs

**Tasks:**
- Write and apply migration 011 (`content_packs`, `content_pack_manifests`, `content_pack_items`)
- Implement `ContentPackService`: `install_content_pack` (verify signature, parse manifest, insert curriculum + questions)
- Implement pack manifest format (JSON schema with version, subjects, question count, curriculum version)
- Wire `ContentReadinessStatus` checks in coach brain
- Implement `AdminService`: `get_system_status`, `run_migration_check`, `get_content_pack_status`, `install_content_pack`

**Migrations applied:** `011_content_packs.sql`

**Commands exposed:**
- `install_content_pack`, `get_content_pack_status`, `get_system_status`, `run_migration_check`, `get_audit_log`, `toggle_feature_flag`

**Test criteria:**
- Pack install populates curriculum and question tables correctly
- Coach lifecycle advances from `ContentReadinessRequired` to `PlanGenerationRequired` after pack install
- Migration check correctly reports pending vs applied migrations

---

## Phase 3 — Exam Simulation & Intelligence (Weeks 15–20)

### Week 15: Diagnostic Battery Engine

**Tasks:**
- Write and apply migration 018 (`diagnostic_sessions`, `diagnostic_question_log`, `diagnostic_problem_cards`, `diagnostic_hypotheses`)
- Implement `DiagnosticBatteryService` with all 7 stages (`FastBaselineScan` through `MicroRecheck`)
- Implement all 12 diagnostic dimensions
- Implement guessing detection (8-signal multi-condition)
- Implement adaptive stage routing (zoom into weak areas)
- Implement `ProblemCardFactory`: generate cards from diagnostic results
- Implement 3 duration modes (light ~35min, standard ~60min, deep ~90min)

**Migrations applied:** `018_intake.sql` (repurposed; diagnostic tables created here)

**Commands exposed:**
- `start_diagnostic`, `submit_diagnostic_response`, `get_diagnostic_report`, `get_problem_cards`

**Test criteria:**
- Light mode completes in ≤24 baseline + 10 adaptive items
- Stage routing zooms into topics where accuracy < 50%
- Guessing detection fires when confidence = "guessed" and item is correct
- Problem cards generated for all detected weaknesses

---

### Week 16: Glossary System

**Tasks:**
- Write and apply migration 013 (`glossary_entries`, `glossary_entry_relations`, `glossary_topic_bundles`)
- Implement `GlossaryService` with FTS (SQLite full-text search via FTS5 virtual table)
- Implement related entry traversal
- Implement topic bundle queries

**Migrations applied:** `013_glossary.sql`

**Commands exposed:**
- `search_glossary`, `get_glossary_entry`, `get_related_entries`, `get_topic_glossary`

**Test criteria:**
- FTS search returns results for partial term matches
- Related entries traversal returns correct neighbors
- Topic glossary returns all entries for a given topic

---

### Week 17: Library System

**Tasks:**
- Write and apply migration 014 (`library_shelves`, `library_items`, `mistake_bank`, `revision_packs`, `revision_pack_items`)
- Implement `LibraryService`: all 6 commands
- Implement default shelves on account creation (Saved, Mistakes, Revision)
- Implement `RevisionPackBuilder`: aggregate weak topics + mistakes into session-ready pack

**Migrations applied:** `014_library.sql`

**Commands exposed:**
- `get_library_shelves`, `get_shelf_items`, `save_to_library`, `add_to_mistake_bank`, `get_mistake_bank`, `create_revision_pack`

**Test criteria:**
- Default shelves created on new account
- Mistake bank entry created from attempt with `is_correct = false`
- Revision pack contains correct question count from specified topics

---

### Week 18: Traps + Past Papers

**Tasks:**
- Write and apply migration 017 (`traps_sessions`, `contrast_profiles`, `traps_cards`, `traps_attempts`)
- Write and apply migration 016 (`past_papers`, `past_paper_questions`, `past_paper_question_families`)
- Implement `TrapsEngine`: all 5 `TrapsMode` variants, 5 `TrapsTimerMode` variants, end-of-round summary
- Implement past paper question family mining and recurrence tracking

**Migrations applied:** `016_past_papers.sql`, `017_traps.sql`

**Commands exposed:** (traps sessions exposed via `start_session` with `session_type = "traps"`)

**Test criteria:**
- Traps session routes cards correctly by `contrast_profile`
- Timer mode `Pressure` auto-skips on timeout
- End-of-round summary accurately reflects confusion breakdown

---

### Week 19: Game Engines

**Tasks:**
- Write and apply migration 015 (`game_sessions`, `game_actions`, `leaderboard_entries`)
- Implement `MindStackEngine`: control ladder (4 levels), 5 variants, mercy design rules, reshape mechanics
- Implement `TugOfWarEngine`: rope zones (5), power-ups (5), momentum meter rules
- Implement leaderboard persistence (local, per-device)

**Migrations applied:** `015_games.sql`

**Commands exposed:**
- `start_game`, `submit_game_action`, `end_game`, `get_game_leaderboard`

**Test criteria:**
- MindStack control level correctly reflects answer quality (Excellent → full movement + reshape)
- Tug of War rope position updates correctly on correct/wrong answers
- Streak of 3 triggers "stronger pull" in Tug of War
- Leaderboard correctly ranks scores descending

---

### Week 20: Reporting Engine Extensions

**Tasks:**
- Extend migration 012 with `readiness_proofs`, `readiness_contracts`, `readiness_claims`
- Implement `ReadinessProofEngine`: generate readiness claim with evidence summary
- Implement `WeeklyDigestService`
- Implement printable report PDF data assembly
- Wire all reporting read models (home dashboard, coach directives, parent digest)

**Migrations applied:** (012 extensions)

**Commands exposed:** (all reporting commands fully functional)

**Test criteria:**
- Readiness proof generates correctly when mastery > 72% across all subjects
- Weekly digest contains correct session count and mastery gains for the week
- Printable report data JSON contains all 5 parent dashboard sections

---

## Phase 4 — Premium, Elite & Hardening (Weeks 21–26)

### Week 21: Premium Concierge

**Tasks:**
- Write and apply migration 019 (`premium_strategy_sessions`, `premium_alerts`, `premium_interventions`)
- Implement `PremiumService`: `get_premium_parent_command_center`, `generate_strategy_session`, `detect_premium_alerts`
- Implement 9 premium alert detection conditions (memory slippage, false confidence, plateau, etc.)
- Implement intervention catalog routing (9 intervention types by diagnosis)
- Implement 6-layer premium architecture (Diagnosis → Strategy → Execution → Oversight → HumanExcellence → ParentConfidence)

**Migrations applied:** `019_premium.sql`

**Commands exposed:** (premium commands added to `reporting_commands.rs` and `coach_commands.rs`)

**Test criteria:**
- Premium alerts fire correctly for each of the 9 detection conditions
- Strategy session generated with correct intervention type for each diagnosis
- Premium parent command center fields all populated

---

### Week 22: Elite Mode

**Tasks:**
- Write and apply migration 020 (`elite_sessions`, `elite_performance_profiles`, `elite_missions`, `elite_badges`)
- Implement `EliteEngine`: all 4 `EliteTier` progressions, all 7 `ElitePillar` scoring dimensions, all 7 `EliteSessionType` variants
- Implement `ElitePerformanceDimensions` scoring (8 dimensions)
- Implement EPS (Elite Performance Score) computation
- Implement elite entry criteria check (80%+ mastery + consistency + speed + low hint dependence)
- Implement elite badge award logic (7 badges)

**Migrations applied:** `020_elite.sql`

**Commands exposed:** (elite sessions exposed via `start_session` with `session_type = "elite"`)

**Test criteria:**
- Entry criteria correctly blocks students below threshold
- EPS computation correct for known dimension vectors
- `PerfectRun` session type collapses on first error
- Badge awarded when `PrecisionBeast` criteria met (≥95% accuracy over 20 elite questions)

---

### Week 23: Event Sourcing + Audit

**Tasks:**
- Write and apply migration 021 (`event_log`, `outbox_events`, `audit_log`)
- Implement full append-only event log
- Implement outbox pattern: events written to `outbox_events` before processing, marked processed after handler succeeds
- Implement correlation ID tracking across cascading events
- Implement `AuditTrailService`: query audit log, filter by event type / account / date range
- Implement replay/rebuild tools for state regeneration

**Migrations applied:** `021_event_sourcing.sql`

**Commands exposed:**
- `get_audit_log` (fully functional)

**Test criteria:**
- Every `submit_attempt` call writes to event log
- Outbox pattern: handler failure leaves event in `outbox_events` for retry
- State rebuild from event log produces identical result to current materialized state (round-trip test)

---

### Week 24: Document Intake / OCR Bridge

**Tasks:**
- Wire `IntakeService` to local OCR bridge (external process or WASM component)
- Implement document intelligence extracts (6 intelligence signals)
- Implement candidate question extraction from parsed document
- Implement approval/rejection workflow
- Implement goal creation from document intelligence (e.g., teacher correction areas → `ParentTeacher` goal)

**Migrations applied:** (018 fully activated)

**Commands exposed:**
- `upload_document`, `get_intake_status`, `approve_intake_question`, `reject_intake_question` (fully functional)

**Test criteria:**
- Upload triggers background parse job
- Parsed candidate questions appear in review queue
- Approved question added to student's personal question bank

---

### Week 25: Performance Benchmarks + Query Optimization

**Tasks:**
- Benchmark all hot-path queries: `submit_attempt` pipeline end-to-end < 100ms
- Add missing indexes (review all slow queries via EXPLAIN QUERY PLAN)
- Optimize `get_mastery_map` read model (pre-aggregate, cache in read model table)
- Implement background job hardening: idempotent handlers, bounded exponential backoff, dead-letter queue
- Implement stale read-model rebuild job
- Benchmark: mock compilation < 500ms, diagnostic report generation < 200ms

**Migrations applied:** (index additions only)

**Commands exposed:** (no new commands; all existing commands optimized)

**Test criteria:**
- `submit_attempt` completes in < 100ms on a database with 10,000 attempts
- `get_mastery_map` completes in < 50ms for a student with full subject coverage
- Background jobs survive handler panic without data corruption
- All benchmark targets met

---

### Week 26: Integration Testing + Final Schema Review

**Tasks:**
- Write and apply migration 022 (`system_settings`, `feature_flags`, `app_config`)
- Full integration test suite: onboarding → diagnostic → daily session → mock → parent report
- Final schema review: check all foreign keys, all indexes, all JSON column shapes
- Cross-crate integration tests: session → student model → coach brain → reporting pipeline
- Build verification: `cargo build --release` clean, no warnings
- Seed data package: complete test fixture for all 26 migrations

**Migrations applied:** `022_system_settings.sql`

**Commands exposed:** (no new commands)

**Test criteria:**
- Full onboarding-to-exam-ready flow passes in integration test
- All 22 migration files apply cleanly on fresh database
- `cargo test` passes across all crates
- `cargo clippy` zero warnings
- Release build size under 15MB (backend only)

---

# PART 8: MASTER REFERENCE

## 8.1 All 22 Crate Responsibilities

| Crate | Owns | Depends On |
|---|---|---|
| `ecoach-substrate` | `BasisPoints`, `Role`, `EntitlementTier`, `SeverityLevel`, `TrendDirection`, `DomainEvent`, `EventEnvelope`, scoring helpers, time utilities, shared error traits | (none) |
| `ecoach-storage` | SQLite connection pool, WAL config, migration runner, `Repository` trait, query builder helpers | `ecoach-substrate` |
| `ecoach-identity` | Account CRUD, PIN hash/verify, session tokens, parent-student links, entitlement checks | `ecoach-substrate`, `ecoach-storage` |
| `ecoach-curriculum` | Curriculum graph reads, topic tree, skill atoms, prerequisite resolution, node coverage | `ecoach-substrate`, `ecoach-storage` |
| `ecoach-content` | Content pack install/verify, manifest parsing, pack manifest schema, signed pack validation | `ecoach-substrate`, `ecoach-storage`, `ecoach-curriculum` |
| `ecoach-questions` | Question CRUD, question selection (candidate fit formula), family classification, question intelligence profiles, 8-axis taxonomy | `ecoach-substrate`, `ecoach-storage`, `ecoach-curriculum` |
| `ecoach-student-model` | `StudentTopicState`, mastery formula, priority formula, error classification, evidence weight, EMA updates, mastery state machine, error profiles | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions` |
| `ecoach-diagnostics` | Diagnostic battery engine (7 stages, 12 dimensions), guessing detection, adaptive routing, problem card generation, diagnostic report | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-student-model` |
| `ecoach-coach-brain` | `CoachLifecycleState` (14 states), all 14 transitions, `NextCoachAction`, mission factory, plan engine, intervention catalog, coaching trigger rules, help ladder | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-curriculum` |
| `ecoach-memory` | `MemoryState` (12 states), MSI formula, decay detection (7 types, 6 signal categories), spaced repetition scheduler, recovery planner, connection rebuilder | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model` |
| `ecoach-sessions` | Session orchestrator, session lifecycle (created/active/paused/completed/abandoned), question sequencing, time tracking, session event log, answer pipeline coordination | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-student-model`, `ecoach-memory`, `ecoach-coach-brain` |
| `ecoach-mock-centre` | Mock compilation (6 types), forecast engine, mock selection formula, mock runtime, 8-section post-mock analysis, weakness scoring, past paper ingestion | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-student-model`, `ecoach-sessions` |
| `ecoach-knowledge-gap` | Gap map, 10 gap types, gap scoring, gap priority ranking, repair session, 7 discovery methods, solidification tracking | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-sessions` |
| `ecoach-goals-calendar` | Exam goals (7 categories, 8 states), deadline pressure engine, 3-layer schedule, daily session composer, readiness timeline projector, availability profiles | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-coach-brain` |
| `ecoach-intake` | Document upload, OCR bridge, candidate question extraction, approval/rejection workflow, document intelligence extraction | `ecoach-substrate`, `ecoach-storage`, `ecoach-curriculum`, `ecoach-questions` |
| `ecoach-reporting` | Parent dashboard (5 sections), parent alert engine (9 conditions), child report, weekly digest, readiness proof, printable report data assembly | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-goals-calendar`, `ecoach-mock-centre` |
| `ecoach-library` | Library shelves, library items, mistake bank, revision packs, personal knowledge state | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-student-model` |
| `ecoach-glossary` | Glossary entries, FTS5 search index, related entry traversal, topic bundles, audio path references | `ecoach-substrate`, `ecoach-storage`, `ecoach-curriculum` |
| `ecoach-games` | `MindStackEngine` (5 variants), `TugOfWarEngine` (5 rope zones), `TrapsEngine` (5 modes), power-ups, leaderboard | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-sessions` |
| `ecoach-past-papers` | Past paper ingestion, question family mining, recurrence tracking, year/section metadata | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-curriculum` |
| `ecoach-premium` | Premium alert detection (9 conditions), strategy session engine, 6-layer premium architecture, intervention catalog routing, premium parent command center | `ecoach-substrate`, `ecoach-storage`, `ecoach-student-model`, `ecoach-coach-brain`, `ecoach-reporting` |
| `ecoach-elite` | Elite tier progression (4 tiers), 7 pillar scoring, 8 EPS dimensions, elite session types (7), elite entry criteria, badge award system (7 badges) | `ecoach-substrate`, `ecoach-storage`, `ecoach-questions`, `ecoach-sessions`, `ecoach-student-model` |
| `ecoach-commands` | Tauri command boundary — all 16 command modules, `CommandError`, `AppState`, service wiring, DTO definitions | all domain crates |

---

## 8.2 Complete State Machine Reference

### CoachLifecycleState — 14 States

| State | Description | Entry Condition | Exit Transitions |
|---|---|---|---|
| `OnboardingRequired` | Account just created, no profile | `first_run = 1` | → `SubjectSelectionRequired` when profile completed |
| `SubjectSelectionRequired` | Profile done, no subjects selected | No subjects enrolled | → `ContentReadinessRequired` when subjects selected |
| `ContentReadinessRequired` | Subjects selected, no content pack | No packs for selected subjects | → `DiagnosticRequired` when packs installed |
| `DiagnosticRequired` | Content ready, no baseline | No diagnostic completed | → `PlanGenerationRequired` when diagnostic done |
| `PlanGenerationRequired` | Diagnostic done, no plan | No active plan | → `ReadyForTodayMission` when plan generated |
| `ReadyForTodayMission` | Plan active, today's mission pending | Plan exists, no active mission | → `MissionInProgress` when session started |
| `MissionInProgress` | Session actively running | Session status = `active` | → `MissionReviewRequired` when session ends |
| `MissionReviewRequired` | Session ended, results not reviewed | Session completed, coach not updated | → `ReadyForTodayMission` after recompute |
| `RepairRequired` | Topic blocked; repair needed | accuracy < 40% after 2 sessions | → `ReadyForTodayMission` after repair session |
| `BlockedOnTopic` | No unblocked topics available | All current topics blocked | → `PlanAdjustmentRequired` |
| `PlanAdjustmentRequired` | Plan drift detected | Missed sessions / low adherence | → `ReadyForTodayMission` after plan updated |
| `ReviewDay` | Scheduled review milestone | Checkpoint day in plan | → `ReadyForTodayMission` after review |
| `ExamMode` | Final exam phase | ≤14 days to exam | → terminal (exam date passed) |
| `StalledNoContent` | Content gap — topic has no questions | Question coverage = Red | → `ContentReadinessRequired` after pack update |

### MemoryState — 12 States

| State | Level | MSI Range | Description |
|---|---|---|---|
| `Unformed` | 0 | 0 | Never encountered |
| `Exposed` | 1 | 1–15 | Seen once; no retention |
| `Familiar` | 2 | 16–30 | Recognizes with context |
| `Recognizable` | 3 | 31–45 | MCQ recognition reliable |
| `SupportedRecall` | 4 | 46–55 | Needs cues to recall |
| `FreeRecall` | 5 | 56–65 | Recalls without prompts |
| `AppliedRecall` | 6 | 66–74 | Uses in problem contexts |
| `TransferRecall` | 7 | 75–83 | Recalls in new wordings |
| `PressureStable` | 8 | 84–89 | Holds under timed conditions |
| `DurableMastery` | 9 | 90–100 | Locked in; stable over time |
| `AtRisk` | 10 | — | Was higher; decay detected |
| `Collapsed` | 11 | 0–25 | Regressed significantly |

**Transition rules:**
- Forward: MSI crosses threshold boundary + minimum evidence count
- Regression: MSI drops below lower boundary of current state
- `AtRisk`: triggered when MSI drops > 15bp from peak without time gap
- `Collapsed`: triggered when MSI < 25 after prior `FreeRecall` or higher

### MasteryState — 8 States

| State | Mastery Range | Evidence Required | Fragility Limit |
|---|---|---|---|
| `Unseen` | 0 | 0 | — |
| `Exposed` | 0–24% | ≥1 | — |
| `Emerging` | 25–44% | ≥3 | — |
| `Partial` | 45–59% | ≥8 | — |
| `Fragile` | 60–71% | ≥15 | < 30% |
| `Stable` | 72–81% | ≥25 | < 20% |
| `Robust` | 82–89% | — | < 15%, transfer ≥ 65%, retention ≥ 70% |
| `ExamReady` | ≥ 90% | — | < 15%, retention ≥ 80%, pressure_collapse < 15% |

Regression is possible at all states. `ExamReady` → `Robust` if mastery drops below 80%.

### JourneyPhase — 5 Phases

| Phase | Focus | Entry Trigger | Exit Trigger |
|---|---|---|---|
| `StabilizeFoundation` | Fix major weaknesses; core concepts | Journey start | Critical gaps closed; foundation_score ≥ 60% |
| `BuildCore` | Cover main syllabus; deepen understanding | Foundation stable | ≥ 70% syllabus coverage |
| `StrengthenWeakLinks` | Attack weak topics; recurring mistakes | Core built | Top-ranked gaps addressed |
| `ExamConditioning` | Timed practice; pressure; mixed-topic | ≤ 6 weeks to exam | Readiness ≥ 65% |
| `FinalReadiness` | Mocks; revision bursts; confidence; strategy | ≤ 3 weeks to exam | Exam date |

### RiseStage — 4 Stages

| Stage | Goal | Entry Criteria | Exit Criteria |
|---|---|---|---|
| `Rescue` | Stop the bleeding; first wins; shame removal | foundation_score < 40% | foundation_score ≥ 40%, first_win achieved |
| `Stabilize` | Repeatable correct thinking; scaffolded | foundation_score ≥ 40% | accuracy ≥ 55% consistently over 3 sessions |
| `Accelerate` | Speed + independence; timed drills; pressure | accuracy ≥ 55%, stable | transformation_readiness ≥ 70% |
| `Dominate` | Outperform top students; elite variants | transformation_readiness ≥ 70% | mastery ≥ 85%, speed ≥ 75% |

### MockType — 6 Types

| Type | Purpose | Selection Strategy | Conditions |
|---|---|---|---|
| `Forecast` | Maximum realism; mirrors likely exam | Weighted by ForecastScore; blueprint-aligned | Full duration; timed |
| `Diagnostic` | Maximum insight; reveals gaps | Maximize information value; probe weaknesses | Full duration; may have confidence capture |
| `Remediation` | Close known weak areas | Weight by gap severity and misconception density | Focused; can be shorter |
| `FinalExam` | Full readiness proof | Cover all blueprint targets; balanced difficulty | Full exam duration; strict timing |
| `Shock` | Resilience training; hard unexpected items | Surprise risk topics; unfamiliar formats | Short; brutal |
| `Wisdom` | Mastery proof; elite standards | High mastery requirement; elite question types | Strict accuracy targets |

---

## 8.3 All 60 Question Types

### Family A — Memory Questions (6)
1. Pure Recall — retrieve fact without any cues
2. Recognition — identify correct item from options
3. Memory Reconstruction — rebuild concept from fragments
4. Retrieval Under Pressure — recall under time constraint
5. Retention Check — recall after deliberate time gap
6. Recovery — recall after prior failure/correction

### Family B — Understanding Questions (6)
7. Concept Understanding — demonstrate grasp of core idea
8. Explanation — articulate why something is true
9. Example Generation — produce valid example of concept
10. Non-Example — identify what does NOT fit concept
11. Compare-and-Contrast — distinguish between two related concepts
12. Classification — assign item to correct category

### Family C — Reasoning Questions (6)
13. Reasoning — follow a logical chain to conclusion
14. Logical Deduction — derive conclusion from given premises
15. Inference — draw implied conclusion from evidence
16. Justification — provide valid reasoning for a claim
17. Claim Evaluation — assess whether a stated claim is correct
18. Counterexample — disprove a generalization

### Family D — Problem-Solving Questions (8)
19. Application — use concept in a new context
20. Transfer — apply knowledge to unfamiliar scenario
21. Multi-Step Problem Solving — coordinate multiple operations
22. Strategy Selection — choose most appropriate method
23. First-Step — identify the correct starting action
24. Next-Step — given partial work, choose next correct move
25. Decision-Making — select best option given constraints
26. Prioritization — rank items by given criterion

### Family E — Accuracy Questions (5)
27. Error Detection — find the mistake in given work
28. Correction — fix identified error
29. Misconception Exposure — reveal the false belief causing an error
30. Precision — answer requiring exact specification
31. Attention Control — question designed to catch careless reading

### Family F — Pattern and Structure Questions (8)
32. Pattern Recognition — identify recurring structure
33. Rule Discovery — infer the governing rule from examples
34. Sequence/Order — arrange items in correct order
35. Cause-and-Effect — identify causal relationship
36. Prediction — forecast outcome given conditions
37. Abstraction — generalize from specific to principle
38. Estimation — produce reasonable approximate answer
39. Representation Conversion — translate between formats (table → graph, etc.)

### Family G — Expression and Interpretation Questions (7)
40. Interpretation — extract meaning from data/text/diagram
41. Visualization — mentally picture described scenario
42. Mental Manipulation — rotate/transform represented object mentally
43. Synthesis — combine multiple concepts into one answer
44. Connection-Making — identify link between two ideas
45. Judgment — evaluate quality/correctness of an argument
46. Open-Ended Reasoning — construct extended explanation

### Family H — Growth-Control Questions (14)
47. Diagnostic — probe for specific weakness
48. Mastery Check — verify mastery level achieved
49. Threshold — gate question determining next difficulty
50. Adaptive Difficulty — system-selected based on current performance
51. Rescue — highly scaffolded recovery question
52. Challenge/Stretch — beyond current mastery, builds ceiling
53. Reflection/Metacognitive — "what made this difficult for you?"
54. Confidence Calibration — pair answer with confidence rating
55. Real-World Scenario — concept applied to real-life context
56. Reverse Reasoning — given outcome, infer input conditions
57. Multiple-Path — two valid methods; choose more efficient
58. Deep Thinking — multi-hop reasoning across concepts
59. Speed Fluency — rapid-fire retrieval drill format
60. Capstone — comprehensive integrating question for topic mastery

---

## 8.4 Post-Mock Analysis — 8 Sections

### Section 1 — Overall Summary
- Final score (basis points and percentage)
- Predicted BECE grade band (1–9 or equivalent)
- Confidence index (how reliable the prediction is)
- Timing assessment (fast/on-pace/slow overall)
- Readiness movement (delta from before mock)

### Section 2 — Subject / Topic Performance
- Per-subject: score, strong topics, weak topics, unstable topics
- Topic-level accuracy table
- Topics that regressed vs improved vs held

### Section 3 — Link-Level Diagnosis
- Exact broken concept links detected
- Prerequisite failures (topic A required for topic B; A is weak)
- Bundle collapses (multiple related concepts failed together)

### Section 4 — Misconception Diagnosis
- Top misconceptions triggered (ranked by frequency)
- Count per misconception
- Misconception status: suspected / active / unresolved / cleared
- Which distractors were selected and what they signal

### Section 5 — Representation Diagnosis
- Performance breakdown by format: text / diagram / graph / table / symbolic
- Weakest representation format
- Representation-specific repair recommendation

### Section 6 — Timing Diagnosis
- Slow-but-correct items (knowledge present; speed deficit)
- Fast-but-careless items (rushed; accuracy cost)
- Performance collapse near session end (fatigue signal)
- Section pacing (early vs late timing patterns)

### Section 7 — Confidence Diagnosis
- Correct-but-unsure count (knowledge fragile; needs consolidation)
- Wrong-but-confident count (false mastery; priority repair)
- Estimated guessing rate
- Confidence calibration score

### Section 8 — Action Plan
- Repair now: topics requiring immediate intervention (critical gaps)
- Drill: topics needing speed or repetition work
- Review: stable topics that slipped
- Recommended next mock date (computed from current readiness trajectory)

---

## 8.5 Seven Journey Engines

### Engine 1 — Starting Point Engine
**Purpose:** Establish current level before planning begins
**Inputs:** Past scores, recent behavior, diagnostic results, self-assessment responses
**Outputs:** Current mastery level per topic, gap map baseline, learner archetype classification
**Crate:** `ecoach-diagnostics` + `ecoach-student-model`

### Engine 2 — Deadline Pressure Engine
**Purpose:** Convert available time into feasible study plan
**Inputs:** Total calendar days to exam, realistic study days per week, daily budget minutes, known unavailable dates
**Outputs:** Urgency level, feasible path assessment, goal realism verdict, weekly load targets
**Crate:** `ecoach-goals-calendar`

### Engine 3 — Curriculum Decomposition Engine
**Purpose:** Break subjects into prioritized topic sequences
**Inputs:** Selected subjects, curriculum version, exam weights per topic, prerequisite graph
**Outputs:** Ordered topic list per subject, dependency map, coverage targets
**Crate:** `ecoach-curriculum`

### Engine 4 — Path Sequencing Engine
**Purpose:** Determine learning order strategy
**Inputs:** Learner archetype, current mastery per topic, time pressure, goal type
**Outputs:** Sequenced topic list using one of three strategies: foundation-first / high-yield-first / confidence-first
**Crate:** `ecoach-coach-brain`

### Engine 5 — Session Composer Engine
**Purpose:** Build each individual session from plan + current state
**Inputs:** Today's plan day, current mastery state, time available, session type, learner energy/fatigue signal
**Outputs:** Session configuration (type, topics, question count, duration, structure template)
**Crate:** `ecoach-sessions` + `ecoach-coach-brain`

### Engine 6 — Adaptation Engine
**Purpose:** Adjust plan after each session based on performance
**Inputs:** Session results, accuracy delta, mastery changes, time spent, coach events
**Outputs:** Plan day status update, next-day adjustments, topic re-ranking, intervention flags
**Crate:** `ecoach-coach-brain`

### Engine 7 — Exam Readiness Engine
**Purpose:** Continuously estimate exam performance probability
**Inputs:** Mastery per topic, timed performance, coverage, consistency, trend, retention, mock performance
**Outputs:** Overall readiness score (basis points), readiness band, predicted exam score, milestone dates
**Formula:** `Readiness = 0.25 × mastery + 0.20 × retention + 0.20 × mock_performance + 0.15 × speed + 0.10 × coverage + 0.10 × consistency` (with penalties for critical topic weakness, recurring mistakes, exam anxiety pattern)
**Crate:** `ecoach-reporting` + `ecoach-student-model`

---

## 8.6 Four BECE Subjects

| Subject | Code | Exam Weight Notes |
|---|---|---|
| English Language | ENG | Reading comprehension, grammar, vocabulary, essay writing |
| Mathematics | MATH | Number, algebra, geometry, statistics, measurement |
| Integrated Science | SCI | Biology, chemistry, physics, earth science concepts |
| Social Studies | SOC | History, geography, civics, economics — Ghana-focused |

All four subjects use the same topic tree, mastery model, memory engine, and question pipeline. Subject-specific configuration is controlled by curriculum pack data.

---

## 8.7 Diagnostic Battery Specification — Academic DNA Test

**Source:** idea34

### Overview

| Parameter | Light Mode | Standard Mode | Deep Mode |
|---|---|---|---|
| Total items | 35–38 | 46–50 | 52–56 |
| Baseline items | 18–20 | 22–24 | 22–24 |
| Adaptive zoom items | 10–12 | 14–16 | 18–20 |
| Condition layer items | 4–5 | 6–7 | 8–9 |
| Stability recheck items | 3–4 | 4–5 | 6–8 |
| Estimated duration | ~35 min | ~60 min | ~90 min |

### Seven Test Stages (Sequential, Adaptive)

1. **FastBaselineScan** — 20–24 items; broad topic coverage; identify strong / weak / uncertain zones
2. **TopicZoom** — automated zoom into weak/unstable areas identified in stage 1; 3–5 items per weak topic
3. **MisconceptionProbing** — expose why student is wrong; distractor-to-misconception mapping; confidence capture
4. **SpeedPressureLayer** — repeat subset under time conditions; compare timed vs untimed performance
5. **TransferLayer** — same concept in 5 forms: direct / word-problem / diagram / comparison / explain-why
6. **ConfidenceCapture** — 3-level rating: `sure` / `not_sure` / `guessed` captured on each item
7. **MicroRecheck** — reintroduce 4–8 items from stage 1 indirectly; test retention vs fluency vs guessing

### Twelve Dimensions Measured

| ID | Dimension | Method |
|---|---|---|
| A | Coverage | Topics touched / total syllabus topics |
| B | Accuracy | Correct rate per topic, per subject |
| C | Recall Strength | Free recall items without hints |
| D | Recognition vs Production | MCQ vs open response performance gap |
| E | Reasoning Depth | Pattern of guess / memorize / understand across items |
| F | Misconception Pattern | Distractor selection → misconception tagging |
| G | Speed | Normalized response latency per item |
| H | Pressure Response | Timed vs untimed accuracy delta |
| I | Transfer Ability | Performance on variant-form items |
| J | Stability | Consistency across parallel items |
| K | Confidence Calibration | Correct-but-unsure, wrong-but-confident ratios |
| L | Fatigue Pattern | Score decay over session duration |

### Output Format

```
DiagnosticReport {
    overall_profile: LearnerArchetype,
    dimensions: DiagnosticDimensionsDto,   // all 12 scores as basis points
    subject_reports: [SubjectDiagnosticDto per subject],
    problem_cards: [ProblemCardDto],        // one per detected weakness
    recommended_focus_topics: [topic_id],
    archetype: "WeakButConsistent" | "StrongButLazy" | "PanickingLastMinute" | "Overconfident" | "Discouraged"
}
```

---

## 8.8 Intelligence Constitution — Engine Registry

All backend engines organized by domain. Priority: P0 = build first, P1 = second wave.

### Domain A — Evidence Domain (6 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Response Evidence Ingestion | A1 | Capture raw attempt signal | Attempt submission DTO | Raw evidence record | `ecoach-sessions` | P0 |
| Content Signal Ingestion | A2 | Capture question/content quality signals | Question usage stats, feedback | Content quality event | `ecoach-questions` | P0 |
| Curriculum Ingestion | A3 | Parse and store curriculum versions | Curriculum pack JSON | Curriculum graph | `ecoach-curriculum` | P1 |
| Learner Signal Ingestion | A4 | Capture behavioral/engagement signals | Session events, timing data | Behavioral signal record | `ecoach-sessions` | P1 |
| Pack Signal Ingestion | A5 | Track content pack usage and coverage | Pack install events | Coverage ledger | `ecoach-content` | P1 |
| Evidence Normalization | A6 | Normalize raw signals into weighted evidence | Raw attempt + context | `EvidenceWeight` value | `ecoach-student-model` | P0 |

### Domain B — Knowledge Domain (10 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Topic Scope | B1 | Resolve topic boundaries and atom graph | Topic ID | Atom list, prerequisite chain | `ecoach-curriculum` | P1 |
| Concept State | B2 | Compute per-concept mastery state | Evidence records | `ConceptState` | `ecoach-student-model` | P0 |
| Topic State | B3 | Aggregate concept states to topic level | Concept states | `StudentTopicState` | `ecoach-student-model` | P0 |
| Hypothesis Competition | B4 | Maintain competing hypotheses about student knowledge | Evidence stream | Hypothesis set with probabilities | `ecoach-student-model` | P0 |
| Misconception | B5 | Track active misconceptions per student | Distractor selections, error types | Misconception profile | `ecoach-student-model` | P0 |
| Interference | B6 | Detect when similar concepts contaminate each other | Error patterns, concept neighbors | Interference pairs | `ecoach-memory` | P0 |
| Learner State | B7 | 7-dimension learner model | All student state tables | `LearnerState` aggregate | `ecoach-student-model` | P0 |
| Mastery Proof | B8 | Generate evidence-backed mastery claim | Topic state + evidence log | `MasteryProof` with evidence refs | `ecoach-student-model` | P0 |
| Coverage Gap | B9 | Compute syllabus coverage and gap map | Topic states, curriculum graph | `GapMapDto` | `ecoach-knowledge-gap` | P1 |
| Knowledge Graph | B10 | Maintain live knowledge graph for student | All mastery/gap data | Graph structure for sequencing | `ecoach-curriculum` | P1 |

### Domain C — Decision Domain (9 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Teaching Strategy | C1 | Select pedagogic approach per content type | Content type, student state | `TeachingStrategy` selection | `ecoach-coach-brain` | P0 |
| Sequencing | C2 | Order topics for maximum learning trajectory | Topic priority scores, dependencies | Ordered topic queue | `ecoach-coach-brain` | P0 |
| Timing | C3 | Determine session timing and pressure level | Student state, availability, urgency | `SessionTimingConfig` | `ecoach-goals-calendar` | P0 |
| Risk | C4 | Assess academic risk level | Mastery trends, exam countdown, gap severity | `RiskProfile` | `ecoach-coach-brain` | P0 |
| Adaptation | C5 | Adjust plan after session outcome | Session result, mastery delta | Plan adjustment commands | `ecoach-coach-brain` | P0 |
| Session Composer | C6 | Build session content from plan + state | Plan day, current state, available questions | `SessionConfig` with question list | `ecoach-sessions` | P0 |
| Content Selection | C7 | Score and select questions for a session | Session config, student state, question pool | Ordered `Vec<SelectedQuestion>` | `ecoach-questions` | P1 |
| Diagnostic Experiment | C8 | Design adaptive diagnostic probes | Weak topic hypothesis set | Diagnostic probe questions | `ecoach-diagnostics` | P1 |
| Protection Rule | C9 | Prevent harmful or counterproductive actions | Proposed action, student state | Allow / block / modify decision | `ecoach-coach-brain` | P1 |

### Domain D — Execution Design Domain (4 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Intervention Design | D1 | Select and configure repair intervention | Error type, gap type, student archetype | `InterventionConfig` | `ecoach-coach-brain` | P1 |
| Drill Generation | D2 | Build targeted drill from gap/error profile | Error fingerprint, topic scope | Drill question sequence | `ecoach-questions` | P1 |
| Mock Orchestration | D3 | Compile mock exam from forecast + gaps | Mock type, blueprint, student state | `MockInstance` with question list | `ecoach-mock-centre` | P1 |
| Assessment Construction | D4 | Build diagnostic or mastery check session | Target dimensions, topic scope | Assessment session config | `ecoach-diagnostics` | P1 |

### Domain E — Memory and Meta-Learning Domain (8 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Topic Memory | E1 | Track memory state per topic | Recall attempts, time gaps, decay signals | `MemoryState` + MSI score | `ecoach-memory` | P1 |
| Learner Memory | E2 | Aggregate memory health across all topics | All `MemoryState` records | `MemoryHealthDto` | `ecoach-memory` | P1 |
| Strategy Memory | E3 | Remember which interventions worked per student | Intervention outcomes | Strategy effectiveness profile | `ecoach-coach-brain` | P1 |
| Coach Self-Evaluation | E4 | Assess whether coach decisions are producing results | Session outcome trends, mastery velocity | Coach decision audit log | `ecoach-coach-brain` | P1 |
| Improvement Velocity | E5 | Track rate of mastery improvement over time | Mastery time series | Velocity score and trend | `ecoach-student-model` | P1 |
| Memory Strength Engine | E6 | Compute MSI from 6-component formula | Accuracy, speed, retention, variant, independence, connection scores | `BasisPoints` MSI | `ecoach-memory` | P0 |
| Decay Detection Engine | E7 | Detect memory decay before collapse | Signal categories (accuracy, time, confidence, support, stability, transfer, interference, behavioral) | `DecayAlert` with decay type | `ecoach-memory` | P0 |
| Recovery Planner | E8 | Build recovery sequence for decayed knowledge | Decay type, current memory state, intervention case (A–G) | Recovery session config | `ecoach-memory` | P1 |

### Domain F — Governance Domain (5 engines)

| Engine | ID | Purpose | Inputs | Outputs | Crate | Priority |
|---|---|---|---|---|---|---|
| Decision Arbitration | F1 | Resolve conflicts between competing engine outputs | Multiple engine recommendations | Single arbitrated action | `ecoach-coach-brain` | P1 |
| Confidence Gate | F2 | Block action when evidence is too weak | Evidence count, confidence score | Allow / defer decision | `ecoach-coach-brain` | P1 |
| Contradiction Check | F3 | Detect inconsistencies in student state | Student state snapshot | Contradiction flags | `ecoach-student-model` | P1 |
| Policy Guardrail | F4 | Enforce 8 Day-1 invariants | Proposed engine action | Compliant / non-compliant verdict | `ecoach-coach-brain` | P1 |
| Audit and Trace | F5 | Record all engine decisions with correlation IDs | All engine outputs | Event log entries | `ecoach-storage` | P0 |

---

*End of agent1_part4.md — Parts 6, 7, and 8 complete.*
