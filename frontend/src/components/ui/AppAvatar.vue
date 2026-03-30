<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  name: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
  color?: string
}>()

const initial = computed(() => props.name?.charAt(0)?.toUpperCase() || '?')

const colors = ['from-teal-500 to-emerald-600', 'from-violet-500 to-purple-600', 'from-amber-500 to-orange-600', 'from-sky-500 to-blue-600', 'from-rose-500 to-pink-600']
const colorClass = computed(() => {
  if (props.color) return ''
  const idx = props.name.charCodeAt(0) % colors.length
  return colors[idx]
})
</script>

<template>
  <div
    class="inline-flex items-center justify-center rounded-xl font-bold text-white bg-gradient-to-br shadow-sm"
    :class="[
      colorClass,
      size === 'sm' ? 'w-8 h-8 text-xs rounded-lg' :
      size === 'lg' ? 'w-14 h-14 text-xl rounded-2xl' :
      size === 'xl' ? 'w-20 h-20 text-3xl rounded-2xl' :
      'w-10 h-10 text-sm',
    ]"
    :style="color ? { background: color } : {}"
  >
    {{ initial }}
  </div>
</template>
