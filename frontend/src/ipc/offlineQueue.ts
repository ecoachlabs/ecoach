import type { CommandPolicy } from './offlinePolicy'

const STORAGE_KEY = 'ecoach.offlineQueue.v1'
const QUEUE_CHANGED_EVENT = 'ecoach:offline-queue-changed'

export interface QueuedIpcCall {
  id: string
  fingerprint: string
  command: string
  args?: Record<string, unknown>
  label: string
  createdAt: string
  attempts: number
  lastError: string | null
}

type QueuedRunner = (call: QueuedIpcCall) => Promise<unknown>
type QueueGate = (call: QueuedIpcCall) => boolean

export function getOfflineQueue(): QueuedIpcCall[] {
  if (typeof localStorage === 'undefined') return []

  try {
    const rawQueue = localStorage.getItem(STORAGE_KEY)
    if (!rawQueue) return []

    const parsed = JSON.parse(rawQueue)
    return Array.isArray(parsed)
      ? parsed.map(normalizeQueuedIpcCall).filter((call): call is QueuedIpcCall => call !== null)
      : []
  } catch {
    return []
  }
}

export function enqueueOfflineCall(
  command: string,
  args: Record<string, unknown> | undefined,
  policy: Pick<CommandPolicy, 'label'>,
): QueuedIpcCall {
  const queue = getOfflineQueue()
  const fingerprint = createQueuedCallFingerprint(command, args)
  const existingCallIndex = queue.findIndex((call) => call.fingerprint === fingerprint)

  if (existingCallIndex >= 0) {
    const existingCall = queue[existingCallIndex]
    const updatedCall: QueuedIpcCall = {
      ...existingCall,
      args,
      label: policy.label,
      lastError: null,
    }
    const updatedQueue = [...queue]
    updatedQueue[existingCallIndex] = updatedCall
    setOfflineQueue(updatedQueue)
    return updatedCall
  }

  const call: QueuedIpcCall = {
    id: `${Date.now()}-${Math.random().toString(36).slice(2)}`,
    fingerprint,
    command,
    args,
    label: policy.label,
    createdAt: new Date().toISOString(),
    attempts: 0,
    lastError: null,
  }

  setOfflineQueue([...queue, call])
  return call
}

export function clearOfflineQueue() {
  setOfflineQueue([])
}

export function offlineQueueCount(): number {
  return getOfflineQueue().length
}

export async function flushOfflineQueue(
  runner: QueuedRunner,
  canRun: QueueGate = () => true,
): Promise<{ flushed: number; remaining: number }> {
  let flushed = 0
  const queue = getOfflineQueue()

  for (const call of queue) {
    if (!canRun(call)) break

    try {
      await runner(call)
      removeOfflineCall(call.id)
      flushed += 1
    } catch (error) {
      updateOfflineCall(call.id, {
        attempts: call.attempts + 1,
        lastError: error instanceof Error ? error.message : String(error),
      })
    }
  }

  return { flushed, remaining: offlineQueueCount() }
}

function setOfflineQueue(queue: QueuedIpcCall[]) {
  if (typeof localStorage === 'undefined') return

  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(queue))
    notifyQueueChanged(queue.length)
  } catch {
    notifyQueueChanged(queue.length)
  }
}

function removeOfflineCall(id: string) {
  setOfflineQueue(getOfflineQueue().filter((call) => call.id !== id))
}

function updateOfflineCall(id: string, patch: Partial<QueuedIpcCall>) {
  setOfflineQueue(
    getOfflineQueue().map((call) => (
      call.id === id ? { ...call, ...patch } : call
    )),
  )
}

function notifyQueueChanged(count: number) {
  if (typeof window === 'undefined') return
  window.dispatchEvent(new CustomEvent(QUEUE_CHANGED_EVENT, { detail: { count } }))
}

function createQueuedCallFingerprint(command: string, args?: Record<string, unknown>): string {
  return `${command}:${stableStringify(args ?? {})}`
}

function stableStringify(value: unknown): string {
  if (Array.isArray(value)) {
    return `[${value.map(stableStringify).join(',')}]`
  }

  if (value && typeof value === 'object') {
    return `{${Object.entries(value as Record<string, unknown>)
      .filter(([, item]) => item !== undefined)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, item]) => `${JSON.stringify(key)}:${stableStringify(item)}`)
      .join(',')}}`
  }

  return JSON.stringify(value)
}

function isQueuedIpcCall(value: unknown): value is QueuedIpcCall {
  const call = value as QueuedIpcCall
  return (
    typeof call?.id === 'string' &&
    typeof call.command === 'string' &&
    typeof call.label === 'string' &&
    typeof call.createdAt === 'string' &&
    typeof call.attempts === 'number'
  )
}

function normalizeQueuedIpcCall(value: unknown): QueuedIpcCall | null {
  if (!isQueuedIpcCall(value)) return null

  const call = value as QueuedIpcCall
  return {
    ...call,
    fingerprint: typeof call.fingerprint === 'string'
      ? call.fingerprint
      : createQueuedCallFingerprint(call.command, call.args),
  }
}
