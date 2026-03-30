<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  notifications: { id: number; type: string; title: string; message: string; time: string; read: boolean }[]
  open: boolean
}>()

defineEmits<{ close: []; markRead: [id: number]; clearAll: [] }>()

const typeIcons: Record<string, string> = {
  decay: '🧠', milestone: '🏆', risk: '⚠', reminder: '⏰', coach: '💡', update: '📦',
}
</script>

<template>
  <Transition name="notif">
    <div v-if="open" class="absolute top-full right-0 mt-2 w-80 max-h-96 overflow-y-auto rounded-[var(--radius-xl)] border"
      :style="{backgroundColor:'var(--card-bg)',borderColor:'var(--card-border)',boxShadow:'var(--shadow-xl)',zIndex:'var(--z-dropdown)'}">
      <div class="flex items-center justify-between px-4 py-3 border-b" :style="{borderColor:'var(--card-border)'}">
        <h3 class="text-xs font-semibold" :style="{color:'var(--text)'}">Notifications</h3>
        <AppButton variant="ghost" size="sm" @click="$emit('clearAll')">Clear all</AppButton>
      </div>
      <div v-if="notifications.length" class="divide-y" :style="{borderColor:'var(--card-border)'}">
        <div v-for="n in notifications" :key="n.id"
          class="px-4 py-3 flex items-start gap-2.5 cursor-pointer transition-colors"
          :class="!n.read ? 'bg-[var(--primary-light)]' : ''"
          @click="$emit('markRead', n.id)">
          <span class="text-base mt-0.5">{{ typeIcons[n.type] || '●' }}</span>
          <div class="flex-1 min-w-0">
            <p class="text-xs font-medium" :style="{color:'var(--text)'}">{{ n.title }}</p>
            <p class="text-[10px] line-clamp-2" :style="{color:'var(--text-3)'}">{{ n.message }}</p>
            <p class="text-[9px] mt-0.5" :style="{color:'var(--text-3)'}">{{ n.time }}</p>
          </div>
          <div v-if="!n.read" class="w-2 h-2 rounded-full mt-1.5 shrink-0" :style="{backgroundColor:'var(--accent)'}" />
        </div>
      </div>
      <div v-else class="px-4 py-8 text-center">
        <p class="text-xs" :style="{color:'var(--text-3)'}">No notifications</p>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.notif-enter-active { transition: all 200ms var(--ease-out); }
.notif-leave-active { transition: all 100ms; }
.notif-enter-from, .notif-leave-to { opacity: 0; transform: translateY(-4px) scale(0.97); }
</style>
