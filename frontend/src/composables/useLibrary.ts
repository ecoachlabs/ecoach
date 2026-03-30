import { ref } from 'vue'
import { getLibraryHome, saveLibraryItem } from '@/ipc/library'

export function useLibrary(studentId: number) {
  const loading = ref(false)
  const error = ref('')
  const home = ref<any>(null)

  async function loadHome() {
    loading.value = true
    try {
      home.value = await getLibraryHome(studentId)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load library'
    } finally {
      loading.value = false
    }
  }

  async function saveItem(input: any) {
    try {
      return await saveLibraryItem(input)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to save item'
      return null
    }
  }

  return { loading, error, home, loadHome, saveItem }
}
