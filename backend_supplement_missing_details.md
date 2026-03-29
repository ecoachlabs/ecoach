# eCoach — Missing Details Supplement
## Deep Re-Read of All 38 Files | Gap-Fill for Detailed Implementation Plan
## Generated: 2026-03-29

This document captures every specific detail found during a second-pass micro-chunked re-read of all 38 source idea files that was not fully specified in `detailed_backend_implementation_plan.md`. It is organized by module/domain and cross-referenced by source file.

---

## TABLE OF CONTENTS

1. [Mock Centre & Forecast Engine — Full Formulas](#1-mock-centre--forecast-engine)
2. [Journey Mode — Full Engine Specs](#2-journey-mode)
3. [Identity System — Full Schema](#3-identity-system)
4. [Rise Mode (Weakest-to-Best Engine)](#4-rise-mode)
5. [Knowledge Gap Mode — Full Spec](#5-knowledge-gap-mode)
6. [Elite Mode — Full Spec](#6-elite-mode)
7. [Memory Mode — Full Spec](#7-memory-mode)
8. [Question Factory — Full 60-Type List & Operators](#8-question-factory)
9. [Game Engines — Tug of War & MindStack Detail](#9-game-engines)
10. [Wrong Answer Intelligence — Full Spec](#10-wrong-answer-intelligence)
11. [Premium Concierge — Full Architecture](#11-premium-concierge)
12. [Pedagogy Engine — Content Type Strategies](#12-pedagogy-engine)
13. [Coach Architecture — State Machine & Evidence Model](#13-coach-architecture)
14. [Academic Resource Intelligence (ARIL)](#14-academic-resource-intelligence)
15. [Question Intelligence — Multi-Axis Classification](#15-question-intelligence)
16. [Academic Decay & Recall Resilience Engine](#16-academic-decay--recall-resilience-engine)
17. [Traps Feature — Difference Drill Full Spec](#17-traps-feature)
18. [Diagnostic Test (Academic DNA)](#18-diagnostic-test-academic-dna)
19. [Smart Central Curriculum — Full Schema](#19-smart-central-curriculum)
20. [CoachHub Goals & Document Upload](#20-coachhub-goals--document-upload)
21. [Intelligence Constitution — Engine Registry](#21-intelligence-constitution)
22. [Curriculum Intelligence Portal — Full Pipeline](#22-curriculum-intelligence-portal)
23. [Time Orchestration Engine — Full Spec](#23-time-orchestration-engine)
24. [Coach Brain — Teaching & Intervention Model](#24-coach-brain)
25. [Master Formulas Not Previously Captured](#25-master-formulas-not-previously-captured)

---

# 1. MOCK CENTRE & FORECAST ENGINE
**Source: idea1.txt**

## 1.1 Forecast Score Formula (FULL)
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

Output probability bands: High / Medium / Surprise Risk / Uncertain

## 1.2 Mock Orchestration Formula (FULL)
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

## 1.3 Mock Selection Formula (FULL)
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

## 1.4 Weakness Scoring Formula (FULL)
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

## 1.5 Readiness Formula
```
Readiness =
  0.45 × Mastery
+ 0.20 × TimedPerformance
+ 0.15 × Coverage
+ 0.10 × Consistency
+ 0.10 × Trend
```

## 1.6 Predicted Exam Score Formula
```
PredictedExamScore =
  ∑(k ∈ BlueprintTargets)
    BlueprintWeight_k × Mastery_k × TimingFactor_k × RetentionFactor_k × MisconceptionImmunity_k
```

## 1.7 Six Mock Types
```rust
pub enum MockType {
    Forecast,     // Maximum realism, mirrors likely exam
    Diagnostic,   // Maximum insight, reveals gaps
    Remediation,  // Closes known weak areas
    FinalExam,    // Full readiness proof
    Shock,        // Resilience training, hard unexpected items
    Wisdom,       // Mastery proof, elite standards
}
```

## 1.8 Post-Mock Analysis (8 Sections)
1. Overall summary: score, predicted BECE range, confidence, timing, readiness movement
2. Subject/topic performance: strong/weak/unstable topics
3. Link-level diagnosis: exact broken concept links, prerequisite failures, bundles
4. Misconception diagnosis: top misconceptions, count per misconception, status (suspected/active/unresolved)
5. Representation diagnosis: text vs diagram vs graph vs table
6. Timing diagnosis: slow-but-correct, fast-but-careless, collapse near end, section pacing
7. Confidence diagnosis: correct-but-unsure, wrong-but-confident, guessing rate
8. Action plan: repair now / drill / review / next mock date

## 1.9 Past Paper Ingestion Metadata (Per Question)
- subject, topic, atom_ids, link_ids
- intent_type, representation_type, difficulty_band
- misconception_targets, marks_weight_proxy
- year, paper, section, number

## 1.10 Orchestration Policies
1. Fix prerequisites first
2. Prioritize unresolved misconceptions
3. Enforce coverage
4. Rotate representations
5. Increase exam simulation near exam
6. Block repetitive similarity

## 1.11 Four Initial Subjects (BECE)
- English Language
- Mathematics
- Integrated Science
- Social Studies

## 1.12 Question Metadata Fields (Full)
- subject, strand/sub-strand, objective
- difficulty, question_type, answer
- distractor_reasoning, time_expectation
- common_misconception_tag
- source_type: `past | adapted | original`

## 1.13 Rust Module Layout for Mock Centre
```
mock_centre::ingest   — past paper ingestion, record parsing, syllabus mapping, provenance
mock_centre::forecast — frequency computation, recency/trend, bundle detection, blueprint emission
mock_centre::compiler — mock assembly from forecast
mock_centre::analyzer — post-attempt analysis and diagnosis
mock_centre::sync     — local persistence
```

---

# 2. JOURNEY MODE
**Source: idea2.txt**

## 2.1 Journey Core Inputs (Full)
- Exam target: which exam, which subjects, exam date, exam board/curriculum
- Target: score, grade, or readiness goal
- Time reality: days left, hours/day/week, preferred study periods, unavailable days
- Path intensity: relaxed | balanced | intense
- Current level: self-assessment, quick diagnostic, past performance, confidence per topic
- Learning style: reading speed, preferred format (videos/notes/questions/flashcards/mixed), distraction tendency, session length preference, pressure preference

## 2.2 Journey Phases (5)
```rust
pub enum JourneyPhase {
    StabilizeFoundation,  // Fix major weaknesses and core concepts
    BuildCore,            // Cover main syllabus, deepen understanding
    StrengthenWeakLinks,  // Attack weak topics and recurring mistakes
    ExamConditioning,     // Timed questions, pressure, mixed-topic practice
    FinalReadiness,       // Mocks, revision bursts, confidence repair, exam strategy
}
```

## 2.3 Seven Journey Engines
1. Starting Point Engine: diagnostic tests, past scores, recent behavior → current level by topic, gap map, baseline
2. Deadline Pressure Engine: total days, realistic study days, weekly load, revision windows, mock windows, buffer → urgency level, feasible path, goal realism
3. Curriculum Decomposition Engine: break subjects into topic tree
4. Path Sequencing Engine: foundation-first | high-yield-first | confidence-first
5. Session Composer Engine: build each session (5-part structure)
6. Adaptation Engine: post-session adjustments
7. Exam Readiness Engine: coverage + mastery + retention + mock + speed + weakness + consistency

## 2.4 Session Structure (Standard)
- 10 min concept refresh
- 15 min guided learning
- 20 min practice
- 10 min error review
- 5 min recap

## 2.5 Session Structure (Stronger Learner)
- rapid quiz
- targeted timed drill
- mistake analysis
- mixed challenge set

## 2.6 Readiness Score Formula (Simple Version)
```
Readiness = 0.25 × topic_mastery
          + 0.20 × retention
          + 0.20 × mock_performance
          + 0.15 × speed
          + 0.10 × syllabus_coverage
          + 0.10 × consistency
Penalties for: weak critical topics, recurring mistakes, missed sessions,
               exam anxiety pattern, low recent performance trend
```

## 2.7 Learner State Model (7 Dimensions)
```rust
pub struct LearnerState {
    knowledge_state: TopicMasteryMap,      // What learner currently knows
    confidence_state: TopicConfidenceMap,   // How stable that knowledge is
    misconception_state: MisconceptionSet,  // Wrong mental models
    dependency_state: DependencyGapMap,     // Which weaknesses block others
    readiness_state: ReadinessEstimate,     // Exam performance capability
    coverage_state: SyllabusCoverageMap,    // Which parts of graph touched
    momentum_state: MomentumTrend,          // Improving/plateauing/regressing
}
```

## 2.8 Question Intent Classes (12)
```rust
pub enum QuestionIntent {
    Discovery,            // Find where learner stands
    Diagnosis,            // Specific weakness probe
    Confirmation,         // Verify improvement from another angle
    Coverage,             // Breadth of syllabus
    Reinforcement,        // Strengthen fragile gains
    MisconceptionExposure,// Test if wrong mental model present
    RetentionCheck,       // After time has passed
    TransferCheck,        // New context/wording
    FluencyCheck,         // Can perform smoothly
    SpeedCheck,           // Timed performance
    ExamSimulation,       // Exam-like conditions
    ConfidenceRepair,     // Rebuild confidence
}
```

## 2.9 Learner State Levels
`untested → emerging → unstable → functional → stable → transferable → exam-ready → mastered`

## 2.10 User Response Archetypes
```rust
pub enum LearnerArchetype {
    WeakButConsistent,     // Foundation-first progression
    StrongButLazy,         // Shorter, sharper, accountable sessions
    PanickingLastMinute,   // High-yield triage mode
    Overconfident,         // Diagnostics + timed challenges to reveal blindspots
    Discouraged,           // Confidence-first sequencing with visible wins
}
```

## 2.11 Journey v1 Scope (Confirmed)
- exam date + target score setup
- baseline diagnostic
- topic gap map
- adaptive weekly plan
- daily session composer
- checkpoints every 1-2 weeks
- readiness score
- rescue replan logic
- final mock phase

---

# 3. IDENTITY SYSTEM
**Source: idea3.txt**

## 3.1 Account System Additional Detail

**Learner Account Full Fields:**
- account_id, account_type, display_name, avatar_path
- class_level, exam_target, curriculum_track
- subjects_enrolled (JSON array)
- local_pin (hashed)
- academic_history, progress_state
- performance_analytics, activity_timeline
- question_attempt_history, mastery_profile_by_topic

**Parent Account Additional Fields:**
- linked_learner_accounts (via join table)
- relationship_label per learner (son/daughter/ward)
- high-level academic summary across all learners
- alerts_and_recommendations
- viewing_permission_for_learner_history
- optional_controls (schedules, limits, printing, locking settings)

## 3.2 PIN System Rules (Full)
```
Learner PIN:
- 4-digit or 6-digit
- Simple and fast
- Changeable by parent
- Protects personal history

Parent PIN:
- 6 digits minimum
- Optional security question or recovery phrase
- Gates supervisory data and controls

Rules:
- Each account has separate PIN
- Wrong PIN lockout after repeated failed attempts
- Optional cooldown timer
- Parent can reset learner PINs
- Learner CANNOT access parent dashboard
```

## 3.3 Parent Dashboard 5 Sections (Full)
1. **Children Overview** — name, class/level, academic status, recent activity, strongest/weakest subject, readiness, streak, alert badge
2. **Performance Summary** — average score trend, subject breakdown, improvement/decline, test frequency, mock performance, readiness level
3. **History & Activity** — when studied, which subjects, tests taken, time spent, abandoned sessions, milestones
4. **Attention Needed** — auto-detected issues + recommended actions
5. **Detailed Child View** — full child breakdown with printable report option

## 3.4 Auto-Detected Issues (Parent Alert Engine)
- learner not studied several days
- scores declining
- weak topic persisting
- exam date approaching
- mock test frequency too low
- one subject severely lagging
- performance gap between children widening
- child failing same concept
- learner lost confidence

## 3.5 Internal → Parent Translation Logic
```
Internal: "math trend: -8% over 3 assessments, fractions mastery: low,
           activity drop: 5 inactive days, exam countdown: 17 days"

Parent:    "Ama needs urgent support in Mathematics. Her recent scores
           have fallen, especially in Fractions, and she has not practiced
           consistently this week. With the exam approaching, the best
           next step is 3 short Math revision sessions and one mock test."
```

## 3.6 Learner vs Parent UI Philosophy
- **Learner:** colorful, animated, game-like, journey-based, reward-oriented, motivational, interactive
- **Parent:** clean, spacious, low-clutter, big typography, simple icons, limited menu choices, plain-language signals

## 3.7 Additional Database Entities
```sql
-- Entities not in migration 001
CREATE TABLE parent_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_account_id INTEGER NOT NULL REFERENCES accounts(id),
    student_account_id INTEGER NOT NULL REFERENCES accounts(id),
    alert_type TEXT NOT NULL,   -- 'inactivity', 'decline', 'exam_near', etc.
    message TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'watch',
    is_read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    recommendation_type TEXT NOT NULL,
    message TEXT NOT NULL,
    source_engine TEXT NOT NULL,
    acted_on INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE report_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id),
    report_type TEXT NOT NULL,   -- 'weekly', 'monthly', 'term', 'pre_exam'
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    data_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

# 4. RISE MODE (Weakest-to-Best Engine)
**Source: idea4.txt**

## 4.1 Four Transformation Stages
```rust
pub enum RiseStage {
    Rescue,      // Stop the bleeding; find root gaps; first wins; shame removal
    Stabilize,   // Make correct thinking repeatable; scaffolded; concept clusters
    Accelerate,  // Speed + independence; timed drills; mixed topics; pressure mode
    Dominate,    // Outperform top students; trap questions; elite variants; speed+accuracy
}
```

## 4.2 Real Failure Reasons (Rise Mode Diagnostic Focus)
1. Foundational gaps — missed earlier concepts
2. Slow thinking speed — cannot answer fast enough
3. Weak recall — understand in class but forget during tests
4. Question blindness — don't recognize what a question is really asking
5. Low confidence/panic — pressure breaks thinking
6. Poor consistency — not enough practice
7. Wrong study method — reads a lot but doesn't train properly

## 4.3 Error Type → Intervention Mapping
```
forgot concept          → reteach + memory reinforcement
misunderstood wording   → wording training
calculation slip        → attention training
guessed                 → strategy training
panic under time        → pressure training
chose familiar distractor → distractor training
wrong first step        → step-selection exercises
incomplete reasoning    → reasoning training
misread units/keywords  → reading training
careless errors         → attention-control drills
```

## 4.4 Transformation Session Format
- 5 min concept repair
- 7 min targeted drills
- 5 min error replay
- 3 min memory recall
- 5 min timed pressure set
- 2 min confidence summary

## 4.5 Internal Intelligence Scores (8 Scores)
```rust
pub struct RiseStudentScores {
    foundation_score: BasisPoints,       // How broken are prerequisite concepts?
    recall_score: BasisPoints,            // How well do they retain what they learned?
    speed_score: BasisPoints,             // How fast can they solve correctly?
    accuracy_score: BasisPoints,          // How often are they right?
    pressure_stability_score: BasisPoints,// Performance drop under pressure
    misconception_density: BasisPoints,   // Repeated wrong patterns count (normalized)
    momentum_score: BasisPoints,          // Rising / flat / falling
    transformation_readiness: BasisPoints,// Can they move to next stage?
}
```

## 4.6 Momentum Score Formula
```
MomentumScore = 0.35 × volume + 0.40 × accuracy + 0.25 × pace
```

## 4.7 Strain Score Formula
```
StrainScore = 0.30 × accuracy_drop + 0.20 × completion_drop
            + 0.20 × hint_spike + 0.15 × skip + 0.15 × pace_instability
```

## 4.8 Session Block Structure (Beat Yesterday Engine)
- Block 1: Warm Start
- Block 2: Core Climb
- Block 3: Speed Burst
- Block 4: Finish Strong

## 4.9 Pressure Mode Ladder
```rust
pub enum PressureLevel {
    Calm,
    GuidedTimed,
    Mild,
    Moderate,
    ExamPressure,
    ElitePressure,
}
```

---

# 5. KNOWLEDGE GAP MODE
**Source: idea6.txt**

## 5.1 Ten Knowledge Gap Types
```rust
pub enum KnowledgeGapType {
    Content,      // Never learned concept at all
    Understanding,// Seen it but don't truly understand it
    Application,  // Understand theory but cannot use in questions
    Process,      // Know idea but steps are weak
    Speed,        // Can solve but too slowly for exam conditions
    Accuracy,     // Know it but careless mistakes
    Retention,    // Once knew it but forgot
    Confidence,   // Hesitate/panic/avoid despite some knowledge
    Interest,     // Disengaged, never quality effort
    Transfer,     // Solve familiar but fail new versions
}
```

## 5.2 Gap Discovery Methods (7)
1. Diagnostic questions — short sets targeting exact weaknesses
2. Continuous passive detection — update gap map on every answer
3. Step analysis — inspect where student broke in solution path
4. Error pattern tracking — "keeps confusing mean and median"
5. Time analysis — hesitation signals uncertainty even on correct answers
6. Confidence check — low confidence despite correct = weak mastery
7. Variation testing — same concept different forms; performance drop = shallow

## 5.3 Core Loop
```
Detect → Explain → Repair → Recheck → Lock In → Expand
```

## 5.4 Session Addictive Hook (One Per Session)
- uncover a new hidden weakness
- close a visible gap
- increase coverage %
- reduce critical gaps
- unlock a mastery badge
- clear fog from map
- move closer to exam readiness

## 5.5 Gap Score Display
- Coverage: 28%
- Gap: 72%
- Critical gaps: N
- Hidden gaps: N
- Fixed this month: N

## 5.6 Gap Scoring Schema (Per Skill)
```sql
-- Columns needed on skill_mastery table
seen_flag INTEGER NOT NULL DEFAULT 0,
attempts INTEGER NOT NULL DEFAULT 0,
correct_rate INTEGER NOT NULL DEFAULT 0,  -- basis points
avg_response_time_ms INTEGER,
confidence_level INTEGER,                 -- 0-100
error_types TEXT,                         -- JSON array of error type codes
misconception_tags TEXT,                  -- JSON array
last_seen_at TEXT,
gap_type TEXT,
gap_severity INTEGER,                     -- basis points
gap_rank INTEGER                          -- priority rank within subject
```

---

# 6. ELITE MODE
**Source: idea5.txt**

## 6.1 Elite Progression Tiers
```rust
pub enum EliteTier {
    Foundation,  // Accuracy discipline, cleaner thinking, light pressure
    Core,        // Mixed challenge, trap handling, faster, deeper reasoning
    Apex,        // Ruthless precision, sustained perfection, exam-grade intensity
    Legend,      // Perfect streaks, no-hint runs, timed boss sets, elite rankings
}
```

## 6.2 Seven Pillars of Elite Mode
```rust
pub enum ElitePillar {
    Precision,   // Near-identical options, trap-heavy, wording sensitivity, careless detection
    Speed,       // Compressed time, time-to-first-step, personal best records
    Depth,       // Multi-step reasoning, concept blending, method comparison
    Endurance,   // Longer clean runs, accuracy decay tracking, fatigue analytics
    TrapSense,   // Trap library by topic, misconception distractors, "most likely error" review
    ElegantSolve,// Shortest valid method, most efficient reasoning path, expert vs ordinary way
    Pressure,    // Countdown intensity, rising tempo, no pauses, streak preservation
}
```

## 6.3 Elite Session Types
```rust
pub enum EliteSessionType {
    EliteSprint,    // Short high-intensity (speed + precision + warm-up)
    EliteGauntlet,  // Mixed challenge all-round
    PrecisionLab,   // Trap sensitivity + careful reading
    DepthLab,       // Fewer questions, harder reasoning
    PerfectRun,     // One error collapses run (focus + mental discipline)
    EnduranceTrack, // Longer session with score stability tracking
    ApexMock,       // Elite-standard exam simulation
}
```

## 6.4 Ten Elite Question Types
```rust
pub enum EliteQuestionType {
    Lean,              // Less scaffolding, fewer clues
    Precision,         // One word changes everything
    MultiHop,          // Multiple reasoning hops (syllabus-based)
    MethodComparison,  // "Which approach is most efficient?"
    ErrorDiagnosis,    // "Where did this student go wrong?"
    Reverse,           // "Which condition must be true for this answer?"
    IncompletePath,    // "Choose the best next step"
    NearMissDistractor,// Attractive to strong but careless students
    TimePressureMicro, // Short brutal bursts
    PerfectRunSet,     // One error breaks streak
}
```

## 6.5 Elite Entry Criteria
- Qualification: 80-90%+ mastery in topic + strong consistency + good speed + low hint dependence
- Self-selection: with app warning
- Hybrid: anyone can try, app calibrates to Foundation/Core/Apex

## 6.6 Elite Performance Dimension Profile
```rust
pub struct ElitePerformanceDimensions {
    speed: BasisPoints,
    precision: BasisPoints,
    endurance: BasisPoints,
    reasoning_depth: BasisPoints,
    trap_resistance: BasisPoints,
    consistency: BasisPoints,
    confidence_calibration: BasisPoints,
    elegance_score: BasisPoints,
}
```

## 6.7 Elite Badges
- Precision Beast
- Trap Hunter
- Speed Authority
- Perfect Run
- Distinction Machine
- No-Hint Master
- Examiner-Proof

---

# 7. MEMORY MODE
**Source: idea7.txt**

## 7.1 MSI Formula (6-Component Version)
```
MSI = (0.30 × A) + (0.15 × S) + (0.20 × R) + (0.15 × V) + (0.10 × I) + (0.10 × C)

Where:
  A = recent accuracy (across last N attempts)
  S = recall speed (latency normalized)
  R = retention over time (decay resistance)
  V = variant transfer (performance on varied question forms)
  I = independence (without hints or prompts)
  C = connection (used correctly inside larger problems)
```

## 7.2 Memory Strength Status Labels
| Range | Status |
|-------|--------|
| 85-100 | Locked In |
| 70-84 | Stable |
| 55-69 | Vulnerable |
| 40-54 | Fading |
| 0-39 | Critical Recovery Needed |

## 7.3 Session Size Parameters Per Type
| Session Type | Size | Purpose |
|-------------|------|---------|
| Memory Scan | 8-15 questions | Identify strong/weak areas, fading zones |
| Rescue Burst | 6-10 questions | Stop the slide, restore familiarity |
| Deep Repair | 12-20 questions | Repair prerequisites, rebuild broken chain |
| Recall Builder | 8-12 questions | Strengthen speed, deepen stability |
| Chain Repair | 10-18 questions | Repair root skill, reconnect dependents |
| Rapid Recall Drill | 30-90 seconds or 10 prompts | Fluency, automatic retrieval |

## 7.4 Question Recovery Ladder (6 Stages)
```rust
pub enum RecoveryStage {
    Recognition,     // "Have you seen this before?"
    GuidedRecall,    // "Can you remember it with a clue?"
    UnguidedRecall,  // "Can you recall it without support?"
    VariationRecall, // "Can you still recall it when question changes form?"
    ConnectedRecall, // "Can you use it inside a related problem?"
    PressureRecall,  // "Can you still retrieve it quickly?"
}
```

## 7.5 Spaced Recheck Schedule
```
Intervals: 1 day → 3 days → 7 days → 14 days → 30 days
```

## 7.6 Memory Evidence Score (Total = 100 points)
| Component | Points |
|-----------|--------|
| Independent Recall | 25 |
| Delayed Recall | 20 |
| Variant Recall | 20 |
| Embedded Use | 15 |
| Speed Stability | 10 |
| Recheck Stability | 10 |

## 7.7 Memory Evidence States
| Score | State |
|-------|-------|
| 0-39 | Not in memory yet |
| 40-59 | Emerging |
| 60-74 | Fragile memory |
| 75-89 | Strong memory |
| 90-100 | Locked in |

## 7.8 Memory State Machine (12 States)
```rust
pub enum MemoryState {
    Unformed,          // State 0
    Exposed,           // State 1
    Familiar,          // State 2
    Recognizable,      // State 3
    SupportedRecall,   // State 4
    FreeRecall,        // State 5
    AppliedRecall,     // State 6
    TransferRecall,    // State 7
    PressureStable,    // State 8
    DurableMastery,    // State 9 (formerly "Locked In")
    AtRisk,            // State 10
    Collapsed,         // State 11
}
```

## 7.9 Decay Signals Engine Detects
- Rising response time despite prior mastery
- Recent accuracy drop after time gap
- Strong performance only on familiar question shapes
- Repeated confusion between similar concepts
- More hint usage than before
- Previously mastered skill failing in mixed contexts
- Performance collapse in dependent skills

## 7.10 Six Core Memory Engines
1. Memory Strength Engine
2. Decay Detection Engine
3. Micro-Recall Mapper
4. Recovery Planner
5. Connection Rebuilder
6. Recheck Scheduler

## 7.11 Student Memory Profile Fields (Per Skill)
- total_attempts, total_correct, recent_correct_rate
- avg_response_time_ms, response_time_trend
- last_seen_at, last_correct_at
- hint_usage_rate, confidence_history (JSON)
- variant_performance_history (JSON)
- relapse_count
- prerequisite_dependency_health
- memory_stability_score
- current_status_label

---

# 8. QUESTION FACTORY
**Source: idea9.txt**

## 8.1 Complete 60-Question Type List

**Family A — Memory Questions (6):**
Pure Recall, Recognition, Memory Reconstruction, Retrieval Under Pressure, Retention Check, Recovery

**Family B — Understanding Questions (6):**
Concept Understanding, Explanation, Example Generation, Non-Example, Compare-and-Contrast, Classification

**Family C — Reasoning Questions (6):**
Reasoning, Logical Deduction, Inference, Justification, Claim Evaluation, Counterexample

**Family D — Problem-Solving Questions (8):**
Application, Transfer, Multi-Step Problem Solving, Strategy Selection, First-Step, Next-Step, Decision-Making, Prioritization

**Family E — Accuracy Questions (5):**
Error Detection, Correction, Misconception Exposure, Precision, Attention Control

**Family F — Pattern and Structure Questions (8):**
Pattern Recognition, Rule Discovery, Sequence/Order, Cause-and-Effect, Prediction, Abstraction, Estimation, Representation Conversion

**Family G — Expression and Interpretation Questions (7):**
Interpretation, Visualization, Mental Manipulation, Synthesis, Connection-Making, Judgment, Open-Ended Reasoning

**Family H — Growth-Control Questions (14):**
Diagnostic, Mastery Check, Threshold, Adaptive Difficulty, Rescue, Challenge/Stretch, Reflection/Metacognitive, Confidence Calibration, Real-World Scenario, Reverse Reasoning, Multiple-Path, Deep Thinking, Speed Fluency, Capstone

## 8.2 Core 12 Question Types (Build First)
1. Recall
2. Memory Reconstruction
3. Concept Understanding
4. Explanation
5. Reasoning
6. Application
7. Multi-Step Problem Solving
8. Error Detection
9. Misconception Exposure
10. Transfer
11. Diagnostic
12. Retention Check

## 8.3 Q_Difficulty Formula (7-Component Vector)
```
Q_difficulty = B_c + S_l + A_b + D_s + R_s + T_p + C_u

Where:
  B_c = base concept difficulty
  S_l = solution length
  A_b = abstraction load
  D_s = distractor strength
  R_s = representation shift load
  T_p = time pressure
  C_u = context unfamiliarity
```

## 8.4 Concept Record Full Fields
- concept_id, topic, subtopic, skill, difficulty_band
- concept_statement, definition, core_facts
- rules_laws_formulae, steps_procedure
- examples, non_examples, common_misconceptions
- causes, effects, prerequisites, related_concepts
- representation_forms (verbal, symbolic, numeric, diagrammatic, graphical)
- edge_cases_tricky_boundaries
- application_contexts, proof_signals, error_signatures

## 8.5 Ten Freshness Methods (Controlled Variation)
1. Parameter variation (change numbers, names, objects, cases)
2. Surface paraphrase (different wording, same intent)
3. Context recasting (same deep structure, different scenario)
4. Representation shift (table → graph → story → symbolic)
5. Step slicing (first step, middle step, full step, repair step)
6. Distractor mutation (swap misconception bait)
7. Cue adjustment (more or fewer hints)
8. Difficulty band shift (same family, different strength)
9. Time condition shift (untimed → lightly timed → pressure)
10. Memory distance shift (immediate → 2-day → 1-week → 1-month)

## 8.6 Universal Generation Operators (21)
```rust
pub enum TransformationOp {
    Hide, Mask, Paraphrase, Reorder, Perturb,
    InsertError, Contrast, Transfer, CompressTime, ExpandExplanation,
    SwitchRepresentation, ReduceCues, AddDistractors, Fragment,
    Chain, Gate, Delay, WrapInScenario, IntroduceConflict,
    MirrorMisconception, ReverseReasoning,
}
```

## 8.7 Seven Thought-Provoking Question Patterns
1. Anomaly pattern — something surprising happens; explain it
2. Conflict pattern — two claims appear to disagree; resolve them
3. Boundary pattern — case sits at edge of rule; decide if rule still applies
4. Hidden assumption pattern — conclusion seems right until missing assumption exposed
5. Counterexample pattern — disprove a sweeping claim
6. Reverse reasoning pattern — given outcome, infer cause or missing condition
7. Multiple-path pattern — two methods possible; which is better and why?

---

# 9. GAME ENGINES
**Source: idea10.txt**

## 9.1 MindStack (Tetris) Control Ladder
```rust
pub enum ControlLevel {
    // Level 0: No mastery — block falls fast, no rotation, no lateral movement
    NoMastery,
    // Level 1: Partial mastery — move left/right 1-2x, no reshape, normal speed
    Partial,
    // Level 2: Good answer — full movement, full rotation, slower speed
    Good,
    // Level 3: Excellent/streak — full movement + rotation + 1 reshape + gravity slowdown + multiplier
    Excellent,
}
```

## 9.2 Reshape Options
- Option A: one-time morph
- Option B: choose from 3 morphs
- Option C: partial morph (remove or add one unit)

## 9.3 Question-to-Power Mapping
- Recall question → unlock movement
- Concept question → unlock rotation
- Reasoning question → unlock reshaping
- Fast answer streak → temporary gravity slowdown + board clear bomb + shield

## 9.4 Mercy Design Rules
1. Wrong hurts but doesn't instantly kill
2. Allow recovery on next question
3. Add grace zone for early levels
4. Streak forgiveness: 4 correct + 1 miss doesn't collapse everything
5. Rescue mechanics: tokens for (stabilize block, retry answer, slow gravity)

## 9.5 Five MindStack Variants
1. Answer-to-Control Tetris (base)
2. Answer-to-Rotation Tetris (simpler)
3. Answer-to-Morph Tetris (premium)
4. Answer-to-Time Tetris (simplest)
5. Dual-Answer Tetris (two questions per block)

## 9.6 Tug of War (MindPull) Rope Zones
```rust
pub enum RopeZone {
    NeutralZone,
    PressureZone,
    RecoveryZone,
    VictoryZone,
    CollapseZone,
}
```

## 9.7 Power-Ups (Tug of War)
- Freeze Slip (protects against one wrong answer)
- Double Pull (next correct counts twice)
- Time Shield (adds 3 extra seconds)
- Hint Rope (small clue but weaker reward)
- Misconception Scan (reveals trap to avoid)

## 9.8 Momentum Meter Rules
- 1 correct = normal pull
- 3 streak = stronger pull
- 5 streak = "Overdrive Pull"

---

# 10. WRONG ANSWER INTELLIGENCE
**Source: idea11.txt**

## 10.1 Twelve Error Classes
```rust
pub enum ErrorClass {
    RecallFailure,
    RecognitionTrap,
    ConceptConfusion,
    ApplicationFailure,
    MultiStepBreakdown,
    ReadingFailure,
    DistractorSeduction,
    CarelessExecution,
    PressureCollapse,
    FragileMastery,
    PrerequisiteGap,
    FalseConfidence,
}
```

## 10.2 Eight Error Signals (Diagnostic)
1. Chosen answer (maps to misconception via distractor tagging)
2. Response time: too fast = guess; too slow = uncertainty; medium-but-wrong = stable misconception
3. Answer changes: correct→wrong = self-doubt; wrong→wrong = concept fog
4. Confidence rating: high-but-wrong = false mastery; low-but-wrong = weak mastery
5. Steps taken (for step-based questions, where student diverged)
6. Previous mistake history (same topic, family, reasoning type, distractor, pressure condition)
7. Question language (indirect wording, negative phrasing, multi-condition)
8. Behavioral history (rushing, overthinking, collapsing under timers, missing exception words)

## 10.3 Question Metadata Intelligence Types (A-G)
- **A. Content tags:** subject, topic, subtopic, skill, subskill, curriculum objective, difficulty
- **B. Reasoning tags:** recall, recognition, comparison, inference, elimination, multi-step, calculation, interpretation, application, evaluation, synthesis
- **C. Family tags:** definition-distinction, formula-selection, unit-conversion, cause-vs-effect, etc.
- **D. Misconception tags:** "confuses perimeter with area", "confuses mass with weight"
- **E. Distractor intent tags:** why tempting, what misconception targeted, what student type would pick it
- **F. Correct reasoning path:** ideal path, minimum steps, alternate valid paths, key discriminators
- **G. Prerequisite links:** prerequisite concepts, subskills, supporting examples

## 10.4 Error Fingerprint Structure
```rust
pub struct ErrorFingerprint {
    subject: String,
    topic: String,
    subtopic: String,
    family: String,
    reasoning_type: String,
    chosen_distractor_type: String,
    misconception: String,
    timing_pattern: String,    // fast/slow/medium
    confidence: u8,
    severity: SeverityLevel,
    first_wrong_step: Option<String>,
    likely_cause: String,
}
```

## 10.5 Eight Intervention Types on Wrong Answer
```rust
pub enum WrongAnswerIntervention {
    InstantRepair,       // 1-3 mini questions on exact weakness
    ContrastRepair,      // Two similar questions side by side
    StepRepair,          // Break reasoning into steps, train only broken step
    MisconceptionRepair, // Directly attack false idea with targeted examples
    FamilyRepair,        // Fresh questions from same family with increasing support
    PrerequisiteRepair,  // Go backward, strengthen missing foundation
    PressureRepair,      // Retrain slowly first, reintroduce speed
    ReflectionRepair,    // Ask "What made B attractive?" + compare with system inference
}
```

## 10.6 Seven Live Profiles (Academic Analyst)
1. Concept Weakness Profile
2. Misconception Profile
3. Reasoning Profile (inference, elimination, comparison, multi-step, symbolic translation)
4. Distractor Vulnerability Profile (familiar keyword, half-true, extreme-option, reversed-cause, formula lookalikes)
5. Pressure Profile (timing effects)
6. Transfer Profile (familiar vs new forms)
7. Recovery Profile (relapse patterns)

## 10.7 Three Levels of Wrongness
```rust
pub enum WrongnessLevel {
    Surface,   // Final answer is wrong
    Process,   // Student used wrong path
    Pattern,   // Wrong path is part of recurring trend
}
```

## 10.8 Wrong Answer Review Card (10 Sections)
1. What you chose
2. Why [answer] looked reasonable
3. Why [answer] fails
4. Why [correct answer] is correct
5. Why [other answers] are wrong
6. What kind of mistake this is
7. What your brain likely did
8. The lesson
9. Your pattern history
10. Repair action

---

# 11. PREMIUM CONCIERGE
**Source: idea12.txt**

## 11.1 Six-Layer Premium Architecture
```rust
pub enum PremiumLayer {
    Diagnosis,         // Deep baseline testing + pattern analysis
    Strategy,          // Clear performance plan per child
    Execution,         // Daily adaptive learning + drills + memory support + weakness repair
    Oversight,         // Tracking, alerts, intervention logic
    HumanExcellence,   // Strategist reviews, specialist escalation, premium support
    ParentConfidence,  // Elegant reporting, certainty, clarity, calm
}
```

## 11.2 Premium Alert System Detection
- memory slippage
- repeated misconception family
- falling speed
- rising panic errors
- topic neglect
- exam-readiness risk
- false confidence
- plateau after improvement
- hidden weakness masked by easy question success

## 11.3 Premium Parent Command Center Fields
- overall readiness score
- subject risk map
- current intervention focus
- weekly progress summary
- top 3 concerns
- top 3 gains
- strategy changes this week
- exam countdown
- recommended parent action

## 11.4 Intervention Catalog (By Diagnosis)
- concept rebuild
- misconception correction
- speed training
- exam pressure training
- memory reinforcement
- multi-step reasoning repair
- distractor immunity training
- elite challenge mode
- careless error reduction mode

---

# 12. PEDAGOGY ENGINE — CONTENT TYPE STRATEGIES
**Source: idea29.txt**

## 12.1 Fourteen Content Types
```rust
pub enum ContentType {
    Definition,
    Concept,
    Process,
    Sequence,
    Comparison,
    Formula,
    Derivation,
    WorkedProcedure,
    Application,
    Interpretation,
    DiagramSpatial,
    EssayStructured,
    ProofJustification,
    WordProblemTranslation,
}
```

## 12.2 Sixteen Strategy Families
1. Boundary strategy
2. Contrast strategy
3. Example/non-example strategy
4. Semantic unpacking strategy
5. Causal chain strategy
6. Stage map strategy
7. Sequence chaining strategy
8. Representation switching strategy
9. Faded example strategy
10. Error exposure strategy
11. Explain-back strategy
12. Decision-gate strategy
13. Translation strategy
14. Structure-and-expression strategy
15. Proof-chain strategy
16. Timed-performance strategy

## 12.3 Definition Failure Modes
- Verbatim illusion (can recite but not apply)
- Boundary blur (can't say when it applies)
- Near-neighbor confusion (confused with similar term)
- Example-only dependence (knows examples, not definition)
- One-way knowledge (definition → term but not term → definition)
- Language trap (misleading phrasing in question)

## 12.4 Definition Mastery Evidence Requirements
- State definition in clear language
- Recognize valid examples
- Reject non-examples
- Distinguish from similar terms
- Use correctly in context
- Retrieve under pressure

## 12.5 Formula Mastery Evidence Requirements
- Read formula in words
- Know what each symbol means
- Know units/dimensions
- Know when formula applies and when it doesn't
- Substitute correctly
- Manipulate where needed
- Use to solve problems
- Connect to concept and context

## 12.6 Formula Typical Failure Modes
- Symbol blindness (doesn't know what variables mean)
- Substitution errors
- Chooses formula by keyword (not understanding)
- Cannot tell similar formulas apart
- Cannot interpret result
- Formula used outside valid conditions
- No sense of unit reasonableness

---

# 13. COACH ARCHITECTURE
**Source: idea20.txt**

## 13.1 CoachLifecycleState Enum (Full)
```rust
pub enum CoachLifecycleState {
    OnboardingRequired,
    SubjectSelectionRequired,
    DiagnosticRequired,
    ContentReadinessRequired,
    PlanGenerationRequired,
    ReadyForTodayMission,
    MissionInProgress,
    MissionReviewRequired,
    RepairRequired,
    BlockedOnTopic,
    PlanAdjustmentRequired,
    ReviewDay,
    ExamMode,
    StalledNoContent,
}
```

## 13.2 ContentReadinessStatus Enum
```rust
pub enum ContentReadinessStatus {
    Ready,
    NoSubjectsSelected,
    NoPacksInstalled,
    NoTopicsAvailable,
    TopicsExistButNoQuestions,
    InsufficientQuestionCoverage,
}
```

## 13.3 NextCoachAction Enum
```rust
pub enum NextCoachAction {
    ContinueOnboarding,
    SelectSubjects,
    StartDiagnostic,
    InstallContent,
    GeneratePlan,
    StartTodayMission,
    ResumeMission,
    ReviewResults,
    StartRepair,
    ViewTodayPlan,
}
```

## 13.4 Plan Engine V1 Data Model (Full)
```rust
pub struct Plan {
    exam_target: String,
    start_date: String,
    exam_date: String,
    selected_subjects: Vec<SubjectId>,
    daily_study_budget_minutes: u32,
    phase_structure: Vec<PlanPhase>,
    targets: Vec<TopicTarget>,
}

pub struct PlanDay {
    date: String,
    subject_focus: SubjectId,
    time_budget_minutes: u32,
    mission_types: Vec<MissionType>,
    target_outcomes: Vec<OutcomeTarget>,
    completion_status: PlanDayStatus,
}

pub struct TopicTrack {
    topic_id: TopicId,
    mastery_estimate: BasisPoints,
    fragility: BasisPoints,
    speed_score: BasisPoints,
    misconception_count: u32,
    exposure_count: u32,
    is_blocked: bool,
    is_unlocked: bool,
    last_intervention_type: Option<InterventionType>,
}
```

## 13.5 Progression Rules (Evidence-Based)
```
if attempts < minimum_evidence_threshold → don't classify yet
if 2 sessions AND accuracy < 40% → mark blocked
if misconception_family repeats 3 times → queue targeted_repair
if untimed_accuracy >= 75% AND timed_accuracy < 45% → queue speed_conversion
if strong recall BUT weak application → queue worked_example + transfer_drill
if stable over 3 exposures → unlock next dependency topic
```

## 13.6 Topic Blocking Rules
```
Block trigger:    accuracy < 40% after 2 sessions
Block action:     deploy reteach + worked_example + guided_practice
Unblock trigger:  accuracy >= 60%
Force repair:     same misconception recurs 3+ times
```

## 13.7 Eight-Phase Teaching Flow
```
Phase 1 — Diagnose:        pre-check, prerequisite check, misconception probe, confidence check
Phase 2 — Teach:           simple overview, vocabulary, visual+verbal, think-aloud, first worked example
Phase 3 — Guided Use:      solve with hints, explain step, choose method, compare concepts
Phase 4 — Independent Use: straightforward, disguised, diagram, mixed-format, timed
Phase 5 — Repair:          diagnose error, explain failed path, corrected reasoning, near-match
Phase 6 — Strengthen:      retrieval without notes, mini summary, teach-back, micro-quiz, interleaved
Phase 7 — Transfer:        real-world, past-question form, unfamiliar wording, multi-step, compare topics
Phase 8 — Review Later:    spaced recall, weak-point revival, speed-accuracy retest, mastery confirmation
```

## 13.8 Coach Trigger Rules
```
miss 2 similar questions       → trigger contrast drill
direct form right, disguised wrong → trigger transfer repair
repeated hint requests         → lower complexity, reteach
speed collapse, concept correct → switch to pacing support
same misconception reappears after 3 days → schedule reinforced review
```

## 13.9 Help Ladder
```
hint_1 → hint_2 → visual_clue → worked_example →
contrast_explanation → wrong_answer_explanation →
easier_bridge_question → mastery_recheck
```

## 13.10 Mastery State Chain (Extended)
```
unseen → introduced → partial → fragile → improving →
stable → exam-ready → transfer-ready → at-risk-forgetting
```

---

# 14. ACADEMIC RESOURCE INTELLIGENCE (ARIL)
**Source: idea21.txt**

## 14.1 Knowledge Spine Fields Per Concept
- concept_name, plain_explanation, exam_explanation
- formal_definition, child_friendly_explanation
- examples, non_examples, related_concepts, prerequisite_concepts
- formulae_or_rules, worked_examples, common_errors
- question_patterns, answer_logic, marking_points
- difficulty_levels
- representations: text, audio, image, drill, quiz, flashcard, teach_mode, diagnostic_mode

## 14.2 Fifteen Content Atom Types
```rust
pub enum ContentAtomType {
    Definition, Explanation, Formula, Step,
    Example, Counterexample, Comparison, Misconception,
    DiagramLabel, Clue, Hint, MarkingPoint,
    Vocabulary, Objective, Outcome,
}
```

## 14.3 Resource Type Families
- **Knowledge:** definition, explanation, concept_note, formula_card, glossary_entry, comparison_table, cheat_sheet, summary_note
- **Learning:** flashcards, fill-in_drills, recall_prompts, teach_mode_walkthroughs, audio_lessons, story_explanations, real_life_applications
- **Assessment:** MCQs, short_answer, structured, essay, diagram, worked_calculations, error_spotting, matching, sequence_ordering, timed_drills
- **Intervention:** misconception_correction, weak_area_booster, confidence_builder, speed_recovery_drill, accuracy_rehab, formula_repair, term_confusion_drill, recap_mission

## 14.4 Resource Sufficiency Levels
```rust
pub enum SufficiencyLevel {
    Red,   // Insufficient — cannot coach this topic
    Amber, // Usable but limited
    Green, // Ready — full coaching possible
    Blue,  // Mastery-grade rich
}
```

## 14.5 Question Pattern Archetypes (14)
- definition_recall, term_identification, comparison, classification
- process_explanation, diagram_interpretation, application_scenario
- formula_substitution, derivation, error_spotting, statement_correction
- sequence_ordering, cause_and_effect, experiment_interpretation

---

# 15. QUESTION INTELLIGENCE — MULTI-AXIS CLASSIFICATION
**Source: idea22.txt**

## 15.1 Eight Classification Axes (Complete Vocabulary)

### Axis 1 — Knowledge Role (18 values)
```
definition, key_concept, principle, rule, formula_recall,
formula_derivation_proof, worked_example_pattern, procedure_method,
explanation, interpretation, comparison, classification,
cause_and_effect, exception_edge_case, application_scenario,
real_world_use, graph_table_reading, diagram_interpretation
```

### Axis 2 — Cognitive Demand (10 values)
```
recognition, recall, comprehension, application, transfer,
analysis, inference, synthesis, evaluation, justification
```

### Axis 3 — Solve Pattern (15 values)
```
direct_retrieval, step_by_step_computation, formula_substitution,
rearrangement, elimination, pattern_spotting, concept_matching,
diagram_inspection, rule_selection, multi_step_reasoning,
deduction_from_conditions, estimation, unit_conversion,
proof_chain, error_identification
```

### Axis 4 — Pedagogic Function (10 values)
```
teaches_core_idea, reinforces_memory, checks_foundation,
diagnoses_misconception, builds_speed, tests_transfer,
deepens_conceptual_clarity, scaffolds_harder_idea,
bridges_concept_to_method, exam_pattern_familiarization
```

### Axis 5 — Content Grain (10 levels)
```
subject, topic, subtopic, concept, micro_concept,
skill, subskill, formula, rule, vocabulary_term
```

### Axis 6 — Question Family (Named clusters per subject)

### Axis 7 — Misconception Exposure (9 types)
```
confusing_term_with_definition, memorized_formula_without_conditions,
sign_errors, wrong_unit_conversion, mixing_similar_concepts,
applying_wrong_theorem, failing_to_identify_what_asked,
skipping_hidden_condition, using_surface_clues_not_principle
```

### Axis 8 — Evidence & Confidence
```rust
pub struct EvidenceMetadata {
    system_classified: bool,
    human_verified: bool,
    confidence_score: f32,  // 0.0-1.0
    evidence_notes: Option<String>,
    version: u32,
    reviewer_id: Option<UserId>,
    last_reviewed_at: Option<String>,
}
```

## 15.2 Question Intelligence Profile JSON
```json
{
  "knowledge_role": ["definition", "key_concept"],
  "cognitive_demand": ["recall", "comprehension"],
  "solve_pattern": ["direct_retrieval", "concept_matching"],
  "pedagogic_function": ["checks_foundation", "reinforces_memory"],
  "content_grain": {
    "subject": "Integrated Science",
    "topic": "Living Things",
    "subtopic": "Cells",
    "concept": "Cell definition",
    "subskill": "Identify core biological definition"
  },
  "question_family_id": "bio_cells_definition_core_01",
  "misconceptions": ["confuses cell with tissue", "recalls partial definition only"],
  "confidence_score": 0.93,
  "classification_source": "model+rules",
  "human_verified": false
}
```

## 15.3 Auto-Approve Threshold
- Confidence >= 0.88 → auto-approve without human review
- Secondary label: >= 0.60 AND within 0.25 of primary

## 15.4 Feature-Specific Query Profiles
- **Past Questions:** filter by knowledge_role + solve_pattern + pedagogic_function
- **Topic Tests balanced:** 20% definition, 30% key_concept, 30% worked_example, 20% explanation/application
- **Memory Mode:** pull definition, core_fact, key_concept, short_recall
- **Elite Mode:** non-obvious application, multi-step reasoning, exception_handling, transfer
- **Teach Mode sequence:** Definition → Key Concept → Explanation → Worked Example → Guided Practice → Mixed Application → Trap Question → Exam Question

---

# 16. ACADEMIC DECAY & RECALL RESILIENCE ENGINE
**Source: idea32.txt**

## 16.1 Memory Strength 6-Level Model
```rust
pub enum MemoryStrengthLevel {
    Absent,      // Level 0
    Familiar,    // Level 1
    Recognizable,// Level 2
    Retrievable, // Level 3
    Usable,      // Level 4
    Durable,     // Level 5
}
```

## 16.2 Seven Technical Decay Types
```rust
pub enum DecayType {
    TimeDecay,        // Trace fades with time
    StabilityDecay,   // Encoding was always weak
    SpeedDecay,       // Recall becomes slow
    TransferDecay,    // Context-specific knowledge
    PressureDecay,    // Breaks under timed conditions
    StructureDecay,   // Knowledge fragments
    InterferenceDecay,// Similar concepts contaminate
}
```

## 16.3 Six Core Signal Categories
```
Accuracy Signals: correct/incorrect/partial/technically-correct-flawed-method
Time Signals: latency_before_answer, total_answer_time, time_to_first_commit, speed_trend
Confidence Signals: self_rated, confidence_vs_correctness_mismatch, answer_switching
Support Signals: needed_hint, needed_options, needed_formula_bank, needed_cue, needed_elimination
Stability Signals: remembered_after_1d/3d/7d/14d, mixed_review, variant_wording, timed_conditions
Transfer Signals: same_idea_different_wording/context/arrangement/phrasing/real_world
Interference Signals: confused_with_sibling, mixed_terms/steps/formulas, imported_wrong_rule
Behavioral Signals: abandoned, froze, rushed, returned_and_solved, inconsistent, guessed_quickly
```

## 16.4 Knowledge Unit Data Fields (Per Concept)
```sql
-- Full field list for student_concept_mastery table extension
knowledge_unit_id TEXT NOT NULL,
last_seen_at TEXT,
last_successful_free_recall_at TEXT,
last_successful_applied_recall_at TEXT,
last_pressure_success_at TEXT,
recall_success_rate INTEGER,         -- basis points
recognition_success_rate INTEGER,    -- basis points
free_recall_success_rate INTEGER,    -- basis points
transfer_success_rate INTEGER,       -- basis points
pressure_success_rate INTEGER,       -- basis points
avg_recall_latency_ms INTEGER,
confidence_trend TEXT,               -- 'improving'|'stable'|'declining'
hint_dependency_score INTEGER,       -- basis points
decay_risk_score INTEGER,            -- basis points
memory_state TEXT,                   -- enum value
likely_failure_mode TEXT,            -- enum value
recommended_intervention TEXT,       -- enum value
next_review_at TEXT,
interference_neighbors TEXT          -- JSON array of concept_ids
```

## 16.5 Decay Severity Color Model
```rust
pub enum DecaySeverity {
    Green,   // Stable
    Yellow,  // Watchlist
    Orange,  // Fragile
    Red,     // Decaying
    Black,   // Collapsed
}
```

## 16.6 Five Anti-Forgetfulness Forces
```rust
pub enum AntiForgetfulnessForce {
    Preserve,   // Keep memory alive before collapse
    Detect,     // Catch fragile knowledge early
    Recover,    // Bring weak knowledge back
    Stabilize,  // Convert recovered into durable
    Defend,     // Protect against future collapse
}
```

## 16.7 Intervention Cases A-G (Full)
```
Case A: Recognition present, free recall absent
→ hidden-answer recall, fill-the-gap, verbal retrieval, cue fading, short repeated free recall

Case B: Free recall present, applied recall weak
→ worked-example bridge, classify-then-solve, identify-which-rule, context variation

Case C: Untimed success, timed collapse
→ compressed recall drills, time-boxed bursts, escalating pace, small timed sets

Case D: Similar-topic confusion
→ contrast tables, "which is which" drills, paired examples, error explanation, deliberate discrimination

Case E: Learned before, now fading
→ no-option recall, minimal cue trigger, short recovery cycle, spaced rechecks

Case F: Fragmented knowledge
→ concept map reconstruction, sequence rebuilding, step ordering, explain-from-start-to-finish drills

Case G: Confidence collapse
→ confidence-calibration drills, fast-first-response practice, second-guess tracking, low-stakes speed rounds
```

## 16.8 Priority Formula (Decay Engine)
```
Priority = importance × exam_relevance × decay_severity × recurrence_risk × dependency_weight
```

---

# 17. TRAPS FEATURE — DIFFERENCE DRILL
**Source: idea33.txt**

## 17.1 Five Traps Game Modes
```rust
pub enum TrapsMode {
    DifferenceDrill,  // Which concept does this feature belong to?
    SimilarityTrap,   // Which statements are true for both/one/neither?
    KnowTheDifference,// Open-ended difference prompts
    WhichIsWhich,     // Fast timed card sorting
    Unmask,           // Identify the hidden misconception
}
```

## 17.2 Five Timer Modes
```rust
pub enum TrapsTimerMode {
    Calm,        // No timer or very generous
    Standard,    // Per-card ~5-8 seconds
    Pressure,    // Per-card ~2-4 seconds (timeout auto-skips)
    SuddenDrop,  // Cards appear at irregular speed
    Heat,        // Card drop speed increases as learner does well
}
```

## 17.3 End-of-Round Summary Structure
- Accuracy: correct/wrong/skipped counts
- Confusion breakdown by type: mixed_examples, mixed_functions, timed_out_under_pressure, specific_feature_confusion
- Concept health: definition/feature/example/speed each rated (Strong/Fragile/Weak/Unstable)

## 17.4 Error Classification Types (Traps)
```rust
pub enum TrapsErrorType {
    NamingConfusion,
    FeatureConfusion,
    SpeedError,
    ShallowRecall,
    IncompleteUnderstanding,
    DistractorTrap,
    ConceptOverlapConfusion,
}
```

## 17.5 Card Item Types
- Feature prompts (Requires a selectively permeable membrane)
- Function prompts (Helps plant root hair cells take in water)
- Example prompts (Perfume spreading through air)
- Condition prompts (Happens only with water molecules)
- Misconception prompts (incorrect but plausible claim)
- Scenario prompts (A raisin swells after being placed in water)
- Contrast prompts (Can happen in gases)

## 17.6 Contrast Profile Schema
```sql
CREATE TABLE contrast_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    concept_a_id INTEGER NOT NULL REFERENCES knowledge_atoms(id),
    concept_b_id INTEGER NOT NULL REFERENCES knowledge_atoms(id),
    subject_id INTEGER NOT NULL,
    topic_id INTEGER NOT NULL,
    shared_attributes TEXT NOT NULL DEFAULT '[]',    -- JSON
    distinct_a_attributes TEXT NOT NULL DEFAULT '[]',-- JSON
    distinct_b_attributes TEXT NOT NULL DEFAULT '[]',-- JSON
    common_confusions TEXT NOT NULL DEFAULT '[]',    -- JSON
    difficulty_tier INTEGER NOT NULL DEFAULT 1,      -- 1-5
    validated INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

# 18. DIAGNOSTIC TEST (ACADEMIC DNA)
**Source: idea34.txt**

## 18.1 Seven Test Stages
```rust
pub enum DiagnosticStage {
    FastBaselineScan,    // 20-30 items; broad coverage; identify strong/weak/uncertain
    TopicZoom,           // Automated zoom into weak/unstable areas
    MisconceptionProbing,// Expose why student is wrong; distractor-to-misconception mapping
    SpeedPressureLayer,  // Untimed vs timed conditions
    TransferLayer,       // Same concept in 5 forms: direct/word-problem/diagram/comparison/explain-why
    ConfidenceCapture,   // 3-level confidence: sure/not_sure/guessed
    MicroRecheck,        // Reintroduce indirectly; test retention vs fluency vs guessing
}
```

## 18.2 Twelve Test Dimensions
```
A. Coverage          - Which topics/subtopics touched
B. Accuracy          - Can get correct answers normally
C. Recall Strength   - Pull from memory without hints
D. Recognition vs Production - MCQ vs open response
E. Reasoning Depth   - Guess/memorize/understand
F. Misconception Pattern - Exact wrong belief
G. Speed             - Know it but too slowly
H. Pressure Response - Break down when timed
I. Transfer Ability  - Solve when reworded/applied
J. Stability         - Consistent or fragile
K. Confidence Calibration - Wrongly confident
L. Fatigue Pattern   - Deteriorate over time
```

## 18.3 Adaptive Test Durations
| Mode | Duration |
|------|----------|
| Light | ~35 minutes |
| Standard | ~60 minutes |
| Deep | ~90 minutes |

Item counts: 35-54 total (18-24 baseline + 12-20 adaptive + 6-10 condition + 4-8 stability)

## 18.4 Guessing Detection Signals (Require Multiple)
- correct_answer + low_confidence
- correct_answer + very_long_hesitation
- correct_answer + answer_changed_multiple_times
- correct_answer + later_failure_on_sibling_question
- MCQ_correct + failure_on_short_answer_form
- hard_item_correct + poor_surrounding_performance_on_prerequisites
- random_fast_answer_on_difficult_question
- inconsistent_performance_on_same_concept

## 18.5 Report Section B — Topic Report Example
```
Topic: Fractions
Mastery: Weak
Accuracy: 42%
Speed: Slow
Confidence: High on wrong answers (false confidence)
Pressure: Drops sharply
Misconceptions detected:
  - adds denominators directly
  - struggles to compare unlike fractions
  - weak translation from word form to operation
Recommended focus:
  - denominator meaning
  - visual fraction models
  - common denominator reasoning
  - timed comparison drills after concept repair
```

## 18.6 Per-Question Data Logged
```sql
-- diagnostic_question_log additions
start_time TEXT NOT NULL,
submit_time TEXT NOT NULL,
total_time_ms INTEGER NOT NULL,
first_selected_answer TEXT,
final_selected_answer TEXT,
answer_changed INTEGER NOT NULL DEFAULT 0,
change_count INTEGER NOT NULL DEFAULT 0,
skipped INTEGER NOT NULL DEFAULT 0,
hint_used INTEGER NOT NULL DEFAULT 0,
confidence_level TEXT,           -- 'sure'|'not_sure'|'guessed'
is_correct INTEGER NOT NULL,
misconception_tag TEXT,          -- what the wrong option represents
has_sibling_question INTEGER NOT NULL DEFAULT 0,
sibling_question_id INTEGER,
guessing_likelihood_score INTEGER -- basis points
```

---

# 19. SMART CENTRAL CURRICULUM
**Source: idea31.txt**

## 19.1 Sixteen Curriculum Entity Types
```rust
pub struct CurriculumEntityList {
    // In migration 002_curriculum.sql
    entities: [
        "curriculum_versions",
        "subjects",
        "strands",
        "sub_strands",
        "content_standards",
        "indicators",
        "exemplars",
        "curriculum_annotations",
        "curriculum_comments",
        "curriculum_node_relations",
        "public_translations",
        "content_blueprints",
        "assessment_blueprints",
        "node_coverage_ledger",
        "version_diffs",
        "visibility_policies",
    ]
}
```

## 19.2 Node Coverage States (9)
```rust
pub enum NodeCoverageState {
    NotIntroduced,
    Introduced,
    Taught,
    Practiced,
    Assessed,
    Mastered,
    Unstable,
    Decayed,
    ReOpened,
}
```

## 19.3 Curriculum Change Classification
```rust
pub enum CurriculumChangeType {
    TypeA, // Cosmetic — wording/typo, no scope change
    TypeB, // Interpretive — misconceptions, examples, question mapping
    TypeC, // Scope — new topic, expanded objective, removed subtopic
    TypeD, // Structural — topic moved year, indicator split, nodes merged, prerequisite chain changed
}
```

## 19.4 Node ID Format
```
Format: B7/JHS1.1.1.2.1
        ^^  ^^^^         = Class (B7 = Basic 7 = JHS 1)
            ^^^^         = Strand.SubStrand.ContentStd.Indicator
```

## 19.5 Curriculum Node Interpretation Fields (Hidden)
- friendly_topic_name, internal_subtopic_atoms
- knowledge_points, skills_involved
- cognitive_verb (examine/explain/identify/discuss)
- expected_evidence_type
- common_misconceptions, prerequisite_nodes, next_dependent_nodes
- difficulty_ladder, recommended_teaching_strategies
- question_families, worked_example_templates
- memory_recall_tags, local_context_examples
- bece_past_question_mapping

## 19.6 Content Asset Metadata (Required Per Asset)
- curriculum_version, subject, strand, sub_strand
- node_id, objective_type, difficulty, content_type, approval_state

## 19.7 PDF Extraction Pipeline (7 Stages)
1. Upload layer
2. Extraction layer (digital/scanned/mixed PDF detection)
3. Structure detection layer
4. Curriculum parser
5. Brain/intelligence enrichment layer
6. Human review layer
7. Publish layer

---

# 20. COACHHUB — GOALS & DOCUMENT UPLOAD
**Source: idea36.txt**

## 20.1 Goal Types (7 Categories)
```rust
pub enum GoalCategory {
    Outcome,         // Pass BECE, get Grade 1 in Science
    Preparation,     // Prepare for class test, revise Term 2
    WeaknessRepair,  // Fix fractions, improve equation solving
    Behavior,        // Study 5 days/week, reduce skipped sessions
    SpeedAccuracy,   // Increase speed without accuracy collapse
    Resource,        // Finish uploaded teacher booklet, past questions 2020-2025
    ParentTeacher,   // Follow teacher correction areas, improve report card topics
}
```

## 20.2 Goal States (8)
```rust
pub enum GoalState {
    Drafted,
    Confirmed,
    Active,
    Paused,
    Blocked,
    Completed,
    AtRisk,
    Recalibrating,
}
```

## 20.3 Goal Hierarchy (4 Levels)
```rust
pub enum GoalLevel {
    NorthStar,       // Main big goal (e.g., Pass BECE 2026)
    CurrentCampaign, // Matters in present phase
    ActiveTactical,  // Currently being worked on
    Background,      // Not current focus but tracked
}
```

## 20.4 Goal Table Schema
```sql
CREATE TABLE goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id),
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    level TEXT NOT NULL,
    subject_id INTEGER REFERENCES subjects(id),
    topics_json TEXT DEFAULT '[]',
    urgency_level TEXT DEFAULT 'normal',
    start_date TEXT,
    deadline TEXT,
    exam_id INTEGER,
    confidence_score INTEGER DEFAULT 5000,    -- basis points
    coach_priority_score INTEGER DEFAULT 5000,
    parent_priority_flag INTEGER DEFAULT 0,
    evidence_sources TEXT DEFAULT '[]',       -- JSON
    dependency_goal_ids TEXT DEFAULT '[]',    -- JSON
    risk_level TEXT DEFAULT 'low',
    suggested_weekly_effort_minutes INTEGER,
    current_momentum INTEGER DEFAULT 5000,    -- basis points
    completion_criteria TEXT,
    state TEXT NOT NULL DEFAULT 'drafted',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## 20.5 Document Upload Portal — Accepted Types
- homework, class_notes, class_tests, assignments
- teacher_handouts, revision_sheets, report_cards
- exam_papers, textbook_snapshots, worksheets

## 20.6 Document Intelligence Extracts
- What this student is currently being taught
- What type of schoolwork is coming in
- Patterns in student materials
- Teacher's focus areas
- What gaps exist in current learning
- What can be improved from existing materials

---

# 21. INTELLIGENCE CONSTITUTION — ENGINE REGISTRY
**Source: idea25.txt**

## 21.1 Current System Baseline (Audit Context)
- Total audited code: **12,068 lines**
- Topic Intelligence engine alone: **3,996 lines**
- Current engines: **6** (Topic, Coverage, Sequencing, Timing, Risk, Adaptation)
- Named algorithms: **53**

## 21.2 Eight Permanent Intelligence Domains
1. Evidence Domain (6 engines)
2. Knowledge Domain (7 engines)
3. Learner Domain (7 engines)
4. Diagnostic Domain (9 engines)
5. Decision Domain (9 engines)
6. Execution Design Domain (7 engines)
7. Memory and Meta-Learning Domain (8 engines)
8. Governance Domain (8 engines)
**Total: ~61 engines**

## 21.3 Engine Registry — P0 (Must Build First)
```
A1: Response Evidence Ingestion
A2: Content Signal Ingestion
A6: Evidence Normalization
B2: Concept State
B3: Topic State
B4: Hypothesis Competition
B5: Misconception
B6: Interference
B7: Learner State
B8: Mastery Proof
C1: Teaching Strategy
C2: Sequencing
C3: Timing
C4: Risk
C5: Adaptation
C6: Session Composer
F5: Audit and Trace
```

## 21.4 Engine Registry — P1 (Second Wave)
```
A3: Curriculum Ingestion
A4: Learner Signal Ingestion
B1: Topic Scope
B9: Coverage Gap
B10: Knowledge Graph
C7: Content Selection
C8: Diagnostic Experiment
C9: Protection Rule
D1: Intervention Design
D2: Drill Generation
D4: Assessment Construction
E1: Topic Memory
E2: Learner Memory
E3: Strategy Memory
E4: Coach Self-Evaluation
E5: Improvement Velocity
F1: Decision Arbitration
F2: Confidence Gate
F3: Contradiction Check
F4: Policy Guardrail
```

## 21.5 Eleven-Step Runtime Loop
```rust
pub enum RuntimeStep {
    Observe,          // 1
    Normalize,        // 2
    Infer,            // 3
    ResolveAmbiguity, // 4
    Decide,           // 5
    Compose,          // 6
    Execute,          // 7
    Verify,           // 8
    Remember,         // 9
    MetaEvaluate,     // 10
    Govern,           // 11
}
```

## 21.6 Eight Day-1 Non-Negotiable Invariants
1. No single-score diagnosis
2. No mastery without proof
3. No strong action on weak evidence
4. No topic reasoning without learner reasoning
5. No adaptation without memory
6. No timing pressure before readiness
7. No content selection without content fitness
8. No plan without traceability

## 21.7 Database Namespace Prefixes (From idea25)
```
j_   — Journey/coach brain tables
c_   — Curriculum/content tables
ic_  — Intelligence-curriculum junction
ie_  — Intelligence-evidence tables
im_  — Intelligence-memory tables
ig_  — Intelligence-goal tables
ix_  — Intelligence index tables
```

---

# 22. CURRICULUM INTELLIGENCE PORTAL
**Source: idea26.txt**

## 22.1 Source Trust Tiers
```rust
pub enum SourceTrustTier {
    TierA, // Official curriculum bodies, exam councils, approved publishers
    TierB, // Teacher-authored, school-approved notes, high-quality textbooks
    TierC, // Well-structured external explainers, reputable tutorials
    TierD, // General web, low-authority blogs, weakly structured pages
}
```

## 22.2 Four Content Zones
```rust
pub enum ContentZone {
    Raw,       // Exactly what came in
    Parsed,    // Cleaned and chunked, not trusted yet
    Verified,  // Mapped, corroborated, scored, ready
    Published, // What the user-facing app can retrieve
}
```

## 22.3 Full Canonical Data Entities (19 tables)
```sql
-- All entities that belong in the curriculum intelligence portal
curriculum_sources, curriculum_versions, curriculum_tracks,
curriculum_levels, curriculum_terms, curriculum_units,
curriculum_topics, curriculum_subtopics, learning_objectives,
competencies, knowledge_atoms, formulas, glossary_terms,
curriculum_relationships, curriculum_aliases,
curriculum_resource_links, curriculum_assessment_patterns,
curriculum_publish_states, curriculum_parse_jobs, curriculum_review_flags
```

## 22.4 Inferred Intelligence Per Topic (Auto-Computed)
- prerequisite_topics, adjacent_topics, next_best_topic
- dependency_risk, topic_weight_in_exam_prep
- typical_cognitive_demand, question_family_types
- related_glossary_bundle, likely_misconceptions
- likely_remedial_strategy

## 22.5 Document Source Metadata
- country, exam_board, curriculum_family, education_level
- class_grade_form, subject, academic_year, term_semester
- source_type, official_vs_internal, language
- status (draft/parsed/reviewed/published/archived)
- version_number, effective_start_date, replacement_relationship

---

# 23. TIME ORCHESTRATION ENGINE
**Source: idea30.txt**

## 23.1 Availability Profile Components
```
A. Weekly Recurring: earliest start, latest end, total hours, preferred bands, hard blocks
B. Day-Type Differences: weekday/Saturday/Sunday/holiday/exam_season/school_vs_vacation
C. Session Tolerance:
   - ideal_session_length_minutes
   - max_tolerable_session_length_minutes
   - split_sessions_allowed: bool
   - min_break_after_90min_minutes
D. Trigger Preference: auto | manual | hybrid
E. Exception Calendar: unavailable dates, events, travel, mock days, sickness, emergency blackout
```

## 23.2 Learning Credit System
```
60 min focused concept building       = 0.9 learning credit
60 min shallow reading                = 0.4 learning credit
45 min strong retrieval under pressure = 0.8 learning credit
30 min confused struggle              = 0.2 learning credit (unless followed by correction)
```

## 23.3 Twelve Session Type Tags
```rust
pub enum SessionTypeTag {
    Acquire,          // Learn new material
    Rebuild,          // Reteach weak concept
    Reinforce,        // Strengthen partially learned
    Retrieve,         // Recall from memory with minimal cues
    Separate,         // Untangle confusing similar concepts
    PressureDrill,    // Timed speed + accuracy
    ResilienceProbe,  // Test after a gap
    MockSegment,      // Exam-like chunk
    CorrectionClinic, // Analyze mistakes deeply
    LightTouchReview, // Short maintenance session
    Catchup,          // Absorb missed work
    OpportunisticSession, // Student is free now
}
```

## 23.4 Three-Layer Schedule Structure
```
Layer 1 — Macro Plan: total hours, weeks remaining, weekly pace, topics priority, major review/mock points
Layer 2 — Rolling Active Plan: next 7-10 days tight, beyond that semi-flexible
Layer 3 — Daily Session Selection: fatigue, recent performance, time available, decay pressure, urgency
```

## 23.5 100-Hour Allocation Example
```
Concept acquisition:     30 hours
Worked practice:         20 hours
Guided correction:       12 hours
Retrieval strengthening: 14 hours
Timed pressure:           8 hours
Interleaving:             6 hours
Mock exams:               5 hours
Recovery/buffer:          5 hours
```

## 23.6 Effective Credit Formula
```
Realistic Capacity = raw × attendance × fatigue × time_of_day × friction
```

---

# 24. COACH BRAIN — TEACHING & INTERVENTION MODEL
**Source: idea23.txt, idea24.txt**

## 24.1 Coach Brain Six Layers
```rust
pub enum CoachBrainLayer {
    Perception,        // Signal ingestion (25+ signal types)
    StudentModel,      // Continuously updated dimensions
    DomainIntelligence,// Academic knowledge graph
    ReasoningDiagnosis,// Interprets evidence
    StrategyEngine,    // Decides next actions
    ExperienceComposer,// Assembles dynamic UX
}
```

## 24.2 Student Model Dimensions (Full)
```rust
pub struct StudentModelDimensions {
    // Academic state
    subject_mastery_by_topic: TopicMasteryMap,
    concept_dependency_gaps: Vec<ConceptGap>,
    formula_recall_strength: BasisPoints,
    definition_strength: BasisPoints,
    application_strength: BasisPoints,
    explanation_strength: BasisPoints,
    problem_solving_strength: BasisPoints,

    // Performance traits
    speed: BasisPoints,
    accuracy: BasisPoints,
    endurance: BasisPoints,
    pressure_tolerance: BasisPoints,
    recovery_after_failure: BasisPoints,
    consistency: BasisPoints,
    guess_tendency: BasisPoints,

    // Learning style tendencies
    prefers_worked_examples_first: bool,
    prefers_direct_practice_first: bool,
    needs_visual_explanation: bool,
    does_better_short_bursts: bool,
    responds_to_challenge_mode: bool,
    shuts_down_after_failure: bool,

    // Risk state
    at_risk_of_forgetting: bool,
    at_risk_of_burnout: bool,
    at_risk_of_exam_panic: bool,
    at_risk_of_overconfidence: bool,

    // Motivation state
    motivation_state: MotivationState,
}

pub enum MotivationState {
    Engaged, Neutral, Discouraged,
    StreakDriven, TitleDriven, RewardResponsive,
}
```

## 24.3 Eighteen Possible Causes of Weak Score
```rust
pub enum WeakScoreCause {
    ConceptNeverUnderstood,
    TermConfusion,
    FormulaRetrievalFailure,
    CarelessError,
    SlowProcessing,
    PanicUnderTime,
    WeakWorkingMemory,
    MisreadingQuestion,
    FragileProceduralChain,
    ConfusionBetweenSimilarRules,
    KnowledgeDecay,
    FalseConfidence,
    InabilityToTransfer,
    LanguageComprehensionIssue,
    PoorStamina,
    RandomGuessingStrategy,
    LearnedDependencyOnHints,
    TopicInterference,
}
```

## 24.4 Hypothesis-Based Diagnostic Flow (6 Steps)
1. Observe failure pattern
2. Generate likely causes (hypotheses)
3. Run targeted probes
4. Eliminate wrong hypotheses
5. Identify root cause or cluster
6. Choose intervention based on cause

## 24.5 Eight Human Roles System Must Replace
1. Teacher — explain multiple ways (simple/formal/example/analogy/diagram)
2. Tutor — diagnose individually, correct immediately, scaffold appropriately
3. Test Maker — generate questions by type, difficulty, topic
4. Psychologist — manage confidence, pressure, motivation
5. Strategist — plan weeks ahead, adapt daily, know what to prioritize
6. Monitor — track progress, detect faking, prove progress
7. Pressure Coach — train time management, speed, panic recovery
8. Motivator — prevent burnout, celebrate growth appropriately

## 24.6 "Definition of Done" Per Level
```
SESSION done only when:
  - target subskill actually tested
  - intervention delivered
  - evidence changed
  - next action determined

TOPIC done only when:
  - core concept understood
  - recall verified
  - application verified
  - confusion with neighbors checked
  - timed performance checked
  - delayed retention checked

EXAM READINESS done only when:
  - coverage threshold met
  - weakness density low enough
  - timed resilience acceptable
  - recent retention stable
  - priority topics secured
```

## 24.7 Six Non-Negotiable Cores
1. Truth — can correctly figure out what's wrong
2. Strategy — can choose right next move
3. Memory — can remember student properly over time
4. Proof — can prove improvement and exam readiness
5. Orchestration — can unify all features into one mind
6. Resilience — can work offline, locally, reliably

---

# 25. MASTER FORMULAS NOT PREVIOUSLY CAPTURED

## From idea4.txt
```
MomentumScore = 0.35 × volume + 0.40 × accuracy + 0.25 × pace
StrainScore   = 0.30 × accuracy_drop + 0.20 × completion_drop
              + 0.20 × hint_spike + 0.15 × skip + 0.15 × pace_instability
```

## From idea1.txt
```
ForecastScore(u) = 0.25*Frequency + 0.20*Recency + 0.15*Trend
                 + 0.15*BundleStrength + 0.10*SyllabusPriority
                 + 0.10*StyleRegimeFit + 0.05*ExaminerGoalFit

MockScore     = 0.25*weakness + 0.20*coverage_gap + 0.20*misconception_pressure
              + 0.15*spaced_due + 0.10*exam_weight + 0.10*info_value
              + 0.05*variety_bonus − 0.25*anti_repeat_penalty

Weakness(t)   = 0.35*(1−Mastery) + 0.20*LinkBreakage + 0.15*MisconceptionPressure
              + 0.10*RepresentationGap + 0.10*TimedGap
              + 0.05*GuessPenalty + 0.05*RecencyDecay

Readiness     = 0.45*Mastery + 0.20*TimedPerformance + 0.15*Coverage
              + 0.10*Consistency + 0.10*Trend

PredictedScore = ∑ BlueprintWeight_k × Mastery_k × TimingFactor_k
                 × RetentionFactor_k × MisconceptionImmunity_k
```

## From idea2.txt
```
Readiness(Journey) = 0.25*topic_mastery + 0.20*retention + 0.20*mock_performance
                   + 0.15*speed + 0.10*syllabus_coverage + 0.10*consistency
                   − penalties
```

## From idea7.txt
```
MSI = 0.30*accuracy + 0.15*speed + 0.20*retention_over_time
    + 0.15*variant_transfer + 0.10*independence + 0.10*connection
    (Alternative 5-component: 0.35*A + 0.20*S + 0.20*R + 0.15*V + 0.10*C)
```

## From idea9.txt
```
Q_difficulty = B_c + S_l + A_b + D_s + R_s + T_p + C_u
```

## From idea30.txt
```
RealisticCapacity = raw × attendance × fatigue × time_of_day × friction
RetentionSchedulingPriority = importance × fragility × decay_risk × recall_urgency
```

## From idea32.txt
```
DecayPriority = importance × exam_relevance × decay_severity × recurrence_risk × dependency_weight
```

---

# APPENDIX: MISSING SQL TABLES (Supplement to Migration Files)

## A. Supplement to migration 001_identity.sql

```sql
CREATE TABLE parent_alerts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    alert_type TEXT NOT NULL
        CHECK (alert_type IN ('inactivity','decline','exam_near','mock_overdue',
                              'subject_lagging','misconception_repeat','false_confidence')),
    message TEXT NOT NULL,
    severity TEXT NOT NULL DEFAULT 'watch'
        CHECK (severity IN ('info','watch','urgent','critical')),
    is_read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE report_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    report_type TEXT NOT NULL
        CHECK (report_type IN ('weekly','monthly','term','pre_exam','milestone')),
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    data_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## B. Supplement to migration 004_student_state.sql

```sql
-- Extended memory tracking (supplement to student_topic_mastery)
ALTER TABLE student_topic_mastery ADD COLUMN last_successful_free_recall_at TEXT;
ALTER TABLE student_topic_mastery ADD COLUMN last_successful_applied_recall_at TEXT;
ALTER TABLE student_topic_mastery ADD COLUMN last_pressure_success_at TEXT;
ALTER TABLE student_topic_mastery ADD COLUMN free_recall_success_rate INTEGER DEFAULT 0;
ALTER TABLE student_topic_mastery ADD COLUMN transfer_success_rate INTEGER DEFAULT 0;
ALTER TABLE student_topic_mastery ADD COLUMN pressure_success_rate INTEGER DEFAULT 0;
ALTER TABLE student_topic_mastery ADD COLUMN avg_recall_latency_ms INTEGER;
ALTER TABLE student_topic_mastery ADD COLUMN hint_dependency_score INTEGER DEFAULT 0;
ALTER TABLE student_topic_mastery ADD COLUMN decay_risk_score INTEGER DEFAULT 0;
ALTER TABLE student_topic_mastery ADD COLUMN likely_failure_mode TEXT;
ALTER TABLE student_topic_mastery ADD COLUMN interference_neighbors TEXT DEFAULT '[]';

-- Knowledge gap tracking
CREATE TABLE knowledge_gaps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    skill_id INTEGER,
    gap_type TEXT NOT NULL,    -- uses KnowledgeGapType enum values
    gap_severity INTEGER NOT NULL DEFAULT 5000,  -- basis points
    gap_rank INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    first_detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_confirmed_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT,
    resolution_method TEXT
);

-- Rise Mode student scores
CREATE TABLE rise_student_scores (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    foundation_score INTEGER NOT NULL DEFAULT 0,
    recall_score INTEGER NOT NULL DEFAULT 0,
    speed_score INTEGER NOT NULL DEFAULT 0,
    accuracy_score INTEGER NOT NULL DEFAULT 0,
    pressure_stability_score INTEGER NOT NULL DEFAULT 0,
    misconception_density INTEGER NOT NULL DEFAULT 0,
    momentum_score INTEGER NOT NULL DEFAULT 0,
    transformation_readiness INTEGER NOT NULL DEFAULT 0,
    current_stage TEXT NOT NULL DEFAULT 'rescue'
        CHECK (current_stage IN ('rescue','stabilize','accelerate','dominate')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(student_id)
);
```

## C. Supplement to migration 005_sessions.sql

```sql
-- Diagnostic question log extensions
CREATE TABLE diagnostic_question_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    diagnostic_session_id INTEGER NOT NULL,
    question_id INTEGER NOT NULL,
    stage TEXT NOT NULL,           -- DiagnosticStage enum
    start_time TEXT NOT NULL,
    submit_time TEXT NOT NULL,
    total_time_ms INTEGER NOT NULL,
    first_selected_answer TEXT,
    final_selected_answer TEXT,
    answer_changed INTEGER NOT NULL DEFAULT 0,
    change_count INTEGER NOT NULL DEFAULT 0,
    skipped INTEGER NOT NULL DEFAULT 0,
    hint_used INTEGER NOT NULL DEFAULT 0,
    confidence_level TEXT CHECK (confidence_level IN ('sure','not_sure','guessed')),
    is_correct INTEGER NOT NULL,
    misconception_tag TEXT,
    has_sibling INTEGER NOT NULL DEFAULT 0,
    sibling_question_id INTEGER,
    guessing_likelihood_score INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## D. Supplement to migration 010_goals_calendar.sql

```sql
CREATE TABLE goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    category TEXT NOT NULL
        CHECK (category IN ('outcome','preparation','weakness_repair','behavior',
                            'speed_accuracy','resource','parent_teacher')),
    level TEXT NOT NULL DEFAULT 'active_tactical'
        CHECK (level IN ('north_star','current_campaign','active_tactical','background')),
    subject_id INTEGER REFERENCES subjects(id),
    topics_json TEXT NOT NULL DEFAULT '[]',
    urgency_level TEXT NOT NULL DEFAULT 'normal'
        CHECK (urgency_level IN ('low','normal','high','critical')),
    start_date TEXT,
    deadline TEXT,
    exam_id INTEGER,
    confidence_score INTEGER NOT NULL DEFAULT 5000,
    coach_priority_score INTEGER NOT NULL DEFAULT 5000,
    parent_priority_flag INTEGER NOT NULL DEFAULT 0,
    evidence_sources TEXT NOT NULL DEFAULT '[]',
    dependency_goal_ids TEXT NOT NULL DEFAULT '[]',
    risk_level TEXT NOT NULL DEFAULT 'low',
    suggested_weekly_effort_minutes INTEGER,
    current_momentum INTEGER NOT NULL DEFAULT 5000,
    completion_criteria TEXT,
    state TEXT NOT NULL DEFAULT 'drafted'
        CHECK (state IN ('drafted','confirmed','active','paused','blocked',
                         'completed','at_risk','recalibrating')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## E. Supplement to migration 015_games.sql

```sql
CREATE TABLE contrast_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    concept_a_id INTEGER NOT NULL,
    concept_b_id INTEGER NOT NULL,
    subject_id INTEGER NOT NULL,
    topic_id INTEGER NOT NULL,
    shared_attributes TEXT NOT NULL DEFAULT '[]',
    distinct_a_attributes TEXT NOT NULL DEFAULT '[]',
    distinct_b_attributes TEXT NOT NULL DEFAULT '[]',
    common_confusions TEXT NOT NULL DEFAULT '[]',
    difficulty_tier INTEGER NOT NULL DEFAULT 1
        CHECK (difficulty_tier BETWEEN 1 AND 5),
    validated INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE traps_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mode TEXT NOT NULL
        CHECK (mode IN ('difference_drill','similarity_trap','know_the_difference',
                        'which_is_which','unmask')),
    timer_mode TEXT NOT NULL DEFAULT 'standard'
        CHECK (timer_mode IN ('calm','standard','pressure','sudden_drop','heat')),
    contrast_profile_id INTEGER NOT NULL REFERENCES contrast_profiles(id),
    total_items INTEGER NOT NULL DEFAULT 0,
    correct_count INTEGER NOT NULL DEFAULT 0,
    wrong_count INTEGER NOT NULL DEFAULT 0,
    skipped_count INTEGER NOT NULL DEFAULT 0,
    streak_best INTEGER NOT NULL DEFAULT 0,
    session_score INTEGER NOT NULL DEFAULT 0,    -- basis points
    accuracy_score INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT,
    concept_health_json TEXT DEFAULT '{}'
);
```

---

---

# 26. PAST EXAM INTELLIGENCE SYSTEM
**Source: idea13.txt**

## 26.1 Seven View Types
```rust
pub enum PastPaperViewType {
    YearView,         // Full paper practice
    TopicView,        // Targeted revision
    PatternView,      // Recurring patterns, wording shifts, examiner habits
    WeaknessView,     // Student's personal weaknesses clustered by type
    ExamReplayView,   // Reconstructed exam with original pacing/pressure
    LikelyNextStyle,  // Probability-based concept prediction
    CrossYearBlended, // Cross-year papers combining patterns
}
```

## 26.2 Question Family Classification (Past Papers)
- concept_family
- reasoning_family
- examiner_trick_family
- misconception_family
- answer_path_family

## 26.3 Recovery Mode Structure
```
2 easier bridge questions → 2 equivalent sibling questions → re-ask similar → delayed review
```

## 26.4 Paper DNA Profile Components
- concept_spread
- difficulty_curve
- trap_density
- speed_demand
- explanation_demand
- memory_vs_reasoning_balance

## 26.5 Four Pillars
```rust
pub enum PastPaperPillar {
    Archive,          // Year/topic access
    Intelligence,     // Pattern mining, question families, evolution
    Personalization,  // Weakness view, recovery mode, readiness
    Simulation,       // Real paper mode, exam replay, timed drills, marking scheme training
}
```

## 26.6 Question Evolution Tracking
Track how question types mutate year-over-year, e.g.:
- Percentages: 2016 direct → 2018 shopping context → 2020 reverse → 2023 multi-step

---

# 27. CUSTOMISED TESTING ENGINE
**Source: idea14.txt**

## 27.1 Test Type Selection
```rust
pub enum TestType {
    ClassTest, Quiz, Midterm, MockExam, EndOfTermExam,
    RevisionDrill, TeachersTest, CustomTestMix,
}
```

## 27.2 Topic Scope Options
```rust
pub enum TopicScope {
    OneTopic,
    MultipleTopics,
    EntireUnit,
    RecentTopicsOnly,
    WeakTopicsOnly,
    TeacherSelectedBundle,
}
```

## 27.3 Preparation Mode Options
```rust
pub enum PrepMode {
    TestLikeTheRealThing,
    TeachMeThroughQuestions,
    FixMyWeakAreas,
    WarmUpThenTest,
    GiveMeLikelyQuestions,
    PutMeUnderPressure,
    ChallengeAboveExpected,
}
```

## 27.4 Six Sub-Modes
```rust
pub enum TestSubMode {
    PredictiveTestMix,    // Likely test based on patterns
    WeaknessAwareMix,     // Injects weak sub-areas
    ExamSimulationMix,    // Realistic experience
    ConfidenceBuilderMix, // Starts easier, increases
    PressureMix,          // Tight timing, fewer hints
    SmartRescueMix,       // Repairs weak concept before continuing
}
```

## 27.5 Class Test Mode Distribution
```
50% direct
30% moderate application
20% tricky teacher-style
```

## 27.6 Mock Exam Mode Distribution
```
20% direct
40% reasoning
40% exam-standard traps
```

## 27.7 Engine Input → Decision Mapping
```
Inputs: test_type, subject, selected_topics, exam_date, days_left,
        mastery_per_topic, past_errors, speed_level, confidence_level,
        preferred_mode, recent_learning_history

Decisions: num_questions, question_family_distribution, difficulty_blend,
           distractor_intensity, timer_behavior, hint_policy, feedback_policy,
           question_ordering, recovery_questions, confidence_restoring_questions
```

## 27.8 Prebuilt Packs
- Tomorrow's Class Test Pack
- Midterm Survival Pack
- Mock Pressure Pack
- Likely Questions Pack
- Topic Rescue Pack
- Teacher Trap Pack
- Fast Revision Pack

## 27.9 Feedback Timing Control
```rust
pub enum FeedbackTiming {
    Instant,
    Delayed,
    NoneUntilEnd,
    HiddenScore,
}
```

---

# 28. FULL FEATURE MAP & PRODUCT ARCHITECTURE
**Source: idea15.txt**

## 28.1 Ten-Layer Feature Inventory
```
Layer 1 — Learning:    Teach Mode, Examples, Library, Ask Tutor, Syllabus Explorer
Layer 2 — Practice:    Prepare Test, Quick Drills, Topic Practice, Games, Mental, Marathon, Answer Construction Lab
Layer 3 — Simulation:  Mock Centre, Class Test, Midterm, Final Exam, Pressure Mode, Strict Timing
Layer 4 — Diagnosis:   Knowledge Gap, Analytics, Mistake Lab, Misconception Detection, Exam Intel
Layer 5 — Repair:      Wrong Answer Repair, Concept Recovery, Misconception Correction, Prerequisite Repair
Layer 6 — Retention:   Memory Mode, Spaced Revision, Revision Box, Recall Drills, Decay Alerts
Layer 7 — Planning:    Mission Control, Daily Plan, Weekly Plan, Exam Countdown, Readiness Forecast
Layer 8 — Progression: Journey, Rise, Milestones, Streaks, Momentum, Recovery Paths
Layer 9 — Intelligence: Student Model, Question Intelligence, Misconception, Recommendation, Memory Decay, Exam Pattern
Layer 10 — Infrastructure: Offline packs, Local storage, Sync engine, Content Upload, Exam board packs
```

## 28.2 New Sidebar Architecture (5 Sections)
```
Learn:    Home, Explore, Teach, Examples, Library, Ask Tutor
Practice: Prepare Test, Games, Mental, Marathon, Mock Centre, Answer Lab
Diagnose: Knowledge Gap, Mistake Lab, Analytics, Exam Intel, Mastery Map
Retain:   Memory, Revision Box, History
Progress: Journey, Rise, Mission Control, Profile
```

## 28.3 Feature Tiers
- **Tier 1 Critical:** Teach Mode, Mistake Lab, Memory/Retention, Mission Control, Mastery Map, Ask Tutor, Answer Construction Lab
- **Tier 2 Strategic:** Revision Box, Speed vs Accuracy Lab, Focus Mode, Confidence Calibration, Advanced Exam Intel, Content Upload Engine
- **Tier 3 Polish:** Stronger Rise, Recovery mode, Advanced Journey, Offline management, Game integration depth

## 28.4 Six Core Functions
```
Teach:    Teach Mode, Examples, Ask Tutor
Test:     Prepare Test, Mock Centre, Mental, Marathon, Games
Diagnose: Knowledge Gap, Analytics, Mistake Lab, Exam Intel
Repair:   Misconception correction, targeted drills, prerequisite recovery
Retain:   Memory Mode, Revision Box, spaced recall
Guide:    Mission Control, Journey, Rise, Readiness Score
```

## 28.5 Student Model Engine Tracks
- Topic/subtopic/skill mastery
- Confidence quality
- Memory strength
- Pressure behavior
- Speed profile
- Misconception profile
- Preferred explanation style
- Engagement pattern
- Recovery capacity

## 28.6 Streaks + Momentum Types
```rust
pub enum StreakType {
    DailyLearning, TopicMastery, Revision, Accuracy,
}

pub enum MomentumType {
    RecentPerformance, Comeback, WeakTopicRecovery,
}
```

---

# 29. LIBRARY INTELLIGENCE SYSTEM
**Source: idea16.txt**

## 29.1 Library Sections (Full)
```
Library Home:     continue, due for revision, weak areas, recently saved, recommended packs, exam trends
My Shelf:         saved questions/explanations/notes/formulas/diagrams/packs
Topics:           all subjects, topic explorer, concept map, by difficulty/importance/exam frequency
Past Exam Vault:  by subject/year/topic, repeated questions, families, trends, inverse appearance
Mistake Bank:     recent/recurring/careless/memory/concept-mismatch/high-cost mistakes
Memory Shelf:     due for review, fading concepts, flashcards, formula recall, definition recall
Teach Shelf:      teach from scratch, explain simply, worked examples, concept builder
Revision Packs:   quick/weak-area/mock-prep/saved-custom/teacher/last-minute packs
Downloads/Offline: downloaded notes/packs/diagrams/audio/revision packs
```

## 29.2 Library Item States (13)
```rust
pub enum LibraryItemState {
    New, Saved, Studied, Understood, Weak, Revisit,
    Fading, Mastered, ExamCritical, Confusing,
    NeedsTeaching, NeedsMemoryReinforcement, InProgress,
}
```

## 29.3 Library Item Actions
- save_it, tag_it, add_note, ask_explanation, ask_worked_solution
- see_related_concepts, see_similar_questions, see_opposite_variants
- see_common_mistakes, test_on_it, add_to_revision_pack
- mark_for_memory_review, mark_as_weak, mark_as_mastered
- download_offline, share_to_parent_teacher

## 29.4 Mosaic Accordion Card Architecture

### Main Cards (with sizes)
- **Large:** Due Now, Continue Where Stopped, Topic Explorer, Mistake Bank, Memory Shelf
- **Medium:** Revision Packs, Exam Hotspots, Saved Questions, Formula Bank, My Notes
- **Small:** Reminders, mini stats, recently added, quick shortcuts, teacher tips

### Four Zone Layout
```
Zone A — Hero Intelligence Strip:  greeting, academic status, most urgent next action, mini indicators
Zone B — Primary Mosaic:          mixed card sizes based on importance/density
Zone C — Secondary Ribbon Shelves: horizontally scrollable rows
Zone D — Deep Content Dock:       accordion expansions with subcards
```

### Card States
```rust
pub enum CardState {
    Default, Hover, Pressed, Flipped, Expanded, Inactive, Loading, UpdatedSinceLastVisit,
}
```

### Card Interaction Depths (3)
```
Level 1 (Glance):  weak areas, memory fading, saved questions, mistake patterns, exam hotspots
Level 2 (Hover):   why matters, how urgent, what inside, what changed, available actions
Level 3 (Dive):    question lists, topic ladders, memory items, mistake clusters, filters, analytics
```

### Card Anatomy (4 States)
- **Front Face:** title, icon, short metric, status tone, quick label
- **Hover Layer:** why card matters now, recent changes, top 1-3 items, quick actions
- **Flip Face:** counts ↔ states, summary ↔ trend, concept list ↔ concept map
- **Expanded State:** sub-sections, filters, recommended actions, content list, analytics

## 29.5 Six Hidden Systems for Library
```
A. Content relationship engine   (question ↔ topic ↔ concept ↔ mistake ↔ exam family ↔ memory)
B. State engine                  (weak, fading, urgent, mastered, untouched)
C. Recommendation engine         (what appears on Due Now / Continue)
D. Memory engine                 (surface fading knowledge / reinforcement timing)
E. Mistake analysis engine       (classify / cluster errors)
F. Pack builder engine           (auto-create revision bundles)
```

## 29.6 Mistake Bank Error Types
```rust
pub enum MistakeBankErrorType {
    ConceptConfusion,
    CarelessMistake,
    MemoryFailure,
    MisreadQuestion,
    WeakMethod,
    PartialUnderstanding,
    SpeedPressureError,
    WrongEliminationLogic,
    FormulaMisuse,
}
```

## 29.7 Revision Pack Auto-Generation Targets
- Weak area pack
- Mock exam prep pack
- Formula rescue pack
- Last-minute revision pack
- Likely exam question pack
- Saved questions pack
- "Things I keep failing" pack
- Custom class test pack
- Topic crash pack
- Memory recovery pack

---

# 30. GLOSSARY LAB — KNOWLEDGE INTELLIGENCE LAYER
**Source: idea17.txt**

## 30.1 Unified Knowledge Entry Model (Full Schema)
```sql
CREATE TABLE knowledge_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id),
    topic_id INTEGER REFERENCES topics(id),
    subtopic_id INTEGER REFERENCES subtopics(id),
    entry_type TEXT NOT NULL
        CHECK (entry_type IN ('definition','formula','concept','law','rule','theorem','method','symbol')),
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    canonical_name TEXT NOT NULL,
    short_explanation TEXT,
    full_explanation TEXT,
    student_friendly_explanation TEXT,
    technical_explanation TEXT,
    difficulty_level TEXT,
    exam_level TEXT,
    importance_score INTEGER DEFAULT 5000,  -- basis points
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft','review','published','archived')),
    search_vector TEXT,    -- for FTS5 or simple search
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE entry_aliases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    alias_text TEXT NOT NULL,
    alias_type TEXT NOT NULL  -- 'synonym','abbreviation','common_name','misspelling'
);

CREATE TABLE entry_content_blocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    block_type TEXT NOT NULL,  -- 'short_definition','formula_display','diagram','worked_example','warning','comparison','quiz'
    order_index INTEGER NOT NULL DEFAULT 0,
    content_json TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE entry_relationships (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    from_entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id),
    to_entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id),
    relationship_type TEXT NOT NULL,  -- see RelationshipType enum
    strength_score INTEGER DEFAULT 5000,  -- basis points
    notes TEXT
);

CREATE TABLE entry_examples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    example_type TEXT NOT NULL,  -- 'positive','negative','worked'
    example_text TEXT NOT NULL,
    solution_text TEXT
);

CREATE TABLE entry_misconceptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    misconception_text TEXT NOT NULL,
    why_it_happens TEXT,
    correction TEXT,
    severity_score INTEGER DEFAULT 5000  -- basis points
);

CREATE TABLE entry_question_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id),
    question_id INTEGER NOT NULL,
    relation_type TEXT NOT NULL,  -- 'tests','explains','requires','illustrates'
    frequency_score INTEGER DEFAULT 5000  -- basis points
);

CREATE TABLE student_entry_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    student_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id),
    state TEXT NOT NULL DEFAULT 'unseen'
        CHECK (state IN ('unseen','seen','opened','read','saved','confused',
                         'partially_understood','mastered','forgotten','needs_review',
                         'frequently_mistaken','formula_known_application_weak',
                         'concept_known_recognition_weak')),
    mastery_score INTEGER DEFAULT 0,
    confusion_score INTEGER DEFAULT 0,
    last_viewed_at TEXT,
    last_tested_at TEXT,
    review_due_at TEXT,
    UNIQUE(student_id, entry_id)
);
```

## 30.2 Entry Relationship Types (13)
```rust
pub enum EntryRelationshipType {
    DependsOn,      // prerequisite
    PartOf,         // is a component of
    OppositeOf,     // semantic opposite
    ConfusedWith,   // commonly confused
    ExampleOf,      // is an instance of
    FormulaFor,     // formula applies to this concept
    DerivedFrom,    // mathematical derivation
    UsedIn,         // appears in this context
    TestedWith,     // co-occurs in exam questions
    PrerequisiteFor,
    SimplerThan,
    DeeperThan,
    SynonymOf,
}
```

## 30.3 Type-Specific Extensions
```rust
pub struct DefinitionMeta {
    formal_definition: String,
    plain_english: String,
    real_world_meaning: String,
    examples: Vec<String>,
    non_examples: Vec<String>,
}

pub struct FormulaMeta {
    formula_latex: String,
    formula_plaintext: String,
    variable_map: HashMap<String, VariableInfo>,  // symbol → meaning, unit, range
    derived_from: Option<String>,
    rearrangements: Vec<String>,
    conditions_of_use: String,
    assumptions: String,
    common_errors: Vec<String>,
}

pub struct ConceptMeta {
    intuition: String,
    mental_model: String,
    why_it_matters: String,
    key_signals: Vec<String>,       // how to recognize in questions
    prerequisite_concepts: Vec<i64>,
    dependent_concepts: Vec<i64>,
    misconceptions: Vec<String>,
}
```

## 30.4 Five Formula Experience Features
```
A. Formula display:     normal math form, plain text, audio-readable
B. Variable explorer:   tap variable → meaning, unit, values, common confusion
C. Formula rearranger:  make any variable the subject, show step-by-step
D. Formula origin/proof: why it works, how formed, proof path, intuition path
E. Formula use-detection: this question likely needs this formula
```

## 30.5 Eight Smart Glossary Views
1. Discover — featured concepts by topic, recently used, important this week
2. Search — fast semantic with grouped results
3. Concept Page — detailed structured entry
4. Compare — side-by-side explanation (e.g., Osmosis vs Diffusion)
5. Formula Lab — interactive formula exploration with live values
6. Concept Map — visual relationship graph
7. My Weak Concepts — student-personalized view
8. Exam Hotspots — terms appearing heavily in exam patterns

## 30.6 Layered Explanation Modes
```rust
pub enum ExplanationDepth {
    Quick,   // One-line answer
    Simple,  // Student-friendly explanation
    Exam,    // How it is tested
    Deep,    // Detailed reasoning/proof/derivation
    Visual,  // Diagram-based
    Audio,   // Spoken explanation
}
```

## 30.7 Knowledge Cluster Model
```sql
CREATE TABLE knowledge_clusters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_name TEXT NOT NULL,
    subject_id INTEGER NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE cluster_entries (
    cluster_id INTEGER NOT NULL REFERENCES knowledge_clusters(id),
    entry_id INTEGER NOT NULL REFERENCES knowledge_entries(id),
    role TEXT,   -- 'anchor','formula','context','related','confusion'
    PRIMARY KEY (cluster_id, entry_id)
);
```

## 30.8 Glossary V1/V2/V3 Roadmap
- **V1:** unified entry model, definitions/formulas/concepts, search, detailed pages, related entries, worked examples, misconceptions
- **V2:** comparison mode, formula rearranger, student mastery states, linked question families, adaptive entry depth
- **V3:** semantic search, concept map graph, AI-generated explanations, auto-detection from question text, personalized weak-concept feed, voice/visual modes

---

# 31. DIAGNOSTIC TEST SYSTEM (FULL BATTERY SPEC)
**Source: idea18.txt**

## 31.1 Six Diagnostic Sessions + Two Bonus
```rust
pub enum DiagnosticSession {
    // Core 6
    BaselineMastery,     // What can student do under normal fair conditions?
    SpeedResponse,       // How fluent is retrieval?
    AccuracyControl,     // When less rushed, can they do it correctly?
    PressureTolerance,   // What happens under cognitive stress?
    FragilityTest,       // Does knowledge survive variation?
    WeaknessIsolation,   // Pinpoint exact component failure

    // Bonus 2
    TransferRecognition, // Can student recognize concept without announcement?
    EnduranceFatigue,    // Does performance drop over time?
}
```

## 31.2 Four Performance Bands
```rust
pub enum PerformanceBand {
    Solid,              // Consistent even with format/pressure changes
    StableNeedsFirming, // Usually right but not automatic/resilient
    Fragile,            // Sometimes right; breaks under changed conditions
    CriticallyWeak,     // Lacks underlying understanding/retrieval
}
```

## 31.3 Eight Weakness Types
```rust
pub enum DiagnosticWeaknessType {
    Knowledge,           // Doesn't know it
    Retrieval,           // Knows but can't recall quickly
    Recognition,         // Knows idea but can't identify when tested indirectly
    Execution,           // Knows what to do but makes mistakes
    Pressure,            // Worse under time pressure
    Fragility,           // Succeeds only in familiar format
    Endurance,           // Performance falls as session length increases
    ConfidenceControl,   // Second-guesses, rushes, changes right answers
}
```

## 31.4 Six Student Diagnostic Types
```rust
pub enum StudentDiagnosticType {
    Sprinter,              // Fast, decent recall, accuracy drops in complexity
    CarefulThinker,        // Slow but accurate; needs fluency not reteaching
    FragileKnower,         // OK in familiar forms, breaks under variation
    PressureCollapser,     // Understands calmly, loses under time
    FormulaMemorizer,      // Remembers formulas but can't interpret/apply
    ConceptualWeakExecutor,// Knows idea, makes arithmetic/step errors
    // Note: "Recognition Gap Student" also identified as implicit type
}
```

## 31.5 Seven Delta Comparison Metrics
```rust
pub struct DiagnosticDeltas {
    speed_accuracy_delta: i32,       // How much accuracy falls when time shortens
    calm_pressure_delta: i32,        // How much performance changes under pressure
    direct_variant_delta: i32,       // How much weaker when wording changes
    recall_application_delta: i32,   // Can remember but not use?
    formula_recall_use_delta: i32,   // Recite but fail to deploy?
    early_late_session_delta: i32,   // Fatigue effect
    confidence_correctness_delta: i32, // Fast confident ≠ correct?
}
```

## 31.6 Six Master Diagnostic Blocks
```rust
pub enum DiagnosticBlock {
    KnowledgeBaseline,   // What do you know in normal conditions?
    FluencySpeed,        // How quickly can you retrieve/respond?
    PrecisionAccuracy,   // How correctly when given time?
    PressureResilience,  // What happens under time/tension?
    FlexibilityTransfer, // Can still solve when wording/form/context changes?
    RootCauseIsolation,  // Where is breakdown: term/concept/formula/execution/control?
}
```

## 31.7 Five Diagnostic Dimensions
```rust
pub enum DiagnosticDimension {
    Knowledge,   // Term recognition, definition, concept, formula recall/meaning, calculation, application
    Fluency,     // Retrieval speed, response initiation, automaticity, recognition speed
    Precision,   // Careful execution, step accuracy, distractor resistance, answer checking
    Resilience,  // Pressure tolerance, time-compression tolerance, error recovery, fatigue resistance
    Flexibility, // Recognition under different wording, transfer, formula use in disguised problems
}
```

## 31.8 Session Group Designs
**For Terms:**
- define term → identify term from meaning → spot misuse → match to example → distinguish close terms
- Tells you: vocabulary solid or superficial

**For Formulae:**
- recall from memory → identify correct formula → explain meaning → substitute correctly → solve transformed → detect when doesn't apply
- Tells you: formula as memory only vs as understanding

**For Calculation:**
- arithmetic speed → accuracy → multi-step chain → error propagation → rounding/units/sign
- Tells you: weakness is actual math execution

**For Concepts:**
- explain in own words → identify from example → compare with related → apply in novel situation → reject incorrect application
- Reveals: real understanding

**For Pressure:**
- timed rapid-fire → shrinking windows → streak retention → answer-commit deadlines → surprise difficulty shift
- Reveals: response under stress

## 31.9 Post-Diagnostic Action Mapping
```
Strong areas:   low-frequency maintenance only
Needs firming:  spaced reinforcement
Fragile:        same skill across variants + mixed contexts
Critically weak: reteach + guided examples + rebuild from basics
Pressure issue: pressure ladder drills
Speed issue:    fluency training
Recognition:    "name the concept" drills
Formula issue:  formula meaning + selection drills
```

## 31.10 Session Names
- **Premium labels:** Foundation Scan, Speed Scan, Precision Scan, Pressure Scan, Flex Scan, Weakness Scan
- **Standard labels:** Baseline, Fluency, Precision, Pressure, Transfer, Root Cause

## 31.11 Core Principle
> Do not conclude "weak" too early. A student may be knowledgeable but slow, accurate but not fast, fast but careless, conceptually sound but computationally weak, fine in calm but poor under pressure, good in direct but weak in transfer. Every conclusion requires cross-session evidence.

---

# APPENDIX F: SUPPLEMENT NOTES ON EXISTING PLAN GAPS

## F.1 Subject-Specific Confirmation Requirements (Memory Mode, idea7)
- **Mathematics:** accuracy + speed + no-hint + calculation-variant
- **Science:** definitions + process-description + diagram-interpretation + cause-effect
- **Theory subjects:** recall + explanation + comparison + application-in-context

## F.2 Curriculum Node ID Format (idea31)
- Format: `B7/JHS1.1.1.1.1`
- Encodes: Class.Strand.SubStrand.ContentStandard.Indicator
- B7 = Basic 7 = JHS1

## F.3 Content Downstream Object Requirements (idea31)
Every curriculum node MUST be able to generate or link to:
- Lesson, Question, WorkedExample, Flashcard
- DiagnosticProbe, RevisionSet, MockPaperSection, RemediationPlan

## F.4 Publishing States (idea31)
```rust
pub enum PublishState { Draft, Review, Published }
```

## F.5 ARIL Coverage Score Thresholds (idea21)
```
Red   = insufficient — cannot coach this topic
Amber = usable but limited
Green = ready — full coaching possible
Blue  = mastery-grade rich
```

## F.6 Intelligence Constitution Scaling Target (idea25)
```
Minimum serious v2 stack: 60k–120k LOC
World-class coach brain:   120k–250k+ LOC

Current nucleus: 12,068 LOC (6 engines, 53 algorithms)
```

## F.7 Behavioral Signal Collection (idea18, idea32)
Capture on EVERY question attempt:
- response_time_ms
- time_to_first_action_ms
- answer_change_count
- skipped flag
- hesitation_pattern (derived from timing gaps)
- confidence_level (if asked)
- whether wrong answer close/random
- error_classification: conceptual | careless | procedural | recognition

---

*This supplement was generated from a complete deep micro-chunk re-read of all 38 source files.*
*It is intended to be read alongside `detailed_backend_implementation_plan.md`.*
*Together these two documents constitute the complete backend specification for eCoach.*
