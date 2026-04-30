<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
  adminListPastPapers,
  adminDeletePastPaper,
  type AdminPastPaperListItem,
} from '@/ipc/pastPaperAdmin'

const router = useRouter()

const rows = ref<AdminPastPaperListItem[]>([])
const loading = ref(true)
const error = ref('')
const search = ref('')
const busyId = ref<number | null>(null)

// Bulk selection. Keyed by paper_id so the set survives filter changes.
const selected = ref<Set<number>>(new Set())
const bulkBusy = ref(false)

function toggleSelected(paperId: number): void {
  const next = new Set(selected.value)
  next.has(paperId) ? next.delete(paperId) : next.add(paperId)
  selected.value = next
}

function clearSelection(): void {
  selected.value = new Set()
}

function toggleSelectAllFiltered(): void {
  const allInView = filtered.value.map(r => r.paper_id)
  const allSelected = allInView.every(id => selected.value.has(id))
  const next = new Set(selected.value)
  if (allSelected) {
    for (const id of allInView) next.delete(id)
  } else {
    for (const id of allInView) next.add(id)
  }
  selected.value = next
}

async function onBulkDelete(): Promise<void> {
  if (bulkBusy.value || selected.value.size === 0) return
  const count = selected.value.size
  const ok = window.confirm(
    `Delete ${count} paper${count === 1 ? '' : 's'}? Question bank records are preserved; ` +
      `only the paper metadata and question-links will be removed.`,
  )
  if (!ok) return
  bulkBusy.value = true
  error.value = ''
  // Run sequentially so one failure doesn't leak a partial half-deleted state.
  const ids = Array.from(selected.value)
  const failed: number[] = []
  for (const id of ids) {
    try {
      await adminDeletePastPaper(id)
      rows.value = rows.value.filter(r => r.paper_id !== id)
    } catch (e: any) {
      failed.push(id)
      error.value = typeof e === 'string' ? e : e?.message ?? 'Some deletes failed'
    }
  }
  selected.value = new Set(failed) // keep failures selected so admin can retry
  bulkBusy.value = false
}

onMounted(async () => {
  await reload()
})

async function reload(): Promise<void> {
  loading.value = true
  error.value = ''
  try {
    rows.value = await adminListPastPapers(null)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load past papers'
  } finally {
    loading.value = false
  }
}

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return rows.value
  return rows.value.filter(r =>
    r.subject_name.toLowerCase().includes(q) ||
    r.title.toLowerCase().includes(q) ||
    String(r.exam_year).includes(q) ||
    (r.paper_code ?? '').toLowerCase().includes(q),
  )
})

function formatDate(iso: string): string {
  if (!iso) return ''
  const d = new Date(iso.replace(' ', 'T'))
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}

async function onDelete(paperId: number): Promise<void> {
  if (busyId.value != null) return
  const ok = window.confirm(
    'Delete this past paper? Question records stay in the bank (they may be reused), ' +
      'but the paper metadata and links will be removed.',
  )
  if (!ok) return
  busyId.value = paperId
  try {
    await adminDeletePastPaper(paperId)
    rows.value = rows.value.filter(r => r.paper_id !== paperId)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Delete failed'
  } finally {
    busyId.value = null
  }
}
</script>

<template>
  <div class="ppl-shell">
    <header class="ppl-head">
      <div>
        <p class="ppl-eyebrow">CONTENT · PAST PAPERS</p>
        <h1 class="ppl-title">Past Papers</h1>
        <p class="ppl-sub">
          Every paper that feeds the student "Past Questions" page. Authoring is supported via manual
          entry or PDF / Word / image import.
        </p>
      </div>
      <button class="ppl-cta" @click="router.push('/admin/past-papers/new')">
        <span>+ NEW PAPER</span>
      </button>
    </header>

    <section class="ppl-search">
      <label class="ppl-search-label" for="ppl-search-input">SEARCH</label>
      <input
        id="ppl-search-input"
        v-model="search"
        type="text"
        class="ppl-search-input"
        placeholder="subject · year · paper code · title"
        autocomplete="off"
      />
    </section>

    <!-- Bulk action bar — only shown when at least one row is selected. -->
    <section v-if="selected.size > 0" class="ppl-bulk">
      <p class="ppl-bulk-count">
        {{ selected.size }} selected
      </p>
      <button class="ppl-bulk-btn" type="button" @click="clearSelection">Clear selection</button>
      <button
        class="ppl-bulk-btn ppl-bulk-btn--danger"
        type="button"
        :disabled="bulkBusy"
        @click="onBulkDelete"
      >
        {{ bulkBusy ? 'DELETING…' : `DELETE ${selected.size}` }}
      </button>
    </section>

    <p v-if="error" class="ppl-banner">[ ERROR ] {{ error }}</p>

    <p v-if="loading" class="ppl-mono-tag">[ LOADING... ]</p>

    <p v-else-if="filtered.length === 0" class="ppl-empty">
      {{ rows.length === 0 ? 'No past papers yet. Create one to get started.' : `No papers match "${search}".` }}
    </p>

    <div v-else class="ppl-list-wrap">
      <!-- Select-all header (shows count in view, toggles all filtered). -->
      <header class="ppl-select-head">
        <button
          type="button"
          class="ppl-select-check"
          :class="{ 'ppl-select-check--on': filtered.length > 0 && filtered.every(r => selected.has(r.paper_id)) }"
          :aria-pressed="filtered.length > 0 && filtered.every(r => selected.has(r.paper_id))"
          :disabled="filtered.length === 0"
          @click="toggleSelectAllFiltered"
        >
          <span aria-hidden="true">
            {{ filtered.length > 0 && filtered.every(r => selected.has(r.paper_id)) ? '✓' : '' }}
          </span>
        </button>
        <span class="ppl-select-head-label">{{ filtered.length }} paper{{ filtered.length === 1 ? '' : 's' }}</span>
      </header>

      <ol class="ppl-list" role="list">
        <li
          v-for="row in filtered"
          :key="row.paper_id"
          class="ppl-row"
          :class="{ 'ppl-row--selected': selected.has(row.paper_id) }"
        >
          <button
            type="button"
            class="ppl-row-check"
            :class="{ 'ppl-row-check--on': selected.has(row.paper_id) }"
            :aria-pressed="selected.has(row.paper_id)"
            :title="selected.has(row.paper_id) ? 'Deselect' : 'Select for bulk action'"
            @click="toggleSelected(row.paper_id)"
          >
            <span aria-hidden="true">{{ selected.has(row.paper_id) ? '✓' : '' }}</span>
          </button>
          <button
            type="button"
            class="ppl-row-main"
            @click="router.push(`/admin/past-papers/${row.paper_id}`)"
          >
            <div class="ppl-row-copy">
              <p class="ppl-row-code">
                {{ row.subject_name.toUpperCase().slice(0, 4) }}
                <span class="ppl-dot" aria-hidden="true">·</span>
                {{ row.exam_year }}<template v-if="row.paper_code"> · {{ row.paper_code }}</template>
              </p>
              <h2 class="ppl-row-title">{{ row.title }}</h2>
              <p class="ppl-row-meta">
                {{ row.question_count }} question{{ row.question_count === 1 ? '' : 's' }}
                <span class="ppl-dot" aria-hidden="true">·</span>
                Added {{ formatDate(row.created_at) }}
              </p>
            </div>
            <span class="ppl-arrow" aria-hidden="true">→</span>
          </button>
          <button
            type="button"
            class="ppl-delete"
            :disabled="busyId === row.paper_id"
            @click="onDelete(row.paper_id)"
          >
            {{ busyId === row.paper_id ? '…' : '✕' }}
          </button>
        </li>
      </ol>
    </div>
  </div>
</template>

<style scoped>
.ppl-shell {
  --paper: #faf8f5;
  --paper-dim: #f2efe9;
  --ink: #1a1612;
  --ink-secondary: rgba(26, 22, 18, 0.60);
  --ink-muted: rgba(26, 22, 18, 0.40);
  --rule: rgba(26, 22, 18, 0.12);
  --rule-strong: rgba(26, 22, 18, 0.28);
  --danger: #b91c1c;

  min-height: 100%;
  padding: 36px clamp(20px, 3.5vw, 48px) 64px;
  background: var(--paper);
  color: var(--ink);
  font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  letter-spacing: -0.005em;
}

.ppl-head {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 32px;
  margin-bottom: 24px;
}
.ppl-eyebrow {
  margin: 0 0 10px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.ppl-title {
  margin: 0 0 8px;
  font-family: 'Space Grotesk', sans-serif;
  font-weight: 400;
  font-size: clamp(32px, 4vw, 48px);
  letter-spacing: -0.025em;
  color: var(--ink);
}
.ppl-sub { margin: 0; max-width: 62ch; font-size: 14px; color: var(--ink-secondary); }

.ppl-cta {
  padding: 14px 24px;
  border: 1px solid var(--ink);
  border-radius: 999px;
  background: var(--ink);
  color: var(--paper);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.22em;
  cursor: pointer;
  transition: transform 140ms ease;
  white-space: nowrap;
}
.ppl-cta:hover { transform: translateY(-1px); }

.ppl-search {
  display: grid;
  gap: 8px;
  max-width: 520px;
  margin-bottom: 28px;
}
.ppl-search-label {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.ppl-search-input {
  padding: 12px 0;
  border: none;
  border-bottom: 1px solid var(--rule-strong);
  background: transparent;
  color: var(--ink);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 14px;
  outline: none;
  transition: border-color 140ms ease;
}
.ppl-search-input::placeholder { color: var(--ink-muted); }
.ppl-search-input:focus { border-color: var(--ink); }

.ppl-banner {
  margin: 0 0 16px;
  padding: 10px 14px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--danger);
  border: 1px solid rgba(185, 28, 28, 0.3);
  border-radius: 8px;
}

.ppl-mono-tag {
  margin: 32px 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.ppl-empty {
  margin: 32px 0;
  padding: 48px;
  border: 1px dashed var(--rule-strong);
  border-radius: 12px;
  text-align: center;
  color: var(--ink-muted);
  font-size: 14px;
}

/* Bulk action bar */
.ppl-bulk {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  margin-bottom: 16px;
  border: 1px solid var(--ink);
  border-radius: 8px;
  background: color-mix(in srgb, var(--ink) 5%, transparent);
}
.ppl-bulk-count {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--ink);
}
.ppl-bulk-btn {
  padding: 8px 14px;
  border: 1px solid var(--rule-strong);
  border-radius: 6px;
  background: transparent;
  color: var(--ink-secondary);
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  cursor: pointer;
  transition: border-color 140ms ease, color 140ms ease;
}
.ppl-bulk-btn:hover:not(:disabled) { border-color: var(--ink); color: var(--ink); }
.ppl-bulk-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.ppl-bulk-btn--danger { margin-left: auto; color: var(--danger); border-color: rgba(185, 28, 28, 0.3); }
.ppl-bulk-btn--danger:hover:not(:disabled) {
  border-color: var(--danger);
  background: var(--danger);
  color: var(--paper);
}

/* Select-all header + row checkbox */
.ppl-list-wrap { }
.ppl-select-head {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 4px 12px;
  border-bottom: 1px solid var(--rule-strong);
}
.ppl-select-head-label {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-secondary);
}
.ppl-select-check,
.ppl-row-check {
  width: 22px;
  height: 22px;
  border: 1px solid var(--rule-strong);
  border-radius: 4px;
  background: transparent;
  color: var(--ink);
  font-size: 12px;
  line-height: 1;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background 140ms ease, border-color 140ms ease, color 140ms ease;
}
.ppl-select-check:hover,
.ppl-row-check:hover { border-color: var(--ink); }
.ppl-select-check--on,
.ppl-row-check--on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}
.ppl-select-check:disabled { opacity: 0.3; cursor: not-allowed; }

/* List */
.ppl-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
}
.ppl-row {
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr) 40px;
  align-items: center;
  gap: 12px;
  padding: 0 4px;
  border-bottom: 1px solid var(--rule);
  transition: background 140ms ease;
}
.ppl-row:first-child { border-top: 1px solid var(--rule); }
.ppl-row--selected { background: color-mix(in srgb, var(--ink) 4%, transparent); }

.ppl-row-main {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 32px;
  align-items: center;
  gap: 16px;
  padding: 20px 4px;
  background: transparent;
  border: none;
  text-align: left;
  cursor: pointer;
  color: inherit;
  transition: background 140ms ease, padding-left 180ms cubic-bezier(0.16, 1, 0.3, 1);
}
.ppl-row-main:hover { background: var(--paper-dim); padding-left: 16px; }

.ppl-row-copy { display: grid; gap: 4px; min-width: 0; }
.ppl-row-code {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-secondary);
}
.ppl-row-title {
  margin: 0;
  font-family: 'Space Grotesk', sans-serif;
  font-weight: 400;
  font-size: 18px;
  letter-spacing: -0.01em;
  color: var(--ink);
}
.ppl-row-meta {
  margin: 0;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}
.ppl-dot { color: var(--ink-muted); margin: 0 4px; }
.ppl-arrow {
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 16px;
  color: var(--ink-secondary);
  justify-self: end;
}

.ppl-delete {
  width: 32px;
  height: 32px;
  margin: auto 0;
  border: 1px solid var(--rule-strong);
  border-radius: 8px;
  background: transparent;
  color: var(--ink-muted);
  font-size: 12px;
  cursor: pointer;
  transition: border-color 140ms ease, color 140ms ease;
}
.ppl-delete:hover:not(:disabled) { border-color: var(--danger); color: var(--danger); }
.ppl-delete:disabled { opacity: 0.4; cursor: not-allowed; }

@media (prefers-color-scheme: dark) {
  .ppl-shell {
    --paper: #0a0906;
    --paper-dim: #15130f;
    --ink: #f3ede2;
    --ink-secondary: rgba(243, 237, 226, 0.60);
    --ink-muted: rgba(243, 237, 226, 0.40);
    --rule: rgba(243, 237, 226, 0.12);
    --rule-strong: rgba(243, 237, 226, 0.28);
  }
}
</style>
