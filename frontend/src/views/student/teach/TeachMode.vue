<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppProgress from '@/components/ui/AppProgress.vue'

const auth = useAuthStore()
const router = useRouter()
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">

    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--ink)' }">Teach Mode</h1>
    <p class="text-sm mb-6" :style="{ color: 'var(--ink-muted)' }">Learn a topic step by step, from simple to exam-level.</p>
    <div class="space-y-4">
      <AppCard v-for="block in ['Concept Overview','Why It Matters','Simple Explanation','Formal Definition','Visual Understanding','Worked Examples','Common Mistakes','Misconception Alerts','Exam Appearance','Practice Problem']" :key="block" padding="md">
        <h3 class="text-sm font-semibold mb-1" :style="{ color: 'var(--ink)' }">{{ block }}</h3>
        <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">Teaching content for this block will be generated from the curriculum and rendered here.</p>
      </AppCard>
    </div>
  </div>
</template>
