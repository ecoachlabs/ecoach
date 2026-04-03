# Backend Layer Map

## Layer Ownership

| Layer | Owns | Representative crates |
| --- | --- | --- |
| persistence and protection | migrations, SQLite access, backup, recovery snapshot packaging | `ecoach-storage` |
| identity and access | accounts, PIN login, role and entitlement state | `ecoach-identity` |
| academic truth | curriculum graph, content ingestion, question intelligence | `ecoach-curriculum`, `ecoach-content`, `ecoach-questions` |
| learner truth | mastery, confidence, pressure, memory, misconceptions | `ecoach-student-model`, `ecoach-memory` |
| decision engines | diagnostics, coach logic, scheduling, reporting | `ecoach-diagnostics`, `ecoach-coach-brain`, `ecoach-goals-calendar`, `ecoach-reporting` |
| learning runtime | sessions, mocks, games, glossary, library | `ecoach-sessions`, `ecoach-mock-centre`, `ecoach-games`, `ecoach-glossary`, `ecoach-library` |
| integration boundary | DTOs and invoke-safe commands | `ecoach-commands`, `src-tauri` |

## Rule

If a behavior belongs in the backend, it should live in one of the domain crates and cross the UI boundary as a command, not as duplicated frontend logic.
