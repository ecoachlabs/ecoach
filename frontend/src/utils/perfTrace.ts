type PerfEvent = {
  kind: 'point' | 'measure'
  label: string
  at: number
  duration?: number
  detail?: unknown
}

type PerfTraceRecord = {
  id: string
  name: string
  startedAt: number
  endedAt?: number
  duration?: number
  meta?: unknown
  events: PerfEvent[]
}

type PerfStore = {
  traces: PerfTraceRecord[]
  activeTraceId: string | null
}

type PerfTraceHandle = {
  id: string
  name: string
  activate: () => void
  deactivate: () => void
  point: (label: string, detail?: unknown) => void
  measureSync: <T>(label: string, fn: () => T, detail?: unknown) => T
  measureAsync: <T>(label: string, fn: () => Promise<T>, detail?: unknown) => Promise<T>
  finish: (detail?: unknown) => void
}

declare global {
  interface Window {
    __ecoachPerf?: PerfStore
  }
}

const env = import.meta.env

export const perfFlags = {
  trace: env.DEV && env.VITE_PERF_TRACE === '1',
  localOnly: env.DEV && env.VITE_PERF_LOCAL_ONLY === '1',
  noPersist: env.DEV && env.VITE_PERF_NO_PERSIST === '1',
  minimalRender: env.DEV && env.VITE_PERF_MINIMAL_RENDER === '1',
  disableBuffer: env.DEV && env.VITE_PERF_DISABLE_BUFFER === '1',
} as const

export const perfEnabled = Object.values(perfFlags).some(Boolean)

let traceSequence = 0

function ensureStore(): PerfStore {
  if (typeof window === 'undefined') {
    return { traces: [], activeTraceId: null }
  }
  if (!window.__ecoachPerf) {
    window.__ecoachPerf = { traces: [], activeTraceId: null }
  }
  return window.__ecoachPerf
}

function getTrace(traceId: string): PerfTraceRecord | null {
  const store = ensureStore()
  return store.traces.find(trace => trace.id === traceId) ?? null
}

function logPerf(name: string, label: string, duration?: number, detail?: unknown) {
  if (!perfEnabled) return
  const summary = duration === undefined
    ? `[perf][${name}] ${label}`
    : `[perf][${name}] ${label} ${duration.toFixed(1)}ms`
  if (detail === undefined) console.info(summary)
  else console.info(summary, detail)
}

function recordPoint(traceId: string, label: string, detail?: unknown) {
  if (!perfEnabled) return
  const trace = getTrace(traceId)
  if (!trace) return
  const at = performance.now()
  trace.events.push({ kind: 'point', label, at, detail })
  logPerf(trace.name, label, undefined, detail)
}

function measureSyncInternal<T>(
  traceId: string,
  label: string,
  fn: () => T,
  detail?: unknown,
): T {
  if (!perfEnabled) return fn()

  const trace = getTrace(traceId)
  if (!trace) return fn()

  const suffix = `${traceId}:${label}:${trace.events.length}`
  const startMark = `${suffix}:start`
  const endMark = `${suffix}:end`
  const measureName = `${suffix}:measure`
  performance.mark(startMark)
  try {
    return fn()
  } finally {
    performance.mark(endMark)
    performance.measure(measureName, startMark, endMark)
    const entries = performance.getEntriesByName(measureName)
    const duration = entries.length > 0 ? entries[entries.length - 1].duration : 0
    trace.events.push({
      kind: 'measure',
      label,
      at: performance.now(),
      duration,
      detail,
    })
    logPerf(trace.name, label, duration, detail)
    performance.clearMarks(startMark)
    performance.clearMarks(endMark)
    performance.clearMeasures(measureName)
  }
}

async function measureAsyncInternal<T>(
  traceId: string,
  label: string,
  fn: () => Promise<T>,
  detail?: unknown,
): Promise<T> {
  if (!perfEnabled) return await fn()

  const trace = getTrace(traceId)
  if (!trace) return await fn()

  const suffix = `${traceId}:${label}:${trace.events.length}`
  const startMark = `${suffix}:start`
  const endMark = `${suffix}:end`
  const measureName = `${suffix}:measure`
  performance.mark(startMark)
  try {
    return await fn()
  } finally {
    performance.mark(endMark)
    performance.measure(measureName, startMark, endMark)
    const entries = performance.getEntriesByName(measureName)
    const duration = entries.length > 0 ? entries[entries.length - 1].duration : 0
    trace.events.push({
      kind: 'measure',
      label,
      at: performance.now(),
      duration,
      detail,
    })
    logPerf(trace.name, label, duration, detail)
    performance.clearMarks(startMark)
    performance.clearMarks(endMark)
    performance.clearMeasures(measureName)
  }
}

export function startPerfTrace(name: string, meta?: unknown): PerfTraceHandle {
  const id = `${name}:${Date.now()}:${traceSequence += 1}`
  if (perfEnabled) {
    const trace: PerfTraceRecord = {
      id,
      name,
      startedAt: performance.now(),
      meta,
      events: [],
    }
    ensureStore().traces.push(trace)
    logPerf(name, 'trace.start', undefined, meta)
  }

  return {
    id,
    name,
    activate() {
      if (!perfEnabled) return
      ensureStore().activeTraceId = id
    },
    deactivate() {
      if (!perfEnabled) return
      const store = ensureStore()
      if (store.activeTraceId === id) {
        store.activeTraceId = null
      }
    },
    point(label: string, detail?: unknown) {
      recordPoint(id, label, detail)
    },
    measureSync<T>(label: string, fn: () => T, detail?: unknown): T {
      return measureSyncInternal(id, label, fn, detail)
    },
    measureAsync<T>(label: string, fn: () => Promise<T>, detail?: unknown): Promise<T> {
      return measureAsyncInternal(id, label, fn, detail)
    },
    finish(detail?: unknown) {
      if (!perfEnabled) return
      const trace = getTrace(id)
      if (!trace) return
      trace.endedAt = performance.now()
      trace.duration = trace.endedAt - trace.startedAt
      logPerf(name, 'trace.finish', trace.duration, detail)
    },
  }
}

export function recordActivePerfPoint(label: string, detail?: unknown) {
  if (!perfEnabled) return
  const activeTraceId = ensureStore().activeTraceId
  if (!activeTraceId) return
  recordPoint(activeTraceId, label, detail)
}

export function measureActivePerfSync<T>(label: string, fn: () => T, detail?: unknown): T {
  if (!perfEnabled) return fn()
  const activeTraceId = ensureStore().activeTraceId
  if (!activeTraceId) return fn()
  return measureSyncInternal(activeTraceId, label, fn, detail)
}

export async function measurePerfAsync<T>(
  label: string,
  fn: () => Promise<T>,
  detail?: unknown,
): Promise<T> {
  if (!perfEnabled) return await fn()
  const trace = startPerfTrace(label, detail)
  try {
    return await trace.measureAsync('total', fn, detail)
  } finally {
    trace.finish()
  }
}
