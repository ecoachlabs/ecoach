export interface Subject {
  id: number
  curriculum_version_id: number
  code: string
  name: string
  display_order: number
}

export interface TopicSummary {
  id: number
  subject_id: number
  parent_topic_id: number | null
  code: string | null
  name: string
  node_type: string
  display_order: number
}

export interface AcademicNode {
  id: number
  topic_id: number
  node_type: string
  canonical_title: string
  core_meaning: string | null
  exam_relevance_score: number
  created_at: string
}
