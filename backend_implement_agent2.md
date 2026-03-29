# backend_implement_agent2

Grounded backend implementation plan based on full reading of:
- `idea1.txt` through `idea38.txt`
- `adeo eCoachGames - IDEAS & CONCEPTS.txt`

This plan is backend-only. It distinguishes:
- `Explicit corpus`: directly and repeatedly stated in the documents
- `Strong inference`: strongly implied by the corpus
- `Recommendation`: my implementation choice where the corpus leaves room

## 0. Source of Truth and Ideas Coverage

Primary source of truth for this implementation plan:
- `idea1.txt` through `idea38.txt`
- `adeo eCoachGames - IDEAS & CONCEPTS.txt`

Secondary reconciliation only:
- `backend_execution_plan.md`
- `detailed_backend_implementation_plan.md`

Rule:
- the existing Markdown plans do not override the ideas corpus
- where they add useful runtime specificity, they are included only if they remain consistent with the ideas documents

Ideas-to-implementation coverage map:

- `idea1`: offline-first exam-performance operating system, mock centre, question reactor, local Rust runtime
- `idea2`: journey graph, subject progression, route recalculation, retention/reactivation, subject packs
- `idea3`: local multi-user model, student/parent/admin roles, PIN auth, role-scoped shells
- `idea4`: weak-learner transformation engine, daily growth, rescue/stabilize/accelerate logic
- `idea5`: elite-performance layer, advanced scoring, domination states, entitlement overlay
- `idea6`: knowledge-gap engine, gap-vs-priority logic, blocker tracing, repair planning
- `idea7`: memory mode, spaced proof, decay and rescue logic
- `idea8`: duplicate memory-mode reinforcement of `idea7`
- `idea9`: cognitive question taxonomy, question factory, structured concept records
- `idea10`: game-runtime implications, especially answer-linked stateful play
- `idea11`: wrong-answer intelligence, distractor intent, recognition-failure diagnosis
- `idea12`: premium parent command-center outputs, risk/intervention/readiness projection
- `idea13`: past-paper intelligence, family graph, recurrence/co-appearance/replacement analytics
- `idea14`: customized testing / prepare-for-my-test orchestration and session blueprinting
- `idea15`: missing-core systems, especially Teach Mode, Mistake Lab, Mission Control, retention loops
- `idea16`: Library as knowledge hub with typed objects, shelves, and recommendation/state engines
- `idea17`: Glossary Lab, knowledge graph, bundles, audio programs, transfer-oriented glossary testing
- `idea18`: multi-session diagnostic battery, learning-MRI profiling, root-cause inference
- `idea19`: coach-orchestrated topic learning, missions, calendar-aware planning, evented runtime
- `idea20`: deterministic coach state machine, next-action resolver, capability orchestration
- `idea21`: resource intelligence/orchestration, coverage sufficiency, retrieval/generation fallback rules
- `idea22`: multi-axis question-intelligence engine, taxonomy governance, review thresholds
- `idea23`: local coach brain, causal diagnosis, doctrine rules, tension/interference handling
- `idea24`: autonomous offline academic coach, evidence-led readiness proof, learner digital twin
- `idea25`: intelligence constitution, governed engine lattice, proof gates, audit/governance substrate
- `idea26`: curriculum intelligence portal and Academic Foundry, provenance, publish gates, delta enrichment
- `idea27`: content intelligence engine / knowledge refinery, verified internal retrieval, source tiers
- `idea28`: expansion from current 6-engine nucleus to shared fabrics, hard gates, DB ownership, recompute chain
- `idea29`: pedagogy-to-backend instructional platform, content-type-aware teaching runtime, review-as-diagnosis
- `idea30`: time orchestration, live session awareness, counted-time truth, replanning
- `idea31`: versioned executable central curriculum graph, update diff/impact/regeneration controls
- `idea32`: academic decay and recall resilience subsystem, event-sourced memory state
- `idea33`: Traps / contrast engine for confusable concepts, reusable contrast profiles and evidence atoms
- `idea34`: one-session adaptive DNA diagnostic, problem-cause-fix outputs, constrained diagnostic generation
- `idea35`: rebuild discipline, Rust-first delivery guidance, vertical-slice backend sequencing
- `idea36`: CoachHub operational model, multi-goal orchestration, upload bundle intelligence, admin oversight
- `idea37`: meta-architecture, directive/read-model layer, Content OS, capability-to-diagnosis registry
- `idea38`: top-level local-first academic intelligence OS, policy-layer premium/elite, shared student/content truth
- `adeo eCoachGames - IDEAS & CONCEPTS`: expansion game ideas informing future game-runtime architecture only

Implementation rule:
- if a future revision removes or weakens a major area above, that revision should explicitly say which idea file(s) it is intentionally deferring
- no major backend module should exist without at least one idea-source justification
- no major idea-family should be silently dropped from roadmap scope

## 1. Implementation Intent

Build eCoach as a local-first academic intelligence platform with two connected backend planes:

1. `Learner Runtime Plane`
- Runs locally on the student device
- Owns learner truth, session execution, diagnostics, planning, memory, readiness, glossary/library state, and parent/admin read models on-device

2. `Content Operating System Plane`
- Runs centrally for super-admin/content operations
- Owns curriculum ingestion, source mining, OCR/parsing, artifact building, quality gates, publishing, versioning, and pack delivery

`Explicit corpus`: the product is not a simple question app; it is an academic coach with a hidden intelligence core and a content operating system.

`Recommendation`: treat these as two modular monoliths sharing schemas/contracts, not a microservice mesh.

## 2. Canonical Decisions To Freeze Before Coding

These need to be frozen first because many documents use drifting names.

### 2.1 Canonical academic unit

`Recommendation`
- `CurriculumNode`: official syllabus node
- `ConceptAtom`: smallest teachable meaning unit
- `SkillAtom`: smallest assessable/performance unit
- `KnowledgeUnit`: umbrella interface implemented by `ConceptAtom` and `SkillAtom`

Rule:
- learner evidence anchors to `SkillAtom`
- explanatory/knowledge content anchors to `ConceptAtom`
- both map upward to `CurriculumNode`

### 2.2 Shared truth rules

`Explicit corpus`
- One shared learner truth
- One shared content truth
- Premium/elite are policy layers, not separate architectures

`Recommendation`
- local runtime is source of truth for learner state
- foundry is source of truth for approved curriculum/content artifacts
- read models are derived only; never source of truth

### 2.3 Runtime style

`Explicit corpus`
- local-first
- Rust backend emphasis
- offline behavior is mandatory for core student flows

`Recommendation`
- runtime backend = Rust modular monolith + SQLite + local filesystem asset store
- foundry backend = Rust control plane + Postgres + object storage
- optional specialized OCR/ML worker adapters may be separate if needed, but domain logic remains in shared backend modules
- Tauri v2 is the frontend/backend boundary for the local app
- SQLite should run in WAL mode
- runtime background work should use in-process `tokio` tasks
- all percentage-like scores should use `BasisPoints = u16`

### 2.4 Decision discipline

`Explicit corpus`
- no mastery from one success
- no readiness without explainable evidence
- no live content without provenance and review rules
- UI should render decisions, not invent academic logic

`Recommendation`
- enforce this with hard gates and policy evaluators at the backend layer

## 3. Architecture Style

## 3.1 Learner Runtime Plane

`Recommendation`: modular monolith

Why:
- shared learner truth is central
- many engines need low-latency in-process coordination
- offline-first behavior is easier with one local process
- event replay, recompute, and state consistency are easier than in distributed services

Internal communication:
- synchronous command/query for hot user actions
- internal domain events for recompute pipelines
- async local worker queue for heavier jobs

## 3.2 Content Operating System Plane

`Recommendation`: separate modular monolith

Why:
- source ingestion/OCR/quality/publish are operationally different from learner runtime
- review flows, artifact lifecycles, and versioning need central governance
- packs can be signed and shipped from here into runtime clients

Internal communication:
- API handlers for admin actions
- job queue for OCR, parsing, build, comparison, publish, regeneration

## 4. Major Backend Modules

## 4.1 Shared Domain Modules

1. `domain-curriculum`
- official curriculum versions
- node hierarchy
- node relationships
- node coverage/accountability

2. `domain-knowledge`
- concepts, definitions, formulae, glossary entries
- knowledge bundles
- contrast/traps pairs

3. `domain-question-intelligence`
- question taxonomy
- question families
- misconception tags
- lineage and mutation

4. `domain-content-artifacts`
- explanations
- worked examples
- drills
- revision packs
- instructional artifacts

## 4.2 Learner Runtime Modules

5. `domain-learner-truth`
- mastery state
- memory state
- pressure state
- confidence/risk state
- learner snapshots

6. `domain-diagnostics`
- multi-session diagnostics
- one-session DNA diagnostics
- problem cards
- hypothesis/probe flows

7. `domain-instruction`
- teach mode
- repair/reteach
- content-type-aware move selection
- review intent routing

8. `domain-memory`
- decay
- recall classification
- rescue/review scheduling
- interference handling

9. `domain-planning`
- coach state
- next action resolution
- mission/day/week planning
- time orchestration
- live session awareness

10. `domain-session-runtime`
- session launch
- attempt capture
- counted-time truth
- adaptation events

11. `domain-readiness`
- readiness claims
- proof gates
- danger zones
- readiness reports

12. `domain-library`
- typed library items
- shelves
- mistake bank
- memory shelf
- revision packs

13. `domain-parent-admin-runtime`
- local role views
- parent digest/read models
- entitlements
- local admin controls

## 4.3 Content OS / Foundry Modules

14. `foundry-source-intake`
- uploads
- source classification
- OCR/layout extraction

15. `foundry-curriculum-ingestion`
- parse curriculum docs
- recover hierarchy
- map official codes
- review/publish versions

16. `foundry-artifact-build`
- source segmentation
- evidence extraction
- artifact build jobs
- delta enrichment

17. `foundry-quality-governance`
- provenance
- trust scoring
- comparisons
- review queues
- publish gating

18. `foundry-packaging-delivery`
- pack compilation
- signatures
- version manifests
- pack release

## 4.4 Platform Modules

19. `platform-storage`
20. `platform-events`
21. `platform-jobs`
22. `platform-search`
23. `platform-audit`
24. `platform-policy`
25. `platform-evaluation`

## 5. Canonical Domain Model

## 5.1 Curriculum

- `CurriculumVersion`
- `Subject`
- `Strand`
- `SubStrand`
- `CurriculumNode`
- `CurriculumRelation`
- `NodeCoverageLedger`
- `VersionDiff`
- `PublishingRule`

## 5.2 Knowledge / Glossary / Formula / Contrast

- `KnowledgeEntry`
- `DefinitionMeta`
- `FormulaMeta`
- `ConceptMeta`
- `KnowledgeRelation`
- `KnowledgeBundle`
- `KnowledgeBundleItem`
- `ContrastPair`
- `ContrastEvidenceAtom`
- `QuestionGlossaryLink`

## 5.3 Questions

- `QuestionItem`
- `QuestionOption`
- `QuestionAsset`
- `QuestionIntelligenceSnapshot`
- `QuestionFamily`
- `QuestionFamilyMember`
- `MisconceptionPattern`
- `DistractorIntent`

## 5.4 Learner Truth

- `LearnerProfile`
- `LearnerStateSnapshot`
- `SkillState`
- `ConceptState`
- `MemoryState`
- `PressureState`
- `RiskAssessment`
- `ConfidenceSignal`
- `ReadinessState`
- `InterferenceEvent`

## 5.5 Evidence / Diagnostics / Interventions

- `EvidenceEvent`
- `EvidencePacket`
- `DiagnosticCase`
- `DiagnosticHypothesis`
- `DiagnosticProbe`
- `ProblemCard`
- `InterventionPlan`
- `InterventionStep`
- `RepairPlan`
- `ReadinessClaim`
- `ReadinessContract`

## 5.6 Planning / Runtime

- `CoachLifecycleState`
- `NextCoachAction`
- `Mission`
- `MissionItem`
- `DailyPlan`
- `WeeklyPlan`
- `SessionPlan`
- `SessionBlock`
- `SessionObject`
- `EngagementSegment`
- `PresenceStateSnapshot`
- `ScheduleLedger`

## 5.7 Library / Personal Knowledge

- `LibraryItem`
- `LibraryShelf`
- `MistakeCase`
- `RevisionPack`
- `SavedBundle`
- `StudentEntryState`

## 5.8 Foundry / Content OS

- `SourceDocument`
- `SourceSegment`
- `ParseJob`
- `Artifact`
- `ArtifactVersion`
- `ArtifactQualityReport`
- `ArtifactReview`
- `ContentTrustState`
- `FoundryJob`
- `PublishVersion`
- `PackManifest`

## 6. Persistence Strategy

## 6.1 Learner Runtime Storage

`Recommendation`
- SQLite for structured state
- local filesystem for packs, audio, images, OCR artifacts, and cached derived assets
- FTS search index for library/glossary/content search

Data categories:

1. `append-only event tables`
- evidence
- behavior
- session events
- engagement
- decision traces

2. `state tables`
- learner mastery/memory/risk/readiness
- plan/missions
- glossary entry state
- local role/account state

3. `reference tables`
- installed curriculum/content pack data
- question metadata
- glossary structures
- family mappings

4. `read models`
- home dashboard
- coach directives
- parent digest
- library shelves

Runtime storage conventions:
- table names: `snake_case`
- timestamps: ISO 8601 text
- flexible metadata: validated JSON text columns
- foreign keys: always enabled
- every hot-query foreign key gets an index

Recommended runtime migration rollout:
- `001_identity`
- `002_curriculum`
- `003_questions`
- `004_student_state`
- `005_sessions`
- `006_coach`
- `007_memory`
- `008_mock_centre`
- `009_knowledge_gap`
- `010_goals_calendar`
- `011_content_packs`
- `012_reporting`
- `013_glossary`
- `014_library`
- `015_games`
- `016_past_papers`
- `017_traps`
- `018_intake`
- `019_premium`
- `020_elite`

## 6.2 Foundry Storage

`Recommendation`
- Postgres for operational metadata and workflows
- object storage for raw files, OCR outputs, parsed JSON, artifact payloads, pack outputs
- search index for source and artifact retrieval

## 6.3 Event-Sourced Pattern

`Strong inference`
- many docs want replay, audit, and explainability

`Recommendation`
- use hybrid event-sourcing:
  - append-only evidence/decision/event ledger
  - materialized state tables for hot reads
  - replay/rebuild tools for state regeneration

## 7. Backend Boundaries and Ownership

Freeze module write ownership early.

### 7.1 Runtime ownership

- `domain-session-runtime` writes raw attempt/session events
- `domain-learner-truth` owns derived learner state
- `domain-memory` owns decay/review scheduling state
- `domain-diagnostics` owns diagnostic cases/problem cards
- `domain-planning` owns missions/plans/coach lifecycle state
- `domain-readiness` owns readiness claims/contracts/verdicts
- `domain-library` owns shelves and personal item states

### 7.2 Foundry ownership

- `foundry-source-intake` owns raw-source lifecycle
- `foundry-curriculum-ingestion` owns official curriculum versions and review states
- `foundry-artifact-build` owns candidate artifact generation
- `foundry-quality-governance` owns trust/review/comparison/publish gating
- `foundry-packaging-delivery` owns built pack versions and manifests

### 7.3 Shared truth rule

No module writes another module's source-of-truth tables directly.
Cross-domain updates happen through:
- commands
- domain events
- recompute jobs

## 8. Core Recompute and Event Flows

## 8.1 Runtime hot path

`submit_attempt`
-> persist raw attempt event
-> classify question outcome/misconception
-> update skill/concept evidence
-> update memory/risk/pressure as applicable
-> recompute topic state
-> recompute coach lifecycle/next action
-> update read models

## 8.2 Session completion path

`session_completed`
-> finalize counted-time truth
-> update plan adherence
-> recalculate readiness deltas
-> generate new missions or recovery actions
-> emit parent/admin digest deltas if role-enabled

## 8.3 Diagnostic flow

`launch_diagnostic`
-> build diagnostic blueprint
-> run adaptive session
-> create/update hypotheses
-> generate problem cards
-> route to repair/intervention plans
-> update plan and readiness

## 8.4 Memory flow

`retrieval_attempt_recorded`
-> classify recall type
-> detect decay/interference
-> update memory state
-> reschedule review queue
-> optionally generate rescue pack or weakness radio

## 8.5 Foundry flow

`source_uploaded`
-> classify source
-> OCR/layout parse
-> segment
-> map to curriculum
-> evidence extraction
-> build candidate artifacts
-> compare against existing artifact version
-> quality gate
-> review/approve
-> publish
-> compile pack

## 8.6 Concrete Runtime Contracts To Freeze

Identity/auth contract:
- `accounts`
- `student_profiles`
- `parent_profiles`
- `parent_student_links`
- `admin_profiles`
- entitlement tier on account
- `first_run`
- failed-attempt lockout
- `argon2id` PIN hashing

Curriculum/content-pack contract:
- `curriculum_versions`
- `subjects`
- `topics`
- `academic_nodes`
- `node_edges`
- `misconception_patterns`
- `learning_objectives`
- signed manifests
- checksum verification
- transactional pack install and rollback

Question/evidence contract:
- `question_families`
- `questions`
- `question_options`
- `question_skill_links`
- `student_topic_states`
- `student_error_profiles`
- `student_question_attempts`

Diagnostic/coach contract:
- 5-phase diagnostic battery
- per-topic outputs for mastery, fluency, precision, pressure, flexibility, and stability
- `coach_plans`
- `coach_plan_days`
- `coach_missions`
- `coach_topic_profiles`
- `coach_session_evidence`
- `coach_blockers`
- strict coach-state resolution order

Memory/repair contract:
- `memory_states`
- `memory_evidence_events`
- `recheck_schedules`
- `interference_edges`
- `gap_repair_plans`
- `gap_repair_plan_items`
- `solidification_sessions`

Parent reporting contract:
- backend-generated plain-language summaries
- child readiness/risk/trend read models
- milestone and weekly memo outputs

## 9. APIs / Command Surface

These are backend operations; frontend should stay thin.

Command boundary rules:
- expose runtime APIs through Tauri DTO-only commands
- never return raw domain entities directly to frontend
- every major mutation should emit an append-only domain event with `event_id`, `event_type`, `aggregate_id`, `occurred_at`, `payload`, and `trace_id`

## 9.1 Runtime commands

- `create_local_account`
- `authenticate_with_pin`
- `switch_role_context`
- `install_content_pack`
- `resolve_next_coach_action`
- `start_session`
- `submit_attempt`
- `pause_session`
- `resume_session`
- `complete_session`
- `launch_diagnostic`
- `generate_repair_plan`
- `mark_library_item_state`
- `start_glossary_audio_program`
- `record_glossary_event`
- `generate_daily_plan`
- `generate_parent_digest`
- `export_local_backup`
- `restore_local_backup`

## 9.2 Runtime queries

- `get_home_read_model`
- `get_topic_state`
- `get_skill_state`
- `get_diagnostic_report`
- `get_readiness_report`
- `get_library_shelves`
- `search_glossary`
- `get_question_family_profile`
- `get_mission_stack`
- `get_parent_overview`

## 9.3 Foundry commands

- `upload_source_document`
- `start_parse_job`
- `review_parsed_curriculum`
- `publish_curriculum_version`
- `start_artifact_build`
- `review_artifact`
- `approve_artifact_for_publish`
- `publish_pack_version`
- `rollback_pack_version`
- `run_quality_evaluation`

## 9.4 Foundry queries

- `get_source_pipeline_status`
- `get_curriculum_diff`
- `get_artifact_comparison`
- `get_topic_health`
- `get_publish_history`
- `get_pack_manifest`

## 10. Background Jobs and Queues

## 10.1 Runtime jobs

- learner-state recompute
- memory scan
- review queue generation
- readiness recompute
- plan regeneration
- search index update
- glossary audio pre-generation
- pack migration
- stale read-model rebuild

Implementation rule:
- these run as in-process `tokio` jobs, not an external broker-backed queue, unless the foundry later grows large enough to justify separate workers

## 10.2 Foundry jobs

- OCR/layout extraction
- structural curriculum parse
- source segmentation
- evidence extraction
- artifact build
- comparison job
- quality scoring
- review queue population
- pack compile/sign
- regeneration on curriculum change

## 10.3 Queue guarantees

`Recommendation`
- idempotent handlers
- retry with bounded exponential backoff
- dead-letter queue
- event correlation IDs
- input version pinning

## 11. Auth, Roles, Permissions, Security

## 11.1 Runtime roles

`Explicit corpus`
- student
- parent
- admin

`Strong inference`
- super-admin is central-only
- teacher/coach is optional later

## 11.2 Permission model

### Student
- own sessions
- own library
- own plans
- own diagnostics

### Parent
- view child summaries, readiness, risks, interventions, plans
- no raw internal diagnosis machinery by default
- no content-governance powers

### Local Admin
- device/user management
- pack install/update
- local exports/restores

### Super Admin
- central content operations only
- curriculum publish, artifact review, trust/policy administration

## 11.3 Security rules

- encrypt sensitive local snapshots and exports
- hash PINs
- signed pack manifests
- immutable publish audit trail
- provenance mandatory for live content
- explicit consent/segregation for student uploads reused beyond personal vault

## 12. Observability, Audit, and Governance

## 12.1 Must-have audit streams

- learner evidence history
- decision trace per major coach action
- readiness claim trace
- intervention lifecycle trace
- content lineage/provenance
- publish/review history

## 12.2 Explainability requirements

For each major output, store:
- what evidence was used
- what hypotheses/weights/gates fired
- what was rejected
- final verdict/action

## 12.3 Governance engine

`Recommendation`
- one shared policy module enforcing:
  - no mastery without proof
  - no readiness without threshold evidence
  - no new learning when protection gate blocks it
  - no publish without provenance/review criteria

## 13. Search and Retrieval Strategy

## 13.1 Runtime search

- glossary/library search via FTS + typed filters + ranking
- question retrieval via metadata filters + family links + learner-need boosts
- pack-local only during offline runtime

## 13.2 Foundry retrieval

- curriculum-aware source/artifact retrieval
- internal-first retrieval for build pipelines
- controlled external retrieval only in foundry, never as runtime truth

## 13.3 Ranking signals

- exact match
- alias match
- semantic signal
- topic context
- learner weakness relevance
- exam relevance
- recency/frequency

## 14. Codebase / Repository Structure

`Recommendation`

```text
apps/
  runtime-backend/
  foundry-backend/

crates/
  domain-curriculum/
  domain-knowledge/
  domain-question-intelligence/
  domain-content-artifacts/
  domain-learner-truth/
  domain-diagnostics/
  domain-instruction/
  domain-memory/
  domain-planning/
  domain-session-runtime/
  domain-readiness/
  domain-library/
  domain-parent-admin-runtime/
  foundry-source-intake/
  foundry-curriculum-ingestion/
  foundry-artifact-build/
  foundry-quality-governance/
  foundry-packaging-delivery/
  platform-storage/
  platform-events/
  platform-jobs/
  platform-search/
  platform-audit/
  platform-policy/
  platform-evaluation/
  shared-contracts/
  shared-types/

migrations/
  runtime/
  foundry/

packs/
  schemas/
  compiler/
  fixtures/

tests/
  unit/
  integration/
  replay/
  contracts/
  performance/
```

## 15. Implementation Phases

## Phase 0: Freeze Core Truth

### Goals
- freeze canonical academic units
- freeze role model
- freeze event taxonomy
- freeze runtime/foundry boundary

### Deliverables
- domain glossary of canonical terms
- event naming spec
- source-of-truth ownership map
- threshold registry for mastery, readiness, memory, and planner gates
- frozen appendix for canonical entities, state machines, formulas, and command names
- pack schema v1
- migration skeletons

### Dependencies
- none

### Risks
- drifting names will corrupt later schema design

### Exit criteria
- one canonical model document approved
- no major open ambiguity on `CurriculumNode` vs `ConceptAtom` vs `SkillAtom`

## Phase 1: Runtime Foundation

### Goals
- establish local backend skeleton
- create storage, auth, event ledger, pack install, and read-model basics

### Deliverables
- SQLite schema
- local auth/PIN
- append-only event tables
- pack installer and manifest validator
- account/role switching
- backup/restore v1

### Dependencies
- Phase 0

### Risks
- getting pack/version semantics wrong early

### Exit criteria
- one content pack installs and can be queried locally
- one learner account can complete a trivial session and generate persisted events

## Phase 2: Content and Question Intelligence Core

### Goals
- make curriculum, knowledge, and question structures real

### Deliverables
- curriculum graph tables
- knowledge entry/glossary/formula schema
- question intelligence schema
- question family and misconception mappings
- library item typing
- search index foundations

### Dependencies
- Phase 1

### Risks
- over-modeling before one subject works end-to-end

### Exit criteria
- one subject is fully installable/queryable by curriculum node, concept, skill, family, and glossary entry

## Phase 3: Learner Truth Engine

### Goals
- create the shared learner state substrate

### Deliverables
- mastery state reducers
- memory state reducers
- risk/confidence/readiness state slices
- learner snapshot materialization
- event-to-state recompute pipeline

### Dependencies
- Phase 2

### Risks
- weak evidence mapping will poison all downstream decisions

### Exit criteria
- question attempts update learner state deterministically and replayably

## Phase 4: Coach Core and Session Runtime

### Goals
- implement actual coaching behavior

### Deliverables
- coach lifecycle state machine
- next-action resolver
- session runtime v1
- counted-time truth
- wrong-answer intelligence v1
- directive/read-model generation

### Dependencies
- Phase 3

### Risks
- page-local logic leaking into frontend

### Exit criteria
- backend can decide what a learner should do next and explain why

## Phase 5: Diagnostics and Repair

### Goals
- make the system causally diagnostic, not merely reactive

### Deliverables
- multi-session diagnostic engine
- one-session DNA diagnostic v1
- problem cards
- hypothesis/probe pipeline
- repair plan generation

### Dependencies
- Phase 4

### Risks
- item-bank quality may lag engine ambition

### Exit criteria
- backend can generate a defensible problem-cause-fix report for one supported subject

## Phase 6: Memory, Planning, and Readiness

### Goals
- add long-horizon intelligence

### Deliverables
- decay/review engine
- time orchestration
- live session awareness
- daily/weekly planning
- readiness contracts and proof gates
- parent digest projection v1

### Dependencies
- Phase 5

### Risks
- threshold tuning and over-alerting

### Exit criteria
- backend can produce daily missions, review queues, and readiness outputs from real evidence

## Phase 7: Library / Glossary / Transfer Layer

### Goals
- activate the knowledge system beyond question practice

### Deliverables
- typed library shelves
- glossary lab backend
- knowledge bundles
- question-to-glossary links
- contrast/traps engine
- glossary audio queue/programs v1

### Dependencies
- Phases 3 through 6

### Risks
- feature sprawl

### Exit criteria
- learner can move from a mistake to a linked glossary recovery flow and back into question practice

## Phase 8: Foundry / Content OS

### Goals
- make curriculum/content operations safe and scalable

### Deliverables
- source intake pipeline
- OCR/layout parse
- curriculum ingestion/review/publish
- artifact build/compare/quality gate
- pack compiler/signing

### Dependencies
- Phase 0 and shared contracts

### Risks
- OCR noise, lineage mistakes, publish-policy drift

### Exit criteria
- one admin can ingest sources, review artifacts, publish a pack, and runtime can install it safely

## Phase 9: Hardening and Evaluation

### Goals
- make the platform trustworthy under real use

### Deliverables
- replay harness
- shadow decision evaluation
- benchmark suites
- pack migration tests
- backup/restore hardening
- audit viewers
- calibration tooling

### Dependencies
- all earlier phases

### Risks
- accuracy drift and invisible decision regressions

### Exit criteria
- every critical engine has replay coverage and release gates

## 16. Testing Strategy

## 16.1 Unit tests

- state reducers
- scoring formulas
- policy gates
- ranking/sequencing
- formula speech rendering
- planner conflict precedence

## 16.2 Integration tests

- attempt submission -> learner truth recompute
- diagnostic session -> problem card generation
- memory event -> review queue update
- upload bundle -> OCR/align -> insight extraction
- publish pack -> install pack -> query content

## 16.3 Replay tests

- stored event streams reconstruct same learner state
- decision traces stay stable across refactors unless intentionally changed

## 16.4 Contract tests

- pack schema
- runtime/foundry message contracts
- directive/read-model schemas
- glossary/audio program schemas

## 16.5 Performance tests

- local recompute latency
- SQLite contention under bursty attempts
- pack install/migration time
- OCR queue throughput

## 17. Risks To Manage Early

1. `Canonical unit drift`
- topic vs node vs concept atom vs skill atom

2. `Scope explosion`
- many files describe platform-complete futures

3. `Weak evidence attribution`
- if question-to-skill mapping is weak, all learner truth degrades

4. `Publish-trust failure`
- raw OCR/source noise leaking into live content

5. `Frontend logic leakage`
- backend must own academic decisioning

6. `Threshold sprawl`
- readiness/memory/diagnosis/planning all need governed calibration

7. `Migration complexity`
- packs, schemas, and curriculum versions all evolve

## 18. What Not To Build First

- multiplayer game backends
- live audience features
- full concierge service automation
- broad cloud sync for all learner state
- free-form generation as source of truth
- every glossary/audio innovation before core learner truth is stable
- full RL/policy optimization before deterministic baselines exist

## 19. Immediate Build Order

1. Freeze canonical entities, enums, event names, and ownership map
2. Create runtime database schema and migration system
3. Implement local auth/PIN, accounts, and role switching
4. Implement pack manifest, signature, install, and version tracking
5. Build curriculum graph and one subject pack compiler
6. Build question intelligence metadata and family/misconception links
7. Build learner evidence ledger and state recompute core
8. Build coach lifecycle state machine and next-action resolver
9. Build session runtime and attempt capture pipeline
10. Build wrong-answer intelligence v1
11. Build diagnostic v1 and problem-card generation
12. Build memory/decay engine and review scheduler
13. Build time orchestration and daily plan generation
14. Build readiness gates and readiness report v1
15. Build parent digest/local admin read models
16. Build typed library backend
17. Build glossary/knowledge graph and search v1
18. Build traps/contrast backend
19. Build glossary transfer/recovery flows
20. Build foundry source intake and OCR pipeline
21. Build curriculum ingestion/review/publish workflow
22. Build artifact build/compare/quality gate
23. Build pack compile/sign/release flow
24. Add replay/evaluation/calibration tooling
25. Harden migrations, backups, audits, and release discipline

## 20. Final Recommendation

The correct implementation path is:
- `not` a microservice rewrite
- `not` a frontend-led tutor
- `not` a generic AI assistant with loose state

It should be:
- a local modular runtime with one learner truth
- a central content operating system with strict provenance and publishing
- a deterministic, evented coaching core
- a shared curriculum/question/knowledge substrate
- an audited readiness-and-proof model

That is the backend shape that best matches the full corpus.
