import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AccountDto, AccountSummaryDto } from '@/types'
import * as identityIpc from '@/ipc/identity'

export const useAuthStore = defineStore('auth', () => {
  const accounts = ref<AccountSummaryDto[]>([])
  const currentAccount = ref<AccountDto | null>(null)
  const isAuthenticated = computed(() => currentAccount.value !== null)
  const role = computed(() => currentAccount.value?.account_type ?? null)

  async function loadAccounts() {
    accounts.value = await identityIpc.listAccounts()
  }

  async function login(accountId: number, pin: string) {
    currentAccount.value = await identityIpc.loginWithPin(accountId, pin)
  }

  function logout() {
    currentAccount.value = null
  }

  return {
    accounts,
    currentAccount,
    isAuthenticated,
    role,
    loadAccounts,
    login,
    logout,
  }
})
