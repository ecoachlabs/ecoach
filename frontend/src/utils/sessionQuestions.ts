import { listSessionQuestions, type SessionQuestionDto } from '@/ipc/questions'
import { startPracticeSession } from '@/ipc/sessions'
import type { PracticeSessionStartInput } from '@/types'

export interface PracticeSessionQuestionsResult {
  sessionId: number
  questions: SessionQuestionDto[]
}

export function extractRenderableSessionQuestions(
  questions: SessionQuestionDto[],
): SessionQuestionDto[] {
  return questions.filter(question =>
    !question.is_answered
    && typeof question.stem === 'string'
    && question.stem.trim().length > 0
    && Array.isArray(question.options)
    && question.options.length > 0,
  )
}

export async function startPracticeSessionWithQuestions(
  input: PracticeSessionStartInput,
  fallbackTopicSets: number[][] = [],
): Promise<PracticeSessionQuestionsResult> {
  const topicSets = [input.topic_ids, ...fallbackTopicSets]
  let attemptedLaunch = false

  for (const rawTopicIds of topicSets) {
    const topicIds = Array.from(new Set(rawTopicIds)).filter(id => Number.isFinite(id))
    if (topicIds.length === 0) continue

    attemptedLaunch = true
    const session = await startPracticeSession({
      ...input,
      topic_ids: topicIds,
    })
    const questions = extractRenderableSessionQuestions(
      await listSessionQuestions(session.session_id),
    )

    if (questions.length > 0) {
      return {
        sessionId: session.session_id,
        questions,
      }
    }
  }

  if (!attemptedLaunch) {
    throw new Error('No topics are available for practice yet.')
  }

  throw new Error('No questions are available for this selection yet.')
}
