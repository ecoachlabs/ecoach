export interface CmsMetricItem {
  label: string
  value: string | number
  caption?: string
  tone?: 'neutral' | 'good' | 'review' | 'danger'
}

export interface CmsActionItem {
  key: string
  title: string
  summary: string
  tone?: 'neutral' | 'review' | 'danger'
  actionLabel?: string
  route?: string
}
