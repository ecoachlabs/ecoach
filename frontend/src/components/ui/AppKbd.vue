<script setup lang="ts">
defineProps<{ keys: string[] }>()
</script>

<template>
  <span class="inline-flex items-center gap-0.5">
    <kbd v-for="(key, i) in keys" :key="i"
      class="inline-flex items-center justify-center min-w-[20px] h-5 px-1.5 text-[10px] font-mono font-medium rounded border"
      :style="{backgroundColor:'var(--primary-light)',borderColor:'var(--border-strong)',color:'var(--text-2)'}">
      {{ key }}
    </kbd>
  </span>
</template>
