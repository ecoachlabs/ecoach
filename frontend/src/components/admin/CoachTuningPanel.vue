<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppSlider from '@/components/ui/AppSlider.vue'
import AppSwitch from '@/components/ui/AppSwitch.vue'
import AppButton from '@/components/ui/AppButton.vue'

const aggressiveness = ref(50)
const pressureTolerance = ref(60)
const recoveryGentleness = ref(70)
const autoRescue = ref(true)
const antiCheat = ref(true)
const hintLimit = ref(3)

defineEmits<{ save: [config: any] }>()
</script>

<template>
  <div class="max-w-lg space-y-5">
    <h3 class="text-sm font-bold" :style="{color:'var(--text)'}">Coach Behavior Tuning</h3>

    <AppCard padding="md">
      <AppSlider v-model="aggressiveness" label="Coach Aggressiveness" :show-value="true" :format-value="v => v + '%'" />
      <p class="text-[9px] mt-1" :style="{color:'var(--text-3)'}">How quickly the coach pushes students to harder content</p>
    </AppCard>

    <AppCard padding="md">
      <AppSlider v-model="pressureTolerance" label="Pressure Tolerance" :show-value="true" :format-value="v => v + '%'" />
      <p class="text-[9px] mt-1" :style="{color:'var(--text-3)'}">How much timed pressure to introduce</p>
    </AppCard>

    <AppCard padding="md">
      <AppSlider v-model="recoveryGentleness" label="Recovery Gentleness" :show-value="true" :format-value="v => v + '%'" />
      <p class="text-[9px] mt-1" :style="{color:'var(--text-3)'}">How gentle the coach is after failure</p>
    </AppCard>

    <AppCard padding="md">
      <AppSlider v-model="hintLimit" label="Hint Limit per Session" :min="0" :max="10" :show-value="true" />
    </AppCard>

    <AppCard padding="md" class="space-y-3">
      <AppSwitch v-model="autoRescue" label="Auto-rescue on repeated failure" />
      <AppSwitch v-model="antiCheat" label="Anti-cheat answer controls" />
    </AppCard>

    <AppButton variant="primary" @click="$emit('save', { aggressiveness, pressureTolerance, recoveryGentleness, autoRescue, antiCheat, hintLimit })">
      Save Configuration
    </AppButton>
  </div>
</template>
