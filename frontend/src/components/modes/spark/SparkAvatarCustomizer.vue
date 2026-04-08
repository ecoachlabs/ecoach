<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

defineProps<{
  currentAvatar: string
  unlockedItems: { id: string; type: string; name: string; icon: string; equipped: boolean }[]
  lockedItems: { id: string; type: string; name: string; icon: string; requirement: string }[]
}>()

defineEmits<{ equip: [itemId: string]; unequip: [itemId: string] }>()
</script>

<template>
  <div>
    <div class="text-center mb-6">
      <div class="w-24 h-24 rounded-3xl mx-auto mb-3 flex items-center justify-center text-5xl"
        :style="{ backgroundColor: 'var(--gold-light)' }">
        {{ currentAvatar || '😊' }}
      </div>
      <h3 class="font-display text-base font-semibold" :style="{ color: 'var(--text)' }">Your Avatar</h3>
    </div>

    <!-- Unlocked items -->
    <div v-if="unlockedItems.length" class="mb-4">
      <h4 class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--success)' }">Unlocked</h4>
      <div class="grid grid-cols-4 gap-2">
        <button v-for="item in unlockedItems" :key="item.id"
          class="p-2 rounded-[var(--radius-md)] text-center border transition-all"
          :class="item.equipped ? 'ring-2 ring-[var(--accent)] bg-[var(--accent-light)]' : 'border-[var(--card-border)]'"
          :style="{ backgroundColor: !item.equipped ? 'var(--card-bg)' : undefined }"
          @click="item.equipped ? $emit('unequip', item.id) : $emit('equip', item.id)">
          <span class="text-2xl block mb-0.5">{{ item.icon }}</span>
          <span class="text-[8px] font-medium" :style="{ color: 'var(--text-3)' }">{{ item.name }}</span>
        </button>
      </div>
    </div>

    <!-- Locked items -->
    <div v-if="lockedItems.length">
      <h4 class="text-[10px] font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-3)' }">Locked</h4>
      <div class="grid grid-cols-4 gap-2">
        <div v-for="item in lockedItems" :key="item.id"
          class="p-2 rounded-[var(--radius-md)] text-center opacity-40" :style="{ backgroundColor: 'var(--card-bg)' }">
          <span class="text-2xl block mb-0.5">🔒</span>
          <span class="text-[8px] font-medium" :style="{ color: 'var(--text-3)' }">{{ item.requirement }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
