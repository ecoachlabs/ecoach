# Feature Inventory

This inventory is backend-focused and aligned to `ideas/idea35.txt`.

## Tier 1: Must Ship

| Feature | Why it exists | Roles | Key inputs | Key outputs | Backend areas | Truth pack | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Core platform and identity | Establishes local access and role-aware shells | Student, Parent, Super Admin | account data, PINs, entitlement | account DTOs, role gating | `identity`, `commands`, `src-tauri` | `docs/features/core-platform-foundation.md` | Present |
| Super admin foundation | Gives the product a controlled publishing and oversight layer | Super Admin | user changes, configuration, curriculum edits | managed runtime state | `identity`, `curriculum`, `content`, `reporting` | `docs/features/super-admin-foundation.md` | Present |
| Curriculum ingestion and intelligence | Creates canonical academic structure and source provenance | Super Admin, Student | uploads, parse candidates, curriculum edits | published curriculum graph | `curriculum`, `content`, `storage` | `docs/features/curriculum-ingestion-and-intelligence.md` | Present |
| Question bank and learning content | Fuels diagnostics, sessions, drills, and remediation | Student, Super Admin | pack content, generation requests, review actions | question intelligence and retrieval results | `questions`, `content`, `diagnostics` | `docs/features/question-bank-and-content-layer.md` | Present |
| Student and identity module | Tracks who the learner is and what state they are in | Student, Parent | attempts, goals, role-linked access | learner truth and dashboard state | `identity`, `student-model`, `commands` | `docs/features/student-and-identity.md` | Present |
| DNA and diagnostics | Detects weaknesses, causes, and recommended actions | Student, Parent, Super Admin | diagnostic launch parameters, item routing, attempts | results, recommendations, audience reports | `diagnostics`, `coach-brain` | `docs/features/dna-diagnostic.md` | Present |
| Study plan and journey | Converts evidence into guided next steps | Student, Parent | learner truth, exams, time, performance | missions, routes, goal recommendations | `coach-brain`, `goals-calendar` | `docs/features/study-plan-and-journey.md` | Present |
| Session runtime | Runs practice, coach, and mock-time flows | Student | session start inputs, attempts, presence events | completion snapshots and evidence | `sessions`, `student-model`, `commands` | `docs/features/session-runtime.md` | Present |
| Analytics and reporting | Makes state visible to students, parents, and admin | Parent, Super Admin, Student | synced performance and readiness data | dashboards, summaries, alerts | `reporting`, `premium`, `diagnostics` | `docs/features/analytics-and-reporting.md` | Present |
| Memory, decay, and retention | Keeps knowledge alive between sessions | Student | retrieval attempts, decay scans, interventions | review queues and knowledge-state updates | `memory`, `student-model`, `coach-brain` | `docs/features/memory-decay-and-retention.md` | Present |
| Backup, restore, and recovery snapshots | Prevents a repeat of the original crash-loss scenario | Super Admin | workspace root, backup path, snapshot path | workspace audits, backup files, restored runtime, recovery zips | `storage`, `commands`, `src-tauri` | `docs/recovery/recovery_checklist.md` | Present |

## Tier 2: Strong Differentiators

| Feature | Why it exists | Roles | Backend areas | Status |
| --- | --- | --- | --- | --- |
| Library intelligence | Organizes revision assets and saved learning context | Student | `library`, `glossary` | Present |
| Glossary lab | Teaches concepts with relationships, comparison, and audio | Student | `glossary`, `content` | Present |
| Games and traps | Drives contrast learning and deliberate confusion repair | Student | `games`, `questions` | Present |
| Goal and calendar orchestration | Shapes prep intensity around real deadlines and availability | Student, Parent | `goals-calendar`, `coach-brain` | Present |
| Intake and reconstruction | Converts uploaded materials into usable evidence | Super Admin, Student | `intake`, `content` | Present |

## Cross-Document References

- Recovery blueprint: [ECOACH_REBUILD_MASTER.md](ECOACH_REBUILD_MASTER.md)
- Architecture map: [ARCHITECTURE.md](ARCHITECTURE.md)
- Slice order: [REBUILD_ORDER.md](REBUILD_ORDER.md)
- Screen contract map: [SCREEN_INVENTORY.md](SCREEN_INVENTORY.md)

## Inventory Rule

If a feature has schema and service logic but no command boundary, it is partial. If it has command exposure but no truth pack, it is undocumented. The rebuild is complete only when both exist.
