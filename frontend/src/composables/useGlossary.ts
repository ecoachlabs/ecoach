import { ref } from 'vue'
import { searchGlossary, getLibraryHome } from '@/ipc/library'

export function useGlossary() {
  const loading = ref(false)
  const error = ref('')
  const searchResults = ref<any[]>([])
  const currentEntry = ref<any>(null)

  async function search(query: string) {
    if (!query.trim()) { searchResults.value = []; return }
    loading.value = true
    try {
      searchResults.value = await searchGlossary(query)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Search failed'
      searchResults.value = []
    } finally {
      loading.value = false
    }
  }

  function clearSearch() {
    searchResults.value = []
  }

  return { loading, error, searchResults, currentEntry, search, clearSearch }
}
