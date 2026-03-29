# PART 2: COMPLETE TYPESCRIPT TYPES & TAURI IPC COMMAND BINDINGS

---

## SECTION 6: COMPLETE TYPESCRIPT TYPES

Every type below mirrors the exact Rust DTO from the corresponding crate. Field names use camelCase (serde serializes snake_case, but Tauri's invoke bridge deserializes to camelCase by convention in the frontend). Where Rust uses `BasisPoints` (u16, range 0-10000), TypeScript uses `number`. Where Rust uses `i64`, TypeScript uses `number`. Where Rust uses `Option<T>`, TypeScript uses `T | null`. Where Rust uses `DateTime<Utc>`, TypeScript uses `string` (ISO 8601).

All types live in `src/types/` organized by domain.

---

### 6.1 Substrate Types (`src/types/substrate.ts`)

```typescript
// BasisPoints: u16 in Rust, 0-10000 range representing 0.00% to 100.00%
export type BasisPoints = number;

export type Role = 'student' | 'parent' | 'admin' | 'super_admin';

export type AccountType = 'student' | 'parent' | 'admin';

export type EntitlementTier = 'standard' | 'premium' | 'elite';

export interface DomainEvent {
  eventId: string;
  eventType: string;
  aggregateId: string;
  occurredAt: string;
  payload: unknown;
  traceId: string;
}

export interface ThresholdRegistry {
  masteryExamReady: BasisPoints;
  masteryStable: BasisPoints;
  readinessExamReady: BasisPoints;
  readinessBuilding: BasisPoints;
}

export interface EngineContract {
  key: string;
  title: string;
  purpose: string;
  ownsState: string[];
  primaryInputs: string[];
  primaryOutputs: string[];
  controlTags: string[];
  offlineRequired: boolean;
  modules: string[];
}

export interface EngineRegistry {
  engines: EngineContract[];
}

export type EcoachErrorKind =
  | 'validation'
  | 'not_found'
  | 'unauthorized'
  | 'storage'
  | 'serialization'
  | 'unsupported';

export interface EcoachError {
  kind: EcoachErrorKind;
  message: string;
}
```

---

### 6.2 Identity Types (`src/types/identity.ts`)

```typescript
import type { AccountType, EntitlementTier } from './substrate';

export interface CreateAccountInput {
  accountType: AccountType;
  displayName: string;
  pin: string;
  entitlementTier: EntitlementTier;
}

export interface Account {
  id: number;
  accountType: AccountType;
  displayName: string;
  entitlementTier: EntitlementTier;
  failedPinAttempts: number;
  lockedUntil: string | null;
  status: string;
  firstRun: boolean;
  lastActiveAt: string | null;
}

export interface AccountSummary {
  id: number;
  displayName: string;
  accountType: AccountType;
  status: string;
  firstRun: boolean;
  lastActiveAt: string | null;
}

export interface AuthResult {
  success: boolean;
  account: Account | null;
  error: string | null;
  remainingAttempts: number | null;
  lockedUntil: string | null;
}

export interface UpdateAccountInput {
  displayName?: string;
  entitlementTier?: EntitlementTier;
}

export interface ChangePinInput {
  currentPin: string;
  newPin: string;
}
```

---

### 6.3 Curriculum Types (`src/types/curriculum.ts`)

```typescript
export interface Subject {
  id: number;
  curriculumVersionId: number;
  code: string;
  name: string;
  displayOrder: number;
}

export interface TopicSummary {
  id: number;
  subjectId: number;
  parentTopicId: number | null;
  code: string | null;
  name: string;
  nodeType: string;
  displayOrder: number;
}

export type NodeType =
  | 'definition'
  | 'concept'
  | 'procedure'
  | 'theorem'
  | 'formula'
  | 'property'
  | 'model'
  | 'heuristic'
  | 'convention';

export interface AcademicNode {
  id: number;
  topicId: number;
  nodeType: string;
  canonicalTitle: string;
  coreMeaning: string | null;
  examRelevanceScore: number;
  createdAt: string;
}

export interface CurriculumSourceUpload {
  id: number;
  uploaderAccountId: number;
  sourceKind: string;
  title: string;
  sourcePath: string | null;
  countryCode: string | null;
  examBoard: string | null;
  educationLevel: string | null;
  subjectCode: string | null;
  academicYear: string | null;
  languageCode: string;
  versionLabel: string | null;
  sourceStatus: string;
  confidenceScore: number;
  metadata: unknown;
}

export interface CurriculumParseCandidate {
  id: number;
  sourceUploadId: number;
  candidateType: string;
  parentCandidateId: number | null;
  rawLabel: string;
  normalizedLabel: string | null;
  payload: unknown;
  confidenceScore: number;
  reviewStatus: string;
}

export interface CurriculumReviewTask {
  id: number;
  sourceUploadId: number;
  candidateId: number | null;
  taskType: string;
  status: string;
  severity: string;
  notes: string | null;
}

export interface CurriculumSourceReport {
  sourceUpload: CurriculumSourceUpload;
  candidates: CurriculumParseCandidate[];
  reviewTasks: CurriculumReviewTask[];
}

export interface CurriculumVersion {
  id: number;
  countryCode: string;
  examBoard: string;
  educationLevel: string;
  versionLabel: string;
  status: string;
}
```

---

### 6.4 Questions Types (`src/types/questions.ts`)

```typescript
import type { BasisPoints } from './substrate';

export type QuestionFormat =
  | 'multiple_choice'
  | 'true_false'
  | 'fill_blank'
  | 'short_answer'
  | 'multi_select'
  | 'ordering'
  | 'matching';

export interface Question {
  id: number;
  subjectId: number;
  topicId: number;
  subtopicId: number | null;
  familyId: number | null;
  stem: string;
  questionFormat: string;
  explanationText: string | null;
  difficultyLevel: BasisPoints;
  estimatedTimeSeconds: number;
  marks: number;
  primarySkillId: number | null;
}

export interface QuestionOption {
  id: number;
  questionId: number;
  optionLabel: string;
  optionText: string;
  isCorrect: boolean;
  misconceptionId: number | null;
  distractorIntent: string | null;
  position: number;
}

export interface QuestionFamily {
  id: number;
  subjectId: number;
  topicId: number;
  familyCode: string;
  familyName: string;
  questionCount: number;
}

export interface QuestionIntelligenceLink {
  axisCode: string;
  conceptCode: string;
  displayName: string;
  confidenceScore: BasisPoints;
  isPrimary: boolean;
}

export interface QuestionIntelligenceProfile {
  question: Question;
  links: QuestionIntelligenceLink[];
}

export interface QuestionIntelligenceQuery {
  axisCode: string;
  conceptCode: string;
  subjectId: number | null;
  topicId: number | null;
  limit: number;
}

export interface QuestionSelectionRequest {
  subjectId: number;
  topicIds: number[];
  targetQuestionCount: number;
  targetDifficulty: BasisPoints | null;
  weaknessTopicIds: number[];
  recentlySeenQuestionIds: number[];
  timed: boolean;
}

export interface SelectedQuestion {
  question: Question;
  fitScore: number;
}
```

---

### 6.5 Student Model Types (`src/types/student-model.ts`)

```typescript
import type { BasisPoints } from './substrate';

export interface AnswerSubmission {
  questionId: number;
  selectedOptionId: number;
  sessionId: number | null;
  sessionType: string | null;
  startedAt: string;
  submittedAt: string;
  responseTimeMs: number | null;
  confidenceLevel: string | null;
  hintCount: number;
  changedAnswerCount: number;
  skipped: boolean;
  timedOut: boolean;
  supportLevel: string | null;
  wasTimed: boolean;
  wasTransferVariant: boolean;
  wasRetentionCheck: boolean;
  wasMixedContext: boolean;
}

export type ErrorType =
  | 'knowledge_gap'
  | 'conceptual_confusion'
  | 'recognition_failure'
  | 'execution_error'
  | 'carelessness'
  | 'pressure_breakdown'
  | 'expression_weakness'
  | 'speed_error'
  | 'guessing_detected'
  | 'misconception_triggered';

export type MasteryState =
  | 'unseen'
  | 'exposed'
  | 'emerging'
  | 'partial'
  | 'fragile'
  | 'stable'
  | 'robust'
  | 'exam_ready';

export interface StudentTopicState {
  id: number;
  studentId: number;
  topicId: number;
  masteryScore: BasisPoints;
  masteryState: MasteryState;
  accuracyScore: BasisPoints;
  speedScore: BasisPoints;
  confidenceScore: BasisPoints;
  retentionScore: BasisPoints;
  transferScore: BasisPoints;
  consistencyScore: BasisPoints;
  gapScore: BasisPoints;
  priorityScore: BasisPoints;
  trendState: string;
  fragilityScore: BasisPoints;
  pressureCollapseIndex: BasisPoints;
  totalAttempts: number;
  correctAttempts: number;
  evidenceCount: number;
  lastSeenAt: string | null;
  lastCorrectAt: string | null;
  memoryStrength: BasisPoints;
  nextReviewAt: string | null;
  version: number;
}

export interface AnswerProcessingResult {
  isCorrect: boolean;
  errorType: ErrorType | null;
  diagnosisSummary: string | null;
  recommendedAction: string | null;
  explanation: string | null;
  selectedOptionText: string;
  correctOptionText: string | null;
  updatedMastery: BasisPoints;
  updatedGap: BasisPoints;
  misconceptionInfo: string | null;
}

export interface LearnerTruthTopicSummary {
  topicId: number;
  topicName: string;
  masteryScore: BasisPoints;
  masteryState: string;
  gapScore: BasisPoints;
  priorityScore: BasisPoints;
  memoryStrength: BasisPoints;
  nextReviewAt: string | null;
}

export interface LearnerTruthSkillSummary {
  nodeId: number;
  title: string;
  masteryScore: BasisPoints;
  gapScore: BasisPoints;
  priorityScore: BasisPoints;
  state: string;
}

export interface LearnerTruthMemorySummary {
  topicId: number | null;
  topicName: string | null;
  nodeId: number | null;
  nodeTitle: string | null;
  memoryState: string;
  memoryStrength: BasisPoints;
  recallFluency: BasisPoints;
  decayRisk: BasisPoints;
  reviewDueAt: string | null;
}

export interface LearnerTruthDiagnosisSummary {
  diagnosisId: number;
  topicId: number;
  topicName: string;
  primaryDiagnosis: string;
  severity: string;
  recommendedAction: string;
  createdAt: string;
}

export interface LearnerTruthSnapshot {
  studentId: number;
  studentName: string;
  overallMasteryScore: BasisPoints;
  overallReadinessBand: string;
  pendingReviewCount: number;
  dueMemoryCount: number;
  topicSummaries: LearnerTruthTopicSummary[];
  skillSummaries: LearnerTruthSkillSummary[];
  memorySummaries: LearnerTruthMemorySummary[];
  recentDiagnoses: LearnerTruthDiagnosisSummary[];
}
```

---

### 6.6 Sessions Types (`src/types/sessions.ts`)

```typescript
import type { BasisPoints } from './substrate';

export type SessionStatus = 'active' | 'paused' | 'completed' | 'abandoned';

export type SessionType = 'practice' | 'custom_test' | 'diagnostic' | 'mission' | 'mock_exam' | 'repair';

export interface Session {
  id: number;
  studentId: number;
  sessionType: string;
  subjectId: number | null;
  status: string;
  activeItemIndex: number;
  startedAt: string | null;
  pausedAt: string | null;
  completedAt: string | null;
  lastActivityAt: string | null;
}

export interface SessionItem {
  id: number;
  sessionId: number;
  questionId: number;
  displayOrder: number;
  sourceFamilyId: number | null;
  sourceTopicId: number | null;
  status: string;
  selectedOptionId: number | null;
  flagged: boolean;
  responseTimeMs: number | null;
  isCorrect: boolean | null;
}

export interface SessionSnapshot {
  session: Session;
  items: SessionItem[];
}

export interface SessionSummary {
  sessionId: number;
  accuracyScore: number | null;
  answeredQuestions: number;
  correctQuestions: number;
  status: string;
}

export interface PracticeSessionStartInput {
  studentId: number;
  subjectId: number;
  topicIds: number[];
  questionCount: number;
  isTimed: boolean;
}

export interface CustomTestStartInput {
  studentId: number;
  subjectId: number;
  topicIds: number[];
  questionCount: number;
  durationMinutes: number | null;
  isTimed: boolean;
  targetDifficulty: BasisPoints | null;
  weaknessBias: boolean;
}

export interface SessionAnswerInput {
  itemId: number;
  selectedOptionId: number;
  responseTimeMs: number | null;
}
```

---

### 6.7 Coach Brain Types (`src/types/coach.ts`)

```typescript
import type { BasisPoints } from './substrate';

export type LearnerJourneyState =
  | 'onboarding_required'
  | 'subject_selection_required'
  | 'content_readiness_required'
  | 'diagnostic_required'
  | 'plan_generation_required'
  | 'ready_for_today_mission'
  | 'mission_in_progress'
  | 'mission_review_required'
  | 'repair_required'
  | 'blocked_on_topic'
  | 'plan_adjustment_required'
  | 'review_day'
  | 'exam_mode'
  | 'stalled_no_content';

export type CoachActionType =
  | 'continue_onboarding'
  | 'select_subjects'
  | 'resolve_content'
  | 'start_diagnostic'
  | 'generate_plan'
  | 'start_today_mission'
  | 'resume_mission'
  | 'review_results'
  | 'start_repair'
  | 'adjust_plan'
  | 'view_overview';

export type ContentReadinessStatus =
  | 'ready'
  | 'no_subjects_selected'
  | 'no_packs_installed'
  | 'no_topics_available'
  | 'topics_exist_but_no_questions'
  | 'insufficient_question_coverage';

export interface CoachStateResolution {
  state: LearnerJourneyState;
  reason: string | null;
}

export interface ContentReadinessResolution {
  status: ContentReadinessStatus;
  subjectCodes: string[];
  activePackCount: number;
  topicCount: number;
  questionCount: number;
  reason: string | null;
}

export interface CoachNextAction {
  state: LearnerJourneyState;
  actionType: CoachActionType;
  title: string;
  subtitle: string;
  estimatedMinutes: number | null;
  route: string;
  context: unknown;
}

export interface TopicCase {
  studentId: number;
  topicId: number;
  topicName: string;
  subjectCode: string;
  priorityScore: BasisPoints;
  masteryScore: BasisPoints;
  masteryState: string;
  gapScore: BasisPoints;
  fragilityScore: BasisPoints;
  pressureCollapseIndex: BasisPoints;
  memoryState: string;
  memoryStrength: BasisPoints;
  decayRisk: BasisPoints;
  evidenceCount: number;
  recentAttemptCount: number;
  recentAccuracy: BasisPoints | null;
  activeBlocker: TopicCaseBlocker | null;
  recentDiagnoses: TopicCaseDiagnosis[];
  activeHypotheses: TopicCaseHypothesis[];
  primaryHypothesisCode: string;
  diagnosisCertainty: BasisPoints;
  requiresProbe: boolean;
  recommendedIntervention: TopicCaseIntervention;
  proofGaps: string[];
  openQuestions: string[];
}

export interface TopicCaseBlocker {
  reason: string;
  severity: string;
}

export interface TopicCaseDiagnosis {
  diagnosisId: number;
  errorType: string;
  primaryDiagnosis: string;
  severity: string;
  diagnosisSummary: string;
  recommendedAction: string;
  confidenceScore: BasisPoints;
  createdAt: string;
}

export interface TopicCaseHypothesis {
  code: string;
  label: string;
  confidenceScore: BasisPoints;
  evidenceSummary: string;
  recommendedProbe: string | null;
  recommendedResponse: string;
}

export interface TopicCaseIntervention {
  mode: string;
  urgency: string;
  nextActionType: string;
  recommendedMinutes: number;
  reason: string;
}

export interface CoachMissionMemory {
  id: number;
  missionId: number;
  planDayId: number | null;
  studentId: number;
  sessionId: number | null;
  subjectId: number | null;
  topicId: number | null;
  missionStatus: string;
  attemptCount: number;
  correctCount: number;
  accuracyScore: number | null;
  avgLatencyMs: number | null;
  misconceptionTags: string[];
  reviewDueAt: string | null;
  nextActionType: string;
  strategyEffect: string | null;
  summaryJson: string;
  reviewStatus: string;
}

export interface CoachPlan {
  id: number;
  studentId: number;
  examTarget: string;
  examDate: string;
  startDate: string;
  totalDays: number;
  dailyBudgetMinutes: number;
  currentPhase: string;
  status: string;
  planDataJson: string;
}

export interface CoachPlanDay {
  id: number;
  planId: number;
  date: string;
  phase: string;
  targetMinutes: number;
  status: string;
}

export interface CoachMission {
  id: number;
  planDayId: number;
  studentId: number;
  title: string;
  reason: string;
  subjectId: number | null;
  primaryTopicId: number | null;
  activityType: string;
  targetMinutes: number;
  status: string;
}
```

---

### 6.8 Diagnostics Types (`src/types/diagnostics.ts`)

```typescript
import type { BasisPoints } from './substrate';

export type DiagnosticMode = 'quick' | 'standard' | 'deep';

export type DiagnosticPhaseCode =
  | 'baseline'
  | 'speed'
  | 'precision'
  | 'pressure'
  | 'flex'
  | 'root_cause';

export interface DiagnosticBattery {
  diagnosticId: number;
  studentId: number;
  subjectId: number;
  sessionMode: string;
  status: string;
  phases: DiagnosticPhasePlan[];
}

export interface DiagnosticPhasePlan {
  phaseId: number;
  phaseNumber: number;
  phaseCode: string;
  phaseTitle: string;
  phaseType: string;
  status: string;
  questionCount: number;
  timeLimitSeconds: number | null;
  conditionType: string;
}

export interface DiagnosticPhaseItem {
  phaseId: number;
  questionId: number;
  displayOrder: number;
  conditionType: string;
  stem: string;
  questionFormat: string;
  topicId: number;
}

export interface DiagnosticResult {
  overallReadiness: BasisPoints;
  readinessBand: string;
  topicResults: TopicDiagnosticResult[];
  recommendedNextActions: string[];
}

export interface TopicDiagnosticResult {
  topicId: number;
  topicName: string;
  masteryScore: BasisPoints;
  fluencyScore: BasisPoints;
  precisionScore: BasisPoints;
  pressureScore: BasisPoints;
  flexibilityScore: BasisPoints;
  stabilityScore: BasisPoints;
  classification: string;
}

export interface WrongAnswerDiagnosis {
  id: number;
  studentId: number;
  questionId: number;
  topicId: number;
  errorType: string;
  primaryDiagnosis: string;
  secondaryDiagnosis: string | null;
  severity: string;
  diagnosisSummary: string;
  recommendedAction: string;
  confidenceScore: BasisPoints;
  createdAt: string;
}
```

---

### 6.9 Reporting Types (`src/types/reporting.ts`)

```typescript
export interface SubjectSummary {
  subjectId: number;
  subjectName: string;
  readinessBand: string;
  masteredTopicCount: number;
  weakTopicCount: number;
  totalTopicCount: number;
}

export interface StudentDashboard {
  studentName: string;
  examTarget: string | null;
  subjectSummaries: SubjectSummary[];
  overallReadinessBand: string;
}

export interface ParentDashboardSnapshot {
  parentId: number;
  parentName: string;
  students: ParentStudentSummary[];
  generatedAt: string;
}

export interface ParentStudentSummary {
  studentId: number;
  studentName: string;
  overallReadinessBand: string;
  examTarget: string | null;
  activeRisks: ParentRiskSummary[];
  recommendations: string[];
  trendSummary: string[];
  weeklyMemo: string;
  subjectSummaries: SubjectSummary[];
}

export interface ParentRiskSummary {
  severity: string;
  title: string;
  description: string;
}
```

---

### 6.10 Glossary Types (`src/types/glossary.ts`)

```typescript
export interface KnowledgeEntry {
  id: number;
  title: string;
  entryType: string;
  shortText: string | null;
  topicId: number | null;
}

export interface KnowledgeBundle {
  id: number;
  title: string;
  bundleType: string;
  topicId: number | null;
}

export interface QuestionKnowledgeLink {
  questionId: number;
  entryId: number;
  title: string;
  entryType: string;
  shortText: string | null;
  topicId: number | null;
  relationType: string;
  linkSource: string;
  linkReason: string | null;
  confidenceScore: number;
  isPrimary: boolean;
}
```

---

### 6.11 Library Types (`src/types/library.ts`)

```typescript
import type { BasisPoints } from './substrate';

export interface LibraryItem {
  id: number;
  studentId: number;
  itemType: string;
  itemRefId: number;
  state: string;
  tags: string[];
  noteText: string | null;
  topicId: number | null;
  urgencyScore: BasisPoints;
}

export interface SaveLibraryItemInput {
  itemType: string;
  itemRefId: number;
  state: string;
  tags: string[];
  noteText: string | null;
  topicId: number | null;
  urgencyScore: BasisPoints;
}

export interface SavedQuestionCard {
  libraryItemId: number;
  questionId: number;
  topicId: number;
  topicName: string;
  stem: string;
  state: string;
  relatedFamilyName: string | null;
  linkedKnowledgeCount: number;
  urgencyScore: BasisPoints;
  savedAt: string;
}

export interface LibraryShelfItem {
  itemType: string;
  itemRefId: number | null;
  title: string;
  subtitle: string | null;
  reason: string;
  rankScore: BasisPoints;
  metadata: unknown;
}

export interface GeneratedLibraryShelf {
  shelfId: number | null;
  shelfType: string;
  title: string;
  generated: boolean;
  items: LibraryShelfItem[];
}

export interface ContinueLearningCard {
  title: string;
  activityType: string;
  topicId: number | null;
  topicName: string | null;
  missionId: number | null;
  sessionId: number | null;
  route: string;
}

export interface RevisionPackItem {
  id: number;
  itemType: string;
  itemRefId: number;
  sequenceOrder: number;
  required: boolean;
  metadata: unknown;
}

export interface RevisionPackSummary {
  packId: number;
  title: string;
  sourceType: string | null;
  topicIds: number[];
  questionCount: number;
  createdAt: string;
}

export interface LibraryHomeSnapshot {
  dueNowCount: number;
  pendingReviewCount: number;
  fadingConceptCount: number;
  untouchedSavedCount: number;
  continueCard: ContinueLearningCard | null;
  generatedShelves: GeneratedLibraryShelf[];
  savedQuestions: SavedQuestionCard[];
}
```

---

### 6.12 Games Types (`src/types/games.ts`)

```typescript
import type { BasisPoints } from './substrate';

export type GameType = 'mind_stack' | 'tug_of_war' | 'speed_round' | 'concept_chain';

export interface GameSession {
  id: number;
  studentId: number;
  gameType: string;
  sessionState: string;
  score: number;
  startedAt: string;
  completedAt: string | null;
}

// MindStack: Stack-based game where concepts are placed in order
export interface MindStackState {
  sessionId: number;
  topicId: number;
  stack: MindStackItem[];
  currentLevel: number;
  score: number;
  livesRemaining: number;
  timeRemainingMs: number | null;
}

export interface MindStackItem {
  conceptId: number;
  label: string;
  position: number;
  isLocked: boolean;
}

// TugOfWar: Competing concept identification
export interface TugOfWarState {
  sessionId: number;
  topicId: number;
  leftConcept: string;
  rightConcept: string;
  items: TugOfWarItem[];
  leftScore: number;
  rightScore: number;
  roundNumber: number;
  totalRounds: number;
}

export interface TugOfWarItem {
  id: number;
  text: string;
  correctSide: 'left' | 'right';
  placed: boolean;
  placedSide: 'left' | 'right' | null;
}

export interface GameResult {
  sessionId: number;
  gameType: string;
  score: number;
  accuracy: BasisPoints;
  topicId: number;
  conceptsReinforced: number;
  bonusXp: number;
}
```

---

### 6.13 Traps Types (`src/types/traps.ts`)

```typescript
import type { BasisPoints } from './substrate';

export interface TrapPattern {
  id: number;
  topicId: number;
  trapType: string;
  description: string;
  triggerCondition: string;
  prevalenceScore: BasisPoints;
}

export interface StudentTrapProfile {
  studentId: number;
  topicId: number;
  trapId: number;
  hitCount: number;
  lastHitAt: string | null;
  resistanceScore: BasisPoints;
  status: string;
}

export interface TrapDrillItem {
  questionId: number;
  trapId: number;
  stem: string;
  isTrapVariant: boolean;
  explanationText: string | null;
}

export interface TrapDrillSession {
  sessionId: number;
  studentId: number;
  topicId: number;
  trapIds: number[];
  items: TrapDrillItem[];
  status: string;
}
```

---

### 6.14 Past Papers Types (`src/types/past-papers.ts`)

```typescript
import type { BasisPoints } from './substrate';

export interface PastPaperSet {
  id: number;
  subjectId: number;
  examYear: number;
  paperCode: string | null;
  title: string;
}

export interface PastPaperSetSummary {
  paperId: number;
  examYear: number;
  title: string;
  questionCount: number;
}

export interface PastPaperFamilyAnalytics {
  familyId: number;
  familyCode: string;
  familyName: string;
  topicId: number | null;
  recurrenceScore: BasisPoints;
  coappearanceScore: BasisPoints;
  replacementScore: BasisPoints;
  paperCount: number;
  lastSeenYear: number | null;
}

export interface PastPaperQuestionLink {
  paperId: number;
  questionId: number;
  sectionLabel: string | null;
  questionNumber: string | null;
}
```

---

### 6.15 Goals & Calendar Types (`src/types/goals-calendar.ts`)

```typescript
import type { BasisPoints } from './substrate';

export interface Goal {
  id: number;
  studentId: number;
  goalType: string;
  title: string;
  description: string | null;
  status: string;
}

export interface CalendarEvent {
  id: number;
  studentId: number;
  eventType: string;
  title: string;
  scheduledFor: string;
}

export interface AvailabilityProfile {
  studentId: number;
  timezoneName: string;
  preferredDailyMinutes: number;
  minSessionMinutes: number;
  maxSessionMinutes: number;
}

export interface AvailabilityWindow {
  weekday: number;
  startMinute: number;
  endMinute: number;
  isPreferred: boolean;
}

export interface AvailabilityException {
  exceptionDate: string;
  startMinute: number | null;
  endMinute: number | null;
  availabilityMode: string;
  minutesDelta: number;
  reason: string | null;
}

export interface DailyAvailabilitySummary {
  date: string;
  baseMinutes: number;
  adjustedMinutes: number;
  blocked: boolean;
  reason: string | null;
}

export interface FreeNowRecommendation {
  date: string;
  minuteOfDay: number;
  availableNow: boolean;
  windowEndMinute: number | null;
  suggestedDurationMinutes: number;
  sessionType: string;
  rationale: string;
  focusTopicIds: number[];
  targetId: number | null;
  carryoverAttempts: number;
  carryoverCorrect: number;
}

export interface DailyReplan {
  date: string;
  availableNow: boolean;
  remainingCapacityMinutes: number;
  remainingTargetMinutes: number;
  recommendedSessionCount: number;
  nextSessionType: string;
  focusTopicIds: number[];
  targetId: number | null;
  rationale: string;
}

export interface BeatYesterdayProfile {
  studentId: number;
  subjectId: number;
  currentStage: string;
  currentMode: string;
  momentumScore: BasisPoints;
  strainScore: BasisPoints;
  readinessScore: BasisPoints;
  recoveryNeedScore: BasisPoints;
  streakDays: number;
}

export interface BeatYesterdayDailyTarget {
  id: number;
  studentId: number;
  subjectId: number;
  targetDate: string;
  stage: string;
  mode: string;
  targetAttempts: number;
  targetCorrect: number;
  targetAvgResponseTimeMs: number | null;
  warmStartMinutes: number;
  coreClimbMinutes: number;
  speedBurstMinutes: number;
  finishStrongMinutes: number;
  focusTopicIds: number[];
  rationale: unknown;
  status: string;
}

export interface BeatYesterdayDailySummary {
  id: number;
  targetId: number | null;
  studentId: number;
  subjectId: number;
  summaryDate: string;
  actualAttempts: number;
  actualCorrect: number;
  actualAvgResponseTimeMs: number | null;
  beatAttemptTarget: boolean;
  beatAccuracyTarget: boolean;
  beatPaceTarget: boolean;
  momentumScore: BasisPoints;
  strainScore: BasisPoints;
  recoveryModeTriggered: boolean;
  summary: unknown;
}

export interface ClimbTrendPoint {
  summaryDate: string;
  actualAttempts: number;
  actualCorrect: number;
  actualAvgResponseTimeMs: number | null;
  momentumScore: BasisPoints;
  strainScore: BasisPoints;
  recoveryModeTriggered: boolean;
}

export interface BeatYesterdayDashboard {
  profile: BeatYesterdayProfile;
  target: BeatYesterdayDailyTarget | null;
  latestSummary: BeatYesterdayDailySummary | null;
  previousSummary: BeatYesterdayDailySummary | null;
}
```

---

### 6.16 Intake Types (`src/types/intake.ts`)

```typescript
export interface SubmissionBundle {
  id: number;
  studentId: number;
  title: string;
  status: string;
}

export interface BundleFile {
  id: number;
  bundleId: number;
  fileName: string;
  filePath: string;
  mimeType: string | null;
  fileKind: string;
}

export interface ExtractedInsight {
  id: number;
  bundleId: number;
  insightType: string;
  payload: unknown;
  createdAt: string;
}

export interface BundleProcessReport {
  bundle: SubmissionBundle;
  files: BundleFile[];
  insights: ExtractedInsight[];
  detectedSubjects: string[];
  detectedExamYears: number[];
  questionLikeFileCount: number;
}

export interface ContentAcquisitionJob {
  id: number;
  subjectId: number | null;
  topicId: number | null;
  intentType: string;
  queryText: string;
  sourceScope: string;
  status: string;
  resultSummary: unknown;
}

export interface AcquisitionEvidenceCandidate {
  id: number;
  jobId: number;
  sourceLabel: string;
  sourceUrl: string | null;
  sourceKind: string;
  title: string | null;
  snippet: string | null;
  extractedPayload: unknown;
  qualityScore: number;
  freshnessScore: number;
  reviewStatus: string;
}

export interface AcquisitionJobReport {
  job: ContentAcquisitionJob;
  candidates: AcquisitionEvidenceCandidate[];
}
```

---

### 6.17 Premium Types (`src/types/premium.ts`)

```typescript
export interface RiskFlag {
  id: number;
  studentId: number;
  title: string;
  severity: string;
  description: string | null;
  status: string;
  createdAt: string;
}

export interface InterventionRecord {
  id: number;
  studentId: number;
  riskFlagId: number | null;
  title: string;
  status: string;
  createdAt: string;
}

export interface WeeklyMemo {
  id: number;
  studentId: number;
  audience: string;
  weekStart: string;
  memoBody: string;
  metadataJson: string;
}
```

---

### 6.18 Elite Types (`src/types/elite.ts`)

```typescript
import type { BasisPoints } from './substrate';

export interface EliteProfile {
  studentId: number;
  subjectId: number;
  epsScore: BasisPoints;
  tier: string;
  precisionScore: BasisPoints;
  speedScore: BasisPoints;
  depthScore: BasisPoints;
  composureScore: BasisPoints;
}

export interface EliteTopicProfile {
  topicId: number;
  topicName: string;
  precisionScore: BasisPoints;
  speedScore: BasisPoints;
  depthScore: BasisPoints;
  composureScore: BasisPoints;
  consistencyScore: BasisPoints;
  trapResistanceScore: BasisPoints;
  dominationScore: BasisPoints;
  status: string;
}

export interface EliteSessionScore {
  sessionId: number;
  studentId: number;
  subjectId: number;
  sessionClass: string;
  accuracyScore: BasisPoints;
  precisionScore: BasisPoints;
  speedScore: BasisPoints;
  depthScore: BasisPoints;
  trapResistanceScore: BasisPoints;
  composureScore: BasisPoints;
  consistencyScore: BasisPoints;
  epsScore: BasisPoints;
  sessionLabel: string;
  debriefText: string;
  recommendedNextSession: string;
  metadata: unknown;
}
```

---

### 6.19 Content / Pack Types (`src/types/content.ts`)

```typescript
import type { BasisPoints } from './substrate';

export interface PackManifest {
  packId: string;
  packVersion: string;
  subjectCode: string;
  curriculumVersion: string;
  examTarget: string | null;
  gradeLevels: string[];
  topicCount: number;
  questionCount: number;
  minAppVersion: string | null;
  checksums: Record<string, string>;
  createdAt: string | null;
  author: string | null;
}

export interface PackInstallResult {
  packId: string;
  packVersion: string;
  installPath: string;
}

export interface PackSummary {
  packId: string;
  packVersion: string;
  subjectCode: string;
  status: string;
}

export interface TopicResourceReadiness {
  subjectId: number;
  topicId: number;
  topicName: string;
  nodeCount: number;
  objectiveCount: number;
  misconceptionCount: number;
  nodeEdgeCount: number;
  questionFamilyCount: number;
  questionCount: number;
  explanationCount: number;
  glossaryCount: number;
  formulaCount: number;
  workedExampleCount: number;
  readinessScore: BasisPoints;
  missingResources: string[];
  generationModes: string[];
}

export interface SubjectResourceReadiness {
  subjectId: number;
  subjectCode: string;
  subjectName: string;
  averageReadinessScore: BasisPoints;
  strongTopicCount: number;
  thinTopicCount: number;
  missingResourceTopics: number;
  topics: TopicResourceReadiness[];
}

export interface ContentTypeStrategy {
  nodeType: string;
  strategyFamilies: string[];
  drillFamilies: string[];
  failureModes: string[];
  masteryEvidence: string[];
  reviewMode: string;
  timeSensitivity: string;
}

export interface ContentStrategyRegistry {
  strategies: ContentTypeStrategy[];
}
```

---

### 6.20 Memory Types (`src/types/memory.ts`)

```typescript
import type { BasisPoints } from './substrate';

export type MemoryState =
  | 'new'
  | 'learning'
  | 'young'
  | 'mature'
  | 'stable'
  | 'fading'
  | 'forgotten';

export interface MemoryStateRecord {
  id: number;
  studentId: number;
  topicId: number | null;
  nodeId: number | null;
  memoryState: string;
  memoryStrength: BasisPoints;
  recallFluency: BasisPoints;
  decayRisk: BasisPoints;
  repetitionCount: number;
  intervalDays: number;
  easeFactor: number;
  reviewDueAt: string | null;
  lastReviewedAt: string | null;
}

export interface MemoryReviewItem {
  memoryStateId: number;
  topicId: number | null;
  topicName: string | null;
  nodeId: number | null;
  nodeTitle: string | null;
  memoryStrength: BasisPoints;
  decayRisk: BasisPoints;
  reviewDueAt: string;
  overdueByDays: number;
}

export interface MemoryReviewResult {
  memoryStateId: number;
  recalled: boolean;
  responseTimeMs: number;
  confidenceLevel: string;
}
```

---

### 6.21 Time Orchestration Types (`src/types/time-orchestration.ts`)

```typescript
// Time orchestration types are shared with goals-calendar.
// Re-exported here for domain clarity.

export type {
  AvailabilityProfile,
  AvailabilityWindow,
  AvailabilityException,
  DailyAvailabilitySummary,
  FreeNowRecommendation,
  DailyReplan,
  BeatYesterdayProfile,
  BeatYesterdayDailyTarget,
  BeatYesterdayDailySummary,
  ClimbTrendPoint,
  BeatYesterdayDashboard,
} from './goals-calendar';

export interface TimeSlot {
  startMinute: number;
  endMinute: number;
  durationMinutes: number;
  isPreferred: boolean;
  sessionType: string | null;
}

export interface DaySchedule {
  date: string;
  slots: TimeSlot[];
  totalAvailableMinutes: number;
  totalPlannedMinutes: number;
  isBlocked: boolean;
}
```

---

### 6.22 Frontend-Only UI Types (`src/types/ui.ts`)

```typescript
// ─── Theme ───────────────────────────────────────
export type ThemeMode = 'light' | 'dark' | 'system';

export interface ThemeColors {
  primary: string;
  secondary: string;
  accent: string;
  background: string;
  surface: string;
  text: string;
  textSecondary: string;
  border: string;
  error: string;
  warning: string;
  success: string;
  info: string;
}

// ─── Sidebar ─────────────────────────────────────
export type SidebarState = 'expanded' | 'collapsed' | 'hidden';

// ─── Modals ──────────────────────────────────────
export interface ModalState {
  isOpen: boolean;
  component: string | null;
  props: Record<string, unknown>;
  onClose: (() => void) | null;
  persistent: boolean;
}

// ─── Toast / Notifications ───────────────────────
export type ToastSeverity = 'info' | 'success' | 'warning' | 'error';

export interface ToastMessage {
  id: string;
  severity: ToastSeverity;
  title: string;
  message: string;
  durationMs: number;
  dismissible: boolean;
  action: ToastAction | null;
}

export interface ToastAction {
  label: string;
  handler: () => void;
}

// ─── Navigation ──────────────────────────────────
export interface NavItem {
  key: string;
  label: string;
  icon: string;
  route: string;
  badge: string | number | null;
  children: NavItem[];
  requiredRole: string | null;
  requiredTier: string | null;
  disabled: boolean;
}

export interface Breadcrumb {
  label: string;
  route: string | null;
  icon: string | null;
}

export interface TabItem {
  key: string;
  label: string;
  icon: string | null;
  badge: string | number | null;
  disabled: boolean;
}

// ─── Command Palette ─────────────────────────────
export interface CommandPaletteItem {
  id: string;
  label: string;
  description: string | null;
  icon: string | null;
  shortcut: string | null;
  action: () => void;
  group: string;
}

// ─── Data Tables ─────────────────────────────────
export type SortDirection = 'asc' | 'desc';

export interface TableColumn<T = unknown> {
  key: string;
  label: string;
  sortable: boolean;
  width: string | null;
  align: 'left' | 'center' | 'right';
  render: ((value: unknown, row: T) => string) | null;
}

export interface PaginationState {
  page: number;
  pageSize: number;
  totalItems: number;
  totalPages: number;
}

// ─── Forms ───────────────────────────────────────
export interface FormFieldState {
  value: unknown;
  error: string | null;
  touched: boolean;
  dirty: boolean;
  validating: boolean;
}

export interface FormState {
  fields: Record<string, FormFieldState>;
  isValid: boolean;
  isSubmitting: boolean;
  submitError: string | null;
}

export type ValidationRule = {
  type: 'required' | 'min' | 'max' | 'minLength' | 'maxLength' | 'pattern' | 'custom';
  value?: unknown;
  message: string;
  validator?: (value: unknown) => boolean;
};

// ─── Loading / Empty / Error States ──────────────
export type AsyncStatus = 'idle' | 'loading' | 'success' | 'error';

export interface AsyncState<T> {
  status: AsyncStatus;
  data: T | null;
  error: string | null;
}

// ─── Drag and Drop ──────────────────────────────
export interface DragItem {
  id: string;
  type: string;
  data: unknown;
}

export interface DropZone {
  id: string;
  accepts: string[];
  isOver: boolean;
}

// ─── Charts ──────────────────────────────────────
export interface ChartDataPoint {
  label: string;
  value: number;
  color: string | null;
  metadata: Record<string, unknown>;
}

export interface ChartSeries {
  name: string;
  data: ChartDataPoint[];
  color: string;
  type: 'line' | 'bar' | 'area' | 'radar';
}

// ─── Session UI State ────────────────────────────
export interface QuestionUIState {
  questionId: number;
  selectedOptionId: number | null;
  isConfirmed: boolean;
  isFlagged: boolean;
  startedAt: number; // timestamp ms
  hintCount: number;
  changedAnswerCount: number;
  timeRemainingMs: number | null;
}

export interface SessionUIState {
  currentIndex: number;
  totalQuestions: number;
  questions: QuestionUIState[];
  isReviewMode: boolean;
  isPaused: boolean;
  elapsedMs: number;
  timeLimitMs: number | null;
}

// ─── Onboarding UI ──────────────────────────────
export interface OnboardingStep {
  key: string;
  title: string;
  description: string;
  completed: boolean;
  active: boolean;
  icon: string;
}

export interface OnboardingState {
  currentStep: number;
  steps: OnboardingStep[];
  formData: Record<string, unknown>;
}
```

---

### 6.23 Barrel Export (`src/types/index.ts`)

```typescript
export * from './substrate';
export * from './identity';
export * from './curriculum';
export * from './questions';
export * from './student-model';
export * from './sessions';
export * from './coach';
export * from './diagnostics';
export * from './reporting';
export * from './glossary';
export * from './library';
export * from './games';
export * from './traps';
export * from './past-papers';
export * from './goals-calendar';
export * from './intake';
export * from './premium';
export * from './elite';
export * from './content';
export * from './memory';
export * from './time-orchestration';
export * from './ui';
```

---

---

## SECTION 7: COMPLETE TAURI IPC COMMAND BINDINGS

Every command below represents a `#[tauri::command]` function the frontend invokes via `invoke()`. Commands are organized by crate. Each entry specifies:

- **Command name**: The exact string passed to `invoke()`
- **Parameters**: Typed inputs
- **Return type**: The typed result
- **Used by**: Which pages/components consume this command
- **Status**: `implemented` (backend service exists) or `needs backend` (frontend needs it, backend stub needed)

All commands return `Promise<T>` and throw on error (Tauri converts Rust `Err` to rejected promise). The frontend IPC layer wraps each in a composable.

---

### 7.1 Identity Commands (`src/ipc/identity.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `create_account` | `input: CreateAccountInput` | `Account` | Onboarding, Admin user creation | implemented |
| 2 | `authenticate` | `accountId: number, pin: string` | `Account` | Login screen, PIN entry | implemented |
| 3 | `list_accounts` | (none) | `AccountSummary[]` | Login screen account picker, Admin user list | implemented |
| 4 | `get_account` | `accountId: number` | `Account \| null` | Settings, Profile page | implemented |
| 5 | `update_account` | `accountId: number, input: UpdateAccountInput` | `Account` | Settings page | needs backend |
| 6 | `change_pin` | `accountId: number, input: ChangePinInput` | `void` | Settings > Security | needs backend |
| 7 | `delete_account` | `accountId: number` | `void` | Admin user management | needs backend |
| 8 | `link_parent_student` | `parentId: number, studentId: number` | `void` | Parent onboarding, Admin | needs backend |
| 9 | `unlink_parent_student` | `parentId: number, studentId: number` | `void` | Parent settings | needs backend |
| 10 | `list_linked_students` | `parentId: number` | `AccountSummary[]` | Parent dashboard | needs backend |
| 11 | `mark_first_run_complete` | `accountId: number` | `void` | Onboarding completion | needs backend |
| 12 | `update_student_profile` | `accountId: number, preferredSubjects: string[], examTarget: string, dailyBudgetMinutes: number` | `void` | Onboarding, Settings | needs backend |
| 13 | `get_student_profile` | `accountId: number` | `StudentProfile` | Coach, Settings | needs backend |

---

### 7.2 Curriculum Commands (`src/ipc/curriculum.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `get_subjects` | `curriculumVersionId: number` | `Subject[]` | Subject selector, Onboarding, Dashboard | implemented |
| 2 | `list_topics_for_subject` | `subjectId: number` | `TopicSummary[]` | Topic browser, Session setup, Coach | implemented |
| 3 | `get_topic` | `topicId: number` | `TopicSummary \| null` | Topic detail page | implemented |
| 4 | `list_academic_nodes` | `topicId: number` | `AcademicNode[]` | Topic detail, Glossary, Coach repair | implemented |
| 5 | `get_curriculum_versions` | (none) | `CurriculumVersion[]` | Admin settings | needs backend |
| 6 | `get_active_curriculum_version` | (none) | `CurriculumVersion` | Global app init | needs backend |
| 7 | `upload_curriculum_source` | `filePath: string, metadata: Record<string, unknown>` | `CurriculumSourceUpload` | Admin curriculum import | implemented |
| 8 | `get_curriculum_source_report` | `sourceUploadId: number` | `CurriculumSourceReport` | Admin curriculum review | implemented |
| 9 | `approve_parse_candidate` | `candidateId: number` | `void` | Admin curriculum review | needs backend |
| 10 | `reject_parse_candidate` | `candidateId: number, reason: string` | `void` | Admin curriculum review | needs backend |
| 11 | `search_topics` | `query: string, subjectId?: number` | `TopicSummary[]` | Global search, Command palette | needs backend |

---

### 7.3 Questions Commands (`src/ipc/questions.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `get_question` | `questionId: number` | `Question \| null` | Session runtime, Review screen | implemented |
| 2 | `get_question_options` | `questionId: number` | `QuestionOption[]` | Session runtime, Review screen | implemented |
| 3 | `get_question_profile` | `questionId: number` | `QuestionIntelligenceProfile \| null` | Question detail modal, Admin | implemented |
| 4 | `select_questions` | `request: QuestionSelectionRequest` | `SelectedQuestion[]` | Session setup (preview), Admin testing | implemented |
| 5 | `find_questions_by_intelligence` | `query: QuestionIntelligenceQuery` | `Question[]` | Admin question explorer | implemented |
| 6 | `list_intelligence_links` | `questionId: number` | `QuestionIntelligenceLink[]` | Question detail, Coach repair | implemented |
| 7 | `get_question_family` | `familyId: number` | `QuestionFamily \| null` | Past papers, Question variants | needs backend |
| 8 | `list_questions_for_topic` | `topicId: number` | `Question[]` | Topic detail, Admin | needs backend |
| 9 | `get_question_with_options` | `questionId: number` | `{ question: Question, options: QuestionOption[] }` | Session runtime (combined call) | needs backend |
| 10 | `list_question_families_for_topic` | `topicId: number` | `QuestionFamily[]` | Past papers analysis | needs backend |

---

### 7.4 Student Model Commands (`src/ipc/student-model.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `process_answer` | `studentId: number, submission: AnswerSubmission` | `AnswerProcessingResult` | Session runtime, Diagnostic | implemented |
| 2 | `get_learner_truth_snapshot` | `studentId: number` | `LearnerTruthSnapshot` | Dashboard, Coach, Parent view | implemented |
| 3 | `get_topic_state` | `studentId: number, topicId: number` | `StudentTopicState \| null` | Topic detail, Coach repair | implemented |
| 4 | `list_topic_states` | `studentId: number` | `StudentTopicState[]` | Dashboard subject view, Heat map | implemented |
| 5 | `list_topic_states_for_subject` | `studentId: number, subjectId: number` | `StudentTopicState[]` | Subject detail page | needs backend |
| 6 | `get_error_profile` | `studentId: number, topicId: number` | `ErrorProfile \| null` | Coach repair, Topic case | needs backend |
| 7 | `list_weak_topics` | `studentId: number, threshold: BasisPoints` | `StudentTopicState[]` | Coach priority list, Dashboard alerts | needs backend |
| 8 | `list_due_memory_reviews` | `studentId: number` | `MemoryReviewItem[]` | Library, Dashboard, Coach mission | needs backend |
| 9 | `submit_memory_review` | `studentId: number, result: MemoryReviewResult` | `void` | Memory review session | needs backend |
| 10 | `get_student_skill_states` | `studentId: number, topicId: number` | `LearnerTruthSkillSummary[]` | Skill breakdown view | needs backend |
| 11 | `reset_topic_state` | `studentId: number, topicId: number` | `void` | Admin tools (dev only) | needs backend |

---

### 7.5 Sessions Commands (`src/ipc/sessions.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `start_practice_session` | `input: PracticeSessionStartInput` | `SessionSnapshot` | Practice mode launcher | implemented |
| 2 | `start_custom_test` | `input: CustomTestStartInput` | `SessionSnapshot` | Custom test builder | implemented |
| 3 | `get_session_snapshot` | `sessionId: number` | `SessionSnapshot` | Session resume, Review | implemented |
| 4 | `submit_session_answer` | `sessionId: number, input: SessionAnswerInput` | `AnswerProcessingResult` | Session runtime | implemented |
| 5 | `flag_session_item` | `sessionId: number, itemId: number, flagged: boolean` | `void` | Session runtime (flag toggle) | implemented |
| 6 | `get_session_summary` | `sessionId: number` | `SessionSummary` | Post-session results | implemented |
| 7 | `pause_session` | `sessionId: number` | `void` | Session runtime (pause button) | implemented |
| 8 | `resume_session` | `sessionId: number` | `SessionSnapshot` | Session resume | implemented |
| 9 | `abandon_session` | `sessionId: number` | `void` | Session runtime (exit) | implemented |
| 10 | `complete_session` | `sessionId: number` | `SessionSummary` | Session runtime (finish) | implemented |
| 11 | `list_sessions` | `studentId: number, limit?: number` | `Session[]` | History page, Dashboard recent | needs backend |
| 12 | `list_sessions_for_subject` | `studentId: number, subjectId: number` | `Session[]` | Subject history | needs backend |
| 13 | `get_session_review_data` | `sessionId: number` | `SessionSnapshot & { results: AnswerProcessingResult[] }` | Post-session detailed review | needs backend |

---

### 7.6 Coach Brain Commands (`src/ipc/coach.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `resolve_coach_state` | `studentId: number` | `CoachStateResolution` | Coach dashboard, App init | implemented |
| 2 | `resolve_next_coach_action` | `studentId: number` | `CoachNextAction` | Coach dashboard hero card | implemented |
| 3 | `assess_content_readiness` | `studentId: number` | `ContentReadinessResolution` | Coach content gate, Onboarding | implemented |
| 4 | `build_topic_case` | `studentId: number, topicId: number` | `TopicCase` | Topic case detail, Coach repair | implemented |
| 5 | `list_priority_topic_cases` | `studentId: number, limit: number` | `TopicCase[]` | Coach priority list | implemented |
| 6 | `generate_plan` | `studentId: number, examTarget: string, examDate: string, dailyBudgetMinutes: number` | `number (planId)` | Coach plan generation | implemented |
| 7 | `generate_today_mission` | `studentId: number` | `number (missionId)` | Coach daily mission | implemented |
| 8 | `start_mission` | `missionId: number` | `void` | Coach mission start | implemented |
| 9 | `complete_mission` | `missionId: number, sessionId: number` | `CoachMissionMemory` | Mission completion | implemented |
| 10 | `get_mission_memory` | `memoryId: number` | `CoachMissionMemory` | Mission review page | implemented |
| 11 | `review_mission_memory` | `memoryId: number` | `void` | Mission review confirm | implemented |
| 12 | `get_active_plan` | `studentId: number` | `CoachPlan \| null` | Coach plan view | needs backend |
| 13 | `list_plan_days` | `planId: number` | `CoachPlanDay[]` | Coach calendar view | needs backend |
| 14 | `list_missions_for_day` | `planDayId: number` | `CoachMission[]` | Coach day detail | needs backend |
| 15 | `refresh_plan` | `studentId: number` | `number (planId)` | Coach plan refresh | needs backend |
| 16 | `list_mission_memories` | `studentId: number, limit: number` | `CoachMissionMemory[]` | Mission history | needs backend |
| 17 | `defer_mission` | `missionId: number` | `void` | Mission defer button | needs backend |

---

### 7.7 Diagnostics Commands (`src/ipc/diagnostics.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `start_diagnostic` | `studentId: number, subjectId: number, mode: DiagnosticMode` | `number (diagnosticId)` | Diagnostic launcher | implemented |
| 2 | `start_diagnostic_battery` | `studentId: number, subjectId: number, topicIds: number[], mode: DiagnosticMode` | `DiagnosticBattery` | Diagnostic with topic selection | implemented |
| 3 | `get_diagnostic_battery` | `diagnosticId: number` | `DiagnosticBattery` | Diagnostic session view | implemented |
| 4 | `get_diagnostic_phase_items` | `phaseId: number` | `DiagnosticPhaseItem[]` | Diagnostic phase runtime | implemented |
| 5 | `advance_diagnostic_phase` | `diagnosticId: number, phaseId: number` | `DiagnosticPhasePlan \| null` | Diagnostic phase transition | implemented |
| 6 | `complete_diagnostic` | `diagnosticId: number` | `DiagnosticResult` | Diagnostic completion | implemented |
| 7 | `get_diagnostic_result` | `diagnosticId: number` | `DiagnosticResult` | Diagnostic results page | implemented |
| 8 | `list_wrong_answer_diagnoses` | `studentId: number, topicId?: number` | `WrongAnswerDiagnosis[]` | Coach repair, Topic case | implemented |
| 9 | `list_diagnostics` | `studentId: number` | `DiagnosticBattery[]` | Diagnostic history | needs backend |
| 10 | `get_topic_diagnostic_results` | `studentId: number, subjectId: number` | `TopicDiagnosticResult[]` | Subject diagnostic overview | needs backend |

---

### 7.8 Reporting Commands (`src/ipc/reporting.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `get_student_dashboard` | `studentId: number` | `StudentDashboard` | Student home dashboard | implemented |
| 2 | `build_parent_dashboard` | `parentId: number` | `ParentDashboardSnapshot` | Parent dashboard | implemented |
| 3 | `get_subject_summary` | `studentId: number, subjectId: number` | `SubjectSummary` | Subject overview card | needs backend |
| 4 | `export_dashboard_pdf` | `studentId: number` | `string (filePath)` | Dashboard export button | needs backend |
| 5 | `get_weekly_memo` | `studentId: number, weekStart: string` | `WeeklyMemo \| null` | Parent weekly memo view | needs backend |
| 6 | `list_weekly_memos` | `studentId: number, limit: number` | `WeeklyMemo[]` | Parent memo history | needs backend |

---

### 7.9 Glossary Commands (`src/ipc/glossary.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `search_glossary_entries` | `query: string` | `KnowledgeEntry[]` | Glossary search page | implemented |
| 2 | `get_glossary_entry` | `entryId: number` | `KnowledgeEntry \| null` | Glossary detail | needs backend |
| 3 | `create_glossary_entry` | `title: string, entryType: string, shortText?: string, topicId?: number` | `number (entryId)` | Admin glossary management | implemented |
| 4 | `create_knowledge_bundle` | `title: string, bundleType: string, topicId?: number` | `number (bundleId)` | Admin bundle creation | implemented |
| 5 | `list_bundles_for_topic` | `topicId: number` | `KnowledgeBundle[]` | Topic glossary tab | implemented |
| 6 | `get_question_knowledge_links` | `questionId: number` | `QuestionKnowledgeLink[]` | Question detail, Post-answer | implemented |
| 7 | `list_entries_for_topic` | `topicId: number` | `KnowledgeEntry[]` | Topic glossary tab | needs backend |
| 8 | `list_entries_for_bundle` | `bundleId: number` | `KnowledgeEntry[]` | Bundle detail view | needs backend |

---

### 7.10 Library Commands (`src/ipc/library.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `save_library_item` | `studentId: number, itemType: string, itemRefId: number` | `number (itemId)` | Save button on questions/concepts | implemented |
| 2 | `save_library_item_with_metadata` | `studentId: number, input: SaveLibraryItemInput` | `number (itemId)` | Save with notes/tags | implemented |
| 3 | `update_library_item_metadata` | `itemId: number, state: string, tags: string[], noteText?: string, urgencyScore: BasisPoints` | `void` | Library item editor | implemented |
| 4 | `remove_library_item` | `itemId: number` | `void` | Library item remove | implemented |
| 5 | `list_saved_questions` | `studentId: number` | `SavedQuestionCard[]` | Library saved tab | implemented |
| 6 | `get_library_home` | `studentId: number` | `LibraryHomeSnapshot` | Library home page | implemented |
| 7 | `generate_revision_pack` | `studentId: number, topicIds: number[], title: string` | `RevisionPackSummary` | Library revision pack builder | implemented |
| 8 | `list_revision_packs` | `studentId: number` | `RevisionPackSummary[]` | Library revision packs tab | implemented |
| 9 | `get_revision_pack_items` | `packId: number` | `RevisionPackItem[]` | Revision pack detail | implemented |
| 10 | `get_continue_learning_card` | `studentId: number` | `ContinueLearningCard \| null` | Library home hero | needs backend |

---

### 7.11 Games Commands (`src/ipc/games.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `start_game_session` | `studentId: number, gameType: string` | `number (sessionId)` | Game launcher | implemented |
| 2 | `get_mind_stack_state` | `sessionId: number` | `MindStackState` | MindStack game view | needs backend |
| 3 | `submit_mind_stack_move` | `sessionId: number, conceptId: number, position: number` | `MindStackState` | MindStack game interaction | needs backend |
| 4 | `get_tug_of_war_state` | `sessionId: number` | `TugOfWarState` | TugOfWar game view | needs backend |
| 5 | `submit_tug_of_war_placement` | `sessionId: number, itemId: number, side: string` | `TugOfWarState` | TugOfWar game interaction | needs backend |
| 6 | `complete_game_session` | `sessionId: number` | `GameResult` | Game completion | needs backend |
| 7 | `list_game_sessions` | `studentId: number, gameType?: string` | `GameSession[]` | Game history | needs backend |
| 8 | `get_game_result` | `sessionId: number` | `GameResult` | Post-game results | needs backend |

---

### 7.12 Traps Commands (`src/ipc/traps.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `list_trap_patterns` | `topicId: number` | `TrapPattern[]` | Trap explorer page | needs backend |
| 2 | `get_student_trap_profile` | `studentId: number, topicId: number` | `StudentTrapProfile[]` | Trap resistance view | needs backend |
| 3 | `start_trap_drill` | `studentId: number, topicId: number, trapIds: number[]` | `TrapDrillSession` | Trap drill launcher | needs backend |
| 4 | `submit_trap_drill_answer` | `sessionId: number, questionId: number, selectedOptionId: number` | `AnswerProcessingResult` | Trap drill runtime | needs backend |
| 5 | `complete_trap_drill` | `sessionId: number` | `GameResult` | Trap drill completion | needs backend |

---

### 7.13 Past Papers Commands (`src/ipc/past-papers.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `create_paper_set` | `subjectId: number, examYear: number, title: string` | `number (paperId)` | Admin past paper management | implemented |
| 2 | `get_paper_set` | `paperId: number` | `PastPaperSet \| null` | Past paper detail | implemented |
| 3 | `list_paper_sets` | `subjectId: number` | `PastPaperSetSummary[]` | Past papers browser | implemented |
| 4 | `link_question_to_paper` | `paperId: number, questionId: number, sectionLabel?: string, questionNumber?: string` | `number (linkId)` | Admin paper-question linking | implemented |
| 5 | `get_family_analytics` | `subjectId: number` | `PastPaperFamilyAnalytics[]` | Past paper analytics view | implemented |
| 6 | `list_questions_for_paper` | `paperId: number` | `Question[]` | Past paper question list | needs backend |
| 7 | `start_past_paper_session` | `studentId: number, paperId: number, timed: boolean` | `SessionSnapshot` | Past paper practice launcher | needs backend |
| 8 | `get_paper_performance` | `studentId: number, paperId: number` | `SessionSummary` | Past paper results | needs backend |

---

### 7.14 Goals & Calendar Commands (`src/ipc/goals-calendar.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `create_goal` | `studentId: number, goalType: string, title: string, description?: string` | `number (goalId)` | Goal creation page | implemented |
| 2 | `list_goals` | `studentId: number` | `Goal[]` | Goals overview | implemented |
| 3 | `update_goal_status` | `goalId: number, status: string` | `void` | Goal status toggle | implemented |
| 4 | `create_calendar_event` | `studentId: number, eventType: string, title: string, scheduledFor: string` | `number (eventId)` | Calendar event creation | implemented |
| 5 | `list_calendar_events` | `studentId: number, fromDate: string, toDate: string` | `CalendarEvent[]` | Calendar view | implemented |
| 6 | `get_availability_profile` | `studentId: number` | `AvailabilityProfile` | Availability settings | implemented |
| 7 | `update_availability_profile` | `studentId: number, profile: AvailabilityProfile` | `void` | Availability settings | implemented |
| 8 | `list_availability_windows` | `studentId: number` | `AvailabilityWindow[]` | Weekly schedule view | implemented |
| 9 | `set_availability_windows` | `studentId: number, windows: AvailabilityWindow[]` | `void` | Weekly schedule editor | implemented |
| 10 | `add_availability_exception` | `studentId: number, exception: AvailabilityException` | `void` | Calendar exception | implemented |
| 11 | `get_daily_availability` | `studentId: number, date: string` | `DailyAvailabilitySummary` | Day detail view | implemented |
| 12 | `get_free_now_recommendation` | `studentId: number` | `FreeNowRecommendation` | "Study Now" button | implemented |
| 13 | `get_daily_replan` | `studentId: number, date: string` | `DailyReplan` | Daily replan view | implemented |
| 14 | `get_beat_yesterday_dashboard` | `studentId: number, subjectId: number` | `BeatYesterdayDashboard` | BeatYesterday page | implemented |
| 15 | `get_beat_yesterday_target` | `studentId: number, subjectId: number, date: string` | `BeatYesterdayDailyTarget \| null` | BeatYesterday daily view | implemented |
| 16 | `list_climb_trend` | `studentId: number, subjectId: number, days: number` | `ClimbTrendPoint[]` | BeatYesterday trend chart | implemented |
| 17 | `delete_goal` | `goalId: number` | `void` | Goal management | needs backend |
| 18 | `delete_calendar_event` | `eventId: number` | `void` | Calendar event management | needs backend |

---

### 7.15 Intake Commands (`src/ipc/intake.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `create_intake_bundle` | `studentId: number, title: string` | `number (bundleId)` | Upload flow start | implemented |
| 2 | `add_bundle_file` | `bundleId: number, fileName: string, filePath: string` | `number (fileId)` | File upload step | implemented |
| 3 | `list_bundle_files` | `bundleId: number` | `BundleFile[]` | Bundle file list | implemented |
| 4 | `process_bundle` | `bundleId: number` | `BundleProcessReport` | Bundle processing trigger | implemented |
| 5 | `get_bundle_report` | `bundleId: number` | `BundleProcessReport` | Bundle results view | implemented |
| 6 | `create_acquisition_job` | `subjectId?: number, topicId?: number, intentType: string, queryText: string` | `number (jobId)` | Content acquisition trigger | implemented |
| 7 | `get_acquisition_report` | `jobId: number` | `AcquisitionJobReport` | Acquisition results | implemented |
| 8 | `list_bundles` | `studentId: number` | `SubmissionBundle[]` | Upload history | needs backend |
| 9 | `approve_acquisition_candidate` | `candidateId: number` | `void` | Admin acquisition review | needs backend |
| 10 | `reject_acquisition_candidate` | `candidateId: number` | `void` | Admin acquisition review | needs backend |

---

### 7.16 Premium Commands (`src/ipc/premium.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `create_risk_flag` | `studentId: number, title: string, severity: string` | `number (flagId)` | Auto-generated by reporting, Admin manual | implemented |
| 2 | `create_intervention` | `studentId: number, title: string, riskFlagId?: number` | `number (interventionId)` | Parent/Coach intervention | implemented |
| 3 | `list_risk_flags` | `studentId: number` | `RiskFlag[]` | Risk flags panel | needs backend |
| 4 | `update_risk_flag_status` | `flagId: number, status: string` | `void` | Risk flag management | needs backend |
| 5 | `list_interventions` | `studentId: number` | `InterventionRecord[]` | Intervention history | needs backend |
| 6 | `complete_intervention` | `interventionId: number` | `void` | Intervention completion | needs backend |

---

### 7.17 Elite Commands (`src/ipc/elite.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `get_elite_profile` | `studentId: number, subjectId: number` | `EliteProfile \| null` | Elite dashboard | implemented |
| 2 | `score_elite_session` | `studentId: number, sessionId: number, sessionClass: string` | `EliteSessionScore` | Post-session elite scoring | implemented |
| 3 | `list_elite_topic_profiles` | `studentId: number, subjectId: number` | `EliteTopicProfile[]` | Elite topic breakdown | implemented |
| 4 | `list_elite_session_scores` | `studentId: number, subjectId: number, limit: number` | `EliteSessionScore[]` | Elite session history | needs backend |
| 5 | `get_elite_leaderboard` | `subjectId: number` | `EliteProfile[]` | Elite leaderboard | needs backend |
| 6 | `start_elite_challenge` | `studentId: number, subjectId: number, challengeType: string` | `SessionSnapshot` | Elite challenge launcher | needs backend |

---

### 7.18 Content / Pack Commands (`src/ipc/content.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `install_content_pack` | `packPath: string` | `PackInstallResult` | Content pack install flow | implemented |
| 2 | `list_installed_packs` | (none) | `PackSummary[]` | Content management page | implemented |
| 3 | `get_pack_manifest` | `packPath: string` | `PackManifest` | Pack preview before install | implemented |
| 4 | `uninstall_pack` | `packId: string` | `void` | Content management | needs backend |
| 5 | `get_topic_resource_readiness` | `topicId: number` | `TopicResourceReadiness \| null` | Topic detail, Coach | implemented |
| 6 | `get_subject_resource_readiness` | `subjectId: number` | `SubjectResourceReadiness` | Subject readiness overview | implemented |
| 7 | `get_content_strategy_registry` | (none) | `ContentStrategyRegistry` | Admin content strategy view | implemented |
| 8 | `validate_pack` | `packPath: string` | `{ valid: boolean, errors: string[] }` | Pack validation before install | needs backend |
| 9 | `export_pack` | `subjectId: number, outputPath: string` | `string (filePath)` | Admin pack export | needs backend |

---

### 7.19 Memory Commands (`src/ipc/memory.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `list_due_reviews` | `studentId: number` | `MemoryReviewItem[]` | Memory review page, Dashboard badge | needs backend |
| 2 | `get_memory_state` | `studentId: number, topicId?: number, nodeId?: number` | `MemoryStateRecord \| null` | Topic detail memory tab | needs backend |
| 3 | `submit_review_result` | `studentId: number, result: MemoryReviewResult` | `MemoryStateRecord` | Memory review session | needs backend |
| 4 | `list_fading_concepts` | `studentId: number, limit: number` | `MemoryReviewItem[]` | Library fading concepts shelf | needs backend |
| 5 | `get_memory_calendar` | `studentId: number, fromDate: string, toDate: string` | `{ date: string, dueCount: number }[]` | Calendar memory overlay | needs backend |

---

### 7.20 Time Orchestration Commands (`src/ipc/time-orchestration.ts`)

These commands are implemented within the goals-calendar crate but are listed separately for domain clarity.

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `get_day_schedule` | `studentId: number, date: string` | `DaySchedule` | Daily schedule view | needs backend |
| 2 | `get_week_schedule` | `studentId: number, weekStart: string` | `DaySchedule[]` | Weekly schedule view | needs backend |
| 3 | `rebalance_day` | `studentId: number, date: string` | `DailyReplan` | Manual rebalance trigger | needs backend |
| 4 | `snooze_session` | `studentId: number, minutes: number` | `FreeNowRecommendation` | Snooze study reminder | needs backend |

---

### 7.21 System / Storage Commands (`src/ipc/system.ts`)

| # | Command | Parameters | Return | Used By | Status |
|---|---------|-----------|--------|---------|--------|
| 1 | `get_app_version` | (none) | `string` | Settings, About | needs backend |
| 2 | `get_database_stats` | (none) | `{ tableCount: number, sizeBytes: number }` | Admin diagnostics | needs backend |
| 3 | `export_database` | `outputPath: string` | `string (filePath)` | Admin backup | needs backend |
| 4 | `import_database` | `filePath: string` | `void` | Admin restore | needs backend |
| 5 | `get_engine_registry` | (none) | `EngineRegistry` | Admin engine overview | implemented |
| 6 | `get_threshold_registry` | (none) | `ThresholdRegistry` | Admin config view | implemented |
| 7 | `open_file_dialog` | `filters?: { name: string, extensions: string[] }[]` | `string \| null` | File picker | needs backend |
| 8 | `open_folder_dialog` | (none) | `string \| null` | Folder picker | needs backend |
| 9 | `get_runtime_events` | `aggregateId?: string, limit: number` | `DomainEvent[]` | Admin event log | needs backend |

---

### 7.22 IPC Composable Layer (`src/composables/useIpc.ts`)

All commands above are wrapped in a typed composable that:

1. Calls `invoke<T>(commandName, params)` from `@tauri-apps/api/core`
2. Catches errors and maps them to `EcoachError`
3. Provides loading state via `AsyncState<T>`
4. Caches results where appropriate (curriculum data, glossary entries)

```typescript
// Pattern for every IPC composable:
export function useIdentity() {
  const createAccount = (input: CreateAccountInput) =>
    invoke<Account>('create_account', { input });

  const authenticate = (accountId: number, pin: string) =>
    invoke<Account>('authenticate', { accountId, pin });

  const listAccounts = () =>
    invoke<AccountSummary[]>('list_accounts');

  // ... all identity commands

  return { createAccount, authenticate, listAccounts /* ... */ };
}
```

Each domain gets its own composable:
- `useIdentity()` -- 13 commands
- `useCurriculum()` -- 11 commands
- `useQuestions()` -- 10 commands
- `useStudentModel()` -- 11 commands
- `useSessions()` -- 13 commands
- `useCoach()` -- 17 commands
- `useDiagnostics()` -- 10 commands
- `useReporting()` -- 6 commands
- `useGlossary()` -- 8 commands
- `useLibrary()` -- 10 commands
- `useGames()` -- 8 commands
- `useTraps()` -- 5 commands
- `usePastPapers()` -- 8 commands
- `useGoalsCalendar()` -- 18 commands
- `useIntake()` -- 10 commands
- `usePremium()` -- 6 commands
- `useElite()` -- 6 commands
- `useContent()` -- 9 commands
- `useMemory()` -- 5 commands
- `useTimeOrchestration()` -- 4 commands
- `useSystem()` -- 9 commands

**Total: 187 IPC commands across 21 composables**

---

### 7.23 Command Summary by Implementation Status

| Status | Count | Percentage |
|--------|-------|------------|
| **Implemented** (backend service method exists) | 108 | 57.8% |
| **Needs backend** (frontend needs it, Rust stub/impl required) | 79 | 42.2% |
| **Total** | **187** | 100% |

The "needs backend" commands fall into these categories:
- **CRUD completions**: delete, update, list variants for existing entities (31 commands)
- **Aggregation queries**: combined/filtered views not yet exposed (18 commands)
- **New features**: games runtime, traps, memory reviews, time orchestration (22 commands)
- **System/admin**: backup, export, file dialogs (8 commands)

---

*End of Part 2: Types and IPC*
