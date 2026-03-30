import { invoke } from '@tauri-apps/api/core'

/**
 * Typed Tauri IPC invoke wrapper.
 * All frontend-to-backend communication goes through this.
 */
export async function ipc<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    console.error(`[IPC] ${command} failed:`, error)
    throw error
  }
}
