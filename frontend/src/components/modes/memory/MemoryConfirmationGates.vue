<script setup lang="ts">
defineProps<{
  currentGate: number  // 0-4
  gateResults: (boolean | null)[]  // passed or not per gate
}>()

const gates = [
  { label: 'Independent Recall', icon: '🧠', desc: 'Can you recall without help?' },
  { label: 'Delayed Recall', icon: '⏰', desc: 'Can you recall after time passes?' },
  { label: 'Variant Recall', icon: '↻', desc: 'Same concept, different form?' },
  { label: 'Embedded Use', icon: '🔗', desc: 'Can you use it in context?' },
  { label: 'Stability Check', icon: '🔒', desc: 'Retained over time?' },
]
</script>

<template>
  <div>
    <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--text-3)'}">Memory Confirmation</h3>
    <div class="space-y-1.5">
      <div v-for="(gate, i) in gates" :key="i"
        class="flex items-center gap-3 px-3 py-2 rounded-[var(--radius-md)] transition-all"
        :class="i === currentGate ? 'bg-[var(--accent-light)]' : ''">
        <div class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold shrink-0"
          :class="gateResults[i] === true ? 'bg-emerald-500 text-white' : gateResults[i] === false ? 'bg-red-400 text-white' : i === currentGate ? 'bg-[var(--accent)] text-white' : ''"
          :style="gateResults[i] === null && i !== currentGate ? {backgroundColor:'var(--primary-light)',color:'var(--text-3)'} : {}">
          {{ gateResults[i] === true ? '✓' : gateResults[i] === false ? '✕' : gate.icon }}
        </div>
        <div>
          <p class="text-xs font-medium" :style="{color: i === currentGate ? 'var(--accent)' : 'var(--text)'}">{{ gate.label }}</p>
          <p class="text-[9px]" :style="{color:'var(--text-3)'}">{{ gate.desc }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
