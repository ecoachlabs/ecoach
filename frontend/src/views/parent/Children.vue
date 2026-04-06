<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { createAccount, linkParentStudent, listLinkedStudents } from '@/ipc/identity'
import { buildParentDashboard } from '@/ipc/reporting'
import type { AccountSummaryDto, ParentDashboardSnapshot } from '@/types'
import AppInput from '@/components/ui/AppInput.vue'
import AppModal from '@/components/ui/AppModal.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { PIN_LENGTH, isValidPin } from '@/utils/validation'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const error = ref('')
const successMessage = ref('')
const children = ref<AccountSummaryDto[]>([])
const dashboard = ref<ParentDashboardSnapshot | null>(null)

const showCreateModal = ref(false)
const savingChild = ref(false)
const childName = ref('')
const childPin = ref('')
const childTier = ref('standard')
const createError = ref('')

const tierOptions = [
  { value: 'standard', label: 'Standard' },
  { value: 'premium', label: 'Premium' },
  { value: 'elite', label: 'Elite' },
]

const childRows = computed(() =>
  children.value.map((child) => ({
    child,
    insight: dashboard.value?.students.find((student) => student.student_id === child.id) ?? null,
  })),
)

const childrenNeedingSupport = computed(
  () => dashboard.value?.students.filter((student) => student.active_risks.length > 0).length ?? 0,
)
const steadyChildren = computed(() => Math.max(0, children.value.length - childrenNeedingSupport.value))

onMounted(() => { void loadChildren() })

async function loadChildren() {
  if (!auth.currentAccount) { loading.value = false; return }
  loading.value = true
  error.value = ''
  const [linkedResult, dashboardResult] = await Promise.allSettled([
    listLinkedStudents(auth.currentAccount.id),
    buildParentDashboard(auth.currentAccount.id),
  ])
  if (linkedResult.status === 'fulfilled') children.value = linkedResult.value
  else error.value = extractError(linkedResult.reason, 'Failed to load linked children')
  if (dashboardResult.status === 'fulfilled') dashboard.value = dashboardResult.value
  loading.value = false
}

function openCreateModal() { showCreateModal.value = true; createError.value = ''; successMessage.value = '' }
function closeCreateModal() { showCreateModal.value = false; childName.value = ''; childPin.value = ''; childTier.value = 'standard'; createError.value = '' }

async function createChildAccount() {
  if (!auth.currentAccount || savingChild.value) return
  const displayName = childName.value.trim()
  const submittedPin = childPin.value.trim()
  createError.value = ''
  if (!displayName) { createError.value = 'Enter your child name.'; return }
  if (!isValidPin(submittedPin)) { createError.value = `Child PINs must be exactly ${PIN_LENGTH} digits.`; return }
  savingChild.value = true
  try {
    const account = await createAccount({ account_type: 'student', display_name: displayName, pin: submittedPin, entitlement_tier: childTier.value })
    await linkParentStudent(auth.currentAccount.id, account.id)
    closeCreateModal()
    await loadChildren()
    successMessage.value = `${account.display_name} is now linked to your parent account.`
  } catch (err) { createError.value = extractError(err, 'Failed to create child account') }
  savingChild.value = false
}

function readinessColor(band: string | null | undefined): string {
  const n = (band ?? '').toLowerCase()
  if (n.includes('strong') || n.includes('ready')) return 'var(--accent)'
  if (n.includes('risk') || n.includes('not ready')) return 'var(--warm)'
  return 'var(--gold)'
}

function extractError(err: unknown, fallback: string) {
  if (typeof err === 'string') return err
  if (err && typeof err === 'object' && 'message' in err && typeof (err as any).message === 'string') return (err as any).message
  return fallback
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Family</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Children</h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          Manage the student accounts linked to your family
        </p>
      </div>
      <button class="add-btn" @click="openCreateModal">+ Add Child</button>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>
    <div v-if="successMessage" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(13,148,136,0.08); color: var(--accent);">{{ successMessage }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-hidden flex">

      <!-- Left: children list -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4">

        <div v-if="loading" class="space-y-3">
          <div v-for="i in 3" :key="i" class="h-32 rounded-2xl animate-pulse"
            :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>

        <div v-else-if="childRows.length === 0" class="flex flex-col items-center justify-center h-64 gap-4">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No child accounts linked yet.</p>
          <button class="add-btn" @click="openCreateModal">Create First Child Account</button>
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="row in childRows"
            :key="row.child.id"
            class="child-card px-6 py-5 rounded-2xl border"
            :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
          >
            <div class="flex items-start justify-between gap-3 mb-4">
              <div class="flex items-center gap-3">
                <div class="child-avatar">{{ row.child.display_name.charAt(0).toUpperCase() }}</div>
                <div>
                  <h2 class="font-display text-lg font-bold" :style="{ color: 'var(--ink)' }">{{ row.child.display_name }}</h2>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                    Last active: {{ row.child.last_active_label || 'Not yet active' }}
                  </p>
                </div>
              </div>
              <span class="readiness-badge" :style="{ color: readinessColor(row.insight?.overall_readiness_band) }">
                {{ (row.insight?.overall_readiness_band ?? 'Building').replace(/_/g, ' ') }}
              </span>
            </div>

            <div class="grid grid-cols-2 gap-3 mb-4">
              <div class="meta-box">
                <p class="text-[10px] uppercase font-bold mb-1" :style="{ color: 'var(--ink-muted)' }">Target</p>
                <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ row.insight?.exam_target ?? 'Not set' }}</p>
              </div>
              <div class="meta-box">
                <p class="text-[10px] uppercase font-bold mb-1" :style="{ color: 'var(--ink-muted)' }">Active Risks</p>
                <p class="text-sm font-semibold" :style="{ color: row.insight?.active_risks.length ? 'var(--warm)' : 'var(--accent)' }">
                  {{ row.insight?.active_risks.length ?? 0 }}
                </p>
              </div>
            </div>

            <div v-if="row.insight?.active_risks.length" class="mb-4">
              <div class="flex flex-wrap gap-1.5">
                <span
                  v-for="risk in row.insight.active_risks.slice(0, 3)"
                  :key="risk.title"
                  class="risk-tag"
                >{{ risk.title }}</span>
              </div>
            </div>

            <div class="flex gap-2">
              <button class="action-btn primary" @click="router.push('/parent/child/' + row.child.id)">
                Open Dashboard →
              </button>
              <button class="action-btn" @click="router.push('/parent/curriculum')">Curriculum</button>
            </div>
          </div>
        </div>
      </div>

      <!-- Right: stats -->
      <div
        class="w-64 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Overview</p>
        </div>
        <div class="flex-1 p-4 space-y-3">
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ children.length }}</p>
            <p class="stat-label">Linked Children</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--warm)' }">{{ childrenNeedingSupport }}</p>
            <p class="stat-label">Need Support</p>
          </div>
          <div class="stat-card text-center">
            <p class="text-3xl font-black tabular-nums" :style="{ color: 'var(--accent)' }">{{ steadyChildren }}</p>
            <p class="stat-label">Steady Progress</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Create modal -->
    <AppModal :open="showCreateModal" title="Create Child Account" @close="closeCreateModal">
      <div class="space-y-4">
        <div v-if="createError" class="p-3 rounded-lg text-sm"
          style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ createError }}</div>
        <AppInput v-model="childName" label="Child Name" placeholder="Enter your child's name" />
        <AppInput v-model="childPin" label="Student PIN" type="password" placeholder="4 digits" />
        <AppSelect :model-value="childTier" label="Plan" :options="tierOptions" @update:model-value="childTier = String($event)" />
        <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
          Child accounts use 4-digit PINs. The account will be linked to you automatically.
        </p>
      </div>
      <template #footer>
        <AppButton variant="ghost" @click="closeCreateModal">Cancel</AppButton>
        <AppButton variant="primary" :loading="savingChild" @click="createChildAccount">Create Account</AppButton>
      </template>
    </AppModal>
  </div>
</template>

<style scoped>
.eyebrow { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.16em; color: var(--accent); margin-bottom: 4px; }
.section-label { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.14em; color: var(--ink-muted); }

.add-btn {
  padding: 8px 20px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  background: var(--accent);
  color: white;
  border: none;
  transition: opacity 120ms;
}
.add-btn:hover { opacity: 0.87; }

.child-card { transition: background-color 100ms; }
.child-card:hover { background-color: var(--paper) !important; }

.child-avatar {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: var(--ink);
  color: var(--paper);
  font-size: 18px;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
}

.readiness-badge {
  font-size: 11px;
  font-weight: 700;
  text-transform: capitalize;
}

.meta-box {
  padding: 12px;
  border-radius: 12px;
  border: 1px solid var(--border-soft);
  background: var(--paper);
}

.risk-tag {
  font-size: 10px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 999px;
  background: rgba(194,65,12,0.08);
  color: var(--warm);
  border: 1px solid rgba(194,65,12,0.2);
}

.action-btn {
  padding: 7px 16px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: transparent;
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 100ms;
}
.action-btn:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }
.action-btn.primary { background: var(--ink); color: var(--paper); border-color: var(--ink); }
.action-btn.primary:hover { opacity: 0.85; background: var(--ink); }

.stat-card { padding: 16px; border-radius: 14px; border: 1px solid var(--border-soft); background: var(--paper); }
.stat-label { font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; color: var(--ink-muted); margin-top: 4px; }
</style>
