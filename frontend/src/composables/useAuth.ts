import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useUiStore } from '@/stores/ui'

export function useAuth() {
  const auth = useAuthStore()
  const ui = useUiStore()
  const router = useRouter()

  const isAuthenticated = computed(() => auth.isAuthenticated)
  const currentAccount = computed(() => auth.currentAccount)
  const role = computed(() => auth.role)
  const studentId = computed(() => auth.currentAccount?.id ?? null)
  const displayName = computed(() => auth.currentAccount?.display_name ?? '')
  const isStudent = computed(() => auth.role === 'student')
  const isParent = computed(() => auth.role === 'parent')
  const isAdmin = computed(() => auth.role === 'admin')

  async function login(accountId: number, pin: string) {
    await auth.login(accountId, pin)

    // Set theme based on role
    if (auth.role === 'parent') ui.setTheme('parent')
    else if (auth.role === 'admin') ui.setTheme('admin')
    else ui.setTheme('student')

    // Navigate to correct portal
    if (auth.role === 'parent') router.push('/parent')
    else if (auth.role === 'admin') router.push('/admin')
    else router.push('/student')
  }

  function logout() {
    auth.logout()
    ui.setTheme('student')
    ui.setEmotionalMode('normal')
    router.push('/')
  }

  function requireAuth(): boolean {
    if (!isAuthenticated.value) {
      router.push('/')
      return false
    }
    return true
  }

  return {
    isAuthenticated,
    currentAccount,
    role,
    studentId,
    displayName,
    isStudent,
    isParent,
    isAdmin,
    login,
    logout,
    requireAuth,
  }
}
