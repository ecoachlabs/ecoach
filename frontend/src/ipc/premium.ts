import { ipc } from '.'

export function getRiskDashboard(studentId: number): Promise<any> {
  return ipc('get_risk_dashboard', { studentId })
}

export function autoDetectRisks(studentId: number): Promise<any> {
  return ipc('auto_detect_risks', { studentId })
}

export function createIntervention(input: any): Promise<any> {
  return ipc('create_intervention', { input })
}

export function resolveRiskFlag(flagId: number, resolution: any): Promise<any> {
  return ipc('resolve_risk_flag', { flagId, resolution })
}

export function resolveIntervention(interventionId: number, resolution: any): Promise<any> {
  return ipc('resolve_intervention', { interventionId, resolution })
}

export function checkEntitlement(studentId: number): Promise<any> {
  return ipc('check_entitlement', { studentId })
}

export function isFeatureEnabled(featureName: string, studentId: number): Promise<boolean> {
  return ipc('is_feature_enabled', { featureName, studentId })
}
