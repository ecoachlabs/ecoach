use std::path::Path;

use ecoach_commands::{AppState, CommandError, content_commands};

fn main() {
    if let Err(err) = run() {
        eprintln!("install-pack failed: [{}] {}", err.code, err.message);
        std::process::exit(1);
    }
}

fn run() -> Result<(), CommandError> {
    let mut args = std::env::args();
    let _bin = args.next();
    let db_path = args.next().ok_or_else(|| CommandError {
        code: "invalid_args".to_string(),
        message: "usage: install_pack <db_path> <pack_path>".to_string(),
    })?;
    let pack_path = args.next().ok_or_else(|| CommandError {
        code: "invalid_args".to_string(),
        message: "usage: install_pack <db_path> <pack_path>".to_string(),
    })?;

    if args.next().is_some() {
        return Err(CommandError {
            code: "invalid_args".to_string(),
            message: "usage: install_pack <db_path> <pack_path>".to_string(),
        });
    }

    let state = AppState::open_runtime(Path::new(&db_path))?;
    let install = content_commands::install_pack(&state, pack_path.clone())?;

    let (topic_count, objective_count, node_count, misconception_count, question_count) =
        state.with_connection(|conn| {
            let storage_err = |err: rusqlite::Error| CommandError {
                code: "storage_error".to_string(),
                message: err.to_string(),
            };
            let topic_count: i64 = conn.query_row("SELECT COUNT(*) FROM topics", [], |row| {
                row.get(0)
            }).map_err(storage_err)?;
            let objective_count: i64 =
                conn.query_row("SELECT COUNT(*) FROM learning_objectives", [], |row| {
                    row.get(0)
                }).map_err(storage_err)?;
            let node_count: i64 =
                conn.query_row("SELECT COUNT(*) FROM academic_nodes", [], |row| row.get(0))
                    .map_err(storage_err)?;
            let misconception_count: i64 =
                conn.query_row("SELECT COUNT(*) FROM misconception_patterns", [], |row| {
                    row.get(0)
                }).map_err(storage_err)?;
            let question_count: i64 =
                conn.query_row("SELECT COUNT(*) FROM questions", [], |row| row.get(0))
                    .map_err(storage_err)?;
            Ok((
                topic_count,
                objective_count,
                node_count,
                misconception_count,
                question_count,
            ))
        })?;

    println!(
        "installed pack {}@{} from {}",
        install.pack_id, install.pack_version, pack_path
    );
    println!(
        "runtime counts => topics: {}, objectives: {}, academic_nodes: {}, misconceptions: {}, questions: {}",
        topic_count, objective_count, node_count, misconception_count, question_count
    );
    println!("db_path => {}", db_path);

    Ok(())
}
