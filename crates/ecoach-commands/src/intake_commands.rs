use ecoach_intake::IntakeService;

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
