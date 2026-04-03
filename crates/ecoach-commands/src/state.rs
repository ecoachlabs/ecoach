use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use ecoach_storage::{BackupResult, BackupService, run_runtime_migrations};
use rusqlite::Connection;

use crate::error::CommandError;

pub struct AppState {
    conn: Mutex<Option<Connection>>,
    db_path: Option<PathBuf>,
}

impl AppState {
    pub fn new(conn: Connection, db_path: Option<PathBuf>) -> Self {
        Self {
            conn: Mutex::new(Some(conn)),
            db_path,
        }
    }

    pub fn open_runtime(path: &Path) -> Result<Self, CommandError> {
        let mut conn = Connection::open(path).map_err(|err| CommandError {
            code: "storage_error".to_string(),
            message: err.to_string(),
        })?;
        run_runtime_migrations(&mut conn)?;
        Ok(Self::new(conn, Some(path.to_path_buf())))
    }

    pub fn in_memory() -> Result<Self, CommandError> {
        let mut conn = Connection::open_in_memory().map_err(|err| CommandError {
            code: "storage_error".to_string(),
            message: err.to_string(),
        })?;
        run_runtime_migrations(&mut conn)?;
        Ok(Self::new(conn, None))
    }

    pub fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, CommandError>,
    ) -> Result<T, CommandError> {
        let guard = self.conn.lock().map_err(|_| CommandError {
            code: "state_poisoned".to_string(),
            message: "app state connection lock was poisoned".to_string(),
        })?;
        let conn = guard.as_ref().ok_or(CommandError {
            code: "state_unavailable".to_string(),
            message: "runtime database connection is temporarily unavailable".to_string(),
        })?;
        operation(conn)
    }

    pub fn restore_runtime_backup(&self, backup_path: &Path) -> Result<BackupResult, CommandError> {
        let db_path = self.db_path.as_ref().cloned().ok_or(CommandError {
            code: "unsupported".to_string(),
            message: "runtime backup restore is only available for file-backed databases"
                .to_string(),
        })?;

        let mut guard = self.conn.lock().map_err(|_| CommandError {
            code: "state_poisoned".to_string(),
            message: "app state connection lock was poisoned".to_string(),
        })?;

        let current_conn = guard.take().ok_or(CommandError {
            code: "state_unavailable".to_string(),
            message: "runtime database connection is temporarily unavailable".to_string(),
        })?;
        drop(current_conn);

        let restore_result =
            BackupService::restore_backup(backup_path, &db_path).map_err(CommandError::from);

        let mut reopened = Connection::open(&db_path).map_err(|err| CommandError {
            code: "storage_error".to_string(),
            message: format!("failed to reopen runtime database after restore: {err}"),
        })?;
        run_runtime_migrations(&mut reopened)?;
        *guard = Some(reopened);

        restore_result
    }
}
