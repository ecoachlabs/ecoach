pub mod content_commands;
pub mod dtos;
pub mod error;
pub mod identity_commands;
pub mod session_commands;
pub mod state;
pub mod traps_commands;

pub use error::CommandError;
pub use state::AppState;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::PackService;
    use ecoach_games::TrapsMode;
    use ecoach_identity::CreateAccountInput;
    use ecoach_sessions::PracticeSessionStartInput;
    use ecoach_substrate::{AccountType, EntitlementTier};

    use crate::{
        content_commands, identity_commands, session_commands, state::AppState, traps_commands,
    };

    #[test]
    fn command_boundary_returns_dtos_for_core_runtime_flows() {
        let state = AppState::in_memory().expect("in-memory command state should build");
        state
            .with_connection(|conn| {
                let service = PackService::new(conn);
                service.install_pack(&sample_pack_path())?;
                Ok(())
            })
            .expect("sample pack should install");

        let account = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Student,
                display_name: "Kwame".to_string(),
                pin: "1234".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("account dto should create");

        let packs =
            content_commands::list_installed_packs(&state).expect("pack summaries should load");
        let practice = session_commands::start_practice_session(
            &state,
            PracticeSessionStartInput {
                student_id: account.id,
                subject_id: 1,
                topic_ids: vec![2],
                question_count: 2,
                is_timed: false,
            },
        )
        .expect("practice session dto should create");
        let pairs = traps_commands::list_traps_pairs(&state, account.id, 1, vec![2])
            .expect("contrast pair dto should load");
        let traps = traps_commands::start_traps_session(
            &state,
            ecoach_games::StartTrapsSessionInput {
                student_id: account.id,
                subject_id: 1,
                topic_ids: vec![2],
                pair_id: Some(pairs[0].pair_id),
                mode: TrapsMode::DifferenceDrill,
                round_count: 4,
                timer_seconds: Some(6),
            },
        )
        .expect("trap snapshot dto should create");

        assert_eq!(packs.len(), 1);
        assert_eq!(practice.item_count, 2);
        assert_eq!(pairs.len(), 1);
        assert_eq!(traps.round_count, 4);
    }

    fn sample_pack_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate directory should have workspace parent")
            .parent()
            .expect("workspace root should exist")
            .join("packs")
            .join("math-bece-sample")
    }
}
