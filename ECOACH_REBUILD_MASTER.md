# eCoach Rebuild Master

This is the idea35 source-of-truth document for rebuilding eCoach without depending on memory.

## Product Vision

eCoach is a local-first academic intelligence desktop application. It uses a Rust backend, SQLite runtime storage, and a Tauri command boundary to drive curriculum management, diagnostics, coaching, sessions, memory recovery, reporting, and recovery operations.

The product must be reconstructed from:
- `ideas/`
- surviving backend crates and migrations
- `features/`
- `implementation/`
- `backend notes/`
- screenshots and future UI captures

## Roles

| Role | What they see | What they do | What they do not own |
| --- | --- | --- | --- |
| Student | Dashboard, diagnostics, sessions, memory, library, reports | Learn, answer, review, follow journeys, run drills | Curriculum publishing and system recovery |
| Parent | Child progress, alerts, strategy summaries, reports | Monitor risk, read summaries, respond to interventions | Content authoring and runtime administration |
| Super Admin | Curriculum, content, question intelligence, user oversight, recovery tooling | Publish structure, inspect runtime, back up and restore, audit flows | Student-side daily learning |

## Core Product Slices

| Slice | Current backend anchor |
| --- | --- |
| Core platform and identity | `ecoach-identity`, `ecoach-commands`, `src-tauri` |
| Curriculum and academic spine | `ecoach-curriculum`, `ecoach-content` |
| Question intelligence | `ecoach-questions`, `ecoach-content` |
| Student truth and memory | `ecoach-student-model`, `ecoach-memory` |
| Diagnostics and DNA | `ecoach-diagnostics`, `ecoach-coach-brain` |
| Coaching and journey | `ecoach-coach-brain`, `ecoach-goals-calendar` |
| Session runtime | `ecoach-sessions` |
| Library and glossary | `ecoach-library`, `ecoach-glossary` |
| Reporting and parent surfaces | `ecoach-reporting`, `ecoach-premium` |
| Recovery and rebuild safety | `ecoach-storage`, `ecoach-commands`, `src-tauri` |

## Domain Model

The rebuild should continue to treat these as first-class backend entities:
- account, role, entitlement
- curriculum family, version, subject, topic, objective, node intelligence
- content source, parse candidate, foundry job, resource intelligence
- question, question family, question intelligence profile, review action
- learner truth, topic state, confidence, pressure, memory, misconception state
- diagnostic run, phase item, skill result, recommendation, audience report
- journey route, mission, intervention, schedule event, reminder
- session, attempt, focus mode, presence event, completion pipeline
- library item, glossary entry, knowledge link, shelf, note, test session
- recovery snapshot, runtime backup, rebuild workspace status

## Major Workflows

1. Identity and role entry
   - Account creation or PIN login opens the correct app shell.
2. Curriculum ingestion and publication
   - Admin registers sources, reconstructs content, reviews, and publishes.
3. Diagnostic assessment
   - Student launches a battery, submits attempts, completes a phase, and receives synchronized analytics.
4. Guided learning
   - Coach computes next action, journey route, session composition, and memory follow-up.
5. Session completion cascade
   - Session evidence updates learner truth, memory, readiness, and recommendations.
6. Recovery protection
   - Admin inspects rebuild documents, exports a database backup, restores the runtime, and exports a recovery snapshot zip.

## Technical Architecture

- Backend language remains Rust.
- Local persistence remains SQLite.
- Frontend talks to the backend only through Tauri commands.
- Domain logic stays in workspace crates, not in frontend state.
- Recovery documentation is committed alongside code.

## Required Rebuild Artifacts

The rebuild stays healthy only if these remain current:
- [FEATURE_INVENTORY.md](FEATURE_INVENTORY.md)
- [SCREEN_INVENTORY.md](SCREEN_INVENTORY.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [REBUILD_ORDER.md](REBUILD_ORDER.md)
- `docs/features/*.md` truth packs
- `docs/recovery/*.md`
- `docs/architecture/*.md`
- `docs/decisions/*.md`

## Protection Rule

Do not treat a slice as done until backend logic, command exposure, tests, docs, and a recovery artifact all exist.
