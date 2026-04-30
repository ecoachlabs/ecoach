<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'
import MathRenderer from '@/components/question/MathRenderer.vue'

const props = defineProps<{
  formulaName: string
  formula: string
  variables: { symbol: string; meaning: string; unit: string }[]
}>()

const variableValues = ref<Record<string, string>>({})
const result = ref<string | null>(null)
const errorMessage = ref<string | null>(null)

function calculate() {
  errorMessage.value = null
  result.value = null

  const equalsIndex = props.formula.indexOf('=')
  const resultLabel = equalsIndex >= 0 ? props.formula.slice(0, equalsIndex).trim() : props.formulaName
  let expression = equalsIndex >= 0 ? props.formula.slice(equalsIndex + 1).trim() : props.formula

  for (const variable of props.variables) {
    const rawValue = variableValues.value[variable.symbol]?.trim()
    if (!rawValue) {
      errorMessage.value = `Enter ${variable.meaning.toLowerCase()}.`
      return
    }

    const numericValue = Number(rawValue)
    if (!Number.isFinite(numericValue)) {
      errorMessage.value = `${variable.meaning} must be a number.`
      return
    }

    const keys = [variable.symbol, variable.meaning.toLowerCase()]
      .filter(Boolean)
      .sort((left, right) => right.length - left.length)

    for (const key of keys) {
      const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
      expression = expression.replace(new RegExp(`\\b${escaped}\\b`, 'gi'), rawValue)
    }
  }

  const safeExpression = expression.replace(/\^/g, '**').replace(/\s+/g, ' ').trim()
  if (!/^[0-9+\-*/().\s*]+$/.test(safeExpression)) {
    errorMessage.value = 'This formula needs a guided rearrangement before it can be evaluated here.'
    return
  }

  try {
    const evaluated = Function(`"use strict"; return (${safeExpression});`)()
    if (!Number.isFinite(evaluated)) {
      errorMessage.value = 'The calculation did not produce a valid result.'
      return
    }

    result.value = `${resultLabel} = ${Number(evaluated).toFixed(3).replace(/\.?0+$/, '')}`
  } catch {
    errorMessage.value = 'I could not evaluate that formula with the current values.'
  }
}
</script>

<template>
  <div>
    <h3 class="mb-3 text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--text-3)' }">
      Formula Lab: {{ formulaName }}
    </h3>

    <AppCard padding="lg" class="mb-4 text-center">
      <MathRenderer :expression="formula" display size="lg" />
    </AppCard>

    <div class="mb-4 grid grid-cols-2 gap-3">
      <div v-for="variable in variables" :key="variable.symbol">
        <AppInput
          v-model="variableValues[variable.symbol]"
          type="number"
          :label="`${variable.symbol} - ${variable.meaning}`"
          :placeholder="variable.unit"
        />
      </div>
    </div>

    <div class="mb-4 flex items-center gap-3">
      <AppButton variant="primary" @click="calculate">Calculate</AppButton>
      <AppButton variant="ghost" size="sm" @click="variableValues = {}; result = null; errorMessage = null">Clear</AppButton>
    </div>

    <p v-if="errorMessage" class="mb-4 text-xs font-medium text-[var(--danger)]">{{ errorMessage }}</p>

    <AppCard v-if="result" padding="md" glow="accent">
      <p class="mb-1 text-xs font-semibold uppercase" :style="{ color: 'var(--accent)' }">Result</p>
      <p class="text-lg font-display font-bold" :style="{ color: 'var(--text)' }">{{ result }}</p>
    </AppCard>
  </div>
</template>
