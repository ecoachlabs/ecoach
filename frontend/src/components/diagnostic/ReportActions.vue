<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  actions: string[]
}>()

defineEmits<{ startAction: [index: number] }>()
</script>

<template>
  <div class="space-y-2">
    <AppCard v-for="(action, i) in actions" :key="i" padding="md">
      <div class="flex items-start gap-3">
        <div class="w-7 h-7 rounded-lg flex items-center justify-center text-xs font-bold shrink-0"
          :style="{backgroundColor:'var(--accent-light)',color:'var(--accent)'}">{{ i + 1 }}</div>
        <div class="flex-1">
          <p class="text-sm" :style="{color:'var(--text-2)'}">{{ action }}</p>
        </div>
        <AppButton variant="secondary" size="sm" @click="$emit('startAction', i)">Start</AppButton>
      </div>
    </AppCard>
  </div>
</template>
