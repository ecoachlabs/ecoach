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
