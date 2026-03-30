<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppAvatar from '@/components/ui/AppAvatar.vue'
import ProgressRing from '@/components/viz/ProgressRing.vue'

defineProps<{
  studentId: number
  studentName: string
  readinessBand: string
  examTarget?: string
  risks: { severity: string; title: string }[]
  recommendations: string[]
  subjectCount?: number
}>()

defineEmits<{ click: [] }>()

const bandColors: Record<string, string> = {
  strong: 'success', developing: 'gold', weak: 'danger', critical: 'danger',
}
</script>

<template>
  <AppCard hover padding="lg" @click="$emit('click')">
    <div class="flex items-start gap-4">
      <AppAvatar :name="studentName" size="lg" />
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2 mb-1">
          <h3 class="text-base font-semibold" :style="{ color: 'var(--text)' }">{{ studentName }}</h3>
          <AppBadge :color="(bandColors[readinessBand] as any) || 'muted'" size="sm">
            {{ readinessBand }}
          </AppBadge>
        </div>
        <p v-if="examTarget" class="text-xs mb-3" :style="{ color: 'var(--text-3)' }">{{ examTarget }}</p>

        <!-- Risks -->
        <div v-if="risks.length" class="space-y-1 mb-3">
          <div v-for="(risk, i) in risks.slice(0, 2)" :key="i"
            class="text-xs px-2.5 py-1.5 rounded-[var(--radius-sm)]"
            :class="risk.severity === 'high' ? 'bg-red-50 text-red-700' : risk.severity === 'medium' ? 'bg-amber-50 text-amber-700' : 'bg-blue-50 text-blue-700'">
            {{ risk.title }}
          </div>
        </div>

        <!-- First recommendation -->
        <p v-if="recommendations.length" class="text-xs" :style="{ color: 'var(--text-2)' }">
          {{ recommendations[0] }}
        </p>
      </div>
    </div>
    <div class="mt-3 text-right">
      <span class="text-xs font-medium" :style="{ color: 'var(--accent)' }">View details →</span>
    </div>
  </AppCard>
</template>
