use std::str::FromStr;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{DateTime, Duration, Utc};
use ecoach_substrate::{AccountType, EcoachError, EcoachResult, EntitlementTier};
use rand_core::OsRng;
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::{Account, AccountSummary, CreateAccountInput};

pub struct IdentityService<'a> {
    conn: &'a Connection,
}

impl<'a> IdentityService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn create_account(&self, input: CreateAccountInput) -> EcoachResult<Account> {
        validate_pin(&input.account_type, &input.pin)?;
        let (pin_hash, pin_salt) = hash_pin(&input.pin)?;

        self.conn
            .execute(
                "INSERT INTO accounts (account_type, display_name, pin_hash, pin_salt, entitlement_tier)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    input.account_type.as_str(),
                    input.display_name,
                    pin_hash,
                    pin_salt,
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

fn validate_pin(account_type: &AccountType, pin: &str) -> EcoachResult<()> {
    if !pin.chars().all(|char| char.is_ascii_digit()) {
        return Err(EcoachError::Validation(
            "pin must contain only digits".to_string(),
        ));
    }
    if pin.len() < account_type.min_pin_len() || pin.len() > 12 {
        return Err(EcoachError::Validation(format!(
            "pin length for {:?} must be between {} and 12 digits",
            account_type,
            account_type.min_pin_len()
        )));
    }
    Ok(())
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
    value
        .and_then(|raw| DateTime::<Utc>::from_str(&raw).ok())
        .map(|dt| dt.with_timezone(&Utc))
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
}
