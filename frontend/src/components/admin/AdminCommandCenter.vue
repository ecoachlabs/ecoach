<script setup lang="ts">
import { ref, onMounted } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AdminStatCard from './AdminStatCard.vue'
import SystemHealthCard from './SystemHealthCard.vue'

const stats = ref([
  { label: 'Active Students', value: '24', change: 3, icon: '👤' },
  { label: 'Questions', value: '1,240', change: 12, icon: '❓' },
  { label: 'Content Packs', value: '3', change: 0, icon: '📦' },
  { label: 'Avg Readiness', value: '52%', change: 5, icon: '◐' },
])

const health = ref([
  { label: 'Database', value: '48 MB', status: 'healthy' as const },
  { label: 'Content Coverage', value: '78%', status: 'warning' as const },
  { label: 'Question Quality', value: '94%', status: 'healthy' as const },
  { label: 'Last Backup', value: '2h ago', status: 'healthy' as const },
])

const recentActivity = ref([
  { time: '10:23', action: 'Pack installed', detail: 'math-bece-2025 v1.2' },
  { time: '09:45', action: 'Student created', detail: 'Ama Mensah' },
  { time: '09:12', action: 'Diagnostic completed', detail: 'Kwame Asante — Mathematics' },
])
</script>

<template>
  <div class="space-y-6">
    <!-- Stats grid -->
    <div class="grid grid-cols-4 gap-3">
      <AdminStatCard v-for="s in stats" :key="s.label" v-bind="s" />
    </div>

    <div class="grid grid-cols-3 gap-4">
      <!-- System Health -->
      <SystemHealthCard :metrics="health" />

      <!-- Recent Activity -->
      <AppCard padding="md" class="col-span-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Recent Activity</h3>
        <div class="space-y-2">
          <div v-for="a in recentActivity" :key="a.time" class="flex items-center gap-3 text-xs py-1 border-b" :style="{ borderColor: 'var(--card-border)' }">
            <span class="font-mono tabular-nums shrink-0" :style="{ color: 'var(--text-3)' }">{{ a.time }}</span>
            <span class="font-medium" :style="{ color: 'var(--text)' }">{{ a.action }}</span>
            <span class="truncate" :style="{ color: 'var(--text-3)' }">{{ a.detail }}</span>
          </div>
        </div>
      </AppCard>
    </div>
  </div>
</template>
