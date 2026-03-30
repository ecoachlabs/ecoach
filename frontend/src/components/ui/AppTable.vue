<script setup lang="ts">
defineProps<{
  columns: { key: string; label: string; width?: string; align?: 'left' | 'center' | 'right' }[]
  rows: Record<string, any>[]
  sortable?: boolean
  compact?: boolean
}>()

defineEmits<{ rowClick: [row: Record<string, any>] }>()
</script>

<template>
  <div class="overflow-x-auto rounded-[var(--radius-lg)] border" :style="{ borderColor: 'var(--card-border)' }">
    <table class="w-full text-left" :class="compact ? 'text-xs' : 'text-sm'">
      <thead>
        <tr :style="{ backgroundColor: 'var(--primary-light)' }">
          <th v-for="col in columns" :key="col.key"
            class="font-semibold uppercase tracking-wider px-4"
            :class="compact ? 'py-2 text-[10px]' : 'py-3 text-[11px]'"
            :style="{ color: 'var(--text-3)', width: col.width, textAlign: col.align || 'left' }">
            {{ col.label }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, i) in rows" :key="i"
          class="border-t cursor-pointer transition-colors hover:bg-[var(--primary-light)]"
          :style="{ borderColor: 'var(--card-border)', backgroundColor: 'var(--card-bg)' }"
          @click="$emit('rowClick', row)">
          <td v-for="col in columns" :key="col.key"
            :class="compact ? 'px-4 py-2' : 'px-4 py-3'"
            :style="{ color: 'var(--text)', textAlign: col.align || 'left' }">
            <slot :name="col.key" :value="row[col.key]" :row="row">
              {{ row[col.key] }}
            </slot>
          </td>
        </tr>
        <tr v-if="!rows.length">
          <td :colspan="columns.length" class="px-4 py-8 text-center" :style="{ color: 'var(--text-3)' }">
            No data available
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
