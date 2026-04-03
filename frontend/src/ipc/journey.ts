import { ipc } from '.'

export interface JourneyRoute {
  id: number
  student_id: number
  subject_id: number
  route_type: string
  status: string
  current_station_code: string | null
  route_summary: Record<string, unknown>
}

export interface JourneyStation {
  id: number
  route_id: number
  station_code: string
  title: string
  topic_id: number | null
  sequence_no: number
  station_type: string
  target_mastery_score: number | null
  target_accuracy_score: number | null
  target_readiness_score: number | null
  status: string
  progress_score: number
  completion_confidence: number
  times_entered: number
  times_reactivated: number
}

export interface JourneyRouteSnapshot {
  route: JourneyRoute
  stations: JourneyStation[]
}

export function buildOrRefreshJourneyRoute(
  studentId: number,
  subjectId: number,
  targetExam: string | null = null,
): Promise<JourneyRouteSnapshot> {
  return ipc<JourneyRouteSnapshot>('build_or_refresh_journey_route', {
    studentId,
    subjectId,
    targetExam,
  })
}

export function getActiveJourneyRoute(
  studentId: number,
  subjectId: number,
): Promise<JourneyRouteSnapshot | null> {
  return ipc<JourneyRouteSnapshot | null>('get_active_journey_route', { studentId, subjectId })
}

export function completeJourneyStation(
  stationId: number,
  evidence: Record<string, unknown> = {},
): Promise<JourneyRouteSnapshot> {
  return ipc<JourneyRouteSnapshot>('complete_journey_station', { stationId, evidence })
}

export interface JourneyStationDetail {
  id: number
  route_id: number
  station_code: string
  title: string
  station_type: string
  sequence_no: number
  status: string
  progress_score: number
  completion_confidence: number
  student_id: number
  subject_id: number
}

export function getJourneyStation(stationId: number): Promise<JourneyStationDetail> {
  return ipc<JourneyStationDetail>('get_journey_station', { stationId })
}

export interface AdaptationResult {
  needs_rebuild: boolean
  actions: string[]
}

export function adaptJourneyRoute(
  studentId: number,
  subjectId: number,
): Promise<AdaptationResult> {
  return ipc<AdaptationResult>('adapt_journey_route', { studentId, subjectId })
}
