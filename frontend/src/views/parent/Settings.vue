<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { resetPin, listLinkedStudents } from '@/ipc/identity'
import type { AccountSummaryDto } from '@/types'
import AppButton from '@/components/ui/AppButton.vue'
import { PIN_LENGTH, isValidPin } from '@/utils/validation'

const auth = useAuthStore()
const router = useRouter()

const showPinForm = ref(false)
const newPin = ref('')
const confirmPin = ref('')
const pinSaving = ref(false)
const pinError = ref('')
const pinSuccess = ref('')

const children = ref<AccountSummaryDto[]>([])
const loadingChildren = ref(true)

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    children.value = await listLinkedStudents(auth.currentAccount.id)
  } catch {}
  loadingChildren.value = false
})

async function savePin() {
  pinError.value = ''
  pinSuccess.value = ''
  if (!isValidPin(newPin.value)) { pinError.value = `PIN must be exactly ${PIN_LENGTH} digits`; return }
  if (newPin.value !== confirmPin.value) { pinError.value = 'PINs do not match'; return }
  if (!auth.currentAccount) return
  pinSaving.value = true
  try {
    await resetPin(auth.currentAccount.id, newPin.value)
    pinSuccess.value = 'PIN updated successfully'
    newPin.value = ''
    confirmPin.value = ''
    showPinForm.value = false
  } catch (e: any) {
    pinError.value = typeof e === 'string' ? e : e?.message ?? 'Failed to update PIN'
  }
  pinSaving.value = false
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden" :style="{ backgroundColor: 'var(--paper)' }">

    <!-- Header -->
    <div
      class="flex-shrink-0 px-7 pt-6 pb-5 border-b"
      :style="{ borderColor: 'var(--border-soft)', backgroundColor: 'var(--surface)' }"
    >
      <p class="eyebrow">Parent</p>
      <h1 class="font-display text-2xl font-bold tracking-tight" :style="{ color: 'var(--ink)' }">Settings</h1>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto p-7 max-w-lg space-y-4">

      <!-- Linked children -->
      <div class="settings-card">
        <div class="flex items-center justify-between mb-4">
          <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Family Accounts</p>
          <AppButton variant="secondary" size="sm" @click="router.push('/parent/children')">Manage</AppButton>
        </div>
        <div v-if="loadingChildren" class="text-xs" :style="{ color: 'var(--ink-muted)' }">Loading...</div>
        <div v-else-if="children.length === 0" class="text-xs" :style="{ color: 'var(--ink-muted)' }">
          No children linked yet. Create child accounts from the Children page.
        </div>
        <div v-else class="space-y-2">
          <div v-for="child in children" :key="child.id"
            class="flex items-center justify-between py-1.5">
            <div>
              <span class="text-sm font-semibold block" :style="{ color: 'var(--ink)' }">{{ child.display_name }}</span>
              <span class="text-[11px]" :style="{ color: 'var(--ink-muted)' }">{{ child.last_active_label || 'Not yet active' }}</span>
            </div>
            <AppButton variant="ghost" size="sm" @click="router.push('/parent/child/' + child.id)">Open</AppButton>
          </div>
        </div>
      </div>

      <!-- Change PIN -->
      <div class="settings-card">
        <div class="flex items-center justify-between">
          <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Change PIN</p>
          <AppButton variant="secondary" size="sm" @click="showPinForm = !showPinForm">
            {{ showPinForm ? 'Cancel' : 'Update' }}
          </AppButton>
        </div>
        <div v-if="showPinForm" class="mt-4 space-y-3">
          <div v-if="pinError" class="text-xs p-2 rounded-lg"
            style="background: rgba(194,65,12,0.08); color: var(--warm);">{{ pinError }}</div>
          <div>
            <label class="block text-[11px] font-semibold mb-1.5" :style="{ color: 'var(--ink-muted)' }">New PIN</label>
            <input v-model="newPin" type="password" inputmode="numeric" maxlength="4"
              placeholder="Enter new 4-digit PIN"
              class="w-full px-3 py-2 rounded-lg text-sm border outline-none"
              :style="{ backgroundColor: 'var(--paper)', borderColor: 'var(--border-soft)', color: 'var(--ink)' }" />
          </div>
          <div>
            <label class="block text-[11px] font-semibold mb-1.5" :style="{ color: 'var(--ink-muted)' }">Confirm PIN</label>
            <input v-model="confirmPin" type="password" inputmode="numeric" maxlength="4"
              placeholder="Repeat new 4-digit PIN"
              class="w-full px-3 py-2 rounded-lg text-sm border outline-none"
              :style="{ backgroundColor: 'var(--paper)', borderColor: 'var(--border-soft)', color: 'var(--ink)' }" />
          </div>
          <AppButton variant="primary" size="sm" :loading="pinSaving" @click="savePin">Save PIN</AppButton>
        </div>
        <p v-if="pinSuccess" class="mt-2 text-xs" :style="{ color: 'var(--accent)' }">{{ pinSuccess }}</p>
      </div>

      <!-- Account -->
      <div class="settings-card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-bold" :style="{ color: 'var(--ink)' }">Account</p>
            <p class="text-xs mt-0.5" :style="{ color: 'var(--ink-muted)' }">
              Signed in as {{ auth.currentAccount?.display_name }}
            </p>
          </div>
          <AppButton variant="ghost" size="sm" @click="auth.logout(); router.push('/')">Switch Account</AppButton>
        </div>
      </div>

      <!-- Sign out -->
      <button
        class="w-full py-3 rounded-2xl border text-sm font-semibold"
        :style="{ color: 'var(--warm)', borderColor: 'rgba(194,65,12,0.2)', backgroundColor: 'rgba(194,65,12,0.04)' }"
        @click="auth.logout(); router.push('/')"
      >Sign Out</button>
    </div>
  </div>
</template>

<style scoped>
.eyebrow { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.16em; color: var(--accent); margin-bottom: 4px; }
.settings-card {
  padding: 20px;
  border-radius: 16px;
  border: 1px solid var(--border-soft);
  background: var(--surface);
}
</style>
