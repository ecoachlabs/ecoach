use ecoach_games::{
    GameType, GamesService, MindstackState, StartGameInput, SubmitGameAnswerInput, TugOfWarState,
};

use crate::{
    dtos::{TrapRoundCardDto, TrapSessionSnapshotDto},
    error::CommandError,
    state::AppState,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSessionDto {
    pub id: i64,
    pub game_type: String,
    pub session_state: String,
    pub score: i64,
    pub rounds_total: i64,
    pub rounds_played: i64,
    pub streak: i64,
    pub best_streak: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnswerResultDto {
    pub is_correct: bool,
    pub points_earned: i64,
    pub new_score: i64,
    pub streak: i64,
    pub effect_type: String,
    pub round_number: i64,
    pub session_complete: bool,
    pub explanation: Option<String>,
    pub misconception_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSummaryDto {
    pub session_id: i64,
    pub game_type: String,
    pub score: i64,
    pub accuracy_bp: i64,
    pub rounds_played: i64,
    pub best_streak: i64,
    pub average_response_time_ms: i64,
    pub misconception_hits: i64,
    pub performance_label: String,
    pub focus_signals: Vec<String>,
    pub recommended_next_step: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntryDto {
    pub student_id: i64,
    pub display_name: String,
    pub best_score: i64,
    pub games_played: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MindstackStateDto {
    pub board_height: i64,
    pub cleared_rows: i64,
    pub pending_block_type: String,
}

impl From<MindstackState> for MindstackStateDto {
    fn from(value: MindstackState) -> Self {
        Self {
            board_height: value.board_height,
            cleared_rows: value.cleared_rows,
            pending_block_type: value.pending_block_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TugOfWarStateDto {
    pub position: i64,
    pub opponent_difficulty: i64,
}

impl From<TugOfWarState> for TugOfWarStateDto {
    fn from(value: TugOfWarState) -> Self {
        Self {
            position: value.position,
            opponent_difficulty: value.opponent_difficulty as i64,
        }
    }
}

pub fn start_game(state: &AppState, input: StartGameInput) -> Result<GameSessionDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let session = service.start_game_session(&input)?;
        Ok(GameSessionDto {
            id: session.id,
            game_type: session.game_type,
            session_state: session.session_state,
            score: session.score,
            rounds_total: session.rounds_total,
            rounds_played: session.rounds_played,
            streak: session.streak,
            best_streak: session.best_streak,
        })
    })
}

pub fn submit_game_answer(
    state: &AppState,
    input: SubmitGameAnswerInput,
) -> Result<GameAnswerResultDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let result = service.submit_answer(&input)?;
        Ok(GameAnswerResultDto {
            is_correct: result.is_correct,
            points_earned: result.points_earned,
            new_score: result.new_score,
            streak: result.streak,
            effect_type: result.effect_type,
            round_number: result.round_number,
            session_complete: result.session_complete,
            explanation: result.explanation,
            misconception_triggered: result.misconception_triggered,
        })
    })
}

pub fn get_game_summary(state: &AppState, session_id: i64) -> Result<GameSummaryDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let summary = service.get_summary(session_id)?;
        Ok(GameSummaryDto {
            session_id: summary.session_id,
            game_type: summary.game_type,
            score: summary.score,
            accuracy_bp: summary.accuracy_bp as i64,
            rounds_played: summary.rounds_played,
            best_streak: summary.best_streak,
            average_response_time_ms: summary.average_response_time_ms,
            misconception_hits: summary.misconception_hits,
            performance_label: summary.performance_label,
            focus_signals: summary.focus_signals,
            recommended_next_step: summary.recommended_next_step,
        })
    })
}

pub fn get_trap_session_snapshot(
    state: &AppState,
    session_id: i64,
) -> Result<TrapSessionSnapshotDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let snapshot = service.get_traps_snapshot(session_id)?;
        Ok(TrapSessionSnapshotDto::from(snapshot))
    })
}

pub fn reveal_trap_unmask_clue(
    state: &AppState,
    session_id: i64,
    round_id: i64,
) -> Result<TrapRoundCardDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let round = service.reveal_unmask_clue(session_id, round_id)?;
        Ok(TrapRoundCardDto::from(round))
    })
}

pub fn get_mindstack_state(
    state: &AppState,
    session_id: i64,
) -> Result<MindstackStateDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let session_state = service.get_mindstack_state(session_id)?;
        Ok(MindstackStateDto::from(session_state))
    })
}

pub fn get_tug_of_war_state(
    state: &AppState,
    session_id: i64,
) -> Result<TugOfWarStateDto, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let session_state = service.get_tug_of_war_state(session_id)?;
        Ok(TugOfWarStateDto::from(session_state))
    })
}

pub fn list_game_sessions(
    state: &AppState,
    student_id: i64,
    limit: usize,
) -> Result<Vec<GameSessionDto>, CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        let sessions = service.list_sessions_for_student(student_id, limit)?;
        Ok(sessions
            .into_iter()
            .map(|session| GameSessionDto {
                id: session.id,
                game_type: session.game_type,
                session_state: session.session_state,
                score: session.score,
                rounds_total: session.rounds_total,
                rounds_played: session.rounds_played,
                streak: session.streak,
                best_streak: session.best_streak,
            })
            .collect())
    })
}

pub fn get_leaderboard(
    state: &AppState,
    game_type: String,
    limit: usize,
) -> Result<Vec<LeaderboardEntryDto>, CommandError> {
    state.with_connection(|conn| {
        let gt = GameType::from_str(&game_type).ok_or_else(|| CommandError {
            code: "validation_error".to_string(),
            message: format!("unknown game type: {}", game_type),
        })?;
        let service = GamesService::new(conn);
        let entries = service.get_leaderboard(gt, limit)?;
        Ok(entries
            .into_iter()
            .map(|e| LeaderboardEntryDto {
                student_id: e.student_id,
                display_name: e.display_name,
                best_score: e.best_score,
                games_played: e.games_played,
            })
            .collect())
    })
}

pub fn pause_game(state: &AppState, session_id: i64) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        service.pause_session(session_id)?;
        Ok(())
    })
}

pub fn resume_game(state: &AppState, session_id: i64) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        service.resume_session(session_id)?;
        Ok(())
    })
}

pub fn abandon_game(state: &AppState, session_id: i64) -> Result<(), CommandError> {
    state.with_connection(|conn| {
        let service = GamesService::new(conn);
        service.abandon_session(session_id)?;
        Ok(())
    })
}
