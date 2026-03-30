<script setup lang="ts">
/**
 * MockHomeContent - inner content for MockHome view.
 * Renders readiness, mock types, and quick actions.
 */
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { getReadinessReport } from '@/ipc/readiness'
import { listMockSessions } from '@/ipc/mock'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ReadinessGauge from '@/components/viz/ReadinessGauge.vue'
import MockHistory from './MockHistory.vue'

const auth = useAuthStore()
const readiness = ref<any>(null)
const sessions = ref<any[]>([])
const loading = ref(true)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    const [r, s] = await Promise.all([
      getReadinessReport(auth.currentAccount.id).catch(() => null),
      listMockSessions(auth.currentAccount.id).catch(() => []),
    ])
    readiness.value = r
    sessions.value = s
  } catch {}
  loading.value = false
})

const mockTypes = [
  { key: 'full', label: 'Full Mock', desc: 'Complete exam simulation', icon: '⊞', time: '2h 30m' },
  { key: 'topic', label: 'Topic Mock', desc: 'Focus on specific topics', icon: '◎', time: '30-45m' },
  { key: 'mini', label: 'Mini Mock', desc: 'Quick 20-question check', icon: '◇', time: '15-20m' },
  { key: 'pressure', label: 'Pressure Mock', desc: 'Tighter timing, harder questions', icon: '⚡', time: '45m' },
  { key: 'recovery', label: 'Recovery Mock', desc: 'Gentle, after a bad result', icon: '🛟', time: '30m' },
]
</script>

<template>
  <div>
    <div v-if="loading" class="space-y-3">
      <div v-for="i in 3" :key="i" class="h-20 rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>
    <template v-else>
      <div class="grid grid-cols-2 lg:grid-cols-3 gap-3 mb-6">
        <AppCard v-for="m in mockTypes" :key="m.key" hover padding="md">
          <div class="flex items-start gap-2.5">
            <div class="w-9 h-9 rounded-lg flex items-center justify-center text-base" :style="{backgroundColor:'var(--accent-light)',color:'var(--accent)'}">{{ m.icon }}</div>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-semibold" :style="{color:'var(--text)'}">{{ m.label }}</p>
              <p class="text-[9px]" :style="{color:'var(--text-3)'}">{{ m.desc }}</p>
              <AppBadge color="muted" size="xs" class="mt-1">{{ m.time }}</AppBadge>
            </div>
          </div>
        </AppCard>
      </div>
      <MockHistory v-if="sessions.length" :sessions="sessions.map((s,i) => ({id:s.id||i,type:s.type||'Mock',date:s.date||'',score:s.score||0,change:s.change||0,questionCount:s.questionCount||0}))" />
    </template>
  </div>
</template>
