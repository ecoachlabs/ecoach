<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppInput from '@/components/ui/AppInput.vue'
import { installPack, listInstalledPacks } from '@/ipc/sessions'
import type { PackSummaryDto } from '@/types'

const packs = ref<PackSummaryDto[]>([])
const loading = ref(true)
const installingPath = ref('')
const customPath = ref('C:\\Users\\surfaceSudio\\OneDrive\\ecoach\\packs\\math-ghana-ccp-b7b9-foundation')
const error = ref('')
const success = ref('')

const bundledPacks = [
  {
    label: 'Ghana CCP B7-B9 Foundation',
    path: 'C:\\Users\\surfaceSudio\\OneDrive\\ecoach\\packs\\math-ghana-ccp-b7b9-foundation',
  },
  {
    label: 'Math BECE Sample',
    path: 'C:\\Users\\surfaceSudio\\OneDrive\\ecoach\\packs\\math-bece-sample',
  },
]

async function load() {
  loading.value = true
  error.value = ''
  try {
    packs.value = await listInstalledPacks()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not load packs.'
  } finally {
    loading.value = false
  }
}

async function install(path: string) {
  if (!path.trim()) return
  installingPath.value = path
  error.value = ''
  success.value = ''
  try {
    const result = await installPack(path.trim())
    success.value = `Installed ${result.pack_id} ${result.pack_version}.`
    await load()
  } catch (err) {
    error.value = typeof err === 'string' ? err : (err as { message?: string })?.message ?? 'Could not install pack.'
  } finally {
    installingPath.value = ''
  }
}

onMounted(load)
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="flex items-start justify-between gap-4 mb-6">
      <div>
        <h1 class="text-xl font-bold mb-1" :style="{ color: 'var(--ink)' }">Pack Manager</h1>
        <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">Install and verify local content packs that seed curriculum, questions, answers, families, and content objects.</p>
      </div>
      <AppButton variant="secondary" size="sm" @click="load">Refresh</AppButton>
    </div>

    <p v-if="error" class="mb-4 text-sm" :style="{ color: 'var(--warm)' }">{{ error }}</p>
    <p v-if="success" class="mb-4 text-sm" :style="{ color: 'var(--accent)' }">{{ success }}</p>

    <div class="grid grid-cols-1 xl:grid-cols-[420px_minmax(0,1fr)] gap-5">
      <div class="space-y-4">
        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Bundled Packs</h2>
          <div class="space-y-2">
            <div v-for="pack in bundledPacks" :key="pack.path" class="rounded-lg p-3" :style="{ backgroundColor: 'var(--paper)' }">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ pack.label }}</p>
              <p class="text-[10px] truncate mb-3" :style="{ color: 'var(--ink-muted)' }">{{ pack.path }}</p>
              <AppButton variant="primary" size="sm" :loading="installingPath === pack.path" @click="install(pack.path)">Install</AppButton>
            </div>
          </div>
        </AppCard>

        <AppCard padding="md">
          <h2 class="text-sm font-bold mb-3" :style="{ color: 'var(--ink)' }">Install By Path</h2>
          <AppInput v-model="customPath" label="Pack Folder" placeholder="C:\path\to\pack" class="mb-3" />
          <AppButton variant="primary" :loading="installingPath === customPath" @click="install(customPath)">Install Pack</AppButton>
        </AppCard>
      </div>

      <AppCard padding="md">
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Installed Packs</h2>
          <AppBadge color="muted" size="xs">{{ packs.length }} installed</AppBadge>
        </div>
        <div v-if="loading" class="space-y-2">
          <div v-for="i in 4" :key="i" class="h-14 rounded-lg animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
        </div>
        <div v-else class="space-y-2">
          <div v-for="pack in packs" :key="pack.pack_id" class="flex items-center gap-3 rounded-lg px-3 py-3" :style="{ backgroundColor: 'var(--paper)' }">
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold truncate" :style="{ color: 'var(--ink)' }">{{ pack.pack_id }}</p>
              <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">v{{ pack.pack_version }} · {{ pack.subject_code }}</p>
            </div>
            <AppBadge :color="pack.status === 'active' ? 'success' : pack.status === 'failed' ? 'danger' : 'gold'" size="xs">{{ pack.status }}</AppBadge>
          </div>
          <p v-if="!packs.length" class="text-sm py-8 text-center" :style="{ color: 'var(--ink-muted)' }">No content packs installed yet.</p>
        </div>
      </AppCard>
    </div>
  </div>
</template>
