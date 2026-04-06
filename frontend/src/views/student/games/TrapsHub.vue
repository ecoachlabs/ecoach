<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listTrapsPairs } from '@/ipc/games'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import PageHeader from '@/components/layout/PageHeader.vue'

const auth = useAuthStore()
const router = useRouter()
const loading = ref(true)
const pairs = ref<any[]>([])

onMounted(async () => {
  if (!auth.currentAccount) return
  try {
    pairs.value = await listTrapsPairs(auth.currentAccount.id, 1, [])
  } catch {}
  loading.value = false
})

const modes = [
  { key: 'difference', label: 'Difference Drill', desc: 'Sort cards into concept bins. Physics-based card dropping.', icon: '◈', difficulty: 'All levels' },
  { key: 'similarity', label: 'Similarity Trap', desc: 'Survive deliberately deceptive questions. Danger aesthetic.', icon: '⚠', difficulty: 'Advanced' },
  { key: 'know', label: 'Know the Difference', desc: 'Compare concepts side by side. Tap-to-reveal.', icon: '⟺', difficulty: 'Beginner' },
  { key: 'which', label: 'Which Is Which', desc: 'Speed recognition. Two buttons. Heartbeat acceleration.', icon: '⚡', difficulty: 'All levels' },
  { key: 'unmask', label: 'Unmask', desc: 'Guess the concept from progressive clues.', icon: '◉', difficulty: 'Intermediate' },
]
</script>

<template>
  <div class="flex-1 overflow-y-auto p-7">
    <PageHeader title="Concept Traps" subtitle="Can you tell the difference? Five modes to test your concept separation skills." back-to="/student/games" />

    <!-- Modes -->
    <div class="space-y-3 mb-6">
      <AppCard v-for="m in modes" :key="m.key" hover padding="md" @click="router.push('/student/games/traps')">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl flex items-center justify-center text-lg"
            :style="{backgroundColor:'var(--gold-light)',color:'var(--gold)'}">{{ m.icon }}</div>
          <div class="flex-1">
            <div class="flex items-center gap-2">
              <p class="text-sm font-semibold" :style="{color:'var(--ink)'}">{{ m.label }}</p>
              <AppBadge color="muted" size="xs">{{ m.difficulty }}</AppBadge>
            </div>
            <p class="text-[10px]" :style="{color:'var(--ink-muted)'}">{{ m.desc }}</p>
          </div>
          <AppButton variant="secondary" size="sm">Play</AppButton>
        </div>
      </AppCard>
    </div>

    <!-- Available pairs -->
    <div v-if="pairs.length">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{color:'var(--ink-muted)'}">Available Concept Pairs</h3>
      <div class="grid grid-cols-2 gap-2">
        <AppCard v-for="p in pairs" :key="p.pair_id" padding="sm" hover>
          <p class="text-xs font-semibold" :style="{color:'var(--ink)'}">{{ p.title }}</p>
          <div class="flex items-center gap-2 mt-1 text-[10px]" :style="{color:'var(--ink-muted)'}">
            <span>{{ p.left_label }}</span> <span>vs</span> <span>{{ p.right_label }}</span>
          </div>
          <AppBadge color="gold" size="xs" class="mt-1">{{ p.recommended_mode }}</AppBadge>
        </AppCard>
      </div>
    </div>

    <!-- Recommended journey -->
    <AppCard padding="md" class="mt-6" glow="gold">
      <p class="text-xs font-semibold uppercase mb-1" :style="{color:'var(--gold)'}">Recommended Journey</p>
      <p class="text-sm" :style="{color:'var(--ink-secondary)'}">
        Know the Difference → Difference Drill → Similarity Trap → Which Is Which → Unmask
      </p>
    </AppCard>
  </div>
</template>
