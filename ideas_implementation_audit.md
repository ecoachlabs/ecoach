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
| idea1.txt | read_complete | implemented_gap, partial | Added durable session runtime, append-only runtime events, mock blueprint compilation, quota/coverage logic, mock session startup, a persisted local Question Reactor backbone with lineage/transform logs/family health, runtime reactor top-up for thin session pools, and anti-repeat/question-graph edges with duplicate checks; deeper calibration and orchestration work still remains |
| idea2.txt | read_complete | implemented_gap, partial | Added first skill-level learner truth slice plus Journey route/station persistence and advancement; fuller Journey orchestration is still missing |
| idea3.txt | read_complete | implemented_gap, partial | Added parent insight generation/persistence plus strategy-aware household snapshots, household-level attention summaries, and deduped household actions; deeper intervention execution flows still remain |
| idea4.txt | read_complete | implemented_gap, partial | Added Beat Yesterday daily target generation, summary scoring, recovery-aware mode selection, and climb trend projection; full teacher/risk layer still missing |
| idea5.txt | read_complete | implemented_gap, partial | Added elite session scoring, debrief recommendations, rolling profile updates, and topic domination tracking; session generation and richer authoring still missing |
| idea6.txt | read_complete | implemented_gap, partial | Added skill-level knowledge gap substrate plus richer repair-plan detail, actionable repair item guidance, candidate question routing, misconception/resource hints, and a fix for the broken academic-node ordering query; broader gap-engine orchestration still remains |
| idea7.txt | read_complete | implemented_gap, partial | Memory mode audit complete; live memory updates, due recheck listing, overdue decay scanning, and a real memory return-loop planner now exist, but deeper long-horizon orchestration still remains |
| idea8.txt | read_complete | implemented_gap, partial | Same backend implications as idea7; live memory updates, due recheck listing, overdue decay scanning, and a real memory return-loop planner now exist, but deeper long-horizon orchestration still remains |
| idea9.txt | read_complete | implemented_gap, partial | Linked sample questions to skill nodes, added skill truth updates, implemented a persisted Question Factory/Reactor flow for family selection, variant generation, lineage tracking, family health, runtime session/mock top-up, anti-repeat candidate scoring, and durable question-graph edges/related-question reads; richer family-by-family generation specs and scoring layers still remain |
| idea10.txt | read_complete | implemented_gap, partial | Added real MindStack and Tug-of-War runtime state reads, command-boundary coverage, and app-facing Tauri registration on top of the game session backend; broader game expansion still remains |
| idea11.txt | read_complete | implemented_gap, partial | Added persisted wrong-answer diagnosis records and retrieval; deeper diagnosis stack still missing |
| idea12.txt | read_complete | implemented_gap, partial | Added parent dashboard and memo generation pipeline plus premium strategy-aware household/admin reporting summaries and aligned follow-up actions; broader intervention execution layers still remain |
| idea13.txt | read_complete | implemented_gap, partial | Added past-paper question linking, family recurrence/coappearance analytics recompute, and high-frequency family read models; richer inverse appearance intelligence still missing |
| idea14.txt | read_complete | implemented_gap, partial | Added custom-test composer plus mock blueprint compilation and mock session startup on top of local session runtime, then wired reactor-backed top-up so custom tests and mock blueprints can exceed the authored pool; richer adaptive interpretation still missing |
| idea15.txt | read_complete | implemented_gap, partial | Added live memory updates, due recheck listing, overdue decay scanning, and memory return-loop session planning; deeper adaptation across longer learning arcs still remains |
| idea16.txt | read_complete | implemented_gap, partial | Added generated library shelves, saved-question intelligence, weak-topic revision packs, topic relationship hints, and teach-action plans derived from learner state plus content graph links; richer personalization and deeper relationship tooling still remain |
| idea17.txt | read_complete | implemented_gap, partial | Added persisted question-glossary linkage and retrieval, plus derived glossary audio programs for topic and question repair flows; richer audio personalization still remains |
| idea18.txt | read_complete | implemented_gap, partial | Added persisted multi-phase diagnostic battery assembly plus phase submission, attempt-level phase item IDs, topic analytics, root-cause hypotheses, adaptive phase retargeting, confidence-distortion detection, and a full completion sync that pushes diagnostic outputs into learner truth, coach profiles, blockers, plan rewrites, and Journey refresh; richer longitudinal interpretation still remains |
| idea19.txt | read_complete | implemented_gap, partial | Added persisted mission/debrief memory and review handoff; richer experience composition still missing |
| idea20.txt | read_complete | implemented_gap, partial | Added content-readiness gating, canonical next-action resolution, subject readiness engine, plan rewrite support, Journey route/runtime command exposure, today-mission generation exposure, a fix for persisted journey station evidence, and diagnostic-driven plan/Journey resync; deeper coach memory and mission adaptation still missing |
| idea21.txt | read_complete | implemented_gap, partial | Added resource-readiness scoring for topics and subjects across atoms, questions, misconceptions, and knowledge assets |
| idea22.txt | read_complete | implemented_gap, partial | Added normalized question-intelligence registry, taxonomy links, and question intelligence query surfaces |
| idea23.txt | read_complete | implemented_gap, partial | Added coach topic-case reasoning service with case synthesis, hypotheses, and intervention recommendations |
| idea24.txt | read_complete | implemented_gap, partial | Added offline hypothesis-based coach diagnosis through topic-case certainty, proof gaps, and open questions |
| idea25.txt | read_complete | implemented_gap, partial | Added shared engine contract and registry layer in substrate for runtime engine coordination |
| idea26.txt | read_complete | implemented_gap, partial | Added staged curriculum source uploads, parse candidates, review tasks, source-finalization workflow, review resolution, app-facing foundry commands, and source-aware curriculum builders that now materialize reviewed concept/objective/formula candidates into topic nodes and objectives instead of generic-only scaffolds; richer admin ingestion tooling is still missing |
| idea27.txt | read_complete | implemented_gap, partial | Added content acquisition jobs, evidence candidates, acquisition job reports, acquisition evidence consumption inside topic package health scoring, queueable source/topic follow-up foundry jobs, and source-aware acquisition seeding that now turns reviewed parse candidates into approved evidence seeds for downstream builders; richer acquisition automation still missing |
| idea28.txt | read_complete | implemented_gap, partial | Added shared learner/evidence fabric contracts and cross-engine learner evidence read model; broader fabric consumers still missing |
| idea29.txt | read_complete | implemented_gap, partial | Added content-type strategy registry for node-level pedagogy, drill, failure-mode, and mastery rules |
| idea30.txt | read_complete | implemented_gap, partial | Added availability/capacity substrate plus free-now recommendation and daily replanning orchestration |
| idea31.txt | read_complete | implemented_gap, partial | Added publish jobs, quality reports, quality-gated publish transitions, source-to-publish orchestration, topic package gating, publish staging from the foundry coordinator, durable foundry jobs for publish/build follow-up, executable foundry automation for staging/quality/publish activation, and source-aware note/formula/worked-example/question/contrast builders that now consume reviewed source candidates and approved evidence instead of only generic placeholders; broader content-build automation still missing |
| idea32.txt | read_complete | implemented_gap, partial | Added live memory/recheck updates plus due recheck listing and overdue decay scanning; deeper memory recompute/orchestration is still missing |
| idea33.txt | read_complete | implemented_gap, partial | Added contrast-profile ingestion, trap round/session persistence, Difference Drill, Similarity Trap, Know the Difference, Which Is Which, Unmask runtime flows, replay review, and learner confusion state; deeper remediation routing is still missing |
| idea34.txt | read_complete | implemented_gap, partial | Added diagnostic phase submission, phase-aware topic analytics, durable root-cause hypotheses, adaptive next-phase retargeting, and confidence-weighted distortion diagnosis; fuller adaptive orchestration is still missing |
| idea35.txt | read_complete | implemented_gap, partial | Added a DTO-only runtime command boundary crate with shared AppState, unified CommandError, and a broad Tauri-registered app surface covering identity, coach, Journey, curriculum, sessions, attempts, foundry, diagnostics with battery/phase/submit/advance/complete-sync flow, questions, past-paper pressure, elite blueprints, games, traps, library teach-action and relationship hints, glossary audio, memory return-loop reads, detailed repair plans, readiness, premium, reporting, and mock-centre; this pass also fixed command-layer error coercion across the newer routes so the workspace now compiles cleanly again, though feature-depth work still remains |
| idea36.txt | read_complete | implemented_gap, partial | Extended upload intelligence with sidecar OCR/text recovery for scans and PDFs, page-aware recovery summaries, richer document-role classification (`report_card`, `corrected_script`, `teacher_handout`), mined dates/topics/question blocks/score signals/remarks/glossary terms, coach-action and student-model update signals, confirmation/alignment states, and stronger aggregate bundle reporting; true native OCR execution and deeper coach-goal consumption still remain outside the intake crate |
| idea37.txt | read_complete | implemented_gap, partial | Added learner-truth snapshot read model with topic, skill, memory, and diagnosis projections; broader cross-engine fabric still missing |
| idea38.txt | read_complete | implemented_gap, partial | Added source staging, acquisition jobs, learner evidence fabrics, publish/trust job substrate, topic package snapshots, subject foundry dashboards, a coordinator that links source review, topic health, and publish readiness, durable foundry job orchestration, executable foundry queue automation with run/run-next behavior, and source-aware Foundry builders that now carry reviewed curriculum/support signals through curriculum, knowledge, question, and contrast artifact creation; the broader Foundry/Content OS is still not fully complete |

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
  - sidecar OCR/text recovery for scan-style uploads
  - page-aware recovery summaries and confirmation states
  - mined report-card/corrected-script findings such as topics, scores, remarks, question blocks, and coach actions
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
- Added `migrations/runtime/033_mock_and_journey_runtime.sql` and extended `crates/ecoach-storage/src/migrations.rs` with:
  - mock blueprint storage
  - Journey route persistence
  - Journey station persistence
- Added `crates/ecoach-coach-brain/src/readiness_engine.rs`, `crates/ecoach-coach-brain/src/journey.rs`, and updated `crates/ecoach-coach-brain/src/plan_engine.rs` and `crates/ecoach-coach-brain/src/lib.rs` with:
  - subject readiness snapshots
  - recommended mock blueprint modes
  - plan rewrite support
  - Journey route/station build and advancement
- Extended `crates/ecoach-sessions/src/models.rs`, `crates/ecoach-sessions/src/lib.rs`, `crates/ecoach-sessions/src/service.rs`, `crates/ecoach-sessions/Cargo.toml`, and `crates/ecoach-questions/src/service.rs` with:
  - mock blueprint DTOs
  - mock blueprint compilation
  - topic quota and coverage logic
  - mock session startup from compiled blueprint
  - scoped question listing for blueprint compilation
- Added `migrations/runtime/034_diagnostic_analytics.sql` and extended `crates/ecoach-storage/src/migrations.rs` with:
  - diagnostic topic analytics
  - diagnostic root-cause hypotheses
- Extended `crates/ecoach-diagnostics/src/models.rs`, `crates/ecoach-diagnostics/src/lib.rs`, and `crates/ecoach-diagnostics/src/engine.rs` with:
  - diagnostic phase attempt submission
  - phase-aware topic analytics
  - durable root-cause hypotheses
  - persisted diagnostic analytics reads
- Added `migrations/runtime/035_traps_runtime.sql` and extended `crates/ecoach-storage/src/migrations.rs` with:
  - contrast-pair runtime columns
  - traps round persistence
  - learner contrast state persistence
- Extended `crates/ecoach-content/src/pack_service.rs` and the sample pack under `packs/math-bece-sample/content/` with:
  - optional contrast-profile ingestion
  - contrast pair insertion
  - contrast evidence atom insertion
  - sample trap-ready contrast content
- Extended `crates/ecoach-games/src/models.rs`, `crates/ecoach-games/src/lib.rs`, `crates/ecoach-games/src/service.rs`, and `crates/ecoach-games/Cargo.toml` with:
  - Traps mode/session DTOs
  - contrast pair listing for the Traps hub
  - Difference Drill runtime
  - Similarity Trap runtime
  - Know the Difference runtime
  - Which Is Which runtime
  - Unmask clue-reveal runtime
  - round review and confusion-reason persistence
  - learner contrast/confusion analytics state
- Added `crates/ecoach-commands/` with:
  - `AppState` runtime context
  - unified `CommandError`
  - DTO-only identity commands
  - DTO-only content pack commands
  - DTO-only session commands
  - DTO-only traps commands
  - a command-boundary integration test over in-memory runtime state

Verification:
- `cargo check`
- `cargo test -p ecoach-content --lib`
- `cargo test -p ecoach-games --lib`
- `cargo test -p ecoach-commands --lib`

Verification caveat:
- Earlier slices hit Windows Application Control (`os error 4551`) for some newly built test executables, but the latest content and games crates were runtime-verified successfully in this environment.

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
