use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::{Connection, params};

pub struct PremiumService<'a> {
    conn: &'a Connection,
}

impl<'a> PremiumService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create_risk_flag(
        &self,
        student_id: i64,
        title: &str,
        severity: &str,
    ) -> EcoachResult<i64> {
        self.conn.execute(
            "INSERT INTO risk_flags (student_id, title, severity, status) VALUES (?1, ?2, ?3, 'active')",
            params![student_id, title, severity],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn create_intervention(
        &self,
        student_id: i64,
        title: &str,
        risk_flag_id: Option<i64>,
    ) -> EcoachResult<i64> {
        self.conn.execute(
            "INSERT INTO intervention_records (student_id, risk_flag_id, title) VALUES (?1, ?2, ?3)",
            params![student_id, risk_flag_id, title],
        ).map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(self.conn.last_insert_rowid())
    }
}
