#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ecoach_commands::AppState;
use tauri::Manager;

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data directory");
            let db_path = app_data_dir.join("ecoach.db");

            let state = AppState::open_runtime(&db_path).expect("failed to open runtime database");
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Identity
            commands::list_accounts,
            commands::create_account,
            commands::login_with_pin,
            // Coach
            commands::get_coach_state,
            commands::get_coach_next_action,
            commands::get_content_readiness,
            commands::get_priority_topics,
            commands::get_student_dashboard,
            // Curriculum
            commands::list_subjects,
            commands::list_topics,
            // Student Model
            commands::get_learner_truth,
            // Content
            commands::list_installed_packs,
            commands::install_pack,
            // Sessions
            commands::start_practice_session,
            commands::compose_custom_test,
            commands::complete_session,
            commands::generate_mock_blueprint,
            commands::start_mock_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running eCoach");
}
