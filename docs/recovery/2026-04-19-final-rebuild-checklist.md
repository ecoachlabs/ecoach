# Ecoach Final Rebuild Checklist

Goal: finish the reconstruction with a single source of truth for what must be real, what is already fixed, and which screens still need implementation, cleanup, or verification.

Architecture: keep the app honest by routing every learner-facing surface through the real Tauri command path, real persisted session data, and real derived analytics. Use the recovered Vue and Rust files as design/history evidence, not as blind drop-ins.

Tech Stack: Vue 3, TypeScript, Tauri, Rust, SQLite-backed command/services, student shell and shared question/session components.

---

## Truth Spine

- [x] Practice and homepage arena completion goes through the real session completion pipeline.
- [x] Session completion passes real summary data into debrief instead of replaying completion.
- [x] Question flags persist through the real command path.
- [x] Memory Quick Scan now carries real `topic_id` values end to end.
- [x] Mock flow records presented, skipped, timed-out, and answered question events.
- [x] Diagnostic session records answer, skip, timeout, focus, and input interactions.
- [x] Student shell top bar, sidebar stats, and badges now use real backend values.
- [x] Timed custom tests now enter the live session view as actual timed sessions.
- [x] Mock Hall now hydrates from real remaining time and forces expiry back through the backend.
- [x] Paused and in-progress mocks can be reopened from home/history instead of dead-ending.
- [x] Mock question flags now persist honestly against the current backend behavior.
- [x] Topic Mock setup now sends real selected topic IDs instead of an empty topic list.
- [ ] Every learner-facing score chip still visible in the UI must be audited for placeholder values.
- [ ] History, heatmap, misconception, readiness, and coach surfaces must all be re-checked against live data after every session path.

## Shared Shell And Navigation

- [x] `/student` is the main home dashboard.
- [x] Coach mode is separate and is toggled from the top bar.
- [x] Diagnostic "Go to Coach Hub" actions route to the separate coach mode.
- [x] Finalize real shell stats and remove remaining hardcoded trophies/levels/counters.
- [x] Coach mode now uses the shell's real dark/light state instead of a page-local theme toggle.
- [x] Sidebar active-state now stays anchored on Home while coach mode is active from the top bar.
- [ ] Audit top-bar and sidebar microcopy/icons against recovered presentation direction.
- [ ] Verify route-state persistence for coach activation, split panes, and dark mode.

## Home Dashboard (`/student`)

- [x] Restore the non-CoachHub homepage direction.
- [x] Bring back the nested glowing outline rings direction using real subject data.
- [x] Move the right-side question block higher and reduce the live feed below it.
- [x] Wire the jump-in launcher tiles to real routes.
- [x] Remove the fabricated homepage leaderboard and keep the arena feedback local to the learner's real run.
- [x] Replace fake live-feed padding with real coach, topic, subject, streak, and accuracy signals only.
- [ ] Verify homepage questions are always served from the real question/session source.
- [ ] Verify answer -> next question auto-advances without getting stuck in "starting".
- [ ] Show a temporary weakness/solidify popup on wrong answers using real misconception/topic context.
- [ ] Verify homepage answers update readiness, history, heatmap, misconception state, and coach planning.
- [ ] Final visual pass: spacing, right rail, rings, live feed density, and responsiveness.

## Coach Mode (`/student/coach`)

- [x] Separate coach mode entry point exists in the shell.
- [x] Remove the page-local dark toggle so coach mode inherits the real shell theme.
- [ ] Rework `CoachHubV3.vue` into the activated coach mode instead of an alternate homepage skin.
- [ ] Pull real preparation intensity, next mission, repair plan, and planner data from coach/preparation services.
- [ ] Remove any local-only novelty controls that belong to the main shell instead.
- [ ] Reconcile visual language with recovered `preparation.rs`, `view_builder_1.rs`, and the user's screenshot notes.

## Journey / Explore

- [ ] `frontend/src/views/student/journey/JourneyHome.vue`
- [ ] `frontend/src/views/student/journey/JourneyStation.vue`
- [x] Remove synthetic Journey course counters (`questions`, `attempted`, fake level rail counts) and replace them with real route/mastery signals.
- [x] Journey Station practice launch now targets the live station topic or the route's real topic scope instead of the first three learner topics.
- [x] Journey Station phase labels now recognize the backend's live station types and only exposes manual advance when a station is already marked `passed`.
- [ ] Restore the map/station progression direction from recovered `view_builder_1.rs`.
- [ ] Verify station progress, mastery gates, and dependency logic are real.

## Diagnostic DNA

- [x] Fix the start bug path so the flow can begin.
- [x] Record answer/skip/timeout/focus/input interactions.
- [ ] `frontend/src/views/student/diagnostic/DiagnosticLauncher.vue`
- [ ] `frontend/src/views/student/diagnostic/DiagnosticSession.vue`
- [ ] `frontend/src/views/student/diagnostic/DiagnosticReport.vue`
- [ ] Verify launcher, session, and report all use real diagnosis outputs, misconceptions, and repair plan data.
- [ ] Bring the report UX closer to the recovered diagnosis/repair-plan direction.

## Practice / Session / Debrief

- [ ] `frontend/src/views/student/practice/PracticeHub.vue`
- [ ] `frontend/src/views/student/practice/CustomTest.vue`
- [ ] `frontend/src/views/student/session/SessionView.vue`
- [ ] `frontend/src/views/student/session/SessionDebrief.vue`
- [ ] `frontend/src/components/question/QuestionCard.vue`
- [ ] Verify presented/answered/skipped/timed-out/flagged/solidify interactions are all persisted.
- [ ] Verify scores and readiness updates are visible immediately after completion.

## Mock Centre / Prepare Test

- [x] Mock session event persistence is wired for presented/skip/timeout/answer.
- [ ] `frontend/src/views/student/mock/MockHome.vue`
- [ ] `frontend/src/views/student/mock/MockSetup.vue`
- [ ] `frontend/src/views/student/mock/MockHall.vue`
- [ ] `frontend/src/views/student/mock/MockReview.vue`
- [ ] `frontend/src/views/student/mock/MockHistory.vue`
- [ ] Verify setup, simulation, review, and history all derive from the real mock session state.
- [ ] Align Prepare Test UX with recovered `dto_29.rs` and preparation engine clues.

## Knowledge Gap / Mistake Lab / Memory

- [x] Quick Scan and review queue topic IDs are real.
- [x] `frontend/src/views/student/knowledge-gap/GapHome.vue`
- [x] `frontend/src/views/student/knowledge-gap/GapScan.vue`
- [x] `frontend/src/views/student/mistakes/MistakeLab.vue`
- [x] `frontend/src/views/student/memory/MemoryHome.vue`
- [x] Gap Home sections now group by real backend `severity_label` values instead of invented thresholds.
- [x] Gap Scan copy and stat cards now state the real subject-probe fallback when no recorded gaps exist.
- [x] Mistake Lab repair buttons now launch real targeted drills from recorded misconception topics where available.
- [x] Memory recovery ladder is labeled as a UI derivation from real strength, not a fake persisted stage.
- [ ] Verify misconception graph, retry zone, review queue, and weak-topic planning are all live.
- [ ] Reuse the recovered misconception graph/node direction where it fits.

## Progress / History / Mastery

- [x] `frontend/src/views/student/progress/ProgressOverview.vue`
- [x] `frontend/src/views/student/progress/Analytics.vue`
- [x] `frontend/src/views/student/progress/History.vue`
- [x] `frontend/src/views/student/progress/MasteryMap.vue`
- [x] Learner truth counts now come from full tracked topic/skill/memory tables instead of the top-5 summary window, so Progress and Parent dashboards no longer undercount tracked rows.
- [x] Learner truth mastery/readiness now averages across all tracked topic states instead of the capped summary window.
- [x] History sidebar labels now explicitly describe a recent 48-session window instead of posing as lifetime totals.
- [x] History now includes persisted game sessions and elite runs instead of dropping those modes from the learner timeline.
- [x] Mastery Map now recomputes from current learner data on load and subject switch, with any cached fallback explicitly labeled as a stale snapshot instead of being treated as live truth.
- [x] Student readiness labels and colors are now normalized across Progress, Practice Hub, and Mock Home so real readiness data no longer renders with the wrong tone or raw token text.
- [x] Topic practice CTAs now hand a real `topic` target through to Practice Hub instead of dropping into generic practice.
- [x] Verify history is fully real and not placeholder data anywhere in the page tree.
- [ ] Verify mastery map and heatmap are derived from persisted learner activity and topic mastery tables.
- [ ] Verify analytics totals match session history and debrief outputs.

## Exam Intel / Family View

- [ ] `frontend/src/views/student/exam-intel/ExamIntelHome.vue`
- [ ] `frontend/src/views/student/exam-intel/FamilyView.vue`
- [x] Practice session selection can now constrain to real `family_id` values without reactor top-ups leaking other families into the run.
- [x] Family and hotspot practice CTAs now pass real question-family IDs into practice sessions, so those drills are actually family-scoped.
- [x] Family health surface is now labeled as content/observed health until a student-scoped family-performance endpoint exists.
- [ ] Reconnect family/lineage, blockers, exam risk, and quick-win insights using recovered lineage/question-editor clues.

## Library / Teach / Spark / Glossary / Curriculum

- [ ] `frontend/src/views/student/library/LibraryHome.vue`
- [ ] `frontend/src/views/student/teach/TeachMode.vue`
- [ ] `frontend/src/views/student/spark/SparkHome.vue`
- [ ] `frontend/src/views/student/glossary/GlossaryHome.vue`
- [ ] `frontend/src/views/student/glossary/GlossaryEntry.vue`
- [ ] `frontend/src/views/student/glossary/GlossaryAudio.vue`
- [ ] `frontend/src/views/student/glossary/GlossaryCompareView.vue`
- [ ] `frontend/src/views/student/glossary/FormulaLabView.vue`
- [ ] `frontend/src/views/student/curriculum/CurriculumHome.vue`
- [x] Teach mode lesson opens and micro-check reveals now write into the real interaction log instead of disappearing locally.
- [x] Library and Glossary question/bundle handoffs now route into real practice-topic or revision-box entry points instead of generic dead-end pages.
- [x] Glossary Entry linked bundle/question cards now open real downstream routes instead of display-only cards.
- [x] Glossary Compare shared bundles now hand off to the live revision-box flow instead of dead-ending.
- [x] Glossary Audio now speaks the live queue script when speech synthesis is available and its suggested bundles open a real follow-on route.
- [ ] Verify these surfaces use real library/curriculum/example/glossary content instead of placeholder copy.

## Performance / Motivation / Games

- [ ] `frontend/src/views/student/beat-yesterday/BeatYesterdayHome.vue`
- [ ] `frontend/src/views/student/rise/RiseHome.vue`
- [ ] `frontend/src/views/student/elite/EliteHome.vue`
- [ ] `frontend/src/views/student/elite/EliteArena.vue`
- [ ] `frontend/src/views/student/elite/EliteSession.vue`
- [ ] `frontend/src/views/student/elite/EliteRecords.vue`
- [ ] `frontend/src/views/student/elite/EliteInsights.vue`
- [ ] `frontend/src/views/student/games/GamesHub.vue`
- [ ] `frontend/src/views/student/games/MindStackGame.vue`
- [ ] `frontend/src/views/student/games/TugOfWarGame.vue`
- [ ] `frontend/src/views/student/games/TrapsHub.vue`
- [x] MindStack and Tug of War now stay inside the selected subject, mark timed answers honestly, use the real completion pipeline, and compute accuracy from real attempts.
- [x] Beat Yesterday now hands subject/topic/target context through Custom Test and closes the daily climb from debrief.
- [x] Elite score sessions now write real personal-best rows and badge rows that can be surfaced back into the Records wall.
- [x] Elite Records now reads persisted personal bests and earned badges instead of inferring trophies from the current profile snapshot.
- [x] Traps Hub now launches real trap sessions, persists round submissions/confusion reasons, and shows the live review loop from backend trap state.
- [x] Rise now derives its proof card from real subject activity history and checks for live stage transitions instead of shipping static proof copy.
- [ ] Verify scores, streaks, and challenge outcomes persist into history and analytics.

## Calendar / Upload / Settings / Onboarding

- [ ] `frontend/src/views/student/calendar/AcademicCalendar.vue`
- [ ] `frontend/src/views/student/upload/UploadWizard.vue`
- [ ] `frontend/src/views/student/Settings.vue`
- [ ] `frontend/src/views/student/onboarding/Welcome.vue`
- [ ] `frontend/src/views/student/onboarding/Subjects.vue`
- [ ] `frontend/src/views/student/onboarding/ContentPacks.vue`
- [ ] `frontend/src/views/student/onboarding/Diagnostic.vue`
- [ ] Verify onboarding and settings correctly initialize real learner state and content readiness.

## Shared Components / IPC / Backend

- [ ] Audit `frontend/src/layouts/StudentLayout.vue` for remaining fake shell state.
- [ ] Audit question/session-related composables and IPC wrappers for placeholder fallbacks.
- [ ] Audit `src-tauri/src/commands.rs` and `crates/ecoach-commands/src/*` command coverage for every student flow.
- [ ] Audit persistence tables/services for attempts, history, topic mastery, misconceptions, memory, and readiness recompute triggers.
- [ ] Add or restore focused regression tests for the most failure-prone flows.

## Recovered Clues To Keep Reusing

- [ ] `QuestionEditorModern.vue`: visual polish, graph relationships, family/lineage concepts.
- [ ] `MisconceptionNode.vue`: misconception graph language.
- [ ] `preparation.rs`: coach/preparation mode separation and intensity logic.
- [ ] `view_builder_1.rs`: journey/home/map/station/dashboard payload composition.
- [ ] `repository_3.rs`: quick wins, comebacks, blockers, exam snapshot dashboard structure.

## Recovered Material To Treat As Reference Only

- [ ] `CoachHubV2.vue` and older screenshot: useful for structure, not the final visual target.
- [ ] Corrupt files such as `brain.rs`, `intelligence.rs`, and mixed binary/text artifacts: do not treat as source of truth.

## Final Verification Sweep

- [x] `pnpm -C frontend exec vue-tsc --noEmit`
- [x] `pnpm -C frontend build`
- [x] `cargo check -p ecoach-commands -p ecoach-tauri`
- [x] Focused cargo tests for touched flows
- [ ] Manual browser smoke pass on home, coach, diagnostic, practice, mock, history, and misconception flows
