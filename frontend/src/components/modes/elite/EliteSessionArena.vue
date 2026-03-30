<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  selectedSession?: string
}>()

defineEmits<{ select: [sessionType: string]; start: [] }>()

const sessionTypes = [
  { key: 'precision', label: 'Precision Lab', desc: 'Accuracy-focused. Steady timer. Verification emphasis.', icon: '◎', difficulty: 'Demanding' },
  { key: 'sprint', label: 'Elite Sprint', desc: 'Speed under pressure. Shrinking timer. Method efficiency.', icon: '⚡', difficulty: 'Intense' },
  { key: 'depth', label: 'Depth Lab', desc: 'Multi-step application problems. Deep reasoning.', icon: '◈', difficulty: 'Complex' },
  { key: 'trapsense', label: 'TrapSense', desc: 'Distractor-heavy questions. Trap detection skills.', icon: '⚠', difficulty: 'Tricky' },
  { key: 'endurance', label: 'Endurance Track', desc: '30+ questions. Consistency under fatigue.', icon: '∞', difficulty: 'Long' },
  { key: 'perfect', label: 'Perfect Run', desc: 'One error breaks the streak. Maximum tension.', icon: '★', difficulty: 'Unforgiving' },
  { key: 'apex', label: 'Apex Mock', desc: 'Full exam simulation at elite difficulty.', icon: '◉', difficulty: 'Ultimate' },
]
</script>

<template>
  <div>
    <h2 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Select Challenge</h2>
    <div class="grid grid-cols-2 lg:grid-cols-3 gap-3 mb-6">
      <AppCard v-for="st in sessionTypes" :key="st.key" hover padding="md"
        :class="selectedSession === st.key ? 'ring-2 ring-[var(--primary)]' : ''"
        @click="$emit('select', st.key)">
        <div class="flex items-start gap-2.5">
          <div class="w-9 h-9 rounded-lg flex items-center justify-center text-base shrink-0"
            :style="{ backgroundColor: 'var(--primary-light)', color: 'var(--primary)' }">{{ st.icon }}</div>
          <div class="min-w-0">
            <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ st.label }}</p>
            <p class="text-[10px] mt-0.5" :style="{ color: 'var(--text-3)' }">{{ st.desc }}</p>
            <AppBadge color="muted" size="xs" class="mt-1.5">{{ st.difficulty }}</AppBadge>
          </div>
        </div>
      </AppCard>
    </div>
    <AppButton v-if="selectedSession" variant="primary" size="lg" @click="$emit('start')">
      Begin {{ sessionTypes.find(s => s.key === selectedSession)?.label }} →
    </AppButton>
  </div>
</template>
