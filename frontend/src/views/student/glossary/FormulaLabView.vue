<script setup lang="ts">
import { computed, ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import FormulaLab from '@/components/glossary/FormulaLab.vue'

type FormulaPreset = {
  id: string
  name: string
  formula: string
  variables: { symbol: string; meaning: string; unit: string }[]
}

const presets: FormulaPreset[] = [
  {
    id: 'speed',
    name: 'Speed',
    formula: 'v = \\frac{d}{t}',
    variables: [
      { symbol: 'd', meaning: 'Distance travelled', unit: 'meters' },
      { symbol: 't', meaning: 'Time taken', unit: 'seconds' },
    ],
  },
  {
    id: 'density',
    name: 'Density',
    formula: '\\rho = \\frac{m}{V}',
    variables: [
      { symbol: 'm', meaning: 'Mass', unit: 'kilograms' },
      { symbol: 'V', meaning: 'Volume', unit: 'm^3' },
    ],
  },
  {
    id: 'pressure',
    name: 'Pressure',
    formula: 'P = \\frac{F}{A}',
    variables: [
      { symbol: 'F', meaning: 'Force', unit: 'newtons' },
      { symbol: 'A', meaning: 'Area', unit: 'm^2' },
    ],
  },
]

const selectedId = ref(presets[0].id)

const selectedPreset = computed(() => presets.find(preset => preset.id === selectedId.value) ?? presets[0])
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="mb-6">
      <p class="text-[10px] font-bold uppercase tracking-[0.16em] mb-2" style="color: #3B82F6;">Glossary Tool</p>
      <h1 class="font-display text-2xl font-black tracking-tight mb-2" :style="{ color: 'var(--ink)' }">Formula Lab</h1>
      <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
        Work through important formulas without leaving the glossary. Pick a formula set and plug in the values you know.
      </p>
    </div>

    <div class="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
      <AppCard padding="md">
        <p class="text-[10px] font-semibold uppercase tracking-[0.14em] mb-3" :style="{ color: 'var(--ink-muted)' }">Quick Picks</p>
        <div class="space-y-2">
          <button
            v-for="preset in presets"
            :key="preset.id"
            class="w-full text-left rounded-2xl border px-4 py-3 transition-colors"
            :style="{
              borderColor: selectedId === preset.id ? '#3B82F6' : 'var(--border-soft)',
              backgroundColor: selectedId === preset.id ? 'rgba(59,130,246,0.08)' : 'var(--surface)',
            }"
            @click="selectedId = preset.id"
          >
            <p class="text-sm font-semibold mb-1" :style="{ color: 'var(--ink)' }">{{ preset.name }}</p>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ preset.formula }}</p>
          </button>
        </div>
      </AppCard>

      <AppCard padding="lg">
        <FormulaLab
          :formula-name="selectedPreset.name"
          :formula="selectedPreset.formula"
          :variables="selectedPreset.variables"
        />

        <div class="mt-5 flex items-center gap-3">
          <AppButton variant="secondary" @click="$router.push('/student/glossary')">Back to Glossary</AppButton>
        </div>
      </AppCard>
    </div>
  </div>
</template>
