<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import MathRenderer from '@/components/question/MathRenderer.vue'

defineProps<{
  formulaName: string
  formula: string
  variables: { symbol: string; meaning: string; unit: string }[]
}>()

const variableValues = ref<Record<string, string>>({})
const result = ref<string | null>(null)

function calculate() {
  result.value = 'Result will be calculated based on the formula engine.'
}
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Formula Lab: {{ formulaName }}</h3>

    <!-- Formula display -->
    <AppCard padding="lg" class="text-center mb-4">
      <MathRenderer :expression="formula" display size="lg" />
    </AppCard>

    <!-- Variable inputs -->
    <div class="grid grid-cols-2 gap-3 mb-4">
      <div v-for="v in variables" :key="v.symbol">
        <AppInput v-model="variableValues[v.symbol]" :label="v.symbol + ' — ' + v.meaning" :placeholder="v.unit" />
      </div>
    </div>

    <!-- Calculate -->
    <div class="flex items-center gap-3 mb-4">
      <AppButton variant="primary" @click="calculate">Calculate</AppButton>
      <AppButton variant="ghost" size="sm" @click="variableValues = {}; result = null">Clear</AppButton>
    </div>

    <!-- Result -->
    <AppCard v-if="result" padding="md" glow="accent">
      <p class="text-xs font-semibold uppercase mb-1" :style="{color:'var(--accent)'}">Result</p>
      <p class="text-lg font-display font-bold" :style="{color:'var(--text)'}">{{ result }}</p>
    </AppCard>
  </div>
</template>
