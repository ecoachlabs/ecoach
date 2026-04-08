import { ipc } from '.'
import type { AccountDto, AccountSummaryDto, CreateAccountInput } from '@/types'

export function listAccounts(): Promise<AccountSummaryDto[]> {
  return ipc<AccountSummaryDto[]>('list_accounts')
}

export function loginWithPin(accountId: number, pin: string): Promise<AccountDto> {
  return ipc<AccountDto>('login_with_pin', { accountId, pin })
}

export function createAccount(input: CreateAccountInput): Promise<AccountDto> {
  return ipc<AccountDto>('create_account', { input })
}

export function linkParentStudent(parentId: number, studentId: number): Promise<void> {
  return ipc<void>('link_parent_student', { parentId, studentId })
}

export function listLinkedStudents(parentId: number): Promise<AccountSummaryDto[]> {
  return ipc<AccountSummaryDto[]>('list_linked_students', { parentId })
}

export interface ParentAlertRecordDto {
  id: number
  learner_id: number
  parent_id: number
  trigger_type: string
  severity: string
  message: string
  action_required: string | null
  status: string
  created_at: string
  acknowledged_at: string | null
}

export function listParentAlerts(
  parentId: number,
  learnerId: number | null,
  status: string | null,
  limit: number,
): Promise<ParentAlertRecordDto[]> {
  return ipc<ParentAlertRecordDto[]>('list_parent_alerts', { parentId, learnerId, status, limit })
}

export function acknowledgeParentAlert(alertId: number): Promise<ParentAlertRecordDto | null> {
  return ipc<ParentAlertRecordDto | null>('acknowledge_parent_alert', { alertId })
}

export function resetPin(accountId: number, newPin: string): Promise<void> {
  return ipc<void>('reset_pin', { accountId, newPin })
}
