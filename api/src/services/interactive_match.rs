use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{Room, RoomStatus, Match, MatchStatus, MatchParticipant, Game, GameType},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_interactive_match_from_room(
    db: &Database,
    room_id: String,
) -> ApiResult<Match> {
    let room_thing = room_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid room id".to_string()))?;

    let room: Option<Room> = db.select(("room", room_thing.id.to_string())).await?;
    let room = room.ok_or_else(|| ApiError::NotFound("Room not found".to_string()))?;

    if room.status != RoomStatus::Playing {
        return Err(ApiError::BadRequest("Room is not in playing state".to_string()));
    }

    if room.players.len() != 2 {
        return Err(ApiError::BadRequest("Interactive matches require exactly 2 players".to_string()));
    }

    // Get game details
    let game: Option<Game> = db.select(("game", room.game_id.id.to_string())).await?;
    let game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    if game.game_type != GameType::Interactive {
        return Err(ApiError::BadRequest("Game is not interactive".to_string()));
    }

    // Create match participants
    let participants = vec![
        MatchParticipant {
            user_id: Some(room.players[0].clone()),
            submission_id: None,
            score: None,
            metadata: None,
        },
        MatchParticipant {
            user_id: Some(room.players[1].clone()),
            submission_id: None,
            score: None,
            metadata: None,
        },
    ];

    // Create match
    let match_data = Match {
        id: None,
        tournament_id: None, // Interactive matches can be standalone
        game_id: room.game_id.clone(),
        status: MatchStatus::Running,
        participants,
        metadata: None,
        room_id: Some(room_thing),
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: Some(Datetime::default()),
        completed_at: None,
    };

    let created: Option<Match> = db.create("match").content(match_data).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create match".to_string()))
}

pub async fn complete_interactive_match(
    db: &Database,
    match_id: String,
    player_scores: Vec<f64>,
    result_data: Option<serde_json::Value>,
) -> ApiResult<Match> {
    let match_thing = match_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid match id".to_string()))?;

    let match_data: Option<Match> = db.select(("match", match_thing.id.to_string())).await?;
    let mut match_data = match_data.ok_or_else(|| ApiError::NotFound("Match not found".to_string()))?;

    // Update participant scores
    for (i, participant) in match_data.participants.iter_mut().enumerate() {
        if i < player_scores.len() {
            participant.score = Some(player_scores[i]);
        }
    }

    match_data.status = MatchStatus::Completed;
    match_data.metadata = result_data;
    match_data.completed_at = Some(Datetime::default());
    match_data.updated_at = Datetime::default();

    // Update room status to finished
    if let Some(room_id) = &match_data.room_id {
        let room: Option<Room> = db.select(("room", room_id.id.to_string())).await?;
        if let Some(mut room) = room {
            room.status = RoomStatus::Finished;
            room.updated_at = Datetime::default();
            let _: Option<Room> = db.update(("room", room_id.id.to_string())).content(room).await?;
        }
    }

    let updated: Option<Match> = db.update(("match", match_thing.id.to_string())).content(match_data).await?;
    updated.ok_or_else(|| ApiError::Internal("Failed to update match".to_string()))
}
