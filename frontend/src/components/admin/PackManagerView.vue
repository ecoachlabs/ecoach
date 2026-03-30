<script setup lang="ts">
import { ref, onMounted } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppTable from '@/components/ui/AppTable.vue'
import UploadDropzone from '@/components/upload/UploadDropzone.vue'
import { listInstalledPacks, installPack, type PackSummaryDto } from '@/ipc/sessions'

const packs = ref<PackSummaryDto[]>([])
const loading = ref(true)
const showUpload = ref(false)

onMounted(async () => {
  try { packs.value = await listInstalledPacks() } catch {}
  loading.value = false
})

async function handleFiles(files: File[]) {
  for (const file of files) {
    try {
      await installPack(file.name)
      packs.value = await listInstalledPacks()
    } catch {}
  }
  showUpload.value = false
}

const columns = [
  { key: 'pack_id', label: 'Pack ID' },
  { key: 'pack_version', label: 'Version', width: '100px' },
  { key: 'subject_code', label: 'Subject', width: '120px' },
  { key: 'status', label: 'Status', width: '100px' },
]
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-base font-bold" :style="{ color: 'var(--text)' }">Content Packs</h2>
      <AppButton variant="primary" size="sm" @click="showUpload = !showUpload">{{ showUpload ? 'Cancel' : '+ Install Pack' }}</AppButton>
    </div>

    <div v-if="showUpload" class="mb-4">
      <UploadDropzone @files="handleFiles" />
    </div>

    <div v-if="loading" class="space-y-2">
      <div v-for="i in 3" :key="i" class="h-12 rounded animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <AppTable v-else :columns="columns" :rows="packs" compact>
      <template #status="{ value }">
        <AppBadge :color="value === 'active' ? 'success' : 'muted'" size="xs">{{ value }}</AppBadge>
      </template>
    </AppTable>
  </div>
</template>
