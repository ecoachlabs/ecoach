<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import MathRenderer from '@/components/question/MathRenderer.vue'

defineProps<{
  title: string
  steps: { explanation: string; math?: string; tip?: string }[]
  difficulty?: string
}>()

const revealedSteps = ref(1)

function revealNext() {
  revealedSteps.value++
}

function revealAll() {
  revealedSteps.value = Infinity
}
</script>

<template>
  <AppCard padding="lg">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-sm font-semibold" :style="{color:'var(--text)'}">📋 {{ title }}</h3>
      <AppButton variant="ghost" size="sm" @click="revealAll">Show All</AppButton>
    </div>

    <div class="space-y-3">
      <div v-for="(step, i) in steps" :key="i"
        :class="i < revealedSteps ? 'reveal' : 'opacity-0 h-0 overflow-hidden'">
        <div class="flex gap-3">
          <span class="w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold shrink-0 mt-0.5"
            :style="{backgroundColor:'var(--accent-light)',color:'var(--accent)'}">{{ i + 1 }}</span>
          <div class="flex-1">
            <p class="text-sm leading-relaxed" :style="{color:'var(--text-2)'}">{{ step.explanation }}</p>
            <div v-if="step.math" class="my-2 px-4 py-2 rounded-[var(--radius-md)] text-center"
              :style="{backgroundColor:'var(--primary-light)'}">
              <MathRenderer :expression="step.math" display />
            </div>
            <p v-if="step.tip" class="text-[10px] italic mt-1" :style="{color:'var(--gold)'}">💡 {{ step.tip }}</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="revealedSteps < steps.length" class="mt-4 text-center">
      <AppButton variant="secondary" size="sm" @click="revealNext">Show Next Step →</AppButton>
    </div>
  </AppCard>
</template>
