# Screen Inventory

This is the rebuild-facing screen map for eCoach. Each screen exists to exercise a backend contract, not to invent new logic in the UI.

## Global Screen Rule

Every screen must define:
- empty state
- loading state
- error state
- success state
- primary commands it depends on

## Core Shell Screens

| Screen | Role | Purpose | Primary backend commands |
| --- | --- | --- | --- |
| Login / PIN | All | Authenticate locally and load the correct role shell | `create_account`, `login_with_pin`, `list_accounts` |
| Student shell | Student | Enter learning flows and show next actions | `get_coach_state`, `get_student_dashboard` |
| Parent shell | Parent | Show summaries, alerts, and readiness | reporting and premium command surfaces |
| Admin shell | Super Admin | Enter curriculum, content, oversight, and recovery operations | curriculum, content, question, recovery commands |

## Student Screens

| Screen | Purpose | Primary backend commands |
| --- | --- | --- |
| Student dashboard | Show today mission, route, readiness, and priorities | `get_student_dashboard`, `generate_today_mission`, `get_active_journey_route` |
| Diagnostic runner | Launch and progress adaptive assessment | `launch_diagnostic`, `list_diagnostic_phase_items`, `submit_diagnostic_attempt`, `complete_diagnostic_and_sync` |
| Session player | Run practice or coach sessions with evidence capture | `start_practice_session`, `start_coach_mission_session`, `submit_attempt`, `complete_session_with_pipeline` |
| Study path | Show route, milestones, and route edits | `build_or_refresh_journey_route`, `adapt_journey_route`, `get_goal_recommendation` |
| Memory recovery | Show due review, interventions, and knowledge state | `get_review_queue`, `get_memory_dashboard`, `complete_recheck` |
| Library and glossary | Browse saved items, concepts, notes, and test flows | library and glossary command surfaces |
| Games and traps | Run contrast drills and game-backed practice | game and traps command surfaces |
| Reports | Display readiness, progress, and history | diagnostic, coach, and reporting command surfaces |

## Parent Screens

| Screen | Purpose | Primary backend commands |
| --- | --- | --- |
| Parent overview | Show child status, risk, and trajectory | readiness, reporting, and premium summary surfaces |
| Child detail | Drill into one learner's evidence and next needs | learner truth, diagnostics, reporting surfaces |
| Alerts and interventions | Show important strategy changes | `list_parent_alerts`, `list_strategy_adjustments`, premium strategy surfaces |

## Admin Screens

| Screen | Purpose | Primary backend commands |
| --- | --- | --- |
| Curriculum manager | Create and manage academic structure | curriculum save and publish commands |
| Content foundry | Register sources, reconstruct, review, and publish | content intake and foundry commands |
| Question intelligence | Inspect families, lineage, and review actions | question intelligence commands |
| Student management | Manage accounts, roles, and entitlements | identity commands |
| Recovery and backup | Audit rebuild docs and protect runtime state | `inspect_rebuild_workspace`, `export_database_backup`, `check_database_backup_status`, `restore_database_backup`, `export_recovery_snapshot` |

## Screen-to-Docs Mapping

- Feature truth packs live in `docs/features/`
- Architecture references live in `docs/architecture/`
- Recovery procedures live in `docs/recovery/`
