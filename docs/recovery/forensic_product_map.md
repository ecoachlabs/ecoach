# Forensic Product Map

Date: 2026-04-19

Purpose: preserve the recovered product DNA from the Vue files, Rust files, screenshot, and current codebase before rebuilding more UI or behavior. This is not an implementation plan yet. It is the evidence map we should use so the rebuild does not drift into fake data, placeholder scores, or a homepage that was never the intended product.

## Source Evidence

| Source | Reliability | What it gives us |
| --- | --- | --- |
| Recovered Vue files from `C:\Users\surfaceSudio\Downloads\New folder (3)-20260419T161544Z-3-001\New folder (3)` | Mixed. A minority are readable SFCs; many are corrupt, zero-byte, or fragments. | Admin/content studio contracts, question families, key concept quiz authoring, essay flow, misconception graph, design language hints. |
| Recovered Rust files from the same recovery folder | Mixed. Some files are clean domain contracts; some are compiler logs, generated code, or unrelated fragments. | Learning intelligence model, misconception engine concepts, topic clinic, preparation, mock/test review, coach states, elite layer, parent reporting. |
| Screenshot: `C:\Users\surfaceSudio\Downloads\WhatsApp Image 2026-04-08 at 3.08.45 PM.jpeg` | High for older dashboard visual structure; not final version. | Left rail, top utility bar, central readiness metrics, mode cards, subject health, right trail/live feed. |
| User memory from this thread | High for product intent; exact pixel details need reconstruction. | Final dashboard had top-bar Activate Coach, nested glowing outline doughnut rings, live question on the right, reduced feed below, and Coach Hub as an activated mode rather than the homepage itself. |
| Current app in `C:\Users\surfaceSudio\OneDrive\ecoach` | High for actual working code. | Current routes, available backend commands, real vs placeholder data gaps, pages that exist today. |

## Executive Summary

The recovered files do not give us a clean runnable old app. They give us a product map.

The strongest recoveries show that the old system was not only a quiz app. It was a learning intelligence system built around real questions, question families, misconception diagnosis, memory decay, topic clinic repair, readiness scoring, preparation planning, mock exam review, and a coach that could be activated from the shell.

The current app already contains many of the same ingredients, but the homepage and several learner-facing pages have drifted into a mixed state: some metrics are real, some interaction paths are not persisted, some visual feed/history items are synthetic, and some engines exist in the backend but are not wired into the UI.

The rebuild should therefore move in two tracks:

1. Product reconstruction: recreate the intended UX across every page, using the screenshot and recovered Vue/Rust as product evidence.
2. Truth reconstruction: make every learner-facing interaction recordable and make every score, heatmap, weakness, history item, and misconception claim traceable to persisted evidence.

## Evidence Reliability Notes

### High Confidence

These should influence the rebuild directly:

- Question management and authoring contracts from readable Vue SFCs.
- Question family and lineage fields.
- Key concept quiz authoring model.
- Misconception graph UI concept from `MisconceptionNode.vue`.
- Domain vocabulary from readable Rust DTOs and planners.
- Screenshot dashboard structure.
- User clarification that Activate Coach belongs in the top bar and Coach Hub is a mode, not the default homepage brand.
- Current backend commands for home stats, dashboard, diagnostic, practice attempts, memory, priority topics, learner truth, and mastery.

### Medium Confidence

These should inform design and backlog, but need code verification before implementation:

- R-Z Vue filename taxonomy such as `StudioQuestionBrowser`, `SearchQuestions`, `ViewQuestionModal`, `ViewRelatedQuestions`, `UploadQuestionData`.
- Rust compiler-warning files that preserve old paths and names.
- Partial `QuestionEditorModern.vue` design language.
- Question generation lab and content studio flows beyond the readable components.

### Low Confidence or Discard

These should not be copied into the rebuild without proof:

- Generated/filter/adblock-looking Rust files.
- Polluted TypeScript diagnostic fragments saved as `.rs`.
- Zero-byte Vue files.
- Corrupt Vue files with only strings or partial tags.
- Unrelated business/idea-analysis pipeline fragments.

## Product Lineage

The application appears to have evolved through at least three eras:

1. Content and intelligence foundation:
   - Question bank.
   - Key concept questions.
   - Essay questions.
   - Families, variants, follow-ups, remediation.
   - Misconception graph.
   - Admin moderation and publishing.

2. Learner intelligence engine:
   - Diagnostic DNA.
   - Memory decay.
   - Misconception detection.
   - Topic Clinic.
   - Readiness and mastery tracking.
   - Mock and preparation planning.
   - Emotional/coach state logic.

3. Dashboard and coach experience:
   - Left navigation shell.
   - Top utility/status bar.
   - Readiness rings.
   - Live question.
   - Live feed/trail.
   - Activate Coach mode.
   - Subject health and action modes.

The current app has pieces from all three, but the user-facing homepage does not yet fully represent the intended final dashboard or the real recording contract.

## Recovered Vue Map

### Forensic Coverage

Recovered Vue files were reviewed by filename range:

| Range | Count | Useful files | Main signal |
| --- | ---: | --- | --- |
| A-C | 51 | `CreateQuestionForm_1.vue`, `CreateKeyConceptQuestionModal.vue`, partial `CreateTrueOrFalseQuestionModal.vue` | Question authoring, key concept quiz, admin modal patterns. |
| D-K | 45 | `EssayQuestion.vue`, `EssayQuestions_1.vue`, `KeyConceptQuiz.vue`, one readable icon/design artifact | Essay authoring, key concept quiz creation and listing. |
| L-Q | 52 | `MisconceptionNode.vue`, `QuestionsManagement.vue`, partial `QuestionEditorModern.vue` | Misconception graph, admin question management, modern question editor design. |
| R-Z | 47 | No trustworthy full SFCs; useful filenames only | Studio browser, search, view/update modals, upload/import, related questions. |

### Question Management

Recovered Vue shows a mature question management workflow:

- Level, course, and topic filters.
- Search and pagination.
- Add Question entry.
- Bulk confirm, publish, trash, and delete.
- Row actions for View Details, Move to Trash, and Delete Permanently.
- Status flags including confirmed, public, and flagged.
- Admin table rather than a simple static list.

Product implication: the current app should treat questions as governed content, not loose quiz records. Learner pages should only serve eligible real questions from the bank or generated questions that have an explicit provenance state.

### Question Authoring

Recovered Vue shows a structured authoring workflow:

- Question setup card.
- Level, course, topic, question type, status.
- Supported types include single-choice, essay, and true/false.
- Child editors like `SingleQuestion`, `EssayQuestion`, and `TrueOrFalse`.
- Batch creation using an Add More Questions style flow.
- Save current and save all actions.

Product implication: the authoring system was designed to create many related records in one session. The frontend should preserve this batch authoring mental model instead of forcing one isolated question at a time.

### Family and Lineage Model

Recovered Vue fields include:

- `family_id`
- `family_title`
- `variant_label`
- `variant_index`
- `lineage_id`
- `instance_id`
- `role`
- `tags`
- `notes`

Recovered roles include:

- Standalone
- Root
- Variant
- Follow-up
- Remediation

Product implication: this is central. A wrong answer should not just mark a score down. It should be able to trigger a related remediation question, a follow-up, a variant, or a misconception repair path. This directly supports the user's requested temporary popup: "known weakness" and "solidify it".

### Key Concept Quiz

`KeyConceptQuiz.vue` and related files show a deep content model:

- Level, course, topic.
- Difficulty.
- Confirmed and publish states.
- Key concept, law, phenomenon, or principle.
- Question or scenario.
- Solution.
- Ordered answer options.
- Correct answer.
- Rich text and math support.
- Accordion admin list.
- Excel upload path.

Product implication: key concept questions are one of the clearest recovered bridges between content management and learner diagnosis. They should be used for "real questions" on the homepage if available, especially because they are concept-tagged.

### Essay Flow

Recovered essay components show:

- Prompt-level framing.
- Multiple essay items in one flow.
- Rich-text expected responses.
- Rich-text explanations.

Product implication: the app was not only multiple-choice. Essay pages and future diagnostic modes should support structured response evaluation and explanatory feedback.

### Misconception Graph

`MisconceptionNode.vue` used `@vue-flow/core` and included:

- Warning-triangle visual language.
- Diagnostic value.
- Connection handles.
- Node-level presentation for misconception states.

Product implication: the "misconception engine" should not be hidden as a backend-only object. It should have a visual map or graph surface where misconceptions are seen as connected, repairable nodes.

### Modern Question Studio

`QuestionEditorModern.vue` showed a dark/glass modern question studio with:

- `GlassPanel`
- `GlassInput`
- `GlassButton`
- `GlassBadge`
- `AnswerOptionsModern`
- `RelationshipVisualizer`
- Unsaved changes state.
- Family graph.

Product implication: keep the idea of a premium authoring studio, but do not copy old glass UI blindly. Use it as evidence that relationship visualization and family editing were important.

### Filename Taxonomy

The R-Z recovered Vue filenames still matter even when source is corrupt:

- `StudioQuestionBrowser`
- `SearchQuestions`
- `ViewQuestionModal`
- `ViewRelatedQuestions`
- `UploadQuestionData`
- `UpdateQuestion`
- `QuestionDetails`
- `SingleQuestion`
- `RiddleQuestion`
- Flagged and deleted question management.

Product implication: there was a full content studio ecosystem. The current rebuild should not reduce admin/content tooling to CRUD alone.

## Recovered Rust Map

### High-Confidence Files

The most product-relevant Rust evidence included:

- `compound_planner.rs`
- `misconception_engine.rs`
- `emotional_engine.rs`
- `freshness_manager.rs`
- `subject_graph.rs`
- `preparation.rs`
- `repository.rs`
- `repository_3.rs`
- `orchestrator_3.rs`
- `dto.rs`
- `dto_1.rs`
- `dto_2.rs`
- `dto_27.rs`
- `dto_29.rs`
- `commands_3.rs`
- `view_builder_1.rs`
- `image_5.rs`
- `mod_3037.rs`

These point to a learning system built on topic graph, mastery, misconception repair, memory, preparation, mock exams, and coach behavior.

### Medium-Confidence Files

Useful but requiring care:

- `windows_71.rs`: compiler warnings preserving old paths like `journey/diagnostic_engine.rs`, `memory_mode`, `mental`, and `keyword`.
- `core_5.rs`: project/status text around "Quizmine".
- `main_116.rs`: PHP/Laravel-like Studio CRUD for concept atoms, links, and misconceptions.

### Discard or Quarantine

Do not use as product truth without separate confirmation:

- `risk_intelligence.rs`: appears to be TypeScript diagnostics rather than product code.
- `generated_8.rs`
- `wrappers_2.rs`
- `eq_6.rs`
- Most `mod_*`, `group_*`, `layers_*`, and `mesh.rs` style files.
- Separate `evaluator.rs` / `orchestrator.rs` business or idea-analysis pipeline.

## Rust Product Concepts

### Learning Journey

Recovered Rust points to:

- Subjects.
- Concepts.
- Stations.
- Prerequisites.
- Dependents.
- Detours.
- Concept graph and subject graph.

Expected UX:

- Mastery Map should be a real graph of topic/concept relationships.
- Detours should be generated when a learner fails prerequisite checks.
- Journey progress should be based on attempts, diagnostics, and memory state.

### Topic Clinic / Topic Room

Recovered concepts include:

- Per-concept diagnosis.
- Action modes.
- Proof contract.
- Evidence spine.
- Strategy file.
- Cause hypotheses.
- Intervention memory.
- Delayed recall.

Expected UX:

- Topic Clinic should not only show weak topics. It should explain why the weakness exists, what evidence supports it, and what repair action will be taken.
- Each repair action should write back to history, memory, and misconception state.

### Misconception Intelligence

Recovered Rust points to:

- Stored misconceptions.
- Bait questions.
- Error-detection questions.
- Correction questions.
- Wrong-answer patterns.
- Repair strategies.

Expected UX:

- A wrong answer should be classified when possible.
- The app should identify known weakness or likely misconception.
- The app should offer a temporary popup asking whether the learner wants to solidify it.
- If the learner accepts, the app should open or queue a remediation path tied to the question family and misconception record.

### Mastery and Readiness

Recovered terms include:

- Novice to mastery state progression.
- Exam readiness.
- Fragile.
- At risk.
- Reactivation.
- Retention decay.

Expected UX:

- Readiness rings must be backed by real learner evidence.
- A "fragile" state should differ from "unknown" and "weak".
- A topic can be strong today but due for reactivation tomorrow.

### Mock and Test Center

Recovered Rust points to:

- Mock history.
- Active sessions.
- Readiness score.
- Predicted range.
- Confidence.
- Recommended mode.
- Timing.
- Flags.
- Diagnosis.
- Next actions.

Expected UX:

- Mock Centre should generate review data, not just finish screens.
- Timing, skipped questions, flags, and confidence should be persisted.
- Post-test review should feed readiness, misconception, memory, and topic clinic.

### Prepare Test Mode

Recovered preparation concepts include:

- Selected topics.
- Days left.
- Target date.
- Target score.
- Session length.
- Difficulty preference.
- Timer preference.
- Hint preference.
- Live guidance.
- Danger zones.
- Revise tonight.

Expected UX:

- Prepare Test should behave like a plan with daily missions.
- It should not be a static quiz launcher.
- It should adapt based on real attempts and upcoming exam dates.

### Coach Intelligence

Recovered Rust includes:

- Emotional signals.
- Coach states.
- Prep phases.
- Calendar-aware planning.

Recovered coach states include:

- `calm_guide`
- `teacher_mode`
- `rescue_mode`
- `confidence_repair`
- `performance_coach`
- `accountability`

Recovered prep phases include:

- `build`
- `strengthen`
- `firming_up`
- `wrap_up`
- `performance`

Expected UX:

- Activate Coach should switch the system into a context-aware mode.
- Coach tone and actions should depend on evidence, not random copy.
- Coach should be available from the global shell, not as the default homepage identity.

### Compound Planning

Recovered planning factors include:

- Nearest exam relevance.
- Final exam relevance.
- Cross-exam frequency.
- Weakness.
- Foundation value.
- Retention decay.
- Goal alignment.

Expected UX:

- Today's recommendation should explain why it matters.
- A topic can be prioritized because it is weak, foundational, exam-relevant, decaying, or goal-aligned.

### Elite Layer

Recovered elite concepts include:

- EPS.
- Tier.
- Speed.
- Precision.
- Depth.
- Composure.
- Consistency.
- Independence.
- Trap resistance.
- Streaks.
- Records.
- Badges.
- Challenges.
- Domination board.

Expected UX:

- Elite should be built on real performance metrics.
- Beat Yesterday, Speed Lab, Answer Lab, Traps, and Marathon should feed the same evidence model.

### Content Freshness

Recovered content governance points to:

- Knowledge atom freshness.
- Stale content.
- Verification status.

Expected UX:

- Question and concept content should have freshness/verification state.
- Learner-facing pages should avoid unverified content unless clearly marked.

### Parent Surface

Recovered Rust mentions:

- Child intelligence.
- Clarity cards.
- Weekly digest.
- Progress report.

Expected UX:

- Parent reporting should be based on the same learner evidence model.
- It should explain what changed, what matters, and what the child should do next.

## Screenshot and User Memory Visual Contract

### Older Screenshot

The screenshot shows an older dashboard with:

- Persistent left navigation.
- Top utility/status bar.
- Greeting and short motivational line.
- Readiness score card with circular ring.
- Session, accuracy, and study-time metrics.
- Daily goal and streak widgets.
- Colored mode cards:
  - Start Quiz
  - Marathon
  - Mock Centre
  - Games
  - Prepare Test
  - Topic Clinic
- Subject health section.
- Right rail named "Your Trail".
- Continue card.
- Live feed entries.
- Floating help/action button.

### User Memory of the Later Lost Version

The later version was more organized and visually stronger:

- Activate Coach was a button in the top bar with light/dark mode and other utilities.
- The homepage was not "CoachHub".
- CoachHub was a different mode/state activated by clicking the coach button.
- Dashboard used doughnut rings, including nested rings.
- Rings had subtle glows.
- Rings were outline-only, not filled blobs.
- The live question was on the right side.
- The live feed was reduced and placed beneath the question.
- The whole app, not only the homepage, should inherit the restored product logic.

### Reconstructed Homepage Direction

The homepage should become a learner cockpit:

- Left rail: persistent global navigation.
- Top bar: global status and controls.
- Main canvas: readiness rings, daily goal, subject health, and mode launchers.
- Right rail: live question first, compressed activity/feed below.
- Activate Coach: global top-bar control that turns on coaching context.
- Coach active state: visibly changes guidance and action priority without replacing the dashboard with a separate CoachHub page.

## Current App Reality

### Current Routing

Current `/student` and `/student/v3` render `CoachHubV3` through `frontend/src/router/index.ts`. Older v1/v2 surfaces exist but are disabled or redirected. Legacy `/coach/*` paths redirect to student routes.

Product implication: the current homepage is technically a CoachHub page even though the intended product says CoachHub should be activated as a mode. This is a naming and UX mismatch.

### Current Student Layout

`StudentLayout.vue` includes a large left nav with routes for:

- Home.
- Explore.
- Teach.
- Past Questions.
- Custom Test.
- Prepare Test.
- Mock History.
- Examples.
- Diagnostic DNA.
- Knowledge Gap.
- Gap Scan.
- Mistake Lab.
- Retry Zone.
- Library.
- Curriculum.
- Revision Box.
- Glossary.
- Audio Glossary.
- Formula Lab.
- Mental.
- Review Queue.
- Memory.
- Progress.
- Analytics.
- Mastery Map.
- History.
- Calendar.
- Exam Intel.
- Onboarding.
- Beat Yesterday.
- Marathon.
- Rise.
- Elite.
- Answer Lab.
- Speed Lab.
- Games.
- Traps.
- Uploads.
- Settings.
- Adeo v2 shortcut.

Product implication: the app already has the scope of the old system. The work is not to invent pages. It is to wire them into one truthful learning record and restore their intended hierarchy.

### Current Homepage Dependencies

Current `CoachHubV3.vue` uses:

- `getCoachNextAction`.
- `getHomeLearningStats`.
- `getStudentDashboard`.
- `getPriorityTopics`.
- `useHomepageArena`.

Product implication: the page is close to the right surface, but the right rail, scoring, feed, coach activation, and metric provenance need cleanup.

### Current Real Backend Capabilities

These appear to have real backend support:

- Home stats.
- Student dashboard.
- Priority topics.
- Next action.
- Learner truth.
- Diagnostic battery/session/report/sync.
- Practice session attempt pipeline.
- Memory dashboard.
- Memory queue.
- Memory topic summaries.
- Misconception data during attempts.
- Mastery map backend.
- Streak from attempt days.
- Admin question/content studio.

### Current Gaps and Mixed-Reality Areas

These need to be fixed before claiming "everything is real":

- History page is currently a static placeholder.
- Admin coverage heatmap is placeholder.
- Mastery Map UI fabricates graph links from priority topics instead of using the real mastery map command.
- Homepage live feed has synthetic items.
- Homepage quick-arena score and XP appear local rather than persisted.
- Session skip advances UI without recording a skip.
- Session flag does not appear to write a backend record.
- Session completion uses a plain `complete_session` path rather than the richer pipeline path in some places.
- Memory review queue is real, but direct retrieval/recheck flows are not fully wired.
- Gap repair pages read real dashboard/priority gaps but do not appear to fully advance repairs or generate plans from the main surface.
- Admin content editor has question record support, while lesson/answer/curriculum types are disabled or unimplemented.
- IPC mismatches were observed:
  - `frontend/src/ipc/memory.ts` sends `studentId` to `process_decay_batch`, while backend expects `limit`.
  - `frontend/src/ipc/memory.ts` sends `input` to `complete_recheck`, while backend expects `recheckId`.
  - `frontend/src/ipc/gap.ts` sends `result` to `advance_repair_item`, while backend expects `completed`.

## Truth and Recording Doctrine

This is the rule for rebuilding the app from here forward:

Every learner-facing claim must be either evidence-backed or explicitly labeled as local/demo/session-only.

### Events That Must Be Recorded

The app should record:

- Every question served.
- Question source and provenance.
- Topic, concept, difficulty, family, variant, and misconception tags.
- Answer submitted.
- Correctness.
- Selected option or free response.
- Time to answer.
- Hints used.
- Skips.
- Flags.
- Confidence if captured.
- Retry attempts.
- Diagnostic phase entry and exit.
- Mode entry and exit.
- Coach activation and deactivation.
- Coach suggestion shown.
- Coach suggestion accepted or dismissed.
- Weakness popup shown.
- Solidify accepted or dismissed.
- Remediation path opened.
- Memory review shown.
- Memory review result.
- Recheck completed.
- Topic clinic action started.
- Topic clinic action completed.
- Mock/test question events.
- Mock/test completion and review outcomes.
- Feed item generated.

### Scores That Must Have Source of Truth

These must not be decorative:

- Readiness.
- Accuracy.
- Streak.
- Study time.
- XP.
- Level.
- Topic mastery.
- Subject health.
- Knowledge gap count.
- Misconception count.
- Memory risk.
- Heatmap intensity.
- Mock readiness.
- Elite/EPS metrics.

Each should answer:

- What table or command produced this?
- What event changes it?
- Is it cached or computed live?
- When was it last refreshed?
- What does zero mean: no data, weak, or truly zero?

### Feed Rules

The live feed must not invent activity.

Valid feed item sources:

- Answer event.
- Diagnostic event.
- Coach event.
- Memory event.
- Topic clinic event.
- Mock/test event.
- Content unlocked.
- Streak/goal achievement.
- Weakness detected.
- Misconception repaired.

If a feed item is motivational copy with no event behind it, it should not be mixed into the activity feed. It belongs in coach copy or empty state copy.

### Homepage Question Rules

The homepage question should:

- Be served from a real question source.
- Carry a stable question id.
- Carry topic/concept/difficulty metadata.
- Persist the served event.
- Persist answer, skip, flag, and timeout events.
- Update relevant stats immediately after answer.
- Advance automatically to the next question after answer.
- Trigger the known weakness popup when wrong and a misconception/weakness can be inferred.
- Offer "solidify it" and record the learner's response.

### Heatmap Rules

Heatmaps should be computed from real evidence:

- Attempts by topic/concept.
- Accuracy.
- Recency.
- Time spent.
- Diagnostic outcomes.
- Misconception frequency.
- Memory decay.
- Mock/test performance.

Heatmap cells should not be filled by static placeholder values.

### Misconception Engine Rules

Misconceptions should be created or updated from:

- Wrong answer patterns.
- Known distractor mappings.
- Bait question failures.
- Diagnostic outcomes.
- Repeated repair failures.
- Teacher/admin tagged evidence.

Misconception claims should include:

- Misconception id.
- Confidence.
- Evidence count.
- Last observed date.
- Repair status.
- Linked questions or concepts.

## Page-by-Page Reconstruction Requirements

### Home

Target:

- Learner cockpit, not CoachHub as a page identity.
- Global top bar with Activate Coach.
- Nested glowing outline rings for readiness/mastery/memory/gap/exam.
- Right rail with live question first and reduced feed below.
- Mode cards as launchers.
- Subject health from real topic/subject data.
- Automatic next question after answer.
- Wrong-answer weakness popup with solidify option.

Must be real:

- Question served.
- Answer recorded.
- Scores updated.
- Feed item recorded.
- Weakness/misconception event recorded when detected.

### Activate Coach Mode

Target:

- Top-bar button.
- Activates coach overlay/state/directive.
- Does not replace homepage navigation.
- Can focus the right rail or open a coach directive panel.

Must be real:

- Activation event.
- Active coach state.
- Suggested action.
- Accept/dismiss outcome.

### Diagnostic DNA

Target:

- Diagnostic should start reliably.
- It should not get stuck at "starting".
- Diagnostic should produce a report and sync to learner truth.

Must be real:

- Battery selection.
- Session start.
- Every diagnostic answer.
- Session completion.
- Report generation.
- Learner truth update.

### Practice / Quiz / Homepage Arena

Target:

- Same question event contract across all question surfaces.
- Next question should appear automatically after answer where the mode expects flow.
- Wrong answer can trigger weakness popup.

Must be real:

- Answer.
- Skip.
- Flag.
- Timeout.
- Hint use.
- Completion pipeline.

### Topic Clinic

Target:

- Diagnoses topic weakness and shows proof.
- Offers targeted repairs.
- Uses question family/remediation records where possible.

Must be real:

- Repair item start.
- Repair item complete.
- Recheck result.
- Memory follow-up.

### Knowledge Gap / Gap Scan / Retry Zone / Mistake Lab

Target:

- These should be different lenses over the same evidence, not separate fake score worlds.

Must be real:

- Gap source.
- Attempt history.
- Misconception link.
- Repair status.

### Memory / Review Queue

Target:

- Real spaced review and recheck queue.
- Shows why something is due.

Must be real:

- Due item.
- Retrieval attempt.
- Recheck completion.
- Decay batch.
- Topic memory state.

### Mastery Map

Target:

- Use real mastery map backend, not fabricated frontend links.
- Show concepts, prerequisites, detours, fragile states, and mastery progress.

Must be real:

- Node source.
- Edge source.
- Mastery evidence.
- Last update.

### History

Target:

- Replace placeholder with actual learner timeline.
- Filter by mode, topic, date, event type, correctness, misconception, and repair status.

Must be real:

- Attempts.
- Diagnostics.
- Memory reviews.
- Coach actions.
- Repair work.
- Mock/test sessions.

### Mock Centre and Mock History

Target:

- Test sessions feed readiness, timing, flags, and post-test diagnosis.
- Mock history should show actual sessions and reviews.

Must be real:

- Session start/end.
- Question events.
- Timing.
- Flags.
- Score.
- Review outcomes.
- Next actions.

### Prepare Test

Target:

- Calendar-aware plan.
- Daily missions from target date, topics, weakness, retention, and exam relevance.

Must be real:

- Plan settings.
- Mission generated.
- Mission progress.
- Danger zones.
- Revise-tonight items.

### Elite / Beat Yesterday / Marathon / Speed Lab / Answer Lab / Traps

Target:

- Performance modes tied to the same attempt record.
- Elite metrics based on speed, precision, depth, composure, consistency, independence, and trap resistance.

Must be real:

- Mode-specific event metadata.
- Metric calculations.
- Records.
- Streaks.
- Badges.

### Content Studio / Admin

Target:

- Preserve recovered content governance:
  - question families.
  - variants.
  - follow-ups.
  - remediation.
  - key concept questions.
  - essay questions.
  - upload/import.
  - moderation.

Must be real:

- Content status.
- Publication state.
- Verification/freshness.
- Family lineage.
- Topic/concept tags.

## Keep, Pick Up, or Discard

### Keep from Current App

- Existing Tauri/Vue/Rust structure.
- Student shell and broad navigation.
- Current backend commands for dashboard, diagnostics, practice, memory, misconception, learner truth, and mastery.
- Existing pages as route inventory.
- Current content studio foundation.

### Pick Up from Recovered Vue

- Question family/lineage fields.
- Batch authoring flow.
- Key concept quiz structure.
- Misconception node/graph visual concept.
- Relationship visualizer idea.
- Admin moderation actions.
- Excel upload path.
- Search and browser patterns for question studio.

### Pick Up from Recovered Rust

- Topic Clinic proof/evidence model.
- Misconception repair strategy.
- Compound planning factors.
- Coach states and prep phases.
- Memory decay and reactivation language.
- Mock/test review intelligence.
- Elite performance dimensions.
- Content freshness/verification.

### Pick Up from Screenshot and User Memory

- Left rail and top utility shell.
- Activate Coach in top bar.
- Nested glowing outline rings.
- Right rail with live question above feed.
- Mode cards as action launchers.
- Subject health section.
- Feed/trail as actual activity history, not decorative filler.

### Discard or Avoid

- Treating CoachHub as the homepage brand.
- Synthetic scores without source.
- Synthetic feed/history items.
- Frontend-only XP or score that never persists.
- Fabricated mastery map links.
- Decorative heatmaps.
- Placeholder history.
- Copying corrupt recovered source.
- Overusing glass UI just because one recovered editor had glass components.

## Open Questions for User

These are not blockers for the truth rebuild, but they matter for final UX fidelity:

1. What exact utilities were in the final top bar beside Activate Coach, light/dark mode, level, streak, and engine lab?
2. Did the nested rings represent fixed metrics, or could the learner choose which metrics appeared?
3. Was the live question always visible on the homepage, or only when a quick practice mode was active?
4. When the learner answered wrong, did the old app immediately pause flow for weakness repair, or did it let the learner continue and queue the repair?
5. Should "solidify it" open Topic Clinic, a short remediation micro-drill, or an activated coach explanation first?
6. Were feed items grouped by day/session, or was it a simple chronological trail?
7. Should parent/teacher reporting be part of this rebuild phase, or kept as a later slice?

## Immediate Next Work

### Slice 1: Truth Audit

Goal: identify every homepage and learning-interaction value that is synthetic, local-only, or not persisted.

Outputs:

- List of fake/local values.
- Backend command/table source for real values.
- Missing event types.
- IPC mismatches.
- Minimum schema or command changes.

### Slice 2: Unified Interaction Event Contract

Goal: define one event contract for served, answered, skipped, flagged, timed out, hinted, remediated, and reviewed questions.

Outputs:

- Event DTO.
- Backend command names.
- Frontend adapter.
- Required persistence.
- Immediate stat refresh behavior.

### Slice 3: Homepage Reconstruction

Goal: rebuild homepage around intended layout and real data.

Outputs:

- Top bar Activate Coach.
- Nested outline rings.
- Right live question rail.
- Reduced real feed.
- Auto-advance question behavior.
- Wrong-answer weakness popup.

### Slice 4: History, Heatmap, and Misconception Wiring

Goal: prove that interactions flow into learner history, heatmaps, and misconception engine.

Outputs:

- Real history page.
- Real heatmap computation.
- Misconception evidence view.
- Topic Clinic repair linkage.

### Slice 5: Page-by-Page UX Recovery

Goal: apply the same evidence-backed design and recording model to Diagnostic DNA, Topic Clinic, Memory, Mastery Map, Mock Centre, Prepare Test, Elite, and Content Studio.

Outputs:

- One truth pack per page.
- Real data sources.
- Visual direction.
- Missing implementation tasks.

## Non-Negotiables

- Do not serve demo questions as real learner work.
- Do not show score movement unless backed by persisted interaction evidence.
- Do not show heatmap or misconception claims from placeholder arrays.
- Do not let answer, skip, flag, or hint interactions disappear.
- Do not bury Coach as a separate homepage when the intended model is top-bar activation.
- Do not implement the screenshot literally; use it as older evidence and combine it with the user's final-version memory.
- Do not copy corrupt recovered files directly into production.

## Working Definition of "Real"

A feature is real when all of these are true:

1. It uses a stable backend source of truth or explicitly declared local-only state.
2. It records the learner action that caused the state change.
3. It can be inspected later in history or another evidence surface.
4. It affects the relevant engine when appropriate: mastery, memory, misconception, heatmap, readiness, streak, or coach.
5. The UI can explain why the learner is seeing the score, weakness, question, or recommendation.

Anything less than this should be treated as prototype behavior until upgraded.
