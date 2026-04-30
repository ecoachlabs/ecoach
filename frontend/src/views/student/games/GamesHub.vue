<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import {
  getLeaderboard,
  listGameSessions,
  type GameSessionDto,
  type LeaderboardEntryDto,
} from '@/ipc/games'

type GameCard = {
  key: string
  gameType: string
  label: string
  tagline: string
  desc: string
  to: string
  tags: string[]
  icon: string
  color: string
  colorEnd: string
  bgGrad: string
  heroGrad: string
}

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const loadError = ref('')
const sessions = ref<GameSessionDto[]>([])
const leaderboards = ref<Record<string, LeaderboardEntryDto[]>>({})

const games: GameCard[] = [
  {
    key: 'mindstack',
    gameType: 'mindstack',
    label: 'The Mind Stack',
    tagline: 'Control the fall. Master the knowledge.',
    desc: 'Answer correctly to clear pressure. Wrong answers stack the board against you.',
    to: '/student/games/mindstack',
    tags: ['PRECISION', 'KNOWLEDGE DEPTH', 'Board survival'],
    icon: 'MS',
    color: '#8B5CF6',
    colorEnd: '#6366F1',
    bgGrad: 'linear-gradient(145deg, #0d0b1e 0%, #1e1b4b 50%, #312e81 100%)',
    heroGrad: 'radial-gradient(ellipse at 68% 38%, rgba(139,92,246,0.45) 0%, rgba(99,102,241,0.15) 55%, transparent 75%)',
  },
  {
    key: 'tugofwar',
    gameType: 'tug_of_war',
    label: 'The Battle Ground',
    tagline: 'Speed meets knowledge on the rope.',
    desc: 'Pressure-based recall battles. Pull with correct answers and recover ground before time slips away.',
    to: '/student/games/tugofwar',
    tags: ['SPEED SCHOLAR', 'BATTLE MODE', 'Recall sprint'],
    icon: 'TW',
    color: '#EF4444',
    colorEnd: '#B91C1C',
    bgGrad: 'linear-gradient(145deg, #0f0000 0%, #450a0a 50%, #7f1d1d 100%)',
    heroGrad: 'radial-gradient(ellipse at 65% 35%, rgba(239,68,68,0.50) 0%, rgba(185,28,28,0.2) 55%, transparent 75%)',
  },
  {
    key: 'traps',
    gameType: 'traps',
    label: 'Concept Traps',
    tagline: 'See through the deception before it catches you.',
    desc: 'High-risk misconception rounds built to expose overconfidence and sharpen contrast thinking.',
    to: '/student/games/traps',
    tags: ['TRAP IMMUNITY', 'CLASSIFICATION', 'Misconception pressure'],
    icon: 'TR',
    color: '#F59E0B',
    colorEnd: '#D97706',
    bgGrad: 'linear-gradient(145deg, #0a0700 0%, #451a03 50%, #78350f 100%)',
    heroGrad: 'radial-gradient(ellipse at 65% 35%, rgba(245,158,11,0.45) 0%, rgba(217,119,6,0.18) 55%, transparent 75%)',
  },
]

const selectedIndex = ref(0)
const featured = computed(() => games[selectedIndex.value])

const featuredSessions = computed(() =>
  sessions.value
    .filter(session => session.game_type === featured.value.gameType)
    .slice()
    .sort((left, right) => right.id - left.id),
)

const featuredMetrics = computed(() => {
  const relevantSessions = featuredSessions.value
  const completedSessions = relevantSessions.filter(session => session.session_state === 'completed')
  const activeSession = relevantSessions.find(session => session.session_state === 'active') ?? null
  const best = relevantSessions.reduce((max, session) => Math.max(max, session.score), 0)
  const bestStreak = relevantSessions.reduce((max, session) => Math.max(max, session.best_streak), 0)
  const avg =
    completedSessions.length > 0
      ? Math.round(
          completedSessions.reduce((total, session) => total + session.score, 0) / completedSessions.length,
        )
      : 0

  return {
    runs: relevantSessions.length,
    best,
    avg,
    bestStreak,
    activeSession,
  }
})

const featuredStatusText = computed(() => {
  if (featuredMetrics.value.activeSession) {
    const session = featuredMetrics.value.activeSession
    return `Active run at ${session.score} points across ${session.rounds_played}/${session.rounds_total} rounds.`
  }

  if (featuredMetrics.value.runs > 0) {
    return `Best streak ${featuredMetrics.value.bestStreak}. ${featuredMetrics.value.runs} recorded runs so far.`
  }

  return 'No recorded runs yet. The first score is still waiting.'
})

const featuredLeaderboard = computed(() => leaderboards.value[featured.value.key] ?? [])
const recentRuns = computed(() => featuredSessions.value.slice(0, 3))

onMounted(() => {
  void loadGamesHub()
})

async function loadGamesHub() {
  if (!auth.currentAccount) {
    loading.value = false
    return
  }

  loading.value = true
  loadError.value = ''

  try {
    const studentId = auth.currentAccount.id
    const [gameSessions, ...allLeaderboards] = await Promise.all([
      listGameSessions(studentId, 24),
      ...games.map(game => getLeaderboard(game.gameType, 5)),
    ])

    sessions.value = gameSessions
    leaderboards.value = Object.fromEntries(
      games.map((game, index) => [game.key, allLeaderboards[index] as LeaderboardEntryDto[]]),
    )
  } catch (cause: any) {
    loadError.value = typeof cause === 'string' ? cause : cause?.message ?? 'Failed to load game history'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div
    class="game-hub h-full flex flex-col overflow-hidden"
    :style="{ background: featured.bgGrad, transition: 'background 500ms ease' }"
  >

    <div class="flex items-center justify-between px-6 py-3.5 shrink-0"
      style="border-bottom: 1px solid rgba(255,255,255,0.06);">
      <div class="flex items-center gap-3">
        <button
          class="text-[11px] text-white/40 hover:text-white/70 transition-colors"
          @click="router.push('/student')"
        >
          Back
        </button>
        <span class="text-white/15 text-lg">|</span>
        <h1 class="font-display text-base font-bold text-white tracking-tight">Game Arcade</h1>
      </div>
      <div class="flex items-center gap-5">
        <div class="text-center">
          <p class="text-xs font-bold font-display" :style="{ color: featured.color }">{{ featuredMetrics.runs }}</p>
          <p class="text-[9px] uppercase text-white/25 tracking-wider">Runs</p>
        </div>
        <div class="text-center">
          <p class="text-xs font-bold font-display text-white/30">{{ featuredMetrics.best || '--' }}</p>
          <p class="text-[9px] uppercase text-white/25 tracking-wider">Best</p>
        </div>
      </div>
    </div>

    <div class="flex-1 relative overflow-hidden flex min-h-0">
      <div
        class="absolute inset-0 pointer-events-none"
        :style="{ background: featured.heroGrad, transition: 'background 500ms ease' }"
      />

      <div
        class="absolute right-8 top-1/2 -translate-y-1/2 select-none pointer-events-none"
        :style="{
          fontSize: '180px',
          lineHeight: 1,
          color: featured.color,
          opacity: 0.05,
          fontFamily: 'Inter, sans-serif',
          transform: 'translateY(-50%) rotate(-10deg)',
          fontWeight: 900,
          letterSpacing: '-0.08em',
        }"
      >{{ featured.icon }}</div>

      <div class="relative z-10 flex flex-col justify-center pl-10 pr-6 py-8 w-[520px] shrink-0">
        <p v-if="loadError" class="text-xs mb-4" :style="{ color: 'rgba(255,255,255,0.6)' }">
          {{ loadError }}
        </p>

        <div class="flex items-center gap-2 mb-5 flex-wrap">
          <span
            v-for="(tag, i) in featured.tags"
            :key="tag"
            class="text-[10px] font-bold px-2.5 py-1 rounded tracking-wider uppercase"
            :style="{
              background: i === 0 ? featured.color : 'rgba(255,255,255,0.07)',
              color: i === 0 ? '#fff' : 'rgba(255,255,255,0.45)',
              borderRadius: '5px',
            }"
          >{{ tag }}</span>
        </div>

        <h2
          class="font-display font-bold text-white leading-none mb-3"
          style="font-size: clamp(2rem, 3.5vw, 2.75rem); letter-spacing: -0.025em;"
        >
          {{ featured.label }}
        </h2>

        <p class="text-[13px] font-medium italic mb-4" :style="{ color: featured.color }">
          {{ featured.tagline }}
        </p>

        <p class="text-[13px] text-white/45 leading-relaxed mb-8 max-w-[380px]">
          {{ featured.desc }}
        </p>

        <div class="flex items-end gap-8 mb-8">
          <div>
            <p class="text-xl font-bold font-display text-white/20 leading-none">{{ featuredMetrics.best || '--' }}</p>
            <p class="text-[10px] uppercase tracking-wider text-white/25 mt-1">Best</p>
          </div>
          <div>
            <p class="text-xl font-bold font-display text-white/20 leading-none">{{ featuredMetrics.avg || '--' }}</p>
            <p class="text-[10px] uppercase tracking-wider text-white/25 mt-1">Avg</p>
          </div>
          <div>
            <p class="text-xl font-bold font-display text-white/20 leading-none">{{ featuredMetrics.runs }}</p>
            <p class="text-[10px] uppercase tracking-wider text-white/25 mt-1">Runs</p>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <button
            class="play-btn flex items-center gap-2.5 px-7 py-3 rounded-full font-bold text-sm text-white"
            :style="{ background: `linear-gradient(135deg, ${featured.color}, ${featured.colorEnd})` }"
            @click="router.push(featured.to)"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
              <path d="M2.5 2l8 4-8 4V2z"/>
            </svg>
            Play Now
          </button>
          <span class="text-[11px] text-white/25">{{ featuredStatusText }}</span>
        </div>
      </div>

      <div class="flex-1 flex items-center justify-center relative pr-8">
        <div class="icon-showcase" :style="{ '--c': featured.color }">
          <div class="ring ring-1" />
          <div class="ring ring-2" />
          <div class="ring ring-3" />
          <div class="icon-glyph text-6xl" :style="{ color: featured.color }">
            {{ featured.icon }}
          </div>
        </div>

        <div class="info-panel">
          <div>
            <p class="panel-label">Top Scores</p>
            <div v-if="featuredLeaderboard.length" class="space-y-2 mt-3">
              <div
                v-for="(entry, index) in featuredLeaderboard.slice(0, 3)"
                :key="`${entry.student_id}-${index}`"
                class="panel-row"
              >
                <span class="rank-chip" :style="{ color: featured.color }">#{{ index + 1 }}</span>
                <div class="flex-1 min-w-0">
                  <p class="truncate text-sm font-semibold text-white/85">{{ entry.display_name }}</p>
                  <p class="text-[10px] text-white/35">{{ entry.games_played }} runs</p>
                </div>
                <span class="text-sm font-bold text-white/70">{{ entry.best_score }}</span>
              </div>
            </div>
            <p v-else class="panel-empty">No leaderboard entries recorded yet.</p>
          </div>

          <div class="mt-4">
            <p class="panel-label">Recent Runs</p>
            <div v-if="recentRuns.length" class="space-y-2 mt-3">
              <div
                v-for="run in recentRuns"
                :key="run.id"
                class="panel-row"
              >
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-semibold text-white/85">{{ run.score }} points</p>
                  <p class="text-[10px] text-white/35">
                    {{ run.rounds_played }}/{{ run.rounds_total }} rounds · {{ run.session_state }}
                  </p>
                </div>
                <span class="text-[10px] font-semibold" :style="{ color: featured.color }">
                  streak {{ run.best_streak }}
                </span>
              </div>
            </div>
            <p v-else class="panel-empty">Your first run will show up here.</p>
          </div>
        </div>
      </div>
    </div>

    <div class="shrink-0 px-6 py-4"
      style="border-top: 1px solid rgba(255,255,255,0.06); background: rgba(0,0,0,0.3);">
      <div class="flex items-center gap-2.5 overflow-x-auto no-scrollbar">
        <button
          v-for="(game, i) in games"
          :key="game.key"
          class="game-chip shrink-0 flex flex-col items-center gap-2 px-4 py-3 rounded-xl"
          :style="{
            background: selectedIndex === i ? 'rgba(255,255,255,0.10)' : 'rgba(255,255,255,0.04)',
            border: selectedIndex === i ? `1px solid ${game.color}44` : '1px solid rgba(255,255,255,0.05)',
            minWidth: '90px',
          }"
          @click="selectedIndex = i"
        >
          <div
            class="w-11 h-11 rounded-[10px] flex items-center justify-center text-sm font-bold"
            :style="{
              background: selectedIndex === i ? game.color + '22' : 'rgba(255,255,255,0.05)',
              color: selectedIndex === i ? game.color : 'rgba(255,255,255,0.3)',
            }"
          >{{ game.icon }}</div>
          <p
            class="text-[10px] font-semibold text-center leading-tight"
            :style="{
              color: selectedIndex === i ? 'rgba(255,255,255,0.85)' : 'rgba(255,255,255,0.3)',
              maxWidth: '76px',
            }"
          >{{ game.label }}</p>
          <div
            v-if="selectedIndex === i"
            class="w-4 h-[2px] rounded-full"
            :style="{ backgroundColor: game.color }"
          />
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.play-btn {
  box-shadow: 0 6px 24px rgba(0,0,0,0.35);
  transition: transform 180ms cubic-bezier(0.34,1.56,0.64,1), box-shadow 150ms ease;
}

.play-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 32px rgba(0,0,0,0.45);
}

.play-btn:active {
  transform: scale(0.97);
}

.icon-showcase {
  position: relative;
  width: 180px;
  height: 180px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ring {
  position: absolute;
  border-radius: 50%;
  border: 1px solid var(--c);
}

.ring-1 {
  width: 168px;
  height: 168px;
  opacity: 0.12;
  animation: ring-breathe 3.5s ease-in-out infinite;
}

.ring-2 {
  width: 130px;
  height: 130px;
  opacity: 0.18;
  animation: ring-breathe 3.5s 0.6s ease-in-out infinite;
}

.ring-3 {
  width: 96px;
  height: 96px;
  opacity: 0.10;
  background: color-mix(in srgb, var(--c) 12%, transparent);
  animation: ring-breathe 3.5s 1.2s ease-in-out infinite;
}

.icon-glyph {
  position: relative;
  z-index: 2;
  animation: icon-float 4s ease-in-out infinite;
  font-family: Inter, sans-serif;
  font-weight: 900;
  letter-spacing: -0.08em;
}

@keyframes ring-breathe {
  0%, 100% {
    transform: scale(1);
    opacity: 0.10;
  }

  50% {
    transform: scale(1.1);
    opacity: 0.20;
  }
}

@keyframes icon-float {
  0%, 100% {
    transform: translateY(0) rotate(0deg);
  }

  50% {
    transform: translateY(-10px) rotate(3deg);
  }
}

.game-chip {
  cursor: pointer;
  transition: background 200ms ease, border-color 200ms ease, transform 150ms ease;
}

.game-chip:hover {
  transform: translateY(-3px);
}

.info-panel {
  position: absolute;
  right: 0;
  bottom: 2rem;
  width: 320px;
  padding: 16px;
  border-radius: 8px;
  background: rgba(8, 8, 10, 0.58);
  border: 1px solid rgba(255,255,255,0.08);
  backdrop-filter: blur(12px);
}

.panel-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: rgba(255,255,255,0.32);
}

.panel-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid rgba(255,255,255,0.06);
}

.panel-row:last-child {
  border-bottom: none;
}

.panel-empty {
  margin-top: 12px;
  font-size: 12px;
  color: rgba(255,255,255,0.42);
}

.rank-chip {
  width: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
}

.no-scrollbar::-webkit-scrollbar {
  display: none;
}

.no-scrollbar {
  scrollbar-width: none;
}
</style>
