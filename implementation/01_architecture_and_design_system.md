# eCoach -- Frontend Architecture & Design System

> Full-vision implementation blueprint for the Academic Intelligence Desktop OS
> 2,183 features | Student + Parent + Admin portals | Tauri 2.x + Nuxt 3

---

## Table of Contents

1. [Philosophy](#1-philosophy)
2. [Tech Stack](#2-tech-stack)
3. [Architecture](#3-architecture)
4. [Complete Project Structure](#4-complete-project-structure)
5. [Complete Design System](#5-complete-design-system)

---

## 1. Philosophy

### 1.1 Full Vision from Day One

eCoach is not an MVP that grows into a product. It is an academic intelligence operating system designed, architected, and built to its full 2,183-feature specification from the first commit. Every abstraction, every component, every store, every IPC channel is laid out with the complete feature surface in mind. There is no "we'll refactor later." The architecture carries the weight of the final product on day one.

### 1.2 Core Tenets

**Architecture-first, features second.** No feature is implemented until the architectural layer it depends on is complete, tested, and documented. A Button component exists before any page that uses it. A Pinia store for mastery state exists before any widget reads from it. The IPC contract for a Rust command is defined before either side implements it.

**Every component built complete.** A component is not "done" when it renders. It is done when it handles every prop variant, every edge case, every error state, every loading state, every animation, every accessibility attribute, every dark-mode token, every role-based style override, and every responsive breakpoint. Partial components do not ship.

**Three portals, one codebase.** The student, parent, and admin experiences are three shells rendered from one Nuxt application. They share primitives, design tokens, utilities, and stores. They diverge at the layout, page, and permission layer. Code is never duplicated across portals.

**Offline-first, sync-later.** eCoach runs as a Tauri desktop application. The Rust backend owns the SQLite database, the file system, and all heavy computation. The frontend never assumes network access. Every interaction works offline. Sync is a background reconciliation process, never a blocking dependency.

**Type safety is non-negotiable.** TypeScript strict mode is enabled globally. No `any` types. No `@ts-ignore`. Every IPC payload has a Zod schema on the frontend and a serde struct on the backend. Every Pinia store is fully typed. Every component prop is typed. Every composable return value is typed.

**Performance is a feature.** The application renders 60fps during animations, loads pages in under 100ms, and handles datasets of 10,000+ records without jank. Virtual scrolling, lazy loading, code splitting, Web Workers for heavy computation, and PixiJS for GPU-accelerated visualizations are baseline expectations, not optimizations.

**Accessibility is not optional.** Every interactive element is keyboard-navigable. Every image has alt text. Every color combination meets WCAG 2.1 AA contrast. Every modal traps focus. Every status change is announced to screen readers. ARIA attributes are applied semantically, not decoratively.

**Motion is intentional.** Animations exist to communicate state changes, guide attention, and reinforce spatial relationships. They never exist for decoration. Every motion has a purpose, a duration token, and an easing curve. Reduced-motion preferences are respected globally.

### 1.3 Design Philosophy

eCoach is an environment students live inside for hours. The design must be:

- **Calm.** Low-contrast backgrounds, muted accent colors, no visual noise.
- **Dense when needed.** Dashboards pack information tightly with clear hierarchy.
- **Spacious when needed.** Reading and problem-solving modes remove all chrome.
- **Emotionally aware.** The UI adapts its tone (color temperature, animation intensity, encouragement messaging) based on the student's detected emotional state.
- **Gamified without being childish.** XP bars, streak counters, achievement badges, and mastery trees feel like professional tools, not cartoon games.

---

## 2. Tech Stack

### 2.1 Core Runtime

| Technology | Version | Purpose |
|---|---|---|
| **Tauri** | 2.x | Desktop application shell. Provides the webview, native file system access, system tray, auto-updater, window management, and the IPC bridge to the Rust backend. Chosen over Electron for its 10x smaller binary size, lower memory footprint, and Rust backend. |
| **Nuxt** | 3.x | Vue meta-framework. Provides file-based routing, auto-imports, server/client separation (used for SSG/prerender of static pages), module system, and build toolchain (Vite under the hood). |
| **Vue** | 3.x | Reactive UI framework. Composition API exclusively (no Options API). `<script setup>` syntax for all SFCs. Provides reactivity system, component model, slots, teleport, suspense, and transition primitives. |
| **TypeScript** | 5.x (strict) | Type system for the entire frontend. `strict: true`, `noUncheckedIndexedAccess: true`, `exactOptionalPropertyTypes: true`. Every file is `.ts` or `.vue` with `<script setup lang="ts">`. |

### 2.2 Styling & Layout

| Technology | Version | Purpose |
|---|---|---|
| **TailwindCSS** | 4.x | Utility-first CSS framework. All design tokens (colors, spacing, typography, shadows, radii, z-indices) are defined in `tailwind.config.ts` and consumed as utilities. No custom CSS files except for third-party overrides and keyframe definitions. Dark mode via `class` strategy. |

### 2.3 State Management

| Technology | Version | Purpose |
|---|---|---|
| **Pinia** | 2.x | Global state management. One store per domain (mastery, schedule, user, preferences, notifications, gamification, sync, etc.). Stores use the setup syntax (`defineStore` with a function). Persistence plugin for offline-first hydration from Tauri's SQLite via IPC. |

### 2.4 Specialized Libraries

| Technology | Purpose |
|---|---|
| **KaTeX** | LaTeX math rendering. Used in every context where mathematical notation appears: problem display, solution steps, formula sheets, notes editor, flashcards, exam papers. Server-side rendering mode for static content, client-side for dynamic/interactive math. |
| **D3.js** | Data-driven SVG visualizations. Used for mastery trees, knowledge graphs, grade trend charts, attendance heatmaps, comparative analytics, learning path DAGs, schedule Gantt charts. Vue components wrap D3 selections; D3 handles data joins and transitions, Vue handles reactivity and lifecycle. |
| **PixiJS** | GPU-accelerated 2D rendering via WebGL/WebGPU. Used for the gamification layer: particle effects on achievements, animated XP orbs, interactive skill constellations, progress ring animations with hundreds of nodes, and any visualization where SVG/DOM performance is insufficient. |
| **Howler.js** | Audio playback engine. Used for notification sounds, timer alarms, achievement unlock sounds, ambient study sounds (rain, lo-fi, white noise), and audio playback in language-learning modules. Handles Web Audio API abstraction, sprite sheets, volume control, and fade transitions. |
| **Motion One** | Animation library built on the Web Animations API. Used for all UI transitions beyond CSS: page transitions, list reordering, layout animations, spring physics for draggable elements, scroll-triggered reveals, and orchestrated multi-element sequences. Integrates with Vue's `<Transition>` and `<TransitionGroup>`. |
| **jsPDF** | Client-side PDF generation. Used for report cards, progress reports, study guides, exported notes, printed schedules, certificates, and any document the user downloads. Combined with html2canvas for screenshot-based PDF sections and custom layout engine for structured documents. |
| **Lucide Icons** | Icon library. Tree-shakeable SVG icons. Every icon in the application comes from Lucide. Custom icons (eCoach-specific) extend the Lucide format and are stored in `assets/icons/`. Vue components via `lucide-vue-next`. |
| **VueUse** | Composition utility library. Provides 200+ composables: `useStorage`, `useDark`, `useMediaQuery`, `useIntersectionObserver`, `useVirtualList`, `useDraggable`, `useWebSocket`, `useClipboard`, `useKeyModifier`, `useFocusTrap`, etc. Avoids reinventing browser API wrappers. |

### 2.5 Build & Dev Tools

| Technology | Purpose |
|---|---|
| **Vite** | Build tool (bundled with Nuxt 3). HMR in development, optimized chunking in production. |
| **ESLint** | Linting with `@nuxt/eslint-config`, `@typescript-eslint`, and custom rules. |
| **Prettier** | Code formatting. Single config, no debates. |
| **Vitest** | Unit and component testing. Vue Test Utils for component tests. |
| **Playwright** | End-to-end testing in the Tauri webview. |
| **Zod** | Runtime schema validation for all IPC payloads, form inputs, and API responses. |

---

## 3. Architecture

### 3.1 Tauri IPC Communication Model

The frontend (Nuxt/Vue in a webview) and backend (Rust) communicate exclusively through Tauri's IPC bridge. There is no HTTP server. There are no WebSockets between frontend and backend. All communication is synchronous command invocation or asynchronous event streaming.

#### 3.1.1 Commands (Frontend -> Backend)

Commands are invoked via `@tauri-apps/api/core`'s `invoke` function. Every command has:

- A unique string name following the pattern `domain:action` (e.g., `mastery:get_state`, `schedule:create_block`).
- A typed request payload (TypeScript interface + Zod schema on frontend, serde struct on backend).
- A typed response payload.
- An error type that is a tagged union of possible failure modes.

```
Frontend                          Tauri IPC Bridge                    Rust Backend
-----------                       ------------------                  ------------
invoke('mastery:get_state',  -->  serialize to JSON   -->  #[tauri::command]
  { studentId, subjectId })       via IPC channel          fn get_mastery_state(...)
                                                             -> Result<MasteryState, AppError>
Result<MasteryState> <----------  deserialize from JSON <--  return value
```

All IPC calls are wrapped in typed composables:

```typescript
// composables/ipc/useMasteryIpc.ts
export function useMasteryIpc() {
  async function getMasteryState(studentId: string, subjectId: string): Promise<MasteryState> {
    const raw = await invoke('mastery:get_state', { studentId, subjectId })
    return MasteryStateSchema.parse(raw)
  }
  // ... other commands
  return { getMasteryState, /* ... */ }
}
```

#### 3.1.2 Events (Backend -> Frontend)

The backend emits events via Tauri's event system for:

- Real-time sync status updates
- Background task completion notifications
- System-level alerts (low disk space, update available)
- Timer/alarm triggers
- Gamification triggers (XP earned, achievement unlocked, streak milestone)

Events are consumed via `@tauri-apps/api/event`'s `listen` function, wrapped in composables that return reactive refs:

```typescript
// composables/ipc/useSyncEvents.ts
export function useSyncEvents() {
  const syncStatus = ref<SyncStatus>('idle')

  onMounted(() => {
    const unlisten = listen<SyncStatus>('sync:status_changed', (event) => {
      syncStatus.value = event.payload
    })
    onUnmounted(() => unlisten.then(fn => fn()))
  })

  return { syncStatus }
}
```

#### 3.1.3 IPC Contract Registry

Every IPC command and event is registered in a central contract file (`types/ipc/contracts.ts`) that serves as the single source of truth for the frontend. This file is auto-generated from the Rust backend's command definitions to prevent drift.

### 3.2 Three Role-Based Shells

The application renders one of three top-level shells based on the authenticated user's role. Each shell has its own layout, navigation structure, and page tree, but they share all primitive components, design tokens, and domain stores.

#### 3.2.1 Student Shell

- **Layout:** Sidebar navigation (collapsible) + top bar (breadcrumbs, search, notifications, profile) + main content area.
- **Navigation groups:** Dashboard, Subjects, Schedule, Practice, Exams, Notes, Goals, Achievements, Study Room, Settings.
- **Unique features:** Focus mode (hides all chrome), Pomodoro timer overlay, ambient sound player, emotional state selector, gamification HUD.

#### 3.2.2 Parent Shell

- **Layout:** Top navigation bar + sidebar for child selector + main content area.
- **Navigation groups:** Overview, Children (per-child drill-down), Reports, Messages, Payments, Settings.
- **Unique features:** Multi-child dashboard, comparative view, notification digest, report PDF export.

#### 3.2.3 Admin Shell

- **Layout:** Sidebar navigation (always expanded) + top bar (search, notifications, role badge) + main content area.
- **Navigation groups:** Dashboard, Students, Parents, Teachers, Curricula, Exams, Analytics, Content Management, System Settings, Audit Log.
- **Unique features:** Bulk operations toolbar, data table views with server-side pagination, real-time analytics dashboard, system health monitor.

#### 3.2.4 Shell Selection Routing

```
app.vue
  -> middleware/auth.global.ts (checks auth state, redirects to /login if unauthenticated)
  -> middleware/role-router.global.ts (reads user role, sets activeShell in useAppStore)
  -> layouts/
       student.vue   (renders <StudentShell />)
       parent.vue    (renders <ParentShell />)
       admin.vue     (renders <AdminShell />)
  -> pages/
       student/...   (definePageMeta({ layout: 'student' }))
       parent/...    (definePageMeta({ layout: 'parent' }))
       admin/...     (definePageMeta({ layout: 'admin' }))
```

### 3.3 State Management Strategy with Pinia

#### 3.3.1 Store Categories

**App-level stores** (singleton, shared across all portals):
- `useAppStore` -- active shell, theme, locale, feature flags, global loading state.
- `useAuthStore` -- current user, session, permissions, role.
- `usePreferencesStore` -- user preferences (font size, dark mode, sound enabled, reduced motion, language).
- `useNotificationStore` -- notification queue, read/unread state, notification preferences.
- `useSyncStore` -- sync status, last sync timestamp, conflict queue, retry queue.

**Domain stores** (one per bounded context):
- `useMasteryStore` -- mastery states for all subjects/topics, mastery transitions, decay tracking.
- `useScheduleStore` -- daily/weekly schedule blocks, recurring events, calendar integrations.
- `usePracticeStore` -- active practice session, question queue, answer history, adaptive difficulty state.
- `useExamStore` -- upcoming exams, past results, exam preparation checklists.
- `useNotesStore` -- notes tree, active note, editor state, tags, search index.
- `useGoalStore` -- active goals, milestones, progress tracking.
- `useGamificationStore` -- XP, level, streak, achievements, badges, leaderboard position.
- `useStudyRoomStore` -- Pomodoro state, ambient sound selection, focus session history.
- `useCurriculumStore` -- curriculum tree, syllabus mapping, learning objectives.
- `useAnalyticsStore` -- computed analytics, chart data, trend calculations.
- `useContentStore` -- content library, resource metadata, download state.
- `useMessageStore` -- conversations, message threads, unread counts.
- `usePaymentStore` -- subscription state, payment history, invoice data.
- `useAuditStore` -- (admin) audit log entries, filters, pagination.
- `useSystemStore` -- (admin) system health, feature toggles, configuration.

#### 3.3.2 Store Composition Pattern

Stores never call other stores directly inside actions. Cross-store coordination happens in composables:

```typescript
// composables/domain/useCompletePractice.ts
export function useCompletePractice() {
  const practice = usePracticeStore()
  const mastery = useMasteryStore()
  const gamification = useGamificationStore()
  const notifications = useNotificationStore()

  async function completePracticeSession(sessionId: string) {
    const result = await practice.submitSession(sessionId)
    await mastery.recalculate(result.subjectId, result.topicId)
    const xpGained = await gamification.awardXP(result.xpEarned)
    if (xpGained.leveledUp) {
      notifications.push({ type: 'level_up', data: xpGained })
    }
  }
  return { completePracticeSession }
}
```

#### 3.3.3 Persistence Strategy

Every store that holds user data implements a persistence layer:

1. On store initialization, hydrate from Tauri backend via IPC (`invoke('store:hydrate', { domain })`).
2. On every mutation, debounce-persist to backend via IPC (`invoke('store:persist', { domain, patch })`).
3. The Rust backend writes to SQLite. The frontend never touches the database directly.
4. Optimistic updates: the UI updates immediately, the persist call runs in the background, and conflicts are resolved on the next hydration cycle.

### 3.4 Component Architecture

#### 3.4.1 Component Hierarchy

```
Level 0: Design Tokens (CSS custom properties, Tailwind config)
Level 1: Primitives (Button, Input, Badge, Card, Modal, etc.)
Level 2: Composites (FormField, DataTable, MasteryBadge, ScheduleBlock, etc.)
Level 3: Features (PracticeSession, NoteEditor, ExamCountdown, GoalTracker, etc.)
Level 4: Pages (assembled from features, connected to stores)
Level 5: Layouts (shells, navigation, chrome)
```

#### 3.4.2 Component Conventions

- Every component is a single `.vue` file using `<script setup lang="ts">`.
- Props are defined with `defineProps<T>()` using a TypeScript interface.
- Emits are defined with `defineEmits<T>()`.
- Slots are typed with `defineSlots<T>()`.
- Components never import stores directly. Data flows down via props; actions flow up via emits. Only page-level components and feature-level components may access stores.
- Every primitive component supports a `class` prop for style overrides (via Tailwind's `cn()` merge utility).
- Every interactive component supports `disabled`, `loading`, and appropriate ARIA attributes.

#### 3.4.3 Component Documentation

Every primitive and composite component has a co-located `.story.vue` file for visual testing and documentation using Histoire (Vue's Storybook alternative).

### 3.5 Routing Strategy

Nuxt's file-based routing is used for all pages. The routing tree mirrors the three-shell architecture:

```
pages/
  index.vue                     -> redirects to /student, /parent, or /admin based on role
  login.vue                     -> authentication page
  student/
    index.vue                   -> student dashboard
    subjects/
      index.vue                 -> subject list
      [subjectId]/
        index.vue               -> subject detail
        topics/
          [topicId].vue         -> topic detail with mastery
    schedule/
      index.vue                 -> weekly schedule view
      daily.vue                 -> daily schedule view
    practice/
      index.vue                 -> practice session launcher
      [sessionId].vue           -> active practice session
    exams/
      index.vue                 -> exam list
      [examId]/
        index.vue               -> exam detail
        prepare.vue             -> exam preparation checklist
        review.vue              -> post-exam review
    notes/
      index.vue                 -> notes browser
      [noteId].vue              -> note editor
    goals/
      index.vue                 -> goals dashboard
    achievements/
      index.vue                 -> achievements gallery
    study-room/
      index.vue                 -> study room with timer and ambient sounds
    settings/
      index.vue                 -> settings hub
      profile.vue               -> profile editor
      preferences.vue           -> preferences editor
      accessibility.vue         -> accessibility settings
  parent/
    index.vue                   -> parent dashboard
    children/
      index.vue                 -> children overview
      [childId]/
        index.vue               -> child detail dashboard
        subjects.vue            -> child's subject performance
        schedule.vue            -> child's schedule
        reports.vue             -> child's reports
    messages/
      index.vue                 -> message center
    payments/
      index.vue                 -> payment history
    settings/
      index.vue                 -> parent settings
  admin/
    index.vue                   -> admin dashboard
    students/
      index.vue                 -> student management table
      [studentId].vue           -> student detail
    parents/
      index.vue                 -> parent management table
    teachers/
      index.vue                 -> teacher management table
      [teacherId].vue           -> teacher detail
    curricula/
      index.vue                 -> curriculum management
      [curriculumId]/
        index.vue               -> curriculum editor
        objectives.vue          -> learning objectives
    exams/
      index.vue                 -> exam management
      [examId]/
        index.vue               -> exam editor
        results.vue             -> exam results analytics
    analytics/
      index.vue                 -> analytics dashboard
      engagement.vue            -> engagement analytics
      performance.vue           -> performance analytics
      predictions.vue           -> predictive analytics
    content/
      index.vue                 -> content library management
      upload.vue                -> content upload
    system/
      index.vue                 -> system settings
      feature-flags.vue         -> feature flag management
      audit-log.vue             -> audit log viewer
      health.vue                -> system health monitor
```

#### 3.5.1 Route Guards

- `auth.global.ts` -- Redirects unauthenticated users to `/login`. Runs on every navigation.
- `role-access.ts` -- Ensures a student cannot navigate to `/admin/*`, a parent cannot navigate to `/student/*`, etc. Applied per-page via `definePageMeta`.
- `feature-flag.ts` -- Gates access to pages behind feature flags. If a feature is disabled, redirects to a "coming soon" page.
- `unsaved-changes.ts` -- Prompts the user before leaving pages with unsaved form data (notes editor, curriculum editor, etc.).

---

## 4. Complete Project Structure

```
ecoach/
├── src-tauri/                                 # Rust backend (Tauri)
│   ├── Cargo.toml                             # Rust dependencies
│   ├── tauri.conf.json                        # Tauri configuration (window, security, plugins)
│   ├── capabilities/                          # Tauri 2.x permission capabilities
│   ├── src/
│   │   ├── main.rs                            # Tauri application entry point
│   │   ├── lib.rs                             # Library root, command registration
│   │   ├── commands/                          # IPC command handlers organized by domain
│   │   ├── db/                                # SQLite database layer (sqlx)
│   │   ├── models/                            # Data models (serde structs)
│   │   ├── services/                          # Business logic layer
│   │   └── events/                            # Event emission helpers
│   └── icons/                                 # Application icons for all platforms
│
├── src/                                       # Nuxt 3 frontend (root of Nuxt project)
│   ├── app.vue                                # Root Vue component (shell selector)
│   ├── app.config.ts                          # Nuxt app configuration (runtime)
│   ├── error.vue                              # Global error page
│   │
│   ├── assets/                                # Static assets processed by Vite
│   │   ├── css/
│   │   │   ├── main.css                       # Tailwind directives (@tailwind base, components, utilities)
│   │   │   ├── fonts.css                      # @font-face declarations
│   │   │   ├── katex-overrides.css            # KaTeX rendering style overrides
│   │   │   ├── d3-overrides.css               # D3 chart style overrides
│   │   │   └── transitions.css                # Named CSS transition/animation keyframes
│   │   ├── fonts/
│   │   │   ├── inter/                         # Inter font files (variable weight)
│   │   │   ├── jetbrains-mono/                # JetBrains Mono font files (code)
│   │   │   └── literata/                      # Literata font files (reading/serif)
│   │   ├── icons/
│   │   │   └── custom/                        # Custom SVG icons extending Lucide set
│   │   ├── images/
│   │   │   ├── illustrations/                 # Empty state illustrations, onboarding art
│   │   │   ├── avatars/                       # Default avatar set
│   │   │   └── badges/                        # Achievement badge artwork (SVG)
│   │   └── sounds/
│   │       ├── notifications/                 # Notification sound effects (mp3/ogg)
│   │       ├── achievements/                  # Achievement unlock sounds
│   │       ├── timer/                          # Timer alarm sounds
│   │       └── ambient/                       # Ambient study sounds (rain, lo-fi loops, etc.)
│   │
│   ├── components/                            # Vue components (auto-imported by Nuxt)
│   │   ├── primitives/                        # Level 1: Design system primitives
│   │   │   ├── EButton.vue                    # Button with variants, sizes, loading state
│   │   │   ├── EIconButton.vue                # Icon-only button with tooltip
│   │   │   ├── EInput.vue                     # Text input with label, error, prefix/suffix slots
│   │   │   ├── ETextarea.vue                  # Multi-line text input with auto-resize
│   │   │   ├── ESelect.vue                    # Dropdown select with search, multi-select
│   │   │   ├── ECheckbox.vue                  # Checkbox with indeterminate state
│   │   │   ├── ERadio.vue                     # Radio button
│   │   │   ├── ERadioGroup.vue                # Radio button group with horizontal/vertical layout
│   │   │   ├── ESwitch.vue                    # Toggle switch with label
│   │   │   ├── ESlider.vue                    # Range slider (single and dual-thumb)
│   │   │   ├── EBadge.vue                     # Badge/tag with color variants and removable
│   │   │   ├── EChip.vue                      # Interactive chip (selectable, filterable)
│   │   │   ├── EAvatar.vue                    # User avatar with fallback initials and status dot
│   │   │   ├── ECard.vue                      # Container card with header, body, footer slots
│   │   │   ├── EModal.vue                     # Modal dialog with backdrop, focus trap, esc-close
│   │   │   ├── EDrawer.vue                    # Side drawer (left/right/bottom) with overlay
│   │   │   ├── ESheet.vue                     # Bottom sheet (mobile-style pull-up panel)
│   │   │   ├── EDropdown.vue                  # Dropdown menu with items, dividers, nested menus
│   │   │   ├── EPopover.vue                   # Popover with arrow, placement, trigger modes
│   │   │   ├── ETooltip.vue                   # Tooltip with delay, placement, rich content
│   │   │   ├── EToast.vue                     # Toast notification (success, error, warning, info)
│   │   │   ├── EAlert.vue                     # Inline alert/banner with icon and dismiss
│   │   │   ├── EProgress.vue                  # Progress bar (determinate, indeterminate, circular)
│   │   │   ├── ESkeleton.vue                  # Skeleton loader (text, circle, rect, custom shape)
│   │   │   ├── ESpinner.vue                   # Loading spinner with size variants
│   │   │   ├── EDivider.vue                   # Horizontal/vertical divider with optional label
│   │   │   ├── EAccordion.vue                 # Expandable accordion (single/multi-open)
│   │   │   ├── ETabs.vue                      # Tab navigation (horizontal, vertical, pills)
│   │   │   ├── ETabPanel.vue                  # Tab content panel (lazy-rendered)
│   │   │   ├── EBreadcrumb.vue                # Breadcrumb navigation with overflow handling
│   │   │   ├── EPagination.vue                # Pagination controls with page size selector
│   │   │   ├── ETable.vue                     # Data table with sorting, selection, row actions
│   │   │   ├── ETableColumn.vue               # Table column definition (sortable, filterable)
│   │   │   ├── ETag.vue                       # Colored tag for categorization
│   │   │   ├── EKbd.vue                       # Keyboard shortcut display (<kbd> styled)
│   │   │   ├── EEmptyState.vue                # Empty state with illustration, title, description, action
│   │   │   ├── EErrorState.vue                # Error state with retry action
│   │   │   ├── EScrollArea.vue                # Custom scrollbar container
│   │   │   ├── ECollapsible.vue               # Collapsible content section
│   │   │   ├── ECommandPalette.vue            # Command palette (Cmd+K) with search, categories
│   │   │   ├── EContextMenu.vue               # Right-click context menu
│   │   │   ├── EDatePicker.vue                # Date picker with range selection
│   │   │   ├── ETimePicker.vue                # Time picker with 12/24h format
│   │   │   └── EColorPicker.vue               # Color picker for customization features
│   │   │
│   │   ├── composite/                         # Level 2: Multi-primitive compositions
│   │   │   ├── EFormField.vue                 # Label + input + error + hint composition
│   │   │   ├── ESearchInput.vue               # Input with search icon, clear, debounce, results dropdown
│   │   │   ├── EFileUpload.vue                # File upload with drag-and-drop, preview, progress
│   │   │   ├── EDataTable.vue                 # Full data table with pagination, filters, bulk actions
│   │   │   ├── EDataGrid.vue                  # Grid layout for card-based data display
│   │   │   ├── EStatCard.vue                  # Statistic display card with trend indicator
│   │   │   ├── EMasteryBadge.vue              # Mastery level badge (8 states with colors and icons)
│   │   │   ├── EMasteryRing.vue               # Circular mastery progress indicator
│   │   │   ├── EDecayIndicator.vue            # Visual decay level indicator (5 states)
│   │   │   ├── EXPBar.vue                     # XP progress bar with level indicator
│   │   │   ├── EStreakCounter.vue             # Streak display with flame animation
│   │   │   ├── EAchievementCard.vue           # Achievement badge card with locked/unlocked states
│   │   │   ├── ELeaderboardRow.vue            # Leaderboard entry with rank, avatar, score
│   │   │   ├── EScheduleBlock.vue             # Schedule time block (draggable, resizable)
│   │   │   ├── ETimeline.vue                  # Vertical timeline for activity history
│   │   │   ├── ETimelineItem.vue              # Single timeline entry
│   │   │   ├── ECalendarDay.vue               # Single day cell in calendar view
│   │   │   ├── ECalendarWeek.vue              # Week view for schedule
│   │   │   ├── ECalendarMonth.vue             # Month view for schedule
│   │   │   ├── ENotificationItem.vue          # Notification list item with actions
│   │   │   ├── EUserCard.vue                  # User info card (avatar, name, role, status)
│   │   │   ├── ESubjectCard.vue               # Subject card with mastery, grade, icon
│   │   │   ├── ETopicCard.vue                 # Topic card with mastery state and decay
│   │   │   ├── EQuestionCard.vue              # Practice question display with math rendering
│   │   │   ├── EAnswerInput.vue               # Answer input (multiple choice, free text, math input)
│   │   │   ├── ESolutionDisplay.vue           # Step-by-step solution with KaTeX rendering
│   │   │   ├── ENoteCard.vue                  # Note preview card with tags and modified date
│   │   │   ├── EGoalCard.vue                  # Goal card with progress bar and deadline
│   │   │   ├── EExamCountdown.vue             # Exam countdown timer with urgency colors
│   │   │   ├── EReportCard.vue                # Report card component for PDF and screen
│   │   │   ├── EGradeDisplay.vue              # Grade display with color coding (A-F, percentage)
│   │   │   ├── EEmotionSelector.vue           # Emotional state picker (6 modes)
│   │   │   ├── ERichTextEditor.vue            # Rich text editor for notes (Tiptap-based)
│   │   │   ├── EMathInput.vue                 # Mathematical expression input with preview
│   │   │   ├── ECodeBlock.vue                 # Syntax-highlighted code display
│   │   │   ├── EAudioPlayer.vue               # Audio player with waveform and controls
│   │   │   └── EConfirmDialog.vue             # Confirmation dialog (destructive/non-destructive)
│   │   │
│   │   ├── charts/                            # Level 2: D3-powered chart components
│   │   │   ├── EMasteryTree.vue               # Mastery tree visualization (node-link diagram)
│   │   │   ├── EKnowledgeGraph.vue            # Knowledge graph with force-directed layout
│   │   │   ├── EGradeTrendChart.vue           # Line chart for grade trends over time
│   │   │   ├── EAttendanceHeatmap.vue         # Calendar heatmap for attendance/activity
│   │   │   ├── EPerformanceRadar.vue          # Radar chart for multi-subject performance
│   │   │   ├── EDistributionHistogram.vue     # Histogram for score distributions
│   │   │   ├── EProgressTimeline.vue          # Timeline chart for progress milestones
│   │   │   ├── EComparativeBar.vue            # Comparative bar chart (student vs class avg)
│   │   │   ├── ELearningPathDAG.vue           # Directed acyclic graph for learning paths
│   │   │   ├── EScheduleGantt.vue             # Gantt chart for schedule visualization
│   │   │   ├── EEngagementChart.vue           # Engagement metrics over time
│   │   │   └── EPredictionChart.vue           # Predictive analytics visualization
│   │   │
│   │   ├── canvas/                            # Level 2: PixiJS-powered components
│   │   │   ├── EParticleEffect.vue            # Particle system for celebrations
│   │   │   ├── EXPOrbs.vue                    # Animated XP orb collection
│   │   │   ├── ESkillConstellation.vue        # Interactive skill constellation map
│   │   │   ├── EProgressRing.vue              # GPU-accelerated progress ring with glow
│   │   │   └── EAchievementUnlock.vue         # Achievement unlock animation sequence
│   │   │
│   │   ├── features/                          # Level 3: Feature-level components
│   │   │   ├── student/
│   │   │   │   ├── StudentDashboard.vue       # Student dashboard layout with widgets
│   │   │   │   ├── SubjectBrowser.vue         # Subject grid/list with filtering
│   │   │   │   ├── TopicMasteryView.vue       # Topic detail with mastery tree
│   │   │   │   ├── PracticeSession.vue        # Active practice session controller
│   │   │   │   ├── PracticeResults.vue        # Practice session results summary
│   │   │   │   ├── ScheduleManager.vue        # Schedule view/edit interface
│   │   │   │   ├── NoteEditor.vue             # Full note editor with toolbar
│   │   │   │   ├── NotesBrowser.vue           # Notes list with search and tags
│   │   │   │   ├── GoalDashboard.vue          # Goals overview with progress tracking
│   │   │   │   ├── GoalEditor.vue             # Goal creation/editing form
│   │   │   │   ├── AchievementGallery.vue     # Achievement grid with categories
│   │   │   │   ├── StudyRoom.vue              # Study room with timer and ambient sounds
│   │   │   │   ├── PomodoroTimer.vue          # Pomodoro timer with phases
│   │   │   │   ├── AmbientSoundMixer.vue      # Ambient sound selection and mixing
│   │   │   │   ├── ExamPreparation.vue        # Exam prep checklist and study plan
│   │   │   │   ├── FlashcardDeck.vue          # Flashcard study interface
│   │   │   │   ├── FormulaSheet.vue           # Formula reference sheet with KaTeX
│   │   │   │   ├── FocusMode.vue              # Distraction-free mode wrapper
│   │   │   │   └── DailyReview.vue            # Daily review/summary screen
│   │   │   │
│   │   │   ├── parent/
│   │   │   │   ├── ParentDashboard.vue        # Parent overview dashboard
│   │   │   │   ├── ChildSelector.vue          # Multi-child selector widget
│   │   │   │   ├── ChildOverview.vue          # Per-child performance overview
│   │   │   │   ├── ChildComparison.vue        # Side-by-side child comparison
│   │   │   │   ├── ProgressReport.vue         # Detailed progress report viewer
│   │   │   │   ├── ReportExporter.vue         # PDF report generation interface
│   │   │   │   ├── MessageCenter.vue          # Parent-teacher messaging interface
│   │   │   │   ├── PaymentHistory.vue         # Payment and subscription management
│   │   │   │   └── ParentNotifications.vue    # Notification preferences and digest
│   │   │   │
│   │   │   └── admin/
│   │   │       ├── AdminDashboard.vue         # Admin overview with KPIs
│   │   │       ├── StudentManager.vue         # Student CRUD with bulk operations
│   │   │       ├── ParentManager.vue          # Parent CRUD with child associations
│   │   │       ├── TeacherManager.vue         # Teacher CRUD with subject assignments
│   │   │       ├── CurriculumEditor.vue       # Curriculum tree editor
│   │   │       ├── ObjectiveEditor.vue        # Learning objective editor
│   │   │       ├── ExamBuilder.vue            # Exam creation and question bank
│   │   │       ├── QuestionEditor.vue         # Question editor with KaTeX
│   │   │       ├── AnalyticsDashboard.vue     # Multi-metric analytics view
│   │   │       ├── EngagementAnalytics.vue    # Student engagement analysis
│   │   │       ├── PerformanceAnalytics.vue   # Performance trend analysis
│   │   │       ├── PredictiveAnalytics.vue    # At-risk student prediction
│   │   │       ├── ContentManager.vue         # Content library CRUD
│   │   │       ├── ContentUploader.vue        # Bulk content upload interface
│   │   │       ├── FeatureFlagManager.vue     # Feature flag toggle interface
│   │   │       ├── AuditLogViewer.vue         # Audit log table with filters
│   │   │       ├── SystemHealthMonitor.vue    # System metrics dashboard
│   │   │       └── BulkOperationsBar.vue      # Bulk action toolbar for tables
│   │   │
│   │   └── shell/                             # Level 5: Layout shell components
│   │       ├── StudentShell.vue               # Student portal shell (sidebar + topbar + main)
│   │       ├── ParentShell.vue                # Parent portal shell (topnav + sidebar + main)
│   │       ├── AdminShell.vue                 # Admin portal shell (sidebar + topbar + main)
│   │       ├── AppSidebar.vue                 # Sidebar navigation (collapsible)
│   │       ├── AppTopbar.vue                  # Top bar (breadcrumbs, search, notifications)
│   │       ├── AppSearch.vue                  # Global search overlay
│   │       ├── NotificationPanel.vue          # Notification dropdown panel
│   │       ├── UserMenu.vue                   # User avatar dropdown menu
│   │       ├── ThemeToggle.vue                # Dark/light mode toggle
│   │       └── GamificationHUD.vue            # XP bar, level, streak overlay (student only)
│   │
│   ├── composables/                           # Vue composables (auto-imported by Nuxt)
│   │   ├── ipc/                               # Tauri IPC wrappers
│   │   │   ├── useMasteryIpc.ts               # Mastery domain IPC commands
│   │   │   ├── useScheduleIpc.ts              # Schedule domain IPC commands
│   │   │   ├── usePracticeIpc.ts              # Practice domain IPC commands
│   │   │   ├── useExamIpc.ts                  # Exam domain IPC commands
│   │   │   ├── useNotesIpc.ts                 # Notes domain IPC commands
│   │   │   ├── useGoalIpc.ts                  # Goals domain IPC commands
│   │   │   ├── useGamificationIpc.ts          # Gamification domain IPC commands
│   │   │   ├── useAuthIpc.ts                  # Authentication IPC commands
│   │   │   ├── useSyncIpc.ts                  # Sync domain IPC commands
│   │   │   ├── useAnalyticsIpc.ts             # Analytics domain IPC commands
│   │   │   ├── useContentIpc.ts               # Content domain IPC commands
│   │   │   ├── useCurriculumIpc.ts            # Curriculum domain IPC commands
│   │   │   ├── useAdminIpc.ts                 # Admin-only IPC commands
│   │   │   └── useStoreIpc.ts                 # Store hydration/persistence IPC
│   │   │
│   │   ├── domain/                            # Cross-store domain logic
│   │   │   ├── useCompletePractice.ts         # Practice completion with XP + mastery update
│   │   │   ├── useStudySession.ts             # Study session lifecycle management
│   │   │   ├── useMasteryDecay.ts             # Mastery decay calculation and display
│   │   │   ├── useAdaptiveDifficulty.ts       # Adaptive difficulty for practice sessions
│   │   │   ├── useExamPreparation.ts          # Exam prep workflow orchestration
│   │   │   ├── useStreakTracking.ts            # Streak calculation and maintenance
│   │   │   ├── useGoalProgress.ts             # Goal progress computation
│   │   │   └── useLearningPath.ts             # Learning path recommendation logic
│   │   │
│   │   ├── ui/                                # UI-related composables
│   │   │   ├── useTheme.ts                    # Theme management (dark/light/system)
│   │   │   ├── useBreakpoint.ts               # Responsive breakpoint detection
│   │   │   ├── useToast.ts                    # Toast notification trigger
│   │   │   ├── useConfirm.ts                  # Confirmation dialog trigger
│   │   │   ├── useCommandPalette.ts           # Command palette registration and trigger
│   │   │   ├── useShortcuts.ts                # Keyboard shortcut registration
│   │   │   ├── useFocusTrap.ts                # Focus trap for modals/drawers
│   │   │   ├── useVirtualScroll.ts            # Virtual scrolling for large lists
│   │   │   ├── useInfiniteScroll.ts           # Infinite scroll with cursor pagination
│   │   │   ├── useDragDrop.ts                 # Drag and drop utilities
│   │   │   ├── useContextMenu.ts              # Context menu registration
│   │   │   ├── useMotion.ts                   # Motion One animation helpers
│   │   │   └── useReducedMotion.ts            # Reduced motion preference detection
│   │   │
│   │   └── audio/                             # Audio-related composables
│   │       ├── useNotificationSound.ts        # Play notification sounds via Howler
│   │       ├── useAmbientSound.ts             # Ambient sound management
│   │       └── useTimerAlarm.ts               # Timer alarm playback
│   │
│   ├── layouts/                               # Nuxt layouts
│   │   ├── default.vue                        # Default layout (redirect to role-specific)
│   │   ├── student.vue                        # Student shell layout
│   │   ├── parent.vue                         # Parent shell layout
│   │   ├── admin.vue                          # Admin shell layout
│   │   ├── auth.vue                           # Authentication layout (centered card)
│   │   └── blank.vue                          # Blank layout (for focus mode, print views)
│   │
│   ├── middleware/                             # Nuxt route middleware
│   │   ├── auth.global.ts                     # Authentication check (global, runs on every nav)
│   │   ├── role-router.global.ts              # Role-based shell selection (global)
│   │   ├── role-access.ts                     # Per-route role authorization
│   │   ├── feature-flag.ts                    # Feature flag gate
│   │   └── unsaved-changes.ts                 # Unsaved changes warning
│   │
│   ├── pages/                                 # Nuxt file-based routing (see section 3.5)
│   │   ├── index.vue
│   │   ├── login.vue
│   │   ├── student/                           # (full tree documented in section 3.5)
│   │   ├── parent/                            # (full tree documented in section 3.5)
│   │   └── admin/                             # (full tree documented in section 3.5)
│   │
│   ├── plugins/                               # Nuxt plugins
│   │   ├── 01.tauri.client.ts                 # Tauri API initialization (client-only)
│   │   ├── 02.pinia-persist.ts                # Pinia persistence plugin registration
│   │   ├── 03.katex.client.ts                 # KaTeX global configuration
│   │   ├── 04.motion.client.ts               # Motion One initialization
│   │   ├── 05.howler.client.ts               # Howler.js global audio context setup
│   │   ├── 06.d3.client.ts                   # D3 locale and format configuration
│   │   └── 07.error-handler.ts               # Global error handler (logs to Rust backend)
│   │
│   ├── stores/                                # Pinia stores
│   │   ├── app.ts                             # App-level state (shell, theme, locale, loading)
│   │   ├── auth.ts                            # Authentication state (user, session, permissions)
│   │   ├── preferences.ts                     # User preferences (font size, dark mode, sounds)
│   │   ├── notifications.ts                   # Notification queue and state
│   │   ├── sync.ts                            # Sync status and conflict management
│   │   ├── mastery.ts                         # Mastery states, transitions, decay tracking
│   │   ├── schedule.ts                        # Schedule blocks, events, calendar
│   │   ├── practice.ts                        # Practice session state, question queue
│   │   ├── exams.ts                           # Exam data, results, preparation
│   │   ├── notes.ts                           # Notes tree, editor state, tags
│   │   ├── goals.ts                           # Goals, milestones, progress
│   │   ├── gamification.ts                    # XP, level, streak, achievements, badges
│   │   ├── study-room.ts                      # Pomodoro state, ambient sounds
│   │   ├── curriculum.ts                      # Curriculum tree, objectives
│   │   ├── analytics.ts                       # Computed analytics, chart data
│   │   ├── content.ts                         # Content library, resource metadata
│   │   ├── messages.ts                        # Conversations, threads
│   │   ├── payments.ts                        # Subscription, payment history
│   │   ├── audit.ts                           # Admin audit log
│   │   └── system.ts                          # Admin system health, feature flags
│   │
│   ├── types/                                 # TypeScript type definitions
│   │   ├── ipc/
│   │   │   ├── contracts.ts                   # IPC command/event contract registry
│   │   │   ├── commands.ts                    # Command payload types
│   │   │   ├── events.ts                      # Event payload types
│   │   │   └── errors.ts                      # IPC error types (tagged union)
│   │   ├── domain/
│   │   │   ├── user.ts                        # User, Student, Parent, Admin, Teacher types
│   │   │   ├── mastery.ts                     # MasteryState, MasteryLevel, DecayLevel types
│   │   │   ├── subject.ts                     # Subject, Topic, Subtopic types
│   │   │   ├── schedule.ts                    # ScheduleBlock, RecurringEvent types
│   │   │   ├── practice.ts                    # PracticeSession, Question, Answer types
│   │   │   ├── exam.ts                        # Exam, ExamResult, ExamQuestion types
│   │   │   ├── note.ts                        # Note, NoteTag, NoteTree types
│   │   │   ├── goal.ts                        # Goal, Milestone, GoalProgress types
│   │   │   ├── gamification.ts                # XP, Level, Streak, Achievement, Badge types
│   │   │   ├── curriculum.ts                  # Curriculum, Objective, LearningPath types
│   │   │   ├── content.ts                     # ContentItem, Resource, MediaType types
│   │   │   ├── analytics.ts                   # AnalyticsMetric, TrendData, Prediction types
│   │   │   ├── notification.ts                # Notification, NotificationType types
│   │   │   ├── message.ts                     # Message, Conversation, Thread types
│   │   │   └── payment.ts                     # Subscription, Invoice, PaymentMethod types
│   │   ├── ui/
│   │   │   ├── component-props.ts             # Shared prop types (Size, Variant, Color)
│   │   │   ├── form.ts                        # Form field, validation rule types
│   │   │   ├── table.ts                       # Table column, sort, filter types
│   │   │   ├── chart.ts                       # Chart data, axis, legend types
│   │   │   └── theme.ts                       # Theme, ColorToken, DesignToken types
│   │   └── util/
│   │       ├── branded.ts                     # Branded types (StudentId, SubjectId, etc.)
│   │       ├── result.ts                      # Result<T, E> type for error handling
│   │       ├── pagination.ts                  # Pagination, CursorPage, OffsetPage types
│   │       └── common.ts                      # Timestamp, UUID, NonEmptyString, etc.
│   │
│   ├── utils/                                 # Utility functions (auto-imported by Nuxt)
│   │   ├── cn.ts                              # Tailwind class merge utility (clsx + twMerge)
│   │   ├── format.ts                          # Date, number, percentage formatting
│   │   ├── validators.ts                      # Zod schemas for form validation
│   │   ├── ipc-schemas.ts                     # Zod schemas for IPC payload validation
│   │   ├── color.ts                           # Color manipulation (lighten, darken, alpha)
│   │   ├── math-render.ts                     # KaTeX rendering helpers
│   │   ├── pdf.ts                             # jsPDF document generation helpers
│   │   ├── audio.ts                           # Howler.js sound sprite helpers
│   │   ├── dates.ts                           # Date arithmetic, school year/term helpers
│   │   ├── mastery.ts                         # Mastery level calculation utilities
│   │   ├── xp.ts                              # XP and level calculation formulas
│   │   ├── decay.ts                           # Decay rate calculation
│   │   ├── permissions.ts                     # Role-based permission checking
│   │   ├── debounce.ts                        # Debounce/throttle utilities
│   │   ├── retry.ts                           # Retry with backoff for IPC calls
│   │   └── id.ts                              # ID generation (nanoid wrapper)
│   │
│   ├── public/                                # Static files served as-is
│   │   └── favicon.ico                        # Application favicon
│   │
│   └── server/                                # Nuxt server directory (minimal use in Tauri)
│       └── tsconfig.json                      # Server-specific TypeScript config
│
├── nuxt.config.ts                             # Nuxt configuration
├── tailwind.config.ts                         # Tailwind CSS configuration (all design tokens)
├── tsconfig.json                              # Root TypeScript configuration
├── .eslintrc.cjs                              # ESLint configuration
├── .prettierrc                                # Prettier configuration
├── vitest.config.ts                           # Vitest test runner configuration
├── playwright.config.ts                       # Playwright E2E test configuration
├── package.json                               # NPM dependencies and scripts
└── pnpm-lock.yaml                             # Lockfile (pnpm is the package manager)
```

---

## 5. Complete Design System

### 5.1 Color Tokens

All colors are defined as CSS custom properties in HSL format and mapped to Tailwind utilities. Each palette includes a full range from 50 (lightest) to 950 (darkest).

#### 5.1.1 Brand Colors

```
--brand-primary:     222 59% 52%      // Deep scholarly blue
--brand-secondary:   262 52% 56%      // Wisdom purple
--brand-accent:      172 56% 42%      // Growth teal
```

#### 5.1.2 Student Portal Palette

```
--student-50:        210 40% 98%
--student-100:       210 40% 96%
--student-200:       214 32% 91%
--student-300:       213 27% 84%
--student-400:       215 20% 65%
--student-500:       220 18% 50%      // Primary student surface
--student-600:       222 20% 42%
--student-700:       224 22% 33%
--student-800:       226 25% 25%
--student-900:       228 28% 18%
--student-950:       230 32% 10%

--student-accent:    200 85% 48%      // Student call-to-action blue
--student-success:   152 60% 42%      // Correct answer green
--student-focus:     250 50% 55%      // Focus mode purple
```

#### 5.1.3 Parent Portal Palette

```
--parent-50:         140 30% 97%
--parent-100:        142 30% 94%
--parent-200:        144 25% 87%
--parent-300:        146 22% 78%
--parent-400:        148 18% 60%
--parent-500:        150 16% 46%      // Primary parent surface
--parent-600:        152 18% 38%
--parent-700:        154 20% 30%
--parent-800:        156 24% 22%
--parent-900:        158 28% 15%
--parent-950:        160 32% 8%

--parent-accent:     160 65% 40%      // Parent nurturing green
--parent-alert:      0 72% 55%        // Urgent alert red
--parent-info:       210 70% 52%      // Informational blue
```

#### 5.1.4 Admin Portal Palette

```
--admin-50:          250 25% 97%
--admin-100:         252 25% 94%
--admin-200:         254 20% 88%
--admin-300:         256 18% 78%
--admin-400:         258 15% 60%
--admin-500:         260 14% 46%      // Primary admin surface
--admin-600:         262 16% 38%
--admin-700:         264 18% 30%
--admin-800:         266 22% 22%
--admin-900:         268 26% 15%
--admin-950:         270 30% 8%

--admin-accent:      262 60% 52%      // Admin authority purple
--admin-system:      220 55% 50%      // System metrics blue
--admin-danger:      0 68% 52%        // Destructive action red
```

#### 5.1.5 Mastery Level Colors (8 States)

Each state has a primary color, a background tint, a text color, and an icon.

```
--mastery-unstarted:       220 15% 60%       // Gray -- not yet attempted
--mastery-unstarted-bg:    220 15% 95%
--mastery-unstarted-text:  220 15% 40%

--mastery-exposed:         200 60% 52%       // Light blue -- seen but not practiced
--mastery-exposed-bg:      200 60% 94%
--mastery-exposed-text:    200 60% 30%

--mastery-developing:      180 50% 45%       // Teal -- early practice
--mastery-developing-bg:   180 50% 93%
--mastery-developing-text: 180 50% 25%

--mastery-emerging:        160 55% 42%       // Green-teal -- building understanding
--mastery-emerging-bg:     160 55% 92%
--mastery-emerging-text:   160 55% 24%

--mastery-proficient:      140 55% 40%       // Green -- solid understanding
--mastery-proficient-bg:   140 55% 91%
--mastery-proficient-text: 140 55% 22%

--mastery-advanced:        80 60% 42%        // Lime-green -- above expectations
--mastery-advanced-bg:     80 60% 91%
--mastery-advanced-text:   80 60% 24%

--mastery-expert:          45 80% 50%        // Gold -- exceptional mastery
--mastery-expert-bg:       45 80% 92%
--mastery-expert-text:     45 80% 28%

--mastery-transcendent:    280 65% 55%       // Purple -- beyond curriculum, creative mastery
--mastery-transcendent-bg: 280 65% 93%
--mastery-transcendent-text: 280 65% 30%
```

#### 5.1.6 Error Type Colors (10 Types)

Categorized error colors for diagnostic precision in practice feedback.

```
--error-conceptual:        0 72% 55%         // Red -- fundamental misunderstanding
--error-procedural:        20 75% 52%        // Orange-red -- wrong method/steps
--error-computational:     35 80% 50%        // Orange -- arithmetic/calculation errors
--error-transcription:     50 70% 48%        // Amber -- copying/writing errors
--error-reading:           280 50% 55%       // Purple -- misread the question
--error-notation:          200 55% 50%       // Blue -- notation/symbol errors
--error-unit:              320 55% 52%       // Pink -- unit/dimension errors
--error-sign:              170 50% 45%       // Teal -- sign/direction errors
--error-boundary:          100 45% 45%       // Olive-green -- edge case/boundary errors
--error-timeout:           220 30% 50%       // Slate -- ran out of time
```

#### 5.1.7 Decay Level Colors (5 Levels)

Visual indicators for knowledge retention decay since last practice.

```
--decay-none:              140 55% 40%       // Green -- recently practiced, no decay
--decay-slight:            90 50% 48%        // Yellow-green -- 3-7 days, minor decay
--decay-moderate:          45 70% 50%        // Amber -- 1-2 weeks, noticeable decay
--decay-significant:       20 75% 52%        // Orange -- 2-4 weeks, significant decay
--decay-critical:          0 72% 55%         // Red -- 4+ weeks, critical decay
```

#### 5.1.8 Emotional Mode Colors (6 Modes)

The UI adapts its accent color and ambient tone based on the student's self-reported emotional state.

```
--emotion-focused:         220 70% 55%       // Blue -- calm concentration
--emotion-focused-bg:      220 20% 97%       // Cool, clean background

--emotion-confident:       140 60% 42%       // Green -- feeling strong
--emotion-confident-bg:    140 15% 97%       // Warm, affirming background

--emotion-curious:         270 55% 55%       // Purple -- exploring, experimenting
--emotion-curious-bg:      270 15% 97%       // Inviting, open background

--emotion-struggling:      35 70% 52%        // Warm amber -- needs support
--emotion-struggling-bg:   35 15% 97%        // Warm, supportive background

--emotion-frustrated:      0 55% 52%         // Muted red -- feeling stuck
--emotion-frustrated-bg:   0 10% 97%         // Calming, de-escalating background

--emotion-tired:           210 20% 55%       // Soft gray-blue -- low energy
--emotion-tired-bg:        210 10% 97%       // Gentle, easy-on-eyes background
```

#### 5.1.9 Semantic Colors

```
--success:                 152 60% 42%       // Positive outcome, correct answer
--success-bg:              152 60% 94%
--success-text:            152 60% 22%
--success-border:          152 60% 80%

--warning:                 38 92% 50%        // Caution, upcoming deadline
--warning-bg:              38 92% 94%
--warning-text:            38 92% 24%
--warning-border:          38 92% 78%

--error:                   0 72% 55%         // Error, failure, invalid
--error-bg:                0 72% 95%
--error-text:              0 72% 28%
--error-border:            0 72% 82%

--info:                    210 70% 52%       // Informational, neutral
--info-bg:                 210 70% 95%
--info-text:               210 70% 26%
--info-border:             210 70% 82%
```

#### 5.1.10 Neutral/Surface Colors

```
--surface-0:               0 0% 100%         // White (card backgrounds)
--surface-1:               220 14% 98%       // Subtle off-white (page background)
--surface-2:               220 13% 96%       // Elevated surface
--surface-3:               220 12% 93%       // Higher elevation
--surface-4:               220 11% 89%       // Highest elevation

--border-light:            220 13% 91%       // Subtle borders
--border-default:          220 13% 82%       // Default borders
--border-strong:           220 13% 65%       // Emphasized borders

--text-primary:            224 28% 12%       // Primary text
--text-secondary:          220 15% 40%       // Secondary text
--text-tertiary:           220 10% 58%       // Tertiary/muted text
--text-disabled:           220 8% 72%        // Disabled text
--text-inverse:            0 0% 100%         // Text on dark backgrounds
```

#### 5.1.11 Dark Mode Overrides

Dark mode inverts the surface scale and adjusts all palette luminosities. Triggered by `.dark` class on `<html>`.

```
.dark {
  --surface-0:             224 28% 8%        // Dark card backgrounds
  --surface-1:             224 25% 10%       // Dark page background
  --surface-2:             224 22% 13%       // Dark elevated surface
  --surface-3:             224 20% 17%       // Dark higher elevation
  --surface-4:             224 18% 21%       // Dark highest elevation

  --border-light:          224 15% 18%
  --border-default:        224 12% 25%
  --border-strong:         224 10% 38%

  --text-primary:          210 20% 92%
  --text-secondary:        210 12% 68%
  --text-tertiary:         210 8% 50%
  --text-disabled:         210 5% 35%
  --text-inverse:          224 28% 8%

  /* All palette colors shift: backgrounds darken, foregrounds lighten */
  /* Mastery, error, decay, emotion colors maintain relative contrast */
  /* Accent colors increase saturation slightly to compensate for dark backgrounds */
}
```

### 5.2 Typography

#### 5.2.1 Font Families

```
--font-sans:          'Inter Variable', 'Inter', system-ui, -apple-system, sans-serif
--font-serif:         'Literata Variable', 'Literata', Georgia, 'Times New Roman', serif
--font-mono:          'JetBrains Mono Variable', 'JetBrains Mono', 'Fira Code', monospace
--font-math:          'KaTeX_Main', 'Latin Modern Math', 'STIX Two Math', serif
```

**Inter** (sans-serif): Primary UI font. Used for all interface text: navigation, buttons, labels, form inputs, body copy, headings. Variable font with weight axis 100-900.

**Literata** (serif): Reading font. Used for long-form content: notes, study material, question text, solution explanations. Optimized for extended on-screen reading.

**JetBrains Mono** (monospace): Code font. Used for code blocks, formula input fields, ID displays, audit log entries, and any fixed-width content.

**KaTeX_Main** (math): Mathematical notation font. Loaded by KaTeX for rendered math expressions. Not used directly in CSS.

#### 5.2.2 Type Scale

Based on a 1.200 minor third ratio with a 16px base.

```
--text-xs:            0.694rem    // 11.1px  -- fine print, footnotes, timestamps
--text-sm:            0.833rem    // 13.3px  -- captions, helper text, badges
--text-base:          1rem        // 16px    -- body text, input text, list items
--text-md:            1.2rem      // 19.2px  -- emphasized body, card titles
--text-lg:            1.44rem     // 23px    -- section headings, modal titles
--text-xl:            1.728rem    // 27.6px  -- page headings
--text-2xl:           2.074rem    // 33.2px  -- dashboard hero metrics
--text-3xl:           2.488rem    // 39.8px  -- landing page headings
--text-4xl:           2.986rem    // 47.8px  -- display headings (achievement unlock, level up)
```

#### 5.2.3 Font Weights

```
--font-light:         300         // Rarely used; large display text only
--font-regular:       400         // Body text, input text, descriptions
--font-medium:        500         // Labels, navigation items, subtle emphasis
--font-semibold:      600         // Headings, button text, card titles
--font-bold:          700         // Strong emphasis, page headings, metrics
--font-extrabold:     800         // Display headings, hero numbers, level indicators
```

#### 5.2.4 Line Heights

```
--leading-none:       1           // Metrics, display numbers, single-line badges
--leading-tight:      1.25        // Headings (text-lg and above)
--leading-snug:       1.375       // Card titles, subheadings
--leading-normal:     1.5         // Body text, form labels
--leading-relaxed:    1.625       // Long-form reading content (Literata)
--leading-loose:      2           // Spaced-out list items, accessibility mode
```

#### 5.2.5 Letter Spacing

```
--tracking-tighter:   -0.05em    // Large display headings (text-3xl+)
--tracking-tight:     -0.025em   // Headings (text-xl, text-2xl)
--tracking-normal:    0           // Body text
--tracking-wide:      0.025em    // Uppercase labels, small caps
--tracking-wider:     0.05em     // Badge text, button text (when uppercase)
--tracking-widest:    0.1em      // Tiny all-caps labels
```

#### 5.2.6 Role-Specific Base Sizes

The base font size adjusts per portal to match the primary audience:

```
Student portal:       16px       // Standard readability for students
Parent portal:        16px       // Comfortable for adult reading
Admin portal:         14px       // Denser information display
```

These are applied via `font-size` on the `<html>` element when the shell loads, and all `rem`-based sizes scale accordingly.

### 5.3 Spacing Scale

4px grid system. Every spacing value is a multiple of 4px.

```
--space-0:         0px
--space-0.5:       2px           // Hairline gaps (icon-to-text nudge, border offsets)
--space-1:         4px           // Minimum spacing (inline badge padding, tight groups)
--space-1.5:       6px           // Small icon padding
--space-2:         8px           // Compact padding (badge, chip, small button)
--space-3:         12px          // Default inline spacing (icon + label gap)
--space-4:         16px          // Default padding (card, input, button)
--space-5:         20px          // Comfortable padding (form fields)
--space-6:         24px          // Section padding within cards
--space-8:         32px          // Card margin, section gap
--space-10:        40px          // Large section gap
--space-12:        48px          // Page section gap
--space-16:        64px          // Major section divider
--space-20:        80px          // Page-level vertical rhythm
--space-24:        96px          // Large layout gaps
--space-32:        128px         // Hero section padding
--space-40:        160px         // Maximum layout spacing
```

### 5.4 Border Radius Scale

```
--radius-none:     0px           // Square corners (code blocks, tables)
--radius-sm:       4px           // Subtle rounding (badges, chips, small buttons)
--radius-md:       6px           // Default rounding (inputs, cards, buttons)
--radius-lg:       8px           // Prominent rounding (modals, drawers)
--radius-xl:       12px          // Strong rounding (dropdown menus, popovers)
--radius-2xl:      16px          // Large rounding (dashboard cards, image containers)
--radius-3xl:      24px          // Very rounded (pill buttons, floating action buttons)
--radius-full:     9999px        // Circular (avatars, status dots, round buttons)
```

### 5.5 Shadow / Elevation Scale

Each elevation level has a composite box-shadow value. Shadows use a two-layer approach (ambient + direct) for realism.

```
--shadow-xs:
  0 1px 2px 0 hsl(0 0% 0% / 0.03),
  0 1px 1px 0 hsl(0 0% 0% / 0.02)
  // Usage: Subtle elevation (badges, chips, inline elements)

--shadow-sm:
  0 1px 3px 0 hsl(0 0% 0% / 0.06),
  0 1px 2px -1px hsl(0 0% 0% / 0.04)
  // Usage: Cards at rest, form inputs on focus

--shadow-md:
  0 4px 6px -1px hsl(0 0% 0% / 0.07),
  0 2px 4px -2px hsl(0 0% 0% / 0.05)
  // Usage: Cards on hover, dropdowns, popovers

--shadow-lg:
  0 10px 15px -3px hsl(0 0% 0% / 0.08),
  0 4px 6px -4px hsl(0 0% 0% / 0.05)
  // Usage: Modals, drawers, floating panels

--shadow-xl:
  0 20px 25px -5px hsl(0 0% 0% / 0.08),
  0 8px 10px -6px hsl(0 0% 0% / 0.04)
  // Usage: Command palette, notification panel, lifted elements

--shadow-2xl:
  0 25px 50px -12px hsl(0 0% 0% / 0.18)
  // Usage: Drag-and-drop lifted items, full-screen overlays

--shadow-inner:
  inset 0 2px 4px 0 hsl(0 0% 0% / 0.04)
  // Usage: Pressed buttons, input fields, recessed surfaces

--shadow-glow:
  0 0 15px 2px hsl(var(--brand-primary) / 0.15)
  // Usage: Focus rings, active selection, XP glow effects
```

Dark mode shadows increase opacity by 1.5x and use darker base hue.

### 5.6 Motion Tokens

#### 5.6.1 Durations

```
--duration-instant:     0ms        // Immediate (color changes, opacity snaps)
--duration-fastest:     50ms       // Near-instant (checkbox toggle, radio select)
--duration-fast:        100ms      // Fast feedback (button press, hover color)
--duration-normal:      200ms      // Default transitions (fade, slide, scale)
--duration-moderate:    300ms      // Deliberate transitions (modal open, drawer slide)
--duration-slow:        400ms      // Complex transitions (page transition, layout shift)
--duration-slower:      500ms      // Dramatic transitions (achievement unlock intro)
--duration-slowest:     700ms      // Orchestrated sequences (multi-element stagger)
--duration-scenic:      1000ms     // Cinematic (level-up celebration, confetti burst)
```

#### 5.6.2 Easing Curves

```
--ease-linear:          linear
  // Usage: Progress bar fill, continuous animations

--ease-in:              cubic-bezier(0.4, 0, 1, 1)
  // Usage: Elements exiting the viewport

--ease-out:             cubic-bezier(0, 0, 0.2, 1)
  // Usage: Elements entering the viewport, appearing overlays

--ease-in-out:          cubic-bezier(0.4, 0, 0.2, 1)
  // Usage: Default for most transitions

--ease-bounce:          cubic-bezier(0.34, 1.56, 0.64, 1)
  // Usage: Playful feedback (XP orbs, badge bounce, streak flame)

--ease-spring:          cubic-bezier(0.22, 1.0, 0.36, 1.0)
  // Usage: Draggable elements snapping to position, card expand

--ease-smooth:          cubic-bezier(0.25, 0.1, 0.25, 1.0)
  // Usage: Smooth scrolling, sidebar collapse

--ease-sharp:           cubic-bezier(0.4, 0, 0.6, 1)
  // Usage: Quick, decisive movements (tab switch, dropdown open)
```

#### 5.6.3 Named Transitions

These are pre-defined transition combinations used across the application:

```
fade-in:                opacity 0->1, duration-normal, ease-out
fade-out:               opacity 1->0, duration-fast, ease-in

slide-up:               translateY(8px)->0 + fade-in, duration-normal, ease-out
slide-down:             translateY(-8px)->0 + fade-in, duration-normal, ease-out
slide-left:             translateX(16px)->0 + fade-in, duration-moderate, ease-out
slide-right:            translateX(-16px)->0 + fade-in, duration-moderate, ease-out

scale-in:               scale(0.95)->1 + fade-in, duration-normal, ease-out
scale-out:              scale(1)->0.95 + fade-out, duration-fast, ease-in

modal-enter:            scale(0.95)->1 + fade-in, duration-moderate, ease-spring
modal-leave:            scale(1)->0.98 + fade-out, duration-fast, ease-in

drawer-enter:           translateX(100%)->0, duration-moderate, ease-smooth
drawer-leave:           translateX(0)->100%, duration-normal, ease-in

dropdown-enter:         translateY(-4px)->0 + scale(0.98)->1 + fade-in, duration-normal, ease-out
dropdown-leave:         fade-out, duration-fast, ease-in

toast-enter:            translateY(16px)->0 + fade-in, duration-moderate, ease-bounce
toast-leave:            translateX(100%) + fade-out, duration-normal, ease-in

page-enter:             translateY(4px)->0 + fade-in, duration-slow, ease-out
page-leave:             fade-out, duration-fast, ease-in

card-hover:             translateY(0)->(-2px) + shadow-sm->shadow-md, duration-normal, ease-out
card-press:             scale(1)->0.98, duration-fastest, ease-in

list-item-enter:        translateY(8px)->0 + fade-in, duration-normal, ease-out
                        Staggered: each item delays by 30ms

xp-collect:             scale(1)->1.2->1 + glow pulse, duration-scenic, ease-bounce
level-up:               scale(0.8)->1.1->1 + confetti burst, duration-scenic, ease-spring
achievement-unlock:     scale(0)->1 + rotate(0)->360 + glow, duration-scenic, ease-bounce
streak-flame:           scaleY(1)->1.1->1 loop, duration-slower, ease-in-out (continuous)

mastery-transition:     background-color + border-color + glow, duration-slow, ease-smooth
                        Plays when a mastery level changes
```

#### 5.6.4 Reduced Motion

When the user's OS or eCoach preferences indicate `prefers-reduced-motion`, all transitions collapse to:

- Duration: `--duration-instant` (0ms) or `--duration-fastest` (50ms) for essential state changes.
- Easing: `linear`.
- All transform-based animations (scale, translate, rotate) are removed.
- Opacity transitions are preserved but shortened.
- Continuous animations (streak flame, particle effects) are stopped entirely.
- PixiJS particle effects are replaced with static SVG representations.

### 5.7 Z-Index Layers

Strict z-index layering prevents stacking context conflicts. Every z-index in the application uses one of these tokens.

```
--z-deep:              -1         // Behind-content elements (background patterns, watermarks)
--z-base:              0          // Default layer (page content, cards, form elements)
--z-raised:            10         // Slightly raised elements (sticky table headers, floating labels)
--z-dropdown:          100        // Dropdowns, selects, popovers, context menus
--z-sticky:            200        // Sticky navigation, sticky sidebar, sticky table header
--z-banner:            300        // Banners, announcement bars, cookie consent
--z-overlay:           400        // Overlay/backdrop behind modals and drawers
--z-modal:             500        // Modal dialogs
--z-drawer:            500        // Drawers (same layer as modals; only one open at a time)
--z-toast:             600        // Toast notifications (above modals)
--z-tooltip:           700        // Tooltips (above everything except command palette)
--z-command:           800        // Command palette (Cmd+K -- topmost interactive layer)
--z-notification:      900        // System-level notifications (update available, sync conflict)
--z-dev:               9999       // Development-only overlays (grid guides, debug panels)
```

### 5.8 Breakpoints

While eCoach is a desktop application, the webview can be resized. Breakpoints ensure the UI adapts to window size, particularly useful for split-screen usage, smaller displays, and future tablet support.

```
--breakpoint-sm:       640px      // Compact sidebar, stacked layouts
--breakpoint-md:       768px      // Default minimum window size
--breakpoint-lg:       1024px     // Standard desktop layout
--breakpoint-xl:       1280px     // Wide desktop, expanded sidebar
--breakpoint-2xl:      1536px     // Ultra-wide, multi-column dashboards
```

### 5.9 UI Primitive Component Specifications

Each primitive component below is described with its variants, props, and design specs.

---

#### EButton

Full-featured button supporting multiple visual variants and interactive states.

- **Variants:** `solid` (filled background), `outline` (border only), `ghost` (no border, transparent bg), `soft` (light tinted bg), `link` (text only with underline).
- **Colors:** `primary`, `secondary`, `success`, `warning`, `danger`, `neutral`. Per-portal accent colors available.
- **Sizes:** `xs` (28px height, text-xs), `sm` (32px height, text-sm), `md` (36px height, text-base), `lg` (40px height, text-md), `xl` (48px height, text-lg).
- **States:** `default`, `hover` (subtle bg shift), `active/pressed` (darkened bg + scale(0.98)), `focus-visible` (2px ring with --shadow-glow), `disabled` (40% opacity, cursor-not-allowed), `loading` (spinner replaces text or appears beside it).
- **Props:** `variant`, `color`, `size`, `disabled`, `loading`, `icon` (leading icon), `iconRight` (trailing icon), `fullWidth`, `as` (render as `<a>`, `<router-link>`, or `<button>`).
- **Spacing:** Horizontal padding scales with size: xs=8px, sm=12px, md=16px, lg=20px, xl=24px. Icon gap: 6px.
- **Radius:** `--radius-md` default. `--radius-full` for pill variant.

#### EIconButton

Icon-only button with mandatory tooltip for accessibility.

- **Sizes:** `xs` (24px), `sm` (28px), `md` (32px), `lg` (36px), `xl` (44px). Always square.
- **Variants:** Same as EButton.
- **Props:** `icon` (Lucide icon name), `label` (tooltip text, also used as `aria-label`), `variant`, `color`, `size`, `disabled`, `loading`.
- **Radius:** `--radius-md` default, `--radius-full` for circular.

#### EInput

Text input with label, optional prefix/suffix slots, and validation.

- **Types:** `text`, `email`, `password`, `number`, `search`, `url`, `tel`.
- **Sizes:** `sm` (32px height), `md` (36px height), `lg` (40px height).
- **States:** `default` (border-light), `hover` (border-default), `focus` (border-primary + shadow-glow ring), `error` (border-error + error message below), `disabled` (bg-surface-2, text-disabled).
- **Props:** `modelValue`, `type`, `size`, `label`, `placeholder`, `hint`, `error`, `disabled`, `readonly`, `prefix` (slot), `suffix` (slot), `clearable`.
- **Padding:** 12px horizontal, prefix/suffix slots add 36px on their side.
- **Radius:** `--radius-md`.

#### ETextarea

Multi-line text input with auto-resize option.

- **Props:** `modelValue`, `label`, `placeholder`, `hint`, `error`, `disabled`, `readonly`, `rows` (initial rows), `maxRows`, `autoResize` (grows with content up to maxRows).
- **States:** Same as EInput.
- **Radius:** `--radius-md`.

#### ESelect

Dropdown select with search filtering and multi-select support.

- **Props:** `modelValue`, `options` (array of `{ value, label, icon?, disabled?, group? }`), `label`, `placeholder`, `searchable`, `multiple`, `clearable`, `disabled`, `error`, `size`.
- **Dropdown:** Positioned below input using Floating UI. Max height 320px with scroll. Items 36px height. Highlighted item uses `--student-accent` bg at 10% opacity.
- **Multi-select:** Selected items render as EChip components inside the input area.
- **Radius:** Input `--radius-md`, dropdown `--radius-xl`.

#### ECheckbox

Checkbox with label and indeterminate state support.

- **Sizes:** `sm` (16px box), `md` (18px box), `lg` (20px box).
- **States:** `unchecked`, `checked` (primary color fill + white checkmark), `indeterminate` (primary color fill + dash), `disabled`, `error`.
- **Props:** `modelValue` (boolean), `label`, `disabled`, `indeterminate`, `size`.
- **Animation:** Check/uncheck: scale(0.8)->1 + fade, duration-fast.

#### ERadio

Single radio button. Always used within ERadioGroup.

- **Sizes:** `sm` (16px), `md` (18px), `lg` (20px).
- **States:** `unselected` (empty circle), `selected` (filled inner dot), `disabled`, `error`.
- **Animation:** Inner dot scales from 0 to 1, duration-fast, ease-bounce.

#### ERadioGroup

Container for ERadio components with layout control.

- **Props:** `modelValue`, `options`, `label`, `direction` (`horizontal` | `vertical`), `disabled`, `error`.
- **Spacing:** 12px gap between items (horizontal), 8px gap (vertical).

#### ESwitch

Toggle switch with optional label.

- **Sizes:** `sm` (16px height, 28px width), `md` (20px height, 36px width), `lg` (24px height, 44px width).
- **States:** `off` (neutral bg), `on` (primary color bg), `disabled`.
- **Animation:** Thumb slides with ease-spring, bg color transitions over duration-normal.
- **Props:** `modelValue`, `label`, `disabled`, `size`, `onLabel`, `offLabel`.

#### ESlider

Range slider with single or dual-thumb support.

- **Props:** `modelValue` (number or [number, number] for range), `min`, `max`, `step`, `disabled`, `showValue`, `formatValue`.
- **Track:** 4px height, radius-full. Filled portion uses primary color.
- **Thumb:** 16px circle, white fill, shadow-sm, primary border on hover.
- **Dual-thumb:** Range between thumbs is filled. Thumbs cannot cross.

#### EBadge

Small status indicator or tag.

- **Variants:** `solid`, `outline`, `soft`.
- **Colors:** `primary`, `secondary`, `success`, `warning`, `danger`, `neutral`, plus all mastery and error colors.
- **Sizes:** `sm` (20px height, text-xs), `md` (24px height, text-sm), `lg` (28px height, text-base).
- **Props:** `variant`, `color`, `size`, `dot` (leading color dot), `removable` (shows X button), `icon`.
- **Radius:** `--radius-full` (pill shape).

#### EChip

Interactive chip for filtering and selection.

- **States:** `unselected` (outline style), `selected` (solid fill), `disabled`.
- **Props:** `modelValue` (boolean), `label`, `icon`, `removable`, `disabled`, `color`.
- **Animation:** Selection: bg fill transitions over duration-fast.

#### EAvatar

User avatar with fallback and status indicator.

- **Sizes:** `xs` (24px), `sm` (32px), `md` (40px), `lg` (48px), `xl` (64px), `2xl` (96px).
- **Fallback:** When no image provided, shows initials on colored background (color derived from name hash).
- **Status dot:** Optional 8-12px dot positioned at bottom-right. Colors: `online` (green), `offline` (gray), `away` (amber), `busy` (red).
- **Props:** `src`, `alt`, `name` (for initials fallback), `size`, `status`, `square` (switches from circle to rounded square).
- **Radius:** `--radius-full` (circle) by default. `--radius-lg` when `square`.

#### ECard

Container component with optional header, body, and footer sections.

- **Variants:** `elevated` (shadow-sm, bg surface-0), `outlined` (border, no shadow), `flat` (no border, no shadow, bg surface-1), `interactive` (elevated + hover lift).
- **Props:** `variant`, `padding` (`none`, `sm`, `md`, `lg`), `as` (semantic HTML element).
- **Slots:** `header` (top section with bottom border), `default` (body), `footer` (bottom section with top border).
- **Padding:** sm=12px, md=16px, lg=24px.
- **Radius:** `--radius-2xl`.
- **Hover (interactive):** translateY(-2px) + shadow-md, duration-normal, ease-out.

#### EModal

Dialog overlay with backdrop, focus trap, and scroll lock.

- **Sizes:** `sm` (400px max-width), `md` (560px), `lg` (720px), `xl` (960px), `full` (calc(100vw - 64px)).
- **Props:** `modelValue` (open/closed), `title`, `description`, `size`, `closable` (show X button), `persistent` (clicking backdrop does not close), `scrollable` (body scrolls independently).
- **Backdrop:** Black at 40% opacity (light mode), 60% (dark mode).
- **Animation:** Enter: modal-enter (scale 0.95->1 + fade), backdrop fade. Leave: modal-leave + backdrop fade.
- **Slots:** `header`, `default` (body), `footer` (action buttons area).
- **Focus trap:** On open, focus moves to first focusable element. Tab cycles within modal. Esc closes (unless persistent).
- **Radius:** `--radius-xl`.

#### EDrawer

Side panel sliding in from an edge.

- **Positions:** `left`, `right`, `bottom`.
- **Sizes (width for left/right):** `sm` (320px), `md` (420px), `lg` (560px), `xl` (720px). Bottom drawer: 50vh max-height.
- **Props:** `modelValue`, `title`, `position`, `size`, `overlay` (show backdrop), `closable`.
- **Animation:** drawer-enter/drawer-leave per position.
- **Focus trap:** Same as EModal.

#### ESheet

Mobile-style bottom sheet (pull-up panel) useful for compact views.

- **Props:** `modelValue`, `title`, `snapPoints` (array of heights, e.g. [0.25, 0.5, 0.9]), `initialSnap`.
- **Drag handle:** 32px wide, 4px tall rounded bar at top.
- **Gesture:** Draggable up/down, snaps to nearest snap point. Pull below minimum dismisses.

#### EDropdown

Dropdown menu triggered by a button or custom trigger.

- **Props:** `items` (array of `{ label, icon?, shortcut?, disabled?, danger?, action?, children? }`), `placement`, `offset`, `width` (`trigger` matches trigger width, or fixed px).
- **Items:** 36px height, 8px horizontal padding. Hover: bg surface-2. Danger items: red text.
- **Dividers:** 1px border-light with 4px vertical margin.
- **Nested menus:** Sub-items render as a nested dropdown to the right.
- **Animation:** dropdown-enter/dropdown-leave.
- **Radius:** `--radius-xl`.

#### EPopover

Floating panel anchored to a trigger element.

- **Props:** `trigger` (`click`, `hover`, `focus`, `manual`), `placement` (12 positions via Floating UI), `offset`, `arrow`, `interactive` (can interact with popover content without closing).
- **Arrow:** 8px rotated square matching popover background.
- **Animation:** Same as dropdown.
- **Radius:** `--radius-xl`.

#### ETooltip

Simple text tooltip with delay.

- **Props:** `content`, `placement`, `delay` (default 500ms show delay, 0ms hide delay), `offset`.
- **Max width:** 280px. Text wraps.
- **Style:** bg surface-900 (nearly black), text white, text-sm, 6px 10px padding.
- **Radius:** `--radius-sm`.
- **Animation:** fade-in, duration-fast.

#### EToast

Notification toast appearing at a screen edge.

- **Types:** `success` (green icon), `error` (red icon), `warning` (amber icon), `info` (blue icon).
- **Position:** Top-right corner, stacked vertically with 8px gap.
- **Props:** `type`, `title`, `description`, `duration` (auto-dismiss, default 5000ms, 0 for persistent), `action` (optional action button), `closable`.
- **Animation:** toast-enter (slide-up + fade from right), toast-leave (slide-right + fade).
- **Width:** 360px fixed.
- **Radius:** `--radius-lg`.

#### EAlert

Inline alert/banner for contextual messages.

- **Types:** `success`, `error`, `warning`, `info`.
- **Variants:** `solid` (filled bg), `soft` (tinted bg), `outline` (border + icon color).
- **Props:** `type`, `variant`, `title`, `closable`, `icon` (custom icon override).
- **Slots:** `default` (description body), `action` (trailing action button).
- **Radius:** `--radius-lg`.

#### EProgress

Progress indicator in bar or circular form.

- **Variants:** `bar` (horizontal), `circle` (circular/ring).
- **Bar props:** `value` (0-100), `max`, `size` (`sm` 4px, `md` 8px, `lg` 12px), `color`, `striped`, `animated` (striped animation), `indeterminate` (looping animation), `label` (text inside or above bar).
- **Circle props:** `value`, `size` (diameter in px), `strokeWidth`, `color`, `showValue` (percentage text in center).
- **Radius (bar):** `--radius-full`.
- **Animation:** Value changes animate with ease-smooth over duration-moderate.

#### ESkeleton

Placeholder loading state mirroring content shape.

- **Variants:** `text` (single line, 100% width), `circle` (avatar placeholder), `rect` (card/image placeholder), `custom` (any shape via width/height).
- **Props:** `variant`, `width`, `height`, `lines` (for text: render multiple lines with last line at 60% width).
- **Animation:** Shimmer gradient sweeps left to right continuously. Gradient: surface-2 -> surface-3 -> surface-2.
- **Radius:** Matches the element it replaces.

#### ESpinner

Loading spinner for inline or page-level use.

- **Sizes:** `xs` (16px), `sm` (20px), `md` (24px), `lg` (32px), `xl` (48px).
- **Props:** `size`, `color` (inherits current text color by default), `label` (screen reader text).
- **Animation:** Continuous rotation, 750ms per revolution, ease-linear. Partial arc (270 degrees) creates the spinning effect.

#### EDivider

Horizontal or vertical dividing line.

- **Props:** `orientation` (`horizontal`, `vertical`), `label` (text centered on divider), `dashed`.
- **Style:** 1px border-light. Label: text-sm text-tertiary with 8px horizontal padding, bg inherited from parent.

#### EAccordion

Expandable content sections.

- **Props:** `items` (array of `{ title, content }`), `multiple` (allow multiple open), `defaultOpen` (indices of initially open items), `bordered`.
- **Item height (header):** 48px.
- **Animation:** Content height animates via max-height transition, duration-moderate, ease-smooth. Chevron icon rotates 180 degrees.
- **Radius:** `--radius-lg` (outer container).

#### ETabs

Tab navigation component.

- **Variants:** `underline` (bottom border indicator), `pills` (filled bg on active), `outline` (bordered active tab).
- **Props:** `modelValue` (active tab), `items` (array of `{ value, label, icon?, badge?, disabled? }`), `variant`, `orientation` (`horizontal`, `vertical`).
- **Indicator:** Underline variant: 2px bottom border, primary color, slides to active tab position with ease-smooth over duration-normal.
- **Spacing:** 4px gap between tabs. Tab padding: 8px 16px.

#### ETabPanel

Tab content container paired with ETabs.

- **Props:** `value` (matches tab value), `lazy` (only mount when first activated), `keepAlive` (preserve state when deactivated).
- **Animation:** Content fades in on activation, duration-fast.

#### EBreadcrumb

Navigation breadcrumb trail.

- **Props:** `items` (array of `{ label, to?, icon? }`), `separator` (default `/`, also `>`, `chevron`), `maxItems` (collapsed to `...` menu if exceeded).
- **Style:** text-sm, text-tertiary for inactive items, text-primary for current (last) item.
- **Overflow:** When items exceed maxItems, middle items collapse into a dropdown menu.

#### EPagination

Page navigation for data tables and lists.

- **Props:** `totalItems`, `pageSize`, `modelValue` (current page), `pageSizeOptions` ([10, 25, 50, 100]), `showPageSize`, `showTotal`.
- **Style:** Button group with prev/next arrows and page numbers. Active page: primary solid bg.
- **Compact mode:** At small widths, collapses to "Page X of Y" with prev/next only.

#### ETable

Data table with sorting, selection, and actions.

- **Props:** `data` (array), `columns` (ETableColumn definitions), `selectable` (checkbox column), `sortable`, `stickyHeader`, `striped`, `hoverable`, `loading`, `emptyText`.
- **Row height:** Default 48px. Compact mode 36px. Comfortable mode 56px.
- **Header:** 40px height, text-sm font-semibold text-secondary, bg surface-1. Sort indicator: up/down arrow icons.
- **Selection:** Checkbox column 48px wide. Select-all in header. Selected rows: bg primary at 5% opacity.
- **Borders:** Horizontal row borders (border-light). Optional vertical column borders.
- **Radius:** `--radius-lg` (outer container).

#### ETableColumn

Column definition for ETable.

- **Props:** `key` (data property), `label`, `sortable`, `width`, `align` (`left`, `center`, `right`), `fixed` (`left`, `right` for frozen columns), `render` (custom render function).

#### ETag

Colored categorization tag (similar to badge but semantically different).

- **Props:** `color` (any design system color), `size` (`sm`, `md`), `closable`, `icon`.
- **Radius:** `--radius-sm`.

#### EKbd

Keyboard shortcut display.

- **Props:** `keys` (array of key names, e.g. `['Ctrl', 'S']`).
- **Style:** Each key in its own bordered box. bg surface-2, border-default, text-sm font-mono, radius-sm, 2px 6px padding. Keys separated by `+` text.

#### EEmptyState

Placeholder for empty lists and search results.

- **Props:** `icon` (large illustrative icon), `title`, `description`, `action` (CTA button props).
- **Layout:** Centered vertically, max-width 360px. Icon 64px, title text-lg font-semibold, description text-base text-secondary.
- **Spacing:** 16px between icon/title/description, 24px before action button.

#### EErrorState

Error display with retry capability.

- **Props:** `icon`, `title`, `description`, `retryable` (show retry button), `retryLabel`.
- **Layout:** Same as EEmptyState with red-tinted icon.

#### EScrollArea

Custom scrollbar container replacing native browser scrollbars.

- **Props:** `orientation` (`vertical`, `horizontal`, `both`), `scrollbarSize` (`sm` 6px, `md` 8px), `alwaysVisible`.
- **Scrollbar:** Rounded track (surface-2 bg), rounded thumb (surface-4 bg, darker on hover). Appears on scroll, fades after 1.5s inactivity.

#### ECollapsible

Simple collapsible content section (non-accordion).

- **Props:** `modelValue` (open/closed), `disabled`.
- **Slots:** `trigger` (custom toggle element), `default` (collapsible content).
- **Animation:** Height animates via max-height, duration-moderate, ease-smooth.

#### ECommandPalette

Application-wide command palette (Cmd+K / Ctrl+K).

- **Props:** `commands` (registered command list), `recentCommands`, `placeholder`.
- **Layout:** Centered modal, 640px max-width, search input at top, scrollable command list below.
- **Commands:** Grouped by category. Each command: icon + label + shortcut hint. 40px row height.
- **Search:** Fuzzy search with highlighted matching characters.
- **Animation:** Scale(0.95)->1 + fade, duration-normal, ease-out.
- **Z-index:** --z-command (800).
- **Radius:** `--radius-xl`.

#### EContextMenu

Right-click context menu.

- **Props:** `items` (same structure as EDropdown items).
- **Trigger:** `@contextmenu` on parent element.
- **Style:** Same as EDropdown menu.
- **Positioning:** Opens at cursor position, flips to stay in viewport.

#### EDatePicker

Calendar-based date selection.

- **Props:** `modelValue` (Date or [Date, Date] for range), `mode` (`single`, `range`, `multiple`), `minDate`, `maxDate`, `disabledDates`, `locale`.
- **Calendar grid:** 7 columns (days), 6 rows (weeks). Day cell 36px square. Today: primary ring. Selected: primary fill. Range: light primary bg between dates.
- **Navigation:** Month/year selectors in header. Previous/next month arrows.
- **Input:** Text input with calendar icon trigger. Date format based on locale.
- **Radius:** `--radius-xl` (dropdown), `--radius-full` (day cells).

#### ETimePicker

Time selection input.

- **Props:** `modelValue` (string "HH:mm"), `format` (`12h`, `24h`), `minuteStep` (1, 5, 10, 15, 30), `minTime`, `maxTime`.
- **Dropdown:** Two scrollable columns (hours, minutes) + AM/PM toggle for 12h format. Item height 36px.

#### EColorPicker

Color selection for customization features (theme accents, note colors, tag colors).

- **Props:** `modelValue` (hex string), `presets` (array of preset colors), `showInput` (hex input field), `showAlpha`.
- **Layout:** Preset swatches grid + custom picker (hue bar + saturation/brightness square) + hex input.
- **Radius:** `--radius-xl`.

---

*End of document. This specification serves as the canonical reference for all frontend implementation. Every component, token, and architectural decision documented here must be followed exactly during implementation. No deviations without updating this document first.*
