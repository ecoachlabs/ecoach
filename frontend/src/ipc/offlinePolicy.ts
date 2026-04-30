export type ConnectivityMode = 'offline-native' | 'online-enhanced' | 'online-required'

export interface CommandPolicy {
  mode: ConnectivityMode
  retryWhenOnline: boolean
  label: string
  description?: string
}

const DEFAULT_COMMAND_POLICY: CommandPolicy = {
  mode: 'offline-native',
  retryWhenOnline: false,
  label: 'Local app action',
  description: 'Runs against the bundled local application and database.',
}

const ONLINE_ENHANCED_POLICIES: Record<string, CommandPolicy> = {
  check_entitlement: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Entitlement refresh',
    description: 'Uses local entitlement data offline and refreshes when internet is available.',
  },
  is_feature_enabled: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Feature availability refresh',
    description: 'Falls back to the local feature state and can refresh online later.',
  },
  generate_parent_digest: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Parent digest generation',
    description: 'Can be generated locally, with online delivery or enhancement retried later.',
  },
  register_curriculum_source: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Curriculum source registration',
    description: 'Local files can register offline; web sources wait for internet.',
  },
  add_curriculum_parse_candidate: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Curriculum parse candidate',
    description: 'Stores local parse work now and retries network-dependent enrichment later.',
  },
  finalize_curriculum_source: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Curriculum source finalization',
    description: 'Finalizes local content and retries online publishing hooks later.',
  },
  mark_curriculum_source_reviewed: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Curriculum source review',
    description: 'Keeps the review workflow local and retries any online sync later.',
  },
  run_foundry_job: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Content foundry job',
    description: 'Local jobs can run offline; web or model backed jobs recover online.',
  },
  run_next_foundry_job: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Content foundry queue',
    description: 'Keeps the queue available and resumes online-dependent jobs later.',
  },
  process_question_generation_request: {
    mode: 'online-enhanced',
    retryWhenOnline: true,
    label: 'Question generation request',
    description: 'Processes local generation requests and retries online enrichment later.',
  },
}

const ONLINE_REQUIRED_SOURCE_KINDS = new Set([
  'url',
  'web',
  'web_page',
  'web_source',
  'remote_url',
  'remote',
])

export function getCommandPolicy(
  command: string,
  args?: Record<string, unknown>,
): CommandPolicy {
  if (command === 'register_curriculum_source' && hasRemoteCurriculumSource(args)) {
    return {
      mode: 'online-required',
      retryWhenOnline: true,
      label: 'Web curriculum source registration',
      description: 'A remote source needs internet before it can be read and registered.',
    }
  }

  return ONLINE_ENHANCED_POLICIES[command] ?? DEFAULT_COMMAND_POLICY
}

export function getOnlineRequiredReason(
  command: string,
  args?: Record<string, unknown>,
): string | null {
  if (command === 'register_curriculum_source' && hasRemoteCurriculumSource(args)) {
    return 'Remote curriculum sources need internet before they can be read and registered.'
  }

  return null
}

export function isOnlineRequired(policy: CommandPolicy): boolean {
  return policy.mode === 'online-required'
}

export function isNetworkLikeError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error)
  return /network|offline|internet|timed?\s*out|timeout|dns|connection|fetch|failed to fetch|unreachable/i.test(message)
}

function hasRemoteCurriculumSource(args?: Record<string, unknown>): boolean {
  const input = (args?.input ?? args) as Record<string, unknown> | undefined
  const sourceKind = typeof input?.source_kind === 'string' ? input.source_kind.toLowerCase() : ''
  const sourcePath = typeof input?.source_path === 'string' ? input.source_path.toLowerCase() : ''

  return ONLINE_REQUIRED_SOURCE_KINDS.has(sourceKind) || /^https?:\/\//.test(sourcePath)
}
