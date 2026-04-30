<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import OfflineStatusBanner from '@/components/system/OfflineStatusBanner.vue'
import { startOfflineQueueAutoFlush } from '@/ipc'
import { useAuthStore } from '@/stores/auth'
import { useConnectivityStore } from '@/stores/connectivity'
import { useUiStore } from '@/stores/ui'

const ui = useUiStore()
const auth = useAuthStore()
const connectivity = useConnectivityStore()
const router = useRouter()
const route = useRoute()

function resolveLegacyCoachRoute(path: string): string | null {
  const normalized = path.replace(/\/+$/, '') || '/coach'

  const directRedirects: Record<string, string> = {
    '/coach': '/student/coach',
    '/coach/onboarding': '/student/onboarding/welcome',
    '/coach/onboarding/subjects': '/student/onboarding/subjects',
    '/coach/content': '/student/onboarding/content-packs',
    '/coach/diagnostic': '/student/diagnostic',
    '/coach/plan': '/student/journey',
    '/coach/plan/refresh': '/student/journey',
    '/coach/repair': '/student/knowledge-gap',
    '/coach/mission/today': '/student/journey',
  }

  if (directRedirects[normalized]) {
    return directRedirects[normalized]
  }

  if (/^\/coach\/mission\/\d+$/.test(normalized)) {
    return '/student/journey'
  }

  if (/^\/coach\/missions\/\d+$/.test(normalized)) {
    return '/student/journey'
  }

  if (/^\/coach\/review\/\d+$/.test(normalized)) {
    return '/student/journey'
  }

  return normalized.startsWith('/coach/') ? '/student/coach' : null
}

function resolveRoleHome(): string {
  if (!auth.isAuthenticated) {
    return '/'
  }

  if (auth.role === 'parent') {
    return '/parent'
  }

  if (auth.role === 'admin') {
    return '/admin'
  }

  return '/student'
}

onMounted(() => {
  // Default to student theme until auth determines the role
  ui.setTheme('student')
  connectivity.startMonitoring()
  startOfflineQueueAutoFlush()
})

onUnmounted(() => {
  connectivity.stopMonitoring()
})

watch(
  () => [route.path, route.matched.length] as const,
  async ([path, matchedLength]) => {
    const redirectTarget = resolveLegacyCoachRoute(path)
      ?? (matchedLength === 0 ? resolveRoleHome() : null)

    if (redirectTarget && redirectTarget !== path) {
      await router.replace(redirectTarget)
    }
  },
  { immediate: true },
)
</script>

<template>
  <OfflineStatusBanner />
  <RouterView />
</template>
