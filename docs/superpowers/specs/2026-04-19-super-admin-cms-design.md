# Super Admin CMS Design

Date: 2026-04-19

## Purpose

The super-admin area should become an under-the-hood content management system for eCoach. It should let an admin understand what content exists, inspect the question bank, edit structured extracted content, ingest new raw material, refresh content from remote sources, review generated/imported material, and control what becomes active in the student app.

The system should distinguish between raw sources, editable structured records, and generated learning experiences. Tests and practice sessions are assembled on the fly, so the admin does not edit them as primary CMS documents. Instead, the admin edits the structured content records that future tests and sessions draw from.

## Core Model

There are three content layers:

1. Raw source material: PDFs, Word documents, images, CSV/JSON, syllabi, past papers, curriculum files, remote packs, and remote sync sources. These remain provenance.
2. Structured CMS records: extracted DB records such as questions, answer options, explanations, curriculum text, topic mappings, glossary entries, notes, metadata, classifications, and generated drafts. These are editable in the CMS.
3. Generated experiences: practice sessions, tests, diagnostics, and quizzes assembled from structured records. These are inspectable as activity/history, but not edited as canonical content.

The central rule is: Question Bank shows what exists. Content Editor changes what exists. Remote Updates refresh what exists from outside.

## Admin Information Architecture

The admin portal should be organized as a CMS with object-oriented navigation:

- Dashboard
- Content Editor
- Question Bank
- Sources & Ingestion
- Remote Updates
- Review Queue
- Seeding Engine
- Coverage & Stats
- Packs & Publishing
- Users
- Settings

Navigation should be grouped:

- Overview: Dashboard
- Manage Content: Content Editor, Question Bank, Sources & Ingestion, Seeding Engine
- Quality: Review Queue, Coverage & Stats
- Distribution: Remote Updates, Packs & Publishing
- System: Users, Settings

## Dashboard

The dashboard is the operating overview. It should answer:

- How much content do we have?
- What needs attention?
- What is the next useful action?

It should include:

- Content inventory strip: structured records, questions, answers, sources, pending review, active packs, published content.
- Content health: stale sources, failed ingestion jobs, thin topics, low-confidence classifications.
- Action queue: generated drafts needing review, failed parses, remote updates, weak coverage areas.
- Fast actions: add source, open content editor, inspect question bank, run remote update, seed content.
- Recent activity: uploads, generation batches, approvals, pack updates, remote sync attempts.

## Question Bank

Question Bank is a read-heavy inventory and inspection layer. It answers: what questions do we currently have?

The layout should be a two-pane inspector:

- Main pane: searchable/filterable question list or table.
- Right pane: sticky inspection panel for the selected question.

Filters should include subject, topic, source, review status, format, difficulty, generated/manual/remote origin, and active/archive state where available.

Question rows should show stem preview, subject/topic, status, format, source, difficulty, usage, and accuracy where available.

The inspection panel should show:

- Full question stem
- Answer options and correct answer
- Explanation
- What the question tests
- Topic/subtopic mapping
- Source and provenance
- Usage stats such as attempts, accuracy, and average time
- Review status
- Actions: edit in Content Editor, seed similar, archive/deactivate, flag for review, open source

Question Bank should not feel like the main editing screen. Deep editing opens the Content Editor focused on the selected question.

## Content Editor

Content Editor is the mutation layer. It should let admin edit structured records extracted into DB tables.

Supported content types should include:

- Questions
- Answer options
- Explanations
- Curriculum text
- Topic and subtopic records
- Topic mappings
- Glossary entries
- Learning notes and content chunks
- Generated drafts
- Metadata and classification records

The layout should be a three-pane workspace:

- Left pane: content type selector, tree, filters, or record browser.
- Center pane: editable record form or rich editor.
- Right pane: provenance, intelligence, related content, status, and actions.

Editing modes:

- Record Edit: edit one structured item.
- Bulk Organize: move items between topics, archive duplicates, reclassify a group, update tags.
- Generate More: generate more content from a selected record, topic, source, or coverage gap.

Actions should include edit, save, move, merge, archive, delete with confirmation, regenerate specific parts, create variants, send to review, and publish where backend support exists.

Every save should leave a traceable event when the backend supports audit history: who changed it, when, and why or what changed.

## Sources & Ingestion

Sources & Ingestion handles raw material intake and extraction tracking.

It should include:

- Upload/register source area
- Source registry
- Source metadata
- Parse/extraction status
- Extracted record counts
- Failed extraction reasons
- Provenance links into Content Editor

Lifecycle:

Raw source -> extraction -> structured draft records -> review/edit -> publish/update active content

Raw files remain traceable. Extracted records become editable CMS content.

## Remote Updates

Remote Updates refresh the content pool from outside without requiring manual editing in Content Editor.

Remote updates should be treated as incoming batches, not invisible overwrites.

Flow:

Check remote -> preview batch -> apply to draft/review -> publish approved changes

Remote update screen should show:

- Remote source and version
- Added records
- Updated records
- Unchanged records
- Conflicts
- Failures
- Review-required items
- Apply update action
- Path into Review Queue and Content Editor for changed records

Remote update implementation should be honest. If backend sync/apply commands are not ready, the screen should show current status and future-ready structure without fake data.

## Review Queue

Review Queue gates trust for imported, generated, remote-updated, or heavily edited content.

It should support fast triage:

- Left/list pane: review items
- Right/detail pane: selected item, diff or details, provenance, confidence, and suggested classifications
- Always-visible approve, reject, edit, and reclassify actions

Bulk approval can exist only after filtering to low-risk review categories. Changed/generated content should remain traceable after approval.

## Seeding Engine

Seeding Engine creates draft structured content. It should not silently publish generated content.

Flow:

Choose target -> choose output -> set constraints -> generate draft batch -> inspect/edit -> review -> publish

Targets:

- Selected question
- Selected topic
- Source document
- Coverage gap
- Difficulty band
- Cognitive demand

Outputs:

- New questions
- Question variants
- Distractors
- Explanations
- Easier or harder versions
- Glossary entries
- Notes or remedial content

Seeding should be available from the dedicated workspace and contextually from Question Bank or Content Editor.

## Coverage & Stats

Coverage & Stats is the intelligence layer. It should show:

- Questions per subject/topic
- Answers per question
- Coverage by difficulty
- Coverage by cognitive demand
- Generated/manual/remote/source split
- Review status breakdown
- Stale or weak areas
- High-failure or high-confusion questions
- Duplicate or near-duplicate clusters

This screen should help admin decide what to seed, review, edit, archive, or update.

## Packs & Publishing

Packs & Publishing controls what powers the app.

It should show:

- Installed packs
- Pack versions
- Active/inactive status
- Update availability
- Content counts per pack
- Publish/unpublish controls
- Rollback or history when backend support exists

Publishing should make clear whether content is draft, reviewed, published, or active in the student app.

## Visual And UX Direction

The super-admin area should feel calm, precise, and powerful. It should feel like a real CMS, not a student dashboard or marketing page.

Guidelines:

- Light neutral background
- Near-black text
- Restrained teal for primary actions
- Amber for review/warnings
- Red only for destructive or failure states
- Green for approved/published states
- Subtle borders instead of heavy shadows
- Tables, lists, and split-pane workspaces for serious content operations
- Minimal decoration
- No playful student-style treatment
- No marketing hero
- No card-heavy clutter

Every admin screen should answer:

- What exists?
- What state is it in?
- What can I safely do next?
- Where did this content come from?
- Will this change affect the live student app?

## Implementation Passes

Pass 1: Information Architecture And CMS Shell

- Rework admin sidebar into CMS groups.
- Make Dashboard a CMS overview.
- Present Question Bank as inspection inventory.
- Add Content Editor route and shell.
- Add Remote Updates route and shell.
- Link Question Bank to Content Editor.

Pass 2: Content Editor MVP

- Build content type browser.
- Support question record editing using existing admin question commands.
- Support answer option editing.
- Support explanation editing.
- Support metadata and classification editing.
- Add provenance/intelligence side panel.
- Show review/publish/archive labels and actions according to backend support.

Pass 3: Advanced Content Operations

- Remote update preview/apply workflow.
- Rich ingestion detail view.
- Batch organize actions.
- Coverage-gap-driven seeding.
- Generated draft batch review.
- Publish/unpublish controls.
- Change history and versioning where backend supports it.

## Component Strategy

Create focused reusable admin CMS components rather than giant page files:

- AdminShellNav
- CmsMetricStrip
- CmsStatusBadge
- CmsActionQueue
- ContentRecordList
- ContentInspectorPanel
- ContentEditorWorkspace
- SourceStatusTimeline
- RemoteUpdatePreview
- ReviewTriagePanel

Each component should have one clear purpose and communicate through explicit props/events.

## Data Flow

Frontend admin screens should use `frontend/src/ipc/admin.ts` as the IPC boundary.

Existing backend support includes:

- Admin question stats
- Admin question list
- Admin question editor snapshot
- Admin question upsert
- Source registration
- Source registry
- Foundry jobs
- Review queue
- Question generation requests

Where backend support is missing, UI should show real current status and avoid fake data.

## Error Handling

Admin-facing errors should be plain-language and actionable:

- This source was registered, but extraction failed.
- This generated batch needs review before publishing.
- This question cannot be published because it has no correct answer.
- Remote update found conflicts. Review the changed records.

Failed ingestion, generation, sync, or publishing operations should leave visible status somewhere in the admin area.

## Testing And Verification

Implementation should be verified with:

- Frontend production build.
- Tauri cargo check.
- Existing admin question backend tests.
- New backend tests when new backend behavior is added.
- Manual smoke path: unlock admin -> dashboard -> question bank -> open content editor -> save question -> return to bank.

## First Implementation Recommendation

Start with Pass 1 plus the shell of Pass 2:

1. Rework admin navigation into CMS groups.
2. Rename the Questions screen presentation to Question Bank.
3. Add Content Editor top-level workspace.
4. Move heavy editing language/actions toward Content Editor.
5. Add Remote Updates workspace with honest current status.
6. Keep existing functional commands intact.
7. Make the admin area feel like one coherent CMS.
