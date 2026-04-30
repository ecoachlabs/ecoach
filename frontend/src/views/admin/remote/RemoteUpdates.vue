<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppInput from '@/components/ui/AppInput.vue'
import CmsMetricStrip from '@/components/admin/cms/CmsMetricStrip.vue'
import CmsStatusBadge from '@/components/admin/cms/CmsStatusBadge.vue'
import {
  getAdminQuestionBankStats,
  listContentSources,
  type AdminQuestionBankStatsDto,
  type ContentSourceRegistryEntryDto,
} from '@/ipc/admin'
import { installPack, listInstalledPacks, type PackSummaryDto } from '@/ipc/sessions'
import type { CmsMetricItem } from '@/components/admin/cms/types'

interface RemoteChannel {
  label: string
  path: string
  description: string
}

const loading = ref(true)
const applyingPath = ref('')
const error = ref('')
const success = ref('')
const stats = ref<AdminQuestionBankStatsDto | null>(null)
const packs = ref<PackSummaryDto[]>([])
const sources = ref<ContentSourceRegistryEntryDto[]>([])
const remotePath = ref('C:\\Users\\surfaceSudio\\OneDrive\\ecoach\\packs\\math-ghana-ccp-b7b9-foundation')

const remoteChannels: RemoteChannel[] = [
  {
    label: 'Ghana CCP B7-B9 Foundation',
    path: 'C:\\Users\\surfaceSudio\\OneDrive\\ecoach\\packs\\math-ghana-ccp-b7b9-foundation',
    description: 'Foundation pack for curriculum, questions, families, explanations, and glossary objects.',
  },
  {
    label: 'Math BECE Sample',
    path: 'C:\\Users\\surfaceSudio\\OneDrive\\ecoach\\packs\\math-bece-sample',
    description: 'Sample pack for smoke testing question-bank refreshes.',
  },
]

const updateMetrics = computed<CmsMetricItem[]>(() => [
  { label: 'Installed Packs', value: packs.value.length, tone: packs.value.length ? 'good' : 'review' },
  { label: 'Questions', value: stats.value?.total_questions ?? 0, caption: 'Bank inventory' },
  { label: 'Sources', value: stats.value?.source_upload_count ?? 0, caption: 'Registered raw material' },
  { label: 'Pending Review', value: stats.value?.pending_review_count ?? 0, tone: 'review' },
])

async function load() {
  loading.value = true
  error.value = ''
  try {
    const [packResult, statsResult, sourceResult] = await Promise.all([
      listInstalledPacks(),
      getAdminQuestionBankStats(),
      listContentSources(null, null, 8),
    ])
    packs.value = packResult
    stats.value = statsResult
    sources.value = sourceResult
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load remote update status.'
  } finally {
    loading.value = false
  }
}

async function applyUpdate(path: string) {
  if (!path.trim()) return
  applyingPath.value = path
  error.value = ''
  success.value = ''
  try {
    const result = await installPack(path.trim())
    success.value = `${result.pack_id} ${result.pack_version} applied to the question bank.`
    await load()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not apply remote update.'
  } finally {
    applyingPath.value = ''
  }
}

onMounted(load)
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <p class="text-[10px] font-bold uppercase tracking-[0.16em]" :style="{ color: 'var(--ink-muted)' }">
          Distribution
        </p>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Remote Updates</h1>
        <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
          Refresh the question bank from a pack or remote-style source without opening the content editor.
        </p>
      </div>
      <AppButton variant="secondary" size="sm" @click="load">Refresh</AppButton>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    <p v-if="success" class="mb-4 text-sm" :style="{ color: 'var(--accent)' }">{{ success }}</p>

    <CmsMetricStrip :items="updateMetrics" class="mb-6" />

    <div class="grid grid-cols-1 xl:grid-cols-[420px_minmax(0,1fr)] gap-5">
      <div class="space-y-4">
        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-2" :style="{ color: 'var(--ink)' }">Update Source</h2>
          <p class="text-xs mb-4" :style="{ color: 'var(--ink-muted)' }">
            Apply a pack path as an external content refresh. The editor remains available for follow-up cleanup.
          </p>
          <AppInput v-model="remotePath" label="Pack or Remote Path" placeholder="C:\path\to\pack" class="mb-3" />
          <AppButton variant="primary" :loading="applyingPath === remotePath" @click="applyUpdate(remotePath)">
            Apply To Question Bank
          </AppButton>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Available Channels</h2>
          <div class="space-y-2">
            <div v-for="channel in remoteChannels" :key="channel.path" class="rounded-lg p-3" :style="{ backgroundColor: 'var(--paper)' }">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ channel.label }}</p>
                  <p class="text-xs mt-1" :style="{ color: 'var(--ink-muted)' }">{{ channel.description }}</p>
                  <p class="text-[10px] truncate mt-2" :style="{ color: 'var(--ink-muted)' }">{{ channel.path }}</p>
                </div>
                <CmsStatusBadge status="preview" />
              </div>
              <AppButton variant="secondary" size="sm" class="mt-3" :loading="applyingPath === channel.path" @click="applyUpdate(channel.path)">
                Apply
              </AppButton>
            </div>
          </div>
        </AppCard>
      </div>

      <div class="space-y-4">
        <AppCard padding="md">
          <div class="flex items-center justify-between gap-3 mb-3">
            <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Installed Bank Feeds</h2>
            <CmsStatusBadge :status="loading ? 'loading' : 'active'" />
          </div>
          <div v-if="loading" class="space-y-2">
            <div v-for="i in 4" :key="i" class="h-14 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
          </div>
          <div v-else class="space-y-2">
            <div v-for="pack in packs" :key="`${pack.pack_id}:${pack.pack_version}`" class="flex items-center gap-3 rounded-lg px-3 py-3" :style="{ backgroundColor: 'var(--paper)' }">
              <div class="flex-1 min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ pack.pack_id }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">v{{ pack.pack_version }} / {{ pack.subject_code }}</p>
              </div>
              <CmsStatusBadge :status="pack.status" />
            </div>
            <p v-if="!packs.length" class="text-sm py-8 text-center" :style="{ color: 'var(--ink-muted)' }">No update feeds have been applied yet.</p>
          </div>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Recent Registered Sources</h2>
          <div class="space-y-2">
            <div v-for="source in sources" :key="source.id" class="flex items-center gap-3 rounded-lg px-3 py-3" :style="{ backgroundColor: 'var(--paper)' }">
              <div class="flex-1 min-w-0">
                <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ source.title }}</p>
                <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">
                  {{ source.source_kind }} / trust {{ source.trust_score_bp }} / freshness {{ source.freshness_score_bp }}
                </p>
              </div>
              <CmsStatusBadge :status="source.source_status" />
            </div>
            <p v-if="!sources.length" class="text-sm py-8 text-center" :style="{ color: 'var(--ink-muted)' }">No raw sources registered yet.</p>
          </div>
        </AppCard>
      </div>
    </div>
  </div>
</template>
