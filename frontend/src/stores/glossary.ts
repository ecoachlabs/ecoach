import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useGlossaryStore = defineStore('glossary', () => {
  const searchQuery = ref('')
  const searchResults = ref<any[]>([])
  const currentEntry = ref<any>(null)
  const activeDepth = ref('simple')
  const audioPlaying = ref(false)
  const audioTitle = ref('')
  const inlinePanelOpen = ref(false)
  const inlinePanelEntry = ref<any>(null)

  function setSearch(query: string, results: any[]) {
    searchQuery.value = query
    searchResults.value = results
  }

  function setEntry(entry: any) { currentEntry.value = entry }
  function setDepth(depth: string) { activeDepth.value = depth }

  function openInlinePanel(entry: any) {
    inlinePanelEntry.value = entry
    inlinePanelOpen.value = true
  }

  function closeInlinePanel() {
    inlinePanelOpen.value = false
    inlinePanelEntry.value = null
  }

  function setAudio(playing: boolean, title?: string) {
    audioPlaying.value = playing
    if (title) audioTitle.value = title
  }

  function clear() {
    searchQuery.value = ''
    searchResults.value = []
    currentEntry.value = null
  }

  return {
    searchQuery, searchResults, currentEntry, activeDepth, audioPlaying, audioTitle,
    inlinePanelOpen, inlinePanelEntry,
    setSearch, setEntry, setDepth, openInlinePanel, closeInlinePanel, setAudio, clear,
  }
})
