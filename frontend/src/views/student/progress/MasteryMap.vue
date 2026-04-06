<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getPriorityTopics, type TopicCaseDto } from '@/ipc/coach'
import KnowledgeMap, { type MapNode, type MapLink } from '@/components/viz/KnowledgeMap.vue'
import TopicStatusCard from '@/components/viz/TopicStatusCard.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const selectedNodeId = ref<number | null>(null)
const topics = ref<TopicCaseDto[]>([])
const mapNodes = ref<MapNode[]>([])
const mapLinks = ref<MapLink[]>([])
const selectedTopic = ref<TopicCaseDto | null>(null)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    topics.value = await getPriorityTopics(auth.currentAccount.id, 20)
    mapNodes.value = topics.value.map(t => ({
      id: t.topic_id,
      name: t.topic_name,
      masteryState: t.mastery_state,
      score: t.mastery_score,
      type: 'topic' as const,
    }))
    mapLinks.value = topics.value.slice(0, -1).map((t, i) => ({
      source: t.topic_id,
      target: topics.value[i + 1].topic_id,
      type: 'related' as const,
    }))
  } catch {}
  loading.value = false
})

function handleSelect(nodeId: number) {
  selectedNodeId.value = nodeId
  selectedTopic.value = topics.value.find(t => t.topic_id === nodeId) ?? null
}

function masteryColor(state: string): string {
  if (state === 'mastered') return 'var(--accent)'
  if (state === 'learning') return 'var(--gold)'
  return 'var(--warm)'
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b flex items-center justify-between"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="eyebrow">Progress</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
          Mastery Map
        </h1>
        <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">
          Nodes illuminate as understanding deepens
        </p>
      </div>
      <div class="flex gap-2">
        <button class="nav-pill" @click="router.push('/student/progress')">Overview</button>
        <button class="nav-pill" @click="router.push('/student/progress/history')">History</button>
        <button class="nav-pill" @click="router.push('/student/progress/analytics')">Analytics</button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-6">
      <div class="h-full rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <!-- Body -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Map -->
      <div class="flex-1 overflow-hidden p-6">
        <KnowledgeMap
          :nodes="mapNodes"
          :links="mapLinks"
          :selected-node-id="selectedNodeId"
          :height="500"
          @select-node="handleSelect"
        />
      </div>

      <!-- Right: topic detail -->
      <div
        class="w-72 flex-shrink-0 border-l flex flex-col overflow-hidden"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <div class="px-5 py-4 border-b flex-shrink-0" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label">Topic Detail</p>
        </div>

        <div class="flex-1 overflow-y-auto p-4">
          <div v-if="selectedTopic" class="space-y-4">
            <TopicStatusCard
              :topic-name="selectedTopic.topic_name"
              :mastery-state="selectedTopic.mastery_state"
              :mastery-score="selectedTopic.mastery_score"
              :gap-score="selectedTopic.gap_score"
              clickable
            />

            <div class="px-4 py-3 rounded-xl border" :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }">
              <p class="text-xs font-bold mb-1" :style="{ color: 'var(--ink)' }">{{ selectedTopic.intervention_mode }}</p>
              <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ selectedTopic.intervention_reason }}</p>
            </div>

            <div class="flex items-center justify-between px-1">
              <span class="text-[10px] font-semibold uppercase tracking-wide" :style="{ color: 'var(--ink-muted)' }">Mastery</span>
              <span class="text-sm font-black tabular-nums" :style="{ color: masteryColor(selectedTopic.mastery_state) }">
                {{ Math.round(selectedTopic.mastery_score / 100) }}%
              </span>
            </div>
            <div class="h-1.5 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
              <div class="h-full rounded-full"
                :style="{ width: (selectedTopic.mastery_score / 100) + '%', backgroundColor: masteryColor(selectedTopic.mastery_state) }" />
            </div>

            <button
              class="start-btn w-full"
              @click="router.push('/student/practice')"
            >Practice This Topic →</button>
          </div>

          <div v-else class="py-16 text-center px-4">
            <div class="w-12 h-12 rounded-full border-2 flex items-center justify-center mx-auto mb-3"
              :style="{ borderColor: 'var(--border-soft)' }">
              <span class="text-lg" :style="{ color: 'var(--ink-muted)' }">◎</span>
            </div>
            <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">
              Click any node to explore topic details
            </p>
          </div>
        </div>

        <!-- Legend -->
        <div class="px-5 py-4 border-t space-y-2" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label mb-2">Legend</p>
          <div v-for="item in [
            { label: 'Mastered', color: 'var(--accent)' },
            { label: 'Learning', color: 'var(--gold)' },
            { label: 'Gap', color: 'var(--warm)' },
          ]" :key="item.label" class="flex items-center gap-2">
            <div class="w-2.5 h-2.5 rounded-full flex-shrink-0" :style="{ backgroundColor: item.color }" />
            <span class="text-[11px]" :style="{ color: 'var(--ink-secondary)' }">{{ item.label }}</span>
          </div>
        </div>
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
  color: var(--ink-muted);
  margin-bottom: 4px;
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.nav-pill {
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid var(--border-soft);
  transition: all 120ms;
}
.nav-pill:hover { background: var(--accent-glow); color: var(--accent); border-color: var(--accent); }

.start-btn {
  padding: 10px;
  border-radius: 10px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  background: var(--accent);
  color: white;
  border: none;
  transition: opacity 140ms;
}
.start-btn:hover { opacity: 0.87; }
</style>
