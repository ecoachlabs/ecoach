<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const auth = useAuthStore()
const router = useRouter()
const subjects = ref<SubjectDto[]>([])
const loading = ref(true)

onMounted(async () => {
  try { subjects.value = await listSubjects(1) } catch {}
  loading.value = false
})

const entryPoints = [
  { key: 'year', label: 'By Year', desc: 'Browse papers by exam year', icon: '📅' },
  { key: 'topic', label: 'By Topic', desc: 'See how topics are tested', icon: '📊' },
  { key: 'patterns', label: 'Patterns', desc: 'Recurring question families', icon: '🔄' },
  { key: 'weakness', label: 'My Weaknesses', desc: 'Your weak exam patterns', icon: '⚠' },
  { key: 'replay', label: 'Exam Replay', desc: 'Sit a real past paper', icon: '▶' },
  { key: 'predict', label: 'Likely Next', desc: 'Predicted question styles', icon: '🔮' },
  { key: 'atlas', label: 'Family Atlas', desc: 'Visual question network', icon: '🗺' },
]
</script>

<template>
  <div class="p-6 lg:p-8 max-w-4xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Exam Intelligence</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Past questions are not just practice — they are signals. Discover the patterns.</p>

    <div class="grid grid-cols-2 lg:grid-cols-3 gap-3 mb-8">
      <AppCard v-for="ep in entryPoints" :key="ep.key" hover padding="md">
        <div class="flex items-start gap-3">
          <span class="text-xl">{{ ep.icon }}</span>
          <div>
            <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ ep.label }}</p>
            <p class="text-[10px]" :style="{ color: 'var(--text-3)' }">{{ ep.desc }}</p>
          </div>
        </div>
      </AppCard>
    </div>

    <!-- Subject filter -->
    <div v-if="subjects.length" class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Filter by Subject</h3>
      <div class="flex gap-2">
        <AppButton v-for="s in subjects" :key="s.id" variant="secondary" size="sm">{{ s.name }}</AppButton>
      </div>
    </div>

    <!-- Quick insight -->
    <AppCard padding="md" glow="accent">
      <div class="flex items-center gap-4">
        <div class="w-10 h-10 rounded-xl flex items-center justify-center text-lg"
          :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">🔍</div>
        <div class="flex-1">
          <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">Quick Insight</p>
          <p class="text-xs" :style="{ color: 'var(--text-2)' }">Fractions have appeared in 9 of the last 10 BECE papers. Your weakest recurring pattern.</p>
        </div>
      </div>
    </AppCard>
  </div>
</template>
