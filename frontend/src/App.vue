<script setup lang="ts">
import { onMounted } from 'vue'
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()

onMounted(() => {
  // Default to student theme until auth determines the role
  ui.setTheme('student')
})
</script>

<template>
  <RouterView />
</template>
