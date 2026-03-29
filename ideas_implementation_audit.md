# Ideas Backend Implementation Audit

This ledger tracks the document-by-document audit over `C:\Users\victo\OneDrive\ecoach\ideas`.

Legend:
- `not_started`: document not yet audited in this pass
- `read_complete`: document fully read in this pass
- `implemented_gap`: at least one concrete backend gap from the document was implemented
- `partial`: document exposes additional backend work still outstanding

## Document Status

| Document | Audit Status | Implementation Status | Notes |
| --- | --- | --- | --- |
| idea1.txt | read_complete | implemented_gap, partial | Added durable session runtime state and append-only runtime events; larger Mock Centre / Question Reactor scope still outstanding |
| idea2.txt | read_complete | implemented_gap, partial | Added first skill-level learner truth slice; full Journey orchestration still missing |
| idea3.txt | read_complete | implemented_gap, partial | Added parent insight generation/persistence; deeper household/intervention flows still missing |
| idea4.txt | read_complete | implemented_gap, partial | Added Beat Yesterday daily target generation, summary scoring, recovery-aware mode selection, and climb trend projection; full teacher/risk layer still missing |
| idea5.txt | read_complete | implemented_gap, partial | Added elite session scoring, debrief recommendations, rolling profile updates, and topic domination tracking; session generation and richer authoring still missing |
| idea6.txt | read_complete | implemented_gap, partial | Added first skill-level knowledge gap substrate; full gap engine still missing |
| idea7.txt | read_complete | implemented_gap, partial | Memory mode audit complete; live memory updates, due recheck listing, and overdue decay scanning now exist, but the full memory service is still missing |
| idea8.txt | read_complete | implemented_gap, partial | Same backend implications as idea7; live memory updates, due recheck listing, and overdue decay scanning now exist, but the full memory service is still missing |
| idea9.txt | read_complete | implemented_gap, partial | Linked sample questions to skill nodes and added skill truth updates; full Question Factory still missing |
| idea10.txt | read_complete | partial | Games audit complete; MindStack runtime still missing |
| idea11.txt | read_complete | implemented_gap, partial | Added persisted wrong-answer diagnosis records and retrieval; deeper diagnosis stack still missing |
| idea12.txt | read_complete | implemented_gap, partial | Added parent dashboard and memo generation pipeline; premium strategy/readiness layers still missing |
| idea13.txt | read_complete | implemented_gap, partial | Added past-paper question linking, family recurrence/coappearance analytics recompute, and high-frequency family read models; richer inverse appearance intelligence still missing |
| idea14.txt | read_complete | implemented_gap, partial | Added custom-test composer on top of local session runtime; richer adaptive interpretation still missing |
| idea15.txt | read_complete | implemented_gap, partial | Added live memory updates, due recheck listing, and overdue decay scanning; fuller return-loop orchestration still missing |
| idea16.txt | read_complete | implemented_gap, partial | Added generated library shelves, saved-question intelligence, and weak-topic revision packs; richer relationship graph and teach actions still missing |
| idea17.txt | read_complete | implemented_gap, partial | Added persisted question-glossary linkage and retrieval; audio program still missing |
| idea18.txt | read_complete | implemented_gap, partial | Added persisted multi-phase diagnostic battery assembly; deeper scoring and root-cause analytics still missing |
| idea19.txt | read_complete | implemented_gap, partial | Added persisted mission/debrief memory and review handoff; richer experience composition still missing |
| idea20.txt | read_complete | implemented_gap, partial | Added content-readiness gating and canonical next-action resolution; deeper coach memory and mission adaptation still missing |
| idea21.txt | read_complete | implemented_gap, partial | Added resource-readiness scoring for topics and subjects across atoms, questions, misconceptions, and knowledge assets |
| idea22.txt | read_complete | implemented_gap, partial | Added normalized question-intelligence registry, taxonomy links, and question intelligence query surfaces |
| idea23.txt | read_complete | implemented_gap, partial | Added coach topic-case reasoning service with case synthesis, hypotheses, and intervention recommendations |
| idea24.txt | read_complete | implemented_gap, partial | Added offline hypothesis-based coach diagnosis through topic-case certainty, proof gaps, and open questions |
| idea25.txt | read_complete | implemented_gap, partial | Added shared engine contract and registry layer in substrate for runtime engine coordination |
| idea26.txt | read_complete | implemented_gap, partial | Added staged curriculum source uploads, parse candidates, review tasks, and source reports; full publish pipeline still missing |
| idea27.txt | read_complete | implemented_gap, partial | Added content acquisition jobs, evidence candidates, and acquisition job reports; richer acquisition automation still missing |
| idea28.txt | read_complete | implemented_gap, partial | Added shared learner/evidence fabric contracts and cross-engine learner evidence read model; broader fabric consumers still missing |
| idea29.txt | read_complete | implemented_gap, partial | Added content-type strategy registry for node-level pedagogy, drill, failure-mode, and mastery rules |
| idea30.txt | read_complete | implemented_gap, partial | Added availability/capacity substrate plus free-now recommendation and daily replanning orchestration |
| idea31.txt | read_complete | implemented_gap, partial | Added publish jobs, quality reports, and quality-gated publish transitions; full end-to-end pack publish automation still missing |
| idea32.txt | read_complete | implemented_gap, partial | Added live memory/recheck updates plus due recheck listing and overdue decay scanning; deeper memory recompute/orchestration is still missing |
| idea33.txt | read_complete | partial | Traps/contrast audit complete; mode runtime still missing |
| idea34.txt | read_complete | partial | Adaptive diagnostic audit complete; phased orchestration still missing |
| idea35.txt | read_complete | partial | Build-discipline audit complete; command boundary hardening still missing |
| idea36.txt | read_complete | implemented_gap, partial | Added bundle reconstruction/classification and extracted insight reporting; deeper OCR/layout recovery still missing |
| idea37.txt | read_complete | implemented_gap, partial | Added learner-truth snapshot read model with topic, skill, memory, and diagnosis projections; broader cross-engine fabric still missing |
| idea38.txt | read_complete | implemented_gap, partial | Added source staging, acquisition jobs, learner evidence fabrics, and publish/trust job substrate; full content OS orchestration still missing |

## idea1.txt

Read status:
- Fully read and compared against the current backend.

Backend gaps identified from the document:
- Durable mock session runtime was incomplete.
- The backend did not persist selected session items for crash-safe resume.
- There was no append-only runtime event log for session lifecycle actions.
- Mock Centre state transitions such as pause, resume, answer recording, and flagging were not durably represented in local storage.

Implemented from this document in this pass:
- Added runtime event storage migration in `migrations/runtime/008_runtime_events.sql`.
- Added durable session item storage and session runtime migration in `migrations/runtime/009_session_runtime.sql`.
- Extended the migration runner in `crates/ecoach-storage/src/migrations.rs`.
- Extended session domain models in `crates/ecoach-sessions/src/models.rs`.
- Extended the session service in `crates/ecoach-sessions/src/service.rs` with:
  - persisted session items
  - session snapshots
  - pause/resume support
  - durable answer recording
  - item flagging
  - runtime event logging
  - summary computation from session runtime state
- Wired additional append-only runtime events into:
  - `crates/ecoach-content/src/pack_service.rs`
  - `crates/ecoach-student-model/src/service.rs`
  - `crates/ecoach-coach-brain/src/plan_engine.rs`
- Expanded the sample pack MCQs with real options and misconception distractors in `packs/math-bece-sample/questions/questions.json`.

Verification:
- `cargo test -p ecoach-sessions`
- `cargo test`

## idea31.txt through idea38.txt

Read status:
- Fully read in parallel audit and compared against the current backend.

Most important gaps identified across this range:
- live memory/decay/recheck engine (`idea32.txt`)
- traps/contrast runtime (`idea33.txt`)
- phased adaptive diagnostic engine (`idea34.txt`)
- command boundary and slice-discipline hardening (`idea35.txt`)
- upload reconstruction pipeline (`idea36.txt`)
- unified learner-truth/read-model layer (`idea37.txt`)
- full content OS orchestration above the new source/publish substrate (`idea38.txt`)

Implemented from this range in this pass:
- Extended `crates/ecoach-student-model/src/service.rs` so answer processing now:
  - writes `memory_evidence_events`
  - upserts `memory_states`
  - schedules `recheck_schedules`
  - emits `memory.updated` runtime events
- Extended `crates/ecoach-student-model/src/models.rs`, `crates/ecoach-student-model/src/lib.rs`, and `crates/ecoach-student-model/src/service.rs` with:
  - `MemoryDecayUpdate`
  - `MemoryRecheckItem`
  - due recheck listing
  - overdue memory decay scanning
  - missed recheck marking for stale pending schedules
  - `memory.decay_scanned` runtime events
- Extended the integration flow in `crates/ecoach-sessions/src/service.rs` to assert the new memory state and recheck records are created.
- Extended `crates/ecoach-student-model/src/models.rs`, `crates/ecoach-student-model/src/lib.rs`, and `crates/ecoach-student-model/src/service.rs` with a unified `LearnerTruthSnapshot` read model that projects:
  - top topic truth
  - top skill truth
  - due memory state
  - recent diagnosis summaries
  - overall readiness band and pending review counts
- Extended `crates/ecoach-intake/src/models.rs`, `crates/ecoach-intake/src/service.rs`, and `crates/ecoach-intake/src/lib.rs` with:
  - bundle file classification
  - bundle reconstruction and aggregate reporting
  - extracted bundle/file insights
  - detected subject/year hints and question-like file detection
- Added `migrations/runtime/032_content_publish_pipeline.sql` and extended `crates/ecoach-storage/src/migrations.rs` with:
  - content publish jobs
  - quality report storage
  - publish status transitions
- Added `crates/ecoach-content/src/publish_pipeline.rs` and exported it from `crates/ecoach-content/src/lib.rs` with:
  - `ContentPublishJob`
  - `ContentQualityReport`
  - `ContentPublishJobReport`
  - publish job creation
  - quality gate reporting
  - readiness checks
  - publish completion transitions

Verification:
- `cargo check`

Verification caveat:
- Rust test binaries began failing to execute under Windows Application Control (`os error 4551`) after this slice, so this memory implementation is compile-verified but not newly runtime-verified in the current environment.

## idea21.txt through idea30.txt

Read status:
- Fully read in parallel audit and compared against the current backend.

Most important gaps identified across this range:
- deeper free-now/day-queue orchestration and schedule debt handling (`idea30.txt`)

Implemented from this range in this pass:
- Added `availability_profiles`, `availability_windows`, and `availability_exceptions` in `migrations/runtime/022_time_orchestration.sql`.
- Extended the migration runner in `crates/ecoach-storage/src/migrations.rs`.
- Extended `crates/ecoach-goals-calendar/src/models.rs` with:
  - `AvailabilityProfile`
  - `AvailabilityWindow`
  - `AvailabilityException`
  - `DailyAvailabilitySummary`
- Extended `crates/ecoach-goals-calendar/src/models.rs` again with:
  - `FreeNowRecommendation`
  - `DailyReplan`
- Exported the new time-orchestration DTOs from `crates/ecoach-goals-calendar/src/lib.rs`.
- Extended `crates/ecoach-goals-calendar/src/service.rs` with:
  - availability profile upsert
  - weekly availability windows replacement
  - one-off exception insertion
  - daily availability computation
  - `is_free_now` evaluation
  - free-now session recommendation
  - remaining-day replanning
- Added `crates/ecoach-coach-brain/src/topic_case.rs` and exported it from `crates/ecoach-coach-brain/src/lib.rs` with:
  - topic-case synthesis over topic truth, memory, blockers, coach evidence, and wrong-answer diagnoses
  - ranked topic hypotheses
  - diagnosis certainty, proof gaps, and open questions
  - recommended intervention modes and next actions
- Added `crates/ecoach-substrate/src/engine_registry.rs` and exported it from `crates/ecoach-substrate/src/lib.rs` with:
  - shared engine contracts for core runtime engines
  - registry lookups by engine key, produced outputs, and consumed inputs
  - a canonical offline-required engine map for learner truth, diagnostics, coach brain, scheduling, sessions, content, library, glossary, reporting, and intake
- Added `migrations/runtime/030_question_intelligence_registry.sql` and extended `crates/ecoach-storage/src/migrations.rs` with:
  - normalized question-intelligence axes
  - taxonomy storage
  - per-question intelligence links
- Extended `crates/ecoach-content/src/pack_service.rs` and `packs/math-bece-sample/questions/intelligence.json` so pack installs now populate normalized intelligence links for:
  - knowledge role
  - cognitive demand
  - solve pattern
  - pedagogic function
  - content grain
  - question family
  - misconception exposure
- Extended `crates/ecoach-questions/src/models.rs`, `crates/ecoach-questions/src/lib.rs`, `crates/ecoach-questions/src/service.rs`, and `crates/ecoach-questions/Cargo.toml` with:
  - question intelligence profile DTOs
  - normalized intelligence link reads
  - question lookup by intelligence axis and concept code
- Added `crates/ecoach-content/src/resource_readiness.rs` and exported it from `crates/ecoach-content/src/lib.rs` with:
  - topic-level resource readiness scoring
  - subject-level rollups
  - missing-resource flags
  - generation-mode capability projection from actual runtime content assets
- Added `crates/ecoach-content/src/content_strategy_registry.rs` and exported it from `crates/ecoach-content/src/lib.rs` with:
  - node-type strategy families
  - preferred drill families
  - canonical failure-mode maps
  - mastery-evidence rules
  - review and time-sensitivity guidance for definition, concept, formula, procedure, comparison, theorem, application, interpretation, diagram, proof, essay, translation, vocabulary, and notation content
- Added `migrations/runtime/031_content_pipeline.sql` and extended `crates/ecoach-storage/src/migrations.rs` with:
  - curriculum source upload storage
  - parse candidate staging
  - review task tracking
  - content acquisition jobs
  - evidence candidate staging
- Extended `crates/ecoach-curriculum/src/models.rs`, `crates/ecoach-curriculum/src/lib.rs`, and `crates/ecoach-curriculum/src/service.rs` with:
  - `CurriculumSourceUpload`
  - `CurriculumParseCandidate`
  - `CurriculumReviewTask`
  - `CurriculumSourceReport`
  - source upload creation
  - parse candidate staging
  - review task creation
  - source-status updates and reporting
- Extended `crates/ecoach-intake/src/models.rs`, `crates/ecoach-intake/src/lib.rs`, and `crates/ecoach-intake/src/service.rs` with:
  - `ContentAcquisitionJob`
  - `AcquisitionEvidenceCandidate`
  - `AcquisitionJobReport`
  - acquisition job creation
  - evidence candidate staging
  - job completion and reporting
- Added `crates/ecoach-substrate/src/fabrics.rs` and exported it from `crates/ecoach-substrate/src/lib.rs` with:
  - `FabricSignal`
  - `FabricEvidenceRecord`
  - `LearnerEvidenceFabric`
- Extended `crates/ecoach-substrate/src/engine_registry.rs` and `crates/ecoach-student-model/src/service.rs` with:
  - learner-evidence-fabric engine contract output
  - cross-engine learner evidence aggregation over topic truth, skill truth, memory truth, diagnoses, mission memory, attempts, memory events, and runtime events
  - `get_learner_evidence_fabric` for canonical fabric reads

Verification:
- `cargo check -p ecoach-goals-calendar`
- `cargo check`

Still outstanding from idea1:
- richer mock recommendation modes
- mock blueprint resolution
- mock compiler quotas/coverage logic
- readiness engine
- plan rewrite engine
- question graph / family lineage / reactor subsystems

## idea11.txt through idea20.txt

Read status:
- Fully read in parallel audit and compared against the current backend.

Most important gaps identified across this range:
- wrong-answer diagnosis pipeline (`idea11.txt`)
- premium parent dashboard/read-model pipeline (`idea12.txt`)
- past-paper family/recurrence analytics builder (`idea13.txt`)
- multi-phase custom testing and result interpretation (`idea14.txt`)
- memory/return-loop engine (`idea15.txt`)
- library relationship projection (`idea16.txt`)
- question-glossary linking and audio program backend (`idea17.txt`)
- multi-session diagnostic battery (`idea18.txt`)
- coach evidence and mission-memory layer (`idea19.txt`)
- full `CoachBrainService` and content-readiness gates (`idea20.txt`)

Implemented from this range in this pass:
- Added `CustomTestStartInput` to `crates/ecoach-sessions/src/models.rs`.
- Exported the new custom-test DTO from `crates/ecoach-sessions/src/lib.rs`.
- Added `start_custom_test` and custom test composition helpers to `crates/ecoach-sessions/src/service.rs`.
- Added a runtime-tested custom test flow that persists a `custom_test` session and emits `custom_test.composed`.
- Added `ParentInsightService` and parent dashboard DTOs in `crates/ecoach-reporting/src/parent.rs`.
- Exported the parent reporting layer from `crates/ecoach-reporting/src/lib.rs`.
- Extended `crates/ecoach-reporting/Cargo.toml` for JSON-backed parent dashboard persistence.
- The new parent reporting layer now:
  - reads linked students
  - derives readiness, risks, trends, and recommendations
  - persists `parent_dashboards`
  - upserts parent-facing `weekly_memos`
  - upserts derived `risk_flags`
- Added `wrong_answer_diagnoses` in `migrations/runtime/023_wrong_answer_diagnoses.sql`.
- Extended the migration runner in `crates/ecoach-storage/src/migrations.rs`.
- Extended `crates/ecoach-student-model/src/models.rs` so answer processing results can include diagnosis summaries and recommended actions.
- Extended `crates/ecoach-student-model/src/service.rs` so wrong answers now:
  - persist structured diagnosis records
  - emit `wrong_answer.diagnosed`
  - return diagnosis summary and recommended action in the answer result
- Extended `crates/ecoach-diagnostics/src/models.rs`, `crates/ecoach-diagnostics/src/engine.rs`, and `crates/ecoach-diagnostics/src/lib.rs` with retrieval support for recent wrong-answer diagnoses.
- Added `question_glossary_links` in `migrations/runtime/024_question_glossary_links.sql`.
- Extended the migration runner in `crates/ecoach-storage/src/migrations.rs`.
- Extended `crates/ecoach-glossary/src/models.rs`, `crates/ecoach-glossary/src/service.rs`, and `crates/ecoach-glossary/src/lib.rs` with:
  - persisted question-to-entry links
  - upsert support for explicit links
  - retrieval of linked knowledge entries for a question
- Extended `crates/ecoach-content/src/pack_service.rs` so pack install now infers and persists question-linked repair knowledge from shared topic and skill/title overlap.
- Added `027_library_intelligence.sql` to extend `library_items` with note/topic/urgency metadata and to add durable `library_shelf_items` plus `revision_pack_items`.
- Extended `crates/ecoach-library/src/models.rs`, `crates/ecoach-library/src/service.rs`, and `crates/ecoach-library/src/lib.rs` with:
  - enriched saved-item state metadata
  - saved-question intelligence cards
  - generated library shelves for due-now, memory, mistakes, weak topics, and saved questions
  - continue-learning card resolution
  - weak-topic revision pack generation and persisted pack items
- Added `025_diagnostic_battery_templates.sql` to extend diagnostic phase metadata with explicit phase code, title, condition profile, and timer fields.
- Extended `crates/ecoach-diagnostics/src/models.rs` and `crates/ecoach-diagnostics/src/lib.rs` with persisted battery, phase-plan, and phase-item DTOs.
- Extended `crates/ecoach-coach-brain/src/state_machine.rs` with:
  - content-readiness evaluation
  - canonical next-action resolution
  - coach action/content-readiness DTOs
  - compile-checked resolver tests
- Extended `crates/ecoach-coach-brain/src/lib.rs` and `crates/ecoach-coach-brain/Cargo.toml` to export and test the new coach-decision layer.
- Extended `crates/ecoach-coach-brain/src/plan_engine.rs` so plan generation now creates a real multi-day schedule and today’s mission attaches to the active plan day with phase-aware activity selection.
- Added `026_coach_mission_memory.sql` for persisted mission/debrief memory and review handoff.
- Extended `crates/ecoach-coach-brain/src/plan_engine.rs` with:
  - mission start support
  - mission completion memory persistence
  - coach evidence/profile sync
  - blocker updates
  - review scheduling and retrieval
- Extended `crates/ecoach-coach-brain/src/state_machine.rs` so pending mission reviews now resolve to `mission_review_required` and a concrete review action.
- Extended `crates/ecoach-past-papers/src/models.rs`, `crates/ecoach-past-papers/src/service.rs`, and `crates/ecoach-past-papers/src/lib.rs` with:
  - past-paper question linking
  - family analytics recomputation
  - high-frequency family listing
  - paper history lookup per family

Verification:
- `cargo test -p ecoach-sessions`
- `cargo test`
- `cargo check`

## idea2.txt through idea10.txt

Read status:
- Fully read in parallel audit and compared against the current backend.

Most important gaps identified across this range:
- Journey route/station persistence and evidence loop (`idea2.txt`)
- parent/household insight services (`idea3.txt`)
- comeback / Beat Yesterday planner (`idea4.txt`)
- elite scoring and debrief pipeline (`idea5.txt`)
- skill-level gap engine (`idea6.txt`)
- real memory mode service/state machine (`idea7.txt` and `idea8.txt`)
- question factory/generation-validation backbone (`idea9.txt`)
- MindStack runtime backend (`idea10.txt`)

Implemented from this range in this pass:
- Added `student_skill_states` in `migrations/runtime/021_skill_truth.sql`.
- Extended the migration runner in `crates/ecoach-storage/src/migrations.rs`.
- Linked the sample pack questions to their canonical skill node in `packs/math-bece-sample/questions/questions.json`.
- Extended `crates/ecoach-student-model/src/service.rs` so answer processing now:
  - upserts `student_skill_states`
  - emits `skill_truth.updated`
  - keeps skill truth alongside topic truth and memory truth
- Extended `crates/ecoach-sessions/src/service.rs` integration assertions to check the new skill-state records are created.
- Added `028_beat_yesterday.sql` and extended `crates/ecoach-goals-calendar/src/models.rs`, `crates/ecoach-goals-calendar/src/service.rs`, and `crates/ecoach-goals-calendar/src/lib.rs` with:
  - Beat Yesterday profiles
  - daily climb target generation
  - daily climb completion summaries
  - momentum and strain scoring
  - recovery-aware mode selection
  - climb dashboard and trend read models
- Added `029_elite_topic_profiles.sql` and extended `crates/ecoach-elite/src/models.rs`, `crates/ecoach-elite/src/service.rs`, and `crates/ecoach-elite/src/lib.rs` with:
  - elite session scoring
  - deterministic debrief narratives and next-session recommendations
  - rolling elite profile updates
  - topic domination tracking

Verification:
- `cargo check`

Verification caveat:
- The Windows Application Control block on Rust test executables remained active during this slice, so the skill-truth implementation is compile-verified but not newly runtime-verified in the current environment.
