import { ipc } from '.'
import type {
  SessionSnapshotDto,
  SessionSummaryDto,
  PracticeSessionStartInput,
  CustomTestStartInput,
  MockBlueprintInput,
  MockBlueprintDto,
  PackSummaryDto,
  PackInstallResultDto,
} from '@/types'

export function startPracticeSession(input: PracticeSessionStartInput): Promise<SessionSnapshotDto> {
  return ipc<SessionSnapshotDto>('start_practice_session', { input })
}

export function composeCustomTest(input: CustomTestStartInput): Promise<SessionSnapshotDto> {
  return ipc<SessionSnapshotDto>('compose_custom_test', { input })
}

export function completeSession(sessionId: number): Promise<SessionSummaryDto> {
  return ipc<SessionSummaryDto>('complete_session', { sessionId })
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
