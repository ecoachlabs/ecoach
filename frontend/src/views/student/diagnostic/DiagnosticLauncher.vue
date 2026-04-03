<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { launchDiagnostic } from '@/ipc/diagnostic'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const auth = useAuthStore()
const router = useRouter()
const subjects = ref<SubjectDto[]>([])
const selectedSubject = ref<number | null>(null)
const selectedMode = ref<'quick' | 'standard' | 'deep'>('standard')
const loading = ref(true)
const launching = ref(false)
const error = ref('')

const modes = [
  { key: 'quick' as const, label: 'Quick Scan', desc: '10–15 questions · ~10 min', icon: '◇' },
  { key: 'standard' as const, label: 'Standard', desc: '25–35 questions · ~20 min · Multi-phase', icon: '◈' },
  { key: 'deep' as const, label: 'Deep Analysis', desc: '40–60 questions · ~35 min · Full battery', icon: '◉' },
]

onMounted(async () => {
  try {
    subjects.value = await listSubjects(1)
    if (subjects.value.length) selectedSubject.value = subjects.value[0].id
  } catch {}
  loading.value = false
})

async function startDiagnostic() {
  if (!selectedSubject.value || !auth.currentAccount) return
  launching.value = true
  error.value = ''
  try {
    const result = await launchDiagnostic(
      auth.currentAccount.id,
      selectedSubject.value,
      selectedMode.value,
    )
    router.push(`/student/diagnostic/${result.diagnostic_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start diagnostic'
  } finally {
    launching.value = false
  }
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-3xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">
      Diagnostic Assessment
    </h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">
      Discover exactly where you stand. Multiple phases test different dimensions of your knowledge.
    </p>

    <!-- Error -->
    <div v-if="error" class="mb-6 p-3 rounded-[var(--radius-md)] text-sm" :style="{ backgroundColor: 'var(--danger-light)', color: 'var(--danger)' }">
      {{ error }}
    </div>

    <!-- Subject -->
    <div class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Subject</h3>
      <div v-if="loading" class="flex gap-2">
        <div v-for="i in 3" :key="i" class="h-9 w-24 rounded-[var(--radius-md)] animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
      <div v-else class="flex flex-wrap gap-2">
        <button
          v-for="s in subjects" :key="s.id"
          class="px-4 py-2 rounded-[var(--radius-md)] text-sm font-medium transition-all"
          :class="selectedSubject === s.id
            ? 'bg-[var(--accent)] text-white'
            : 'bg-[var(--card-bg)] border border-[var(--card-border)] text-[var(--text-2)] hover:border-[var(--accent)]'"
          @click="selectedSubject = s.id"
        >
          {{ s.name }}
        </button>
      </div>
    </div>

    <!-- Mode -->
    <div class="mb-8">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-3)' }">Diagnostic Depth</h3>
      <div class="space-y-2">
        <AppCard
          v-for="mode in modes" :key="mode.key"
          padding="md" hover
          :class="selectedMode === mode.key ? 'ring-2 ring-[var(--accent)]' : ''"
          @click="selectedMode = mode.key"
        >
          <div class="flex items-center gap-3">
            <div
              class="w-10 h-10 rounded-xl flex items-center justify-center text-lg"
              :style="{
                backgroundColor: selectedMode === mode.key ? 'var(--accent-light)' : 'var(--primary-light)',
                color: selectedMode === mode.key ? 'var(--accent)' : 'var(--text-3)',
              }"
            >
              {{ mode.icon }}
            </div>
            <div>
              <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ mode.label }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ mode.desc }}</p>
            </div>
            <div class="ml-auto">
              <div v-if="selectedMode === mode.key" class="w-5 h-5 rounded-full bg-[var(--accent)] flex items-center justify-center text-white text-xs font-bold">✓</div>
            </div>
          </div>
        </AppCard>
      </div>
    </div>

    <AppButton
      variant="primary" size="lg"
      :disabled="!selectedSubject || launching"
      :loading="launching"
      @click="startDiagnostic"
    >
      Begin Diagnostic →
    </AppButton>
  </div>
</template>
