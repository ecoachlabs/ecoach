use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, params};

pub struct GamesService<'a> {
    conn: &'a Connection,
}

impl<'a> GamesService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn start_game_session(&self, student_id: i64, game_type: &str) -> EcoachResult<i64> {
        self.conn.execute(
            "INSERT INTO game_sessions (student_id, game_type, session_state) VALUES (?1, ?2, 'active')",
            params![student_id, game_type],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }
}
