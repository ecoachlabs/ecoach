<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { listInstalledPacks, type PackSummaryDto } from '@/ipc/sessions'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'

const router = useRouter()
const packs = ref<PackSummaryDto[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    packs.value = await listInstalledPacks()
  } catch (e) {
    console.error('Failed to load packs:', e)
  }
  loading.value = false
})
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--ink)' }">Content Packs</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--ink-muted)' }">Content packs provide the questions and materials for your subjects.</p>

    <div v-if="loading" class="space-y-3">
      <div v-for="i in 3" :key="i" class="h-16 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else-if="packs.length" class="space-y-3 mb-8">
      <AppCard v-for="p in packs" :key="p.pack_id" padding="md">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold"
            :style="{ backgroundColor: 'var(--success-light)', color: 'var(--accent)' }">✓</div>
          <div class="flex-1">
            <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ p.pack_id }}</p>
            <p class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">v{{ p.pack_version }} · {{ p.subject_code }}</p>
          </div>
          <AppBadge color="success" size="xs">{{ p.status }}</AppBadge>
        </div>
      </AppCard>
    </div>

    <div v-else class="text-center py-12 mb-8">
      <div class="w-14 h-14 rounded-2xl mx-auto mb-4 flex items-center justify-center text-xl"
        :style="{ backgroundColor: 'var(--warning-light)', color: 'var(--warning)' }">📦</div>
      <p class="text-sm font-medium mb-1" :style="{ color: 'var(--ink)' }">No content packs installed</p>
      <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">Ask your administrator to install content packs for your subjects.</p>
    </div>

    <div class="flex items-center gap-3">
      <AppButton variant="primary" @click="router.push('/student/onboarding/diagnostic')">Continue →</AppButton>
      <AppButton variant="ghost" size="sm" @click="router.push('/student/onboarding/subjects')">Back</AppButton>
    </div>
  </div>
</template>
