use ecoach_substrate::{EcoachError, EcoachResult};
use rusqlite::Connection;

struct Migration {
    id: &'static str,
    sql: &'static str,
}

const RUNTIME_MIGRATIONS: &[Migration] = &[
    Migration {
        id: "001_identity",
        sql: include_str!("../../../migrations/runtime/001_identity.sql"),
    },
    Migration {
        id: "002_curriculum",
        sql: include_str!("../../../migrations/runtime/002_curriculum.sql"),
    },
    Migration {
        id: "003_questions",
        sql: include_str!("../../../migrations/runtime/003_questions.sql"),
    },
    Migration {
        id: "004_student_state",
        sql: include_str!("../../../migrations/runtime/004_student_state.sql"),
    },
    Migration {
        id: "005_sessions",
        sql: include_str!("../../../migrations/runtime/005_sessions.sql"),
    },
    Migration {
        id: "006_coach",
        sql: include_str!("../../../migrations/runtime/006_coach.sql"),
    },
    Migration {
        id: "007_memory",
        sql: include_str!("../../../migrations/runtime/007_memory.sql"),
    },
    Migration {
        id: "008_runtime_events",
        sql: include_str!("../../../migrations/runtime/008_runtime_events.sql"),
    },
    Migration {
        id: "009_session_runtime",
        sql: include_str!("../../../migrations/runtime/009_session_runtime.sql"),
    },
    Migration {
        id: "010_goals_calendar",
        sql: include_str!("../../../migrations/runtime/010_goals_calendar.sql"),
    },
    Migration {
        id: "011_content_packs",
        sql: include_str!("../../../migrations/runtime/011_content_packs.sql"),
    },
    Migration {
        id: "012_reporting",
        sql: include_str!("../../../migrations/runtime/012_reporting.sql"),
    },
    Migration {
        id: "013_glossary",
        sql: include_str!("../../../migrations/runtime/013_glossary.sql"),
    },
    Migration {
        id: "014_library",
        sql: include_str!("../../../migrations/runtime/014_library.sql"),
    },
    Migration {
        id: "015_games",
        sql: include_str!("../../../migrations/runtime/015_games.sql"),
    },
    Migration {
        id: "016_past_papers",
        sql: include_str!("../../../migrations/runtime/016_past_papers.sql"),
    },
    Migration {
        id: "017_traps",
        sql: include_str!("../../../migrations/runtime/017_traps.sql"),
    },
    Migration {
        id: "018_intake",
        sql: include_str!("../../../migrations/runtime/018_intake.sql"),
    },
    Migration {
        id: "019_premium",
        sql: include_str!("../../../migrations/runtime/019_premium.sql"),
    },
    Migration {
        id: "020_elite",
        sql: include_str!("../../../migrations/runtime/020_elite.sql"),
    },
    Migration {
        id: "021_skill_truth",
        sql: include_str!("../../../migrations/runtime/021_skill_truth.sql"),
    },
    Migration {
        id: "022_time_orchestration",
        sql: include_str!("../../../migrations/runtime/022_time_orchestration.sql"),
    },
    Migration {
        id: "023_wrong_answer_diagnoses",
        sql: include_str!("../../../migrations/runtime/023_wrong_answer_diagnoses.sql"),
    },
    Migration {
        id: "024_question_glossary_links",
        sql: include_str!("../../../migrations/runtime/024_question_glossary_links.sql"),
    },
    Migration {
        id: "025_diagnostic_battery_templates",
        sql: include_str!("../../../migrations/runtime/025_diagnostic_battery_templates.sql"),
    },
    Migration {
        id: "026_coach_mission_memory",
        sql: include_str!("../../../migrations/runtime/026_coach_mission_memory.sql"),
    },
    Migration {
        id: "027_library_intelligence",
        sql: include_str!("../../../migrations/runtime/027_library_intelligence.sql"),
    },
    Migration {
        id: "028_beat_yesterday",
        sql: include_str!("../../../migrations/runtime/028_beat_yesterday.sql"),
    },
    Migration {
        id: "029_elite_topic_profiles",
        sql: include_str!("../../../migrations/runtime/029_elite_topic_profiles.sql"),
    },
    Migration {
        id: "030_question_intelligence_registry",
        sql: include_str!("../../../migrations/runtime/030_question_intelligence_registry.sql"),
    },
    Migration {
        id: "031_content_pipeline",
        sql: include_str!("../../../migrations/runtime/031_content_pipeline.sql"),
    },
    Migration {
        id: "032_content_publish_pipeline",
        sql: include_str!("../../../migrations/runtime/032_content_publish_pipeline.sql"),
    },
    Migration {
        id: "033_mock_and_journey_runtime",
        sql: include_str!("../../../migrations/runtime/033_mock_and_journey_runtime.sql"),
    },
    Migration {
        id: "033b_mock_centre",
        sql: include_str!("../../../migrations/runtime/033b_mock_centre.sql"),
    },
    Migration {
        id: "034_diagnostic_analytics",
        sql: include_str!("../../../migrations/runtime/034_diagnostic_analytics.sql"),
    },
    Migration {
        id: "035_traps_runtime",
        sql: include_str!("../../../migrations/runtime/035_traps_runtime.sql"),
    },
    Migration {
        id: "036_question_reactor",
        sql: include_str!("../../../migrations/runtime/036_question_reactor.sql"),
    },
    Migration {
        id: "037_question_graph",
        sql: include_str!("../../../migrations/runtime/037_question_graph.sql"),
    },
    Migration {
        id: "038_content_foundry",
        sql: include_str!("../../../migrations/runtime/038_content_foundry.sql"),
    },
    Migration {
        id: "039_foundry_jobs",
        sql: include_str!("../../../migrations/runtime/039_foundry_jobs.sql"),
    },
    Migration {
        id: "040_forecast",
        sql: include_str!("../../../migrations/runtime/040_forecast.sql"),
    },
    Migration {
        id: "041_mock_extensions",
        sql: include_str!("../../../migrations/runtime/041_mock_extensions.sql"),
    },
    Migration {
        id: "042_mock_diagnosis",
        sql: include_str!("../../../migrations/runtime/042_mock_diagnosis.sql"),
    },
    Migration {
        id: "043_distractor_health",
        sql: include_str!("../../../migrations/runtime/043_distractor_health.sql"),
    },
    Migration {
        id: "044_calibration_crosswalk",
        sql: include_str!("../../../migrations/runtime/044_calibration_crosswalk.sql"),
    },
    Migration {
        id: "045_remaining_gaps",
        sql: include_str!("../../../migrations/runtime/045_remaining_gaps.sql"),
    },
    Migration {
        id: "046_journey_adaptation",
        sql: include_str!("../../../migrations/runtime/046_journey_adaptation.sql"),
    },
    Migration {
        id: "047_journey_deep_model",
        sql: include_str!("../../../migrations/runtime/047_journey_deep_model.sql"),
    },
    Migration {
        id: "048_topic_action_engine",
        sql: include_str!("../../../migrations/runtime/048_topic_action_engine.sql"),
    },
    Migration {
        id: "049_rise_mode",
        sql: include_str!("../../../migrations/runtime/049_rise_mode.sql"),
    },
    Migration {
        id: "050_beat_yesterday_deep",
        sql: include_str!("../../../migrations/runtime/050_beat_yesterday_deep.sql"),
    },
    Migration {
        id: "051_elite_deep",
        sql: include_str!("../../../migrations/runtime/051_elite_deep.sql"),
    },
    Migration {
        id: "052_elite_deep_model",
        sql: include_str!("../../../migrations/runtime/052_elite_deep_model.sql"),
    },
    Migration {
        id: "053_knowledge_gap_deep",
        sql: include_str!("../../../migrations/runtime/053_knowledge_gap_deep.sql"),
    },
    Migration {
        id: "054_knowledge_gap_deep2",
        sql: include_str!("../../../migrations/runtime/054_knowledge_gap_deep2.sql"),
    },
];

pub fn run_runtime_migrations(connection: &mut Connection) -> EcoachResult<()> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_migrations (
                id TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;

    for migration in RUNTIME_MIGRATIONS {
        let already_applied = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE id = ?1)",
                [migration.id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        if already_applied == 1 {
            continue;
        }

        let tx = connection
            .transaction()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        tx.execute_batch(migration.sql).map_err(|err| {
            EcoachError::Storage(format!("migration {} failed: {}", migration.id, err))
        })?;
        tx.execute(
            "INSERT INTO schema_migrations (id) VALUES (?1)",
            [migration.id],
        )
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
        tx.commit()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_migrations_apply_on_in_memory_db() {
        let mut conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");

        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('accounts', 'questions', 'sessions', 'coach_plans', 'memory_states')",
                [],
                |row| row.get(0),
            )
            .expect("table count should be queryable");

        assert_eq!(table_count, 5);
    }
}
