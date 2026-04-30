use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Instant;

use crate::{
    CanonicalIntelligenceStore, CoachIntelligenceDomeService, CoachIntelligenceDomeSnapshot,
    ContentReadinessResolution, ContentReadinessStatus, ReadinessEngine, StudentReadinessSnapshot,
    assess_content_readiness, resolve_next_coach_action,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachEvidenceLedgerEntry {
    pub ledger_code: String,
    pub ledger_label: String,
    pub topic_id: Option<i64>,
    pub evidence_score: BasisPoints,
    pub confidence_score: BasisPoints,
    pub status: String,
    pub summary: String,
    pub details: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureActivationDecision {
    pub feature_code: String,
    pub feature_label: String,
    pub activation_priority_score: BasisPoints,
    pub urgency_score: BasisPoints,
    pub confidence_score: BasisPoints,
    pub readiness_guardrail: String,
    pub rationale: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeacherlessCapabilityReview {
    pub role_code: String,
    pub role_label: String,
    pub capability_score: BasisPoints,
    pub readiness_label: String,
    pub strengths: Vec<String>,
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGovernorSnapshot {
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub readiness_score: BasisPoints,
    pub quality_score: BasisPoints,
    pub provenance_score: BasisPoints,
    pub contradiction_risk_score: BasisPoints,
    pub quality_state: String,
    pub blocking_issues: Vec<String>,
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachJudgmentSnapshot {
    pub student_id: i64,
    pub subject_id: Option<i64>,
    pub topic_id: Option<i64>,
    pub overall_judgment_score: BasisPoints,
    pub judgment_confidence_score: BasisPoints,
    pub independence_band: String,
    pub biggest_risk: String,
    pub next_best_move: String,
    pub evidence_ledger: Vec<CoachEvidenceLedgerEntry>,
    pub feature_activations: Vec<FeatureActivationDecision>,
    pub capability_reviews: Vec<TeacherlessCapabilityReview>,
    pub content_governor: Option<ContentGovernorSnapshot>,
}

#[derive(Debug, Clone, Default)]
struct ImprovementAggregate {
    event_count: i64,
    avg_mastery_delta: i64,
    avg_retention_delta: i64,
    avg_transfer_delta: i64,
    avg_timed_delta: i64,
    confirmed_count: i64,
    challenged_count: i64,
}

#[derive(Debug, Clone)]
struct MotivationSnapshot {
    risk_level: String,
    risk_score_bp: BasisPoints,
    consecutive_misses: i64,
    recent_partial_sessions: i64,
    next_recovery_action: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct PressureSnapshot {
    pressure_collapse_index: BasisPoints,
    average_speed_score: BasisPoints,
    timed_attempt_count: i64,
}

pub struct CoachJudgmentEngine<'a> {
    conn: &'a Connection,
}

impl<'a> CoachJudgmentEngine<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn build_judgment_snapshot(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<CoachJudgmentSnapshot> {
        let total_start = Instant::now();
        eprintln!(
            "[perf][coach.build_judgment_snapshot] enter student_id={} subject_id={:?} topic_id={:?}",
            student_id, subject_id, topic_id
        );
        let dome_start = Instant::now();
        let dome = CoachIntelligenceDomeService::new(self.conn)
            .build_intelligence_dome(student_id, subject_id, topic_id)?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] build_intelligence_dome {:.1}ms intervention_profiles={} evidence_probes={} interference_cases={} surprise_events={} reflections={}",
            dome_start.elapsed().as_secs_f64() * 1000.0,
            dome.intervention_effectiveness.len(),
            dome.best_next_evidence.len(),
            dome.interference_cases.len(),
            dome.surprise_events.len(),
            dome.reflection_cycles.len()
        );
        self.build_judgment_snapshot_from_dome(
            student_id,
            subject_id,
            topic_id,
            &dome,
            total_start,
        )
    }

    pub fn build_judgment_snapshot_with_dome(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        dome: &CoachIntelligenceDomeSnapshot,
    ) -> EcoachResult<CoachJudgmentSnapshot> {
        let total_start = Instant::now();
        eprintln!(
            "[perf][coach.build_judgment_snapshot] enter student_id={} subject_id={:?} topic_id={:?} dome=reused",
            student_id, subject_id, topic_id
        );
        eprintln!(
            "[perf][coach.build_judgment_snapshot] reuse_intelligence_dome intervention_profiles={} evidence_probes={} interference_cases={} surprise_events={} reflections={}",
            dome.intervention_effectiveness.len(),
            dome.best_next_evidence.len(),
            dome.interference_cases.len(),
            dome.surprise_events.len(),
            dome.reflection_cycles.len()
        );
        self.build_judgment_snapshot_from_dome(student_id, subject_id, topic_id, dome, total_start)
    }

    fn build_judgment_snapshot_from_dome(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        dome: &CoachIntelligenceDomeSnapshot,
        total_start: Instant,
    ) -> EcoachResult<CoachJudgmentSnapshot> {
        let resolve_ids_start = Instant::now();
        let resolved_topic_id = topic_id
            .or_else(|| dome.topic_strategy.as_ref().map(|item| item.topic_id))
            .or_else(|| dome.uncertainty_profile.as_ref().map(|item| item.topic_id));
        let mut resolved_subject_id = subject_id
            .or_else(|| dome.topic_strategy.as_ref().map(|item| item.subject_id))
            .or_else(|| {
                dome.uncertainty_profile
                    .as_ref()
                    .map(|item| item.subject_id)
            });
        if resolved_subject_id.is_none() {
            resolved_subject_id = self.lookup_subject_for_topic(resolved_topic_id)?;
        }
        eprintln!(
            "[perf][coach.build_judgment_snapshot] resolve_ids {:.1}ms resolved_subject_id={:?} resolved_topic_id={:?}",
            resolve_ids_start.elapsed().as_secs_f64() * 1000.0,
            resolved_subject_id,
            resolved_topic_id
        );

        let content_start = Instant::now();
        let content_readiness = assess_content_readiness(self.conn, student_id)?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] assess_content_readiness {:.1}ms",
            content_start.elapsed().as_secs_f64() * 1000.0
        );
        let readiness_start = Instant::now();
        let readiness = match resolved_subject_id {
            Some(subject_id) => Some(
                ReadinessEngine::new(self.conn).build_subject_readiness(student_id, subject_id)?,
            ),
            None => None,
        };
        eprintln!(
            "[perf][coach.build_judgment_snapshot] build_subject_readiness {:.1}ms has_value={} topic_slices={}",
            readiness_start.elapsed().as_secs_f64() * 1000.0,
            readiness.is_some(),
            readiness
                .as_ref()
                .map(|item| item.topic_slices.len())
                .unwrap_or(0)
        );
        let next_action_start = Instant::now();
        let next_action = resolve_next_coach_action(self.conn, student_id)?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] resolve_next_coach_action {:.1}ms",
            next_action_start.elapsed().as_secs_f64() * 1000.0
        );
        let content_governor_start = Instant::now();
        let content_governor =
            self.sync_content_governor(resolved_subject_id, resolved_topic_id)?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] sync_content_governor {:.1}ms has_value={} blocking_issues={}",
            content_governor_start.elapsed().as_secs_f64() * 1000.0,
            content_governor.is_some(),
            content_governor
                .as_ref()
                .map(|item| item.blocking_issues.len())
                .unwrap_or(0)
        );
        let improvement_start = Instant::now();
        let improvement = self.load_improvement_aggregate(student_id, resolved_topic_id)?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] load_improvement_aggregate {:.1}ms event_count={}",
            improvement_start.elapsed().as_secs_f64() * 1000.0,
            improvement.event_count
        );
        let motivation_start = Instant::now();
        let motivation = self.load_motivation_snapshot(student_id)?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] load_motivation_snapshot {:.1}ms",
            motivation_start.elapsed().as_secs_f64() * 1000.0
        );
        let pressure_start = Instant::now();
        let pressure =
            self.load_pressure_snapshot(student_id, resolved_topic_id, readiness.as_ref())?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] load_pressure_snapshot {:.1}ms timed_attempt_count={}",
            pressure_start.elapsed().as_secs_f64() * 1000.0,
            pressure.timed_attempt_count
        );
        let evidence_start = Instant::now();
        let evidence_ledger = self.sync_evidence_ledger(
            student_id,
            resolved_subject_id,
            resolved_topic_id,
            &dome,
            &content_readiness,
            readiness.as_ref(),
            content_governor.as_ref(),
            &improvement,
            &motivation,
            &pressure,
        )?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] sync_evidence_ledger {:.1}ms entries={}",
            evidence_start.elapsed().as_secs_f64() * 1000.0,
            evidence_ledger.len()
        );
        let feature_start = Instant::now();
        let feature_activations = self.sync_feature_activations(
            student_id,
            resolved_subject_id,
            resolved_topic_id,
            &dome,
            &content_readiness,
            readiness.as_ref(),
            content_governor.as_ref(),
            &evidence_ledger,
            &motivation,
            next_action.route.as_str(),
            next_action.title.as_str(),
            next_action.subtitle.as_str(),
        )?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] sync_feature_activations {:.1}ms decisions={}",
            feature_start.elapsed().as_secs_f64() * 1000.0,
            feature_activations.len()
        );
        let capability_start = Instant::now();
        let capability_reviews = self.build_capability_reviews(
            &dome,
            &content_readiness,
            readiness.as_ref(),
            content_governor.as_ref(),
            &evidence_ledger,
            &feature_activations,
            &motivation,
        );
        eprintln!(
            "[perf][coach.build_judgment_snapshot] build_capability_reviews {:.1}ms reviews={}",
            capability_start.elapsed().as_secs_f64() * 1000.0,
            capability_reviews.len()
        );
        let scoring_start = Instant::now();
        let overall_judgment_score = clamp_bp(average_bp(
            capability_reviews
                .iter()
                .map(|item| item.capability_score as i64),
        ));
        let judgment_confidence_score = clamp_bp(average_bp(
            evidence_ledger
                .iter()
                .map(|item| item.confidence_score as i64),
        ) as i64);
        let weakest_signal = evidence_ledger
            .iter()
            .min_by_key(|item| item.evidence_score)
            .cloned();
        let biggest_risk = weakest_signal
            .as_ref()
            .map(|item| format!("{}: {}", item.ledger_label, item.summary))
            .unwrap_or_else(|| {
                "The coach still needs more evidence before it can trust this route.".to_string()
            });
        let next_best_move = feature_activations
            .first()
            .map(|item| format!("{}: {}", item.feature_label, item.rationale))
            .unwrap_or_else(|| next_action.title.clone());
        let independence_band = independence_band(overall_judgment_score).to_string();
        eprintln!(
            "[perf][coach.build_judgment_snapshot] score_and_summarize {:.1}ms overall={} confidence={}",
            scoring_start.elapsed().as_secs_f64() * 1000.0,
            overall_judgment_score,
            judgment_confidence_score
        );

        let independence_start = Instant::now();
        self.upsert_independence_review(
            student_id,
            resolved_subject_id,
            resolved_topic_id,
            overall_judgment_score,
            judgment_confidence_score,
            independence_band.as_str(),
            biggest_risk.as_str(),
            next_best_move.as_str(),
            &capability_reviews,
            &json!({
                "next_action_route": next_action.route,
                "content_readiness": format!("{:?}", content_readiness.status),
                "evidence_count": evidence_ledger.len(),
                "feature_count": feature_activations.len(),
            }),
        )?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] upsert_independence_review {:.1}ms",
            independence_start.elapsed().as_secs_f64() * 1000.0
        );

        let snapshot = CoachJudgmentSnapshot {
            student_id,
            subject_id: resolved_subject_id,
            topic_id: resolved_topic_id,
            overall_judgment_score,
            judgment_confidence_score,
            independence_band,
            biggest_risk,
            next_best_move,
            evidence_ledger,
            feature_activations,
            capability_reviews,
            content_governor,
        };
        let risk_start = Instant::now();
        CanonicalIntelligenceStore::new(self.conn).sync_risk_snapshot(&snapshot)?;
        eprintln!(
            "[perf][coach.build_judgment_snapshot] sync_risk_snapshot {:.1}ms",
            risk_start.elapsed().as_secs_f64() * 1000.0
        );
        eprintln!(
            "[perf][coach.build_judgment_snapshot] total {:.1}ms",
            total_start.elapsed().as_secs_f64() * 1000.0
        );
        Ok(snapshot)
    }

    fn lookup_subject_for_topic(&self, topic_id: Option<i64>) -> EcoachResult<Option<i64>> {
        let Some(topic_id) = topic_id else {
            return Ok(None);
        };
        self.conn
            .query_row(
                "SELECT subject_id FROM topics WHERE id = ?1",
                [topic_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }
}

impl<'a> CoachJudgmentEngine<'a> {
    fn sync_evidence_ledger(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        dome: &CoachIntelligenceDomeSnapshot,
        content_readiness: &ContentReadinessResolution,
        readiness: Option<&StudentReadinessSnapshot>,
        content_governor: Option<&ContentGovernorSnapshot>,
        improvement: &ImprovementAggregate,
        motivation: &MotivationSnapshot,
        pressure: &PressureSnapshot,
    ) -> EcoachResult<Vec<CoachEvidenceLedgerEntry>> {
        let mut entries = Vec::new();
        let uncertainty_score = dome
            .uncertainty_profile
            .as_ref()
            .map(|item| item.uncertainty_score)
            .unwrap_or(5_600);
        let false_mastery_risk = dome
            .uncertainty_profile
            .as_ref()
            .map(|item| item.false_mastery_risk)
            .unwrap_or(5_000);
        let diagnostic_score = clamp_bp(average_bp([
            (10_000 - uncertainty_score as i64).max(0),
            (10_000 - false_mastery_risk as i64).max(0),
            dome.topic_strategy
                .as_ref()
                .map(|item| item.plan_confidence_score as i64)
                .unwrap_or(4_800),
        ]));
        entries.push(self.ledger_entry(
            "diagnostic_truth",
            "Diagnostic Truth",
            topic_id,
            diagnostic_score,
            clamp_bp((10_000 - uncertainty_score as i64).max(0)),
            band_status(diagnostic_score),
            if diagnostic_score >= 6_500 {
                "The coach has a stable read on the likely cause of the learner's current failure pattern."
            } else {
                "The coach still needs more proof before it can fully trust the current diagnosis."
            },
            json!({
                "uncertainty_score": uncertainty_score,
                "false_mastery_risk": false_mastery_risk,
                "strategy_mode": dome.topic_strategy.as_ref().map(|item| item.strategy_mode.clone()),
                "primary_hypothesis_code": dome.topic_strategy.as_ref().map(|item| item.primary_hypothesis_code.clone()),
            }),
        ));

        let intervention_score = if dome.intervention_effectiveness.is_empty() {
            4_400
        } else {
            average_bp(dome.intervention_effectiveness.iter().map(|item| {
                ((item.success_rate_score as i64 * 3)
                    + (item.avg_gain_score + 10_000).clamp(0, 10_000))
                    / 4
            }))
        };
        let intervention_score = clamp_bp(intervention_score);
        entries.push(self.ledger_entry(
            "intervention_proof",
            "Intervention Proof",
            topic_id,
            intervention_score,
            clamp_bp(
                average_bp(
                    dome.intervention_effectiveness
                        .iter()
                        .map(|item| item.success_rate_score as i64),
                ) as i64,
            ),
            band_status(intervention_score),
            if intervention_score >= 6_500 {
                "The coach has intervention evidence that the current repair family can move the learner."
            } else {
                "The intervention route still needs stronger evidence before it can be trusted at scale."
            },
            json!({
                "profiles": dome.intervention_effectiveness,
            }),
        ));

        let retention_score = readiness
            .map(|item| {
                let avg_memory = average_bp(
                    item.topic_slices
                        .iter()
                        .map(|slice| slice.memory_strength as i64),
                );
                clamp_bp(avg_memory as i64 - item.due_memory_count * 450)
            })
            .unwrap_or(4_800);
        entries.push(self.ledger_entry(
            "retention_durability",
            "Retention Durability",
            topic_id,
            retention_score,
            clamp_bp((retention_score as i64 + 1_000).min(10_000)),
            band_status(retention_score),
            if retention_score >= 6_300 {
                "Recent memory signals suggest the learner can carry gains forward beyond the current session."
            } else {
                "Retention risk is still high enough that calm-session progress could decay before exam day."
            },
            json!({
                "due_memory_count": readiness.map(|item| item.due_memory_count),
                "due_review_count": readiness.map(|item| item.due_review_count),
                "topic_memory_strengths": readiness.map(|item| item.topic_slices.iter().map(|slice| {
                    json!({
                        "topic_id": slice.topic_id,
                        "memory_strength": slice.memory_strength,
                        "topic_readiness_score": slice.topic_readiness_score,
                    })
                }).collect::<Vec<_>>()),
            }),
        ));

        let pressure_score = clamp_bp(average_bp([
            (10_000 - pressure.pressure_collapse_index as i64).max(0),
            pressure.average_speed_score as i64,
            readiness
                .map(|item| item.readiness_score as i64)
                .unwrap_or(5_200),
        ]));
        entries.push(self.ledger_entry(
            "pressure_readiness",
            "Pressure Readiness",
            topic_id,
            pressure_score,
            clamp_bp((10_000 - pressure.pressure_collapse_index as i64).max(0)),
            band_status(pressure_score),
            if pressure_score >= 6_400 {
                "The learner is increasingly stable when the coach adds time pressure and mixed evidence."
            } else {
                "Pressure still looks capable of breaking apparent mastery, so the route needs conditioning."
            },
            json!({
                "pressure_collapse_index": pressure.pressure_collapse_index,
                "average_speed_score": pressure.average_speed_score,
                "timed_attempt_count": pressure.timed_attempt_count,
                "surprise_events": dome.surprise_events,
            }),
        ));

        let improvement_score = if improvement.event_count == 0 {
            4_200
        } else {
            clamp_bp(average_bp([
                (improvement.avg_mastery_delta + 10_000).clamp(0, 10_000),
                (improvement.avg_retention_delta + 10_000).clamp(0, 10_000),
                (improvement.avg_transfer_delta + 10_000).clamp(0, 10_000),
                (improvement.avg_timed_delta + 10_000).clamp(0, 10_000),
            ]))
        };
        entries.push(self.ledger_entry(
            "improvement_proof",
            "Improvement Proof",
            topic_id,
            improvement_score,
            clamp_bp(
                ((improvement.event_count.min(8) * 1_100)
                    + improvement.confirmed_count * 500
                    - improvement.challenged_count * 250)
                    .clamp(2_500, 9_200),
            ),
            band_status(improvement_score),
            if improvement.event_count == 0 {
                "The coach has not yet accumulated enough evidence events to prove improvement."
            } else if improvement_score >= 6_000 {
                "Evidence events show the learner is improving across more than one signal."
            } else {
                "The improvement signal is still mixed, so the coach should treat apparent gains cautiously."
            },
            json!({
                "event_count": improvement.event_count,
                "avg_mastery_delta": improvement.avg_mastery_delta,
                "avg_retention_delta": improvement.avg_retention_delta,
                "avg_transfer_delta": improvement.avg_transfer_delta,
                "avg_timed_delta": improvement.avg_timed_delta,
                "confirmed_count": improvement.confirmed_count,
                "challenged_count": improvement.challenged_count,
            }),
        ));

        let content_score = content_governor
            .as_ref()
            .map(|item| {
                clamp_bp(average_bp([
                    item.readiness_score as i64,
                    item.quality_score as i64,
                    item.provenance_score as i64,
                    (10_000 - item.contradiction_risk_score as i64).max(0),
                ]))
            })
            .unwrap_or_else(|| match content_readiness.status {
                ContentReadinessStatus::Ready => 6_000,
                ContentReadinessStatus::InsufficientQuestionCoverage => 4_200,
                _ => 2_400,
            });
        entries.push(self.ledger_entry(
            "content_governance",
            "Content Governance",
            topic_id,
            content_score,
            content_governor
                .as_ref()
                .map(|item| item.provenance_score)
                .unwrap_or(4_000),
            band_status(content_score),
            if content_score >= 6_500 {
                "The content layer is strong enough to support the coach's current decisions."
            } else {
                "The coach's decisions are exposed to content thinness or quality risk in this area."
            },
            json!({
                "content_readiness": format!("{:?}", content_readiness.status),
                "content_governor": content_governor,
            }),
        ));

        let exam_score = readiness
            .as_ref()
            .map(|item| item.readiness_score)
            .unwrap_or(4_800);
        entries.push(self.ledger_entry(
            "exam_readiness",
            "Exam Readiness",
            topic_id,
            exam_score,
            clamp_bp(average_bp([
                exam_score as i64,
                (10_000 - false_mastery_risk as i64).max(0),
            ])),
            band_status(exam_score),
            if exam_score >= 6_800 {
                "The learner is beginning to look exam-safe rather than only topic-familiar."
            } else {
                "The learner may be topic-improving, but exam-day reliability is still not proven."
            },
            json!({
                "subject_readiness": readiness,
                "false_mastery_risk": false_mastery_risk,
            }),
        ));

        let motivation_score = clamp_bp((10_000 - motivation.risk_score_bp as i64).max(0));
        entries.push(self.ledger_entry(
            "motivation_persistence",
            "Motivation Persistence",
            topic_id,
            motivation_score,
            clamp_bp((motivation_score as i64 + 600).min(10_000)),
            band_status(motivation_score),
            if motivation_score >= 6_000 {
                "Recent engagement signals suggest the learner can sustain the current effort without outside rescue."
            } else {
                "Persistence risk is high enough that the coach should actively regulate intensity and morale."
            },
            json!({
                "risk_level": motivation.risk_level,
                "risk_score_bp": motivation.risk_score_bp,
                "consecutive_misses": motivation.consecutive_misses,
                "recent_partial_sessions": motivation.recent_partial_sessions,
                "next_recovery_action": motivation.next_recovery_action,
            }),
        ));

        for entry in &entries {
            self.upsert_evidence_ledger_entry(student_id, subject_id, topic_id, entry)?;
        }

        Ok(entries)
    }

    fn sync_feature_activations(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        dome: &CoachIntelligenceDomeSnapshot,
        content_readiness: &ContentReadinessResolution,
        readiness: Option<&StudentReadinessSnapshot>,
        content_governor: Option<&ContentGovernorSnapshot>,
        evidence_ledger: &[CoachEvidenceLedgerEntry],
        motivation: &MotivationSnapshot,
        next_action_route: &str,
        next_action_title: &str,
        next_action_subtitle: &str,
    ) -> EcoachResult<Vec<FeatureActivationDecision>> {
        let mut decisions = Vec::new();
        push_feature_decision(
            &mut decisions,
            FeatureActivationDecision {
                feature_code: "coach_next_action".to_string(),
                feature_label: next_action_title.to_string(),
                activation_priority_score: 6_000,
                urgency_score: 5_800,
                confidence_score: 6_200,
                readiness_guardrail: "follow_state_machine".to_string(),
                rationale: next_action_subtitle.to_string(),
                payload: json!({ "route": next_action_route }),
            },
        );

        if content_readiness.status != ContentReadinessStatus::Ready {
            push_feature_decision(
                &mut decisions,
                FeatureActivationDecision {
                    feature_code: "content_resolution".to_string(),
                    feature_label: "Resolve Content Gaps".to_string(),
                    activation_priority_score: 9_100,
                    urgency_score: 8_800,
                    confidence_score: 8_300,
                    readiness_guardrail: "block_high_stakes_coaching".to_string(),
                    rationale: "The coach should not pretend to be independent while core content coverage is still thin.".to_string(),
                    payload: json!({
                        "content_status": format!("{:?}", content_readiness.status),
                        "subject_codes": content_readiness.subject_codes,
                    }),
                },
            );
        }

        let diagnostic_truth = evidence_score(evidence_ledger, "diagnostic_truth");
        if diagnostic_truth < 6_000 && !dome.best_next_evidence.is_empty() {
            push_feature_decision(
                &mut decisions,
                FeatureActivationDecision {
                    feature_code: "best_next_evidence_probe".to_string(),
                    feature_label: "Run Best-Next-Evidence Probe".to_string(),
                    activation_priority_score: 8_900,
                    urgency_score: 8_600,
                    confidence_score: 8_000,
                    readiness_guardrail: "do_not_scale_current_plan".to_string(),
                    rationale: "The coach still needs discriminating evidence before it should trust the current diagnosis.".to_string(),
                    payload: json!({
                        "probes": dome.best_next_evidence,
                    }),
                },
            );
        }

        let pressure_score = evidence_score(evidence_ledger, "pressure_readiness");
        if pressure_score < 6_000 {
            push_feature_decision(
                &mut decisions,
                FeatureActivationDecision {
                    feature_code: "pressure_conditioning".to_string(),
                    feature_label: "Pressure Conditioning Round".to_string(),
                    activation_priority_score: 8_300,
                    urgency_score: 8_100,
                    confidence_score: 7_300,
                    readiness_guardrail: "validate_under_time".to_string(),
                    rationale: "Calm mastery still looks vulnerable under speed or mixed pressure."
                        .to_string(),
                    payload: json!({
                        "surprise_events": dome.surprise_events,
                    }),
                },
            );
        }

        let retention_score = evidence_score(evidence_ledger, "retention_durability");
        if retention_score < 6_000 {
            push_feature_decision(
                &mut decisions,
                FeatureActivationDecision {
                    feature_code: "retention_reactivation".to_string(),
                    feature_label: "Retention Reactivation".to_string(),
                    activation_priority_score: 8_100,
                    urgency_score: 7_700,
                    confidence_score: 7_100,
                    readiness_guardrail: "do_not_assume_improvement_will_hold".to_string(),
                    rationale:
                        "Recent gains still look too fragile to trust without resurfacing them."
                            .to_string(),
                    payload: json!({
                        "readiness": readiness,
                    }),
                },
            );
        }

        let intervention_score = evidence_score(evidence_ledger, "intervention_proof");
        if intervention_score < 5_800 {
            push_feature_decision(
                &mut decisions,
                FeatureActivationDecision {
                    feature_code: "teach_rebuild".to_string(),
                    feature_label: "Teach Rebuild".to_string(),
                    activation_priority_score: 7_900,
                    urgency_score: 7_200,
                    confidence_score: 6_900,
                    readiness_guardrail: "reopen_the_case".to_string(),
                    rationale: "The current intervention family still lacks proof, so the coach should teach and retest rather than assume.".to_string(),
                    payload: json!({
                        "topic_strategy": dome.topic_strategy,
                        "intervention_effectiveness": dome.intervention_effectiveness,
                    }),
                },
            );
        }

        if motivation.risk_score_bp >= 5_600 {
            push_feature_decision(
                &mut decisions,
                FeatureActivationDecision {
                    feature_code: "recovery_support".to_string(),
                    feature_label: "Recovery Support".to_string(),
                    activation_priority_score: 7_700,
                    urgency_score: 7_500,
                    confidence_score: 7_400,
                    readiness_guardrail: "protect_persistence".to_string(),
                    rationale: "The learner's persistence risk is high enough that the coach should regulate load before pushing harder.".to_string(),
                    payload: json!({
                        "risk_level": motivation.risk_level,
                        "next_recovery_action": motivation.next_recovery_action,
                    }),
                },
            );
        }

        if evidence_score(evidence_ledger, "exam_readiness") >= 6_800 && pressure_score >= 6_000 {
            push_feature_decision(
                &mut decisions,
                FeatureActivationDecision {
                    feature_code: "exam_simulation".to_string(),
                    feature_label: "Exam Simulation".to_string(),
                    activation_priority_score: 7_600,
                    urgency_score: 6_900,
                    confidence_score: 7_200,
                    readiness_guardrail: "run_full_exam_conditions".to_string(),
                    rationale: "The learner is strong enough that the next highest-value proof is exam-condition rehearsal.".to_string(),
                    payload: json!({
                        "readiness_score": readiness.map(|item| item.readiness_score),
                        "recommended_mock_blueprint": readiness.map(|item| item.recommended_mock_blueprint.clone()),
                    }),
                },
            );
        }

        if let Some(governor) = content_governor {
            if governor.quality_score < 6_000 || !governor.blocking_issues.is_empty() {
                push_feature_decision(
                    &mut decisions,
                    FeatureActivationDecision {
                        feature_code: "content_governor_refresh".to_string(),
                        feature_label: "Content Governor Refresh".to_string(),
                        activation_priority_score: 7_500,
                        urgency_score: 7_100,
                        confidence_score: 7_000,
                        readiness_guardrail: "stabilize_source_truth".to_string(),
                        rationale: "The coach should repair local content quality before leaning harder on it.".to_string(),
                        payload: json!({
                            "quality_state": governor.quality_state,
                            "blocking_issues": governor.blocking_issues,
                        }),
                    },
                );
            }
        }

        decisions.sort_by(|left, right| {
            right
                .activation_priority_score
                .cmp(&left.activation_priority_score)
                .then(right.urgency_score.cmp(&left.urgency_score))
        });
        decisions.truncate(5);

        for decision in &decisions {
            self.upsert_feature_activation(student_id, subject_id, topic_id, decision)?;
        }

        Ok(decisions)
    }

    fn build_capability_reviews(
        &self,
        dome: &CoachIntelligenceDomeSnapshot,
        content_readiness: &ContentReadinessResolution,
        readiness: Option<&StudentReadinessSnapshot>,
        content_governor: Option<&ContentGovernorSnapshot>,
        evidence_ledger: &[CoachEvidenceLedgerEntry],
        feature_activations: &[FeatureActivationDecision],
        motivation: &MotivationSnapshot,
    ) -> Vec<TeacherlessCapabilityReview> {
        let teacher_score = clamp_bp(average_bp([
            evidence_score(evidence_ledger, "content_governance") as i64,
            content_governor
                .as_ref()
                .map(|item| item.quality_score as i64)
                .unwrap_or(4_500),
            content_governor
                .as_ref()
                .map(|item| item.readiness_score as i64)
                .unwrap_or(4_800),
        ]));
        let diagnostician_score = evidence_score(evidence_ledger, "diagnostic_truth");
        let tutor_score = clamp_bp(average_bp([
            evidence_score(evidence_ledger, "intervention_proof") as i64,
            dome.topic_strategy
                .as_ref()
                .map(|item| item.plan_confidence_score as i64)
                .unwrap_or(5_000),
        ]));
        let planner_score = clamp_bp(average_bp([
            evidence_score(evidence_ledger, "exam_readiness") as i64,
            if feature_activations.is_empty() {
                4_000
            } else {
                7_200
            },
            readiness
                .map(|item| {
                    if item.plan_rewrite_needed {
                        4_800
                    } else {
                        7_100
                    }
                })
                .unwrap_or(5_000),
        ]));
        let examiner_score = clamp_bp(average_bp([
            evidence_score(evidence_ledger, "pressure_readiness") as i64,
            evidence_score(evidence_ledger, "exam_readiness") as i64,
        ]));
        let memory_score = evidence_score(evidence_ledger, "retention_durability");
        let motivator_score = evidence_score(evidence_ledger, "motivation_persistence");
        let proof_score = clamp_bp(average_bp([
            evidence_score(evidence_ledger, "improvement_proof") as i64,
            evidence_score(evidence_ledger, "exam_readiness") as i64,
            evidence_score(evidence_ledger, "diagnostic_truth") as i64,
        ]));

        vec![
            capability_review(
                "teacher",
                "Teacher",
                teacher_score,
                vec![
                    format!("content status: {:?}", content_readiness.status).to_lowercase(),
                    "resource quality is being audited continuously".to_string(),
                ],
                content_governor
                    .as_ref()
                    .map(|item| item.blocking_issues.clone())
                    .unwrap_or_default(),
            ),
            capability_review(
                "diagnostician",
                "Diagnostician",
                diagnostician_score,
                vec![
                    "topic cases are backed by explicit uncertainty handling".to_string(),
                    format!(
                        "{} evidence probes are available",
                        dome.best_next_evidence.len()
                    ),
                ],
                vec![risk_from_score(
                    diagnostician_score,
                    "diagnosis still needs more causal proof",
                )],
            ),
            capability_review(
                "tutor",
                "Tutor",
                tutor_score,
                vec![
                    dome.topic_strategy
                        .as_ref()
                        .map(|item| format!("strategy mode: {}", item.strategy_mode))
                        .unwrap_or_else(|| "strategy mode not resolved".to_string()),
                    format!(
                        "{} intervention profiles are available",
                        dome.intervention_effectiveness.len()
                    ),
                ],
                vec![risk_from_score(
                    tutor_score,
                    "repair choices still lack enough measured proof",
                )],
            ),
            capability_review(
                "study_planner",
                "Study Planner",
                planner_score,
                vec![
                    format!(
                        "{} feature activations have been prioritized",
                        feature_activations.len()
                    ),
                    readiness
                        .map(|item| format!("readiness band: {}", item.readiness_band))
                        .unwrap_or_else(|| "subject readiness unavailable".to_string()),
                ],
                vec![risk_from_score(
                    planner_score,
                    "the next move is still too reactive instead of campaign-stable",
                )],
            ),
            capability_review(
                "examiner",
                "Examiner",
                examiner_score,
                vec![
                    format!(
                        "{} surprise events are available",
                        dome.surprise_events.len()
                    ),
                    "pressure evidence is tracked separately from calm accuracy".to_string(),
                ],
                vec![risk_from_score(
                    examiner_score,
                    "exam-condition stability is still weaker than calm-topic stability",
                )],
            ),
            capability_review(
                "memory_trainer",
                "Memory Trainer",
                memory_score,
                vec![
                    "retention durability is being checked explicitly".to_string(),
                    readiness
                        .map(|item| format!("due memory count: {}", item.due_memory_count))
                        .unwrap_or_else(|| "due memory count unavailable".to_string()),
                ],
                vec![risk_from_score(
                    memory_score,
                    "progress may decay before exam day",
                )],
            ),
            capability_review(
                "motivator",
                "Motivator",
                motivator_score,
                vec![
                    format!("engagement risk level: {}", motivation.risk_level),
                    motivation
                        .next_recovery_action
                        .clone()
                        .unwrap_or_else(|| "no recovery action queued".to_string()),
                ],
                vec![risk_from_score(
                    motivator_score,
                    "the learner may still need stronger pacing and morale regulation",
                )],
            ),
            capability_review(
                "proof_system",
                "Proof System",
                proof_score,
                vec![
                    format!(
                        "{} ledger entries back the current judgment",
                        evidence_ledger.len()
                    ),
                    "improvement and exam readiness are both tracked".to_string(),
                ],
                vec![risk_from_score(
                    proof_score,
                    "the coach cannot yet prove independence with enough confidence",
                )],
            ),
        ]
    }
}

impl<'a> CoachJudgmentEngine<'a> {
    fn sync_content_governor(
        &self,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<Option<ContentGovernorSnapshot>> {
        if subject_id.is_none() && topic_id.is_none() {
            return Ok(None);
        }

        let package_snapshot = match topic_id {
            Some(topic_id) => self
                .conn
                .query_row(
                    "SELECT resource_readiness_score, quality_score, evidence_score,
                            live_health_state, missing_components_json
                     FROM topic_package_snapshots
                     WHERE topic_id = ?1",
                    [topic_id],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, i64>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                        ))
                    },
                )
                .optional()
                .map_err(|err| EcoachError::Storage(err.to_string()))?,
            None => None,
        };

        let provenance_score = self.compute_provenance_score(subject_id, topic_id)?;
        let contradiction_risk_score = self.compute_contradiction_risk(subject_id, topic_id)?;
        let (readiness_score, quality_score, quality_state, mut blocking_issues, evidence_json) =
            if let Some((
                resource_readiness_score,
                quality_score,
                evidence_score,
                live_health_state,
                missing_components_json,
            )) = package_snapshot
            {
                let mut issues = parse_string_vec(missing_components_json.as_str());
                if contradiction_risk_score >= 4_800 {
                    issues.push("recent content quality reports still need review".to_string());
                }
                (
                    clamp_bp(resource_readiness_score),
                    clamp_bp(quality_score),
                    live_health_state,
                    issues,
                    json!({ "package_evidence_score": evidence_score }),
                )
            } else {
                let fallback = self.compute_fallback_content_scores(topic_id)?;
                (
                    fallback.0,
                    fallback.1,
                    if fallback.1 >= 6_500 {
                        "stable".to_string()
                    } else {
                        "watch".to_string()
                    },
                    fallback.2,
                    json!({ "fallback": true }),
                )
            };

        if provenance_score < 5_500 {
            blocking_issues.push(
                "source provenance is still too weak for a fully independent coach".to_string(),
            );
        }

        let snapshot = ContentGovernorSnapshot {
            subject_id,
            topic_id,
            readiness_score,
            quality_score,
            provenance_score,
            contradiction_risk_score,
            quality_state,
            blocking_issues,
            evidence: evidence_json,
        };
        self.upsert_content_governor_snapshot(&snapshot)?;
        Ok(Some(snapshot))
    }

    fn compute_provenance_score(
        &self,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<BasisPoints> {
        let (resource_count, teacher_verified_count, sourced_count, confidence_avg): (
            i64,
            i64,
            i64,
            i64,
        ) = self
            .conn
            .query_row(
                "SELECT COUNT(*),
                        COALESCE(SUM(CASE WHEN teacher_verified = 1 THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN COALESCE(source, '') != '' THEN 1 ELSE 0 END), 0),
                        COALESCE(CAST(AVG(CASE confidence_tier
                            WHEN 'teacher_authored' THEN 10000
                            WHEN 'syllabus_aligned' THEN 9000
                            WHEN 'performance_tested' THEN 8500
                            WHEN 'partially_matched' THEN 6500
                            WHEN 'inferred' THEN 5200
                            ELSE 4200
                        END) AS INTEGER), 0)
                 FROM resource_metadata_index
                 WHERE (?1 IS NULL OR topic_id = ?1)
                   AND (?2 IS NULL OR subject_id = ?2)",
                params![topic_id, subject_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if resource_count == 0 {
            return Ok(3_200);
        }
        Ok(clamp_bp(average_bp([
            (teacher_verified_count * 10_000 / resource_count).clamp(0, 10_000),
            (sourced_count * 10_000 / resource_count).clamp(0, 10_000),
            confidence_avg,
        ])))
    }

    fn compute_contradiction_risk(
        &self,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
    ) -> EcoachResult<BasisPoints> {
        let blocking_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM content_quality_reports reports
                 JOIN content_publish_jobs jobs ON jobs.id = reports.publish_job_id
                 WHERE reports.status IN ('fail', 'needs_review')
                   AND (?1 IS NULL OR jobs.topic_id = ?1)
                   AND (?2 IS NULL OR jobs.subject_id = ?2)",
                params![topic_id, subject_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(clamp_bp((blocking_count * 2_000).clamp(0, 10_000)))
    }

    fn compute_fallback_content_scores(
        &self,
        topic_id: Option<i64>,
    ) -> EcoachResult<(BasisPoints, BasisPoints, Vec<String>)> {
        let Some(topic_id) = topic_id else {
            return Ok((
                4_500,
                4_800,
                vec!["topic content snapshot has not been generated yet".to_string()],
            ));
        };
        let (
            question_count,
            explanation_count,
            glossary_count,
            formula_count,
            worked_example_count,
            resource_count,
            teacher_verified_count,
        ): (i64, i64, i64, i64, i64, i64, i64) = self
            .conn
            .query_row(
                "SELECT
                    (SELECT COUNT(*) FROM questions WHERE topic_id = ?1 AND is_active = 1),
                    (SELECT COUNT(*) FROM knowledge_entries WHERE topic_id = ?1 AND entry_type = 'explanation' AND status = 'active'),
                    (SELECT COUNT(*) FROM knowledge_entries WHERE topic_id = ?1 AND entry_type = 'definition' AND status = 'active'),
                    (SELECT COUNT(*) FROM knowledge_entries WHERE topic_id = ?1 AND entry_type = 'formula' AND status = 'active'),
                    (SELECT COUNT(*) FROM knowledge_entries WHERE topic_id = ?1 AND entry_type = 'worked_example' AND status = 'active'),
                    (SELECT COUNT(*) FROM resource_metadata_index WHERE topic_id = ?1),
                    (SELECT COALESCE(SUM(CASE WHEN teacher_verified = 1 THEN 1 ELSE 0 END), 0) FROM resource_metadata_index WHERE topic_id = ?1)",
                [topic_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut issues = Vec::new();
        if question_count == 0 {
            issues.push("question coverage is missing".to_string());
        }
        if explanation_count == 0 {
            issues.push("no active explanations are available".to_string());
        }
        if worked_example_count == 0 {
            issues.push("no worked examples are available".to_string());
        }
        let readiness_score = clamp_bp(
            (if question_count > 0 { 2_500 } else { 0 })
                + (if explanation_count > 0 { 2_100 } else { 0 })
                + (if glossary_count > 0 { 1_100 } else { 0 })
                + (if formula_count > 0 { 600 } else { 0 })
                + (if worked_example_count > 0 { 1_700 } else { 0 })
                + (if resource_count >= 4 {
                    1_400
                } else {
                    resource_count * 300
                }),
        );
        let quality_score = clamp_bp(average_bp([
            readiness_score as i64,
            if resource_count > 0 {
                (teacher_verified_count * 10_000 / resource_count).clamp(0, 10_000)
            } else {
                4_000
            },
        ]));
        Ok((readiness_score, quality_score, issues))
    }

    fn load_improvement_aggregate(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
    ) -> EcoachResult<ImprovementAggregate> {
        self.conn
            .query_row(
                "SELECT COUNT(*),
                        COALESCE(CAST(AVG(mastery_delta) AS INTEGER), 0),
                        COALESCE(CAST(AVG(retention_delta) AS INTEGER), 0),
                        COALESCE(CAST(AVG(transfer_delta) AS INTEGER), 0),
                        COALESCE(CAST(AVG(timed_delta) AS INTEGER), 0),
                        COALESCE(SUM(CASE WHEN hypothesis_result = 'confirmed' THEN 1 ELSE 0 END), 0),
                        COALESCE(SUM(CASE WHEN hypothesis_result = 'challenged' THEN 1 ELSE 0 END), 0)
                 FROM evidence_events
                 WHERE student_id = ?1
                   AND (?2 IS NULL OR topic_id = ?2)",
                params![student_id, topic_id],
                |row| {
                    Ok(ImprovementAggregate {
                        event_count: row.get(0)?,
                        avg_mastery_delta: row.get(1)?,
                        avg_retention_delta: row.get(2)?,
                        avg_transfer_delta: row.get(3)?,
                        avg_timed_delta: row.get(4)?,
                        confirmed_count: row.get(5)?,
                        challenged_count: row.get(6)?,
                    })
                },
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn load_motivation_snapshot(&self, student_id: i64) -> EcoachResult<MotivationSnapshot> {
        let snapshot = self
            .conn
            .query_row(
                "SELECT risk_level, risk_score_bp, consecutive_misses, recent_partial_sessions, next_recovery_action
                 FROM engagement_risk_profiles
                 WHERE learner_id = ?1",
                [student_id],
                |row| {
                    Ok(MotivationSnapshot {
                        risk_level: row.get(0)?,
                        risk_score_bp: clamp_bp(row.get::<_, i64>(1)?),
                        consecutive_misses: row.get(2)?,
                        recent_partial_sessions: row.get(3)?,
                        next_recovery_action: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(snapshot.unwrap_or(MotivationSnapshot {
            risk_level: "medium".to_string(),
            risk_score_bp: 4_000,
            consecutive_misses: 0,
            recent_partial_sessions: 0,
            next_recovery_action: Some(
                "Keep sessions short until the coach sees a stable rhythm again.".to_string(),
            ),
        }))
    }

    fn load_pressure_snapshot(
        &self,
        student_id: i64,
        topic_id: Option<i64>,
        readiness: Option<&StudentReadinessSnapshot>,
    ) -> EcoachResult<PressureSnapshot> {
        let (pressure_collapse_index, average_speed_score, timed_attempt_count): (i64, i64, i64) =
            self.conn
                .query_row(
                    "SELECT
                        COALESCE(CAST(AVG(sts.pressure_collapse_index) AS INTEGER), 5000),
                        COALESCE(CAST(AVG(COALESCE(sts.speed_score, 5000)) AS INTEGER), 5000),
                        COALESCE((
                            SELECT COUNT(*)
                            FROM student_question_attempts sqa
                            JOIN questions q ON q.id = sqa.question_id
                            WHERE sqa.student_id = ?1
                              AND sqa.was_timed = 1
                              AND (?2 IS NULL OR q.topic_id = ?2)
                        ), 0)
                     FROM student_topic_states sts
                     WHERE sts.student_id = ?1
                       AND (?2 IS NULL OR sts.topic_id = ?2)",
                    params![student_id, topic_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let readiness_speed_floor = readiness
            .map(|item| {
                average_bp(
                    item.topic_slices
                        .iter()
                        .map(|slice| (10_000 - slice.fragility_score as i64).max(0)),
                )
            })
            .unwrap_or(5_200);
        Ok(PressureSnapshot {
            pressure_collapse_index: clamp_bp(pressure_collapse_index),
            average_speed_score: clamp_bp(average_bp([average_speed_score, readiness_speed_floor])),
            timed_attempt_count,
        })
    }

    fn upsert_evidence_ledger_entry(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        entry: &CoachEvidenceLedgerEntry,
    ) -> EcoachResult<()> {
        let subject_key = subject_id.unwrap_or(0);
        let topic_key = topic_id.unwrap_or(0);
        self.conn
            .execute(
                "INSERT INTO coach_evidence_ledger_entries (
                    student_id, subject_key, subject_id, topic_key, topic_id,
                    ledger_code, ledger_label, evidence_score, confidence_score,
                    status, summary, details_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))
                 ON CONFLICT(student_id, subject_key, topic_key, ledger_code) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_id = excluded.topic_id,
                    ledger_label = excluded.ledger_label,
                    evidence_score = excluded.evidence_score,
                    confidence_score = excluded.confidence_score,
                    status = excluded.status,
                    summary = excluded.summary,
                    details_json = excluded.details_json,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    subject_key,
                    subject_id,
                    topic_key,
                    topic_id,
                    entry.ledger_code,
                    entry.ledger_label,
                    entry.evidence_score,
                    entry.confidence_score,
                    entry.status,
                    entry.summary,
                    serde_json::to_string(&entry.details)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn upsert_feature_activation(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        decision: &FeatureActivationDecision,
    ) -> EcoachResult<()> {
        let subject_key = subject_id.unwrap_or(0);
        let topic_key = topic_id.unwrap_or(0);
        self.conn
            .execute(
                "INSERT INTO coach_feature_activations (
                    student_id, subject_key, subject_id, topic_key, topic_id,
                    feature_code, feature_label, activation_priority_score,
                    urgency_score, confidence_score, readiness_guardrail,
                    rationale, payload_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, datetime('now'))
                 ON CONFLICT(student_id, subject_key, topic_key, feature_code) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_id = excluded.topic_id,
                    feature_label = excluded.feature_label,
                    activation_priority_score = excluded.activation_priority_score,
                    urgency_score = excluded.urgency_score,
                    confidence_score = excluded.confidence_score,
                    readiness_guardrail = excluded.readiness_guardrail,
                    rationale = excluded.rationale,
                    payload_json = excluded.payload_json,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    subject_key,
                    subject_id,
                    topic_key,
                    topic_id,
                    decision.feature_code,
                    decision.feature_label,
                    decision.activation_priority_score,
                    decision.urgency_score,
                    decision.confidence_score,
                    decision.readiness_guardrail,
                    decision.rationale,
                    serde_json::to_string(&decision.payload)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn upsert_content_governor_snapshot(
        &self,
        snapshot: &ContentGovernorSnapshot,
    ) -> EcoachResult<()> {
        let subject_key = snapshot.subject_id.unwrap_or(0);
        let topic_key = snapshot.topic_id.unwrap_or(0);
        self.conn
            .execute(
                "INSERT INTO coach_content_governor_snapshots (
                    subject_key, subject_id, topic_key, topic_id,
                    readiness_score, quality_score, provenance_score,
                    contradiction_risk_score, quality_state, blocking_issues_json,
                    evidence_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'))
                 ON CONFLICT(subject_key, topic_key) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_id = excluded.topic_id,
                    readiness_score = excluded.readiness_score,
                    quality_score = excluded.quality_score,
                    provenance_score = excluded.provenance_score,
                    contradiction_risk_score = excluded.contradiction_risk_score,
                    quality_state = excluded.quality_state,
                    blocking_issues_json = excluded.blocking_issues_json,
                    evidence_json = excluded.evidence_json,
                    updated_at = datetime('now')",
                params![
                    subject_key,
                    snapshot.subject_id,
                    topic_key,
                    snapshot.topic_id,
                    snapshot.readiness_score,
                    snapshot.quality_score,
                    snapshot.provenance_score,
                    snapshot.contradiction_risk_score,
                    snapshot.quality_state,
                    serde_json::to_string(&snapshot.blocking_issues)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(&snapshot.evidence)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn upsert_independence_review(
        &self,
        student_id: i64,
        subject_id: Option<i64>,
        topic_id: Option<i64>,
        overall_score: BasisPoints,
        judgment_confidence_score: BasisPoints,
        independence_band: &str,
        biggest_risk: &str,
        next_best_move: &str,
        capability_reviews: &[TeacherlessCapabilityReview],
        summary: &Value,
    ) -> EcoachResult<()> {
        let subject_key = subject_id.unwrap_or(0);
        let topic_key = topic_id.unwrap_or(0);
        self.conn
            .execute(
                "INSERT INTO coach_independence_reviews (
                    student_id, subject_key, subject_id, topic_key, topic_id,
                    overall_score, judgment_confidence_score, independence_band,
                    biggest_risk, next_best_move, capability_json, summary_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))
                 ON CONFLICT(student_id, subject_key, topic_key) DO UPDATE SET
                    subject_id = excluded.subject_id,
                    topic_id = excluded.topic_id,
                    overall_score = excluded.overall_score,
                    judgment_confidence_score = excluded.judgment_confidence_score,
                    independence_band = excluded.independence_band,
                    biggest_risk = excluded.biggest_risk,
                    next_best_move = excluded.next_best_move,
                    capability_json = excluded.capability_json,
                    summary_json = excluded.summary_json,
                    updated_at = datetime('now')",
                params![
                    student_id,
                    subject_key,
                    subject_id,
                    topic_key,
                    topic_id,
                    overall_score,
                    judgment_confidence_score,
                    independence_band,
                    biggest_risk,
                    next_best_move,
                    serde_json::to_string(capability_reviews)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                    serde_json::to_string(summary)
                        .map_err(|err| EcoachError::Serialization(err.to_string()))?,
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(())
    }

    fn ledger_entry(
        &self,
        ledger_code: &str,
        ledger_label: &str,
        topic_id: Option<i64>,
        evidence_score: BasisPoints,
        confidence_score: BasisPoints,
        status: &str,
        summary: &str,
        details: Value,
    ) -> CoachEvidenceLedgerEntry {
        CoachEvidenceLedgerEntry {
            ledger_code: ledger_code.to_string(),
            ledger_label: ledger_label.to_string(),
            topic_id,
            evidence_score,
            confidence_score,
            status: status.to_string(),
            summary: summary.to_string(),
            details,
        }
    }
}

fn average_bp<I>(values: I) -> i64
where
    I: IntoIterator<Item = i64>,
{
    let collected = values.into_iter().collect::<Vec<_>>();
    if collected.is_empty() {
        return 0;
    }
    collected.iter().sum::<i64>() / collected.len() as i64
}

fn band_status(score: BasisPoints) -> &'static str {
    match score {
        0..=3999 => "critical",
        4000..=5499 => "watch",
        5500..=6999 => "stable",
        _ => "strong",
    }
}

fn independence_band(score: BasisPoints) -> &'static str {
    match score {
        0..=4499 => "fragile",
        4500..=5999 => "emerging",
        6000..=7499 => "credible",
        _ => "independent",
    }
}

fn parse_string_vec(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

fn evidence_score(entries: &[CoachEvidenceLedgerEntry], code: &str) -> BasisPoints {
    entries
        .iter()
        .find(|item| item.ledger_code == code)
        .map(|item| item.evidence_score)
        .unwrap_or(0)
}

fn push_feature_decision(
    decisions: &mut Vec<FeatureActivationDecision>,
    decision: FeatureActivationDecision,
) {
    if decisions
        .iter()
        .any(|item| item.feature_code == decision.feature_code)
    {
        return;
    }
    decisions.push(decision);
}

fn capability_review(
    role_code: &str,
    role_label: &str,
    capability_score: BasisPoints,
    strengths: Vec<String>,
    risks: Vec<String>,
) -> TeacherlessCapabilityReview {
    TeacherlessCapabilityReview {
        role_code: role_code.to_string(),
        role_label: role_label.to_string(),
        capability_score,
        readiness_label: independence_band(capability_score).to_string(),
        strengths: strengths
            .into_iter()
            .filter(|item| !item.trim().is_empty())
            .collect(),
        risks: risks
            .into_iter()
            .filter(|item| !item.trim().is_empty())
            .collect(),
    }
}

fn risk_from_score(score: BasisPoints, message: &str) -> String {
    if score >= 6_400 {
        String::new()
    } else {
        message.to_string()
    }
}

#[cfg(test)]
mod tests {
    use ecoach_content::PackService;
    use ecoach_storage::run_runtime_migrations;
    use rusqlite::{Connection, params};

    use super::*;

    #[test]
    fn judgment_engine_surfaces_teacherless_coach_layers() {
        let mut conn = Connection::open_in_memory().expect("in-memory db should open");
        run_runtime_migrations(&mut conn).expect("migrations should apply");
        PackService::new(&conn)
            .install_pack(&sample_pack_path())
            .expect("sample pack should install");
        let primary_topic_id = seed_judgment_state(&conn);

        let snapshot = CoachJudgmentEngine::new(&conn)
            .build_judgment_snapshot(1, Some(1), Some(primary_topic_id))
            .expect("judgment snapshot should build");

        assert!(snapshot.overall_judgment_score > 0);
        assert!(!snapshot.evidence_ledger.is_empty());
        assert!(!snapshot.feature_activations.is_empty());
        assert!(snapshot.capability_reviews.len() >= 8);
        assert!(snapshot.content_governor.is_some());
        let risk_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ic_risk_assessments WHERE learner_id = 1",
                [],
                |row| row.get(0),
            )
            .expect("idea28 risk rows should query");
        assert!(risk_count >= 1);
    }

    fn sample_pack_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("packs")
            .join("math-bece-sample")
    }

    fn seed_judgment_state(conn: &Connection) -> i64 {
        conn.execute(
            "INSERT INTO accounts (
                id, account_type, display_name, pin_hash, pin_salt, entitlement_tier
             ) VALUES (1, 'student', 'Ama', 'hash', 'salt', 'standard')",
            [],
        )
        .expect("student account should insert");
        conn.execute(
            "INSERT INTO student_profiles (
                account_id, exam_target, exam_target_date, preferred_subjects
             ) VALUES (1, 'BECE', date('now', '+90 day'), '[\"MTH\"]')",
            [],
        )
        .expect("student profile should insert");
        let topic_ids = {
            let mut stmt = conn
                .prepare(
                    "SELECT id
                     FROM topics
                     WHERE subject_id = 1
                     ORDER BY id ASC
                     LIMIT 2",
                )
                .expect("topic query should prepare");
            let rows = stmt
                .query_map([], |row| row.get::<_, i64>(0))
                .expect("topic rows should load");
            rows.collect::<Result<Vec<_>, _>>()
                .expect("topics should collect")
        };
        let primary_topic_id = topic_ids[0];
        let neighbor_topic_id = topic_ids[1];
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, accuracy_score,
                transfer_score, consistency_score, gap_score, priority_score,
                fragility_score, pressure_collapse_index, speed_score,
                total_attempts, correct_attempts, recent_attempts_window,
                recent_correct_window, evidence_count, decay_risk,
                memory_strength, repair_priority, recognition_strength,
                explicit_form_accuracy_bp, disguised_form_accuracy_bp, recognition_gap_bp
             ) VALUES (
                1, ?1, 6800, 'fragile', 6100,
                4200, 5600, 3800, 8200,
                6000, 6600, 4700,
                8, 5, 4,
                2, 8, 5500,
                4100, 7600, 7200,
                7500, 4300, 3200
             )",
            [primary_topic_id],
        )
        .expect("topic state should insert");
        conn.execute(
            "INSERT INTO student_topic_states (
                student_id, topic_id, mastery_score, mastery_state, accuracy_score,
                transfer_score, consistency_score, gap_score, priority_score,
                fragility_score, pressure_collapse_index, speed_score,
                total_attempts, correct_attempts, recent_attempts_window,
                recent_correct_window, evidence_count, decay_risk,
                memory_strength, repair_priority, recognition_strength,
                explicit_form_accuracy_bp, disguised_form_accuracy_bp, recognition_gap_bp
             ) VALUES (
                1, ?1, 6100, 'fragile', 5400,
                4300, 5100, 4300, 7200,
                5600, 5900, 5000,
                7, 4, 3,
                1, 6, 5100,
                4500, 6900, 6800,
                7000, 4300, 3100
             )",
            [neighbor_topic_id],
        )
        .expect("neighbor topic state should insert");
        conn.execute(
            "INSERT INTO memory_states (
                student_id, topic_id, memory_state, memory_strength, decay_risk, review_due_at
             ) VALUES (1, ?1, 'fragile', 4100, 5500, datetime('now', '-1 day'))",
            [primary_topic_id],
        )
        .expect("memory state should insert");
        conn.execute(
            "INSERT INTO engagement_risk_profiles (
                learner_id, risk_level, risk_score_bp, consecutive_misses,
                recent_partial_sessions, last_session_state, next_recovery_action, updated_at
             ) VALUES (
                1, 'medium', 4300, 1,
                2, 'partially_completed', 'Shrink the next session and rebuild momentum.', datetime('now')
             )",
            [],
        )
        .expect("engagement risk should insert");
        conn.execute(
            "INSERT INTO topic_package_snapshots (
                subject_id, topic_id, package_state, live_health_state,
                resource_readiness_score, completeness_score, quality_score,
                evidence_score, source_support_count, contrast_pair_count,
                publishable_artifact_count, published_artifact_count,
                missing_components_json, recommended_jobs_json, computed_at
             ) VALUES (
                1, ?1, 'quality_mixed', 'quality_mixed',
                7200, 6900, 6800,
                6400, 4, 1,
                3, 2,
                '[\"timed worked example\"]', '[\"refresh quality gate\"]', datetime('now')
             )",
            [primary_topic_id],
        )
        .expect("topic package snapshot should insert");
        conn.execute(
            "INSERT INTO curriculum_source_uploads (
                id, uploader_account_id, source_kind, title, source_status, subject_code, metadata_json
             ) VALUES (
                1, 1, 'syllabus', 'Judgment Seed Upload', 'reviewed', 'MTH', '{}'
             )",
            [],
        )
        .expect("source upload should insert");
        conn.execute(
            "INSERT INTO content_publish_jobs (
                id, source_upload_id, subject_id, topic_id, status, decision_trace_json, artifact_summary_json
             ) VALUES (1, 1, 1, ?1, 'review_required', '{}', '{}')",
            [primary_topic_id],
        )
        .expect("publish job should insert");
        conn.execute(
            "INSERT INTO content_quality_reports (
                publish_job_id, report_type, status, confidence_score, metrics_json
             ) VALUES (1, 'quality_gate', 'needs_review', 6200, '{}')",
            [],
        )
        .expect("quality report should insert");
        conn.execute(
            "INSERT INTO resource_metadata_index (
                resource_id, resource_type, subject_id, topic_id,
                difficulty_bp, exam_relevance_bp, teach_suitability_bp,
                test_suitability_bp, pressure_suitability_bp, source,
                confidence_tier, teacher_verified
             ) VALUES (
                2001, 'question', 1, ?1,
                5600, 7800, 6500,
                7900, 7200, 'sample-pack',
                'teacher_authored', 1
             )",
            [primary_topic_id],
        )
        .expect("resource meta should insert");

        let questions = {
            let mut stmt = conn
                .prepare(
                    "SELECT q.id,
                            (SELECT qo.id FROM question_options qo
                             WHERE qo.question_id = q.id
                             ORDER BY qo.is_correct ASC, qo.position ASC
                             LIMIT 1)
                     FROM questions q
                     WHERE q.topic_id IN (?1, ?2)
                     ORDER BY q.id ASC
                     LIMIT 2",
                )
                .expect("question query should prepare");
            let rows = stmt
                .query_map(params![primary_topic_id, neighbor_topic_id], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?))
                })
                .expect("question rows should load");
            rows.collect::<Result<Vec<_>, _>>()
                .expect("questions should collect")
        };

        for (index, (question_id, wrong_option_id)) in questions.iter().enumerate() {
            conn.execute(
                "INSERT INTO student_question_attempts (
                    student_id, question_id, attempt_number, started_at, submitted_at,
                    response_time_ms, selected_option_id, is_correct, confidence_level,
                    changed_answer_count, skipped, timed_out, error_type, was_timed,
                    was_transfer_variant, was_retention_check, was_mixed_context, evidence_weight
                 ) VALUES (
                    1, ?1, 1, datetime('now', '-30 minutes'), datetime('now', '-29 minutes'),
                    ?2, ?3, 0, 'sure', 1, 0, 0, 'pressure_breakdown', 1, 1, 0, 1, 7000
                 )",
                params![question_id, 31_000 + index as i64 * 3_000, wrong_option_id],
            )
            .expect("attempt should insert");
            let attempt_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO evidence_events (
                    attempt_id, student_id, subject_id, topic_id, testing_reason, evidence_weight,
                    mastery_delta, stability_delta, retention_delta, transfer_delta, timed_delta,
                    hypothesis_result, created_at
                 ) VALUES (
                    ?1, 1, 1, ?2, 'transfer', 7000,
                    -120, -80, -60, -180, -220, 'challenged', datetime('now')
                 )",
                params![attempt_id, primary_topic_id],
            )
            .expect("evidence event should insert");
            conn.execute(
                "INSERT INTO wrong_answer_diagnoses (
                    student_id, question_id, topic_id, misconception_id, error_type,
                    primary_diagnosis, secondary_diagnosis, severity, diagnosis_summary,
                    recommended_action, confidence_score, diagnostic_confidence_score
                 ) VALUES (
                    1, ?1, ?2, NULL, 'pressure_breakdown',
                    'boundary_confusion', 'timed_transfer_collapse', 'high',
                    'Learner is confusing nearby forms and gets worse under time.',
                    'run a contrast and pressure repair loop', 7200, 7200
                 )",
                params![question_id, primary_topic_id],
            )
            .expect("diagnosis should insert");
            let diagnosis_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO wrong_answer_interventions (
                    diagnosis_id, student_id, intervention_type, status,
                    outcome_mastery_delta, outcome_notes, assigned_at, completed_at
                 ) VALUES (
                    ?1, 1, 'contrast_repair', 'started',
                    80, 'backend test seed', datetime('now', '-1 day'), datetime('now')
                 )",
                [diagnosis_id],
            )
            .expect("intervention should insert");
        }
        primary_topic_id
    }
}
