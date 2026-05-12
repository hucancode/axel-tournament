use crate::{
    db::Database,
    error::{AppError, AppResult},
    models::{matches::{Match, MatchParticipant}, tournament::Tournament},
};
use axel_core::repo::matches::MatchRepo;

pub(super) async fn write_bracket_match(
    db: &Database,
    tournament: &Tournament,
    p1: MatchParticipant,
    p2: Option<MatchParticipant>,
    round: u32,
    bracket: &str,
    position: u32,
) -> AppResult<()> {
    let tournament_id = tournament
        .id
        .clone()
        .ok_or_else(|| AppError::Internal("Tournament missing id".to_string()))?;
    let mut participants = vec![p1];
    if let Some(p) = p2 {
        participants.push(p);
    }
    let m = Match {
        tournament_id: Some(tournament_id),
        game_id: tournament.game_id.clone(),
        participants,
        round: Some(round),
        bracket: Some(bracket.to_string()),
        bracket_position: Some(position),
        ..Default::default()
    };
    <Database as MatchRepo>::create(db, m).await?;
    Ok(())
}
