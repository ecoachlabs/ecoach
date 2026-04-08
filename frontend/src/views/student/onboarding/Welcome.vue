<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
</script>

<template>
  <div class="h-full flex flex-col items-center justify-center" :style="{ backgroundColor: 'var(--paper)' }">
    <div class="w-full max-w-lg px-8 text-center">

      <!-- Logo mark -->
      <div class="w-16 h-16 rounded-2xl flex items-center justify-center mx-auto mb-8"
        :style="{ backgroundColor: 'var(--ink)', color: 'var(--paper)' }">
        <span class="font-display font-black text-2xl">e</span>
      </div>

      <h1 class="font-display text-4xl font-bold tracking-tight mb-3"
        :style="{ color: 'var(--ink)' }">
        Welcome to eCoach
      </h1>
      <p class="text-base mb-2" :style="{ color: 'var(--ink-secondary)' }">
        Your personal academic intelligence coach for BECE exam preparation.
      </p>
      <p class="text-sm mb-10" :style="{ color: 'var(--ink-muted)' }">
        We will help you understand your strengths, repair your weaknesses, and prepare you for exam success — one session at a time.
      </p>

      <!-- Features row -->
      <div class="grid grid-cols-3 gap-3 mb-10">
        <div v-for="item in [
          { sym: '◎', label: 'Deep Diagnosis', desc: 'Find root causes' },
          { sym: '◈', label: 'Smart Practice', desc: 'Targeted sessions' },
          { sym: '★', label: 'Track Progress', desc: 'See you improve' },
        ]" :key="item.label"
          class="flex flex-col items-center gap-2 px-3 py-4 rounded-2xl border"
          :style="{ borderColor: 'transparent', backgroundColor: 'var(--surface)' }"
        >
          <span class="text-xl" :style="{ color: 'var(--ink-secondary)' }">{{ item.sym }}</span>
          <p class="text-xs font-bold" :style="{ color: 'var(--ink)' }">{{ item.label }}</p>
          <p class="text-[10px]" :style="{ color: 'var(--ink-muted)' }">{{ item.desc }}</p>
        </div>
      </div>

      <button
        class="start-btn w-full"
        @click="router.push('/student/onboarding/subjects')"
      >Get Started →</button>
      <p class="text-xs mt-3" :style="{ color: 'var(--ink-muted)' }">Takes about 5 minutes to set up</p>
    </div>
  </div>
</template>

<style scoped>
.start-btn {
  padding: 14px;
  border-radius: 14px;
  background: var(--accent);
  color: white;
  font-size: 15px;
  font-weight: 700;
  cursor: pointer;
  border: none;
  transition: opacity 140ms, transform 140ms;
}
.start-btn:hover { opacity: 0.87; transform: translateY(-2px); }
</style>

