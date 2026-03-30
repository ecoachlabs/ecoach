<script setup lang="ts">
defineProps<{
  modelValue: string | null
}>()

defineEmits<{ 'update:modelValue': [value: string] }>()

const options = [
  { key: 'sure', label: 'Sure', icon: '✓', desc: 'I know this' },
  { key: 'not_sure', label: 'Not Sure', icon: '?', desc: 'Educated guess' },
  { key: 'guessed', label: 'Guessed', icon: '~', desc: 'Random pick' },
]
</script>

<template>
  <div>
    <p class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">
      How sure are you?
    </p>
    <div class="flex gap-2">
      <button
        v-for="opt in options"
        :key="opt.key"
        class="flex-1 py-2 px-3 rounded-[var(--radius-md)] text-center border transition-all"
        :class="modelValue === opt.key
          ? 'bg-[var(--accent)] text-white border-[var(--accent)] shadow-sm'
          : 'border-[var(--card-border)] text-[var(--text-2)] hover:border-[var(--accent)]'"
        :style="{
          backgroundColor: modelValue === opt.key ? undefined : 'var(--card-bg)',
          transitionDuration: 'var(--dur-fast)',
          transitionTimingFunction: 'var(--ease-spring)',
        }"
        @click="$emit('update:modelValue', opt.key)"
      >
        <div class="text-sm font-medium">{{ opt.icon }} {{ opt.label }}</div>
        <div class="text-[10px] mt-0.5 opacity-70">{{ opt.desc }}</div>
      </button>
    </div>
  </div>
</template>
