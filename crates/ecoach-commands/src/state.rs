use std::{path::Path, sync::Mutex};

use ecoach_storage::run_runtime_migrations;
use rusqlite::Connection;

use crate::error::CommandError;

pub struct AppState {
    conn: Mutex<Connection>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }

    pub fn open_runtime(path: &Path) -> Result<Self, CommandError> {
        let mut conn = Connection::open(path).map_err(|err| CommandError {
            code: "storage_error".to_string(),
            message: err.to_string(),
        })?;
        run_runtime_migrations(&mut conn)?;
        Ok(Self::new(conn))
    }

    pub fn in_memory() -> Result<Self, CommandError> {
        let mut conn = Connection::open_in_memory().map_err(|err| CommandError {
            code: "storage_error".to_string(),
            message: err.to_string(),
        })?;
        run_runtime_migrations(&mut conn)?;
        Ok(Self::new(conn))
    }

    pub fn with_connection<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, CommandError>,
    ) -> Result<T, CommandError> {
        let guard = self.conn.lock().map_err(|_| CommandError {
            code: "state_poisoned".to_string(),
            message: "app state connection lock was poisoned".to_string(),
        })?;
        operation(&guard)
    }
}
