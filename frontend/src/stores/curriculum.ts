import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SubjectDto, TopicDto } from '@/ipc/coach'

export const useCurriculumStore = defineStore('curriculum', () => {
  const subjects = ref<SubjectDto[]>([])
  const topicsBySubject = ref<Record<number, TopicDto[]>>({})
  const selectedSubjectId = ref<number | null>(null)

  const selectedSubject = computed(() =>
    subjects.value.find(s => s.id === selectedSubjectId.value) ?? null
  )
  const selectedTopics = computed(() =>
    selectedSubjectId.value ? (topicsBySubject.value[selectedSubjectId.value] ?? []) : []
  )

  function setSubjects(subs: SubjectDto[]) {
    subjects.value = subs
    if (subs.length && !selectedSubjectId.value) {
      selectedSubjectId.value = subs[0].id
    }
  }

  function setTopics(subjectId: number, topics: TopicDto[]) {
    topicsBySubject.value[subjectId] = topics
  }

  function selectSubject(id: number) {
    selectedSubjectId.value = id
  }

  return {
    subjects, topicsBySubject, selectedSubjectId,
    selectedSubject, selectedTopics,
    setSubjects, setTopics, selectSubject,
  }
})
