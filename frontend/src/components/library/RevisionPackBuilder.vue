<script setup lang="ts">
import { ref, computed } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppInput from '@/components/ui/AppInput.vue'

defineEmits<{ build: [config: any]; cancel: [] }>()

const packName = ref('')
const targetMinutes = ref(20)
const selectedItems = ref<any[]>([])
const useAiSuggestion = ref(true)

const estimatedQuestions = computed(() => Math.round(targetMinutes.value * 1.5))
</script>

<template>
  <div class="max-w-lg">
    <h2 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--text)' }">Build Revision Pack</h2>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-3)' }">Create a custom study pack focused on what you need.</p>

    <div class="space-y-4 mb-6">
      <AppInput v-model="packName" label="Pack Name" placeholder="e.g. Algebra Revision" />

      <div>
        <label class="block text-xs font-medium uppercase tracking-wide mb-2" :style="{ color: 'var(--text-3)' }">Target Duration</label>
        <div class="flex gap-2">
          <button v-for="mins in [10, 15, 20, 30, 45]" :key="mins"
            class="px-3 py-2 rounded-[var(--radius-md)] text-sm font-medium transition-all"
            :class="targetMinutes === mins ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)]'"
            @click="targetMinutes = mins">
            {{ mins }}m
          </button>
        </div>
      </div>

      <AppCard padding="md">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium" :style="{ color: 'var(--text)' }">AI-suggested content</p>
            <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">Auto-select based on your weak areas and review schedule</p>
          </div>
          <button class="px-3 py-1 rounded text-xs font-medium"
            :class="useAiSuggestion ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-3)]'"
            @click="useAiSuggestion = !useAiSuggestion">
            {{ useAiSuggestion ? 'On' : 'Off' }}
          </button>
        </div>
      </AppCard>
    </div>

    <AppCard padding="sm" class="mb-6">
      <div class="flex items-center justify-between text-xs" :style="{ color: 'var(--text-3)' }">
        <span>Estimated: ~{{ estimatedQuestions }} questions</span>
        <span>~{{ targetMinutes }} minutes</span>
      </div>
    </AppCard>

    <div class="flex gap-3">
      <AppButton variant="primary" :disabled="!packName.trim()"
        @click="$emit('build', { name: packName, targetMinutes, useAi: useAiSuggestion })">
        Build Pack →
      </AppButton>
      <AppButton variant="ghost" @click="$emit('cancel')">Cancel</AppButton>
    </div>
  </div>
</template>
