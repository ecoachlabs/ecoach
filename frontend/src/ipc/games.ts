import { ipc } from '.'

export function startGame(input: any): Promise<any> {
  return ipc('start_game', { input })
}

export function submitGameAnswer(input: any): Promise<any> {
  return ipc('submit_game_answer', { input })
}

export function getGameSummary(sessionId: number): Promise<any> {
  return ipc('get_game_summary', { sessionId })
}

export function getMindstackState(sessionId: number): Promise<any> {
  return ipc('get_mindstack_state', { sessionId })
}

export function getTugOfWarState(sessionId: number): Promise<any> {
  return ipc('get_tug_of_war_state', { sessionId })
}

export function listGameSessions(studentId: number): Promise<any[]> {
  return ipc('list_game_sessions', { studentId })
}

export function getLeaderboard(gameType: string): Promise<any[]> {
  return ipc('get_leaderboard', { gameType })
}

export function pauseGame(sessionId: number): Promise<void> {
  return ipc('pause_game', { sessionId })
}

export function resumeGame(sessionId: number): Promise<void> {
  return ipc('resume_game', { sessionId })
}

export function abandonGame(sessionId: number): Promise<void> {
  return ipc('abandon_game', { sessionId })
}

// Traps
export function listTrapsPairs(studentId: number, subjectId: number, topicIds: number[]): Promise<any[]> {
  return ipc('list_traps_pairs', { studentId, subjectId, topicIds })
}

export function startTrapsSession(input: any): Promise<any> {
  return ipc('start_traps_session', { input })
}

export function submitTrapRound(input: any): Promise<any> {
  return ipc('submit_trap_round', { input })
}

export function recordTrapConfusionReason(input: any): Promise<void> {
  return ipc('record_trap_confusion_reason', { input })
}

export function getTrapReview(sessionId: number): Promise<any> {
  return ipc('get_trap_review', { sessionId })
}
