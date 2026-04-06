<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getCoachNextAction,
  getStudentDashboard,
  getPriorityTopics,
  type CoachNextActionDto,
  type StudentDashboardDto,
  type TopicCaseDto,
} from '@/ipc/coach'
import MasteryBadge from '@/components/viz/MasteryBadge.vue'
import {
  PhPencilSimple, PhClockCountdown, PhTarget, PhBrain,
  PhGameController, PhChartBar, PhArrowRight, PhLightning,
  PhBookOpen, PhStar, PhFlame, PhTrendUp,
} from '@phosphor-icons/vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const error = ref('')

const nextAction = ref<CoachNextActionDto | null>(null)
const dashboard = ref<StudentDashboardDto | null>(null)
const topicCases = ref<TopicCaseDto[]>([])

const greeting = computed(() => {
  const h = new Date().getHours()
  if (h < 12) return 'Good morning'
  if (h < 17) return 'Good afternoon'
  return 'Good evening'
})

const dateLabel = computed(() => {
  return new Date().toLocaleDateString('en-GB', { weekday: 'long', day: 'numeric', month: 'long' })
})

onMounted(async () => {
  if (!auth.currentAccount) return
  const sid = auth.currentAccount.id
  try {
    const [action, dash, topics] = await Promise.all([
      getCoachNextAction(sid),
      getStudentDashboard(sid),
      getPriorityTopics(sid, 6),
    ])
    nextAction.value = action
    dashboard.value = dash
    topicCases.value = topics
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load'
  } finally {
    loading.value = false
  }
})

function startAction() {
  if (nextAction.value?.route) router.push(nextAction.value.route)
}

function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}

function bandLabel(band: string): string {
  if (band === 'strong') return 'Strong'
  if (band === 'developing') return 'Developing'
  return 'Weak'
}

const navLinks = [
  { icon: PhPencilSimple, label: 'Practice', sub: 'Subject drill', to: '/student/practice' },
  { icon: PhClockCountdown, label: 'Mock', sub: 'Exam simulation', to: '/student/mock' },
  { icon: PhBrain, label: 'Memory', sub: 'Spaced review', to: '/student/memory' },
  { icon: PhTarget, label: 'Gap Scan', sub: 'Find blind spots', to: '/student/knowledge-gap' },
  { icon: PhStar, label: 'Journey', sub: 'Learning path', to: '/student/journey' },
  { icon: PhChartBar, label: 'Progress', sub: 'Analytics', to: '/student/progress' },
]
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Greeting strip -->
    <div
      class="flex-shrink-0 px-7 py-4 border-b flex items-center justify-between"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <div>
        <p class="text-[11px] font-semibold" :style="{ color: 'var(--ink-muted)' }">{{ dateLabel }}</p>
        <h1 class="font-display text-xl font-bold mt-0.5" :style="{ color: 'var(--ink)' }">
          {{ greeting }}, {{ dashboard?.student_name || auth.currentAccount?.display_name || 'Student' }}
        </h1>
      </div>
      <div v-if="dashboard?.subjects?.length" class="flex items-center gap-5">
        <div
          v-for="s in dashboard.subjects.slice(0, 3)"
          :key="s.subject_id"
          class="text-center"
        >
          <p class="text-base font-black tabular-nums" :style="{ color: 'var(--ink)' }">
            {{ Math.round(s.mastered_topic_count / Math.max(s.total_topic_count, 1) * 100) }}%
          </p>
          <p class="text-[9px] font-bold uppercase tracking-wider" :style="{ color: 'var(--ink-muted)' }">{{ s.subject_name }}</p>
        </div>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="flex-1 p-7 grid grid-cols-12 gap-5">
      <div class="col-span-7 space-y-4">
        <div class="h-48 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        <div class="h-40 rounded-2xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
      <div class="col-span-5 space-y-3">
        <div v-for="i in 4" :key="i" class="h-16 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="flex-1 flex items-center justify-center">
      <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">{{ error }}</p>
    </div>

    <!-- Body -->
    <div v-else class="flex-1 overflow-hidden flex">

      <!-- Left: directive + topics -->
      <div class="flex-1 overflow-y-auto p-7 space-y-5">

        <!-- Coach directive -->
        <div v-if="nextAction"
          class="directive-card rounded-2xl border cursor-pointer"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
          @click="startAction"
        >
          <div class="directive-inner p-6">
            <div class="flex items-start justify-between gap-4">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-3">
                  <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider"
                    :style="{ backgroundColor: 'var(--accent-glow)', color: 'var(--accent)' }">
                    <PhLightning :size="9" weight="fill" />
                    Coach Directive
                  </div>
                </div>
                <h2 class="font-display text-2xl font-bold leading-snug mb-2" :style="{ color: 'var(--ink)' }">
                  {{ nextAction.title }}
                </h2>
                <p class="text-sm leading-relaxed" :style="{ color: 'var(--ink-secondary)' }">
                  {{ nextAction.subtitle }}
                </p>
              </div>
              <div v-if="nextAction.estimated_minutes"
                class="flex-shrink-0 text-center px-3 py-2 rounded-xl border"
                :style="{ borderColor: 'var(--border-soft)' }">
                <p class="text-lg font-black tabular-nums" :style="{ color: 'var(--ink)' }">{{ nextAction.estimated_minutes }}</p>
                <p class="text-[9px] uppercase font-bold tracking-wide" :style="{ color: 'var(--ink-muted)' }">min</p>
              </div>
            </div>
            <div class="mt-5 flex items-center gap-3">
              <button
                class="cta-btn flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-bold"
                :style="{ backgroundColor: 'var(--accent)', color: 'white' }"
                @click.stop="startAction"
              >
                Start Now
                <PhArrowRight :size="14" weight="bold" />
              </button>
              <span class="text-xs capitalize px-3 py-1.5 rounded-lg"
                :style="{ backgroundColor: 'var(--paper)', color: 'var(--ink-muted)' }">
                {{ nextAction.action_type?.replace(/_/g, ' ') }}
              </span>
            </div>
          </div>
        </div>

        <!-- Empty state if no directive -->
        <div v-else
          class="rounded-2xl border p-8 text-center"
          :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
        >
          <p class="font-display text-lg font-bold mb-1" :style="{ color: 'var(--ink)' }">All caught up</p>
          <p class="text-sm" :style="{ color: 'var(--ink-muted)' }">Your coach has no pending actions. Keep practising!</p>
        </div>

        <!-- Priority topics -->
        <div v-if="topicCases.length">
          <p class="section-label mb-3">Priority Topics</p>
          <div class="space-y-1.5">
            <div
              v-for="topic in topicCases"
              :key="topic.topic_id"
              class="topic-row flex items-center gap-3 px-4 py-3 rounded-xl border cursor-pointer"
              :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
              @click="router.push('/student/practice')"
            >
              <MasteryBadge :state="topic.mastery_state" size="sm" />
              <div class="flex-1 min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ topic.topic_name }}</p>
                <p class="text-[10px] capitalize" :style="{ color: 'var(--ink-muted)' }">
                  {{ topic.intervention_mode.replace(/_/g, ' ') }} · gap {{ formatBp(topic.gap_score) }}
                </p>
              </div>
              <span
                class="text-[9px] font-bold uppercase px-2 py-0.5 rounded-full"
                :style="{
                  backgroundColor: topic.intervention_urgency === 'high' ? 'rgba(194,65,12,0.1)' : 'var(--paper)',
                  color: topic.intervention_urgency === 'high' ? 'var(--warm)' : 'var(--ink-muted)',
                }"
              >{{ topic.intervention_urgency }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Right: navigation + subjects -->
      <div
        class="w-72 flex-shrink-0 flex flex-col overflow-hidden border-l"
        :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
      >
        <!-- Quick nav -->
        <div class="flex-shrink-0 p-4 border-b" :style="{ borderColor: 'var(--border-soft)' }">
          <p class="section-label mb-3">Navigate</p>
          <div class="grid grid-cols-2 gap-2">
            <button
              v-for="link in navLinks"
              :key="link.to"
              class="nav-tile flex flex-col gap-1 p-3 rounded-xl border text-left"
              :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }"
              @click="router.push(link.to)"
            >
              <component :is="link.icon" :size="16" weight="duotone"
                :style="{ color: 'var(--ink-secondary)' }" />
              <p class="text-[11px] font-bold" :style="{ color: 'var(--ink)' }">{{ link.label }}</p>
              <p class="text-[9px]" :style="{ color: 'var(--ink-muted)' }">{{ link.sub }}</p>
            </button>
          </div>
        </div>

        <!-- Subject overview -->
        <div v-if="dashboard?.subjects?.length" class="flex-1 overflow-y-auto p-4">
          <p class="section-label mb-3">Subjects</p>
          <div class="space-y-2">
            <div
              v-for="s in dashboard.subjects"
              :key="s.subject_id"
              class="subject-row flex items-center gap-3 px-3 py-3 rounded-xl border cursor-pointer"
              :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--paper)' }"
              @click="router.push('/student/progress')"
            >
              <div class="w-8 h-8 rounded-lg flex items-center justify-center text-xs font-black flex-shrink-0"
                :style="{ backgroundColor: 'var(--border-soft)', color: 'var(--ink)' }">
                {{ s.subject_name?.charAt(0) }}
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between mb-1">
                  <p class="text-[11px] font-bold truncate" :style="{ color: 'var(--ink)' }">{{ s.subject_name }}</p>
                  <p class="text-[10px] font-semibold" :style="{ color: 'var(--ink-secondary)' }">
                    {{ Math.round(s.mastered_topic_count / Math.max(s.total_topic_count, 1) * 100) }}%
                  </p>
                </div>
                <div class="h-1 rounded-full overflow-hidden" :style="{ backgroundColor: 'var(--border-soft)' }">
                  <div class="h-full rounded-full"
                    :style="{
                      width: Math.round(s.mastered_topic_count / Math.max(s.total_topic_count, 1) * 100) + '%',
                      backgroundColor: s.readiness_band === 'strong' ? 'var(--accent)' : s.readiness_band === 'developing' ? 'var(--gold)' : 'var(--warm)',
                    }"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.directive-card {
  transition: border-color 140ms ease, box-shadow 140ms ease;
}
.directive-card:hover {
  border-color: var(--accent) !important;
  box-shadow: 0 0 0 1px var(--accent-glow);
}

.cta-btn {
  transition: opacity 120ms ease, transform 120ms ease;
}
.cta-btn:hover { opacity: 0.88; transform: translateY(-1px); }

.topic-row {
  transition: background-color 100ms ease, border-color 100ms ease;
}
.topic-row:hover {
  background-color: var(--paper) !important;
  border-color: var(--border-strong) !important;
}

.nav-tile {
  cursor: pointer;
  transition: background-color 100ms ease, border-color 100ms ease;
}
.nav-tile:hover {
  background-color: var(--accent-glow) !important;
  border-color: var(--accent) !important;
}
.nav-tile:hover p { color: var(--accent) !important; }

.subject-row {
  transition: background-color 100ms ease;
}
.subject-row:hover { background-color: var(--surface) !important; }
</style>
