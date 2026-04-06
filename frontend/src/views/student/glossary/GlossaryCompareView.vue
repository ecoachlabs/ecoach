<script setup lang="ts">
import { computed, ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import GlossaryCompare from '@/components/glossary/GlossaryCompare.vue'

type ComparePreset = {
  id: string
  title: string
  conceptA: { title: string; type: string; meaning: string; features: string[] }
  conceptB: { title: string; type: string; meaning: string; features: string[] }
  sharedFeatures: string[]
  confusionPoints: string[]
}

const presets: ComparePreset[] = [
  {
    id: 'speed-velocity',
    title: 'Speed vs Velocity',
    conceptA: {
      title: 'Speed',
      type: 'Scalar',
      meaning: 'How fast an object moves regardless of direction.',
      features: ['Magnitude only', 'Never negative in basic motion', 'Distance divided by time'],
    },
    conceptB: {
      title: 'Velocity',
      type: 'Vector',
      meaning: 'How fast an object moves with direction included.',
      features: ['Magnitude and direction', 'Can change if direction changes', 'Displacement divided by time'],
    },
    sharedFeatures: ['Both describe motion', 'Often measured in m/s', 'Can be calculated from distance or displacement over time'],
    confusionPoints: ['Students often ignore direction when velocity is asked for', 'Speed can stay constant while velocity changes'],
  },
  {
    id: 'mass-weight',
    title: 'Mass vs Weight',
    conceptA: {
      title: 'Mass',
      type: 'Property',
      meaning: 'The amount of matter in an object.',
      features: ['Measured in kilograms', 'Same everywhere', 'Related to inertia'],
    },
    conceptB: {
      title: 'Weight',
      type: 'Force',
      meaning: 'The gravitational force acting on an object.',
      features: ['Measured in newtons', 'Changes with gravity', 'Calculated with W = mg'],
    },
    sharedFeatures: ['Both describe physical objects', 'Both are used in mechanics questions', 'Often confused in everyday language'],
    confusionPoints: ['Kilograms are for mass, not weight', 'An object can keep the same mass while its weight changes'],
  },
]

const selectedId = ref(presets[0].id)

const selectedPreset = computed(() => presets.find(preset => preset.id === selectedId.value) ?? presets[0])
</script>

<template>
  <div class="p-6 lg:p-8 max-w-6xl mx-auto reveal-stagger">
    <div class="mb-6">
      <p class="text-[10px] font-bold uppercase tracking-[0.16em] mb-2" style="color: #10B981;">Glossary Tool</p>
      <h1 class="font-display text-2xl font-black tracking-tight mb-2" :style="{ color: 'var(--ink)' }">Compare Terms</h1>
      <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
        Put commonly confused ideas side by side so the difference becomes obvious instead of fuzzy.
      </p>
    </div>

    <div class="grid gap-6 lg:grid-cols-[260px_minmax(0,1fr)]">
      <AppCard padding="md">
        <p class="text-[10px] font-semibold uppercase tracking-[0.14em] mb-3" :style="{ color: 'var(--ink-muted)' }">Comparison Sets</p>
        <div class="space-y-2">
          <button
            v-for="preset in presets"
            :key="preset.id"
            class="w-full text-left rounded-2xl border px-4 py-3 transition-colors"
            :style="{
              borderColor: selectedId === preset.id ? '#10B981' : 'var(--border-soft)',
              backgroundColor: selectedId === preset.id ? 'rgba(16,185,129,0.08)' : 'var(--surface)',
            }"
            @click="selectedId = preset.id"
          >
            <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ preset.title }}</p>
          </button>
        </div>
      </AppCard>

      <AppCard padding="lg">
        <GlossaryCompare
          :concept-a="selectedPreset.conceptA"
          :concept-b="selectedPreset.conceptB"
          :shared-features="selectedPreset.sharedFeatures"
          :confusion-points="selectedPreset.confusionPoints"
        />

        <div class="mt-5 flex items-center gap-3">
          <AppButton variant="secondary" @click="$router.push('/student/glossary')">Back to Glossary</AppButton>
        </div>
      </AppCard>
    </div>
  </div>
</template>
