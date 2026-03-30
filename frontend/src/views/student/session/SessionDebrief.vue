<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { completeSession } from '@/ipc/sessions'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

const route = useRoute()
const router = useRouter()
const sessionId = computed(() => Number(route.params.id))
const loading = ref(true)
const summary = ref<any>(null)

onMounted(async () => {
  try {
    summary.value = await completeSession(sessionId.value)
  } catch (e) {
    console.error('Failed to load summary:', e)
  }
  loading.value = false
})
</script>

<template>
  <div class="p-6 lg:p-8 max-w-3xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Session Complete</h1>

    <div v-if="loading" class="h-64 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />

    <template v-else-if="summary">
      <!-- Score Card -->
      <AppCard padding="lg" glow="accent" class="mb-6">
        <div class="flex items-center gap-6">
          <ProgressRing
            :value="summary.accuracy_score || 0"
            :max="10000"
            :size="80"
            :stroke-width="5"
            :color="(summary.accuracy_score || 0) >= 7000 ? 'var(--success)' : (summary.accuracy_score || 0) >= 4000 ? 'var(--gold)' : 'var(--danger)'"
            label="Accuracy"
          />
          <div>
            <h2 class="font-display text-xl font-semibold mb-1" :style="{ color: 'var(--text)' }">
              {{ summary.correct_questions }}/{{ summary.answered_questions }} correct
            </h2>
            <p class="text-sm" :style="{ color: 'var(--text-2)' }">
              Session #{{ summary.session_id }}
            </p>
          </div>
        </div>
      </AppCard>

      <!-- Stats -->
      <div class="grid grid-cols-3 gap-3 mb-6">
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--accent)' }">{{ summary.answered_questions }}</p>
          <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Answered</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--success)' }">{{ summary.correct_questions }}</p>
          <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Correct</p>
        </AppCard>
        <AppCard padding="md" class="text-center">
          <p class="font-display text-2xl font-bold" :style="{ color: 'var(--danger)' }">{{ summary.answered_questions - summary.correct_questions }}</p>
          <p class="text-[10px] font-medium mt-1 uppercase" :style="{ color: 'var(--text-3)' }">Wrong</p>
        </AppCard>
      </div>
    </template>

    <!-- Actions -->
    <div class="flex items-center gap-3">
      <AppButton variant="primary" @click="router.push('/student')">Back to Home</AppButton>
      <AppButton variant="secondary" @click="router.push('/student/practice')">Practice Again</AppButton>
      <AppButton variant="ghost" size="sm" @click="router.push('/student/mistakes')">Review Mistakes</AppButton>
    </div>
  </div>
</template>
