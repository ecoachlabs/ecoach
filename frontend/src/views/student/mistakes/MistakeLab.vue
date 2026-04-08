<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getRevengeQueue, type RevengeQueueItemDto } from '@/ipc/coach'
import AppTabs from '@/components/ui/AppTabs.vue'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()
const loading = ref(true)
const mistakes = ref<RevengeQueueItemDto[]>([])
const activeTab = ref('pending')

const tabs = [
  { key: 'pending', label: 'Pending' },
  { key: 'beaten', label: 'Beaten' },
  { key: 'patterns', label: 'Patterns' },
]

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    mistakes.value = await getRevengeQueue(auth.currentAccount.id)
  } catch (e) {
    console.error('Failed to load mistake queue:', e)
  }
  loading.value = false
})

const pending = computed(() => mistakes.value.filter(m => !m.is_beaten))
const beaten = computed(() => mistakes.value.filter(m => m.is_beaten))

const errorPatterns = computed(() => {
  const counts: Record<string, number> = {}
  for (const m of mistakes.value) {
    const type = m.original_error_type ?? 'unknown'
    counts[type] = (counts[type] ?? 0) + 1
  }
  return Object.entries(counts).map(([type, count]) => ({ type, count }))
    .sort((a, b) => b.count - a.count)
})

const topicPatterns = computed(() => {
  const map: Record<number, { topic_id: number; count: number; pending: number }> = {}
  for (const m of mistakes.value) {
    if (!m.topic_id) continue
    if (!map[m.topic_id]) map[m.topic_id] = { topic_id: m.topic_id, count: 0, pending: 0 }
    map[m.topic_id].count++
    if (!m.is_beaten) map[m.topic_id].pending++
  }
  return Object.values(map).sort((a, b) => b.pending - a.pending)
})

function errorTypeLabel(type: string): string {
  return type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

watch([() => route.hash, loading], () => {
  if (!loading.value && route.hash) {
    nextTick(() => {
      document.getElementById(route.hash.slice(1))?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    })
  }
}, { immediate: true })
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 flex items-center justify-between px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Mistake Lab</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Every mistake is data
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">Understand your patterns to break them</p>
      </div>
      <div class="flex items-center gap-5">
        <div class="text-right">
          <p class="text-2xl font-black tabular-nums" :style="{ color: 'var(--warm)' }">{{ pending.length }}</p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Pending</p>
        </div>
        <div class="text-right">
          <p class="text-2xl font-black tabular-nums" :style="{ color: 'var(--accent)' }">{{ beaten.length }}</p>
          <p class="text-[10px] uppercase font-semibold" :style="{ color: 'var(--ink-muted)' }">Beaten</p>
        </div>
      </div>
    </div>

    <!-- Tabs -->
    <div id="retry-zone" class="flex-shrink-0 px-7 pt-4 border-b" :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
      <AppTabs :tabs="tabs" v-model="activeTab" />
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-6 space-y-3">
      <div v-for="i in 5" :key="i" class="h-16 rounded-xl animate-pulse"
        :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Content -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Pending Tab -->
      <div v-if="activeTab === 'pending'" class="flex-1 overflow-y-auto">
        <div v-if="!pending.length" class="flex flex-col items-center justify-center h-full gap-4 text-center px-8">
          <div class="w-14 h-14 rounded-2xl flex items-center justify-center text-xl"
            :style="{ background: 'rgba(13,148,136,0.1)', color: 'var(--accent)' }">✓</div>
          <h3 class="font-display text-lg font-bold" :style="{ color: 'var(--ink)' }">No pending mistakes</h3>
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Practice more to track error patterns here.</p>
          <button class="px-5 py-2.5 rounded-xl font-semibold text-sm"
            :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
            @click="router.push('/student/practice')">Start Practice</button>
        </div>

        <div v-else class="divide-y divide-[var(--border-soft)]" :style="{ borderColor: 'transparent' }">
          <div v-for="m in pending" :key="m.id" class="mistake-row px-7 py-4 flex items-start gap-4">
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-xs flex-shrink-0 mt-0.5"
              :style="{ background: 'rgba(194,65,12,0.1)', color: 'var(--warm)' }">✕</div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold line-clamp-2 mb-1" :style="{ color: 'var(--ink)' }">
                {{ m.question_text ?? 'Question #' + m.question_id }}
              </p>
              <div class="flex items-center gap-2 flex-wrap">
                <span v-if="m.original_error_type"
                  class="text-[10px] font-semibold px-2 py-0.5 rounded-full"
                  :style="{ background: 'var(--paper)', color: 'var(--ink-secondary)', border: '1px solid transparent' }">
                  {{ errorTypeLabel(m.original_error_type) }}
                </span>
                <span v-if="m.original_wrong_answer" class="text-[11px] truncate max-w-xs"
                  :style="{ color: 'var(--warm)' }">
                  You chose: {{ m.original_wrong_answer }}
                </span>
              </div>
            </div>
            <div class="flex-shrink-0">
              <span v-if="m.attempts_to_beat > 0"
                class="text-[10px] font-semibold px-2 py-0.5 rounded-full"
                :style="{ background: 'var(--paper)', color: 'var(--ink-muted)', border: '1px solid transparent' }">
                {{ m.attempts_to_beat }}x tried
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Beaten Tab -->
      <div v-else-if="activeTab === 'beaten'" class="flex-1 overflow-y-auto">
        <div v-if="!beaten.length" class="flex flex-col items-center justify-center h-full gap-4 text-center px-8">
          <p class="text-3xl">🎯</p>
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">No beaten mistakes yet. Keep practising!</p>
        </div>
        <div v-else class="divide-y divide-[var(--border-soft)]" :style="{ borderColor: 'transparent' }">
          <div v-for="m in beaten" :key="m.id" class="mistake-row px-7 py-4 flex items-center gap-4">
            <div class="w-8 h-8 rounded-lg flex items-center justify-center text-xs flex-shrink-0"
              :style="{ background: 'rgba(13,148,136,0.1)', color: 'var(--accent)' }">✓</div>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold line-clamp-1" :style="{ color: 'var(--ink)' }">
                {{ m.question_text ?? 'Question #' + m.question_id }}
              </p>
              <p v-if="m.original_wrong_answer" class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">
                Was choosing: {{ m.original_wrong_answer }}
              </p>
            </div>
            <span class="text-[10px] font-bold px-3 py-1 rounded-full flex-shrink-0"
              :style="{ background: 'rgba(13,148,136,0.1)', color: 'var(--accent)' }">beaten</span>
          </div>
        </div>
      </div>

      <!-- Patterns Tab -->
      <div v-else-if="activeTab === 'patterns'" class="flex-1 overflow-y-auto p-6 space-y-6">
        <div v-if="!errorPatterns.length && !topicPatterns.length"
          class="flex flex-col items-center justify-center h-full gap-4 text-center">
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">
            No error patterns yet. Your patterns will appear after you practice.
          </p>
        </div>

        <template v-else>
          <div v-if="errorPatterns.length">
            <p class="section-label mb-3">By Error Type</p>
            <div class="space-y-2">
              <div v-for="ep in errorPatterns" :key="ep.type"
                class="flex items-center gap-4 px-5 py-3.5 rounded-xl border"
                :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
                <div class="w-2 h-2 rounded-full flex-shrink-0" :style="{ backgroundColor: 'var(--ink-muted)' }" />
                <p class="text-sm font-semibold flex-1" :style="{ color: 'var(--ink)' }">{{ errorTypeLabel(ep.type) }}</p>
                <span class="text-sm font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ ep.count }}</span>
                <button class="text-xs font-semibold px-3 py-1.5 rounded-lg"
                  :style="{ background: 'var(--accent-glow)', color: 'var(--accent)' }"
                  @click="router.push('/student/practice')">Fix →</button>
              </div>
            </div>
          </div>

          <div v-if="topicPatterns.length">
            <p class="section-label mb-3">By Topic Cluster</p>
            <div class="space-y-2">
              <div v-for="tp in topicPatterns" :key="tp.topic_id"
                class="flex items-center gap-4 px-5 py-3.5 rounded-xl border"
                :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }">
                <div class="w-9 h-9 rounded-lg flex items-center justify-center text-sm font-black flex-shrink-0"
                  :style="{ background: 'var(--paper)', color: 'var(--ink)', border: '1px solid transparent' }">
                  {{ tp.pending }}
                </div>
                <div class="flex-1">
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">Topic #{{ tp.topic_id }}</p>
                  <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">
                    {{ tp.count }} mistake{{ tp.count > 1 ? 's' : '' }} · {{ tp.pending }} pending
                  </p>
                </div>
                <button class="text-xs font-semibold px-3 py-1.5 rounded-lg"
                  :style="{ background: 'var(--accent-glow)', color: 'var(--accent)' }"
                  @click="router.push('/student/practice')">Drill →</button>
              </div>
            </div>
          </div>
        </template>
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
  color: var(--warm);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.mistake-row {
  transition: background-color 100ms;
}
.mistake-row:hover { background-color: var(--paper); }
</style>


