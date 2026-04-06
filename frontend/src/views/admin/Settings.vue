<script setup lang="ts">
import { nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import CoachTuningPanel from '@/components/admin/CoachTuningPanel.vue'
import SystemHealthCard from '@/components/admin/SystemHealthCard.vue'

const route = useRoute()

const healthMetrics = [
  { label: 'Database', value: '48 MB', status: 'healthy' as const },
  { label: 'Content Coverage', value: '78%', status: 'warning' as const },
  { label: 'Question Quality', value: '94%', status: 'healthy' as const },
  { label: 'Last Backup', value: '2h ago', status: 'healthy' as const },
]

const configItems = [
  { title: 'Runtime Configuration', detail: 'Core app policies, defaults, and environment behavior.' },
  { title: 'Offline Policy', detail: 'Local-first behavior, pack availability, and fallback rules.' },
  { title: 'Security Controls', detail: 'PIN policy, access rules, and local protection settings.' },
]

const backupActions = [
  { title: 'Export Database Backup', detail: 'Create a point-in-time runtime backup for the current database.' },
  { title: 'Check Backup Status', detail: 'Verify backup integrity, size, and snapshot freshness.' },
  { title: 'Restore Backup', detail: 'Reopen the app from a previously exported backup copy.' },
  { title: 'Export Recovery Snapshot', detail: 'Bundle the database and rebuild docs for disaster recovery.' },
]

const entitlementItems = [
  { title: 'Parent Premium Access', tier: 'Premium', detail: 'Unlock concierge-style reporting and elevated parent support.' },
  { title: 'Student Elite Access', tier: 'Elite', detail: 'Enable high-performance learning surfaces and advanced analytics.' },
  { title: 'Role Governance', tier: 'Admin', detail: 'Manage who can publish, review, and oversee the academic system.' },
]

function scrollToHash() {
  if (!route.hash) return
  nextTick(() => {
    const target = document.getElementById(route.hash.slice(1))
    target?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  })
}

watch(() => route.hash, () => {
  scrollToHash()
}, { immediate: true })
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <div class="mb-8">
      <h1 class="text-lg font-bold mb-2" :style="{ color: 'var(--ink)' }">Admin Settings</h1>
      <p class="text-sm max-w-2xl" :style="{ color: 'var(--ink-muted)' }">
        Configuration, recovery, and oversight tools for the local academic operating system.
      </p>
    </div>

    <div class="flex flex-wrap gap-2 mb-8">
      <AppButton variant="secondary" size="sm" @click="$router.push('/admin/settings#system')">System Config</AppButton>
      <AppButton variant="secondary" size="sm" @click="$router.push('/admin/settings#health')">System Health</AppButton>
      <AppButton variant="secondary" size="sm" @click="$router.push('/admin/settings#backup')">Backup & Restore</AppButton>
      <AppButton variant="secondary" size="sm" @click="$router.push('/admin/settings#tuning')">Coach Tuning</AppButton>
      <AppButton variant="secondary" size="sm" @click="$router.push('/admin/settings#entitlements')">Entitlements</AppButton>
    </div>

    <section id="system" class="mb-8 scroll-mt-6">
      <h2 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">System Configuration</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
        <AppCard v-for="item in configItems" :key="item.title" padding="md">
          <div class="flex items-start justify-between gap-3">
            <div>
              <p class="text-sm font-medium mb-1" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
              <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ item.detail }}</p>
            </div>
            <AppButton variant="secondary" size="sm">Configure</AppButton>
          </div>
        </AppCard>
      </div>
    </section>

    <section id="health" class="mb-8 scroll-mt-6">
      <h2 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">System Health</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <SystemHealthCard :metrics="healthMetrics" />
        <AppCard padding="md">
          <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">Operational Notes</h3>
          <div class="space-y-3 text-sm" :style="{ color: 'var(--ink-secondary)' }">
            <p>Recovery coverage is active for runtime backups and rebuild snapshots.</p>
            <p>Curriculum and content pipelines should be checked before major publishing operations.</p>
            <p>Question quality and coverage should be reviewed together to avoid hidden blind spots.</p>
          </div>
        </AppCard>
      </div>
    </section>

    <section id="backup" class="mb-8 scroll-mt-6">
      <h2 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">Backup & Restore</h2>
      <div class="space-y-3">
        <AppCard v-for="action in backupActions" :key="action.title" padding="md">
          <div class="flex items-start justify-between gap-3">
            <div>
              <p class="text-sm font-medium mb-1" :style="{ color: 'var(--ink)' }">{{ action.title }}</p>
              <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ action.detail }}</p>
            </div>
            <AppButton variant="secondary" size="sm">Open</AppButton>
          </div>
        </AppCard>
      </div>
    </section>

    <section id="tuning" class="mb-8 scroll-mt-6">
      <h2 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">Coach Tuning</h2>
      <CoachTuningPanel />
    </section>

    <section id="entitlements" class="scroll-mt-6">
      <h2 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--ink-muted)' }">Entitlement Management</h2>
      <div class="space-y-3">
        <AppCard v-for="item in entitlementItems" :key="item.title" padding="md">
          <div class="flex items-start justify-between gap-3">
            <div>
              <div class="flex items-center gap-2 mb-1">
                <p class="text-sm font-medium" :style="{ color: 'var(--ink)' }">{{ item.title }}</p>
                <AppBadge color="accent" size="xs">{{ item.tier }}</AppBadge>
              </div>
              <p class="text-xs" :style="{ color: 'var(--ink-muted)' }">{{ item.detail }}</p>
            </div>
            <AppButton variant="secondary" size="sm">Manage</AppButton>
          </div>
        </AppCard>
      </div>
    </section>
  </div>
</template>
