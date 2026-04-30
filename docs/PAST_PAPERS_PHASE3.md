# Past Papers — Phase 3 Implementation Plan

Status: **in progress** (2026-04-20)

## Goal

Make the Past Papers CMS production-ready. Today it saves basic MCQ and
short-answer questions; it has known bugs and missing formats that will
break real WASSCE/BECE paper authoring.

## Issues and decisions

### Bugs (correctness — must fix)

| # | Issue | Root cause | Fix |
|---|---|---|---|
| B1 | "Essay" in type dropdown saves fail the `question_format` CHECK constraint | CHECK in migration 003 doesn't include `essay`; adding to CHECK in SQLite requires an expensive table rebuild | **Overload** `short_answer` + mark with `primary_pedagogic_function='essay_response'`. No migration. |
| B2 | Multi-correct MCQ cannot be authored | Author UI calls `setCorrectOption` which wipes all other `is_correct` flags | Replace with `toggleCorrectOption`; backend validator already accepts multiple |
| B3 | Fill-in-the-blank not supported at all | No distinct format | Convention: `question_format='short_answer'` + `primary_pedagogic_function='fill_blank'`; stem uses `[[1]]`, `[[2]]` blank markers; `question_options` holds acceptable answers (`position`=blank index, multiple rows per position = alternatives) |
| B4 | Segmenter's `is_ambiguous` flag invisible to admin | UI never surfaces it | Carry through in import, render a red dot + tooltip on the row |

### Gaps (missing features)

| # | Feature | Priority | Phase |
|---|---|---|---|
| G1 | Bulk paper delete / archive | High (ops hygiene) | **3A** (ship with bug fixes) |
| G2 | Image attachments (stem / option / explanation) | Critical for math & science | **3B** (next turn) |
| G3 | CSV bulk upload | Nice (PDF/Word import already covers most cases) | **3C** |
| G4 | Answer-key page cross-reference | Accelerator | **3C** |
| G5 | Live LaTeX preview | Nice | **3D** |
| G6 | Student-side rendering for multi-correct MCQ, fill-in-blank | Required to *attempt* the new types | **3B** (paired with 3B backend — authoring can run ahead briefly) |

### Decision log

- **No CHECK-constraint migration for `questions.question_format`.** SQLite 3 doesn't support `ALTER COLUMN ... CHECK ...`; the rename-trick is fragile because `questions` has ~15 `ALTER TABLE ADD COLUMN` migrations downstream (see `052_elite_deep_model.sql`, `057_question_factory_deep.sql`, `070_custom_test_deep2.sql`, etc). Overloading `short_answer` with `primary_pedagogic_function` tags is safer and reversible. If we outgrow it, migration 103+ can formalise the schema later.
- **Fill-in-the-blank stem convention:** `[[1]]`, `[[2]]` placeholders. Ghana past-papers typically use underscores; we substitute at authoring time so the storage is structured.
- **Multi-correct MCQ grading:** the `question_options` table already allows multiple `is_correct = 1` rows. Student-side grading must compare option-id sets, not single option id — that's Phase 3B work on `SessionView` / grading path.
- **Image storage:** planned as external files under `{app_data_dir}/past_paper_assets/{paper_id}/{question_id}/{asset_id}.{ext}` with a new `question_assets` table mapping to DB rows. Bytes streamed through a new IPC command rather than served by a local HTTP endpoint. Deferred to Phase 3B to scope properly.

## Phase 3A — ship NOW (this session)

1. ~~Write this plan doc.~~ ✅
2. **B1: Essay mapping.** Author UI shows "Essay" → saves as `short_answer` + `primary_pedagogic_function='essay_response'`. Load path reverses the mapping so the dropdown shows Essay on edit.
3. **B2: Multi-correct MCQ.** Replace `setCorrectOption` with `toggleCorrectOption`. Surface as square check-buttons instead of round radios. Validation already accepts ≥1 correct.
4. **B3: Fill-in-the-blank.** New UI row format: stem field with "insert blank" button that appends `[[N]]`. Per-blank answer list (one or more acceptable strings). Saved via `question_options` (`position`=blank number).
5. **B4: Ambiguous-row flag.** Add `is_ambiguous: boolean` to the editable row type; set from segmenter; render a red `●` indicator + tooltip.
6. **G1: Bulk paper delete.** Select-multiple checkboxes on `PastPaperList.vue`, "Delete selected" button calls admin_delete_past_paper per row (or a batch command if trivial).

Acceptance: author, reload, re-open all four new question types; bulk-delete two papers; type-check passes; backend compiles.

## Phase 3B — images + student-side rendering ✅ SHIPPED

1. ~~Migration 103~~ ✅ `question_assets(id, question_id FK CASCADE, scope ∈ stem/option/explanation, scope_ref, mime_type, byte_size, data BLOB, position, alt_text, created_at)`. BLOB storage — no file-system paths.
2. ~~PastPapersService methods~~ ✅ `attach_question_asset`, `delete_question_asset`, `list_question_assets`, `list_question_assets_for_questions` (batched), `get_question_asset_bytes`.
3. ~~Tauri commands~~ ✅ `admin_attach_question_asset`, `admin_delete_question_asset`, `list_question_assets`, `get_question_asset_bytes`. Registered in `main.rs`.
4. ~~Assets folded into `admin_get_past_paper` response~~ ✅ — metadata only, no bytes. Editor lazy-streams thumbnails.
5. ~~Admin UI~~ ✅ `PastPaperAuthor.vue` shows a thumbnail strip under the stem and after the model-answer; "Attach image" button per scope; "Save paper first" guard when `question_id` is null; image delete revokes object URL.
6. ~~Student-side rendering~~ ✅ new `QuestionAssetGallery.vue` component fetches assets on mount, streams object URLs, revokes on unmount. `QuestionCard.vue` accepts optional `question-id` prop and inlines the gallery below the stem. `SessionView.vue` threads the question_id through.

**Known limitations still open:**
- Option-scope asset gallery is wired in the backend but not exposed in admin UI yet (per-option images are rare in WASSCE diagrams; defer unless requested).

## Phase 3B-ext — student-side rendering of new types ✅ SHIPPED

1. ~~`MultiMcqQuestion.vue`~~ ✅ checkbox-style multi-select variant; toggles per-option correctness.
2. ~~`PastPaperFillBlank.vue`~~ ✅ parses `[[N]]` markers, renders inline inputs sized to the longest acceptable answer, per-blank feedback (correct / missed / accepted list) after answering.
3. ~~`QuestionCard` dispatcher~~ ✅ picks mode from stem + options (`[[N]]` → fill-blank; >1 `is_correct` → multi MCQ; else legacy single MCQ). Per-mode state; `canPickConfidence` computed gates the confidence prompt uniformly.
4. ~~`SessionView` unmask~~ ✅ `is_correct` now passes through to `QuestionCard` so the dispatcher can see the question shape. The prior `undefined` mask was weak anti-cheat (the raw data lived in memory + devtools); noted as Phase 4 to move grading server-side.

**Phase 4 — server-side grading ✅ SHIPPED**

1. ~~`AnswerSubmission.precomputed_is_correct`~~ ✅ optional override threaded through `process_answer` in `ecoach-student-model`. When Some, the pipeline uses it directly instead of deriving from `selected_option.is_correct` / answer-text match. All downstream state (evidence, mastery, memory, misconception diagnosis) honours it.
2. ~~`SubmitAttemptInput.selected_option_ids` + `blank_answers`~~ ✅ optional fields on the hot-path DTO. Non-breaking for legacy callers (serde defaults to `None`).
3. ~~Server-side grading helpers~~ ✅ `grade_multi_correct` (set equality against `is_correct=1` options) and `grade_fill_blank` (per-blank case/whitespace-insensitive match against accepted alternatives). One DB round-trip per submission.
4. ~~`QuestionCard` emits the full payload~~ ✅ extended `answer` event signature with two optional trailing params; mode-aware submit picks multi-ids / blank-values where relevant.
5. ~~`SessionView.handleAnswer` forwards the payload~~ ✅ computes a client-side correctness preview (instant feedback) and passes the same data through `SubmitAttemptInput`; backend grade becomes the persisted truth when `attemptResult` resolves.

**Known limitations (Phase 5+):**
- Fill-blank answer leakage (UI validation) — a cheat-safe flow moves validation entirely server-side via a dedicated command so the acceptable-answer list never reaches the student's DOM.
- Multi-correct partial credit — current behaviour is all-or-nothing (matches typical WASSCE marking). Partial-credit grading would need evidence-weighting changes in `process_answer`.

## Phase 3C — CSV bulk upload ✅ SHIPPED (answer-key assist still queued)

1. ~~CSV schema~~ ✅ `section,number,type,stem,topic,option_a..option_e,correct,answer,explanation,marks,difficulty`. Types: `mcq`, `mcq_multi`, `true_false`, `short_answer`, `numeric`, `essay`. Multi-correct encoded as `A;C`.
2. ~~`src/utils/pastPaperCsvParser.ts`~~ ✅ pure-TS tolerant parser (quoted multi-line fields, `""` escaped quotes, case-insensitive header lookup, arbitrary column order, alias map for `mcq_multi`/`tf`/`long_answer` etc.). Emits the same `DraftPaper` shape the segmenter uses, plus `topicHints` / `answerHints` so the author UI can resolve topic names per subject.
3. ~~CSV tab in `PastPaperAuthor.vue`~~ ✅ third source tab; upload `.csv`, parse, resolve topics (case-insensitive name lookup), seed editable rows. Per-row warnings for unknown topics / bad types / empty stems.
4. ~~Template download~~ ✅ "↓ Download template" button writes a UTF-8 BOM-prefixed CSV with header + two example rows (Excel-friendly).

**Still queued:**
- Fill-in-the-blank via CSV (needs a bespoke column encoding — currently requires Manual or eventual JSON import).

## Phase 5 — follow-on refinements ✅ SHIPPED

1. ~~Stable option-id upsert~~ ✅ `upsert_past_paper_question` now diffs incoming options against the existing rows: UPDATE where `option_id` matches, INSERT new, DELETE removed (and clean up scope='option' assets for deleted rows). `AdminPastPaperOptionInput` gains `option_id: Option<i64>`. Option-scoped image attachments now survive re-saves; before this fix every save regenerated option ids and silently orphaned assets.
2. ~~Option-scope image attach (admin)~~ ✅ per-option `+📎` button beside each MCQ/multi-MCQ row; thumbnails sit below the option with inline remove. "Save paper first" guard when the option is new (no id yet).
3. ~~Option-scope image render (student)~~ ✅ `McqQuestion` and `MultiMcqQuestion` accept optional `questionId` and render `QuestionAssetGallery` with `scope="option"`, `scope-ref=opt.id` inside each option's text column.
4. ~~Save + refresh round-trip~~ ✅ `onSave` now re-fetches the paper after saving so fresh option ids are plumbed back into the editor — guarantees option-scoped assets stay attachable even after reorders.
5. ~~Answer-key cross-reference~~ ✅ `detectAnswerKey` scans the last 20% of imported text for one-per-line or inline `1. B  2. C  3. A` patterns (≥5 entries required to qualify). `applyAnswerKey` auto-marks the matching draft MCQ options (only where none are already marked), clearing the `is_ambiguous` flag on rows it resolves. Advisory warning reports the count + any unmatched key entries.

## Phase 3D — polish (later turn)

1. Live LaTeX preview pane toggle in the author UI (uses `MathText`).
2. `is_ambiguous` counter in the header ("3 rows need review").
3. Paper archive (soft-delete) distinct from destructive delete.
4. Matching / ordering format UIs (infrequent in WASSCE/BECE — only if demand appears).

## Testing plan

- **Backend:** `cargo check` on `ecoach-commands` + `src-tauri` after every Rust edit.
- **Frontend:** `npx vue-tsc --noEmit` after every Vue edit; keep green.
- **Smoke:** author one paper containing each of MCQ-single, MCQ-multi, short_answer, numeric, true/false, essay, fill-blank. Save → reload → verify round-trip preserves every field.
- **Student side (after 3B):** start a past-paper section that contains each type; verify rendering and grading.
