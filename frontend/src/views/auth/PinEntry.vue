<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { PIN_LENGTH } from '@/utils/validation'

const props = defineProps<{ accountId: string }>()
const auth = useAuthStore()
const router = useRouter()

const pin = ref('')
const error = ref('')
const loading = ref(false)

const selectedAccount = computed(() =>
  auth.accounts.find(a => a.id === Number(props.accountId))
)

const pinLength = computed(() => PIN_LENGTH)

function addDigit(digit: string) {
  if (pin.value.length < pinLength.value) {
    pin.value += digit
    error.value = ''
    if (pin.value.length === pinLength.value) attemptLogin()
  }
}

function removeDigit() {
  pin.value = pin.value.slice(0, -1)
  error.value = ''
}

async function attemptLogin() {
  if (loading.value) return
  if (pin.value.length !== pinLength.value) return
  loading.value = true
  error.value = ''
  try {
    await auth.login(Number(props.accountId), pin.value)
    const role = auth.role
    if (role === 'student') router.push('/student')
    else if (role === 'parent') router.push('/parent')
    else if (role === 'admin') router.push('/admin')
    else router.push('/student')
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e?.message ?? 'Wrong PIN. Try again.'
    pin.value = ''
  } finally {
    loading.value = false
  }
}

function goBack() {
  router.push('/')
}

const digits = ['1','2','3','4','5','6','7','8','9','','0','⌫']
</script>

<template>
  <div class="pin-screen">
    <!-- Back -->
    <button class="back-btn" @click="goBack">← Back</button>

    <!-- Avatar + identity -->
    <div class="identity">
      <div class="avatar">
        {{ selectedAccount?.display_name?.charAt(0)?.toUpperCase() || '?' }}
      </div>
      <h2 class="name">{{ selectedAccount?.display_name }}</h2>
      <p class="role">{{ selectedAccount?.account_type ?? 'Student' }}</p>
    </div>

    <!-- PIN dots -->
    <div class="pin-dots">
      <div
        v-for="i in pinLength"
        :key="i"
        class="dot"
        :class="{ filled: i <= pin.length, error: !!error }"
      />
    </div>

    <!-- Error -->
    <p v-if="error" class="error-msg">{{ error }}</p>
    <p v-else class="hint">Enter your {{ pinLength }}-digit PIN</p>

    <!-- Numpad -->
    <div class="numpad">
      <button
        v-for="key in digits"
        :key="key"
        class="key"
        :class="{
          'key--empty': key === '',
          'key--del': key === '⌫',
          'key--loading': loading && key !== '⌫' && key !== '',
        }"
        :disabled="key === '' || loading"
        @click="key === '⌫' ? removeDigit() : key ? addDigit(key) : null"
      >
        <span v-if="key === '⌫'" class="del-icon">⌫</span>
        <span v-else-if="key">{{ key }}</span>
      </button>
    </div>

    <!-- Loading indicator -->
    <div v-if="loading" class="loading-bar" />
  </div>
</template>

<style scoped>
.pin-screen {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: var(--paper);
  padding: 32px 24px;
  gap: 0;
  position: relative;
}

.back-btn {
  position: absolute;
  top: 24px;
  left: 24px;
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-muted);
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 8px;
  transition: background-color 100ms;
  background: transparent;
}
.back-btn:hover { background-color: var(--border-soft); color: var(--ink); }

.identity {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 36px;
}

.avatar {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background-color: var(--ink);
  color: var(--paper);
  font-family: var(--font-display);
  font-size: 28px;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 14px;
}

.name {
  font-family: var(--font-display);
  font-size: 22px;
  font-weight: 700;
  color: var(--ink);
  margin: 0 0 4px;
}

.role {
  font-size: 12px;
  color: var(--ink-muted);
  text-transform: capitalize;
  margin: 0;
}

.pin-dots {
  display: flex;
  gap: 14px;
  margin-bottom: 12px;
}

.dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background-color: var(--border-soft);
  border: 2px solid var(--border-soft);
  transition: background-color 150ms, transform 150ms, border-color 150ms;
}
.dot.filled {
  background-color: var(--ink);
  border-color: var(--ink);
  transform: scale(1.15);
}
.dot.error {
  background-color: var(--warm);
  border-color: var(--warm);
}

.hint {
  font-size: 12px;
  color: var(--ink-muted);
  margin: 0 0 32px;
  min-height: 18px;
}

.error-msg {
  font-size: 12px;
  color: var(--warm);
  font-weight: 600;
  margin: 0 0 32px;
  min-height: 18px;
}

.numpad {
  display: grid;
  grid-template-columns: repeat(3, 72px);
  gap: 12px;
}

.key {
  width: 72px;
  height: 52px;
  border-radius: 14px;
  font-size: 20px;
  font-weight: 600;
  color: var(--ink);
  background-color: var(--surface);
  border: 1px solid transparent;
  cursor: pointer;
  transition: background-color 80ms ease, transform 80ms ease, border-color 80ms ease;
  display: flex;
  align-items: center;
  justify-content: center;
}
.key:hover:not(:disabled) {
  background-color: var(--paper);
  border-color: var(--border-strong);
  transform: translateY(-1px);
}
.key:active:not(:disabled) { transform: scale(0.95); }

.key--empty {
  background: transparent;
  border-color: transparent;
  pointer-events: none;
}

.key--del {
  font-size: 16px;
  color: var(--ink-secondary);
}

.key--loading {
  opacity: 0.5;
  pointer-events: none;
}

.loading-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--accent), transparent);
  animation: slide 1.2s ease-in-out infinite;
}

@keyframes slide {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}
</style>

