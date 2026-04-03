<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listMockSessions, type MockSessionSummaryDto } from '@/ipc/mock'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const auth = useAuthStore()
const router = useRouter()

const loading = ref(true)
const sessions = ref<MockSessionSummaryDto[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    sessions.value = await listMockSessions(auth.currentAccount.id, 50)
  } catch (e) {
    console.error('Failed to load mock history:', e)
  }
  loading.value = false
})

function gradeColor(grade: string | null): string {
  if (!grade) return 'muted'
  if (['A1', 'B2', 'B3'].includes(grade)) return 'success'
  if (['C4', 'C5', 'C6'].includes(grade)) return 'gold'
  return 'danger'
}

function gradeTextColor(grade: string | null): string {
  if (!grade) return 'var(--text-3)'
  if (['A1', 'B2', 'B3'].includes(grade)) return 'var(--success)'
  if (['C4', 'C5', 'C6'].includes(grade)) return 'var(--gold)'
  return 'var(--danger)'
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-3xl mx-auto reveal-stagger">
    <div class="flex items-center justify-between mb-8">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Battle History</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">All your mock exam sessions.</p>
      </div>
      <AppButton variant="ghost" size="sm" @click="router.push('/student/mock')">← Back</AppButton>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="space-y-3">
      <div v-for="i in 5" :key="i" class="h-16 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Empty -->
    <AppCard v-else-if="sessions.length === 0" padding="lg" class="text-center">
      <p class="text-sm mb-4" :style="{ color: 'var(--text-3)' }">No mock sessions yet.</p>
      <AppButton variant="primary" @click="router.push('/student/mock/setup')">Start Your First Mock</AppButton>
    </AppCard>

    <!-- List -->
    <div v-else class="space-y-3">
      <AppCard
        v-for="s in sessions"
        :key="s.id"
        padding="md"
        hover
        @click="s.status === 'completed' ? router.push(`/student/mock/review/${s.id}`) : null"
      >
        <div class="flex items-center gap-4">
          <!-- Grade badge -->
          <div class="w-12 h-12 rounded-xl flex items-center justify-center text-sm font-bold shrink-0"
            :style="{
              backgroundColor: s.grade ? 'var(--card-bg)' : 'var(--border-soft)',
              color: gradeTextColor(s.grade),
              border: '2px solid',
              borderColor: s.grade ? gradeTextColor(s.grade) : 'var(--card-border)',
            }">
            {{ s.grade ?? '—' }}
          </div>

          <div class="flex-1 min-w-0">
            <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ s.mock_type.replace(/_/g, ' ') }}</p>
            <div class="flex items-center gap-2 mt-0.5 text-[11px]" :style="{ color: 'var(--text-3)' }">
              <span v-if="s.percentage != null">{{ s.percentage.toFixed(1) }}%</span>
              <span v-if="s.paper_year" class="ml-1">· {{ s.paper_year }}</span>
            </div>
          </div>

          <AppBadge
            :color="s.status === 'completed' ? 'success' : s.status === 'in_progress' ? 'warm' : 'muted'"
            size="xs"
          >
            {{ s.status.replace(/_/g, ' ') }}
          </AppBadge>
        </div>
      </AppCard>
    </div>
  </div>
</template>
