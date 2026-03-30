<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { listSubjects, type SubjectDto } from '@/ipc/coach'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'

const router = useRouter()
const subjects = ref<SubjectDto[]>([])
const selected = ref<number[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    subjects.value = await listSubjects(1)
  } catch (e) {
    console.error('Failed to load subjects:', e)
  }
  loading.value = false
})

function toggle(id: number) {
  const idx = selected.value.indexOf(id)
  if (idx >= 0) selected.value.splice(idx, 1)
  else selected.value.push(id)
}
</script>

<template>
  <div class="p-6 lg:p-8 max-w-3xl mx-auto reveal-stagger">
    <h1 class="font-display text-2xl font-bold tracking-tight mb-2" :style="{ color: 'var(--text)' }">Select Your Subjects</h1>
    <p class="text-sm mb-8" :style="{ color: 'var(--text-3)' }">Choose the subjects you are preparing for the BECE exam.</p>

    <div v-if="loading" class="space-y-3">
      <div v-for="i in 4" :key="i" class="h-16 rounded-xl animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
    </div>

    <div v-else class="space-y-3 mb-8">
      <AppCard
        v-for="s in subjects" :key="s.id" padding="md" hover
        :class="selected.includes(s.id) ? 'ring-2 ring-[var(--accent)]' : ''"
        @click="toggle(s.id)"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg flex items-center justify-center text-sm font-bold"
            :style="{ backgroundColor: 'var(--accent-light)', color: 'var(--accent)' }">
            {{ s.code?.charAt(0) || s.name.charAt(0) }}
          </div>
          <div class="flex-1">
            <p class="text-sm font-semibold" :style="{ color: 'var(--text)' }">{{ s.name }}</p>
            <p class="text-[11px]" :style="{ color: 'var(--text-3)' }">{{ s.code }}</p>
          </div>
          <div v-if="selected.includes(s.id)" class="w-6 h-6 rounded-full bg-[var(--accent)] flex items-center justify-center text-white text-xs font-bold">
            ✓
          </div>
        </div>
      </AppCard>
    </div>

    <div class="flex items-center gap-3">
      <AppButton variant="primary" :disabled="selected.length === 0" @click="router.push('/student/onboarding/content-packs')">
        Continue with {{ selected.length }} subject{{ selected.length !== 1 ? 's' : '' }} →
      </AppButton>
      <AppButton variant="ghost" size="sm" @click="router.push('/student/onboarding/welcome')">Back</AppButton>
    </div>
  </div>
</template>
