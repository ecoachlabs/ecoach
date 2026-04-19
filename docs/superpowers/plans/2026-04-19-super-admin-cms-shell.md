# Super Admin CMS Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing super-admin area into a coherent CMS shell where Dashboard gives the overview, Question Bank inspects content, Content Editor edits structured records, and Remote Updates has an honest future-ready workspace.

**Architecture:** Keep the current Vue 3/Tauri/Rust command architecture. Frontend admin screens should continue using `frontend/src/ipc/admin.ts` as the IPC boundary; this phase should not add backend behavior unless a compile-time mismatch is discovered. Split new CMS UI into focused reusable components under `frontend/src/components/admin/cms/` and route-level workspaces under `frontend/src/views/admin/`.

**Tech Stack:** Vue 3 `<script setup lang="ts">`, Vue Router, Pinia auth store, existing local UI components, Tauri IPC wrappers, Rust command crate checks, `pnpm build`, `cargo check --manifest-path src-tauri/Cargo.toml`.

---

## Scope

This plan implements Pass 1 plus the shell of Pass 2 from `docs/superpowers/specs/2026-04-19-super-admin-cms-design.md`.

Included:

- CMS grouped admin sidebar.
- Dashboard as CMS operating overview.
- Question Bank as inventory/inspection.
- Content Editor route and question-editing MVP shell.
- Remote Updates route and honest current-state shell.
- Reusable CMS components.
- Route redirects so old admin question-author paths still work.

Not included:

- Backend remote sync/apply behavior.
- Universal editing for every structured record type.
- Rich diffing/version history.
- Batch organize backend mutations.

## Files

Create:

- `frontend/src/components/admin/cms/CmsMetricStrip.vue`  
  Small reusable metric strip for Dashboard, Question Bank, Sources, and Content Editor.

- `frontend/src/components/admin/cms/types.ts`  
  Shared CMS component types that route components can import without relying on Vue SFC type exports.

- `frontend/src/components/admin/cms/CmsStatusBadge.vue`  
  CMS-specific status badge that maps content statuses to consistent colors.

- `frontend/src/components/admin/cms/CmsActionQueue.vue`  
  Reusable action queue for Dashboard and Remote Updates.

- `frontend/src/components/admin/cms/ContentInspectorPanel.vue`  
  Right-side read-only inspector for selected question/content records.

- `frontend/src/views/admin/content-editor/ContentEditorHome.vue`  
  Main Content Editor workspace. MVP supports question records through existing admin question editor/upsert IPC commands and includes future content type rails honestly marked as not yet wired.

- `frontend/src/views/admin/remote/RemoteUpdates.vue`  
  Remote Updates workspace. Shows current pack/source/update status from existing commands and describes unavailable remote apply behavior without fake data.

Modify:

- `frontend/src/layouts/AdminLayout.vue`  
  Replace current admin nav groups with CMS groups and calmer colors.

- `frontend/src/router/index.ts`  
  Add `content-editor` and `remote-updates` routes; redirect old `/admin/questions/author` to the editor with query compatibility.

- `frontend/src/views/admin/AdminHome.vue`  
  Reframe as CMS Dashboard using metric/action components.

- `frontend/src/views/admin/questions/QuestionsHome.vue`  
  Reframe as Question Bank, read-heavy inspection surface, with edit actions pointing to Content Editor.

- `frontend/src/views/admin/content/ContentPipeline.vue`  
  Keep as Sources & Ingestion; align copy/actions with CMS language.

Verify:

- `pnpm build` from `frontend/`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo test -p ecoach-commands admin_question_ -- --nocapture`
- Manual smoke path in Tauri: unlock admin -> Dashboard -> Question Bank -> open Content Editor -> save question -> return to bank

---

### Task 1: Add CMS Route Skeletons

**Files:**

- Create: `frontend/src/views/admin/content-editor/ContentEditorHome.vue`
- Create: `frontend/src/views/admin/remote/RemoteUpdates.vue`
- Modify: `frontend/src/router/index.ts`

- [ ] **Step 1: Create Content Editor route shell component**

Create `frontend/src/views/admin/content-editor/ContentEditorHome.vue` with this initial component:

```vue
<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="mb-6">
      <p class="text-[10px] font-bold uppercase tracking-[0.16em]" :style="{ color: 'var(--ink-muted)' }">
        Structured CMS Records
      </p>
      <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Content Editor</h1>
      <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
        Edit extracted content records, starting with questions, answers, explanations, and classification metadata.
      </p>
    </div>

    <AppCard padding="lg">
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
        Content Editor workspace is ready for the question-record editor in the next task.
      </p>
    </AppCard>
  </div>
</template>
```

- [ ] **Step 2: Create Remote Updates route shell component**

Create `frontend/src/views/admin/remote/RemoteUpdates.vue` with this initial component:

```vue
<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="mb-6">
      <p class="text-[10px] font-bold uppercase tracking-[0.16em]" :style="{ color: 'var(--ink-muted)' }">
        Distribution
      </p>
      <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Remote Updates</h1>
      <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
        Preview external content updates before they become reviewable structured records.
      </p>
    </div>

    <AppCard padding="lg">
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
        Remote update preview/apply commands are not wired yet. This workspace will show current pack and source status using existing local data first.
      </p>
    </AppCard>
  </div>
</template>
```

- [ ] **Step 3: Wire routes**

Modify the admin children section in `frontend/src/router/index.ts` so it includes the new routes and replaces the existing `/admin/questions/author` route with a compatibility redirect:

```ts
      { path: '', name: 'admin-home', component: AdminHome },
      { path: 'content-editor', name: 'admin-content-editor', component: () => import('@/views/admin/content-editor/ContentEditorHome.vue') },
      { path: 'content-editor/question/:id?', name: 'admin-content-editor-question', component: () => import('@/views/admin/content-editor/ContentEditorHome.vue'), props: true },
      { path: 'remote-updates', name: 'admin-remote-updates', component: () => import('@/views/admin/remote/RemoteUpdates.vue') },
      { path: 'questions/author', redirect: to => ({ path: '/admin/content-editor', query: { ...to.query, type: 'question' } }) },
```

Keep the existing `questions`, `questions/review`, `content`, `content/coverage`, `packs`, `users`, `quality`, and `settings` routes.

- [ ] **Step 4: Run route build check**

Run:

```powershell
pnpm build
```

from `frontend/`.

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add frontend/src/views/admin/content-editor/ContentEditorHome.vue frontend/src/views/admin/remote/RemoteUpdates.vue frontend/src/router/index.ts
git commit -m "feat: add admin cms routes"
```

---

### Task 2: Add Reusable CMS Components

**Files:**

- Create: `frontend/src/components/admin/cms/CmsMetricStrip.vue`
- Create: `frontend/src/components/admin/cms/types.ts`
- Create: `frontend/src/components/admin/cms/CmsStatusBadge.vue`
- Create: `frontend/src/components/admin/cms/CmsActionQueue.vue`
- Create: `frontend/src/components/admin/cms/ContentInspectorPanel.vue`

- [ ] **Step 1: Create shared CMS component types**

Create `frontend/src/components/admin/cms/types.ts`:

```ts
export interface CmsMetricItem {
  label: string
  value: string | number
  caption?: string
  tone?: 'neutral' | 'good' | 'review' | 'danger'
}

export interface CmsActionItem {
  key: string
  title: string
  summary: string
  tone?: 'neutral' | 'review' | 'danger'
  actionLabel?: string
  route?: string
}
```

- [ ] **Step 2: Create `CmsMetricStrip.vue`**

```vue
<script setup lang="ts">
import type { CmsMetricItem } from './types'

defineProps<{
  items: CmsMetricItem[]
}>()

function colorFor(tone?: CmsMetricItem['tone']) {
  if (tone === 'good') return 'var(--success)'
  if (tone === 'review') return 'var(--gold)'
  if (tone === 'danger') return 'var(--warm)'
  return 'var(--ink)'
}
</script>

<template>
  <div class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-6 gap-3">
    <div
      v-for="item in items"
      :key="item.label"
      class="rounded-lg border px-4 py-3"
      :style="{ backgroundColor: 'var(--surface)', borderColor: 'var(--border-soft)' }"
    >
      <p class="font-display text-2xl font-bold tabular-nums" :style="{ color: colorFor(item.tone) }">
        {{ item.value }}
      </p>
      <p class="text-[10px] uppercase tracking-wide mt-1" :style="{ color: 'var(--ink-muted)' }">{{ item.label }}</p>
      <p v-if="item.caption" class="text-[10px] mt-1 truncate" :style="{ color: 'var(--ink-muted)' }">{{ item.caption }}</p>
    </div>
  </div>
</template>
```

- [ ] **Step 3: Create `CmsStatusBadge.vue`**

```vue
<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'

const props = defineProps<{
  status: string | null | undefined
  size?: 'xs' | 'sm' | 'md'
}>()

function normalizedStatus() {
  return (props.status || 'unknown').replaceAll('_', ' ')
}

function badgeColor(): 'accent' | 'warm' | 'gold' | 'success' | 'danger' | 'muted' | 'ember' {
  const status = (props.status || '').toLowerCase()
  if (['approved', 'published', 'active', 'installed', 'complete', 'completed'].includes(status)) return 'success'
  if (['failed', 'blocked', 'rejected', 'error'].includes(status)) return 'danger'
  if (['pending', 'needs_review', 'queued', 'draft', 'preview'].includes(status)) return 'gold'
  if (['running', 'extracting', 'processing'].includes(status)) return 'accent'
  return 'muted'
}
</script>

<template>
  <AppBadge :color="badgeColor()" :size="size ?? 'xs'">
    {{ normalizedStatus() }}
  </AppBadge>
</template>
```

- [ ] **Step 4: Create `CmsActionQueue.vue`**

```vue
<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import type { CmsActionItem } from './types'

defineProps<{
  items: CmsActionItem[]
  emptyText?: string
}>()

defineEmits<{
  open: [item: CmsActionItem]
}>()
</script>

<template>
  <div class="space-y-3">
    <div
      v-for="item in items"
      :key="item.key"
      class="rounded-lg border p-3"
      :style="{ backgroundColor: 'var(--paper)', borderColor: 'var(--border-soft)' }"
    >
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="flex items-center gap-2 mb-1">
            <AppBadge :color="item.tone === 'danger' ? 'danger' : item.tone === 'review' ? 'gold' : 'muted'" size="xs">
              {{ item.tone === 'danger' ? 'Issue' : item.tone === 'review' ? 'Review' : 'Info' }}
            </AppBadge>
            <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
          </div>
          <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ item.summary }}</p>
        </div>
        <AppButton v-if="item.actionLabel" variant="ghost" size="sm" @click="$emit('open', item)">
          {{ item.actionLabel }}
        </AppButton>
      </div>
    </div>

    <p v-if="!items.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">
      {{ emptyText ?? 'No admin actions need attention.' }}
    </p>
  </div>
</template>
```

- [ ] **Step 5: Create `ContentInspectorPanel.vue`**

```vue
<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CmsStatusBadge from './CmsStatusBadge.vue'
import type { AdminQuestionListItemDto } from '@/ipc/admin'

const props = defineProps<{
  question: AdminQuestionListItemDto | null
  accuracy: number | null
}>()

defineEmits<{
  edit: [question: AdminQuestionListItemDto]
  seed: [question: AdminQuestionListItemDto]
}>()

function displayAccuracy() {
  if (props.accuracy === null) return 'No attempts'
  return `${props.accuracy}% accuracy`
}
</script>

<template>
  <AppCard padding="md" class="sticky top-4">
    <div v-if="!question">
      <h2 class="text-sm font-bold mb-2" :style="{ color: 'var(--ink)' }">Question Inspector</h2>
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Select a question to inspect its content, provenance, and learning signals.</p>
    </div>

    <div v-else>
      <div class="flex items-center gap-2 mb-3">
        <CmsStatusBadge :status="question.review_status" />
        <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">Question #{{ question.question_id }}</span>
      </div>

      <h2 class="text-base font-bold mb-3 leading-snug" :style="{ color: 'var(--ink)' }">{{ question.stem }}</h2>

      <div class="grid grid-cols-2 gap-3 mb-4 text-xs">
        <div>
          <p class="font-semibold" :style="{ color: 'var(--ink)' }">{{ question.subject_name }}</p>
          <p :style="{ color: 'var(--ink-muted)' }">Subject</p>
        </div>
        <div>
          <p class="font-semibold" :style="{ color: 'var(--ink)' }">{{ question.topic_name }}</p>
          <p :style="{ color: 'var(--ink-muted)' }">Topic</p>
        </div>
        <div>
          <p class="font-semibold" :style="{ color: 'var(--ink)' }">{{ question.question_format }}</p>
          <p :style="{ color: 'var(--ink-muted)' }">Format</p>
        </div>
        <div>
          <p class="font-semibold" :style="{ color: 'var(--ink)' }">{{ question.source_type }}</p>
          <p :style="{ color: 'var(--ink-muted)' }">Source</p>
        </div>
      </div>

      <div class="rounded-lg p-3 mb-4" :style="{ backgroundColor: 'var(--paper)' }">
        <p class="text-xs font-semibold mb-1" :style="{ color: 'var(--ink)' }">Usage</p>
        <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
          {{ question.attempt_count }} attempts · {{ displayAccuracy() }} · {{ question.option_count }} answers
        </p>
      </div>

      <div class="flex gap-2">
        <AppButton variant="primary" size="sm" @click="$emit('edit', question)">Edit in Content Editor</AppButton>
        <AppButton variant="secondary" size="sm" @click="$emit('seed', question)">Seed Similar</AppButton>
      </div>
    </div>
  </AppCard>
</template>
```

- [ ] **Step 6: Run frontend build**

Run:

```powershell
pnpm build
```

from `frontend/`.

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add frontend/src/components/admin/cms
git commit -m "feat: add admin cms components"
```

---

### Task 3: Rework Admin Navigation Into CMS Groups

**Files:**

- Modify: `frontend/src/layouts/AdminLayout.vue`

- [ ] **Step 1: Update icon imports**

In `AdminLayout.vue`, include the icons needed by CMS navigation. Keep existing icons that are still used.

Use this import block:

```ts
import {
  PhArrowsClockwise,
  PhBook,
  PhBooks,
  PhChartLineUp,
  PhCheckCircle,
  PhDatabase,
  PhGear,
  PhHouseLine,
  PhMagnifyingGlass,
  PhMoon,
  PhNewspaper,
  PhPencilSimple,
  PhSignOut,
  PhSun,
  PhTreeStructure,
  PhUploadSimple,
  PhUsers,
} from '@phosphor-icons/vue'
```

- [ ] **Step 2: Replace `navGroups`**

Replace the current `navGroups` constant with this CMS grouping:

```ts
const navGroups: Array<{ label: string; items: NavItem[] }> = [
  {
    label: 'Overview',
    items: [
      { to: '/admin', label: 'Dashboard', icon: PhHouseLine, color: '#0F766E', match: 'exact' },
    ],
  },
  {
    label: 'Manage Content',
    items: [
      { to: '/admin/content-editor', label: 'Content Editor', icon: PhPencilSimple, color: '#0F766E', match: 'prefix' },
      { to: '/admin/questions', label: 'Question Bank', icon: PhDatabase, color: '#B45309', match: 'exact' },
      { to: '/admin/content', label: 'Sources & Ingestion', icon: PhUploadSimple, color: '#2563EB', match: 'exact' },
      { to: '/admin/questions', hash: '#seeding', activeHash: '#seeding', label: 'Seeding Engine', icon: PhTreeStructure, color: '#7C2D12', match: 'exact' },
    ],
  },
  {
    label: 'Quality',
    items: [
      { to: '/admin/questions/review', label: 'Review Queue', icon: PhCheckCircle, color: '#CA8A04', match: 'exact' },
      { to: '/admin/content/coverage', label: 'Coverage & Stats', icon: PhChartLineUp, color: '#166534', match: 'exact' },
      { to: '/admin/quality', label: 'Quality Dashboard', icon: PhMagnifyingGlass, color: '#991B1B', match: 'exact' },
    ],
  },
  {
    label: 'Distribution',
    items: [
      { to: '/admin/remote-updates', label: 'Remote Updates', icon: PhArrowsClockwise, color: '#0369A1', match: 'exact' },
      { to: '/admin/packs', label: 'Packs & Publishing', icon: PhNewspaper, color: '#4D7C0F', match: 'exact' },
    ],
  },
  {
    label: 'System',
    items: [
      { to: '/admin/students', label: 'Students', icon: PhUsers, color: '#0E7490', match: 'prefix' },
      { to: '/admin/users', label: 'Users', icon: PhUsers, color: '#0F766E', match: 'exact' },
      {
        to: '/admin/settings',
        label: 'Settings',
        icon: PhGear,
        color: '#525252',
        match: 'exact',
        excludeHashes: ['#system', '#health', '#backup', '#tuning', '#entitlements'],
      },
      { to: '/admin/settings', hash: '#health', activeHash: '#health', label: 'System Health', icon: PhChartLineUp, color: '#0369A1', match: 'exact' },
      { to: '/admin/settings', hash: '#backup', activeHash: '#backup', label: 'Backup & Restore', icon: PhBooks, color: '#525252', match: 'exact' },
    ],
  },
]
```

- [ ] **Step 3: Update admin brand copy and icon style**

Change the sidebar header subtitle from `Admin Portal` to `CMS Console`.

Change the logo background from violet/purple to teal/stone:

```vue
<div class="w-8 h-8 rounded-[8px] bg-[var(--accent)] flex items-center justify-center shadow-sm">
  <span class="text-white font-display font-bold text-sm">e</span>
</div>
```

- [ ] **Step 4: Run frontend build**

```powershell
pnpm build
```

from `frontend/`.

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add frontend/src/layouts/AdminLayout.vue
git commit -m "feat: organize admin nav as cms"
```

---

### Task 4: Reframe Dashboard As CMS Overview

**Files:**

- Modify: `frontend/src/views/admin/AdminHome.vue`

- [ ] **Step 1: Import CMS components**

Add imports:

```ts
import CmsActionQueue from '@/components/admin/cms/CmsActionQueue.vue'
import CmsMetricStrip from '@/components/admin/cms/CmsMetricStrip.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import type { CmsActionItem, CmsMetricItem } from '@/components/admin/cms/types'
```

- [ ] **Step 2: Replace `statCards` with CMS metric data**

Use this computed block:

```ts
const cmsMetrics = computed<CmsMetricItem[]>(() => [
  { label: 'Questions', value: questionStats.value?.total_questions ?? 0, caption: 'Bank inventory' },
  { label: 'Answers', value: questionStats.value?.total_options ?? 0, caption: 'Options stored' },
  { label: 'Sources', value: tower.value?.content_health.source_count ?? 0, caption: 'Raw provenance' },
  { label: 'Pending Review', value: questionStats.value?.pending_review_count ?? 0, tone: 'review' },
  { label: 'Published Packs', value: questionStats.value?.installed_pack_count ?? 0, tone: 'good' },
  { label: 'Attempts', value: questionStats.value?.total_attempts ?? 0, caption: 'Learning signal' },
])

const actionItems = computed<CmsActionItem[]>(() => {
  const items: CmsActionItem[] = []
  if (hasNoContent.value) {
    items.push({
      key: 'no-content',
      title: 'No question content installed',
      summary: 'Install a pack, register sources, or open the editor to create structured questions.',
      tone: 'review',
      actionLabel: 'Install Pack',
      route: '/admin/packs',
    })
  }
  for (const item of tower.value?.action_recommendations ?? []) {
    items.push({
      key: item.recommendation_key,
      title: item.label,
      summary: item.summary,
      tone: 'review',
      actionLabel: 'Open',
      route: '/admin/content',
    })
  }
  return items
})

function openAction(item: CmsActionItem) {
  if (item.route) router.push(item.route)
}
```

- [ ] **Step 3: Update dashboard header copy**

Set heading and copy to:

```vue
<p class="text-[10px] font-bold uppercase tracking-[0.16em]" :style="{ color: 'var(--ink-muted)' }">Super Admin CMS</p>
<h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Dashboard</h1>
<p class="text-sm mt-1 max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
  The operating overview for structured content, question inventory, source health, review work, and publishing readiness.
</p>
```

- [ ] **Step 4: Replace the metric button grid**

Replace the existing stat-card button grid with:

```vue
<CmsMetricStrip :items="cmsMetrics" class="mb-6" />
```

- [ ] **Step 5: Replace Action Queue block**

Replace the existing action queue card contents with:

```vue
<AppCard padding="md">
  <div class="flex items-center justify-between mb-3">
    <h2 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--ink-muted)' }">Needs Attention</h2>
    <AppButton variant="ghost" size="sm" @click="router.push('/admin/questions/review')">Review Queue</AppButton>
  </div>
  <CmsActionQueue :items="actionItems" empty-text="No urgent CMS actions." @open="openAction" />
</AppCard>
```

- [ ] **Step 6: Use CMS status badges for source governance**

In the source governance list, replace the status `AppBadge` with:

```vue
<CmsStatusBadge :status="source.source_status" />
```

- [ ] **Step 7: Run frontend build**

```powershell
pnpm build
```

from `frontend/`.

Expected: PASS.

- [ ] **Step 8: Commit**

```powershell
git add frontend/src/views/admin/AdminHome.vue
git commit -m "feat: reframe admin dashboard as cms overview"
```

---

### Task 5: Reframe Questions Screen As Question Bank

**Files:**

- Modify: `frontend/src/views/admin/questions/QuestionsHome.vue`

- [ ] **Step 1: Import CMS components**

Add:

```ts
import CmsMetricStrip from '@/components/admin/cms/CmsMetricStrip.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import ContentInspectorPanel from '@/components/admin/cms/ContentInspectorPanel.vue'
import type { CmsMetricItem } from '@/components/admin/cms/types'
```

- [ ] **Step 2: Rename metrics computed**

Replace the current `statCards` computed with:

```ts
const bankMetrics = computed<CmsMetricItem[]>(() => [
  { label: 'Questions', value: stats.value?.total_questions ?? 0 },
  { label: 'Answers', value: stats.value?.total_options ?? 0 },
  { label: 'Attempts', value: stats.value?.total_attempts ?? 0 },
  { label: 'Families', value: stats.value?.family_count ?? 0 },
  { label: 'Pending Review', value: stats.value?.pending_review_count ?? 0, tone: 'review' },
  { label: 'Approved', value: stats.value?.approved_review_count ?? 0, tone: 'good' },
])
```

- [ ] **Step 3: Add navigation helpers**

Add:

```ts
function openEditor(question?: AdminQuestionListItemDto | null) {
  if (question) {
    router.push({ path: '/admin/content-editor', query: { type: 'question', id: question.question_id } })
  } else {
    router.push({ path: '/admin/content-editor', query: { type: 'question' } })
  }
}

function jumpToSeeding() {
  window.location.hash = 'seeding'
}
```

- [ ] **Step 4: Update header**

Set heading and actions:

```vue
<h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Question Bank</h1>
<p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
  Inspect the permanent question inventory, provenance, review status, usage signals, and what each question tests.
</p>
...
<AppButton variant="secondary" size="sm" @click="loadQuestions">Refresh</AppButton>
<AppButton variant="secondary" size="sm" @click="jumpToSeeding">Seeding Engine</AppButton>
<AppButton variant="primary" size="sm" @click="openEditor(null)">New Structured Question</AppButton>
```

- [ ] **Step 5: Replace metric card grid**

Replace the metric grid with:

```vue
<CmsMetricStrip :items="bankMetrics" class="mb-6" />
```

- [ ] **Step 6: Update selected inspector**

Replace the existing right-side selected-question card header/actions with `ContentInspectorPanel` at the top of the right column:

```vue
<ContentInspectorPanel
  :question="selected"
  :accuracy="selectedAccuracy"
  @edit="openEditor"
  @seed="seedFromSelected"
/>
```

Keep the deeper intelligence details below this panel so the bank remains useful for inspection.

- [ ] **Step 7: Replace review status badges in question rows**

Where question rows display `review_status`, use:

```vue
<CmsStatusBadge :status="question.review_status" />
```

- [ ] **Step 8: Add seeding section anchor**

On the Seeding Engine card root element, add:

```vue
id="seeding"
```

- [ ] **Step 9: Run frontend build**

```powershell
pnpm build
```

from `frontend/`.

Expected: PASS.

- [ ] **Step 10: Commit**

```powershell
git add frontend/src/views/admin/questions/QuestionsHome.vue
git commit -m "feat: present questions as bank inventory"
```

---

### Task 6: Build Content Editor Question MVP

**Files:**

- Modify: `frontend/src/views/admin/content-editor/ContentEditorHome.vue`
- Optionally stop linking to: `frontend/src/views/admin/questions/QuestionAuthor.vue`

- [ ] **Step 1: Replace route shell script with question editor logic**

Use this script as the base:

```vue
<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import { listSubjects, listTopics, type SubjectDto, type TopicDto } from '@/ipc/curriculum'
import {
  getAdminQuestionEditor,
  upsertAdminQuestion,
  type AdminQuestionEditorDto,
  type AdminQuestionOptionInput,
} from '@/ipc/admin'

const route = useRoute()
const router = useRouter()

const loading = ref(false)
const saving = ref(false)
const error = ref('')
const success = ref('')
const subjects = ref<SubjectDto[]>([])
const topics = ref<TopicDto[]>([])
const recordType = ref('question')
const loadedQuestion = ref<AdminQuestionEditorDto | null>(null)

const questionId = computed(() => route.query.id ? Number(route.query.id) : null)
const isEditing = computed(() => questionId.value !== null)

const subjectId = ref('')
const topicId = ref('')
const subtopicId = ref<number | null>(null)
const familyId = ref<number | null>(null)
const stem = ref('')
const questionFormat = ref('mcq')
const sourceType = ref('authored')
const sourceRef = ref<string | null>(null)
const examYear = ref('')
const difficultyLevel = ref(5000)
const estimatedTime = ref(45)
const marks = ref(1)
const explanation = ref('')
const knowledgeRole = ref('')
const cognitiveDemand = ref('')
const solvePattern = ref('')
const pedagogicFunction = ref('')
const contentGrain = ref('topic')
const options = ref<AdminQuestionOptionInput[]>([
  { option_label: 'A', option_text: '', is_correct: true, position: 1 },
  { option_label: 'B', option_text: '', is_correct: false, position: 2 },
  { option_label: 'C', option_text: '', is_correct: false, position: 3 },
  { option_label: 'D', option_text: '', is_correct: false, position: 4 },
])

const contentTypes = [
  { id: 'question', label: 'Questions', wired: true },
  { id: 'curriculum_text', label: 'Curriculum Text', wired: false },
  { id: 'glossary', label: 'Glossary Entries', wired: false },
  { id: 'notes', label: 'Learning Notes', wired: false },
  { id: 'metadata', label: 'Metadata', wired: false },
]

function resetQuestion() {
  loadedQuestion.value = null
  subtopicId.value = null
  familyId.value = null
  stem.value = ''
  questionFormat.value = 'mcq'
  sourceType.value = 'authored'
  sourceRef.value = null
  examYear.value = ''
  difficultyLevel.value = 5000
  estimatedTime.value = 45
  marks.value = 1
  explanation.value = ''
  knowledgeRole.value = ''
  cognitiveDemand.value = ''
  solvePattern.value = ''
  pedagogicFunction.value = ''
  contentGrain.value = 'topic'
  options.value = [
    { option_label: 'A', option_text: '', is_correct: true, position: 1 },
    { option_label: 'B', option_text: '', is_correct: false, position: 2 },
    { option_label: 'C', option_text: '', is_correct: false, position: 3 },
    { option_label: 'D', option_text: '', is_correct: false, position: 4 },
  ]
}

function normalizeOptionPositions() {
  options.value = options.value.map((option, index) => ({
    ...option,
    option_label: option.option_label || 'ABCDEFGH'[index] || String(index + 1),
    position: index + 1,
  }))
}

function addOption() {
  const index = options.value.length
  options.value.push({
    option_label: 'ABCDEFGH'[index] || String(index + 1),
    option_text: '',
    is_correct: false,
    position: index + 1,
  })
}

function removeOption(index: number) {
  if (options.value.length <= 2) return
  options.value.splice(index, 1)
  if (!options.value.some(option => option.is_correct)) options.value[0].is_correct = true
  normalizeOptionPositions()
}

function markCorrect(index: number) {
  options.value = options.value.map((option, optionIndex) => ({
    ...option,
    is_correct: optionIndex === index,
  }))
}

async function loadLookups() {
  subjects.value = await listSubjects(1)
  if (!subjectId.value && subjects.value[0]) subjectId.value = String(subjects.value[0].id)
  if (subjectId.value) topics.value = await listTopics(Number(subjectId.value))
  if (!topicId.value && topics.value[0]) topicId.value = String(topics.value[0].id)
}

async function loadQuestion() {
  resetQuestion()
  if (!questionId.value) return
  loading.value = true
  error.value = ''
  try {
    const snapshot = await getAdminQuestionEditor(questionId.value)
    loadedQuestion.value = snapshot
    subjectId.value = String(snapshot.subject_id)
    topics.value = await listTopics(snapshot.subject_id)
    topicId.value = String(snapshot.topic_id)
    subtopicId.value = snapshot.subtopic_id
    familyId.value = snapshot.family_id
    stem.value = snapshot.stem
    questionFormat.value = snapshot.question_format
    sourceType.value = snapshot.source_type
    sourceRef.value = snapshot.source_ref
    examYear.value = snapshot.exam_year ? String(snapshot.exam_year) : ''
    difficultyLevel.value = snapshot.difficulty_level
    estimatedTime.value = snapshot.estimated_time_seconds
    marks.value = snapshot.marks
    explanation.value = snapshot.explanation_text ?? ''
    knowledgeRole.value = snapshot.primary_knowledge_role ?? ''
    cognitiveDemand.value = snapshot.primary_cognitive_demand ?? ''
    solvePattern.value = snapshot.primary_solve_pattern ?? ''
    pedagogicFunction.value = snapshot.primary_pedagogic_function ?? ''
    contentGrain.value = snapshot.primary_content_grain ?? 'topic'
    options.value = snapshot.options.map((option, index) => ({
      id: option.id,
      option_label: option.option_label,
      option_text: option.option_text,
      is_correct: option.is_correct,
      misconception_id: option.misconception_id,
      distractor_intent: option.distractor_intent,
      position: option.position ?? index + 1,
    }))
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load question.'
  } finally {
    loading.value = false
  }
}

async function saveQuestion() {
  if (saving.value) return
  error.value = ''
  success.value = ''
  if (!subjectId.value || !topicId.value) {
    error.value = 'Choose a subject and topic before saving.'
    return
  }
  saving.value = true
  normalizeOptionPositions()
  try {
    const result = await upsertAdminQuestion({
      question_id: questionId.value,
      subject_id: Number(subjectId.value),
      topic_id: Number(topicId.value),
      subtopic_id: subtopicId.value,
      family_id: familyId.value,
      stem: stem.value,
      question_format: questionFormat.value,
      explanation_text: explanation.value || null,
      difficulty_level: Number(difficultyLevel.value),
      estimated_time_seconds: Number(estimatedTime.value),
      marks: Number(marks.value),
      source_type: sourceType.value,
      source_ref: isEditing.value ? sourceRef.value : 'admin-authored',
      exam_year: examYear.value ? Number(examYear.value) : null,
      primary_knowledge_role: knowledgeRole.value || null,
      primary_cognitive_demand: cognitiveDemand.value || null,
      primary_solve_pattern: solvePattern.value || null,
      primary_pedagogic_function: pedagogicFunction.value || null,
      primary_content_grain: contentGrain.value || null,
      cognitive_level: cognitiveDemand.value || null,
      options: options.value,
    })
    success.value = `Question #${result.question_id} saved.`
    router.replace({ path: '/admin/content-editor', query: { type: 'question', id: result.question_id } })
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not save question.'
  } finally {
    saving.value = false
  }
}

watch(subjectId, async value => {
  topics.value = value ? await listTopics(Number(value)) : []
  if (!topics.value.some(topic => String(topic.id) === topicId.value)) {
    topicId.value = topics.value[0] ? String(topics.value[0].id) : ''
  }
})

watch(() => route.query.id, loadQuestion)

onMounted(async () => {
  recordType.value = String(route.query.type ?? 'question')
  await loadLookups()
  await loadQuestion()
})
</script>
```

- [ ] **Step 2: Replace route shell template with three-pane CMS editor**

Use this template:

```vue
<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <p class="text-[10px] font-bold uppercase tracking-[0.16em]" :style="{ color: 'var(--ink-muted)' }">
          Structured CMS Records
        </p>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Content Editor</h1>
        <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
          Edit extracted content records. Question editing is wired now; other structured record types are listed for the CMS model.
        </p>
      </div>
      <div class="flex gap-2">
        <AppButton variant="secondary" size="sm" @click="router.push('/admin/questions')">Question Bank</AppButton>
        <AppButton variant="primary" size="sm" :loading="saving" @click="saveQuestion">Save Record</AppButton>
      </div>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    <p v-if="success" class="mb-4 text-sm" :style="{ color: 'var(--accent)' }">{{ success }}</p>

    <div class="grid grid-cols-1 xl:grid-cols-[240px_minmax(0,1fr)_340px] gap-5">
      <AppCard padding="md">
        <h2 class="text-xs font-bold uppercase tracking-wide mb-3" :style="{ color: 'var(--ink-muted)' }">Content Types</h2>
        <div class="space-y-1">
          <button
            v-for="type in contentTypes"
            :key="type.id"
            class="w-full text-left rounded-lg px-3 py-2"
            :style="{
              backgroundColor: recordType === type.id ? 'var(--accent-light)' : 'transparent',
              color: recordType === type.id ? 'var(--accent)' : 'var(--ink)',
            }"
            @click="recordType = type.id"
          >
            <span class="text-sm font-semibold">{{ type.label }}</span>
            <span v-if="!type.wired" class="block text-[10px]" :style="{ color: 'var(--ink-muted)' }">not wired yet</span>
          </button>
        </div>
      </AppCard>

      <AppCard padding="md">
        <div v-if="recordType !== 'question'" class="min-h-[420px] flex items-center justify-center text-center">
          <div>
            <AppBadge color="gold" size="sm">Coming next</AppBadge>
            <h2 class="text-base font-bold mt-3 mb-2" :style="{ color: 'var(--ink)' }">This structured record type is not wired yet</h2>
            <p class="text-sm max-w-md" :style="{ color: 'var(--ink-muted)' }">
              This CMS shell is ready for future curriculum text, glossary, notes, and metadata editors once their backend commands are available.
            </p>
          </div>
        </div>

        <div v-else>
          <div v-if="loading" class="h-72 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--surface)' }" />
          <div v-else>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
              <AppSelect v-model="subjectId" label="Subject" :options="subjects.map(subject => ({ value: String(subject.id), label: subject.name }))" />
              <AppSelect v-model="topicId" label="Topic" :options="topics.map(topic => ({ value: String(topic.id), label: topic.name }))" />
            </div>

            <AppTextarea v-model="stem" label="Question Stem" placeholder="Enter the question text" :rows="5" class="mb-4" />

            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
              <AppSelect
                v-model="questionFormat"
                label="Format"
                :options="[
                  { value: 'mcq', label: 'Multiple Choice' },
                  { value: 'short_answer', label: 'Short Answer' },
                  { value: 'numeric', label: 'Numeric' },
                  { value: 'true_false', label: 'True/False' },
                  { value: 'matching', label: 'Matching' },
                  { value: 'ordering', label: 'Ordering' },
                ]"
              />
              <AppSelect
                v-model="sourceType"
                label="Source"
                :options="[
                  { value: 'authored', label: 'Authored' },
                  { value: 'past_question', label: 'Past Question' },
                  { value: 'generated', label: 'Generated' },
                  { value: 'teacher_upload', label: 'Teacher Upload' },
                ]"
              />
              <AppInput v-model.number="difficultyLevel" label="Difficulty BP" type="number" />
              <AppInput v-model.number="marks" label="Marks" type="number" />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
              <AppInput v-model.number="estimatedTime" label="Estimated Seconds" type="number" />
              <AppInput v-model="examYear" label="Exam Year" type="number" />
            </div>

            <AppTextarea v-model="explanation" label="Explanation" placeholder="Why is the answer correct?" :rows="3" class="mb-5" />

            <div class="flex items-center justify-between mb-3">
              <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Answer Options</h2>
              <AppButton variant="secondary" size="sm" @click="addOption">Add Option</AppButton>
            </div>

            <div class="space-y-2">
              <div v-for="(option, index) in options" :key="index" class="flex items-center gap-2">
                <button
                  class="w-9 h-9 rounded-lg text-xs font-bold"
                  :style="option.is_correct ? { backgroundColor: 'var(--accent)', color: 'white' } : { backgroundColor: 'var(--paper)', color: 'var(--ink-muted)' }"
                  @click="markCorrect(index)"
                >{{ option.option_label }}</button>
                <AppInput v-model="option.option_text" class="flex-1" :placeholder="`Option ${option.option_label}`" />
                <AppButton variant="ghost" size="sm" :disabled="options.length <= 2" @click="removeOption(index)">Remove</AppButton>
              </div>
            </div>
          </div>
        </div>
      </AppCard>

      <div class="space-y-4">
        <AppCard padding="md">
          <div class="flex items-center gap-2 mb-3">
            <CmsStatusBadge :status="loadedQuestion?.review_status ?? (isEditing ? 'loaded' : 'draft')" />
            <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ isEditing ? `Question #${questionId}` : 'New question record' }}</span>
          </div>
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">What This Tests</h2>
          <div class="space-y-3">
            <AppSelect v-model="knowledgeRole" label="Knowledge Role" :options="[
              { value: '', label: 'Let engine infer' },
              { value: 'definition', label: 'Definition' },
              { value: 'procedure', label: 'Procedure' },
              { value: 'application', label: 'Application' },
              { value: 'comparison', label: 'Comparison' },
            ]" />
            <AppSelect v-model="cognitiveDemand" label="Cognitive Demand" :options="[
              { value: '', label: 'Let engine infer' },
              { value: 'recognition', label: 'Recognition' },
              { value: 'recall', label: 'Recall' },
              { value: 'application', label: 'Application' },
              { value: 'analysis', label: 'Analysis' },
              { value: 'inference', label: 'Inference' },
            ]" />
            <AppSelect v-model="solvePattern" label="Solve Pattern" :options="[
              { value: '', label: 'Let engine infer' },
              { value: 'direct_retrieval', label: 'Direct Retrieval' },
              { value: 'substitute_and_solve', label: 'Substitute and Solve' },
              { value: 'pattern_spotting', label: 'Pattern Spotting' },
              { value: 'multi_step_reasoning', label: 'Multi-step Reasoning' },
            ]" />
            <AppSelect v-model="pedagogicFunction" label="Pedagogic Function" :options="[
              { value: '', label: 'Let engine infer' },
              { value: 'foundation_check', label: 'Foundation Check' },
              { value: 'misconception_diagnosis', label: 'Misconception Diagnosis' },
              { value: 'transfer_check', label: 'Transfer Check' },
              { value: 'speed_build', label: 'Speed Build' },
            ]" />
          </div>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-2" :style="{ color: 'var(--ink)' }">Provenance</h2>
          <p class="text-xs mb-1" :style="{ color: 'var(--ink-muted)' }">Source: {{ sourceType }}</p>
          <p class="text-xs mb-1" :style="{ color: 'var(--ink-muted)' }">Reference: {{ sourceRef ?? 'admin-authored' }}</p>
          <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">Family: {{ familyId ?? 'none assigned' }}</p>
        </AppCard>
      </div>
    </div>
  </div>
</template>
```

- [ ] **Step 3: Run frontend build**

```powershell
pnpm build
```

from `frontend/`.

Expected: PASS.

- [ ] **Step 4: Manually smoke the route**

With the existing dev server/Tauri app:

1. Unlock super admin.
2. Open `/admin/content-editor`.
3. Confirm question editor shell renders.
4. Open `/admin/questions`, select a question if any exist, click `Edit in Content Editor`.
5. Confirm the Content Editor receives the question id in the URL.

- [ ] **Step 5: Commit**

```powershell
git add frontend/src/views/admin/content-editor/ContentEditorHome.vue frontend/src/router/index.ts frontend/src/views/admin/questions/QuestionsHome.vue
git commit -m "feat: add content editor question workspace"
```

---

### Task 7: Build Remote Updates Honest Workspace

**Files:**

- Modify: `frontend/src/views/admin/remote/RemoteUpdates.vue`

- [ ] **Step 1: Replace route shell script**

```vue
<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CmsActionQueue from '@/components/admin/cms/CmsActionQueue.vue'
import CmsMetricStrip from '@/components/admin/cms/CmsMetricStrip.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import type { CmsActionItem, CmsMetricItem } from '@/components/admin/cms/types'
import { getAdminQuestionBankStats, getContentHealthReadModel, listContentSources, type ContentHealthReadModelDto, type ContentSourceRegistryEntryDto, type AdminQuestionBankStatsDto } from '@/ipc/admin'
import { listInstalledPacks, type PackSummaryDto } from '@/ipc/sessions'

const router = useRouter()
const loading = ref(true)
const error = ref('')
const health = ref<ContentHealthReadModelDto | null>(null)
const stats = ref<AdminQuestionBankStatsDto | null>(null)
const sources = ref<ContentSourceRegistryEntryDto[]>([])
const packs = ref<PackSummaryDto[]>([])

const metrics = computed<CmsMetricItem[]>(() => [
  { label: 'Packs', value: packs.value.length, caption: 'Local installs' },
  { label: 'Sources', value: health.value?.source_count ?? 0, caption: 'Registered raw inputs' },
  { label: 'Questions', value: stats.value?.total_questions ?? 0, caption: 'Current bank' },
  { label: 'Pending Review', value: stats.value?.pending_review_count ?? 0, tone: 'review' },
  { label: 'Stale Sources', value: health.value?.stale_source_count ?? 0, tone: (health.value?.stale_source_count ?? 0) > 0 ? 'review' : 'neutral' },
  { label: 'Blocked Publish', value: health.value?.blocked_publish_count ?? 0, tone: (health.value?.blocked_publish_count ?? 0) > 0 ? 'danger' : 'neutral' },
])

const actions = computed<CmsActionItem[]>(() => [
  {
    key: 'remote-not-wired',
    title: 'Remote sync/apply commands are not wired yet',
    summary: 'This workspace shows local pack and source readiness now. Preview/apply behavior should be added when backend remote update commands exist.',
    tone: 'review',
    actionLabel: 'Open Packs',
    route: '/admin/packs',
  },
  {
    key: 'review-after-update',
    title: 'Incoming updates should move through review',
    summary: 'Remote additions and changes should become structured draft records before publishing.',
    tone: 'neutral',
    actionLabel: 'Review Queue',
    route: '/admin/questions/review',
  },
])

async function load() {
  loading.value = true
  error.value = ''
  try {
    const [healthResult, statsResult, sourceResult, packResult] = await Promise.all([
      getContentHealthReadModel(),
      getAdminQuestionBankStats(),
      listContentSources(null, null, 20),
      listInstalledPacks(),
    ])
    health.value = healthResult
    stats.value = statsResult
    sources.value = sourceResult
    packs.value = packResult
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load remote update readiness.'
  } finally {
    loading.value = false
  }
}

function openAction(item: CmsActionItem) {
  if (item.route) router.push(item.route)
}

onMounted(load)
</script>
```

- [ ] **Step 2: Replace route shell template**

```vue
<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <p class="text-[10px] font-bold uppercase tracking-[0.16em]" :style="{ color: 'var(--ink-muted)' }">
          Distribution
        </p>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Remote Updates</h1>
        <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
          Prepare the CMS for external content refreshes without hiding what will change.
        </p>
      </div>
      <AppButton variant="secondary" size="sm" @click="load">Refresh</AppButton>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>

    <CmsMetricStrip :items="metrics" class="mb-6" />

    <div class="grid grid-cols-1 xl:grid-cols-3 gap-5">
      <AppCard padding="md" class="xl:col-span-1">
        <h2 class="text-xs font-bold uppercase tracking-wide mb-3" :style="{ color: 'var(--ink-muted)' }">Update Readiness</h2>
        <CmsActionQueue :items="actions" @open="openAction" />
      </AppCard>

      <AppCard padding="md" class="xl:col-span-2">
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Installed Packs</h2>
          <AppButton variant="ghost" size="sm" @click="router.push('/admin/packs')">Manage Packs</AppButton>
        </div>
        <div v-if="loading" class="space-y-2">
          <div v-for="i in 3" :key="i" class="h-12 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
        <div v-else class="space-y-2">
          <div v-for="pack in packs" :key="`${pack.pack_id}:${pack.pack_version}`" class="flex items-center gap-3 rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--paper)' }">
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ pack.pack_id }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ pack.subject_code }} · {{ pack.pack_version }}</p>
            </div>
            <CmsStatusBadge :status="pack.status" />
          </div>
          <p v-if="!packs.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">No packs installed yet.</p>
        </div>
      </AppCard>

      <AppCard padding="md" class="xl:col-span-3">
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Recent Local Sources</h2>
          <AppButton variant="ghost" size="sm" @click="router.push('/admin/content')">Sources & Ingestion</AppButton>
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
          <div v-for="source in sources" :key="source.id" class="flex items-center gap-3 rounded-lg px-3 py-2" :style="{ backgroundColor: 'var(--paper)' }">
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ source.title }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ source.source_kind }} · {{ source.subject_code ?? 'No subject' }}</p>
            </div>
            <CmsStatusBadge :status="source.source_status" />
          </div>
        </div>
        <p v-if="!loading && !sources.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">No source records yet.</p>
      </AppCard>
    </div>
  </div>
</template>
```

- [ ] **Step 3: Run frontend build**

```powershell
pnpm build
```

from `frontend/`.

Expected: PASS.

- [ ] **Step 4: Commit**

```powershell
git add frontend/src/views/admin/remote/RemoteUpdates.vue
git commit -m "feat: add remote updates readiness workspace"
```

---

### Task 8: Align Sources & Ingestion Copy

**Files:**

- Modify: `frontend/src/views/admin/content/ContentPipeline.vue`

- [ ] **Step 1: Update page title and copy**

Change title to `Sources & Ingestion` and copy to:

```vue
Register raw material, track extraction status, and send structured records into review and editing.
```

- [ ] **Step 2: Rename Add Source card**

Change the card heading from `Add Source` to:

```vue
Register Raw Source
```

Keep the explanatory copy:

```vue
Files are registered into the local source registry first. Extraction and publishing happen through foundry/review jobs.
```

- [ ] **Step 3: Add Content Editor action**

In the header action group, add:

```vue
<AppButton variant="secondary" size="sm" @click="$router.push('/admin/content-editor')">Content Editor</AppButton>
```

If the component does not have `$router` access in template, import `useRouter`, initialize `const router = useRouter()`, and use `@click="router.push('/admin/content-editor')"`.

- [ ] **Step 4: Run frontend build**

```powershell
pnpm build
```

from `frontend/`.

Expected: PASS.

- [ ] **Step 5: Commit**

```powershell
git add frontend/src/views/admin/content/ContentPipeline.vue
git commit -m "feat: align source ingestion cms copy"
```

---

### Task 9: Final Verification

**Files:**

- Verify working tree changes from prior tasks.

- [ ] **Step 1: Run frontend production build**

```powershell
pnpm build
```

from `frontend/`.

Expected: exit code 0. The existing CSS `@import` ordering warning may remain.

- [ ] **Step 2: Run Tauri compile check**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml
```

from repo root.

Expected: exit code 0. Existing Rust warnings may remain.

- [ ] **Step 3: Run admin question backend tests**

```powershell
cargo test -p ecoach-commands admin_question_ -- --nocapture
```

Expected: `2 passed; 0 failed`.

- [ ] **Step 4: Run diff whitespace check**

```powershell
git diff --check
```

Expected: no whitespace errors. Windows LF-to-CRLF warnings are acceptable.

- [ ] **Step 5: Manual smoke path**

With the Tauri app/dev server already running:

1. Unlock super admin from the profile switcher.
2. Confirm sidebar says `CMS Console`.
3. Open Dashboard.
4. Open Question Bank.
5. Select a question if content exists.
6. Click `Edit in Content Editor`.
7. Save the question without changing text.
8. Return to Question Bank.
9. Open Sources & Ingestion.
10. Open Remote Updates.

Expected: no blank screens, no console-level Tauri command errors, and successful save if a question exists.

- [ ] **Step 6: Final commit**

If all checks pass and no follow-up fixes are needed:

```powershell
git status --short
git add frontend/src/layouts/AdminLayout.vue frontend/src/router/index.ts frontend/src/components/admin/cms frontend/src/views/admin/AdminHome.vue frontend/src/views/admin/questions/QuestionsHome.vue frontend/src/views/admin/content-editor/ContentEditorHome.vue frontend/src/views/admin/remote/RemoteUpdates.vue frontend/src/views/admin/content/ContentPipeline.vue
git commit -m "feat: shape super admin as cms shell"
```

Expected: commit succeeds and includes only the CMS shell work from this plan.
