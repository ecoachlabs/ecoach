import { ref } from 'vue'

export type SoundCategory = 'feedback' | 'transition' | 'ambient' | 'game' | 'celebration'

interface SoundConfig {
  enabled: boolean
  volume: number // 0-1
  mutedCategories: Set<SoundCategory>
}

const config = ref<SoundConfig>({
  enabled: true,
  volume: 0.7,
  mutedCategories: new Set(),
})

// Sound effect registry - maps names to paths
const sounds: Record<string, { path: string; category: SoundCategory }> = {
  // Feedback
  correct: { path: '/sounds/feedback/correct.mp3', category: 'feedback' },
  wrong: { path: '/sounds/feedback/wrong.mp3', category: 'feedback' },
  streak: { path: '/sounds/feedback/streak.mp3', category: 'feedback' },
  combo: { path: '/sounds/feedback/combo.mp3', category: 'feedback' },

  // Transitions
  modeEnter: { path: '/sounds/transitions/mode-enter.mp3', category: 'transition' },
  phaseChange: { path: '/sounds/transitions/phase-change.mp3', category: 'transition' },
  sessionStart: { path: '/sounds/transitions/session-start.mp3', category: 'transition' },
  sessionEnd: { path: '/sounds/transitions/session-end.mp3', category: 'transition' },

  // Celebrations
  milestone: { path: '/sounds/celebration/milestone.mp3', category: 'celebration' },
  levelUp: { path: '/sounds/celebration/level-up.mp3', category: 'celebration' },
  mastery: { path: '/sounds/celebration/mastery.mp3', category: 'celebration' },

  // Game
  blockDrop: { path: '/sounds/game/block-drop.mp3', category: 'game' },
  morphActivate: { path: '/sounds/game/morph.mp3', category: 'game' },
  cardSort: { path: '/sounds/game/card-sort.mp3', category: 'game' },
}

// Audio cache
const audioCache = new Map<string, HTMLAudioElement>()

export function useSound() {
  function play(name: string, volumeOverride?: number) {
    if (!config.value.enabled) return

    const sound = sounds[name]
    if (!sound) return
    if (config.value.mutedCategories.has(sound.category)) return

    try {
      let audio = audioCache.get(name)
      if (!audio) {
        audio = new Audio(sound.path)
        audioCache.set(name, audio)
      }
      audio.volume = volumeOverride ?? config.value.volume
      audio.currentTime = 0
      audio.play().catch(() => {}) // Silently fail if audio not available
    } catch {
      // Audio not critical, fail silently
    }
  }

  function setEnabled(enabled: boolean) {
    config.value.enabled = enabled
  }

  function setVolume(volume: number) {
    config.value.volume = Math.max(0, Math.min(1, volume))
  }

  function muteCategory(category: SoundCategory) {
    config.value.mutedCategories.add(category)
  }

  function unmuteCategory(category: SoundCategory) {
    config.value.mutedCategories.delete(category)
  }

  return {
    play,
    config,
    setEnabled,
    setVolume,
    muteCategory,
    unmuteCategory,
  }
}
