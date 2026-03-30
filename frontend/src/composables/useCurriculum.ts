import { ref } from 'vue'
import { listSubjects, listTopics, type SubjectDto, type TopicDto } from '@/ipc/coach'

export function useCurriculum() {
  const subjects = ref<SubjectDto[]>([])
  const topics = ref<TopicDto[]>([])
  const loading = ref(false)
  const error = ref('')

  async function loadSubjects(curriculumVersionId: number = 1) {
    loading.value = true
    try {
      subjects.value = await listSubjects(curriculumVersionId)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load subjects'
    } finally {
      loading.value = false
    }
  }

  async function loadTopics(subjectId: number) {
    loading.value = true
    try {
      topics.value = await listTopics(subjectId)
    } catch (e: any) {
      error.value = typeof e === 'string' ? e : e?.message ?? 'Failed to load topics'
    } finally {
      loading.value = false
    }
  }

  return {
    subjects, topics, loading, error,
    loadSubjects, loadTopics,
  }
}
