<script setup lang="ts">
import MathRenderer from '@/components/question/MathRenderer.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  formula: string
  name: string
  variables?: { symbol: string; meaning: string; unit?: string }[]
  whenToUse?: string
  whenNotToUse?: string
}>()
</script>

<template>
  <div>
    <!-- Formula display -->
    <AppCard padding="lg" class="text-center mb-4">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">{{ name }}</h3>
      <div class="py-4 px-6 rounded-[var(--radius-md)] inline-block" :style="{ backgroundColor: 'var(--primary-light)' }">
        <MathRenderer :expression="formula" display size="lg" />
      </div>
    </AppCard>

    <!-- Variables -->
    <div v-if="variables?.length" class="mb-4">
      <h4 class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Variables</h4>
      <div class="space-y-1.5">
        <div v-for="v in variables" :key="v.symbol" class="flex items-center gap-3">
          <span class="w-8 h-8 rounded-lg flex items-center justify-center font-mono font-bold text-sm"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">{{ v.symbol }}</span>
          <div class="flex-1">
            <p class="text-sm" :style="{ color: 'var(--text)' }">{{ v.meaning }}</p>
            <p v-if="v.unit" class="text-[10px]" :style="{ color: 'var(--text-3)' }">Unit: {{ v.unit }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- When to use -->
    <div class="grid grid-cols-2 gap-3">
      <AppCard v-if="whenToUse" padding="sm">
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--success)' }">When to use</p>
        <p class="text-xs" :style="{ color: 'var(--text-2)' }">{{ whenToUse }}</p>
      </AppCard>
      <AppCard v-if="whenNotToUse" padding="sm">
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-1" :style="{ color: 'var(--danger)' }">When NOT to use</p>
        <p class="text-xs" :style="{ color: 'var(--text-2)' }">{{ whenNotToUse }}</p>
      </AppCard>
    </div>
  </div>
</template>
