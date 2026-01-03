use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{room::{Room, RoomStatus, RoomMessage}, game::find_game_by_id},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_room(
    db: &Database,
    game_id: String,
    host_id: String,
    name: String,
    max_players: u32,
    human_timeout_ms: Option<u64>,
) -> ApiResult<Room> {
    // Verify game exists in hardcoded registry
    find_game_by_id(&game_id)
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    // Note: All games now support both automated and interactive modes,
    // so we don't need to check game type anymore

    let host_thing = host_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid host id".to_string()))?;

    let room = Room {
        id: None,
        game_id,
        host_id: host_thing.clone(),
        name,
        max_players,
        status: RoomStatus::Waiting,
        players: vec![host_thing],
        human_timeout_ms,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };

    let created: Option<Room> = db.create("room").content(room).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create room".to_string()))
}

pub async fn get_room(db: &Database, room_id: Thing) -> ApiResult<Room> {
    let room: Option<Room> = db.select(("room", room_id.id.to_string())).await?;
    room.ok_or_else(|| ApiError::NotFound("Room not found".to_string()))
}

pub async fn list_rooms(db: &Database, game_id: Option<String>) -> ApiResult<Vec<Room>> {
    let rooms: Vec<Room> = if let Some(gid) = game_id {
        db.query("SELECT * FROM room WHERE game_id = $game_id AND status = 'waiting' ORDER BY created_at DESC")
            .bind(("game_id", gid))
            .await?
            .take(0)?
    } else {
        db.query("SELECT * FROM room WHERE status = 'waiting' ORDER BY created_at DESC")
            .await?
            .take(0)?
    };
    Ok(rooms)
}

pub async fn join_room(db: &Database, room_id: Thing, user_id: String) -> ApiResult<Room> {
    let user_thing = user_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid user id".to_string()))?;

    let mut room = get_room(db, room_id.clone()).await?;
    
    if room.status != RoomStatus::Waiting {
        return Err(ApiError::BadRequest("Room is not accepting players".to_string()));
    }
    
    if room.players.contains(&user_thing) {
        return Err(ApiError::BadRequest("User already in room".to_string()));
    }
    
    if room.players.len() as u32 >= room.max_players {
        return Err(ApiError::BadRequest("Room is full".to_string()));
    }

    room.players.push(user_thing);
    room.updated_at = Datetime::default();

    let updated: Option<Room> = db.update(("room", room_id.id.to_string())).content(room).await?;
    updated.ok_or_else(|| ApiError::Internal("Failed to join room".to_string()))
}

pub async fn leave_room(db: &Database, room_id: Thing, user_id: String) -> ApiResult<Room> {
    let user_thing = user_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid user id".to_string()))?;

    let mut room = get_room(db, room_id.clone()).await?;
    
    if !room.players.contains(&user_thing) {
        return Err(ApiError::BadRequest("User not in room".to_string()));
    }

    room.players.retain(|p| p != &user_thing);
    room.updated_at = Datetime::default();

    // If host leaves, transfer to another player or delete room
    if room.host_id == user_thing {
        if let Some(new_host) = room.players.first() {
            room.host_id = new_host.clone();
        } else {
            // Delete empty room
            let _: Option<Room> = db.delete(("room", room_id.id.to_string())).await?;
            return Err(ApiError::NotFound("Room deleted".to_string()));
        }
    }

    let updated: Option<Room> = db.update(("room", room_id.id.to_string())).content(room).await?;
    updated.ok_or_else(|| ApiError::Internal("Failed to leave room".to_string()))
}

pub async fn start_game(db: &Database, room_id: Thing, host_id: String) -> ApiResult<Room> {
    let host_thing = host_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid host id".to_string()))?;

    let mut room = get_room(db, room_id.clone()).await?;
    
    if room.host_id != host_thing {
        return Err(ApiError::Forbidden("Only host can start the game".to_string()));
    }
    
    if room.status != RoomStatus::Waiting {
        return Err(ApiError::BadRequest("Game already started".to_string()));
    }
    
    if room.players.len() < 2 {
        return Err(ApiError::BadRequest("Need at least 2 players to start".to_string()));
    }

    room.status = RoomStatus::Playing;
    room.updated_at = Datetime::default();

    let updated: Option<Room> = db.update(("room", room_id.id.to_string())).content(room.clone()).await?;
    let updated_room = updated.ok_or_else(|| ApiError::Internal("Failed to start game".to_string()))?;

    // Create interactive match
    let _ = crate::services::interactive_match::create_interactive_match_from_room(
        db, 
        room_id.to_string()
    ).await;

    Ok(updated_room)
}

pub async fn create_room_message(
    db: &Database,
    room_id: String,
    user_id: String,
    message: String,
) -> ApiResult<RoomMessage> {
    let room_thing = room_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid room id".to_string()))?;
    
    let user_thing = user_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid user id".to_string()))?;

    // Verify user is in room
    let room = get_room(db, room_thing.clone()).await?;
    if !room.players.contains(&user_thing) {
        return Err(ApiError::Forbidden("User not in room".to_string()));
    }

    let room_message = RoomMessage {
        id: None,
        room_id: room_thing,
        user_id: user_thing,
        message,
        created_at: Datetime::default(),
    };

    let created: Option<RoomMessage> = db.create("room_message").content(room_message).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create message".to_string()))
}

pub async fn get_room_messages(db: &Database, room_id: String, limit: Option<u32>) -> ApiResult<Vec<RoomMessage>> {
    let room_thing = room_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid room id".to_string()))?;

    let messages: Vec<RoomMessage> = if let Some(l) = limit {
        db.query("SELECT * FROM room_message WHERE room_id = $room_id ORDER BY created_at DESC LIMIT $limit")
            .bind(("room_id", room_thing))
            .bind(("limit", l))
            .await?
            .take(0)?
    } else {
        db.query("SELECT * FROM room_message WHERE room_id = $room_id ORDER BY created_at DESC")
            .bind(("room_id", room_thing))
            .await?
            .take(0)?
    };
    Ok(messages)
}
