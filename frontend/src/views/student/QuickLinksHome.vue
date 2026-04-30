<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { quickLinkItems } from '@/layouts/studentNav'

const router = useRouter()

const quickLinkGroups = computed(() => [
  {
    title: 'Practice',
    subtitle: 'Open the sessions and one-off runs you want fast.',
    items: quickLinkItems.filter(item => ['custom-test', 'mock-history', 'elite'].includes(item.name)),
  },
  {
    title: 'Repair',
    subtitle: 'Go straight to the places that sharpen weak spots.',
    items: quickLinkItems.filter(item => ['gap-scan', 'retry-zone', 'beat-yesterday'].includes(item.name)),
  },
  {
    title: 'Study tools',
    subtitle: 'Pick up the references, audio, and working space you need.',
    items: quickLinkItems.filter(item => ['revision-box', 'audio-glossary', 'formula-lab'].includes(item.name)),
  },
  {
    title: 'Planning',
    subtitle: 'Keep momentum, organize time, and manage your extras.',
    items: quickLinkItems.filter(item => ['progress', 'calendar', 'onboarding', 'uploads'].includes(item.name)),
  },
])

function openLink(to: string, hash?: string) {
  void router.push(hash ? { path: to, hash } : to)
}
</script>

<template>
  <div class="quick-links-page">
    <header class="quick-links-header">
      <div>
        <p class="eyebrow">Quick Links</p>
        <h1>Everything else, one tap away.</h1>
        <p class="subtitle">Jump straight into the extra tools and one-off tasks you still need close.</p>
      </div>
      <div class="count-badge">{{ quickLinkItems.length }} links</div>
    </header>

    <div class="quick-links-grid">
      <section v-for="group in quickLinkGroups" :key="group.title" class="quick-links-group">
        <div class="group-heading">
          <h2>{{ group.title }}</h2>
          <p>{{ group.subtitle }}</p>
        </div>

        <div class="group-links">
          <button
            v-for="item in group.items"
            :key="item.name"
            class="quick-link-card"
            type="button"
            @click="openLink(item.to, item.hash)"
          >
            <div class="quick-link-icon" :style="{ '--card-accent': item.color }">
              <component :is="item.icon" :size="18" weight="fill" :style="{ color: item.color }" />
            </div>
            <div class="quick-link-copy">
              <p class="quick-link-label">{{ item.label }}</p>
              <p class="quick-link-path">{{ item.hash ? `${item.to}${item.hash}` : item.to }}</p>
            </div>
            <span class="quick-link-arrow">Open</span>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.quick-links-page {
  height: 100%;
  overflow-y: auto;
  padding: 28px 30px 36px;
  background:
    radial-gradient(circle at top right, rgba(99, 102, 241, 0.10), transparent 28%),
    radial-gradient(circle at bottom left, rgba(14, 165, 233, 0.10), transparent 30%),
    var(--paper);
}

.quick-links-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 22px;
}

.eyebrow {
  margin: 0 0 8px;
  font-size: 10px;
  line-height: 1;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--accent);
}

.quick-links-header h1 {
  margin: 0;
  color: var(--ink);
  font-size: 32px;
  line-height: 0.96;
  font-weight: 800;
  letter-spacing: 0;
}

.subtitle {
  max-width: 580px;
  margin: 10px 0 0;
  color: var(--ink-muted);
  font-size: 14px;
  line-height: 1.5;
}

.count-badge {
  flex-shrink: 0;
  min-width: 92px;
  padding: 10px 14px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--accent) 10%, var(--surface));
  color: var(--accent);
  font-size: 12px;
  font-weight: 700;
  text-align: center;
}

.quick-links-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.quick-links-group {
  padding: 18px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--surface) 92%, white 8%);
  border: 1px solid color-mix(in srgb, var(--border-soft) 82%, transparent);
}

.group-heading h2 {
  margin: 0;
  color: var(--ink);
  font-size: 17px;
  line-height: 1.1;
  font-weight: 700;
}

.group-heading p {
  margin: 6px 0 0;
  color: var(--ink-muted);
  font-size: 12px;
  line-height: 1.45;
}

.group-links {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 16px;
}

.quick-link-card {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--border-soft) 78%, transparent);
  border-radius: 8px;
  background: var(--paper);
  text-align: left;
  cursor: pointer;
  transition: transform 140ms ease, border-color 140ms ease, background-color 140ms ease;
}

.quick-link-card:hover {
  transform: translateY(-1px);
  border-color: color-mix(in srgb, var(--accent) 24%, var(--border-soft));
  background: color-mix(in srgb, var(--surface) 82%, white 18%);
}

.quick-link-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--card-accent) 12%, white 88%);
  flex-shrink: 0;
}

.quick-link-copy {
  min-width: 0;
  flex: 1;
}

.quick-link-label {
  margin: 0;
  color: var(--ink);
  font-size: 13px;
  line-height: 1.2;
  font-weight: 700;
}

.quick-link-path {
  margin: 4px 0 0;
  color: var(--ink-muted);
  font-size: 11px;
  line-height: 1.35;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.quick-link-arrow {
  flex-shrink: 0;
  color: var(--ink-muted);
  font-size: 11px;
  font-weight: 700;
}

@media (max-width: 960px) {
  .quick-links-page {
    padding: 22px 20px 30px;
  }

  .quick-links-header {
    flex-direction: column;
  }

  .quick-links-grid {
    grid-template-columns: 1fr;
  }

  .quick-links-header h1 {
    font-size: 28px;
  }
}
</style>
