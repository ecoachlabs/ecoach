# PART 3: DOMAIN IMPLEMENTATION SPECIFICATIONS (Domains 1-11)

---

## HOW TO READ THIS DOCUMENT

Each domain section follows this exact structure:

1. **Domain Overview** -- Purpose, scope, entry points
2. **Features** -- Every feature with acceptance criteria
3. **Screens** -- Every screen with sections, states, data sources
4. **Components** -- Every component with props, slots, events
5. **User Flows** -- Every user journey step-by-step
6. **Tauri Commands** -- Every IPC call with parameters and return types
7. **Store State** -- Pinia store shape with every field
8. **Screen States** -- Loading, empty, error, normal for every screen
9. **Animations & Sounds** -- Every motion and audio event

Types reference: `02_types_and_ipc.md`. Component paths reference: `frontend_implementation_plan.md`.

---

# DOMAIN 1: IDENTITY & AUTH

## 1.1 Domain Overview

The Identity & Auth domain is the entry gate to the entire application. It manages account creation, PIN-based authentication, profile switching (Netflix-style), role-based routing into one of three shells (student/parent/admin), account management, and PIN reset flows. This domain uses the `auth` layout -- a full-screen, neutral, sidebar-less surface. Once authenticated, users are routed to their role-specific shell and never see the auth layout again until they switch profiles or lock the app.

**Entry points**: App launch, profile switch, session lock, deep-link without active session.

---

## 1.2 Features

### F1.1 Profile Switcher
- Netflix-style grid of profile tiles showing all accounts on this device
- Each tile displays: avatar (initials-based with color), display name, account type badge, last active timestamp, progress ring (overall readiness for students)
- "Add Account" tile always visible as last item in grid
- Maximum 8 accounts per device
- Tiles sorted by last active, most recent first
- First-run state: only "Add Account" tile visible with welcome message
- Clicking a tile navigates to PIN entry for that account
- Long-press/right-click on tile opens context menu: Edit, Delete (with confirmation)

### F1.2 PIN Entry
- 4-6 digit numeric PIN for students, 6+ digit for parents/admins
- PIN pad with digits 0-9, backspace, and clear
- Dots show entered digit count (filled vs empty circles)
- Wrong PIN: shake animation on dots, error message, remaining attempts display
- Lockout after 5 failed attempts: 15-minute lockout with countdown timer
- Parent accounts can unlock student accounts that are locked out
- PIN entry auto-submits when correct digit count reached
- "Forgot PIN?" link routes to PIN reset flow

### F1.3 Account Creation
- Multi-step wizard: Account Type > Display Name > PIN Setup > Confirm PIN > Done
- Account type selection: Student, Parent (each with description and icon)
- Admin accounts created only through special flow (not exposed in normal UI)
- Display name: 2-30 characters, alphanumeric + spaces
- PIN setup: visual strength indicator, minimum 4 digits for student, 6 for parent
- PIN confirm: must match exactly
- Success screen with role-specific welcome message and "Get Started" CTA
- Student first-run flag set to `true` -- triggers onboarding on first login

### F1.4 Role Routing
- On successful auth, read `account.accountType` to determine target layout
- `student` -> `/student/` (student layout with Coach Hub)
- `parent` -> `/parent/` (parent layout with Family Overview)
- `admin` -> `/admin/` (admin layout with Command Center)
- If `account.firstRun === true` for students, redirect to `/student/onboarding/welcome`
- Parent first-run: redirect to `/parent/` with onboarding banner
- Preserve intended route if deep-linked (store in query param, redirect after auth)

### F1.5 Account Management
- Accessible from settings in any portal
- Edit display name
- Change PIN (requires current PIN verification first)
- View account info: type, tier (standard/premium/elite), created date
- Delete account (requires PIN confirmation + "type DELETE to confirm")
- Parent: can view and manage linked student accounts
- Parent: can reset student PINs without knowing current PIN
- Parent: can unlock locked-out student accounts

### F1.6 PIN Reset
- "Forgot PIN?" from PIN entry screen
- For students: requires parent PIN to authorize reset
- For parents: requires security confirmation (re-enter display name + creation date)
- For admins: requires super-admin key
- Reset flow: Verify identity > Enter new PIN > Confirm new PIN > Success
- Failed PIN count resets to 0 after successful reset

---

## 1.3 Screens

### Screen: Profile Switcher (`pages/index.vue`)
**Layout**: `auth` (full-screen, centered, no sidebar)
**Route**: `/`

**Sections**:
| Section | Content | Position |
|---------|---------|----------|
| App Header | eCoach logo + tagline "Your Academic Intelligence Coach" | Top center, 20% from top |
| Profile Grid | Grid of ProfileTile components (2-4 columns responsive) | Center |
| Add Account | Special tile with + icon and "Add Account" label | Last position in grid |
| Version | App version number | Bottom center, muted text |

**Data Sources**:
- `listAccounts()` -> `AccountSummary[]`
- `getLearnerTruthSnapshot(accountId)` -> for progress ring on student tiles (lazy-loaded)

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | `listAccounts()` pending | 3-4 skeleton tiles pulsing |
| Empty (first run) | `accounts.length === 0` | Welcome message + single "Create Your First Account" card |
| Normal | `accounts.length >= 1` | Grid of profile tiles + Add Account tile |
| Error | `listAccounts()` failed | AppError with "Could not load accounts" + retry button |
| Full | `accounts.length >= 8` | Grid without Add Account tile, subtle "Maximum accounts reached" note |

---

### Screen: PIN Entry (`pages/pin.vue`)
**Layout**: `auth`
**Route**: `/pin?accountId=<id>`

**Sections**:
| Section | Content | Position |
|---------|---------|----------|
| Back Button | Arrow-left icon, returns to profile switcher | Top left |
| Profile Info | Avatar + display name of selected account | Top center |
| PIN Dots | Row of empty/filled circles (4-6 based on account type) | Center |
| PIN Pad | 3x4 numeric grid (1-9, backspace, 0, clear) | Below dots |
| Forgot PIN | "Forgot PIN?" text link | Below pad |
| Lockout Banner | Countdown timer when locked | Replaces PIN pad area |

**Data Sources**:
- `authenticate(accountId, pin)` -> `AuthResult`
- Account info from route query param (pre-fetched from profile switcher)

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Normal | Default | PIN pad active, dots empty |
| Entering | Digits entered, not yet submitted | Filled dots up to entered count |
| Authenticating | PIN submitted, awaiting response | Brief spinner on dots, pad disabled |
| Success | `authResult.success === true` | Green check animation on dots, 400ms delay, then route to portal |
| Failed | `authResult.success === false` | Shake animation on dots, red flash, error text "Incorrect PIN", remaining attempts shown |
| Locked | `authResult.lockedUntil !== null` | Lockout banner with countdown, pad disabled, "Ask a parent to unlock" (for students) |
| Error | IPC call failed | AppError with retry |

---

### Screen: Account Creation (`pages/create-account.vue`)
**Layout**: `auth`
**Route**: `/create-account`

**Sections (multi-step wizard)**:

**Step 1 -- Account Type**:
| Section | Content |
|---------|---------|
| Title | "Who is this account for?" |
| Options | Two large cards: Student (with backpack icon, "I'm a learner preparing for exams") and Parent (with shield icon, "I want to track my child's progress") |
| Back | Return to profile switcher |

**Step 2 -- Display Name**:
| Section | Content |
|---------|---------|
| Title | "What should we call you?" |
| Input | Text field with character count (2-30), live validation |
| Avatar Preview | Generated avatar from initials, updates live as name typed |

**Step 3 -- PIN Setup**:
| Section | Content |
|---------|---------|
| Title | "Create a PIN to protect your account" |
| PIN Dots | Empty circles (4 for student, 6 for parent) |
| PIN Pad | Numeric keypad |
| Strength | Visual indicator (weak/ok/strong based on digit variety) |
| Hint | "Use a PIN you'll remember but others can't guess" |

**Step 4 -- Confirm PIN**:
| Section | Content |
|---------|---------|
| Title | "Enter your PIN again to confirm" |
| PIN Dots + Pad | Same as step 3 |
| Mismatch Error | "PINs don't match. Try again." with clear and retry |

**Step 5 -- Success**:
| Section | Content |
|---------|---------|
| Icon | Animated checkmark (green) |
| Title | "Welcome, {name}!" |
| Subtitle | Role-specific: "Your coaching journey starts now" (student) / "You're all set to track progress" (parent) |
| CTA | "Get Started" button -> auto-authenticate and route to portal |

**Data Sources**:
- `createAccount(input: CreateAccountInput)` -> `Account`
- `authenticate(accountId, pin)` -> `AuthResult` (auto-login after creation)

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Normal | Each step active | Step content with stepper indicator (5 dots) |
| Creating | `createAccount()` pending | Loading spinner on "Create" button, all inputs disabled |
| Created | Account returned successfully | Transition to success step |
| Error | Creation failed (e.g., name taken on device) | Inline error on relevant step |

---

### Screen: Account Management (embedded in Settings)
**Location**: `/student/settings`, `/parent/settings`, `/admin/settings` -- Account section
**Layout**: Role-specific layout (student/parent/admin)

**Sections**:
| Section | Content |
|---------|---------|
| Profile Card | Avatar, display name (editable), account type, tier badge, member since |
| PIN Section | "Change PIN" button -> opens Change PIN modal |
| Linked Accounts | (Parent only) List of linked student accounts with unlock/reset buttons |
| Danger Zone | "Delete Account" button with red styling |

---

### Screen: PIN Reset (`pages/pin-reset.vue`)
**Layout**: `auth`
**Route**: `/pin-reset?accountId=<id>`

**Sections (multi-step)**:

**Step 1 -- Identity Verification**:
| Section | Content |
|---------|---------|
| Title | "Reset your PIN" |
| Method (student) | "Enter a parent PIN to authorize this reset" + PIN pad |
| Method (parent) | "Confirm your identity" + display name field + account creation month/year dropdowns |

**Step 2 -- New PIN**:
Same as Account Creation Step 3

**Step 3 -- Confirm New PIN**:
Same as Account Creation Step 4

**Step 4 -- Success**:
"PIN has been reset. You can now sign in with your new PIN." + "Sign In" CTA

**Data Sources**:
- `authenticate(parentAccountId, parentPin)` -> verify parent identity for student reset
- `resetPin(accountId, newPin)` -> success/error
- `changePinWithParentAuth(studentAccountId, parentAccountId, parentPin, newPin)` -> for parent-authorized student reset

---

## 1.4 Components

### ProfileSwitcher (`components/layout/ProfileSwitcher.vue`)
**Props**: None (fetches own data)
**Emits**: `@select(accountId: number)`, `@add-account()`
**Slots**: None
**Behavior**: Calls `listAccounts()` on mount. Renders grid of ProfileTile components. Handles loading/empty/error internally.
**Internal state**: `accounts: AccountSummary[]`, `loading: boolean`, `error: string | null`

### ProfileTile (`components/layout/ProfileTile.vue`)
**Props**:
- `account: AccountSummary` (required)
- `readinessScore?: BasisPoints` (optional, for student progress ring)
**Emits**: `@click()`, `@context-menu(action: 'edit' | 'delete')`
**Display**:
- Avatar circle (colored by account type, initials from display name)
- Display name (truncated at 20 chars)
- Account type badge (student/parent)
- Last active relative time ("2 hours ago", "Yesterday")
- Progress ring around avatar (students only, if readinessScore provided)
- Status dot: green (active), gray (inactive >7 days), red (locked)

### PinPad (`components/layout/PinPad.vue`)
**Props**:
- `length: number` (4 or 6, default 4)
- `disabled: boolean` (default false)
- `error: boolean` (triggers shake animation)
- `success: boolean` (triggers check animation)
**Emits**: `@complete(pin: string)`, `@change(digits: number)`
**Display**:
- Row of `length` circle indicators (empty/filled)
- 4x3 grid: [1][2][3] / [4][5][6] / [7][8][9] / [backspace][0][clear]
- Buttons have hover/press states, ripple effect on tap
**Animations**:
- Dot fill: scale-in spring animation (100ms, ease-spring)
- Error shake: horizontal oscillation 3x over 400ms
- Success: all dots turn green, scale up slightly, checkmark appears center (600ms)

### OnboardingWizard (`components/layout/OnboardingWizard.vue`)
**Props**:
- `steps: number` (total steps)
- `currentStep: number`
- `canProceed: boolean`
**Emits**: `@next()`, `@back()`, `@complete()`
**Slots**: `default` (step content), `footer` (custom actions)
**Display**: Step indicator dots at top, content area, Back/Next buttons at bottom
**Animation**: Step transitions slide left-to-right (next) or right-to-left (back), 300ms ease-out

---

## 1.5 User Flows

### Flow: First-Time App Launch
1. App opens -> `/` (Profile Switcher)
2. Empty state: welcome message + "Create Your First Account" card
3. User taps "Create Account" -> `/create-account`
4. Step 1: Select "Student" -> Step 2: Enter name -> Step 3: Create PIN -> Step 4: Confirm PIN -> Step 5: Success
5. Tap "Get Started" -> auto-authenticate -> redirect to `/student/onboarding/welcome`
6. Student `firstRun` flag is `true`, onboarding begins (Domain 9)

### Flow: Returning User Login
1. App opens -> `/` (Profile Switcher)
2. Profile tiles shown, most recent first
3. User taps their tile -> `/pin?accountId=5`
4. Enter PIN digits -> auto-submit on 4th/6th digit
5. Success animation (400ms) -> route to `/student/` (Coach Hub)

### Flow: Profile Switch (from within app)
1. User clicks profile avatar in sidebar header
2. Dropdown: "Switch Account" option
3. Tap -> navigate to `/` (Profile Switcher)
4. Current session state preserved in store (but not active)
5. Select different profile -> PIN entry -> route to that account's portal

### Flow: Wrong PIN -> Lockout
1. Enter incorrect PIN -> shake, "Incorrect PIN. 4 attempts remaining."
2. Enter incorrect PIN x4 more -> "Account locked for 15 minutes."
3. Lockout screen shows countdown timer
4. Student lockout: "Ask a parent to unlock your account" link
5. Parent opens their account -> Settings -> Linked Accounts -> "Unlock" button on locked student
6. Student returns to PIN entry, lockout cleared

### Flow: Parent Resets Student PIN
1. Parent logged in -> Settings -> Linked Accounts
2. Finds locked/forgotten student account -> "Reset PIN"
3. Confirm with parent's own PIN
4. Enter new student PIN -> Confirm -> Success
5. Student can now login with new PIN

### Flow: Change Own PIN
1. Settings -> Account -> "Change PIN"
2. Modal opens: "Enter current PIN" -> PIN pad
3. Correct: "Enter new PIN" -> PIN pad
4. "Confirm new PIN" -> PIN pad
5. Match: Success toast, modal closes
6. Mismatch: "PINs don't match" error, retry from step 3

---

## 1.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `list_accounts` | none | `AccountSummary[]` | Fetch all accounts on device |
| `get_account` | `accountId: number` | `Account` | Fetch single account details |
| `create_account` | `input: CreateAccountInput` | `Account` | Create new account |
| `authenticate` | `accountId: number, pin: string` | `AuthResult` | Verify PIN, return auth result |
| `update_account` | `accountId: number, input: UpdateAccountInput` | `Account` | Update display name or tier |
| `change_pin` | `accountId: number, input: ChangePinInput` | `boolean` | Change PIN (requires current PIN) |
| `reset_pin` | `accountId: number, newPin: string` | `boolean` | Force-reset PIN (admin/parent only) |
| `unlock_account` | `accountId: number` | `boolean` | Clear lockout on account |
| `delete_account` | `accountId: number` | `boolean` | Permanently delete account and all data |

---

## 1.7 Store State

### Auth Store (`stores/auth.ts`)

```typescript
interface AuthState {
  // Current authenticated account
  currentAccount: Account | null;
  currentRole: 'student' | 'parent' | 'admin' | null;
  isAuthenticated: boolean;

  // All accounts on device
  accounts: AccountSummary[];
  accountsLoaded: boolean;
  accountsLoading: boolean;
  accountsError: string | null;

  // PIN entry state
  selectedAccountId: number | null;
  pinAttemptCount: number;
  lockedUntil: string | null;
  pinError: string | null;
  remainingAttempts: number | null;

  // Account creation wizard state
  creationStep: number;
  creationInput: Partial<CreateAccountInput>;
}
```

**Actions**:
- `loadAccounts()` -- fetch all accounts, set `accounts`
- `selectAccount(id)` -- set `selectedAccountId`, navigate to PIN
- `submitPin(pin)` -- call `authenticate`, handle success/failure/lockout
- `createAccount(input)` -- call `create_account`, auto-login on success
- `switchAccount()` -- clear `currentAccount`, navigate to `/`
- `logout()` -- clear auth state, navigate to `/`
- `changePin(current, newPin)` -- call `change_pin`
- `resetStudentPin(studentId, parentPin, newPin)` -- parent-authorized reset
- `unlockAccount(accountId)` -- clear lockout

**Getters**:
- `isStudent` -- `currentRole === 'student'`
- `isParent` -- `currentRole === 'parent'`
- `isAdmin` -- `currentRole === 'admin'`
- `isFirstRun` -- `currentAccount?.firstRun === true`
- `isLocked` -- `lockedUntil !== null && new Date(lockedUntil) > new Date()`

---

## 1.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Profile tile appear | Stagger fade-in + scale from 0.95 to 1.0, 200ms each, 50ms stagger | None |
| Profile tile hover | Scale 1.0 -> 1.03, shadow elevation increase, 150ms ease-out | None |
| Profile tile click | Scale press 1.0 -> 0.97 -> 1.0, 100ms | Soft click (`feedback/tap.mp3`) |
| PIN digit entered | Dot scale 0 -> 1.2 -> 1.0, spring easing, 150ms | Subtle tick (`feedback/pin-tick.mp3`) |
| PIN incorrect | Dots shake horizontally (translate-x: 0 -> -8px -> 8px -> -4px -> 4px -> 0), 400ms | Error buzz (`feedback/wrong-soft.mp3`) |
| PIN correct | Dots all turn green, scale to 1.2, checkmark fades in center, 500ms | Success chime (`feedback/correct-soft.mp3`) |
| Wizard step transition | Current step slides out left, next slides in from right, 300ms ease-out | None |
| Account created | Confetti particle burst from checkmark icon, 1200ms | Celebration chime (`celebration/account-created.mp3`) |
| Lockout banner appear | Slide down from top, 400ms ease-out | Warning tone (`feedback/lockout.mp3`) |
| Delete account confirm | Red pulse on delete button before action, 300ms | None |

---
---

# DOMAIN 2: COACH BRAIN & HUB

## 2.1 Domain Overview

The Coach Hub is the student's home screen -- the single most important surface in the application. It answers "What should I do right now?" by rendering the output of the Coach Brain's `resolveCoachState()` and `resolveNextCoachAction()` commands. The hub must handle 14 distinct learner journey states, render 11 action types, display contextual insight cards, and provide the rescue dock for struggling students. It is the command center that routes students into sessions, diagnostics, reviews, and every other mode.

**Entry points**: Student login, sidebar "Home" tap, session complete redirect, back navigation from any student page.

---

## 2.2 Features

### F2.1 Coach Hub Home (14 Journey States)
Each of the 14 `LearnerJourneyState` values produces a completely different Coach Hub configuration:

| State | Hub Configuration |
|-------|-------------------|
| `onboarding_required` | Welcome card, "Let's get started" CTA, step indicator showing onboarding progress |
| `subject_selection_required` | Subject picker card, "Choose your subjects" prompt, subject grid with checkboxes |
| `content_readiness_required` | Content pack installer card, progress bar for pack downloads, list of required packs |
| `diagnostic_required` | Diagnostic launcher card, "Let's find out where you stand" messaging, estimated time |
| `plan_generation_required` | Loading/processing animation, "Building your personalized plan..." |
| `ready_for_today_mission` | **Primary state** -- Full hub with Today's Mission card, insight strip, readiness snapshot |
| `mission_in_progress` | Resume card with progress indicator, "Continue where you left off" CTA |
| `mission_review_required` | Review prompt card, completed mission summary, "Review your results" CTA |
| `repair_required` | Repair banner (amber), struggling topic highlight, "Let's fix this first" directive |
| `blocked_on_topic` | Blocker alert, prerequisite dependency display, alternate path suggestion |
| `plan_adjustment_required` | Adjustment notice, "Your plan needs updating" with reason, auto-adjust CTA |
| `review_day` | Review-day theme (calm blue), spaced repetition queue, memory rescue highlights |
| `exam_mode` | Exam countdown prominent, final prep checklist, mock/drill shortcuts, pressure theme |
| `stalled_no_content` | Empty state with explanation, content pack installation guide, admin contact suggestion |

### F2.2 Coach Action Types (11 Types)
Each `CoachActionType` maps to a specific card design and CTA:

| Action Type | Card Title | CTA Label | Route Target |
|-------------|-----------|-----------|--------------|
| `continue_onboarding` | "Let's continue setting up" | "Continue Setup" | `/student/onboarding/{step}` |
| `select_subjects` | "Choose your subjects" | "Select Subjects" | `/student/onboarding/subjects` |
| `resolve_content` | "Get your study materials" | "Install Packs" | `/student/onboarding/content-packs` |
| `start_diagnostic` | "Discover your starting point" | "Start Diagnostic" | `/student/diagnostic/` |
| `generate_plan` | "Building your plan..." | (auto-processing) | stays on hub |
| `start_today_mission` | Today's mission title + reason | "Start Session" | `/student/session/{id}` |
| `resume_mission` | "Pick up where you left off" | "Resume" | `/student/session/{id}` |
| `review_results` | "See what you accomplished" | "Review" | `/student/session/debrief/{id}` |
| `start_repair` | "Let's strengthen {topic}" | "Begin Repair" | `/student/session/{id}` |
| `adjust_plan` | "Your plan needs a tune-up" | "Update Plan" | (triggers plan regeneration) |
| `view_overview` | "Here's your progress" | "View Progress" | `/student/progress/` |

### F2.3 Coach Directive Cards
- Render `CoachNextAction` as the primary hero card on the hub
- Card contains: icon (mapped from actionType), title, subtitle, estimated minutes badge, CTA button
- Card uses gradient background appropriate to urgency/mood
- Secondary actions shown as smaller cards below primary

### F2.4 Insight Cards
- Contextual intelligence surfaced from `TopicCase` data
- Types: mastery milestone reached, topic slipping, memory decay warning, error pattern detected, streak broken, new content available
- Each card: icon, title, one-line summary, tap to expand with detail + action button
- Maximum 5 insight cards visible, priority-sorted by `priorityScore`
- Dismissible with swipe-left or X button (persists dismissal for 24 hours)

### F2.5 Rescue Dock (7 Buttons)
Always-accessible help bar for struggling students:
| Button | Icon | Action |
|--------|------|--------|
| Simplify | puzzle-piece | Simplify current task to easier variant |
| Hint | lightbulb | Show progressive hint (up to 3 levels) |
| First Step | footprints | Show only the first step of the solution |
| Compare | columns | Show side-by-side comparison of similar concepts |
| Explain | book-open | Open explanation panel at current depth |
| Listen | headphones | Play audio explanation |
| Example | clipboard-check | Show a worked example of similar problem |

- Dock is collapsible (chevron toggle)
- Dock appears as bottom-anchored bar during sessions
- On Coach Hub, dock is hidden (only surfaces during active questions)
- Each button has tooltip on hover, icon + short label

### F2.6 Coach Voice
- The "voice" of the coach is a styled text element that appears throughout the hub
- Three tone registers: Supportive (default), Urgent (repair/blocked), Celebrating (milestones)
- Voice component renders a speech-bubble-style card with coach icon
- Copy is generated from `CoachNextAction.subtitle` and `TopicCase.recommendedIntervention.reason`
- Font style: slightly larger, warm color, italic for emphasis phrases

### F2.7 Recovery Banner
- Appears when student returns after absence (>3 days since last session)
- Gentle amber-toned banner at top of hub
- Message: "Welcome back, {name}. It's been {days} days. Let's ease back in."
- CTA: "Start Recovery Session" (reduced difficulty, shorter duration)
- Dismissible after viewing, but re-appears if no session started within 2 hours

### F2.8 Exam Countdown
- Persistent widget showing days until next exam
- Three visual modes based on distance:
  - >30 days: calm blue badge, "42 days to go"
  - 7-30 days: amber badge, "18 days -- building momentum"
  - <7 days: red pulsing badge, "3 days -- final push"
- Tap to expand: exam name, date, readiness score, topic coverage percentage
- Displayed in hub header area, always visible

### F2.9 Today's Mission
- The primary actionable card when state is `ready_for_today_mission`
- Displays: mission title, subject chip, primary topic, reason ("This topic is slipping"), estimated minutes, difficulty indicator
- Sources from `CoachNextAction` with `actionType === 'start_today_mission'`
- Large, prominent card with gradient background matching subject color
- "Start Session" CTA button, pulsing gently to attract attention

### F2.10 Phase Indicator
- Shows current learning phase from the journey plan
- Five phases: Stabilize, Build, Strengthen, Condition, Ready
- Visual: pill-shaped badge with phase icon, name, and progress within phase
- Color-coded: each phase has its own accent color
- Tap to expand: phase description, goals, estimated completion date

### F2.11 "Why This?" Explainer
- Small "Why this?" link on every recommended action card
- Tap opens a slide-up panel with plain-language explanation
- Content drawn from `CoachNextAction.context` and `TopicCase` data
- Example: "This topic dropped from Stable to Fragile 3 days ago. Two recent answers showed the same conceptual confusion. Repairing this now prevents it from blocking Algebra next week."
- Close with tap outside or X button

---

## 2.3 Screens

### Screen: Coach Hub Home (`pages/student/index.vue`)
**Layout**: `student`
**Route**: `/student/`

**Sections** (top to bottom, for `ready_for_today_mission` state):
| Section | Content | Data Source |
|---------|---------|-------------|
| Recovery Banner | (conditional) Welcome-back message | Last session timestamp comparison |
| Exam Countdown | Days to exam + readiness band | `CoachPlan.examDate` + `LearnerTruthSnapshot.overallReadinessBand` |
| Phase Indicator | Current learning phase badge | `CoachPlan.currentPhase` |
| Coach Voice | Coach greeting/direction message | `CoachNextAction.subtitle` |
| Today's Mission Card | Primary action card (hero) | `resolveNextCoachAction()` -> `CoachNextAction` |
| Secondary Actions | Smaller action cards (up to 3) | Additional `CoachNextAction` items |
| Insight Strip | Horizontal scrollable insight cards (up to 5) | `listPriorityTopicCases()` -> `TopicCase[]` |
| Quick Stats | Readiness score, topics mastered, streak count | `LearnerTruthSnapshot` |
| "Why This?" Panel | (slide-up, on demand) | `CoachNextAction.context` + `TopicCase` |

**Data Sources**:
- `resolveCoachState(studentId)` -> `CoachStateResolution`
- `resolveNextCoachAction(studentId)` -> `CoachNextAction`
- `listPriorityTopicCases(studentId, limit: 5)` -> `TopicCase[]`
- `getLearnerTruthSnapshot(studentId)` -> `LearnerTruthSnapshot`
- `getCoachPlan(studentId)` -> `CoachPlan` (for exam date, phase)
- `getActiveSession(studentId)` -> `Session | null` (for resume detection)

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Any primary data fetch pending | Skeleton: large card placeholder + 3 small card placeholders + stat bar skeleton |
| Onboarding Required | `state === 'onboarding_required'` | Single large onboarding card, no insights/stats |
| Subject Selection | `state === 'subject_selection_required'` | Subject selection card with grid |
| Content Required | `state === 'content_readiness_required'` | Content pack installation card |
| Diagnostic Required | `state === 'diagnostic_required'` | Diagnostic launcher card, estimated time |
| Plan Generating | `state === 'plan_generation_required'` | Processing animation with "Building your plan..." |
| Ready (primary) | `state === 'ready_for_today_mission'` | Full hub layout as described above |
| Mission Active | `state === 'mission_in_progress'` | Resume card (prominent), mission progress indicator |
| Review Needed | `state === 'mission_review_required'` | Review prompt with session summary |
| Repair Mode | `state === 'repair_required'` | Amber-tinted hub, repair-focused mission card |
| Blocked | `state === 'blocked_on_topic'` | Blocker alert, prerequisite display, alternate path |
| Plan Adjust | `state === 'plan_adjustment_required'` | Plan update notice with reason and CTA |
| Review Day | `state === 'review_day'` | Calm blue theme, review queue, memory highlights |
| Exam Mode | `state === 'exam_mode'` | Red/dark urgent theme, countdown prominent, prep checklist |
| Stalled | `state === 'stalled_no_content'` | Empty state, content installation guide |
| Error | Primary fetch failed | AppError with "Could not load your coaching dashboard" + retry |

---

## 2.4 Components

### CoachHub (`components/coach/CoachHub.vue`)
**Props**: `studentId: number`
**Internal**: Orchestrator that calls all data sources, determines journey state, renders appropriate layout configuration.
**Template**: Conditional rendering blocks for each of the 14 journey states.

### CoachDirectiveCard (`components/coach/CoachDirectiveCard.vue`)
**Props**:
- `action: CoachNextAction` (required)
- `variant: 'hero' | 'secondary'` (default 'hero')
- `showWhyThis: boolean` (default true)
**Emits**: `@execute()`, `@why-this()`
**Display**:
- Hero variant: full-width card, gradient background (mapped from actionType), large icon, title (text-2xl), subtitle, estimated minutes badge, CTA button
- Secondary variant: compact card, icon left, title + subtitle, CTA right
- "Why this?" link in bottom-left corner

### InsightCard (`components/coach/InsightCard.vue`)
**Props**:
- `topicCase: TopicCase` (required)
- `insightType: 'milestone' | 'slipping' | 'decay' | 'error_pattern' | 'streak' | 'new_content'`
**Emits**: `@action(actionType: string)`, `@dismiss()`
**Display**:
- Compact card (180px wide for horizontal scroll)
- Top: colored accent bar (green for milestone, amber for slipping, red for decay)
- Icon + title (1 line)
- Summary (2 lines max)
- Action chip at bottom
- X button top-right for dismiss

### CoachVoice (`components/coach/CoachVoice.vue`)
**Props**:
- `message: string` (required)
- `tone: 'supportive' | 'urgent' | 'celebrating'` (default 'supportive')
**Display**:
- Speech-bubble card with tail pointing to coach icon
- Coach icon: small circle avatar with graduation-cap icon
- Tone styles: supportive (warm gray bg, normal weight), urgent (amber bg, semibold), celebrating (green bg, with confetti dots)

### RescueDock (`components/coach/RescueDock.vue`)
**Props**:
- `visible: boolean` (default false)
- `enabledButtons: string[]` (subset of 7 button keys, context-dependent)
**Emits**: `@action(button: 'simplify' | 'hint' | 'first_step' | 'compare' | 'explain' | 'listen' | 'example')`
**Display**:
- Fixed bottom bar (60px height), above session controls
- 7 icon buttons in a row with labels below
- Disabled buttons grayed out
- Collapse toggle (chevron-up/down) at left edge
- Collapsed state: thin 4px colored line at bottom with expand handle

### ExamCountdown (`components/coach/ExamCountdown.vue`)
**Props**:
- `examDate: string` (ISO date)
- `examName: string`
- `readinessBand: string`
- `readinessScore: BasisPoints`
**Emits**: `@expand()`
**Display**:
- Compact: pill badge with days count, colored by distance
- Expanded (tap): card with exam name, date, readiness gauge, coverage bar

### TodaysMission (`components/coach/TodaysMission.vue`)
**Props**:
- `action: CoachNextAction` (required)
- `subjectColor: string` (hex color for gradient)
**Emits**: `@start()`
**Display**: Large card (takes 60% of hub width), subject-colored gradient, mission title large, topic name, reason text, minutes badge, difficulty dots, large "Start Session" button

### PhaseIndicator (`components/coach/PhaseIndicator.vue`)
**Props**:
- `phase: string` (current phase name)
- `phaseProgress: BasisPoints` (progress within phase)
- `totalPhases: number` (default 5)
- `currentPhaseIndex: number`
**Display**:
- Pill badge: phase icon + phase name + mini progress bar inside pill
- 5 phase dots below (filled up to current phase)
- Tap to expand: phase description, goals, estimated completion

### RecoveryBanner (`components/coach/RecoveryBanner.vue`)
**Props**:
- `daysSinceLastSession: number`
- `studentName: string`
**Emits**: `@start-recovery()`, `@dismiss()`
**Display**:
- Full-width banner, amber background, gentle gradient
- Coach icon + warm message
- "Start Recovery Session" button + "Maybe later" dismiss link

### WhyThisCard (`components/coach/WhyThisCard.vue`)
**Props**:
- `explanation: string` (plain-language reason)
- `topicCase: TopicCase | null` (optional, for detailed evidence)
**Display**:
- Slide-up bottom sheet (60% screen height)
- Title: "Why this session?"
- Explanation paragraph
- If topicCase provided: mastery score, gap score, recent accuracy, last seen date, active hypotheses list
- Close button (X) top-right

### CoachStateIndicator (`components/coach/CoachStateIndicator.vue`)
**Props**:
- `posture: 'calm_guide' | 'teacher' | 'rescue' | 'confidence_repair' | 'performance_coach' | 'accountability'`
**Display**:
- Small badge in hub header: icon + label
- Colors: calm_guide (blue), teacher (indigo), rescue (amber), confidence_repair (warm pink), performance_coach (green), accountability (slate)

---

## 2.5 User Flows

### Flow: Student Arrives at Coach Hub (Normal Day)
1. Login -> route to `/student/`
2. Hub calls `resolveCoachState(studentId)` -> returns `ready_for_today_mission`
3. Hub calls `resolveNextCoachAction(studentId)` -> returns primary mission
4. Hub calls `listPriorityTopicCases(studentId, 5)` -> returns insight data
5. Hub calls `getLearnerTruthSnapshot(studentId)` -> returns stats
6. Hub renders: Exam Countdown (header), Phase Indicator, Coach Voice greeting, Today's Mission card, Insight strip, Quick Stats
7. Student reads coach message, reviews mission
8. Taps "Why this?" -> bottom sheet explains reasoning
9. Closes "Why this?" -> taps "Start Session" -> navigates to session player

### Flow: Student Returns After Absence
1. Login -> route to `/student/`
2. Hub detects `daysSinceLastSession > 3`
3. Recovery Banner slides in at top
4. Coach state may be `repair_required` or `plan_adjustment_required`
5. Coach Voice: "Welcome back. Let's take it easy and rebuild."
6. Today's Mission is a recovery session (shorter, easier)
7. Student taps "Start Recovery Session" from banner or mission card

### Flow: Student in Exam Mode
1. Exam date < 7 days away
2. Coach state: `exam_mode`
3. Hub theme shifts to urgent (darker, more focused)
4. Exam Countdown pulsing red
5. Coach Voice: "3 days left. Focus on your weakest areas."
6. Mission card shows targeted drill or mock exam
7. Quick shortcuts to Mock Centre and weak topics visible

### Flow: Rescue Dock Usage (During Session)
1. Student is in session, encounters difficult question
2. Rescue Dock visible at bottom of session screen
3. Student taps "Hint" -> first hint displayed inline below question
4. Still stuck -> taps "Hint" again -> second hint (more specific)
5. Still stuck -> taps "First Step" -> only the first step shown
6. Student attempts answer -> if wrong -> "Explain" becomes highlighted
7. Student taps "Explain" -> explanation panel opens with full walkthrough

---

## 2.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `resolve_coach_state` | `studentId: number` | `CoachStateResolution` | Determine current journey state |
| `resolve_next_coach_action` | `studentId: number` | `CoachNextAction` | Get primary recommended action |
| `assess_content_readiness` | `studentId: number` | `ContentReadinessResolution` | Check if content packs are sufficient |
| `build_topic_case` | `studentId: number, topicId: number` | `TopicCase` | Deep analysis of single topic |
| `list_priority_topic_cases` | `studentId: number, limit: number` | `TopicCase[]` | Top priority topics for insights |
| `get_coach_plan` | `studentId: number` | `CoachPlan` | Current learning plan |
| `get_coach_plan_day` | `studentId: number, date: string` | `CoachPlanDay` | Plan for specific day |
| `list_coach_missions` | `planDayId: number` | `CoachMission[]` | Missions for a plan day |
| `get_mission_memory` | `missionId: number` | `CoachMissionMemory` | Historical data for a mission |
| `get_active_session` | `studentId: number` | `Session \| null` | Check for in-progress session |

---

## 2.7 Store State

### Coach Store (`stores/coach.ts`)

```typescript
interface CoachState {
  // Journey state
  journeyState: LearnerJourneyState | null;
  journeyStateReason: string | null;
  journeyStateLoading: boolean;
  journeyStateError: string | null;

  // Next action
  nextAction: CoachNextAction | null;
  nextActionLoading: boolean;
  nextActionError: string | null;

  // Content readiness
  contentReadiness: ContentReadinessResolution | null;

  // Topic cases (insights)
  priorityTopicCases: TopicCase[];
  topicCasesLoading: boolean;

  // Coach plan
  currentPlan: CoachPlan | null;
  todayPlanDay: CoachPlanDay | null;
  todayMissions: CoachMission[];

  // Active directives (queue of coach communications)
  activeDirectives: CoachDirective[];

  // Recovery state
  daysSinceLastSession: number;
  showRecoveryBanner: boolean;

  // Coach posture
  coachPosture: 'calm_guide' | 'teacher' | 'rescue' | 'confidence_repair' | 'performance_coach' | 'accountability';

  // Dismissed insight IDs (persisted for 24h)
  dismissedInsightIds: Set<number>;
}
```

---

## 2.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Hub initial load | Cards stagger fade-in from bottom (translate-y: 20px -> 0), 200ms each, 80ms stagger | None |
| Coach Voice appear | Slide in from left + fade, speech bubble tail grows, 400ms ease-out | Soft chime (`feedback/coach-voice.mp3`) |
| Today's Mission CTA pulse | Gentle scale pulse 1.0 -> 1.02 -> 1.0, looping every 3s, ease-in-out | None |
| Insight card scroll | Momentum-based horizontal scroll with snap points | None |
| Insight card dismiss | Swipe left, card slides out + fades, remaining cards shift left, 300ms | None |
| Exam Countdown pulse (<7 days) | Border glow pulse red, 2s cycle, ease-in-out | None |
| Recovery Banner enter | Slide down from top, 500ms ease-spring | Warm welcome chime (`transitions/recovery-enter.mp3`) |
| Phase Indicator update | Old phase fades, new phase scales in, dot fills, 600ms | Phase-change tone (`transitions/phase-change.mp3`) |
| Why This panel open | Slide up from bottom, backdrop fade, 400ms ease-out | None |
| Journey state transition | Entire hub cross-fades between state layouts, 600ms | Transition whoosh (`transitions/state-change.mp3`) |
| Rescue Dock expand | Slide up from bottom edge, buttons stagger appear, 300ms + 50ms stagger | Dock open sound (`feedback/dock-open.mp3`) |
| Rescue Dock button tap | Button scale press + color flash, 150ms | Soft tap (`feedback/tap.mp3`) |

---
---

# DOMAIN 3: QUESTION ENGINE

## 3.1 Domain Overview

The Question Engine is the most reused, most complex component system in the application. It powers every question interaction across sessions, diagnostics, mocks, games, Beat Yesterday, Elite mode, and more. The engine consists of a `QuestionCard` orchestrator that routes to one of 15 format-specific renderers, a timer system with 6 variants, confidence capture, feedback display, and the wrong answer review/mistake clinic subsystems. All math content renders through KaTeX.

**Entry points**: SessionPlayer, DiagnosticPhasePlayer, MockExamHall, GameQuestion, BeatYesterday blocks, EliteSession, CustomTest.

---

## 3.2 Features

### F3.1 QuestionCard Orchestrator
- Receives a `Question` object + `QuestionOption[]` + context config
- Reads `question.questionFormat` to select the appropriate format renderer
- Manages the answer lifecycle: unanswered -> selected -> submitted -> feedback
- Handles timer integration (receives timer config from parent session)
- Handles confidence capture (pre-submit or post-submit based on config)
- Handles flag-for-review toggle
- Passes answer selection to parent via events
- Supports read-only mode (for review screens)

### F3.2 Format Renderers (15 Types)

#### F3.2.1 MCQ (`McqQuestion.vue`)
- 4 options (A/B/C/D) displayed as tappable cards
- Single selection: tap to select, tap again to deselect, tap another to switch
- Selected state: blue border, filled radio indicator
- Submit button activates when selection made
- Feedback: correct option highlighted green, incorrect red, unselected options dimmed
- Option shuffling (randomized order per session, stored for consistency)

#### F3.2.2 Short Answer (`ShortAnswerQuestion.vue`)
- Single-line text input with character limit indicator
- Case-insensitive matching by default
- Auto-trim whitespace
- Submit button activates when input non-empty
- Feedback: show correct answer below if wrong, with "close match" detection for typos

#### F3.2.3 Drag Reorder (`DragReorderQuestion.vue`)
- List of items that must be dragged into correct order
- Drag handle on each item (hamburger icon)
- Smooth drag animation with placeholder drop zone
- Touch and mouse support
- Items snap to positions
- Numbered slots on the left (1, 2, 3...)
- Submit: compare order to correct sequence
- Feedback: correct items in place highlighted green, misplaced items red with arrows showing correct position

#### F3.2.4 Matching (`MatchingQuestion.vue`)
- Two columns: left (prompts) and right (responses)
- Draw lines between matching pairs by clicking left then right item
- Connected pairs shown with colored lines
- Click a connected item to disconnect
- All pairs must be matched before submit enabled
- Feedback: correct matches green lines, incorrect red lines, correct answers revealed

#### F3.2.5 Fill Blank (`FillBlankQuestion.vue`)
- Sentence/paragraph with one or more blanks (indicated by `___` in stem)
- Each blank rendered as an inline text input
- Tab between blanks
- Auto-sizing input width based on expected answer length
- Submit when all blanks filled
- Feedback: correct blanks turn green, incorrect turn red with correct answer shown below

#### F3.2.6 Diagram Label (`DiagramLabelQuestion.vue`)
- Image/SVG diagram displayed with numbered hotspot markers
- Each hotspot has a text input or dropdown for the label
- Hotspots connected to their markers by lines
- Zoom/pan on diagram (pinch or scroll)
- Submit when all labels filled
- Feedback: correct labels green, incorrect red with correct labels shown

#### F3.2.7 Comparison Table (`ComparisonTableQuestion.vue`)
- Table with row headers and column headers pre-filled
- Student fills in cells (text inputs)
- Some cells may be pre-filled as hints
- Auto-tab between cells (left-to-right, top-to-bottom)
- Submit when all empty cells filled
- Feedback: correct cells green, incorrect red with correct values shown

#### F3.2.8 Step-by-Step (`StepByStepQuestion.vue`)
- Multi-step solution input where each step has its own text area
- "Add Step" button to add more steps
- Step numbering (Step 1, Step 2, ...)
- KaTeX rendering in step inputs (live preview)
- Reorder steps via drag
- Submit all steps
- Feedback: each step evaluated independently, partial credit display

#### F3.2.9 Classification (`ClassificationQuestion.vue`)
- Items listed at top, category bins below
- Drag items into correct category bins
- Multiple items per bin allowed
- Items display as chips/cards
- Bins show count of items inside
- Submit when all items placed
- Feedback: correctly placed items green, misplaced items red with correct bin indicated

#### F3.2.10 Sequencing (`SequencingQuestion.vue`)
- Similar to Drag Reorder but specifically for process/sequence ordering
- Timeline/arrow visual between positions (items connect with arrows)
- Horizontal or vertical layout option
- Items start in shuffled pool, drag onto timeline positions
- Submit when all positions filled
- Feedback: correct positions green, arrows between correct adjacencies green

#### F3.2.11 True/False (`TrueFalseQuestion.vue`)
- Statement displayed prominently
- Two large buttons: "True" (green-tinted) and "False" (red-tinted)
- Single tap to select (not submit)
- Separate submit button for consistency
- Trap logic: some statements are deliberately tricky (indicated in question metadata)
- Feedback: correct answer highlighted, explanation of why true/false

#### F3.2.12 Essay (`EssayQuestion.vue`)
- Large text area (minimum 200px height, expandable)
- Word count display (current/target range)
- Basic formatting toolbar (bold, italic, bullet list) -- optional
- Auto-save every 30 seconds
- Character limit indicator if applicable
- Submit button with word count validation
- Feedback: model answer displayed side-by-side, key points checklist

#### F3.2.13 Equation Builder (`EquationBuilderQuestion.vue`)
- Math-aware input field with symbol palette
- Symbol palette: numbers, basic operators (+, -, x, /), fractions, exponents, roots, parentheses, Greek letters, equals, inequalities
- Live KaTeX preview of built equation
- Palette organized in tabs: Basic, Algebra, Geometry, Calculus
- Input via palette tap or keyboard shortcuts
- Submit the equation
- Feedback: correct equation displayed, step-by-step if available

#### F3.2.14 Canvas Draw (`CanvasDrawQuestion.vue`)
- HTML5 Canvas drawing surface
- Tool palette: pencil, line, circle, rectangle, text, eraser
- Color picker (limited palette: black, red, blue, green)
- Line thickness selector (3 options)
- Undo/redo buttons
- Clear canvas button
- Background image support (e.g., graph paper, coordinate axes)
- Submit captures canvas as image data
- Feedback: model answer overlaid on student drawing with transparency

#### F3.2.15 First Step (`FirstStepQuestion.vue`)
- Shows a problem and asks "What is the first step?"
- 4 options (MCQ-style) describing possible first steps
- Unique styling: step-numbered options (Step 1a, 1b, 1c, 1d)
- Emphasis on the "first" -- visually highlight "FIRST step" in prompt
- Feedback: correct first step explained, full solution path shown progressively

### F3.3 Timer System (6 Variants)

| Variant | Visual | Behavior | Use Context |
|---------|--------|----------|-------------|
| Soft Timer | Thin progress bar, muted color, no numbers | Counts up, informational only | Practice sessions |
| Strict Countdown | Bold countdown numbers, progress bar depleting | Counts down, auto-submit on zero | Custom tests, mocks |
| Shrinking Timer | Timer bar that physically shrinks in width over time | Visual urgency builds as bar disappears | Beat Yesterday Speed Burst |
| Burst Timer | Large centered countdown (60/30/15s), pulsing | Short intense countdown, dramatic | Speed challenges |
| Pressure Timer | Dark background, red-tinted, heartbeat animation | Strict countdown with pressure atmosphere | Pressure training, Elite |
| Cluster Timer | Timer for a group of questions (e.g., "5 questions in 3 min") | Shared timer across question cluster | Diagnostic speed phase |

Each timer component:
- Props: `durationSeconds`, `variant`, `showNumbers`, `onExpire callback`
- Emits: `@tick(remaining)`, `@warning(threshold)`, `@expire()`
- Warning thresholds: 50%, 25%, 10% remaining -> visual change (color shift)
- Pause/resume support (for session pause)

### F3.4 Confidence Capture
- Three-option selector: "Sure" (green), "Not Sure" (amber), "Guessed" (red)
- Appears after answer selection, before or after submit (configurable)
- Compact: three pill buttons in a row
- Selected state: filled background, check icon
- Required in diagnostic and mock modes, optional in practice
- Data flows into `AnswerSubmission.confidenceLevel`

### F3.5 Feedback Display
- Correct answer: green banner "Correct!" with checkmark icon, celebration micro-animation
- Incorrect answer: red banner "Not quite" with X icon
- Partial credit: amber banner "Partially correct" (for multi-part questions)
- Feedback shows: your answer, correct answer, brief explanation
- "See Full Explanation" link expands to multi-layer explanation panel
- Explanation panel has progressive depth: Quick (1 sentence) > Simple (paragraph) > Detailed (full walkthrough with examples)
- KaTeX renders inline math in all explanation text

### F3.6 Wrong Answer Review (10-Part Progressive)
Full specification in Domain 7, but the component lives in the question engine:

| Part | Title | Content |
|------|-------|---------|
| 1 | What You Chose | Display student's selected answer with highlighting |
| 2 | Why It Looked Right | Explain the appeal of the wrong answer |
| 3 | Why It's Wrong | Precise explanation of the error |
| 4 | The Correct Answer | Display + explain the right answer |
| 5 | Why Others Are Wrong | Brief on each remaining option |
| 6 | The Mistake Type | Error type classification with icon |
| 7 | What Your Brain Did | Speculative thought reconstruction |
| 8 | The Lesson | One-sentence takeaway |
| 9 | Pattern Check | "You've made this type of mistake X times" |
| 10 | Repair Action | Specific next step to fix this gap |

- Parts revealed progressively (tap "Show More" or auto-expand on scroll)
- Each part is a collapsible section
- Part 9 and 10 link to the Academic Analyst / Mistake Lab

### F3.7 Mistake Clinic (5-Step)
Interactive coaching flow for wrong answers:

| Step | Title | Student Action |
|------|-------|---------------|
| 1 | What Happened | Review the question and your answer. "Does this look right?" Yes/No |
| 2 | Why Was It Tempting | Student selects or types why they chose the wrong answer (self-report chips) |
| 3 | What Clue Did You Miss | Highlight the key information in the question stem that should have guided them |
| 4 | The Repair Move | Worked example showing correct approach step-by-step |
| 5 | Confirm Recovery | Micro-question testing the same concept (simpler variant). Must answer correctly to complete. |

- Step transitions: slide left-to-right
- Step 5 failure: loops back with additional scaffolding
- Completion: "recovery confirmed" green banner + topic state update

### F3.8 KaTeX Math Rendering
- `MathRenderer` component wraps KaTeX for inline and block math
- Detects `$...$` for inline math and `$$...$$` for display math in any text
- Renders in question stems, options, explanations, feedback, worked examples
- Error handling: if KaTeX parse fails, show raw LaTeX in monospace as fallback
- Supports: fractions, exponents, roots, matrices, Greek letters, operators, set notation, geometry symbols
- Font size matches surrounding text context

---

## 3.3 Screens

The Question Engine does not own standalone screens. Its components are embedded in:
- Session Player (`pages/student/session/[id].vue`)
- Diagnostic Player (`pages/student/diagnostic/[id].vue`)
- Mock Exam Hall (`pages/student/mock/hall/[id].vue`)
- Game question overlays
- Beat Yesterday climb blocks
- Elite session player

These host screens are documented in their respective domains. The Question Engine provides the reusable component layer.

---

## 3.4 Components

### QuestionCard (`components/question/QuestionCard.vue`)
**Props**:
- `question: Question` (required)
- `options: QuestionOption[]` (required for MCQ/TF types)
- `timerConfig: { variant: TimerVariant, durationSeconds: number } | null`
- `showConfidence: boolean` (default false)
- `showFlag: boolean` (default false)
- `readOnly: boolean` (default false, for review mode)
- `correctAnswer: string | null` (for review mode reveal)
- `feedbackMode: 'immediate' | 'deferred' | 'none'` (default 'immediate')
**Emits**:
- `@submit(answer: AnswerPayload)`
- `@flag-toggle(flagged: boolean)`
- `@confidence-select(level: 'sure' | 'not_sure' | 'guessed')`
- `@hint-request()`
- `@time-expire()`
**Slots**: `rescue-dock` (for injecting rescue dock below question)
**Behavior**: Routes to format renderer based on `question.questionFormat`, manages answer state, triggers timer, captures confidence.

### QuestionStem (`components/question/QuestionStem.vue`)
**Props**:
- `text: string` (required, may contain LaTeX)
- `image: string | null` (optional diagram URL)
- `imageAlt: string` (accessibility)
**Display**: Renders text with inline KaTeX math. If image present, shows above text with zoom-on-click.

### QuestionOption (`components/question/QuestionOption.vue`)
**Props**:
- `option: QuestionOption` (required)
- `selected: boolean`
- `disabled: boolean`
- `state: 'default' | 'correct' | 'incorrect' | 'dimmed'` (for feedback)
**Emits**: `@select()`
**Display**: Card with radio indicator, option label (A/B/C/D), option text (with KaTeX), background color based on state.

### QuestionTimer (`components/question/QuestionTimer.vue`)
**Props**:
- `durationSeconds: number` (required)
- `variant: 'soft' | 'strict' | 'shrinking' | 'burst' | 'pressure' | 'cluster'`
- `paused: boolean`
- `questionCount: number` (for cluster variant)
**Emits**: `@tick(remaining: number)`, `@warning(threshold: number)`, `@expire()`
**Display**: Variant-specific rendering as described in F3.3.

### ConfidenceCapture (`components/question/ConfidenceCapture.vue`)
**Props**:
- `selected: 'sure' | 'not_sure' | 'guessed' | null`
- `required: boolean`
**Emits**: `@select(level: string)`
**Display**: Three pill buttons in a row. Unselected: outlined. Selected: filled with color + check icon.

### QuestionFeedback (`components/question/QuestionFeedback.vue`)
**Props**:
- `isCorrect: boolean`
- `partialCredit: boolean`
- `selectedAnswer: string`
- `correctAnswer: string`
- `explanation: string | null` (may contain LaTeX)
- `errorType: ErrorType | null`
**Emits**: `@see-full-explanation()`, `@next()`
**Display**: Banner (green/red/amber) + selected vs correct comparison + brief explanation + "See Full Explanation" link + "Next" button.

### QuestionExplanation (`components/question/QuestionExplanation.vue`)
**Props**:
- `explanation: string` (required, LaTeX-enabled)
- `depth: 'quick' | 'simple' | 'detailed'`
- `relatedEntries: QuestionKnowledgeLink[]` (glossary links)
**Display**: Progressive disclosure tabs (Quick/Simple/Detailed). KaTeX-rendered text. Related glossary entries as clickable chips at bottom.

### WrongAnswerReview (`components/question/WrongAnswerReview.vue`)
**Props**:
- `question: Question`
- `options: QuestionOption[]`
- `selectedOptionId: number`
- `diagnosis: WrongAnswerDiagnosis`
- `answerResult: AnswerProcessingResult`
- `errorHistory: { errorType: string, count: number }[]` (for part 9)
**Display**: 10 collapsible sections as described in F3.6. Progressive reveal with "Show More" between groups.

### MistakeClinicFlow (`components/question/MistakeClinicFlow.vue`)
**Props**:
- `question: Question`
- `options: QuestionOption[]`
- `selectedOptionId: number`
- `diagnosis: WrongAnswerDiagnosis`
- `recoveryQuestion: Question | null` (for step 5)
**Emits**: `@complete(recovered: boolean)`, `@skip()`
**Display**: 5-step wizard with step indicator. Each step has specific interaction. Step 5 embeds a QuestionCard for the recovery question.

### MathRenderer (`components/question/MathRenderer.vue`)
**Props**:
- `text: string` (required, may contain `$...$` and `$$...$$`)
- `block: boolean` (force block display)
**Display**: Parsed text with KaTeX-rendered math segments. Fallback to monospace on parse error.

### QuestionFlag (`components/question/QuestionFlag.vue`)
**Props**:
- `flagged: boolean`
**Emits**: `@toggle()`
**Display**: Flag icon button. Unflagged: outlined gray. Flagged: filled orange. Tooltip "Flag for review".

### QuestionNav (`components/question/QuestionNav.vue`)
**Props**:
- `items: { index: number, status: 'unanswered' | 'answered' | 'flagged' | 'current' }[]`
- `currentIndex: number`
**Emits**: `@navigate(index: number)`
**Display**: Grid of numbered circles. Colors: unanswered (gray outline), answered (blue filled), flagged (orange filled), current (blue ring + pulsing).

---

## 3.5 User Flows

### Flow: Answer a MCQ Question
1. QuestionCard renders with stem, 4 options, timer (if configured)
2. Student reads question, taps option B
3. Option B highlights (blue border, filled radio)
4. If confidence capture enabled: "Sure / Not Sure / Guessed" appears below options
5. Student taps "Not Sure"
6. Submit button now active (answer + confidence both set)
7. Student taps "Submit"
8. If `feedbackMode === 'immediate'`: feedback banner appears
9. Correct: green "Correct!" banner, option B highlighted green, +1 sound
10. Incorrect: red "Not quite" banner, option B highlighted red, correct option highlighted green
11. Brief explanation shown, "See Full Explanation" link available
12. "Next" button appears, or auto-advance after 2 seconds (configurable)

### Flow: Wrong Answer -> Mistake Clinic
1. Student answers incorrectly, feedback shown
2. If coach determines this error is worth clinicing (repeated pattern, critical topic):
3. "Let's work through this" prompt appears after feedback
4. Student taps "OK" -> Mistake Clinic flow begins
5. Step 1: Review question + answer. "Does this look right to you now?" -> Student taps "No"
6. Step 2: "Why did you pick {wrong answer}?" -> Self-report chips (looked similar, rushed, forgot formula, confused concepts, etc.) -> Student selects "confused concepts"
7. Step 3: Key clue highlighted in stem. "This word changes everything." Student reads.
8. Step 4: Worked example shown step-by-step
9. Step 5: Recovery question (simpler variant). Student answers correctly -> "Recovery confirmed!" green banner
10. Flow completes, returns to session

### Flow: Drag-Reorder Question
1. QuestionCard renders DragReorderQuestion with shuffled items
2. Student presses and holds an item -> item lifts (scale + shadow increase)
3. Drags item to target position -> placeholder shows where it will drop
4. Releases -> item snaps to position with spring animation
5. Repeats for all items
6. Submit button activates when all items have been repositioned at least once (or student is satisfied)
7. Submits -> feedback shows correct positions, items in wrong position shake + show arrow to correct slot

---

## 3.6 Tauri Commands

The Question Engine primarily uses data passed from parent session contexts. Direct IPC calls:

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `get_question` | `questionId: number` | `Question` | Fetch single question |
| `list_question_options` | `questionId: number` | `QuestionOption[]` | Fetch options for a question |
| `select_questions` | `request: QuestionSelectionRequest` | `SelectedQuestion[]` | Select questions for a session |
| `get_question_family` | `familyId: number` | `QuestionFamily` | Get question family info |
| `get_question_intelligence` | `questionId: number` | `QuestionIntelligenceProfile` | Get 8-axis intelligence profile |
| `get_question_knowledge_links` | `questionId: number` | `QuestionKnowledgeLink[]` | Get linked glossary entries |

---

## 3.7 Store State

The Question Engine does not have its own Pinia store. Question state is managed locally within each host component (SessionPlayer, DiagnosticPlayer, MockExamHall) via composables:

### `useQuestionState()` composable

```typescript
interface QuestionState {
  currentQuestion: Question | null;
  currentOptions: QuestionOption[];
  selectedOptionId: number | null;
  selectedAnswer: string | null; // for non-MCQ types
  confidenceLevel: 'sure' | 'not_sure' | 'guessed' | null;
  flagged: boolean;
  startedAt: string;
  submittedAt: string | null;
  responseTimeMs: number | null;
  hintCount: number;
  changedAnswerCount: number;
  feedbackVisible: boolean;
  isCorrect: boolean | null;
  answerResult: AnswerProcessingResult | null;
}
```

---

## 3.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Question appear | Fade in + slide up 15px, 300ms ease-out | None |
| Option hover | Background color shift, 100ms | None |
| Option select | Border color transition + radio fill, scale press 0.98 -> 1.0, 150ms | Soft selection tick (`feedback/select.mp3`) |
| Answer correct | Green flash on card, checkmark scale-in with bounce, confetti particles (3-5), 500ms | Correct chime (`feedback/correct.mp3`) |
| Answer incorrect | Red flash on card, X icon scale-in, gentle shake, 400ms | Wrong buzz (`feedback/wrong.mp3`) |
| Streak (3+ correct) | Golden glow border on question card, streak counter increment with bounce, 300ms | Streak sound (`feedback/streak.mp3`) -- escalating tone |
| Timer warning (25%) | Timer color shifts yellow -> orange, 200ms transition | None |
| Timer warning (10%) | Timer color shifts orange -> red, pulse animation begins | Subtle tick-tock (`feedback/timer-warning.mp3`) |
| Timer expire | Timer flashes red 3x, auto-submit triggers | Time-up sound (`feedback/time-up.mp3`) |
| Drag start | Item lifts (scale 1.05, shadow increase), 150ms | Pick-up sound (`feedback/drag-start.mp3`) |
| Drag drop | Item settles into position with spring bounce, 200ms | Drop sound (`feedback/drag-drop.mp3`) |
| Confidence select | Pill fill animation, 150ms | Soft click (`feedback/tap.mp3`) |
| Mistake Clinic step | Slide transition left-to-right, 300ms | None |
| Recovery confirmed | Green pulse border, checkmark animation, 600ms | Recovery chime (`feedback/recovery.mp3`) |
| Explanation panel open | Slide up from bottom, 400ms ease-out | None |
| KaTeX render | Instant (no animation) | None |

---
---

# DOMAIN 4: SESSION SYSTEM

## 4.1 Domain Overview

The Session System is the runtime engine that plays learning sessions. It dynamically composes sessions from typed blocks (quiz, explanation, drill, worked-example, reflection, timed-check, recall, memory-anchor), manages session lifecycle (start, pause, resume, complete, abandon), handles crash recovery, block transitions, and session debriefing. It also powers the Beat Yesterday 4-block structure. The SessionPlayer is the single most complex page in the application.

**Entry points**: Coach Hub "Start Session" CTA, Resume session, Beat Yesterday climb, Journey station session, repair session, diagnostic-to-session handoff.

---

## 4.2 Features

### F4.1 SessionPlayer Dynamic Block Renderer
- Receives a session definition with ordered blocks
- Each block has a type, configuration, and content
- Renders blocks sequentially (or allows navigation for some session types)
- Block types:

| Block Type | Content | Interaction |
|------------|---------|-------------|
| `quiz` | Question(s) from Question Engine | Answer questions, confidence, timer |
| `explanation` | Teach-mode content (text + diagrams + KaTeX) | Read, scroll, "Got it" confirmation |
| `drill` | Rapid-fire question sequence (shorter timer) | Quick answers, no explanation between |
| `worked_example` | Step-by-step solution walkthrough | Follow along, "I understand" per step |
| `reflection` | Coach prompt asking student to self-assess | Select reflection chips or free-text |
| `timed_check` | Quick timed micro-quiz (2-3 questions, strict timer) | Fast answers under time pressure |
| `recall` | "What do you remember about X?" free recall | Text input, then reveal comparison |
| `memory_anchor` | Key takeaway highlighted for memory encoding | Read, repeat, "I'll remember this" |

### F4.2 Session Progress
- Progress bar showing blocks completed vs total
- Block-type icons along the progress bar (quiz icon, explain icon, etc.)
- Current block highlighted/pulsing
- Estimated time remaining display
- Percentage complete

### F4.3 Session Timer
- Session-level timer (total session duration)
- Independent of per-question timers
- Shows elapsed time for untimed sessions
- Shows remaining time for timed sessions
- Pause-aware (timer stops when paused)

### F4.4 Pause/Resume
- Pause button always visible during session
- Pause overlay: darkened backdrop, "Session Paused" message, resume/stop buttons
- Pause reason capture (optional): "Taking a break", "Need to think", "Distracted"
- Timer freezes on pause, resumes on resume
- Auto-pause detection: 60 seconds of no interaction triggers "Still there?" prompt
- "Solving on paper" button: acknowledges student is working offline, prevents idle detection

### F4.5 Crash Recovery
- Session state saved to local storage after every answer/block transition
- On app relaunch, check for interrupted sessions
- Recovery prompt: "You have an unfinished session. Resume or discard?"
- Resume restores exact state (current block, current question, timer position, all previous answers)
- Discard marks session as abandoned

### F4.6 Block Transitions
- Between-block transition screens (1-2 seconds)
- Show: upcoming block type name, brief description, estimated duration
- Visual transition: current block slides out, brief interstitial, next block slides in
- Coach micro-message at transitions: "Good work. Now let's practice." / "Time to check your recall."
- Transition style varies by block pair (quiz -> reflection = calming, explanation -> drill = energizing)

### F4.7 Beat Yesterday 4-Block Structure
Specialized session structure for Beat Yesterday mode:

| Block | Name | Mood | Duration | Config |
|-------|------|------|----------|--------|
| 1 | Warm Start | Calm, gentle | 2-3 min | Easy questions, no timer, confidence-building |
| 2 | Core Climb | Focused, steady | 5-8 min | Medium-hard questions, soft timer, main improvement area |
| 3 | Speed Burst | Intense, energetic | 60 seconds | Rapid-fire easy-medium questions, strict countdown |
| 4 | Finish Strong | Positive, rewarding | 2-3 min | Questions student is likely to get right, end on high note |

- Block transition screens include micro-motivation messages
- Speed Burst has dramatic countdown intro (3-2-1-GO)
- Finish Strong guarantees ending with a correct answer (selects easier questions)

### F4.8 Session Brief (Pre-Session)
- Shown before session starts
- Content: session type, topic(s), estimated duration, block composition preview, why this session
- "Ready? Let's go" CTA
- Optional: warm-up toggle, difficulty preference
- Coach voice: "Here's what we're doing today and why."

### F4.9 Session Debrief (Post-Session)
- Shown after session completes
- Summary: accuracy %, questions attempted, time spent, streaks
- "What changed" section: mastery score changes per topic (before/after with delta arrows)
- "What's still fragile" section: topics that didn't improve enough
- "The real issue" section: dominant error type identified
- "What happens next" section: coach recommendation for next action
- Celebration for good sessions (confetti, upbeat message)
- Encouragement for tough sessions (warm, recovery-focused message)

### F4.10 Mid-Session Adaptation
- After every 3-5 questions, the system re-evaluates
- If accuracy is very high: increase difficulty, skip easier blocks
- If accuracy is very low: decrease difficulty, insert explanation blocks, slow timer
- If specific error pattern: insert targeted contrast teaching or worked example
- Adaptation is invisible to student (no jarring changes), gradual difficulty curve

---

## 4.3 Screens

### Screen: Session Player (`pages/student/session/[id].vue`)
**Layout**: `student` (minimal sidebar) or `focus` (full-screen)
**Route**: `/student/session/:id`

**Sections**:
| Section | Content | Position |
|---------|---------|----------|
| Session Header | Session type label, subject chip, session timer | Top bar, fixed |
| Progress Bar | Block progress with type icons | Below header |
| Block Content | Current block renderer (full remaining space) | Center, scrollable |
| Rescue Dock | 7 help buttons (collapsible) | Bottom, fixed |
| Coach Strip | Thin coach message bar | Above rescue dock |
| Pause Button | Pause icon button | Top-right corner |

**Data Sources**:
- `getSessionSnapshot(sessionId)` -> `SessionSnapshot`
- `getQuestion(questionId)` + `listQuestionOptions(questionId)` per question
- `processAnswer(submission)` -> `AnswerProcessingResult` per answer
- `pauseSession(sessionId)` / `resumeSession(sessionId)`
- `completeSession(sessionId)` -> triggers debrief

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Session data fetching | Skeleton: header + progress bar + large content area placeholder |
| Session Brief | Session loaded, not yet started | Pre-session brief card with "Start" CTA |
| Active - Quiz Block | Current block is quiz type | QuestionCard component with timer and confidence |
| Active - Explanation Block | Current block is explanation | Rich text content with "Got it" button |
| Active - Drill Block | Current block is drill | Rapid-fire question stream with minimal UI |
| Active - Worked Example | Current block is worked_example | Step-by-step solution display |
| Active - Reflection | Current block is reflection | Coach prompt + chip selection |
| Active - Timed Check | Current block is timed_check | Strict-timer questions, focused UI |
| Active - Recall | Current block is recall | Text input + reveal comparison |
| Active - Memory Anchor | Current block is memory_anchor | Key point display + confirmation |
| Block Transition | Between blocks | Transition interstitial (1-2 seconds) |
| Paused | Session paused | Dark overlay, "Session Paused" card, resume/stop buttons |
| Idle Warning | No interaction for 60s | "Still there?" prompt with resume/"Solving on paper" buttons |
| Completing | All blocks done, processing | Completing spinner |
| Complete | Session finished | Auto-redirect to debrief |
| Error | Session fetch failed or IPC error | AppError with "Session could not be loaded" + retry/go back |
| Crash Recovery | App relaunched with interrupted session | Recovery prompt: "Resume or discard?" |

---

### Screen: Session Debrief (`pages/student/session/debrief/[id].vue`)
**Layout**: `student`
**Route**: `/student/session/debrief/:id`

**Sections**:
| Section | Content | Data Source |
|---------|---------|-------------|
| Header | "Session Complete" + session type | Session data |
| Summary Stats | Accuracy %, questions count, time, streak | `SessionSummary` |
| What Changed | Per-topic mastery deltas (before/after bars) | `StudentTopicState` comparisons |
| What's Still Fragile | Topics with insufficient improvement | Filtered `TopicCase[]` |
| The Real Issue | Dominant error type + count | `WrongAnswerDiagnosis[]` aggregation |
| What Happens Next | Coach recommendation | `resolveNextCoachAction(studentId)` |
| Actions | "Continue to Coach Hub" / "Review Mistakes" / "Start Another Session" | Navigation buttons |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Debrief data computing | Skeleton layout |
| Good Session (>70% accuracy) | High accuracy | Celebration theme, confetti, upbeat coach voice |
| Okay Session (40-70%) | Medium accuracy | Encouraging theme, "solid effort" messaging |
| Tough Session (<40%) | Low accuracy | Warm recovery theme, "this is how you grow" messaging |
| Error | Debrief data failed | AppError with basic session stats from local state |

---

## 4.4 Components

### SessionPlayer (`components/session/SessionPlayer.vue`)
**Props**:
- `sessionId: number` (required)
**Internal State**: Manages block queue, current block index, session timer, pause state, answer history, adaptation engine.
**Template**: Renders session header + progress + current block component + rescue dock + coach strip.

### SessionBlock (`components/session/SessionBlock.vue`)
**Props**:
- `block: { type: string, config: object, content: object }` (required)
- `blockIndex: number`
**Display**: Routes to block-type-specific renderer. Quiz blocks embed QuestionCard. Explanation blocks render rich content.

### SessionProgress (`components/session/SessionProgress.vue`)
**Props**:
- `blocks: { type: string, status: 'pending' | 'active' | 'completed' }[]`
- `currentIndex: number`
**Display**: Horizontal progress bar with block-type icons. Completed blocks filled, current pulsing, pending outlined.

### SessionTimer (`components/session/SessionTimer.vue`)
**Props**:
- `totalSeconds: number | null` (null for untimed)
- `paused: boolean`
- `mode: 'elapsed' | 'countdown'`
**Emits**: `@tick(seconds: number)`, `@expire()`
**Display**: Digital clock format (MM:SS) in header area.

### SessionPause (`components/session/SessionPause.vue`)
**Props**:
- `visible: boolean`
**Emits**: `@resume()`, `@stop(reason?: string)`
**Display**: Full-screen dark overlay. Centered card: "Session Paused" title, elapsed time, "Resume" (primary) and "End Session" (danger) buttons. Optional reason selector chips.

### SessionComplete (`components/session/SessionComplete.vue`)
**Props**:
- `accuracy: number`
- `questionCount: number`
**Display**: Brief completion animation (1-2 seconds) before redirect to debrief. Checkmark icon + "Session Complete" text.

### BlockTransition (`components/session/BlockTransition.vue`)
**Props**:
- `fromBlock: string` (type)
- `toBlock: string` (type)
- `coachMessage: string`
**Display**: Full-screen interstitial. Coach message, upcoming block type name + icon, "Ready" button or auto-advance after 2 seconds.

### WarmStartBlock (`components/session/WarmStartBlock.vue`)
**Props**:
- `questions: SelectedQuestion[]`
**Display**: Gentle UI (softer colors, no timer visible), easy questions, confidence-building coach messages between questions.

### SpeedBurstBlock (`components/session/SpeedBurstBlock.vue`)
**Props**:
- `questions: SelectedQuestion[]`
- `durationSeconds: number` (default 60)
**Display**: Dramatic countdown intro (3-2-1-GO), rapid question display, shrinking timer bar, question counter, no explanations between questions.

### ReflectionBlock (`components/session/ReflectionBlock.vue`)
**Props**:
- `prompt: string`
- `chips: string[]` (selectable reflection options)
**Emits**: `@complete(selections: string[], freeText: string)`
**Display**: Coach prompt in speech bubble, chip grid for selection, optional free-text area, "Done" button.

---

## 4.5 User Flows

### Flow: Normal Session (Coach-Driven)
1. Coach Hub -> "Start Session" -> route to `/student/session/42`
2. Session Brief shown: "Today's focus: Algebraic Expressions. 15 min, 4 blocks."
3. Student taps "Let's go"
4. Block 1: Quiz (5 questions, medium difficulty, soft timer)
5. Student answers questions -> feedback after each
6. Block transition: "Good start. Let's see a worked example."
7. Block 2: Worked Example (step-by-step walkthrough)
8. Student follows, taps "I understand" per step
9. Block transition: "Now try some on your own."
10. Block 3: Drill (3 rapid questions, similar to worked example)
11. Block transition: "Let's reflect."
12. Block 4: Reflection ("How confident do you feel about this topic now?")
13. Student selects "Getting better" chip
14. Session complete -> redirect to debrief

### Flow: Session Pause & Resume
1. During Block 2, student needs a break
2. Taps pause button (top-right)
3. Session pauses: timer freezes, dark overlay appears
4. "Session Paused" card: elapsed time "4:32", Resume button, End Session button
5. Student returns after 5 minutes, taps "Resume"
6. Overlay fades, session continues from exact position
7. Timer resumes

### Flow: Crash Recovery
1. During session, app crashes (or student closes window)
2. Session state was saved to local storage after last answer
3. Student relaunches app, logs in
4. Coach Hub detects interrupted session (coach state: `mission_in_progress`)
5. Prompt: "You have an unfinished session from 10 minutes ago. Resume?"
6. Student taps "Resume" -> route to session with restored state
7. Session continues from last completed question in the current block

### Flow: Beat Yesterday Session
1. Beat Yesterday Home -> "Start Today's Climb" -> route to session
2. Session Brief: "4 blocks: Warm Start, Core Climb, Speed Burst, Finish Strong. ~12 min."
3. Block 1 - Warm Start: 3 easy questions, soft timer, calm colors. All correct -> "Good warm-up!"
4. Block 2 - Core Climb: 5 medium-hard questions, soft timer, focused. Mixed results.
5. Block 3 - Speed Burst: 3-2-1-GO countdown, 60 seconds, rapid-fire. 7 questions answered.
6. Block 4 - Finish Strong: 3 questions student is likely to ace. Ends with correct answer.
7. Session complete -> Beat Yesterday specific debrief with yesterday comparison.

---

## 4.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `start_practice_session` | `input: PracticeSessionStartInput` | `Session` | Start a practice session |
| `start_custom_test` | `input: CustomTestStartInput` | `Session` | Start a custom test |
| `get_session_snapshot` | `sessionId: number` | `SessionSnapshot` | Get full session state |
| `submit_session_answer` | `input: SessionAnswerInput` | `AnswerProcessingResult` | Submit answer within session |
| `pause_session` | `sessionId: number` | `boolean` | Pause active session |
| `resume_session` | `sessionId: number` | `boolean` | Resume paused session |
| `complete_session` | `sessionId: number` | `SessionSummary` | Complete and finalize session |
| `abandon_session` | `sessionId: number, reason: string` | `boolean` | Mark session as abandoned |
| `get_session_summary` | `sessionId: number` | `SessionSummary` | Get post-session summary |
| `get_active_session` | `studentId: number` | `Session \| null` | Find any in-progress session |

---

## 4.7 Store State

### Session Store (`stores/session.ts`)

```typescript
interface SessionState {
  // Active session
  activeSession: Session | null;
  sessionItems: SessionItem[];
  sessionLoading: boolean;
  sessionError: string | null;

  // Current block
  currentBlockIndex: number;
  currentBlockType: string | null;
  blocks: SessionBlock[];

  // Current question within block
  currentQuestionIndex: number;
  currentQuestion: Question | null;
  currentOptions: QuestionOption[];

  // Timer state
  sessionElapsedSeconds: number;
  sessionTimerPaused: boolean;
  questionTimerSeconds: number | null;

  // Answer tracking
  answersThisSession: AnswerProcessingResult[];
  correctCount: number;
  totalAnswered: number;

  // Pause state
  isPaused: boolean;
  pausedAt: string | null;

  // Crash recovery
  hasRecoverableSession: boolean;
  recoverableSessionId: number | null;

  // Debrief
  sessionSummary: SessionSummary | null;
  debriefLoading: boolean;

  // Beat Yesterday specific
  isBeatYesterdaySession: boolean;
  yesterdayScore: number | null;
  todayScore: number | null;
}
```

---

## 4.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Session start | Block progress bar builds in from left, 600ms | Session start chime (`transitions/session-start.mp3`) |
| Block transition | Current block slides out left, interstitial fades in, next block slides in from right; total 1200ms | Transition whoosh (`transitions/block-change.mp3`) |
| Quiz block enter | Questions fade in with stagger, 200ms each | None |
| Drill block enter | Dramatic "focus" animation -- borders tighten, colors intensify, 400ms | Intensity ramp (`transitions/drill-enter.mp3`) |
| Speed Burst countdown | Large numbers 3, 2, 1 scale in and out center screen, then "GO!" with flash | Countdown beeps + GO sound (`transitions/countdown.mp3`) |
| Speed Burst active | Background subtle pulse to music tempo, timer urgently shrinks | Ambient energy beat (`ambient/speed-burst.mp3`) |
| Pause overlay | Fade in dark backdrop 300ms, card scales in from center 200ms | Pause sound (`transitions/pause.mp3`) |
| Resume | Overlay fades out 200ms | Resume sound (`transitions/resume.mp3`) |
| Session complete | Checkmark draws on (SVG path animation), 800ms | Completion chime (`celebration/session-complete.mp3`) |
| Good debrief (>70%) | Confetti burst from top, 1500ms | Celebration fanfare (`celebration/good-session.mp3`) |
| Tough debrief (<40%) | Warm glow border, gentle encouragement animation | Soft supportive chime (`feedback/encouragement.mp3`) |
| Mastery delta (positive) | Green arrow bounces up next to topic name, number counts up | Positive tick (`feedback/mastery-up.mp3`) |
| Mastery delta (negative) | Red arrow slides down, subtle | None |
| Idle warning | Gentle pulse on "Still there?" card, 2s cycle | Soft notification (`feedback/idle-ping.mp3`) |
| Crash recovery prompt | Card slides down from top, 400ms | Alert chime (`feedback/recovery-prompt.mp3`) |

---
---

# DOMAIN 5: DIAGNOSTIC SYSTEM

## 5.1 Domain Overview

The Diagnostic System delivers multi-phase adaptive assessments to establish a student's academic baseline. It supports 3 modes (quick/standard/deep) with up to 6 phase types, each with a distinct visual atmosphere. The system produces a comprehensive 7-section report with PDF export capability and 3 audience-specific views (student, parent, admin). Diagnostics are high-stakes assessments that create the foundation for all coaching decisions.

**Entry points**: Coach Hub diagnostic directive, onboarding flow, manual launch from practice hub.

---

## 5.2 Features

### F5.1 Launcher & Mode Selection
- Three diagnostic modes with clear descriptions:

| Mode | Questions | Phases | Duration | Description |
|------|-----------|--------|----------|-------------|
| Quick | 15-20 | 2-3 (baseline, speed) | ~15 min | "A quick snapshot of where you stand" |
| Standard | 30-40 | 4 (baseline, speed, precision, pressure) | ~30 min | "A thorough assessment of your abilities" |
| Deep | 50-70 | 6 (all phases) | ~60 min | "The complete academic DNA test" |

- Mode cards with estimated time, phase count, and recommendation badge
- Coach recommends a mode based on context (first-time: standard, returning: quick, pre-exam: deep)
- Subject selection: which subject(s) to diagnose
- "Start Diagnostic" CTA

### F5.2 Phase Types (6 Distinct Atmospheres)

#### Phase 1: Baseline (Calm)
- **Atmosphere**: Light, airy, spacious
- **Colors**: Soft blues and whites
- **Timer**: None (or very generous soft timer)
- **Music/Ambient**: Gentle ambient hum
- **Purpose**: Establish raw mastery without time pressure
- **Question types**: Mix of difficulties, no tricks
- **Behavior logging**: Time per question, confidence, answer changes

#### Phase 2: Speed (Brisk)
- **Atmosphere**: Energetic, slightly more urgent
- **Colors**: Warmer tones, amber accents
- **Timer**: Moderate cluster timer (e.g., 8 questions in 5 min)
- **Music/Ambient**: Subtle ticking background
- **Purpose**: Test fluency and automatic recall
- **Question types**: Easier questions but rapid pace
- **Behavior logging**: Speed, hesitation patterns, timeout rate

#### Phase 3: Precision (Spacious)
- **Atmosphere**: Clean, detailed, clinical
- **Colors**: Cool grays, sharp contrasts
- **Timer**: Generous per-question timer
- **Music/Ambient**: Silence or minimal ambient
- **Purpose**: Test careful reasoning without time pressure
- **Question types**: Harder questions, more complex formats
- **Behavior logging**: Answer changes, option consideration time, precision of responses

#### Phase 4: Pressure (Urgent)
- **Atmosphere**: Dark, intense, focused
- **Colors**: Dark background, red/amber accents
- **Timer**: Strict countdown (tight per-question limits)
- **Music/Ambient**: Low heartbeat pulse
- **Purpose**: Test performance under exam-like pressure
- **Question types**: Medium-hard with time constraints
- **Behavior logging**: Panic indicators (rapid option switching, early submit, timeouts)

#### Phase 5: Flex (Varied)
- **Atmosphere**: Unpredictable, shifting
- **Colors**: Alternating between phases
- **Timer**: Mixed (some timed, some not)
- **Music/Ambient**: Varied
- **Purpose**: Test adaptability and consistency across conditions
- **Question types**: Random mix of difficulties and formats
- **Behavior logging**: Consistency of performance across conditions

#### Phase 6: Root Cause (Surgical)
- **Atmosphere**: Deep, investigative, focused
- **Colors**: Muted purples, dark backgrounds
- **Timer**: None
- **Music/Ambient**: Focus ambient
- **Purpose**: Drill into specific weaknesses identified in earlier phases
- **Question types**: Targeted at areas of uncertainty, probing specific misconceptions
- **Behavior logging**: Response to targeted probes, hypothesis confirmation/rejection

### F5.3 Phase Transitions
- Between-phase transition screen (3-5 seconds)
- Shows: completed phase summary, upcoming phase name + description + mood shift preview
- Coach message: "Phase 1 complete. Now let's test your speed."
- Progress indicator: phases completed vs total
- Atmosphere shift: smooth color/mood transition during interstitial
- Optional 30-second break prompt between phases

### F5.4 Adaptive Routing
- After each phase, the system analyzes results to decide:
  - Which topics to probe deeper in next phase
  - Whether to skip remaining phases (if enough data)
  - Which question difficulty range to use
- Student sees only the phases relevant to their profile
- Root-cause phase only triggered if specific inconsistencies detected

### F5.5 Per-Item Behavioral Logging
Every diagnostic question captures:
- `responseTimeMs` -- time to answer
- `confidenceLevel` -- sure/not_sure/guessed
- `changedAnswerCount` -- how many times answer was changed
- `hintCount` -- hints requested (usually 0 in diagnostic)
- `timedOut` -- whether timer expired before answer
- `skipped` -- whether student skipped the question
- `optionConsiderationPattern` -- which options were hovered/considered (if trackable)

### F5.6 Diagnostic Report (7 Sections)

#### Section 1: Overall Dashboard
- Readiness band (Not Ready / Building / Almost Ready / Exam Ready) with large colored indicator
- Overall readiness score (0-100%)
- Radar chart: 6 dimensions (Knowledge, Speed, Precision, Pressure Resilience, Flexibility, Stability)
- Subjects tested with per-subject readiness bars
- Date, duration, mode used

#### Section 2: Academic Profile
- Student type classification (e.g., "Steady Learner", "Speed Demon", "Careful Thinker", "Pressure Vulnerable")
- Strengths summary (top 3 areas)
- Weaknesses summary (top 3 areas)
- Learning style indicators inferred from behavior
- Confidence calibration: how well confidence matched accuracy

#### Section 3: Topic Breakdown
- Table of all topics tested with per-topic scores:
  - Mastery score, fluency score, precision score, pressure score, flexibility score, stability score
  - Classification (strong, developing, weak, critical)
  - Trend indicator (if previous diagnostic exists)
- Sortable by any column
- Color-coded rows by classification
- Expandable rows showing individual question results per topic

#### Section 4: Guessed Answers
- List of questions where `confidenceLevel === 'guessed'`
- Percentage of guessed answers that were correct ("luck vs knowledge" metric)
- Guessed-and-correct highlighted as "uncertain knowledge -- needs proof"
- Guessed-and-wrong highlighted as "knowledge gap confirmed"

#### Section 5: Misconception Bank
- All identified misconceptions from wrong answers
- Grouped by topic
- Each misconception: description, related error type, frequency, severity
- "Repair Priority" ranking
- Links to contrast teaching / worked examples for each misconception

#### Section 6: Exam Behavior Profile
- Performance under pressure (calm vs pressure accuracy comparison chart)
- Time management profile: average time per question by phase
- Guess rate by phase
- Confidence calibration by phase
- Panic indicators: rapid switching, early abandonment
- Fatigue detection: performance decline over time

#### Section 7: Intervention Map
- Priority-ordered list of interventions
- Each intervention: topic, issue type, recommended action, estimated time, urgency level
- Action buttons: "Start now", "Schedule", "Add to plan"
- Links to relevant sessions, drills, or teach mode content

### F5.7 PDF Export
- One-click PDF generation of full report
- Formatted for A4 printing
- Includes all 7 sections with charts rendered as images
- Header: student name, date, diagnostic mode
- Footer: page numbers, eCoach branding
- 3 audience views:
  - Student view: encouraging language, action-oriented, simplified charts
  - Parent view: plain-language summaries, risk-focused, recommended support actions
  - Admin view: full data, technical metrics, raw scores

### F5.8 Three Audience Views
- Toggle at top of report: "Student View" | "Parent View" | "Admin View"
- Each view shows the same 7 sections but with:
  - Different language register (encouraging vs informative vs technical)
  - Different emphasis (action vs understanding vs data)
  - Different chart complexity (simplified vs standard vs detailed)
  - Different recommendations (do this vs support this vs monitor this)

---

## 5.3 Screens

### Screen: Diagnostic Launcher (`pages/student/diagnostic/index.vue`)
**Layout**: `student`
**Route**: `/student/diagnostic/`

**Sections**:
| Section | Content |
|---------|---------|
| Title | "Academic Diagnostic" |
| Coach Recommendation | Badge indicating recommended mode with reason |
| Mode Cards | 3 cards: Quick, Standard, Deep -- each with description, time, phase count |
| Subject Selector | Checkboxes for available subjects |
| CTA | "Start Diagnostic" (disabled until mode + subjects selected) |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Subjects loading | Skeleton mode cards + loading subject list |
| Normal | Subjects loaded, no active diagnostic | Mode cards + subject selector + CTA |
| Previous Exists | Previous diagnostic results available | "Previous results" link + "Re-test" option |
| Active Diagnostic | An in-progress diagnostic exists | "Resume" card instead of launcher |
| Error | Data fetch failed | AppError with retry |

---

### Screen: Diagnostic Phase Player (`pages/student/diagnostic/[id].vue`)
**Layout**: `focus` (full-screen, minimal chrome)
**Route**: `/student/diagnostic/:id`

**Sections**:
| Section | Content |
|---------|---------|
| Phase Header | Phase name, phase number / total, phase-specific atmosphere bar |
| Phase Timer | Cluster timer or per-question timer (phase-dependent) |
| Question Area | QuestionCard (full width) |
| Progress | Question X of Y within phase |
| Behavioral UI | Confidence capture (always on), flag button |
| Phase Transition | Between-phase interstitial |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Phase data loading | Phase-colored skeleton |
| Active - Baseline | Phase 1 | Calm atmosphere: light blues, spacious, no timer |
| Active - Speed | Phase 2 | Brisk atmosphere: amber accents, cluster timer visible |
| Active - Precision | Phase 3 | Spacious atmosphere: cool grays, generous timer |
| Active - Pressure | Phase 4 | Urgent atmosphere: dark bg, red accents, strict timer, heartbeat |
| Active - Flex | Phase 5 | Varied atmosphere: shifting colors per question |
| Active - Root Cause | Phase 6 | Surgical atmosphere: muted purples, no timer, focused |
| Phase Transition | Between phases | Interstitial screen with summary + next phase preview |
| Processing | All phases complete, generating report | Processing animation "Analyzing your results..." |
| Error | Phase data or submission failed | Error with "Try Again" |

---

### Screen: Diagnostic Report (`pages/student/diagnostic/report/[id].vue`)
**Layout**: `student`
**Route**: `/student/diagnostic/report/:id`

**Sections**: 7 report sections as described in F5.6, plus:
| Section | Content |
|---------|---------|
| Report Header | Student name, diagnostic date, mode, audience toggle |
| Audience Toggle | 3-button group: Student / Parent / Admin |
| Export Button | "Download PDF" button |
| Section Navigation | Side navigation with 7 section links (scroll-spy) |
| Each Section | Section-specific content as per F5.6 |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Report data loading | Full-page skeleton with section placeholders |
| Normal | Report loaded | 7 sections with audience-appropriate content |
| Generating PDF | PDF export in progress | Loading overlay "Generating PDF..." |
| PDF Ready | PDF generated | Browser download triggered |
| Error | Report fetch failed | AppError with retry |

---

## 5.4 Components

### DiagnosticLauncher (`components/diagnostic/DiagnosticLauncher.vue`)
**Props**: `studentId: number`, `subjects: Subject[]`
**Emits**: `@start(mode: DiagnosticMode, subjectIds: number[])`
**Display**: Mode cards + subject selector + CTA.

### DiagnosticPhasePlayer (`components/diagnostic/DiagnosticPhasePlayer.vue`)
**Props**:
- `battery: DiagnosticBattery`
- `currentPhase: DiagnosticPhasePlan`
- `items: DiagnosticPhaseItem[]`
**Emits**: `@phase-complete(phaseId: number)`, `@answer(questionId: number, optionId: number, meta: object)`
**Display**: Phase-atmosphere-styled question player.

### PhaseTransition (`components/diagnostic/PhaseTransition.vue`)
**Props**:
- `completedPhase: DiagnosticPhasePlan`
- `nextPhase: DiagnosticPhasePlan`
- `phaseAccuracy: number`
**Emits**: `@continue()`, `@take-break()`
**Display**: Phase summary card, next phase preview, atmosphere preview, coach message.

### DiagnosticReport (`components/diagnostic/DiagnosticReport.vue`)
**Props**:
- `result: DiagnosticResult`
- `topicResults: TopicDiagnosticResult[]`
- `audienceView: 'student' | 'parent' | 'admin'`
**Display**: 7-section report renderer.

### ReportOverview (`components/diagnostic/ReportOverview.vue`)
**Props**: `result: DiagnosticResult`
**Display**: Readiness band indicator, overall score, radar chart (6 dimensions), subject bars.

### ReportAcademicProfile (`components/diagnostic/ReportAcademicProfile.vue`)
**Props**: `result: DiagnosticResult`, `audienceView: string`
**Display**: Student type classification, strengths/weaknesses, confidence calibration.

### ReportTopicBreakdown (`components/diagnostic/ReportTopicBreakdown.vue`)
**Props**: `topicResults: TopicDiagnosticResult[]`
**Display**: Sortable table, color-coded rows, expandable detail per topic.

### ReportMisconceptionBank (`components/diagnostic/ReportMisconceptionBank.vue`)
**Props**: `diagnoses: WrongAnswerDiagnosis[]`
**Display**: Grouped misconceptions by topic, severity badges, repair priority.

### ReportExamBehavior (`components/diagnostic/ReportExamBehavior.vue`)
**Props**: `behaviorData: object` (phase-specific behavior aggregation)
**Display**: Calm vs pressure accuracy chart, time management profile, guess rate chart, panic indicators.

### ReportInterventionMap (`components/diagnostic/ReportInterventionMap.vue`)
**Props**: `interventions: { topic: string, issue: string, action: string, urgency: string, estimatedMinutes: number }[]`
**Emits**: `@start-intervention(index: number)`, `@schedule(index: number)`
**Display**: Priority-ordered intervention cards with action buttons.

---

## 5.5 User Flows

### Flow: First-Time Diagnostic (Standard)
1. Coach Hub state `diagnostic_required` -> "Discover your starting point" card
2. Student taps "Start Diagnostic" -> route to `/student/diagnostic/`
3. Launcher shows with "Standard" recommended. Student selects Mathematics. Taps "Start."
4. `startDiagnostic(studentId, subjectId, 'standard')` -> returns `DiagnosticBattery`
5. Route to `/student/diagnostic/42`
6. Phase 1 (Baseline): Calm atmosphere, 10 questions, no timer. Student answers all.
7. Phase Transition: "Baseline complete. 7/10 correct. Now let's test your speed."
8. Phase 2 (Speed): Brisk atmosphere, 8 questions in 5 minutes. Student answers under time.
9. Phase Transition: "Speed phase done. Now for precision."
10. Phase 3 (Precision): Spacious atmosphere, 8 harder questions, generous timer.
11. Phase Transition: "Precision checked. One more phase."
12. Phase 4 (Pressure): Dark atmosphere, 8 questions, strict timer, heartbeat pulse.
13. All phases complete. Processing screen: "Analyzing your results..."
14. Report generated. Route to `/student/diagnostic/report/42`
15. Student reviews 7-section report, sees strengths and weaknesses, intervention map.

### Flow: PDF Export for Parent
1. On report screen, student toggles to "Parent View"
2. Report language shifts to parent-appropriate
3. Student taps "Download PDF"
4. PDF generates (loading overlay, ~3 seconds)
5. PDF downloads to system
6. Student can share with parent

---

## 5.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `start_diagnostic` | `studentId: number, subjectId: number, mode: DiagnosticMode` | `DiagnosticBattery` | Initialize diagnostic |
| `get_diagnostic_battery` | `diagnosticId: number` | `DiagnosticBattery` | Fetch diagnostic state |
| `get_diagnostic_phase` | `phaseId: number` | `DiagnosticPhasePlan` | Get phase info |
| `list_diagnostic_phase_items` | `phaseId: number` | `DiagnosticPhaseItem[]` | Get questions for phase |
| `submit_diagnostic_answer` | `phaseId: number, questionId: number, optionId: number, meta: object` | `AnswerProcessingResult` | Submit answer with behavior data |
| `complete_diagnostic_phase` | `phaseId: number` | `boolean` | Finalize phase |
| `get_diagnostic_result` | `diagnosticId: number` | `DiagnosticResult` | Get full results |
| `export_diagnostic_pdf` | `diagnosticId: number, audience: string` | `string` (file path) | Generate and save PDF |

---

## 5.7 Store State

### Diagnostic Store (`stores/diagnostic.ts`)

```typescript
interface DiagnosticState {
  // Active diagnostic
  activeBattery: DiagnosticBattery | null;
  batteryLoading: boolean;
  batteryError: string | null;

  // Current phase
  currentPhaseIndex: number;
  currentPhase: DiagnosticPhasePlan | null;
  phaseItems: DiagnosticPhaseItem[];
  currentItemIndex: number;

  // Phase atmosphere (drives visual theming)
  atmosphere: 'calm' | 'brisk' | 'spacious' | 'urgent' | 'varied' | 'surgical';

  // Answers within current phase
  phaseAnswers: { questionId: number, optionId: number, meta: object }[];
  phaseAccuracy: number;

  // Report
  diagnosticResult: DiagnosticResult | null;
  topicResults: TopicDiagnosticResult[];
  reportLoading: boolean;
  reportAudienceView: 'student' | 'parent' | 'admin';

  // PDF
  pdfGenerating: boolean;
  pdfPath: string | null;

  // Phase transition
  showPhaseTransition: boolean;
  completedPhaseAccuracy: number;
}
```

---

## 5.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Diagnostic start | Focus mode transition: UI narrows, sidebars collapse, 600ms | Diagnostic start tone (`transitions/diagnostic-start.mp3`) |
| Phase 1 (Baseline) enter | Soft blue gradient fade in, spacious layout expand, 500ms | Calm ambient loop begins (`ambient/baseline-calm.mp3`) |
| Phase 2 (Speed) enter | Amber tones warm in, layout slightly tightens, cluster timer appears, 500ms | Ticking ambient begins (`ambient/speed-brisk.mp3`) |
| Phase 3 (Precision) enter | Cool gray sharpens in, layout opens wider, 500ms | Silence or minimal hum |
| Phase 4 (Pressure) enter | Dark background fades in, red accents glow, 500ms | Heartbeat pulse begins (`ambient/pressure-heartbeat.mp3`) |
| Phase 5 (Flex) enter | Colors shift randomly, 500ms | Varied ambient |
| Phase 6 (Root Cause) enter | Deep purple fades in, spotlight effect on question area, 500ms | Focus drone begins (`ambient/root-cause-focus.mp3`) |
| Phase transition | Cross-fade between atmospheres, summary slides in from bottom, 800ms total | Phase complete chime (`transitions/phase-complete.mp3`) |
| Processing screen | DNA helix rotation animation (SVG), pulsing "Analyzing..." text, 3-8 seconds | Processing ambient (`ambient/analyzing.mp3`) |
| Report reveal | Sections stagger fade-in from top, 200ms each, 100ms stagger | Report reveal chime (`celebration/report-ready.mp3`) |
| Radar chart draw | Chart lines draw in from center outward, 800ms | None |
| PDF export | Download icon animation (bounce down), 300ms | Download click (`feedback/download.mp3`) |
| Audience view toggle | Content cross-fades, 300ms | None |

---
---

# DOMAIN 6: MOCK CENTRE

## 6.1 Domain Overview

The Mock Centre simulates real exam conditions with ceremonial exam-hall transitions, strict timing, and comprehensive post-mock analysis. It supports 5 mock types, a setup wizard, pre-flight checks, a distraction-free exam hall interface, and a detailed 6-tab post-mock review. The Mock Centre also tracks mock history over time and provides forecast insights. This is the closest the app gets to the real exam experience.

**Entry points**: Coach Hub mock directive, sidebar "Mock Centre" nav item, Journey mode mock stations, Elite mock.

---

## 6.2 Features

### F6.1 Mock Centre Home (Readiness Display)
- Prominent readiness gauge: current exam readiness score (0-100%)
- Readiness band label (Not Ready / Building / Almost Ready / Exam Ready)
- Next exam date + countdown
- Quick stats: total mocks taken, best score, average score, trend arrow
- Mock type cards for launching
- Recent mock history (last 3-5) with score trend mini-chart
- "Warm Up First?" option -- short 5-question warm-up before mock

### F6.2 Five Mock Types

| Type | Description | Duration | Question Count |
|------|------------|----------|----------------|
| Full Mock | Complete exam simulation | Full exam duration | Full paper count |
| Topic Mock | Focused on specific topic(s) | 30-45 min | 15-25 questions |
| Mini Mock | Quick exam snapshot | 15-20 min | 10-15 questions |
| Recovery Mock | After a bad session, easier selection | 20-30 min | 15-20 questions |
| Pressure Mock | Harder than real exam, tighter time | Full or reduced | Full or reduced count |

### F6.3 Setup Wizard
Step-by-step mock configuration:
1. **Mock Type**: Select from 5 types (cards with descriptions)
2. **Subject**: Select subject (if multiple enrolled)
3. **Topics**: Select specific topics or "Full Syllabus" (for full/mini/pressure mocks)
4. **Duration**: Confirm or customize time limit
5. **Preview**: Summary of configuration + "Ready" CTA

### F6.4 Pre-Flight Card
- Final confirmation screen before exam begins
- Content: mock type, subject, topic scope, question count, time limit
- Reminders: "Make sure you have a quiet space", "No notes or textbooks"
- "I'm Ready" button (prominent, green)
- "Not Yet" link (returns to home)

### F6.5 Ceremonial Exam Hall Transition (3-5 seconds)
- Full-screen cinematic transition from normal UI to exam hall
- Steps:
  1. Current UI fades to black (500ms)
  2. "Exam Hall" text fades in with subtle glow (500ms)
  3. Exam hall UI builds in: desk surface appears, paper slides in, timer appears (2000ms)
  4. "Begin" text fades in (500ms)
- Sound: door closing, paper shuffling, clock starting
- Creates psychological separation between "studying" and "exam"

### F6.6 Live Exam Hall Interface
Minimal, distraction-free exam interface:

| Element | Details |
|---------|---------|
| Question Display | Full-width question with QuestionCard (MCQ or appropriate format) |
| Progress | "Question 12 of 40" in header |
| Timer | Strict countdown in top-right, color-shifts at 25%/10% |
| Flag Button | Flag current question for review |
| Confidence Tag | Sure/Not Sure/Guessed below question |
| Pacing Badge | "On Pace" (green) / "Behind" (amber) / "Ahead" (blue) -- based on expected time per question |
| Nav Grid | Expandable question navigation grid (numbered circles showing status) |
| Next/Previous | Navigation buttons at bottom |
| Submit | "End Exam" button (top-right, requires confirmation) |

- No sidebar, no coach messages, no rescue dock, no explanations
- Color scheme: neutral, serious, exam-appropriate
- Font: clean, high-contrast for readability

### F6.7 Submission Confirmation
- "Submit your exam?" modal
- Shows: answered count, unanswered count, flagged count
- "Go Back" (review more) vs "Submit" (confirm)
- If flagged questions exist: "You have {n} flagged questions. Are you sure?"
- After submit: brief processing, then redirect to debrief

### F6.8 Post-Mock Debrief (Emotional Arc)
The debrief follows an emotional arc designed to land feedback without crushing motivation:

1. **Score Reveal** (2 seconds): Large score number counts up with animation
2. **Context** (immediate): "This puts you in the {band} zone" + comparison to last mock
3. **Celebration/Encouragement** (3 seconds): Good score = confetti + "Excellent!"; Low score = warm message + "Every mock teaches you something"
4. **Detailed Tabs** (persistent): 6-tab review interface

### F6.9 Six-Tab Post-Mock Review

#### Tab 1: Overview
- Score, grade (A-F), time used, completion rate
- Score trend chart (if multiple mocks exist)
- Comparison bar: this mock vs average vs best
- Band indicator with progress within band

#### Tab 2: Strengths & Weaknesses
- Two columns: Green (strengths) and Red (weaknesses)
- Topics listed with accuracy per topic
- "Strongest area" highlight
- "Biggest opportunity" highlight

#### Tab 3: What Hurt You
- The questions/topics that cost the most marks
- Sorted by impact (marks lost)
- Each entry: question preview, your answer, correct answer, marks lost, error type
- "These {n} questions cost you {m} marks"

#### Tab 4: Question Review (6 Lenses)
All questions reviewable through 6 different filters:
1. **All**: Sequential order
2. **Incorrect Only**: Just wrong answers
3. **Flagged**: Questions student flagged during exam
4. **By Topic**: Grouped by topic
5. **By Time Spent**: Sorted by response time (slowest first)
6. **By Confidence**: Grouped by confidence tag

Each question review shows: stem, your answer, correct answer, explanation, time spent, confidence level, error type (if wrong)

#### Tab 5: Timing Analysis
- Time spent per question (bar chart)
- Average time vs expected time per question
- Time distribution: which quartile took the most time
- Rush pattern: did student speed up at end?
- Time wasted: time on questions eventually answered incorrectly
- Pacing chart: actual vs ideal pace over exam duration

#### Tab 6: Next Steps
- Coach-generated action plan based on mock results
- Priority interventions (topic + action + estimated time)
- "If you focus on these 3 areas, your score could improve by ~{n} marks"
- Quick-launch buttons: "Drill weak topics", "Review mistakes", "Retake mock"

### F6.10 Mock History Timeline
- Chronological list of all completed mocks
- Each entry: date, type, subject, score, grade, trend arrow
- Score trend line chart across all mocks
- Filter by: subject, type, date range
- Tap any entry to view full post-mock review

### F6.11 Forecast Insights
- Predictive analysis based on mock history + student model
- "If the exam were today" score estimate
- Predicted grade range (with confidence interval)
- Topics likely to appear on exam (from past paper intelligence)
- "Score ceiling" -- maximum achievable with current knowledge
- "Quick wins" -- easiest points to gain

### F6.12 Warm-Up Option
- Optional 5-question warm-up before mock
- Easy-medium questions from mock topics
- No scoring, no pressure
- Purpose: activate knowledge, reduce test anxiety
- "Skip warm-up" always available

### F6.13 Recovery Mock Mode
- Special mock type triggered after poor performance or long absence
- Easier question selection (60% at or below student's level)
- Encouraging coach messages after each section
- Slightly more generous timing
- Debrief emphasizes what student CAN do, not what they can't
- Amber/warm visual theme instead of standard neutral

---

## 6.3 Screens

### Screen: Mock Centre Home (`pages/student/mock/index.vue`)
**Layout**: `student`
**Route**: `/student/mock/`

**Sections**:
| Section | Content |
|---------|---------|
| Readiness Gauge | Large circular gauge (0-100%), band label |
| Exam Countdown | Days to exam, date |
| Quick Stats | Mocks taken, best score, average, trend |
| Mock Type Cards | 5 mock type cards with descriptions |
| Recent Mocks | Last 3-5 mocks with mini score chart |
| Warm-Up Toggle | "Warm up before your mock?" toggle |
| Forecast | "If exam were today: ~{score}" insight card |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Data fetching | Skeleton gauge + card placeholders |
| Normal | Data loaded, mocks available | Full layout with all sections |
| No Mocks Yet | Never taken a mock | Empty state: "Your first mock awaits" + recommended type |
| No Exam Set | No exam date configured | Readiness gauge hidden, prompt to set exam date |
| Error | Data fetch failed | AppError with retry |

---

### Screen: Mock Setup (`pages/student/mock/setup.vue`)
**Layout**: `student`
**Route**: `/student/mock/setup?type=<type>`

**Sections**: 5-step wizard as described in F6.3.

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Step 1-5 | Active wizard step | Step content with stepper |
| Creating | Mock being created after preview confirmed | Loading on "Start" button |
| Error | Mock creation failed | Error message with retry |

---

### Screen: Live Exam Hall (`pages/student/mock/hall/[id].vue`)
**Layout**: `focus` (full-screen, no sidebar, no distractions)
**Route**: `/student/mock/hall/:id`

**Sections**: As described in F6.6 (question, progress, timer, flag, confidence, pacing, nav grid).

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Transition | Ceremonial entrance | 3-5 second transition animation |
| Active | Exam in progress | Full exam hall interface |
| Nav Grid Open | Question grid expanded | Overlay grid showing all question statuses |
| Submitting | Student confirmed submission | Processing overlay |
| Time Up | Timer expired | Auto-submit with "Time's up" overlay |
| Error | Submission or fetch error | Subtle error toast (don't break exam flow) |

---

### Screen: Post-Mock Review (`pages/student/mock/review/[id].vue`)
**Layout**: `student`
**Route**: `/student/mock/review/:id`

**Sections**: Score reveal animation, then 6-tab interface as described in F6.9.

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Review data loading | Skeleton |
| Score Reveal | Initial load, score animating | Large score number counting up, band reveal |
| Review | Score revealed, tabs active | 6-tab interface with full review data |
| Error | Data fetch failed | AppError with retry |

---

### Screen: Mock History (`pages/student/mock/history.vue`)
**Layout**: `student`
**Route**: `/student/mock/history`

**Sections**:
| Section | Content |
|---------|---------|
| Score Trend | Line chart of scores over time |
| Filter Bar | Subject, type, date range filters |
| Mock List | Chronological list of mock entries |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | History loading | Skeleton chart + list |
| Normal | History loaded | Chart + list |
| Empty | No mocks taken | "No mocks yet" + CTA to start first mock |
| Error | Fetch failed | AppError |

---

## 6.4 Components

### MockHome (`components/mock/MockHome.vue`)
**Props**: `studentId: number`
**Display**: Readiness gauge, stats, mock type cards, recent mocks, forecast.

### MockSetup (`components/mock/MockSetup.vue`)
**Props**: `mockType: string`, `subjects: Subject[]`
**Emits**: `@start(config: MockConfig)`
**Display**: 5-step wizard.

### MockPreFlight (`components/mock/MockPreFlight.vue`)
**Props**: `config: MockConfig`
**Emits**: `@ready()`, `@cancel()`
**Display**: Final confirmation card with reminders.

### MockExamHall (`components/mock/MockExamHall.vue`)
**Props**: `sessionId: number`, `questions: SessionItem[]`, `durationSeconds: number`
**Emits**: `@submit()`, `@answer(itemId: number, optionId: number, confidence: string)`
**Display**: Distraction-free exam interface.

### MockPacingBadge (`components/mock/MockPacingBadge.vue`)
**Props**:
- `currentQuestion: number`
- `totalQuestions: number`
- `elapsedSeconds: number`
- `totalSeconds: number`
**Display**: "On Pace" (green), "Behind" (amber), "Ahead" (blue) pill badge.

### MockSubmission (`components/mock/MockSubmission.vue`)
**Props**: `answeredCount: number`, `totalCount: number`, `flaggedCount: number`
**Emits**: `@confirm()`, `@cancel()`
**Display**: Confirmation modal with stats.

### MockReview (`components/mock/MockReview.vue`)
**Props**: `reviewData: PostMockReview`
**Display**: 6-tab interface.

### MockReviewTab (`components/mock/MockReviewTab.vue`)
**Props**: `tab: string`, `data: object`
**Display**: Tab-specific content renderer.

### MockHistory (`components/mock/MockHistory.vue`)
**Props**: `history: MockHistoryEntry[]`
**Display**: Timeline list with trend chart.

### MockForecast (`components/mock/MockForecast.vue`)
**Props**: `forecast: { predictedScore: number, gradeRange: string, quickWins: string[], ceiling: number }`
**Display**: Forecast insight card.

---

## 6.5 User Flows

### Flow: Take a Full Mock
1. Mock Centre Home -> tap "Full Mock" card
2. Setup wizard: Type (Full) -> Subject (Math) -> Topics (Full Syllabus) -> Duration (2h) -> Preview
3. Pre-flight card: "Make sure you have a quiet space. No notes." -> "I'm Ready"
4. Ceremonial transition (3-5s): fade to black, "Exam Hall" text, desk appears, timer starts
5. Exam hall: Question 1 of 40, timer counting down, answer questions one by one
6. Student flags question 12 for review, tags question 15 as "guessed"
7. Pacing badge shows "On Pace" through question 25, then "Behind" at question 30
8. Student opens nav grid, sees 3 unanswered + 2 flagged, navigates to flagged questions
9. Reviews and answers remaining questions
10. "End Exam" -> confirmation modal: "38 answered, 2 unanswered, 2 flagged. Submit?"
11. Student taps "Submit"
12. Processing (1-2s) -> Score reveal animation: 67% counts up
13. "Building zone. 5 points higher than last time!"
14. 6-tab review available for detailed analysis

### Flow: Recovery Mock After Bad Session
1. Coach Hub shows repair-required state with recovery mock suggestion
2. Student taps "Recovery Mock" -> mock setup pre-configured with Recovery type
3. Setup: easier defaults pre-selected, warm amber theme
4. Pre-flight: encouraging message "This is about rebuilding confidence"
5. Exam hall: slightly easier questions, more generous timing
6. Debrief: emphasizes correct answers, "You got {n} right!" before showing missed items
7. Coach note: "You're stronger than you think. Let's keep building."

---

## 6.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `create_mock_blueprint` | `config: MockConfig` | `MockBlueprint` | Plan mock exam structure |
| `start_mock` | `blueprintId: number` | `Session` | Start mock as session |
| `submit_mock_answer` | `sessionId: number, itemId: number, optionId: number, confidence: string` | `boolean` | Submit answer in mock |
| `complete_mock` | `sessionId: number` | `PostMockReview` | Finalize and generate review |
| `get_mock_review` | `sessionId: number` | `PostMockReview` | Fetch post-mock review data |
| `get_mock_history` | `studentId: number, subjectId?: number` | `MockHistory` | Get mock history |
| `get_readiness_score` | `studentId: number, subjectId: number` | `ReadinessScore` | Get current readiness |
| `get_mock_forecast` | `studentId: number, subjectId: number` | `object` | Get predictive forecast |

---

## 6.7 Store State

### Mock Store (merged into `stores/session.ts` or dedicated `stores/mock.ts`)

```typescript
interface MockState {
  // Mock centre home
  readinessScore: ReadinessScore | null;
  mockHistory: MockHistory | null;
  forecast: object | null;
  homeLoading: boolean;

  // Setup
  mockConfig: Partial<MockConfig>;
  setupStep: number;

  // Exam hall
  examActive: boolean;
  examSessionId: number | null;
  examItems: SessionItem[];
  currentExamQuestionIndex: number;
  examTimerSeconds: number;
  flaggedQuestions: Set<number>;
  confidenceTags: Map<number, string>;
  answeredCount: number;

  // Pacing
  pacingStatus: 'on_pace' | 'behind' | 'ahead';

  // Review
  postMockReview: PostMockReview | null;
  activeReviewTab: string;
  reviewLoading: boolean;
}
```

---

## 6.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Exam hall transition | 5-phase cinematic: fade black (500ms) -> text glow (500ms) -> desk build (1500ms) -> timer appear (500ms) -> "Begin" (500ms) | Door close (`transitions/door-close.mp3`), paper shuffle (`transitions/paper-shuffle.mp3`), clock start (`transitions/clock-tick.mp3`) |
| Question change | Cross-fade, 200ms | Page turn (`feedback/page-turn.mp3`) |
| Flag toggle | Flag icon fills/unfills with bounce, 200ms | Flag tap (`feedback/flag.mp3`) |
| Pacing badge change | Badge color transition + text slide, 300ms | None |
| Timer warning (25%) | Timer turns amber, 200ms | None |
| Timer warning (10%) | Timer turns red + starts pulsing | Subtle ticking increases (`feedback/timer-urgent.mp3`) |
| Time's up | Timer flashes, full-screen overlay "Time's Up", 800ms | Time's up bell (`feedback/times-up.mp3`) |
| Submission confirm | Modal slide in, 300ms | None |
| Score reveal | Score number counts from 0 to final, 2000ms, with accelerating ease | Score count sound (`celebration/score-count.mp3`) |
| Score reveal (good) | Confetti burst after number settles | Celebration fanfare (`celebration/mock-good.mp3`) |
| Score reveal (okay) | Green-up arrow slides in | Encouragement chime (`celebration/mock-okay.mp3`) |
| Score reveal (poor) | Warm glow surrounds score | Supportive tone (`feedback/encouragement.mp3`) |
| Tab switch | Content cross-fade, 200ms | None |
| Review question expand | Accordion expand, 200ms | None |

---
---

# DOMAIN 7: WRONG ANSWER INTELLIGENCE

## 7.1 Domain Overview

Wrong Answer Intelligence transforms every incorrect answer into a structured learning opportunity. It goes far beyond "here's the right answer" to diagnose WHY the student got it wrong, classify the error into a taxonomy, track error patterns over time, and provide targeted repair. The domain includes the 10-part progressive review card, thought replay, self-report, contrast teaching, micro-proof questions, 8-axis error fingerprinting, 7-profile academic analyst, error trend visualization, and the Mistake Lab page.

**Entry points**: Inline after wrong answer (QuestionFeedback), Session Debrief "Review Mistakes" link, Mistake Lab page, Academic Analyst page.

---

## 7.2 Features

### F7.1 Ten-Part Progressive Review Card
As specified in Domain 3 (F3.6), the full 10-part review:

| Part | Title | Content Source |
|------|-------|---------------|
| 1 | What You Chose | `selectedOptionId` -> option text, highlighted in red |
| 2 | Why It Looked Right | `WrongAnswerDiagnosis.diagnosisSummary` -- explains the appeal |
| 3 | Why It's Wrong | `WrongAnswerDiagnosis.primaryDiagnosis` -- precise error explanation |
| 4 | The Correct Answer | Correct option highlighted green + explanation |
| 5 | Why Others Are Wrong | Brief explanation of each distractor |
| 6 | The Mistake Type | `WrongAnswerDiagnosis.errorType` with icon, name, description |
| 7 | What Your Brain Did | Thought replay -- speculative reconstruction |
| 8 | The Lesson | One-sentence takeaway from this mistake |
| 9 | Pattern Check | Error history lookup -- "This is the {n}th time you've made a {type} error" |
| 10 | Repair Action | Recommended next step: drill, contrast teaching, or worked example |

- Progressive disclosure: parts 1-4 shown by default, parts 5-7 expandable, parts 8-10 revealed on "Show Full Analysis"
- Each part is a collapsible card section with header icon

### F7.2 Thought Replay
- AI-generated reconstruction of the student's probable thinking process
- Format: numbered thought steps ("You probably thought: 1. Saw X in the question, 2. Connected it to Y, 3. Chose Z because...")
- "Was this accurate?" feedback prompt (Yes / Partly / No)
- If "No": student can describe their actual thinking (free text)
- Feeds back into the analyst to improve future reconstructions

### F7.3 "Why Did You Pick It?" Self-Report
- Post-wrong-answer prompt: "Why did you choose {option}?"
- Predefined chip options:
  - "It looked similar to the right answer"
  - "I confused two concepts"
  - "I forgot the formula/rule"
  - "I rushed"
  - "I panicked under pressure"
  - "I misread the question"
  - "I guessed randomly"
  - "I second-guessed myself"
  - Custom text input option
- Selection feeds into error classification and academic analyst

### F7.4 Contrast Teaching (Side-by-Side)
- Two-column comparison of the wrong concept vs the correct concept
- Left: "What you thought" (the misconception or wrong approach)
- Right: "What's actually true" (the correct concept or approach)
- Key differences highlighted with colored markers
- Visual: clean table or side-by-side cards with connecting lines showing differences
- "Can you see the difference?" confirmation prompt at bottom

### F7.5 Micro-Proof Questions
- After review, a quick 1-2 question quiz testing the same concept
- Simpler variant of the original question
- Purpose: confirm the student actually understood the correction
- Correct: "Recovery confirmed!" -- updates student model
- Incorrect: "Not yet. Let's try one more approach." -- additional scaffolding

### F7.6 Error Fingerprint (8-Axis)
Every wrong answer is classified across 8 axes:

| Axis | Values | Purpose |
|------|--------|---------|
| Topic | Which topic the error relates to | Topic-level error tracking |
| Family | Question family (related question patterns) | Family-level pattern detection |
| Error Type | 10 types (knowledge_gap, conceptual_confusion, etc.) | Categorize the nature of the error |
| Distractor Type | Distractor intent (common_misconception, partial_application, etc.) | Why the wrong answer was tempting |
| Pressure State | Calm, moderate, high pressure | How pressure affected the error |
| Confidence | Sure but wrong, not sure, guessed | Confidence calibration |
| Prerequisite Gap | Whether a prerequisite knowledge gap caused this | Dependency chain analysis |
| Severity | Minor, moderate, critical | How impactful this error is |

### F7.7 Academic Analyst (7 Live Profiles)

| Profile | Tracks | Visualization |
|---------|--------|---------------|
| Concept Weakness | Topics where conceptual understanding fails | Heat map by topic |
| Misconception | Persistent wrong mental models | Misconception list with frequency |
| Reasoning | Logic and reasoning errors | Pattern chart |
| Distractor Vulnerability | Susceptibility to specific distractor types | Radar chart |
| Pressure | Performance collapse under pressure | Calm vs pressure comparison |
| Transfer | Inability to apply knowledge in new contexts | Transfer success rate by topic |
| Recovery | How quickly errors are corrected after intervention | Recovery rate chart |

- Each profile auto-updates after every wrong answer
- Dashboard view shows all 7 profiles as cards
- Tap any profile for deep-dive view with historical trends

### F7.8 Error Trend Visualization
- Line chart: error rate over time (by week/month)
- Stacked area chart: error types over time (shows shifting error composition)
- "Same mistake, different clothes" section: groups errors that have the same root cause but appear in different questions
- Bar chart: error count by type (aggregated)
- Filter by: subject, topic, time range, error type

### F7.9 "Same Mistake, Different Clothes"
- Identifies when a student makes the same conceptual error across different questions
- Groups these questions together with connecting visual
- Shows: the common misconception, all questions where it appeared, the student's answers
- "You've fallen for this {n} times in different forms"
- Repair link: targeted drill that addresses the root misconception

### F7.10 Mistake Lab Page
The main mistakes management page:

**Sections**:
| Section | Content |
|---------|---------|
| Header Stats | Total mistakes, most common error type, repair rate, trending errors |
| Errors by Type | Grouped cards for each error type with count + recent examples |
| Errors by Topic | Topic-sorted mistake list with per-topic error count |
| Error Patterns | "Same mistake, different clothes" groups |
| Recent Mistakes | Chronological list of recent wrong answers |
| Repair Drill Launcher | CTA to start a targeted repair session based on mistake analysis |

### F7.11 Repair Drill Launcher
- One-click session generation targeting the student's most impactful mistakes
- Config based on: most frequent error type, most affected topics, recent patterns
- Session type: repair (shorter, focused, with contrast teaching blocks)
- "Fix {n} mistakes in ~{m} minutes" CTA

### F7.12 Recovery Mode After Frustration
- Detects frustration signals: 3+ consecutive wrong answers, rapid incorrect submissions, declining confidence
- Triggers recovery mode: warm amber theme, easier questions, encouraging coach messages
- "Let's take a step back and build from what you know"
- Gradually increases difficulty as confidence rebuilds
- Does not explicitly tell student they're in recovery mode (avoids stigma)

---

## 7.3 Screens

### Screen: Mistake Lab (`pages/student/mistakes/index.vue`)
**Layout**: `student`
**Route**: `/student/mistakes/`

**Sections**: As described in F7.10.

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Mistake data loading | Skeleton cards |
| Normal | Mistakes loaded | Full layout with all sections |
| Empty | No mistakes recorded | Celebration: "No mistakes yet! Keep it up." (But also: "Mistakes are how you learn -- they'll show up here.") |
| Error | Fetch failed | AppError with retry |

---

### Screen: Error Pattern Detail (`pages/student/mistakes/pattern/[id].vue`)
**Layout**: `student`
**Route**: `/student/mistakes/pattern/:id`

**Sections**:
| Section | Content |
|---------|---------|
| Pattern Header | Error type, frequency, affected topics |
| Example Questions | List of questions exhibiting this pattern |
| Root Cause | AI-generated explanation of underlying issue |
| Repair Plan | Recommended actions with launch buttons |
| Historical Trend | This error type count over time |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Pattern data loading | Skeleton |
| Normal | Data loaded | Full layout |
| Error | Fetch failed | AppError |

---

## 7.4 Components

### WrongAnswerReview (`components/question/WrongAnswerReview.vue`)
(Detailed in Domain 3, section 3.4)

### MistakeClinicFlow (`components/question/MistakeClinicFlow.vue`)
(Detailed in Domain 3, section 3.4)

### ThoughtReplay (`components/mistakes/ThoughtReplay.vue`)
**Props**:
- `replaySteps: string[]` (numbered thought steps)
**Emits**: `@feedback(accurate: 'yes' | 'partly' | 'no')`, `@custom-explanation(text: string)`
**Display**: Numbered step list in speech-bubble style, "Was this accurate?" prompt with 3 buttons.

### SelfReportChips (`components/mistakes/SelfReportChips.vue`)
**Props**:
- `chips: string[]` (predefined reasons)
- `allowCustom: boolean`
**Emits**: `@select(reasons: string[])`
**Display**: Grid of selectable chips + optional text input for custom reason.

### ContrastTeaching (`components/mistakes/ContrastTeaching.vue`)
**Props**:
- `wrongConcept: { title: string, description: string, example: string }`
- `correctConcept: { title: string, description: string, example: string }`
- `differences: { key: string, wrong: string, correct: string }[]`
**Emits**: `@understood()`
**Display**: Two-column card layout, differences highlighted, "Can you see the difference?" prompt.

### MicroProof (`components/mistakes/MicroProof.vue`)
**Props**:
- `proofQuestion: Question`
- `proofOptions: QuestionOption[]`
**Emits**: `@result(correct: boolean)`
**Display**: Simplified QuestionCard with "Prove you got it" header.

### ErrorFingerprint (`components/mistakes/ErrorFingerprint.vue`)
**Props**:
- `fingerprint: { topic: string, family: string, errorType: string, distractorType: string, pressureState: string, confidence: string, prerequisiteGap: boolean, severity: string }`
**Display**: 8-axis display as pill badges or mini radar chart.

### AcademicAnalyst (`components/mistakes/AcademicAnalyst.vue`)
**Props**:
- `profiles: { profileType: string, data: object, trend: object }[]`
**Display**: 7 profile cards in a grid, each with title, key metric, mini visualization, trend arrow.

### ErrorTrendChart (`components/mistakes/ErrorTrendChart.vue`)
**Props**:
- `trendData: { date: string, errorType: string, count: number }[]`
- `chartType: 'line' | 'stacked_area' | 'bar'`
**Display**: D3-rendered chart.

### SameMistakeDifferentClothes (`components/mistakes/SameMistakeDifferentClothes.vue`)
**Props**:
- `groups: { misconception: string, questions: { stem: string, studentAnswer: string, date: string }[] }[]`
**Display**: Grouped cards with connecting line visual, misconception label, repair link.

### RepairDrillLauncher (`components/mistakes/RepairDrillLauncher.vue`)
**Props**:
- `targetErrorType: string`
- `targetTopics: number[]`
- `estimatedMinutes: number`
**Emits**: `@launch()`
**Display**: CTA card: "Fix {n} mistakes in ~{m} minutes", topic chips, "Start Repair" button.

---

## 7.5 User Flows

### Flow: Inline Wrong Answer Review
1. Student answers MCQ incorrectly during session
2. QuestionFeedback shows: "Not quite" + correct answer
3. Parts 1-4 of wrong answer review expand automatically below feedback
4. Student reads why their answer was wrong and why the correct answer is right
5. "Show Full Analysis" link reveals parts 5-10
6. Part 10 offers: "Practice this concept now" (micro-proof) or "Add to repair list"
7. Student taps "Practice now" -> MicroProof question appears inline
8. Answers correctly -> "Recovery confirmed!" -> continue session

### Flow: Mistake Lab Deep Dive
1. Sidebar -> "Mistakes" -> `/student/mistakes/`
2. Mistake Lab shows: 47 total mistakes, most common: "conceptual confusion" (18x)
3. Student taps "Conceptual Confusion" error type card
4. Pattern detail page: shows all 18 questions grouped by topic
5. "Same mistake, different clothes" section: 3 groups identified
6. Student taps a group: sees the root misconception + all affected questions
7. "Fix this pattern" CTA -> repair drill launcher with pre-configured settings
8. Student starts repair session

### Flow: Frustration Recovery
1. Student is in session, gets 4 questions wrong in a row
2. System detects frustration signal
3. Next question quietly shifts to easier (recovery mode activated)
4. UI subtly warms (amber tint appears, softer shadows)
5. Coach strip shows: "Let's build from what you know."
6. Questions get progressively easier until student answers 2-3 correctly
7. Confidence rebuilds, difficulty gradually returns to normal
8. Recovery mode deactivates without student being explicitly told

---

## 7.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `get_wrong_answer_diagnosis` | `answerId: number` | `WrongAnswerDiagnosis` | Get diagnosis for a wrong answer |
| `list_wrong_answer_diagnoses` | `studentId: number, limit: number, offset: number` | `WrongAnswerDiagnosis[]` | List recent diagnoses |
| `get_error_fingerprint` | `answerId: number` | `object` (8-axis) | Get error fingerprint |
| `get_error_trends` | `studentId: number, timeRange: string` | `object` (trend data) | Error trends over time |
| `get_academic_analyst_profiles` | `studentId: number` | `object[]` (7 profiles) | Get all analyst profiles |
| `get_same_mistake_groups` | `studentId: number` | `object[]` | Get "same mistake, different clothes" groups |
| `submit_self_report` | `answerId: number, reasons: string[]` | `boolean` | Submit self-report reasons |
| `submit_thought_replay_feedback` | `answerId: number, accurate: string` | `boolean` | Feedback on thought replay |
| `get_repair_drill_config` | `studentId: number` | `object` | Get optimal repair drill config |

---

## 7.7 Store State

No dedicated store. Wrong answer data is ephemeral (loaded per-question from IPC) or managed within the Mistake Lab page component. The Session Store holds `answersThisSession` which includes wrong answer results.

For the Mistake Lab page, local component state:

```typescript
interface MistakeLabState {
  diagnoses: WrongAnswerDiagnosis[];
  errorTrends: object;
  analystProfiles: object[];
  sameMistakeGroups: object[];
  loading: boolean;
  error: string | null;
  filterErrorType: string | null;
  filterTopic: number | null;
  filterTimeRange: string;
}
```

---

## 7.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Wrong answer review expand | Each part slides in from right, stagger 100ms per part | None |
| "Show Full Analysis" | Remaining parts accordion expand, 300ms | None |
| Thought replay steps | Steps appear one by one (typewriter effect), 500ms per step | None |
| Self-report chip select | Chip fills with color + scale press, 150ms | Soft click (`feedback/tap.mp3`) |
| Contrast teaching reveal | Two columns slide in from opposite sides and meet at center, 600ms | None |
| Micro-proof correct | Green flash + checkmark + "Confirmed!" text bounce, 500ms | Recovery chime (`feedback/recovery.mp3`) |
| Micro-proof incorrect | Gentle shake, encourage retry, 300ms | Soft wrong (`feedback/wrong-soft.mp3`) |
| Error fingerprint display | 8 badges stagger appear in circle/row, 200ms each | None |
| Analyst profile update | Card value animates (counter roll), trend arrow slides in, 400ms | None |
| Error trend chart draw | Lines draw in from left to right, 800ms | None |
| Same Mistake group expand | Card expands, questions list fade in, 300ms | None |
| Recovery mode enter (subtle) | Background color shifts from neutral to warm amber, 2000ms (very slow, unnoticeable) | None |
| Recovery mode exit | Background color shifts back to neutral, 2000ms | None |
| Frustration detection | No visible animation (invisible to student) | None |

---
---

# DOMAIN 8: STUDENT MODEL & PROGRESS

## 8.1 Domain Overview

The Student Model & Progress domain visualizes the entire student's academic state: mastery maps, analytics dashboards, activity timelines, readiness gauges, and predictive insights. It provides the "what do I know?" and "am I ready?" views that complement the Coach Hub's "what should I do?" It includes an interactive mastery map with 8 states and drill-down from Subject to Skill level, filterable layers, dependency arrows, and a comprehensive analytics dashboard.

**Entry points**: Sidebar "Progress" nav item, Coach Hub quick stats tap, post-session debrief links, parent portal child detail.

---

## 8.2 Features

### F8.1 Mastery Map
Interactive hierarchical visualization of academic mastery:

**Hierarchy**: Subject -> Topic -> Subtopic -> Skills

**8 Mastery States** (displayed as colored nodes):
| State | Color | Icon | Description |
|-------|-------|------|-------------|
| Unseen | Gray (#D1D5DB) | Circle outline | Never attempted |
| Exposed | Light blue (#93C5FD) | Eye icon | Seen but not practiced |
| Emerging | Light yellow (#FDE68A) | Sprout icon | Early learning, few attempts |
| Partial | Yellow (#FCD34D) | Half-circle | Some understanding, gaps remain |
| Fragile | Orange (#FB923C) | Warning triangle | Known but unstable, may decay |
| Stable | Green (#4ADE80) | Shield icon | Reliable under normal conditions |
| Robust | Dark green (#22C55E) | Double shield | Strong, resists pressure |
| Exam Ready | Blue (#2563EB) | Star icon | Ready for exam conditions |

**Drill-down behavior**:
- Subject level: large cards showing subject name, overall mastery %, state distribution pie chart
- Topic level: grid of topic cards, each showing mastery badge, score, trend arrow
- Subtopic level: detailed cards with score, gap, memory strength, last seen
- Skill level: individual skill badges with state, score, evidence count

**Interactions**:
- Tap subject card -> drill into topics
- Tap topic card -> drill into subtopics
- Tap subtopic -> drill into skills
- Breadcrumb navigation for hierarchy traversal
- Zoom/pan on map view (if graph visualization mode)

### F8.2 Filterable Layers
The mastery map can be viewed through different lenses:

| Layer | Shows | Color Scheme |
|-------|-------|-------------|
| Mastery | Mastery state per node | 8 mastery state colors |
| Gap | Knowledge gap severity | Red (high gap) -> Green (no gap) |
| Memory | Memory strength/decay risk | Blue (strong) -> Red (decaying) |
| Readiness | Exam readiness per topic | Standard readiness band colors |
| Stability | How stable knowledge is | Green (stable) -> Amber (fragile) |

- Layer toggle buttons at top of map
- Smooth color transition when switching layers
- Legend shows color scale for active layer

### F8.3 Dependency Arrows
- Show prerequisite relationships between topics
- Arrows from prerequisite to dependent topic
- Color: green (prerequisite mastered), red (prerequisite weak), gray (not yet relevant)
- Toggle dependency arrows on/off
- When a topic is blocked by prerequisite, arrow pulses red

### F8.4 Analytics Dashboard
Comprehensive statistics view:

| Section | Content | Visualization |
|---------|---------|---------------|
| Readiness | Overall and per-subject readiness | Gauge + bars |
| Subjects | Per-subject mastery distribution | Stacked bar chart per subject |
| Trends | Mastery and accuracy over time | Line charts |
| Errors | Error type distribution and frequency | Pie chart + bar chart |
| Pressure | Performance under pressure vs calm | Comparison bars |
| Memory | Memory health across topics | Color-coded grid |
| Study Time | Time spent per subject/topic | Stacked area chart |

### F8.5 Activity Timeline
Chronological feed of all academic activities:
- Session completions with accuracy
- Diagnostic results
- Mock scores
- Mastery state changes
- Error pattern detections
- Coach plan adjustments
- Filter by: date range, activity type, subject
- Infinite scroll or paginated

### F8.6 Readiness Gauge
- Large circular gauge showing overall exam readiness (0-100%)
- Band labels: Not Ready (0-30%), Building (30-55%), Almost Ready (55-80%), Exam Ready (80-100%)
- Color transitions smoothly across the range
- Trend arrow (up/down/stable compared to last week)
- Per-subject mini gauges below main gauge

### F8.7 Exam Countdown Widget
- Days until next exam
- Visual urgency escalation (calm > moderate > urgent > critical)
- Expandable: shows exam details, readiness per subject, coverage gaps
- Integrated into dashboard header

### F8.8 Topic Status Cards
Per-topic summary cards showing:
- Topic name
- Mastery state badge
- Mastery score (0-100%)
- Gap score
- Memory strength indicator
- Trend arrow (improving/declining/stable)
- Last practiced date
- Next review date
- Priority score (higher = needs more attention)
- Quick-action buttons: "Practice", "Review", "Teach Me"

### F8.9 "If Exam Were Today"
Predictive analysis page:
- Predicted score range with confidence interval
- Predicted grade
- Per-subject predictions
- "What would help most" -- highest-impact actions
- Score breakdown: guaranteed marks (strong topics) + likely marks (developing) + risky marks (weak)
- Visual: stacked bar showing guaranteed/likely/risky segments

---

## 8.3 Screens

### Screen: Progress Home (`pages/student/progress/index.vue`)
**Layout**: `student`
**Route**: `/student/progress/`

**Sections**:
| Section | Content |
|---------|---------|
| Readiness Gauge | Large overall gauge + per-subject mini gauges |
| Exam Countdown | Days to exam widget |
| Subject Cards | Per-subject mastery summary cards |
| Quick Stats | Topics mastered, total practice time, current streak |
| Recent Activity | Last 5 activities |
| "If Exam Were Today" | Predicted score card |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Data fetching | Skeleton gauge + skeleton cards |
| Normal | Data loaded | Full layout |
| No Data | Brand new student, no activity | "Start your first session to see progress here" |
| Error | Fetch failed | AppError |

---

### Screen: Mastery Map (`pages/student/progress/mastery-map.vue`)
**Layout**: `student`
**Route**: `/student/progress/mastery-map`

**Sections**:
| Section | Content |
|---------|---------|
| Layer Toggle | 5 filter buttons (Mastery/Gap/Memory/Readiness/Stability) |
| Dependency Toggle | Toggle arrows on/off |
| Breadcrumb | Navigation hierarchy path |
| Map Area | Interactive node grid/graph |
| Legend | Color scale for active layer |
| Detail Panel | Slide-out panel when node selected |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Map data loading | Skeleton grid |
| Subject Level | Top level view | Large subject cards |
| Topic Level | Drilled into a subject | Topic node grid |
| Subtopic Level | Drilled into a topic | Subtopic cards |
| Skill Level | Drilled into a subtopic | Skill badges |
| Node Selected | A node is tapped | Detail slide-out panel |
| Error | Fetch failed | AppError |

---

### Screen: Analytics Dashboard (`pages/student/progress/analytics.vue`)
**Layout**: `student`
**Route**: `/student/progress/analytics`

**Sections**: As described in F8.4 (7 sections).

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Analytics computing | Skeleton charts |
| Normal | Data loaded | Full dashboard |
| Insufficient Data | <10 answers total | "Complete more sessions to see detailed analytics" |
| Error | Fetch failed | AppError |

---

### Screen: Activity Timeline (`pages/student/progress/history.vue`)
**Layout**: `student`
**Route**: `/student/progress/history`

**Sections**:
| Section | Content |
|---------|---------|
| Filters | Activity type, subject, date range |
| Timeline | Chronological activity feed |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Timeline loading | Skeleton list |
| Normal | Activities loaded | Timeline feed |
| Empty | No activities | "Your learning journey starts here" |
| Error | Fetch failed | AppError |

---

## 8.4 Components

### ReadinessGauge (`components/viz/ReadinessGauge.vue`)
**Props**: `score: BasisPoints`, `band: string`, `trend: 'up' | 'down' | 'stable'`
**Display**: Circular gauge with animated fill, band label, trend arrow.

### TopicStatusCard (`components/viz/TopicStatusCard.vue`)
**Props**: `topicState: StudentTopicState`, `topicName: string`
**Emits**: `@practice()`, `@review()`, `@teach()`
**Display**: Card with mastery badge, scores, trend, action buttons.

### MasteryBadge (`components/viz/MasteryBadge.vue`)
**Props**: `state: MasteryState`
**Display**: Colored circle/badge with state-specific icon and label.

### KnowledgeMap (`components/viz/KnowledgeMap.vue`)
**Props**:
- `nodes: { id: number, name: string, level: string, state: MasteryState, score: BasisPoints, parentId: number | null }[]`
- `dependencies: { from: number, to: number, satisfied: boolean }[]`
- `layer: 'mastery' | 'gap' | 'memory' | 'readiness' | 'stability'`
- `showDependencies: boolean`
**Emits**: `@node-click(nodeId: number)`, `@drill-down(nodeId: number)`
**Display**: Interactive node graph/grid with colored nodes, dependency arrows, zoom/pan.

### TrendLine (`components/viz/TrendLine.vue`)
**Props**: `data: { date: string, value: number }[]`, `color: string`
**Display**: D3 line chart.

### TrendArrow (`components/viz/TrendArrow.vue`)
**Props**: `direction: 'up' | 'down' | 'stable'`, `magnitude: number`
**Display**: Colored arrow icon (green up, red down, gray horizontal).

### ComparisonCard (`components/viz/ComparisonCard.vue`)
**Props**: `label: string`, `valueBefore: number`, `valueAfter: number`
**Display**: Before/after bar with delta indicator.

### BarChart (`components/viz/BarChart.vue`)
**Props**: `data: { label: string, value: number, color?: string }[]`, `orientation: 'horizontal' | 'vertical'`
**Display**: D3 bar chart.

### RadarChart (`components/viz/RadarChart.vue`)
**Props**: `dimensions: { name: string, value: number, max: number }[]`
**Display**: D3 spider/radar chart.

### HeatMap (`components/viz/HeatMap.vue`)
**Props**: `data: { row: string, col: string, value: number }[]`, `colorScale: string[]`
**Display**: D3 heat map grid.

---

## 8.5 User Flows

### Flow: Review Progress After Session
1. Session debrief -> "View Full Progress" link
2. Route to `/student/progress/`
3. Readiness gauge shows 62% (Building zone)
4. Subject cards: Math at 58%, Science at 67%
5. "If Exam Were Today" card: "Predicted: C+ (could reach B with focused effort)"
6. Student taps Math subject card -> route to mastery map at topic level
7. Sees topic grid: Algebra (Stable), Geometry (Fragile), Statistics (Emerging)
8. Taps Geometry -> drills into subtopics
9. Sees "Angles" (Partial), "Triangles" (Fragile), "Circles" (Unseen)
10. Taps "Triangles" -> sees skill-level detail + "Practice" button

### Flow: Mastery Map with Layers
1. Route to `/student/progress/mastery-map`
2. Default view: Mastery layer at subject level
3. Student taps "Memory" layer toggle
4. Node colors shift to memory-strength scheme (blue -> red)
5. Several topics show red (decaying memory)
6. Student toggles "Dependencies" on
7. Arrows appear between topics, some red (prerequisite weak)
8. Student identifies a prerequisite blocker -> taps node -> detail panel shows "This topic requires {X} which is at Fragile state"

---

## 8.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `get_learner_truth_snapshot` | `studentId: number` | `LearnerTruthSnapshot` | Get full student state |
| `get_student_topic_states` | `studentId: number, subjectId?: number` | `StudentTopicState[]` | Get all topic states |
| `get_student_topic_state` | `studentId: number, topicId: number` | `StudentTopicState` | Get single topic state |
| `get_student_dashboard` | `studentId: number` | `StudentDashboard` | Get dashboard summary |
| `get_topic_dependencies` | `subjectId: number` | `{ from: number, to: number }[]` | Get prerequisite links |
| `get_activity_timeline` | `studentId: number, limit: number, offset: number, filters: object` | `object[]` | Get activity feed |
| `get_study_time_stats` | `studentId: number, timeRange: string` | `object` | Get study time breakdown |
| `get_exam_prediction` | `studentId: number, subjectId: number` | `object` | Get "if exam were today" prediction |

---

## 8.7 Store State

### Student Store (`stores/student.ts`)

```typescript
interface StudentState {
  // Learner truth (central student model)
  learnerTruth: LearnerTruthSnapshot | null;
  learnerTruthLoading: boolean;
  learnerTruthError: string | null;

  // Topic states cache
  topicStates: Map<number, StudentTopicState>;
  topicStatesLoading: boolean;

  // Dashboard
  dashboard: StudentDashboard | null;
  dashboardLoading: boolean;

  // Mastery map
  currentMapLevel: 'subject' | 'topic' | 'subtopic' | 'skill';
  currentMapParentId: number | null;
  mapLayer: 'mastery' | 'gap' | 'memory' | 'readiness' | 'stability';
  showDependencies: boolean;
  selectedNodeId: number | null;

  // Activity timeline
  activities: object[];
  activitiesLoading: boolean;
  activityFilters: { type: string | null, subject: number | null, dateRange: string };
  activitiesHasMore: boolean;

  // Exam prediction
  examPrediction: object | null;
  predictionLoading: boolean;
}
```

---

## 8.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Readiness gauge fill | Gauge fills from 0 to target value with ease-out, 1200ms | None |
| Readiness gauge band change | Band label and color pulse transition, 600ms | Level-change tone (`feedback/band-change.mp3`) |
| Mastery map node appear | Nodes stagger fade-in with slight bounce, 150ms each, 30ms stagger | None |
| Layer switch | Node colors cross-fade to new layer scheme, 400ms | None |
| Dependency arrows appear | Arrows draw in from source to target, 300ms each | None |
| Drill-down | Current view zooms into selected node area, children nodes expand out, 500ms | Drill-down whoosh (`transitions/drill-down.mp3`) |
| Drill-up (breadcrumb) | Current view zooms out, parent view restores, 500ms | None |
| Topic card hover | Subtle lift (translate-y -2px, shadow increase), 150ms | None |
| Mastery state change | Badge animates from old state to new (color shift, icon morph), 600ms | Mastery change tone (`feedback/mastery-change.mp3`) |
| Trend chart draw | Lines draw left-to-right, 800ms | None |
| Activity timeline item appear | Fade in from left, stagger, 200ms each | None |
| Prediction card reveal | Card slides up + fade, 400ms | None |
| "If exam were today" bar | Stacked segments grow from left, 1000ms total | None |

---
---

# DOMAIN 9: JOURNEY MODE

## 9.1 Domain Overview

Journey Mode is the long-term goal-based learning path. Students set an exam goal (exam, date, target grade), complete an initial diagnostic, and then follow a personalized journey map through 5 phases (Stabilize -> Build -> Strengthen -> Condition -> Ready). The visual journey map shows stations (topics/checkpoints) with mastery/lock states, and the system adapts the route based on performance. Journey Mode is where daily coaching activity connects to long-term exam preparation.

**Entry points**: Onboarding completion (first journey creation), sidebar "Journey" nav item, Coach Hub journey-related directives.

---

## 9.2 Features

### F9.1 Onboarding Flow
New student journey creation sequence:
1. **Set Mission**: Select target exam (e.g., BECE Mathematics), set exam date, set target grade
2. **Diagnostic**: Complete initial diagnostic (routes to Diagnostic System)
3. **Map Reveal**: Diagnostic results generate personalized journey map, revealed with ceremony

### F9.2 Journey Map Visual Path
- Visual representation of the learning journey as a path with stations
- Horizontal scrollable path (or vertical timeline)
- Stations are nodes on the path representing topics, checkpoints, or milestones
- Path curves and bends to create visual interest (not a straight line)
- Current position indicator (student avatar on path)
- Completed stations: filled, colored by mastery level achieved
- Current station: pulsing ring, enlarged
- Upcoming stations: visible but muted
- Locked stations: padlock icon, grayed out (prerequisites not met)
- Milestone stations: larger, starred (mini-mocks, phase transitions)
- Subject-specific visual identity: Math (blue path), Science (green path), English (purple path)

### F9.3 Five Journey Phases

| Phase | Name | Color | Purpose | Activities |
|-------|------|-------|---------|------------|
| 1 | Stabilize | Amber | Fix critical gaps, establish foundation | Gap repair, prerequisite work, confidence building |
| 2 | Build | Blue | Grow knowledge breadth | New topic introduction, practice, teach sessions |
| 3 | Strengthen | Green | Deepen understanding, fix fragilities | Deep practice, contrast teaching, error repair |
| 4 | Condition | Purple | Exam conditioning, pressure training | Timed drills, mini-mocks, pressure sessions |
| 5 | Ready | Gold | Final preparation | Full mocks, targeted revision, confidence cementing |

- Phase transitions are ceremonial (animation + coach message)
- Each phase has distinct visual atmosphere on the journey map (background color/pattern changes)
- Phase progress bar visible on journey map

### F9.4 Station States

| State | Visual | Meaning |
|-------|--------|---------|
| Locked | Padlock, gray | Prerequisites not met |
| Available | Colored outline, unlocked icon | Ready to attempt |
| In Progress | Pulsing ring, partial fill | Currently working on |
| Passed | Filled green/blue, checkmark | Completed at required level |
| Mastered | Gold star, glowing | Exceeded requirements |
| Failed | Red outline, retry icon | Attempted but not passed, needs retry |
| Skipped | Dashed outline | Bypassed by route mode change |
| Review Due | Clock icon overlay | Passed but memory review needed |

### F9.5 Route Modes

| Mode | Description | Behavior |
|------|------------|----------|
| Steady | Balanced pace, all topics covered | Normal progression through all stations |
| Intense | Aggressive pace, prioritize weak areas | Skip strong topics, focus on gaps |
| Recovery | Gentle pace after absence/struggle | Easier stations, more review, confidence focus |
| Custom | Student-defined priorities | Student can reorder/skip stations |
| Rescue | Emergency mode near exam | Only highest-impact topics, skip non-essential |

- Route mode selector on journey settings
- Changing route mode re-plans remaining stations
- Visual: path changes (steady = smooth curves, intense = direct lines, recovery = wider gentle curves)

### F9.6 Mini-Mock & Challenge Surfaces
- Mini-mock stations: timed quiz assessing recent phase learning
- Challenge stations: harder-than-normal questions for engagement
- Both appear as special station types on the journey map
- Results feed back into mastery model and journey progression

### F9.7 Recovery/Re-Entry After Absence
- If student absent >3 days, journey activates recovery protocol
- Memory scan: quick check on recently learned topics
- Re-entry plan: identify what may have decayed, plan review stations
- Journey map highlights stations that may need re-visiting (memory warning icons)
- Coach message: "You've been away. Let's see what stuck and what needs refreshing."

### F9.8 Archive & History
- Completed journey phases archived
- View historical journey maps (past journeys)
- Journey statistics: total stations completed, accuracy trend, time spent

### F9.9 Final Readiness
- Special view at journey end (all phases complete or exam imminent)
- "You're as ready as you can be" celebration if Exam Ready band
- Final confidence-building activities
- Last-minute revision suggestions
- Calm, confident atmosphere

### F9.10 Nine Progress Dimensions
Journey tracks progress across 9 dimensions:
1. Knowledge breadth (topics covered)
2. Knowledge depth (mastery level)
3. Speed (fluency)
4. Accuracy (correctness)
5. Pressure resilience (performance under time constraints)
6. Memory stability (retention over time)
7. Transfer ability (applying knowledge to new contexts)
8. Error reduction (fewer mistakes over time)
9. Consistency (stable performance across sessions)

### F9.11 Subject-Specific Visual Identity
- Each subject has a unique visual theme on the journey map:
  - Mathematics: blue palette, geometric shapes, grid backgrounds
  - Science: green palette, organic shapes, molecule patterns
  - English: purple palette, flowing lines, book motifs
  - Social Studies: earth tones, map-style visuals
- Stations inherit subject colors
- Path decorations match subject theme

---

## 9.3 Screens

### Screen: Journey Home (`pages/student/journey/index.vue`)
**Layout**: `student`
**Route**: `/student/journey/`

**Sections**:
| Section | Content |
|---------|---------|
| Journey Header | Exam name, target grade, exam date, days remaining |
| Phase Indicator | Current phase with progress bar |
| Route Mode | Current route mode badge + change button |
| Journey Map | Interactive visual path with stations |
| Current Station Card | Expanded detail of current/next station |
| Dimension Progress | 9-dimension mini radar chart |
| Quick Stats | Stations completed, current streak, phase progress |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Journey data loading | Skeleton map + cards |
| No Journey | No journey created | Onboarding CTA: "Start Your Journey" |
| Active | Journey in progress | Full layout with map |
| Phase Transition | Moving to next phase | Ceremonial phase transition screen |
| Recovery Entry | Returning after absence | Recovery banner + memory scan prompt |
| Journey Complete | All phases done | Final readiness celebration |
| Error | Fetch failed | AppError |

---

### Screen: Journey Station (`pages/student/journey/station/[id].vue`)
**Layout**: `student`
**Route**: `/student/journey/station/:id`

**Sections**:
| Section | Content |
|---------|---------|
| Station Header | Station name, type, phase, mastery target |
| Station Status | Current state (locked/available/in progress/passed/mastered) |
| Requirements | What's needed to pass this station |
| Practice Button | "Start Practice" -> routes to session |
| Related Content | Teach mode link, glossary entries, past questions |
| Mastery Progress | Progress toward station mastery target |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Station data loading | Skeleton |
| Locked | Prerequisites not met | Lock icon, list of prerequisites needed |
| Available | Ready to start | Full station detail with "Start" CTA |
| In Progress | Currently working | Progress indicator + "Continue" CTA |
| Passed | Completed | Green status, stats, "Move to Next" CTA |
| Mastered | Exceeded requirements | Gold status, celebration badge |
| Error | Fetch failed | AppError |

---

## 9.4 Components

### JourneyMap (`components/modes/journey/JourneyMap.vue`)
**Props**:
- `stations: Station[]`
- `currentStationId: number`
- `phase: JourneyPhase`
- `subjectTheme: string`
**Emits**: `@station-click(stationId: number)`
**Display**: SVG-based visual path with station nodes, scroll/pan, current position indicator.

### JourneyStation (`components/modes/journey/JourneyStation.vue`)
**Props**:
- `station: Station`
- `state: string`
- `isCurrentt: boolean`
**Emits**: `@click()`
**Display**: Station node on map: icon, state-specific styling, pulse if current.

### JourneyMission (`components/modes/journey/JourneyMission.vue`)
**Props**:
- `mission: CoachMission`
- `stationName: string`
**Emits**: `@start()`
**Display**: Mission card with station context, "Start" CTA.

### JourneyPhaseIndicator
**Props**: `phase: JourneyPhase`, `progress: BasisPoints`
**Display**: Phase badge with name, color, and progress bar.

---

## 9.5 User Flows

### Flow: First Journey Setup (Onboarding)
1. Student completes account creation, arrives at onboarding
2. Step 1: "What exam are you preparing for?" -> Select BECE Mathematics
3. Step 2: "When is your exam?" -> Date picker -> June 15, 2027
4. Step 3: "What grade do you want?" -> Select "B or above"
5. Step 4: "Let's find out where you stand" -> Route to diagnostic
6. Diagnostic completes -> results analyzed
7. Journey map generated -> Ceremonial reveal: path unrolls with stations appearing
8. "Your journey has 5 phases and 42 stations. Let's begin!"
9. Student arrives at Journey Home with Phase 1 (Stabilize) active

### Flow: Daily Journey Engagement
1. Login -> Coach Hub shows "Today's Mission" from journey plan
2. Mission is a station practice session
3. Student starts session -> completes session -> debrief
4. Station mastery updated -> if target met, station marked "Passed"
5. Next station becomes available (if it was locked by this station as prerequisite)
6. Return to journey map to see updated progress

### Flow: Phase Transition
1. Student completes last station in Stabilize phase
2. Ceremonial transition: "Phase 1 Complete! Moving to Build Phase."
3. Animated: Phase 1 stations all glow green, new section of path reveals Phase 2 stations
4. Coach message: "Your foundation is set. Now let's build your knowledge."
5. Journey map scrolls to show new stations

---

## 9.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `get_journey_map` | `studentId: number` | `JourneyMap` | Get full journey state |
| `get_station` | `stationId: number` | `Station` | Get station detail |
| `start_journey_session` | `stationId: number` | `Session` | Start session for a station |
| `get_journey_phase` | `studentId: number` | `JourneyPhase` | Get current phase |
| `set_route_mode` | `studentId: number, mode: RouteMode` | `JourneyMap` | Change route mode |
| `create_journey` | `studentId: number, examTarget: string, examDate: string, targetGrade: string` | `JourneyMap` | Create new journey |
| `get_journey_history` | `studentId: number` | `object[]` | Get completed journeys |
| `trigger_recovery_scan` | `studentId: number` | `object` | Run recovery memory scan |

---

## 9.7 Store State

```typescript
interface JourneyState {
  journeyMap: JourneyMap | null;
  stations: Station[];
  currentStationId: number | null;
  currentPhase: JourneyPhase | null;
  routeMode: RouteMode;
  journeyLoading: boolean;
  journeyError: string | null;
  showPhaseTransition: boolean;
  transitionFromPhase: string | null;
  transitionToPhase: string | null;
  dimensionProgress: { dimension: string, value: BasisPoints }[];
  isRecoveryMode: boolean;
}
```

---

## 9.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Journey map reveal (first time) | Path draws in from left, stations pop in along path with stagger, 3000ms total | Journey reveal fanfare (`celebration/journey-reveal.mp3`) |
| Station unlock | Lock icon shatters, station colors fill in, 500ms | Unlock sound (`feedback/station-unlock.mp3`) |
| Station pass | Station fills green, checkmark stamps in, 400ms | Pass chime (`feedback/station-pass.mp3`) |
| Station master | Gold glow + star animation, 600ms | Mastery fanfare (`celebration/station-master.mp3`) |
| Phase transition | Current phase glows, new path section unfurls with stations, 2500ms | Phase transition music (`transitions/phase-transition.mp3`) |
| Current position move | Avatar slides along path to new station, 800ms | Walk/slide sound (`transitions/avatar-move.mp3`) |
| Map scroll/pan | Smooth inertia-based scroll | None |
| Route mode change | Path morphs (curves change), stations reposition, 1000ms | Route change sound (`transitions/route-change.mp3`) |
| Recovery entry | Amber glow spreads across map from current position, 1500ms | Warm recovery tone (`transitions/recovery-enter.mp3`) |
| Memory warning on station | Clock icon pulses on station, gentle yellow glow, 2s cycle | None |

---
---

# DOMAIN 10: BEAT YESTERDAY

## 10.1 Domain Overview

Beat Yesterday is a daily micro-improvement system built on the philosophy "just be a little better than yesterday." It features a hero screen comparing yesterday's performance to today's target, a specialized 4-block session structure, 5 growth modes, micro-gain indicators, streaks, and anti-burnout detection. It's designed to be the most motivating daily touchpoint for students.

**Entry points**: Sidebar "Beat Yesterday" nav item, Coach Hub Beat Yesterday directive, daily notification.

---

## 10.2 Features

### F10.1 Hero Screen (Yesterday vs Today)
- Large comparison card: left side "Yesterday" with stats, right side "Today's Target" with goals
- Yesterday stats: questions answered, accuracy %, speed (avg time/question), streak
- Today's target: slightly better than yesterday on the active growth mode metric
- Visual: arrow or bridge graphic connecting yesterday to today's target
- Motivational copy: "Yesterday you got 8/10 correct. Today, let's aim for 9."

### F10.2 Four-Block Session Structure

| Block | Name | Duration | Mood | Colors | Purpose |
|-------|------|----------|------|--------|---------|
| 1 | Warm Start | 2-3 min | Calm, gentle | Soft greens/blues | Easy questions, activate knowledge |
| 2 | Core Climb | 5-8 min | Focused, steady | Blue/indigo | Main growth area questions |
| 3 | Speed Burst | 60 seconds | Intense, energetic | Orange/red | Rapid-fire, beat the clock |
| 4 | Finish Strong | 2-3 min | Positive, rewarding | Warm gold | Easy-medium, end on a high |

Each block has its own visual atmosphere that transitions between blocks.

### F10.3 Five Growth Modes

| Mode | Primary Metric | Target Logic | Description |
|------|---------------|-------------|-------------|
| Volume | Questions answered | +1-2 more questions than yesterday | "Do more" |
| Accuracy | Correct percentage | +5-10% accuracy improvement | "Get more right" |
| Speed | Average response time | 2-5 seconds faster per question | "Think faster" |
| Mixed | Composite of all three | Weighted improvement across all | "Overall growth" |
| Recovery | Confidence score | Rebuild after decline | "Gentle rebuild" |

- System selects optimal growth mode based on recent performance
- Student can override with manual selection
- Mode displayed as badge on hero screen

### F10.4 Micro-Gain Indicators
- Small floating indicators showing incremental improvements
- Format: "+2 questions", "-3s average", "+12% accuracy"
- Color: green for positive, amber for maintained, red for decline
- Appear next to relevant metrics throughout the session and on debrief
- Subtle bounce animation when gains appear

### F10.5 Daily Target Display
- Clear numeric target for the day
- Progress toward target shown as filling bar during session
- Target adjusts based on growth mode
- "Beat Yesterday" means exceeding, not just matching

### F10.6 Session Summary with Animated Gains
- Post-session comparison: Yesterday column vs Today column
- Each metric animates: yesterday number -> today number
- Gains highlighted with green pulse
- Losses shown gently with amber
- Overall verdict: "You Beat Yesterday!" (celebration) or "Almost! Try again tomorrow." (encouragement)

### F10.7 Weekly Trends
- 7-day chart showing daily performance across the active growth mode metric
- Streak indicator: consecutive days of beating yesterday
- Weekly summary: best day, total improvement, average performance
- Trend line showing trajectory

### F10.8 Growth Badges & Streaks
- Daily streak counter: consecutive days of beating yesterday
- Streak milestones: 3-day, 7-day, 14-day, 30-day badges
- Growth badges: "Volume Champion", "Speed Demon", "Accuracy Master"
- Badges displayed on hero screen and progress page

### F10.9 Recovery Mode
- Activated after 2+ days of not beating yesterday
- Reduces targets to achievable level
- Warm amber theme
- Encouraging messaging: "Everyone has off days. Let's rebuild."
- Auto-deactivates after student beats yesterday again

### F10.10 Anti-Burnout Detection
- Monitors for burnout signals: declining performance despite effort, session abandonment, decreasing session frequency
- If detected: suggests rest day, reduces intensity, switches to Recovery mode
- Coach message: "Your body and brain need rest too. How about a lighter day?"
- Does not force rest, but makes it easy to choose

### F10.11 Six Climb States

| State | Condition | Visual |
|-------|-----------|--------|
| Fresh Start | First day, no history | Welcome card, "Set your baseline today" |
| Warming Up | In Warm Start block | Calm colors, gentle progress |
| Climbing | In Core Climb block | Focused colors, steady progress |
| Sprinting | In Speed Burst block | Intense colors, timer prominent |
| Finishing | In Finish Strong block | Warm gold, positive energy |
| Summit | Session complete, yesterday beaten | Celebration, peak visual |

### F10.12 Teacher/Parent Dashboards
- Teacher view: class-level Beat Yesterday engagement, per-student streaks, who needs encouragement
- Parent view: child's Beat Yesterday streak, recent trends, celebration highlights
- Both views accessible from parent/admin portals

---

## 10.3 Screens

### Screen: Beat Yesterday Home (`pages/student/beat-yesterday/index.vue`)
**Layout**: `student`
**Route**: `/student/beat-yesterday/`

**Sections**:
| Section | Content |
|---------|---------|
| Hero Card | Yesterday vs Today comparison |
| Growth Mode Badge | Current mode + change option |
| Streak Counter | Consecutive days, badges |
| Start Climb CTA | Large "Start Today's Climb" button |
| Weekly Trends | 7-day mini chart |
| Recent Badges | Recently earned badges |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Data fetching | Skeleton hero card + chart |
| First Day | No yesterday data | "Set Your Baseline" card, no comparison |
| Normal | Yesterday data exists | Full hero card with comparison + CTA |
| Already Completed | Today's climb already done | Summary of today's results + "Go Again?" option |
| Streak Active | On a streak | Streak counter prominent, fire icon |
| Recovery Mode | Anti-burnout or recovery active | Warm amber theme, gentler targets |
| Error | Fetch failed | AppError |

---

### Screen: Beat Yesterday Climb (Session)
Uses SessionPlayer (Domain 4) with Beat Yesterday 4-block configuration.
**Route**: `/student/session/:id` (session type: beat_yesterday)

---

### Screen: Beat Yesterday Trends (`pages/student/beat-yesterday/trends.vue`)
**Layout**: `student`

**Sections**:
| Section | Content |
|---------|---------|
| Weekly Chart | 7-day performance line chart |
| Monthly Chart | 30-day trend chart |
| All-Time Stats | Total climbs, total beats, longest streak |
| Badge Wall | All earned badges |
| Mode History | Growth mode usage over time |

---

## 10.4 Components

### BeatYesterdayHome (`components/modes/beat-yesterday/BeatYesterdayHome.vue`)
**Props**: `studentId: number`
**Display**: Hero card, mode badge, streak, CTA, trends.

### DailyClimb (`components/modes/beat-yesterday/DailyClimb.vue`)
**Props**: `sessionId: number`, `yesterdayStats: object`, `todayTarget: object`
**Display**: 4-block session with Beat Yesterday-specific UI.

### MicroGainIndicator (`components/modes/beat-yesterday/MicroGainIndicator.vue`)
**Props**: `metric: string`, `value: string`, `direction: 'up' | 'down' | 'stable'`
**Display**: Small floating chip: "+2 questions" in green, or "-1s" in green for speed.

### ClimbTrends (`components/modes/beat-yesterday/ClimbTrends.vue`)
**Props**: `weeklyData: object[]`, `monthlyData: object[]`
**Display**: Trend charts with streak overlay.

### ComparisonCard (reused from viz domain)
Yesterday vs Today comparison with animated value transitions.

---

## 10.5 User Flows

### Flow: Daily Beat Yesterday Session
1. Student opens Beat Yesterday Home
2. Hero card: "Yesterday: 12 questions, 75% accuracy. Today's target: 13 questions, 75% accuracy." (Volume mode)
3. Streak shows: "4-day streak" with fire icon
4. Student taps "Start Today's Climb"
5. Session brief: "4 blocks: Warm Start, Core Climb, Speed Burst, Finish Strong"
6. Block 1 (Warm Start): 3 easy questions, all correct. Micro-gain: "+3 correct"
7. Block 2 (Core Climb): 6 medium questions, 4 correct. Running total displayed.
8. Block 3 (Speed Burst): 3-2-1-GO! 60 seconds, answers 5 questions.
9. Block 4 (Finish Strong): 3 questions, ends with correct answer.
10. Summary: Yesterday 12/75% -> Today 14/78%. "You Beat Yesterday!" Celebration.
11. Streak updates: "5-day streak! New badge: Consistency Champion!"

### Flow: Anti-Burnout Detection
1. Student has not beaten yesterday for 3 consecutive days
2. Performance declining despite regular sessions
3. Next visit to Beat Yesterday Home shows recovery mode
4. Hero card: warm amber, "Let's take it easy today. Aim for 10 questions at a comfortable pace."
5. Growth mode badge shows "Recovery"
6. Session has gentler targets, more encouraging messages
7. Student beats the reduced target easily -> confidence rebuilds
8. Next day: targets increase slightly, recovery mode continues until beaten normally

---

## 10.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `get_daily_target` | `studentId: number` | `DailyTarget` | Get today's Beat Yesterday target |
| `start_climb` | `studentId: number` | `Session` | Start Beat Yesterday session |
| `get_climb_state` | `studentId: number` | `ClimbState` | Get current climb state |
| `get_daily_performance` | `studentId: number, date?: string` | `DailyPerformanceProfile` | Get performance for a date |
| `get_beat_yesterday_history` | `studentId: number, days: number` | `DailyPerformanceProfile[]` | Get historical performance |
| `get_growth_mode` | `studentId: number` | `GrowthMode` | Get current growth mode |
| `set_growth_mode` | `studentId: number, mode: GrowthMode` | `boolean` | Override growth mode |
| `get_streak` | `studentId: number` | `{ current: number, longest: number, badges: string[] }` | Get streak info |

---

## 10.7 Store State

```typescript
interface BeatYesterdayState {
  dailyTarget: DailyTarget | null;
  yesterdayPerformance: DailyPerformanceProfile | null;
  todayPerformance: DailyPerformanceProfile | null;
  climbState: ClimbState | null;
  growthMode: GrowthMode | null;
  streak: { current: number, longest: number, badges: string[] } | null;
  weeklyHistory: DailyPerformanceProfile[];
  loading: boolean;
  error: string | null;
  isRecoveryMode: boolean;
  todayCompleted: boolean;
}
```

---

## 10.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Hero card load | Yesterday and Today sides slide in from left/right, meet in center, 600ms | None |
| Yesterday stats appear | Numbers count up from 0, 500ms | None |
| Today target appear | Number scales in with emphasis, 300ms | Target set sound (`feedback/target-set.mp3`) |
| Start Climb button | Pulsing glow, scale 1.0 -> 1.02 -> 1.0, 3s cycle | None |
| Warm Start enter | Soft green gradient fades in, calm atmosphere, 500ms | Calm ambient start (`ambient/warm-start.mp3`) |
| Core Climb enter | Blue/indigo gradient, focused atmosphere, 500ms | Focus ramp (`ambient/core-climb.mp3`) |
| Speed Burst countdown | 3-2-1-GO large center text, dramatic, 3s | Countdown beeps + GO (`transitions/countdown.mp3`) |
| Speed Burst active | Background pulses with energy, timer shrinks dramatically | Energy beat (`ambient/speed-burst.mp3`) |
| Finish Strong enter | Warm gold gradient, positive atmosphere, 500ms | Positive tone (`ambient/finish-strong.mp3`) |
| Micro-gain appear | Chip bounces in from side, settles, 300ms | Gain tick (`feedback/micro-gain.mp3`) |
| "You Beat Yesterday!" | Large text scales in, confetti burst, streak counter increments, 2000ms | Victory fanfare (`celebration/beat-yesterday.mp3`) |
| "Almost!" | Encouraging text fades in, warm embrace animation, 1000ms | Encouragement tone (`feedback/encouragement.mp3`) |
| Streak milestone | Badge scales in with gold particles, 800ms | Badge earned (`celebration/badge-earned.mp3`) |
| Recovery mode enter | Gradual amber shift over 1500ms | Gentle recovery tone (`transitions/recovery-enter.mp3`) |

---
---

# DOMAIN 11: ELITE MODE

## 11.1 Domain Overview

Elite Mode is the premium high-performance layer for students who have surpassed standard mastery levels. It features a distinct dark premium visual theme, 7 arena session types, a 6-tier progression system, precision-focused scoring (Elite Performance Score / EPS), a topic domination board, records wall, analytical debriefs, and premium feedback language. Elite Mode is the aspirational ceiling of the product -- students work toward it, not everyone reaches it.

**Entry points**: Sidebar "Elite Mode" nav item (only visible to eligible students), promotion from Journey Mode, Coach Hub elite directive.

---

## 11.2 Features

### F11.1 Entry (3 Paths)
Three ways to enter Elite Mode:

| Path | Description | Criteria |
|------|------------|----------|
| Organic | Natural mastery growth | 3+ topics at Robust/Exam-Ready, overall readiness >75% |
| Challenge | Complete an Elite Entry Challenge | 12-question entry test with high bar |
| Invitation | Coach recommends entry | Coach identifies elite potential from performance patterns |

### F11.2 Baseline Assessment (12-15 Questions)
- Entry assessment to calibrate Elite tier
- 12-15 questions at high difficulty
- Precision scoring (partial credit, time bonus, streak bonus)
- Covers breadth of curriculum at advanced level
- No hints, no rescue dock, strict timing
- Results determine starting tier

### F11.3 Profile Reveal with Tier Assignment
- Dramatic reveal after baseline:
  1. Dark screen with "Analyzing your performance..."
  2. EPS (Elite Performance Score) counts up
  3. Tier badge appears with ceremony
  4. Strengths and specializations highlighted
- Tier assignment based on EPS score
- If score is too low: "You're on the path. Keep building and try again soon." (graceful redirect back to normal mode)

### F11.4 Elite Home
Premium dark-themed home screen:

| Section | Content |
|---------|---------|
| Elite Header | Tier badge, EPS score, rank |
| Identity Panel (6 Dimensions) | Precision, Speed, Depth, Endurance, TrapSense, Pressure Resilience -- each with score and rating |
| Today's Push | Recommended elite session |
| Challenge Queue | Available arena challenges |
| Domination Snapshot | Topic domination progress overview |
| Records | Personal bests + records approaching |
| Momentum | Trend of recent EPS changes |

### F11.5 Identity Panel (6 Dimensions)

| Dimension | Measures | Rating Scale |
|-----------|----------|-------------|
| Precision | Accuracy under all conditions | S / A / B / C / D |
| Speed | Response time relative to expected | S / A / B / C / D |
| Depth | Performance on complex, multi-step questions | S / A / B / C / D |
| Endurance | Performance maintenance over long sessions | S / A / B / C / D |
| TrapSense | Ability to avoid distractor traps | S / A / B / C / D |
| Pressure Resilience | Performance under pressure vs calm | S / A / B / C / D |

Each dimension displayed as a card with dimension name, score bar, letter rating, and trend arrow.

### F11.6 Arena (7 Session Types)

| Type | Name | Description | Duration | Focus |
|------|------|-------------|----------|-------|
| 1 | Precision Lab | No time pressure, every answer must be correct | 15-20 min | Accuracy above all |
| 2 | Sprint | Maximum questions in minimum time | 5 min | Speed |
| 3 | Depth Lab | Complex multi-step problems | 20-30 min | Deep reasoning |
| 4 | TrapSense | Questions with sophisticated distractors | 15 min | Trap avoidance |
| 5 | Endurance Track | Long session without breaks | 45-60 min | Sustained performance |
| 6 | Perfect Run | Attempt to answer all questions correctly, one wrong and it's over | Until failure | Perfection |
| 7 | Apex Mock | Elite-level mock exam with premium analysis | Full exam | Exam simulation |

Each arena type has its own card with description, best record, and "Enter Arena" CTA.

### F11.7 Live Elite Session (Premium Theme)
Dark premium visual theme during Elite sessions:

| Element | Details |
|---------|---------|
| Background | Dark (#1A1A2E), subtle gradient |
| Accent Color | Purple/gold (#A78BFA / #F5C842) |
| Precision Score | Live accuracy percentage, prominent display |
| Streak Meter | Current correct streak with visual meter |
| Performance Ring | Circular ring showing session performance (fills green as it goes well, orange/red for mistakes) |
| Pressure Bar | Bar showing pressure level (increases with time/difficulty) |
| Benchmark | Line on performance ring showing tier threshold |
| Goal Chip | Current session goal displayed as chip |
| Question Display | Premium card styling with subtle glow border |

### F11.8 Analytical Debrief
Premium post-session analysis:
- EPS change: before -> after with delta
- 6-dimension breakdown: how this session affected each dimension
- Per-question analysis: time, correctness, difficulty rating, precision contribution
- "Elite insight": AI-generated tactical observation
- Comparison to tier average and next-tier threshold
- "What separated good from great this session" analysis

### F11.9 Records Wall
- Personal best records across all session types
- Records: longest streak, highest accuracy, fastest average time, longest endurance, best Perfect Run
- Title earned: "Precision Master", "Speed King", "Endurance Champion"
- Badges: visual badges for each achievement
- "Records approaching": when close to breaking a personal best, highlighted

### F11.10 Topic Domination Board
Per-topic elite mastery tracking:

| Tier | Descriptor | Requirement |
|------|-----------|-------------|
| Uncharted | "Undiscovered territory" | Not yet attempted at elite level |
| Contested | "First foothold" | Some elite-level correct answers |
| Advancing | "Building your position" | Consistent correct answers |
| Commanding | "Strong hold" | High accuracy, good speed |
| Dominant | "Unquestioned authority" | Near-perfect accuracy + speed |
| Legendary | "Total supremacy" | Perfect record + speed + pressure |

Each topic card shows: topic name, domination tier, tier descriptor, progress to next tier, EPS contribution.

### F11.11 Insights (Trends/Weakness/Pressure-Behavior)
- Performance trends: EPS over time, dimension changes, session-type-specific trends
- Weakness analysis: which questions/topics cause the most elite-level errors
- Pressure-behavior mapping: how performance changes under different pressure levels
- Charts: trend lines, comparison bars, radar chart of 6 dimensions over time

### F11.12 Elite Mock Centre
- Special mock section within Elite Mode
- Harder question selection, stricter timing, premium analysis
- Comparison to elite-tier average performance
- "If this were the real exam, you'd place in the top {n}%"

### F11.13 Settings
- Growth mode preferences within elite
- Session duration preferences
- Notification preferences for challenges
- Opt-out of elite mode (return to standard)

### F11.14 Six-Tier Progression

| Tier | Name | EPS Range | Visual |
|------|------|-----------|--------|
| 1 | Foundation | 0-1999 | Bronze badge |
| 2 | Contender | 2000-3999 | Silver badge |
| 3 | Achiever | 4000-5999 | Gold badge |
| 4 | Expert | 6000-7999 | Platinum badge |
| 5 | Master | 8000-9499 | Diamond badge |
| 6 | Legend | 9500-10000 | Legendary emblem (animated) |

- Tier transitions are ceremonial (special animation + title reveal)
- Each tier unlocks progressively harder challenges
- Tier can decrease if EPS drops (with grace period)

### F11.15 EPS Formula Display
- "How is EPS calculated?" expandable section
- Formula shown in mathematical notation (KaTeX rendered)
- Components: base accuracy score + speed bonus + streak multiplier + difficulty weight + pressure modifier + consistency factor
- Each component explained with plain language

### F11.16 Premium Feedback Language
- Elite mode uses a distinct copy register:
  - Standard: "Good job!" -> Elite: "Precision confirmed."
  - Standard: "You got it wrong" -> Elite: "Deviation detected."
  - Standard: "Try harder" -> Elite: "Gap identified. Adjust approach."
  - Standard: "Great streak!" -> Elite: "Streak active. Maintain discipline."
- Language is analytical, respectful, performance-focused (not condescending)
- Coach voice in elite is "Performance Analyst" rather than "Supportive Coach"

---

## 11.3 Screens

### Screen: Elite Home (`pages/student/elite/index.vue`)
**Layout**: `student` with elite theme override (dark premium CSS)
**Route**: `/student/elite/`

**Sections**:
| Section | Content |
|---------|---------|
| Elite Header | Tier badge (animated for Legend), EPS display, rank |
| Identity Panel | 6-dimension grid with scores and ratings |
| Today's Push | Recommended elite session card |
| Challenge Queue | Available challenges (1-3 cards) |
| Domination Snapshot | Mini topic domination grid |
| Records | Key records with "approaching" highlights |
| Momentum | EPS trend sparkline |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Data fetching | Dark skeleton with purple pulse |
| Entry Required | Not yet in elite mode | Entry options: 3 paths |
| Baseline Active | Currently taking baseline | "Complete your baseline" resume card |
| Profile Reveal | Baseline complete, tier not yet shown | Dramatic reveal animation |
| Not Qualified | Baseline score too low | Graceful redirect with encouragement |
| Normal | Active elite member | Full home layout |
| Tier Transition | EPS crossed tier boundary | Tier promotion/demotion ceremony |
| Error | Fetch failed | AppError with dark theme styling |

---

### Screen: Elite Arena (`pages/student/elite/arena.vue`)
**Layout**: `student` (elite theme)
**Route**: `/student/elite/arena`

**Sections**:
| Section | Content |
|---------|---------|
| Arena Header | "The Arena" title with dramatic styling |
| Session Type Cards | 7 arena type cards in grid |
| Records Per Type | Personal best for each type |
| Recommended | AI-recommended session type highlighted |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Loading arena data | Dark skeleton cards |
| Normal | Data loaded | 7 arena cards |
| Locked Types | Some types locked by tier | Locked cards with tier requirement badge |
| Error | Fetch failed | AppError |

---

### Screen: Elite Live Session (`pages/student/elite/session/[id].vue`)
**Layout**: `focus` (full-screen, elite dark theme)
**Route**: `/student/elite/session/:id`

**Sections**:
| Section | Content | Position |
|---------|---------|----------|
| Session Type Badge | Arena type name | Top-left |
| Precision Score | Live accuracy % | Top-center |
| Streak Meter | Current streak bar | Top-right area |
| Performance Ring | Circular performance indicator | Right sidebar or overlay |
| Pressure Bar | Pressure level indicator | Below performance ring |
| Benchmark Line | Tier threshold on performance ring | Overlaid on ring |
| Goal Chip | Session goal | Below session type badge |
| Question Area | Premium-styled QuestionCard | Center, full width |
| Timer | Session-type-specific timer | Integrated into header |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Session loading | Dark skeleton with purple pulse |
| Active | Session in progress | Full elite session interface |
| Perfect Run - Active | Perfect run, all correct so far | Green glow on performance ring |
| Perfect Run - Failed | First wrong answer in perfect run | Red flash, "Run Ended" overlay |
| Endurance - Fatigue | Long session, checking endurance | Subtle pressure indicators increase |
| Sprint - Active | Sprint session, timer prominent | Large countdown timer, rapid question flow |
| Complete | Session finished | Transition to debrief |
| Error | Session error | Subtle error handling (don't break flow) |

---

### Screen: Elite Debrief (`pages/student/elite/debrief/[id].vue`)
**Layout**: `student` (elite theme)

**Sections**:
| Section | Content |
|---------|---------|
| EPS Delta | Before -> After with animated delta |
| 6-Dimension Impact | How session affected each dimension |
| Question Analysis | Per-question breakdown table |
| Elite Insight | AI tactical observation |
| Tier Comparison | Performance vs tier average and next tier |
| "Good vs Great" | What separated this session from perfection |

---

### Screen: Elite Records (`pages/student/elite/records.vue`)
**Layout**: `student` (elite theme)

**Sections**:
| Section | Content |
|---------|---------|
| Records Wall | Grid of personal bests with badges |
| Title Gallery | Earned titles with badge icons |
| "Approaching" | Records close to being broken |
| History | Record-breaking moments timeline |

**States**:
| State | Condition | Display |
|-------|-----------|---------|
| Loading | Loading records | Dark skeleton |
| Normal | Records loaded | Full records wall |
| New Record | Just broke a record (navigation from debrief) | Record celebration animation |
| Error | Fetch failed | AppError |

---

### Screen: Topic Domination (`pages/student/elite/domination.vue`)
**Layout**: `student` (elite theme)

**Sections**:
| Section | Content |
|---------|---------|
| Overview Stats | Total topics, domination distribution |
| Topic Grid | All topics with domination tier, descriptor, progress bar |
| Subject Filter | Filter by subject |
| Leaderboard | (If competitive feature enabled) ranking |

---

### Screen: Elite Insights (`pages/student/elite/insights.vue`)
**Layout**: `student` (elite theme)

**Sections**:
| Section | Content |
|---------|---------|
| EPS Trend | Line chart of EPS over time |
| Dimension Trends | Per-dimension trend lines |
| Weakness Map | Topics/areas causing most elite errors |
| Pressure Behavior | Performance under pressure analysis |
| Session-Type Analysis | Performance trends per arena type |

---

## 11.4 Components

### EliteHome (`components/modes/elite/EliteHome.vue`)
**Props**: `studentId: number`
**Display**: Full elite home layout with all sections.

### EliteArena (`components/modes/elite/EliteArena.vue`)
**Props**: `availableTypes: string[]`, `records: object`, `lockedTypes: string[]`
**Emits**: `@select(type: string)`
**Display**: 7 arena type cards.

### EliteSession (`components/modes/elite/EliteSession.vue`)
**Props**: `sessionId: number`, `sessionType: string`, `tierThreshold: number`
**Display**: Premium dark session interface with all elite HUD elements.

### EliteDebrief (`components/modes/elite/EliteDebrief.vue`)
**Props**: `sessionResult: EliteSessionResult`, `epsBefore: number`, `epsAfter: number`
**Display**: Analytical debrief with 6-dimension breakdown.

### EliteRecords (`components/modes/elite/EliteRecords.vue`)
**Props**: `records: object[]`, `titles: string[]`
**Display**: Records wall grid with badges and titles.

### TopicDominationBoard (`components/modes/elite/TopicDominationBoard.vue`)
**Props**: `topics: TopicDomination[]`
**Display**: Grid of topic cards with domination tiers and descriptors.

### EliteInsights (`components/modes/elite/EliteInsights.vue`)
**Props**: `trendData: object`, `weaknessData: object`, `pressureData: object`
**Display**: Multiple chart panels.

### IdentityPanel (`components/modes/elite/IdentityPanel.vue`)
**Props**: `dimensions: { name: string, score: number, rating: string, trend: string }[]`
**Display**: 6-dimension grid with score bars and letter ratings.

### PrecisionScore (`components/modes/elite/PrecisionScore.vue`)
**Props**: `accuracy: number`, `totalQuestions: number`
**Display**: Large percentage display with decimal precision.

### StreakMeter (`components/modes/elite/StreakMeter.vue`)
**Props**: `current: number`, `best: number`
**Display**: Horizontal meter filling toward best, current count prominent.

### PerformanceRing (`components/modes/elite/PerformanceRing.vue`)
**Props**: `performance: number`, `tierThreshold: number`
**Display**: SVG circular ring, fills green/orange/red, benchmark line overlaid.

### PressureBar (`components/modes/elite/PressureBar.vue`)
**Props**: `level: number` (0-100)
**Display**: Horizontal bar, color shifts from blue (low) to red (high).

### TierBadge (`components/modes/elite/TierBadge.vue`)
**Props**: `tier: EliteTier`, `eps: number`
**Display**: Badge icon sized and colored by tier. Legend tier has animated glow.

### EpsDisplay (`components/modes/elite/EpsDisplay.vue`)
**Props**: `score: number`, `delta: number | null`
**Display**: Large EPS number with optional delta indicator (+/- with color).

### GoalChip (`components/modes/elite/GoalChip.vue`)
**Props**: `goal: string`
**Display**: Small pill chip showing session goal text.

### DominationTierCard (`components/modes/elite/DominationTierCard.vue`)
**Props**: `topic: TopicDomination`
**Display**: Card with topic name, tier badge, rich-text descriptor, progress bar to next tier.

---

## 11.5 User Flows

### Flow: Elite Mode Entry (Organic)
1. Student has been performing well, 4 topics at Robust, readiness >80%
2. Coach Hub shows: "You're performing at an elite level. Ready for a challenge?"
3. Student taps "Explore Elite Mode" -> route to `/student/elite/`
4. Entry screen: 3 paths displayed, "Organic Qualification" highlighted as eligible
5. "Confirm Entry" -> route to baseline assessment
6. 12 hard questions, strict timing, no hints, dark premium theme
7. Assessment complete -> processing screen "Calibrating your elite profile..."
8. Profile Reveal: EPS counts up to 4,200 -> Tier badge: "Achiever" (Gold) appears
9. 6 dimensions revealed one by one with ratings
10. "Welcome to Elite Mode" -> route to Elite Home

### Flow: Arena Session (Precision Lab)
1. Elite Home -> "The Arena" -> Arena screen
2. Student selects "Precision Lab" card
3. Brief: "No timer. Every answer must be correct. Precision is everything."
4. Enter session: dark premium theme, precision score prominently displayed
5. Question 1: correct -> precision 100%, streak 1
6. Question 5: correct -> precision 100%, streak 5, streak meter filling
7. Question 8: incorrect -> precision drops to 87.5%, streak resets, red flash
8. Question 15: session complete -> debrief
9. Debrief: EPS +45, Precision dimension: A -> A (maintained), "Your speed on question 8 was 2x your average. Slow down on unfamiliar formats."

### Flow: Perfect Run
1. Arena -> select "Perfect Run"
2. Brief: "Answer correctly until you can't. One wrong answer ends it all."
3. Enter: performance ring shows green, filling clockwise
4. Questions get progressively harder
5. Question 1-8: all correct, streak growing, ring filling, tension building
6. Question 9: wrong answer -> red flash, "Run Ended at 8."
7. Debrief: "Personal best: 8 consecutive. Record: 12. Keep pushing."

### Flow: Tier Promotion
1. Student completes an elite session
2. EPS crosses from 5,900 to 6,100 (crosses into Expert tier)
3. Post-debrief: special ceremony screen
4. "TIER PROMOTION" text appears dramatically
5. Old badge (Gold/Achiever) shatters, new badge (Platinum/Expert) forms
6. New title unlocked: "Expert" displayed
7. Confetti + premium celebration
8. Return to Elite Home with updated tier badge

---

## 11.6 Tauri Commands

| Command | Parameters | Returns | Purpose |
|---------|-----------|---------|---------|
| `get_elite_profile` | `studentId: number` | `EliteProfile` | Get full elite profile |
| `check_elite_eligibility` | `studentId: number` | `{ eligible: boolean, path: string, reason: string }` | Check if student can enter elite |
| `start_elite_baseline` | `studentId: number` | `Session` | Start baseline assessment |
| `complete_elite_baseline` | `sessionId: number` | `{ eps: number, tier: EliteTier, dimensions: object }` | Finalize baseline and assign tier |
| `start_elite_session` | `studentId: number, sessionType: string` | `Session` | Start arena session |
| `get_elite_performance_score` | `studentId: number` | `ElitePerformanceScore` | Get current EPS |
| `get_topic_domination` | `studentId: number` | `TopicDomination[]` | Get domination board |
| `get_elite_records` | `studentId: number` | `object` | Get records and badges |
| `get_elite_insights` | `studentId: number` | `object` | Get trend and weakness data |
| `get_elite_session_result` | `sessionId: number` | `EliteSessionResult` | Get elite session debrief data |
| `get_eps_formula` | none | `{ formula: string, components: object[] }` | Get EPS formula for display |

---

## 11.7 Store State

```typescript
interface EliteState {
  // Profile
  eliteProfile: EliteProfile | null;
  profileLoading: boolean;
  profileError: string | null;

  // Eligibility
  isEligible: boolean;
  eligibilityPath: string | null;
  eligibilityChecked: boolean;

  // EPS
  currentEps: number;
  currentTier: EliteTier | null;
  epsHistory: { date: string, eps: number }[];

  // Identity panel
  dimensions: { name: string, score: number, rating: string, trend: string }[];

  // Arena
  availableSessionTypes: string[];
  lockedSessionTypes: string[];
  arenaRecords: Map<string, object>;

  // Active session
  activeEliteSession: Session | null;
  sessionType: string | null;
  livePrecision: number;
  liveStreak: number;
  livePressure: number;
  livePerformance: number;

  // Domination
  topicDominations: TopicDomination[];
  dominationLoading: boolean;

  // Records
  records: object[];
  titles: string[];
  approachingRecords: object[];

  // Insights
  insights: object | null;
  insightsLoading: boolean;

  // Debrief
  lastSessionResult: EliteSessionResult | null;
  epsDelta: number;

  // Tier transition
  showTierTransition: boolean;
  oldTier: EliteTier | null;
  newTier: EliteTier | null;
}
```

---

## 11.8 Animations & Sounds

| Trigger | Animation | Sound |
|---------|-----------|-------|
| Elite Mode enter | Dark theme fade in from edges (vignette), premium gradient builds, 800ms | Elite entrance tone (`transitions/elite-enter.mp3`) -- deep, precise |
| Profile reveal | Dark screen, EPS number counts up with golden particles, 2000ms | Score count with metallic undertone (`celebration/elite-reveal.mp3`) |
| Tier badge reveal | Badge materializes with shatter-in effect (particles form badge), 1000ms | Tier reveal impact (`celebration/tier-reveal.mp3`) |
| Dimension reveal | 6 cards stagger in with slide + fade, 200ms each, 100ms stagger | Subtle tick for each (`feedback/dimension-tick.mp3`) |
| Arena card select | Card lifts (scale + shadow), brief glow, 200ms | Selection click (`feedback/elite-select.mp3`) |
| Elite session start | Dark focus, HUD elements build in from edges, 600ms | Session initialize tone (`transitions/elite-session-start.mp3`) |
| Correct answer (elite) | Precision score ticks up, streak meter fills, green pulse on performance ring, 300ms | Precise click (`feedback/elite-correct.mp3`) -- sharp, clean |
| Incorrect answer (elite) | Red flash on performance ring, streak meter resets, precision ticks down, 400ms | Error tone (`feedback/elite-wrong.mp3`) -- low, definitive |
| Streak milestone (5, 10, 15...) | Streak meter glows gold, number pulses, 400ms | Streak milestone (`feedback/elite-streak.mp3`) |
| Perfect Run fail | Red shatter animation from wrong answer, "Run Ended" slides in, 800ms | Impact sound (`feedback/elite-run-end.mp3`) |
| Tier promotion | Old badge shatters to particles, particles reform as new badge, golden burst, 2500ms | Tier promotion fanfare (`celebration/tier-promotion.mp3`) |
| Tier demotion | Badge dims, cracks appear, drops to lower tier badge, 1500ms | Somber but dignified tone (`feedback/tier-demotion.mp3`) |
| EPS delta display | Number slides in with color (green for +, red for -), 400ms | None |
| Performance ring fill | Ring draws clockwise with glow, 600ms | None |
| Pressure bar increase | Bar fills smoothly with color shift, 400ms | Subtle pressure hum if >70% (`ambient/pressure-hum.mp3`) |
| Domination tier up | Card glows, tier badge morphs to new level, 600ms | Domination advance (`feedback/domination-up.mp3`) |
| Record broken | Golden explosion from record entry, "NEW RECORD" text stamps in, 1000ms | Record broken fanfare (`celebration/record-broken.mp3`) |
| Legend tier glow | Continuous animated glow/shimmer on Legend badge, looping | Subtle ambient hum (very quiet, `ambient/legend-presence.mp3`) |

---
---

# APPENDIX A: CROSS-DOMAIN COMPONENT REUSE MAP

| Component | Used In Domains |
|-----------|----------------|
| QuestionCard | 3, 4, 5, 6, 7, 10, 11 |
| QuestionTimer | 3, 4, 5, 6, 10, 11 |
| ConfidenceCapture | 3, 5, 6 |
| WrongAnswerReview | 3, 6, 7 |
| MistakeClinicFlow | 3, 4, 7 |
| MathRenderer | 3, 4, 5, 6, 7, 8, 11 |
| ReadinessGauge | 6, 8, 9 |
| MasteryBadge | 8, 9, 11 |
| TrendLine | 7, 8, 10, 11 |
| RadarChart | 5, 8, 11 |
| CoachVoice | 2, 4, 9, 10 |
| RescueDock | 2, 3, 4 |
| SessionPlayer | 4, 10 (Beat Yesterday uses 4-block variant) |
| ProgressRing | 1, 8, 9 |
| ExamCountdown | 2, 6, 8 |
| PinPad | 1 (also used in parent reset flows) |

---

# APPENDIX B: SOUND FILE INVENTORY (Domains 1-11)

```
assets/sounds/
├── feedback/
│   ├── tap.mp3
│   ├── pin-tick.mp3
│   ├── select.mp3
│   ├── correct.mp3
│   ├── correct-soft.mp3
│   ├── wrong.mp3
│   ├── wrong-soft.mp3
│   ├── streak.mp3
│   ├── timer-warning.mp3
│   ├── time-up.mp3
│   ├── drag-start.mp3
│   ├── drag-drop.mp3
│   ├── recovery.mp3
│   ├── encouragement.mp3
│   ├── lockout.mp3
│   ├── flag.mp3
│   ├── page-turn.mp3
│   ├── dock-open.mp3
│   ├── coach-voice.mp3
│   ├── band-change.mp3
│   ├── mastery-change.mp3
│   ├── mastery-up.mp3
│   ├── target-set.mp3
│   ├── micro-gain.mp3
│   ├── download.mp3
│   ├── idle-ping.mp3
│   ├── recovery-prompt.mp3
│   ├── dimension-tick.mp3
│   ├── elite-select.mp3
│   ├── elite-correct.mp3
│   ├── elite-wrong.mp3
│   ├── elite-streak.mp3
│   ├── elite-run-end.mp3
│   ├── domination-up.mp3
│   └── tier-demotion.mp3
├── transitions/
│   ├── session-start.mp3
│   ├── block-change.mp3
│   ├── countdown.mp3
│   ├── pause.mp3
│   ├── resume.mp3
│   ├── state-change.mp3
│   ├── phase-change.mp3
│   ├── recovery-enter.mp3
│   ├── drill-enter.mp3
│   ├── diagnostic-start.mp3
│   ├── phase-complete.mp3
│   ├── door-close.mp3
│   ├── paper-shuffle.mp3
│   ├── clock-tick.mp3
│   ├── drill-down.mp3
│   ├── route-change.mp3
│   ├── avatar-move.mp3
│   ├── phase-transition.mp3
│   ├── elite-enter.mp3
│   ├── elite-session-start.mp3
│   └── times-up.mp3
├── ambient/
│   ├── baseline-calm.mp3
│   ├── speed-brisk.mp3
│   ├── pressure-heartbeat.mp3
│   ├── root-cause-focus.mp3
│   ├── analyzing.mp3
│   ├── speed-burst.mp3
│   ├── warm-start.mp3
│   ├── core-climb.mp3
│   ├── finish-strong.mp3
│   ├── pressure-hum.mp3
│   └── legend-presence.mp3
└── celebration/
    ├── account-created.mp3
    ├── session-complete.mp3
    ├── good-session.mp3
    ├── report-ready.mp3
    ├── mock-good.mp3
    ├── mock-okay.mp3
    ├── score-count.mp3
    ├── beat-yesterday.mp3
    ├── badge-earned.mp3
    ├── journey-reveal.mp3
    ├── station-unlock.mp3
    ├── station-pass.mp3
    ├── station-master.mp3
    ├── elite-reveal.mp3
    ├── tier-reveal.mp3
    ├── tier-promotion.mp3
    └── record-broken.mp3
```

---

# APPENDIX C: EMOTIONAL THEME MAPPING (Domains 1-11)

| Context | Theme CSS | Domains Using |
|---------|----------|---------------|
| Normal student | `student.css` | 1, 2, 3, 4, 8, 9 |
| Recovery/rescue | `modes/recovery.css` | 2, 4, 7, 9, 10 |
| Pressure/exam | `modes/pressure.css` | 5 (phase 4), 6, 11 |
| Elite premium | `modes/elite.css` | 11 |
| Celebration | `modes/celebration.css` | 2, 4, 6, 9, 10, 11 |
| Focus/zen | `modes/focus.css` | 4, 5, 6 |
| Game/energy | `modes/game.css` | 10 (Speed Burst) |

---

*End of Domain Specifications 1-11. Total specification covers: 11 domains, 87 features, 32 screens, 123 components, 34 user flows, 78 Tauri commands, 11 store definitions, 146 animation/sound entries.*
