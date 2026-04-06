<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { listParentAlerts, acknowledgeParentAlert, type ParentAlertRecordDto } from '@/ipc/identity'
import { buildParentDashboard, type ParentDashboardSnapshot } from '@/ipc/reporting'
import AppButton from '@/components/ui/AppButton.vue'

const auth = useAuthStore()

const loading = ref(true)
const error = ref('')
const alerts = ref<ParentAlertRecordDto[]>([])
const dashboard = ref<ParentDashboardSnapshot | null>(null)
const acknowledging = ref<number | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    ;[alerts.value, dashboard.value] = await Promise.all([
      listParentAlerts(auth.currentAccount.id, null, 'pending', 50),
      buildParentDashboard(auth.currentAccount.id),
    ])
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load data'
  }
  loading.value = false
})

const systemAlerts = computed(() =>
  alerts.value.filter(a => a.trigger_type?.includes('auto') || a.trigger_type?.includes('system')),
)
const parentAlerts = computed(() =>
  alerts.value.filter(a => a.severity === 'high' && !systemAlerts.value.includes(a)),
)
const lowPriorityAlerts = computed(() =>
  alerts.value.filter(a => !systemAlerts.value.includes(a) && !parentAlerts.value.includes(a)),
)
const recommendations = computed(() => {
  const recs: string[] = []
  dashboard.value?.students.forEach(s => { s.recommendations?.forEach(r => recs.push(r)) })
  return recs
})

async function acknowledge(alertId: number) {
  acknowledging.value = alertId
  try {
    const updated = await acknowledgeParentAlert(alertId)
    if (updated) {
      const idx = alerts.value.findIndex(a => a.id === alertId)
      if (idx !== -1) alerts.value[idx] = updated
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to acknowledge'
  }
  acknowledging.value = null
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <p class="eyebrow">Family</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Intervention Center
      </h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
        See what the AI is handling, what needs your attention, and what you can do
      </p>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-6">
      <div v-if="loading" class="grid grid-cols-3 gap-4">
        <div v-for="i in 3" :key="i" class="h-40 rounded-2xl animate-pulse"
          :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <div v-else class="grid grid-cols-3 gap-5">

        <!-- System Handling -->
        <div>
          <p class="col-label" :style="{ color: 'var(--accent)' }">System Handling</p>
          <div class="space-y-2">
            <div v-for="alert in systemAlerts" :key="alert.id" class="alert-card">
              <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">{{ alert.message }}</p>
            </div>
            <div v-if="systemAlerts.length === 0" class="alert-card">
              <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">No automated interventions active</p>
            </div>
          </div>
        </div>

        <!-- Needs Parent -->
        <div>
          <p class="col-label" :style="{ color: 'var(--warm)' }">Needs Your Attention</p>
          <div class="space-y-2">
            <div v-for="alert in parentAlerts" :key="alert.id" class="alert-card">
              <p class="text-xs font-semibold mb-1" :style="{ color: 'var(--ink)' }">{{ alert.message }}</p>
              <p v-if="alert.action_required" class="text-xs mb-2" :style="{ color: 'var(--ink-muted)' }">{{ alert.action_required }}</p>
              <AppButton variant="ghost" size="sm" :loading="acknowledging === alert.id" @click="acknowledge(alert.id)">
                Mark done
              </AppButton>
            </div>
            <div v-if="parentAlerts.length === 0" class="alert-card">
              <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">No urgent items</p>
            </div>
          </div>
        </div>

        <!-- Recommendations -->
        <div>
          <p class="col-label" :style="{ color: 'var(--gold)' }">Recommended Actions</p>
          <div class="space-y-2">
            <div v-for="(rec, i) in recommendations.slice(0, 4)" :key="i" class="alert-card">
              <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">{{ rec }}</p>
            </div>
            <div v-for="alert in lowPriorityAlerts.slice(0, Math.max(0, 3 - recommendations.length))" :key="alert.id" class="alert-card">
              <p class="text-xs" :style="{ color: 'var(--ink-secondary)' }">{{ alert.message }}</p>
            </div>
            <div v-if="recommendations.length === 0 && lowPriorityAlerts.length === 0" class="alert-card">
              <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">No recommendations right now</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.16em; color: var(--accent); margin-bottom: 4px; }
.col-label { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.14em; margin-bottom: 12px; display: block; }
.alert-card {
  padding: 12px 14px;
  border-radius: 14px;
  border: 1px solid var(--border-soft);
  background: var(--surface);
}
</style>
