import { ipc } from '.'
import type { SessionQuestionDto } from './questions'

export interface CompileMockInput {
  student_id: number
  subject_id: number
  duration_minutes: number
  question_count: number
  topic_ids: number[]
  paper_year: string | null
  mock_type: string | null
  blueprint_id: number | null
}

export interface MockSessionDto {
  id: number
  subject_id: number
  mock_type: string
  status: string
  duration_minutes: number
  question_count: number
  answered_count: number
  time_remaining_seconds: number | null
  paper_year: string | null
  blueprint_id: number | null
}

export interface MockAnswerResultDto {
  question_id: number
  was_correct: boolean
  answered_count: number
  remaining_count: number
  time_remaining_seconds: number | null
}

export interface MockReportDto {
  mock_session_id: number
  grade: string
  percentage: number
  accuracy_bp: number
  total_score: number
  max_score: number
  time_used_seconds: number
  questions_answered: number
  questions_unanswered: number
  topic_count: number
  improvement_direction: string | null
}

export interface MockSessionSummaryDto {
  id: number
  subject_id: number
  mock_type: string
  grade: string | null
  percentage: number | null
  status: string
  paper_year: string | null
}

export interface SubmitMockAnswerInput {
  mock_session_id: number
  question_id: number
  selected_option_id: number
  confidence_level: string | null
}

export interface MockCentreSnapshotDto {
  student_id: number
  subject_id: number
  available_mock_types: string[]
  total_mocks_completed: number
  latest_grade: string | null
  latest_percentage: number | null
  has_forecast_blueprint: boolean
  recommended_mock_type: string | null
}

export function compileMock(input: CompileMockInput): Promise<MockSessionDto> {
  return ipc<MockSessionDto>('compile_mock', { input })
}

export function startMock(mockSessionId: number): Promise<MockSessionDto> {
  return ipc<MockSessionDto>('start_mock', { mockSessionId })
}

export function submitMockAnswer(input: SubmitMockAnswerInput): Promise<MockAnswerResultDto> {
  return ipc<MockAnswerResultDto>('submit_mock_answer', { input })
}

export function getMockReport(mockSessionId: number): Promise<MockReportDto> {
  return ipc<MockReportDto>('get_mock_report', { mockSessionId })
}

export function pauseMock(mockSessionId: number): Promise<MockSessionDto> {
  return ipc<MockSessionDto>('pause_mock', { mockSessionId })
}

export function resumeMock(mockSessionId: number): Promise<MockSessionDto> {
  return ipc<MockSessionDto>('resume_mock', { mockSessionId })
}

export function listMockSessions(studentId: number, limit: number = 20): Promise<MockSessionSummaryDto[]> {
  return ipc<MockSessionSummaryDto[]>('list_mock_sessions', { studentId, limit })
}

export function abandonMock(mockSessionId: number): Promise<void> {
  return ipc<void>('abandon_mock', { mockSessionId })
}

export function startFirstMock(studentId: number, subjectId: number): Promise<MockSessionDto> {
  return ipc<MockSessionDto>('start_first_mock', { studentId, subjectId })
}

export function flagMockQuestion(mockSessionId: number, questionId: number): Promise<void> {
  return ipc<void>('flag_mock_question', { mockSessionId, questionId })
}

export function getMockCentreSnapshot(studentId: number, subjectId: number): Promise<MockCentreSnapshotDto> {
  return ipc<MockCentreSnapshotDto>('get_mock_centre_snapshot', { studentId, subjectId })
}

export function listMockQuestions(mockSessionId: number): Promise<SessionQuestionDto[]> {
  return ipc<SessionQuestionDto[]>('list_mock_questions', { mockSessionId })
}
