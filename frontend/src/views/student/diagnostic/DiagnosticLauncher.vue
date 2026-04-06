<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import { launchDiagnostic } from '@/ipc/diagnostic'

const auth = useAuthStore()
const router = useRouter()
const subjects = ref<SubjectDto[]>([])
const selectedSubject = ref<number | null>(null)
const selectedMode = ref<'quick' | 'standard' | 'deep'>('standard')
const loading = ref(true)
const launching = ref(false)
const error = ref('')

const modes = [
  { key: 'quick' as const, label: 'Quick Scan', desc: '10–15 questions · ~10 min', time: '10 min' },
  { key: 'standard' as const, label: 'Standard', desc: '25–35 questions · ~20 min · Multi-phase', time: '20 min' },
  { key: 'deep' as const, label: 'Deep Analysis', desc: '40–60 questions · ~35 min · Full battery', time: '35 min' },
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
    const result = await launchDiagnostic(auth.currentAccount.id, selectedSubject.value, selectedMode.value)
    router.push(`/student/diagnostic/${result.diagnostic_id}`)
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to start diagnostic'
  } finally {
    launching.value = false
  }
}

const selectedModeData = () => modes.find(m => m.key === selectedMode.value) ?? modes[1]
</script>

<template>
  <div class="h-full flex overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Left: info panel -->
    <div
      class="w-80 flex-shrink-0 flex flex-col justify-between p-8 border-r"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow mb-4">Diagnostic</p>
        <h1 class="font-display text-3xl font-bold leading-tight mb-4" :style="{ color: 'var(--ink)' }">
          Discover exactly where you stand
        </h1>
        <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
          Multiple phases test different dimensions of your knowledge — revealing blind spots and hidden strengths.
        </p>
      </div>

      <div class="space-y-4">
        <div class="flex items-start gap-3">
          <div class="w-7 h-7 rounded-lg flex items-center justify-center flex-shrink-0 text-xs font-bold"
            :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink)', border: '1px solid var(--border-soft)' }">1</div>
          <div>
            <p class="text-xs font-semibold" :style="{ color: 'var(--ink)' }">Choose subject & depth</p>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">Pick what to test and how deep</p>
          </div>
        </div>
        <div class="flex items-start gap-3">
          <div class="w-7 h-7 rounded-lg flex items-center justify-center flex-shrink-0 text-xs font-bold"
            :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink)', border: '1px solid var(--border-soft)' }">2</div>
          <div>
            <p class="text-xs font-semibold" :style="{ color: 'var(--ink)' }">Answer honestly</p>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">No guessing — accuracy matters</p>
          </div>
        </div>
        <div class="flex items-start gap-3">
          <div class="w-7 h-7 rounded-lg flex items-center justify-center flex-shrink-0 text-xs font-bold"
            :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink)', border: '1px solid var(--border-soft)' }">3</div>
          <div>
            <p class="text-xs font-semibold" :style="{ color: 'var(--ink)' }">Get your report</p>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">Personalized gap analysis & plan</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Right: selection -->
    <div class="flex-1 flex flex-col overflow-hidden">

      <div class="flex-1 overflow-y-auto px-10 py-8 space-y-8">
        <div v-if="error" class="p-3 rounded-xl text-sm"
          style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ error }}</div>

        <!-- Subject selector -->
        <div>
          <p class="section-label mb-3">Choose Subject</p>
          <div v-if="loading" class="flex gap-2">
            <div v-for="i in 3" :key="i" class="h-10 w-28 rounded-xl animate-pulse"
              :style="{ backgroundColor: 'var(--border-soft)' }" />
          </div>
          <div v-else class="flex flex-wrap gap-2">
            <button
              v-for="s in subjects"
              :key="s.id"
              class="subject-chip"
              :class="{ active: selectedSubject === s.id }"
              @click="selectedSubject = s.id"
            >{{ s.name }}</button>
          </div>
        </div>

        <!-- Mode selector -->
        <div>
          <p class="section-label mb-3">Diagnostic Depth</p>
          <div class="space-y-3">
            <button
              v-for="mode in modes"
              :key="mode.key"
              class="mode-card w-full text-left"
              :class="{ selected: selectedMode === mode.key }"
              @click="selectedMode = mode.key"
            >
              <div class="mode-check">
                <div v-if="selectedMode === mode.key" class="mode-check-dot" />
              </div>
              <div class="flex-1">
                <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ mode.label }}</p>
                <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ mode.desc }}</p>
              </div>
              <span class="text-[10px] font-semibold px-2 py-0.5 rounded-full flex-shrink-0"
                :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink-secondary)', border: '1px solid var(--border-soft)' }">
                {{ mode.time }}
              </span>
            </button>
          </div>
        </div>
      </div>

      <!-- Launch footer -->
      <div class="flex-shrink-0 px-10 py-6 border-t flex items-center justify-between"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }">
        <div>
          <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">
            {{ selectedModeData().label }} · {{ selectedModeData().time }}
          </p>
          <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ selectedModeData().desc }}</p>
        </div>
        <button
          class="launch-btn"
          :disabled="!selectedSubject || launching"
          @click="startDiagnostic"
        >{{ launching ? 'Starting…' : 'Begin Diagnostic →' }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--accent);
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.subject-chip {
  padding: 8px 18px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid var(--border-soft);
  background: var(--surface);
  color: var(--ink-secondary);
  transition: all 120ms;
}
.subject-chip.active,
.subject-chip:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}

.mode-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 20px;
  border-radius: 16px;
  border: 1.5px solid var(--border-soft);
  background: var(--surface);
  cursor: pointer;
  transition: border-color 120ms;
}
.mode-card.selected {
  border-color: var(--accent);
  background: var(--accent-glow);
}
.mode-card:hover:not(.selected) { border-color: var(--ink-muted); }

.mode-check {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: 2px solid var(--border-soft);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: border-color 120ms;
}
.mode-card.selected .mode-check {
  border-color: var(--accent);
  background: var(--accent);
}
.mode-check-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: white;
}

.launch-btn {
  padding: 12px 28px;
  border-radius: 14px;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  background: var(--accent);
  color: white;
  transition: opacity 140ms, transform 140ms;
}
.launch-btn:hover:not(:disabled) { opacity: 0.87; transform: translateY(-1px); }
.launch-btn:disabled { opacity: 0.45; cursor: not-allowed; }
</style>
