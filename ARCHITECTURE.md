# Architecture

This document captures the current rebuild architecture for eCoach after the idea35 audit.

## Stack

- Backend: Rust workspace crates
- Storage: SQLite with runtime migrations
- Desktop shell: Tauri
- Frontend boundary: typed Tauri commands

## Layer Map

| Layer | Responsibility | Main locations |
| --- | --- | --- |
| storage | migrations, runtime connection, backup, recovery snapshot packaging | `crates/ecoach-storage` |
| domain services | curriculum, questions, diagnostics, coaching, sessions, memory, reporting, library, glossary, games, intake, premium | `crates/ecoach-*` |
| command boundary | DTO shaping, validation, frontend-safe invoke surface | `crates/ecoach-commands` |
| desktop entry | state creation and command registration | `src-tauri/src/main.rs`, `src-tauri/src/commands.rs` |
| documentation spine | rebuild truth packs, architecture notes, recovery procedures, decisions | root docs plus `docs/` |

## Runtime Ownership Rules

The backend owns:
- mastery and learner truth
- question intelligence and selection
- diagnostic scoring and recommendations
- memory and decay calculations
- session evidence and completion state
- goal and schedule orchestration
- reporting summaries
- recovery audit and snapshot generation

The frontend owns:
- screen composition
- layout and interaction flow
- rendering returned DTOs
- command invocation timing

## Command Boundary Rules

The frontend should never implement parallel business logic for:
- readiness
- mastery
- memory
- diagnostic cause analysis
- plan generation
- backup state inspection

If the UI needs a value, the command layer should expose it from the backend instead of recalculating it.

## Recovery Architecture

The rebuild protection path is now:
1. Audit workspace docs with `inspect_rebuild_workspace`.
2. Export a point-in-time runtime backup with `export_database_backup`.
3. Verify a backup path with `check_database_backup_status`.
4. Restore the live runtime from a backup with `restore_database_backup`.
5. Export a zipped recovery package with `export_recovery_snapshot`.

## Documentation Structure

The repo should maintain both legacy planning folders and the idea35 documentation shape:
- `features/`
- `implementation/`
- `backend notes/`
- `docs/features/`
- `docs/recovery/`
- `docs/architecture/`
- `docs/ui/`
- `docs/decisions/`

## Reference Docs

- [ECOACH_REBUILD_MASTER.md](ECOACH_REBUILD_MASTER.md)
- [REBUILD_ORDER.md](REBUILD_ORDER.md)
- `docs/architecture/backend-layer-map.md`
- `docs/decisions/0001-keep-rust-for-core-backend.md`
