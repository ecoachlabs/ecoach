import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type UploadStep = 'select' | 'classify' | 'align' | 'review' | 'complete'

export interface UploadedFile {
  id: string
  name: string
  size: number
  type: string
  classification?: 'question' | 'answer' | 'marked' | 'other'
}

export const useUploadStore = defineStore('upload', () => {
  const step = ref<UploadStep>('select')
  const files = ref<UploadedFile[]>([])
  const bundleId = ref<number | null>(null)
  const processing = ref(false)
  const reviewItems = ref<any[]>([])

  const fileCount = computed(() => files.value.length)
  const classifiedCount = computed(() => files.value.filter(f => f.classification).length)
  const allClassified = computed(() => files.value.length > 0 && classifiedCount.value === files.value.length)

  function addFiles(newFiles: UploadedFile[]) {
    files.value.push(...newFiles)
  }

  function removeFile(id: string) {
    files.value = files.value.filter(f => f.id !== id)
  }

  function classifyFile(id: string, classification: UploadedFile['classification']) {
    const file = files.value.find(f => f.id === id)
    if (file) file.classification = classification
  }

  function setStep(s: UploadStep) { step.value = s }
  function setBundleId(id: number) { bundleId.value = id }
  function setReviewItems(items: any[]) { reviewItems.value = items }

  function reset() {
    step.value = 'select'
    files.value = []
    bundleId.value = null
    reviewItems.value = []
    processing.value = false
  }

  return {
    step, files, bundleId, processing, reviewItems,
    fileCount, classifiedCount, allClassified,
    addFiles, removeFile, classifyFile, setStep, setBundleId, setReviewItems, reset,
  }
})
