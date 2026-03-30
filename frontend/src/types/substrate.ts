/** Score on 0-10000 scale (basis points). Display as percentage: (bp / 100).toFixed(0) + '%' */
export type BasisPoints = number

export type AccountType = 'student' | 'parent' | 'admin'
export type EntitlementTier = 'standard' | 'premium' | 'elite'
export type Role = 'student' | 'parent' | 'admin' | 'super_admin'
