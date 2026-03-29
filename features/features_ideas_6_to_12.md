# Features Extracted from Ideas 6-12

Good, I've now read every line of all three files. Here is the exhaustive feature extraction.

---

**NOTE: idea7.txt and idea8.txt are byte-for-byte identical (confirmed via diff). Features are extracted once and attributed to both.**

---

## FEATURES FOUND IN idea6.txt

---

- **Feature Name**: Knowledge Gap Model (10-Layer Gap Taxonomy)
- **Type**: engine-output-needing-UI
- **Description**: A classification system for student knowledge gaps with 10 distinct categories: Content gap, Understanding gap, Application gap, Process gap, Speed gap, Accuracy gap, Retention gap, Confidence gap, Interest gap, Transfer gap. Each gap is modeled as Topic -> Subtopic -> Skill -> Error pattern -> Cause -> Fix path.
- **User Role**: student
- **UI Details**: None specified at this level (internal classification)
- **Sub-features**: Content gap, Understanding gap, Application gap, Process gap, Speed gap, Accuracy gap, Retention gap, Confidence gap, Interest gap, Transfer gap

---

- **Feature Name**: Knowledge Gap Mode ("Teach Me What I Don't Know Yet")
- **Type**: mode
- **Description**: A living system inside the app that continuously detects what the student has not learned, what they think they know but don't truly know, what they knew before but are forgetting, and what they can do slowly/weakly/inconsistently. The student's mission is to reduce their knowledge gap to zero.
- **User Role**: student
- **UI Details**: Mode names suggested: "Show Me What I Don't Know", "Teach Me What I Don't Know Yet", "My Knowledge Gap", "Gap Finder", "Zero Gap Mode", "Unknowns Mode", "What You're Missing". Buttons: "Scan My Mind", "Reveal My Gaps", "Fix My Weak Spots", "Start Closing Gaps", "Show Missing Skills"
- **Sub-features**: Gap Detection Engine, Gap Repair Engine

---

- **Feature Name**: Knowledge Gap Score Display
- **Type**: data-visualization
- **Description**: A central numeric display showing the student's knowledge gap with multiple metrics including gap percentage remaining, skills missing count, critical gaps count, weak zones count, and mastered-this-month count. Can be shown as Coverage % vs Gap % dual display.
- **User Role**: student
- **UI Details**: Shows both Coverage (e.g., 28%) and Gap (e.g., 72%) for progress and urgency. Example: "72% gap remaining, 41 skills missing, 12 critical gaps, 7 weak zones, 22 mastered this month"
- **Sub-features**: None

---

- **Feature Name**: Knowledge Gap Home Screen
- **Type**: screen
- **Description**: The landing screen when the student opens Knowledge Gap Mode. Shows current gap percentage, critical gaps count, hidden gaps count, recently fixed count, biggest holes list, what to fix first (ranked path), and zero-gap path.
- **User Role**: student
- **UI Details**: Shows: "Total knowledge gap: 46%, Critical gaps: 8, Hidden gaps: 13, Recently fixed: 5". Fix priority: "Fix this now / Fix this next / Leave this for later". Zero-gap path: "3 quick wins, 2 foundational repairs, 1 danger zone"
- **Sub-features**: Biggest holes list, Fix-first ranked path, Zero-gap path

---

- **Feature Name**: Gap Type Tracking Engine
- **Type**: engine-output-needing-UI
- **Description**: Tracks 10 categories of gaps: Unknown gap, Fragile gap, False mastery gap, Retention gap, Speed gap, Accuracy gap, Foundation gap, Transfer gap, Confidence gap, Interest gap. Each gap type is tracked differently from just "wrong answers."
- **User Role**: student
- **UI Details**: None specific (internal engine driving UI labels)
- **Sub-features**: Unknown gap, Fragile gap, False mastery gap, Retention gap, Speed gap, Accuracy gap, Foundation gap, Transfer gap, Confidence gap, Interest gap

---

- **Feature Name**: Hidden Knowledge Gap Discovery
- **Type**: engine-output-needing-UI
- **Description**: The system uncovers hidden gaps where the student's self-diagnosis is wrong. Example: student says "I am bad at algebra" but the true issue is weak integers, poor equation balancing, confusion around variable substitution, low reading accuracy in word problems. Acts as a diagnostic intelligence layer.
- **User Role**: student
- **UI Details**: System says things like: "You are not mainly failing algebra. Your real issue is weak number operations and first-step breakdown."
- **Sub-features**: None

---

- **Feature Name**: Gap Detection Methods
- **Type**: engine-output-needing-UI
- **Description**: Multiple methods for finding knowledge gaps: Diagnostic questions (targeted question sets), Continuous passive detection (updates gap map from all app interactions), Step analysis (inspects where student broke in solution path), Error pattern tracking, Time analysis (hesitation detection), Confidence check ("How sure are you?"), Variation testing (same concept in different forms).
- **User Role**: student
- **UI Details**: Confidence check prompt: "How sure are you?" displayed after questions
- **Sub-features**: Diagnostic questions, Continuous passive detection, Step analysis, Error pattern tracking, Time analysis, Confidence check, Variation testing

---

- **Feature Name**: Gap Scan Flow ("Show Me What I Don't Know")
- **Type**: flow
- **Description**: A 4-step student flow: Step 1 - Scan me (student taps button, app runs adaptive scan), Step 2 - Reveal the gaps (shows what is missing, why it matters, severity, cause), Step 3 - Fix mode (teach simply, worked example, guided questions, test, confirm repair), Step 4 - Update the map (gap bar shrinks in real time).
- **User Role**: student
- **UI Details**: Shrinking gap bar animation described as "addictive". Visual identity options: Knowledge map (dark/illuminated zones), Hole repair (wall with cracks), Fog clearing, Brain grid (matrix of knowledge cells), Journey path (broken bridges). Fog clearing or knowledge map metaphor recommended.
- **Sub-features**: Scan, Reveal, Fix, Update

---

- **Feature Name**: Knowledge Map Visualization
- **Type**: data-visualization
- **Description**: Visual metaphor for the student's knowledge state. Multiple options described: knowledge map with dark/light zones, hole repair metaphor with cracks in wall, fog clearing metaphor, brain grid with filling cells, journey path with broken bridges.
- **User Role**: student
- **UI Details**: Color coding: red = critical, orange = weak, yellow = fragile, blue = stable, green = mastered, purple outline = slipping/declining, gray fog = unknown/unmapped. Gentle gradients, not harsh alarming tones. Map + fog-clearing hybrid recommended.
- **Sub-features**: None

---

- **Feature Name**: Gap-Closing Loop
- **Type**: interaction
- **Description**: Core engine loop: Detect -> Explain -> Repair -> Recheck -> Lock in -> Expand. In plain words: Find what is missing, Teach it, Test it, Confirm it is fixed, Revisit later so it stays fixed, Move to the next gap. Also expressed as: Detect -> Rank -> Teach -> Practice -> Recheck -> Shrink.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Gap Progress Tracking
- **Type**: data-visualization
- **Description**: Tracks current active gap, total mapped gap, gap closed this week, zero-gap streak in mastered topics. Shows weekly trends like "Last week: 51%, Today: 43%, Critical gaps fixed: 2, New hidden gaps found: 1, Net improvement: strong."
- **User Role**: student
- **UI Details**: Visual shrinking: fog clearing from subject map, bar reducing from 60% to 48%, weak zones turning red to yellow to green, "3 gaps closed this week", "1 hidden blocker removed"
- **Sub-features**: None

---

- **Feature Name**: Addictive Session Feedback
- **Type**: interaction
- **Description**: Every session should visibly accomplish one of: uncover a new hidden weakness, close a visible gap, increase coverage %, reduce critical gaps, unlock a mastery badge, clear fog from the map, move closer to exam readiness.
- **User Role**: student
- **UI Details**: None specified beyond the gap shrinking animations
- **Sub-features**: Mastery badge unlocks, Fog clearing animation

---

- **Feature Name**: Critical Gap Ranking
- **Type**: engine-output-needing-UI
- **Description**: Not all gaps matter equally. Some gaps block many future topics. Those should be fixed first. Gaps are tagged by exam importance: high-frequency exam gap, foundational gap, advanced gap, low-priority gap.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: Exam relevance tagging

---

- **Feature Name**: Dependency Logic / Prerequisite Chains
- **Type**: engine-output-needing-UI
- **Description**: The app knows prerequisite chains so weakness in subtraction can be identified as affecting algebra, fractions, and ratios. Powers the "hidden blocker" detection.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Confidence Repair System
- **Type**: interaction
- **Description**: If a student has trauma around a topic, the system lowers difficulty and rebuilds belief gradually.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Daily Minimum Gap Reduction Target
- **Type**: setting
- **Description**: Sets daily targets like "reduce 1% today", "close 2 micro-gaps today", "repair 1 critical concept today."
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Teacher/Parent Gap View
- **Type**: report
- **Description**: Shows biggest active gaps, newly fixed gaps, stubborn recurring gaps. Example: "Victor's main blocker is not algebra itself. It is weak negative-number handling, which is affecting algebra and graphs."
- **User Role**: parent | teacher
- **UI Details**: Dashboard showing overall gap, critical blockers, recurring weaknesses, recent gains, slipping skills, intervention adherence
- **Sub-features**: None

---

- **Feature Name**: Empowering Microcopy / Emotional Framing
- **Type**: interaction
- **Description**: Language system that avoids making students feel dumb. Uses phrases like "Here's what we'll unlock next", "Here are your missing pieces", "You're closer than you think". Avoids "You are bad at this", "You failed this topic."
- **User Role**: student
- **UI Details**: Specific copy examples throughout the mode
- **Sub-features**: None

---

- **Feature Name**: Skill State Machine
- **Type**: engine-output-needing-UI
- **Description**: Each skill lives in a state machine with states: Unknown, Fragile, Weak, Declining, Critical, Repairing, Stable, Mastered, At-risk mastered. Includes movement tracking (stable, improving, fragile, declining, critical, repaired).
- **User Role**: student
- **UI Details**: State badges displayed on skills
- **Sub-features**: None

---

- **Feature Name**: Knowledge Gap Priority Engine
- **Type**: engine-output-needing-UI
- **Description**: Every skill has two separate values: Gap Score (how incomplete) and Priority Score (how urgently to fix). Priority formula: Priority = 0.30*Gap + 0.20*TrendRisk + 0.15*DependencyImpact + 0.15*ExamWeight + 0.10*Recurrence + 0.05*ForgettingRisk + 0.05*MisconceptionPenalty. Priority levels: Low (0-24), Watch (25-49), Active repair (50-69), Urgent solidify (70-84), Critical blocker (85-100).
- **User Role**: student
- **UI Details**: Priority cards showing urgency and recommended action
- **Sub-features**: Gap Score, Priority Score, Trend Risk, Dependency Impact, Exam Weight, Recurrence, Forgetting Risk, Misconception Penalty

---

- **Feature Name**: Mastery Score Formula
- **Type**: engine-output-needing-UI
- **Description**: Mastery = 0.35*Accuracy + 0.15*Speed + 0.10*Confidence + 0.15*Retention + 0.15*Transfer + 0.10*Consistency. Gap = 100 - Mastery. Simplified version: Mastery = 0.4*Accuracy + 0.2*Retention + 0.15*Transfer + 0.15*Speed + 0.1*Confidence.
- **User Role**: student
- **UI Details**: Displayed as mastery percentage per skill
- **Sub-features**: Accuracy signal, Speed signal, Confidence signal, Trend signal, Retention signal, Misconception signal, Transfer signal, Dependency signal, Exam relevance signal, Recurrence signal

---

- **Feature Name**: Force Solidification Triggers
- **Type**: engine-output-needing-UI
- **Description**: Automatic reinforcement triggers: consecutive failure (2-3 times in short window), declining trend, dependency damage, exam urgency, recurrence (keeps relapsing), slow mastery (correct but too slow), confidence collapse (right answers with high hesitation).
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: 7 trigger rules

---

- **Feature Name**: Real-Time Knowledge Gap Updates
- **Type**: interaction
- **Description**: Knowledge gap updates after every meaningful interaction: answering a question, choosing wrong option, taking too long, abandoning a question, asking for too many hints, failing a transfer version, succeeding independently after scaffolding, forgetting on later review.
- **User Role**: student
- **UI Details**: Live update on screen
- **Sub-features**: None

---

- **Feature Name**: Intervention Sequence (Solidification)
- **Type**: flow
- **Description**: 8-step intervention when a skill becomes urgent: Diagnose precisely, Pull prerequisite check, Teach briefly, Guided examples, Targeted practice, Variation test, Timed recheck, Lock or continue.
- **User Role**: student
- **UI Details**: None specified at this point (detailed later in frontend flow)
- **Sub-features**: None

---

- **Feature Name**: Gap Repair Queue
- **Type**: engine-output-needing-UI
- **Description**: System maintains a repair queue ordered by: Repair Queue Rank = Priority Score + Readiness Bonus + Momentum Bonus - Fatigue Penalty. Prevents oppressive repetition.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: Readiness Bonus, Momentum Bonus, Fatigue Penalty

---

- **Feature Name**: Knowledge Gap Display (Structured View)
- **Type**: data-visualization
- **Description**: Structured gap display showing main number (Knowledge Gap: 38%), Breakdown (Unknown: 11%, Weak: 12%, Declining: 7%, Forgetting: 4%, Critical blockers: 4%), and Priority panel (Fix now: Fractions, Watch closely: Graph interpretation, Stable again: Sentence transformation).
- **User Role**: student
- **UI Details**: Specific layout described
- **Sub-features**: None

---

- **Feature Name**: Gap Shrinking Logic
- **Type**: engine-output-needing-UI
- **Description**: Gap shrinks only when real evidence appears (not from watching lessons). Repair levels: explanation viewed = no reduction, guided correct = slight, independent correct = medium, repeated correct across variation = major, retained later = long-term reduction. Prevents fake mastery.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Solidification Strategy Types
- **Type**: engine-output-needing-UI
- **Description**: Different repair styles for different gap types: Unknown skills (teach from scratch), Weak skills (target breakdown points), Declining skills (reinforce quickly and retest), Forgetting skills (spaced retrieval), Misconception-heavy skills (confront wrong mental model), Speed gaps (timed repetition after accuracy stabilizes).
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: 6 strategy types

---

- **Feature Name**: Exam-Proximity Priority Adjustment
- **Type**: engine-output-needing-UI
- **Description**: As exams approach, priority weights change. Increases exam weight, time sensitivity, speed pressure. Formula adjusts to: PriorityExam = 0.25G + 0.20TR + 0.15DI + 0.25EW + 0.10RC + 0.05FR.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Personalized Priority Weighting
- **Type**: setting
- **Description**: Different students get different weighting profiles. Anxious student: increase confidence/speed weighting. Careless student: increase consistency/error pattern weighting. Weak foundation: increase dependency impact weighting. Exam crammer: increase exam relevance/trend weighting.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: 4 student type profiles

---

- **Feature Name**: Knowledge Gap Mode Entry Points
- **Type**: interaction
- **Description**: Student can enter the mode from: Home dashboard card, After a mock exam prompt, After repeated mistakes in practice, Study planner suggestion.
- **User Role**: student
- **UI Details**: Home card: "Knowledge Gap: 42%, 3 urgent areas need solidifying", CTA: "Open Gap Mode". After mock: "We found 7 weak zones and 3 hidden gaps. Want us to fix them?", CTA: "Start Gap Repair". After mistakes: "You're slipping in Fractions. Lock it down now.", CTA: "Solidify This Area". Planner: "Today's best move: reduce your gap by 2%"
- **Sub-features**: None

---

- **Feature Name**: First-Time Onboarding Flow (Knowledge Gap Mode)
- **Type**: flow
- **Description**: 3-screen onboarding: Screen 1 - "We'll show you what you haven't mastered yet" with foggy map clearing visual. Screen 2 - "Your gap shrinks as you prove mastery" with bar animation visual. Screen 3 - "You'll always know what to fix next" with bullets: Missing skills, Weak skills, Slipping skills, Critical blockers, Best next repair path.
- **User Role**: student
- **UI Details**: Each screen has specific title, text, visual, and CTA. Final CTA: "Scan My Knowledge"
- **Sub-features**: None

---

- **Feature Name**: Initial Scan Screen
- **Type**: screen
- **Description**: First active diagnostic screen. Runs 10-15 quick adaptive questions in 3-5 minutes. Shows animated subject map or brain grid with unknown cells. Updates as student answers.
- **User Role**: student
- **UI Details**: Top: "Teach Me What I Don't Know Yet". Subtext: "We're mapping what you know, what is weak, and what needs repair first." Info chips: "10-15 quick questions, adaptive difficulty, takes about 3-5 minutes, updates as you answer." CTAs: "Start Scan" and "Use My Past Performance Instead"
- **Sub-features**: None

---

- **Feature Name**: Live Scan Experience
- **Type**: screen
- **Description**: Feels like an intelligent probe, not a normal quiz. Shows real-time discoveries during scanning like "You seem strong in basic arithmetic" and "Equivalent fractions may need reinforcement."
- **User Role**: student
- **UI Details**: Header: "Scanning your knowledge..." Top progress strip: "7 skills mapped, 2 weaknesses detected, 1 hidden blocker suspected." Question card: large clean question, minimal distraction. Bottom: confidence toggle (Not sure/Somewhat sure/Very sure), optional hint, skip button.
- **Sub-features**: Real-time side discovery messages

---

- **Feature Name**: Scan Results Reveal Screen
- **Type**: screen
- **Description**: Major emotional moment showing scan results with big headline "Here's what we found", main card showing Knowledge Gap percentage, breakdown cards (Unknown, Weak, Declining, Forgetting, Critical blockers), and a dynamic color-coded map.
- **User Role**: student
- **UI Details**: Color coding: red = critical, orange = weak, yellow = fragile, blue = stable, green = mastered, purple outline = slipping/declining. CTAs: "Start Closing My Gaps" and "See My Full Gap Map"
- **Sub-features**: None

---

- **Feature Name**: Main Knowledge Gap Dashboard
- **Type**: screen
- **Description**: Core home of the mode, a personal academic control panel. Top hero section with main metric (Knowledge Gap %), secondary metrics (Coverage, Critical blockers, Weak skills, Slipping areas, Fixed this week). Contains 6 sections: Fix Now, What You Don't Know Yet, Weak But Improving, Slipping Areas, Hidden Blockers, Recent Shrinkage.
- **User Role**: student
- **UI Details**: Gap ring animates subtly when updated. Section A (Fix Now): Priority cards with severity, why it matters, what it affects, action button. Section B (Unknown): "Learn from zero" CTA. Section C (Weak but improving): before/after trend, confidence gain, mastery progress, "Keep building" CTA. Section D (Slipping): "Was stable 2 weeks ago, Now accuracy dropped", "Solidify Again" CTA. Section E (Hidden blockers): "You seem weak in ratios, but the real blocker is equivalent fractions", "Fix Root Cause" CTA. Section F (Recent shrinkage): trend graph/animated shrink bar.
- **Sub-features**: Fix Now panel, What You Don't Know Yet panel, Weak But Improving panel, Slipping Areas panel, Hidden Blockers panel, Recent Shrinkage panel

---

- **Feature Name**: Full Gap Map Screen
- **Type**: screen
- **Description**: Detailed exploration view showing full structure of subject knowledge. Three visual options: Subject map (topic clusters connected by lines), Grid map (cells for topics/subtopics/skills), Journey map (roads, bridges, zones, fog). Supports filters: all, unknown, weak, slipping, critical, mastered, exam-important. Tap any topic to expand into subtopics, micro-skills, status, trend, related dependencies.
- **User Role**: student
- **UI Details**: Map + fog-clearing hybrid recommended
- **Sub-features**: Filters, Topic expansion drill-down

---

- **Feature Name**: Skill Detail Screen
- **Type**: screen
- **Description**: Explains a specific weakness clearly. Shows status badge (e.g., "Critical Blocker"), reason ("You are missing this skill, and it is affecting fractions, ratios, and percentage questions"), insight block (what the skill is, why student is weak, evidence used, what improves if fixed), performance evidence (recent accuracy, average time, confidence, recurrence, last stable date), related impact on other skills.
- **User Role**: student
- **UI Details**: CTAs: "Start Solidification Session" and "See Example Mistakes"
- **Sub-features**: Insight block, Performance evidence block, Related impact block

---

- **Feature Name**: Example Mistake Screen
- **Type**: screen
- **Description**: Shows students exactly how they are going wrong. Displays their most common wrong pattern with a system note explaining the misconception, then shows the correct approach.
- **User Role**: student
- **UI Details**: Example: "You often do this: 1/2 + 1/3 = 2/5. System note: You are adding numerators and denominators directly. That is the misconception." Then: "What should happen instead: Find a common denominator first." CTA: "Teach Me Properly"
- **Sub-features**: None

---

- **Feature Name**: Solidification Session Screen
- **Type**: screen
- **Description**: 5-stage focused repair experience that feels like "focused academic therapy": Stage 1 - Quick diagnosis, Stage 2 - Mini teaching (short explanation, visual example, maybe animation), Stage 3 - Guided practice (scaffolded support), Stage 4 - Independent proof, Stage 5 - Variation test.
- **User Role**: student
- **UI Details**: Header: "Solidifying: Equivalent Fractions". Subtext: "Goal: make this skill stable and usable under pressure." Progress strip: diagnose -> teach -> guide -> prove -> lock in. Live reinforcement messages. Bottom: gap effect preview ("Stabilizing this skill can reduce your Mathematics gap by up to 2% and improve fractions, ratios, and percentages")
- **Sub-features**: 5 stages: Diagnose, Teach, Guide, Prove, Lock in

---

- **Feature Name**: Post-Session Result Screen
- **Type**: screen
- **Description**: Rewarding but credible result display. Shows skills stabilized, updated gap numbers (before -> after), state changes (e.g., critical -> fragile), visual fog clearing or color changes, insights ("You solved 4 in a row correctly", "You handled a new variation", "You still need a later recheck to lock it in fully").
- **User Role**: student
- **UI Details**: CTAs: "Fix the next urgent gap", "Reinforce this later", "See what improved"
- **Sub-features**: None

---

- **Feature Name**: Real-Time Slipping Alert
- **Type**: notification
- **Description**: Proactive alert when the engine detects decline. Shows title "This area is starting to slip", body explaining what is weakening and what it is affecting, with a pulsing skill card or downward trend arrow.
- **User Role**: student
- **UI Details**: Actions: "Solidify now", "Add to today's repair plan", "Remind me later"
- **Sub-features**: None

---

- **Feature Name**: Daily Gap Plan Screen
- **Type**: screen
- **Description**: System creates a daily plan around gap reduction. Shows tasks like "fix 1 critical blocker, reinforce 2 slipping areas, protect 1 strong topic from decay." Each task card shows topic, urgency, estimated time, expected impact.
- **User Role**: student
- **UI Details**: Title: "Today's Gap Reduction Plan". Estimated result: "Complete this plan and you may reduce your active gap by 2-3%". CTA: "Start Today's Plan"
- **Sub-features**: None

---

- **Feature Name**: Weekly Gap Review Screen
- **Type**: report
- **Description**: Shows gap last week vs now, skills fixed, skills that slipped, strongest improvements, recurring danger zones. Includes honest messaging like "You are reducing your gap, but two repaired areas are becoming unstable again."
- **User Role**: student
- **UI Details**: Example: "Gap: 51% -> 44%, 4 critical skills repaired, 2 topics relapsed, biggest improvement: algebraic substitution, next danger zone: graph interpretation"
- **Sub-features**: None

---

- **Feature Name**: Gap Ring Component
- **Type**: component
- **Description**: Circular progress UI showing current gap percentage. Reusable throughout the mode.
- **User Role**: student
- **UI Details**: Circular ring, animates on update
- **Sub-features**: None

---

- **Feature Name**: Priority Cards Component
- **Type**: component
- **Description**: Cards showing urgency level and recommended action for each gap. Reusable throughout the mode.
- **User Role**: student
- **UI Details**: Shows urgency and action buttons
- **Sub-features**: None

---

- **Feature Name**: Trend Arrows Component
- **Type**: component
- **Description**: Visual arrows showing rising mastery, flat, or declining trend for each skill.
- **User Role**: student
- **UI Details**: Arrow directions indicating trend
- **Sub-features**: None

---

- **Feature Name**: Skill State Badges Component
- **Type**: component
- **Description**: Badges showing current state: unknown, weak, fragile, declining, critical, stable, mastered.
- **User Role**: student
- **UI Details**: Badge labels
- **Sub-features**: None

---

- **Feature Name**: Dependency Tags Component
- **Type**: component
- **Description**: Tags showing "This affects 3 other skills" on skill cards.
- **User Role**: student
- **UI Details**: Tag label with count
- **Sub-features**: None

---

- **Feature Name**: Shrink Animation
- **Type**: animation
- **Description**: When the student improves, the map should visually tighten/clear/heal. Gap shrinking animation for fog clearing, bar reducing, colors changing.
- **User Role**: student
- **UI Details**: Fog clears, red turns orange/yellow, bar reduces
- **Sub-features**: None

---

- **Feature Name**: Root Cause Insight Card Component
- **Type**: component
- **Description**: Card displaying "Your issue here is actually coming from an earlier skill" with observed failure area, root cause, and linked downstream topics.
- **User Role**: student
- **UI Details**: Shows root cause chain
- **Sub-features**: None

---

- **Feature Name**: Color and Emotional Logic System
- **Type**: interaction
- **Description**: Consistent color system across the mode: red = urgent/critical, orange = weak/active repair, yellow = fragile, blue = stable, green = mastered, purple = slipping/declining, gray fog = unknown/unmapped. Uses gentle gradients.
- **User Role**: student
- **UI Details**: Specific color assignments
- **Sub-features**: None

---

- **Feature Name**: Gamification (Knowledge Gap Mode)
- **Type**: interaction
- **Description**: Gap shrinking animation, "fixed" badges, streak for daily reduction, "3 hidden blockers uncovered", "foundation repaired", "weak zone stabilized." Explicitly avoids childish gamification (no excessive coins, noisy rewards, meaningless badges, distracting confetti).
- **User Role**: student
- **UI Details**: Badges and streak displays
- **Sub-features**: Gap shrinking animation, Fixed badges, Daily reduction streak, Hidden blocker uncovered notification, Foundation repaired notification, Weak zone stabilized notification

---

- **Feature Name**: Real-Time Background Updating
- **Type**: interaction
- **Description**: Knowledge Gap Mode updates silently in the background while student uses any part of the app. When opened, shows activity feed: "Fractions moved to urgent solidify", "Graph interpretation improved", "A hidden root cause was found in decimals."
- **User Role**: student
- **UI Details**: Activity feed: "Recent knowledge updates" with specific skill changes listed
- **Sub-features**: None

---

- **Feature Name**: Knowledge Gap Mode Navigation Structure
- **Type**: component
- **Description**: Tab navigation inside the mode: Overview (main dashboard), Gap Map (full subject/skill structure), Fix Now (priority repair queue), Slipping (declining and at-risk skills), Progress (trend, shrink history, repaired gaps).
- **User Role**: student
- **UI Details**: Bottom tab or top tab layout
- **Sub-features**: Overview tab, Gap Map tab, Fix Now tab, Slipping tab, Progress tab

---

- **Feature Name**: First-Time User Flow
- **Type**: flow
- **Description**: Open mode -> onboarding -> scan -> results reveal -> dashboard -> first urgent repair -> shrink animation -> next recommended repair.
- **User Role**: student
- **UI Details**: None beyond previously described screens
- **Sub-features**: None

---

- **Feature Name**: Returning User Flow
- **Type**: flow
- **Description**: Open mode -> dashboard shows new slipping areas -> choose urgent repair -> solidification session -> post-session result -> progress log updates.
- **User Role**: student
- **UI Details**: None beyond previously described screens
- **Sub-features**: None

---

- **Feature Name**: Post-Mock-Exam Flow
- **Type**: flow
- **Description**: Mock completed -> weak areas detected -> enter gap mode -> hidden blockers revealed -> root-cause repair begins.
- **User Role**: student
- **UI Details**: None beyond previously described screens
- **Sub-features**: None

---

- **Feature Name**: Realtime Transport to Frontend (WebSocket/Event Bus)
- **Type**: engine-output-needing-UI
- **Description**: WebSocket or local event bus pushes state changes to frontend: knowledge_gap.updated, skill_state.changed, topic_state.changed, repair_plan.generated, hidden_blocker.detected, skill_declining.detected, solidification_session.completed. Frontend updates gap ring, urgent list, map colors, activity feed.
- **User Role**: student
- **UI Details**: Live-updating dashboard elements
- **Sub-features**: None

---

- **Feature Name**: Knowledge Gap Backend Entity Model
- **Type**: engine-output-needing-UI
- **Description**: Full backend model with 22+ entities: Subject, Topic, Subtopic, Skill, Skill Dependency, Question/Assessment Item, Question Skill Link, Misconception Pattern, Student, Student Skill State (central table), Student Topic Aggregate, Student Subject Aggregate, Learning Event, Student Question Attempt, Attempt Skill Outcome, Misconception Detection Event, Gap Repair Plan, Gap Repair Plan Item, Solidification Session, Solidification Session Step, Knowledge Update Feed Item, Gap Snapshot, Skill Score Explanation.
- **User Role**: admin
- **UI Details**: Powers all frontend views
- **Sub-features**: 22+ database entities

---

- **Feature Name**: Dashboard APIs
- **Type**: engine-output-needing-UI
- **Description**: Full API set: GET overview, GET map, GET updates, GET skill state, GET skill mistakes, POST scan start, POST repair plan generate, GET repair plan current, POST solidification session start, POST step complete, POST session complete, POST learning events, POST question attempts, POST confidence events, POST retention checks.
- **User Role**: admin
- **UI Details**: Powers all frontend interactions
- **Sub-features**: 16+ API endpoints

---

- **Feature Name**: Materialized Frontend View Models (DTOs)
- **Type**: engine-output-needing-UI
- **Description**: Backend view models for frontend: Gap Overview DTO, Gap Map DTO, Skill Detail DTO, Repair Session DTO. Each contains pre-computed fields to avoid frontend logic.
- **User Role**: admin
- **UI Details**: Powers efficient rendering
- **Sub-features**: 4 DTO types

---

- **Feature Name**: Hidden Blocker Detection Logic
- **Type**: engine-output-needing-UI
- **Description**: Detects when a student appears weak in one area but the real cause is elsewhere. Observes downstream failures, inspects prerequisite graph, compares performance, computes blocker confidence score from dependency strength/upstream weakness severity/downstream failure frequency/recency, marks blocker.
- **User Role**: student
- **UI Details**: Displayed in Hidden Blockers panel on dashboard
- **Sub-features**: None

---

- **Feature Name**: Forgetting/Decay Model
- **Type**: engine-output-needing-UI
- **Description**: Each skill has memory_strength, last_correct_at, predicted_decay_at, next_retention_check_at. Memory strength falls over time unless reinforced. Successful recall pushes decay prediction further out; failed recall reduces strength sharply.
- **User Role**: student
- **UI Details**: None specified (internal engine)
- **Sub-features**: None

---

- **Feature Name**: Skill Score Explanation Record
- **Type**: engine-output-needing-UI
- **Description**: Stores calculation explanation as JSON for each skill score, including recent accuracy drop, slow response trend, recurring misconception, downstream impact, exam weight. Used for teacher views, debugging, trust, and future AI explanations.
- **User Role**: teacher | admin
- **UI Details**: Accessible from teacher view
- **Sub-features**: None

---

## FEATURES FOUND IN idea7.txt (and idea8.txt -- DUPLICATE CONFIRMED)

**NOTE: idea7.txt and idea8.txt are byte-for-byte identical files. All features below apply to both.**

---

- **Feature Name**: Memory Mode
- **Type**: mode
- **Description**: A memory recovery and reinforcement system that detects what is fading in a student's memory, traces the exact subtopic or micro-skill that is weakening, and intervenes fast before that weakness spreads into wider recall failure. Treats memory as a living structure, not a static score. Distinct from Knowledge Gap Mode: Knowledge Gap = absence of understanding; Memory Gap = weakening of retained knowledge.
- **User Role**: student
- **UI Details**: Main mode card on home screen. Card copy: "Strengthen what you already know" / "Catch fading memory before it affects your score". Shows today's fading areas, strongest memory area, quick scan CTA, rescue CTA.
- **Sub-features**: Memory Strength Engine, Decay Detection Engine, Micro-Recall Mapper, Recall Recovery Engine, Memory Connection Engine

---

- **Feature Name**: Memory Dimensions Model (6 Dimensions)
- **Type**: engine-output-needing-UI
- **Description**: Memory is defined as 6 measurable dimensions: Retention (can student still answer after time), Recall speed (how quickly), Recall stability (consistency across attempts), Connection strength (use in related questions), Recovery ability (how quickly restored when weakened), Transfer recall (answer when question changes form).
- **User Role**: student
- **UI Details**: None specified (internal model)
- **Sub-features**: Retention, Recall speed, Recall stability, Connection strength, Recovery ability, Transfer recall

---

- **Feature Name**: Memory Strength Engine
- **Type**: engine-output-needing-UI
- **Description**: Scores how strong each subtopic is in the learner's memory using states: Strong, Stable, Shaking, Weak, Critical (or 0-100 score). Calculated from: Accuracy (35%), Recall Speed (20%), Retention Over Time (20%), Variation Handling (15%), Confidence Match (10%).
- **User Role**: student
- **UI Details**: Score displayed per subtopic
- **Sub-features**: None

---

- **Feature Name**: Memory Decay Detector
- **Type**: engine-output-needing-UI
- **Description**: Watches for falling performance over time. Signals: previously mastered concept now showing repeated slips, correct answers becoming slower, similar question families causing confusion, growing dependence on hints, success dropping after time gap, confusion between related concepts.
- **User Role**: student
- **UI Details**: Generates "fading now" alerts
- **Sub-features**: None

---

- **Feature Name**: Micro-Recall Mapper
- **Type**: engine-output-needing-UI
- **Description**: Zooms into the exact broken memory point. Instead of repairing "solving linear equations" as a whole, identifies specific broken units like negative sign handling, inverse operation order, collecting like terms, transposition confusion.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Recall Recovery Engine
- **Type**: engine-output-needing-UI
- **Description**: Serves recovery content after weakness detection: similar questions, near-similar questions, reverse-form questions, clue-based recall, rapid-fire familiarity drills, cue-triggered reminder prompts, step reconstruction questions, sensory reinforcement prompts. Rebuilds: recognition, familiarity, speed, confidence, independent recall.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Memory Connection Engine
- **Type**: engine-output-needing-UI
- **Description**: After recovery, reconnects the subtopic to earlier prerequisites, nearby subtopics, applied question forms, mixed-topic questions, exam-style combinations. Ensures student does not relearn in isolation.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Memory Scan (Sub-Mode)
- **Type**: sub-mode
- **Description**: Quick diagnostic mode (8-15 questions) to identify strong vs weak memory areas, detect fading zones, detect unstable recall. Output: memory heatmap, memory strength by subtopic, "fading now" alerts, recommended rescue areas. Best for daily check-in, weekly scan, start of study session.
- **User Role**: student
- **UI Details**: Simple, clean, low-friction. Shows: question card, confidence toggle after each answer, subtle timer, progress ring, optional "I'm unsure" button.
- **Sub-features**: Memory heatmap output, Fading alerts

---

- **Feature Name**: Memory Rescue (Sub-Mode)
- **Type**: sub-mode
- **Description**: Intervention mode. Focuses on subtopics showing memory decline, rebuilds familiarity fast, stops decay from spreading. Should feel targeted, short, urgent, effective.
- **User Role**: student
- **UI Details**: Rescue screen shows: subtopic being repaired, current rescue stage, progress bar called "Memory rebuild", encouraging copy like "You're rebuilding this pathway"
- **Sub-features**: None

---

- **Feature Name**: Recall Builder (Sub-Mode)
- **Type**: sub-mode
- **Description**: Strengthens already-known areas. Deepens memory roots, increases speed, increases stability, moves concepts from "fragile" to "strong." 8-12 questions.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Chain Repair (Sub-Mode)
- **Type**: sub-mode
- **Description**: Used when forgetting one concept has damaged connected areas. Repairs prerequisite gaps, reconnects broken memory paths, rebuilds knowledge chain. 10-18 questions. App says: "You are missing this because something earlier is weakening."
- **User Role**: student
- **UI Details**: Chain repair screen displays root weak skill and related skills being affected. Example: "Equivalent Fractions is weakening Addition of Fractions."
- **Sub-features**: None

---

- **Feature Name**: Rapid Recall Drill (Sub-Mode)
- **Type**: sub-mode
- **Description**: Fast-response memory practice (30-90 seconds or 10 quick prompts). Sharpens response time, improves fluency, builds automaticity. Good for formulas, definitions, grammar rules, mental math, science facts, vocabulary, symbolic interpretation.
- **User Role**: student
- **UI Details**: Game-like fast-paced interface
- **Sub-features**: None

---

- **Feature Name**: Memory Mode Experience Flow (8 Steps)
- **Type**: flow
- **Description**: Step 1: Scan memory (short recall test), Step 2: Detect weakness (memory drop in subtopic), Step 3: Isolate the real problem (map to exact sub-skill), Step 4: Launch rescue (similar and progressively reinforcing questions), Step 5: Trigger familiarity (repetition, variation, cues, pattern reminders), Step 6: Reconnect memory paths (link back to surrounding concepts), Step 7: Re-test under light pressure (prove memory is back), Step 8: Track recovery (update memory score, monitor relapse risk).
- **User Role**: student
- **UI Details**: None beyond individual screen specs
- **Sub-features**: None

---

- **Feature Name**: Recall Repair Question Ladder (6 Levels)
- **Type**: interaction
- **Description**: Structured ladder of recall repair: Level 1: Recognition ("Have you seen this before?"), Level 2: Guided Recall (with a clue), Level 3: Unguided Recall (without support), Level 4: Variation Recall (changed form), Level 5: Connected Recall (inside related problem), Level 6: Pressure Recall (quickly, under time).
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: 6 levels

---

- **Feature Name**: Sensory Triggers and Familiarity Builders
- **Type**: interaction
- **Description**: Subtle reinforcement tools: familiar visual patterns, repeated symbol layouts, short audio cues for correct recovery, highlight cues on recurring structures, "you've seen this before" prompts, quick pattern reminders, color-coded concept families, mini flash memory cards before questions, visual anchors for formulas or steps.
- **User Role**: student
- **UI Details**: Visual patterns, audio cues, color coding, flash cards, visual anchors
- **Sub-features**: None

---

- **Feature Name**: Memory Mode Emotional Layer / UX Tone
- **Type**: interaction
- **Description**: Protective morale messaging. Uses: "This area is fading slightly", "Let's rebuild this memory", "You almost had this", "Your recall is coming back", "Memory restored", "This concept is stable again", "This pathway is stronger now." Avoids: "You are weak", "You failed this badly." Emotional effect: safety, momentum, recovery, control.
- **User Role**: student
- **UI Details**: Specific copy examples
- **Sub-features**: None

---

- **Feature Name**: Memory Stability Index (MSI)
- **Type**: engine-output-needing-UI
- **Description**: Scoring model: MSI = (0.30 x Accuracy) + (0.15 x Speed) + (0.20 x Retention) + (0.15 x Variant) + (0.10 x Independence) + (0.10 x Connection). States: 85-100 = Locked In, 70-84 = Stable, 55-69 = Vulnerable, 40-54 = Fading, 0-39 = Critical Recovery Needed. Powers alerts, rescue sessions, dashboards, prioritization.
- **User Role**: student
- **UI Details**: Displayed on dashboard and per-skill views
- **Sub-features**: Accuracy score (weighted recent attempts), Speed score (clamp formula), Retention score, Variant transfer score, Independence score (penalizes hint reliance), Connection score

---

- **Feature Name**: Memory Gamification Elements
- **Type**: interaction
- **Description**: Memory Shield (protects streak when recovering fading areas), Recall Streak (consecutive restored memories), Brain Spark (fast recall burst reward), Memory Chain (keep connected topics strong), Recovery Badge (when critical area becomes stable again), Locked-In Status (topic remains strong for many days), Glow Map (memory heatmap that brightens as recall strengthens).
- **User Role**: student
- **UI Details**: Badges, streaks, glow effects on map
- **Sub-features**: Memory Shield, Recall Streak, Brain Spark, Memory Chain, Recovery Badge, Locked-In Status, Glow Map

---

- **Feature Name**: Memory Mode Dashboard
- **Type**: screen
- **Description**: Core page showing: strongest memory areas, fading memory areas, critical recall zones, recently recovered concepts, unstable chains, average recall speed, retention over time, memory recovery streak. Visualized as: glowing topic web, strength bars, recovery zones, fading alerts, recall map.
- **User Role**: student
- **UI Details**: Memory Map, At Risk Today, Recently Recovered, Strongest Zones, Recall Speed Trend, Memory Streak. Topic web/skill tree with glow intensity: bright = strong, dim = vulnerable, blinking/amber = fading, red = critical, green pulse = recovered.
- **Sub-features**: Memory Map, At Risk Today, Recently Recovered, Strongest Zones, Recall Speed Trend, Memory Streak

---

- **Feature Name**: Memory Scan Screen
- **Type**: screen
- **Description**: Simple, clean, low-friction diagnostic interface. Shows question card, confidence toggle after each answer, subtle timer, progress ring, optional "I'm unsure" button.
- **User Role**: student
- **UI Details**: Described above
- **Sub-features**: Confidence toggle, Timer, Progress ring, "I'm unsure" button

---

- **Feature Name**: Memory Rescue Screen
- **Type**: screen
- **Description**: Focused and reassuring repair interface. Shows the subtopic being repaired, current rescue stage, a progress bar called "Memory rebuild", encouraging copy.
- **User Role**: student
- **UI Details**: Progress bar labeled "Memory rebuild", copy: "You're rebuilding this pathway"
- **Sub-features**: None

---

- **Feature Name**: Chain Repair Screen
- **Type**: screen
- **Description**: Displays the root weak skill and the related skills being affected, explaining why the intervention matters.
- **User Role**: student
- **UI Details**: Example: "Equivalent Fractions is weakening Addition of Fractions."
- **Sub-features**: None

---

- **Feature Name**: Memory Session End Screen
- **Type**: screen
- **Description**: Shows what was repaired, what is still vulnerable, speed improvement, memory status change, next recommended check time.
- **User Role**: student
- **UI Details**: Example: "3 skills moved from Fading to Rebuilding", "1 skill fully recovered", "Next short recheck: tomorrow"
- **Sub-features**: None

---

- **Feature Name**: Memory Mode Visual and Sensory Design
- **Type**: animation
- **Description**: Soft pulse animations, glow restoration effects, memory strand visuals reconnecting, subtle sound cues for restored recall, calm success chimes, visual anchors for repeated patterns. Should feel like the brain being repaired, not attacked.
- **User Role**: student
- **UI Details**: Specific visual and audio effects described
- **Sub-features**: Soft pulse animations, Glow restoration effects, Memory strand reconnection visuals, Sound cues for recall, Calm success chimes

---

- **Feature Name**: Memory Mode Sound Design
- **Type**: sound
- **Description**: Short audio cues for correct recovery, calm success chimes, subtle sound cues for restored recall. Sound feedback can be toggled on or off in first-time intro.
- **User Role**: student
- **UI Details**: Toggle on/off setting
- **Sub-features**: None

---

- **Feature Name**: First-Time Intro Screen (Memory Mode)
- **Type**: screen
- **Description**: Simple explainer: "Memory Mode tracks what is fading, repairs weak recall, and helps you keep knowledge alive." Asks: preferred session length, gentle vs challenging recovery preference, sound feedback on/off.
- **User Role**: student
- **UI Details**: 3 preference settings
- **Sub-features**: Session length preference, Difficulty preference, Sound toggle

---

- **Feature Name**: Memory Mode Session Types (6 Types)
- **Type**: sub-mode
- **Description**: Six distinct session types: Memory Scan (8-15 questions, daily check-in), Rescue Burst (6-10 questions, one fading subtopic), Deep Repair (12-20 questions, structural weakness), Recall Builder (8-12 questions, strengthening), Chain Repair (10-18 questions, dependency damage), Rapid Recall Drill (30-90 seconds or 10 quick prompts, fluency).
- **User Role**: student
- **UI Details**: Each has specific question count and purpose
- **Sub-features**: Memory Scan, Rescue Burst, Deep Repair, Recall Builder, Chain Repair, Rapid Recall Drill

---

- **Feature Name**: Dynamic Support Rules
- **Type**: interaction
- **Description**: If learner fails two questions in a row: reduce difficulty, add a cue, show a smaller pattern, or step down to a prerequisite. If learner succeeds three times in a row: remove cues, increase variation, introduce mixed context, then validate under time.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Decay Risk Score
- **Type**: engine-output-needing-UI
- **Description**: Second metric beyond MSI: Decay Risk = f(time_since_last_mastery, performance_drop, speed_drop, relapse_rate, dependency_damage). Tells system likelihood of near-future forgetting, not just current strength. Enables early intervention before actual decay.
- **User Role**: student
- **UI Details**: None specified (drives alerts)
- **Sub-features**: None

---

- **Feature Name**: Memory Graph (Concept Network)
- **Type**: engine-output-needing-UI
- **Description**: Every skill belongs to a graph with prerequisites, siblings, dependents, and application edges. Supports local diagnosis (exact weak skill) and chain diagnosis (broader pathway affected). Example: forgetting "LCM for denominators" weakens adding fractions, subtracting fractions, algebraic fractions, ratio normalization.
- **User Role**: student
- **UI Details**: Visualized as topic web/skill tree on dashboard
- **Sub-features**: Local diagnosis, Chain diagnosis

---

- **Feature Name**: Six Core Memory Mode Engines
- **Type**: engine-output-needing-UI
- **Description**: Memory Strength Engine (maintains MSI per micro-skill), Decay Detection Engine (detects silent forgetting), Micro-Recall Mapper (finds smallest broken unit), Recovery Planner (designs rescue path), Connection Rebuilder (reconnects recovered memory), Recheck Scheduler (schedules spaced follow-up checks).
- **User Role**: student
- **UI Details**: None (backend engines)
- **Sub-features**: Memory Strength Engine, Decay Detection Engine, Micro-Recall Mapper, Recovery Planner, Connection Rebuilder, Recheck Scheduler

---

- **Feature Name**: Memory vs Understanding Classifier
- **Type**: engine-output-needing-UI
- **Description**: Lightweight classifier distinguishing memory weakness (previously mastered now slipping, works with cue but fails without, strong on familiar form weak on changed, slower retrieval) from understanding weakness (repeated failure across all forms, poor logic even with cues, conceptual misunderstanding). If understanding weakness detected, hands off to "Teach Me This Topic" / "Fix My Weakness" / "Guided Lesson Mode."
- **User Role**: student
- **UI Details**: None specified (internal routing)
- **Sub-features**: None

---

- **Feature Name**: Memory Mode Analytics and Success Metrics
- **Type**: report
- **Description**: Tracks: reduction in relapse rate, increase in MSI over time, average recall speed improvement, recovery success rate, performance lift in linked topics, percentage of fading skills caught before exam failure, retention after 7 and 30 days, session completion rate, rescue-to-recovery conversion rate.
- **User Role**: admin | teacher
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Question Tagging System (Memory Mode)
- **Type**: engine-output-needing-UI
- **Description**: Each question linked to: subject, topic, subtopic, micro-skill, prerequisite skills, neighboring/related skills, question family, question variant type, cognitive action required, misconception type, expected response time, difficulty level, memory cue type. Critical for micro-level intervention.
- **User Role**: admin
- **UI Details**: None (content system)
- **Sub-features**: None

---

- **Feature Name**: Student Memory Profile
- **Type**: engine-output-needing-UI
- **Description**: Live memory profile per micro-skill storing: total attempts, total correct, recent correct rate, average response time, response time trend, last seen timestamp, last correct timestamp, hint usage rate, confidence rating history, variant performance history, relapse count, prerequisite dependency health, memory stability score, current status label.
- **User Role**: student
- **UI Details**: Powers all per-skill views
- **Sub-features**: None

---

- **Feature Name**: Memory Status Ladder (7 States)
- **Type**: engine-output-needing-UI
- **Description**: Each micro-skill has a visible status: Locked In (strong/fast/stable), Stable (good, monitored), Vulnerable (beginning to weaken), Fading (clear signs of decay), Critical (active recovery needed), Rebuilding (rescue begun), Recovered (recently restored, monitor relapse).
- **User Role**: student
- **UI Details**: Status badges on dashboard
- **Sub-features**: None

---

- **Feature Name**: Recheck Scheduling Logic (Spaced Repetition)
- **Type**: engine-output-needing-UI
- **Description**: Adaptive follow-up checks: 1 day, 3 days, 7 days, 14 days, 30 days baseline. Personalized based on relapse history. If student repeatedly relapses: shorten intervals, increase variation, strengthen prerequisite review.
- **User Role**: student
- **UI Details**: "Next short recheck: tomorrow" displayed on session end screen
- **Sub-features**: None

---

- **Feature Name**: Question Selection Algorithm (Memory Mode)
- **Type**: engine-output-needing-UI
- **Description**: Ordered selection: same micro-skill familiar form, slightly changed form, different distractor style, applied context, mixed question with related skill, timed validation item. Avoids: repeating exact stem too often, over-serving strong skills, serving effectively identical variants. Prefers: controlled variation, misconception-aware distractors, spaced repetition with freshness.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Memory Mode Backend Architecture (8 Services)
- **Type**: engine-output-needing-UI
- **Description**: 8 backend services: Skill Graph Service (concept graph), Learner Memory Profile Service (per student per skill metrics), Event Logger (answers, timing, hints, confidence, retries, session outcomes), Memory Scoring Engine (MSI and status changes), Decay Detector (runs after sessions and periodically), Recovery Planner (rescue plans and question ladders), Question Selector (serves right items), Scheduler (recheck intervals based on personalized behavior).
- **User Role**: admin
- **UI Details**: None (backend)
- **Sub-features**: 8 services

---

- **Feature Name**: Related Modes Ecosystem
- **Type**: engine-output-needing-UI
- **Description**: Memory Mode fits with other modes: Knowledge Gap Mode (finds what not yet learned), Mental Mode (pushes speed, sharpness, performance under pressure), Elite Mode (challenges strong students with harder forms and higher standards). Memory Mode is the stability and recall preservation layer.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: Mental Mode reference, Elite Mode reference, Knowledge Gap Mode reference

---

- **Feature Name**: Memory Decay Science Model (Storage Strength vs Retrieval Strength)
- **Type**: engine-output-needing-UI
- **Description**: Implements Bjork's framework: storage strength vs retrieval strength. Tracks two separate things: Memory Access (current retrievability) and Memory Entrenchment (durable storage). A student may look good today and still forget tomorrow.
- **User Role**: student
- **UI Details**: None specified (internal model)
- **Sub-features**: Memory Access tracking, Memory Entrenchment tracking

---

- **Feature Name**: Memory Confirmation Sequence (5 Gates)
- **Type**: interaction
- **Description**: A skill is marked "in memory" only after passing 5 gates: Gate 1: Independent recall (without answer choices), Gate 2: Delayed recall (after time gap), Gate 3: Variant recall (changed shape/wording), Gate 4: Embedded use (inside larger/mixed problem), Gate 5: Stability check (recalled correctly across separate sessions). Six kinds of evidence: retrieval, time, transfer, application, fluency, stability.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: 5 gates

---

- **Feature Name**: Memory Evidence Score
- **Type**: engine-output-needing-UI
- **Description**: Per-skill scoring: Independent Recall = 25, Delayed Recall = 20, Variant Recall = 20, Embedded Use = 15, Speed Stability = 10, Recheck Stability = 10. Total = 100. States: 0-39 = Not in memory yet, 40-59 = Emerging, 60-74 = Fragile memory, 75-89 = Strong memory, 90-100 = Locked in. Must have time-separated evidence.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Memory Confirmation Question Types (5 Types)
- **Type**: interaction
- **Description**: A. Pure Recall Question (no options, no cues), B. Near-Transfer Variant (same skill, different presentation), C. Interference Question (similar concepts side by side, e.g., area vs perimeter), D. Embedded Chain Question (skill hidden inside multi-step problem), E. Delayed Recheck (brought back later).
- **User Role**: student
- **UI Details**: Different question UI formats
- **Sub-features**: 5 question types

---

- **Feature Name**: Memory State Model (8 States, Detailed)
- **Type**: engine-output-needing-UI
- **Description**: Detailed states: Fresh but Unproven, Accessible, Fragile, Anchoring, Confirmed in Memory, At Risk of Decay, Faded but Recoverable, Collapsed. More granular than basic status ladder.
- **User Role**: student
- **UI Details**: User-facing simplified as: Fresh, Building, Needs Reinforcement, Strong, Locked In, Slipping
- **Sub-features**: None

---

- **Feature Name**: Per-Skill Internal Memory Fields
- **Type**: engine-output-needing-UI
- **Description**: For every micro-skill, stores: retrieval_strength_now, storage_confidence, last_success_at, days_since_last_recall, variant_success_rate, interference_success_rate, embedded_use_success_rate, response_time_median, recovery_speed_on_relearning, memory_state.
- **User Role**: admin
- **UI Details**: None (backend data)
- **Sub-features**: None

---

- **Feature Name**: Memory Learning Sequence (6 Steps)
- **Type**: flow
- **Description**: How a student learns into memory: 1. Initial encoding, 2. First successful retrieval, 3. Spaced re-retrieval (after some forgetting begins), 4. Variation exposure (changed conditions), 5. Connected use (with related ideas), 6. Periodic reactivation (before fading too far).
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Evidence Tiers System (4 Tiers)
- **Type**: engine-output-needing-UI
- **Description**: Every response classified by diagnostic strength: Tier 1 (Weak) - repeated identical item, familiar MCQ, strong cueing, immediate post-lesson success. Tier 2 (Moderate) - short-answer recall, slightly changed form. Tier 3 (Strong) - free recall, delayed recall, transfer, discrimination, mixed problem. Tier 4 (Durable) - repeated success across sessions, survival after spacing, stable despite interference.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: 4 tiers

---

- **Feature Name**: Universal Memory Confirmation Engine Pipeline
- **Type**: engine-output-needing-UI
- **Description**: 6-step pipeline per micro-skill: Step A - Initial encoding, Step B - Immediate retrieval check, Step C - Variation check, Step D - Delay check, Step E - Embedded-use check, Step F - Stability recheck. Only after full sequence is concept treated as truly in memory.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Canonical Memory States (12 States)
- **Type**: engine-output-needing-UI
- **Description**: Full state machine: SEEN, ENCODED, ACCESSIBLE, FRAGILE, ANCHORING, CONFIRMED, LOCKED_IN, AT_RISK, FADING, REBUILDING, RECOVERED, COLLAPSED. Each has specific meaning and transition rules.
- **User Role**: student
- **UI Details**: Simplified for student-facing display
- **Sub-features**: 12 states with full transition rules

---

- **Feature Name**: Math Memory Layers (4 Layers)
- **Type**: engine-output-needing-UI
- **Description**: Math memory has 4 layers: A. Fact memory (multiplication facts, formulas, definitions), B. Procedure memory (long division, solving equations), C. Decision memory (which method applies, when and why), D. Structure memory (recognizing hidden mathematical structure when surface changes).
- **User Role**: student
- **UI Details**: Per-skill explanation: "You can solve this directly, but you still need to prove it in mixed problems."
- **Sub-features**: 4 memory layers

---

- **Feature Name**: Math Memory Question Taxonomy (7 Types)
- **Type**: interaction
- **Description**: M1: Pure fact recall, M2: Bare procedure recall, M3: Method-selection question (which method, first step, why), M4: Variant-form question (symbolic, word problem, table, diagram, reverse), M5: Interference question (area vs perimeter, mean vs median), M6: Embedded multi-step question, M7: Timed fluency check.
- **User Role**: student
- **UI Details**: Different question formats
- **Sub-features**: 7 question types

---

- **Feature Name**: Science Memory Layers (5 Layers)
- **Type**: engine-output-needing-UI
- **Description**: Science memory has 5 layers: A. Term and fact memory, B. Concept memory (what idea means), C. Causal memory (why something happens), D. Process memory (sequence/mechanism), E. Representation memory (moving between words, diagrams, equations, tables, graphs, experiments).
- **User Role**: student
- **UI Details**: Per-skill explanation: "You remember the definition, but you still need to show it in diagrams and scenarios."
- **Sub-features**: 5 memory layers

---

- **Feature Name**: Science Memory Question Taxonomy (7 Types)
- **Type**: interaction
- **Description**: S1: Term recall, S2: Explanation recall ("Explain why..."), S3: Similarity discrimination (diffusion vs osmosis), S4: Diagram or graph interpretation, S5: Scenario transfer ("What happens if..."), S6: Process ordering/mechanism reconstruction, S7: Experimental reasoning (identify variable, state control, interpret outcome).
- **User Role**: student
- **UI Details**: Different question formats including diagrams/graphs
- **Sub-features**: 7 question types

---

- **Feature Name**: Theory-Subject Memory Layers (5 Layers)
- **Type**: engine-output-needing-UI
- **Description**: For History, Government, Religious studies, Literature, Geography, etc.: A. Fact memory (dates, names, events), B. Relational memory (connections, cause/effect), C. Sequence memory (timelines, event flow), D. Argument memory (reasons, significance, themes), E. Retrieval-under-cue-loss (recall without exact textbook wording).
- **User Role**: student
- **UI Details**: Per-skill explanation: "You remember the facts, but you still need to explain them in your own words after time has passed."
- **Sub-features**: 5 memory layers

---

- **Feature Name**: Theory-Subject Memory Question Taxonomy (7 Types)
- **Type**: interaction
- **Description**: T1: Direct fact recall, T2: Own-words explanation, T3: Cause-effect link question, T4: Sequence/chronology question, T5: Contrast/comparison question, T6: Evidence-use question ("Use one example to support..."), T7: Reworded delayed recall.
- **User Role**: student
- **UI Details**: Different question formats
- **Sub-features**: 7 question types

---

- **Feature Name**: Subject Proof Template Engine
- **Type**: engine-output-needing-UI
- **Description**: Engine knowing the required evidence stack per subject. Math: procedure, method selection, variation, delay, embedded use. Science: recall, explanation, representation shift, transfer, delay. Theory: fact, own-words explanation, relation/sequence, contrast, delay. A skill cannot become "Confirmed in Memory" until the required template is satisfied.
- **User Role**: student
- **UI Details**: None (backend logic)
- **Sub-features**: Math template, Science template, Theory template

---

- **Feature Name**: Memory Evidence Weighting Engine
- **Type**: engine-output-needing-UI
- **Description**: Different correct answers count differently. Weights: identical-item success (0.5-0.8), familiar MCQ (1.0), low-cue short answer (1.5-2.0), free recall (2.5-3.0), variant transfer (2.5-3.0), embedded use (2.5-3.0), delayed recall (3.0-3.5), interference success (2.5-3.0), repeated time-separated (3.5-4.0). Lower cue support earns more credit.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Recovery Planner with Breakdown Classification
- **Type**: engine-output-needing-UI
- **Description**: Classifies breakdowns into types: PURE_DECAY (independent recall + delay repeat), INTERFERENCE_CONFUSION (contrast drills, side-by-side discrimination), PREREQUISITE_BREAK (repair root skill first), SURFACE_DEPENDENCE (changed wording/representation/context), SPEED_ONLY_WEAKNESS (light fluency drills), UNDERSTANDING_NOT_MEMORY (hand off to concept-teaching mode).
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: 6 breakdown types

---

- **Feature Name**: Confirmation Gate Engine
- **Type**: engine-output-needing-UI
- **Description**: Final decision layer. A skill can only move to Confirmed in Memory if: evidence threshold is met, subject proof template is satisfied, at least one delayed success, at least one non-identical success, no unresolved severe interference remains.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Memory Mode Backend Data Model (Full Schema)
- **Type**: engine-output-needing-UI
- **Description**: Full relational schema including tables: skill_nodes, learner_skill_memory, memory_evidence_events, proof_dimension_status, memory_sessions, proof_templates, recheck_schedules, skill_graph_edges. Includes SQL create statements for all tables.
- **User Role**: admin
- **UI Details**: None (backend)
- **Sub-features**: 8+ tables with full field definitions

---

- **Feature Name**: LearnerSkillMemory Record
- **Type**: engine-output-needing-UI
- **Description**: Central per-learner per-skill memory record with fields across categories: State fields, Current-access fields (retrieval_access_score, response time, accuracy, confidence), Durability fields (durability_confidence_score, time_separated_success_count, delay_bands, relearning_speed_index), Proof coverage fields (9 proof dimensions), Risk fields (decay_risk_score, interference_risk_score, relapse_count), Schedule fields, Counts.
- **User Role**: admin
- **UI Details**: None (backend)
- **Sub-features**: None

---

- **Feature Name**: Memory Evidence Event Record
- **Type**: engine-output-needing-UI
- **Description**: Every answer generates an evidence event with: performance data (correct, response time, confidence, hint), diagnostic metadata (question type, variant type, cue level, diagnostic role, delay band, interference condition, representation mode), derived scoring (evidence weight, 9 boolean support flags).
- **User Role**: admin
- **UI Details**: None (backend audit trail)
- **Sub-features**: None

---

- **Feature Name**: Numeric Scoring Models (4 Scores)
- **Type**: engine-output-needing-UI
- **Description**: RetrievalAccessScore (RAS = 35*accuracy + 20*speed + 20*independence + 15*consistency + 10*confidence_alignment), DurabilityConfidenceScore (DCS = 25*time_separated + 20*variant + 15*embedded_use + 15*interference_resistance + 15*recheck_stability + 10*relearning_efficiency), DecayRiskScore (DRS = 25*overdue + 20*accuracy_drop + 15*speed_drift + 15*relapse + 15*dependency_damage + 10*interference_growth), InterferenceRiskScore (confusion with similar concepts).
- **User Role**: admin
- **UI Details**: None (internal scoring)
- **Sub-features**: 4 score formulas

---

- **Feature Name**: Proof Coverage Vector
- **Type**: engine-output-needing-UI
- **Description**: 9 proof dimensions stored per learner-skill: independent_recall, delayed_recall, variant_transfer, embedded_use, interference_resistance, explanation_reasoning, representation_shift, sequence_relation, speed_fluency. Each stores: status (none/partial/satisfied), evidence_count, best_recent_score, last_satisfied_at.
- **User Role**: admin
- **UI Details**: None (backend)
- **Sub-features**: 9 dimensions

---

- **Feature Name**: State Transition Decision Logic (Pseudocode)
- **Type**: engine-output-needing-UI
- **Description**: Full pseudocode for evaluate_skill function that computes RAS, DCS, DRS, checks template satisfaction, delay/variant/interference conditions, and assigns state from COLLAPSED through LOCKED_IN.
- **User Role**: admin
- **UI Details**: None (backend logic)
- **Sub-features**: None

---

- **Feature Name**: Anti-Gaming Rules
- **Type**: engine-output-needing-UI
- **Description**: System resists fake memory signals: identical question repeats give diminishing weight, back-to-back same-family items capped, high cue support cannot satisfy independent recall, immediate post-lesson success cannot confirm durability, only time-separated evidence satisfies delayed proof, only distinct surface forms satisfy transfer proof, only mixed/interference items satisfy those dimensions.
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

- **Feature Name**: Memory Mode Analytics Layer
- **Type**: report
- **Description**: Three levels: Learner level (skills by state, rescue success rate, relapse rate, median fading-to-recovered time, strongest/weakest proof dimensions, spacing survival rate). Content level (which questions produce strongest evidence, which skills relapse most, which interference pairs most problematic, which proof templates too strict/loose). System level (percentage of confirmed skills that later fade, time to first decay, predictive accuracy of DecayRiskScore, recovery lift after rescue).
- **User Role**: admin | teacher
- **UI Details**: None specified
- **Sub-features**: Learner analytics, Content analytics, System analytics

---

- **Feature Name**: Memory Mode API Surface
- **Type**: engine-output-needing-UI
- **Description**: Read endpoints: GET /learners/{id}/memory/overview, GET /memory/skills/{skillId}, GET /memory/at-risk, GET /memory/rechecks. Write endpoints: POST /memory/sessions/start, POST /memory/evidence, POST /sessions/{id}/complete, POST /skills/{skillId}/rescue, POST /rechecks/run. Internal: POST /internal/memory/recompute, POST /schedule-next, POST /run-state-machine.
- **User Role**: admin
- **UI Details**: None (backend)
- **Sub-features**: 11 API endpoints

---

- **Feature Name**: Skill Lifecycle Example (Full State Walkthrough)
- **Type**: flow
- **Description**: Complete lifecycle for "simplifying fractions": ENCODED -> ACCESSIBLE (independent success) -> FRAGILE (reworded/embedded success) -> ANCHORING (delay + ratio problem success) -> CONFIRMED (mixed-topic + delayed recheck) -> LOCKED_IN (3 weeks stable) -> AT_RISK (response time drift) -> FADING (delayed recheck fails) -> REBUILDING (rescue burst) -> RECOVERED (short delay success) -> CONFIRMED (medium delay + proof re-satisfied).
- **User Role**: student
- **UI Details**: None specified
- **Sub-features**: None

---

That completes the exhaustive extraction. In total:

- **idea6.txt**: ~55 distinct features covering Knowledge Gap Mode end-to-end (concept, engine, frontend flow, backend data model, APIs)
- **idea7.txt / idea8.txt** (identical): ~50 distinct features covering Memory Mode end-to-end (concept, sub-modes, science of memory decay, subject-specific memory confirmation, frontend flow, backend data model, state machines, scoring, APIs)

---

Now I have read all of both files completely. Let me compile the exhaustive feature extraction.

---

## FEATURES FOUND IN idea9.txt

---

### 1
- **Feature Name**: Cognitive Question Type Taxonomy
- **Type**: engine-output-needing-UI
- **Description**: A 60-type taxonomy of cognitive question types that classifies questions not by answer format (MCQ, fill-in-the-blank) but by the kind of mental work they force. Organized into 8 major families: Memory, Understanding, Reasoning, Problem-Solving, Accuracy, Pattern and Structure, Expression and Interpretation, and Growth-Control.
- **User Role**: all
- **UI Details**: Questions tagged with both format type and cognitive question type. Engine stores both classifications per question.
- **Sub-features**:
  - Pure Recall Questions
  - Recognition Questions
  - Memory Reconstruction Questions
  - Retrieval Under Pressure Questions
  - Concept Understanding Questions
  - Explanation Questions
  - Reasoning Questions
  - Logical Deduction Questions
  - Inference Questions
  - Application Questions
  - Transfer Questions
  - Multi-Step Problem Solving Questions
  - Strategy Selection Questions
  - First-Step Questions
  - Next-Step Questions
  - Error Detection Questions
  - Correction Questions
  - Misconception Exposure Questions
  - Compare-and-Contrast Questions
  - Classification Questions
  - Matching Principle-to-Case Questions
  - Cause-and-Effect Questions
  - Sequence / Order Questions
  - Prediction Questions
  - Estimation Questions
  - Pattern Recognition Questions
  - Rule Discovery Questions
  - Abstraction Questions
  - Example Generation Questions
  - Non-Example Questions
  - Justification Questions
  - Evidence Selection Questions
  - Claim Evaluation Questions
  - Counterexample Questions
  - Decision-Making Questions
  - Prioritization Questions
  - Synthesis Questions
  - Connection-Making Questions
  - Interpretation Questions
  - Representation Conversion Questions
  - Visualization Questions
  - Mental Manipulation Questions
  - Attention Control Questions
  - Precision Questions
  - Speed Fluency Questions
  - Deep Thinking Questions
  - Open-Ended Reasoning Questions
  - Real-World Scenario Questions
  - Judgment Questions
  - Reflection / Metacognitive Questions
  - Confidence Calibration Questions
  - Mastery Check Questions
  - Retention Check Questions
  - Recovery Questions
  - Adaptive Difficulty Questions
  - Challenge / Stretch Questions
  - Rescue Questions
  - Diagnostic Questions
  - Threshold Questions
  - Capstone Questions

---

### 2
- **Feature Name**: Question Family Grouping System
- **Type**: engine-output-needing-UI
- **Description**: Groups the 60 cognitive question types into 8 major families for app-level organization: A. Memory, B. Understanding, C. Reasoning, D. Problem-Solving, E. Accuracy, F. Pattern and Structure, G. Expression and Interpretation, H. Growth-Control.
- **User Role**: all
- **UI Details**: Used internally in the app to organize questions; visible via tagging and filtering.
- **Sub-features**: 8 family categories as listed above

---

### 3
- **Feature Name**: Question Factory / Question Engine (Hidden)
- **Type**: engine-output-needing-UI
- **Description**: A hidden backend system that generates questions using structured transformation. It does not "write questions" but applies cognitive demand layers to knowledge objects. Operates as a 5-layer system: Knowledge Layer, Cognitive Demand Layer, Transformation Layer, Surface Layer, Validation Layer.
- **User Role**: admin (content), student (consumer)
- **UI Details**: Entirely hidden from student; powers all question delivery across all modes.
- **Sub-features**:
  - Knowledge Layer (stores what is to be learned)
  - Cognitive Demand Layer (stores what kind of thinking to force)
  - Transformation Layer (turns knowledge into question types)
  - Surface Layer (wraps question in wording, format, media, timing, context)
  - Validation Layer (checks generated question validity, fairness, clarity, diagnostic usefulness)

---

### 4
- **Feature Name**: Concept Record Data Structure
- **Type**: engine-output-needing-UI
- **Description**: Each concept in the engine stores: concept_id, topic, subtopic, skill, difficulty band, concept statement, definition, core facts, rules/laws/formulae, steps/procedure, examples, non-examples, common misconceptions, causes, effects, prerequisites, related concepts, representation forms (verbal, symbolic, numeric, diagrammatic, graphical), edge cases/tricky boundaries, application contexts, proof signals, error signatures.
- **User Role**: admin
- **UI Details**: Admin content authoring tool for populating concept records.
- **Sub-features**: All fields listed above

---

### 5
- **Feature Name**: Evaluative Intent System
- **Type**: engine-output-needing-UI
- **Description**: Before generating any question, the engine decides what it is trying to prove about the learner (e.g., can they recall, recognize, rebuild, apply, transfer, detect error, justify, hold under pressure, retain after delay). One concept generates many question families based on different evaluative intents.
- **User Role**: all (internal engine decision)
- **UI Details**: Hidden logic; no direct UI but drives question selection.
- **Sub-features**: None separate

---

### 6
- **Feature Name**: Question Generation Master Formula
- **Type**: engine-output-needing-UI
- **Description**: Q = Surface(Transform(K, C, D, S), L) where K=knowledge object, C=cognitive type, D=difficulty settings, S=student state, L=delivery layer. Defines the mathematical model for how every question is generated.
- **User Role**: admin
- **UI Details**: No direct UI; internal formula.
- **Sub-features**: None

---

### 7
- **Feature Name**: Question Generation Pipeline (8 Stages)
- **Type**: engine-output-needing-UI
- **Description**: An 8-stage pipeline: 1. Select target (concept, subskill, mastery goal, learner need), 2. Choose cognitive question type, 3. Pull concept primitives, 4. Apply transformation operator, 5. Adjust difficulty, 6. Render surface (MCQ, short response, oral rapid-fire, drag-and-order, diagram label, step-by-step, timed mental burst), 7. Validate, 8. Log and learn.
- **User Role**: admin (configuration), student (receives output)
- **UI Details**: Stage 6 renders into actual student-facing formats. Stage 8 captures correctness, latency, confidence, error type, hesitation pattern, hint usage, later retention.
- **Sub-features**:
  - Target selection stage
  - Cognitive type selection stage
  - Primitive extraction stage
  - Transformation operator application stage
  - Difficulty adjustment stage
  - Surface rendering stage (7 format types)
  - Validation stage
  - Logging and learning stage

---

### 8
- **Feature Name**: Transformation Operators (21 Universal Operators)
- **Type**: engine-output-needing-UI
- **Description**: Reusable transformation operators that form the building blocks of question generation: Hide, Mask, Paraphrase, Reorder, Perturb, Insert Error, Contrast, Transfer, Compress Time, Expand Explanation, Switch Representation, Reduce Cues, Add Distractors, Fragment, Chain, Gate, Delay, Wrap in Scenario, Introduce Conflict, Mirror Misconception, Reverse Reasoning.
- **User Role**: admin
- **UI Details**: No direct UI; used internally by engine.
- **Sub-features**: All 21 operators listed above

---

### 9
- **Feature Name**: Concept Graph
- **Type**: engine-output-needing-UI
- **Description**: Each concept connects to prerequisites, subskills, linked concepts, common confusions, examples, non-examples, representations, applications, misconceptions, solution procedures, mastery evidence. This graph makes question generation scalable.
- **User Role**: admin
- **UI Details**: No direct student UI; admin tool for managing concept relationships.
- **Sub-features**: All connection types listed above

---

### 10
- **Feature Name**: Difficulty Vector System
- **Type**: setting
- **Description**: Difficulty is modeled as a multi-dimensional vector (not a single value): concept difficulty, step count, abstractness, representation complexity, distractor closeness, cue reduction, context unfamiliarity, time pressure, reasoning depth, language complexity. Formula: Q_difficulty = Bc + Sl + Ab + Ds + Rs + Tp + Cu.
- **User Role**: admin (configuration), student (experiences)
- **UI Details**: No direct UI; affects question delivery difficulty.
- **Sub-features**: 10 difficulty dimensions

---

### 11
- **Feature Name**: Controlled Freshness System
- **Type**: engine-output-needing-UI
- **Description**: Generates fresh questions through structured recombination, not random chaos. Methods: parameter variation, surface paraphrase, context recasting, representation shift, step slicing, distractor mutation, cue adjustment, difficulty band shift, time condition shift, memory distance shift.
- **User Role**: student (receives fresh questions)
- **UI Details**: No direct UI; prevents repetitive questions.
- **Sub-features**: 10 freshness methods

---

### 12
- **Feature Name**: Student/Learner Model for Question Generation
- **Type**: data-visualization
- **Description**: Per-concept student model storing: mastery probability, memory strength, retrieval speed, misconception risk, confidence calibration, recent error pattern, stability over time, pressure tolerance, representation preference weakness, transfer strength. Drives adaptive question selection.
- **User Role**: student (data subject), parent/teacher (viewer)
- **UI Details**: Displayed in diagnostic reports; drives internal engine decisions.
- **Sub-features**: All 10 learner dimensions

---

### 13
- **Feature Name**: Question Type Selection Logic
- **Type**: engine-output-needing-UI
- **Description**: Engine selects question type based on learning goal: memory building (recall, recognition, reconstruction, retrieval under pressure, retention check, recovery), understanding (explanation, compare/contrast, example/non-example, classification, interpretation), weakness diagnosis (misconception exposure, error detection, first-step, next-step, justification, threshold), mastery (transfer, multi-step solving, representation conversion, application in novel context, thought-provoking reasoning, capstone synthesis), elite challenge (stretch problems, hidden-structure problems, anomaly resolution, mixed-topic transfer, speed+reasoning hybrids).
- **User Role**: all
- **UI Details**: Hidden; drives what questions appear in each mode.
- **Sub-features**: 5 goal-based selection categories

---

### 14
- **Feature Name**: Misconception Engine
- **Type**: engine-output-needing-UI
- **Description**: A separate engine storing per concept: misconception_id, false belief, what causes it, what wrong answer it usually produces, how to bait it, how to diagnose it, how to repair it. Enables generation of bait questions, error detection questions, correction questions, compare correct vs incorrect method, recovery sequences.
- **User Role**: admin (content), student (diagnostic)
- **UI Details**: Powers misconception-targeted questions and repair flows.
- **Sub-features**: Misconception library, bait question generation, error detection generation, correction generation, comparison generation, recovery sequence generation

---

### 15
- **Feature Name**: Solution Graph
- **Type**: engine-output-needing-UI
- **Description**: For every non-trivial question family, the engine stores a solution graph (not just a single linear answer) showing: valid first move, acceptable alternative paths, common invalid branches, intermediate checkpoints, explanation anchors, step dependencies. Powers first-step questions, next-step questions, partial credit, hint generation, error diagnosis, reasoning map drawing.
- **User Role**: admin (content creation)
- **UI Details**: Used to generate step-based questions and reasoning maps.
- **Sub-features**: Solution graph with valid nodes, transitions, invalid transitions, terminal states, hint anchors, explanation anchors

---

### 16
- **Feature Name**: Thought-Provoking Question Generation Patterns (7 Patterns)
- **Type**: engine-output-needing-UI
- **Description**: Seven patterns for generating genuine thinking questions: 1. Anomaly pattern (something surprising), 2. Conflict pattern (two claims disagree), 3. Boundary pattern (edge of a rule), 4. Hidden assumption pattern, 5. Counterexample pattern, 6. Reverse reasoning pattern, 7. Multiple-path pattern (two methods, which is better).
- **User Role**: student
- **UI Details**: Drives thought-provoking questions in DNA, Elite, and Journey modes.
- **Sub-features**: All 7 patterns

---

### 17
- **Feature Name**: Memory Strength Confirmation Formula
- **Type**: engine-output-needing-UI
- **Description**: MemoryStrength(c) = w1*R + w2*S + w3*D + w4*T + w5*V, where R=retrieval accuracy, S=retrieval speed, D=delayed recall success, T=transfer to changed form, V=variance stability across attempts. Memory is confirmed only when learner passes multiple retrieval conditions (correct now, correct later, correct in reworded form, correct under moderate time pressure, not obviously guessed).
- **User Role**: student (data subject), parent/teacher (viewer)
- **UI Details**: Displayed in memory mode reports; drives review scheduling.
- **Sub-features**: 5 memory strength dimensions

---

### 18
- **Feature Name**: Multi-Dimensional Response Scoring
- **Type**: engine-output-needing-UI
- **Description**: After answer submission, stores: correctness, response time, hint usage, confidence, misconception triggered, recovery after failure, persistence under pressure, consistency over time, representation-specific weakness. This feeds the factory to generate the right next question (e.g., right but slow leads to fluency questions, wrong in same misconception twice triggers targeted repair).
- **User Role**: student (data subject)
- **UI Details**: No direct student UI; powers adaptive flow and reports.
- **Sub-features**: 9 scoring dimensions plus branching next-action logic

---

### 19
- **Feature Name**: Question Factory Architecture (10 Layers)
- **Type**: engine-output-needing-UI
- **Description**: Full engine architecture: 1. Curriculum and Knowledge Layer, 2. Concept Intelligence Layer, 3. Question Family Layer, 4. Transformation Layer, 5. Difficulty and Variant Layer, 6. Surface Rendering Layer, 7. Validation Layer, 8. Delivery Layer, 9. Response Analysis Layer, 10. Learner State and Adaptation Layer.
- **User Role**: admin
- **UI Details**: Backend architecture; no direct student UI.
- **Sub-features**: All 10 layers

---

### 20
- **Feature Name**: CurriculumNode Entity
- **Type**: engine-output-needing-UI
- **Description**: Database entity storing educational structure: curriculum_node_id, curriculum_id, subject_id, grade_level, strand, topic, subtopic, objective, learning outcome, term/unit, sequence_order, prerequisite_node_ids, curriculum version, active flag.
- **User Role**: admin
- **UI Details**: Admin tool for curriculum management.
- **Sub-features**: All fields listed

---

### 21
- **Feature Name**: Concept Entity
- **Type**: engine-output-needing-UI
- **Description**: Core database entity: concept_id, curriculum_node_id, title, canonical_statement, short_explanation, long_explanation, concept_category, baseline_difficulty, mastery_threshold, prerequisite_concept_ids, related_concept_ids, contrast_concept_ids, concept_tags.
- **User Role**: admin
- **UI Details**: Admin content tool.
- **Sub-features**: All fields listed

---

### 22
- **Feature Name**: ConceptPrimitive Entity
- **Type**: engine-output-needing-UI
- **Description**: Stores internal parts of a concept: primitive_id, concept_id, primitive_type (definition, fact, rule, formula, law, condition, cause, effect, step, clue, explanation_anchor, boundary_case, memory_hook, application_hint), label, content, importance_weight, difficulty_weight, sequence_order, metadata.
- **User Role**: admin
- **UI Details**: Admin content authoring.
- **Sub-features**: 14 primitive types

---

### 23
- **Feature Name**: MisconceptionRecord Entity
- **Type**: engine-output-needing-UI
- **Description**: Database entity: misconception_id, concept_id, title, misconception_statement, cause_type (overgeneralization, memorization_without_understanding, visual_confusion, language_confusion, step_confusion, false_analogy), wrong_reasoning_pattern, wrong_answer_pattern, confidence_trap_level, correction_logic, repair_sequence_ref.
- **User Role**: admin
- **UI Details**: Admin misconception management tool.
- **Sub-features**: 6 cause types

---

### 24
- **Feature Name**: RepresentationRecord Entity
- **Type**: engine-output-needing-UI
- **Description**: Stores different forms of the same concept: representation_id, concept_id, representation_type (text, equation, graph, table, diagram, scenario, worked_example, audio_prompt), title, content, difficulty.
- **User Role**: admin
- **UI Details**: Admin content tool; powers interpretation and representation conversion questions.
- **Sub-features**: 8 representation types

---

### 25
- **Feature Name**: ExampleRecord Entity
- **Type**: engine-output-needing-UI
- **Description**: Stores examples and non-examples: example_id, concept_id, example_type (canonical, tricky, disguised, real_world, borderline, non_example), title, description, explanation, validity_conditions, similarity_score.
- **User Role**: admin
- **UI Details**: Admin content tool.
- **Sub-features**: 6 example types

---

### 26
- **Feature Name**: SolutionGraph Entity
- **Type**: engine-output-needing-UI
- **Description**: Database entity storing: solution_graph_id, concept_id, question_family_id, title, start_state_json, nodes_json, transitions_json, invalid_transitions_json, terminal_states_json, hint_anchors_json, explanation_anchors_json.
- **User Role**: admin
- **UI Details**: Admin content tool for building solution paths.
- **Sub-features**: JSON graph structures

---

### 27
- **Feature Name**: QuestionFamily Entity
- **Type**: engine-output-needing-UI
- **Description**: Database entity: question_family_id, family_name, cognitive_type, evaluative_intent, description, required_primitives_json, allowed_formats_json, difficulty_dimensions_json, transformation_rules_json, scoring_mode, validation_rules_json. Examples: Recall_Fact, Recognition_CloseDistractors, Reconstruction_FromClues, Reasoning_Causal, ErrorDetection_SingleFault, Transfer_DisguisedScenario, Pressure_RapidRetrieval.
- **User Role**: admin
- **UI Details**: Admin configuration tool.
- **Sub-features**: 7 example family types

---

### 28
- **Feature Name**: QuestionTemplate Entity
- **Type**: engine-output-needing-UI
- **Description**: Controlled surface scaffold: template_id, question_family_id, format_type, template_text, slot_definitions_json, slot_constraints_json, language_level, tone.
- **User Role**: admin
- **UI Details**: Admin template editor.
- **Sub-features**: None

---

### 29
- **Feature Name**: GeneratedQuestion Entity
- **Type**: engine-output-needing-UI
- **Description**: Stores the actual question instance: generated_question_id, concept_id, question_family_id, mode_context, cognitive_type, format_type, difficulty_score, difficulty_profile_json, variant_signature, surface_text, media_payload_json, answer_key_json, explanation_key_json, misconception_target_id, representation_source_id, solution_graph_id, timing_profile_json, freshness_hash, validation_status, generation_metadata_json.
- **User Role**: admin (inspection), student (receives)
- **UI Details**: The question the learner sees rendered from this entity.
- **Sub-features**: All metadata fields

---

### 30
- **Feature Name**: StudentConceptState Entity (Learner Model)
- **Type**: data-visualization
- **Description**: Per-student per-concept state: student_id, concept_id, mastery_estimate, memory_strength, recognition_strength, reconstruction_strength, reasoning_strength, transfer_strength, retrieval_speed_score, pressure_tolerance, retention_decay_rate, confidence_calibration, misconception_risks_json, recent_success_rate, recent_attempt_count, last_seen_at, next_review_due_at.
- **User Role**: student (data subject), parent/teacher (viewer)
- **UI Details**: Powers diagnostic displays, knowledge gap reports, and adaptive question selection.
- **Sub-features**: 17 state dimensions

---

### 31
- **Feature Name**: AttemptRecord Entity
- **Type**: engine-output-needing-UI
- **Description**: Stores every answer event: attempt_id, student_id, generated_question_id, concept_id, mode_context, timestamp, correctness, response_time_ms, hints_used, confidence_rating, final_answer_json, selected_option_json, detected_error_type, misconception_triggered_id, partial_credit, recovery_after_error, analysis_json.
- **User Role**: admin (analytics)
- **UI Details**: Powers analytics dashboards and learning reports.
- **Sub-features**: All fields

---

### 32
- **Feature Name**: Review Queue System
- **Type**: engine-output-needing-UI
- **Description**: For spaced retrieval and resurfacing: review_queue_id, student_id, concept_id, review_type (retention_check, recovery, fluency, misconception_repair, transfer_check), due_at, priority, source_reason, status.
- **User Role**: student (receives scheduled reviews)
- **UI Details**: Powers spaced repetition scheduling and "what to study next" recommendations.
- **Sub-features**: 5 review types

---

### 33
- **Feature Name**: Question Factory Backend Modules (14 Modules)
- **Type**: engine-output-needing-UI
- **Description**: Backend module architecture: 1. Curriculum Mapper, 2. Concept Builder, 3. Question Intent Selector, 4. Family Resolver, 5. Primitive Extractor, 6. Transformation Engine, 7. Variant Composer, 8. Difficulty Controller, 9. Surface Renderer, 10. Validator, 11. Response Analyzer, 12. Learner Model Updater, 13. Delivery Engine, 14. Adaptation Engine.
- **User Role**: admin
- **UI Details**: No direct student UI; entirely backend architecture.
- **Sub-features**: All 14 modules with individual responsibilities

---

### 34
- **Feature Name**: Surface Rendering Formats (9 Types)
- **Type**: engine-output-needing-UI
- **Description**: The question engine can render into: Multiple Choice, Short Answer, Open Response, Drag and Order, Match Pair, Diagram Label, Step Continuation, Rapid Fire, Audio Prompt.
- **User Role**: student
- **UI Details**: Each format has distinct UI components (buttons, input boxes, drag targets, audio player, etc.)
- **Sub-features**: 9 format types

---

### 35
- **Feature Name**: Mode Contexts (7 Modes)
- **Type**: mode
- **Description**: The question factory serves 7 mode contexts: Memory, KnowledgeGap, Mental, Journey, Elite, DNA, CustomPrep. Each mode has different question family preferences, constraints, and learner-state update rules.
- **User Role**: student
- **UI Details**: Mode selection UI; each mode uses different question families and difficulty rules.
- **Sub-features**: All 7 modes

---

### 36
- **Feature Name**: Mode-Specific Question Selection Constraints
- **Type**: engine-output-needing-UI
- **Description**: Each mode imposes specific API-level constraints on question generation: Memory Mode (prefer prior concepts, delayed resurfacing, lower cue first), Knowledge Gap Mode (prioritize unknown/unstable concepts, diagnostic traps, branch on failure), Mental Mode (timing profile, rapid-response formats, short items, streak chains), Elite Mode (within curriculum, high disguise/reasoning depth, multi-step/transfer), DNA Mode (maximize diagnostic value, high-detail analytics, branching families, misconception detection).
- **User Role**: admin (configuration)
- **UI Details**: No direct UI; constrains engine behavior per mode.
- **Sub-features**: 5 mode-specific constraint sets

---

### 37
- **Feature Name**: Branching Logic After Student Response
- **Type**: engine-output-needing-UI
- **Description**: Post-answer branching: correct and fast (increase difficulty/move to transfer), correct but slow (queue speed/pressure questions), correct but low confidence (treat as fragile, schedule reinforcement), wrong with misconception pattern (launch targeted repair), wrong but close (move to reconstruction/recovery), wrong at first-step (repair entry strategy), right on recognition but wrong on recall (memory not stable), right on recall but wrong on transfer (not deeply understood), right untimed but wrong under pressure (pressure tolerance issue).
- **User Role**: student (experiences adaptive flow)
- **UI Details**: Drives what happens next in the learning flow; no explicit UI for the branching itself.
- **Sub-features**: 9 branching conditions

---

### 38
- **Feature Name**: Multi-Dimensional Scoring Per Attempt
- **Type**: data-visualization
- **Description**: Each attempt can produce: correctness score, speed score, confidence score, explanation quality score, misconception likelihood score, process quality score, retention evidence score, pressure stability score.
- **User Role**: student (data subject), teacher/parent (viewer)
- **UI Details**: Powers detailed diagnostic reports beyond simple right/wrong.
- **Sub-features**: 8 scoring dimensions

---

### 39
- **Feature Name**: Hybrid Question Generation (Deterministic + Language)
- **Type**: engine-output-needing-UI
- **Description**: Two-layer approach: a deterministic pedagogical core (concept graph, solution logic, misconception targeting, difficulty control, answer validation, sequencing, scoring) and a language generation layer (varied wording, cleaner phrasing, scenario dressing, freshness in expression). Pedagogy engine decides intent; language generation only surfaces it.
- **User Role**: admin
- **UI Details**: No direct UI; architecture principle.
- **Sub-features**: None

---

### 40
- **Feature Name**: Question Metadata Object
- **Type**: engine-output-needing-UI
- **Description**: Every generated question carries metadata: question_id, concept_id, topic_id, subskill_id, cognitive_type, evaluative_intent, difficulty_score, format_type, time_mode, misconception_target, solution_graph_id, representation_type, freshness_signature, prerequisite_ids, expected_evidence_of_mastery, scoring_logic, explanation_logic, retry_logic.
- **User Role**: admin
- **UI Details**: Used for analytics and diagnostics; not shown directly to students.
- **Sub-features**: 18 metadata fields

---

### 41
- **Feature Name**: Rust Backend Architecture
- **Type**: admin-tool
- **Description**: Full Rust project structure with domain modules (curriculum, concepts, learner, question_factory, attempts, reviews), infrastructure (db, sqlite, migrations, repositories), API commands (generate_question, submit_attempt, get_due_reviews), and utils. Uses SQLite for local offline storage with strongly typed Rust models.
- **User Role**: admin (developer)
- **UI Details**: No student UI; developer architecture.
- **Sub-features**: 14+ Rust modules, Tauri command bridge

---

### 42
- **Feature Name**: Rust Enum Types for Engine
- **Type**: engine-output-needing-UI
- **Description**: Strong Rust enums: PrimitiveType (14 variants), CognitiveType (24 variants including Recall, Recognition, Reconstruction, ConceptUnderstanding, Explanation, Reasoning, Application, Transfer, FirstStep, NextStep, ErrorDetection, Correction, MisconceptionExposure, CompareContrast, Classification, CauseEffect, Prediction, PatternRecognition, RuleDiscovery, RepresentationConversion, RetrievalUnderPressure, RetentionCheck, Recovery, Stretch), FormatType (9 variants), ModeContext (7 variants).
- **User Role**: admin (developer)
- **UI Details**: No direct UI; backend type safety.
- **Sub-features**: 4 enum categories

---

### 43
- **Feature Name**: Service Interfaces (Traits)
- **Type**: engine-output-needing-UI
- **Description**: Core service traits: QuestionFactoryService (generate_question), AttemptProcessingService (process_attempt), LearnerStateService (get_or_create_state, update_from_attempt), ReviewQueueService (schedule_review, get_due_reviews), QuestionFamilyGenerator (generate, validate, analyze_attempt per family).
- **User Role**: admin (developer)
- **UI Details**: No direct UI; API contracts.
- **Sub-features**: 5 service interfaces

---

### 44
- **Feature Name**: Family-Specific Validation Rules
- **Type**: engine-output-needing-UI
- **Description**: Each question family has its own validator: Recall validator (atomic target, no answer leaks), Recognition validator (one best answer, plausible distractors), Error Detection validator (single intended fault, correction available), First-Step validator (correct first move in solution graph), Transfer validator (deep structure preserved, surface changed), Pressure validator (reading load appropriate to timer).
- **User Role**: admin
- **UI Details**: No direct UI; quality assurance.
- **Sub-features**: 6 family-specific validator sets

---

### 45
- **Feature Name**: 12 Core Question Family Implementation Specs
- **Type**: engine-output-needing-UI
- **Description**: Detailed implementation specs for: 1. Recall, 2. Recognition, 3. Reconstruction, 4. Explanation, 5. Application, 6. Error Detection, 7. Misconception Exposure, 8. First-Step, 9. Next-Step, 10. Retention Check, 11. Recovery, 12. Retrieval Under Pressure. Each spec defines purpose, when to use, required inputs, generation logic, output structure (Rust struct), scoring logic, validator rules, learner-state update effects, common failure cases, and mode usage.
- **User Role**: all
- **UI Details**: Each family has a distinct Rust payload struct. Output rendered as various surface formats.
- **Sub-features**: All 12 family specs with complete implementation contracts

---

### 46
- **Feature Name**: Logging and Observability System
- **Type**: admin-tool
- **Description**: For each generated question, logs: selected concept, learner-state snapshot, chosen evaluative intent, chosen family, extracted primitive ids, selected misconception target, difficulty profile, variant signature, validation notes. For each attempt, logs: correctness, response time, misconception detection, scoring breakdown, state update deltas, next action chosen.
- **User Role**: admin
- **UI Details**: Admin debugging/analytics dashboard.
- **Sub-features**: Generation logs, attempt logs

---

### 47
- **Feature Name**: Phased Build Order for Question Factory
- **Type**: engine-output-needing-UI
- **Description**: 5-phase implementation plan: Phase 1 (storage, domain foundation, 5 families), Phase 2 (runtime generation skeleton), Phase 3 (response intelligence with attempt processing, learner state updater, review queue), Phase 4 (process/diagnostic power with solution graphs, first-step, next-step, correction, misconception exposure, compare-contrast), Phase 5 (memory and elite intelligence with retention check, recovery, retrieval under pressure, transfer, representation conversion, stretch).
- **User Role**: admin (developer)
- **UI Details**: No direct UI; development roadmap.
- **Sub-features**: 5 build phases

---

## FEATURES FOUND IN idea10.txt

---

### 1
- **Feature Name**: Tug of War Educational Game
- **Type**: mode
- **Description**: An educational game where correct answers pull a virtual rope toward mastery and wrong answers let it slip. Knowledge strength + speed + consistency = victory. The game is not cosmetic; learning drives the mechanics directly.
- **User Role**: student
- **UI Details**: Visual rope with pulling animation, momentum meter, power-ups, rope zones, comeback system.
- **Sub-features**:
  - Solo Tug of War against AI (enemies: Confusion, The Fog, Error Monster, Time Beast, Exam Pressure, Forgetfulness)
  - Student vs Student Tug of War (same questions/timer, adaptive difficulty, equivalent questions)
  - Team Classroom Tug of War (Red vs Blue teams, teacher projects rope on screen)
  - Topic Tug of War (topic as opponent, e.g., Fractions vs student)
  - Misconception Tug of War (rope tied to a specific real weakness)

---

### 2
- **Feature Name**: Tug of War Momentum Meter
- **Type**: component
- **Description**: Tracks consecutive correct answers to increase pull strength. 1 correct = normal pull, 3 streak = stronger pull, 5 streak = "Overdrive Pull." Rewards consistency.
- **User Role**: student
- **UI Details**: Visual meter showing streak progress and pull strength multiplier.
- **Sub-features**: Normal pull, stronger pull, Overdrive Pull

---

### 3
- **Feature Name**: Tug of War Power-Ups
- **Type**: component
- **Description**: Performance-based power-ups (not random luck): Freeze Slip (protects against one wrong answer), Double Pull (next correct counts twice), Time Shield (adds 3 extra seconds), Hint Rope (small clue but weaker reward), Misconception Scan (reveals trap to avoid).
- **User Role**: student
- **UI Details**: Power-up icons that activate based on performance streaks.
- **Sub-features**: 5 power-up types

---

### 4
- **Feature Name**: Tug of War Rope Zones
- **Type**: component
- **Description**: Visual zones on the rope: Neutral Zone, Pressure Zone, Recovery Zone, Victory Zone, Collapse Zone. When learner is in Collapse Zone, sound and visuals intensify.
- **User Role**: student
- **UI Details**: Color-coded zones on the rope with changing audio/visual intensity.
- **Sub-features**: 5 zone types

---

### 5
- **Feature Name**: Tug of War Comeback System
- **Type**: interaction
- **Description**: Prevents one early mistake from ending the fun. After 2 wrong answers, learner can earn a recovery pull. Special "Last Grip" question can reverse momentum.
- **User Role**: student
- **UI Details**: Recovery pull animation, Last Grip special question indicator.
- **Sub-features**: Recovery pull, Last Grip question

---

### 6
- **Feature Name**: Tug of War Pull Strength by Difficulty
- **Type**: interaction
- **Description**: Not every question has same pull strength: easy = small pull, medium = medium pull, hard = strong pull, challenge = giant pull. Rewards real mastery over fast tapping.
- **User Role**: student
- **UI Details**: Visual indicator of pull strength tied to question difficulty badge.
- **Sub-features**: 4 pull strength levels

---

### 7
- **Feature Name**: Educational Tetris Game
- **Type**: mode
- **Description**: Falling blocks that carry educational content (numbers, words, formulas, concepts, symbols, sentence fragments). Learners must place blocks where they belong based on knowledge. The meaning of the block itself matters, not just answering a quiz to trigger a piece.
- **User Role**: student
- **UI Details**: Falling block board with educational content on blocks, structured knowledge space below.
- **Sub-features**:
  - Answer-fit Tetris (place content where it belongs)
  - Equation Tetris (arrange numbers/operators/variables to form valid equations)
  - Word Tetris (build words from letters/syllables/prefixes/suffixes/roots)
  - Concept-stack Tetris (build semantic knowledge towers)
  - Sequence Tetris (place blocks in correct order)
  - Geometry Tetris (area estimation, perimeter comparison, fraction coverage)

---

### 8
- **Feature Name**: Answer-fit Tetris
- **Type**: sub-mode
- **Description**: Falling blocks contain numbers, words, formulas, concepts, symbols, or sentence fragments. The learner places them where they belong (e.g., sort into categories: mammals/reptiles, metals/non-metals, nouns/verbs/adjectives). The board becomes a structured knowledge space.
- **User Role**: student
- **UI Details**: Columns labeled with categories; blocks fall with content to be placed.
- **Sub-features**: Math version, Language version, Science version

---

### 9
- **Feature Name**: Equation Tetris
- **Type**: sub-mode
- **Description**: Blocks fall as numbers, variables, operators, brackets, exponents. Learner must arrange them to form valid equations, match a target value, or solve mini puzzles before the stack rises. Develops number sense, operator fluency, strategic assembly.
- **User Role**: student
- **UI Details**: Target value display, falling math blocks, equation assembly area.
- **Sub-features**: None

---

### 10
- **Feature Name**: Word Tetris
- **Type**: sub-mode
- **Description**: Blocks carry letters, syllables, prefixes, suffixes, root words, meaning fragments. Learners build correct words, longest word possible, words matching a definition, words matching part of speech. Modes: spelling, vocabulary, word family, foreign language.
- **User Role**: student
- **UI Details**: Letter/syllable blocks falling, word construction area.
- **Sub-features**: 4 sub-modes (spelling, vocabulary, word family, foreign language)

---

### 11
- **Feature Name**: Concept-stack Tetris (Semantic Tetris)
- **Type**: sub-mode
- **Description**: Instead of matching shapes physically, learner builds knowledge towers. Blocks with concept names must stack above their prerequisites/components. A block can only clear when connected to the right process chain.
- **User Role**: student
- **UI Details**: Concept blocks with labels, dependency-based stacking rules.
- **Sub-features**: None

---

### 12
- **Feature Name**: Sequence Tetris
- **Type**: sub-mode
- **Description**: Blocks must be placed in the right order (steps in long division, water cycle stages, scientific method, essay structure, historical events, life cycle stages). Board only clears when sequence is correct. Teaches procedural knowledge.
- **User Role**: student
- **UI Details**: Sequential placement area, clear animation on correct order.
- **Sub-features**: None

---

### 13
- **Feature Name**: Geometry Tetris
- **Type**: sub-mode
- **Description**: Most natural adaptation of Tetris. Builds: area estimation, perimeter comparison, angle recognition, symmetry, tessellation, fraction coverage. Target grid provides math conditions (e.g., "fill exactly 3/4 of the rectangle").
- **User Role**: student
- **UI Details**: Grid with math constraint display, shape pieces to fill.
- **Sub-features**: 6 geometry skills

---

### 14
- **Feature Name**: Educational Tetris Game Concepts (6 Specific Games)
- **Type**: mode
- **Description**: Six specific educational Tetris game concepts: 1. Grammar Grid (words into noun/verb/adjective/adverb columns), 2. Fraction Builder (combine fractions to make wholes/equivalents/targets), 3. Chemistry Stack (form valid compounds/balance reactions), 4. Sentence Constructor (build grammatically correct sentences), 5. History Timeline Stack (chronological/causal event ordering), 6. Algebra Tower (simplify/form equations from terms).
- **User Role**: student
- **UI Details**: Each has a themed board (grammar columns, chemistry table, timeline, etc.)
- **Sub-features**: All 6 game concepts

---

### 15
- **Feature Name**: Hybrid Tug of War + Tetris
- **Type**: mode
- **Description**: Three hybrid concepts: A. Tetris board controls Tug of War strength (correct Tetris builds give pull force), B. Tug of War to unlock Tetris rescue mode (clear 2 rows to recover rope position), C. Two-player battle format (each player has Tetris board, better builds power their Tug of War pull).
- **User Role**: student
- **UI Details**: Split screen with both games visible.
- **Sub-features**: 3 hybrid modes

---

### 16
- **Feature Name**: Tug of War Progression System
- **Type**: component
- **Description**: Ranks (Rookie Puller, Rope Master, Titan Grip), topic arenas, boss battles against difficult topics, streak flames, pressure leagues, comeback medals.
- **User Role**: student
- **UI Details**: Rank badges, arena selection screen, boss battle UI, streak flame animation, league table, medal display.
- **Sub-features**: Ranks, arenas, boss battles, streaks, leagues, medals

---

### 17
- **Feature Name**: Tetris Progression System
- **Type**: component
- **Description**: Build cities/labs/towers from cleared boards, unlock themes by subject, special blocks tied to mastery, speed classes, daily puzzle boards, "perfect fit" bonuses.
- **User Role**: student
- **UI Details**: City/lab/tower building visualization, theme unlocks, daily puzzle board screen.
- **Sub-features**: City building, theme unlocks, mastery blocks, speed classes, daily puzzles, perfect fit bonuses

---

### 18
- **Feature Name**: Premium Game Naming
- **Type**: component
- **Description**: Branded names instead of generic labels. Tug of War options: MindPull, BrainRope, ClashLine, GripForce, Recall Tug, Pressure Pull, RopeRush. Tetris options: MindStack, Logic Blocks, GridWise, StackLab, Concept Stack, BuildIQ, SmartStack.
- **User Role**: all
- **UI Details**: Brand name displayed in game mode cards, title screens, and navigation.
- **Sub-features**: 14 name options

---

### 19
- **Feature Name**: MindStack (Control-Through-Knowledge Tetris)
- **Type**: mode
- **Description**: The core educational Tetris variant. A block falls, a question appears on the side panel. Correct answer unlocks block control (movement, rotation, reshaping, slowdown). Wrong answer means the block falls faster in its original shape with limited control. The core law: Knowledge gives control; Uncertainty reduces control; Wrongness increases danger.
- **User Role**: student
- **UI Details**: Split-screen: left panel (question zone ~35-40% width), right panel (game board ~60-65% width). Control status indicator (Locked, Partial Control, Full Control, Morph Ready).
- **Sub-features**:
  - Answer-to-Control mechanic
  - Answer-to-Rotation mechanic
  - Answer-to-Morph mechanic
  - Answer-to-Time mechanic
  - Dual-answer mechanic (two questions per block)

---

### 20
- **Feature Name**: MindStack Control Ladder
- **Type**: interaction
- **Description**: Graduated control levels based on answer quality: Level 0 (No mastery: fast fall, no rotation, no movement, original shape), Level 1 (Partial: limited movement, normal speed), Level 2 (Good: full movement, rotation, slower fall), Level 3 (Excellent: full movement, rotation, reshape option, gravity slowdown, score multiplier).
- **User Role**: student
- **UI Details**: Control permission icons (arrows, rotate, morph, slow, hold) with states: greyed out = locked, lit = active, glowing = premium/streak-powered.
- **Sub-features**: 4 control levels

---

### 21
- **Feature Name**: MindStack Reshape/Morph Mechanic
- **Type**: interaction
- **Description**: Signature feature. Correct answer earns the right to transform the block shape. Options: Option A (one-time morph to a better shape), Option B (choose from 3 morphs), Option C (partial morph - add/remove one unit), Model D (intelligent rescue morph suggesting most useful shape based on board state).
- **User Role**: student
- **UI Details**: Morph button with tap-cycle, hold-for-radial-selector, or auto-suggest options. Morph icon lights up when available.
- **Sub-features**: 4 morph models (Fixed menu, Tiered, Subject-linked, Intelligent rescue)

---

### 22
- **Feature Name**: MindStack Question-to-Power Mapping
- **Type**: interaction
- **Description**: Different cognitive question types unlock different gameplay powers: Recall question = unlock movement, Concept question = unlock rotation, Reasoning question = unlock reshaping, Fast answer streak = temporary gravity slowdown / board clear bomb / shield.
- **User Role**: student
- **UI Details**: Question difficulty badge that hints at the control reward available.
- **Sub-features**: 4 question-to-power mappings

---

### 23
- **Feature Name**: MindStack Mercy Design System
- **Type**: interaction
- **Description**: Anti-frustration rules: 1. Wrong should hurt but not instantly kill, 2. Allow recovery (earn control back on next question), 3. Grace zone (early levels allow emergency moves even on wrong), 4. Streak forgiveness (4 correct + 1 wrong doesn't collapse everything), 5. Rescue mechanics (stabilize block token, retry answer token, slow gravity power-up).
- **User Role**: student
- **UI Details**: Rescue token icons, grace zone indicator.
- **Sub-features**: 5 mercy rules, 3 rescue mechanics

---

### 24
- **Feature Name**: MindStack Block-Question Type Matching
- **Type**: interaction
- **Description**: The kind of question affects the kind of block: Recall questions = simple shapes, Reasoning questions = awkward shapes, Mastery questions = rare shapes with big rewards, Misconception questions = trap blocks dangerous unless answered correctly. Difficulty becomes visible in gameplay form.
- **User Role**: student
- **UI Details**: Visual block shapes that telegraph question difficulty.
- **Sub-features**: 4 block-question type pairings

---

### 25
- **Feature Name**: MindStack Scoring Model
- **Type**: data-visualization
- **Description**: Total Score = Answer Score + Control Bonus + Placement Score + Board Efficiency + Streak Multiplier + Pressure Bonus. Answer Score (correctness, difficulty, speed), Placement Score (clean fit, row clear, multi-row clear), Board Efficiency (low holes, low stack height, structural cleanliness), Streak Multiplier (consecutive correct), Pressure Bonus (correct answers during high danger).
- **User Role**: student
- **UI Details**: Score display in top HUD, detailed breakdown on results screen.
- **Sub-features**: 6 scoring components

---

### 26
- **Feature Name**: MindStack Partial Credit Answers
- **Type**: interaction
- **Description**: Not every wrong answer is equally wrong. Correct = full control, near-correct reasoning = partial control, wrong = no control. Especially good for multi-step subjects.
- **User Role**: student
- **UI Details**: Partial control permissions granted for near-correct answers.
- **Sub-features**: None

---

### 27
- **Feature Name**: MindStack Confidence Mode
- **Type**: sub-mode
- **Description**: After answering, student indicates confidence (sure / not sure). Correct and confident = bigger reward. Correct but unsure = smaller reward. Helps engine understand whether knowledge is truly stable.
- **User Role**: student
- **UI Details**: Confidence toggle (sure / not sure) in question panel.
- **Sub-features**: None

---

### 28
- **Feature Name**: MindStack Misconception-Aware Blocks
- **Type**: interaction
- **Description**: If the system knows a student struggles with a topic (e.g., fractions), blocks tied to fraction questions appear more often until mastery improves. The game becomes adaptive.
- **User Role**: student
- **UI Details**: No explicit UI indicator; adaptive behavior.
- **Sub-features**: None

---

### 29
- **Feature Name**: MindStack Boss Blocks
- **Type**: interaction
- **Description**: Rare blocks that are huge opportunities. A "boss block" appears with a harder question. Correct answer grants: massive reshape freedom, gravity freeze, instant row repair.
- **User Role**: student
- **UI Details**: Visually distinct boss block with special effects.
- **Sub-features**: 3 boss block rewards

---

### 30
- **Feature Name**: MindStack Combo Knowledge Chains
- **Type**: interaction
- **Description**: Three correct answers in a row activate: free block choice, auto-fit guide, double line clear, shield from next wrong answer.
- **User Role**: student
- **UI Details**: Combo counter, chain activation animation.
- **Sub-features**: 4 combo rewards

---

### 31
- **Feature Name**: MindStack Product Identity and Naming
- **Type**: component
- **Description**: Recommended product names: MindStack, AnswerDrop, ThinkStack, ControlFall, BrainBlocks, LogicDrop, StackShift, MasterDrop. MindStack recommended as most premium. Core tagline: "Answer to control the fall."
- **User Role**: all
- **UI Details**: Large title on home screen with tagline subtext.
- **Sub-features**: 8 name options

---

### 32
- **Feature Name**: MindStack Core Gameplay Loop (5 Phases)
- **Type**: flow
- **Description**: Phase 1: Block spawn (block appears, question appears, locked-control state). Phase 2: Question response window (answer before timer/threshold/lock). Phase 3: Answer evaluation (4 outcomes: perfect correct, correct, partial/near-correct, wrong/timeout). Phase 4: Placement window (place block with earned control). Phase 5: Lock + resolve (rows clear, score awarded, streaks update, mastery data updates, next block begins).
- **User Role**: student
- **UI Details**: Visible state transitions between phases.
- **Sub-features**: 5 gameplay phases, 4 answer outcome types

---

### 33
- **Feature Name**: MindStack 4 Real-Time Engines
- **Type**: engine-output-needing-UI
- **Description**: Four engines operating simultaneously: A. Question Engine (chooses what to ask), B. Control Engine (translates answer quality into gameplay permissions), C. Board Engine (manages blocks, gravity, collisions, rows, danger, combos), D. Learning Intelligence Engine (tracks what student really knows, hesitates on, misses under pressure, what collapses when speed increases).
- **User Role**: student (experiences), admin (configures)
- **UI Details**: Board reflects cognitive performance in real time.
- **Sub-features**: 4 engine types

---

### 34
- **Feature Name**: MindStack Control Permissions System (8 Privileges)
- **Type**: interaction
- **Description**: 8 control privileges: 1. Lateral movement (left/right), 2. Rotation, 3. Soft drop control, 4. Hard drop control, 5. Shape morph, 6. Gravity slowdown, 7. Save/hold privilege, 8. Rescue privilege (undo bad orientation).
- **User Role**: student
- **UI Details**: Control rack with icons showing locked/active/glowing states.
- **Sub-features**: 8 privilege types

---

### 35
- **Feature Name**: MindStack Answer-to-Control Matrix
- **Type**: setting
- **Description**: Configurable reward matrix by difficulty level: Beginner (wrong = small speed increase + 1 emergency move, correct = move + rotate, fast correct = + slowdown, streak = + morph), Intermediate (wrong = faster fall + no rotation, correct = full movement + rotation, hard correct = morph privilege), Advanced (wrong = no control + fast fall + locked shape, perfect streak = morph + slowdown + hold + multiplier).
- **User Role**: admin (configuration)
- **UI Details**: Difficulty level selection screen.
- **Sub-features**: 3 difficulty tiers with per-tier reward bundles

---

### 36
- **Feature Name**: MindStack Morph System (4 Models)
- **Type**: interaction
- **Description**: Model A: Fixed morph menu (keep, line, square, L-shape). Model B: Tiered morph (correct = 2 options, fast correct = 3 options, perfect streak = any block). Model C: Subject-linked morph (recall = rotate only, reasoning = full morph, hard challenge = any-shape). Model D: Intelligent rescue morph (board-aware suggestion of most useful shape).
- **User Role**: student
- **UI Details**: Quick radial selector or tap-cycle for morph choices.
- **Sub-features**: 4 morph models

---

### 37
- **Feature Name**: MindStack Question Types Best for Mode
- **Type**: engine-output-needing-UI
- **Description**: 5 best question families: 1. Fast recall (unlock movement/rotation), 2. Recognition/discrimination (unlock rotation/slowdown), 3. Procedural next-step (unlock morph/combo bonus), 4. Misconception trap (higher reward), 5. Confidence-check (correct + sure = bigger reward). Question types to avoid: long word problems, essays, large reading passages, heavy derivation.
- **User Role**: student
- **UI Details**: Question type badges (Quick Choice, Binary Strike, Step Choice, Trap Choice, Boss Choice).
- **Sub-features**: 5 in-game question types

---

### 38
- **Feature Name**: MindStack Subject-Specific Adaptations
- **Type**: mode
- **Description**: Mathematics (arithmetic, factorization, fraction equivalence, next-step algebra, equation completion, geometry facts), English/Language (grammar, synonym/antonym, punctuation, part of speech, spelling, sentence correction), Science (concept recognition, terminology, process sequence, organ/system function, classification, symbol matching), Social Studies/History (dates, chronology, cause/effect, person/event matching, geography).
- **User Role**: student
- **UI Details**: Subject-themed boards (lab chamber for science, etc.)
- **Sub-features**: 4 subject adaptations

---

### 39
- **Feature Name**: MindStack 5-Dimension Difficulty Model
- **Type**: setting
- **Description**: 5 difficulty dimensions: 1. Gravity speed, 2. Question complexity, 3. Answer time, 4. Control forgiveness, 5. Board punishment. Level progression: Levels 1-3 (easy), 4-6 (moderate), 7-9 (hard), 10+ (mixed types, adaptive weakness targeting, high-speed, boss rounds).
- **User Role**: student
- **UI Details**: Difficulty selection screen with themed labels (Calm, Focused, Fast, Intense, Relentless) showing fall speed, question type, forgiveness level, rewards multiplier.
- **Sub-features**: 5 difficulty dimensions, 4+ level tiers, 5 themed difficulty labels

---

### 40
- **Feature Name**: MindStack Anti-Frustration / Recovery Architecture
- **Type**: interaction
- **Description**: 5 recovery rules: 1. Don't destroy with one wrong answer (slightly faster fall, limited movement), 2. Allow bounce-back (single correct restores control), 3. Danger-aware mercy (easier questions when board near collapse), 4. Keep failure informative (show what question types hurt, speed/confusion/misconception), 5. Separate panic from ignorance (tag pressure failures differently from true weakness).
- **User Role**: student
- **UI Details**: Recovery indicators, diagnostic failure display.
- **Sub-features**: 5 recovery rules

---

### 41
- **Feature Name**: MindStack Adaptive Intelligence / Learner States
- **Type**: data-visualization
- **Description**: Tracks per student: answer correctness, speed, confidence, failure under gravity, recognition vs recall performance, misconception-caused collapse, recovery speed. Produces invisible learner states: Stable mastery, Fragile mastery, Guess-heavy, Misconception-bound, Slow understanding.
- **User Role**: student (data subject), teacher/parent (viewer)
- **UI Details**: Post-game insights and diagnostic reports.
- **Sub-features**: 5 learner state categories, 7 tracking dimensions

---

### 42
- **Feature Name**: MindStack Sub-Modes (7 Modes)
- **Type**: sub-mode
- **Description**: A. Classic Control (base version), B. Morph Mode (premium signature - emphasis on reshaping), C. Survival Mode (endless, escalating difficulty), D. Weakness Hunt Mode (feeds weak area questions), E. Memory Rescue Mode (targets decaying knowledge), F. Exam Pressure Mode (faster pace, tighter timers, pressure sounds), G. Elite Stack (harder questions, less forgiveness).
- **User Role**: student
- **UI Details**: Mode cards with icon, description, best score, mastery indicator, recommended/locked/new tag.
- **Sub-features**: 7 sub-modes

---

### 43
- **Feature Name**: MindStack Boss Rounds
- **Type**: interaction
- **Description**: Boss round every few levels or after streak milestone. Board starts in dangerous state, special large-value question appears, correct = powerful rescue privileges, wrong = severe pressure escalation. Boss themes are topic-based (Fractions Boss, Algebra Boss, Grammar Boss, Forces Boss, Ecology Boss).
- **User Role**: student
- **UI Details**: Themed boss UI with dramatic visual/audio treatment.
- **Sub-features**: 5 topic-based boss themes

---

### 44
- **Feature Name**: MindStack Power-Ups (8 Types)
- **Type**: component
- **Description**: Earned through learning performance: Slow Time, Shape Shift, Smart Rotate, Safe Hold, Retry Answer, Misconception Shield, Row Repair, Stability Boost. Tied to streaks, hard correct answers, topic mastery, comeback performance.
- **User Role**: student
- **UI Details**: Power-up icons with usage count.
- **Sub-features**: 8 power-up types

---

### 45
- **Feature Name**: MindStack Scoring System
- **Type**: data-visualization
- **Description**: Total Score = Answer Score + Control Bonus + Placement Score + Board Efficiency + Streak Multiplier + Pressure Bonus + Recovery Bonus + Stability Bonus. Each component depends on specific performance factors (tier, difficulty, topic weight, speed band, permissions, morph usage, lines cleared, combo depth, danger band, stabilization after failure).
- **User Role**: student
- **UI Details**: Score in top HUD, detailed breakdown on results screen.
- **Sub-features**: 8 scoring components

---

### 46
- **Feature Name**: MindStack Secret Analytics System
- **Type**: data-visualization
- **Description**: Behind-the-scenes tracking per question: topic, subtopic, difficulty, response correctness, response speed, confidence, control reward earned, gameplay outcome, block placement success, line clear result. Per run: pressure accuracy, calm accuracy baseline, panic-loss frequency, recovery success rate, morph efficiency, collapse cause, board efficiency, weak-topic concentration.
- **User Role**: admin, teacher, parent
- **UI Details**: Powers knowledge gap, memory mode, elite mode, lowest-to-best mode, personal improvement reports.
- **Sub-features**: 10 per-question metrics, 8 per-run metrics

---

### 47
- **Feature Name**: MindStack UI Layout Spec
- **Type**: screen
- **Description**: Split-screen layout. Left panel (~35-40% width): question text, answer choices/input, timer bar, difficulty tag, topic label, current streak, confidence selector, streak indicator, reward preview (move/rotate/morph/slow/multiplier icons). Right panel (~60-65% width): main board, falling block, ghost placement preview, next block preview, hold area, danger meter, control state badge, combo/line clear pulse. Top bar: score, lines cleared, mode name, level/wave number, lives/run health, control status, pause icon.
- **User Role**: student
- **UI Details**: Precise layout specifications with component placement.
- **Sub-features**: Left panel (8 components), Right panel (8 components), Top bar (7 components)

---

### 48
- **Feature Name**: MindStack Audio and Feel System
- **Type**: sound
- **Description**: Layered sound design: Layer 1 (ambient pressure - increases with board risk), Layer 2 (answer events - correct/wrong/timeout sounds), Layer 3 (board events - drop, lock, line clear, combo, morph), Layer 4 (streak/danger signals). Correct answer: sharp satisfying sound, quick glow pulse, control icons unlock. Wrong answer: heavier tone, gravity sound intensifies, block outline flashes danger, question panel shakes. Streak: rising energy, visual heat, combo meter crackles, morph icon lights up. Near-collapse: deeper ambience, screen pressure, faster pulse, danger meter throbs.
- **User Role**: student
- **UI Details**: 4 audio layers with specific visual feedback for each state.
- **Sub-features**: 4 feedback categories (correct, wrong, streak, near-collapse)

---

### 49
- **Feature Name**: MindStack Primary User Flow
- **Type**: flow
- **Description**: Full user journey: Entry (app > Games/Challenge Modes > MindStack card > MindStack home), Pre-game (choose mode > choose topic/auto-select > choose difficulty > optional boosters/loadout > start round), In-game (block spawns > question appears > answer > control unlocks > block placed > board resolves > next block), Post-game (round ends > score screen > performance breakdown > knowledge insight > weaknesses detected > next suggested action).
- **User Role**: student
- **UI Details**: Screen-by-screen flow with transitions.
- **Sub-features**: 4 flow phases (entry, pre-game, in-game, post-game)

---

### 50
- **Feature Name**: MindStack Home Screen
- **Type**: screen
- **Description**: Top area: large title "MindStack" with subtext "Answer to control the fall." Hero card with falling block board, question panel, streak flame, rank badge, and CTA buttons (Quick Play, Continue Last Run). Mid-section: mode cards as premium tiles (Classic Control, Morph Mode, Weakness Hunt, Survival, Memory Rescue, Exam Pressure, Elite Stack) each showing icon, description, best score, mastery indicator, recommended/locked/new tag. Bottom: 3 insight cards (Your current sharpness, Weakness under pressure, Suggested next mission).
- **User Role**: student
- **UI Details**: Hero card, mode tiles, 3 insight cards with specific metrics.
- **Sub-features**: Hero card, 7 mode tiles, 3 insight cards

---

### 51
- **Feature Name**: MindStack Topic Selection Screen
- **Type**: screen
- **Description**: After choosing a mode, student selects content: A. Specific topic (Fractions, Algebra, Grammar, Ecology, Forces), B. Subject mix (Full Math Mix, Science Blend, English Challenge), C. Weakness-based auto mode (engine chooses from weakest areas), D. Memory risk mode (material likely to be forgotten), E. Exam syllabus track (content according to current syllabus plan and exam timeline).
- **User Role**: student
- **UI Details**: Topic selection cards/list with 5 selection modes.
- **Sub-features**: 5 content selection modes

---

### 52
- **Feature Name**: MindStack Difficulty Selection Screen
- **Type**: screen
- **Description**: Themed difficulty labels: Calm, Focused, Fast, Intense, Relentless. Each affects: block gravity, answer timer, control forgiveness, trap question count, morph access difficulty, recovery support. Cards show fall speed, average question type, forgiveness level, rewards multiplier.
- **User Role**: student
- **UI Details**: Difficulty cards with theme descriptions and stats.
- **Sub-features**: 5 difficulty levels, 6 affected dimensions

---

### 53
- **Feature Name**: MindStack Loadout / Boosters Screen
- **Type**: screen
- **Description**: 2-3 booster slots. Available boosters: Slow Time once, Retry one question, Emergency Rotate, Stability Shield, Save one block, Trap Detector. Shows available boosters, descriptions, uses remaining, recommended booster for mode. Boosters earned through performance, not random purchase.
- **User Role**: student
- **UI Details**: Booster slot cards with usage counts and recommendations.
- **Sub-features**: 6 booster types

---

### 54
- **Feature Name**: MindStack Countdown and Round Intro
- **Type**: screen
- **Description**: 3-2-1 countdown with quick tip display. Tips: "Correct answers unlock control," "Streaks unlock morph," "Wrong answers increase danger," "Stay calm under pressure." Fast, polished, energizing intro sequence.
- **User Role**: student
- **UI Details**: Animated countdown, rotating tip text.
- **Sub-features**: None

---

### 55
- **Feature Name**: MindStack In-Game Question Panel
- **Type**: component
- **Description**: Left panel components: A. Topic chip (Fractions, Grammar, etc.), B. Difficulty chip (Basic, Core, Challenge, Trap, Boss), C. Question text (clean, readable), D. Answer interface (MCQ buttons, input box, select order, true/false, match), E. Timer bar (visual countdown), F. Confidence toggle (sure/not sure, optional), G. Streak indicator, H. Reward preview (icons showing what success unlocks: move, rotate, morph, slow, multiplier).
- **User Role**: student
- **UI Details**: 8 specific panel components.
- **Sub-features**: 8 components

---

### 56
- **Feature Name**: MindStack Board Area Components
- **Type**: component
- **Description**: A. Falling block (active), B. Ghost shadow (projected landing preview), C. Locked stack (current board state), D. Danger zone marker (horizontal line near top), E. Next block preview (1-2 upcoming), F. Hold slot (locked until earned), G. Control state badge (Locked/Move Ready/Rotate Ready/Morph Ready/Panic Drop), H. Combo/line clear pulse.
- **User Role**: student
- **UI Details**: 8 board components with specific visual states.
- **Sub-features**: 8 components

---

### 57
- **Feature Name**: MindStack In-Game States (6 States)
- **Type**: interaction
- **Description**: 6 visible states: 1. Locked control (block spawns, question active), 2. Answer resolved (system determines outcome), 3. Unlocked control (movement/rotation enabled), 4. Enhanced control (morph/slowdown available), 5. Panic state (wrong answer increases gravity/danger), 6. Resolve state (block locks, rows clear, score updates).
- **User Role**: student
- **UI Details**: States made visually obvious through color, icons, and animation changes.
- **Sub-features**: 6 game states

---

### 58
- **Feature Name**: MindStack Second-by-Second Gameplay Loop (7 Steps)
- **Type**: flow
- **Description**: Step 1: Block spawns, question appears, control icons locked. Step 2: Question timer starts, block falls at restricted gravity. Step 3: Student answers. If correct: control icons activate, board sound stabilizes, student steers. If wrong: gravity increases, block flashes danger, control restricted. Step 4: Student places block with earned control. Step 5: Piece locks. Step 6: Lines clear (celebrate) or stack rises (tension). Step 7: Next question + next block immediately.
- **User Role**: student
- **UI Details**: Tight, non-sluggish loop with responsive audio/visual state transitions.
- **Sub-features**: 7 gameplay steps

---

### 59
- **Feature Name**: MindStack In-Game Question Skins (5 Types)
- **Type**: component
- **Description**: Type A: Quick Choice (4 options, fast, for recall/recognition), Type B: Binary Strike (True/False or Yes/No, for intensity bursts), Type C: Step Choice ("What comes next?" for math/science process), Type D: Trap Choice (similar-looking options, for misconception checking), Type E: Boss Choice (harder question, bigger reward).
- **User Role**: student
- **UI Details**: Different visual treatments per question skin type.
- **Sub-features**: 5 question skin types

---

### 60
- **Feature Name**: MindStack Feedback Design
- **Type**: animation
- **Description**: Correct: crisp confirmation sound, answer card flashes success, control icons light up, gravity relaxes. Wrong: heavier tone, answer card flashes red/danger, gravity audio intensifies, control rack stays locked, danger marker pulses. Streak (3, 5, 7): streak banner pulses, morph icon ignites, score multiplier appears. Line clear: satisfying but not distracting from learning side.
- **User Role**: student
- **UI Details**: 4 feedback categories with specific audio/visual effects.
- **Sub-features**: 4 feedback types

---

### 61
- **Feature Name**: MindStack Morph Interaction UX
- **Type**: interaction
- **Description**: Three morph UX options: Option 1 (tap cycle - press morph button to cycle through allowed forms, fast/simple), Option 2 (hold morph button - opens quick radial mini-selector of 2-3 shapes), Option 3 (auto-suggest morph - system highlights recommended shape, allows switching). Tap cycle recommended for first build.
- **User Role**: student
- **UI Details**: Morph button with 3 possible interaction patterns.
- **Sub-features**: 3 UX options

---

### 62
- **Feature Name**: MindStack Pause Screen
- **Type**: screen
- **Description**: Shows: current score, mode, topic, time survived/level reached, streak, current weaknesses triggered this round, boosters remaining. Buttons: Resume, Restart, Exit Run, Controls/Help. Optional tip: "Your slowest responses so far are on fraction comparison."
- **User Role**: student
- **UI Details**: Functional pause screen with intelligent tip.
- **Sub-features**: 7 info items, 4 buttons, optional tip

---

### 63
- **Feature Name**: MindStack Game Over / Round Complete Screen
- **Type**: screen
- **Description**: Section 1: Performance headline (e.g., "Sharp under pressure," "High panic loss in algebra," "Great comeback run"). Section 2: Main stats (score, level, lines cleared, best streak, pressure accuracy, avg answer speed, morphs earned, panic drops survived). Section 3: Learning breakdown (strongest topic, weakest topic, most common mistake pattern, speed vs confusion analysis, whether pressure reduced accuracy). Section 4: Improvement recommendation (e.g., "Train quick recall on decimals," "Enter Weakness Hunt for algebra signs"). Section 5: Buttons (Play Again, Repair Weakness, Review Missed Questions, Continue to Suggested Mode).
- **User Role**: student
- **UI Details**: 5-section results screen with specific metrics, insights, and actionable recommendations.
- **Sub-features**: 5 sections with specific data points

---

### 64
- **Feature Name**: MindStack Review Missed Questions Screen
- **Type**: screen
- **Description**: For every missed question, shows: question, student's answer, correct answer, short explanation, what gameplay effect it caused (e.g., "movement stayed locked and gravity increased"). Action buttons: try similar again, add to memory rescue, mark as weakness target.
- **User Role**: student
- **UI Details**: Question review cards with gameplay impact display and 3 action buttons.
- **Sub-features**: 5 displayed items per question, 3 action buttons

---

### 65
- **Feature Name**: MindStack Progression System
- **Type**: data-visualization
- **Description**: A. Rank progression (Trainee, Controller, Stack Tactician, Pressure Reader, Morph Adept, MindStack Master). B. Topic badges (Fraction Grip, Grammar Control, Science Stabilizer, Algebra Resolver). C. Skill tracks (pressure accuracy, response speed, recovery strength, control efficiency, panic resilience). D. Mode unlocks (Morph Mode after consistent control streaks, Elite Stack after strong performance).
- **User Role**: student
- **UI Details**: Rank badges, topic badges, skill track progress bars, mode unlock indicators.
- **Sub-features**: 4 progression layers (ranks, badges, skills, unlocks)

---

### 66
- **Feature Name**: MindStack Daily and Weekly Hooks
- **Type**: notification
- **Description**: Daily missions (survive 2 min without timeout, earn 5 morphs, clear 10 lines in grammar mode, beat pressure accuracy in fractions). Weekly challenge themes (Fraction Fortress Week, Science Speed Week, Grammar Under Pressure). Rewards: badge, booster, theme skin, leaderboard position.
- **User Role**: student
- **UI Details**: Daily mission cards, weekly challenge banners, reward display.
- **Sub-features**: Daily missions, weekly challenges, 4 reward types

---

### 67
- **Feature Name**: MindStack Theme and Skin System
- **Type**: setting
- **Description**: Board themes: Neon Logic, Math Grid, Science Lab, Library Blue, Exam Heat, Elite Gold. Earned or unlocked. Skins must never reduce readability.
- **User Role**: student
- **UI Details**: Theme selection interface, themed board visuals.
- **Sub-features**: 6 theme options

---

### 68
- **Feature Name**: MindStack Sound Design (4 Layers)
- **Type**: sound
- **Description**: Layer 1: ambient pressure (increases with board risk), Layer 2: answer events (correct/wrong/timeout), Layer 3: board events (drop, lock, line clear, combo, morph), Layer 4: streak/danger signals. Audio guides emotion: control regained, danger rising, mastery building.
- **User Role**: student
- **UI Details**: 4 audio layers with emotional guidance.
- **Sub-features**: 4 audio layers

---

### 69
- **Feature Name**: MindStack Accessibility and Usability
- **Type**: setting
- **Description**: Large text option, colorblind-safe control icons, audio cues with visual alternatives, slower beginner settings, reduced animation mode, left-handed and right-handed layouts.
- **User Role**: student
- **UI Details**: Accessibility settings menu.
- **Sub-features**: 6 accessibility features

---

### 70
- **Feature Name**: MindStack Teacher/Parent Interpretation Layer
- **Type**: report
- **Description**: Summary insights after several runs: "Student is accurate in calm conditions but loses control under speed," "Student struggles with multiplication fact recall, causing repeated timeouts," "Student has improved recovery after mistakes," "Student's grammar control is stable, but science recall weakens under pressure."
- **User Role**: teacher, parent
- **UI Details**: Summary insight cards with natural-language diagnostic descriptions.
- **Sub-features**: Performance diagnosis reports

---

### 71
- **Feature Name**: MindStack Per-Run Analytics Model
- **Type**: data-visualization
- **Description**: Per question tracking: topic, subtopic, difficulty, response correctness, response speed, confidence, control reward earned, gameplay outcome, block placement success, line clear result. Per run tracking: pressure accuracy, calm accuracy baseline, panic-loss frequency, recovery success rate, morph efficiency, collapse cause, board efficiency, weak-topic concentration.
- **User Role**: admin, teacher, parent
- **UI Details**: Analytics dashboard with per-question and per-run views.
- **Sub-features**: 10 per-question metrics, 8 per-run metrics

---

### 72
- **Feature Name**: MindStack Fragile Mastery Detection
- **Type**: data-visualization
- **Description**: If student historically correct in normal modes but repeatedly wrong/timeout in MindStack under pressure, mark concept as "fragile_under_pressure." One of MindStack's most important diagnostic outputs.
- **User Role**: student (data subject), teacher/parent (viewer)
- **UI Details**: Fragile mastery tag in diagnostic reports and weakness displays.
- **Sub-features**: None

---

### 73
- **Feature Name**: MindStack Pressure Accuracy Delta
- **Type**: data-visualization
- **Description**: Computes: pressure_accuracy_delta = calm accuracy baseline - pressure mode accuracy. Calculated per topic/subtopic. Example: fractions -22%, grammar -4%, ecosystems -15%. Identifies what breaks under stress.
- **User Role**: student (data subject), teacher/parent (viewer)
- **UI Details**: Delta display in post-run analysis and teacher reports.
- **Sub-features**: None

---

### 74
- **Feature Name**: MindStack Recovery Strength Metric
- **Type**: data-visualization
- **Description**: Measures comeback ability: how often player stabilizes after wrong answer, reduces danger within N cycles, recovers from top-zone stack states, answers correctly after panic streak. Produces a "Recovery Strength Score."
- **User Role**: student, parent
- **UI Details**: Recovery Strength Score displayed in progression and reports.
- **Sub-features**: 4 recovery measurement dimensions

---

### 75
- **Feature Name**: MindStack Data Persistence Model (3 Levels)
- **Type**: engine-output-needing-UI
- **Description**: A. Run record (run_id, user_id, mode, topic scope, difficulty, started/ended_at, final_score, lines_cleared, waves_completed, topout_reason, answer_accuracy, timeout_rate, best_streak, panic_count, recovery_count, mastery_summary). B. Cycle record (cycle_id, run_id, question_id, block_id, answer_tier, latency_ms, permissions_earned, placement_quality, danger_before/after, line_clear_result, collapse_contribution_flag). C. Aggregated mastery stats (per user per topic: total_attempts, pressure_attempts, pressure_correct, timeout_count, fragile_mastery_score, misconception_score, recovery_success, best_pressure_performance).
- **User Role**: admin
- **UI Details**: Powers analytics dashboards and reports.
- **Sub-features**: 3 persistence levels

---

### 76
- **Feature Name**: MindStack Recommendation Engine
- **Type**: engine-output-needing-UI
- **Description**: Post-run machine-derived recommendations: review fraction comparison, enter Weakness Hunt for algebra signs, switch to Memory Rescue for cell transport, move from Focused to Fast difficulty, identify biggest issue is timeout vs misunderstanding.
- **User Role**: student
- **UI Details**: Recommendation cards on results screen and home screen "Suggested next mission."
- **Sub-features**: None

---

### 77
- **Feature Name**: MindStack Backend Service Contracts
- **Type**: admin-tool
- **Description**: Modular service interfaces: Run service (create/start/pause/resume/end_run), Question service (get_next_question, validate_response), Control service (resolve_permissions), Board service (spawn_block, apply_input, tick_board, lock_block, resolve_lines), Difficulty service (update_difficulty), Recovery service (evaluate_mercy_trigger, apply_recovery_adjustment), Analytics service (record_cycle, finalize_run, update_mastery).
- **User Role**: admin (developer)
- **UI Details**: No direct UI; backend API contracts.
- **Sub-features**: 7 service interfaces

---

### 78
- **Feature Name**: MindStack Balancing Knobs (Config-Driven)
- **Type**: setting
- **Description**: Tuning knobs: gravity per difficulty band, timer per question band, control reward table, morph frequency, mercy trigger thresholds, trap question frequency, streak thresholds, score weights, danger level thresholds, recovery reward sizes. All config-driven.
- **User Role**: admin
- **UI Details**: Admin configuration tool.
- **Sub-features**: 10 tuning knobs

---

### 79
- **Feature Name**: MindStack Anti-Exploit Rules
- **Type**: setting
- **Description**: A. Guess spam prevention (no unlimited input changes on short timers), B. Intentional sandbagging detection, C. Repetition farming prevention, D. Morph abuse limits (max morphs per cycle, log efficiency), E. Pause abuse prevention (freeze limitations in competitive modes).
- **User Role**: admin
- **UI Details**: No direct UI; backend enforcement.
- **Sub-features**: 5 anti-exploit rules

---

### 80
- **Feature Name**: MindStack Competitive/Leaderboard Readiness
- **Type**: data-visualization
- **Description**: Deterministic run event logs for verification: question IDs served, answer timestamps, permission grants, board events in order. Supports integrity and replay for leaderboards.
- **User Role**: student (leaderboard), admin (integrity)
- **UI Details**: Leaderboard UI (future).
- **Sub-features**: Event log, replay capability

---

### 81
- **Feature Name**: MindStack Replay / Debug Mode
- **Type**: admin-tool
- **Description**: Stores event stream, question sequence, player inputs, timing data. Enables replay to inspect: why collapse happened, whether reward tables felt fair, whether a question arrived at a bad time, whether student panicked or was trapped by board.
- **User Role**: admin
- **UI Details**: Replay viewer for debugging and balancing.
- **Sub-features**: Event stream storage, replay viewer

---

### 82
- **Feature Name**: MindStack Loss Conditions
- **Type**: interaction
- **Description**: 5 run-end conditions: A. Top-out (stack reaches terminal threshold), B. Hard collapse (optional for elite modes), C. Timed run complete, D. Goal complete (mission modes), E. Manual exit. Top-out is default.
- **User Role**: student
- **UI Details**: Loss animation/display.
- **Sub-features**: 5 loss condition types

---

### 83
- **Feature Name**: MindStack Mode-Specific Win Conditions
- **Type**: interaction
- **Description**: Classic Control (target score/time/waves), Weakness Hunt (survive while completing N weakness-tagged questions), Memory Rescue (stabilize N decaying concepts), Exam Pressure (survive fixed stress interval), Elite Stack (clear milestone waves with no mercy triggers).
- **User Role**: student
- **UI Details**: Win condition display per mode.
- **Sub-features**: 5 mode-specific win conditions

---

### 84
- **Feature Name**: MindStack Backend Game Engine Spec
- **Type**: engine-output-needing-UI
- **Description**: Full runtime architecture: 10 sub-engines (Run Orchestrator, Board Engine, Block Engine, Question Engine, Answer Evaluation Engine, Control Permission Engine, Scoring Engine, Difficulty Director, Recovery/Mercy Engine, Analytics+Mastery Engine). MindStackRunState canonical live session state with session identity, progress, board snapshot, question cycle snapshot, control state, difficulty state, learning state.
- **User Role**: admin (developer)
- **UI Details**: No direct UI; backend runtime architecture.
- **Sub-features**: 10 sub-engines, runtime state object

---

### 85
- **Feature Name**: MindStack Run Lifecycle States (7 States)
- **Type**: flow
- **Description**: A. initialized, B. countdown, C. active, D. paused, E. resolving, F. completed, G. abandoned.
- **User Role**: student
- **UI Details**: State transitions visible through UI changes (countdown animation, pause overlay, results screen).
- **Sub-features**: 7 lifecycle states

---

### 86
- **Feature Name**: MindStack Per-Block Cycle State Machine (9 States)
- **Type**: flow
- **Description**: QuestionBlockCycleState: 1. spawn_pending, 2. spawned_locked, 3. answer_window_open, 4. answer_resolved, 5. permissions_applied, 6. placement_window, 7. block_locked, 8. board_resolved, 9. cycle_complete.
- **User Role**: student (experiences), admin (debugs)
- **UI Details**: Visible state transitions through control icon changes, gravity changes, and board animations.
- **Sub-features**: 9 cycle states

---

### 87
- **Feature Name**: MindStack Tick Model / Real-Time Runtime
- **Type**: engine-output-needing-UI
- **Description**: Deterministic tick/update loop with 3 tick concerns: Board tick (gravity, movement, collisions, lock delay, line clear animation timing), Question timer tick (answer countdown, timeout, question availability), Run tick (pressure evolution, difficulty director checks, mercy engine checks, event emission).
- **User Role**: admin (developer)
- **UI Details**: No direct UI; controls game feel and timing.
- **Sub-features**: 3 tick types

---

### 88
- **Feature Name**: MindStack Event-Driven Architecture
- **Type**: engine-output-needing-UI
- **Description**: Every meaningful game event emits an event. Core event categories: Block events (spawned, moved, rotated, morphed, soft_dropped, locked), Question events (served, answer_submitted, answer_timed_out, answer_graded), Permission events (movement/rotation/morph/slowdown unlocked, control_restricted), Board events (line_cleared, combo_started/broken, danger_entered/exited, topout_risk), Recovery events (mercy_triggered, rescue_question_served, panic_recovery_success/failed), Run events (started, paused, resumed, ended), Learning events (misconception_detected, fragile_mastery_detected, timeout_pattern_detected, pressure_failure_logged, mastery_gain_logged).
- **User Role**: admin
- **UI Details**: Event stream powers analytics, replay, debugging, and balancing.
- **Sub-features**: 7 event categories with 25+ specific events

---

### 89
- **Feature Name**: MindStack Board Engine Spec
- **Type**: engine-output-needing-UI
- **Description**: Manages: grid dimensions, position validation, gravity movement, collision detection, block locking, full row clearing, row shifting, danger metrics, top-out/loss detection. Board state fields: width, height, settled_cells, active_piece_cells, active_piece_position, active_piece_rotation, pending_line_clear_rows, lock_delay_remaining. Continuous metrics: highest_occupied_row, stack_height, open_holes, surface_roughness, near-top pressure, line_clear_potential.
- **User Role**: admin (developer)
- **UI Details**: Visual board display.
- **Sub-features**: 8 state fields, 6 continuous metrics

---

### 90
- **Feature Name**: MindStack Block Engine Spec
- **Type**: engine-output-needing-UI
- **Description**: Active block fields: block_id, base_shape_type, current_shape_type, orientation, x, y, spawn_time, gravity_rate, lock_state, allowed_permissions, allowed_morph_targets, morphs_used_this_cycle. Distinguishes between base shape, current shape after morph, and allowed morph set.
- **User Role**: admin (developer)
- **UI Details**: Block visual rendering.
- **Sub-features**: 12 block fields

---

### 91
- **Feature Name**: MindStack Shape Morph Rules (6 Rules)
- **Type**: setting
- **Description**: Rule 1: Only morph if morph_unlocked is true. Rule 2: Max N morphs per cycle (default N=1). Rule 3: Board must allow morphed shape (no overlaps). Rule 4: Morph choices can be fixed menu/difficulty-based/reward-tier-based/board-aware. Rule 5: Preserve current anchor position. Rule 6: Log morph usage for analytics.
- **User Role**: admin
- **UI Details**: No direct UI; constrains morph behavior.
- **Sub-features**: 6 morph rules

---

### 92
- **Feature Name**: MindStack Question Selection Engine
- **Type**: engine-output-needing-UI
- **Description**: Inputs: user's topic scope, active mode, current difficulty, pressure state, recent question history, weakness map, memory-risk map, misconception map, board danger state, fatigue/repetition guards. Outputs: QuestionCandidate with question_id, topic, subtopic, type, difficulty, expected_response_time, misconception_tags, control_reward_profile, remediation_value, pressure_suitability. Mode-specific policies for Classic Control, Weakness Hunt, Memory Rescue, Survival, Exam Pressure, Elite Stack.
- **User Role**: admin (developer)
- **UI Details**: No direct UI; powers question delivery.
- **Sub-features**: 10 input dimensions, 10 output fields, 6 mode policies

---

### 93
- **Feature Name**: MindStack Question Eligibility Filters (7 Filters)
- **Type**: engine-output-needing-UI
- **Description**: Before serving, questions pass: A. Content fit, B. Length fit (answerable within current pace), C. Repetition guard, D. Difficulty fit, E. Pressure fit (not too text-heavy for high-speed), F. Diagnostic fit, G. Fairness fit (avoid too many traps back-to-back).
- **User Role**: admin
- **UI Details**: No direct UI; quality control.
- **Sub-features**: 7 filter categories

---

### 94
- **Feature Name**: MindStack Answer Evaluation Engine
- **Type**: engine-output-needing-UI
- **Description**: Produces AnswerQualityResult with: correctness (correct/incorrect/partial/timeout), latency_ms, speed_band (fast/normal/slow/timeout), confidence_match, misconception_triggered, response_quality_score, control_reward_tier. 6 answer quality tiers: Tier 0 (timeout), Tier 1 (wrong), Tier 2 (partial), Tier 3 (correct), Tier 4 (fast correct), Tier 5 (mastery correct).
- **User Role**: student (experiences), admin (configures)
- **UI Details**: Tier-based visual feedback (e.g., control icons light up differently per tier).
- **Sub-features**: 7 output fields, 6 quality tiers

---

### 95
- **Feature Name**: MindStack Control Permission Engine
- **Type**: engine-output-needing-UI
- **Description**: Converts answer quality into gameplay power. Inputs: answer_quality_tier, mode, difficulty, active streak, board danger state, mercy flag, question reward profile. Outputs: movement_unlocked, rotation_unlocked, morph_unlocked, slowdown_active, hold_unlocked, rescue_active, emergency_shift_count. Table-driven with configurable reward matrix: (mode, difficulty, answer_tier, streak_band, danger_band) -> permission bundle.
- **User Role**: admin (configuration)
- **UI Details**: Control rack icon states reflect permission bundles.
- **Sub-features**: 7 input dimensions, 7 output permissions

---

### 96
- **Feature Name**: MindStack Difficulty Director
- **Type**: engine-output-needing-UI
- **Description**: Continuously adjusts challenge based on elapsed progression and student stability. Inputs: time survived, waves completed, recent answer accuracy, recent timeout rate, board danger index, recovery performance, mode rules. Outputs: gravity band changes, timer band changes, question difficulty band changes, trap frequency changes, mercy trigger thresholds. Controls 5 dimensions independently: gravity, question time budget, question cognitive complexity, punishment severity, recovery generosity.
- **User Role**: admin
- **UI Details**: Smooth difficulty transitions during gameplay.
- **Sub-features**: 7 inputs, 5 outputs, 5 difficulty dimensions

---

### 97
- **Feature Name**: MindStack Recovery/Mercy Engine
- **Type**: engine-output-needing-UI
- **Description**: Monitors for death spirals: repeated timeouts, repeated wrong answers, board in danger, low control streak, rapidly rising stack, poor recovery success. Mercy responses: slightly easier question, longer timer, emergency move granted, reduced gravity for one cycle, rescue morph chance, recovery question with higher control reward. Preserves productive pressure without cruelty.
- **User Role**: student (experiences)
- **UI Details**: Subtle mercy indicators (slightly easier questions, rescue opportunities).
- **Sub-features**: 6 spiral indicators, 6 mercy responses

---

### 98
- **Feature Name**: MindStack Implementation Build Phases (4 Phases)
- **Type**: admin-tool
- **Description**: Phase 1: Core playable loop (Board Engine, Run Orchestrator, simple Question/Answer/Control/Scoring engines, basic persistence). Phase 2: Learning intelligence (weakness targeting, timeout analysis, pressure accuracy delta, fragile mastery detection, cycle analytics). Phase 3: Premium systems (morph system, mercy engine, dynamic difficulty director, recommendation engine, mode policies). Phase 4: Advanced depth (replay, leaderboards, teacher/parent reports, boss rounds, confidence-aware evaluation).
- **User Role**: admin (developer)
- **UI Details**: No direct UI; development roadmap.
- **Sub-features**: 4 build phases

---

### 99
- **Feature Name**: MindStack Minimal First Schema Entities
- **Type**: engine-output-needing-UI
- **Description**: Core entities: UserMindStackProfile, MindStackRun, MindStackCycle, MindStackQuestionPolicy, MindStackDifficultyProfile, MindStackPermissionBundle, MindStackAnalyticsAggregate, MindStackRecommendation.
- **User Role**: admin (developer)
- **UI Details**: No direct UI; data model.
- **Sub-features**: 8 schema entities

---

### 100
- **Feature Name**: MindStack Academic Diagnostic Power
- **Type**: data-visualization
- **Description**: Distinguishes: does not know, knows but slow, knows but panics, confuses similar concepts, recovers well after failure, collapses after one mistake, uses earned control efficiently, wastes opportunities. Far beyond normal quiz analytics.
- **User Role**: teacher, parent
- **UI Details**: Diagnostic insight display with 8 student performance categories.
- **Sub-features**: 8 diagnostic categories

---

### 101
- **Feature Name**: MindStack Product Statement
- **Type**: component
- **Description**: "MindStack is a pressure-learning game where every answer shapes your control. Know it, and you command the board. Miss it, and chaos takes over."
- **User Role**: all
- **UI Details**: Marketing/in-app description text.
- **Sub-features**: None

---

### 102
- **Feature Name**: Tug of War Best Concepts (4 Recommended)
- **Type**: mode
- **Description**: Four strongest Tug of War ideas: 1. Misconception Tug of War (battle exact weakness), 2. Mental Math Tug Arena (fast, intense, streak power), 3. Class Team Tug of War (teacher-led, school excitement), 4. Topic Boss Tug (every topic has a boss like Algebra Beast or Grammar Storm).
- **User Role**: student, teacher
- **UI Details**: Each as a distinct selectable mode with boss characters and team interfaces.
- **Sub-features**: 4 recommended Tug of War variants

---

### 103
- **Feature Name**: Best Tetris Concepts (4 Recommended)
- **Type**: mode
- **Description**: Four strongest Tetris ideas: 1. Equation Tetris (strong for math, replayable), 2. Grammar Grid Tetris (easy to understand, strong for language), 3. Sequence Tetris (excellent for science/history/procedural learning), 4. Fraction/Geometry Tetris (cleanest educational adaptation).
- **User Role**: student
- **UI Details**: Each as a distinct selectable game mode.
- **Sub-features**: 4 recommended Tetris variants

---

### 104
- **Feature Name**: Emotional Learning Loop
- **Type**: interaction
- **Description**: Students should feel: "I almost lost, but I recovered," "I held the rope," "I built that correctly under pressure," "I am getting sharper." Tug of War trains: retrieval speed, pressure tolerance, response confidence, resistance to mental freezing, mastery under stress. Tetris trains: pattern recognition, structural thinking, categorization, planning ahead, compositional intelligence.
- **User Role**: student
- **UI Details**: Emotional states conveyed through audio, visual feedback, and post-game summaries.
- **Sub-features**: 5 Tug of War cognitive muscles, 5 Tetris cognitive muscles

---

Now I have read both files completely. Let me compile the exhaustive feature extraction.

---

## FEATURES FOUND IN idea11.txt

### 1
- **Feature Name**: Wrong Answer Intelligence System
- **Type**: engine-output-needing-UI
- **Description**: When a student gets a question wrong, the system performs 6 jobs simultaneously: diagnose the wrong answer, explain the correct answer deeply, explain why every other option is wrong, reconstruct the student's thinking, map the question to question families, and update the student's error trend.
- **User Role**: student
- **UI Details**: Should present layered explanation rather than a simple "correct answer is X" display.
- **Sub-features**: Distractor analysis, reasoning reconstruction, answer-option elimination explanation, misconception detection, question family mapping, pattern comparison across history, student-specific diagnosis, targeted recovery action.

### 2
- **Feature Name**: Inbuilt Academic Analyst
- **Type**: engine-output-needing-UI
- **Description**: A hidden intelligence layer that continuously studies the student, answering why they get questions wrong, which question families hurt them, which distractor types trap them, which cognitive conditions weaken them, and whether they are improving or repeating loops.
- **User Role**: student
- **UI Details**: Dashboard with analyst outputs, readable intelligence narratives, not just raw numbers.
- **Sub-features**: 7 live profiles (Concept Weakness, Misconception, Reasoning, Distractor Vulnerability, Pressure, Transfer, Recovery), trend analysis, pattern detection.

### 3
- **Feature Name**: Wrong Answer Diagnosis Engine
- **Type**: engine-output-needing-UI
- **Description**: Determines for each wrong answer: what concept was missed, what reasoning step broke, whether it was recall failure/interpretation failure/logic failure/speed failure/confusion, and whether the student knew the concept but applied it wrongly.
- **User Role**: student
- **UI Details**: Results displayed as part of the Wrong Answer Review Card.
- **Sub-features**: Concept diagnosis, reasoning step analysis, failure type classification (recall, interpretation, logic, speed, confusion).

### 4
- **Feature Name**: Deep Correct Answer Explanation
- **Type**: component
- **Description**: Explains not just that an answer is correct but why it is right, what evidence supports it, what rule/principle/formula led to it, and what the student should have noticed.
- **User Role**: student
- **UI Details**: Part of the layered explanation flow after a wrong answer.
- **Sub-features**: Evidence presentation, rule/principle/formula linkage, "what you should have noticed" prompt.

### 5
- **Feature Name**: Option-by-Option Elimination Explanation
- **Type**: component
- **Description**: For each wrong option, explains why it is tempting, why it is misleading, why it looks possible but fails, and what exact condition makes it false.
- **User Role**: student
- **UI Details**: Each option can be tapped to see its "autopsy." Structured display showing temptation reason and failure reason per option.
- **Sub-features**: Option autopsy view, distractor temptation explanation, condition-based falsification explanation.

### 6
- **Feature Name**: Student Thinking Reconstruction
- **Type**: engine-output-needing-UI
- **Description**: The engine infers the student's likely thought process: what they focused on, what they confused, whether they rushed, whether they remembered a rule partly but missed an exception.
- **User Role**: student
- **UI Details**: Presented as natural language narrative, e.g., "You probably focused on this keyword..."
- **Sub-features**: Keyword focus detection, term confusion detection, rush detection, partial rule memory detection.

### 7
- **Feature Name**: Question Family Mapping
- **Type**: engine-output-needing-UI
- **Description**: Maps every question to question families (concept family, reasoning family, trap family, misconception family). Clusters questions that share similar concept structure, reasoning demand, trap pattern, solving pathway, and distractor logic.
- **User Role**: student
- **UI Details**: System surfaces family-level insights, e.g., "You repeatedly struggle with questions that require comparing closely related ideas under time pressure."
- **Sub-features**: Math families (choose-the-right-operation, order-of-operations trap, formula selection, unit conversion, proportional reasoning, graph interpretation, equation translation, geometry property discrimination), Science families (variable identification, cause-effect reasoning, process sequence, force-motion distinction, experiment interpretation, classification, diagram label, scientific principle application), Language families (grammar rule application, meaning-in-context, sentence function, inference from passage, tone/attitude, pronoun reference), Social Studies families (chronology, cause-consequence, source interpretation, concept distinction, policy effect).

### 8
- **Feature Name**: Error Trend Tracking / Error Graph
- **Type**: data-visualization
- **Description**: Every wrong answer updates a long-term error graph. Updates the student's misconception profile, family weakness map, reasoning weakness map, pressure weakness map, and improvement trend.
- **User Role**: student
- **UI Details**: Visual error graph showing trends over time, profile dashboards.
- **Sub-features**: Misconception profile update, family weakness map update, reasoning weakness map update, pressure weakness map update, improvement trend tracking.

### 9
- **Feature Name**: Emotional Handling Layer (Layer 1)
- **Type**: interaction
- **Description**: When a student gets a question wrong, the system starts with empathetic messaging rather than cold correction, e.g., "You were close, but one key idea slipped." Reduces frustration.
- **User Role**: student
- **UI Details**: Warm, empathetic tone in the opening of the wrong answer review.
- **Sub-features**: None.

### 10
- **Feature Name**: "Why Your Answer Felt Right" Explanation (Layer 2)
- **Type**: component
- **Description**: The system validates the student's likely logic before correcting, e.g., "You chose B probably because the sentence mentions speed, and B looks like the most direct speed-related formula."
- **User Role**: student
- **UI Details**: Presented as a narrative explanation in natural language.
- **Sub-features**: None.

### 11
- **Feature Name**: "Why Your Answer Is Wrong" Explanation (Layer 3)
- **Type**: component
- **Description**: Shows not just that the answer is wrong but exactly why, connecting the failure to a specific conceptual or logical gap.
- **User Role**: student
- **UI Details**: Clear, specific explanation text.
- **Sub-features**: None.

### 12
- **Feature Name**: "Why the Correct Answer Wins" Explanation (Layer 4)
- **Type**: component
- **Description**: Shows the decisive idea that makes the correct answer right, with specific reasoning.
- **User Role**: student
- **UI Details**: Focused on the key discriminator.
- **Sub-features**: None.

### 13
- **Feature Name**: "Why Other Options Fail" Explanation (Layer 5)
- **Type**: component
- **Description**: Breaks down each wrong option individually with why it fails, building discrimination skill.
- **User Role**: student
- **UI Details**: Per-option breakdown.
- **Sub-features**: None.

### 14
- **Feature Name**: "Your Likely Thinking Trap" Diagnosis (Layer 6)
- **Type**: component
- **Description**: Diagnoses the type of cognitive trap the student fell into, e.g., "You keep choosing answers that match a familiar keyword instead of checking what the question is actually measuring."
- **User Role**: student
- **UI Details**: Presented as an insight or diagnosis statement.
- **Sub-features**: None.

### 15
- **Feature Name**: Immediate Repair Actions (Layer 7)
- **Type**: interaction
- **Description**: After diagnosis, provides a quick fix: one mini follow-up question, one contrast example, one micro drill, and one memory hook.
- **User Role**: student
- **UI Details**: Interactive repair activities immediately following review.
- **Sub-features**: Mini follow-up question, contrast example, micro drill, memory hook.

### 16
- **Feature Name**: Wrong Answer Review Card
- **Type**: component
- **Description**: A standardized card format for every wrong answer review with 10 sections: what you chose, why it looked reasonable, why it fails, why correct is correct, why others are wrong, what kind of mistake this is, what your brain likely did, the lesson, your pattern history, and repair action.
- **User Role**: student
- **UI Details**: Structured card with 10 labeled sections.
- **Sub-features**: 10 sections as listed.

### 17
- **Feature Name**: Response Time Analysis
- **Type**: engine-output-needing-UI
- **Description**: Analyzes student response time as a diagnostic signal: too fast = guess/pattern matching/careless recognition; too slow = uncertainty/overload/weak retrieval; medium but wrong = stable misconception.
- **User Role**: student
- **UI Details**: Used internally to inform diagnosis, surfaced in pattern reports.
- **Sub-features**: Speed-based classification (too fast, too slow, medium).

### 18
- **Feature Name**: Answer Change Detection
- **Type**: engine-output-needing-UI
- **Description**: Tracks if the student changed their answer. Changing from correct to wrong signals self-doubt/distractor seduction/confidence collapse. Changing from wrong to another wrong signals concept fog/broad uncertainty.
- **User Role**: student
- **UI Details**: Used as a diagnostic signal, tracked in error fingerprint.
- **Sub-features**: Correct-to-wrong detection, wrong-to-wrong detection.

### 19
- **Feature Name**: Confidence Rating Capture
- **Type**: interaction
- **Description**: Captures student confidence level before or after answering. High confidence + wrong = strong misconception/false fluency. Low confidence + wrong = weak mastery/uncertainty.
- **User Role**: student
- **UI Details**: Confidence input mechanism on questions.
- **Sub-features**: Confidence-misconception correlation analysis.

### 20
- **Feature Name**: Step-Based Error Tracking
- **Type**: engine-output-needing-UI
- **Description**: For step-based questions, captures where the student diverged, the first wrong move, and whether the error was conceptual or procedural.
- **User Role**: student
- **UI Details**: Step trail capture and display.
- **Sub-features**: Divergence point detection, first wrong step identification, conceptual vs procedural classification.

### 21
- **Feature Name**: Error Type Classification System
- **Type**: engine-output-needing-UI
- **Description**: Classifies wrong answers into 12 core error classes: Recall failure, Recognition trap, Concept confusion, Application failure, Multi-step breakdown, Reading failure, Distractor seduction, Careless execution, Pressure collapse, Fragile mastery, Prerequisite gap, False confidence.
- **User Role**: student
- **UI Details**: Displayed in plain language to students, e.g., "This was a confusion mistake," "This was a rushed recognition mistake."
- **Sub-features**: 12 error classes with specific detection criteria.

### 22
- **Feature Name**: Error Fingerprint System
- **Type**: engine-output-needing-UI
- **Description**: For every wrong answer, generates a structured Error Fingerprint containing: subject, topic, subtopic, family, reasoning type, chosen distractor type, misconception, timing pattern, confidence, severity, first wrong step, likely cause.
- **User Role**: student
- **UI Details**: Fingerprints compared across attempts to detect clusters and patterns.
- **Sub-features**: Fingerprint generation, fingerprint comparison, cluster detection.

### 23
- **Feature Name**: Instant Repair Intervention
- **Type**: interaction
- **Description**: Gives 1-3 mini questions on the exact weakness immediately after diagnosis.
- **User Role**: student
- **UI Details**: Quick interactive questions inline.
- **Sub-features**: None.

### 24
- **Feature Name**: Contrast Repair Intervention
- **Type**: interaction
- **Description**: Shows two similar questions side by side: one where the student's mistaken logic works and one where it fails.
- **User Role**: student
- **UI Details**: Side-by-side question comparison display.
- **Sub-features**: None.

### 25
- **Feature Name**: Step Repair Intervention
- **Type**: interaction
- **Description**: Breaks the reasoning into steps and trains only the broken step.
- **User Role**: student
- **UI Details**: Step-by-step guided practice focusing on the specific broken step.
- **Sub-features**: None.

### 26
- **Feature Name**: Misconception Repair Intervention
- **Type**: interaction
- **Description**: Directly attacks the false idea with targeted examples.
- **User Role**: student
- **UI Details**: Targeted example questions designed to correct specific misconceptions.
- **Sub-features**: None.

### 27
- **Feature Name**: Family Repair Intervention
- **Type**: interaction
- **Description**: Gives fresh questions from the same question family with increasing support.
- **User Role**: student
- **UI Details**: Scaffolded question sequence within a family.
- **Sub-features**: None.

### 28
- **Feature Name**: Prerequisite Repair Intervention
- **Type**: interaction
- **Description**: Goes backward and strengthens the missing foundation when a prerequisite gap is detected.
- **User Role**: student
- **UI Details**: Directed to foundational content before returning to current topic.
- **Sub-features**: None.

### 29
- **Feature Name**: Pressure Repair Intervention
- **Type**: interaction
- **Description**: If the issue is timing, retrains slowly first, then reintroduces speed.
- **User Role**: student
- **UI Details**: Graduated timer reintroduction.
- **Sub-features**: None.

### 30
- **Feature Name**: Reflection Repair Intervention
- **Type**: interaction
- **Description**: Asks the student "What made B attractive to you?" then compares student self-report with system inference to improve metacognition.
- **User Role**: student
- **UI Details**: Self-report prompt with system comparison.
- **Sub-features**: None.

### 31
- **Feature Name**: Mistake Type Classification UI
- **Type**: component
- **Description**: Displays the mistake classification in plain language to the student: "confusion mistake," "rushed recognition mistake," "wrong-step mistake," "shallow-memory mistake," "wording trap mistake," "near-miss."
- **User Role**: student
- **UI Details**: Label displayed on the wrong answer review.
- **Sub-features**: None.

### 32
- **Feature Name**: Pattern Observation Narratives
- **Type**: engine-output-needing-UI
- **Description**: The system produces human-readable observations like: "You often choose answers that contain familiar terms, even when the underlying condition is different," "Your accuracy drops sharply when questions include exception words."
- **User Role**: student
- **UI Details**: Natural language narrative insights surfaced periodically.
- **Sub-features**: None.

### 33
- **Feature Name**: Three Levels of Wrongness Detection
- **Type**: engine-output-needing-UI
- **Description**: Separates Surface wrongness (final answer wrong), Process wrongness (wrong path used), and Pattern wrongness (wrong path is part of a recurring trend).
- **User Role**: student
- **UI Details**: Diagnosis distinguishes these three levels.
- **Sub-features**: Surface level, Process level, Pattern level.

### 34
- **Feature Name**: Why Panel
- **Type**: component
- **Description**: A panel that opens for each wrong answer containing 7 sections: Why your answer felt right, Why it is wrong, Why the correct answer wins, Why the other options fail, What mistake type this is, What this says about your pattern, What you should do next.
- **User Role**: student
- **UI Details**: 7-section expandable panel, described as a "core differentiator."
- **Sub-features**: 7 sections.

### 35
- **Feature Name**: Academic Analyst Dashboard
- **Type**: data-visualization
- **Description**: A live analytics dashboard for each student showing: top recurring weakness families, top misconceptions, error source breakdown (percentages), trigger conditions, improvement markers, and recommended next actions.
- **User Role**: student
- **UI Details**: Dashboard with percentage breakdowns, lists, and actionable recommendations.
- **Sub-features**: Weakness family ranking, misconception ranking, error source pie/bar chart, trigger condition list, improvement markers (recovery speed, relapse rate, family mastery trend, misconception extinction rate), recommended actions list.

### 36
- **Feature Name**: Rich Wrong-Answer Memory Storage
- **Type**: engine-output-needing-UI
- **Description**: Stores each wrong answer with: question fingerprint, chosen option, likely misconception, reasoning family, distractor type, response time, confidence, first wrong step, recovery success, recurrence links, similarity links to past errors.
- **User Role**: student
- **UI Details**: Backend storage powering all diagnostic features.
- **Sub-features**: 11 data fields per wrong answer.

### 37
- **Feature Name**: Question Intelligence Metadata
- **Type**: engine-output-needing-UI
- **Description**: Every question carries hidden metadata: Content tags (subject, topic, subtopic, skill, subskill, curriculum objective, difficulty level), Reasoning tags (recall, recognition, comparison, inference, elimination, multi-step reasoning, calculation, interpretation, application, evaluation, synthesis), Question family tags, Misconception tags, Distractor intent tags, Correct reasoning path, Prerequisite knowledge links.
- **User Role**: admin
- **UI Details**: Authoring interface for tagging questions.
- **Sub-features**: 7 metadata categories (A through G).

### 38
- **Feature Name**: Distractor Intent Tags
- **Type**: engine-output-needing-UI
- **Description**: Every wrong option is authored with intent: why it is tempting, what misconception it targets, what type of student would likely pick it, what reasoning error leads to it.
- **User Role**: admin
- **UI Details**: Tagging interface for distractor authoring.
- **Sub-features**: Temptation reason, targeted misconception, student type, reasoning error.

### 39
- **Feature Name**: Temptation Map
- **Type**: engine-output-needing-UI
- **Description**: Each wrong option has a temptation profile showing what type of student thinking it attracts, e.g., "option B attracts students who focus on keyword similarity."
- **User Role**: student
- **UI Details**: Accessible from wrong answer review, tells student "You were not randomly wrong. You were pulled by a specific trap."
- **Sub-features**: Per-option temptation profiles.

### 40
- **Feature Name**: Thought Replay
- **Type**: engine-output-needing-UI
- **Description**: After a wrong answer, replays the likely thought path: what the student probably noticed first, what assumption they made, where reasoning went off track, what they failed to verify.
- **User Role**: student
- **UI Details**: Narrative walkthrough of inferred thinking process.
- **Sub-features**: Notice detection, assumption inference, off-track point, verification failure.

### 41
- **Feature Name**: Mistake DNA
- **Type**: engine-output-needing-UI
- **Description**: Every wrong answer produces a structured fingerprint (topic, family, error type, distractor type, pressure state, confidence, prerequisite gap, correction difficulty). Over time, clusters into the student's Mistake DNA pattern.
- **User Role**: student
- **UI Details**: Cross-topic pattern display showing deeper root causes.
- **Sub-features**: Fingerprint fields, clustering algorithm, cross-topic pattern detection.

### 42
- **Feature Name**: Error Families (Cross-Subject)
- **Type**: engine-output-needing-UI
- **Description**: Tracks error families across subjects to detect the same thinking weakness appearing in different domains, e.g., "premature recognition" appearing in math, science, and English.
- **User Role**: student
- **UI Details**: Cross-subject error pattern display.
- **Sub-features**: Cross-subject error correlation, root problem identification.

### 43
- **Feature Name**: Mastery State Classification
- **Type**: engine-output-needing-UI
- **Description**: Classifies concept mastery into states: unknown, fragile, partial, surface-level, transferable, pressure-proof. Separates "not knowing" from "unstable knowing."
- **User Role**: student
- **UI Details**: Per-concept mastery state indicators.
- **Sub-features**: 6 mastery states.

### 44
- **Feature Name**: "Why Did You Pick It" Self-Report Check
- **Type**: interaction
- **Description**: Occasionally asks students after a wrong answer: "Why did you choose this?" with options like "I guessed," "It looked most familiar," "I used a formula," "I was between B and C," "I thought this keyword meant..." Compares self-report with engine inference and trend history.
- **User Role**: student
- **UI Details**: Multiple-choice or free-text self-report prompt appearing occasionally.
- **Sub-features**: Self-report capture, inference vs self-report comparison.

### 45
- **Feature Name**: Trap Vulnerability Profile
- **Type**: data-visualization
- **Description**: A hidden profile for each student tracking vulnerability to specific trap types: keyword traps, half-true options, overly broad options, extreme answers, reversed relationships, unit errors, lookalike formulas, exception-word misses. Each with a High/Medium/Low rating.
- **User Role**: student
- **UI Details**: Profile used to personalize feedback depth.
- **Sub-features**: Per-trap-type vulnerability rating.

### 46
- **Feature Name**: Recovery Mode (Frustration Detection)
- **Type**: mode
- **Description**: When the student gets several wrong in a row, the system detects frustration and switches to Recovery Mode: reduces pressure, uses warmer tone, shrinks problem size, shows one contrast clearly, gives a quick win, rebuilds confidence before returning to harder work.
- **User Role**: student
- **UI Details**: Mode switch with softer UI tone, smaller problems, warm encouragement.
- **Sub-features**: Frustration detection, pressure reduction, warm tone, quick win delivery, confidence rebuild.

### 47
- **Feature Name**: Contrast Teaching
- **Type**: interaction
- **Description**: After a wrong answer, shows a very similar question where the student's logic would have worked alongside the current question where that logic fails. Teaches boundary awareness.
- **User Role**: student
- **UI Details**: Side-by-side question comparison.
- **Sub-features**: None.

### 48
- **Feature Name**: Micro-Proof Questions
- **Type**: interaction
- **Description**: After explanation, asks one tiny proof question to verify understanding, e.g., "Which clue makes B wrong?" "What condition must be true for C to be correct?" "Which step did you skip?"
- **User Role**: student
- **UI Details**: Single verification question inline.
- **Sub-features**: None.

### 49
- **Feature Name**: Shadow Student Model (Knowledge + Behavior)
- **Type**: engine-output-needing-UI
- **Description**: Maintains two models: Knowledge Model (what student knows) and Behavior Model (how student behaves while solving: rushing, hesitation, indecision, overconfidence, panic under timer, option-switching patterns, reading discipline, recovery after error).
- **User Role**: student
- **UI Details**: Internal models driving personalization.
- **Sub-features**: Knowledge model, Behavior model with 8+ tracked behaviors.

### 50
- **Feature Name**: First Wrong Step Detection
- **Type**: engine-output-needing-UI
- **Description**: For math and science especially, locates the first point where the student's reasoning became invalid, making correction more surgical: "Your first break happened here."
- **User Role**: student
- **UI Details**: Highlighted step in the solution path.
- **Sub-features**: None.

### 51
- **Feature Name**: "Same Mistake, Different Clothes" Display
- **Type**: component
- **Description**: Shows the student that multiple different-looking questions were missed for the same reason, e.g., "These 4 questions look different, but you missed them for the same reason."
- **User Role**: student
- **UI Details**: Grouped question display with common root cause highlighted.
- **Sub-features**: None.

### 52
- **Feature Name**: Mastery Fragility Score
- **Type**: data-visualization
- **Description**: Tracks per-concept: Accuracy, Stability, Transferability, Pressure resistance, Trap resistance. A student may have 85% accuracy but low stability, indicating hidden danger.
- **User Role**: student
- **UI Details**: Multi-dimensional mastery score display.
- **Sub-features**: 5 mastery dimensions.

### 53
- **Feature Name**: Academic Analyst Weekly Narratives
- **Type**: report
- **Description**: Weekly readable intelligence notes generated by the analyst, e.g., "You are improving in direct recall, but your main issue is still precision under close-option questions..."
- **User Role**: student
- **UI Details**: Narrative text block, not just dashboards.
- **Sub-features**: None.

### 54
- **Feature Name**: Confidence Fracture Detector
- **Type**: engine-output-needing-UI
- **Description**: Detects when a student originally leans toward the right answer but changes to the wrong one, indicating self-doubt, distractor seduction, or weak confidence in correct reasoning. Tracked separately from ordinary wrong answers.
- **User Role**: student
- **UI Details**: Special flag in the error analysis, different intervention (confidence strengthening).
- **Sub-features**: Answer change tracking, confidence diagnosis.

### 55
- **Feature Name**: Tiered Wrong Answer Review Depth
- **Type**: interaction
- **Description**: Not every wrong answer gets the same depth: Light review (obvious slips), Standard review (normal concept errors), Deep forensic review (repeated family misses, high-value concepts, serious misconception patterns).
- **User Role**: student
- **UI Details**: Different review card sizes/depths based on severity.
- **Sub-features**: Light review, Standard review, Deep forensic review.

### 56
- **Feature Name**: Prerequisite Ladder Drill-Down
- **Type**: engine-output-needing-UI
- **Description**: When an error is not caused by the current topic but by an earlier missing concept, the analyst traces downward, e.g., "Your algebra issue is actually coming from weak negative-number handling."
- **User Role**: student
- **UI Details**: Prerequisite chain display, foundational weakness surfacing.
- **Sub-features**: None.

### 57
- **Feature Name**: Counterfactual Review
- **Type**: interaction
- **Description**: After a wrong answer, asks: "What would make your chosen answer correct?" "In what kind of question would B be right?" "What would need to change in the wording for A to work?" Builds deeper reasoning about conditions of correctness.
- **User Role**: student
- **UI Details**: Counterfactual prompt questions inline.
- **Sub-features**: None.

### 58
- **Feature Name**: Recognition Failure Detection
- **Type**: engine-output-needing-UI
- **Description**: Detects when a student possesses a concept in memory but fails to detect/identify/match it in different wording, structure, representation, or context. Separates "I don't know" from "I didn't recognize it."
- **User Role**: student
- **UI Details**: Diagnosis message: "You seem to know this concept, but this question presented it in a less obvious form."
- **Sub-features**: Direct vs disguised performance gap analysis, cue sensitivity tracking, transfer profile.

### 59
- **Feature Name**: Concept Mastery State Spectrum
- **Type**: engine-output-needing-UI
- **Description**: Classifies concept mastery into: Unknown, Recognized but unusable, Known in familiar form only, Recognition-fragile, Transferable, Automatic. Much smarter than simple right/wrong tracking.
- **User Role**: student
- **UI Details**: Per-concept state indicator.
- **Sub-features**: 6 mastery states.

### 60
- **Feature Name**: Recognition Visibility Tagging
- **Type**: engine-output-needing-UI
- **Description**: Each question is tagged by recognition visibility: Level 1 Explicit, Level 2 Semi-explicit, Level 3 Implicit, Level 4 Disguised transfer, Level 5 Mixed-context identification.
- **User Role**: admin
- **UI Details**: Question authoring metadata field.
- **Sub-features**: 5 visibility levels.

### 61
- **Feature Name**: Concept Recognition Strength Score
- **Type**: data-visualization
- **Description**: Separate from concept accuracy, tracks whether the student can spot a concept when named, when hinted, when hidden, under pressure, and when distinguishing from similar concepts. E.g., Concept Knowledge: 82%, Concept Recognition Strength: 46%.
- **User Role**: student
- **UI Details**: Dual score display per concept.
- **Sub-features**: 5 recognition dimensions.

### 62
- **Feature Name**: Misrecognition Error Type
- **Type**: engine-output-needing-UI
- **Description**: A dedicated error type with subtypes: Hidden concept miss, Concept substitution, Familiar-form dependency, Cue blindness, Context transfer miss.
- **User Role**: student
- **UI Details**: Specific feedback messages per subtype.
- **Sub-features**: 5 subtypes.

### 63
- **Feature Name**: Recognition Training Mode
- **Type**: mode
- **Description**: A special practice mode where the first goal is to identify what kind of thinking the question requires before solving. Student must answer: what concept is being tested, what clue reveals that, which tool applies, which similar concept should be ruled out.
- **User Role**: student
- **UI Details**: Pre-solving identification questions before actual problem solving.
- **Sub-features**: Concept identification drills, trigger clue highlighting, similar concept comparison, disguised version practice.

### 64
- **Feature Name**: Recognition Gap Map
- **Type**: data-visualization
- **Description**: For each concept, maps where recognition breaks by form: direct formula form, word problem form, table form, graph form, mixed-topic form, timed condition. Shows where a concept "stops looking like itself" to the student.
- **User Role**: student
- **UI Details**: Matrix or heat map of recognition strength by form.
- **Sub-features**: Per-form recognition strength ratings.

### 65
- **Feature Name**: 8-Class Error Taxonomy
- **Type**: engine-output-needing-UI
- **Description**: Full taxonomy: Knowledge Absence (4 subtypes), Recognition Failure (5 subtypes), Comprehension/Interpretation Failure (5 subtypes), Application Failure (5 subtypes), Reasoning Breakdown (7 subtypes), Execution/Carelessness Failure (5 subtypes), Pressure/State Failure (6 subtypes), Structural/Prerequisite Failure (5 subtypes). Each with detection signals and best interventions.
- **User Role**: student
- **UI Details**: Diagnosis translated into natural language messages, not technical labels.
- **Sub-features**: 8 major classes with 42 subtypes total.

### 66
- **Feature Name**: Hybrid Error Diagnosis (Multi-Class)
- **Type**: engine-output-needing-UI
- **Description**: Recognizes that real mistakes are often hybrid, producing: Primary cause, Secondary cause, Context factor (amplifier). E.g., "Primary: Recognition Failure, Secondary: Similar-Concept Confusion, Context: Time Pressure."
- **User Role**: student
- **UI Details**: Multi-factor diagnosis display.
- **Sub-features**: Primary/Secondary/Amplifier classification.

### 67
- **Feature Name**: 10-Stage Diagnostic Pipeline
- **Type**: engine-output-needing-UI
- **Description**: A sequential diagnostic algorithm: Stage 0-Build Answer Event, Stage 1-Distractor Autopsy, Stage 2-Concept Possession Check, Stage 3-Recognition Check, Stage 4-Interpretation Check, Stage 5-Application Check, Stage 6-Reasoning Breakdown Check, Stage 7-Execution Slip Filter, Stage 8-State Amplifier Check, Stage 9-Prerequisite Trace, Stage 10-Final Diagnostic Resolution.
- **User Role**: student
- **UI Details**: Invisible to user, powers all diagnostic output.
- **Sub-features**: 11 stages, weighted evidence scoring, diagnostic confidence rating.

### 68
- **Feature Name**: Weighted Evidence Diagnostic Model
- **Type**: engine-output-needing-UI
- **Description**: Each error class gets weighted points from evidence signals. Strong evidence (direct mastery history, disguised mastery history, chosen distractor mapping) weighted higher than weak evidence (general topic difficulty, one-off guesses). Normalized to 0-1 scores.
- **User Role**: student
- **UI Details**: Produces confidence-weighted diagnosis.
- **Sub-features**: Evidence weighting, score normalization, confidence calculation.

### 69
- **Feature Name**: Diagnostic Confidence Display
- **Type**: component
- **Description**: The system adjusts its language based on diagnostic confidence. High confidence: "This is..." Low confidence: "This looks most like..." / "You may have..."
- **User Role**: student
- **UI Details**: Tone of diagnostic messages varies by confidence level.
- **Sub-features**: None.

### 70
- **Feature Name**: Structured Diagnostic Object Storage
- **Type**: engine-output-needing-UI
- **Description**: Every wrong answer stored as WrongAnswerDiagnosis object with: primary_error, secondary_error, amplifier, root_factor, confidence, evidence packet, misconception, question_family, error_family, severity, persistence, recommended_intervention array.
- **User Role**: student
- **UI Details**: Backend data structure powering all features.
- **Sub-features**: Full diagnostic object schema.

### 71
- **Feature Name**: Diagnosis-Specific Tone Mapping
- **Type**: interaction
- **Description**: Different diagnoses trigger different communication tones. Recognition Failure: "You seem to know this idea, but this version hid it from you." Application Failure: "You picked the right concept, but the way it was used here broke at this step." Etc.
- **User Role**: student
- **UI Details**: Tone varies by error class.
- **Sub-features**: 8 tone templates per error class.

### 72
- **Feature Name**: Diagnosis Validation by Repair Outcome
- **Type**: engine-output-needing-UI
- **Description**: The system tests its own diagnosis by monitoring whether the targeted intervention works. If improvement follows, diagnosis confidence rises. If not, another error class is reconsidered. Diagnosis becomes dynamic.
- **User Role**: student
- **UI Details**: Invisible to user, improves system accuracy over time.
- **Sub-features**: Outcome tracking, confidence adjustment, diagnosis revision.

### 73
- **Feature Name**: Error Severity Model
- **Type**: engine-output-needing-UI
- **Description**: Grades wrong answer severity across 5 dimensions: Depth (how fundamental), Breadth (how many families affected), Recurrence (how often), Repairability (how easily corrected), Exam Risk (how dangerous in real exam).
- **User Role**: student
- **UI Details**: Severity rating attached to each wrong answer.
- **Sub-features**: 5 severity dimensions.

### 74
- **Feature Name**: Error Persistence Model
- **Type**: engine-output-needing-UI
- **Description**: Tracks whether each mistake is: new, emerging, recurring, entrenched, repairing, resolved, or relapsing. Different persistence states require different responses.
- **User Role**: student
- **UI Details**: Persistence state label per error pattern.
- **Sub-features**: 7 persistence states.

### 75
- **Feature Name**: Intervention Selector Engine
- **Type**: engine-output-needing-UI
- **Description**: Based on finalized diagnosis, immediately decides the best next move: different intervention types for Knowledge Absence, Recognition Failure, Interpretation Failure, Application Failure, Reasoning Breakdown, Execution Slip, Pressure Failure, Prerequisite Failure.
- **User Role**: student
- **UI Details**: Automatically launches appropriate intervention activity.
- **Sub-features**: 8 intervention type mappings.

### 76
- **Feature Name**: Hybrid Diagnostic Architecture (Rule + Statistical + Outcome)
- **Type**: engine-output-needing-UI
- **Description**: Three-layer diagnostic model: Layer 1 (rule-based for explainability), Layer 2 (statistical adjustment from accumulated data), Layer 3 (outcome correction using post-intervention results to refine diagnosis over time).
- **User Role**: student
- **UI Details**: Invisible to user, powers diagnostic quality.
- **Sub-features**: Rule-based layer, statistical layer, outcome correction layer.

### 77
- **Feature Name**: Answer Forensics Engine (Module 1)
- **Type**: engine-output-needing-UI
- **Description**: Examines the wrong answer event.
- **User Role**: student
- **UI Details**: Backend module.
- **Sub-features**: None.

### 78
- **Feature Name**: Distractor Intent Engine (Module 2)
- **Type**: engine-output-needing-UI
- **Description**: Reads what the chosen distractor reveals about the student's thinking.
- **User Role**: student
- **UI Details**: Backend module.
- **Sub-features**: None.

### 79
- **Feature Name**: Reasoning Reconstruction Engine (Module 3)
- **Type**: engine-output-needing-UI
- **Description**: Infers how the student likely thought.
- **User Role**: student
- **UI Details**: Backend module.
- **Sub-features**: None.

### 80
- **Feature Name**: Question Family Mapper (Module 4)
- **Type**: engine-output-needing-UI
- **Description**: Maps question into conceptual/reasoning family clusters.
- **User Role**: student
- **UI Details**: Backend module.
- **Sub-features**: None.

### 81
- **Feature Name**: Misconception Detection Engine (Module 5)
- **Type**: engine-output-needing-UI
- **Description**: Matches the error against known misconception patterns.
- **User Role**: student
- **UI Details**: Backend module.
- **Sub-features**: None.

### 82
- **Feature Name**: Student Pattern Memory (Module 6)
- **Type**: engine-output-needing-UI
- **Description**: Stores and compares all wrong-answer fingerprints over time.
- **User Role**: student
- **UI Details**: Backend module.
- **Sub-features**: None.

### 83
- **Feature Name**: Academic Analyst Engine (Module 7)
- **Type**: engine-output-needing-UI
- **Description**: Produces trend insights and diagnosis summaries.
- **User Role**: student
- **UI Details**: Backend module powering narratives and dashboards.
- **Sub-features**: None.

### 84
- **Feature Name**: Recovery Orchestrator (Module 8)
- **Type**: engine-output-needing-UI
- **Description**: Chooses the best next intervention based on diagnosis.
- **User Role**: student
- **UI Details**: Backend module.
- **Sub-features**: None.

### 85
- **Feature Name**: Concept Identification Drills
- **Type**: interaction
- **Description**: Before solving, ask: "What concept is this testing?" "Which rule applies here?" "What kind of problem is this?" "What clue tells you that?" Trains recognition before execution.
- **User Role**: student
- **UI Details**: Pre-solving identification prompts.
- **Sub-features**: None.

### 86
- **Feature Name**: Lookalike Concept Comparison
- **Type**: interaction
- **Description**: Put two similar concepts side by side and show the difference: speed vs acceleration, mass vs weight, area vs perimeter, main idea vs supporting detail.
- **User Role**: student
- **UI Details**: Side-by-side concept comparison cards.
- **Sub-features**: None.

### 87
- **Feature Name**: Trigger Clue Highlighting
- **Type**: component
- **Description**: Teaches the student what signals should activate the correct concept by highlighting trigger clues in question text.
- **User Role**: student
- **UI Details**: Highlighted text in question wording.
- **Sub-features**: None.

### 88
- **Feature Name**: Concept Recognition Quadrant Model
- **Type**: engine-output-needing-UI
- **Description**: Distinguishes between: Concept possession (stored?), Concept recognition (can identify when relevant?), Concept execution (can use correctly once recognized?), Concept transfer (can use in unfamiliar forms?).
- **User Role**: student
- **UI Details**: Four-dimension concept profiling.
- **Sub-features**: 4 dimensions.

---

## FEATURES FOUND IN idea12.txt

### 1
- **Feature Name**: Premium Academic Concierge Program
- **Type**: flow
- **Description**: A managed, software-powered academic preparation system that continuously assesses the child, updates preparation strategy, coaches the child intelligently, keeps the parent informed, and escalates when risk is high. Positioned as a "private academic excellence system," not a learning app.
- **User Role**: all
- **UI Details**: Five core pillars: Intake/Baseline, Diagnosis/Readiness Intelligence, Dynamic Strategy/Intervention, Guided Student Coaching, Parent Intelligence/Concierge, Escalation/Human Oversight.
- **Sub-features**: Six operational layers: Diagnosis, Strategy, Execution, Oversight, Human Excellence, Parent Confidence.

### 2
- **Feature Name**: White-Glove Premium Onboarding / Intake
- **Type**: flow
- **Description**: Premium onboarding captures: child class/year, school type, exam board, subjects taken, target performance, weak subjects, known concerns, available study time, exam timeline, emotional profile, tutoring history, parent concerns, confidence level, anxiety level, attention consistency, resilience when corrected, tendency to rush/hesitate.
- **User Role**: parent
- **UI Details**: Multi-step form with sections: Student Profile, Parent Goal Profile, Learning History, Study Logistics, Child Mindset Profile. Should feel elite, not like ordinary app signup.
- **Sub-features**: Student Profile section, Parent Goal Profile section, Learning History section, Study Logistics section, Child Mindset Profile section.

### 3
- **Feature Name**: Child Performance Brief / Academic Readiness Brief
- **Type**: report
- **Description**: A polished output document generated after intake containing: current standing, key risks, target trajectory, 30-day action map, first interventions, expected outcomes. Generated at onboarding and updated periodically. Should look "elegant and expensive."
- **User Role**: parent
- **UI Details**: Clean, elite document with Executive Summary, Current Standing, Subject-by-Subject Readiness, Core Risk Factors, Error Pattern Summary, Immediate Priorities, First Intervention Plan, Forecast, Parent Guidance. "Beautifully written, not robotic."
- **Sub-features**: Executive Summary, Current Standing, Subject-by-Subject Readiness, Core Risk Factors, Error Pattern Summary, Immediate Priorities, First Intervention Plan, Forecast, Parent Guidance.

### 4
- **Feature Name**: Premium Baseline Diagnostic Assessment
- **Type**: flow
- **Description**: A deep initial diagnostic measuring: what the child knows, nearly knows, has memorized but cannot apply, where pressure causes errors, where misconceptions cluster, whether weak in reasoning/recall/transfer/pacing. Measures: recall, recognition, application, multi-step reasoning, speed, distractor vulnerability, structured-response ability, topic retention, error patterns, confidence behavior.
- **User Role**: student
- **UI Details**: Structured assessment flow, produces a child readiness model not just a score.
- **Sub-features**: Recall Strength, Recognition vs Application, Multi-step Reasoning, Speed and Time Pressure, Distractor Vulnerability, Error Pattern Type, Confidence Behavior.

### 5
- **Feature Name**: Parent Intelligence Layer / Executive Briefings
- **Type**: report
- **Description**: Weekly parent summary with: what improved, what worsened, what is being done, what to watch, whether intervention is on track, estimated exam readiness. "Academic portfolio reporting," not "student dashboard."
- **User Role**: parent
- **UI Details**: Executive-style update format, not raw analytics.
- **Sub-features**: Weekly summary, improvement tracking, worsening alerts, intervention status, readiness estimation.

### 6
- **Feature Name**: Child-Specific Mastery Map
- **Type**: data-visualization
- **Description**: Parent-visible map showing: subject, topics, subtopics, weakness clusters, mastery zones, fragile knowledge zones, memory decay zones, exam risk hotspots.
- **User Role**: parent
- **UI Details**: Visual map with topic tiles grouped by status (stable, improving, fragile, decaying, not yet secured).
- **Sub-features**: Weakness clusters, mastery zones, fragile zones, decay zones, risk hotspots.

### 7
- **Feature Name**: Precision Intervention Engine
- **Type**: engine-output-needing-UI
- **Description**: Intervention types differ based on diagnosed failure mode: concept rebuild, misconception correction, speed training, exam pressure training, memory reinforcement, multi-step reasoning repair, distractor immunity training, elite challenge mode, careless error reduction mode.
- **User Role**: student
- **UI Details**: System automatically selects and activates the appropriate intervention type.
- **Sub-features**: Concept Rebuild, Misconception Correction, Speed Conditioning, Pressure Adaptation, Recall Reinforcement, Exam Technique Repair, Confidence Stabilization, Elite Stretch.

### 8
- **Feature Name**: Human Review Checkpoints
- **Type**: flow
- **Description**: Premium checkpoints involving human experts: monthly strategist review, subject expert escalation, exam-readiness consultation, pre-exam war-room review.
- **User Role**: parent
- **UI Details**: Scheduled review moments, human escalation paths.
- **Sub-features**: Monthly strategist review, subject expert escalation, exam-readiness consultation, pre-exam war-room review.

### 9
- **Feature Name**: Intervention Alerts / Early Warning System
- **Type**: notification
- **Description**: Detects and surfaces: memory slippage, repeated misconception family, falling speed, rising panic errors, topic neglect, exam-readiness risk, false confidence, plateau after improvement, hidden weakness masked by easy question success.
- **User Role**: parent
- **UI Details**: Alert notifications with severity and intervention status.
- **Sub-features**: Memory slippage alert, misconception recurrence alert, speed decline alert, panic error alert, topic neglect alert, readiness risk alert, false confidence alert, plateau alert, hidden weakness alert.

### 10
- **Feature Name**: Concierge Support / Parent Concierge
- **Type**: interaction
- **Description**: Parent can ask natural-language questions like: "What should my child do this week?" "Is my child on track?" "What is the biggest risk right now?" "Should we intensify?" and get sharp, data-backed, high-quality answers using live child data, strategy state, and readiness context.
- **User Role**: parent
- **UI Details**: Concierge Desk screen with conversation thread, context cards (readiness, risks, strategy), prompt suggestions organized by family (Status, Risks, Strategy, Forecast, Weekend focus, Exam readiness). Response follows structured template: direct answer, evidence, meaning, what's being done, expectations, parent action.
- **Sub-features**: Status questions, Risk questions, Strategy questions, Forecast questions, Action questions, Explanation questions.

### 11
- **Feature Name**: Live Academic Surveillance Engine
- **Type**: engine-output-needing-UI
- **Description**: System continuously knows: what child is strong in, what is fragile, what is decaying, where speed is poor, where exam pressure causes loss, where misconceptions repeat, what topics are being avoided, what the next likely academic problem is.
- **User Role**: student
- **UI Details**: Powers all dashboards and alerts.
- **Sub-features**: Strength tracking, fragility tracking, decay tracking, speed tracking, pressure tracking, misconception tracking, avoidance tracking, predictive problem detection.

### 12
- **Feature Name**: Dynamic Strategy Engine
- **Type**: engine-output-needing-UI
- **Description**: Strategy keeps changing based on new evidence. Parent gets updates on strategy changes, e.g., "We have reduced heavy geometry drills this week because your child is now stable there. We are shifting focus to word-problem translation." Uses inputs: diagnostic data, recent performance, error patterns, memory decay signals, time to exam, subject importance, session consistency, pressure behavior, parent goals, prior intervention outcomes.
- **User Role**: all
- **UI Details**: Strategy changes visible in parent dashboard, strategy timeline, and weekly memos.
- **Sub-features**: Subject priority ranking, topic priority ranking, mode selection, intervention sequence, review schedule, parent-facing explanation, escalation recommendation. Core principles: stop mark leakage first, stabilize before accelerating, protect memory before decay, don't over-focus stable areas, shift visibly when evidence changes.

### 13
- **Feature Name**: Parent-Facing Executive Updates
- **Type**: notification
- **Description**: Executive-style updates (not noisy notifications), e.g., "Your child's math accuracy has improved from 61% to 74% in 10 days, but timed execution remains below target."
- **User Role**: parent
- **UI Details**: Composed, precise language. Categories: Milestone updates, Strategy updates, Risk alerts, Readiness updates, Effort/consistency updates, Concierge replies.
- **Sub-features**: 6 update categories.

### 14
- **Feature Name**: Automatic Intervention System
- **Type**: engine-output-needing-UI
- **Description**: System does not wait for parent to notice problems. Automatically: activates memory repair, shifts topic priority, assigns confidence rebuild mode, reduces overload if fatigue detected, switches to coach-led explanation flow, triggers misconception correction mode, recommends expert review.
- **User Role**: student
- **UI Details**: Interventions visible in intervention ledger.
- **Sub-features**: Memory repair activation, topic priority shifting, confidence rebuild, fatigue detection, coach-led mode, misconception correction trigger, expert review recommendation.

### 15
- **Feature Name**: Premium Parent Dashboard / Parent Command Center
- **Type**: screen
- **Description**: A private academic briefing room showing: Overall Readiness Panel (readiness band, confidence band, intervention intensity, trajectory vs target), Subject Risk Map (stable/improving/fragile/at risk/high priority per subject), Current Strategy Panel (current focus, why it changed, expected improvement), Risk Radar, Progress Highlights, Intervention Ledger Preview, Weekly Brief Panel, Concierge Prompt Panel, Exam Countdown Intelligence.
- **User Role**: parent
- **UI Details**: Deep midnight/charcoal base, warm ivory content surfaces, muted gold/slate blue/forest green accents, soft shadows, layered cards, large whitespace, premium typography. Executive summary bar at top with child name, exam target, days to exam, readiness band, intensity level, weekly status. Two-column layout for Strategy/Risk. Full-width weekly briefing card. Floating concierge dock. Calm, serious, no clutter, no cheap gamification. Three states: Default, Quiet (no risks), Alert (refined banner, not red noise).
- **Sub-features**: Readiness Overview Card, Current Strategy Card, Risk Radar, Progress Highlights, Intervention Ledger Preview, Weekly Brief Panel, Concierge Panel.

### 16
- **Feature Name**: Premium Student Inbuilt Coaching
- **Type**: mode
- **Description**: System acts as an elite tutor inside the app. When child gets a question wrong: diagnoses likely thought path, compares chosen option to correct one, explains why other options wrong, links error to pattern, applies repair action. Knows when to reteach vs drill vs challenge, encourages without being soft, adjusts pressure level intelligently.
- **User Role**: student
- **UI Details**: 5-step coaching flow per wrong answer.
- **Sub-features**: Thought path diagnosis, option comparison, pattern linking, repair action, pressure adjustment.

### 17
- **Feature Name**: BECE Readiness Brief
- **Type**: report
- **Description**: For BECE preparation specifically: current standing, most dangerous weaknesses, probable exam risks, subject-by-subject priority order, first 14-day intervention plan, projected readiness path.
- **User Role**: parent
- **UI Details**: Polished report document.
- **Sub-features**: Current standing, weakness analysis, risk assessment, subject prioritization, 14-day plan, readiness projection.

### 18
- **Feature Name**: Weekly Strategy Memo
- **Type**: report
- **Description**: Weekly premium artifact sent to parent with: Header (child name, week ending date, readiness band, days to exam), Executive Summary (3-5 sentences), What Improved (max 3), What Remains Exposed (max 3), Strategy Changes This Week, Next 7 Days plan, Parent Note (optional/minimal).
- **User Role**: parent
- **UI Details**: Editorial composition with generous whitespace. Archive view with chronological list tagged as "strategy shift," "major improvement," "high-risk week," "stable week." Page should feel like opening a private academic briefing note.
- **Sub-features**: 6 sections plus header.

### 19
- **Feature Name**: Pre-Exam Readiness Forecasting / Exam Forecast Center
- **Type**: data-visualization
- **Description**: Shows: current readiness level, projected readiness by exam date, topics most likely to cost marks if not fixed, confidence stability, speed under pressure, predicted mock band. Includes scenario cards: "If current pace holds," "If intervention succeeds fully," "If current risk persists."
- **User Role**: parent
- **UI Details**: Forecast hero with status/confidence/time remaining. Forecast breakdown with strongest/most exposed subjects, mark-loss zones, projection curve, intensity recommendation. Scenario cards carefully worded and sober.
- **Sub-features**: Current readiness, projected readiness, mark-loss predictions, confidence stability, speed under pressure, mock prediction, scenario analysis.

### 20
- **Feature Name**: Intervention Logs / Intervention Ledger
- **Type**: data-visualization
- **Description**: Shows parent what was detected, what action the system took, whether the action worked. Full ledger with filters (Active, Completed, Redesigned, Escalated, By Subject, By Intervention Class). Each entry: date started, issue detected, intervention class, objective, current status, next review, outcome tag.
- **User Role**: parent
- **UI Details**: Ledger table/card list format. Detail drawer shows: linked risk, why intervention selected, session structure, signals monitored, progress so far, decision at next checkpoint. "Should feel like a case-management interface."
- **Sub-features**: Detection log, action log, outcome tracking, filtering system.

### 21
- **Feature Name**: High-Touch Milestone Reviews
- **Type**: report
- **Description**: Formal review moments: 30-day review (early movement, diagnosis refinement), 60-day review (trend strength, stability vs fragility, subject restructuring), Pre-mock review (readiness under exam conditions, likely weak points, mock strategy, do-not-overfocus list), Pre-exam review (final exposure areas, tactical focus for final days/weeks). Should feel like private progress consultations.
- **User Role**: parent
- **UI Details**: Review cover card with review type, date, readiness position, overall trend, major status phrase. Sections: executive position, subject progression, intervention effectiveness, confirmed strengths, unresolved risks, strategic adjustments, forecast, parent guidance. Signature footer for premium tiers ("Prepared by the Academic Concierge System"). "Closer to an advisory report than an app update."
- **Sub-features**: 30-day, 60-day, pre-mock, pre-exam reviews.

### 22
- **Feature Name**: Premium Communication Style / UX Writing
- **Type**: setting
- **Description**: Premium language guidelines: use "intervention," "readiness," "trajectory," "strategy," "calibration" instead of "fun learning," "exciting quizzes," "badge unlocked," "awesome job." Language shapes prestige perception.
- **User Role**: all
- **UI Details**: Writing guidelines for all parent-facing communications. Words to use: readiness, trajectory, stability, exposure, intervention, review, forecast, calibration, strengthening, command center, strategy. Words to avoid: awesome, super fun, streak reward, badge unlocked, keep crushing it.
- **Sub-features**: None.

### 23
- **Feature Name**: Coach Mode (Student)
- **Type**: mode
- **Description**: For learning and correction. The system teaches, explains, and walks back errors.
- **User Role**: student
- **UI Details**: Part of the guided student experience.
- **Sub-features**: None.

### 24
- **Feature Name**: Fix My Weakness Mode
- **Type**: mode
- **Description**: For targeted interventions. The system isolates a weakness and actively repairs it. 3-5 carefully sequenced items with guided correction moments and final summary card stating whether issue is "less severe," "still active," or "improving but fragile."
- **User Role**: student
- **UI Details**: Issue title, explanation of why it matters, sequenced items, guided correction, summary card.
- **Sub-features**: None.

### 25
- **Feature Name**: Memory Recovery Mode
- **Type**: mode
- **Description**: For topics slipping out of retention. Reactivates memory through carefully shaped recall challenges. Quieter tone, confidence-restoring, recall prompts, contrast prompts, confidence checks, mixed-form retrieval, final memory stability note.
- **User Role**: student
- **UI Details**: Quieter visual tone. Final state message like "Your understanding is returning, but this topic still needs one more clean recall check later."
- **Sub-features**: Recall prompts, contrast prompts, confidence checks, mixed-form retrieval.

### 26
- **Feature Name**: Mental Pressure Mode
- **Type**: mode
- **Description**: For speed and resilience. Simulates exam pressure and trains thinking under time constraints. Darker, more focused UI mode with stronger timer emphasis, sharper sound/animation, immediate calming review after each stretch. Flow: pressure set intro, timed burst, micro recovery review, second burst, final performance interpretation. Distinguishes between knowledge weakness, speed weakness, and pressure weakness.
- **User Role**: student
- **UI Details**: Darker focused mode, strong timer, sound/animation support, calming review between bursts.
- **Sub-features**: Timed bursts, micro recovery reviews, knowledge vs speed vs pressure diagnosis.

### 27
- **Feature Name**: Exam Readiness Mode
- **Type**: mode
- **Description**: Integrated mixed-topic exam simulation and final sharpening. Mixed-topic structure, pacing indicator, confidence check moments, end-of-session readiness note.
- **User Role**: student
- **UI Details**: Session objective, mixed-topic structure, pacing indicator, readiness note at end. "Should feel like dress rehearsal with intelligence."
- **Sub-features**: Mixed-topic practice, pacing tracking, confidence checks, readiness assessment.

### 28
- **Feature Name**: Elite Mode / Elite Stretch
- **Type**: mode
- **Description**: For exceptional students who need harder but syllabus-bound challenge. Harder item set, richer reasoning prompts, precision-focused feedback, optional benchmark language. Dignified, aspirational, difficult but not punishing. "High-performance training, not generic hard mode."
- **User Role**: student
- **UI Details**: Elevated difficulty within syllabus scope.
- **Sub-features**: Harder items, rich reasoning prompts, precision feedback.

### 29
- **Feature Name**: Child Readiness Model (Live Profile)
- **Type**: engine-output-needing-UI
- **Description**: Central premium data object containing: overall readiness band, subject readiness bands, topic mastery graph, fragile knowledge zones, memory decay markers, recurring misconception families, pressure performance rating, speed band, confidence stability band, consistency band, intervention priority queue, projected readiness trajectory, exam risk level.
- **User Role**: all
- **UI Details**: Powers parent reporting, strategy changes, concierge responses, and escalation rules.
- **Sub-features**: 13 readiness dimensions.

### 30
- **Feature Name**: Readiness Scoring Model and Bands
- **Type**: data-visualization
- **Description**: Layered composite readiness score from: knowledge solidity, application strength, reasoning quality, speed under time pressure, memory stability, confidence resilience, consistency, exam technique quality. Top-level bands: Fragile, Building, Strengthening, Strong, Exam-Ready, Elite Ready.
- **User Role**: parent
- **UI Details**: Parent sees bands and interpreted movement, not naked numbers. Internal 0-100 scale per dimension.
- **Sub-features**: 8 readiness dimensions, 6 readiness bands.

### 31
- **Feature Name**: Subject Risk Map
- **Type**: data-visualization
- **Description**: Each subject shows status: stable, improving, fragile, at risk, high priority. With current focus rank and intervention type underway.
- **User Role**: parent
- **UI Details**: Grid of subject cards on parent dashboard.
- **Sub-features**: Per-subject status, focus rank, intervention type.

### 32
- **Feature Name**: Risk Radar
- **Type**: component
- **Description**: Visible but restrained module showing active risks with: label, severity, direction (trend arrow), intervention status. Parent can click into any risk to see: what triggered it, what it means, what is being done, when it will be reviewed.
- **User Role**: parent
- **UI Details**: Risk cards with severity chips, trend arrows. Tabbed view: Active, Watching, Improving, Resolved, Escalated.
- **Sub-features**: Risk detail pages with trigger, meaning, action, review date.

### 33
- **Feature Name**: Risk Flag Taxonomy
- **Type**: engine-output-needing-UI
- **Description**: Formal risk groups: Knowledge risks (concept gap, incomplete recall, fragile mastery, topic decay, shallow recognition), Reasoning risks (multi-step breakdown, condition-missing, surface-reading trap, transfer failure, weak elimination, poor structured-response), Performance risks (slow completion, time-pressure collapse, careless error spike, pacing inconsistency, answer abandonment, weak finishing), Behavioral risks (subject avoidance, low session consistency, emotional drop, hesitation, rushing, passive guessing), Strategic risks (wrong focus allocation, poor revision spacing, over-practice of stable topics, insufficient exam simulation), Confidence risks (fragile confidence, collapse after wrong answers, overconfidence sloppiness, recovery lag). Severity: Low/Moderate/High/Critical. States: New/Active/Watching/Improving/Resolved/Escalated.
- **User Role**: all
- **UI Details**: Risk Center screen with tabs by state.
- **Sub-features**: 6 risk groups with 30+ specific risk types, 4 severity levels, 6 states.

### 34
- **Feature Name**: Risk Trigger Rules
- **Type**: engine-output-needing-UI
- **Description**: Each risk has explicit detection logic: Memory decay (stable topic falls below retention threshold across two spaced checks), Slow completion (untimed strong, timed weak repeatedly), Misconception recurrence (same error across multiple items in family), Plateau (no movement despite active intervention), Confidence instability (accuracy drops after one-two wrong answers in session), Avoidance (repeatedly under-engages with high-priority subject).
- **User Role**: all
- **UI Details**: Automated triggers powering the risk system.
- **Sub-features**: 6 defined trigger rules.

### 35
- **Feature Name**: Strategy Timeline Screen
- **Type**: screen
- **Description**: Chronological view of how the system has changed approach over time. Each entry: what changed, why, what evidence caused it, what outcome was expected, actual outcome when available. Shows intelligent adaptation.
- **User Role**: parent
- **UI Details**: Vertical timeline with elegant milestone nodes. Top summary (current strategy label, age, last shift date, next review date). Right-side insight rail showing strategy themes, intensity changes, subject priority changes.
- **Sub-features**: Timeline entries, insight rail, strategy theme tracking.

### 36
- **Feature Name**: Risk Center Screen
- **Type**: screen
- **Description**: Structured view of all current and past risks with tabs (Active, Watching, Improving, Resolved, Escalated). Risk list cards with: title, category icon, severity chip, state label, interpretation, linked intervention state, next review date.
- **User Role**: parent
- **UI Details**: Risk detail page with: header (name, severity, state, opened date, category), why flagged (plain language), why it matters, what system is doing, evidence stream, review plan, outcome history.
- **Sub-features**: Tabbed risk views, risk detail pages.

### 37
- **Feature Name**: Intervention Lifecycle Management
- **Type**: flow
- **Description**: Interventions follow a managed lifecycle: Detection, Qualification, Assignment, Activation, Monitoring, Review, Resolution/Continuation/Redesign/Escalation. Each intervention has trigger conditions, objective, mode type, session pattern, expected outcome, review point, success criteria.
- **User Role**: all
- **UI Details**: Visible in intervention ledger and intervention cards.
- **Sub-features**: 7 lifecycle states.

### 38
- **Feature Name**: Intervention Card (Parent-Facing)
- **Type**: component
- **Description**: Each active intervention shown to parent as a card with: Issue, Why it matters, Current action, Status, Review date, Expected outcome.
- **User Role**: parent
- **UI Details**: Card format in the intervention ledger.
- **Sub-features**: None.

### 39
- **Feature Name**: Human Escalation System
- **Type**: flow
- **Description**: Defined escalation rules: escalate when progress stalls, risk persists beyond threshold, exam date close with below-target readiness, unusual pattern needing human judgment, parent expresses serious concern, behavioral issues affect progress. Escalation outputs: strategist review, subject specialist review, updated intervention plan, parent briefing, intensified schedule, pre-exam support plan.
- **User Role**: all
- **UI Details**: Escalated risks visible in Risk Center.
- **Sub-features**: 6 escalation triggers, 6 escalation outputs.

### 40
- **Feature Name**: Human Expert Roles
- **Type**: admin-tool
- **Description**: Optional human roles for premium tiers: academic strategist, subject specialist, exam technique coach, parent success manager, concierge academic advisor, intervention reviewer.
- **User Role**: admin
- **UI Details**: Reviewer signature block on milestone reviews.
- **Sub-features**: 6 human roles.

### 41
- **Feature Name**: Premium Package Tiers
- **Type**: setting
- **Description**: Three tiers: Tier 1 Core Premium (full diagnostic, mastery map, personalized strategy, adaptive daily work, memory/weakness engine, parent dashboard, weekly summary, risk alerts, readiness tracking), Tier 2 Concierge Premium (adds concierge Q&A, strategy shift notes, milestone reviews, richer communication, stronger reporting, custom academic targets, exam countdown planning), Tier 3 Signature/Elite Concierge (adds dedicated strategist touchpoint, priority support, more frequent reviews, pre-mock readiness review, pre-exam war-room plan, custom intervention intensification, white-glove reporting, direct parent briefing sessions).
- **User Role**: parent
- **UI Details**: Pricing based on oversight intensity, not gimmicks.
- **Sub-features**: 3 tiers with escalating features.

### 42
- **Feature Name**: Readiness Deep View Screen
- **Type**: screen
- **Description**: Parent sees subject and topic intelligence in depth. Sections: Readiness Summary Shelf (compact cards for overall readiness, subject average stability, memory stability, pressure readiness, confidence resilience), Subject Readiness Grid (per-subject cards with readiness band, 14-day movement, stable/fragile areas, active risk, intervention), Topic Mastery Matrix (topic chips grouped by stable/strengthening/fragile/decaying/unproven), Pressure Sensitivity View, Memory Stability View, Reasoning Pattern Summary narrative.
- **User Role**: parent
- **UI Details**: Professional performance review feel. Hover/tap on topic chips reveals last checked date, current risk, whether under reinforcement.
- **Sub-features**: Readiness Summary Shelf, Subject Readiness Grid, Topic Mastery Matrix, Pressure Sensitivity View, Memory Stability View, Reasoning Pattern Summary.

### 43
- **Feature Name**: Intervention Ledger Screen
- **Type**: screen
- **Description**: Full-screen view of all system interventions with filters (Active, Completed, Redesigned, Escalated, By Subject, By Intervention Class). Detail drawer shows linked risk, why selected, session structure, signals monitored, progress, next checkpoint decision.
- **User Role**: parent
- **UI Details**: Case-management interface feel.
- **Sub-features**: Filtering system, detail drawer.

### 44
- **Feature Name**: Weekly Strategy Memo Archive
- **Type**: screen
- **Description**: Chronological archive of all weekly strategy memos with tags (strategy shift, major improvement, high-risk week, stable week).
- **User Role**: parent
- **UI Details**: Readable enough to print or share.
- **Sub-features**: Tagged archive.

### 45
- **Feature Name**: Milestone Reviews Archive
- **Type**: screen
- **Description**: Archive of all formal milestone review documents (30-day, 60-day, pre-mock, pre-exam).
- **User Role**: parent
- **UI Details**: Review cover cards with review type, date, readiness position, trend.
- **Sub-features**: None.

### 46
- **Feature Name**: Concierge Desk Screen
- **Type**: screen
- **Description**: Premium intelligence assistant screen. Left column: conversation thread with premium bubbles/cards. Right column/upper shelf: context cards (current readiness, top active risks, current strategy, next review point). Prompt suggestions organized by family.
- **User Role**: parent
- **UI Details**: Response styling visually separates: direct answer, evidence-based explanation, meaning, what system is doing, expectations.
- **Sub-features**: Conversation thread, context cards, prompt suggestions.

### 47
- **Feature Name**: Exam Forecast Center Screen
- **Type**: screen
- **Description**: Forward-looking premium view of where child is headed. Forecast hero (status, confidence, time remaining). Forecast breakdown (strongest/most exposed subjects, mark-loss zones, projection curve, intensity recommendation). Scenario cards (current pace, full intervention success, risk persistence).
- **User Role**: parent
- **UI Details**: Sober, carefully worded scenarios. Not sensational.
- **Sub-features**: Forecast hero, breakdown, scenario cards.

### 48
- **Feature Name**: Student Guided Home Screen
- **Type**: screen
- **Description**: Child's premium home showing: Hero card (today's main mission, reason, duration estimate, start button), Focus cards (current strength area, current repair area, upcoming challenge), Momentum section (consistency, progress against week's mission, premium labels like "stability building"), Reflection panel (smart reflection note instead of badges).
- **User Role**: student
- **UI Details**: Guided, calm, purposeful. No childish gamification. Example reflection: "You are becoming more accurate in algebra, but still rush when two-step reasoning is required."
- **Sub-features**: Hero card, Focus cards, Momentum section, Reflection panel.

### 49
- **Feature Name**: Today's Focus Session Screen
- **Type**: screen
- **Description**: Structured guided study mission aligned to strategy. Entry screen (mission title, why it matters, expected improvement, calm start CTA). Session body (question/prompt area, subtle progress indicator, coaching rail/help panel, whitespace). Session end (what improved, what needs work, next session target).
- **User Role**: student
- **UI Details**: "More like a guided performance studio than a worksheet."
- **Sub-features**: Entry screen, session body, end state.

### 50
- **Feature Name**: Wrong-Answer Coaching View Screen
- **Type**: screen
- **Description**: Screen showing: Top explanation card (what you chose, likely reason, why tempting), Middle comparison panel (split/stacked: why your answer fails, why correct works, why others don't hold), Pattern note (insight card linking to recurring error), Repair CTA (retry with guidance, contrast example, short drill).
- **User Role**: student
- **UI Details**: Very calm, no punishing red. Purpose is recovery, not embarrassment.
- **Sub-features**: Explanation card, comparison panel, pattern note, repair CTA.

### 51
- **Feature Name**: Weakness Repair Session Screen
- **Type**: screen
- **Description**: Highly targeted mini-treatment experience: issue title, explanation of why it matters, 3-5 carefully sequenced items, guided correction moments, final summary card.
- **User Role**: student
- **UI Details**: End summary states whether issue is less severe, still active, or improving but fragile.
- **Sub-features**: Issue title, explanation, sequenced items, correction moments, summary card.

### 52
- **Feature Name**: Memory Recovery Session Screen
- **Type**: screen
- **Description**: Reactivates slipping knowledge. Quieter tone, confidence-restoring, recall prompts, contrast prompts, confidence checks, mixed-form retrieval, final memory stability note.
- **User Role**: student
- **UI Details**: Quieter visual tone than normal practice.
- **Sub-features**: Recall prompts, contrast prompts, confidence checks, mixed-form retrieval, stability note.

### 53
- **Feature Name**: Mental Pressure Session Screen
- **Type**: screen
- **Description**: High-pressure sharpness mode. Darker focused mode, stronger timer emphasis, sharper sound/animation, immediate calming review. Flow: pressure set intro, timed burst, micro recovery review, second burst, final performance interpretation.
- **User Role**: student
- **UI Details**: Darker mode, strong timer, sound/animation, calming reviews.
- **Sub-features**: Pressure set intro, timed bursts, micro recovery, performance interpretation.

### 54
- **Feature Name**: Exam Readiness Session Screen
- **Type**: screen
- **Description**: Dress rehearsal with intelligence. Session objective, mixed-topic structure, pacing indicator, confidence check moments, end-of-session readiness note.
- **User Role**: student
- **UI Details**: End note like "You handled familiar questions well, but lost marks when science interpretation and time pressure combined."
- **Sub-features**: None.

### 55
- **Feature Name**: Elite Stretch Session Screen
- **Type**: screen
- **Description**: Harder but syllabus-bound challenge for high performers. Harder item set, richer reasoning prompts, precision-focused feedback, optional benchmark language.
- **User Role**: student
- **UI Details**: Dignified, aspirational, not punishing.
- **Sub-features**: None.

### 56
- **Feature Name**: Progress Reflection View
- **Type**: screen
- **Description**: Listed as a student surface for reflecting on progress.
- **User Role**: student
- **UI Details**: No detailed specification beyond listing.
- **Sub-features**: None.

### 57
- **Feature Name**: Parent Update System / Messaging Discipline
- **Type**: notification
- **Description**: Structured update types: Weekly Strategy Memo (scheduled), Meaningful Progress Notice (when milestone crossed), Risk Alert (when something important appears), Strategy Shift Notice (when system changes approach), Milestone Review (at formal checkpoints), Concierge Response (on demand). Priority hierarchy from immediate (high-severity risk) to scheduled (weekly memo) to meaningful-only (progress). Avoids low-value completion messages, childish celebration spam, tiny percentage changes.
- **User Role**: parent
- **UI Details**: Priority 1-5 hierarchy controlling notification frequency.
- **Sub-features**: 6 update types, 5 priority levels.

### 58
- **Feature Name**: Premium Visual Design System
- **Type**: setting
- **Description**: Parent-facing: deep midnight/charcoal base, warm ivory/soft white content surfaces, muted gold/slate blue/forest green/subtle crimson accents, soft shadows, layered cards, restrained motion, large whitespace, premium serif/humanist display for headlines, readable sans-serif for body. Student-facing: motivational, intelligent, focused, animated only for clarity/momentum, confidence-building not childish. Card system: large, layered, premium rounded corners. Status chips: subtle and elegant. Motion: soft transitions, fade-ups, content continuity, state morphing. No bouncy cartoon motion on parent screens.
- **User Role**: all
- **UI Details**: Complete design direction specification.
- **Sub-features**: Color system, typography pairing, spacing system, card system, status chips, motion system, empty state design, microcopy style.

### 59
- **Feature Name**: Exam Countdown Intelligence
- **Type**: component
- **Description**: Premium countdown with intelligent interpretation: days left, current readiness, most urgent topics, schedule intensity suggestion, urgency level, coverage status, top pending risks.
- **User Role**: parent
- **UI Details**: Part of parent command center.
- **Sub-features**: Days left, readiness, urgent topics, intensity suggestion.

### 60
- **Feature Name**: Strategy Engine Decision Logic
- **Type**: engine-output-needing-UI
- **Description**: Core decision principles: stop mark leakage first, stabilize before accelerating, protect memory before decay spreads, do not over-focus already stable areas, shift visibly when evidence changes. Inputs: baseline, live readiness, risk flags, intervention outcomes, time to exam, subject importance, parent goals, consistency, pressure behavior. Outputs: subject/topic priority, mode selection, intervention sequence, review schedule, parent explanation, escalation recommendation.
- **User Role**: all
- **UI Details**: Strategy decisions visible in strategy timeline and current strategy card.
- **Sub-features**: 5 decision principles, multiple inputs and outputs.

### 61
- **Feature Name**: Intervention Engine (Separate from Strategy Engine)
- **Type**: engine-output-needing-UI
- **Description**: Strategy engine decides what to do; intervention engine decides how. 8 intervention classes: Concept Rebuild, Misconception Correction, Recall Reinforcement, Speed Conditioning, Pressure Adaptation, Exam Technique Repair, Confidence Stabilization, Elite Stretch. Each has trigger conditions, objective, mode type, session pattern, expected outcome, review point, success criteria.
- **User Role**: student
- **UI Details**: Drives student session content.
- **Sub-features**: 8 intervention classes with full specifications.

### 62
- **Feature Name**: Memory and Decay Engine
- **Type**: engine-output-needing-UI
- **Description**: Tracks: when topic was last mastered, how often recall weakened, how solid retrieval is, how much reinforcement needed, whether child retains under different question forms, which topics decay faster. Functions: detect weak retention, rank decay risk, schedule timely reinforcement, verify genuine vs shallow memory, distinguish "I once got it right" from "now stable."
- **User Role**: student
- **UI Details**: Powers memory recovery sessions and decay alerts.
- **Sub-features**: Retention tracking, decay risk ranking, reinforcement scheduling, memory genuineness verification.

### 63
- **Feature Name**: Premium Backend Data Model
- **Type**: engine-output-needing-UI
- **Description**: Formal backend objects: ParentProfile (goals, concerns, communication preferences, tier), StudentPremiumProfile (academic context, emotional context, study logistics, targets), ReadinessProfile (live readiness across dimensions), StrategyState (current priorities, active logic, next review, mode plan), InterventionRecord (type, objective, dates, status, results), RiskFlag (category, severity, state, evidence, linked intervention, review dates), StrategyMemo (weekly briefing), ConciergeResponse (question, answer, evidence, guidance), MilestoneReview (formal checkpoint report), ParentDashboardSnapshot (current executive state), CommunicationLedger (all briefings, memos, alerts, responses).
- **User Role**: admin
- **UI Details**: Backend objects powering all premium surfaces.
- **Sub-features**: 11 formal data objects.

### 64
- **Feature Name**: Premium KPI Framework
- **Type**: admin-tool
- **Description**: Four KPI categories: Parent confidence metrics (perceived clarity, control, reassurance, concierge usefulness), Student improvement metrics (readiness improvement rate, misconception correction rate, memory stability growth, pressure-performance improvement, subject stabilization rate), System intelligence metrics (early risk detection rate, intervention success rate, time to corrective action, forecast quality), Commercial metrics (premium upgrade rate, retention, milestone review engagement, referral rate).
- **User Role**: admin
- **UI Details**: Internal metrics dashboard.
- **Sub-features**: 4 metric categories with 16+ individual metrics.

### 65
- **Feature Name**: Premium Program Naming Convention
- **Type**: setting
- **Description**: Product should be called a "Program" not a "subscription." Examples: Excellence Program, Concierge Prep Program, Signature Readiness Program, BECE Elite Program, BECE Excellence Concierge, BECE Signature Prep, BECE Private Readiness Program.
- **User Role**: all
- **UI Details**: "Programs sound managed. Subscriptions sound passive."
- **Sub-features**: None.

### 66
- **Feature Name**: Pre-Exam War-Room Mode
- **Type**: mode
- **Description**: Final run-up before exam becomes more intensive, strategic, and tightly monitored. Includes final readiness position, high-confidence strengths, last exposed areas, final intervention priorities, pace/calm recommendations, final week/final days guidance.
- **User Role**: all
- **UI Details**: War-room briefing format.
- **Sub-features**: Final readiness assessment, strength/weakness summary, priority guidance, pacing recommendations.

### 67
- **Feature Name**: Quiet Prestige Design / Premium UI Aesthetic
- **Type**: setting
- **Description**: Dark, elegant, refined, non-gimmicky UI. Not flashy, not toy-like, not high-saturation edtech. More refined than playful, more private office than classroom. No visual chaos, no clutter, no cheap gamification, no noisy UI on parent side.
- **User Role**: all
- **UI Details**: Design philosophy applied across all premium screens.
- **Sub-features**: None.

### 68
- **Feature Name**: Wrong-Answer Intelligence Module (Premium)
- **Type**: engine-output-needing-UI
- **Description**: Premium version of wrong-answer analysis with full 7-step flow: identify likely reason, explain why chosen answer was tempting, explain why it fails, explain why correct answer works, explain why remaining options fail, link miss to known pattern, trigger repair action. Typical inferred reasons: surface keyword trap, partial concept recognition, missing condition, careless calculation, rushed reading, concept confusion, weak multi-step reasoning, overconfident guess.
- **User Role**: student
- **UI Details**: 7-step analysis flow within the premium student experience.
- **Sub-features**: 8 inferred reason types, 6 repair action types (mini reteach, compare-and-contrast drill, slower guided retry, mini question family drill, delayed retest, future pressure variant).

### 69
- **Feature Name**: Parent Command Center UX Behavior Rules
- **Type**: setting
- **Description**: 7 rules: always interpret before visualizing, show only decision-relevant info on home screen, make current strategy visible always, show what changed and why, make risk status legible and calm, never make parent dig too far, language must remain composed/exact/non-gimmicky.
- **User Role**: parent
- **UI Details**: UX guidelines for dashboard behavior.
- **Sub-features**: 7 behavior rules.

### 70
- **Feature Name**: Cross-Screen Design Language
- **Type**: setting
- **Description**: Card system (large, layered, premium, rounded but not childish), Status chips (subtle, elegant, never loud), Motion (soft transitions, fade-ups, state morphing, no bouncy cartoon on parent screens), Empty states (always intelligent, e.g., "No active academic risks require escalation right now"), Microcopy (composed private advisory system voice).
- **User Role**: all
- **UI Details**: Design system specification.
- **Sub-features**: Card system, status chips, motion rules, empty states, microcopy voice.

### 71
- **Feature Name**: Concierge Response Templates
- **Type**: component
- **Description**: Master template for concierge answers: direct answer, what data shows, what it means, what is being done, what to expect next, parent action if needed. Three example responses provided for "Are we on track?", "Why did performance drop?", and "What should we focus on this weekend?"
- **User Role**: parent
- **UI Details**: Structured response format that sounds like a private academic advisor.
- **Sub-features**: 6-part response structure.

### 72
- **Feature Name**: Premium Enrollment Flow
- **Type**: flow
- **Description**: 8-stage parent journey: Enrollment (join program not platform), Parent Intake, Baseline Diagnostic, Readiness Brief Delivered, Strategy Activation, Continuous Management, Milestone Reviews, Pre-Exam War-Room Mode.
- **User Role**: parent
- **UI Details**: Curated, deliberate journey. Language makes clear this is a managed experience with assessment first, strategy following diagnosis, and continuous visibility.
- **Sub-features**: 8 stages.

### 73
- **Feature Name**: Confidence Rebuild Mode
- **Type**: mode
- **Description**: Activated when system detects child spiraling after errors. Part of the automatic intervention system.
- **User Role**: student
- **UI Details**: Mode with reduced pressure and confidence-building activities.
- **Sub-features**: None.

### 74
- **Feature Name**: Coach-Led Explanation Flow
- **Type**: mode
- **Description**: Switched to automatically when the system detects the child needs more guided explanation rather than self-directed practice.
- **User Role**: student
- **UI Details**: More hand-held, step-by-step explanations.
- **Sub-features**: None.

### 75
- **Feature Name**: Revision Path Mode
- **Type**: mode
- **Description**: Structured review flow based on current risk and upcoming exam needs.
- **User Role**: student
- **UI Details**: Listed as a student-facing guided mode.
- **Sub-features**: None.

### 76
- **Feature Name**: Effort and Consistency Updates
- **Type**: notification
- **Description**: Premium parents receive effort/discipline updates, e.g., "completed all scheduled intervention cycles this week," "skipped two high-priority reinforcement sessions," "consistency improved significantly over the last 10 days."
- **User Role**: parent
- **UI Details**: Part of the update system.
- **Sub-features**: None.

### 77
- **Feature Name**: Parent-Visible Strategy Changes
- **Type**: component
- **Description**: The parent sees updates on strategy changes, e.g., "We reduced emphasis on basic number operations because your child is now stable there. We increased focus on science interpretation questions."
- **User Role**: parent
- **UI Details**: Visible in current strategy card, weekly memos, and strategy timeline.
- **Sub-features**: None.

### 78
- **Feature Name**: Pressure Sensitivity View
- **Type**: data-visualization
- **Description**: Shows where the child performs differently under time pressure, in the Readiness Deep View.
- **User Role**: parent
- **UI Details**: Part of Readiness Deep View screen.
- **Sub-features**: None.

### 79
- **Feature Name**: Memory Stability View
- **Type**: data-visualization
- **Description**: Shows which topics are retained well and which need reinforcement, in the Readiness Deep View.
- **User Role**: parent
- **UI Details**: Part of Readiness Deep View screen.
- **Sub-features**: None.

### 80
- **Feature Name**: Reasoning Pattern Summary
- **Type**: data-visualization
- **Description**: Narrative-driven card in Readiness Deep View describing recurring thought patterns, e.g., "Your child currently handles direct recall well, but still loses marks when a question requires a shift from recognition to application."
- **User Role**: parent
- **UI Details**: Narrative card interpreting child's academic style.
- **Sub-features**: None.

### 81
- **Feature Name**: Premium Implementation Priority Order
- **Type**: setting
- **Description**: First design sprint should focus on: Parent Command Center, Weekly Strategy Memo, Risk Center, Intervention Ledger, Concierge Desk, Student Guided Home, Wrong-Answer Coaching View, Mental Pressure Session. These carry the premium identity most strongly.
- **User Role**: admin
- **UI Details**: Implementation ordering specification.
- **Sub-features**: 8 priority screens.

### 82
- **Feature Name**: Signature Launch Scope (MVP)
- **Type**: setting
- **Description**: First premium release must include: premium intake, baseline diagnostic, live readiness profile, parent command center, risk center, current strategy card, intervention ledger, weekly strategy memo, wrong-answer intelligence, concierge v1, milestone reviews. Phase 2 adds: concierge Q&A, intervention logs, exam forecast model, memory decay alerts, readiness milestone reviews. Phase 3 adds: human escalation layer, strategist review workflows, pre-exam war-room reports, premium account management.
- **User Role**: admin
- **UI Details**: Three-phase launch plan.
- **Sub-features**: Phase 1 (11 items), Phase 2 (5 items), Phase 3 (4 items).

---

