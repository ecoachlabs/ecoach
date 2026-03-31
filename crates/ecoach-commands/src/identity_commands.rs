use ecoach_identity::{CreateAccountInput, IdentityService};

use crate::{
    dtos::{AccountDto, AccountSummaryDto},
    error::CommandError,
    state::AppState,
};

pub fn create_account(
    state: &AppState,
    input: CreateAccountInput,
) -> Result<AccountDto, CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        let account = service.create_account(input)?;
        Ok(AccountDto::from(account))
    })
}

pub fn login_with_pin(
    state: &AppState,
    account_id: i64,
    pin: String,
) -> Result<AccountDto, CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        let account = service.authenticate(account_id, &pin)?;
        Ok(AccountDto::from(account))
    })
}

pub fn list_accounts(state: &AppState) -> Result<Vec<AccountSummaryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        let accounts = service.list_accounts()?;
        Ok(accounts.into_iter().map(AccountSummaryDto::from).collect())
    })
}

pub fn reset_pin(
    state: &AppState,
    account_id: i64,
    new_pin: String,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        service.reset_pin(account_id, &new_pin)?;
        Ok(())
    })
}

pub fn link_parent_student(
    state: &AppState,
    parent_id: i64,
    student_id: i64,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        service.link_parent_student(parent_id, student_id)?;
        Ok(())
    })
}

pub fn list_linked_students(
    state: &AppState,
    parent_id: i64,
) -> Result<Vec<AccountSummaryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        let students = service.get_linked_students(parent_id)?;
        Ok(students.into_iter().map(AccountSummaryDto::from).collect())
    })
}
