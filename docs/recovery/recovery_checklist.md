# Recovery Checklist

This is the operational checklist for the crash-protection path described in idea35.

## Runtime Recovery Commands

- `inspect_rebuild_workspace`
- `export_database_backup`
- `check_database_backup_status`
- `restore_database_backup`
- `export_recovery_snapshot`

## End-of-Slice Routine

1. Update the five root rebuild docs if the slice changed scope.
2. Update the relevant truth pack under `docs/features/`.
3. Run focused backend tests for the slice.
4. Export a database backup to a dated path.
5. Export a recovery snapshot zip that includes the runtime backup and workspace docs.
6. Commit the docs together with the code.

## Minimum Files That Must Stay Present

- `ECOACH_REBUILD_MASTER.md`
- `FEATURE_INVENTORY.md`
- `SCREEN_INVENTORY.md`
- `ARCHITECTURE.md`
- `REBUILD_ORDER.md`

## Failure Rule

If a backup path, workspace root, or snapshot path is blank, the command layer should reject it immediately. Recovery operations should fail loud, not silently.
