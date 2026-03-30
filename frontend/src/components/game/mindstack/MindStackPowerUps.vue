<script setup lang="ts">
defineProps<{
  powerUps: { type: string; count: number; icon: string; label: string }[]
}>()

defineEmits<{ use: [type: string] }>()
</script>

<template>
  <div class="flex gap-1.5">
    <button v-for="pu in powerUps" :key="pu.type"
      class="relative w-9 h-9 rounded-lg flex items-center justify-center text-sm transition-all border"
      :class="pu.count > 0 ? 'hover:scale-110 active:scale-95' : 'opacity-30 cursor-not-allowed'"
      :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }"
      :title="pu.label"
      :disabled="pu.count <= 0"
      @click="pu.count > 0 && $emit('use', pu.type)">
      {{ pu.icon }}
      <span v-if="pu.count > 0"
        class="absolute -top-1 -right-1 w-4 h-4 rounded-full text-[8px] font-bold flex items-center justify-center text-white"
        :style="{ backgroundColor: 'var(--accent)' }">
        {{ pu.count }}
      </span>
    </button>
  </div>
</template>
