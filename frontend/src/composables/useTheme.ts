import { computed } from 'vue'
import { useUiStore, type ThemeMode, type EmotionalMode } from '@/stores/ui'

export function useTheme() {
  const ui = useUiStore()

  const theme = computed(() => ui.theme)
  const emotionalMode = computed(() => ui.emotionalMode)
  const isDark = computed(() => ui.isDark)

  function setTheme(t: ThemeMode) {
    ui.setTheme(t)
  }

  function setMode(mode: EmotionalMode) {
    ui.setEmotionalMode(mode)
  }

  function toggleDark() {
    ui.toggleDark()
  }

  // Helper: apply emotional mode for a duration then revert
  function flashMode(mode: EmotionalMode, durationMs: number = 2000) {
    const previous = ui.emotionalMode
    ui.setEmotionalMode(mode)
    setTimeout(() => ui.setEmotionalMode(previous), durationMs)
  }

  return {
    theme,
    emotionalMode,
    isDark,
    setTheme,
    setMode,
    toggleDark,
    flashMode,
  }
}
