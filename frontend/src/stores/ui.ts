import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ThemeMode = 'student' | 'parent' | 'admin'
export type EmotionalMode = 'normal' | 'recovery' | 'pressure' | 'elite' | 'game' | 'celebration' | 'focus'

export const useUiStore = defineStore('ui', () => {
  const theme = ref<ThemeMode>('student')
  const emotionalMode = ref<EmotionalMode>('normal')
  const isDark = ref(localStorage.getItem('ecoach-dark') === 'true')
  const sidebarCollapsed = ref(false)

  // Apply the current dark state to <html> immediately (called on store init and toggle)
  function applyDark() {
    if (isDark.value) {
      document.documentElement.setAttribute('data-dark', 'true')
    } else {
      document.documentElement.removeAttribute('data-dark')
    }
  }

  // Apply on store creation so persisted preference takes effect before first render
  applyDark()

  function setTheme(t: ThemeMode) {
    theme.value = t
    document.documentElement.setAttribute('data-theme', t)
  }

  function setEmotionalMode(mode: EmotionalMode) {
    emotionalMode.value = mode
    if (mode === 'normal') {
      document.documentElement.removeAttribute('data-mode')
    } else {
      document.documentElement.setAttribute('data-mode', mode)
    }
  }

  function toggleDark() {
    isDark.value = !isDark.value
    localStorage.setItem('ecoach-dark', String(isDark.value))
    applyDark()
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  return {
    theme,
    emotionalMode,
    isDark,
    sidebarCollapsed,
    setTheme,
    setEmotionalMode,
    toggleDark,
    toggleSidebar,
  }
})
