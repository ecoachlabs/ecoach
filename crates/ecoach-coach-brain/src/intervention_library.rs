use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{TopicCase, build_topic_case};

pub struct InterventionLibraryService<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionModeDefinition {
    pub mode_code: String,
    pub mode_name: String,
    pub mode_family: String,
    pub objective: String,
    pub entry_rules: Vec<String>,
    pub contraindications: Vec<String>,
    pub success_signals: Vec<String>,
    pub next_modes: Vec<String>,
    pub report_translation: Option<String>,
    pub sort_order: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemCauseFixCard {
    pub topic_id: i64,
    pub topic_name: String,
    pub problem_summary: String,
    pub cause_summary: String,
    pub fix_summary: String,
    pub confidence_score: BasisPoints,
    pub impact_score: BasisPoints,
    pub unlock_summary: Option<String>,
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionPrescription {
    pub topic_id: i64,
    pub topic_name: String,
    pub primary_mode_code: String,
    pub support_mode_code: Option<String>,
    pub recheck_mode_code: Option<String>,
    pub mode_chain: Vec<String>,
    pub contraindications: Vec<String>,
    pub success_signals: Vec<String>,
    pub confidence_score: BasisPoints,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticPrescriptionSync {
    pub problem_cause_fix_cards: Vec<ProblemCauseFixCard>,
    pub intervention_prescriptions: Vec<InterventionPrescription>,
}

impl<'a> InterventionLibraryService<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn list_intervention_modes(&self) -> EcoachResult<Vec<InterventionModeDefinition>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT mode_code, mode_name, mode_family, objective, entry_rules_json,
                        contraindications_json, success_signals_json, next_modes_json,
                        report_translation, sort_order, is_active
                 FROM intervention_mode_library
                 WHERE is_active = 1
                 ORDER BY sort_order ASC, mode_code ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                let entry_rules_json: String = row.get(4)?;
                let contraindications_json: String = row.get(5)?;
                let success_signals_json: String = row.get(6)?;
                let next_modes_json: String = row.get(7)?;
                Ok(InterventionModeDefinition {
                    mode_code: row.get(0)?,
                    mode_name: row.get(1)?,
                    mode_family: row.get(2)?,
                    objective: row.get(3)?,
                    entry_rules: serde_json::from_str(&entry_rules_json).unwrap_or_default(),
                    contraindications: serde_json::from_str(&contraindications_json)
                        .unwrap_or_default(),
                    success_signals: serde_json::from_str(&success_signals_json)
                        .unwrap_or_default(),
                    next_modes: serde_json::from_str(&next_modes_json).unwrap_or_default(),
                    report_translation: row.get(8)?,
                    sort_order: row.get(9)?,
                    is_active: row.get::<_, i64>(10)? == 1,
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut modes = Vec::new();
        for row in rows {
            modes.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(modes)
    }

    pub fn build_topic_prescription(
        &self,
        student_id: i64,
        topic_id: i64,
    ) -> EcoachResult<DiagnosticPrescriptionSync> {
        let topic_case = build_topic_case(self.conn, student_id, topic_id)?;
        let card = self.build_problem_card(&topic_case);
        let prescription = self.build_prescription(&topic_case)?;
        Ok(DiagnosticPrescriptionSync {
            problem_cause_fix_cards: vec![card],
            intervention_prescriptions: vec![prescription],
        })
    }

    pub fn sync_diagnostic_prescriptions(
        &self,
        diagnostic_id: i64,
        student_id: i64,
        topic_ids: &[i64],
    ) -> EcoachResult<DiagnosticPrescriptionSync> {
        let mut seen = std::collections::BTreeSet::new();
        let mut cards = Vec::new();
        let mut prescriptions = Vec::new();

        for &topic_id in topic_ids {
            if !seen.insert(topic_id) {
                continue;
            }
            let topic_case = build_topic_case(self.conn, student_id, topic_id)?;
            let card = self.build_problem_card(&topic_case);
            self.persist_problem_card(diagnostic_id, student_id, &card)?;
            let prescription = self.build_prescription(&topic_case)?;
            self.persist_prescription(diagnostic_id, student_id, &prescription)?;
            cards.push(card);
            prescriptions.push(prescription);
        }

        Ok(DiagnosticPrescriptionSync {
            problem_cause_fix_cards: cards,
            intervention_prescriptions: prescriptions,
        })
    }

    pub fn list_problem_cause_fix_cards(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<ProblemCauseFixCard>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, topic_name, problem_summary, cause_summary, fix_summary,
                        confidence_score_bp, impact_score_bp, unlock_summary, evidence_json
                 FROM diagnostic_problem_cause_fix_cards
                 WHERE diagnostic_id = ?1
                 ORDER BY impact_score_bp DESC, confidence_score_bp DESC, topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                let evidence_json: String = row.get(8)?;
                Ok(ProblemCauseFixCard {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    problem_summary: row.get(2)?,
                    cause_summary: row.get(3)?,
                    fix_summary: row.get(4)?,
                    confidence_score: row.get(5)?,
                    impact_score: row.get(6)?,
                    unlock_summary: row.get(7)?,
                    evidence: serde_json::from_str(&evidence_json).unwrap_or_else(|_| json!({})),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut cards = Vec::new();
        for row in rows {
            cards.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(cards)
    }

    pub fn list_intervention_prescriptions(
        &self,
        diagnostic_id: i64,
    ) -> EcoachResult<Vec<InterventionPrescription>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT topic_id, topic_name, primary_mode_code, support_mode_code,
                        recheck_mode_code, mode_chain_json, contraindications_json,
                        success_signals_json, confidence_score_bp, payload_json
                 FROM diagnostic_intervention_prescriptions
                 WHERE diagnostic_id = ?1
                 ORDER BY confidence_score_bp DESC, topic_id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([diagnostic_id], |row| {
                let mode_chain_json: String = row.get(5)?;
                let contraindications_json: String = row.get(6)?;
                let success_signals_json: String = row.get(7)?;
                let payload_json: String = row.get(9)?;
                Ok(InterventionPrescription {
                    topic_id: row.get(0)?,
                    topic_name: row.get(1)?,
                    primary_mode_code: row.get(2)?,
                    support_mode_code: row.get(3)?,
                    recheck_mode_code: row.get(4)?,
                    mode_chain: serde_json::from_str(&mode_chain_json).unwrap_or_default(),
                    contraindications: serde_json::from_str(&contraindications_json)
                        .unwrap_or_default(),
                    success_signals: serde_json::from_str(&success_signals_json)
                        .unwrap_or_default(),
                    confidence_score: row.get(8)?,
                    payload: serde_json::from_str(&payload_json).unwrap_or_else(|_| json!({})),
                })
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut prescriptions = Vec::new();
        for row in rows {
            prescriptions.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(prescriptions)
    }

    fn build_problem_card(&self, topic_case: &TopicCase) -> ProblemCauseFixCard {
        let primary_hypothesis = topic_case.active_hypotheses.first();
        let problem_summary = if let Some(blocker) = &topic_case.active_blocker {
            format!("{} is currently blocked because {}.", topic_case.topic_name, blocker.reason)
        } else if topic_case.primary_hypothesis_code == "pressure_collapse" {
            format!(
                "{} works better in calm conditions than under time pressure.",
                topic_case.topic_name
            )
        } else if topic_case.primary_hypothesis_code == "memory_decay" {
            format!(
                "{} is slipping mainly because retrieval is not stable yet.",
                topic_case.topic_name
            )
        } else {
            format!(
                "{} is still costing marks through {}.",
                topic_case.topic_name,
                primary_hypothesis
                    .map(|item| item.label.to_lowercase())
                    .unwrap_or_else(|| "an unresolved weakness".to_string())
            )
        };
        let cause_summary = primary_hypothesis
            .map(|item| item.evidence_summary.clone())
            .unwrap_or_else(|| topic_case.recommended_intervention.reason.clone());
        let fix_summary = if topic_case.recommended_intervention.mode == "pressure_conditioning" {
            "Stabilize one calm success first, then rebuild timed performance in layers."
                .to_string()
        } else if topic_case.recommended_intervention.mode == "contrast_repair" {
            "Use explicit compare-and-contrast repair until the learner can explain the decisive difference."
                .to_string()
        } else {
            format!(
                "Start with {} for about {} minutes, then confirm the gain with a fresh recheck.",
                topic_case.recommended_intervention.mode.replace('_', " "),
                topic_case.recommended_intervention.recommended_minutes
            )
        };
        let unlock_summary = Some(match topic_case.primary_hypothesis_code.as_str() {
            "pressure_collapse" => {
                "Fixing this should unlock more of the learner's real understanding in exam conditions."
                    .to_string()
            }
            "memory_decay" => {
                "Fixing this should make later revision and transfer much more efficient.".to_string()
            }
            "knowledge_gap" => {
                "Fixing this should unlock several downstream questions that depend on the topic."
                    .to_string()
            }
            _ => "Fixing this should remove a high-impact source of avoidable mark loss.".to_string(),
        });
        ProblemCauseFixCard {
            topic_id: topic_case.topic_id,
            topic_name: topic_case.topic_name.clone(),
            problem_summary,
            cause_summary,
            fix_summary,
            confidence_score: topic_case.diagnosis_certainty,
            impact_score: topic_case.priority_score,
            unlock_summary,
            evidence: json!({
                "primary_hypothesis_code": topic_case.primary_hypothesis_code,
                "requires_probe": topic_case.requires_probe,
                "proof_gaps": topic_case.proof_gaps,
                "open_questions": topic_case.open_questions,
                "recommended_intervention": topic_case.recommended_intervention,
            }),
        }
    }

    fn build_prescription(&self, topic_case: &TopicCase) -> EcoachResult<InterventionPrescription> {
        let (primary_mode_code, support_mode_code, recheck_mode_code) =
            prescription_triplet(topic_case);
        let library = self.list_intervention_modes()?;
        let mut mode_chain = vec![primary_mode_code.to_string()];
        if let Some(mode_code) = support_mode_code {
            mode_chain.push(mode_code.to_string());
        }
        if let Some(mode_code) = recheck_mode_code {
            mode_chain.push(mode_code.to_string());
        }
        let primary_definition = library
            .iter()
            .find(|item| item.mode_code == primary_mode_code)
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "intervention mode {} was not found in the library",
                    primary_mode_code
                ))
            })?;
        let contraindications = primary_definition.contraindications.clone();
        let success_signals = primary_definition.success_signals.clone();
        let confidence_score = clamp_bp(i64::from(
            topic_case
                .diagnosis_certainty
                .max(topic_case.priority_score)
                .max(topic_case.gap_score),
        ));

        Ok(InterventionPrescription {
            topic_id: topic_case.topic_id,
            topic_name: topic_case.topic_name.clone(),
            primary_mode_code: primary_mode_code.to_string(),
            support_mode_code: support_mode_code.map(ToString::to_string),
            recheck_mode_code: recheck_mode_code.map(ToString::to_string),
            mode_chain,
            contraindications,
            success_signals,
            confidence_score,
            payload: json!({
                "recommended_minutes": topic_case.recommended_intervention.recommended_minutes,
                "urgency": topic_case.recommended_intervention.urgency,
                "reason": topic_case.recommended_intervention.reason,
                "primary_hypothesis_code": topic_case.primary_hypothesis_code,
                "proof_gaps": topic_case.proof_gaps,
                "open_questions": topic_case.open_questions,
            }),
        })
    }

    fn persist_problem_card(
        &self,
        diagnostic_id: i64,
        student_id: i64,
        card: &ProblemCauseFixCard,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO diagnostic_problem_cause_fix_cards (
                    diagnostic_id, student_id, topic_id, topic_name, problem_summary,
                    cause_summary, fix_summary, confidence_score_bp, impact_score_bp,
                    unlock_summary, evidence_json, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'))
                 ON CONFLICT(diagnostic_id, topic_id) DO UPDATE SET
                    student_id = excluded.student_id,
                    topic_name = excluded.topic_name,
                    problem_summary = excluded.problem_summary,
                    cause_summary = excluded.cause_summary,
                    fix_summary = excluded.fix_summary,
                    confidence_score_bp = excluded.confidence_score_bp,
                    impact_score_bp = excluded.impact_score_bp,
                    unlock_summary = excluded.unlock_summary,
                    evidence_json = excluded.evidence_json,
                    created_at = datetime('now')",
                params![
                    diagnostic_id,
                    student_id,
                    card.topic_id,
                    card.topic_name,
                    card.problem_summary,
                    card.cause_summary,
                    card.fix_summary,
                    card.confidence_score,
                    card.impact_score,
                    card.unlock_summary,
                    serde_json::to_string(&card.evidence)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn persist_prescription(
        &self,
        diagnostic_id: i64,
        student_id: i64,
        prescription: &InterventionPrescription,
    ) -> EcoachResult<()> {
        self.conn
            .execute(
                "INSERT INTO diagnostic_intervention_prescriptions (
                    diagnostic_id, student_id, topic_id, topic_name, primary_mode_code,
                    support_mode_code, recheck_mode_code, mode_chain_json,
                    contraindications_json, success_signals_json, confidence_score_bp,
                    payload_json, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))
                 ON CONFLICT(diagnostic_id, topic_id) DO UPDATE SET
                    student_id = excluded.student_id,
                    topic_name = excluded.topic_name,
                    primary_mode_code = excluded.primary_mode_code,
                    support_mode_code = excluded.support_mode_code,
                    recheck_mode_code = excluded.recheck_mode_code,
                    mode_chain_json = excluded.mode_chain_json,
                    contraindications_json = excluded.contraindications_json,
                    success_signals_json = excluded.success_signals_json,
                    confidence_score_bp = excluded.confidence_score_bp,
                    payload_json = excluded.payload_json,
                    created_at = datetime('now')",
                params![
                    diagnostic_id,
                    student_id,
                    prescription.topic_id,
                    prescription.topic_name,
                    prescription.primary_mode_code,
                    prescription.support_mode_code,
                    prescription.recheck_mode_code,
                    serde_json::to_string(&prescription.mode_chain)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&prescription.contraindications)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&prescription.success_signals)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    prescription.confidence_score,
                    serde_json::to_string(&prescription.payload)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }
}

fn prescription_triplet(topic_case: &TopicCase) -> (&'static str, Option<&'static str>, Option<&'static str>) {
    if topic_case
        .active_hypotheses
        .iter()
        .any(|item| item.code == "conceptual_confusion")
    {
        ("compare_contrast_drill", Some("misconception_correction_set"), Some("guided_transfer_drill"))
    } else if topic_case.primary_hypothesis_code == "pressure_collapse" {
        ("pressure_ladder", Some("fluency_burst"), Some("stability_recheck_cycle"))
    } else if topic_case.primary_hypothesis_code == "memory_decay" {
        ("recall_probe", Some("stability_recheck_cycle"), Some("guided_transfer_drill"))
    } else if topic_case.primary_hypothesis_code == "knowledge_gap" {
        ("concept_rebuild", Some("translation_scaffold"), Some("guided_transfer_drill"))
    } else if topic_case.primary_hypothesis_code == "execution_drift" {
        ("guided_worked_step_repair", Some("error_diagnosis_drill"), Some("stability_recheck_cycle"))
    } else if topic_case
        .active_hypotheses
        .iter()
        .any(|item| item.code.contains("confidence"))
    {
        ("confidence_reflection_check", Some("misconception_correction_set"), Some("stability_recheck_cycle"))
    } else if topic_case.requires_probe {
        ("mixed_root_repair_set", Some("guided_transfer_drill"), Some("stability_recheck_cycle"))
    } else {
        ("secure_zone_reinforcement", Some("guided_transfer_drill"), Some("stability_recheck_cycle"))
    }
}

#[cfg(test)]
mod tests {
    use super::InterventionLibraryService;
    use ecoach_content::install_sample_pack;
    use ecoach_storage::open_test_database;

    #[test]
    fn intervention_library_lists_seeded_modes_from_runtime_schema() {
        let conn = open_test_database();
        install_sample_pack(&conn);

        let service = InterventionLibraryService::new(&conn);
        let modes = service
            .list_intervention_modes()
            .expect("intervention modes should load");

        assert!(modes.iter().any(|item| item.mode_code == "concept_rebuild"));
        assert!(modes.iter().any(|item| item.mode_code == "pressure_ladder"));
        assert!(modes.iter().any(|item| item.mode_code == "guided_transfer_drill"));
    }
}
