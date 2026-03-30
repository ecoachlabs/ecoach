<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getPriorityTopics, type TopicCaseDto } from '@/ipc/coach'
import KnowledgeMap, { type MapNode, type MapLink } from '@/components/viz/KnowledgeMap.vue'
import TopicStatusCard from '@/components/viz/TopicStatusCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import PageHeader from '@/components/layout/PageHeader.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const selectedNodeId = ref<number | null>(null)
const topics = ref<TopicCaseDto[]>([])
const mapNodes = ref<MapNode[]>([])
const mapLinks = ref<MapLink[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    topics.value = await getPriorityTopics(auth.currentAccount.id, 20)
    // Convert topic cases to map nodes
    mapNodes.value = topics.value.map(t => ({
      id: t.topic_id,
      name: t.topic_name,
      masteryState: t.mastery_state,
      score: t.mastery_score,
      type: 'topic' as const,
    }))
    // Generate links between related topics (simplified)
    mapLinks.value = topics.value.slice(0, -1).map((t, i) => ({
      source: t.topic_id,
      target: topics.value[i + 1].topic_id,
      type: 'related' as const,
    }))
  } catch {}
  loading.value = false
})

const selectedTopic = ref<TopicCaseDto | null>(null)
function handleSelect(nodeId: number) {
  selectedNodeId.value = nodeId
  selectedTopic.value = topics.value.find(t => t.topic_id === nodeId) ?? null
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-5xl mx-auto reveal-stagger">
    <PageHeader title="Mastery Map" subtitle="Visual map of your knowledge. Nodes glow as understanding deepens." back-to="/student/progress">
      <template #actions>
        <AppButton variant="secondary" size="sm" @click="router.push('/student/progress/analytics')">Analytics</AppButton>
      </template>
    </PageHeader>

    <div v-if="loading" class="h-[400px] rounded-xl animate-pulse" :style="{backgroundColor:'var(--border-soft)'}" />

    <div v-else class="grid grid-cols-[1fr_300px] gap-4">
      <!-- Map -->
      <KnowledgeMap :nodes="mapNodes" :links="mapLinks" :selected-node-id="selectedNodeId" :height="450" @select-node="handleSelect" />

      <!-- Selected topic detail -->
      <div v-if="selectedTopic" class="space-y-3">
        <TopicStatusCard :topic-name="selectedTopic.topic_name" :mastery-state="selectedTopic.mastery_state"
          :mastery-score="selectedTopic.mastery_score" :gap-score="selectedTopic.gap_score" clickable />
        <div class="px-3 py-2 rounded-[var(--radius-md)] text-xs" :style="{backgroundColor:'var(--primary-light)',color:'var(--text-2)'}">
          <p class="font-semibold mb-1">{{ selectedTopic.intervention_mode }}</p>
          <p>{{ selectedTopic.intervention_reason }}</p>
        </div>
        <AppButton variant="primary" size="sm" class="w-full">Start Session →</AppButton>
      </div>
      <div v-else class="flex items-center justify-center text-xs" :style="{color:'var(--text-3)'}">
        Click a node to see details
      </div>
    </div>
  </div>
</template>
