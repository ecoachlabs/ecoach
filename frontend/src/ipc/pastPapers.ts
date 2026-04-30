import { ipc } from '.'
import type { SessionSnapshotDto } from '@/types'

// ── DTOs ──────────────────────────────────────────────────────────

export interface PastPaperCourseSummaryDto {
  subject_id: number
  subject_name: string
  subject_code: string
  paper_count: number
  first_year: number | null
  last_year: number | null
  total_questions: number
}

export type PastPaperSectionKind = 'objective' | 'essay' | 'mixed'

export interface PastPaperSectionDto {
  section_label: string
  section_kind: PastPaperSectionKind
  question_count: number
}

export interface PastPaperYearDto {
  paper_id: number
  exam_year: number
  title: string
  paper_code: string | null
  sections: PastPaperSectionDto[]
  topic_ids: number[]
  keywords: string[]
}

export interface PastPaperTopicCountDto {
  topic_id: number
  topic_name: string
  /** Total questions in this topic across every past paper of the subject. */
  question_count: number
  /** MCQ + true/false only. Drives the objectives-session launcher. */
  objective_count: number
  /** Everything non-objective (short-answer, fill-blank, numeric, …). */
  essay_count: number
}

/** Format filter accepted by `start_past_paper_topic_session`. The UI
 *  always picks one of these two — objective and essay sessions are
 *  kept strictly separate. "all" exists server-side for future flexibility. */
export type PastPaperTopicFormat = 'objective' | 'essay'

// ── Commands ──────────────────────────────────────────────────────

export function listPastPaperCourses(): Promise<PastPaperCourseSummaryDto[]> {
  return ipc<PastPaperCourseSummaryDto[]>('list_past_paper_courses', {})
}

export function listPastPapersForSubject(subjectId: number): Promise<PastPaperYearDto[]> {
  return ipc<PastPaperYearDto[]>('list_past_papers_for_subject', { subjectId })
}

export function listPastPaperTopicCounts(subjectId: number): Promise<PastPaperTopicCountDto[]> {
  return ipc<PastPaperTopicCountDto[]>('list_past_paper_topic_counts', { subjectId })
}

export function startPastPaperSection(
  studentId: number,
  paperId: number,
  sectionLabel: string,
  isTimed: boolean = false,
): Promise<SessionSnapshotDto> {
  return ipc<SessionSnapshotDto>('start_past_paper_section', {
    studentId,
    paperId,
    sectionLabel,
    isTimed,
  })
}

export function startPastPaperTopicSession(
  studentId: number,
  subjectId: number,
  topicId: number,
  format: PastPaperTopicFormat,
  isTimed: boolean = false,
): Promise<SessionSnapshotDto> {
  return ipc<SessionSnapshotDto>('start_past_paper_topic_session', {
    studentId,
    subjectId,
    topicId,
    format,
    isTimed,
  })
}
