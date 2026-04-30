import { computed, ref } from 'vue'
import {
  listSubjects,
  listTopics,
  type SubjectDto,
  type TopicCaseDto,
  type TopicDto,
} from '@/ipc/coach'
import {
  startPracticeSession,
  completeSession,
  completeSessionWithPipeline,
  recordSessionPresenceEvent,
} from '@/ipc/sessions'
import {
  listSessionQuestions,
  submitAttempt,
  type AttemptResultDto,
  type QuestionOptionDto,
  type SessionQuestionDto,
  type SubmitAttemptInput,
} from '@/ipc/questions'
import {
  buildLearnerTopicIndex,
  type LearnerTopic,
} from '@/utils/learnerTopics'

const OPTION_LETTERS = ['A', 'B', 'C', 'D', 'E', 'F']

export interface HomepageArenaOption {
  id: string
  optionId: number
  letter: string
  label: string
  text: string
}

export interface HomepageArenaQuestion {
  key: string
  topic: string
  prompt: string
  options: HomepageArenaOption[]
  estimatedTimeSeconds: number | null
}

interface HomepageArenaConfig {
  getStudentId: () => number | null | undefined
  getTopicCases: () => TopicCaseDto[]
  onRecorded?: () => Promise<void> | void
  questionCount?: number
  isTimed?: boolean
}

function normalizeText(value: string | null | undefined): string {
  return (value ?? '').trim().replace(/\s+/g, ' ').toLowerCase()
}

function nowMs(): number {
  return typeof performance === 'undefined' ? Date.now() : performance.now()
}

function chooseSubject(subjects: SubjectDto[], topicCases: TopicCaseDto[]): SubjectDto | null {
  const prioritySubject = topicCases.find(topic => topic.subject_code)?.subject_code
  if (prioritySubject) {
    const match = subjects.find(subject => subject.code === prioritySubject)
    if (match) return match
  }
  return subjects[0] ?? null
}

function chooseLearnerTopics(
  topics: TopicDto[],
  topicCases: TopicCaseDto[],
  subject: SubjectDto,
): LearnerTopic[] {
  const learnerIndex = buildLearnerTopicIndex(topics)
  const seen = new Set<number>()
  const priorityTopics: LearnerTopic[] = []

  for (const topicCase of topicCases) {
    if (topicCase.subject_code !== subject.code) continue
    const learnerTopic = learnerIndex.bySourceTopicId.get(topicCase.topic_id)
    if (!learnerTopic || seen.has(learnerTopic.id)) continue
    seen.add(learnerTopic.id)
    priorityTopics.push(learnerTopic)
  }

  const chosen = priorityTopics.length > 0
    ? priorityTopics
    : learnerIndex.topics.slice(0, 4)

  return chosen.slice(0, 4)
}

function topicLabel(subject: SubjectDto, topics: LearnerTopic[]): string {
  if (topics.length === 1) {
    return topics[0]?.name ?? subject.name
  }
  return subject.name
}

function toArenaQuestion(question: SessionQuestionDto, topic: string): HomepageArenaQuestion {
  return {
    key: `${question.item_id}:${question.question_id}`,
    topic,
    prompt: question.stem,
    estimatedTimeSeconds: question.estimated_time_seconds,
    options: question.options.map((option, index) => ({
      id: String(option.id),
      optionId: option.id,
      letter: option.label || OPTION_LETTERS[index] || String(index + 1),
      label: option.text,
      text: option.text,
    })),
  }
}

export function useHomepageArena(config: HomepageArenaConfig) {
  const sessionId = ref<number | null>(null)
  const rawQuestions = ref<SessionQuestionDto[]>([])
  const questionIndex = ref(0)
  const selectedOptionKey = ref<string | null>(null)
  const attemptResult = ref<AttemptResultDto | null>(null)
  const loading = ref(false)
  const submitting = ref(false)
  const error = ref('')
  const currentTopicLabel = ref('Real Question')
  const questionStartedAt = ref<number>(nowMs())
  const presentedQuestionKeys = new Set<string>()
  const pendingSubmissionTasks = new Map<number, Promise<AttemptResultDto>>()
  const unsavedSubmissionInputs = new Map<number, SubmitAttemptInput>()

  const questions = computed(() =>
    rawQuestions.value.map(question => toArenaQuestion(question, currentTopicLabel.value)),
  )
  const currentRawQuestion = computed(() => rawQuestions.value[questionIndex.value] ?? null)
  const currentQuestion = computed(() => questions.value[questionIndex.value] ?? null)
  const questionPips = computed(() => questions.value.map(question => question.key))
  const selectedOptionId = computed(() => selectedOptionKey.value)
  const locked = computed(() => selectedOptionKey.value !== null || submitting.value)
  const isCorrect = computed(() => attemptResult.value?.is_correct === true)
  const selectedRawOption = computed(() => {
    const question = currentRawQuestion.value
    if (!question || !selectedOptionKey.value || selectedOptionKey.value === '__timeout__') return null
    return question.options.find(option => String(option.id) === selectedOptionKey.value) ?? null
  })
  const correctRawOption = computed(() => {
    const question = currentRawQuestion.value
    return question ? findCorrectOption(question) : null
  })
  const feedbackText = computed(() => {
    const result = attemptResult.value
    if (!result) return ''
    return result.explanation
      || result.diagnosis_summary
      || result.recommended_action
      || result.correct_option_text
      || ''
  })
  const feedbackPayload = computed(() => {
    const question = currentRawQuestion.value
    const result = attemptResult.value
    if (!question || !result) return null

    return {
      isCorrect: result.is_correct,
      questionStem: question.stem,
      explanation: result.explanation ?? question.explanation_text ?? null,
      errorType: result.error_type ?? null,
      diagnosisSummary: result.diagnosis_summary ?? null,
      recommendedAction: result.recommended_action ?? null,
      selectedOptionText: result.selected_option_text
        ?? selectedRawOption.value?.text
        ?? (selectedOptionKey.value === '__timeout__' ? 'No answer selected.' : ''),
      correctOptionText: result.correct_option_text ?? correctRawOption.value?.text ?? null,
      misconceptionInfo: result.misconception_info
        ?? (selectedRawOption.value?.is_correct ? null : selectedRawOption.value?.distractor_intent ?? null),
      selectedOptionId: selectedRawOption.value?.id ?? null,
      correctOptionId: correctRawOption.value?.id ?? null,
      options: question.options,
      loadingExplanation: false,
    }
  })
  const studyRoute = computed(() => attemptResult.value?.next_action_route || '/student/knowledge-gap')
  const weaknessTitle = computed(() => {
    const weakness = config.getTopicCases()[0]
    return weakness?.topic_name || currentQuestion.value?.topic || 'this skill'
  })
  const weaknessDetail = computed(() => {
    const result = attemptResult.value
    return result?.misconception_info
      || result?.diagnosis_summary
      || result?.recommended_action
      || 'Focused practice is ready.'
  })

  function resetQuestionState() {
    selectedOptionKey.value = null
    attemptResult.value = null
    questionStartedAt.value = nowMs()
  }

  function findCorrectOption(question: SessionQuestionDto): QuestionOptionDto | null {
    return question.options.find(option => option.is_correct) ?? null
  }

  function markQuestionAnswered(question: SessionQuestionDto) {
    rawQuestions.value = rawQuestions.value.map(candidate => (
      candidate.item_id === question.item_id
        ? { ...candidate, is_answered: true }
        : candidate
    ))
  }

  function countAnsweredQuestions(includingItemId?: number): number {
    return rawQuestions.value.reduce((count, question) => (
      count + (question.is_answered || question.item_id === includingItemId ? 1 : 0)
    ), 0)
  }

  function buildOptimisticResult(
    question: SessionQuestionDto,
    selectedOption: QuestionOptionDto | null,
    timedOut = false,
  ): AttemptResultDto {
    const correctOption = findCorrectOption(question)
    const sessionAnswered = countAnsweredQuestions(question.item_id)
    const sessionRemaining = Math.max(rawQuestions.value.length - sessionAnswered, 0)
    const isCorrectAnswer = !timedOut && selectedOption?.is_correct === true

    return {
      attempt_id: 0,
      is_correct: isCorrectAnswer,
      explanation: question.explanation_text,
      correct_option_text: correctOption?.text ?? null,
      selected_option_text: timedOut ? null : selectedOption?.text ?? null,
      misconception_info: timedOut
        ? 'Timed out before choosing an answer.'
        : (selectedOption?.is_correct ? null : selectedOption?.distractor_intent ?? null),
      error_type: timedOut ? 'timed_out' : null,
      diagnosis_summary: timedOut ? 'Time ran out before an answer was submitted.' : null,
      recommended_action: isCorrectAnswer
        ? 'Keep the same method and move quickly to the next item.'
        : 'Review the breakdown and compare each option before locking an answer.',
      updated_mastery: 0,
      updated_gap: 0,
      session_answered: sessionAnswered,
      session_remaining: sessionRemaining,
      session_complete: sessionRemaining === 0,
      next_action_type: isCorrectAnswer ? 'continue' : 'review_topic',
      next_action_title: isCorrectAnswer ? 'Continue' : 'Study topic',
      next_action_route: '/student/knowledge-gap',
    }
  }

  function queueSubmission(
    question: SessionQuestionDto,
    input: SubmitAttemptInput,
    selectedKey: string,
  ) {
    unsavedSubmissionInputs.set(question.item_id, input)
    const submission = submitAttempt(input)
    pendingSubmissionTasks.set(question.item_id, submission)

    void submission
      .then(async result => {
        unsavedSubmissionInputs.delete(question.item_id)
        if (
          currentRawQuestion.value?.item_id === question.item_id
          && selectedOptionKey.value === selectedKey
        ) {
          attemptResult.value = result
        }
        await config.onRecorded?.()
      })
      .catch(() => {
        // Retry before recycling the arena.
      })
      .finally(() => {
        pendingSubmissionTasks.delete(question.item_id)
      })
  }

  async function flushPendingSubmissions() {
    const inFlight = Array.from(pendingSubmissionTasks.values())
    if (inFlight.length > 0) {
      await Promise.allSettled(inFlight)
    }

    for (const [itemId, input] of Array.from(unsavedSubmissionInputs.entries())) {
      try {
        await submitAttempt(input)
        unsavedSubmissionInputs.delete(itemId)
        await config.onRecorded?.()
      } catch {
        // Keep the optimistic UI if a retry still fails.
      }
    }
  }

  async function markQuestionPresented(question: SessionQuestionDto | null | undefined) {
    const activeSessionId = sessionId.value
    if (!activeSessionId || !question) return

    const key = `${activeSessionId}:${question.item_id}`
    if (presentedQuestionKeys.has(key)) return
    presentedQuestionKeys.add(key)

    try {
      await recordSessionPresenceEvent(activeSessionId, {
        event_type: 'question_presented',
        occurred_at: new Date().toISOString(),
        metadata_json: {
          item_id: question.item_id,
          question_id: question.question_id,
          source: 'homepage_arena',
        },
      })
    } catch {
      presentedQuestionKeys.delete(key)
    }
  }

  async function loadArena() {
    const studentId = config.getStudentId()
    if (!studentId) return

    loading.value = true
    error.value = ''
    try {
      const subjects = await listSubjects(1)
      const subject = chooseSubject(subjects, config.getTopicCases())
      if (!subject) throw new Error('No subjects are available yet.')

      const topics = await listTopics(subject.id)
      const chosenTopics = chooseLearnerTopics(topics, config.getTopicCases(), subject)
      const topicIds = Array.from(new Set(chosenTopics.flatMap(topic => topic.sourceTopicIds)))
      if (topicIds.length === 0) throw new Error('No topics are available for practice yet.')

      currentTopicLabel.value = topicLabel(subject, chosenTopics)
      const session = await startPracticeSession({
        student_id: studentId,
        subject_id: subject.id,
        topic_ids: topicIds,
        question_count: config.questionCount ?? 4,
        is_timed: config.isTimed ?? true,
      })

      sessionId.value = session.session_id
      presentedQuestionKeys.clear()
      pendingSubmissionTasks.clear()
      unsavedSubmissionInputs.clear()
      rawQuestions.value = (await listSessionQuestions(session.session_id))
        .filter(question => !question.is_answered && question.options.length > 0)
      questionIndex.value = 0
      resetQuestionState()

      if (rawQuestions.value.length === 0) {
        throw new Error('No real questions are available for this topic yet.')
      }
      await markQuestionPresented(rawQuestions.value[0])
    } catch (err: any) {
      error.value = typeof err === 'string' ? err : err?.message ?? 'Failed to load real questions.'
      rawQuestions.value = []
      sessionId.value = null
      resetQuestionState()
    } finally {
      loading.value = false
    }
  }

  async function submitOption(optionKey: string): Promise<AttemptResultDto | null> {
    const studentId = config.getStudentId()
    const activeSessionId = sessionId.value
    const question = currentRawQuestion.value
    const option = question?.options.find(candidate => String(candidate.id) === optionKey)
    if (!studentId || !activeSessionId || !question || !option || locked.value) return null

    selectedOptionKey.value = optionKey
    submitting.value = true
    error.value = ''
    const input: SubmitAttemptInput = {
      student_id: studentId,
      session_id: activeSessionId,
      session_item_id: question.item_id,
      question_id: question.question_id,
      selected_option_id: option.id,
      response_time_ms: Math.max(0, Math.round(nowMs() - questionStartedAt.value)),
      confidence_level: null,
      hint_count: 0,
      changed_answer_count: 0,
      was_timed: config.isTimed ?? true,
    }
    markQuestionAnswered(question)
    const optimisticResult = buildOptimisticResult(question, option)
    attemptResult.value = optimisticResult
    queueSubmission(question, input, optionKey)
    submitting.value = false
    return optimisticResult
  }

  async function markTimedOut(): Promise<AttemptResultDto | null> {
    const studentId = config.getStudentId()
    const activeSessionId = sessionId.value
    const question = currentRawQuestion.value
    if (!studentId || !activeSessionId || !question || locked.value) return null

    selectedOptionKey.value = '__timeout__'
    submitting.value = true
    error.value = ''
    const input: SubmitAttemptInput = {
      student_id: studentId,
      session_id: activeSessionId,
      session_item_id: question.item_id,
      question_id: question.question_id,
      selected_option_id: null,
      response_time_ms: Math.max(0, Math.round(nowMs() - questionStartedAt.value)),
      confidence_level: null,
      hint_count: 0,
      changed_answer_count: 0,
      skipped: false,
      timed_out: true,
      was_timed: config.isTimed ?? true,
    }
    markQuestionAnswered(question)
    const optimisticResult = buildOptimisticResult(question, null, true)
    attemptResult.value = optimisticResult
    queueSubmission(question, input, '__timeout__')
    submitting.value = false
    return optimisticResult
  }

  async function nextQuestion() {
    const atEnd = questionIndex.value >= rawQuestions.value.length - 1
    const activeSessionId = sessionId.value
    if (atEnd) {
      await flushPendingSubmissions()
      if (activeSessionId) {
        try {
          const studentId = config.getStudentId()
          if (studentId) {
            await completeSessionWithPipeline(studentId, activeSessionId)
          } else {
            await completeSession(activeSessionId)
          }
        } catch {
          // Completion is best-effort; the attempt has already been recorded.
        }
      }
      await loadArena()
      return
    }

    questionIndex.value += 1
    resetQuestionState()
    await markQuestionPresented(rawQuestions.value[questionIndex.value])
  }

  function isCorrectOption(option: HomepageArenaOption): boolean {
    const correctText = attemptResult.value?.correct_option_text
    return !!attemptResult.value && normalizeText(option.text) === normalizeText(correctText)
  }

  function isWrongOption(option: HomepageArenaOption): boolean {
    return !!attemptResult.value
      && selectedOptionKey.value === option.id
      && !attemptResult.value.is_correct
  }

  return {
    sessionId,
    questions,
    questionPips,
    questionIndex,
    currentQuestion,
    currentRawQuestion,
    selectedOptionId,
    attemptResult,
    loading,
    submitting,
    error,
    locked,
    isCorrect,
    feedbackText,
    feedbackPayload,
    studyRoute,
    weaknessTitle,
    weaknessDetail,
    loadArena,
    submitOption,
    markTimedOut,
    nextQuestion,
    isCorrectOption,
    isWrongOption,
  }
}
