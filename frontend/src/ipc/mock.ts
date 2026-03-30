import { ipc } from '.'

export function compileMock(input: any): Promise<any> {
  return ipc('compile_mock', { input })
}

export function startMock(blueprintId: number): Promise<any> {
  return ipc('start_mock', { blueprintId })
}

export function submitMockAnswer(input: any): Promise<any> {
  return ipc('submit_mock_answer', { input })
}

export function getMockReport(sessionId: number): Promise<any> {
  return ipc('get_mock_report', { sessionId })
}

export function pauseMock(sessionId: number): Promise<void> {
  return ipc('pause_mock', { sessionId })
}

export function resumeMock(sessionId: number): Promise<void> {
  return ipc('resume_mock', { sessionId })
}

export function listMockSessions(studentId: number): Promise<any[]> {
  return ipc('list_mock_sessions', { studentId })
}

export function abandonMock(sessionId: number): Promise<void> {
  return ipc('abandon_mock', { sessionId })
}
