/** Matches AccountDto from ecoach-commands */
export interface AccountDto {
  id: number
  display_name: string
  account_type: string
  entitlement_tier: string
  status: string
  failed_pin_attempts: number
  is_locked: boolean
  needs_checkup: boolean
  last_active_label: string
}

/** Matches AccountSummaryDto from ecoach-commands */
export interface AccountSummaryDto {
  id: number
  display_name: string
  account_type: string
  status: string
  needs_checkup: boolean
  last_active_label: string
}

/** Matches CreateAccountInput from ecoach-identity */
export interface CreateAccountInput {
  account_type: string
  display_name: string
  pin: string
  entitlement_tier: string
}
