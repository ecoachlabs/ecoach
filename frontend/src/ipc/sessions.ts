import { ipc } from '.'
import type {
  SessionSnapshotDto,
  SessionSummaryDto,
  SessionCompletionResultDto,
  PracticeSessionStartInput,
  CustomTestStartInput,
  MockBlueprintInput,
  MockBlueprintDto,
  PackSummaryDto,
  PackInstallResultDto,
} from '@/types'

export type {
  SessionSnapshotDto,
  SessionSummaryDto,
  SessionCompletionResultDto,
  PracticeSessionStartInput,
  CustomTestStartInput,
  MockBlueprintInput,
  MockBlueprintDto,
  PackSummaryDto,
  PackInstallResultDto,
} from '@/types'

export interface SessionPresenceEventInputDto {
  event_type: string
  occurred_at?: string | null
  metadata_json?: Record<string, unknown> | null
}

export interface DeferredCompletionRecoveryResultDto {
  attempted: number
  succeeded: number
  failed: number
  skipped: number
  remaining: number
  recovered_session_ids: number[]
  failed_session_ids: number[]
  skipped_session_ids: number[]
}

export function startPracticeSession(input: PracticeSessionStartInput): Promise<SessionSnapshotDto> {
  return ipc<SessionSnapshotDto>('start_practice_session', { input })
}

export function composeCustomTest(input: CustomTestStartInput): Promise<SessionSnapshotDto> {
  return ipc<SessionSnapshotDto>('compose_custom_test', { input })
}

export function completeSession(sessionId: number): Promise<SessionSummaryDto> {
  return ipc<SessionSummaryDto>('complete_session', { sessionId })
}

export function completeSessionWithPipeline(
  studentId: number,
  sessionId: number,
): Promise<SessionCompletionResultDto> {
  return ipc<SessionCompletionResultDto>('complete_session_with_pipeline', {
    studentId,
    sessionId,
  })
}

export function recoverDeferredSessionCompletions(
  studentId: number,
  maxSessions?: number,
): Promise<DeferredCompletionRecoveryResultDto> {
  return ipc<DeferredCompletionRecoveryResultDto>('recover_deferred_session_completions', {
    studentId,
    maxSessions,
  })
}

export function flagSessionItem(
  sessionId: number,
  itemId: number,
  flagged: boolean,
): Promise<void> {
  return ipc<void>('flag_session_item', { sessionId, itemId, flagged })
}

export function recordSessionPresenceEvent(
  sessionId: number,
  input: SessionPresenceEventInputDto,
): Promise<unknown> {
  return ipc<unknown>('record_session_presence_event', { sessionId, input })
}

export function generateMockBlueprint(input: MockBlueprintInput): Promise<MockBlueprintDto> {
  return ipc<MockBlueprintDto>('generate_mock_blueprint', { input })
}

export function startMockSession(blueprintId: number): Promise<SessionSnapshotDto> {
  return ipc<SessionSnapshotDto>('start_mock_session', { blueprintId })
}

export function listInstalledPacks(): Promise<PackSummaryDto[]> {
  return ipc<PackSummaryDto[]>('list_installed_packs')
}

export function installPack(path: string): Promise<PackInstallResultDto> {
  return ipc<PackInstallResultDto>('install_pack', { path })
}
