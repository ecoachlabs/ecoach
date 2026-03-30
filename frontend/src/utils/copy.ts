/**
 * Audience-aware copy translation layer.
 * Same data, different language for students, parents, and admins.
 */

export type Audience = 'student' | 'parent' | 'admin'

/** Translate a readiness band to audience-appropriate text */
export function readinessText(band: string, audience: Audience): string {
  const translations: Record<string, Record<Audience, string>> = {
    strong: {
      student: 'You are performing well in this area',
      parent: 'Your child demonstrates strong command of this subject',
      admin: 'Strong readiness band — above 70th percentile',
    },
    developing: {
      student: 'You are getting there — keep practicing',
      parent: 'Your child is making progress but needs continued focus',
      admin: 'Developing band — 40-70th percentile range',
    },
    weak: {
      student: 'This area needs your attention. We have a plan.',
      parent: 'This subject requires focused intervention support',
      admin: 'Weak band — below 40th percentile, priority intervention',
    },
    critical: {
      student: 'We need to rebuild this together. Start with the basics.',
      parent: 'Urgent attention needed — foundational gaps identified',
      admin: 'Critical band — below 20th percentile, rescue pathway required',
    },
  }
  return translations[band]?.[audience] ?? band
}

/** Translate a mastery state to audience-appropriate text */
export function masteryText(state: string, audience: Audience): string {
  if (audience === 'admin') return state
  const studentTexts: Record<string, string> = {
    unseen: 'Not started yet',
    exposed: 'Just getting started',
    emerging: 'Beginning to understand',
    partial: 'Getting there',
    fragile: 'You know it but it breaks under pressure',
    stable: 'Solid understanding',
    robust: 'Strong and reliable',
    exam_ready: 'Ready for the exam',
  }
  const parentTexts: Record<string, string> = {
    unseen: 'Not yet covered',
    exposed: 'Introduced but not practiced',
    emerging: 'Early understanding forming',
    partial: 'Inconsistent performance',
    fragile: 'Knows concept but unstable under test conditions',
    stable: 'Consistent correct performance',
    robust: 'Well retained and reliable',
    exam_ready: 'Exam-ready performance level',
  }
  return (audience === 'parent' ? parentTexts : studentTexts)[state] ?? state
}

/** Get a greeting based on time of day */
export function timeGreeting(): string {
  const hour = new Date().getHours()
  if (hour < 12) return 'Good morning'
  if (hour < 17) return 'Good afternoon'
  return 'Good evening'
}
