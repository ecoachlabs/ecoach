<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppButton from '@/components/ui/AppButton.vue'

const auth = useAuthStore()
const router = useRouter()
const loaded = ref(false)

onMounted(async () => {
  await auth.loadAccounts()
  loaded.value = true
})

function selectProfile(id: number) {
  router.push({ name: 'pin', params: { accountId: String(id) } })
}

const roleColors: Record<string, string> = {
  student: 'from-teal-500 to-emerald-600',
  parent: 'from-slate-600 to-slate-800',
  admin: 'from-violet-500 to-purple-700',
}

const roleIcons: Record<string, string> = {
  student: '📚',
  parent: '👤',
  admin: '⚙',
}
</script>

<template>
  <div class="min-h-screen flex flex-col items-center justify-center relative overflow-hidden kente-accent"
    :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Subtle geometric pattern background -->
    <div class="absolute inset-0 opacity-[0.02]" style="
      background-image: radial-gradient(circle at 20% 50%, rgba(13,148,136,0.3) 0%, transparent 50%),
                         radial-gradient(circle at 80% 50%, rgba(180,83,9,0.2) 0%, transparent 50%);
    " />

    <!-- Content -->
    <div class="relative z-10 flex flex-col items-center px-8 py-12 max-w-xl w-full reveal-stagger">

      <!-- Logo -->
      <div class="mb-2">
        <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-teal-500 to-emerald-600 flex items-center justify-center shadow-lg shadow-teal-500/20">
          <span class="text-white font-display font-bold text-2xl">e</span>
        </div>
      </div>

      <!-- Title -->
      <h1 class="font-display text-3xl font-bold tracking-tight mb-1" :style="{ color: 'var(--ink)' }">
        eCoach
      </h1>
      <p class="text-sm mb-10" :style="{ color: 'var(--ink-muted)' }">
        Who is studying today?
      </p>

      <!-- Profile Grid -->
      <div v-if="loaded && auth.accounts.length > 0" class="grid grid-cols-2 sm:grid-cols-3 gap-4 w-full">
        <button
          v-for="account in auth.accounts"
          :key="account.id"
          class="group flex flex-col items-center gap-3 p-5 rounded-[var(--radius-xl)] border transition-all cursor-pointer"
          :style="{
            backgroundColor: 'var(--surface)',
            borderColor: 'var(--border-soft)',
          }"
          style="transition: transform 220ms cubic-bezier(0.34,1.56,0.64,1), box-shadow 220ms ease;"
          @mouseenter="($event.target as HTMLElement).style.transform = 'translateY(-3px)'"
          @mouseleave="($event.target as HTMLElement).style.transform = ''"
          @click="selectProfile(account.id)"
        >
          <!-- Avatar -->
          <div
            class="w-16 h-16 rounded-2xl flex items-center justify-center text-2xl font-bold text-white shadow-md bg-gradient-to-br"
            :class="roleColors[account.account_type] || 'from-stone-400 to-stone-600'"
          >
            {{ account.display_name.charAt(0).toUpperCase() }}
          </div>

          <!-- Name -->
          <div class="text-center">
            <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ account.display_name }}</p>
            <p class="text-[11px] capitalize mt-0.5 flex items-center justify-center gap-1" :style="{ color: 'var(--ink-muted)' }">
              <span>{{ roleIcons[account.account_type] || '●' }}</span>
              {{ account.account_type }}
            </p>
          </div>

          <!-- Active indicator -->
          <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ account.last_active_label }}</p>
        </button>
      </div>

      <!-- Empty state -->
      <div v-else-if="loaded" class="text-center py-12">
        <div class="w-20 h-20 rounded-3xl mx-auto mb-4 flex items-center justify-center" :style="{ backgroundColor: 'var(--accent-light)' }">
          <span class="text-3xl">👋</span>
        </div>
        <h2 class="font-display text-lg font-semibold mb-1" :style="{ color: 'var(--ink)' }">Welcome to eCoach</h2>
        <p class="text-sm mb-6" :style="{ color: 'var(--ink-muted)' }">Create your first account to get started.</p>
        <AppButton variant="primary" size="lg">Create Account</AppButton>
      </div>

      <!-- Loading shimmer -->
      <div v-else class="grid grid-cols-3 gap-4 w-full">
        <div v-for="i in 3" :key="i" class="h-36 rounded-[var(--radius-xl)] animate-pulse" :style="{ backgroundColor: 'var(--border-soft)' }" />
      </div>
    </div>
  </div>
</template>
