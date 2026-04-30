<script setup lang="ts">
import { onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { rememberEliteSessionClass } from '@/ipc/elite'

const route = useRoute()
const router = useRouter()

onMounted(() => {
  const sessionId = Number(route.params.id)
  const sessionClass = typeof route.query.class === 'string' ? route.query.class : null

  if (Number.isFinite(sessionId) && sessionClass) {
    rememberEliteSessionClass(sessionId, sessionClass)
  }

  void router.replace(`/student/session/${sessionId}`)
})
</script>

<template>
  <div class="h-full flex items-center justify-center" :style="{ backgroundColor: 'var(--paper)' }">
    <div class="text-center">
      <p class="eyebrow mb-3">Elite Session</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">
        Entering the arena
      </h1>
      <p class="text-sm mt-2" :style="{ color: 'var(--ink-muted)' }">
        Syncing elite session tracking and opening the live question stream.
      </p>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--gold);
}
</style>
