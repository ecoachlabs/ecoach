use ecoach_identity::{
    CreateAccountInput, EntitlementAuditEntry, EntitlementEvent, IdentityService,
    UpdateAccountAccessInput, UpdateEntitlementInput,
};

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

pub fn reset_pin(state: &AppState, account_id: i64, new_pin: String) -> Result<(), CommandError> {
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

pub type EntitlementAuditEntryDto = EntitlementAuditEntry;
pub type EntitlementEventDto = EntitlementEvent;
pub type UpdateEntitlementInputDto = UpdateEntitlementInput;

pub fn update_account_access(
    state: &AppState,
    account_id: i64,
    input: UpdateAccountAccessInput,
) -> Result<AccountDto, CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        let account = service.update_account_access(account_id, input)?;
        Ok(AccountDto::from(account))
    })
}

pub fn list_entitlement_audit_entries(
    state: &AppState,
    limit: usize,
) -> Result<Vec<EntitlementAuditEntryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        service
            .list_entitlement_audit_entries(limit)
            .map_err(Into::into)
    })
}

pub fn update_account_entitlement(
    state: &AppState,
    account_id: i64,
    input: UpdateEntitlementInputDto,
) -> Result<AccountDto, CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        let account = service.update_entitlement(account_id, input)?;
        Ok(AccountDto::from(account))
    })
}

pub fn list_entitlement_events(
    state: &AppState,
    account_id: i64,
    limit: usize,
) -> Result<Vec<EntitlementEventDto>, CommandError> {
    state.with_connection(|conn| {
        let service = IdentityService::new(conn);
        service
            .list_entitlement_events(account_id, limit)
            .map_err(Into::into)
    })
}
