use std::str::FromStr;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use ecoach_substrate::{
    ACCOUNT_PIN_LENGTH, AccountType, EcoachError, EcoachResult, EntitlementTier,
};
use rand_core::OsRng;
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{
    Account, AccountSummary, CreateAccountInput, EntitlementAuditEntry, EntitlementEvent,
    UpdateAccountAccessInput, UpdateEntitlementInput,
};

pub struct IdentityService<'a> {
    conn: &'a Connection,
}

impl<'a> IdentityService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create_account(&self, input: CreateAccountInput) -> EcoachResult<Account> {
        validate_pin(&input.pin)?;
        let (pin_hash, pin_salt) = hash_pin(&input.pin)?;
        let pin_length = input.pin.len() as i64;

        self.conn
            .execute(
                "INSERT INTO accounts (
                    account_type, display_name, pin_hash, pin_salt, pin_length, entitlement_tier
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    input.account_type.as_str(),
                    input.display_name,
                    pin_hash,
                    pin_salt,
                    pin_length,
                    input.entitlement_tier.as_str()
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let id = self.conn.last_insert_rowid();
        self.get_account(id)?
            .ok_or_else(|| EcoachError::NotFound(format!("account {} was not created", id)))
    }

    pub fn authenticate(&self, account_id: i64, pin: &str) -> EcoachResult<Account> {
        let account = self
            .get_account_row(account_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("account {} not found", account_id)))?;

        if let Some(locked_until) = account.locked_until {
            if locked_until > Utc::now() {
                return Err(EcoachError::Unauthorized(
                    "account temporarily locked".to_string(),
                ));
            }
        }

        validate_pin(pin)?;

        if verify_pin(&account.pin_hash, pin)? {
            self.conn
                .execute(
                    "UPDATE accounts SET failed_pin_attempts = 0, locked_until = NULL, last_active_at = ?1 WHERE id = ?2",
                    params![Utc::now().to_rfc3339(), account_id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;

            return self.get_account(account_id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("account {} disappeared", account_id))
            });
        }

        let next_failed_attempts = account.failed_pin_attempts + 1;
        let locked_until = if next_failed_attempts >= 5 {
            Some((Utc::now() + Duration::minutes(5)).to_rfc3339())
        } else {
            None
        };

        self.conn
            .execute(
                "UPDATE accounts SET failed_pin_attempts = ?1, locked_until = COALESCE(?2, locked_until) WHERE id = ?3",
                params![next_failed_attempts, locked_until, account_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        Err(EcoachError::Unauthorized("invalid pin".to_string()))
    }

    pub fn list_accounts(&self) -> EcoachResult<Vec<AccountSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, account_type, display_name, status, first_run, last_active_at
                 FROM accounts
                 WHERE status != 'archived'
                 ORDER BY last_active_at DESC NULLS LAST, created_at DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([], |row| {
                Ok(AccountSummary {
                    id: row.get(0)?,
                    account_type: parse_account_type(row.get::<_, String>(1)?)?,
                    display_name: row.get(2)?,
                    status: row.get(3)?,
                    first_run: row.get::<_, i64>(4)? == 1,
                    last_active_at: parse_datetime(row.get::<_, Option<String>>(5)?),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(accounts)
    }

    pub fn link_parent_student(&self, parent_id: i64, student_id: i64) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO parent_student_links (parent_account_id, student_account_id) VALUES (?1, ?2)",
                params![parent_id, student_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    pub fn get_linked_students(&self, parent_id: i64) -> EcoachResult<Vec<AccountSummary>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT a.id, a.account_type, a.display_name, a.status, a.first_run, a.last_active_at
                 FROM parent_student_links p
                 JOIN accounts a ON a.id = p.student_account_id
                 WHERE p.parent_account_id = ?1
                 ORDER BY a.display_name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([parent_id], |row| {
                Ok(AccountSummary {
                    id: row.get(0)?,
                    account_type: parse_account_type(row.get::<_, String>(1)?)?,
                    display_name: row.get(2)?,
                    status: row.get(3)?,
                    first_run: row.get::<_, i64>(4)? == 1,
                    last_active_at: parse_datetime(row.get::<_, Option<String>>(5)?),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut students = Vec::new();
        for row in rows {
            students.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(students)
    }

    pub fn update_account_access(
        &self,
        account_id: i64,
        input: UpdateAccountAccessInput,
    ) -> EcoachResult<Account> {
        let current = self
            .get_account(account_id)?
            .ok_or_else(|| EcoachError::NotFound(format!("account {} not found", account_id)))?;
        let next_status = input.status.unwrap_or_else(|| current.status.clone());
        validate_account_status(&next_status)?;
        if current.entitlement_tier == input.entitlement_tier && current.status == next_status {
            return Ok(current);
        }

        self.conn
            .execute(
                "UPDATE accounts
                 SET entitlement_tier = ?1,
                     status = ?2,
                     updated_at = datetime('now')
                 WHERE id = ?3",
                params![input.entitlement_tier.as_str(), next_status, account_id],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.conn
            .execute(
                "INSERT INTO account_entitlement_events (
                    account_id, previous_tier, new_tier, changed_by_account_id,
                    note, previous_status, new_status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    account_id,
                    current.entitlement_tier.as_str(),
                    input.entitlement_tier.as_str(),
                    input.changed_by_account_id,
                    input.reason,
                    current.status,
                    next_status,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        self.get_account(account_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!(
                "account {} disappeared after access update",
                account_id
            ))
        })
    }

    pub fn update_entitlement(
        &self,
        account_id: i64,
        input: UpdateEntitlementInput,
    ) -> EcoachResult<Account> {
        self.update_account_access(
            account_id,
            UpdateAccountAccessInput {
                entitlement_tier: input.entitlement_tier,
                status: None,
                changed_by_account_id: input.changed_by_account_id,
                reason: input.note,
            },
        )
    }

    pub fn list_entitlement_audit_entries(
        &self,
        limit: usize,
    ) -> EcoachResult<Vec<EntitlementAuditEntry>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, account_id, changed_by_account_id, previous_tier, new_tier,
                        previous_status, new_status, note, created_at
                 FROM account_entitlement_events
                 ORDER BY id DESC
                 LIMIT ?1",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![limit.max(1) as i64], |row| {
                Ok(EntitlementAuditEntry {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    changed_by_account_id: row.get(2)?,
                    previous_tier: row.get(3)?,
                    new_tier: row.get(4)?,
                    previous_status: row.get(5)?,
                    new_status: row.get(6)?,
                    reason: row.get(7)?,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(entries)
    }

    pub fn list_entitlement_events(
        &self,
        account_id: i64,
        limit: usize,
    ) -> EcoachResult<Vec<EntitlementEvent>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, account_id, previous_tier, new_tier, changed_by_account_id, note, created_at
                 FROM account_entitlement_events
                 WHERE account_id = ?1
                 ORDER BY id DESC
                 LIMIT ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![account_id, limit.max(1) as i64], |row| {
                let created_at_raw: String = row.get(6)?;
                Ok(EntitlementEvent {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    previous_tier: parse_entitlement(row.get::<_, String>(2)?)?,
                    new_tier: parse_entitlement(row.get::<_, String>(3)?)?,
                    changed_by_account_id: row.get(4)?,
                    note: row.get(5)?,
                    created_at: parse_required_datetime_column(6, &created_at_raw)?,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(events)
    }

    /// Reset a learner's PIN. Only callable by parent/admin.
    pub fn reset_pin(&self, account_id: i64, new_pin: &str) -> EcoachResult<()> {
        let exists: i64 = self
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = ?1)",
                [account_id],
                |row| row.get(0),
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;
        if exists == 0 {
            return Err(EcoachError::NotFound(format!(
                "account {} not found",
                account_id
            )));
        }

        validate_pin(new_pin)?;
        let (pin_hash, pin_salt) = hash_pin(new_pin)?;
        let pin_length = new_pin.len() as i64;

        self.conn
            .execute(
                "UPDATE accounts SET pin_hash = ?1, pin_salt = ?2, pin_length = ?3, failed_pin_attempts = 0,
                     locked_until = NULL, updated_at = datetime('now')
                 WHERE id = ?4",
                params![pin_hash, pin_salt, pin_length, account_id],
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        Ok(())
    }

    fn get_account(&self, account_id: i64) -> EcoachResult<Option<Account>> {
        self.get_account_row(account_id).map(|row| {
            row.map(|raw| Account {
                id: raw.id,
                account_type: raw.account_type,
                display_name: raw.display_name,
                entitlement_tier: raw.entitlement_tier,
                failed_pin_attempts: raw.failed_pin_attempts,
                locked_until: raw.locked_until,
                status: raw.status,
                first_run: raw.first_run,
                last_active_at: raw.last_active_at,
            })
        })
    }

    fn get_account_row(&self, account_id: i64) -> EcoachResult<Option<RawAccount>> {
        self.conn
            .query_row(
                "SELECT id, account_type, display_name, pin_hash, pin_salt, entitlement_tier,
                        failed_pin_attempts, locked_until, status, first_run, last_active_at
                 FROM accounts WHERE id = ?1",
                [account_id],
                |row| {
                    Ok(RawAccount {
                        id: row.get(0)?,
                        account_type: parse_account_type(row.get::<_, String>(1)?)?,
                        display_name: row.get(2)?,
                        pin_hash: row.get(3)?,
                        entitlement_tier: parse_entitlement(row.get::<_, String>(5)?)?,
                        failed_pin_attempts: row.get(6)?,
                        locked_until: parse_datetime(row.get::<_, Option<String>>(7)?),
                        status: row.get(8)?,
                        first_run: row.get::<_, i64>(9)? == 1,
                        last_active_at: parse_datetime(row.get::<_, Option<String>>(10)?),
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }
}

struct RawAccount {
    id: i64,
    account_type: AccountType,
    display_name: String,
    pin_hash: String,
    entitlement_tier: EntitlementTier,
    failed_pin_attempts: i64,
    locked_until: Option<DateTime<Utc>>,
    status: String,
    first_run: bool,
    last_active_at: Option<DateTime<Utc>>,
}

fn validate_pin(pin: &str) -> EcoachResult<()> {
    if !pin.chars().all(|char| char.is_ascii_digit()) {
        return Err(EcoachError::Validation(
            "pin must contain only digits".to_string(),
        ));
    }
    if pin.len() != ACCOUNT_PIN_LENGTH {
        return Err(EcoachError::Validation(format!(
            "pin must be exactly {} digits",
            ACCOUNT_PIN_LENGTH
        )));
    }
    Ok(())
}

fn validate_account_status(status: &str) -> EcoachResult<()> {
    if matches!(status, "active" | "inactive" | "archived") {
        return Ok(());
    }
    Err(EcoachError::Validation(format!(
        "unknown account status: {}",
        status
    )))
}

fn hash_pin(pin: &str) -> EcoachResult<(String, String)> {
    let salt = SaltString::generate(&mut OsRng);
    let pin_hash = Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|err| EcoachError::Storage(err.to_string()))?
        .to_string();
    Ok((pin_hash, salt.to_string()))
}

fn verify_pin(stored_hash: &str, pin: &str) -> EcoachResult<bool> {
    let parsed_hash =
        PasswordHash::new(stored_hash).map_err(|err| EcoachError::Storage(err.to_string()))?;
    Ok(Argon2::default()
        .verify_password(pin.as_bytes(), &parsed_hash)
        .is_ok())
}

fn parse_account_type(value: String) -> rusqlite::Result<AccountType> {
    match value.as_str() {
        "student" => Ok(AccountType::Student),
        "parent" => Ok(AccountType::Parent),
        "admin" => Ok(AccountType::Admin),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(format!(
                "unknown account type: {}",
                other
            ))),
        )),
    }
}

fn parse_entitlement(value: String) -> rusqlite::Result<EntitlementTier> {
    match value.as_str() {
        "standard" => Ok(EntitlementTier::Standard),
        "premium" => Ok(EntitlementTier::Premium),
        "elite" => Ok(EntitlementTier::Elite),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(format!(
                "unknown entitlement tier: {}",
                other
            ))),
        )),
    }
}

fn parse_datetime(value: Option<String>) -> Option<DateTime<Utc>> {
    value.and_then(|raw| parse_datetime_value(&raw))
}

fn parse_datetime_value(value: &str) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_str(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        })
}

fn parse_required_datetime_column(
    index: usize,
    value: &str,
) -> rusqlite::Result<DateTime<Utc>> {
    parse_datetime_value(value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(format!(
                "invalid datetime value: {}",
                value
            ))),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::Connection;

    #[test]
    fn create_and_authenticate_account() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        run_runtime_migrations(&mut conn).expect("migrations");

        let service = IdentityService::new(&conn);
        let account = service
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Ama".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("account should be created");

        let authenticated = service
            .authenticate(account.id, "1234")
            .expect("pin should authenticate");
        assert_eq!(authenticated.id, account.id);
        assert_eq!(authenticated.display_name, "Ama");
    }

    #[test]
    fn update_entitlement_records_audit_event() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        run_runtime_migrations(&mut conn).expect("migrations");

        let service = IdentityService::new(&conn);
        let student = service
            .create_account(CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Kojo".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            })
            .expect("student should be created");
        let admin = service
            .create_account(CreateAccountInput {
                account_type: AccountType::Admin,
                display_name: "Control".to_string(),
                pin: "2468".to_string(),
                entitlement_tier: EntitlementTier::Elite,
            })
            .expect("admin should be created");

        let updated = service
            .update_entitlement(
                student.id,
                UpdateEntitlementInput {
                    entitlement_tier: EntitlementTier::Elite,
                    changed_by_account_id: Some(admin.id),
                    note: Some("Manual elite activation".to_string()),
                },
            )
            .expect("entitlement update should succeed");
        let events = service
            .list_entitlement_events(student.id, 10)
            .expect("events should load");

        assert_eq!(updated.entitlement_tier, EntitlementTier::Elite);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].previous_tier, EntitlementTier::Standard);
        assert_eq!(events[0].new_tier, EntitlementTier::Elite);
        assert_eq!(events[0].changed_by_account_id, Some(admin.id));
    }

    #[test]
    fn all_accounts_require_exactly_four_digit_pins() {
        let mut conn = Connection::open_in_memory().expect("in-memory db");
        run_runtime_migrations(&mut conn).expect("migrations");

        let service = IdentityService::new(&conn);
        let result = service.create_account(CreateAccountInput {
            account_type: AccountType::Parent,
            display_name: "Guardian".to_string(),
            pin: "18035".to_string(),
            entitlement_tier: EntitlementTier::Standard,
        });

        assert!(result.is_err());
    }
}
