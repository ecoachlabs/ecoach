<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const props = defineProps<{ accountId: string }>()
const auth = useAuthStore()
const router = useRouter()

const pin = ref('')
const error = ref('')
const loading = ref(false)

const selectedAccount = computed(() =>
  auth.accounts.find(a => a.id === Number(props.accountId))
)

function addDigit(digit: string) {
  if (pin.value.length < 6) {
    pin.value += digit
    error.value = ''
  }
  if (pin.value.length >= 4) {
    attemptLogin()
  }
}

function removeDigit() {
  pin.value = pin.value.slice(0, -1)
  error.value = ''
}

async function attemptLogin() {
  if (loading.value) return
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
    const msg = typeof e === 'string' ? e : e?.message ?? 'Wrong PIN. Try again.'
    error.value = msg
    pin.value = ''
  } finally {
    loading.value = false
  }
}

function goBack() {
  router.push('/')
}
</script>

<template>
  <div class="min-h-screen flex flex-col items-center justify-center bg-[#fafaf8] p-8">
    <button class="absolute top-6 left-6 text-gray-400 hover:text-gray-600 text-sm" @click="goBack">
      &larr; Back
    </button>

    <div class="flex flex-col items-center mb-8">
      <div class="w-20 h-20 rounded-full flex items-center justify-center text-3xl font-bold text-white mb-3 bg-blue-500">
        {{ selectedAccount?.display_name?.charAt(0)?.toUpperCase() || '?' }}
      </div>
      <h2 class="text-xl font-semibold text-gray-900">{{ selectedAccount?.display_name }}</h2>
      <p class="text-sm text-gray-400">Enter your PIN</p>
    </div>

    <div class="flex gap-3 mb-6">
      <div
        v-for="i in 6"
        :key="i"
        class="w-4 h-4 rounded-full transition-all duration-150"
        :class="i <= pin.length ? 'bg-blue-500 scale-110' : 'bg-gray-200'"
      />
    </div>

    <p v-if="error" class="text-red-500 text-sm mb-4">{{ error }}</p>

    <div class="grid grid-cols-3 gap-3 max-w-[240px]">
      <button
        v-for="digit in ['1','2','3','4','5','6','7','8','9','','0','del']"
        :key="digit"
        class="w-20 h-14 rounded-xl text-xl font-medium transition-all"
        :class="digit === '' ? 'invisible' : 'bg-white hover:bg-gray-50 active:bg-gray-100 shadow-sm border border-gray-100 text-gray-900'"
        :disabled="digit === ''"
        @click="digit === 'del' ? removeDigit() : addDigit(digit)"
      >
        {{ digit === 'del' ? '&larr;' : digit }}
      </button>
    </div>
  </div>
</template>
