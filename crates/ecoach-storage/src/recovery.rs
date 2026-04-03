use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use chrono::Utc;
use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::BackupService;

const CORE_REBUILD_DOCS: &[&str] = &[
    "ECOACH_REBUILD_MASTER.md",
    "FEATURE_INVENTORY.md",
    "SCREEN_INVENTORY.md",
    "ARCHITECTURE.md",
    "REBUILD_ORDER.md",
];

const REFERENCE_DOC_DIRS: &[&str] = &["features", "implementation", "backend notes", "docs"];

const STRUCTURED_DOC_DIRS: &[(&str, &str)] = &[
    ("docs/features", "feature_truth_pack"),
    ("docs/recovery", "recovery_doc"),
    ("docs/architecture", "architecture_doc"),
    ("docs/ui", "ui_doc"),
    ("docs/decisions", "decision_doc"),
];

const EXTRA_REFERENCE_FILES: &[&str] = &[
    "ideas/idea35.txt",
    "ideas_implementation_audit.md",
    "backend_execution_plan.md",
    "detailed_backend_implementation_plan.md",
];

const PROTECTION_CHECKLIST: &[&str] = &[
    "Commit rebuild docs with each major slice.",
    "Create a remote-backed recovery snapshot at the end of the day.",
    "Keep the database backup routine reachable from the command boundary.",
    "Avoid destructive file operations outside the repo root.",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryDocumentStatus {
    pub relative_path: String,
    pub exists: bool,
    pub required: bool,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebuildWorkspaceStatus {
    pub workspace_root: String,
    pub core_documents: Vec<RecoveryDocumentStatus>,
    pub documentation_structure: Vec<RecoveryDocumentStatus>,
    pub reference_documents: Vec<RecoveryDocumentStatus>,
    pub feature_doc_count: usize,
    pub implementation_doc_count: usize,
    pub missing_required_documents: Vec<String>,
    pub recommended_next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySnapshotResult {
    pub path: String,
    pub size_bytes: i64,
    pub generated_at: String,
    pub included_entries: Vec<String>,
    pub skipped_entries: Vec<String>,
    pub missing_required_documents: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RecoverySnapshotManifest {
    schema_version: &'static str,
    generated_at: String,
    protection_checklist: Vec<&'static str>,
    workspace_status: Option<RebuildWorkspaceStatus>,
    database_backup_entry: Option<String>,
    included_entries: Vec<String>,
    skipped_entries: Vec<String>,
}

pub struct RecoverySnapshotService;

impl RecoverySnapshotService {
    pub fn inspect_workspace(workspace_root: &Path) -> EcoachResult<RebuildWorkspaceStatus> {
        validate_workspace_root(workspace_root)?;

        let core_documents: Vec<RecoveryDocumentStatus> = CORE_REBUILD_DOCS
            .iter()
            .map(|relative_path| {
                let path = workspace_root.join(relative_path);
                RecoveryDocumentStatus {
                    relative_path: (*relative_path).to_string(),
                    exists: path.exists(),
                    required: true,
                    category: "core_rebuild_doc".to_string(),
                }
            })
            .collect();

        let documentation_structure: Vec<RecoveryDocumentStatus> = STRUCTURED_DOC_DIRS
            .iter()
            .map(|(relative_path, category)| {
                let path = workspace_root.join(relative_path);
                RecoveryDocumentStatus {
                    relative_path: (*relative_path).to_string(),
                    exists: path.exists() && path.is_dir(),
                    required: false,
                    category: (*category).to_string(),
                }
            })
            .collect();

        let mut reference_documents = Vec::new();
        for relative_path in EXTRA_REFERENCE_FILES {
            let path = workspace_root.join(relative_path);
            reference_documents.push(RecoveryDocumentStatus {
                relative_path: (*relative_path).to_string(),
                exists: path.exists(),
                required: false,
                category: "reference_doc".to_string(),
            });
        }

        let feature_doc_count = collect_text_documents(&workspace_root.join("features")).len()
            + collect_text_documents(&workspace_root.join("docs").join("features")).len();
        let implementation_doc_count =
            collect_text_documents(&workspace_root.join("implementation")).len()
                + collect_text_documents(&workspace_root.join("docs").join("recovery")).len()
                + collect_text_documents(&workspace_root.join("docs").join("architecture")).len()
                + collect_text_documents(&workspace_root.join("docs").join("ui")).len()
                + collect_text_documents(&workspace_root.join("docs").join("decisions")).len();

        let missing_required_documents = core_documents
            .iter()
            .filter(|item| !item.exists)
            .map(|item| item.relative_path.clone())
            .collect::<Vec<_>>();

        let missing_structured_dirs = documentation_structure
            .iter()
            .filter(|item| !item.exists)
            .map(|item| item.relative_path.clone())
            .collect::<Vec<_>>();

        let mut recommended_next_actions = Vec::new();
        if !missing_required_documents.is_empty() {
            recommended_next_actions.push(format!(
                "Create the missing idea35 rebuild documents: {}.",
                missing_required_documents.join(", ")
            ));
        }
        if !missing_structured_dirs.is_empty() {
            recommended_next_actions.push(format!(
                "Create the missing idea35 documentation directories: {}.",
                missing_structured_dirs.join(", ")
            ));
        }
        if feature_doc_count == 0 {
            recommended_next_actions.push(
                "Add feature inventory markdown under the workspace features/ and docs/features/ folders."
                    .into(),
            );
        }
        if implementation_doc_count == 0 {
            recommended_next_actions.push(
                "Add implementation and architecture notes under implementation/ plus docs/architecture, docs/recovery, docs/ui, and docs/decisions."
                    .into(),
            );
        }
        if recommended_next_actions.is_empty() {
            recommended_next_actions.push(
                "Workspace recovery artifacts are present; keep exporting snapshots after major slices."
                    .into(),
            );
        }

        Ok(RebuildWorkspaceStatus {
            workspace_root: workspace_root.to_string_lossy().to_string(),
            core_documents,
            documentation_structure,
            reference_documents,
            feature_doc_count,
            implementation_doc_count,
            missing_required_documents,
            recommended_next_actions,
        })
    }

    pub fn export_snapshot(
        source_conn: &Connection,
        output_zip_path: &Path,
        workspace_root: Option<&Path>,
    ) -> EcoachResult<RecoverySnapshotResult> {
        ensure_parent_directory(output_zip_path)?;
        if output_zip_path.exists() {
            fs::remove_file(output_zip_path).map_err(|err| {
                EcoachError::Storage(format!("cannot replace old snapshot: {err}"))
            })?;
        }

        let generated_at = Utc::now().to_rfc3339();
        let temp_backup_path = build_temp_backup_path(output_zip_path);
        let workspace_status = match workspace_root {
            Some(root) => Some(Self::inspect_workspace(root)?),
            None => None,
        };

        let export_result = BackupService::export_backup(source_conn, &temp_backup_path);
        let result = match export_result {
            Ok(_) => {
                let mut included_entries = Vec::new();
                let mut skipped_entries = Vec::new();
                let database_entry = "database/ecoach-runtime.sqlite3".to_string();

                let file = File::create(output_zip_path).map_err(|err| {
                    EcoachError::Storage(format!(
                        "cannot create recovery snapshot archive {}: {err}",
                        output_zip_path.display()
                    ))
                })?;
                let mut zip = ZipWriter::new(file);
                let options =
                    SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

                add_file_to_zip(&mut zip, &database_entry, &temp_backup_path, options)?;
                included_entries.push(database_entry.clone());

                if let Some(root) = workspace_root {
                    for source_path in collect_workspace_entries(root, &mut skipped_entries)? {
                        let relative = source_path.strip_prefix(root).map_err(|err| {
                            EcoachError::Storage(format!(
                                "cannot compute relative recovery path for {}: {err}",
                                source_path.display()
                            ))
                        })?;
                        let entry_name = format!("workspace/{}", normalize_relative_path(relative));
                        add_file_to_zip(&mut zip, &entry_name, &source_path, options)?;
                        included_entries.push(entry_name);
                    }
                }

                let manifest = RecoverySnapshotManifest {
                    schema_version: "ecoach-recovery-snapshot/v1",
                    generated_at: generated_at.clone(),
                    protection_checklist: PROTECTION_CHECKLIST.to_vec(),
                    workspace_status,
                    database_backup_entry: Some(database_entry),
                    included_entries: included_entries.clone(),
                    skipped_entries: skipped_entries.clone(),
                };

                zip.start_file("manifest.json", options).map_err(|err| {
                    EcoachError::Storage(format!("cannot start recovery manifest entry: {err}"))
                })?;
                let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|err| {
                    EcoachError::Serialization(format!("cannot serialize recovery manifest: {err}"))
                })?;
                zip.write_all(&manifest_bytes).map_err(|err| {
                    EcoachError::Storage(format!("cannot write recovery manifest: {err}"))
                })?;
                included_entries.push("manifest.json".to_string());

                zip.finish().map_err(|err| {
                    EcoachError::Storage(format!(
                        "cannot finalize recovery snapshot archive: {err}"
                    ))
                })?;

                let size_bytes = fs::metadata(output_zip_path)
                    .map(|metadata| metadata.len() as i64)
                    .unwrap_or(0);
                let missing_required_documents = manifest
                    .workspace_status
                    .as_ref()
                    .map(|status| status.missing_required_documents.clone())
                    .unwrap_or_default();

                Ok(RecoverySnapshotResult {
                    path: output_zip_path.to_string_lossy().to_string(),
                    size_bytes,
                    generated_at,
                    included_entries,
                    skipped_entries,
                    missing_required_documents,
                })
            }
            Err(err) => Err(err),
        };

        if temp_backup_path.exists() {
            let _ = fs::remove_file(&temp_backup_path);
        }

        result
    }
}

fn validate_workspace_root(workspace_root: &Path) -> EcoachResult<()> {
    if !workspace_root.exists() {
        return Err(EcoachError::NotFound(format!(
            "workspace root not found: {}",
            workspace_root.display()
        )));
    }
    if !workspace_root.is_dir() {
        return Err(EcoachError::Validation(format!(
            "workspace root is not a directory: {}",
            workspace_root.display()
        )));
    }
    Ok(())
}

fn ensure_parent_directory(path: &Path) -> EcoachResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            EcoachError::Storage(format!(
                "cannot create parent directory {}: {err}",
                parent.display()
            ))
        })?;
    }
    Ok(())
}

fn build_temp_backup_path(output_zip_path: &Path) -> PathBuf {
    let stem = output_zip_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("ecoach-recovery");
    output_zip_path.with_file_name(format!("{stem}.snapshot.sqlite3"))
}

fn collect_workspace_entries(
    workspace_root: &Path,
    skipped_entries: &mut Vec<String>,
) -> EcoachResult<Vec<PathBuf>> {
    validate_workspace_root(workspace_root)?;

    let mut entries = BTreeSet::new();

    for relative_path in CORE_REBUILD_DOCS {
        let full_path = workspace_root.join(relative_path);
        if full_path.exists() {
            entries.insert(full_path);
        } else {
            skipped_entries.push((*relative_path).to_string());
        }
    }

    for directory in REFERENCE_DOC_DIRS {
        for path in collect_text_documents(&workspace_root.join(directory)) {
            entries.insert(path);
        }
    }

    for relative_path in EXTRA_REFERENCE_FILES {
        let full_path = workspace_root.join(relative_path);
        if full_path.exists() {
            entries.insert(full_path);
        } else {
            skipped_entries.push((*relative_path).to_string());
        }
    }

    Ok(entries.into_iter().collect())
}

fn collect_text_documents(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_text_documents_recursive(root, &mut files);
    files
}

fn collect_text_documents_recursive(root: &Path, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_text_documents_recursive(&path, files);
            continue;
        }

        let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
            continue;
        };
        if matches!(extension, "md" | "txt") {
            files.push(path);
        }
    }
}

fn normalize_relative_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn add_file_to_zip(
    zip: &mut ZipWriter<File>,
    entry_name: &str,
    source_path: &Path,
    options: SimpleFileOptions,
) -> EcoachResult<()> {
    zip.start_file(entry_name, options).map_err(|err| {
        EcoachError::Storage(format!("cannot start zip entry {entry_name}: {err}"))
    })?;

    let mut file = File::open(source_path).map_err(|err| {
        EcoachError::Storage(format!(
            "cannot open recovery source file {}: {err}",
            source_path.display()
        ))
    })?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|err| {
        EcoachError::Storage(format!(
            "cannot read recovery source file {}: {err}",
            source_path.display()
        ))
    })?;
    zip.write_all(&buffer).map_err(|err| {
        EcoachError::Storage(format!("cannot write zip entry {entry_name}: {err}"))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::Read,
        path::{Path, PathBuf},
        process,
    };

    use chrono::Utc;
    use rusqlite::Connection;
    use std::fs::File;
    use zip::ZipArchive;

    use crate::run_runtime_migrations;

    use super::RecoverySnapshotService;

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "ecoach-recovery-{label}-{}-{}",
                process::id(),
                Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ));
            fs::create_dir_all(&path).expect("temp directory should create");
            Self { path }
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn inspect_workspace_reports_missing_required_docs() {
        let temp = TempDirGuard::new("inspect");
        write_file(&temp.path, "FEATURE_INVENTORY.md", "# Features\n");
        write_file(&temp.path, "features/catalog.md", "feature inventory");
        write_file(
            &temp.path,
            "implementation/runtime.md",
            "implementation notes",
        );

        let status =
            RecoverySnapshotService::inspect_workspace(&temp.path).expect("workspace status");

        assert_eq!(status.feature_doc_count, 1);
        assert_eq!(status.implementation_doc_count, 1);
        assert!(
            status
                .documentation_structure
                .iter()
                .any(|item| item.relative_path == "docs/features" && !item.exists)
        );
        assert!(
            status
                .missing_required_documents
                .contains(&"ECOACH_REBUILD_MASTER.md".to_string())
        );
        assert!(
            status
                .recommended_next_actions
                .iter()
                .any(|item| item.contains("missing idea35 rebuild documents"))
        );
        assert!(
            status
                .recommended_next_actions
                .iter()
                .any(|item| item.contains("missing idea35 documentation directories"))
        );
    }

    #[test]
    fn export_snapshot_packages_database_and_workspace_docs() {
        let temp = TempDirGuard::new("export");
        for relative_path in [
            "ECOACH_REBUILD_MASTER.md",
            "FEATURE_INVENTORY.md",
            "SCREEN_INVENTORY.md",
            "ARCHITECTURE.md",
            "REBUILD_ORDER.md",
            "features/features_ideas_32_to_38.md",
            "implementation/01_architecture_and_design_system.md",
            "backend notes/ecoach_backend_implementation_plan_canonical_v1.md",
            "ideas/idea35.txt",
            "docs/recovery/checklist.md",
        ] {
            write_file(
                &temp.path,
                relative_path,
                &format!("content for {relative_path}"),
            );
        }

        let output_path = temp.path.join("snapshots").join("idea35-recovery.zip");
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        run_runtime_migrations(&mut conn).expect("runtime migrations should apply");

        let result =
            RecoverySnapshotService::export_snapshot(&conn, &output_path, Some(&temp.path))
                .expect("snapshot export should succeed");

        assert!(result.size_bytes > 0);
        assert!(result.missing_required_documents.is_empty());
        assert!(
            result
                .included_entries
                .iter()
                .any(|entry| entry == "database/ecoach-runtime.sqlite3")
        );
        assert!(
            result
                .included_entries
                .iter()
                .any(|entry| entry == "workspace/ECOACH_REBUILD_MASTER.md")
        );

        let file = File::open(&output_path).expect("zip file should open");
        let mut archive = ZipArchive::new(file).expect("zip archive should parse");
        let mut entries = Vec::new();
        for index in 0..archive.len() {
            let file = archive.by_index(index).expect("zip entry should read");
            entries.push(file.name().to_string());
        }

        assert!(entries.contains(&"manifest.json".to_string()));
        assert!(entries.contains(&"database/ecoach-runtime.sqlite3".to_string()));
        assert!(entries.contains(&"workspace/ECOACH_REBUILD_MASTER.md".to_string()));
        assert!(entries.contains(&"workspace/features/features_ideas_32_to_38.md".to_string()));
        assert!(entries.contains(&"workspace/ideas/idea35.txt".to_string()));

        let mut manifest = String::new();
        archive
            .by_name("manifest.json")
            .expect("manifest entry should exist")
            .read_to_string(&mut manifest)
            .expect("manifest should read");
        assert!(manifest.contains("ecoach-recovery-snapshot/v1"));
        assert!(manifest.contains("ECOACH_REBUILD_MASTER.md"));
    }

    fn write_file(root: &Path, relative_path: &str, contents: &str) {
        let full_path = root.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("parent directory should create");
        }
        fs::write(full_path, contents).expect("test file should write");
    }
}
