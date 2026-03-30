export type ErrorTypeKey =
  | 'knowledge_gap' | 'conceptual_confusion' | 'recognition_failure' | 'execution_error'
  | 'carelessness' | 'pressure_breakdown' | 'expression_weakness' | 'speed_error'
  | 'guessing_detected' | 'misconception_triggered'

export interface ErrorTypeDisplay {
  label: string
  color: string
  bg: string
  icon: string
  description: string
  coachMessage: string
}

export const errorTypes: Record<ErrorTypeKey, ErrorTypeDisplay> = {
  knowledge_gap: {
    label: 'Knowledge Gap',
    color: '#dc2626', bg: '#fee2e2', icon: '❌',
    description: 'You do not know this concept yet',
    coachMessage: 'This is a topic you need to learn. Let us teach it to you properly.',
  },
  conceptual_confusion: {
    label: 'Conceptual Confusion',
    color: '#ea580c', bg: '#fff7ed', icon: '🔄',
    description: 'You confuse this with a similar concept',
    coachMessage: 'You are mixing up two related ideas. Let us separate them clearly.',
  },
  recognition_failure: {
    label: 'Recognition Failure',
    color: '#d97706', bg: '#fef3c7', icon: '👁',
    description: 'You know the concept but did not recognize it in this form',
    coachMessage: 'You know this — you just did not see it. Let us practice recognizing it in different disguises.',
  },
  execution_error: {
    label: 'Execution Error',
    color: '#ca8a04', bg: '#fef9c3', icon: '⚙',
    description: 'You know what to do but made an error in the steps',
    coachMessage: 'Your method was right but a step went wrong. Let us practice the procedure.',
  },
  carelessness: {
    label: 'Careless Error',
    color: '#65a30d', bg: '#ecfccb', icon: '⚡',
    description: 'A simple mistake that you would normally get right',
    coachMessage: 'You know this. Slow down and check your work.',
  },
  pressure_breakdown: {
    label: 'Pressure Breakdown',
    color: '#7c3aed', bg: '#f5f3ff', icon: '😰',
    description: 'You know this but timing pressure caused failure',
    coachMessage: 'Under calm conditions you would get this right. Let us build your pressure resilience.',
  },
  expression_weakness: {
    label: 'Expression Weakness',
    color: '#db2777', bg: '#fce7f3', icon: '✍',
    description: 'You understand but struggle to express the answer clearly',
    coachMessage: 'You understand the concept. Let us work on expressing your answer clearly.',
  },
  speed_error: {
    label: 'Speed Error',
    color: '#0891b2', bg: '#ecfeff', icon: '⏱',
    description: 'You ran out of time or rushed the answer',
    coachMessage: 'Speed is your bottleneck here. Let us build fluency through timed practice.',
  },
  guessing_detected: {
    label: 'Guessing Detected',
    color: '#71717a', bg: '#f4f4f5', icon: '🎲',
    description: 'The response pattern suggests guessing',
    coachMessage: 'It looks like you guessed. That is okay — let us learn this properly.',
  },
  misconception_triggered: {
    label: 'Misconception',
    color: '#be123c', bg: '#ffe4e6', icon: '🚫',
    description: 'A specific wrong mental model led to this error',
    coachMessage: 'You have a specific misunderstanding here. Let us correct it.',
  },
}

export function getErrorDisplay(errorType: string): ErrorTypeDisplay {
  return errorTypes[errorType as ErrorTypeKey] ?? {
    label: errorType, color: '#71717a', bg: '#f4f4f5', icon: '●',
    description: 'Error type', coachMessage: 'Let us work on this.',
  }
}
