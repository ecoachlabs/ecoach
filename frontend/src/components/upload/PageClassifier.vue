<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  pages: { id: number; thumbnail?: string; filename: string }[]
}>()

defineEmits<{ classify: [classifications: Record<number, string>] }>()

const classifications = ref<Record<number, string>>({})

const types = [
  { key: 'question', label: 'Question', color: 'accent' },
  { key: 'answer', label: 'Answer', color: 'success' },
  { key: 'marked', label: 'Marked', color: 'gold' },
  { key: 'other', label: 'Other', color: 'muted' },
]

function classify(pageId: number, type: string) {
  classifications.value[pageId] = type
}
</script>

<template>
  <div>
    <p class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">
      Classify each page
    </p>
    <div class="space-y-3">
      <AppCard v-for="page in pages" :key="page.id" padding="sm">
        <div class="flex items-center gap-3">
          <div class="w-12 h-16 rounded bg-stone-100 flex items-center justify-center text-xs" :style="{ color: 'var(--text-3)' }">
            {{ page.thumbnail ? '' : '📄' }}
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate" :style="{ color: 'var(--text)' }">{{ page.filename }}</p>
            <div class="flex gap-1 mt-1.5">
              <button v-for="t in types" :key="t.key"
                class="px-2 py-0.5 rounded text-[10px] font-medium border transition-all"
                :class="classifications[page.id] === t.key ? 'text-white border-transparent' : 'border-[var(--card-border)] text-[var(--text-3)]'"
                :style="classifications[page.id] === t.key ? { backgroundColor: `var(--${t.color})` } : {}"
                @click="classify(page.id, t.key)">
                {{ t.label }}
              </button>
            </div>
          </div>
        </div>
      </AppCard>
    </div>
    <div class="mt-4">
      <AppButton variant="primary" :disabled="Object.keys(classifications).length < pages.length"
        @click="$emit('classify', classifications)">Continue →</AppButton>
    </div>
  </div>
</template>
