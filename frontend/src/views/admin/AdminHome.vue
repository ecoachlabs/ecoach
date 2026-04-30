<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import CmsActionQueue from '@/components/admin/cms/CmsActionQueue.vue'
import CmsMetricStrip from '@/components/admin/cms/CmsMetricStrip.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import type { CmsActionItem, CmsMetricItem } from '@/components/admin/cms/types'
import {
  getAdminQuestionBankStats,
  getSuperAdminControlTower,
  type AdminQuestionBankStatsDto,
  type SuperAdminControlTowerDto,
} from '@/ipc/admin'

const router = useRouter()
const auth = useAuthStore()

const loading = ref(true)
const error = ref('')
const tower = ref<SuperAdminControlTowerDto | null>(null)
const questionStats = ref<AdminQuestionBankStatsDto | null>(null)

const contentHealth = computed(() => tower.value?.content_health)
const hasNoContent = computed(() => (questionStats.value?.total_questions ?? 0) === 0)

const cmsMetrics = computed<CmsMetricItem[]>(() => [
  { label: 'Questions', value: questionStats.value?.total_questions ?? 0, caption: 'Bank inventory' },
  { label: 'Answers', value: questionStats.value?.total_options ?? 0, caption: 'Options stored' },
  { label: 'Sources', value: tower.value?.content_health.source_count ?? 0, caption: 'Raw provenance' },
  { label: 'Pending Review', value: questionStats.value?.pending_review_count ?? 0, tone: 'review' },
  { label: 'Published Packs', value: questionStats.value?.installed_pack_count ?? 0, tone: 'good' },
  { label: 'Attempts', value: questionStats.value?.total_attempts ?? 0, caption: 'Learning signal' },
])

const actionItems = computed<CmsActionItem[]>(() => {
  const items: CmsActionItem[] = []
  if (hasNoContent.value) {
    items.push({
      key: 'no-content',
      title: 'No question content installed',
      summary: 'Install a pack, register sources, or open the editor to create structured questions.',
      tone: 'review',
      actionLabel: 'Install Pack',
      route: '/admin/packs',
    })
  }
  for (const item of tower.value?.action_recommendations ?? []) {
    items.push({
      key: item.recommendation_key,
      title: item.label,
      summary: item.summary,
      tone: 'review',
      actionLabel: 'Open',
      route: '/admin/content',
    })
  }
  return items
})

async function load() {
  const adminId = auth.currentAccount?.id
  if (!adminId) {
    error.value = 'Admin account is not loaded.'
    loading.value = false
    return
  }

  loading.value = true
  error.value = ''
  try {
    const [towerResult, statsResult] = await Promise.all([
      getSuperAdminControlTower(adminId, 12),
      getAdminQuestionBankStats(),
    ])
    tower.value = towerResult
    questionStats.value = statsResult
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load admin data.'
  } finally {
    loading.value = false
  }
}

function openAction(item: CmsActionItem) {
  if (item.route) router.push(item.route)
}

onMounted(load)
</script>

<template>
  <div class="h-full overflow-y-auto p-7" :style="{ backgroundColor: 'var(--paper)' }">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <p class="text-[10px] font-bold uppercase tracking-[0.16em]" :style="{ color: 'var(--ink-muted)' }">Super Admin CMS</p>
        <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Dashboard</h1>
        <p class="text-sm mt-1 max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
          The operating overview for structured content, question inventory, source health, review work, and publishing readiness.
        </p>
      </div>
      <div class="flex gap-2">
        <AppButton variant="secondary" size="sm" @click="load">Refresh</AppButton>
        <AppButton variant="secondary" size="sm" @click="router.push('/admin/content-editor')">Content Editor</AppButton>
        <AppButton variant="primary" size="sm" @click="router.push('/admin/content')">Add Source</AppButton>
      </div>
    </div>

    <div v-if="loading" class="grid grid-cols-3 gap-3">
      <div v-for="i in 6" :key="i" class="h-28 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--surface)' }" />
    </div>

    <AppCard v-else-if="error" padding="lg">
      <p class="text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    </AppCard>

    <template v-else>
      <CmsMetricStrip :items="cmsMetrics" class="mb-6" />

      <AppCard v-if="hasNoContent" padding="lg" class="mb-5">
        <div class="flex items-start justify-between gap-4">
          <div>
            <CmsStatusBadge status="needs_review" size="sm" />
            <h2 class="text-base font-bold mt-3 mb-1" :style="{ color: 'var(--ink)' }">No question content is installed yet</h2>
            <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
              Install a pack or register source material, then use the editor and seeding tools to grow the bank.
            </p>
          </div>
          <div class="flex gap-2 shrink-0">
            <AppButton variant="secondary" size="sm" @click="router.push('/admin/packs')">Install Pack</AppButton>
            <AppButton variant="primary" size="sm" @click="router.push('/admin/content')">Add Source</AppButton>
          </div>
        </div>
      </AppCard>

      <div class="grid grid-cols-1 xl:grid-cols-3 gap-4 mb-6">
        <AppCard padding="md">
          <h2 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">Content Health</h2>
          <div class="grid grid-cols-2 gap-3 text-sm">
            <div>
              <p class="text-xl font-bold tabular-nums" :style="{ color: 'var(--ink)' }">{{ contentHealth?.average_quality_bp ?? 0 }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Avg quality bp</p>
            </div>
            <div>
              <p class="text-xl font-bold tabular-nums" :style="{ color: 'var(--ink)' }">{{ contentHealth?.active_mission_count ?? 0 }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Active missions</p>
            </div>
            <div>
              <p class="text-xl font-bold tabular-nums" :style="{ color: 'var(--ink)' }">{{ contentHealth?.stale_source_count ?? 0 }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Stale sources</p>
            </div>
            <div>
              <p class="text-xl font-bold tabular-nums" :style="{ color: 'var(--ink)' }">{{ contentHealth?.blocked_publish_count ?? 0 }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Blocked publishes</p>
            </div>
          </div>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">Question Review</h2>
          <div class="space-y-2">
            <div v-for="item in questionStats?.by_review_status ?? []" :key="item.label" class="flex items-center justify-between text-sm">
              <span class="capitalize" :style="{ color: 'var(--ink)' }">{{ item.label.replaceAll('_', ' ') }}</span>
              <CmsStatusBadge :status="item.label" />
            </div>
            <p v-if="!questionStats?.by_review_status.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">No classified questions yet.</p>
          </div>
        </AppCard>

        <AppCard padding="md">
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--ink-muted)' }">Needs Attention</h2>
            <AppButton variant="ghost" size="sm" @click="router.push('/admin/questions/review')">Review Queue</AppButton>
          </div>
          <CmsActionQueue :items="actionItems" empty-text="No urgent CMS actions." @open="openAction" />
        </AppCard>
      </div>

      <div class="grid grid-cols-1 xl:grid-cols-2 gap-4">
        <AppCard padding="md">
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--ink-muted)' }">Source Governance</h2>
            <AppButton variant="ghost" size="sm" @click="router.push('/admin/content')">Manage</AppButton>
          </div>
          <div class="space-y-2">
            <div v-for="source in tower?.source_objects ?? []" :key="source.source_upload_id ?? source.title" class="flex items-center gap-3 text-sm">
              <CmsStatusBadge :status="source.source_kind" />
              <span class="flex-1 truncate" :style="{ color: 'var(--ink)' }">{{ source.title }}</span>
              <CmsStatusBadge :status="source.source_status" />
            </div>
            <p v-if="!tower?.source_objects?.length" class="text-sm" :style="{ color: 'var(--ink-muted)' }">No registered sources yet.</p>
          </div>
        </AppCard>

        <AppCard padding="md">
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--ink-muted)' }">Distribution Readiness</h2>
            <AppButton variant="ghost" size="sm" @click="router.push('/admin/remote-updates')">Remote Updates</AppButton>
          </div>
          <div class="grid grid-cols-3 gap-3 text-center">
            <div>
              <p class="font-display text-2xl font-bold" :style="{ color: 'var(--ink)' }">{{ tower?.entitlement_manager?.premium_count ?? 0 }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Premium</p>
            </div>
            <div>
              <p class="font-display text-2xl font-bold" :style="{ color: 'var(--ink)' }">{{ tower?.entitlement_manager?.elite_count ?? 0 }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Elite</p>
            </div>
            <div>
              <p class="font-display text-2xl font-bold" :style="{ color: 'var(--ink)' }">{{ tower?.entitlement_manager?.inactive_count ?? 0 }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Inactive</p>
            </div>
          </div>
        </AppCard>
      </div>
    </template>
  </div>
</template>
