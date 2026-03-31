use std::path::Path;

use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// Local backup and restore for the entire SQLite database.
pub struct BackupService;

impl BackupService {
    /// Export the entire database to a backup file using VACUUM INTO.
    pub fn export_backup(
        source_conn: &Connection,
        backup_path: &Path,
    ) -> EcoachResult<BackupResult> {
        // Remove existing backup if present
        if backup_path.exists() {
            std::fs::remove_file(backup_path)
                .map_err(|e| EcoachError::Storage(format!("cannot remove old backup: {e}")))?;
        }

        let path_str = backup_path.to_string_lossy();
        source_conn
            .execute_batch(&format!("VACUUM INTO '{}'", path_str))
            .map_err(|e| EcoachError::Storage(format!("backup failed: {e}")))?;

        let file_size = std::fs::metadata(backup_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        Ok(BackupResult {
            path: path_str.to_string(),
            size_bytes: file_size,
            success: true,
        })
    }

    /// Restore from a backup: copy backup DB over current DB path.
    /// Caller must close and reopen the connection after this.
    pub fn restore_backup(backup_path: &Path, db_path: &Path) -> EcoachResult<BackupResult> {
        if !backup_path.exists() {
            return Err(EcoachError::NotFound(format!(
                "backup file not found: {}",
                backup_path.display()
            )));
        }

        // Verify the backup is a valid ecoach database
        let source = Connection::open(backup_path)
            .map_err(|e| EcoachError::Storage(format!("cannot open backup: {e}")))?;
        let has_migrations: bool = source
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_migrations'",
                [],
                |row| Ok(row.get::<_, i64>(0)? > 0),
            )
            .unwrap_or(false);
        drop(source);

        if !has_migrations {
            return Err(EcoachError::Validation(
                "backup file does not appear to be a valid ecoach database".into(),
            ));
        }

        // Copy backup over current DB
        std::fs::copy(backup_path, db_path)
            .map_err(|e| EcoachError::Storage(format!("restore copy failed: {e}")))?;

        let file_size = std::fs::metadata(db_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        Ok(BackupResult {
            path: db_path.to_string_lossy().to_string(),
            size_bytes: file_size,
            success: true,
        })
    }

    /// Check backup status.
    pub fn check_backup_status(backup_path: &Path) -> BackupStatus {
        if !backup_path.exists() {
            return BackupStatus {
                exists: false,
                path: backup_path.to_string_lossy().to_string(),
                size_bytes: 0,
                last_modified: None,
            };
        }

        let metadata = std::fs::metadata(backup_path).ok();
        let size = metadata.as_ref().map(|m| m.len() as i64).unwrap_or(0);
        let modified = metadata
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let elapsed = t.elapsed().unwrap_or_default();
                format!("{} seconds ago", elapsed.as_secs())
            });

        BackupStatus {
            exists: true,
            path: backup_path.to_string_lossy().to_string(),
            size_bytes: size,
            last_modified: modified,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub path: String,
    pub size_bytes: i64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    pub exists: bool,
    pub path: String,
    pub size_bytes: i64,
    pub last_modified: Option<String>,
}
