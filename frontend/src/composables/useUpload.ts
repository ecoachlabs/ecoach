import { ref } from 'vue'
import { ipc } from '@/ipc'

export type UploadStep = 'select' | 'classify' | 'align' | 'review' | 'complete'

export function useUpload() {
  const step = ref<UploadStep>('select')
  const files = ref<File[]>([])
  const bundleId = ref<number | null>(null)
  const loading = ref(false)
  const error = ref('')

  function addFiles(newFiles: File[]) {
    files.value.push(...newFiles)
  }

  function removeFile(index: number) {
    files.value.splice(index, 1)
  }

  async function createBundle(studentId: number) {
    loading.value = true
    error.value = ''
    try {
      const result = await ipc<any>('create_submission_bundle', {
        studentId,
        title: files.value.length === 1
          ? files.value[0].name
          : `Offline upload bundle (${files.value.length} files)`,
      })
      bundleId.value = result?.id ?? null
      step.value = 'classify'
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to create bundle'
    } finally {
      loading.value = false
    }
  }

  function nextStep() {
    const steps: UploadStep[] = ['select', 'classify', 'align', 'review', 'complete']
    const idx = steps.indexOf(step.value)
    if (idx < steps.length - 1) step.value = steps[idx + 1]
  }

  function prevStep() {
    const steps: UploadStep[] = ['select', 'classify', 'align', 'review', 'complete']
    const idx = steps.indexOf(step.value)
    if (idx > 0) step.value = steps[idx - 1]
  }

  function reset() {
    step.value = 'select'
    files.value = []
    bundleId.value = null
    error.value = ''
  }

  return { step, files, bundleId, loading, error, addFiles, removeFile, createBundle, nextStep, prevStep, reset }
}
