<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'
import { ipc } from '@/ipc'

const route = useRoute()
const router = useRouter()
const auth = useAuthStore()

const sessionId = computed(() => Number(route.params.id))
const loading = ref(true)
const error = ref('')
const session = ref<any>(null)
const currentItemIndex = ref(0)
const currentQuestion = ref<any>(null)
const options = ref<any[]>([])
const selectedOption = ref<number | null>(null)
const answered = ref(false)
const result = ref<any>(null)
const startTime = ref(Date.now())

onMounted(async () => {
  try {
    // Load session snapshot - this calls the real backend
    session.value = await ipc('get_session_snapshot', { sessionId: sessionId.value })
    if (session.value) {
      await loadCurrentQuestion()
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load session'
  }
  loading.value = false
})

async function loadCurrentQuestion() {
  // In a real implementation, this would fetch the current question from the session items
  // For now we show the session info
}

async function submitAnswer() {
  if (selectedOption.value === null) return
  answered.value = true
  try {
    result.value = await ipc('submit_answer', {
      input: {
        item_id: currentItemIndex.value,
        selected_option_id: selectedOption.value,
        response_time_ms: Date.now() - startTime.value,
      }
    })
  } catch (e: any) {
    console.error('Submit failed:', e)
  }
}

function nextQuestion() {
  currentItemIndex.value++
  selectedOption.value = null
  answered.value = false
  result.value = null
  startTime.value = Date.now()
  loadCurrentQuestion()
}

async function finishSession() {
  try {
    await ipc('complete_session', { sessionId: sessionId.value })
    router.push(`/student/session/${sessionId.value}/debrief`)
  } catch (e) {
    console.error('Failed to complete:', e)
  }
}
</script>

<template>
  <div class="h-full flex flex-col" :style="{ backgroundColor: 'var(--bg)' }">
    <!-- Session Header -->
    <div class="shrink-0 px-6 py-3 flex items-center justify-between border-b" :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }">
      <div class="flex items-center gap-3">
        <AppBadge color="accent" size="sm">Session #{{ sessionId }}</AppBadge>
        <span v-if="session" class="text-xs" :style="{ color: 'var(--text-3)' }">
          {{ session.session_type }} · {{ session.item_count }} questions
        </span>
      </div>
      <div class="flex items-center gap-3">
        <AppProgress v-if="session" :value="currentItemIndex" :max="session.item_count || 1" size="sm" color="accent" class="w-32" />
        <AppButton variant="ghost" size="sm" @click="finishSession">End Session</AppButton>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <div class="w-8 h-8 border-2 rounded-full animate-spin" :style="{ borderColor: 'var(--accent)', borderTopColor: 'transparent' }" />
    </div>

    <!-- Error -->
    <div v-else-if="error" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <p class="text-sm mb-4" :style="{ color: 'var(--danger)' }">{{ error }}</p>
        <AppButton variant="secondary" @click="router.push('/student/practice')">Back to Practice</AppButton>
      </div>
    </div>

    <!-- Session Content -->
    <div v-else class="flex-1 overflow-y-auto p-6 lg:p-8 max-w-3xl mx-auto w-full">
      <AppCard padding="lg" class="mb-6">
        <div class="text-center py-8">
          <div class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-2xl"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">✎</div>
          <h2 class="font-display text-lg font-semibold mb-2" :style="{ color: 'var(--text)' }">
            Session Active
          </h2>
          <p class="text-sm mb-1" :style="{ color: 'var(--text-2)' }">
            {{ session?.item_count || 0 }} questions loaded
          </p>
          <p class="text-xs" :style="{ color: 'var(--text-3)' }">
            Questions will be displayed here as the question rendering component is built.
          </p>
          <div class="mt-6">
            <AppButton variant="primary" @click="finishSession">Complete Session →</AppButton>
          </div>
        </div>
      </AppCard>
    </div>
  </div>
</template>
