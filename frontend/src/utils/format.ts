/** Convert BasisPoints (0-10000) to percentage string */
export function formatBp(bp: number): string {
  return (bp / 100).toFixed(0) + '%'
}

/** Convert BasisPoints to decimal (0-1) */
export function bpToDecimal(bp: number): number {
  return bp / 10000
}

/** Format seconds to mm:ss */
export function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return mins > 0 ? `${mins}:${String(secs).padStart(2, '0')}` : `${secs}s`
}

/** Format milliseconds to human-readable */
export function formatMs(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  const secs = (ms / 1000).toFixed(1)
  return `${secs}s`
}

/** Format a date string to relative time */
export function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays === 1) return 'Yesterday'
  if (diffDays < 7) return `${diffDays}d ago`
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`
  return date.toLocaleDateString()
}

/** Format date to short display */
export function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-GB', {
    day: 'numeric', month: 'short', year: 'numeric',
  })
}

/** Pluralize a word */
export function pluralize(count: number, singular: string, plural?: string): string {
  return count === 1 ? singular : (plural || singular + 's')
}

/** Clamp a number between min and max */
export function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}
