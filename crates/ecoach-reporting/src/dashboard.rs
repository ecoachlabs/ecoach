use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectSummary {
    pub subject_id: i64,
    pub subject_name: String,
    pub readiness_band: String,
    pub mastered_topic_count: usize,
    pub weak_topic_count: usize,
    pub total_topic_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentDashboard {
    pub student_name: String,
    pub exam_target: Option<String>,
    pub subject_summaries: Vec<SubjectSummary>,
    pub overall_readiness_band: String,
}

pub struct DashboardService<'a> {
    conn: &'a Connection,
}

impl<'a> DashboardService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get_student_dashboard(&self, student_id: i64) -> EcoachResult<StudentDashboard> {
        let (student_name, exam_target): (String, Option<String>) = self
            .conn
            .query_row(
                "SELECT a.display_name, sp.exam_target
                 FROM accounts a
                 LEFT JOIN student_profiles sp ON sp.account_id = a.id
                 WHERE a.id = ?1",
                [student_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut statement = self
            .conn
            .prepare(
                "SELECT s.id, s.name
                 FROM subjects s
                 ORDER BY s.display_order ASC, s.name ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut summaries = Vec::new();
        let mut readiness_accumulator: Vec<BasisPoints> = Vec::new();
        for row in rows {
            let (subject_id, subject_name) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let topic_states = self.topic_states_for_subject(student_id, subject_id)?;
            if topic_states.is_empty() {
                continue;
            }
            let mastered_topic_count = topic_states.iter().filter(|state| state.0 >= 7200).count();
            let weak_topic_count = topic_states.iter().filter(|state| state.0 < 4000).count();
            let total_topic_count = topic_states.len();
            let readiness = (topic_states.iter().map(|state| state.0 as i64).sum::<i64>()
                / total_topic_count as i64) as BasisPoints;
            readiness_accumulator.push(readiness);
            summaries.push(SubjectSummary {
                subject_id,
                subject_name,
                readiness_band: readiness_band(readiness).to_string(),
                mastered_topic_count,
                weak_topic_count,
                total_topic_count,
            });
        }

        let overall = if readiness_accumulator.is_empty() {
            0
        } else {
            (readiness_accumulator
                .iter()
                .map(|bp| *bp as i64)
                .sum::<i64>()
                / readiness_accumulator.len() as i64) as BasisPoints
        };

        Ok(StudentDashboard {
            student_name,
            exam_target,
            subject_summaries: summaries,
            overall_readiness_band: readiness_band(overall).to_string(),
        })
    }

    fn topic_states_for_subject(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<(BasisPoints, i64)>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT sts.mastery_score, sts.topic_id
                 FROM student_topic_states sts
                 JOIN topics t ON t.id = sts.topic_id
                 WHERE sts.student_id = ?1 AND t.subject_id = ?2",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let rows = statement
            .query_map([student_id, subject_id], |row| {
                Ok((row.get::<_, BasisPoints>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;

        let mut states = Vec::new();
        for row in rows {
            states.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(states)
    }
}

pub(crate) fn readiness_band(score: BasisPoints) -> &'static str {
    match score {
        8500..=10000 => "Exam Ready",
        7000..=8499 => "Strong",
        5500..=6999 => "Building",
        4000..=5499 => "At Risk",
        _ => "Not Ready",
    }
}
