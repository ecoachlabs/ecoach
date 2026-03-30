<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  conceptA: string
  conceptB: string
  rows: { dimension: string; id: number }[]
  answered: boolean
  correctAnswers?: Record<number, { a: string; b: string }>
}>()

const emit = defineEmits<{ submit: [answers: Record<number, { a: string; b: string }>] }>()
const answers = ref<Record<number, { a: string; b: string }>>({})

props.rows.forEach(r => { answers.value[r.id] = { a: '', b: '' } })

const allFilled = () => Object.values(answers.value).every(v => v.a.trim() && v.b.trim())
</script>

<template>
  <div class="overflow-x-auto">
    <table class="w-full text-sm border-collapse">
      <thead>
        <tr>
          <th class="text-left px-3 py-2 border-b font-semibold text-xs uppercase" :style="{ borderColor: 'var(--card-border)', color: 'var(--text-3)' }">Dimension</th>
          <th class="text-left px-3 py-2 border-b font-semibold text-xs uppercase" :style="{ borderColor: 'var(--card-border)', color: 'var(--accent)' }">{{ conceptA }}</th>
          <th class="text-left px-3 py-2 border-b font-semibold text-xs uppercase" :style="{ borderColor: 'var(--card-border)', color: 'var(--gold)' }">{{ conceptB }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in rows" :key="row.id">
          <td class="px-3 py-2 border-b font-medium" :style="{ borderColor: 'var(--card-border)', color: 'var(--text)' }">{{ row.dimension }}</td>
          <td class="px-3 py-2 border-b" :style="{ borderColor: 'var(--card-border)' }">
            <input v-model="answers[row.id].a" :disabled="answered"
              class="w-full px-2 py-1 rounded border text-sm" :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }" placeholder="..." />
          </td>
          <td class="px-3 py-2 border-b" :style="{ borderColor: 'var(--card-border)' }">
            <input v-model="answers[row.id].b" :disabled="answered"
              class="w-full px-2 py-1 rounded border text-sm" :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)', color: 'var(--text)' }" placeholder="..." />
          </td>
        </tr>
      </tbody>
    </table>
    <div v-if="!answered && allFilled()" class="mt-3 text-right">
      <button class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium text-white" :style="{ backgroundColor: 'var(--accent)' }"
        @click="emit('submit', answers)">Submit</button>
    </div>
  </div>
</template>
