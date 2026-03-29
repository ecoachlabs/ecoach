# eCoach Backend Implementation Plan — Part 4 & Part 5
## Scoring Formulas as Rust Code + Business Rules & Constants
## Generated: 2026-03-29

---

# PART 4: ALL SCORING FORMULAS AS RUST CODE

All scores use the `BasisPoints` type alias (`type BasisPoints = i32`, range 0–10000 representing 0.0–1.0). Helper functions `to_bp(f: f64) -> BasisPoints` and `from_bp(bp: BasisPoints) -> f64` convert between the two representations.

```rust
/// Shared type definitions used across all scoring functions
pub type BasisPoints = i32;

/// Convert a float in [0.0, 1.0] to basis points [0, 10000]
#[inline]
pub fn to_bp(v: f64) -> BasisPoints {
    (v.clamp(0.0, 1.0) * 10_000.0).round() as BasisPoints
}

/// Convert basis points [0, 10000] to float [0.0, 1.0]
#[inline]
pub fn from_bp(bp: BasisPoints) -> f64 {
    bp as f64 / 10_000.0
}
```

---

## 4.1 ForecastScore

Estimates the probability that a given topic/question unit `u` will appear on the next BECE paper. Used by the Mock Centre to build Forecast mock sets and produce the Blueprint.

**Formula:**
```
ForecastScore(u) =
    0.25 × Frequency(u)
  + 0.20 × Recency(u)
  + 0.15 × Trend(u)
  + 0.15 × BundleStrength(u)
  + 0.10 × SyllabusPriority(u)
  + 0.10 × StyleRegimeFit(u)
  + 0.05 × ExaminerGoalFit(u)
```

**Output Bands:**
- High: score ≥ 0.70 (≥ 7000 bp)
- Medium: 0.45–0.69 (4500–6999 bp)
- Surprise Risk: 0.30–0.44 (3000–4499 bp)
- Uncertain: < 0.30 (< 3000 bp)

```rust
/// Probability band for a forecast score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForecastBand {
    /// Highly likely to appear on next exam (≥ 7000 bp)
    High,
    /// Moderately likely (4500–6999 bp)
    Medium,
    /// Possible but unlikely; worth including for resilience (3000–4499 bp)
    SurpriseRisk,
    /// Insufficient signal; do not rely on this appearing (< 3000 bp)
    Uncertain,
}

impl ForecastBand {
    pub fn from_score(score: BasisPoints) -> Self {
        match score {
            s if s >= 7_000 => Self::High,
            s if s >= 4_500 => Self::Medium,
            s if s >= 3_000 => Self::SurpriseRisk,
            _               => Self::Uncertain,
        }
    }
}

/// Inputs for the ForecastScore computation.
/// All component values are normalised to [0.0, 1.0].
pub struct ForecastInputs {
    /// How often this unit has appeared across past papers (normalised frequency).
    pub frequency: f64,
    /// How recently it appeared — more recent = higher value.
    pub recency: f64,
    /// Direction of appearance trend over the last N papers (rising trend = 1.0).
    pub trend: f64,
    /// Strength of the question bundle / topic cluster it belongs to.
    pub bundle_strength: f64,
    /// Weight this topic carries in the official syllabus.
    pub syllabus_priority: f64,
    /// Fit with the identified examiner style regime for this paper cycle.
    pub style_regime_fit: f64,
    /// Fit with inferred examiner goal for this sitting.
    pub examiner_goal_fit: f64,
}

/// Compute the ForecastScore for a topic/question unit.
///
/// # Source
/// idea1.txt §1.1; backend_supplement §1.1
///
/// # Returns
/// `(score_bp, band)` — raw score in basis points and its categorical band.
pub fn forecast_score(inputs: &ForecastInputs) -> (BasisPoints, ForecastBand) {
    let score = 0.25 * inputs.frequency
        + 0.20 * inputs.recency
        + 0.15 * inputs.trend
        + 0.15 * inputs.bundle_strength
        + 0.10 * inputs.syllabus_priority
        + 0.10 * inputs.style_regime_fit
        + 0.05 * inputs.examiner_goal_fit;

    let bp = to_bp(score);
    let band = ForecastBand::from_score(bp);
    (bp, band)
}
```

---

## 4.2 MockOrchestrationScore

Scores every candidate topic/question for inclusion in a mock session. A high score means the item should be prioritised in the next mock. The anti-repeat penalty subtracts weight when an item was recently seen.

**Formula:**
```
score =
    0.25 × weakness
  + 0.20 × coverage_gap
  + 0.20 × misconception_pressure
  + 0.15 × spaced_due
  + 0.10 × exam_weight
  + 0.10 × info_value
  + 0.05 × variety_bonus
  − 0.25 × anti_repeat_penalty
```

```rust
/// Inputs for MockOrchestrationScore.
/// All values normalised to [0.0, 1.0].
pub struct MockOrchestrationInputs {
    /// Student weakness score for this topic (see §4.4).
    pub weakness: f64,
    /// How uncovered this topic is in recent mocks.
    pub coverage_gap: f64,
    /// Active misconception pressure on this topic.
    pub misconception_pressure: f64,
    /// Spaced-repetition urgency: is a recheck overdue?
    pub spaced_due: f64,
    /// Exam blueprint weight for this topic.
    pub exam_weight: f64,
    /// Diagnostic information value: how much would a result tell us?
    pub info_value: f64,
    /// Variety bonus: reward for adding representation/topic diversity.
    pub variety_bonus: f64,
    /// Penalty for repeating the same topic/question family too soon.
    pub anti_repeat_penalty: f64,
}

/// Compute the mock orchestration priority score for a candidate topic/item.
///
/// # Source
/// idea1.txt §1.2; backend_supplement §1.2
///
/// # Returns
/// Score in basis points. Negative values are clamped to 0.
pub fn mock_orchestration_score(inputs: &MockOrchestrationInputs) -> BasisPoints {
    let raw = 0.25 * inputs.weakness
        + 0.20 * inputs.coverage_gap
        + 0.20 * inputs.misconception_pressure
        + 0.15 * inputs.spaced_due
        + 0.10 * inputs.exam_weight
        + 0.10 * inputs.info_value
        + 0.05 * inputs.variety_bonus
        - 0.25 * inputs.anti_repeat_penalty;

    to_bp(raw.max(0.0))
}
```

---

## 4.3 MockSelectionScore

Scores individual questions (not whole topics) for selection into a compiled mock paper. Complements MockOrchestrationScore by operating at question granularity.

**Formula:**
```
MockSelect(f) =
    0.30 × BlueprintFit(f)
  + 0.20 × DiagnosticNeed(f)
  + 0.15 × CoverageNeed(f)
  + 0.10 × InfoValue(f)
  + 0.10 × RepresentationNeed(f)
  + 0.10 × Variety(f)
  + 0.05 × SurpriseRisk(f)
  − 0.25 × AntiRepeat(f)
```

```rust
/// Inputs for MockSelectionScore, evaluated per question candidate `f`.
/// All values normalised to [0.0, 1.0].
pub struct MockSelectionInputs {
    /// How well this question fits the exam blueprint target distribution.
    pub blueprint_fit: f64,
    /// How much diagnostic signal this question would produce for this student.
    pub diagnostic_need: f64,
    /// How much this question covers a currently under-represented area.
    pub coverage_need: f64,
    /// Raw informational value (uncertainty reduction about student state).
    pub info_value: f64,
    /// Need for this representation type (text/diagram/graph/table) in the mock.
    pub representation_need: f64,
    /// Question variety contribution relative to already-selected questions.
    pub variety: f64,
    /// Surprise risk: this is a low-frequency but plausible exam question.
    pub surprise_risk: f64,
    /// Penalty for selecting a question too similar to a recent attempt.
    pub anti_repeat: f64,
}

/// Compute the selection score for a question candidate in mock compilation.
///
/// # Source
/// idea1.txt §1.3; backend_supplement §1.3
pub fn mock_selection_score(inputs: &MockSelectionInputs) -> BasisPoints {
    let raw = 0.30 * inputs.blueprint_fit
        + 0.20 * inputs.diagnostic_need
        + 0.15 * inputs.coverage_need
        + 0.10 * inputs.info_value
        + 0.10 * inputs.representation_need
        + 0.10 * inputs.variety
        + 0.05 * inputs.surprise_risk
        - 0.25 * inputs.anti_repeat;

    to_bp(raw.max(0.0))
}
```

---

## 4.4 WeaknessScore

Quantifies how weak a student is on a specific topic `t`. Higher score = higher urgency to address the topic.

**Formula:**
```
Weakness(t) =
    0.35 × (1 − Mastery_t)
  + 0.20 × LinkBreakage_t
  + 0.15 × MisconceptionPressure_t
  + 0.10 × RepresentationGap_t
  + 0.10 × TimedGap_t
  + 0.05 × GuessPenalty_t
  + 0.05 × RecencyDecay_t
```

```rust
/// Inputs for WeaknessScore on topic `t`.
/// All values normalised to [0.0, 1.0].
pub struct WeaknessInputs {
    /// Current mastery score for this topic (used as `1 - mastery` internally).
    pub mastery: f64,
    /// Degree to which prerequisite concept links are broken for this topic.
    pub link_breakage: f64,
    /// Intensity of active misconceptions on this topic.
    pub misconception_pressure: f64,
    /// Gap in the student's ability across different question representations.
    pub representation_gap: f64,
    /// Performance gap when the question is timed vs untimed.
    pub timed_gap: f64,
    /// Penalty for detected guessing patterns on this topic.
    pub guess_penalty: f64,
    /// Decay signal: how long since meaningful correct engagement.
    pub recency_decay: f64,
}

/// Compute the weakness score for topic `t`.
///
/// # Source
/// idea1.txt §1.4; backend_supplement §1.4
///
/// # Returns
/// Score in basis points: 10000 = maximally weak, 0 = no weakness detected.
pub fn weakness_score(inputs: &WeaknessInputs) -> BasisPoints {
    let raw = 0.35 * (1.0 - inputs.mastery)
        + 0.20 * inputs.link_breakage
        + 0.15 * inputs.misconception_pressure
        + 0.10 * inputs.representation_gap
        + 0.10 * inputs.timed_gap
        + 0.05 * inputs.guess_penalty
        + 0.05 * inputs.recency_decay;

    to_bp(raw)
}
```

---

## 4.5 ReadinessScore

Estimates overall exam readiness. Penalties are subtracted for known high-risk conditions.

**Formula:**
```
Readiness =
    0.45 × Mastery
  + 0.20 × TimedPerformance
  + 0.15 × Coverage
  + 0.10 × Consistency
  + 0.10 × Trend

Penalties applied for:
  - Weak critical topics (exam-critical topic with mastery < 4000 bp)
  - Recurring mistakes (same error class seen ≥ 3 times)
  - Missed sessions (> 3 consecutive days inactive)
  - Exam anxiety signals (pressure collapse index elevated)
  - Low recent trend (last 3-session accuracy declining)
```

```rust
/// Raw component inputs for ReadinessScore.
/// All values normalised to [0.0, 1.0].
pub struct ReadinessInputs {
    /// Weighted average mastery across all exam-relevant topics.
    pub mastery: f64,
    /// Performance accuracy and speed under timed conditions.
    pub timed_performance: f64,
    /// Percentage of syllabus topics with at least functional mastery (≥ 5500 bp).
    pub coverage: f64,
    /// Consistency of performance across sessions and question types.
    pub consistency: f64,
    /// Directional improvement trend over the last N sessions.
    pub trend: f64,
    /// Number of exam-critical topics below mastery gate (4000 bp).
    pub critical_topic_failures: u32,
    /// Whether recurring mistakes are detected (same class ≥ 3 times).
    pub has_recurring_mistakes: bool,
    /// Number of consecutive inactive days.
    pub consecutive_inactive_days: u32,
    /// Whether exam anxiety / pressure collapse pattern is elevated.
    pub has_exam_anxiety: bool,
    /// Whether the 3-session recent accuracy trend is declining.
    pub recent_trend_declining: bool,
}

/// Penalty magnitudes (in raw float units, not basis points).
const PENALTY_CRITICAL_TOPIC: f64 = 0.05;   // per failing critical topic, capped at 0.20
const PENALTY_RECURRING_MISTAKES: f64 = 0.04;
const PENALTY_INACTIVITY: f64 = 0.03;        // applied when inactive_days >= 3
const PENALTY_EXAM_ANXIETY: f64 = 0.04;
const PENALTY_RECENT_DECLINE: f64 = 0.03;

/// Compute the ReadinessScore.
///
/// # Source
/// idea1.txt §1.5; idea2.txt §2.6; backend_supplement §1.5, §2.6
///
/// # Returns
/// Score in basis points. Represents overall exam readiness.
pub fn readiness_score(inputs: &ReadinessInputs) -> BasisPoints {
    let base = 0.45 * inputs.mastery
        + 0.20 * inputs.timed_performance
        + 0.15 * inputs.coverage
        + 0.10 * inputs.consistency
        + 0.10 * inputs.trend;

    let critical_penalty = (inputs.critical_topic_failures as f64 * PENALTY_CRITICAL_TOPIC)
        .min(0.20);
    let recurring_penalty = if inputs.has_recurring_mistakes {
        PENALTY_RECURRING_MISTAKES
    } else {
        0.0
    };
    let inactivity_penalty = if inputs.consecutive_inactive_days >= 3 {
        PENALTY_INACTIVITY
    } else {
        0.0
    };
    let anxiety_penalty = if inputs.has_exam_anxiety {
        PENALTY_EXAM_ANXIETY
    } else {
        0.0
    };
    let trend_penalty = if inputs.recent_trend_declining {
        PENALTY_RECENT_DECLINE
    } else {
        0.0
    };

    let total_penalty = critical_penalty
        + recurring_penalty
        + inactivity_penalty
        + anxiety_penalty
        + trend_penalty;

    to_bp((base - total_penalty).max(0.0))
}

/// Journey-mode variant of ReadinessScore (from idea2.txt §2.6).
/// Weights differ from the Mock Centre version.
///
/// Readiness(Journey) =
///     0.25 × topic_mastery
///   + 0.20 × retention
///   + 0.20 × mock_performance
///   + 0.15 × speed
///   + 0.10 × syllabus_coverage
///   + 0.10 × consistency
///   − penalties (same penalty set as ReadinessScore)
pub struct JourneyReadinessInputs {
    pub topic_mastery: f64,
    pub retention: f64,
    pub mock_performance: f64,
    pub speed: f64,
    pub syllabus_coverage: f64,
    pub consistency: f64,
    pub critical_topic_failures: u32,
    pub has_recurring_mistakes: bool,
    pub consecutive_inactive_days: u32,
    pub has_exam_anxiety: bool,
    pub recent_trend_declining: bool,
}

/// Compute the Journey-mode readiness score (§4.14 alias).
pub fn journey_readiness_score(inputs: &JourneyReadinessInputs) -> BasisPoints {
    let base = 0.25 * inputs.topic_mastery
        + 0.20 * inputs.retention
        + 0.20 * inputs.mock_performance
        + 0.15 * inputs.speed
        + 0.10 * inputs.syllabus_coverage
        + 0.10 * inputs.consistency;

    let critical_penalty = (inputs.critical_topic_failures as f64 * PENALTY_CRITICAL_TOPIC)
        .min(0.20);
    let recurring_penalty = if inputs.has_recurring_mistakes { PENALTY_RECURRING_MISTAKES } else { 0.0 };
    let inactivity_penalty = if inputs.consecutive_inactive_days >= 3 { PENALTY_INACTIVITY } else { 0.0 };
    let anxiety_penalty = if inputs.has_exam_anxiety { PENALTY_EXAM_ANXIETY } else { 0.0 };
    let trend_penalty = if inputs.recent_trend_declining { PENALTY_RECENT_DECLINE } else { 0.0 };

    let total_penalty = critical_penalty + recurring_penalty + inactivity_penalty
        + anxiety_penalty + trend_penalty;

    to_bp((base - total_penalty).max(0.0))
}
```

---

## 4.6 PredictedExamScore

Projects the student's expected BECE score as a weighted product across all blueprint targets. Each topic `k` contributes its blueprint weight multiplied by the student's mastery on that topic, adjusted for timing readiness, memory retention, and misconception immunity.

**Formula:**
```
PredictedExamScore =
    ∑(k ∈ BlueprintTargets)
        BlueprintWeight_k × Mastery_k × TimingFactor_k
        × RetentionFactor_k × MisconceptionImmunity_k
```

```rust
/// A single topic's contribution data for PredictedExamScore.
pub struct BlueprintTopicFactors {
    /// Fraction of the exam blueprint this topic represents (sums to 1.0 across all k).
    pub blueprint_weight: f64,
    /// Student's mastery on this topic, normalised to [0.0, 1.0].
    pub mastery: f64,
    /// Readiness to perform under timed exam conditions (0.0 = not ready, 1.0 = fully ready).
    pub timing_factor: f64,
    /// Retention stability — how well knowledge will hold at exam time (0.0–1.0).
    pub retention_factor: f64,
    /// Immunity to known misconceptions on this topic (0.0 = high risk, 1.0 = immune).
    pub misconception_immunity: f64,
}

/// Compute the predicted exam score from blueprint topic factors.
///
/// # Source
/// idea1.txt §1.6; backend_supplement §1.6, §25
///
/// # Returns
/// Predicted score in basis points (0 = 0% predicted, 10000 = 100% predicted).
///
/// # Panics
/// Does not panic; returns 0 if `topics` is empty.
pub fn predicted_exam_score(topics: &[BlueprintTopicFactors]) -> BasisPoints {
    let sum: f64 = topics.iter().map(|t| {
        t.blueprint_weight
            * t.mastery
            * t.timing_factor
            * t.retention_factor
            * t.misconception_immunity
    }).sum();

    to_bp(sum)
}
```

---

## 4.7 MomentumScore

Measures the student's learning momentum across a recent session window. Used by Rise Mode to detect whether the student is rising, plateauing, or declining.

**Formula:**
```
MomentumScore = 0.35 × volume + 0.40 × accuracy + 0.25 × pace
```

```rust
/// Inputs for MomentumScore over a recent session window.
/// All values normalised to [0.0, 1.0].
pub struct MomentumInputs {
    /// Volume of meaningful attempts completed (normalised against target volume).
    pub volume: f64,
    /// Accuracy rate across the window (correct / total attempts).
    pub accuracy: f64,
    /// Pace: average speed relative to expected time-per-question (faster = higher).
    pub pace: f64,
}

/// Compute the MomentumScore.
///
/// # Source
/// idea4.txt §4.6; backend_supplement §4.6, §25
pub fn momentum_score(inputs: &MomentumInputs) -> BasisPoints {
    let raw = 0.35 * inputs.volume
        + 0.40 * inputs.accuracy
        + 0.25 * inputs.pace;

    to_bp(raw)
}
```

---

## 4.8 StrainScore

Detects cognitive overload or burnout signals within a session. A high strain score means the student is struggling beyond their productive zone and may need a break or de-escalation.

**Formula:**
```
StrainScore =
    0.30 × accuracy_drop
  + 0.20 × completion_drop
  + 0.20 × hint_spike
  + 0.15 × skip
  + 0.15 × pace_instability
```

```rust
/// Inputs for StrainScore within a session window.
/// All values normalised to [0.0, 1.0].
pub struct StrainInputs {
    /// Drop in accuracy compared to the student's recent baseline.
    pub accuracy_drop: f64,
    /// Drop in question completion rate (more questions abandoned mid-session).
    pub completion_drop: f64,
    /// Spike in hint usage relative to the student's usual rate.
    pub hint_spike: f64,
    /// Rate of skipped questions this session.
    pub skip: f64,
    /// Instability in answer pace (high variance in response times).
    pub pace_instability: f64,
}

/// Compute the StrainScore.
///
/// # Source
/// idea4.txt §4.7; backend_supplement §4.7, §25
pub fn strain_score(inputs: &StrainInputs) -> BasisPoints {
    let raw = 0.30 * inputs.accuracy_drop
        + 0.20 * inputs.completion_drop
        + 0.20 * inputs.hint_spike
        + 0.15 * inputs.skip
        + 0.15 * inputs.pace_instability;

    to_bp(raw)
}
```

---

## 4.9 MSI — Multi-Source Intelligence Score

Aggregates six independent evidence signals into a single memory-strength index for a knowledge unit. Used by Memory Mode to decide scheduling urgency.

**Formula:**
```
MSI =
    0.30 × Accuracy
  + 0.15 × Speed
  + 0.20 × Retention
  + 0.15 × Variability
  + 0.10 × Interference
  + 0.10 × Consistency
```

Where:
- `Accuracy` = recent accuracy rate (last N attempts)
- `Speed` = normalised recall latency (lower latency = higher score)
- `Retention` = decay resistance over time
- `Variability` = variant/transfer performance
- `Interference` = independence from hints and cues
- `Consistency` = stable performance across contexts

```rust
/// Inputs for the MSI (Multi-Source Intelligence) score.
/// All values normalised to [0.0, 1.0]; higher = better memory on that dimension.
pub struct MsiInputs {
    /// Recent accuracy across last N attempts on this knowledge unit.
    pub accuracy: f64,
    /// Recall speed — inverse of normalised latency (fast = high).
    pub speed: f64,
    /// Retention over time — how well the concept resists decay.
    pub retention: f64,
    /// Transfer/variant performance — performance across changed question forms.
    pub variability: f64,
    /// Independence — performance without hints or prompts.
    pub interference: f64,
    /// Consistency — performance stability across sessions and contexts.
    pub consistency: f64,
}

/// Compute the MSI score for a knowledge unit.
///
/// # Source
/// idea7.txt §7.1; backend_supplement §7.1, §25
pub fn msi_score(inputs: &MsiInputs) -> BasisPoints {
    let raw = 0.30 * inputs.accuracy
        + 0.15 * inputs.speed
        + 0.20 * inputs.retention
        + 0.15 * inputs.variability
        + 0.10 * inputs.interference
        + 0.10 * inputs.consistency;

    to_bp(raw)
}
```

---

## 4.10 EMA — Exponentially Weighted Moving Average Mastery Update

All mastery updates use an EMA so that new evidence has high impact but the total score is smoothed against noise. Evidence weight is reduced by hints, boosted by transfer contexts and delayed recall.

**Formulas:**
```
// EMA update
new_score = alpha × new_evidence + (1 - alpha) × old_score   [alpha = 0.3]

// Evidence weight
effective_weight = base_weight
    × (0.5 ^ hint_count)
    × (1.30 if transfer_context else 1.0)
    × (1.50 if delayed_recall else 1.0)
    × (0.60 if repeat_same_day else 1.0)

// Weighted evidence input
weighted_evidence = raw_score × effective_weight
```

```rust
/// Alpha for the EMA update (smoothing factor).
pub const EMA_ALPHA: f64 = 0.3;

/// Compute the effective evidence weight for a single attempt.
///
/// # Source
/// idea7.txt; idea28.txt; detailed_backend_plan §evidence weight computation
///
/// # Parameters
/// - `hint_count`: number of hints used (each halves the weight)
/// - `is_transfer_context`: question tested transfer to a new context
/// - `is_delayed_recall`: attempted ≥ 24 hours after last exposure
/// - `is_repeat_same_day`: same question attempted again within the same day
pub fn compute_evidence_weight(
    hint_count: u32,
    is_transfer_context: bool,
    is_delayed_recall: bool,
    is_repeat_same_day: bool,
) -> f64 {
    let mut weight: f64 = 1.0;

    // Each hint halves the evidence weight (cap at 3 halvings = 0.125 minimum from hints)
    if hint_count > 0 {
        weight *= 0.5_f64.powi(hint_count.min(3) as i32);
    }

    // Transfer context: knowing it in a new form is stronger evidence
    if is_transfer_context {
        weight *= 1.30;
    }

    // Delayed recall: recalling after ≥ 24h gap signals durable memory
    if is_delayed_recall {
        weight *= 1.50;
    }

    // Same-day repeat: diminishing returns on repeated attempts the same day
    if is_repeat_same_day {
        weight *= 0.60;
    }

    weight.clamp(0.05, 2.0)
}

/// Apply an EMA update to a mastery score using new evidence.
///
/// # Source
/// idea28.txt; detailed_backend_plan §1.3 (evidence weight computation)
///
/// # Parameters
/// - `old_bp`: current mastery score in basis points
/// - `new_score`: raw score from the new attempt, normalised to [0.0, 1.0]
/// - `weight`: effective evidence weight from `compute_evidence_weight`
///
/// # Returns
/// Updated mastery score in basis points.
pub fn update_mastery_ema(old_bp: BasisPoints, new_score: f64, weight: f64) -> BasisPoints {
    let old_score = from_bp(old_bp);
    // Weight the incoming evidence
    let weighted_evidence = new_score * weight;
    // Normalise weighted evidence back to [0,1] (weight may exceed 1.0 for boosted evidence)
    let effective_input = weighted_evidence.clamp(0.0, 1.0);
    // EMA blend
    let new_mastery = EMA_ALPHA * effective_input + (1.0 - EMA_ALPHA) * old_score;
    to_bp(new_mastery)
}
```

---

## 4.11 GapPriorityScore

Ranks knowledge gaps by how urgently they need to be repaired. A higher score means the gap should be addressed before others.

**Formula:**
```
GapPriority =
    0.30 × dependency_block
  + 0.20 × recency_decay
  + 0.15 × exam_weight
  + 0.15 × misconception_density
  + 0.10 × repair_effort
  + 0.10 × confidence_gap
```

```rust
/// Inputs for GapPriorityScore.
/// All values normalised to [0.0, 1.0].
pub struct GapPriorityInputs {
    /// Degree to which this gap blocks downstream prerequisite topics.
    pub dependency_block: f64,
    /// How rapidly mastery on this topic is decaying (recency decay pressure).
    pub recency_decay: f64,
    /// Weight of this topic in the exam blueprint.
    pub exam_weight: f64,
    /// Density of active misconceptions on this topic.
    pub misconception_density: f64,
    /// Estimated repair effort (higher = more effort needed; inverted for priority).
    /// Note: this term adds priority because harder repairs need earlier scheduling.
    pub repair_effort: f64,
    /// Gap between student's confidence and actual performance on this topic.
    pub confidence_gap: f64,
}

/// Compute the gap priority score.
///
/// # Source
/// idea6.txt; backend_supplement §5.1
pub fn gap_priority_score(inputs: &GapPriorityInputs) -> BasisPoints {
    let raw = 0.30 * inputs.dependency_block
        + 0.20 * inputs.recency_decay
        + 0.15 * inputs.exam_weight
        + 0.15 * inputs.misconception_density
        + 0.10 * inputs.repair_effort
        + 0.10 * inputs.confidence_gap;

    to_bp(raw)
}
```

---

## 4.12 Memory Decay Function

Models the exponential decay of memory strength over time. Decay rate is governed by the current `MemoryState` of the knowledge unit.

**Formula:**
```
MemoryStrength(t) = initial_strength × e^(−decay_rate × days_since_last_review)

Decay rates by memory state:
  Fresh          → 0.05
  Consolidating  → 0.08
  Stable         → 0.03
  AtRisk         → 0.15
```

```rust
/// Memory states that govern the decay rate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryState {
    /// Recently learned; not yet consolidated. Decay rate: 0.05 / day.
    Fresh,
    /// In the consolidation window. Decay rate: 0.08 / day.
    Consolidating,
    /// Durable, well-rehearsed memory. Decay rate: 0.03 / day.
    Stable,
    /// Showing signs of degradation. Decay rate: 0.15 / day.
    AtRisk,
    /// Actively collapsing — requires immediate rescue.
    Decaying,
    /// Fully collapsed — no recall without full reteach.
    Collapsed,
}

impl MemoryState {
    /// Return the daily exponential decay rate for this memory state.
    pub fn decay_rate(&self) -> f64 {
        match self {
            Self::Fresh         => 0.05,
            Self::Consolidating => 0.08,
            Self::Stable        => 0.03,
            Self::AtRisk        => 0.15,
            Self::Decaying      => 0.25,
            Self::Collapsed     => 0.50,
        }
    }
}

/// Compute the decayed memory strength after `days_since_review` days.
///
/// # Source
/// idea32.txt; backend_supplement §16; idea7.txt §7.5
///
/// # Parameters
/// - `initial_strength_bp`: memory strength at the time of last successful review (basis points)
/// - `memory_state`: current classification of this memory unit
/// - `days_since_last_review`: elapsed days since last confirmed successful recall
///
/// # Returns
/// Decayed memory strength in basis points.
pub fn memory_decay(
    initial_strength_bp: BasisPoints,
    memory_state: &MemoryState,
    days_since_last_review: f64,
) -> BasisPoints {
    let initial = from_bp(initial_strength_bp);
    let decay_rate = memory_state.decay_rate();
    let decayed = initial * (-decay_rate * days_since_last_review).exp();
    to_bp(decayed)
}
```

---

## 4.13 EvidenceWeight Computation

Full evidence weight formula incorporating all modifiers. Returns a normalised weight suitable for use in EMA mastery updates.

**Formula:**
```
effective_weight =
    base_weight
    × (0.5 ^ hint_count)
    × (1.30 if transfer_context)
    × (1.50 if delayed_recall ≥ 24h)
    × (0.60 if repeat_same_day)

For support level:
    "guided"        → × 0.70
    "heavily_guided" → × 0.40

For confidence-correct guesses:
    correct + confidence == "guessed" → × 0.50
```

```rust
/// Support level during an attempt, affecting evidence weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportLevel {
    Independent,
    Guided,
    HeavilyGuided,
}

/// Full evidence weight inputs for a single attempt.
pub struct EvidenceWeightInputs {
    pub hint_count: u32,
    pub support_level: SupportLevel,
    pub is_transfer_context: bool,
    /// True if ≥ 24 hours elapsed since last exposure to this topic.
    pub is_delayed_recall: bool,
    pub is_repeat_same_day: bool,
    /// True if the student answered correctly but self-reported as "guessed".
    pub is_correct_but_guessed: bool,
}

/// Compute the full effective evidence weight for an attempt.
///
/// # Source
/// idea7.txt; idea28.txt; detailed_backend_plan §1.3
///
/// # Returns
/// Weight as a plain float; caller divides by this or uses directly in EMA.
/// Clamped to [0.05, 2.0].
pub fn compute_full_evidence_weight(inputs: &EvidenceWeightInputs) -> f64 {
    let mut weight: f64 = 1.0;

    // Hint penalty: each hint halves weight, maximum 3 halvings
    if inputs.hint_count > 0 {
        weight *= 0.5_f64.powi(inputs.hint_count.min(3) as i32);
    }

    // Support level penalty
    match inputs.support_level {
        SupportLevel::Guided        => weight *= 0.70,
        SupportLevel::HeavilyGuided => weight *= 0.40,
        SupportLevel::Independent   => {}
    }

    // Transfer context boost
    if inputs.is_transfer_context {
        weight *= 1.30;
    }

    // Delayed recall boost
    if inputs.is_delayed_recall {
        weight *= 1.50;
    }

    // Same-day repeat diminishment
    if inputs.is_repeat_same_day {
        weight *= 0.60;
    }

    // Guessed-but-correct penalty
    if inputs.is_correct_but_guessed {
        weight *= 0.50;
    }

    weight.clamp(0.05, 2.0)
}
```

---

## 4.14 JourneyReadinessScore

See §4.5 — the `journey_readiness_score` function is defined there. It uses different weights from the Mock Centre ReadinessScore:

```
JourneyReadiness =
    0.25 × topic_mastery
  + 0.20 × retention
  + 0.20 × mock_performance
  + 0.15 × speed
  + 0.10 × syllabus_coverage
  + 0.10 × consistency
  − penalties
```

The `journey_readiness_score` function defined in §4.5 is the canonical implementation.

---

## 4.15 ElitePerformanceScore (EPS)

Produces a composite Elite Performance Score from the five core elite dimensions. Used to rank and benchmark elite-tier students.

**Formula:**
```
EPS = weighted_average(
    speed,
    accuracy_under_pressure,
    transfer_ability,
    novelty_handling,
    misconception_immunity
)
```

Default weights are equal (0.20 each) but can be configured per subject or mission type.

```rust
/// Inputs for the Elite Performance Score.
/// All values normalised to [0.0, 1.0].
pub struct EpsInputs {
    /// Speed performance — time-to-correct relative to elite benchmark.
    pub speed: f64,
    /// Accuracy under pressure conditions (timed, high-stakes).
    pub accuracy_under_pressure: f64,
    /// Transfer ability — performance on novel question forms.
    pub transfer_ability: f64,
    /// Novelty handling — performance on new/unseen question types.
    pub novelty_handling: f64,
    /// Immunity to active misconceptions on this topic set.
    pub misconception_immunity: f64,
}

/// Optional custom weights for EPS dimensions (must sum to 1.0).
pub struct EpsWeights {
    pub speed: f64,
    pub accuracy_under_pressure: f64,
    pub transfer_ability: f64,
    pub novelty_handling: f64,
    pub misconception_immunity: f64,
}

impl Default for EpsWeights {
    fn default() -> Self {
        Self {
            speed: 0.20,
            accuracy_under_pressure: 0.20,
            transfer_ability: 0.20,
            novelty_handling: 0.20,
            misconception_immunity: 0.20,
        }
    }
}

/// Compute the Elite Performance Score.
///
/// # Source
/// idea5.txt; backend_supplement §6
///
/// # Parameters
/// - `inputs`: performance dimension values
/// - `weights`: dimension weights (use `EpsWeights::default()` for equal weighting)
pub fn elite_performance_score(inputs: &EpsInputs, weights: &EpsWeights) -> BasisPoints {
    let raw = weights.speed * inputs.speed
        + weights.accuracy_under_pressure * inputs.accuracy_under_pressure
        + weights.transfer_ability * inputs.transfer_ability
        + weights.novelty_handling * inputs.novelty_handling
        + weights.misconception_immunity * inputs.misconception_immunity;

    to_bp(raw)
}
```

---

## 4.16 RiseTransformationReadiness — Stage Gate Checks

Each Rise Mode stage transition requires meeting a pair of score thresholds. The gate function returns `true` only when ALL required conditions are satisfied.

**Thresholds:**
```
Rescue → Stabilize:
    foundation_score ≥ 4500 AND misconception_density < 3500

Stabilize → Accelerate:
    recall_score ≥ 6000 AND accuracy_score ≥ 6500

Accelerate → Dominate:
    speed_score ≥ 7500 AND pressure_stability_score ≥ 7000
```

```rust
/// Rise Mode transformation stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiseStage {
    /// Stop the bleeding; find root gaps; build first wins.
    Rescue,
    /// Make correct thinking repeatable; scaffolded concept clusters.
    Stabilize,
    /// Speed + independence; timed drills; mixed topics; pressure mode.
    Accelerate,
    /// Outperform top students; trap questions; elite variants; speed + accuracy.
    Dominate,
}

/// The student's current Rise Mode score profile (all in basis points).
pub struct RiseScores {
    pub foundation_score: BasisPoints,
    pub recall_score: BasisPoints,
    pub speed_score: BasisPoints,
    pub accuracy_score: BasisPoints,
    pub pressure_stability_score: BasisPoints,
    pub misconception_density: BasisPoints,
    pub momentum_score: BasisPoints,
    pub transformation_readiness: BasisPoints,
}

/// Check whether the student is ready to advance to the next Rise stage.
///
/// # Source
/// idea4.txt; backend_supplement §4.1, §4.5
///
/// # Returns
/// `true` if all threshold conditions for the current → next transition are met.
pub fn rise_stage_gate(current_stage: &RiseStage, scores: &RiseScores) -> bool {
    match current_stage {
        RiseStage::Rescue => {
            // Gate: foundation_score ≥ 4500 AND misconception_density < 3500
            scores.foundation_score >= 4_500
                && scores.misconception_density < 3_500
        }
        RiseStage::Stabilize => {
            // Gate: recall_score ≥ 6000 AND accuracy_score ≥ 6500
            scores.recall_score >= 6_000
                && scores.accuracy_score >= 6_500
        }
        RiseStage::Accelerate => {
            // Gate: speed_score ≥ 7500 AND pressure_stability_score ≥ 7000
            scores.speed_score >= 7_500
                && scores.pressure_stability_score >= 7_000
        }
        RiseStage::Dominate => {
            // Already at the final stage; no further transition.
            false
        }
    }
}

/// Return the next Rise stage given the current one, if a transition exists.
pub fn rise_next_stage(current: &RiseStage) -> Option<RiseStage> {
    match current {
        RiseStage::Rescue     => Some(RiseStage::Stabilize),
        RiseStage::Stabilize  => Some(RiseStage::Accelerate),
        RiseStage::Accelerate => Some(RiseStage::Dominate),
        RiseStage::Dominate   => None,
    }
}
```

---

# PART 5: ALL BUSINESS RULES & CONSTANTS

---

## 5.1 Mastery Gate Rules

Mastery is stored and compared as `BasisPoints` (0–10000). A student **cannot progress** to a dependent topic until the prerequisite topic meets the minimum gate threshold.

```rust
/// Named mastery bands with their basis-point boundaries.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MasteryBand {
    /// 0–3999: Blocked. Must resolve prerequisites before progressing.
    Blocked,
    /// 4000–5499: Emerging. Limited progression allowed; no dependency unlock.
    Emerging,
    /// 5500–6999: Functional. Normal progression; dependency topics may unlock.
    Functional,
    /// 7000–8499: Stable. Confident performance; exam-weight topics can be tackled.
    Stable,
    /// 8500–8999: ExamReady. Suitable for final exam conditioning and mock inclusion.
    ExamReady,
    /// 9000–10000: Mastered. Full elite access permitted.
    Mastered,
}

impl MasteryBand {
    pub fn classify(mastery_bp: BasisPoints) -> Self {
        match mastery_bp {
            m if m >= 9_000 => Self::Mastered,
            m if m >= 8_500 => Self::ExamReady,
            m if m >= 7_000 => Self::Stable,
            m if m >= 5_500 => Self::Functional,
            m if m >= 4_000 => Self::Emerging,
            _               => Self::Blocked,
        }
    }

    /// Returns true if the student may advance to dependent topics at this band.
    pub fn allows_progression(&self) -> bool {
        !matches!(self, Self::Blocked | Self::Emerging)
    }

    /// Returns true if the topic is accessible for standard practice sessions.
    pub fn allows_standard_session(&self) -> bool {
        !matches!(self, Self::Blocked)
    }
}

/// Hard progression gate: mastery below this threshold blocks dependent topics entirely.
pub const MASTERY_GATE_MINIMUM: BasisPoints = 4_000;

/// Gate for dependency unlock (must reach Functional to unlock next topic).
pub const MASTERY_GATE_DEPENDENCY_UNLOCK: BasisPoints = 5_500;

/// Gate for mock inclusion (topic should not appear in mock if below this).
pub const MASTERY_GATE_MOCK_INCLUSION: BasisPoints = 4_000;

/// Gate for exam readiness claim on a topic.
pub const MASTERY_GATE_EXAM_READY: BasisPoints = 8_500;

/// Gate for elite mode access per topic.
pub const MASTERY_GATE_ELITE: BasisPoints = 8_500;
```

---

## 5.2 Evidence Weighting Rules

These rules are applied whenever computing the weight of a new attempt before it enters the EMA mastery update pipeline. All rules compose multiplicatively.

| Rule | Multiplier |
|------|-----------|
| Baseline (first attempt) | 1.00 |
| Each hint used | × 0.50 per hint (max 3 halvings → min 0.125 from hints alone) |
| Guided support level | × 0.70 |
| Heavily guided support level | × 0.40 |
| Transfer context (new form/wording) | × 1.30 |
| Delayed recall (≥ 24h since last exposure) | × 1.50 |
| Same-day repeat attempt on same question | × 0.60 |
| Correct answer + "guessed" confidence | × 0.50 |

**Caps:**
- Final effective weight is clamped to [0.05, 2.0] before use.
- A single attempt cannot produce a weight of 0.0 (the system always registers some evidence).

**Rule: Minimum Evidence Before Mastery Update**
- At least 3 attempts must have been recorded before any mastery band transition is applied.
- A single successful attempt NEVER grants a mastery upgrade by itself.
- See §5.4 for the full session minimum rules.

```rust
pub const EVIDENCE_MIN_ATTEMPTS_FOR_UPDATE: u32 = 3;
pub const EVIDENCE_HINT_FACTOR: f64 = 0.50;
pub const EVIDENCE_GUIDED_FACTOR: f64 = 0.70;
pub const EVIDENCE_HEAVILY_GUIDED_FACTOR: f64 = 0.40;
pub const EVIDENCE_TRANSFER_BOOST: f64 = 1.30;
pub const EVIDENCE_DELAYED_RECALL_BOOST: f64 = 1.50;
pub const EVIDENCE_SAME_DAY_REPEAT_FACTOR: f64 = 0.60;
pub const EVIDENCE_GUESSED_CORRECT_FACTOR: f64 = 0.50;
pub const EVIDENCE_WEIGHT_MIN: f64 = 0.05;
pub const EVIDENCE_WEIGHT_MAX: f64 = 2.00;
```

---

## 5.3 Memory Scheduling Rules

The memory scheduling system assigns every knowledge unit to a review state and computes the `next_review_at` timestamp accordingly.

**Review Intervals by Memory State:**

| Memory State | Review Interval | Action |
|-------------|----------------|--------|
| Fresh | 1 day | Schedule normal review |
| Consolidating | 3 days | Schedule normal review |
| Stable | 7 days | Schedule maintenance review |
| AtRisk | 2 days | Schedule rescue review |
| Decaying | Today (0 days) | Schedule immediate rescue |
| Collapsed | Today (0 days) | Block session; require rescue before other topics |

**Spaced Recheck Ladder (after successful recall at each step):**
```
1 day → 3 days → 7 days → 14 days → 30 days
```

**Promotion and Demotion:**
- After **successful recall**: promote the unit one memory state level up.
- After **failed recall**: demote the unit one memory state level down and schedule rescue.
- After **failed recall at Critical/Collapsed**: block the session until rescue is completed.

```rust
/// Days until next review for a given memory state.
/// Returns `0` for states requiring immediate action.
pub fn next_review_days(state: &MemoryState) -> u32 {
    match state {
        MemoryState::Fresh         => 1,
        MemoryState::Consolidating => 3,
        MemoryState::Stable        => 7,
        MemoryState::AtRisk        => 2,
        MemoryState::Decaying      => 0,
        MemoryState::Collapsed     => 0,
    }
}

/// Promote a memory state after successful recall.
pub fn memory_promote(state: &MemoryState) -> MemoryState {
    match state {
        MemoryState::Collapsed     => MemoryState::AtRisk,
        MemoryState::Decaying      => MemoryState::AtRisk,
        MemoryState::AtRisk        => MemoryState::Fresh,
        MemoryState::Fresh         => MemoryState::Consolidating,
        MemoryState::Consolidating => MemoryState::Stable,
        MemoryState::Stable        => MemoryState::Stable, // Already at top
    }
}

/// Demote a memory state after failed recall.
pub fn memory_demote(state: &MemoryState) -> MemoryState {
    match state {
        MemoryState::Stable        => MemoryState::Consolidating,
        MemoryState::Consolidating => MemoryState::AtRisk,
        MemoryState::Fresh         => MemoryState::AtRisk,
        MemoryState::AtRisk        => MemoryState::Decaying,
        MemoryState::Decaying      => MemoryState::Collapsed,
        MemoryState::Collapsed     => MemoryState::Collapsed, // Cannot go lower
    }
}

/// Returns true if this memory state requires the session to be blocked
/// until a rescue sequence is completed.
pub fn requires_session_block(state: &MemoryState) -> bool {
    matches!(state, MemoryState::Collapsed)
}
```

---

## 5.4 Session Rules

These rules govern every study session across all modes.

| Rule | Value |
|------|-------|
| Minimum attempts before any mastery update | 3 |
| No mastery from single success | Hard rule — one correct answer never triggers upgrade |
| Maximum session duration | 90 minutes |
| Session must be formally closed to count time | If client crashes, time is NOT counted |
| Abandoned session — time | NOT counted toward daily study time |
| Abandoned session — attempts | Still recorded (evidence is preserved) |
| Minimum sessions spread for readiness claim | ≥ 3 separate sessions on the topic |
| Maximum active questions in session queue | 20 |

```rust
pub const SESSION_MIN_ATTEMPTS: u32 = 3;
pub const SESSION_MAX_DURATION_MINUTES: u32 = 90;
pub const SESSION_MIN_SESSIONS_FOR_READINESS: u32 = 3;
pub const SESSION_MAX_QUEUE_SIZE: u32 = 20;

/// A session is only eligible for time accounting if it has been formally closed.
/// `is_formally_closed` must be set to true by the session end handler.
/// Abandoned sessions preserve attempts but contribute 0 minutes to study time.
pub fn session_time_is_countable(is_formally_closed: bool) -> bool {
    is_formally_closed
}
```

---

## 5.5 Coach State Machine — All 14 States and Valid Transitions

The `CoachLifecycleState` drives every screen the learner sees. Transitions are triggered by specific events and can only occur along valid edges.

**States:**

| # | State | Meaning |
|---|-------|---------|
| 1 | `OnboardingRequired` | Account created but profile setup not complete |
| 2 | `SubjectSelectionRequired` | Profile done; no subjects selected yet |
| 3 | `DiagnosticRequired` | Subjects selected; baseline diagnostic not yet run |
| 4 | `ContentReadinessRequired` | Diagnostic done; content packs not installed |
| 5 | `PlanGenerationRequired` | Content ready; study plan not yet generated |
| 6 | `ReadyForTodayMission` | Plan exists; awaiting student to start today's mission |
| 7 | `MissionInProgress` | Student is actively working a session |
| 8 | `MissionReviewRequired` | Session completed; post-session analysis not yet shown |
| 9 | `RepairRequired` | Coach has detected a topic that must be repaired before continuing |
| 10 | `BlockedOnTopic` | Student's mastery < 4000 bp on a prerequisite; cannot advance |
| 11 | `PlanAdjustmentRequired` | Performance has diverged enough to require plan rebalancing |
| 12 | `ReviewDay` | It is a scheduled spaced-repetition review day |
| 13 | `ExamMode` | Exam is ≤ 14 days away; final conditioning active |
| 14 | `StalledNoContent` | No content packs available and no network to fetch them |

**Valid Transitions Table:**

| From State | → To State | Trigger Condition |
|-----------|-----------|------------------|
| `OnboardingRequired` | `SubjectSelectionRequired` | Onboarding profile steps completed |
| `SubjectSelectionRequired` | `DiagnosticRequired` | At least one subject selected |
| `DiagnosticRequired` | `ContentReadinessRequired` | Baseline diagnostic session completed |
| `ContentReadinessRequired` | `PlanGenerationRequired` | Minimum content pack installed (SufficiencyLevel ≥ Amber) |
| `ContentReadinessRequired` | `StalledNoContent` | No content packs available and cannot be fetched |
| `PlanGenerationRequired` | `ReadyForTodayMission` | Study plan generated and approved |
| `ReadyForTodayMission` | `MissionInProgress` | Student starts today's session |
| `ReadyForTodayMission` | `ReviewDay` | Scheduler determines today is a review day |
| `ReadyForTodayMission` | `ExamMode` | Exam date is ≤ 14 days away |
| `MissionInProgress` | `MissionReviewRequired` | Session formally closed |
| `MissionInProgress` | `ReadyForTodayMission` | Session abandoned (attempts saved, time not counted) |
| `MissionReviewRequired` | `ReadyForTodayMission` | Post-session review acknowledged by student |
| `MissionReviewRequired` | `RepairRequired` | Coach detects critical failure requiring immediate repair |
| `MissionReviewRequired` | `BlockedOnTopic` | Topic mastery < 4000 bp on a prerequisite topic |
| `MissionReviewRequired` | `PlanAdjustmentRequired` | Performance drift exceeds replan threshold |
| `RepairRequired` | `ReadyForTodayMission` | Repair sequence completed; mastery ≥ 4000 bp |
| `BlockedOnTopic` | `ReadyForTodayMission` | Prerequisite mastery restored to ≥ 4000 bp |
| `PlanAdjustmentRequired` | `ReadyForTodayMission` | Plan rebalanced and accepted |
| `ReviewDay` | `MissionInProgress` | Student starts review session |
| `ReviewDay` | `ReadyForTodayMission` | Review session completed |
| `ExamMode` | `MissionInProgress` | Student starts final conditioning session |
| `ExamMode` | `ReadyForTodayMission` | Exam conditioning session completed |
| `StalledNoContent` | `ContentReadinessRequired` | Content pack installed |
| Any state | `ExamMode` | Exam date becomes ≤ 14 days away (override transition) |

```rust
impl CoachLifecycleState {
    /// Determine if a transition to `next` is a valid edge from `self`.
    pub fn can_transition_to(&self, next: &CoachLifecycleState) -> bool {
        use CoachLifecycleState::*;
        matches!(
            (self, next),
            (OnboardingRequired,       SubjectSelectionRequired)
            | (SubjectSelectionRequired,  DiagnosticRequired)
            | (DiagnosticRequired,        ContentReadinessRequired)
            | (ContentReadinessRequired,  PlanGenerationRequired)
            | (ContentReadinessRequired,  StalledNoContent)
            | (PlanGenerationRequired,    ReadyForTodayMission)
            | (ReadyForTodayMission,      MissionInProgress)
            | (ReadyForTodayMission,      ReviewDay)
            | (ReadyForTodayMission,      ExamMode)
            | (MissionInProgress,         MissionReviewRequired)
            | (MissionInProgress,         ReadyForTodayMission)
            | (MissionReviewRequired,     ReadyForTodayMission)
            | (MissionReviewRequired,     RepairRequired)
            | (MissionReviewRequired,     BlockedOnTopic)
            | (MissionReviewRequired,     PlanAdjustmentRequired)
            | (RepairRequired,            ReadyForTodayMission)
            | (BlockedOnTopic,            ReadyForTodayMission)
            | (PlanAdjustmentRequired,    ReadyForTodayMission)
            | (ReviewDay,                 MissionInProgress)
            | (ReviewDay,                 ReadyForTodayMission)
            | (ExamMode,                  MissionInProgress)
            | (ExamMode,                  ReadyForTodayMission)
            | (StalledNoContent,          ContentReadinessRequired)
            // Global override: any state can transition to ExamMode when exam ≤ 14 days
            | (_, ExamMode)
        )
    }
}
```

---

## 5.6 Mock Orchestration Rules — 6 Policies

These policies are applied in order during mock compilation. A question/topic must pass all applicable policies to be included in a mock.

### Policy 1: Fix Prerequisites First
- A topic **must not appear in any mock** if its prerequisite topics have mastery < 5000 bp (Emerging band).
- The mock engine checks the full prerequisite chain, not just immediate parents.
- Exception: `MockType::Diagnostic` bypasses this rule to allow gap detection.

```rust
pub const MOCK_PREREQUISITE_GATE: BasisPoints = 5_000;

/// Returns true if a topic is eligible for mock inclusion based on prerequisite mastery.
/// `prerequisite_masteries`: the mastery scores of all prerequisite topics in the chain.
/// `mock_type`: Diagnostic mocks bypass this gate.
pub fn mock_prerequisite_gate_passes(
    prerequisite_masteries: &[BasisPoints],
    mock_type: &MockType,
) -> bool {
    if matches!(mock_type, MockType::Diagnostic) {
        return true;
    }
    prerequisite_masteries.iter().all(|&m| m >= MOCK_PREREQUISITE_GATE)
}
```

### Policy 2: Prioritize Unresolved Misconceptions
- Any topic with active misconception tags has its `MockOrchestrationScore` multiplied by a 1.25× misconception urgency boost.
- "Active" means the misconception has been triggered ≥ 2 times in the last 10 attempts and has not been resolved.

```rust
pub const MISCONCEPTION_URGENCY_MULTIPLIER: f64 = 1.25;
pub const MISCONCEPTION_ACTIVE_TRIGGER_THRESHOLD: u32 = 2;
pub const MISCONCEPTION_ACTIVE_WINDOW: u32 = 10; // last N attempts
```

### Policy 3: Enforce Coverage
- Within any 3-mock window, every syllabus strand must be represented at least once.
- A strand that has been absent from the last 3 mocks receives a `coverage_gap` boost of 0.30 (added to its MockOrchestrationScore input).

```rust
pub const COVERAGE_ENFORCEMENT_WINDOW: u32 = 3; // mocks
pub const COVERAGE_GAP_BOOST: f64 = 0.30;        // added to coverage_gap input when triggered
```

### Policy 4: Rotate Representations
- No more than 3 consecutive questions of the same representation type (text / diagram / graph / table) may appear within a single mock.
- When the limit is reached, the next question must use a different representation type.

```rust
pub const MAX_CONSECUTIVE_SAME_REPRESENTATION: u32 = 3;
```

### Policy 5: Increase Exam Simulation Near Exam
- When ≤ 14 days remain until the exam date, the default mock type switches to `MockType::FinalExam`.
- `FinalExam` mocks use 100% blueprint-matching question selection, full time pressure, and no scaffolding.

```rust
pub const EXAM_MODE_TRIGGER_DAYS: i64 = 14;

/// Returns the recommended mock type given days remaining until exam.
pub fn recommended_mock_type(days_to_exam: i64) -> MockType {
    if days_to_exam <= EXAM_MODE_TRIGGER_DAYS {
        MockType::FinalExam
    } else {
        MockType::Forecast
    }
}
```

### Policy 6: Block Repetitive Similarity (Anti-Repeat)
- A question that shares the same `question_family_id` as a question already attempted in the current mock receives a 0.25× `anti_repeat_penalty` applied to its MockSelectionScore.
- Questions attempted in the **immediately preceding mock session** also receive a reduced anti-repeat penalty of 0.15×.

```rust
pub const ANTI_REPEAT_SAME_MOCK_PENALTY: f64 = 0.25;
pub const ANTI_REPEAT_PREV_MOCK_PENALTY: f64 = 0.15;
```

---

## 5.7 Readiness Proof Rules

A readiness claim is a formal statement that the student is prepared to sit the exam. It requires passing ALL of the following checks.

| Check | Threshold |
|-------|-----------|
| Minimum attempts per topic | ≥ 15 attempts on the topic |
| Minimum session spread per topic | Attempts spread across ≥ 3 separate sessions |
| Timed evidence requirement | ≥ 5 timed attempts per topic |
| Danger zone check | NO exam-critical topic with mastery < 4000 bp |
| Coverage threshold | ≥ 80% of syllabus topics at ≥ Functional mastery (5500 bp) |
| No critical danger zones | All topics flagged `is_exam_critical = true` must have mastery ≥ 5500 bp |
| Mock requirement | ≥ 1 full mock completed (any `MockType`) |

```rust
pub const READINESS_MIN_ATTEMPTS_PER_TOPIC: u32 = 15;
pub const READINESS_MIN_SESSIONS_PER_TOPIC: u32 = 3;
pub const READINESS_MIN_TIMED_ATTEMPTS: u32 = 5;
pub const READINESS_MIN_COVERAGE_PERCENT: f64 = 0.80;   // 80% of topics
pub const READINESS_MIN_COVERAGE_MASTERY: BasisPoints = 5_500; // Functional band
pub const READINESS_CRITICAL_TOPIC_GATE: BasisPoints = 5_500;
pub const READINESS_MIN_MOCKS_COMPLETED: u32 = 1;

/// Result of a readiness proof check.
#[derive(Debug)]
pub struct ReadinessProofResult {
    pub passed: bool,
    pub failures: Vec<ReadinessProofFailure>,
}

#[derive(Debug)]
pub enum ReadinessProofFailure {
    InsufficientAttempts { topic_id: i64, actual: u32 },
    InsufficientSessionSpread { topic_id: i64, actual: u32 },
    InsufficientTimedAttempts { topic_id: i64, actual: u32 },
    DangerZonePresent { topic_id: i64, mastery: BasisPoints },
    CoverageBelow80 { actual_percent: f64 },
    CriticalTopicBelow { topic_id: i64, mastery: BasisPoints },
    NoMockCompleted,
}
```

---

## 5.8 Parent Alert Trigger Rules

The Parent Alert Engine evaluates these rules after every session completion and on a daily scheduled check. Alerts are generated when any condition is met. Alerts are deduplicated within a 24-hour window (same alert type + same student = only one alert per day).

| Alert Type | Trigger Condition | Severity |
|-----------|-------------------|---------|
| `inactivity` | No study for ≥ 3 consecutive days | `watch` |
| `decline` | Mastery dropped ≥ 1000 bp in any subject over a 7-day window | `urgent` |
| `exam_near` | ≤ 14 days to exam date AND readiness score < 6500 bp | `urgent` |
| `mock_overdue` | ≥ 14 days since last mock AND exam date < 60 days away | `watch` |
| `subject_lagging` | One subject ≥ 2000 bp below the student's average mastery across subjects | `watch` |
| `misconception_repeat` | Same concept failed ≥ 3 times in last 5 attempts | `watch` |
| `false_confidence` | High-confidence wrong answers on exam-critical topic in last session | `watch` |

```rust
pub const ALERT_INACTIVITY_DAYS: u32 = 3;
pub const ALERT_DECLINE_BP_THRESHOLD: BasisPoints = 1_000;
pub const ALERT_DECLINE_WINDOW_DAYS: u32 = 7;
pub const ALERT_EXAM_NEAR_DAYS: i64 = 14;
pub const ALERT_EXAM_NEAR_READINESS_GATE: BasisPoints = 6_500;
pub const ALERT_MOCK_OVERDUE_DAYS: u32 = 14;
pub const ALERT_MOCK_OVERDUE_EXAM_WINDOW_DAYS: i64 = 60;
pub const ALERT_SUBJECT_LAG_BP: BasisPoints = 2_000;
pub const ALERT_MISCONCEPTION_REPEAT_COUNT: u32 = 3;
pub const ALERT_MISCONCEPTION_REPEAT_WINDOW: u32 = 5; // last N attempts
```

**Alert Deduplication Rule:**
- If an alert of the same `alert_type` for the same `student_account_id` was created within the last 24 hours, skip creating a new one.
- Parent sees a read/unread flag; unread alerts display a badge count.

---

## 5.9 Game Engine Rules

### 5.9.1 MindStack (Tetris) — Scoring and Progression Rules

MindStack maps academic performance to Tetris mechanics. The quality of each answer determines how much control the player has over the falling block.

**Control Level by Answer Quality:**

| Answer Quality | Lateral Movement | Rotation | Reshape | Gravity |
|---------------|-----------------|---------|--------|--------|
| No mastery (wrong, no hints) | None | None | None | Fast (normal) |
| Partial (wrong + hint, or low-confidence correct) | 1–2 moves | None | None | Normal speed |
| Good (correct, independent) | Full | Full | None | Normal speed |
| Excellent (correct + fast + streak) | Full | Full | 1 reshape option | Slowed |

**Question-to-Power Mapping:**

| Question Type Answered Correctly | Power Unlocked |
|---------------------------------|----------------|
| Recall | Unlock lateral movement |
| Concept understanding | Unlock rotation |
| Reasoning | Unlock reshaping |
| Fast answer streak (3+ correct fast) | Temporary gravity slowdown + board-clear bomb + shield |

**Streak Bonuses:**

| Streak | Bonus |
|--------|-------|
| 3 consecutive correct | Stronger pull / score multiplier activated |
| 5 consecutive correct | "Overdrive" state — gravity halved, 2× score multiplier |

**Mercy Rules (Preventing Instant Loss):**
1. One wrong answer does not immediately lose the game — the block still falls but with reduced control.
2. The player may recover full control on the very next correct answer.
3. Early game levels have a grace zone (wider board clearing threshold).
4. Streak forgiveness: 4 correct + 1 miss does not cancel the streak multiplier; requires 2 misses in a row.
5. Rescue tokens are available (limited per session): Stabilize Block, Retry Answer, Slow Gravity.

**Session Score Formula:**
```rust
/// MindStack session score computation.
///
/// `base_points` = questions answered correctly × base_points_per_question
/// `streak_multiplier` = 1.0 + (0.1 × streak_length), capped at 2.0
/// `level_multiplier` = 1.0 + (0.05 × current_level)
/// `time_bonus` = max(0, target_time_seconds - actual_time_seconds) × 2
pub fn mindstack_session_score(
    correct_answers: u32,
    base_points_per_question: u32,
    streak_length: u32,
    current_level: u32,
    time_bonus: u32,
) -> u64 {
    let base = (correct_answers * base_points_per_question) as f64;
    let streak_multiplier = (1.0 + 0.1 * streak_length as f64).min(2.0);
    let level_multiplier = 1.0 + 0.05 * current_level as f64;
    ((base * streak_multiplier * level_multiplier) as u64) + time_bonus as u64
}
```

---

### 5.9.2 Tug of War (MindPull) — Scoring and Progression Rules

Two-player or player-vs-AI competitive academic game. Correct answers pull the rope toward the player's side; wrong answers let it slip back.

**Rope Zones:**

| Zone | Description | Effect |
|------|-------------|--------|
| `NeutralZone` | Starting position | Neither side has advantage |
| `PressureZone` | Opponent has slight advantage | Player must answer faster |
| `RecoveryZone` | Player has slight advantage | Maintain with correct answers |
| `VictoryZone` | Player has decisive advantage | One more correct = win |
| `CollapseZone` | Opponent has decisive advantage | One more wrong = loss |

**Pull Mechanics:**

| Answer Outcome | Rope Movement |
|---------------|---------------|
| Correct (normal) | +1 unit toward player |
| Correct (3-answer streak) | +2 units toward player |
| Correct (5-answer streak "Overdrive") | +3 units toward player ("Overdrive Pull") |
| Wrong | −1 unit (rope slips toward opponent) |
| Timed out | −1 unit |

**Power-Ups (available during game):**

| Power-Up | Effect |
|---------|--------|
| Freeze Slip | Next wrong answer does not move the rope |
| Double Pull | Next correct answer counts as 2 pulls |
| Time Shield | Adds 3 extra seconds to next question timer |
| Hint Rope | Uses a small clue; if answered correctly, only +0.5 units (reduced reward) |
| Misconception Scan | Reveals which distractor is the trap; reduces pull reward by 20% if used |

**Win Condition:** Rope reaches `VictoryZone` end boundary. Default rope length = 10 units. Win requires pulling 5+ units from centre.

**Session Score Formula:**
```rust
/// Tug of War session score.
/// Combines accuracy, speed, streak performance, and power-up efficiency.
///
/// `net_pulls` = total pulls toward player minus slips
/// `avg_response_ms` = average response time in milliseconds
/// `streak_best` = best consecutive-correct streak in session
/// `power_ups_used` = number of power-ups consumed (subtracts from score)
pub fn tug_of_war_session_score(
    net_pulls: i32,
    correct_count: u32,
    total_count: u32,
    streak_best: u32,
    power_ups_used: u32,
) -> BasisPoints {
    if total_count == 0 {
        return 0;
    }
    let accuracy = correct_count as f64 / total_count as f64;
    let pull_contribution = (net_pulls.max(0) as f64 / 10.0).min(1.0); // normalise to rope length
    let streak_bonus = (streak_best as f64 * 0.02).min(0.20);
    let power_penalty = (power_ups_used as f64 * 0.02).min(0.10);

    let raw = 0.50 * accuracy + 0.30 * pull_contribution + streak_bonus - power_penalty;
    to_bp(raw.max(0.0))
}
```

---

## 5.10 Elite Mode Gate Rules

Elite Mode is the highest-intensity practice tier. Access requires meeting both an entitlement and a performance threshold.

### Entry Requirements

| Requirement | Threshold |
|-------------|-----------|
| Entitlement tier | Must be `EntitlementTier::Elite` (premium subscriber) |
| Mastery breadth | mastery ≥ 8500 bp (ExamReady band) across ≥ 70% of enrolled topics |
| Entry path A (hard gate) | Both conditions above must be true simultaneously |
| Entry path B (self-select) | Any student may attempt Elite; the system calibrates them to `EliteTier::Foundation` with a warning displayed |

```rust
/// Entitlement tiers for feature gating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementTier {
    Free,
    Standard,
    Elite,
}

pub const ELITE_MASTERY_GATE: BasisPoints = 8_500;
pub const ELITE_TOPIC_COVERAGE_PERCENT: f64 = 0.70; // 70% of enrolled topics

/// Check if a student qualifies for Elite Mode (hard gate).
///
/// `topic_masteries`: mastery scores for all enrolled topics.
pub fn elite_mode_gate_passes(
    entitlement: &EntitlementTier,
    topic_masteries: &[BasisPoints],
) -> bool {
    if !matches!(entitlement, EntitlementTier::Elite) {
        return false;
    }
    if topic_masteries.is_empty() {
        return false;
    }
    let qualifying = topic_masteries.iter()
        .filter(|&&m| m >= ELITE_MASTERY_GATE)
        .count();
    let coverage = qualifying as f64 / topic_masteries.len() as f64;
    coverage >= ELITE_TOPIC_COVERAGE_PERCENT
}
```

### EPS Scoring and Benchmarking

- The `ElitePerformanceScore` (see §4.15) is computed after every Elite session.
- Benchmarks are maintained per subject as percentile bands (Top 10% / Top 25% / Top 50%).
- A student must maintain EPS ≥ 7000 bp across 3 consecutive elite sessions to be promoted to the next `EliteTier`.

```rust
pub const EPS_PROMOTION_GATE: BasisPoints = 7_000;
pub const EPS_PROMOTION_SESSIONS_REQUIRED: u32 = 3;

/// Elite tier levels within Elite Mode itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EliteTier {
    /// Accuracy discipline, cleaner thinking, light pressure.
    Foundation,
    /// Mixed challenge, trap handling, faster reasoning.
    Core,
    /// Ruthless precision, sustained perfection, exam-grade intensity.
    Apex,
    /// Perfect streaks, no-hint runs, timed boss sets, elite rankings.
    Legend,
}
```

### Elite Daily Missions

- Every Elite-tier student is assigned a **daily high-stakes challenge** from the mission pool.
- Missions are selected by `EPS + MockSelectionScore` joint ranking.
- Completing a daily mission awards a `legend_point`; failing deducts one.
- Badges awarded for milestone legend-point totals (see §6.7 of the supplement).
- Elite mission types include: Perfect Run (one error collapses run), Speed Authority (all correct under target time), Trap Hunter (all trap questions answered correctly), Examiner-Proof (all past-question-pattern questions correct).

```rust
pub const ELITE_LEGEND_POINTS_AWARDED: i32 = 1;
pub const ELITE_LEGEND_POINTS_DEDUCTED: i32 = -1;

/// Elite badge thresholds (cumulative legend points).
pub const BADGE_PRECISION_BEAST: i32 = 10;
pub const BADGE_TRAP_HUNTER: i32 = 25;
pub const BADGE_SPEED_AUTHORITY: i32 = 50;
pub const BADGE_PERFECT_RUN: i32 = 75;
pub const BADGE_DISTINCTION_MACHINE: i32 = 100;
pub const BADGE_NO_HINT_MASTER: i32 = 150;
pub const BADGE_EXAMINER_PROOF: i32 = 200;
```

---

## Summary Constants Reference

```rust
// ── Mastery bands ────────────────────────────────────────────────
pub const MASTERY_BLOCKED_MAX:     BasisPoints = 3_999;
pub const MASTERY_EMERGING_MIN:    BasisPoints = 4_000;
pub const MASTERY_EMERGING_MAX:    BasisPoints = 5_499;
pub const MASTERY_FUNCTIONAL_MIN:  BasisPoints = 5_500;
pub const MASTERY_FUNCTIONAL_MAX:  BasisPoints = 6_999;
pub const MASTERY_STABLE_MIN:      BasisPoints = 7_000;
pub const MASTERY_STABLE_MAX:      BasisPoints = 8_499;
pub const MASTERY_EXAM_READY_MIN:  BasisPoints = 8_500;
pub const MASTERY_EXAM_READY_MAX:  BasisPoints = 8_999;
pub const MASTERY_MASTERED_MIN:    BasisPoints = 9_000;

// ── EMA ───────────────────────────────────────────────────────────
pub const EMA_ALPHA: f64 = 0.3;

// ── Rise stage gates ─────────────────────────────────────────────
pub const RISE_GATE_RESCUE_FOUNDATION:    BasisPoints = 4_500;
pub const RISE_GATE_RESCUE_MISCONCEPTION: BasisPoints = 3_500; // must be BELOW
pub const RISE_GATE_STABILIZE_RECALL:     BasisPoints = 6_000;
pub const RISE_GATE_STABILIZE_ACCURACY:   BasisPoints = 6_500;
pub const RISE_GATE_ACCELERATE_SPEED:     BasisPoints = 7_500;
pub const RISE_GATE_ACCELERATE_PRESSURE:  BasisPoints = 7_000;

// ── Coach state / session ────────────────────────────────────────
pub const TOPIC_BLOCK_ACCURACY_THRESHOLD:   f64 = 0.40;  // < 40% after 2 sessions → block
pub const TOPIC_UNBLOCK_ACCURACY_THRESHOLD: f64 = 0.60;  // ≥ 60% → unblock
pub const MISCONCEPTION_FORCE_REPAIR_COUNT: u32 = 3;     // recurs 3 times → queue repair
pub const SESSION_MAX_DURATION_MINUTES:     u32 = 90;
pub const SESSION_MIN_ATTEMPTS:             u32 = 3;

// ── Forecasting ───────────────────────────────────────────────────
pub const FORECAST_HIGH_BAND:     BasisPoints = 7_000;
pub const FORECAST_MEDIUM_BAND:   BasisPoints = 4_500;
pub const FORECAST_SURPRISE_BAND: BasisPoints = 3_000;

// ── Readiness ─────────────────────────────────────────────────────
pub const READINESS_COVERAGE_THRESHOLD:  f64         = 0.80;
pub const READINESS_MIN_MOCK_COUNT:      u32         = 1;
pub const READINESS_EXAM_NEAR_DAYS:      i64         = 14;
pub const READINESS_CRITICAL_TOPIC_GATE: BasisPoints = 4_000;

// ── Parent alerts ─────────────────────────────────────────────────
pub const ALERT_INACTIVITY_DAYS:          u32         = 3;
pub const ALERT_DECLINE_THRESHOLD:        BasisPoints = 1_000;
pub const ALERT_EXAM_NEAR_READINESS_GATE: BasisPoints = 6_500;
pub const ALERT_SUBJECT_LAG_THRESHOLD:    BasisPoints = 2_000;
```

---

*End of Part 4 and Part 5.*
