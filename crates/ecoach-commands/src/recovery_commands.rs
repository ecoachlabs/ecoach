use std::{
    fs,
    path::{Path, PathBuf},
};

use ecoach_storage::{
    BackupResult, BackupService, BackupStatus, RebuildWorkspaceStatus, RecoverySnapshotResult,
    RecoverySnapshotService,
};
use serde::{Deserialize, Serialize};

use crate::{error::CommandError, state::AppState};

pub type DatabaseBackupResultDto = BackupResult;
pub type DatabaseBackupStatusDto = BackupStatus;
pub type RebuildWorkspaceStatusDto = RebuildWorkspaceStatus;
pub type RecoverySnapshotResultDto = RecoverySnapshotResult;
pub type ExportRecoverySnapshotInputDto = ExportRecoverySnapshotInput;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRecoverySnapshotInput {
    pub output_zip_path: String,
    pub workspace_root: Option<String>,
}

pub fn inspect_rebuild_workspace(
    workspace_root: String,
) -> Result<RebuildWorkspaceStatusDto, CommandError> {
    let workspace_root = normalize_non_empty_path(&workspace_root)?;
    RecoverySnapshotService::inspect_workspace(&workspace_root).map_err(Into::into)
}

pub fn export_database_backup(
    state: &AppState,
    output_path: String,
) -> Result<DatabaseBackupResultDto, CommandError> {
    let output_path = normalize_non_empty_path(&output_path)?;
    ensure_parent_directory(&output_path)?;
    state.with_connection(|conn| Ok(BackupService::export_backup(conn, &output_path)?))
}

pub fn check_database_backup_status(
    backup_path: String,
) -> Result<DatabaseBackupStatusDto, CommandError> {
    let backup_path = normalize_non_empty_path(&backup_path)?;
    Ok(BackupService::check_backup_status(&backup_path))
}

pub fn restore_database_backup(
    state: &AppState,
    backup_path: String,
) -> Result<DatabaseBackupResultDto, CommandError> {
    let backup_path = normalize_non_empty_path(&backup_path)?;
    state.restore_runtime_backup(&backup_path)
}

pub fn export_recovery_snapshot(
    state: &AppState,
    input: ExportRecoverySnapshotInput,
) -> Result<RecoverySnapshotResultDto, CommandError> {
    let output_zip_path = normalize_non_empty_path(&input.output_zip_path)?;
    let workspace_root = input
        .workspace_root
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    state.with_connection(|conn| {
        Ok(RecoverySnapshotService::export_snapshot(
            conn,
            &output_zip_path,
            workspace_root.as_deref(),
        )?)
    })
}

fn normalize_non_empty_path(path: &str) -> Result<PathBuf, CommandError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(CommandError {
            code: "validation_error".to_string(),
            message: "path cannot be empty".to_string(),
        });
    }
    Ok(PathBuf::from(trimmed))
}

fn ensure_parent_directory(path: &Path) -> Result<(), CommandError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|err| CommandError {
            code: "storage_error".to_string(),
            message: format!("failed to create parent directory: {err}"),
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process,
    };

    use chrono::Utc;
    use ecoach_identity::CreateAccountInput;
    use ecoach_substrate::{AccountType, EntitlementTier};

    use super::{
        ExportRecoverySnapshotInput, check_database_backup_status, export_database_backup,
        export_recovery_snapshot, inspect_rebuild_workspace, restore_database_backup,
    };
    use crate::{CommandError, identity_commands, state::AppState};

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "ecoach-recovery-command-{label}-{}-{}",
                process::id(),
                Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ));
            fs::create_dir_all(&path).expect("temp directory should create");
            Self { path }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn recovery_commands_surface_backup_and_snapshot_exports() {
        let state = AppState::in_memory().expect("in-memory state should build");
        let temp = TempDirGuard::new("surface");

        for relative_path in [
            "ECOACH_REBUILD_MASTER.md",
            "FEATURE_INVENTORY.md",
            "SCREEN_INVENTORY.md",
            "ARCHITECTURE.md",
            "REBUILD_ORDER.md",
            "features/features_ideas_32_to_38.md",
            "implementation/01_architecture_and_design_system.md",
            "backend notes/ecoach_backend_implementation_plan_canonical_v1.md",
            "ideas/idea35.txt",
        ] {
            write_file(
                &temp.path,
                relative_path,
                &format!("content for {relative_path}"),
            );
        }

        let status = inspect_rebuild_workspace(temp.path.to_string_lossy().to_string())
            .expect("workspace status should build");
        assert!(status.missing_required_documents.is_empty());
        assert!(status.feature_doc_count > 0);

        let backup_path = temp.path.join("runtime-backup.sqlite3");
        let backup = export_database_backup(&state, backup_path.to_string_lossy().to_string())
            .expect("backup export should succeed");
        assert!(backup.success);

        let backup_status = check_database_backup_status(backup_path.to_string_lossy().to_string())
            .expect("backup status should load");
        assert!(backup_status.exists);
        assert!(backup_status.size_bytes > 0);

        let snapshot_path = temp.path.join("snapshots").join("idea35-recovery.zip");
        let snapshot = export_recovery_snapshot(
            &state,
            ExportRecoverySnapshotInput {
                output_zip_path: snapshot_path.to_string_lossy().to_string(),
                workspace_root: Some(temp.path.to_string_lossy().to_string()),
            },
        )
        .expect("recovery snapshot should export");

        assert!(snapshot.missing_required_documents.is_empty());
        assert!(
            snapshot
                .included_entries
                .iter()
                .any(|entry| entry == "database/ecoach-runtime.sqlite3")
        );
        assert!(snapshot_path.exists());
    }

    #[test]
    fn recovery_commands_reject_blank_paths() {
        let state = AppState::in_memory().expect("in-memory state should build");

        let backup_error =
            export_database_backup(&state, "   ".to_string()).expect_err("blank path should fail");
        assert_eq!(backup_error.code, "validation_error");

        let workspace_error =
            inspect_rebuild_workspace("   ".to_string()).expect_err("blank root should fail");
        assert_eq!(workspace_error.code, "validation_error");

        let snapshot_error = export_recovery_snapshot(
            &state,
            ExportRecoverySnapshotInput {
                output_zip_path: "   ".to_string(),
                workspace_root: None,
            },
        )
        .expect_err("blank snapshot path should fail");
        assert_eq!(snapshot_error.code, "validation_error");
    }

    #[test]
    fn restore_database_backup_reopens_file_backed_runtime() {
        let temp = TempDirGuard::new("restore");
        let db_path = temp.path.join("runtime.sqlite3");
        let state = AppState::open_runtime(&db_path).expect("file-backed state should build");

        let backup_path = temp.path.join("backups").join("runtime-backup.sqlite3");
        export_database_backup(&state, backup_path.to_string_lossy().to_string())
            .expect("backup export should succeed");

        identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Temp User".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("mutating runtime database should succeed");

        restore_database_backup(&state, backup_path.to_string_lossy().to_string())
            .expect("restore should succeed");

        let account_count = state
            .with_connection(|conn| {
                conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| {
                    row.get::<_, i64>(0)
                })
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })
            })
            .expect("account count should load");

        assert_eq!(account_count, 0);
    }

    fn write_file(root: &Path, relative_path: &str, contents: &str) {
        let full_path = root.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("parent directory should create");
        }
        fs::write(full_path, contents).expect("test file should write");
    }
}
