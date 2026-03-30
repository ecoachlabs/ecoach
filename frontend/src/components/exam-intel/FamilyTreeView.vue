<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  familyName: string
  parent?: { id: number; label: string; year: string }
  siblings: { id: number; label: string; year: string }[]
  mutations: { id: number; label: string; year: string; change: string }[]
}>()

defineEmits<{ selectQuestion: [id: number] }>()
</script>

<template>
  <div>
    <!-- Parent -->
    <div v-if="parent" class="text-center mb-4">
      <AppCard padding="sm" hover class="inline-block" @click="$emit('selectQuestion', parent.id)">
        <AppBadge color="muted" size="xs">Origin · {{ parent.year }}</AppBadge>
        <p class="text-xs font-medium mt-1" :style="{color:'var(--text)'}">{{ parent.label }}</p>
      </AppCard>
      <div class="w-px h-6 mx-auto" :style="{backgroundColor:'var(--border-soft)'}" />
    </div>

    <!-- Current family node -->
    <div class="text-center mb-4">
      <AppCard padding="md" glow="accent" class="inline-block">
        <p class="text-sm font-bold" :style="{color:'var(--accent)'}">{{ familyName }}</p>
      </AppCard>
    </div>

    <!-- Siblings + Mutations -->
    <div class="grid grid-cols-2 gap-4">
      <div v-if="siblings.length">
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{color:'var(--text-3)'}">Siblings (same generation)</p>
        <div class="space-y-1.5">
          <AppCard v-for="s in siblings" :key="s.id" padding="sm" hover @click="$emit('selectQuestion', s.id)">
            <p class="text-xs font-medium" :style="{color:'var(--text)'}">{{ s.label }}</p>
            <p class="text-[9px]" :style="{color:'var(--text-3)'}">{{ s.year }}</p>
          </AppCard>
        </div>
      </div>
      <div v-if="mutations.length">
        <p class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{color:'var(--gold)'}">Mutations (evolved)</p>
        <div class="space-y-1.5">
          <AppCard v-for="m in mutations" :key="m.id" padding="sm" hover @click="$emit('selectQuestion', m.id)">
            <p class="text-xs font-medium" :style="{color:'var(--text)'}">{{ m.label }}</p>
            <p class="text-[9px]" :style="{color:'var(--gold)'}">{{ m.year }} · {{ m.change }}</p>
          </AppCard>
        </div>
      </div>
    </div>
  </div>
</template>
