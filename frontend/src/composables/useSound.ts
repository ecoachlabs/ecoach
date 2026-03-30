import { ref } from 'vue'
import { Howl } from 'howler'

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

// Sound effect registry
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
  ropePull: { path: '/sounds/game/rope-pull.mp3', category: 'game' },
  heartbeat: { path: '/sounds/game/heartbeat.mp3', category: 'game' },
}

// Howl cache
const howlCache = new Map<string, Howl>()

export function useSound() {
  function getHowl(name: string): Howl | null {
    const sound = sounds[name]
    if (!sound) return null

    if (!howlCache.has(name)) {
      howlCache.set(name, new Howl({
        src: [sound.path],
        volume: config.value.volume,
        preload: false,
      }))
    }
    return howlCache.get(name) || null
  }

  function play(name: string, volumeOverride?: number) {
    if (!config.value.enabled) return

    const sound = sounds[name]
    if (!sound) return
    if (config.value.mutedCategories.has(sound.category)) return

    const howl = getHowl(name)
    if (!howl) return

    howl.volume(volumeOverride ?? config.value.volume)
    howl.play()
  }

  function stop(name: string) {
    const howl = howlCache.get(name)
    if (howl) howl.stop()
  }

  function stopAll() {
    howlCache.forEach(howl => howl.stop())
  }

  function setEnabled(enabled: boolean) {
    config.value.enabled = enabled
    if (!enabled) stopAll()
  }

  function setVolume(volume: number) {
    config.value.volume = Math.max(0, Math.min(1, volume))
    howlCache.forEach(howl => howl.volume(config.value.volume))
  }

  function muteCategory(category: SoundCategory) {
    config.value.mutedCategories.add(category)
  }

  function unmuteCategory(category: SoundCategory) {
    config.value.mutedCategories.delete(category)
  }

  // Preload a set of sounds for a mode
  function preload(names: string[]) {
    names.forEach(name => {
      const howl = getHowl(name)
      if (howl) howl.load()
    })
  }

  return {
    play,
    stop,
    stopAll,
    config,
    setEnabled,
    setVolume,
    muteCategory,
    unmuteCategory,
    preload,
  }
}
