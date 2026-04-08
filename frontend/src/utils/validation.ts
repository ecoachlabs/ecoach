export const PIN_LENGTH = 4

/** Validate PIN format */
export function isValidPin(pin: string): boolean {
  return new RegExp(`^\\d{${PIN_LENGTH}}$`).test(pin)
}

/** Validate display name */
export function isValidName(name: string): boolean {
  return name.trim().length >= 2 && name.trim().length <= 50
}

/** Validate email (basic) */
export function isValidEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)
}

/** Check if a BasisPoints value is in valid range */
export function isValidBp(bp: number): boolean {
  return Number.isInteger(bp) && bp >= 0 && bp <= 10000
}

/** Required field check */
export function required(value: any): string | null {
  if (value === null || value === undefined || value === '') return 'This field is required'
  if (Array.isArray(value) && value.length === 0) return 'At least one item is required'
  return null
}

/** Min length check */
export function minLength(value: string, min: number): string | null {
  if (value.length < min) return `Must be at least ${min} characters`
  return null
}

/** Max length check */
export function maxLength(value: string, max: number): string | null {
  if (value.length > max) return `Must be at most ${max} characters`
  return null
}

/** Numeric range check */
export function inRange(value: number, min: number, max: number): string | null {
  if (value < min || value > max) return `Must be between ${min} and ${max}`
  return null
}
