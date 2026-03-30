<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import { onMounted } from 'vue'

const auth = useAuthStore()
const router = useRouter()
const subjects = ref<SubjectDto[]>([])
const selectedSubject = ref<number | null>(null)
const selectedMode = ref<'quick' | 'standard' | 'deep'>('standard')
const loading = ref(true)

const modes = [
  { key: 'quick' as const, label: 'Quick Scan', desc: '10-15 questions · ~10 min', icon: '◇' },
  { key: 'standard' as const, label: 'Standard', desc: '25-35 questions · ~20 min · Multi-phase', icon: '◈' },
  { key: 'deep' as const, label: 'Deep Analysis', desc: '40-60 questions · ~35 min · Full diagnostic battery', icon: '◉' },
]

onMounted(async () => {
  try {
    subjects.value = await listSubjects(1)
    if (subjects.value.length) selectedSubject.value = subjects.value[0].id
  } catch {}
  loading.value = false
})

function startDiagnostic() {
  if (!selectedSubject.value) return
  // In full implementation, this calls start_diagnostic IPC
  router.push('/student/diagnostic/1')
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-3xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Diagnostic Assessment</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Discover exactly where you stand. Multiple phases test different dimensions of your knowledge.</p>

    <!-- Subject -->
    <div class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Subject</h3>
      <div class="flex gap-2">
        <button v-for="s in subjects" :key="s.id"
          class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium transition-all"
          :class="selectedSubject === s.id ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)]'"
          @click="selectedSubject = s.id">
          {{ s.name }}
        </button>
      </div>
    </div>

    <!-- Mode -->
    <div class="mb-8">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Diagnostic Mode</h3>
      <div class="space-y-2">
        <AppCard v-for="mode in modes" :key="mode.key" padding="md" hover
          :class="selectedMode === mode.key ? 'ring-2 ring-[var(--accent)]' : ''"
          @click="selectedMode = mode.key">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-xl flex items-center justify-center text-lg"
              :style="{ backgroundColor: selectedMode === mode.key ? 'var(--accent-light)' : 'var(--primary-light)', color: selectedMode === mode.key ? 'var(--accent)' : 'var(--text-3)' }">
              {{ mode.icon }}
            </div>
            <div>
              <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ mode.label }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ mode.desc }}</p>
            </div>
          </div>
        </AppCard>
      </div>
    </div>

    <AppButton variant="primary" size="lg" :disabled="!selectedSubject" @click="startDiagnostic">
      Begin Diagnostic →
    </AppButton>
  </div>
</template>
