use std::path::Path;

use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::Connection;

pub struct RuntimeDatabase {
    connection: Connection,
}

impl RuntimeDatabase {
    pub fn open(path: impl AsRef<Path>) -> EcoachResult<Self> {
        let connection =
            Connection::open(path).map_err(|err| EcoachError::Storage(err.to_string()))?;
        connection
            .execute_batch(
                "
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA foreign_keys = ON;
                PRAGMA temp_store = MEMORY;
                ",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(Self { connection })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}
