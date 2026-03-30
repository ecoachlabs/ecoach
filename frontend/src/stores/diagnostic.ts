import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface DiagnosticPhase {
  id: number
  code: string
  title: string
  status: string
  questionCount: number
  timeLimitSeconds: number | null
}

export const useDiagnosticStore = defineStore('diagnostic', () => {
  const diagnosticId = ref<number | null>(null)
  const phases = ref<DiagnosticPhase[]>([])
  const currentPhaseIndex = ref(0)
  const status = ref<'idle' | 'active' | 'complete'>('idle')

  const currentPhase = computed(() => phases.value[currentPhaseIndex.value] ?? null)
  const isActive = computed(() => status.value === 'active')
  const isComplete = computed(() => status.value === 'complete')
  const totalPhases = computed(() => phases.value.length)
  const phaseProgress = computed(() =>
    totalPhases.value > 0 ? ((currentPhaseIndex.value + 1) / totalPhases.value) * 100 : 0
  )

  function startDiagnostic(id: number, phaseList: DiagnosticPhase[]) {
    diagnosticId.value = id
    phases.value = phaseList
    currentPhaseIndex.value = 0
    status.value = 'active'
  }

  function nextPhase() {
    if (currentPhaseIndex.value < phases.value.length - 1) {
      currentPhaseIndex.value++
    } else {
      status.value = 'complete'
    }
  }

  function complete() {
    status.value = 'complete'
  }

  function clear() {
    diagnosticId.value = null
    phases.value = []
    currentPhaseIndex.value = 0
    status.value = 'idle'
  }

  return {
    diagnosticId, phases, currentPhaseIndex, status,
    currentPhase, isActive, isComplete, totalPhases, phaseProgress,
    startDiagnostic, nextPhase, complete, clear,
  }
})
