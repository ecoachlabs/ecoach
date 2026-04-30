import { ref } from 'vue'
import {
  getLibrarySnapshot,
  saveLibraryItemWithMetadata,
  type LibraryHomeSnapshotDto,
  type SaveLibraryItemInputDto,
} from '@/ipc/library'

export function useLibrary(studentId: number) {
  const loading = ref(false)
  const error = ref('')
  const home = ref<LibraryHomeSnapshotDto | null>(null)

  async function loadHome() {
    loading.value = true
    try {
      home.value = await getLibrarySnapshot(studentId)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load library'
    } finally {
      loading.value = false
    }
  }

  async function saveItem(input: SaveLibraryItemInputDto) {
    try {
      return await saveLibraryItemWithMetadata(studentId, input)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to save item'
      return null
    }
  }

  return { loading, error, home, loadHome, saveItem }
}
