<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listLinkedStudents } from '@/ipc/identity'
import { getParentProductSurface, type ParentProductSurfaceDto } from '@/ipc/product'
import type { AccountSummaryDto } from '@/types'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppSelect from '@/components/ui/AppSelect.vue'

const auth = useAuthStore()
const router = useRouter()

const loadingChildren = ref(true)
const loadingSurface = ref(false)
const error = ref('')
const children = ref<AccountSummaryDto[]>([])
const selectedStudentId = ref('')
const surface = ref<ParentProductSurfaceDto | null>(null)

const studentOptions = computed(() =>
  children.value.map((child) => ({ value: String(child.id), label: child.display_name })),
)

onMounted(() => {
  void loadChildren()
})

async function loadChildren() {
  if (!auth.currentAccount) {
    loadingChildren.value = false
    return
  }

  loadingChildren.value = true
  error.value = ''

  try {
    children.value = await listLinkedStudents(auth.currentAccount.id)
    if (children.value[0]) {
      selectedStudentId.value = String(children.value[0].id)
      await loadSurface()
    }
  } catch (err) {
    error.value = extractError(err, 'Failed to load linked children')
  }

  loadingChildren.value = false
}

async function loadSurface() {
  if (!auth.currentAccount) return

  loadingSurface.value = true
  error.value = ''

  try {
    surface.value = await getParentProductSurface(
      auth.currentAccount.id,
      selectedStudentId.value ? Number(selectedStudentId.value) : null,
    )
  } catch (err) {
    error.value = extractError(err, 'Failed to load concierge guidance')
  }

  loadingSurface.value = false
}

function handleStudentChange(value: string | number) {
  selectedStudentId.value = String(value)
  void loadSurface()
}

function priorityColor(value: string) {
  const normalized = value.toLowerCase()
  if (normalized === 'high') return 'danger'
  if (normalized === 'medium') return 'warm'
  return 'accent'
}

function extractError(err: unknown, fallback: string) {
  if (typeof err === 'string') return err
  if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
    return err.message
  }
  return fallback
}
</script>

<template>
  <div class="max-w-6xl mx-auto reveal-stagger">
    <PageHeader
      title="Concierge"
      subtitle="Translated guidance for the selected child so you can act without decoding raw platform data."
    >
      <template #actions>
        <div class="min-w-[220px]" v-if="studentOptions.length">
          <AppSelect
            :model-value="selectedStudentId"
            label="Child"
            :options="studentOptions"
            @update:model-value="handleStudentChange"
          />
        </div>
      </template>
    </PageHeader>

    <div v-if="error" class="mb-4 p-3 rounded-lg text-sm" :style="{ backgroundColor: 'rgba(194,65,12,0.08)', color: 'var(--warm)' }">
      {{ error }}
    </div>

    <div v-if="loadingChildren || loadingSurface" class="space-y-4">
      <div class="h-36 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <div v-for="i in 3" :key="i" class="h-40 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
    </div>

    <div v-else-if="children.length === 0" class="text-center py-16">
      <p class="text-sm font-medium mb-2" :style="{ color: 'var(--text)' }">No child accounts linked yet.</p>
      <p class="text-sm mb-4" :style="{ color: 'var(--ink-muted)' }">
        Link or create a child account first so the concierge can translate progress into parent guidance.
      </p>
      <AppButton variant="primary" @click="router.push('/parent/children')">
        Go To Children
      </AppButton>
    </div>

    <div v-else-if="surface" class="space-y-6">
      <AppCard padding="lg" glow="accent">
        <div class="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-4">
          <div>
            <div class="flex items-center gap-2 mb-2">
              <h2 class="font-display text-2xl font-bold" :style="{ color: 'var(--text)' }">
                {{ surface.student_focus?.student_name ?? 'No child selected' }}
              </h2>
              <AppBadge
                v-if="surface.student_focus"
                :color="(surface.student_focus.overall_readiness_band.toLowerCase().includes('risk') ? 'danger' : 'accent') as any"
                size="sm"
              >
                {{ surface.student_focus.overall_readiness_band.replace(/_/g, ' ') }}
              </AppBadge>
            </div>
            <p class="text-sm leading-relaxed mb-3" :style="{ color: 'var(--ink-secondary)' }">
              {{ surface.directives[0]?.summary ?? surface.student_focus?.weekly_memo ?? 'No concierge summary is available yet.' }}
            </p>
            <div v-if="surface.student_focus?.active_risks.length" class="flex flex-wrap gap-2">
              <AppBadge
                v-for="risk in surface.student_focus.active_risks.slice(0, 3)"
                :key="risk.title"
                :color="(risk.severity === 'high' || risk.severity === 'critical' ? 'danger' : 'warm') as any"
                size="xs"
              >
                {{ risk.title }}
              </AppBadge>
            </div>
          </div>

          <div class="flex flex-wrap gap-2">
            <AppButton variant="primary" size="sm" @click="router.push('/parent/child/' + selectedStudentId)">
              Open Child Dashboard
            </AppButton>
            <AppButton variant="secondary" size="sm" @click="router.push('/parent/attention')">
              Attention Needed
            </AppButton>
          </div>
        </div>
      </AppCard>

      <div v-if="surface.insight_cards.length" class="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <AppCard
          v-for="card in surface.insight_cards"
          :key="card.card_key"
          padding="lg"
        >
          <div class="flex items-start justify-between gap-3 mb-3">
            <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ card.title }}</p>
            <AppBadge :color="priorityColor(card.tone) as any" size="xs">
              {{ card.tone.replace(/_/g, ' ') }}
            </AppBadge>
          </div>
          <p class="text-sm leading-relaxed mb-4" :style="{ color: 'var(--ink-secondary)' }">{{ card.summary }}</p>
          <div v-if="card.metric_value !== null && card.metric_label" class="rounded-xl p-3" :style="{ backgroundColor: 'var(--paper)' }">
            <p class="text-[11px] uppercase font-semibold mb-1" :style="{ color: 'var(--ink-muted)' }">{{ card.metric_label.replace(/_/g, ' ') }}</p>
            <p class="font-display text-2xl font-bold" :style="{ color: 'var(--text)' }">{{ card.metric_value }}</p>
          </div>
        </AppCard>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <AppCard padding="lg">
          <p class="text-xs uppercase font-semibold mb-3" :style="{ color: 'var(--ink-muted)' }">Recommended Parent Actions</p>
          <div class="space-y-3">
            <div
              v-for="action in surface.action_recommendations"
              :key="action.recommendation_key"
              class="rounded-xl p-4"
              :style="{ backgroundColor: 'var(--paper)' }"
            >
              <div class="flex items-center justify-between gap-2 mb-2">
                <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ action.label }}</p>
                <AppBadge :color="priorityColor(action.urgency) as any" size="xs">
                  {{ action.urgency }}
                </AppBadge>
              </div>
              <p class="text-sm mb-2" :style="{ color: 'var(--ink-secondary)' }">{{ action.summary }}</p>
              <p class="text-xs mb-3" :style="{ color: 'var(--ink-muted)' }">{{ action.rationale }}</p>
              <AppButton variant="secondary" size="sm" @click="router.push('/parent')">
                Open Family Overview
              </AppButton>
            </div>
          </div>
        </AppCard>

        <AppCard padding="lg">
          <p class="text-xs uppercase font-semibold mb-3" :style="{ color: 'var(--ink-muted)' }">Why The Concierge Is Saying This</p>
          <div class="space-y-3">
            <div
              v-for="explanation in surface.audience_explanations"
              :key="explanation.headline"
              class="rounded-xl p-4"
              :style="{ backgroundColor: 'var(--paper)' }"
            >
              <p class="text-sm font-semibold mb-2" :style="{ color: 'var(--text)' }">{{ explanation.headline }}</p>
              <p class="text-sm mb-3" :style="{ color: 'var(--ink-secondary)' }">{{ explanation.summary }}</p>
              <div class="space-y-1">
                <p
                  v-for="point in explanation.supporting_points"
                  :key="point"
                  class="text-xs"
                  :style="{ color: 'var(--ink-muted)' }"
                >
                  {{ point }}
                </p>
              </div>
            </div>
          </div>
        </AppCard>
      </div>

      <div v-if="surface.directives.length > 1" class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <AppCard
          v-for="directive in surface.directives.slice(1)"
          :key="directive.title"
          padding="md"
        >
          <div class="flex items-center justify-between gap-2 mb-2">
            <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ directive.title }}</p>
            <AppBadge :color="priorityColor(directive.priority) as any" size="xs">
              {{ directive.priority }}
            </AppBadge>
          </div>
          <p class="text-sm" :style="{ color: 'var(--ink-secondary)' }">{{ directive.summary }}</p>
        </AppCard>
      </div>
    </div>
  </div>
</template>
