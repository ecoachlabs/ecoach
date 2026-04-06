import { ipc } from '.'

export interface CoachDirectiveDto {
  directive_type: string
  audience: string
  title: string
  summary: string
  priority: string
  primary_action: string
  supporting_signals: string[]
}

export interface InsightCardDto {
  card_key: string
  audience: string
  title: string
  summary: string
  tone: string
  metric_label: string | null
  metric_value: number | null
  tags: string[]
}

export interface ActionRecommendationDto {
  recommendation_key: string
  audience: string
  label: string
  summary: string
  route_key: string
  urgency: string
  rationale: string
}

export interface AudienceExplanationDto {
  audience: string
  headline: string
  summary: string
  supporting_points: string[]
}

export interface ParentProductStudentFocusDto {
  student_id: number
  student_name: string
  overall_readiness_band: string
  exam_target: string | null
  active_risks: Array<{
    severity: string
    title: string
    description: string
  }>
  recommendations: string[]
  trend_summary: string[]
  weekly_memo: string
  subject_summaries: Array<{
    subject_id: number
    subject_name: string
    readiness_band: string
    mastered_topic_count: number
    weak_topic_count: number
    total_topic_count: number
  }>
}

export interface ParentProductSurfaceDto {
  parent_id: number
  parent_name: string
  generated_at: string
  student_focus: ParentProductStudentFocusDto | null
  directives: CoachDirectiveDto[]
  insight_cards: InsightCardDto[]
  action_recommendations: ActionRecommendationDto[]
  audience_explanations: AudienceExplanationDto[]
}

export function getParentProductSurface(
  parentId: number,
  studentId: number | null = null,
): Promise<ParentProductSurfaceDto> {
  return ipc<ParentProductSurfaceDto>('get_parent_product_surface', {
    parentId,
    studentId,
  })
}
