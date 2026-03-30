import { ipc } from '.'

export function launchDiagnostic(studentId: number, subjectId: number, mode: string): Promise<any> {
  return ipc('launch_diagnostic', { studentId, subjectId, mode })
}

export function getDiagnosticReport(diagnosticId: number): Promise<any> {
  return ipc('get_diagnostic_report', { diagnosticId })
}
