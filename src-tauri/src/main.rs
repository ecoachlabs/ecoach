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
            commands::build_or_refresh_journey_route,
            commands::get_active_journey_route,
            commands::complete_journey_station,
            commands::generate_today_mission,
            // Curriculum
            commands::list_subjects,
            commands::list_topics,
            // Student Model
            commands::get_learner_truth,
            // Content
            commands::list_installed_packs,
            commands::install_pack,
            commands::register_curriculum_source,
            commands::add_curriculum_parse_candidate,
            commands::finalize_curriculum_source,
            commands::resolve_curriculum_review_task,
            commands::mark_curriculum_source_reviewed,
            commands::stage_curriculum_publish_job,
            commands::recompute_topic_package_snapshot,
            commands::get_subject_foundry_dashboard,
            commands::queue_topic_foundry_jobs,
            commands::queue_source_follow_up_jobs,
            commands::list_foundry_jobs,
            commands::get_foundry_job_board,
            commands::start_foundry_job,
            commands::complete_foundry_job,
            commands::fail_foundry_job,
            commands::run_foundry_job,
            commands::run_next_foundry_job,
            // Intake
            commands::create_submission_bundle,
            commands::add_submission_bundle_file,
            commands::reconstruct_submission_bundle,
            commands::get_submission_bundle_report,
            commands::list_submission_bundle_insights,
            // Sessions
            commands::start_practice_session,
            commands::compose_custom_test,
            commands::complete_session,
            commands::generate_mock_blueprint,
            commands::start_mock_session,
            commands::submit_attempt,
            commands::complete_session_with_pipeline,
            // Diagnostics
            commands::launch_diagnostic,
            commands::get_diagnostic_battery,
            commands::list_diagnostic_phase_items,
            commands::submit_diagnostic_attempt,
            commands::advance_diagnostic_phase,
            commands::get_diagnostic_report,
            commands::complete_diagnostic_and_sync,
            // Questions
            commands::choose_reactor_family,
            commands::create_question_generation_request,
            commands::process_question_generation_request,
            commands::get_question_lineage,
            commands::get_question_family_health,
            commands::list_related_questions,
            commands::detect_near_duplicate,
            commands::recommend_question_remediation_plan,
            commands::list_inverse_pressure_families,
            commands::build_elite_session_blueprint,
            // Games
            commands::start_game,
            commands::submit_game_answer,
            commands::get_game_summary,
            commands::get_mindstack_state,
            commands::get_tug_of_war_state,
            commands::list_game_sessions,
            commands::get_leaderboard,
            commands::pause_game,
            commands::resume_game,
            commands::abandon_game,
            // Traps
            commands::list_traps_pairs,
            commands::start_traps_session,
            commands::submit_trap_round,
            commands::record_trap_confusion_reason,
            commands::get_trap_review,
            // Library and Glossary
            commands::get_library_home,
            commands::get_library_snapshot,
            commands::get_continue_learning_card,
            commands::save_library_item,
            commands::build_revision_pack,
            commands::list_revision_pack_items,
            commands::search_glossary,
            commands::list_glossary_bundles_for_topic,
            commands::list_glossary_entries_for_question,
            commands::build_glossary_audio_program_for_topic,
            commands::build_personalized_glossary_audio_program_for_topic,
            commands::build_glossary_audio_program_for_question,
            commands::build_personalized_glossary_audio_program_for_question,
            commands::build_teach_action_plan,
            commands::list_topic_relationship_hints,
            // Memory
            commands::get_review_queue,
            commands::build_memory_review_queue,
            commands::record_retrieval_attempt,
            commands::get_memory_dashboard,
            commands::list_memory_topic_summaries,
            commands::build_memory_return_loop,
            commands::process_decay_batch,
            commands::complete_recheck,
            // Gap repair
            commands::list_priority_gaps,
            commands::generate_repair_plan,
            commands::get_repair_plan,
            commands::advance_repair_item,
            commands::get_gap_dashboard,
            // Readiness and parent
            commands::get_readiness_report,
            commands::generate_parent_digest,
            commands::get_parent_dashboard,
            commands::get_household_dashboard,
            commands::get_admin_oversight_snapshot,
            // Premium
            commands::get_risk_dashboard,
            commands::auto_detect_risks,
            commands::create_intervention,
            commands::resolve_risk_flag,
            commands::resolve_intervention,
            commands::check_entitlement,
            commands::is_feature_enabled,
            commands::get_premium_strategy_snapshot,
            // Mock centre
            commands::compile_mock,
            commands::start_mock,
            commands::submit_mock_answer,
            commands::get_mock_report,
            commands::pause_mock,
            commands::resume_mock,
            commands::list_mock_sessions,
            commands::abandon_mock,
        ])
        .run(tauri::generate_context!())
        .expect("error while running eCoach");
}
