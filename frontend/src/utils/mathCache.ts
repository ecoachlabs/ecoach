import katex from 'katex'
import { measureActivePerfSync, recordActivePerfPoint } from '@/utils/perfTrace'

/**
 * Shared, module-level KaTeX render cache.
 *
 * `katex.renderToString` is synchronous and costs ~10–30ms per complex
 * expression. A typical test question has a stem + 4 options + an
 * explanation — so a fresh question is 50–150ms of main-thread work
 * before the browser can paint. That's the primary reason the NEXT
 * button feels sluggish.
 *
 * Caching turns every second+ render of the same expression into an
 * O(1) Map lookup. Combined with `prewarmMathText` (see below), which
 * populates the cache in idle time for every question in a session,
 * NEXT becomes a pure DOM swap with zero parsing on the critical path.
 *
 * Bounded to prevent unbounded growth during a long session.
 */
const RENDER_CACHE = new Map<string, string>()
const RENDER_CACHE_MAX = 1000

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

export function renderKatex(expression: string, display: boolean): string {
  if (!expression) return ''
  const key = `${display ? 'D' : 'I'}\x00${expression}`
  const hit = RENDER_CACHE.get(key)
  if (hit !== undefined) return hit

  const html = measureActivePerfSync('mathCache.renderKatex.miss', () => {
    try {
      return katex.renderToString(expression, {
        throwOnError: false,
        displayMode: display,
        output: 'htmlAndMathml',
      })
    } catch {
      return escapeHtml(expression)
    }
  }, {
    display,
    chars: expression.length,
  })
  recordActivePerfPoint('mathCache.miss', {
    display,
    chars: expression.length,
  })

  if (RENDER_CACHE.size >= RENDER_CACHE_MAX) {
    const firstKey = RENDER_CACHE.keys().next().value
    if (firstKey !== undefined) RENDER_CACHE.delete(firstKey)
  }
  RENDER_CACHE.set(key, html)
  return html
}

/**
 * Pull every math expression out of `text` and populate the cache with
 * its rendered HTML. Mirrors the regex in MathText.vue so we warm
 * exactly what the renderer will request.
 */
const SEGMENT_PATTERN = /\\\[((?:.|\n)*?)\\\]|\\\(((?:.|\n)*?)\\\)|\$\$((?:.|\n)*?)\$\$|\$([^$\n]+?)\$/g

export function prewarmMathText(text: string | null | undefined): void {
  if (!text) return

  let match: RegExpExecArray | null
  SEGMENT_PATTERN.lastIndex = 0
  while ((match = SEGMENT_PATTERN.exec(text)) !== null) {
    const content = match[1] ?? match[2] ?? match[3] ?? match[4] ?? ''
    const display = match[1] !== undefined || match[3] !== undefined
    if (content) renderKatex(content, display)
  }
}

/**
 * Walk a list of text strings and prewarm every math segment in them,
 * spread across idle callbacks so the main thread stays responsive.
 * Works in any browser (falls back to setTimeout where idle isn't
 * available).
 */
export function prewarmMathTexts(texts: Array<string | null | undefined>): void {
  if (texts.length === 0) return

  const scheduler: (cb: () => void) => void =
    typeof (globalThis as any).requestIdleCallback === 'function'
      ? (cb) => (globalThis as any).requestIdleCallback(cb, { timeout: 500 })
      : (cb) => setTimeout(cb, 0)

  const CHUNK = 4
  let i = 0
  const step = () => {
    const end = Math.min(i + CHUNK, texts.length)
    for (; i < end; i++) prewarmMathText(texts[i])
    if (i < texts.length) scheduler(step)
  }
  scheduler(step)
}
