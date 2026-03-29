# eCoach Frontend Implementation Plan
## Full Vision Build -- Academic Intelligence Desktop OS
## 2,183 Features | 3 Portals | Complete from Day 1

---

## PHILOSOPHY

This is NOT an MVP build. We are building the full product from day 1. Every component is built to its final specification. Every surface is built complete. No shortcuts that create debt.

The build order is driven by **architectural dependency**, not by "what ships fastest":
1. You cannot build screens without a complete design system
2. You cannot build screens without the full component library
3. You cannot build features without all TypeScript types and Tauri bindings
4. You cannot build any portal without all three portals' navigation in place
5. Every feature built is built COMPLETE -- all states, all edge cases, all roles

---

## CURRENT STATE

### What Exists (Backend)
- **21 Rust backend crates** with mature implementations
- **29 SQL migration files** covering the full domain model
- **Well-defined DTOs** across all domains
- **Coach Brain state machine** with 14 learner journey states
- **Content pack system** with math BECE sample data
- **No frontend code whatsoever**

### Tech Stack
- **Runtime**: Tauri 2.x (Rust backend + web frontend)
- **Frontend Framework**: Nuxt 3 (Vue 3 + TypeScript)
- **Styling**: TailwindCSS 4
- **State Management**: Pinia
- **Database**: SQLite (local, via Tauri IPC to Rust)
- **Rendering**: SPA mode (desktop app, no SSR)
- **Package Manager**: pnpm
- **Math Rendering**: KaTeX
- **Charts**: D3.js + custom SVG
- **Canvas/Game**: PixiJS (MindStack, Tug of War)
- **Audio**: Howler.js
- **Animation**: Motion One (Vue Motion)
- **PDF Export**: jsPDF + html2canvas
- **Icons**: Lucide Icons

---

## ARCHITECTURE

### Communication Model
```
┌─────────────────────────────────────────┐
│           Tauri Window (Desktop)         │
│  ┌───────────────────────────────────┐  │
│  │        Nuxt 3 SPA (Vue 3)        │  │
│  │                                   │  │
│  │  Pages → Components → Composables │  │
│  │              │                    │  │
│  │         Pinia Stores              │  │
│  │              │                    │  │
│  │      Tauri IPC (invoke)           │  │
│  └──────────────┼────────────────────┘  │
│                 │                        │
│  ┌──────────────▼────────────────────┐  │
│  │        Rust Backend (Tauri)       │  │
│  │                                   │  │
│  │  Commands → Services → SQLite     │  │
│  │  (21 crates, 29 migrations)       │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### Three Role-Based Shells
Not themes. Not conditional rendering. **Three completely separate application shells** with their own layouts, navigation, component density, typography, color temperature, and copy register.

```
app.vue
├── layouts/
│   ├── auth.vue          ← PIN entry / profile switcher (neutral)
│   ├── student.vue       ← Warm, engaging, coach-driven
│   ├── parent.vue        ← Clean, premium, insight-driven
│   └── admin.vue         ← Dense, operational, pipeline-driven
```

---

## COMPLETE PROJECT STRUCTURE

```
frontend/
├── nuxt.config.ts
├── tailwind.config.ts
├── app.vue
│
├── assets/
│   ├── css/
│   │   ├── main.css                     ← Tailwind base + design tokens
│   │   ├── themes/
│   │   │   ├── student.css              ← Student palette + overrides
│   │   │   ├── parent.css               ← Parent palette + overrides
│   │   │   ├── admin.css                ← Admin palette + overrides
│   │   │   └── modes/
│   │   │       ├── recovery.css         ← Warm amber, gentle
│   │   │       ├── pressure.css         ← Dark, intense
│   │   │       ├── elite.css            ← Dark premium
│   │   │       ├── game.css             ← Vibrant, energetic
│   │   │       ├── celebration.css      ← Bright, expansive
│   │   │       └── focus.css            ← Minimal, zen
│   │   └── print.css                    ← PDF/print export styles
│   ├── fonts/
│   │   ├── Inter/
│   │   └── JetBrainsMono/
│   ├── sounds/
│   │   ├── feedback/                    ← correct, wrong, streak, combo
│   │   ├── transitions/                 ← mode enter, phase change
│   │   ├── ambient/                     ← focus mode, exam hall
│   │   ├── game/                        ← mindstack, tugofwar, traps
│   │   └── celebration/                 ← milestone, level-up, mastery
│   └── images/
│       ├── mastery-icons/               ← 8 mastery state icons
│       ├── error-type-icons/            ← 10 error type icons
│       └── onboarding/
│
├── types/                               ← COMPLETE mirror of all 21 Rust crates
│   ├── index.ts                         ← Re-exports everything
│   ├── substrate.ts                     ← BasisPoints, Role, AccountType, EntitlementTier
│   ├── identity.ts                      ← Account, AccountSummary, CreateAccountInput
│   ├── curriculum.ts                    ← Subject, TopicSummary, AcademicNode, CurriculumVersion
│   ├── questions.ts                     ← Question, QuestionOption, QuestionFamily, QuestionIntelligence, QuestionSelectionRequest, SelectedQuestion
│   ├── student-model.ts                 ← AnswerSubmission, AnswerProcessingResult, ErrorType, MasteryState, StudentTopicState, LearnerTruthSnapshot, LearnerTruthTopicSummary, LearnerTruthSkillSummary, LearnerTruthMemorySummary, LearnerTruthDiagnosisSummary
│   ├── sessions.ts                      ← Session, SessionItem, SessionSnapshot, SessionSummary, PracticeSessionStartInput, CustomTestStartInput, SessionAnswerInput
│   ├── coach.ts                         ← LearnerJourneyState, CoachActionType, CoachNextAction, CoachStateResolution, ContentReadinessStatus, ContentReadinessResolution, TopicCase, TopicCaseBlocker, TopicCaseDiagnosis, TopicCaseHypothesis, TopicCaseIntervention, CoachMissionMemory
│   ├── diagnostics.ts                   ← DiagnosticMode, DiagnosticPhaseCode, DiagnosticBattery, DiagnosticPhasePlan, DiagnosticPhaseItem, DiagnosticResult, TopicDiagnosticResult, WrongAnswerDiagnosis
│   ├── reporting.ts                     ← StudentDashboard, SubjectSummary, ParentDashboardSnapshot, ParentStudentSummary, ParentRiskSummary
│   ├── glossary.ts                      ← KnowledgeEntry, EntryRelationship, KnowledgeBundle, GlossarySearchResult, AudioScript
│   ├── library.ts                       ← LibraryItem, LibraryShelf, ContentObject, RevisionPack, LibraryState
│   ├── games.ts                         ← MindStackRun, MindStackBlock, TugOfWarMatch, GameAnalytics, GameConfig
│   ├── traps.ts                         ← ContrastProfile, TrapCard, TrapRound, TrapResult, DifferenceDrillState
│   ├── past-papers.ts                   ← QuestionSignal, QuestionFamily, PaperDNA, ExamStoryline, CoAppearanceIndex
│   ├── goals-calendar.ts               ← Goal, GoalHierarchy, AcademicCalendarEvent, ExamTimeline, WeeklyPlan, DailyPlan
│   ├── intake.ts                        ← SubmissionBundle, SourcePage, PageClassification, QuestionAnswerAlignment, SmartReviewItem
│   ├── premium.ts                       ← PremiumProfile, StrategyMemo, InterventionAlert, ConciergeMessage, RiskFlag
│   ├── elite.ts                         ← EliteProfile, ElitePerformanceScore, EliteTier, TopicDomination, EliteSessionResult
│   ├── content.ts                       ← ContentPack, PackManifest, ContentAtom, ContentArtifact, TrustState
│   ├── memory.ts                        ← MemoryState, MemoryStabilityIndex, DecayRisk, RecallMode, RetentionSchedule
│   ├── beat-yesterday.ts                ← DailyTarget, ClimbState, DailyPerformanceProfile, GrowthMode
│   ├── rise.ts                          ← TransformationStage, WeaknessMap, RecoveryPlan, MomentumScore
│   ├── journey.ts                       ← JourneyMap, Station, JourneyPhase, RouteMode, MilestoneExam
│   ├── knowledge-gap.ts                 ← GapScore, GapType, SkillState, SolidificationSession, GapPriority
│   ├── mock-centre.ts                   ← MockConfig, MockBlueprint, ExamHallState, PostMockReview, MockHistory, ReadinessScore
│   └── time-orchestration.ts            ← AvailabilityModel, ScheduleWindow, SessionDemand, LiveSessionState, FreeNowResult
│
├── ipc/                                 ← Tauri command bindings (ALL commands from ALL 21 crates)
│   ├── index.ts                         ← Central invoke wrapper with error handling
│   ├── identity.commands.ts             ← createAccount, authenticate, listAccounts, resetPin, ...
│   ├── curriculum.commands.ts           ← listSubjects, listTopics, getAcademicNode, searchCurriculum, ...
│   ├── questions.commands.ts            ← getQuestion, listOptions, selectQuestions, getQuestionFamily, ...
│   ├── student-model.commands.ts        ← processAnswer, getStudentTopicStates, getLearnerTruthSnapshot, ...
│   ├── sessions.commands.ts             ← startPracticeSession, startCustomTest, submitAnswer, getSessionSnapshot, pauseSession, resumeSession, completeSession, ...
│   ├── coach.commands.ts                ← resolveCoachState, resolveNextCoachAction, assessContentReadiness, buildTopicCase, listPriorityTopicCases, ...
│   ├── diagnostics.commands.ts          ← startDiagnostic, getDiagnosticPhase, listPhaseItems, submitDiagnosticAnswer, completeDiagnosticPhase, getDiagnosticResult, ...
│   ├── reporting.commands.ts            ← getStudentDashboard, buildParentDashboard, generateReport, ...
│   ├── glossary.commands.ts             ← searchGlossary, getEntry, listRelationships, getBundles, getAudioScript, ...
│   ├── library.commands.ts              ← listShelves, getLibraryItems, createRevisionPack, updateItemState, ...
│   ├── games.commands.ts                ← startMindStackRun, submitGameAnswer, getGameAnalytics, startTugOfWar, ...
│   ├── traps.commands.ts                ← getContrastProfile, startTrapRound, submitTrapAnswer, getTrapResult, ...
│   ├── past-papers.commands.ts          ← getQuestionFamilies, getPaperDNA, getExamStoryline, getCoAppearance, ...
│   ├── goals-calendar.commands.ts       ← createGoal, listGoals, getExamTimeline, getWeeklyPlan, getDailyPlan, addCalendarEvent, ...
│   ├── intake.commands.ts               ← createBundle, uploadPage, classifyPages, alignQuestionsAnswers, getSmartReview, ...
│   ├── premium.commands.ts              ← getPremiumProfile, getStrategyMemo, listInterventionAlerts, sendConciergeMessage, ...
│   ├── elite.commands.ts                ← getEliteProfile, startEliteSession, getElitePerformanceScore, getTopicDomination, ...
│   ├── content.commands.ts              ← installPack, listPacks, getPackManifest, verifyPack, ...
│   ├── memory.commands.ts               ← getMemoryStates, startMemoryScan, startMemoryRescue, getDecayRisks, ...
│   ├── beat-yesterday.commands.ts       ← getDailyTarget, startClimb, getClimbState, getDailyPerformance, ...
│   ├── rise.commands.ts                 ← startRiseMode, getWeaknessMap, getRecoveryPlan, getTransformationStage, ...
│   ├── journey.commands.ts              ← getJourneyMap, getStation, startJourneySession, getJourneyPhase, ...
│   ├── knowledge-gap.commands.ts        ← startGapScan, getGapDashboard, startSolidification, getGapPriorities, ...
│   ├── mock-centre.commands.ts          ← createMockBlueprint, startMock, submitMockAnswer, getMockReview, getMockHistory, getReadinessScore, ...
│   └── time-orchestration.commands.ts   ← setAvailability, getSchedule, triggerFreeNow, getSessionAwareness, reportSessionState, ...
│
├── composables/                         ← Vue composables (shared reactive logic)
│   ├── useAuth.ts                       ← Current account, role, PIN state, profile switching
│   ├── useCoach.ts                      ← Coach state, next action, directive rendering, journey state
│   ├── useSession.ts                    ← Session lifecycle (start, answer, pause, resume, complete, debrief)
│   ├── useStudent.ts                    ← LearnerTruthSnapshot, topic states, mastery, memory, gaps
│   ├── useCurriculum.ts                 ← Subjects, topics tree, academic nodes, search
│   ├── useQuestions.ts                  ← Question fetch, options, selection, family grouping
│   ├── useDiagnostic.ts                 ← Diagnostic battery lifecycle, phase management, results
│   ├── useGlossary.ts                   ← Search, entries, relationships, audio, bundles
│   ├── useLibrary.ts                    ← Shelves, items, packs, state management
│   ├── useGames.ts                      ← MindStack, Tug of War, Traps game state
│   ├── usePastPapers.ts                 ← Families, DNA, storylines, patterns
│   ├── useGoals.ts                      ← Goal hierarchy, calendar, exam timeline, plans
│   ├── useIntake.ts                     ← Upload, OCR, alignment, smart review
│   ├── usePremium.ts                    ← Premium features, strategy, concierge
│   ├── useElite.ts                      ← Elite profile, sessions, scoring, tiers
│   ├── useMemory.ts                     ← Memory states, decay, scan, rescue, scheduling
│   ├── useBeatYesterday.ts              ← Daily targets, climb, performance tracking
│   ├── useRise.ts                       ← Transformation stages, weakness maps, recovery
│   ├── useJourney.ts                    ← Journey map, stations, phases, routes
│   ├── useKnowledgeGap.ts               ← Gap scan, dashboard, solidification
│   ├── useMockCentre.ts                 ← Mock lifecycle, exam hall, review, history
│   ├── useTimeOrchestration.ts          ← Schedule, availability, free-now, session awareness
│   ├── useReporting.ts                  ← Dashboards, parent insights, PDF generation
│   ├── useTimer.ts                      ← 6 timer variants (soft, strict, shrinking, burst, pressure, cluster)
│   ├── useSound.ts                      ← Audio playback, mode-specific sound profiles
│   ├── useTheme.ts                      ← Emotional theming (6 modes), role theme, dark/light
│   ├── useAnimation.ts                  ← Shared animation triggers, transitions
│   ├── useOffline.ts                    ← Offline state, content pack management
│   └── usePdf.ts                        ← PDF/print export generation
│
├── stores/                              ← Pinia stores (global reactive state)
│   ├── auth.ts                          ← Current account, role, all accounts, PIN state
│   ├── coach.ts                         ← Coach state, active directives, pending actions
│   ├── session.ts                       ← Active session, items, answers, timer state
│   ├── student.ts                       ← LearnerTruthSnapshot, topic states cache
│   ├── curriculum.ts                    ← Subjects, topics tree, selected subject
│   ├── ui.ts                            ← Theme mode, sidebar state, modals, toasts, breadcrumbs
│   ├── diagnostic.ts                    ← Active diagnostic, current phase, results
│   ├── game.ts                          ← Active game state (MindStack/TugOfWar/Traps)
│   ├── glossary.ts                      ← Search state, current entry, audio playback
│   ├── calendar.ts                      ← Exam timeline, weekly/daily plan, availability
│   └── upload.ts                        ← Upload pipeline state, bundle, review items
│
├── layouts/
│   ├── auth.vue                         ← Full-screen, neutral, no sidebar
│   ├── student.vue                      ← Sidebar(5 primary + AReal) + content + coach overlay
│   ├── parent.vue                       ← Sidebar(5 items) + spacious content
│   ├── admin.vue                        ← Sidebar(8 sections) + dense content + sub-tabs
│   └── focus.vue                        ← Full-screen distraction-free (exams, focus mode)
│
├── components/
│   │
│   ├── ui/                              ← DESIGN SYSTEM PRIMITIVES (~40 components)
│   │   ├── AppButton.vue                ← Primary, secondary, ghost, danger, sizes
│   │   ├── AppCard.vue                  ← Base card with header, body, footer slots
│   │   ├── AppBadge.vue                 ← Status badges, colored, with icons
│   │   ├── AppModal.vue                 ← Centered modal with backdrop
│   │   ├── AppDrawer.vue                ← Slide-in side panel
│   │   ├── AppBottomSheet.vue           ← Mobile-style bottom sheet
│   │   ├── AppTabs.vue                  ← Tab navigation
│   │   ├── AppDropdown.vue              ← Dropdown menu
│   │   ├── AppTooltip.vue               ← Hover tooltip
│   │   ├── AppSkeleton.vue              ← Loading skeleton
│   │   ├── AppEmpty.vue                 ← Empty state with icon + message + action
│   │   ├── AppError.vue                 ← Error state with retry
│   │   ├── AppToast.vue                 ← Toast notifications
│   │   ├── AppBreadcrumb.vue            ← Navigation breadcrumbs
│   │   ├── AppStepper.vue              ← Multi-step wizard indicator
│   │   ├── AppToggle.vue                ← Toggle switch
│   │   ├── AppInput.vue                 ← Text input with validation
│   │   ├── AppTextarea.vue              ← Multiline input
│   │   ├── AppSelect.vue                ← Select dropdown
│   │   ├── AppCheckbox.vue              ← Checkbox
│   │   ├── AppRadio.vue                 ← Radio button group
│   │   ├── AppSlider.vue                ← Range slider
│   │   ├── AppChip.vue                  ← Tag/chip with remove
│   │   ├── AppAvatar.vue                ← User avatar with initials fallback
│   │   ├── AppProgress.vue              ← Linear progress bar
│   │   ├── AppSpinner.vue               ← Loading spinner
│   │   ├── AppDivider.vue               ← Section divider
│   │   ├── AppCollapsible.vue           ← Expandable/collapsible section
│   │   ├── AppAccordion.vue             ← Accordion group
│   │   ├── AppTable.vue                 ← Data table with sorting, filtering
│   │   ├── AppPagination.vue            ← Page navigation
│   │   ├── AppSearch.vue                ← Search input with grouped results
│   │   ├── AppFileUpload.vue            ← Drag-and-drop file upload
│   │   ├── AppColorDot.vue              ← Colored status dot
│   │   ├── AppCountdown.vue             ← Animated countdown display
│   │   ├── AppConfirm.vue               ← Confirmation dialog
│   │   ├── AppScrollArea.vue            ← Custom scrollbar area
│   │   ├── AppResizable.vue             ← Resizable split pane
│   │   ├── AppKanban.vue                ← Kanban board column layout
│   │   └── AppTreeView.vue              ← Expandable tree hierarchy
│   │
│   ├── question/                        ← QUESTION SYSTEM (~20 components)
│   │   ├── QuestionCard.vue             ← Orchestrator: routes to format-specific renderer
│   │   ├── formats/
│   │   │   ├── McqQuestion.vue          ← Multiple choice (A/B/C/D)
│   │   │   ├── ShortAnswerQuestion.vue  ← Text input answer
│   │   │   ├── DragReorderQuestion.vue  ← Drag to reorder items
│   │   │   ├── MatchingQuestion.vue     ← Match left column to right
│   │   │   ├── FillBlankQuestion.vue    ← Fill in the blank
│   │   │   ├── DiagramLabelQuestion.vue ← Label parts of a diagram
│   │   │   ├── ComparisonTableQuestion.vue ← Complete comparison table
│   │   │   ├── StepByStepQuestion.vue   ← Multi-step solution input
│   │   │   ├── ClassificationQuestion.vue ← Sort items into categories
│   │   │   ├── SequencingQuestion.vue   ← Arrange in correct order
│   │   │   ├── TrueFalseQuestion.vue    ← True/false with trap logic
│   │   │   ├── EssayQuestion.vue        ← Long-form text with word count
│   │   │   ├── EquationBuilderQuestion.vue ← Build math equation
│   │   │   ├── CanvasDrawQuestion.vue   ← Free-draw canvas
│   │   │   └── FirstStepQuestion.vue    ← "What is the first step?"
│   │   ├── QuestionStem.vue             ← Question text with math/diagram rendering
│   │   ├── QuestionOption.vue           ← Single answer option (tappable)
│   │   ├── QuestionTimer.vue            ← Timer bar (6 variants)
│   │   ├── QuestionFlag.vue             ← Flag for review button
│   │   ├── ConfidenceCapture.vue        ← Sure / Not Sure / Guessed
│   │   ├── QuestionFeedback.vue         ← Correct/incorrect result
│   │   ├── QuestionExplanation.vue      ← Multi-layer explanation (progressive disclosure)
│   │   ├── WrongAnswerReview.vue        ← 10-part wrong answer card
│   │   ├── MistakeClinicFlow.vue        ← 5-step wrong answer coaching
│   │   ├── QuestionNav.vue              ← Question number grid (for mock/test navigation)
│   │   └── MathRenderer.vue             ← KaTeX wrapper for math expressions
│   │
│   ├── coach/                           ← COACHING SYSTEM (~15 components)
│   │   ├── CoachHub.vue                 ← Main coach home renderer
│   │   ├── CoachDirectiveCard.vue       ← Renders a CoachNextAction
│   │   ├── InsightCard.vue              ← Topic insight with action buttons
│   │   ├── CoachNote.vue                ← Thin strip contextual guidance
│   │   ├── CoachVoice.vue               ← "Coach says" message card
│   │   ├── RescueDock.vue               ← 7 help buttons (simplify, hint, first step, compare, explain, audio, example)
│   │   ├── SessionBrief.vue             ← Pre-session briefing card
│   │   ├── SessionDebrief.vue           ← Post-session analysis
│   │   ├── PhaseIndicator.vue           ← Current learning phase badge
│   │   ├── ExamCountdown.vue            ← Days to exam + readiness band
│   │   ├── TodaysMission.vue            ← Today's recommended action card
│   │   ├── CoachStateIndicator.vue      ← Calm Guide / Teacher / Rescue / etc.
│   │   ├── RecoveryBanner.vue           ← Gentle "you're rebuilding" message
│   │   └── WhyThisCard.vue              ← "Why this session now" explainer
│   │
│   ├── session/                         ← SESSION PLAYER (~10 components)
│   │   ├── SessionPlayer.vue            ← Dynamic block renderer (THE session engine)
│   │   ├── SessionBlock.vue             ← Individual block (quiz, explanation, drill, etc.)
│   │   ├── SessionProgress.vue          ← Progress through session
│   │   ├── SessionTimer.vue             ← Session-level timer
│   │   ├── SessionPause.vue             ← Pause overlay with resume/stop
│   │   ├── SessionComplete.vue          ← Session completion transition
│   │   ├── BlockTransition.vue          ← Animated transition between blocks
│   │   ├── WarmStartBlock.vue           ← Beat Yesterday warm start
│   │   ├── SpeedBurstBlock.vue          ← Timed rapid-fire block
│   │   └── ReflectionBlock.vue          ← Post-session reflection prompt
│   │
│   ├── viz/                             ← DATA VISUALIZATIONS (~25 components)
│   │   ├── HeatMap.vue                  ← Topic heat map (mastery/gap/memory)
│   │   ├── RadarChart.vue               ← Multi-dimension performance spider
│   │   ├── ProgressRing.vue             ← Circular progress indicator
│   │   ├── TrendArrow.vue               ← Directional trend indicator
│   │   ├── TrendLine.vue                ← Line chart over time
│   │   ├── BarChart.vue                 ← Horizontal/vertical bars
│   │   ├── MasteryBadge.vue             ← 8 mastery states (icon + color + label)
│   │   ├── ReadinessGauge.vue           ← Exam readiness meter
│   │   ├── TopicStatusCard.vue          ← Per-topic multi-score card
│   │   ├── StreakCounter.vue             ← Animated streak display
│   │   ├── ComboCounter.vue             ← Game combo multiplier
│   │   ├── ComparisonCard.vue           ← "Yesterday vs Today" card
│   │   ├── GapRing.vue                  ← Knowledge gap circular display
│   │   ├── MemoryStrandViz.vue          ← Memory connection visualization
│   │   ├── DecaySeverityBadge.vue        ← 5-level decay indicator
│   │   ├── KnowledgeMap.vue             ← Zoomable topic dependency graph
│   │   ├── ConstellationView.vue        ← Force-directed question family graph
│   │   ├── TimelineBand.vue             ← Horizontal timeline with markers
│   │   ├── CurriculumTerrain.vue        ← Curriculum progress terrain map
│   │   ├── CoverageMatrix.vue           ← Topic × content-type coverage heatmap
│   │   ├── PaperDNAChart.vue            ← Exam paper profile visualization
│   │   ├── ErrorTypeChart.vue           ← Error distribution visualization
│   │   ├── PressureProfile.vue          ← Calm vs pressure performance comparison
│   │   ├── TopicDominationMeter.vue     ← Elite mode topic mastery bar
│   │   └── AvailabilityGrid.vue         ← Weekly time-block picker
│   │
│   ├── layout/                          ← NAVIGATION & LAYOUT (~15 components)
│   │   ├── StudentSidebar.vue           ← 5 primary items + AReal
│   │   ├── ParentSidebar.vue            ← 5 items
│   │   ├── AdminSidebar.vue             ← 8 sections with sub-items
│   │   ├── SidebarItem.vue              ← Individual nav item with icon + label + badge
│   │   ├── ProfileSwitcher.vue          ← Account tile grid
│   │   ├── ProfileTile.vue              ← Single account: avatar + name + class + progress ring
│   │   ├── PinPad.vue                   ← Numeric PIN entry
│   │   ├── PageHeader.vue               ← Page title + breadcrumb + actions
│   │   ├── SubTabs.vue                  ← Secondary tab navigation within a page
│   │   ├── SplitPane.vue                ← Resizable left/right admin layout
│   │   ├── ContentArea.vue              ← Scrollable content container
│   │   ├── FocusOverlay.vue             ← Full-screen focus mode wrapper
│   │   ├── ExamHallFrame.vue            ← Ceremonial exam simulation frame
│   │   ├── OnboardingWizard.vue         ← Multi-step onboarding container
│   │   └── NotificationCenter.vue       ← Notification dropdown panel
│   │
│   ├── diagnostic/                      ← DIAGNOSTIC SYSTEM (~10 components)
│   │   ├── DiagnosticLauncher.vue       ← Mode selection (quick/standard/deep)
│   │   ├── DiagnosticPhasePlayer.vue    ← Single phase question delivery
│   │   ├── PhaseTransition.vue          ← Between-phase brief screen
│   │   ├── DiagnosticReport.vue         ← 7-section report container
│   │   ├── ReportOverview.vue           ← Overall dashboard section
│   │   ├── ReportAcademicProfile.vue    ← Profile summary section
│   │   ├── ReportTopicBreakdown.vue     ← Per-topic detail section
│   │   ├── ReportMisconceptionBank.vue  ← Misconception listing section
│   │   ├── ReportExamBehavior.vue       ← Exam behavior analysis section
│   │   └── ReportInterventionMap.vue    ← Priority intervention section
│   │
│   ├── mock/                            ← MOCK CENTRE (~10 components)
│   │   ├── MockHome.vue                 ← Mock centre landing
│   │   ├── MockSetup.vue                ← Mock configuration wizard
│   │   ├── MockPreFlight.vue            ← Pre-flight confirmation card
│   │   ├── MockExamHall.vue             ← Live exam interface (minimal, focused)
│   │   ├── MockPacingBadge.vue          ← On Pace / Behind / Fast indicator
│   │   ├── MockSubmission.vue           ← Submission confirmation
│   │   ├── MockReview.vue               ← 6-tab post-mock review
│   │   ├── MockReviewTab.vue            ← Individual review tab content
│   │   ├── MockHistory.vue              ← Battle history timeline
│   │   └── MockForecast.vue             ← Readiness forecast display
│   │
│   ├── glossary/                        ← GLOSSARY SYSTEM (~12 components)
│   │   ├── GlossarySearch.vue           ← Smart search with grouped results
│   │   ├── GlossaryEntryCard.vue        ← Entry card (definition/formula/concept)
│   │   ├── GlossaryEntryPage.vue        ← Full entry page with all sections
│   │   ├── GlossaryDepthTabs.vue        ← Quick/Simple/Exam/Deep/Visual/Audio tabs
│   │   ├── GlossaryInlinePanel.vue      ← Slide-up panel from tapped term
│   │   ├── GlossaryCompare.vue          ← Side-by-side concept comparison
│   │   ├── FormulaDisplay.vue           ← KaTeX formula with variable tap-to-explain
│   │   ├── FormulaLab.vue               ← Interactive formula playground
│   │   ├── ConceptMapView.vue           ← Visual concept relationship network
│   │   ├── AudioPlayer.vue              ← Full audio player (play/pause/speed/skip)
│   │   ├── MiniAudioPlayer.vue          ← Bottom-bar mini player
│   │   └── AudioRadioMode.vue           ← Continuous listening station selector
│   │
│   ├── library/                         ← LIBRARY SYSTEM (~8 components)
│   │   ├── LibraryHome.vue              ← Library command center
│   │   ├── LibraryShelf.vue             ← Single shelf (personal, topic, mistake, etc.)
│   │   ├── ContentObjectCard.vue        ← Content item card with type icon + state badge
│   │   ├── RevisionPackBuilder.vue      ← Build custom revision pack
│   │   ├── StudyFeed.vue                ← Smart recommendation feed
│   │   ├── LibrarySearch.vue            ← Library-specific search
│   │   ├── LibraryItemDetail.vue        ← Item detail view with actions
│   │   └── LibraryStateTag.vue          ← Item state indicator (new/saved/fading/mastered)
│   │
│   ├── exam-intel/                      ← PAST EXAM INTELLIGENCE (~10 components)
│   │   ├── ExamIntelHome.vue            ← Past exam hub with entry points
│   │   ├── QuestionFamilyCard.vue       ← Question family overview
│   │   ├── QuestionDNACard.vue          ← Full question intelligence card
│   │   ├── FamilyTreeView.vue           ← Parent/sibling/descendant tree
│   │   ├── EvolutionTimeline.vue        ← How questions changed over years
│   │   ├── PaperDNADashboard.vue        ← Year-specific paper profile
│   │   ├── StorylineView.vue            ← Narrative topic exam history
│   │   ├── InverseMap.vue               ← Questions that replace each other
│   │   ├── ExamReplayMode.vue           ← Sit a real historical paper
│   │   └── StudentMirrorView.vue        ← Your performance vs exam patterns
│   │
│   ├── game/                            ← GAME SYSTEMS (~20 components)
│   │   ├── mindstack/
│   │   │   ├── MindStackCanvas.vue      ← PixiJS game board
│   │   │   ├── MindStackHUD.vue         ← Score, control status, streak
│   │   │   ├── MindStackQuestion.vue    ← Side-panel question during play
│   │   │   ├── MindStackMorphMenu.vue   ← Shape morph selection
│   │   │   ├── MindStackPowerUps.vue    ← Power-up toolbar
│   │   │   ├── MindStackGameOver.vue    ← End screen with analytics
│   │   │   └── MindStackModeSelect.vue  ← 7 sub-mode selection
│   │   ├── tugofwar/
│   │   │   ├── TugOfWarCanvas.vue       ← Rope/momentum visualization
│   │   │   ├── TugOfWarQuestion.vue     ← Question overlay
│   │   │   ├── TugOfWarPowerUps.vue     ← Power-up display
│   │   │   └── TugOfWarResult.vue       ← Match result
│   │   └── traps/
│   │       ├── TrapsHub.vue             ← 5 mode selection
│   │       ├── DifferenceDrill.vue      ← Card-sort game board
│   │       ├── TrapCard.vue             ← Individual droppable card
│   │       ├── ConceptBin.vue           ← Drop target bin
│   │       ├── SimilarityTrap.vue       ← Deception mode interface
│   │       ├── KnowTheDifference.vue    ← Comparative learning split-panel
│   │       ├── WhichIsWhich.vue         ← Speed recognition mode
│   │       ├── UnmaskMode.vue           ← 5th traps mode
│   │       └── TrapsReview.vue          ← Post-round review with replay
│   │
│   ├── modes/                           ← LEARNING MODE SURFACES (~20 components)
│   │   ├── beat-yesterday/
│   │   │   ├── BeatYesterdayHome.vue    ← Hero screen: yesterday vs today
│   │   │   ├── DailyClimb.vue           ← 4-block session (warm/core/burst/finish)
│   │   │   ├── MicroGainIndicator.vue   ← +2, +1, -3 sec chips
│   │   │   └── ClimbTrends.vue          ← Weekly review trends
│   │   ├── elite/
│   │   │   ├── EliteHome.vue            ← Identity panel + today's push
│   │   │   ├── EliteArena.vue           ← 7 session type selection
│   │   │   ├── EliteSession.vue         ← Live elite session
│   │   │   ├── EliteDebrief.vue         ← Analytical session results
│   │   │   ├── EliteRecords.vue         ← Records wall + badges
│   │   │   ├── EliteInsights.vue        ← Performance analytics
│   │   │   └── TopicDominationBoard.vue ← Topic domination status
│   │   ├── rise/
│   │   │   ├── RiseHome.vue             ← 4-stage transformation view
│   │   │   ├── WeaknessMapView.vue      ← Visual weakness map
│   │   │   └── TransformationSession.vue
│   │   ├── journey/
│   │   │   ├── JourneyMap.vue           ← Visual journey with stations
│   │   │   ├── JourneyStation.vue       ← Station detail + entry
│   │   │   └── JourneyMission.vue       ← Daily journey mission
│   │   ├── knowledge-gap/
│   │   │   ├── GapScan.vue              ← Scan experience
│   │   │   ├── GapDashboard.vue         ← 6-section gap overview
│   │   │   ├── GapMapView.vue           ← Interactive gap map
│   │   │   └── SolidificationSession.vue ← 5-stage gap repair
│   │   ├── memory/
│   │   │   ├── MemoryHome.vue           ← Memory dashboard
│   │   │   ├── MemoryScan.vue           ← Scan experience
│   │   │   ├── MemoryRescue.vue         ← Rescue session
│   │   │   └── ChainRepair.vue          ← Prerequisite chain repair
│   │   ├── spark/
│   │   │   ├── SparkOnboarding.vue      ← Behavioral type classification
│   │   │   ├── SparkMission.vue         ← Ultra-short engagement mission
│   │   │   └── SparkRewards.vue         ← Avatar, titles, surprise mechanics
│   │   └── teach/
│   │       ├── TeachModePage.vue        ← 16-block teaching layout
│   │       ├── TeachBlock.vue           ← Individual teaching block
│   │       └── WorkedExample.vue        ← Step-by-step worked example
│   │
│   ├── parent/                          ← PARENT PORTAL COMPONENTS (~15 components)
│   │   ├── ParentHome.vue               ← Family overview with child cards
│   │   ├── ChildCard.vue                ← Per-child summary tile
│   │   ├── ChildDetail.vue              ← Full child dashboard
│   │   ├── ParentInsightCard.vue        ← Plain-language insight with action
│   │   ├── RiskAlert.vue                ← Severity-colored alert with recommendation
│   │   ├── WeeklyMemo.vue               ← Strategy memo display
│   │   ├── AttentionNeeded.vue          ← Prioritized alert list
│   │   ├── InterventionCenter.vue       ← 3-column: handling / needs you / you can do
│   │   ├── ParentReport.vue             ← Printable progress report
│   │   ├── ParentConcierge.vue          ← Premium Q&A interface
│   │   ├── ParentTimeline.vue           ← Activity timeline per child
│   │   ├── ParentSubjectRisk.vue        ← Subject risk map visualization
│   │   ├── ReadinessBrief.vue           ← Exam readiness summary
│   │   ├── EvidenceViewer.vue           ← Uploaded test paper review
│   │   └── ParentSettings.vue           ← Manage children, PINs, preferences
│   │
│   ├── admin/                           ← ADMIN PORTAL COMPONENTS (~20 components)
│   │   ├── AdminCommandCenter.vue       ← System overview dashboard
│   │   ├── CurriculumUpload.vue         ← File upload with metadata
│   │   ├── CurriculumReviewWorkbench.vue ← Split-pane: source vs extracted
│   │   ├── CurriculumTreeEditor.vue     ← Editable curriculum hierarchy
│   │   ├── CurriculumDiffViewer.vue     ← Version comparison
│   │   ├── QuestionAuthor.vue           ← Question creation form
│   │   ├── QuestionReviewConsole.vue    ← Batch review with classification
│   │   ├── QuestionFamilyBrowser.vue    ← Browse question families
│   │   ├── ContentPipelineBoard.vue     ← Kanban: raw → parsed → verified → published
│   │   ├── ContentCoverageHeatmap.vue   ← Topic × content-type matrix
│   │   ├── StudentMonitor.vue           ← Cross-student analytics
│   │   ├── PackManager.vue              ← Content pack CRUD
│   │   ├── UserManager.vue              ← Account management table
│   │   ├── QualityDashboard.vue         ← Content quality metrics
│   │   ├── AuditLog.vue                 ← System audit trail viewer
│   │   ├── CoachTuning.vue              ← Coach behavior configuration
│   │   ├── SystemHealth.vue             ← Database, pack, engine health
│   │   ├── EntitlementManager.vue       ← Premium/Elite tier management
│   │   ├── TimelineManager.vue          ← Exam dates, calendar admin
│   │   └── PublishCenter.vue            ← Content publishing controls
│   │
│   └── upload/                          ← DOCUMENT UPLOAD SYSTEM (~8 components)
│       ├── UploadWizard.vue             ← Step-by-step upload flow
│       ├── BundleCreator.vue            ← Create submission bundle
│       ├── PageClassifier.vue           ← Mark pages as question/answer/marked
│       ├── OcrConfidenceEditor.vue      ← Review OCR results with corrections
│       ├── QuestionAligner.vue          ← Align questions to answers
│       ├── SmartReview.vue              ← Per-question review with diagnosis
│       ├── EssayAnalysis.vue            ← Essay/Section B analysis
│       └── UploadSummary.vue            ← Bundle completion summary
│
├── pages/                               ← FILE-BASED ROUTING (ALL PAGES)
│   ├── index.vue                        ← Profile switcher (auth layout)
│   ├── pin.vue                          ← PIN entry (auth layout)
│   │
│   ├── student/                         ← STUDENT PORTAL (~50 routes)
│   │   ├── index.vue                    ← Coach Hub Home
│   │   ├── onboarding/
│   │   │   ├── welcome.vue
│   │   │   ├── subjects.vue
│   │   │   ├── content-packs.vue
│   │   │   └── diagnostic.vue
│   │   ├── session/
│   │   │   ├── [id].vue                 ← Live session player
│   │   │   └── debrief/[id].vue
│   │   ├── practice/
│   │   │   ├── index.vue                ← Practice hub
│   │   │   ├── custom-test.vue
│   │   │   └── topic/[id].vue
│   │   ├── diagnostic/
│   │   │   ├── index.vue
│   │   │   ├── [id].vue                 ← Live diagnostic
│   │   │   └── report/[id].vue
│   │   ├── mock/
│   │   │   ├── index.vue                ← Mock Centre home
│   │   │   ├── setup.vue
│   │   │   ├── hall/[id].vue            ← Exam hall
│   │   │   ├── review/[id].vue
│   │   │   └── history.vue
│   │   ├── progress/
│   │   │   ├── index.vue
│   │   │   ├── mastery-map.vue
│   │   │   ├── analytics.vue
│   │   │   └── history.vue
│   │   ├── journey/
│   │   │   ├── index.vue
│   │   │   └── station/[id].vue
│   │   ├── beat-yesterday/
│   │   │   └── index.vue
│   │   ├── elite/
│   │   │   ├── index.vue
│   │   │   ├── arena.vue
│   │   │   ├── session/[id].vue
│   │   │   ├── records.vue
│   │   │   ├── domination.vue
│   │   │   └── insights.vue
│   │   ├── rise/
│   │   │   ├── index.vue
│   │   │   └── session/[id].vue
│   │   ├── spark/
│   │   │   ├── index.vue
│   │   │   └── mission/[id].vue
│   │   ├── knowledge-gap/
│   │   │   ├── index.vue
│   │   │   ├── scan.vue
│   │   │   └── session/[id].vue
│   │   ├── memory/
│   │   │   ├── index.vue
│   │   │   └── session/[id].vue
│   │   ├── glossary/
│   │   │   ├── index.vue                ← Glossary Lab home
│   │   │   ├── entry/[id].vue
│   │   │   ├── compare.vue
│   │   │   ├── audio.vue                ← Radio mode
│   │   │   ├── formula-lab.vue
│   │   │   └── test-lab.vue             ← 14+ test modes
│   │   ├── library/
│   │   │   ├── index.vue
│   │   │   ├── shelf/[id].vue
│   │   │   └── pack-builder.vue
│   │   ├── teach/
│   │   │   └── [topicId].vue            ← 16-block teach mode
│   │   ├── exam-intel/
│   │   │   ├── index.vue
│   │   │   ├── family/[id].vue
│   │   │   ├── paper/[year].vue         ← Paper DNA
│   │   │   ├── constellation.vue
│   │   │   ├── storyline/[topicId].vue
│   │   │   └── replay/[paperId].vue     ← Sit a real paper
│   │   ├── games/
│   │   │   ├── index.vue                ← Games hub
│   │   │   ├── mindstack/
│   │   │   │   ├── index.vue            ← MindStack home
│   │   │   │   ├── play.vue             ← Live game
│   │   │   │   └── review/[id].vue
│   │   │   ├── tugofwar/
│   │   │   │   ├── index.vue
│   │   │   │   └── play.vue
│   │   │   └── traps/
│   │   │       ├── index.vue            ← Traps hub (5 modes)
│   │   │       └── play/[mode].vue      ← Live traps session
│   │   ├── mistakes/
│   │   │   ├── index.vue                ← Mistake Lab
│   │   │   └── pattern/[id].vue         ← Error pattern detail
│   │   ├── calendar/
│   │   │   └── index.vue                ← Academic calendar
│   │   ├── upload/
│   │   │   ├── index.vue                ← Upload wizard
│   │   │   └── review/[bundleId].vue    ← Smart review
│   │   ├── focus.vue                    ← Focus mode launcher
│   │   └── settings.vue
│   │
│   ├── parent/                          ← PARENT PORTAL (~12 routes)
│   │   ├── index.vue                    ← Parent home
│   │   ├── child/[id].vue               ← Per-child dashboard
│   │   ├── performance.vue
│   │   ├── attention.vue
│   │   ├── reports.vue
│   │   ├── intervention.vue             ← Premium
│   │   ├── concierge.vue                ← Premium
│   │   ├── timeline.vue
│   │   ├── upload.vue                   ← Parent upload flow
│   │   ├── curriculum.vue               ← Simplified curriculum view
│   │   └── settings.vue
│   │
│   └── admin/                           ← ADMIN PORTAL (~20 routes)
│       ├── index.vue                    ← Command center
│       ├── curriculum/
│       │   ├── index.vue
│       │   ├── upload.vue
│       │   ├── review.vue               ← Split-pane workbench
│       │   ├── editor.vue               ← Tree editor
│       │   ├── graph.vue                ← Dependency graph view
│       │   ├── coverage.vue
│       │   ├── diff.vue                 ← Version diff
│       │   └── publish.vue
│       ├── questions/
│       │   ├── index.vue
│       │   ├── author.vue
│       │   ├── review.vue
│       │   └── families.vue
│       ├── content/
│       │   ├── index.vue                ← Pipeline dashboard
│       │   ├── coverage.vue             ← Coverage heatmap
│       │   └── quality.vue
│       ├── students/
│       │   ├── index.vue                ← Student monitor
│       │   └── [id].vue                 ← Individual student detail
│       ├── packs/
│       │   └── index.vue
│       ├── users.vue
│       ├── entitlements.vue
│       ├── timeline.vue
│       ├── quality.vue
│       ├── audit.vue
│       ├── coach-tuning.vue
│       ├── health.vue
│       └── settings.vue
│
├── utils/                               ← UTILITY FUNCTIONS
│   ├── format.ts                        ← BasisPoints → %, dates, durations
│   ├── mastery.ts                       ← MasteryState → color, icon, label
│   ├── error-types.ts                   ← ErrorType → color, icon, label, description
│   ├── copy.ts                          ← Audience-aware copy (student/parent/admin translations)
│   ├── sound.ts                         ← Sound effect player with mode profiles
│   ├── pdf.ts                           ← PDF generation helpers
│   ├── math.ts                          ← Math rendering helpers
│   ├── color.ts                         ← Color calculations, accessibility checks
│   ├── validation.ts                    ← Form validation rules
│   └── constants.ts                     ← All magic numbers, state labels, route names
│
└── public/
    ├── icons/                           ← App icons
    └── splash/                          ← Splash screen assets
```

---

## DESIGN SYSTEM (COMPLETE SPECIFICATION)

### Color Tokens

```css
/* === BASE PALETTE === */
--color-white: #FFFFFF;
--color-black: #0A0A0A;

/* === STUDENT PALETTE (Warm) === */
--student-bg: #FAFAF8;
--student-surface: #FFFFFF;
--student-surface-hover: #F5F5F0;
--student-border: #E8E5E0;
--student-text: #1A1A1A;
--student-text-secondary: #6B6B6B;
--student-text-muted: #9CA3AF;
--student-primary: #2563EB;
--student-primary-hover: #1D4ED8;
--student-primary-light: #DBEAFE;
--student-success: #16A34A;
--student-success-light: #DCFCE7;
--student-warning: #F59E0B;
--student-warning-light: #FEF3C7;
--student-danger: #DC2626;
--student-danger-light: #FEE2E2;
--student-info: #0EA5E9;
--student-info-light: #E0F2FE;

/* === PARENT PALETTE (Cool, Premium) === */
--parent-bg: #F8FAFC;
--parent-surface: #FFFFFF;
--parent-surface-hover: #F1F5F9;
--parent-border: #E2E8F0;
--parent-text: #0F172A;
--parent-text-secondary: #475569;
--parent-text-muted: #94A3B8;
--parent-primary: #0F172A;
--parent-accent: #6366F1;
--parent-accent-light: #EEF2FF;
--parent-success: #059669;
--parent-success-light: #D1FAE5;
--parent-risk-high: #DC2626;
--parent-risk-medium: #F59E0B;
--parent-risk-low: #3B82F6;

/* === ADMIN PALETTE (Neutral, Dense) === */
--admin-bg: #F1F5F9;
--admin-surface: #FFFFFF;
--admin-surface-hover: #F8FAFC;
--admin-border: #CBD5E1;
--admin-text: #1E293B;
--admin-text-secondary: #475569;
--admin-text-muted: #94A3B8;
--admin-primary: #1E293B;
--admin-accent: #8B5CF6;
--admin-accent-light: #F5F3FF;
--admin-success: #22C55E;
--admin-danger: #EF4444;
--admin-warning: #F59E0B;

/* === MASTERY STATES (Shared) === */
--mastery-unseen: #D1D5DB;
--mastery-exposed: #93C5FD;
--mastery-emerging: #FDE68A;
--mastery-partial: #FCD34D;
--mastery-fragile: #FB923C;
--mastery-stable: #4ADE80;
--mastery-robust: #22C55E;
--mastery-exam-ready: #2563EB;

/* === ERROR TYPES (Shared) === */
--error-knowledge-gap: #EF4444;
--error-conceptual: #F97316;
--error-recognition: #F59E0B;
--error-execution: #EAB308;
--error-careless: #A3E635;
--error-pressure: #8B5CF6;
--error-expression: #EC4899;
--error-speed: #06B6D4;
--error-guessing: #6B7280;
--error-misconception: #DC2626;

/* === DECAY SEVERITY (Shared) === */
--decay-stable: #22C55E;
--decay-watchlist: #FCD34D;
--decay-fragile: #FB923C;
--decay-decaying: #EF4444;
--decay-collapsed: #1F2937;

/* === EMOTIONAL MODE OVERRIDES === */
--mode-recovery-bg: #FFFBEB;
--mode-recovery-accent: #D97706;
--mode-recovery-text: #92400E;

--mode-pressure-bg: #0F172A;
--mode-pressure-accent: #EF4444;
--mode-pressure-text: #F1F5F9;

--mode-elite-bg: #1A1A2E;
--mode-elite-accent: #A78BFA;
--mode-elite-text: #E2E8F0;
--mode-elite-gold: #F5C842;

--mode-game-bg: #ECFDF5;
--mode-game-accent: #10B981;
--mode-game-energy: #F59E0B;

--mode-celebration-bg: #FFF7ED;
--mode-celebration-accent: #F97316;
--mode-celebration-glow: #FBBF24;

--mode-focus-bg: #F8FAFC;
--mode-focus-accent: #64748B;

/* === DARK MODE VARIANTS === */
--dark-bg: #0A0A0A;
--dark-surface: #171717;
--dark-surface-hover: #262626;
--dark-border: #404040;
--dark-text: #FAFAFA;
--dark-text-secondary: #A3A3A3;
--dark-text-muted: #737373;
```

### Typography Scale
```css
/* Font families */
--font-primary: 'Inter', system-ui, sans-serif;
--font-mono: 'JetBrains Mono', monospace;

/* Size scale (rem) */
--text-xs: 0.75rem;    /* 12px */
--text-sm: 0.875rem;   /* 14px */
--text-base: 1rem;     /* 16px */
--text-lg: 1.125rem;   /* 18px */
--text-xl: 1.25rem;    /* 20px */
--text-2xl: 1.5rem;    /* 24px */
--text-3xl: 1.875rem;  /* 30px */
--text-4xl: 2.25rem;   /* 36px */
--text-5xl: 3rem;      /* 48px */

/* Weight */
--font-normal: 400;
--font-medium: 500;
--font-semibold: 600;
--font-bold: 700;

/* Line height */
--leading-tight: 1.25;
--leading-normal: 1.5;
--leading-relaxed: 1.75;

/* Role-specific base sizes */
--student-text-base: 1rem;      /* 16px - comfortable */
--parent-text-base: 1.125rem;   /* 18px - generous */
--admin-text-base: 0.875rem;    /* 14px - compact */
```

### Spacing Scale
```css
--space-0: 0;
--space-px: 1px;
--space-0.5: 0.125rem;  /* 2px */
--space-1: 0.25rem;     /* 4px */
--space-2: 0.5rem;      /* 8px */
--space-3: 0.75rem;     /* 12px */
--space-4: 1rem;        /* 16px */
--space-5: 1.25rem;     /* 20px */
--space-6: 1.5rem;      /* 24px */
--space-8: 2rem;        /* 32px */
--space-10: 2.5rem;     /* 40px */
--space-12: 3rem;       /* 48px */
--space-16: 4rem;       /* 64px */
--space-20: 5rem;       /* 80px */
--space-24: 6rem;       /* 96px */
```

### Border Radius
```css
--radius-none: 0;
--radius-sm: 0.25rem;   /* 4px */
--radius-md: 0.5rem;    /* 8px */
--radius-lg: 0.75rem;   /* 12px */
--radius-xl: 1rem;      /* 16px */
--radius-2xl: 1.5rem;   /* 24px */
--radius-full: 9999px;
```

### Elevation / Shadows
```css
--shadow-sm: 0 1px 2px rgba(0,0,0,0.05);
--shadow-md: 0 4px 6px rgba(0,0,0,0.07);
--shadow-lg: 0 10px 15px rgba(0,0,0,0.1);
--shadow-xl: 0 20px 25px rgba(0,0,0,0.1);
--shadow-card: 0 1px 3px rgba(0,0,0,0.08);
--shadow-elevated: 0 8px 30px rgba(0,0,0,0.12);
--shadow-modal: 0 25px 50px rgba(0,0,0,0.25);
```

### Motion / Animation
```css
/* Duration */
--duration-instant: 50ms;
--duration-fast: 100ms;
--duration-normal: 200ms;
--duration-slow: 400ms;
--duration-ceremonial: 800ms;
--duration-dramatic: 1200ms;

/* Easing */
--ease-default: cubic-bezier(0.4, 0, 0.2, 1);
--ease-in: cubic-bezier(0.4, 0, 1, 1);
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);
--ease-bounce: cubic-bezier(0.68, -0.55, 0.265, 1.55);

/* Named transitions */
--transition-colors: color, background-color, border-color var(--duration-fast) var(--ease-default);
--transition-transform: transform var(--duration-normal) var(--ease-spring);
--transition-opacity: opacity var(--duration-normal) var(--ease-default);
--transition-all: all var(--duration-normal) var(--ease-default);
```

---

## BUILD PHASES (Full Vision, Architecture-First)

### LAYER 1: INFRASTRUCTURE (Week 1-3)
**Everything that every other layer depends on.**

```
Week 1:
├── Scaffold Tauri 2.x + Nuxt 3 project
├── Configure SPA mode, pnpm, TypeScript strict
├── Install ALL dependencies (Tailwind 4, Pinia, KaTeX, D3, PixiJS, Howler, Motion One, jsPDF, Lucide)
├── Write ALL TypeScript types (types/*.ts) -- mirror every Rust DTO across all 21 crates
├── Write ALL Tauri IPC bindings (ipc/*.commands.ts) -- every command from every crate
├── Create useTauri.ts composable -- typed invoke wrapper with error handling
└── Verify IPC round-trip works (create account, authenticate, list accounts)

Week 2:
├── Complete design system CSS tokens (all colors, typography, spacing, motion, shadows)
├── Build ALL 40 ui/ primitive components
├── Implement 3 theme files (student.css, parent.css, admin.css)
├── Implement 6 emotional mode overrides (recovery, pressure, elite, game, celebration, focus)
├── Dark mode variant tokens
├── Print/PDF stylesheet
└── Font loading (Inter + JetBrains Mono)

Week 3:
├── Build ALL layout components (sidebars, shells, split pane, focus overlay, exam frame)
├── Auth flow complete (ProfileSwitcher, PinPad, role-based redirect)
├── ALL 4 layouts working (auth, student, parent, admin)
├── Student sidebar with ALL nav items (5 primary groups)
├── Parent sidebar with ALL nav items
├── Admin sidebar with ALL 8 sections
├── ALL Pinia stores scaffolded (auth, coach, session, student, curriculum, ui, diagnostic, game, glossary, calendar, upload)
├── ALL composables scaffolded with signatures (every useX.ts)
├── Routing structure complete -- ALL pages exist as stubs
└── Sound system initialized (useSound.ts + Howler integration)
```

**Deliverable**: The complete application skeleton. Every page route exists (as stubs). Every component directory exists. Every type is defined. Every IPC command is bound. Every store is scaffolded. You can navigate the entire app structure across all 3 portals.

---

### LAYER 2: CORE ENGINE COMPONENTS (Week 4-7)
**Build every reusable component to its full specification before assembling any page.**

```
Week 4: QUESTION SYSTEM (complete)
├── QuestionCard.vue orchestrator
├── ALL 15 question format components (MCQ through CanvasDrawQuestion)
├── QuestionStem.vue with KaTeX math rendering
├── QuestionOption.vue with tap/select interaction
├── QuestionTimer.vue (all 6 timer variants: soft, strict, shrinking, burst, pressure, cluster)
├── QuestionFlag.vue
├── ConfidenceCapture.vue (Sure / Not Sure / Guessed)
├── QuestionFeedback.vue (correct/incorrect with animation)
├── QuestionExplanation.vue (multi-layer progressive disclosure)
├── WrongAnswerReview.vue (10-part card, all expandable layers)
├── MistakeClinicFlow.vue (5-step coaching flow)
├── QuestionNav.vue (grid navigation for mocks/tests)
└── MathRenderer.vue (KaTeX wrapper)

Week 5: COACHING + SESSION SYSTEM (complete)
├── CoachHub.vue (renders based on CoachNextAction)
├── CoachDirectiveCard.vue (renders any directive type)
├── InsightCard.vue (topic insight with action buttons)
├── CoachNote.vue, CoachVoice.vue, CoachStateIndicator.vue
├── RescueDock.vue (7 help buttons)
├── SessionBrief.vue, WhyThisCard.vue
├── TodaysMission.vue
├── PhaseIndicator.vue
├── RecoveryBanner.vue
├── ExamCountdown.vue
├── SessionPlayer.vue (dynamic block renderer)
├── SessionBlock.vue, SessionProgress.vue, SessionTimer.vue
├── SessionPause.vue, SessionComplete.vue, BlockTransition.vue
├── WarmStartBlock.vue, SpeedBurstBlock.vue, ReflectionBlock.vue
└── SessionDebrief.vue (full post-session analysis)

Week 6: DATA VISUALIZATION (complete)
├── ALL 25 viz/ components
├── HeatMap, RadarChart, ProgressRing, TrendArrow, TrendLine, BarChart
├── MasteryBadge (8 states), ReadinessGauge, TopicStatusCard
├── StreakCounter, ComboCounter, ComparisonCard, GapRing
├── MemoryStrandViz, DecaySeverityBadge
├── KnowledgeMap (zoomable D3 graph), ConstellationView (force-directed)
├── TimelineBand, CurriculumTerrain
├── CoverageMatrix, PaperDNAChart, ErrorTypeChart
├── PressureProfile, TopicDominationMeter
└── AvailabilityGrid (weekly time-block picker)

Week 7: DIAGNOSTIC + MOCK + UPLOAD COMPONENTS (complete)
├── DiagnosticLauncher, DiagnosticPhasePlayer, PhaseTransition
├── DiagnosticReport (7 sections, all sub-components)
├── MockHome, MockSetup, MockPreFlight, MockExamHall
├── MockPacingBadge, MockSubmission, MockReview (6 tabs), MockHistory, MockForecast
├── UploadWizard, BundleCreator, PageClassifier
├── OcrConfidenceEditor, QuestionAligner, SmartReview
├── EssayAnalysis, UploadSummary
├── GlossarySearch, GlossaryEntryCard, GlossaryEntryPage
├── GlossaryDepthTabs, GlossaryInlinePanel, GlossaryCompare
├── FormulaDisplay, FormulaLab, ConceptMapView
├── AudioPlayer, MiniAudioPlayer, AudioRadioMode
└── ALL library components (LibraryHome through LibraryStateTag)
```

**Deliverable**: Every reusable component exists and works in isolation. You can render any question type, any coach directive, any diagnostic phase, any visualization, any mock state, any glossary entry, any upload step. Nothing is a stub.

---

### LAYER 3: MODE-SPECIFIC COMPONENTS (Week 8-10)
**Build the unique components for each learning mode.**

```
Week 8: BEAT YESTERDAY + ELITE + RISE + SPARK
├── BeatYesterdayHome (hero: yesterday vs today), DailyClimb (4-block), MicroGainIndicator, ClimbTrends
├── EliteHome (identity panel), EliteArena (7 sessions), EliteSession, EliteDebrief
├── EliteRecords, EliteInsights, TopicDominationBoard
├── RiseHome (4-stage view), WeaknessMapView, TransformationSession
├── SparkOnboarding (behavioral classification), SparkMission, SparkRewards

Week 9: JOURNEY + KNOWLEDGE GAP + MEMORY + TEACH
├── JourneyMap (visual path), JourneyStation, JourneyMission
├── GapScan, GapDashboard (6 sections), GapMapView, SolidificationSession
├── MemoryHome, MemoryScan, MemoryRescue, ChainRepair
├── TeachModePage (16 blocks), TeachBlock, WorkedExample

Week 10: GAMES + TRAPS + EXAM INTELLIGENCE
├── MindStack: Canvas (PixiJS), HUD, Question, MorphMenu, PowerUps, GameOver, ModeSelect
├── TugOfWar: Canvas, Question, PowerUps, Result
├── Traps: Hub, DifferenceDrill (card-sort), TrapCard, ConceptBin
├── SimilarityTrap, KnowTheDifference, WhichIsWhich, UnmaskMode, TrapsReview
├── ExamIntelHome, QuestionFamilyCard, QuestionDNACard, FamilyTreeView
├── EvolutionTimeline, PaperDNADashboard, StorylineView, InverseMap
├── ExamReplayMode, StudentMirrorView
└── Sound integration across all game modes
```

**Deliverable**: Every mode-specific component exists. The full component library is complete.

---

### LAYER 4: PARENT + ADMIN PORTAL COMPONENTS (Week 11-12)
**Build every component for the other two portals.**

```
Week 11: PARENT PORTAL (complete)
├── ParentHome, ChildCard, ChildDetail
├── ParentInsightCard, RiskAlert, WeeklyMemo
├── AttentionNeeded, InterventionCenter (3-column)
├── ParentReport (printable), ParentConcierge (Q&A)
├── ParentTimeline, ParentSubjectRisk, ReadinessBrief
├── EvidenceViewer, ParentSettings
└── PDF export for all parent reports

Week 12: ADMIN PORTAL (complete)
├── AdminCommandCenter, CurriculumUpload, CurriculumReviewWorkbench (split-pane)
├── CurriculumTreeEditor, CurriculumDiffViewer
├── QuestionAuthor, QuestionReviewConsole, QuestionFamilyBrowser
├── ContentPipelineBoard (kanban), ContentCoverageHeatmap
├── StudentMonitor, PackManager, UserManager
├── QualityDashboard, AuditLog, CoachTuning
├── SystemHealth, EntitlementManager, TimelineManager, PublishCenter
└── All admin-specific table/filter/action patterns
```

**Deliverable**: All components for all 3 portals are built.

---

### LAYER 5: PAGE ASSEMBLY -- STUDENT PORTAL (Week 13-20)
**Wire components to pages. Connect IPC. Full state management. Every page complete.**

```
Week 13-14: CORE FLOW
├── student/index.vue (Coach Hub -- CoachNextAction rendering for all 14 states)
├── student/onboarding/* (4 pages: welcome, subjects, content-packs, diagnostic)
├── student/session/[id].vue (live session player with full lifecycle)
├── student/session/debrief/[id].vue (post-session debrief)
├── student/practice/* (practice hub, custom test, topic practice)
├── student/settings.vue

Week 15-16: ASSESSMENT
├── student/diagnostic/* (launcher, live phases, multi-phase transitions, 7-section report)
├── student/mock/* (centre home, setup, exam hall, 6-tab review, history, forecast)
├── student/mistakes/* (mistake lab, error patterns)

Week 17-18: PROGRESS + MODES
├── student/progress/* (mastery overview, mastery map, analytics, history)
├── student/beat-yesterday/* (daily climb with 4 blocks)
├── student/journey/* (journey map, stations)
├── student/elite/* (home, arena, session, records, domination, insights)
├── student/rise/* (transformation home, sessions)
├── student/spark/* (onboarding, missions)
├── student/knowledge-gap/* (scan, dashboard, solidification)
├── student/memory/* (home, sessions)
├── student/calendar/* (academic calendar, 5-layer view)

Week 19-20: CONTENT + GAMES
├── student/glossary/* (lab home, entry pages, compare, audio radio, formula lab, test lab)
├── student/library/* (home, shelves, pack builder)
├── student/teach/[topicId].vue (16-block teaching)
├── student/exam-intel/* (home, families, paper DNA, constellation, storylines, replay)
├── student/games/* (hub, mindstack play, tugofwar play, traps play)
├── student/upload/* (upload wizard, smart review)
├── student/focus.vue (focus mode)
└── ALL student pages fully wired with real data from IPC
```

**Deliverable**: Complete student portal. Every page works with real backend data. Every flow (onboarding → diagnostic → coaching → practice → mock → review → progress) is end-to-end functional.

---

### LAYER 6: PAGE ASSEMBLY -- PARENT + ADMIN PORTALS (Week 21-24)
**Wire remaining portal pages.**

```
Week 21-22: PARENT PORTAL
├── parent/index.vue (family overview)
├── parent/child/[id].vue (per-child dashboard)
├── parent/performance.vue, parent/attention.vue
├── parent/reports.vue (with PDF export)
├── parent/intervention.vue (premium)
├── parent/concierge.vue (premium Q&A)
├── parent/timeline.vue, parent/upload.vue
├── parent/curriculum.vue, parent/settings.vue
└── ALL parent pages wired with real data + parent-specific copy

Week 23-24: ADMIN PORTAL
├── admin/index.vue (command center)
├── admin/curriculum/* (upload, review workbench, tree editor, graph, coverage, diff, publish)
├── admin/questions/* (browser, author, review, families)
├── admin/content/* (pipeline, coverage, quality)
├── admin/students/* (monitor, individual detail)
├── admin/packs/*, admin/users.vue, admin/entitlements.vue
├── admin/timeline.vue, admin/quality.vue, admin/audit.vue
├── admin/coach-tuning.vue, admin/health.vue, admin/settings.vue
└── ALL admin pages wired with real data
```

**Deliverable**: All 3 portals are complete and functional.

---

### LAYER 7: INTEGRATION + CROSS-CUTTING (Week 25-28)
**Everything that spans across features.**

```
Week 25-26: CROSS-FEATURE INTEGRATION
├── Coach-driven navigation (coach recommends → user taps → correct mode opens pre-configured)
├── Cross-mode transitions (diagnostic reveals weakness → routes to knowledge gap → routes to repair session)
├── Glossary inline panel working across ALL pages (tap any term anywhere)
├── Ask Tutor / Why Engine as global slide-out panel
├── Notification system (decay alerts, session nudges, milestone celebrations, parent risk flags)
├── Time orchestration integration (schedule view, "Free Now", session awareness, auto-trigger)
├── Academic calendar integration (exam dates drive coach behavior, weekly/daily plans)
├── Goal system integration (goals visible on Coach Hub, progress tracked)
├── Evidence intake → coach reaction → plan update → parent notification flow
├── Premium tier gating (locked features show upgrade path, not hidden)
├── Elite tier gating

Week 27-28: POLISH + ACCESSIBILITY + PERFORMANCE
├── Animation choreography (page transitions, card expand/collapse, celebrations, mode shifts)
├── Sound design integration across all modes and interactions
├── WCAG AA audit + fixes (keyboard nav, screen reader, color contrast, reduced motion)
├── Color-blind safe alternatives for all status colors
├── Configurable timer allowances for accessibility
├── Performance optimization (lazy loading routes, virtual scrolling for long lists)
├── Image/asset optimization
├── Memory leak audit (especially for game canvases)
├── Offline behavior verification (all features work without network)
├── Content pack download/management UX
├── Error boundary components (graceful failure states for every surface)
├── "Uncertainty UX" implementation (stale data, thin evidence, contradictions)
├── Light/dark mode toggle tested across all surfaces
├── PDF export tested for all report types
└── Final copy review (student/parent/admin voice consistency)
```

**Deliverable**: Complete, polished, accessible application. All features integrated. All cross-cutting concerns handled. Production-ready.

---

## IMPLEMENTATION RULES

1. **Frontend is presentation-only.** All intelligence lives in Rust. Never compute mastery, readiness, gap scores, or recommendations in Vue/TypeScript.

2. **BasisPoints everywhere.** Scores are 0-10000 integers. Convert to display format only in `utils/format.ts`: `formatScore(bp) → "72%"`. Never store percentages.

3. **Coach drives the student UI.** The Coach Hub calls `resolve_next_coach_action()` and renders whatever it returns. The student does not browse a feature menu; the coach recommends the next action.

4. **One question component rules all.** `QuestionCard.vue` accepts a `Question` + `QuestionOption[]` and routes to the correct format renderer. Every feature that shows questions uses this one component.

5. **Role determines the entire shell.** After authentication, the app loads a completely different layout, sidebar, component set, and theme. No conditional rendering within shared layouts.

6. **Copy is product.** Every user-facing string uses the audience translation layer. The same underlying data produces different copy for students ("You're rebuilding ratio understanding"), parents ("Your child's ratio skills are recovering after targeted intervention"), and admins ("Topic 4.2.1 mastery_score: 3200bp, trend: improving, intervention: active").

7. **Progressive disclosure everywhere.** Dense data surfaces show the headline first. Details appear on click/expand. The wrong answer review card has 10 layers but shows 2 by default.

8. **Emotional theming is not optional.** Recovery mode feels warm. Pressure mode feels intense. Elite feels premium. Games feel vibrant. The CSS mode classes must be applied to every relevant surface.

9. **Offline is the default.** No loading spinners for data fetches (all local). Show progress indicators only for heavy operations (OCR, content generation, pack installation). Show connectivity status only for sync/download features.

10. **Every state is designed.** Empty states, error states, loading states, uncertain states, stale-data states, no-content states, locked-feature states -- every possible condition has a designed UI, not a blank screen or generic error.

11. **Build complete, not fast.** Every component is built to its full specification the first time. No "simplified version for now" shortcuts. If the wrong answer review needs 10 layers, build 10 layers.

12. **Sound is a first-class design element.** Correct/wrong feedback, streak building, mode transitions, celebrations, pressure heartbeat, game SFX -- all part of the designed experience, not afterthoughts.

---

## TIMELINE SUMMARY

| Layer | What | Weeks | Cumulative |
|-------|------|-------|------------|
| Layer 1 | Infrastructure (scaffold, types, IPC, design system, layouts, auth) | 3 | Week 3 |
| Layer 2 | Core engine components (question, coach, session, viz, diagnostic, mock, upload, glossary) | 4 | Week 7 |
| Layer 3 | Mode-specific components (beat yesterday, elite, rise, spark, journey, gap, memory, teach, games, traps, exam intel) | 3 | Week 10 |
| Layer 4 | Parent + admin portal components | 2 | Week 12 |
| Layer 5 | Student portal page assembly (all ~50 routes wired) | 8 | Week 20 |
| Layer 6 | Parent + admin page assembly (all ~32 routes wired) | 4 | Week 24 |
| Layer 7 | Cross-cutting integration, polish, accessibility, performance | 4 | Week 28 |

**Total: 28 weeks for the complete vision.**

This is faster than the MVP approach (40 weeks) because:
- No rework. Components are built right the first time.
- No refactoring. Architecture supports everything from day 1.
- No "connect the dots" phase. Types and IPC are complete upfront.
- Parallel work is possible. Components are independent. Pages are independent.
