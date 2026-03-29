use std::path::Path;

use ecoach_content::PackService;

use crate::{
    dtos::{PackInstallResultDto, PackSummaryDto},
    error::CommandError,
    state::AppState,
};

pub fn install_pack(
    state: &AppState,
    pack_path: String,
) -> Result<PackInstallResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = PackService::new(conn);
        let result = service.install_pack(Path::new(&pack_path))?;
        Ok(PackInstallResultDto::from(result))
    })
}

pub fn list_installed_packs(state: &AppState) -> Result<Vec<PackSummaryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = PackService::new(conn);
        let packs = service.list_packs()?;
        Ok(packs.into_iter().map(PackSummaryDto::from).collect())
    })
}
