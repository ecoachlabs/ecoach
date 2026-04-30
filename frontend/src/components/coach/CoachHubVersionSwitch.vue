<script setup lang="ts">
/**
 * Small floating switcher that lets the user jump between
 * CoachHub v1 / v2 / v3 for side-by-side comparison.
 *
 * Placement is absolute — drop into any parent with `position: relative`
 * or it will anchor to the nearest non-static ancestor.
 */
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

type VersionKey = 'v1' | 'v2' | 'v3'
interface Version {
  key: VersionKey
  label: string
  to: string
  title: string
}

const versions: Version[] = [
  { key: 'v1', label: 'V1', to: '/student',    title: 'CoachHub — original' },
  { key: 'v2', label: 'V2', to: '/student/v2', title: 'CoachHub — bento' },
  { key: 'v3', label: 'V3', to: '/student/v3', title: 'CoachHub — Nothing' },
]

const route = useRoute()
const router = useRouter()

const current = computed<VersionKey>(() => {
  if (route.path.endsWith('/v3')) return 'v3'
  if (route.path.endsWith('/v2')) return 'v2'
  return 'v1'
})

function go(v: Version) {
  if (v.key !== current.value) router.push(v.to)
}
</script>

<template>
  <div class="ch-vs" role="group" aria-label="CoachHub version switcher">
    <span class="ch-vs-label">VERSION</span>
    <div class="ch-vs-rail">
      <button
        v-for="v in versions" :key="v.key"
        type="button"
        class="ch-vs-btn"
        :class="{ 'ch-vs-btn--on': current === v.key }"
        :title="v.title"
        @click="go(v)"
      >{{ v.label }}</button>
    </div>
  </div>
</template>

<style scoped>
.ch-vs {
  position: absolute;
  top: 14px;
  right: 16px;
  z-index: 40;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px 4px 10px;
  border-radius: 999px;
  background: var(--surface, #ffffff);
  border: 1px solid var(--border-soft, rgba(0,0,0,0.08));
  box-shadow:
    0 1px 2px rgba(26, 22, 18, 0.04),
    0 6px 18px rgba(26, 22, 18, 0.06);
  user-select: none;
}
.ch-vs-label {
  font-family: 'Space Mono', ui-monospace, Consolas, monospace;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.22em;
  color: var(--ink-muted, #9e958d);
}
.ch-vs-rail {
  display: inline-flex;
  background: var(--paper, #faf8f5);
  border-radius: 999px;
  padding: 2px;
  gap: 1px;
}
.ch-vs-btn {
  font-family: 'Space Mono', ui-monospace, Consolas, monospace;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.18em;
  color: var(--ink-secondary, #5c5650);
  background: transparent;
  border: none;
  padding: 4px 10px;
  cursor: pointer;
  border-radius: 999px;
  transition: background 140ms ease, color 140ms ease, transform 120ms ease;
}
.ch-vs-btn:hover {
  color: var(--ink, #1a1612);
  transform: translateY(-0.5px);
}
.ch-vs-btn--on {
  background: var(--ink, #1a1612);
  color: var(--surface, #ffffff);
}
.ch-vs-btn--on:hover {
  color: var(--surface, #ffffff);
  transform: none;
}

/* dark mode, if the app sets data-dark at a parent */
:global([data-dark='true']) .ch-vs {
  background: var(--surface, #1c1917);
  border-color: var(--border-soft, rgba(255,255,255,0.08));
  box-shadow: 0 6px 18px rgba(0,0,0,0.35);
}
:global([data-dark='true']) .ch-vs-rail {
  background: var(--paper, #0e0c0a);
}

@media (max-width: 640px) {
  .ch-vs { top: 10px; right: 10px; padding: 3px 6px 3px 8px; }
  .ch-vs-label { display: none; }
}
</style>
