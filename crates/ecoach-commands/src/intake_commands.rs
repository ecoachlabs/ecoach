use ecoach_content::{FoundryCoordinatorService, SourceUploadInput};
use ecoach_intake::{
    BundleCoachApplicationResult, BundleConfirmationInput, BundleInboxItem, BundleOcrWorkspace,
    BundleReviewReflectionInput, BundleSharedPromotion, IntakeService,
    PersonalAcademicVaultSnapshot, UploadedPaperReviewSnapshot,
};
use rusqlite::OptionalExtension;
use serde_json::json;

use crate::{
    dtos::{BundleProcessReportDto, ExtractedInsightDto, SubmissionBundleDto},
    error::CommandError,
    state::AppState,
};

pub fn create_submission_bundle(
    state: &AppState,
    student_id: i64,
    title: String,
) -> Result<SubmissionBundleDto, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        let bundle_id = service.create_bundle(student_id, &title)?;
        let report = service.get_bundle_report(bundle_id)?;
        Ok(SubmissionBundleDto::from(report.bundle))
    })
}

pub fn add_submission_bundle_file(
    state: &AppState,
    bundle_id: i64,
    file_name: String,
    file_path: String,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        Ok(service.add_bundle_file(bundle_id, &file_name, &file_path)?)
    })
}

pub fn reconstruct_submission_bundle(
    state: &AppState,
    bundle_id: i64,
) -> Result<BundleProcessReportDto, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        let report = service.reconstruct_bundle(bundle_id)?;
        Ok(BundleProcessReportDto::from(report))
    })
}

pub fn get_submission_bundle_report(
    state: &AppState,
    bundle_id: i64,
) -> Result<BundleProcessReportDto, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        let report = service.get_bundle_report(bundle_id)?;
        Ok(BundleProcessReportDto::from(report))
    })
}

pub fn list_submission_bundle_insights(
    state: &AppState,
    bundle_id: i64,
) -> Result<Vec<ExtractedInsightDto>, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        let insights = service.list_bundle_insights(bundle_id)?;
        Ok(insights
            .into_iter()
            .map(ExtractedInsightDto::from)
            .collect())
    })
}

pub type BundleInboxItemDto = BundleInboxItem;
pub type BundleConfirmationInputDto = BundleConfirmationInput;
pub type BundleReviewReflectionInputDto = BundleReviewReflectionInput;
pub type UploadedPaperReviewSnapshotDto = UploadedPaperReviewSnapshot;
pub type BundleCoachApplicationResultDto = BundleCoachApplicationResult;
pub type BundleOcrWorkspaceDto = BundleOcrWorkspace;
pub type PersonalAcademicVaultSnapshotDto = PersonalAcademicVaultSnapshot;
pub type BundleSharedPromotionDto = BundleSharedPromotion;

pub fn list_submission_bundle_inbox(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<BundleInboxItemDto>, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        service.list_bundle_inbox(student_id, limit).map_err(Into::into)
    })
}

pub fn confirm_submission_bundle(
    state: &AppState,
    bundle_id: i64,
    input: BundleConfirmationInputDto,
) -> Result<BundleProcessReportDto, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        let report = service.confirm_bundle(bundle_id, input)?;
        Ok(BundleProcessReportDto::from(report))
    })
}

pub fn record_bundle_review_reflection(
    state: &AppState,
    bundle_id: i64,
    input: BundleReviewReflectionInputDto,
) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        let _ = service.record_bundle_review_reflection(bundle_id, input)?;
        Ok(())
    })
}

pub fn get_uploaded_paper_review(
    state: &AppState,
    bundle_id: i64,
) -> Result<UploadedPaperReviewSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        service.build_uploaded_paper_review(bundle_id).map_err(Into::into)
    })
}

pub fn apply_submission_bundle_to_coach(
    state: &AppState,
    bundle_id: i64,
) -> Result<BundleCoachApplicationResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        service.apply_bundle_to_coach(bundle_id).map_err(Into::into)
    })
}

pub fn get_submission_bundle_ocr_workspace(
    state: &AppState,
    bundle_id: i64,
) -> Result<BundleOcrWorkspaceDto, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        service
            .build_bundle_ocr_workspace(bundle_id)
            .map_err(Into::into)
    })
}

pub fn get_personal_academic_vault(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<PersonalAcademicVaultSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = IntakeService::new(conn);
        service
            .build_personal_academic_vault(student_id, limit)
            .map_err(Into::into)
    })
}

pub fn promote_submission_bundle_to_shared_source(
    state: &AppState,
    bundle_id: i64,
    requested_by_account_id: Option<i64>,
    source_kind_override: Option<String>,
) -> Result<BundleSharedPromotionDto, CommandError> {
    state.with_connection(|conn| {
        let intake = IntakeService::new(conn);
        let report = intake.get_bundle_report(bundle_id)?;
        let files = intake.list_bundle_files(bundle_id)?;
        let primary_file = files.first().ok_or_else(|| CommandError {
            code: "validation_error".to_string(),
            message: "bundle must contain at least one file to promote".to_string(),
        })?;
        let source_kind =
            source_kind_override.unwrap_or_else(|| infer_bundle_source_kind(&report.bundle_kind));
        let source = FoundryCoordinatorService::new(conn).register_source_upload(SourceUploadInput {
            uploader_account_id: requested_by_account_id.unwrap_or(report.bundle.student_id),
            source_kind,
            title: format!("Vault Candidate: {}", report.bundle.title),
            source_path: Some(primary_file.file_path.clone()),
            country_code: None,
            exam_board: None,
            education_level: None,
            subject_code: detect_subject_code(conn, &report.detected_subjects)?,
            academic_year: report
                .detected_exam_years
                .first()
                .map(ToString::to_string),
            language_code: Some("en".to_string()),
            version_label: Some("vault_candidate".to_string()),
            metadata: json!({
                "origin": "personal_academic_vault",
                "bundle_id": bundle_id,
                "student_id": report.bundle.student_id,
                "detected_topics": report.detected_topics,
                "recommended_actions": report.recommended_actions,
            }),
        })?;
        intake
            .record_bundle_shared_promotion(
                bundle_id,
                Some(source.id),
                requested_by_account_id,
                "queued",
                &json!({
                    "source_upload_id": source.id,
                    "source_status": source.source_status,
                    "source_kind": source.source_kind,
                }),
            )
            .map_err(Into::into)
    })
}

fn infer_bundle_source_kind(bundle_kind: &str) -> String {
    match bundle_kind {
        "past_paper_bundle" | "question_and_markscheme" => "past_question",
        "report_card" | "homework_bundle" | "worksheet_bundle" => "worksheet",
        "teacher_note_bundle" | "class_note_bundle" => "guide",
        _ => "worksheet",
    }
    .to_string()
}

fn detect_subject_code(
    conn: &rusqlite::Connection,
    detected_subjects: &[String],
) -> Result<Option<String>, CommandError> {
    for subject in detected_subjects {
        let normalized = subject.trim();
        if normalized.is_empty() {
            continue;
        }
        let code = conn
            .query_row(
                "SELECT code
                 FROM subjects
                 WHERE lower(name) = lower(?1)
                    OR lower(code) = lower(?1)
                 LIMIT 1",
                [normalized],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|err| CommandError {
                code: "storage_error".to_string(),
                message: err.to_string(),
            })?;
        if code.is_some() {
            return Ok(code);
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        process,
        time::{SystemTime, UNIX_EPOCH},
    };

    use serde_json::Value;

    use super::*;

    #[test]
    fn intake_commands_create_and_reconstruct_bundle() {
        let state = AppState::in_memory().expect("in-memory command state should build");
        seed_student(&state);

        let temp_dir = test_temp_dir("intake_commands");
        let question_path = temp_dir.join("Mathematics Questions.txt");
        fs::write(
            &question_path,
            "MATHEMATICS\n1. What is 2 + 2?\na) 3\nb) 4\n2. Solve x + 2 = 5.\n",
        )
        .expect("question file should write");

        let bundle = create_submission_bundle(&state, 1, "Command intake bundle".to_string())
            .expect("bundle should create");
        add_submission_bundle_file(
            &state,
            bundle.id,
            file_name(&question_path).to_string(),
            question_path.to_string_lossy().into_owned(),
        )
        .expect("bundle file should add");
        let report =
            reconstruct_submission_bundle(&state, bundle.id).expect("bundle should reconstruct");

        assert_eq!(report.bundle.id, bundle.id);
        assert!(report.estimated_question_count >= 2);

        let insights = list_submission_bundle_insights(&state, bundle.id)
            .expect("bundle insights should load");
        assert!(
            insights
                .iter()
                .any(|item| item.insight_type == "bundle_overview")
        );
        assert!(
            report
                .insights
                .iter()
                .any(|item| item.insight_type == "file_reconstruction")
        );

        let file_reconstruction = report
            .insights
            .iter()
            .find(|item| item.insight_type == "file_reconstruction")
            .expect("file reconstruction should exist");
        assert_eq!(
            file_reconstruction
                .payload
                .get("document_role")
                .and_then(Value::as_str),
            Some("question_paper")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn intake_commands_surface_vault_and_shared_promotion_flow() {
        let state = AppState::in_memory().expect("in-memory command state should build");
        seed_student(&state);

        let temp_dir = test_temp_dir("intake_commands_vault");
        let question_path = temp_dir.join("Biology Class Test.txt");
        fs::write(
            &question_path,
            "BIOLOGY CLASS TEST\nTopic: Cell Division\n1. Define mitosis.\nTeacher note: Focus on accurate stages.\n",
        )
        .expect("question file should write");

        let bundle = create_submission_bundle(&state, 1, "Vault candidate".to_string())
            .expect("bundle should create");
        add_submission_bundle_file(
            &state,
            bundle.id,
            file_name(&question_path).to_string(),
            question_path.to_string_lossy().into_owned(),
        )
        .expect("bundle file should add");
        reconstruct_submission_bundle(&state, bundle.id).expect("bundle should reconstruct");

        let workspace = get_submission_bundle_ocr_workspace(&state, bundle.id)
            .expect("ocr workspace should load");
        let vault =
            get_personal_academic_vault(&state, 1, 10).expect("personal academic vault should load");
        let promotion = promote_submission_bundle_to_shared_source(
            &state,
            bundle.id,
            Some(1),
            Some("worksheet".to_string()),
        )
        .expect("shared promotion should create");

        assert_eq!(workspace.bundle.id, bundle.id);
        assert_eq!(vault.total_bundle_count, 1);
        assert_eq!(promotion.bundle_id, bundle.id);

        let _ = fs::remove_dir_all(temp_dir);
    }

    fn seed_student(state: &AppState) {
        state
            .with_connection(|conn| {
                conn.execute(
                    "INSERT INTO accounts (id, account_type, display_name, pin_hash, pin_salt, status, first_run)
                     VALUES (1, 'student', 'Ama', 'hash', 'salt', 'active', 0)",
                    [],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                conn.execute(
                    "INSERT INTO student_profiles (account_id, preferred_subjects, daily_study_budget_minutes)
                     VALUES (1, '[\"mathematics\"]', 60)",
                    [],
                )
                .map_err(|err| CommandError {
                    code: "storage_error".to_string(),
                    message: err.to_string(),
                })?;
                Ok(())
            })
            .expect("student should seed");
    }

    fn test_temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = env::temp_dir().join(format!("{}_{}_{}", prefix, process::id(), unique));
        fs::create_dir_all(&dir).expect("temp dir should create");
        dir
    }

    fn file_name(path: &PathBuf) -> &str {
        path.file_name()
            .and_then(|value| value.to_str())
            .expect("file name should be unicode")
    }
}
