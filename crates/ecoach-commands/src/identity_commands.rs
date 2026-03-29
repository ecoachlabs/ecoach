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
