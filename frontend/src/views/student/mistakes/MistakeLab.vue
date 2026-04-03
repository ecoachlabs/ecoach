<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getRevengeQueue, type RevengeQueueItemDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppTabs from '@/components/ui/AppTabs.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const mistakes = ref<RevengeQueueItemDto[]>([])
const activeTab = ref('pending')

const tabs = [
  { key: 'pending', label: 'Pending' },
  { key: 'beaten', label: 'Beaten' },
  { key: 'patterns', label: 'Patterns' },
]

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    mistakes.value = await getRevengeQueue(auth.currentAccount.id)
  } catch (e) {
    console.error('Failed to load mistake queue:', e)
  }
  loading.value = false
})

const pending = computed(() => mistakes.value.filter(m => !m.is_beaten))
const beaten = computed(() => mistakes.value.filter(m => m.is_beaten))

// Group by error type for patterns tab
const errorPatterns = computed(() => {
  const counts: Record<string, number> = {}
  for (const m of mistakes.value) {
    const type = m.original_error_type ?? 'unknown'
    counts[type] = (counts[type] ?? 0) + 1
  }
  return Object.entries(counts).map(([type, count]) => ({ type, count }))
    .sort((a, b) => b.count - a.count)
})

function errorTypeLabel(type: string): string {
  return type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

function errorTypeColor(type: string): string {
  if (type.includes('knowledge')) return 'var(--danger)'
  if (type.includes('conceptual')) return 'var(--accent)'
  if (type.includes('careless')) return 'var(--gold)'
  if (type.includes('pressure')) return 'var(--warm)'
  return 'var(--text-3)'
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <div class="flex items-start justify-between mb-6">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Mistake Lab</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Every mistake is diagnostic data. Understand your patterns to break them.</p>
      </div>
      <AppBadge color="danger" size="md">{{ pending.length }} pending</AppBadge>
    </div>

    <AppTabs :tabs="tabs" v-model="activeTab" class="mb-6" />

    <!-- Loading -->
    <div v-if="loading" class="space-y-3">
      <div v-for="i in 4" :key="i" class="h-16 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Pending mistakes -->
    <div v-else-if="activeTab === 'pending'">
      <div v-if="pending.length" class="space-y-3">
        <AppCard v-for="m in pending" :key="m.id" padding="md" hover>
          <div class="flex items-start gap-3">
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0"
              :style="{ backgroundColor: 'var(--danger-light)', color: 'var(--danger)' }">✕</div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium line-clamp-2" :style="{ color: 'var(--text)' }">
                {{ m.question_text ?? 'Question #' + m.question_id }}
              </p>
              <p v-if="m.original_error_type" class="text-[11px] mt-1" :style="{ color: 'var(--text-3)' }">
                {{ errorTypeLabel(m.original_error_type) }}
              </p>
            </div>
            <AppBadge color="muted" size="xs">{{ m.attempts_to_beat }}x</AppBadge>
          </div>
        </AppCard>
      </div>
      <AppCard v-else padding="lg" class="text-center">
        <div class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
          :style="{ backgroundColor: 'var(--success-light)', color: 'var(--success)' }">✓</div>
        <h3 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">No pending mistakes</h3>
        <p class="text-sm mb-4" :style="{ color: 'var(--text-3)' }">Practice more to track error patterns here.</p>
        <AppButton variant="primary" @click="router.push('/student/practice')">Start Practice</AppButton>
      </AppCard>
    </div>

    <!-- Beaten mistakes -->
    <div v-else-if="activeTab === 'beaten'">
      <div v-if="beaten.length" class="space-y-3">
        <AppCard v-for="m in beaten" :key="m.id" padding="md">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-sm shrink-0"
              :style="{ backgroundColor: 'var(--success-light)', color: 'var(--success)' }">✓</div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium line-clamp-1" :style="{ color: 'var(--text)' }">
                {{ m.question_text ?? 'Question #' + m.question_id }}
              </p>
            </div>
            <AppBadge color="success" size="xs">beaten</AppBadge>
          </div>
        </AppCard>
      </div>
      <AppCard v-else padding="lg" class="text-center">
        <p class="text-sm" :style="{ color: 'var(--text-3)' }">No beaten mistakes yet. Keep practising!</p>
      </AppCard>
    </div>

    <!-- Error Patterns -->
    <div v-else-if="activeTab === 'patterns'">
      <div v-if="errorPatterns.length" class="space-y-3">
        <AppCard v-for="ep in errorPatterns" :key="ep.type" padding="md" hover>
          <div class="flex items-center gap-3">
            <div class="w-3 h-3 rounded-full shrink-0" :style="{ backgroundColor: errorTypeColor(ep.type) }" />
            <div class="flex-1">
              <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ errorTypeLabel(ep.type) }}</p>
            </div>
            <span class="text-sm font-bold tabular-nums" :style="{ color: errorTypeColor(ep.type) }">{{ ep.count }}</span>
            <AppButton variant="ghost" size="sm" @click="router.push('/student/practice')">Fix →</AppButton>
          </div>
        </AppCard>
      </div>
      <AppCard v-else padding="lg" class="text-center">
        <p class="text-sm" :style="{ color: 'var(--text-3)' }">No error patterns yet. Your patterns will appear after you practice.</p>
      </AppCard>
    </div>
  </div>
</template>
