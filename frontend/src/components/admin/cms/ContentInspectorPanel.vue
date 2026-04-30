<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CmsStatusBadge from './CmsStatusBadge.vue'
import type { AdminQuestionListItemDto } from '@/ipc/admin'

const props = defineProps<{
  question: AdminQuestionListItemDto | null
  accuracy: number | null
}>()

defineEmits<{
  edit: [question: AdminQuestionListItemDto]
  seed: [question: AdminQuestionListItemDto]
  archive: [question: AdminQuestionListItemDto]
  restore: [question: AdminQuestionListItemDto]
}>()

function displayAccuracy() {
  if (props.accuracy === null) return 'No attempts'
  return `${props.accuracy}% accuracy`
}
</script>

<template>
  <AppCard padding="md" class="sticky top-4">
    <div v-if="!question">
      <h2 class="text-sm font-bold mb-2" :style="{ color: 'var(--ink)' }">Question Inspector</h2>
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Select a question to inspect its content, provenance, and learning signals.</p>
    </div>

    <div v-else>
      <div class="flex items-center gap-2 mb-3">
        <CmsStatusBadge :status="question.is_active ? question.review_status : 'archived'" />
        <span class="text-xs" :style="{ color: 'var(--ink-muted)' }">Question #{{ question.question_id }}</span>
      </div>

      <h2 class="text-base font-bold mb-3 leading-snug" :style="{ color: 'var(--ink)' }">{{ question.stem }}</h2>

      <div class="grid grid-cols-2 gap-3 mb-4 text-xs">
        <div>
          <p class="font-semibold" :style="{ color: 'var(--ink)' }">{{ question.subject_name }}</p>
          <p :style="{ color: 'var(--ink-muted)' }">Subject</p>
        </div>
        <div>
          <p class="font-semibold" :style="{ color: 'var(--ink)' }">{{ question.topic_name }}</p>
          <p :style="{ color: 'var(--ink-muted)' }">Topic</p>
        </div>
        <div>
          <p class="font-semibold" :style="{ color: 'var(--ink)' }">{{ question.question_format }}</p>
          <p :style="{ color: 'var(--ink-muted)' }">Format</p>
        </div>
        <div>
          <p class="font-semibold" :style="{ color: 'var(--ink)' }">{{ question.source_type }}</p>
          <p :style="{ color: 'var(--ink-muted)' }">Source</p>
        </div>
      </div>

      <div class="rounded-lg p-3 mb-4" :style="{ backgroundColor: 'var(--paper)' }">
        <p class="text-xs font-semibold mb-1" :style="{ color: 'var(--ink)' }">Usage</p>
        <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
          {{ question.attempt_count }} attempts &middot; {{ displayAccuracy() }} &middot; {{ question.option_count }} answers
        </p>
      </div>

      <div v-if="question.is_active" class="flex gap-2">
        <AppButton variant="primary" size="sm" @click="$emit('edit', question)">Edit in Content Editor</AppButton>
        <AppButton variant="secondary" size="sm" @click="$emit('seed', question)">Seed Similar</AppButton>
        <AppButton variant="danger" size="sm" @click="$emit('archive', question)">Archive</AppButton>
      </div>
      <div v-else class="flex gap-2">
        <AppButton variant="primary" size="sm" @click="$emit('restore', question)">Restore To Bank</AppButton>
      </div>
    </div>
  </AppCard>
</template>
