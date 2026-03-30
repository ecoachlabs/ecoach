<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
  backTo?: string
}>()
</script>

<template>
  <div class="flex items-start justify-between mb-8">
    <div>
      <div v-if="backTo" class="mb-2">
        <RouterLink :to="backTo" class="text-xs font-medium hover:underline" :style="{ color: 'var(--accent)' }">
          ← Back
        </RouterLink>
      </div>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--text)' }">
        {{ title }}
      </h1>
      <p v-if="subtitle" class="text-sm mt-1" :style="{ color: 'var(--text-3)' }">
        {{ subtitle }}
      </p>
    </div>
    <div>
      <slot name="actions" />
    </div>
  </div>
</template>
