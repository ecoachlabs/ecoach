export type EliteProfileLike = {
  eps_score: number
  tier: string
  precision_score: number
  speed_score: number
  depth_score: number
  composure_score: number
}

export type EliteTopicDominationLike = {
  topic_name: string
  domination_score: number
}

export type ElitePersonalBestRow = [recordType: string, recordValue: number, achievedAt: string]
export type EliteEarnedBadgeRow = [badgeCode: string, badgeName: string, earnedAt: string]

export type EliteRecordCard = {
  category: string
  value: string
  date: string
  isPersonalBest: boolean
}

export type EliteBadgeCard = {
  name: string
  icon: string
  earned: boolean
  description: string
}

export type EliteTitleCard = {
  title: string
  earned: boolean
}

type EliteMetricConfig = {
  category: string
  recordType: string
  currentValue: (profile: EliteProfileLike) => number
  formatValue: (value: number) => string
}

const eliteMetricConfigs: EliteMetricConfig[] = [
  {
    category: 'EPS Score',
    recordType: 'highest_eps',
    currentValue: profile => profile.eps_score,
    formatValue: value => String(value),
  },
  {
    category: 'Precision',
    recordType: 'highest_precision',
    currentValue: profile => profile.precision_score,
    formatValue: value => `${Math.round(value / 100)}%`,
  },
  {
    category: 'Speed',
    recordType: 'highest_speed',
    currentValue: profile => profile.speed_score,
    formatValue: value => `${Math.round(value / 100)}%`,
  },
  {
    category: 'Depth',
    recordType: 'highest_depth',
    currentValue: profile => profile.depth_score,
    formatValue: value => `${Math.round(value / 100)}%`,
  },
  {
    category: 'Composure',
    recordType: 'highest_composure',
    currentValue: profile => profile.composure_score,
    formatValue: value => `${Math.round(value / 100)}%`,
  },
]

const eliteBadgeLibrary = [
  { code: 'perfect_run', name: 'Perfect Run', icon: 'PR', fallbackDescription: 'Finish a perfect elite run.' },
  { code: 'speed_authority', name: 'Speed Authority', icon: 'SA', fallbackDescription: 'Own the clean sprint board.' },
  { code: 'distinction_machine', name: 'Distinction Machine', icon: 'DM', fallbackDescription: 'Reach the Apex or Legend tier.' },
  { code: 'legend_status', name: 'Legend Status', icon: 'LS', fallbackDescription: 'Finish the climb at Legend.' },
]

const eliteTitles = [
  'Foundation Scholar',
  'Core Contender',
  'Prime Performer',
  'Apex Achiever',
  'Master Strategist',
  'Legend',
]

const tierOrder = ['Foundation', 'Core', 'Prime', 'Apex', 'Master', 'Legend']

function compactDateLabel(value: string): string {
  const parsed = new Date(value)
  if (Number.isNaN(parsed.getTime())) return 'Recorded'
  return parsed.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
  })
}

export function buildEliteRecordsView(
  profile: EliteProfileLike,
  topicDomination: EliteTopicDominationLike[],
  personalBests: ElitePersonalBestRow[],
  earnedBadges: EliteEarnedBadgeRow[],
): {
  records: EliteRecordCard[]
  badges: EliteBadgeCard[]
  titles: EliteTitleCard[]
} {
  const bestMap = new Map(personalBests.map(([recordType, recordValue, achievedAt]) => [
    recordType,
    { recordValue, achievedAt },
  ]))
  const earnedBadgeMap = new Map(earnedBadges.map(([badgeCode, badgeName, earnedAt]) => [
    badgeCode,
    { badgeName, earnedAt },
  ]))

  const records = eliteMetricConfigs.map(config => {
    const persisted = bestMap.get(config.recordType)
    if (persisted) {
      return {
        category: config.category,
        value: config.formatValue(persisted.recordValue),
        date: compactDateLabel(persisted.achievedAt),
        isPersonalBest: true,
      }
    }

    return {
      category: config.category,
      value: config.formatValue(config.currentValue(profile)),
      date: 'Current',
      isPersonalBest: false,
    }
  })

  records.push({
    category: 'Best Topic',
    value: topicDomination[0]?.topic_name ?? '--',
    date: topicDomination[0] ? 'Current' : '',
    isPersonalBest: (topicDomination[0]?.domination_score ?? 0) >= 7000,
  })

  const badges = eliteBadgeLibrary.map(badge => {
    const earned = earnedBadgeMap.get(badge.code)
    return {
      name: earned?.badgeName ?? badge.name,
      icon: badge.icon,
      earned: Boolean(earned),
      description: earned ? `Earned ${compactDateLabel(earned.earnedAt)}` : badge.fallbackDescription,
    }
  })

  const tierIndex = tierOrder.findIndex(tier => tier.toLowerCase() === profile.tier.toLowerCase())
  const titles = eliteTitles.map((title, index) => ({
    title,
    earned: tierIndex >= index,
  }))

  return { records, badges, titles }
}
