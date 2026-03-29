# eCoach Backend Execution Plan
## Complete Backend-Only Plan — Synthesized from 38 Idea Files

---

# PHASE 3 — CROSS-DOCUMENT BACKEND SYNTHESIS

## 1. Backend Vision

eCoach is a **local-first Academic Intelligence Desktop Operating System** built on a Rust backend + Nuxt/Tauri frontend. It targets BECE exam preparation in Ghana, aiming to replace the need for a human tutor by providing an always-on, evidence-driven coaching brain that diagnoses, teaches, drills, retains, simulates exams, and coaches students toward exam readiness — all running offline on modest hardware.

The backend is the **sole source of truth** for all academic intelligence: mastery state, memory/decay, misconceptions, readiness scoring, exam forecasting, question selection, plan generation, session orchestration, and reporting. The frontend is presentation-only.

## 2. Backend Scope

### In Scope (Backend-Only)
- Curriculum graph (GES/NaCCA → canonical academic nodes)
- Content OS (ingestion, parsing, alignment, quality scoring, publishing)
- Question Intelligence (multi-axis classification, families, variants, misconception mapping)
- Student Model (mastery, memory, errors, pressure, confidence, expression, execution)
- Diagnostic Engine (multi-session battery, hypothesis-based, adaptive probing)
- Coach Brain (state machine, plan engine, mission planner, intervention policy, evidence ledger)
- Memory/Decay Engine (recall modes, decay detection, retention scheduling, interference)
- Session Orchestrator (block composition, live adaptation, presence awareness, time tracking)
- Mock Centre (blueprint compilation, exam simulation, scoring, diagnosis, readiness update)
- Knowledge Gap Mode (gap scoring, repair plans, solidification sessions)
- Elite Mode (tier system, EPS scoring, debrief, prestige)
- Beat Yesterday / Rise Mode / Spark Mode (daily growth engines)
- Customised Testing Engine (purpose-shaped test generation)
- Past Paper Intelligence (question families, recurrence, co-appearance, stories)
- Glossary Lab (knowledge entries, relations, bundles, audio, search, tests)
- Library Intelligence (content relationships, state tracking, auto-shelves, recommendations)
- Games (MindStack/Tetris, Tug of War — game state, scoring, analytics bridge)
- Traps (contrast pairs, evidence atoms, 5 confusion-resolution modes)
- Goal System (multi-goal hierarchy, arbitration, decomposition)
- Document Upload / Academic Intake (OCR, bundle processing, QA alignment, evidence extraction)
- Time Orchestration (availability, scheduling, session awareness, counted time)
- Reporting (student, parent, admin views — readiness, weekly memos, milestone reviews)
- Premium/Concierge layer (strategy engine, risk flags, intervention lifecycle, concierge Q&A)
- Role system (Student, Parent, Super Admin + entitlement tiers: Standard, Premium, Elite)
- Content authoring/review pipeline (super admin foundry)
- Local backup/restore, content pack management

### Out of Scope (for now)
- Frontend implementation (Nuxt/Vue components, styling, animations)
- Cloud sync infrastructure (except as optional future enhancement)
- Mobile app
- Multiplayer/real-time networking
- Teacher/school portal (deferred to later phase)
- AI chatbot / LLM-powered AReal live layer (deferred; rule-based coaching first)

## 3. Major Backend Modules

| # | Module | Core Responsibility |
|---|--------|-------------------|
| 1 | **Curriculum Graph** | Canonical academic truth: subjects, topics, subtopics, concepts, skills, prerequisites, exam blueprints |
| 2 | **Content OS** | Ingest, parse, align, package, quality-score, publish all academic content |
| 3 | **Question Intelligence** | Classify, family-group, misconception-map, variant-generate all questions |
| 4 | **Student Model** | Per-student mastery, memory, errors, pressure, confidence, expression, execution state |
| 5 | **Diagnostic Engine** | Multi-phase adaptive assessment producing detailed academic profiles |
| 6 | **Coach Brain** | State machine, plan engine, mission planner, intervention selector, evidence evaluator |
| 7 | **Memory Engine** | Decay detection, recall classification, retention scheduling, interference mapping |
| 8 | **Session Orchestrator** | Session composition, block sequencing, live adaptation, presence/time tracking |
| 9 | **Mock Centre** | Exam simulation: blueprint compilation, runtime, scoring, post-mock diagnosis |
| 10 | **Knowledge Gap** | Gap scoring, repair planning, solidification sessions |
| 11 | **Question Factory** | Generate questions from concept primitives using transformation operators |
| 12 | **Goal & Calendar** | Multi-goal hierarchy, exam timeline, intensity phases, dynamic replanning |
| 13 | **Academic Intake** | Document upload, OCR, bundle processing, QA alignment, evidence extraction |
| 14 | **Reporting & Insights** | Readiness scoring, weekly memos, milestone reviews, parent/admin dashboards |
| 15 | **Identity & Roles** | Users, PINs, role-based access, entitlement tiers (Standard/Premium/Elite) |
| 16 | **Pack Manager** | Content pack install, verification, activation, versioning, migration |
| 17 | **Game Engines** | MindStack, Tug of War, Traps — game state, scoring, learning analytics bridge |
| 18 | **Library Intelligence** | Content relationships, item states, auto-shelves, recommendations, search |
| 19 | **Glossary Lab** | Knowledge entries, relations, bundles, search, audio, tests |
| 20 | **Past Paper Intel** | Question family mining, recurrence analytics, exam stories |

## 4. Domain Model Overview

### Core Academic Truth
- **CurriculumVersion** → **Subject** → **Topic** → **Subtopic** → **AcademicNode** (polymorphic: Definition, Concept, Formula, Procedure, Comparison, etc.)
- Nodes connected by typed edges: prerequisite, related, confused_with, uses_formula, assessed_by, etc.
- MisconceptionPattern linked to nodes
- ExamBlueprint linked to curriculum version

### Content
- **SourceAsset** → **SourcePage** → **SourceSegment** → **EvidenceBlock** / **ContentAtom**
- **ContentArtifact** (note, glossary, formula_card, worked_example, misconception_card, question)
- Each artifact has: trust_state, quality_score, provenance, curriculum alignment, version history

### Questions
- **QuestionFamily** → **QuestionTemplate** → **GeneratedQuestion** / **QuestionInstance**
- **QuestionIntelligence** (8-axis classification: knowledge_role, cognitive_demand, solve_pattern, pedagogic_function, content_grain, family, misconception_exposure, confidence/review)
- **QuestionOption** with misconception tags and distractor intent

### Student
- **User** (account_id, type, pin_hash) → **StudentProfile** / **ParentProfile** / **AdminProfile**
- **StudentTopicState** (mastery, stability, confidence, weakness_severity, trend, evidence_count)
- **StudentMemoryState** (memory_strength, recall_fluency, decay_risk, spacing_need)
- **StudentSkillState** (per-skill mastery, gap_score, priority_score, knowledge_state)
- **StudentPressureState** (timed vs untimed accuracy, speed collapse, recovery)
- **StudentErrorProfile** (per error type intensity scores)
- **StudentConfidenceState** (calibration, over/under-confidence)
- **StudentExpressionState** (completeness, structure, clarity per subject)

### Coach
- **CoachPlan** → **CoachPlanDay** → **CoachPlanActivity**
- **CoachMission** → **MissionStep**
- **CoachTopicProfile** (mastery_estimate, fragility, speed, misconception_recurrence, blocked_status)
- **CoachSessionEvidence** (per-session outcomes)
- **InterventionPlan** (family, goal, steps, verification, fallback)
- **CoachBlocker** / **CoachRepairObligation**

### Sessions & Assessment
- **DiagnosticTestInstance** → **DiagnosticSessionInstance** → **SessionQuestionInstance** → **StudentResponse**
- **MockSession** → **MockItem** → **Answer** → **Score** → **Diagnosis**
- **SolidificationSession** → **SolidificationStep**
- **CoachingEpisode** → **EngagementSegment** (with presence/progress states)

### Memory
- **LearnerSkillMemory** (12-state machine, proof dimensions, RAS/DCS/DRS scores)
- **MemoryEvidenceEvent** (per-attempt with mode, variant, cue level, delay, interference)
- **RecheckSchedule** (adaptive spaced repetition)
- **InterferenceEdge** (confusion pairs with directional strength)

### Goals & Calendar
- **Goal** (hierarchy: north_star, campaign, tactical, background; states: drafted→active→completed)
- **CalendarEvent** (exam, mock, class_test, assignment + preparation profiles)
- **ExamPreparationProfile** / **SubjectPreparationProfile** / **TopicPreparationProfile**

## 5. Service Boundaries

The system is a **modular monolith** in Rust (not microservices). Modules are separate crates in a Cargo workspace with explicit API boundaries:

```
ecoach-backend/
├── crates/
│   ├── substrate/        # Shared types, scoring wrappers, confidence semantics, config
│   ├── storage/          # SQLite repositories, migrations, query builders
│   ├── events/           # Domain event types, outbox, event bus
│   ├── identity/         # Users, PINs, roles, entitlements
│   ├── curriculum/       # Curriculum graph, ingestion, enrichment, publishing
│   ├── content/          # Content OS: artifacts, quality, trust, packs
│   ├── questions/        # Question intelligence, families, generation, classification
│   ├── student-model/    # All student state: mastery, memory, errors, pressure, confidence
│   ├── diagnostics/      # Diagnostic battery, adaptive probing, hypothesis engine
│   ├── coach-brain/      # State machine, plan engine, mission planner, intervention policy
│   ├── memory/           # Decay detection, recall modes, retention scheduling, interference
│   ├── sessions/         # Session orchestrator, block composer, presence, time tracking
│   ├── mock-centre/      # Mock compilation, runtime, scoring, post-mock diagnosis
│   ├── knowledge-gap/    # Gap scoring, repair plans, solidification
│   ├── goals-calendar/   # Goals, exams, timeline, intensity phases
│   ├── intake/           # Document upload, OCR bridge, bundle processing, QA alignment
│   ├── reporting/        # Readiness, weekly memos, parent/admin views
│   ├── library/          # Content relationships, shelves, recommendations
│   ├── glossary/         # Knowledge entries, relations, bundles, search, audio
│   ├── games/            # MindStack, Tug of War, Traps
│   ├── past-papers/      # Question family mining, recurrence, exam stories
│   ├── premium/          # Strategy engine, risk flags, concierge, premium reporting
│   └── commands/         # Tauri command boundary (all frontend-facing API)
├── migrations/           # SQLite schema migrations
├── packs/                # Content pack format definitions
├── tests/                # Integration tests, fixtures, benchmarks
└── Cargo.toml            # Workspace manifest
```

Communication between crates: **direct function calls** within process, using trait-based interfaces. No HTTP between modules. The only HTTP-like boundary is the Tauri command layer to the frontend.

## 6. Auth and Permission Model

- **Local PIN-based auth** (no email/password for v1)
- Three roles: **Student**, **Parent**, **Super Admin**
- Parent links to 1+ students; admin sees all
- Three entitlement tiers: **Standard**, **Premium**, **Elite** (policy layers, not separate codebases)
- Role-based shell loading: different command sets, DTOs, and data visibility per role
- PIN stored as hash+salt; lockout after repeated failures

## 7. Data/Storage Model

- **SQLite** (WAL mode) as primary local database
- **Local filesystem** for content packs, uploaded documents, audio assets, diagrams, backups
- **Append-only event log** for crash safety and optional future sync
- Student data and content data stored separately (content is replaceable; student data is not)
- All scores stored as basis points (u16) for cross-platform consistency
- JSON columns for flexible structured data (difficulty profiles, answer keys, metadata)

## 8. Background Jobs

All background processing runs in-process (no Redis, no external queue — this is an offline desktop app):

- **Post-session cascade**: evidence → mastery update → memory update → plan adjustment → readiness update
- **Decay scan**: periodic check for concepts at risk of being forgotten
- **Retention scheduler**: compute next review dates
- **Content pack operations**: install, verify, index rebuild
- **Daily mission assembly**: from plan + evidence + decay + weakness
- **Weekly plan generation**: from goals + exams + evidence + capacity
- **Document processing pipeline**: OCR → parse → align → score (when documents uploaded)

## 9. Testing Strategy

- **Unit tests**: per-crate, covering scoring formulas, state machines, business rules
- **Integration tests**: cross-crate workflows (e.g., answer → mastery update → plan change)
- **Fixture-based**: seed databases with known student states and content for deterministic testing
- **Property-based**: for scoring formulas, ensure scores stay in valid ranges
- **Snapshot tests**: for complex DTOs (readiness reports, session blueprints)
- **Risk-based priority**: test mastery state machine, scoring formulas, plan engine, question selection first

---

# PHASE 4 — CLARIFICATION GATE

The 38 files provide sufficient information to proceed with planning. I record these **assumptions** where ambiguity exists:

1. **Database**: SQLite locally, not PostgreSQL (despite some files mentioning Postgres — those describe cloud/admin scenarios)
2. **No LLM at runtime**: v1 coaching brain is rule-based and deterministic; AI-assisted content generation happens at authoring/build time, not at student runtime
3. **Teacher role**: deferred to later phase; not in v1
4. **OCR**: will be handled by a bundled local library or deferred; not a cloud service
5. **idea7 = idea8**: confirmed duplicate; treat as one file
6. **Go references** (idea27, idea32): these describe a cloud CIE/admin system; the student-facing local backend remains Rust
7. **AReal**: deferred; v1 uses structured coach messages, not a live AI persona
8. **Product name**: "eCoach" is working name; "Adeo" appears in some files as alternative

---

# PHASE 5 — BACKEND EXECUTION PLAN

## A. Backend Architecture

**Style**: Modular monolith in Rust
**Why not microservices**: Single local desktop app, no network between services, needs instant response times, offline-only
**Why not pure monolith**: 20+ distinct domain areas would become unmaintainable without module boundaries

**Communication**: Direct trait-based function calls between crates. Event outbox pattern for cascading updates (e.g., answer submitted → mastery recalculated → plan adjusted).

**Frontend boundary**: Tauri commands. Frontend calls `invoke('command_name', payload)`, backend returns typed DTOs. Frontend never computes mastery, readiness, or makes academic decisions.

## B. Domain and Data Design

### Core Entities (priority order for implementation)

1. **Curriculum**: Subject, Topic, Subtopic, AcademicNode, SkillDependency, MisconceptionPattern
2. **Questions**: Question, QuestionOption, QuestionFamily, QuestionIntelligence, QuestionSkillLink
3. **Users**: Account, StudentProfile, ParentProfile, AdminProfile, ParentStudentLink
4. **Student State**: StudentTopicState, StudentSkillState, StudentMemoryState, StudentErrorProfile
5. **Sessions**: DiagnosticSession, MockSession, PracticeSession, CoachingEpisode
6. **Coach**: CoachPlan, CoachPlanDay, CoachMission, CoachTopicProfile, SessionEvidence
7. **Content**: ContentPack, ContentArtifact, SourceAsset, TrustState
8. **Goals**: Goal, CalendarEvent, ExamPreparationProfile

### Key Invariants
- No mastery verdict without multi-gate proof (accuracy + retention + transfer + pressure)
- No content published without provenance and quality scoring
- No student state change without evidence
- No plan without traceability
- Every wrong answer must carry diagnostic value

## C. API and Interface Layer

All APIs are **Tauri commands** (Rust functions exposed to the Nuxt frontend via IPC):

### Priority 1 Commands
- `create_account`, `login_with_pin`, `switch_account`, `get_current_user`
- `get_subjects`, `get_topic_tree`, `get_topic_detail`
- `start_diagnostic`, `get_next_question`, `submit_answer`, `complete_diagnostic`, `get_diagnostic_results`
- `get_coach_state`, `get_today_mission`, `start_mission`, `complete_mission`
- `get_student_dashboard`, `get_readiness_overview`
- `install_content_pack`, `list_installed_packs`

### Priority 2 Commands
- `start_mock`, `pause_mock`, `resume_mock`, `submit_mock`, `get_mock_review`
- `get_knowledge_gap_overview`, `start_solidification_session`
- `get_memory_overview`, `start_memory_session`
- `get_weekly_plan`, `get_daily_plan`
- `create_custom_test`, `start_custom_test`

### Priority 3 Commands
- `upload_document`, `get_upload_status`, `get_upload_findings`
- `get_parent_dashboard`, `get_child_summary`, `get_weekly_memo`
- `start_game_session`, `get_game_state`, `submit_game_answer`
- `search_glossary`, `get_glossary_entry`
- `search_library`, `get_library_shelves`

### DTO Pattern
- Commands return **presentation-safe DTOs** that hide engine internals
- Readiness shown as bands (Not Ready / Building / Strong / Exam-Ready), not raw scores
- Error responses are structured with error codes and user-facing messages

## D. Auth, Roles, and Security

- **PIN-based local auth**: 4-6 digit for students, 6+ for parents/admin
- **Role-based command filtering**: student commands, parent commands, admin commands
- **Data isolation**: students cannot access other students' data
- **Parent can**: view linked children, reset child PINs, see summaries
- **Admin can**: manage all accounts, content, curriculum, system settings
- **No secrets in content packs**: packs are signed but not encrypted
- **Backup encryption**: optional, user-controlled

## E. Background Processing

All in-process, using Rust async tasks:

| Job | Trigger | Priority |
|-----|---------|----------|
| Post-answer evidence cascade | Every answer submission | Highest |
| Mastery/memory/plan recalculation | After session completion | Highest |
| Decay scan | App launch + daily timer | High |
| Retention scheduling | After mastery changes | High |
| Daily mission assembly | App launch + plan changes | High |
| Content pack install/verify | User action | Medium |
| Document processing pipeline | Upload action | Medium |
| Weekly plan generation | Weekly timer + event triggers | Medium |
| Library shelf regeneration | After mastery changes | Low |

## F. Infrastructure and DevOps

- **Single binary**: Rust backend compiled into Tauri desktop app
- **SQLite database**: in user's app data directory
- **Content packs**: in app data directory under `/packs/`
- **Migrations**: embedded in binary, run on app launch
- **Backups**: export to user-chosen location (USB, local folder)
- **Logging**: structured local logs for debugging
- **No cloud dependency**: everything runs offline
- **Updates**: app binary updates via standard desktop update mechanism; content updates via pack installation

## G. Codebase Structure

See Section 5 above for the full crate layout.

## H. Testing Strategy

See Section 9 above.

## I. Delivery Roadmap

### Phase 0: Foundations (Weeks 1-3)
**Goals**: Buildable workspace, database, identity, curriculum skeleton
**Deliverables**:
- Cargo workspace with substrate, storage, identity, curriculum crates
- SQLite setup with WAL mode, migration framework
- Account creation, PIN auth, role routing
- Curriculum graph: subjects, topics, subtopics, academic nodes, prerequisites
- Content pack format definition and loader
- Tauri shell with command boundary
**Exit criteria**: Can create accounts, load a curriculum, browse topics

### Phase 1: Core Primitives (Weeks 4-8)
**Goals**: Questions work, students can answer things, evidence is captured
**Deliverables**:
- Question entity with full metadata (topic, difficulty, skill, family, misconception tags)
- Question retrieval by topic/difficulty/family
- Student model: topic mastery state, skill state, error tracking
- Answer processing pipeline: submit → score → classify error → update mastery
- Basic scoring formulas (mastery, gap, priority)
- Practice session: select topic → get questions → answer → see results
- Content pack with sample Mathematics content (1 subject, 10+ topics, 200+ questions)
**Exit criteria**: Student can practice, mastery updates correctly, errors are classified

### Phase 2: Essential Workflows (Weeks 9-14)
**Goals**: Diagnostic works, coach plans, daily missions exist
**Deliverables**:
- Diagnostic engine: multi-phase adaptive battery (baseline, precision, speed, pressure, flex)
- Post-diagnostic student profile generation
- Coach brain state machine (14 states)
- Plan engine V1: deterministic plan from exam date + diagnostic evidence
- Mission planner: daily mission generation from plan + evidence
- Session orchestrator: block composition, presence tracking, time counting
- Memory engine V1: decay detection, basic retention scheduling
- Knowledge gap mode: gap scoring, repair queue, basic solidification sessions
- Parent dashboard: readiness overview, subject summaries, risk flags
**Exit criteria**: Full loop: diagnostic → plan → daily missions → evidence → plan updates

### Phase 3: Exam Simulation & Intelligence (Weeks 15-20)
**Goals**: Mock exams work, question intelligence active, past paper analytics
**Deliverables**:
- Mock Centre: blueprint compilation, exam runtime, strict timing, scoring, post-mock diagnosis
- Question Intelligence Engine: 8-axis classification pipeline (rule-based first)
- Past Paper Intelligence: family grouping, recurrence, co-appearance analytics
- Customised Testing Engine: purpose-shaped test generation
- Goal system: multi-goal hierarchy, exam timeline, intensity phases
- Calendar engine: event-driven replanning, preparation phases
- Premium reporting: weekly strategy memos, milestone reviews
- Readiness engine: multi-dimensional readiness scoring
**Exit criteria**: Student can take realistic mocks, get intelligent feedback, see readiness

### Phase 4: Hardening & Scale (Weeks 21-26)
**Goals**: Polish, edge cases, performance, content depth
**Deliverables**:
- Elite Mode: tier system, EPS scoring, session types
- Beat Yesterday / Rise Mode: daily growth engines
- Games: MindStack (Tetris-learning hybrid)
- Traps: confusion-pair modes (5 modes)
- Glossary Lab: knowledge entries, search, bundles
- Library Intelligence: auto-shelves, recommendations
- Document intake: basic upload → OCR → evidence extraction
- Backup/restore system
- Content expansion: additional subjects, more questions per topic
- Performance optimization: query optimization, caching, startup time
- Comprehensive test suite
**Exit criteria**: Feature-complete for v1 launch, stable, performant

## J. Immediate Build Order

1. **Cargo workspace + SQLite + migrations** — the foundation everything else depends on
2. **Account/PIN system** — need identity before anything else
3. **Curriculum graph tables + loader** — the academic truth layer
4. **Content pack format + installer** — how content enters the system
5. **Question entity + retrieval** — the atomic unit of all learning
6. **Student topic state + answer processing** — the core evidence loop
7. **Basic practice session** — first working vertical slice
8. **Diagnostic engine** — produces the student profile that drives everything
9. **Coach brain state machine + plan engine** — the orchestration layer
10. **Mission planner + session orchestrator** — daily student experience
11. **Memory engine + retention scheduling** — prevents knowledge decay
12. **Knowledge gap mode** — targeted weakness repair
13. **Mock Centre** — exam simulation
14. **Parent dashboard + reporting** — stakeholder visibility
15. **Question intelligence classification** — smarter question selection
16. **Goal system + calendar** — multi-exam planning
17. **Everything else** (games, glossary, library, traps, elite, premium, intake)

---

## Key Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Performance, safety, offline desktop, single binary |
| Database | SQLite (WAL) | Local-first, no server, embedded, proven |
| Architecture | Modular monolith | Single process for speed, module boundaries for sanity |
| Frontend boundary | Tauri IPC commands | Clean separation, typed DTOs |
| AI at runtime | None (v1) | Offline-first, deterministic, rule-based coaching |
| AI at build time | Yes (content generation) | Quality content requires AI assistance during authoring |
| Scoring storage | Basis points (u16) | Cross-platform consistency, no floating-point drift |
| Event model | In-process outbox | Cascading updates without external queue |
| Content delivery | Signed packs | Offline distribution, integrity verification |
| Auth | Local PIN | No internet required, simple for children |

---

## Contradictions Resolved

| Conflict | Resolution |
|----------|------------|
| Rust vs Go backend | Rust for student-facing local backend; Go only for future cloud admin tools |
| SQLite vs PostgreSQL | SQLite locally; Postgres only for future cloud services |
| Offline vs online enrichment | Core is offline; online enrichment is optional premium enhancement |
| LLM vs rule-based coaching | Rule-based for v1 runtime; LLM for content authoring pipeline only |
| Single app vs microservices | Modular monolith; separate crates, single binary |
| idea7 = idea8 | Confirmed duplicate; one Memory Mode spec |

---

## Open Questions Carried Forward (non-blocking)

1. Exact numerical thresholds for mastery promotion/demotion (start with documented defaults, tune with real data)
2. Exact retention decay curve formula (start with modified SM-2, iterate)
3. OCR technology for Ghanaian school documents (evaluate Tesseract locally, defer if quality insufficient)
4. Target hardware specifications for student devices in Ghana
5. Content pack distribution mechanism (USB flash drive primary, LAN secondary)
6. Product name finalization (eCoach vs Adeo)
7. Whether teacher role belongs in v1 or v2
8. Free-text/essay scoring approach for offline use
