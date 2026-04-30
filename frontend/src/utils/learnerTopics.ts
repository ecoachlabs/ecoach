export interface LearnerTopicSource {
  id: number
  subject_id: number
  parent_topic_id: number | null
  code: string | null
  name: string
  description: string | null
  node_type: string
  display_order: number
}

export interface LearnerTopic {
  id: number
  subjectId: number
  parentTopicId: number | null
  strandId: number
  subStrandId: number
  name: string
  description: string | null
  goalDescriptions: string[]
  sourceTopicIds: number[]
  sourceTopicCount: number
  code: string | null
  display_order: number
}

export interface LearnerSubStrand {
  id: number
  name: string
  code: string | null
  display_order: number
  topics: LearnerTopic[]
}

export interface LearnerStrand {
  id: number
  name: string
  code: string | null
  display_order: number
  subStrands: LearnerSubStrand[]
  totalTopics: number
}

export interface LearnerTopicIndex {
  tree: LearnerStrand[]
  topics: LearnerTopic[]
  bySourceTopicId: Map<number, LearnerTopic>
}

const CURRICULUM_CODE_PATTERN = /\b[A-Z]{1,3}\s*\d+(?:\s*\.\s*\d+)+\b/g
const CURRICULUM_CODE_FRAGMENT_PATTERN = /\b[A-Z]{1,3}\s*\d+(?:[.\-\\()]+\d+){2,}\b/g

function tidyWhitespace(value: string): string {
  return value
    .replace(/\s+([,.;:!?])/g, '$1')
    .replace(/\(\s+/g, '(')
    .replace(/\s+\)/g, ')')
    .replace(/\[\s+/g, '[')
    .replace(/\s+\]/g, ']')
    .replace(/\s{2,}/g, ' ')
    .trim()
}

function splitChunks(value: string): string[] {
  return value
    .split(/\n+/)
    .flatMap(line => line.split(/(?<=[.!?])\s+(?=[A-Z0-9])/))
    .flatMap(line => line.split(/\s*;\s+(?=[A-Z0-9])/))
    .map(line => line.trim())
    .filter(Boolean)
}

function stripSyllabusScaffolding(value: string): string {
  return value
    .replace(
      /(?:^|\n{2,})(?:Curriculum link|Objective|Skill focus|Source anchors|Original solution note):[\s\S]*?(?=\n{2,}[A-Z][A-Za-z -]+:|$)/gi,
      ' ',
    )
    .replace(
      /Read the question as (?:an?\s+)?[^.]*?\bitem and restate the target:\s*/gi,
      'Restate the target: ',
    )
    .replace(/Identify the relevant skill:\s*[^.]*\.\s*/gi, '')
    .replace(/\s*while working on\s+[^.]*?\btasks?\b\.?/gi, '')
    .replace(/\btied to source exemplars\b\.?/gi, '')
    .replace(/\bsource exemplars\b/gi, 'worked examples')
}

function stripCurriculumCodes(value: string | null | undefined): string {
  if (!value) return ''
  const cleaned = stripSyllabusScaffolding(value)
    .replace(CURRICULUM_CODE_PATTERN, ' ')
    .replace(CURRICULUM_CODE_FRAGMENT_PATTERN, ' ')
    .replace(/\bas\s+(?:an?\s+)?item\b/gi, '')
    .replace(/\bfor\s+(?:an?\s+)?item\b/gi, '')
    .replace(/\bthis item\b/gi, 'this question')
    .replace(/\bthe item\b/gi, 'the question')
    .replace(/\bitem\b/gi, 'question')
    .replace(/\(\s*\)/g, ' ')
    .replace(/:\s*\./g, '.')
    .replace(/\s{2,}/g, ' ')

  return tidyWhitespace(cleaned)
}

function sanitizeLearnerSnippet(value: string | null | undefined): string {
  if (!value) return ''

  const parts = splitChunks(value)
    .map(part => stripCurriculumCodes(part))
    .filter(Boolean)

  if (parts.length === 0) return stripCurriculumCodes(value)
  return tidyWhitespace(parts.join(' '))
}

function learnerKey(value: string): string {
  return stripCurriculumCodes(value)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, ' ')
    .replace(/\s{2,}/g, ' ')
    .trim()
}

function sortSources(left: LearnerTopicSource, right: LearnerTopicSource): number {
  return left.display_order - right.display_order || left.id - right.id
}

function collectLeaves(
  parentId: number,
  byParent: Map<number | null, LearnerTopicSource[]>,
): LearnerTopicSource[] {
  const children = byParent.get(parentId) ?? []
  if (children.length === 0) return []

  const leaves: LearnerTopicSource[] = []
  for (const child of children) {
    const grandchildren = byParent.get(child.id) ?? []
    if (grandchildren.length === 0) {
      leaves.push(child)
      continue
    }
    leaves.push(...collectLeaves(child.id, byParent))
  }
  return leaves
}

function toLearnerTopic(
  source: LearnerTopicSource,
  strandId: number,
  subStrandId: number,
): LearnerTopic {
  const description = sanitizeLearnerSnippet(source.description)
  return {
    id: source.id,
    subjectId: source.subject_id,
    parentTopicId: source.parent_topic_id,
    strandId,
    subStrandId,
    name: stripCurriculumCodes(source.name),
    description: description || null,
    goalDescriptions: description ? [description] : [],
    sourceTopicIds: [source.id],
    sourceTopicCount: 1,
    code: null,
    display_order: source.display_order,
  }
}

function dedupeOrdered(values: string[]): string[] {
  const seen = new Set<string>()
  const output: string[] = []
  for (const value of values) {
    const normalized = value.trim()
    if (!normalized) continue
    if (seen.has(normalized)) continue
    seen.add(normalized)
    output.push(normalized)
  }
  return output
}

function consolidateTopics(topics: LearnerTopic[]): LearnerTopic[] {
  const groups = new Map<string, LearnerTopic[]>()
  const order: string[] = []

  for (const topic of topics) {
    const key = learnerKey(topic.name) || `topic-${topic.id}`
    if (!groups.has(key)) {
      groups.set(key, [])
      order.push(key)
    }
    groups.get(key)!.push(topic)
  }

  return order.map((key) => {
    const grouped = groups.get(key) ?? []
    const first = grouped[0]
    const sourceTopicIds = dedupeOrdered(
      grouped.flatMap(topic => topic.sourceTopicIds.map(String)),
    ).map(Number)
    const goalDescriptions = dedupeOrdered(
      grouped.flatMap(topic => topic.goalDescriptions),
    )

    return {
      ...first,
      description: goalDescriptions[0] ?? first.description,
      goalDescriptions,
      sourceTopicIds,
      sourceTopicCount: sourceTopicIds.length,
    }
  })
}

export function buildLearnerTopicTree(topics: LearnerTopicSource[]): LearnerStrand[] {
  if (topics.length === 0) return []

  const byParent = new Map<number | null, LearnerTopicSource[]>()
  for (const topic of topics) {
    const parentId = topic.parent_topic_id
    const bucket = byParent.get(parentId) ?? []
    bucket.push(topic)
    byParent.set(parentId, bucket)
  }
  for (const bucket of byParent.values()) {
    bucket.sort(sortSources)
  }

  const roots = (byParent.get(null) ?? []).slice().sort(sortSources)
  const strands: LearnerStrand[] = []

  for (const root of roots) {
    const children = (byParent.get(root.id) ?? []).slice().sort(sortSources)
    if (children.length === 0) continue

    const hasGrandchildren = children.some(child => (byParent.get(child.id) ?? []).length > 0)
    const subStrands: LearnerSubStrand[] = []

    if (hasGrandchildren) {
      for (const child of children) {
        const leaves = collectLeaves(child.id, byParent).sort(sortSources)
        if (leaves.length === 0) continue

        subStrands.push({
          id: child.id,
          name: stripCurriculumCodes(child.name),
          code: null,
          display_order: child.display_order,
          topics: consolidateTopics(
            leaves.map(leaf => toLearnerTopic(leaf, root.id, child.id)),
          ),
        })
      }
    } else {
      subStrands.push({
        id: root.id,
        name: stripCurriculumCodes(root.name),
        code: null,
        display_order: root.display_order,
        topics: consolidateTopics(
          children.map(child => toLearnerTopic(child, root.id, root.id)),
        ),
      })
    }

    const totalTopics = subStrands.reduce((sum, subStrand) => sum + subStrand.topics.length, 0)
    if (totalTopics === 0) continue

    strands.push({
      id: root.id,
      name: stripCurriculumCodes(root.name),
      code: null,
      display_order: root.display_order,
      subStrands,
      totalTopics,
    })
  }

  return strands
}

export function flattenLearnerTopics(strands: LearnerStrand[]): LearnerTopic[] {
  return strands.flatMap(strand => strand.subStrands.flatMap(subStrand => subStrand.topics))
}

export function buildLearnerTopicIndex(topics: LearnerTopicSource[]): LearnerTopicIndex {
  const tree = buildLearnerTopicTree(topics)
  const flattened = flattenLearnerTopics(tree)
  const bySourceTopicId = new Map<number, LearnerTopic>()

  for (const topic of flattened) {
    for (const sourceTopicId of topic.sourceTopicIds) {
      bySourceTopicId.set(sourceTopicId, topic)
    }
  }

  return {
    tree,
    topics: flattened,
    bySourceTopicId,
  }
}

export function expandLearnerTopicIds(
  selectedTopicIds: number[],
  topics: LearnerTopic[],
): number[] {
  const selected = new Set(selectedTopicIds)
  const sourceIds = new Set<number>()

  for (const topic of topics) {
    if (!selected.has(topic.id)) continue
    for (const sourceTopicId of topic.sourceTopicIds) {
      sourceIds.add(sourceTopicId)
    }
  }

  return Array.from(sourceIds)
}
