<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { listSubjects, type SubjectDto } from '@/ipc/coach'

const router = useRouter()
const subjects = ref<SubjectDto[]>([])
const selected = ref<number[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    subjects.value = await listSubjects(1)
  } catch (e) {
    console.error('Failed to load subjects:', e)
  }
  loading.value = false
})

function toggle(id: number) {
  const idx = selected.value.indexOf(id)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push(id)
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <!-- Step indicator -->
      <div class="flex items-center gap-1.5 mb-4">
        <div v-for="i in 4" :key="i"
          class="h-1 rounded-full transition-all"
          :style="{ width: i === 1 ? '24px' : '8px', backgroundColor: i === 1 ? 'var(--accent)' : 'var(--border-soft)' }"
        />
      </div>
      <p class="eyebrow">Setup · Step 1 of 4</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Select Your Subjects
      </h1>
      <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
        Choose the subjects you are preparing for the BECE exam
      </p>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-7">
      <div v-if="loading" class="space-y-3">
        <div v-for="i in 4" :key="i" class="h-16 rounded-2xl animate-pulse"
          :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>

      <div v-else class="space-y-2 max-w-lg">
        <button
          v-for="s in subjects"
          :key="s.id"
          class="subject-row w-full flex items-center gap-4 px-5 py-4 rounded-2xl border text-left"
          :class="{ selected: selected.includes(s.id) }"
          :style="{
            borderColor: selected.includes(s.id) ? 'var(--accent)' : 'var(--border-soft)',
            backgroundColor: selected.includes(s.id) ? 'var(--accent-glow)' : 'var(--surface)',
          }"
          @click="toggle(s.id)"
        >
          <div class="subj-icon flex-shrink-0"
            :style="{ backgroundColor: selected.includes(s.id) ? 'var(--accent)' : 'var(--border-soft)',
                       color: selected.includes(s.id) ? 'white' : 'var(--ink-secondary)' }">
            {{ s.code?.charAt(0) || s.name.charAt(0) }}
          </div>
          <div class="flex-1">
            <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ s.name }}</p>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ s.code }}</p>
          </div>
          <div v-if="selected.includes(s.id)"
            class="w-5 h-5 rounded-full flex items-center justify-center text-white text-[10px] font-bold flex-shrink-0"
            :style="{ backgroundColor: 'var(--accent)' }">✓</div>
        </button>
      </div>
    </div>

    <!-- Footer -->
    <div class="flex-shrink-0 px-7 py-4 border-t flex items-center justify-between"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
      <button class="back-btn" @click="router.push('/student/onboarding/welcome')">← Back</button>
      <button
        class="continue-btn"
        :disabled="selected.length === 0"
        @click="router.push('/student/onboarding/content-packs')"
      >
        Continue with {{ selected.length }} subject{{ selected.length !== 1 ? 's' : '' }} →
      </button>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
  margin-bottom: 4px;
}

.subject-row {
  cursor: pointer;
  transition: all 120ms;
}
.subject-row:not(.selected):hover { background-color: var(--paper) !important; }

.subj-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  font-weight: 800;
  transition: all 120ms;
}

.back-btn {
  padding: 9px 18px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  background: transparent;
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 100ms;
}
.back-btn:hover { background: var(--border-soft); color: var(--ink); }

.continue-btn {
  padding: 10px 24px;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  background: var(--accent);
  color: white;
  border: none;
  transition: opacity 140ms;
}
.continue-btn:hover:not(:disabled) { opacity: 0.87; }
.continue-btn:disabled { opacity: 0.35; cursor: not-allowed; }
</style>


