pub mod attempt_commands;
pub mod coach_commands;
pub mod content_commands;
pub mod curriculum_commands;
pub mod diagnostic_commands;
pub mod dtos;
pub mod error;
pub mod game_commands;
pub mod identity_commands;
pub mod library_commands;
pub mod memory_commands;
pub mod mock_commands;
pub mod premium_commands;
pub mod question_commands;
pub mod readiness_commands;
pub mod repair_commands;
pub mod session_commands;
pub mod state;
pub mod student_commands;
pub mod traps_commands;

pub use error::CommandError;
pub use state::AppState;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ecoach_content::{PackService, ParseCandidateInput, SourceUploadInput};
    use ecoach_games::TrapsMode;
    use ecoach_identity::CreateAccountInput;
    use ecoach_questions::{QuestionGenerationRequestInput, QuestionSlotSpec, QuestionVariantMode};
    use ecoach_sessions::PracticeSessionStartInput;
    use ecoach_substrate::{AccountType, EntitlementTier};

    use crate::{
        CommandError, content_commands, identity_commands, question_commands, session_commands,
        state::AppState, traps_commands,
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
        let family = question_commands::choose_reactor_family(
            &state,
            QuestionSlotSpec {
                subject_id: 1,
                topic_id: Some(2),
                target_cognitive_demand: Some("recognition".to_string()),
                target_question_format: Some("mcq".to_string()),
                max_generated_share: 7_000,
            },
        )
        .expect("reactor family choice should load")
        .expect("reactor family should exist");
        let request = question_commands::create_question_generation_request(
            &state,
            QuestionGenerationRequestInput {
                slot_spec: QuestionSlotSpec {
                    subject_id: 1,
                    topic_id: Some(2),
                    target_cognitive_demand: Some("recognition".to_string()),
                    target_question_format: Some("mcq".to_string()),
                    max_generated_share: 7_000,
                },
                family_id: Some(family.family_id),
                source_question_id: None,
                request_kind: "variant".to_string(),
                variant_mode: QuestionVariantMode::RepresentationShift,
                requested_count: 1,
                rationale: Some("Fresh question for a mock slot".to_string()),
            },
        )
        .expect("generation request dto should create");
        let generated = question_commands::process_question_generation_request(&state, request.id)
            .expect("reactor generation should process");
        let lineage = question_commands::get_question_lineage(&state, generated[0].question_id)
            .expect("lineage dto should load");
        let related = question_commands::list_related_questions(
            &state,
            generated[0].source_question_id,
            None,
            10,
        )
        .expect("related question dtos should load");
        let duplicate = question_commands::detect_near_duplicate(
            &state,
            "Which fraction is equivalent to 1/2?".to_string(),
            Some(family.family_id),
            Some(2),
        )
        .expect("duplicate dto should load");
        let family_health = question_commands::get_question_family_health(&state, family.family_id)
            .expect("family health dto should load")
            .expect("family health should exist");

        assert_eq!(packs.len(), 1);
        assert_eq!(practice.item_count, 2);
        assert_eq!(pairs.len(), 1);
        assert_eq!(traps.round_count, 4);
        assert_eq!(generated.len(), 1);
        assert_eq!(lineage.edge_count, 1);
        assert!(
            related
                .iter()
                .any(|item| item.relation_type == "same_family")
        );
        assert!(duplicate.is_near_duplicate);
        assert_eq!(family_health.generated_instances, 1);
    }

    #[test]
    fn command_boundary_drives_foundry_workflow() {
        let state = AppState::in_memory().expect("in-memory command state should build");
        state
            .with_connection(|conn| {
                let service = PackService::new(conn);
                service.install_pack(&sample_pack_path())?;
                Ok(())
            })
            .expect("sample pack should install");

        let admin = identity_commands::create_account(
            &state,
            CreateAccountInput {
                account_type: AccountType::Admin,
                display_name: "Admin".to_string(),
                pin: "999999".to_string(),
                entitlement_tier: EntitlementTier::Standard,
            },
        )
        .expect("admin account should create");

        let source = content_commands::register_curriculum_source(
            &state,
            SourceUploadInput {
                uploader_account_id: admin.id,
                source_kind: "curriculum".to_string(),
                title: "Math Curriculum v2".to_string(),
                source_path: Some("C:/curriculum/math-v2.pdf".to_string()),
                country_code: Some("GH".to_string()),
                exam_board: Some("WAEC".to_string()),
                education_level: Some("JHS".to_string()),
                subject_code: Some("MATH".to_string()),
                academic_year: Some("2026".to_string()),
                language_code: Some("en".to_string()),
                version_label: Some("v2".to_string()),
                metadata: serde_json::json!({ "source_trust": "tier_a" }),
            },
        )
        .expect("source should register");

        for (candidate_type, raw_label) in [
            ("subject", "Mathematics"),
            ("topic", "Fractions"),
            ("objective", "Identify equivalent fractions"),
        ] {
            content_commands::add_curriculum_parse_candidate(
                &state,
                source.id,
                ParseCandidateInput {
                    candidate_type: candidate_type.to_string(),
                    parent_candidate_id: None,
                    raw_label: raw_label.to_string(),
                    normalized_label: Some(raw_label.to_ascii_lowercase()),
                    payload: serde_json::json!({ "page": 1 }),
                    confidence_score: 9000,
                },
            )
            .expect("parse candidate should insert");
        }

        let report = content_commands::finalize_curriculum_source(&state, source.id)
            .expect("source should finalize");
        assert_eq!(report.source_upload.source_status, "parsed");

        let reviewed = content_commands::mark_curriculum_source_reviewed(&state, source.id)
            .expect("source should mark reviewed");
        assert_eq!(reviewed.source_upload.source_status, "reviewed");

        state
            .with_connection(|conn| {
                let topic_id: i64 = conn
                    .query_row("SELECT id FROM topics WHERE code = 'FRA' LIMIT 1", [], |row| {
                        row.get(0)
                    })
                    .map_err(|err| CommandError {
                        code: "storage_error".to_string(),
                        message: err.to_string(),
                    })?;
                conn.execute(
                    "INSERT INTO content_acquisition_jobs (
                        subject_id, topic_id, intent_type, query_text, source_scope, status, result_summary_json, completed_at
                     ) VALUES (?1, ?2, 'gap_fill', 'fractions note evidence', 'internal', 'completed', '{}', datetime('now'))",
                    rusqlite::params![1, topic_id],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                let acquisition_job_id = conn.last_insert_rowid();
                conn.execute(
                    "INSERT INTO acquisition_evidence_candidates (
                        job_id, source_label, source_url, source_kind, title, snippet,
                        extracted_payload_json, quality_score, freshness_score, review_status
                     ) VALUES (?1, 'Teacher Guide', NULL, 'internal', 'Fractions Guide', 'Aligned support', '{}', 8400, 7800, 'approved')",
                    [acquisition_job_id],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                Ok(())
            })
            .expect("evidence should seed");

        let publish_status = content_commands::stage_curriculum_publish_job(
            &state,
            source.id,
            Some(admin.id),
            Some(1),
            Some(2),
            Some("v2".to_string()),
        )
        .expect("publish should stage");
        let topic_jobs =
            content_commands::queue_topic_foundry_jobs(&state, 2, "snapshot_refresh".to_string())
                .expect("topic jobs should queue");
        let source_jobs = content_commands::queue_source_follow_up_jobs(
            &state,
            source.id,
            "source_review".to_string(),
        )
        .expect("source jobs should queue");
        let running_job = content_commands::start_foundry_job(&state, topic_jobs[0].id)
            .expect("job should start");
        let completed_job = content_commands::complete_foundry_job(
            &state,
            running_job.id,
            serde_json::json!({ "artifacts_built": 1 }),
        )
        .expect("job should complete");
        let failed_job = content_commands::fail_foundry_job(
            &state,
            source_jobs[0].id,
            "manual reviewer blocked publish".to_string(),
        )
        .expect("job should fail");
        let job_board = content_commands::get_foundry_job_board(&state, Some(1))
            .expect("job board should load");
        let dashboard = content_commands::get_subject_foundry_dashboard(&state, 1)
            .expect("dashboard should load")
            .expect("dashboard should exist");

        assert_eq!(publish_status, "ready_to_publish");
        assert!(!topic_jobs.is_empty());
        assert!(!source_jobs.is_empty());
        assert_eq!(completed_job.status, "completed");
        assert_eq!(failed_job.status, "failed");
        assert!(job_board.failed_count >= 1);
        assert!(dashboard.average_package_score > 0);
        assert_eq!(dashboard.subject_code, "MATH");
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
