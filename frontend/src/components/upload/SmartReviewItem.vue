<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  questionNumber: number
  questionText: string
  studentAnswer: string
  isCorrect: boolean
  correctAnswer?: string
  diagnosis?: string
  topicName?: string
}>()

defineEmits<{ reflect: [reason: string] }>()

const reflectionReasons = [
  { key: 'guessed', label: 'I guessed' },
  { key: 'confused', label: 'I was confused' },
  { key: 'rushing', label: 'I was rushing' },
  { key: 'understood', label: 'I understood but made an error' },
]
</script>

<template>
  <AppCard padding="md">
    <div class="flex items-start gap-3">
      <div class="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-bold shrink-0"
        :class="isCorrect ? 'bg-emerald-100 text-emerald-700' : 'bg-red-100 text-red-600'">
        Q{{ questionNumber }}
      </div>
      <div class="flex-1">
        <p class="text-sm font-medium mb-1" :style="{ color: 'var(--text)' }">{{ questionText }}</p>

        <div class="flex items-center gap-2 mb-2">
          <span class="text-xs" :style="{ color: 'var(--text-3)' }">Your answer:</span>
          <span class="text-xs font-medium" :class="isCorrect ? 'text-emerald-600' : 'text-red-500'">{{ studentAnswer }}</span>
        </div>

        <div v-if="!isCorrect && correctAnswer" class="flex items-center gap-2 mb-2">
          <span class="text-xs" :style="{ color: 'var(--text-3)' }">Correct:</span>
          <span class="text-xs font-medium text-emerald-600">{{ correctAnswer }}</span>
        </div>

        <p v-if="diagnosis" class="text-xs mb-2" :style="{ color: 'var(--text-2)' }">{{ diagnosis }}</p>

        <AppBadge v-if="topicName" color="muted" size="xs">{{ topicName }}</AppBadge>

        <!-- Reflection chips (for wrong answers) -->
        <div v-if="!isCorrect" class="flex flex-wrap gap-1 mt-2">
          <button v-for="r in reflectionReasons" :key="r.key"
            class="px-2 py-0.5 rounded-full text-[10px] font-medium border transition-colors"
            :style="{ borderColor: 'var(--card-border)', color: 'var(--text-3)' }"
            @click="$emit('reflect', r.key)">
            {{ r.label }}
          </button>
        </div>
      </div>
    </div>
  </AppCard>
</template>
