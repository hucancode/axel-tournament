use crate::db::Database;
use super::models::{MatchRecord, RoomRecord};
use serde::{Deserialize};
use serde_json;
use std::collections::HashMap;
use surrealdb::sql::{Datetime, Thing};

// ============================================================================
// Room Database Operations
// ============================================================================

/// Create a new room in the database
pub async fn create_room(
    db: &Database,
    game_id: String,
    host_id: Thing,
    name: String,
    max_players: u32,
    human_timeout_ms: Option<u64>,
) -> Result<RoomRecord, surrealdb::Error> {
    let now = Datetime::default();
    let room = RoomRecord {
        id: None,
        game_id,
        host_id: host_id.clone(),
        name,
        max_players,
        status: "waiting".to_string(),
        players: vec![host_id], // Host is automatically added as first player
        human_timeout_ms,
        created_at: now.clone(),
        updated_at: now,
        event_history: Vec::new(),
    };

    let created: Option<RoomRecord> = db.create("room").content(room).await?;
    created.ok_or_else(|| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(
            "Failed to create room".to_string(),
        ))
    })
}

/// Get a room from the database by ID
pub async fn get_room(
    db: &Database,
    room_id: &str,
) -> Result<Option<RoomRecord>, surrealdb::Error> {
    let room: Thing = room_id.parse().map_err(|_| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(
            "Invalid room ID format".to_string(),
        ))
    })?;
    let room: Option<RoomRecord> = db.select((room.tb.as_str(), room.id.to_string())).await?;
    Ok(room)
}

/// List rooms from the database, optionally filtered by game_id and status
pub async fn list_rooms(
    db: &Database,
    game_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<RoomRecord>, surrealdb::Error> {
    let mut query = "SELECT * FROM room".to_string();
    let mut conditions = Vec::new();

    if let Some(gid) = game_id {
        conditions.push(format!("game_id = '{}'", gid));
    }
    if let Some(s) = status {
        conditions.push(format!("status = '{}'", s));
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    query.push_str(" ORDER BY created_at DESC");

    let mut response = db.query(&query).await?;
    let rooms: Vec<RoomRecord> = response.take(0)?;
    Ok(rooms)
}

/// Update a room in the database
pub async fn update_room(
    db: &Database,
    room_id: &str,
    updates: HashMap<String, serde_json::Value>,
) -> Result<Option<RoomRecord>, surrealdb::Error> {
    let mut query = "UPDATE $room_id SET updated_at = time::now()".to_string();

    for (key, _) in &updates {
        query.push_str(&format!(", {} = ${}", key, key));
    }

    query.push_str(" RETURN AFTER");

    let room_thing: Thing = room_id.parse().map_err(|_| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(
            "Invalid room ID format".to_string(),
        ))
    })?;
    let mut db_query = db
        .query(&query)
        .bind(("room_id", room_thing));

    for (key, value) in updates {
        db_query = db_query.bind((key, value));
    }

    let mut response = db_query.await?;
    let updated: Vec<RoomRecord> = response.take(0)?;
    Ok(updated.into_iter().next())
}

/// Add a player to the room
pub async fn add_player(
    db: &Database,
    room_id: &str,
    player_id: Thing,
) -> Result<Option<RoomRecord>, surrealdb::Error> {
    let query =
        "UPDATE $room_id SET players += $player_id, updated_at = time::now() RETURN AFTER";
    let room_thing: Thing = room_id.parse().map_err(|_| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(
            "Invalid room ID format".to_string(),
        ))
    })?;
    let mut response = db
        .query(query)
        .bind(("room_id", room_thing))
        .bind(("player_id", player_id))
        .await?;

    let updated: Vec<RoomRecord> = response.take(0)?;
    Ok(updated.into_iter().next())
}

/// Remove a player from the room
pub async fn remove_player(
    db: &Database,
    room_id: &str,
    player_id: Thing,
) -> Result<Option<RoomRecord>, surrealdb::Error> {
    let query =
        "UPDATE $room_id SET players -= $player_id, updated_at = time::now() RETURN AFTER";
    let room_thing: Thing = room_id.parse().map_err(|_| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(
            "Invalid room ID format".to_string(),
        ))
    })?;
    let mut response = db
        .query(query)
        .bind(("room_id", room_thing))
        .bind(("player_id", player_id))
        .await?;

    let updated: Vec<RoomRecord> = response.take(0)?;
    Ok(updated.into_iter().next())
}

/// Delete a room from the database
pub async fn delete_room(db: &Database, room_id: &str) -> Result<(), surrealdb::Error> {
    let _: Option<RoomRecord> = db.delete(("room", room_id)).await?;
    Ok(())
}

/// Persist full room state to database (for major state changes)
pub async fn persist_room_state(
    db: &Database,
    room_id: &str,
    message_history: Vec<String>,
) -> Result<(), surrealdb::Error> {
    let query = "UPDATE $room_id SET
        event_history = $history,
        updated_at = time::now()";
    let room_thing: Thing = room_id.parse().map_err(|_| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(
            "Invalid room ID format".to_string(),
        ))
    })?;
    db.query(query)
        .bind(("room_id", room_thing))
        .bind(("history", message_history))
        .await?;
    Ok(())
}

/// Append single event to room history (optimized for frequent updates)
/// Uses JSON format to avoid escaping issues with | or \n in messages
pub async fn append_room_history(
    db: &Database,
    room_id: &str,
    message: &str,
) -> Result<(), surrealdb::Error> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    // Use JSON format to avoid escaping issues with | or \n in messages
    let entry = serde_json::json!({
        "timestamp": timestamp,
        "message": message,
    }).to_string();

    let query = "UPDATE $room_id SET
        event_history += $entry,
        updated_at = time::now()";

    let room_thing: Thing = room_id.parse().map_err(|_| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(
            "Invalid room ID format".to_string(),
        ))
    })?;
    db.query(query)
        .bind(("room_id", room_thing))
        .bind(("entry", entry))
        .await?;

    Ok(())
}

/// Get both room and match history atomically for reconnection
pub async fn get_room_and_match_history_atomic(
    db: &Database,
    room_id: &str,
) -> Result<(Vec<String>, Vec<String>), String> {
    // Single query to fetch both room and match history atomically
    let query = "
        LET $room_history = (SELECT event_history FROM room WHERE id = $room_id);
        LET $match_history = (SELECT game_event_source, created_at FROM match
                              WHERE room_id = $room_id AND status IN ['running', 'completed']
                              ORDER BY created_at DESC LIMIT 1);
        RETURN { room: $room_history[0].event_history, match: $match_history[0].game_event_source };
    ";

    let room_thing: Thing = room_id.parse().map_err(|_| "Invalid room ID format".to_string())?;
    let mut response = db
        .query(query)
        .bind(("room_id", room_thing))
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    #[derive(Deserialize)]
    struct HistoryResult {
        room: Option<Vec<String>>,
        #[serde(rename = "match")]
        match_history: Option<String>,
    }

    let result: Option<HistoryResult> = response.take(0).map_err(|e| format!("Parse error: {}", e))?;

    let room_history = if let Some(ref entries) = result.as_ref().and_then(|r| r.room.as_ref()) {
        entries
            .iter()
            .filter_map(|entry| {
                // Parse JSON format: {"timestamp": ..., "message": "..."}
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(entry) {
                    json.get("message").and_then(|m| m.as_str()).map(|s| s.to_string())
                } else {
                    tracing::warn!("Failed to parse room history entry as JSON: {}", entry);
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    let match_history = if let Some(ref history_str) = result.as_ref().and_then(|r| r.match_history.as_ref()) {
        history_str.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    Ok((room_history, match_history))
}

// ============================================================================
// Match Database Operations
// ============================================================================

/// Create a match record for a room game
pub async fn create_match(
    db: &Database,
    room_id: &str,
    game_id: &str,
    players: Vec<Thing>,
) -> Result<Thing, String> {
    use super::models::MatchParticipant;
    // Get hostname for judge_server_name
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());

    let participants: Vec<MatchParticipant> = players
        .iter()
        .map(|id| MatchParticipant {
            user_id: id.clone(),
            submission_id: None, // Interactive matches don't have submissions
            score: None,
        })
        .collect();

    let room_thing: Thing = room_id.parse().map_err(|_| "Invalid room ID format".to_string())?;
    let match_record = MatchRecord {
        id: None,
        tournament_id: None, // Interactive matches are not part of tournaments
        game_id: game_id.to_string(),
        status: "running".to_string(),
        participants,
        metadata: None,
        room_id: Some(room_thing),
        game_event_source: Some(String::new()), // Initialize empty game history
        judge_server_name: Some(hostname),
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
        started_at: Some(Datetime::default()),
        completed_at: None,
    };

    let created: Option<MatchRecord> = db
        .create("match")
        .content(match_record)
        .await
        .map_err(|e| format!("Failed to create match: {}", e))?;

    if let Some(m) = created {
        m.id.ok_or("Unable to create match".to_string())
    } else {
        Err("Unable to create match".to_string())
    }
}

/// Append game event to match history
pub async fn append_game_event(db: &Database, match_id: Thing, event: &str) -> Result<(), String> {
    let query = "UPDATE $match_id SET game_event_source = string::concat(game_event_source, $event, '\n'), updated_at = time::now()";

    db.query(query)
        .bind(("match_id", match_id))
        .bind(("event", event.to_string()))
        .await
        .map_err(|e| format!("Failed to append game event: {}", e))?;

    Ok(())
}

/// Complete a match record
pub async fn complete_match(db: &Database, match_id: Thing) -> Result<(), String> {
    let query = "UPDATE $match_id SET status = 'completed', completed_at = time::now(), updated_at = time::now()";

    db.query(query)
        .bind(("match_id", match_id))
        .await
        .map_err(|e| format!("Failed to complete match: {}", e))?;

    Ok(())
}
