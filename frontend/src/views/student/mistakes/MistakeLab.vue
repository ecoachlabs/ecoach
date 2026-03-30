<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getLearnerTruth, type LearnerTruthDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppTabs from '@/components/ui/AppTabs.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const truth = ref<LearnerTruthDto | null>(null)
const activeTab = ref('recent')

onMounted(async () => {
  if (!auth.currentAccount) return
  try { truth.value = await getLearnerTruth(auth.currentAccount.id) } catch {}
  loading.value = false
})

const tabs = [
  { key: 'recent', label: 'Recent', count: 12 },
  { key: 'patterns', label: 'Patterns', count: 5 },
  { key: 'by-type', label: 'By Type', count: 8 },
  { key: 'by-topic', label: 'By Topic', count: 6 },
]

const errorTypes = [
  { type: 'knowledge_gap', label: 'Knowledge Gap', count: 4, color: 'var(--err-knowledge)' },
  { type: 'conceptual_confusion', label: 'Conceptual Confusion', count: 3, color: 'var(--err-conceptual)' },
  { type: 'carelessness', label: 'Careless Error', count: 3, color: 'var(--err-careless)' },
  { type: 'pressure_breakdown', label: 'Pressure Breakdown', count: 2, color: 'var(--err-pressure)' },
]
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <div class="flex items-start justify-between mb-6">
      <div>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">Mistake Lab</h1>
        <p class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">Every mistake is diagnostic data. Understand your patterns to break them.</p>
      </div>
      <AppBadge color="danger" size="md">{{ truth?.diagnosis_count ?? 12 }} recent errors</AppBadge>
    </div>

    <AppTabs :tabs="tabs" v-model="activeTab" class="mb-6" />

    <!-- Error Type Breakdown -->
    <div v-if="activeTab === 'by-type'" class="space-y-3">
      <AppCard v-for="err in errorTypes" :key="err.type" padding="md" hover>
        <div class="flex items-center gap-3">
          <div class="w-3 h-3 rounded-full" :style="{ backgroundColor: err.color }" />
          <div class="flex-1">
            <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">{{ err.label }}</p>
          </div>
          <span class="text-sm font-bold tabular-nums" :style="{ color: err.color }">{{ err.count }}</span>
          <AppButton variant="ghost" size="sm">Fix →</AppButton>
        </div>
      </AppCard>
    </div>

    <!-- Recent Mistakes (default) -->
    <div v-else>
      <AppCard padding="lg" class="text-center">
        <div class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
          :style="{ backgroundColor: 'var(--danger-light)', color: 'var(--danger)' }">✕</div>
        <h3 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">Your Error Patterns</h3>
        <p class="text-sm mb-4" :style="{ color: 'var(--text-3)' }">Practice more to see detailed error analysis here.</p>
        <AppButton variant="primary" @click="router.push('/student/practice')">Start Practice</AppButton>
      </AppCard>
    </div>
  </div>
</template>
