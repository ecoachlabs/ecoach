<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const router = useRouter()

const darkMode = ref(false)
const soundEffects = ref(true)
const notifications = ref(true)
const compactMode = ref(false)

const settingGroups = [
  {
    label: 'Account',
    items: [
      { id: 'pin', label: 'Change PIN', desc: 'Update your 4-digit access PIN', action: 'button', actionLabel: 'Update' },
      { id: 'display', label: 'Display Name', desc: auth.currentAccount?.display_name ?? '—', action: 'button', actionLabel: 'Edit' },
    ],
  },
  {
    label: 'Appearance',
    items: [
      { id: 'dark', label: 'Dark Mode', desc: 'Switch between light and dark themes', action: 'toggle', ref: darkMode },
      { id: 'compact', label: 'Compact Layout', desc: 'Reduce spacing for more content', action: 'toggle', ref: compactMode },
    ],
  },
  {
    label: 'Sound & Notifications',
    items: [
      { id: 'sound', label: 'Sound Effects', desc: 'Play sounds for correct/incorrect answers', action: 'toggle', ref: soundEffects },
      { id: 'notif', label: 'Study Reminders', desc: 'Daily reminders to keep your streak alive', action: 'toggle', ref: notifications },
    ],
  },
  {
    label: 'Study Preferences',
    items: [
      { id: 'review', label: 'Review Queue', desc: 'Manage your spaced-repetition review schedule', action: 'navigate', actionLabel: 'Open →', route: '/student/memory#reviews' },
      { id: 'goals', label: 'Daily Goal', desc: 'Set your target questions per day', action: 'button', actionLabel: 'Set' },
    ],
  },
]
</script>

<template>
  <div class="h-full flex overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Left nav -->
    <div
      class="w-64 flex-shrink-0 flex flex-col border-r overflow-y-auto"
      :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
    >
      <div class="px-6 pt-8 pb-5 border-b" :style="{ borderColor: 'var(--border-soft)' }">
        <p class="eyebrow mb-1">Settings</p>
        <h1 class="font-display text-xl font-bold" :style="{ color: 'var(--ink)' }">Preferences</h1>
      </div>

      <!-- Profile card -->
      <div class="p-4 mx-4 my-4 rounded-xl border" :style="{ borderColor: 'transparent', backgroundColor: 'var(--paper)' }">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-full flex items-center justify-center text-sm font-black"
            :style="{ backgroundColor: 'var(--ink)', color: 'var(--paper)' }">
            {{ auth.currentAccount?.display_name?.charAt(0).toUpperCase() ?? '?' }}
          </div>
          <div>
            <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">{{ auth.currentAccount?.display_name ?? 'Student' }}</p>
            <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">Student Account</p>
          </div>
        </div>
      </div>

      <nav class="flex-1 px-3 space-y-0.5 pb-4">
        <button
          v-for="group in settingGroups"
          :key="group.label"
          class="group-nav-btn w-full text-left px-3 py-2.5 rounded-xl flex items-center gap-3"
        >
          <span class="text-sm font-semibold" :style="{ color: 'var(--ink-secondary)' }">{{ group.label }}</span>
        </button>
      </nav>
    </div>

    <!-- Right: settings content -->
    <div class="flex-1 overflow-y-auto p-8 space-y-8">

      <div v-for="group in settingGroups" :key="group.label">
        <p class="section-label mb-4">{{ group.label }}</p>

        <div class="rounded-2xl border overflow-hidden divide-y divide-[var(--border-soft)]" :style="{ borderColor: 'transparent' }">
          <div
            v-for="item in group.items"
            :key="item.id"
            class="flex items-center gap-4 px-5 py-4 setting-row"
            :style="{ backgroundColor: 'var(--surface)' }"
          >
            <div class="flex-1 min-w-0">
              <p class="text-sm font-semibold" :style="{ color: 'var(--ink)' }">{{ item.label }}</p>
              <p class="text-[11px] mt-0.5" :style="{ color: 'var(--ink-muted)' }">{{ item.desc }}</p>
            </div>

            <!-- Toggle -->
            <button
              v-if="item.action === 'toggle'"
              class="toggle-switch"
              :class="{ on: (item as any).ref.value }"
              @click="(item as any).ref.value = !(item as any).ref.value"
            >
              <span class="toggle-knob" />
            </button>

            <!-- Button action -->
            <button v-else-if="item.action === 'button'" class="action-pill">
              {{ (item as any).actionLabel }}
            </button>

            <!-- Navigate -->
            <button v-else-if="item.action === 'navigate'" class="action-pill"
              @click="router.push((item as any).route)">
              {{ (item as any).actionLabel }}
            </button>
          </div>
        </div>
      </div>

      <!-- Sign out -->
      <div>
        <button
          class="w-full flex items-center justify-center gap-2 py-3 rounded-2xl border text-sm font-semibold"
          :style="{ color: 'var(--warm)', borderColor: 'rgba(194,65,12,0.2)', backgroundColor: 'rgba(194,65,12,0.04)' }"
          @click="auth.logout(); router.push('/')"
        >Sign Out</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.eyebrow {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.16em;
  color: var(--ink-muted);
}

.section-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
  color: var(--ink-muted);
}

.group-nav-btn {
  transition: background-color 100ms;
}
.group-nav-btn:hover { background-color: var(--paper); }

.setting-row {
  transition: background-color 100ms;
}
.setting-row:hover { background-color: var(--paper) !important; }

.toggle-switch {
  width: 44px;
  height: 24px;
  border-radius: 999px;
  position: relative;
  cursor: pointer;
  background: var(--border-soft);
  transition: background-color 200ms;
  flex-shrink: 0;
  border: 1px solid transparent;
}
.toggle-switch.on {
  background: var(--accent);
  border-color: var(--accent);
}
.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 999px;
  background: white;
  transition: transform 200ms;
  box-shadow: 0 1px 3px rgba(0,0,0,0.15);
}
.toggle-switch.on .toggle-knob {
  transform: translateX(20px);
}

.action-pill {
  padding: 6px 16px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  background: var(--paper);
  color: var(--ink-secondary);
  border: 1px solid transparent;
  transition: all 120ms;
  flex-shrink: 0;
}
.action-pill:hover {
  background: var(--accent-glow);
  color: var(--accent);
  border-color: var(--accent);
}
</style>


