<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { listParentAlerts, acknowledgeParentAlert, type ParentAlertRecordDto } from '@/ipc/identity'
import AppButton from '@/components/ui/AppButton.vue'

const auth = useAuthStore()

const loading = ref(true)
const error = ref('')
const alerts = ref<ParentAlertRecordDto[]>([])
const acknowledging = ref<number | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    alerts.value = await listParentAlerts(auth.currentAccount.id, null, null, 50)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load alerts'
  }
  loading.value = false
})

const pendingAlerts = computed(() => alerts.value.filter(a => a.status === 'pending'))
const resolvedAlerts = computed(() => alerts.value.filter(a => a.status !== 'pending'))

function severityDot(severity: string): string {
  if (severity === 'high') return 'var(--warm)'
  if (severity === 'medium') return 'var(--gold)'
  return 'var(--ink-muted)'
}

async function acknowledge(alert: ParentAlertRecordDto) {
  acknowledging.value = alert.id
  try {
    const updated = await acknowledgeParentAlert(alert.id)
    if (updated) {
      const idx = alerts.value.findIndex(a => a.id === alert.id)
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
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Attention Needed</h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
        Issues flagged by the AI coach that may need your support
      </p>
    </div>

    <div v-if="error" class="px-7 py-2 text-xs flex-shrink-0"
      style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-7 max-w-2xl">
      <div v-if="loading" class="space-y-3">
        <div v-for="i in 3" :key="i" class="h-16 rounded-2xl animate-pulse"
          :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <template v-else>
        <!-- Pending alerts -->
        <div v-if="pendingAlerts.length > 0" class="space-y-2 mb-8">
          <div
            v-for="alert in pendingAlerts"
            :key="alert.id"
            class="alert-row flex items-start gap-4 px-5 py-4 rounded-2xl border"
            :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
          >
            <div class="mt-1.5 w-2.5 h-2.5 rounded-full flex-shrink-0"
              :style="{ backgroundColor: severityDot(alert.severity) }" />
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ alert.message }}</p>
              <p v-if="alert.action_required" class="text-xs mt-0.5" :style="{ color: 'var(--ink-secondary)' }">
                {{ alert.action_required }}
              </p>
              <p class="text-[10px] mt-1.5" :style="{ color: 'var(--ink-muted)' }">
                {{ new Date(alert.created_at).toLocaleDateString('en-GB', { day: 'numeric', month: 'short' }) }}
              </p>
            </div>
            <div class="flex items-center gap-2 flex-shrink-0">
              <span class="severity-chip" :style="{ color: severityDot(alert.severity) }">{{ alert.severity }}</span>
              <AppButton variant="ghost" size="sm" :loading="acknowledging === alert.id" @click="acknowledge(alert)">Done</AppButton>
            </div>
          </div>
        </div>

        <!-- Empty / all clear -->
        <div v-else-if="alerts.length === 0" class="text-center py-16">
          <div class="text-3xl mb-3" :style="{ color: 'var(--accent)' }">✓</div>
          <p class="text-sm font-bold mb-1" :style="{ color: 'var(--ink)' }">All clear!</p>
          <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">No issues flagged for your children.</p>
        </div>

        <div v-else-if="pendingAlerts.length === 0" class="text-center py-8">
          <p class="text-sm font-bold mb-1" :style="{ color: 'var(--ink)' }">All alerts acknowledged</p>
          <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">Nothing needs your attention right now.</p>
        </div>

        <!-- Resolved -->
        <div v-if="resolvedAlerts.length > 0">
          <p class="section-label mb-3">Recently Resolved</p>
          <div class="space-y-1.5">
            <div v-for="alert in resolvedAlerts.slice(0, 5)" :key="alert.id"
              class="flex items-center gap-3 px-4 py-3 rounded-xl"
              :style="{ backgroundColor: 'var(--surface)' }">
              <div class="w-2 h-2 rounded-full flex-shrink-0" :style="{ backgroundColor: 'var(--border-soft)' }" />
              <p class="text-xs flex-1 line-through" :style="{ color: 'var(--ink-muted)' }">{{ alert.message }}</p>
              <span class="text-[10px]" :style="{ color: 'var(--accent)' }">resolved</span>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.eyebrow { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.16em; color: var(--accent); margin-bottom: 4px; }
.section-label { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.14em; color: var(--ink-muted); }
.alert-row { transition: background-color 100ms; }
.alert-row:hover { background-color: var(--paper) !important; }
.severity-chip { font-size: 10px; font-weight: 700; text-transform: uppercase; }
</style>
