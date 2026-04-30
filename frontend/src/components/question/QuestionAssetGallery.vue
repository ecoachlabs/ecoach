<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import {
  fetchQuestionAssetObjectUrl,
  listQuestionAssets,
  type QuestionAssetMetaDto,
  type QuestionAssetScope,
} from '@/ipc/pastPaperAdmin'

/**
 * Lazy image gallery for past-paper attachments.
 *
 * Given a question id and an optional scope filter, fetches the asset
 * metadata on mount, then streams each image's bytes into an object URL
 * only when we're about to render it. URLs are revoked on unmount so a
 * long test session doesn't leak blob memory.
 *
 * Kept deliberately small and Nothing-style: no captions, no zoom
 * plugin — just clean image blocks that sit above the stem or beside
 * an option. If a figure is ever flagged with alt_text, we use it for
 * screen readers.
 */
const props = withDefaults(
  defineProps<{
    questionId: number
    scope?: QuestionAssetScope
    /** When scope === 'option', limit to assets attached to this option id. */
    scopeRef?: number | null
  }>(),
  { scope: undefined, scopeRef: null },
)

const assets = ref<QuestionAssetMetaDto[]>([])
const urls = ref<Record<number, string>>({})
const loading = ref(false)

async function load(): Promise<void> {
  loading.value = true
  try {
    const list = await listQuestionAssets(props.questionId)
    assets.value = list.filter(a => {
      if (props.scope && a.scope !== props.scope) return false
      if (props.scope === 'option' && a.scope_ref !== (props.scopeRef ?? null)) return false
      return true
    })
  } catch {
    assets.value = []
  } finally {
    loading.value = false
  }

  // Kick off byte fetches in parallel. Order doesn't matter — each
  // thumbnail mounts as soon as its URL arrives.
  for (const asset of assets.value) {
    void ensureUrl(asset.asset_id)
  }
}

async function ensureUrl(assetId: number): Promise<void> {
  if (urls.value[assetId]) return
  try {
    const url = await fetchQuestionAssetObjectUrl(assetId)
    urls.value = { ...urls.value, [assetId]: url }
  } catch { /* swallow — a missing image shouldn't crash the question */ }
}

watch(
  () => [props.questionId, props.scope, props.scopeRef],
  () => {
    // Revoke stale URLs before reloading.
    for (const url of Object.values(urls.value)) URL.revokeObjectURL(url)
    urls.value = {}
    assets.value = []
    void load()
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  for (const url of Object.values(urls.value)) URL.revokeObjectURL(url)
})
</script>

<template>
  <div v-if="assets.length > 0" class="qa-gallery" :data-scope="scope">
    <figure
      v-for="asset in assets"
      :key="asset.asset_id"
      class="qa-item"
    >
      <img
        v-if="urls[asset.asset_id]"
        :src="urls[asset.asset_id]"
        :alt="asset.alt_text ?? ''"
        class="qa-img"
      />
      <span v-else class="qa-loading" aria-hidden="true">···</span>
    </figure>
  </div>
</template>

<style scoped>
.qa-gallery {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin: 12px 0 4px;
}
.qa-item {
  margin: 0;
  padding: 0;
  max-width: 320px;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid rgba(0, 0, 0, 0.08);
  background: rgba(0, 0, 0, 0.02);
}
.qa-img {
  display: block;
  max-width: 100%;
  max-height: 240px;
  height: auto;
  object-fit: contain;
}
.qa-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 180px;
  height: 120px;
  font-family: 'Space Mono', ui-monospace, monospace;
  font-size: 14px;
  letter-spacing: 0.3em;
  color: rgba(0, 0, 0, 0.4);
}

/* When the gallery sits inside the option row (scope="option") keep
   thumbnails compact so they don't squash the option text. */
.qa-gallery[data-scope='option'] .qa-item { max-width: 160px; }
.qa-gallery[data-scope='option'] .qa-img { max-height: 120px; }

@media (prefers-color-scheme: dark) {
  .qa-item {
    border-color: rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.04);
  }
  .qa-loading { color: rgba(255, 255, 255, 0.4); }
}
</style>
