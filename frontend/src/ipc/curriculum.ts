import { ipc } from '.'

export interface SubjectDto {
  id: number; curriculum_version_id: number; code: string; name: string; display_order: number
}
export interface TopicDto {
  id: number; subject_id: number; parent_topic_id: number | null; code: string | null; name: string; node_type: string; display_order: number
}

export function listSubjects(versionId: number = 1): Promise<SubjectDto[]> {
  return ipc<SubjectDto[]>('list_subjects', { curriculumVersionId: versionId })
}

export function listTopics(subjectId: number): Promise<TopicDto[]> {
  return ipc<TopicDto[]>('list_topics', { subjectId })
}

// Admin curriculum commands
export function registerCurriculumSource(input: any): Promise<any> {
  return ipc('register_curriculum_source', { input })
}

export function addParseCandiate(sourceId: number, input: any): Promise<any> {
  return ipc('add_curriculum_parse_candidate', { sourceId, input })
}

export function finalizeCurriculumSource(sourceId: number): Promise<any> {
  return ipc('finalize_curriculum_source', { sourceId })
}

export function markSourceReviewed(sourceId: number): Promise<any> {
  return ipc('mark_curriculum_source_reviewed', { sourceId })
}
