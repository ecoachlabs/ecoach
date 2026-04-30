<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import MathText from '@/components/question/MathText.vue'
import {
  listQuestionReviewQueue,
  reviewQuestionIntelligence,
  type QuestionReviewQueueItemDto,
} from '@/ipc/admin'

const router = useRouter()
const auth = useAuthStore()

const loading = ref(true)
const actionId = ref<number | null>(null)
const status = ref('')
const error = ref('')
const success = ref('')
const queue = ref<QuestionReviewQueueItemDto[]>([])

async function load() {
  loading.value = true
  error.value = ''
  try {
    queue.value = await listQuestionReviewQueue(status.value || null, 80)
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load review queue.'
  } finally {
    loading.value = false
  }
}

async function review(question: QuestionReviewQueueItemDto, action: 'approve' | 'reject' | 'send_for_reclassification') {
  actionId.value = question.question_id
  error.value = ''
  success.value = ''
  try {
    await reviewQuestionIntelligence(question.question_id, {
      reviewer_id: `admin:${auth.currentAccount?.id ?? 'local'}`,
      action_code: action,
      review_status: action === 'approve' ? 'approved' : action === 'reject' ? 'rejected' : 'needs_review',
      note: `Marked ${action} from review queue.`,
      primary_knowledge_role: null,
      primary_cognitive_demand: null,
      primary_solve_pattern: null,
      primary_pedagogic_function: null,
      primary_content_grain: null,
      family_id: question.family_candidate?.family_id ?? null,
      misconception_codes: question.misconception_candidates?.map((item: any) => item.misconception_code).filter(Boolean) ?? [],
      request_reclassification: action === 'send_for_reclassification',
    })
    success.value = `Question #${question.question_id} updated.`
    await load()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not update review state.'
  } finally {
    actionId.value = null
  }
}

onMounted(load)
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Review Queue</h1>
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Approve, reject, or reclassify low-confidence question intelligence.</p>
      </div>
      <div class="flex gap-2">
        <AppSelect
          v-model="status"
          :options="[
            { value: '', label: 'All pending' },
            { value: 'pending', label: 'Pending' },
            { value: 'needs_review', label: 'Needs Review' },
            { value: 'taxonomy_gap', label: 'Taxonomy Gap' },
            { value: 'family_unresolved', label: 'Family Unresolved' },
          ]"
        />
        <AppButton variant="secondary" size="sm" @click="load">Refresh</AppButton>
      </div>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    <p v-if="success" class="mb-4 text-sm" :style="{ color: 'var(--accent)' }">{{ success }}</p>

    <div v-if="loading" class="space-y-2">
      <div v-for="i in 8" :key="i" class="h-20 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--surface)' }" />
    </div>

    <div v-else class="space-y-3">
      <AppCard v-for="question in queue" :key="question.question_id" padding="md">
        <div class="flex items-start gap-4">
          <AppBadge color="muted" size="xs">#{{ question.question_id }}</AppBadge>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium mb-2" :style="{ color: 'var(--ink)' }">
              <MathText :text="question.stem" size="sm" />
            </p>
            <div class="flex flex-wrap gap-2">
              <AppBadge color="gold" size="xs">{{ question.review_status }}</AppBadge>
              <AppBadge color="muted" size="xs">confidence {{ question.machine_confidence_score }}</AppBadge>
              <AppBadge v-if="question.family_candidate" color="accent" size="xs">{{ question.family_candidate.family_name }}</AppBadge>
            </div>
            <p v-if="question.review_reason" class="text-xs mt-2" :style="{ color: 'var(--ink-muted)' }">{{ question.review_reason }}</p>
          </div>
          <div class="flex gap-2 shrink-0">
            <AppButton variant="ghost" size="sm" @click="router.push({ path: '/admin/questions/author', query: { id: question.question_id } })">Edit</AppButton>
            <AppButton variant="secondary" size="sm" :loading="actionId === question.question_id" @click="review(question, 'send_for_reclassification')">Reclassify</AppButton>
            <AppButton variant="secondary" size="sm" :loading="actionId === question.question_id" @click="review(question, 'reject')">Reject</AppButton>
            <AppButton variant="primary" size="sm" :loading="actionId === question.question_id" @click="review(question, 'approve')">Approve</AppButton>
          </div>
        </div>
      </AppCard>
      <AppCard v-if="!queue.length" padding="lg" class="text-center">
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No questions need review for this filter.</p>
      </AppCard>
    </div>
  </div>
</template>
