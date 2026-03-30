import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ThemeMode = 'student' | 'parent' | 'admin'
export type EmotionalMode = 'normal' | 'recovery' | 'pressure' | 'elite' | 'game' | 'celebration' | 'focus'

export const useUiStore = defineStore('ui', () => {
  const theme = ref<ThemeMode>('student')
  const emotionalMode = ref<EmotionalMode>('normal')
  const isDark = ref(false)
  const sidebarCollapsed = ref(false)

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
    document.documentElement.setAttribute('data-dark', String(isDark.value))
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
