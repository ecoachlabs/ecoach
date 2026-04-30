import { invoke } from '@tauri-apps/api/core'
import { getCommandPolicy, isNetworkLikeError, isOnlineRequired } from './offlinePolicy'
import type { CommandPolicy } from './offlinePolicy'
import { measurePerfAsync, perfEnabled } from '@/utils/perfTrace'
import {
  enqueueOfflineCall,
  flushOfflineQueue,
  offlineQueueCount,
  type QueuedIpcCall,
} from './offlineQueue'

let autoFlushStarted = false

export class OfflineCommandQueuedError extends Error {
  readonly code = 'offline_queued'
  readonly queued = true
  readonly command: string
  readonly label: string

  constructor(command: string, label: string) {
    super(`${label} was queued and will run when internet is available.`)
    this.name = 'OfflineCommandQueuedError'
    this.command = command
    this.label = label
  }
}

export function isOfflineCommandQueuedError(error: unknown): error is OfflineCommandQueuedError {
  return error instanceof OfflineCommandQueuedError ||
    (
      typeof error === 'object' &&
      error !== null &&
      (error as { code?: unknown }).code === 'offline_queued' &&
      (error as { queued?: unknown }).queued === true
    )
}

/**
 * Typed Tauri IPC invoke wrapper.
 * All frontend-to-backend communication goes through this.
 */
export async function ipc<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const policy = getCommandPolicy(command, args)

  if (isOnlineRequired(policy) && !browserIsOnline()) {
    queueCommand(command, args, policy)
    throw new OfflineCommandQueuedError(command, policy.label)
  }

  try {
    const invokeCall = () => invoke<T>(command, args)
    if (!perfEnabled) {
      return await invokeCall()
    }
    return await measurePerfAsync(`ipc.${command}`, invokeCall, {
      argKeys: Object.keys(args ?? {}),
    })
  } catch (error) {
    if (shouldQueueAfterFailure(policy, error)) {
      queueCommand(command, args, policy)
      throw new OfflineCommandQueuedError(command, policy.label)
    }

    console.error(`[IPC] ${command} failed:`, error)
    throw error
  }
}

export async function flushQueuedIpcCalls(): Promise<{ flushed: number; remaining: number }> {
  if (!browserIsOnline()) {
    return { flushed: 0, remaining: offlineQueueCount() }
  }

  return flushOfflineQueue(
    (call: QueuedIpcCall) => invoke(call.command, call.args),
    () => browserIsOnline(),
  )
}

export function startOfflineQueueAutoFlush() {
  if (autoFlushStarted || typeof window === 'undefined') return

  autoFlushStarted = true
  notifyQueueCount()

  const flushWhenOnline = () => {
    if (browserIsOnline()) {
      void flushQueuedIpcCalls().then((result) => {
        window.dispatchEvent(new CustomEvent('ecoach:offline-queue-flushed', {
          detail: result,
        }))
      })
    }
  }

  window.addEventListener('online', flushWhenOnline)
  window.addEventListener('ecoach:connectivity-restored', flushWhenOnline)
}

function queueCommand(
  command: string,
  args: Record<string, unknown> | undefined,
  policy: Pick<CommandPolicy, 'label'>,
) {
  enqueueOfflineCall(command, args, policy)
}

function shouldQueueAfterFailure(policy: CommandPolicy, error: unknown): boolean {
  return policy.retryWhenOnline && policy.mode !== 'offline-native' && (!browserIsOnline() || isNetworkLikeError(error))
}

function browserIsOnline(): boolean {
  return typeof navigator === 'undefined' ? true : navigator.onLine
}

function notifyQueueCount() {
  if (typeof window === 'undefined') return
  window.dispatchEvent(new CustomEvent('ecoach:offline-queue-changed', {
    detail: { count: offlineQueueCount() },
  }))
}
