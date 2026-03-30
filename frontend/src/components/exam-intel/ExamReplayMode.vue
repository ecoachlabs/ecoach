<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  papers: { year: string; subject: string; questionCount: number; duration: number; yourBestScore?: number }[]
}>()

defineEmits<{ start: [year: string] }>()
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Sit a Real Past Paper</h3>
    <p class="text-xs mb-4" :style="{color:'var(--text-3)'}">Full exam conditions. Timer. No help. Just like the real thing.</p>
    <div class="space-y-2">
      <AppCard v-for="p in papers" :key="p.year" hover padding="md" @click="$emit('start', p.year)">
        <div class="flex items-center gap-3">
          <div class="w-12 h-12 rounded-xl flex items-center justify-center text-base font-display font-bold"
            :style="{backgroundColor:'var(--primary-light)',color:'var(--primary)'}">
            {{ p.year.slice(-2) }}
          </div>
          <div class="flex-1">
            <p class="text-sm font-semibold" :style="{color:'var(--text)'}">BECE {{ p.year }}</p>
            <p class="text-[10px]" :style="{color:'var(--text-3)'}">{{ p.subject }} · {{ p.questionCount }} questions · {{ p.duration }} min</p>
          </div>
          <div v-if="p.yourBestScore !== undefined" class="text-right">
            <p class="text-xs font-bold tabular-nums" :class="p.yourBestScore >= 70 ? 'text-emerald-600' : p.yourBestScore >= 40 ? 'text-amber-600' : 'text-red-500'">
              {{ p.yourBestScore }}%
            </p>
            <p class="text-[8px]" :style="{color:'var(--text-3)'}">Best</p>
          </div>
          <AppButton variant="secondary" size="sm">Start →</AppButton>
        </div>
      </AppCard>
    </div>
  </div>
</template>
