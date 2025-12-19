use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{Game, ProgrammingLanguage},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_game(
    db: &Database,
    name: String,
    description: String,
    supported_languages: Vec<ProgrammingLanguage>,
    owner_id: Option<String>,
    turn_timeout_ms: Option<u64>,
    memory_limit_mb: Option<u64>,
) -> ApiResult<Game> {
    let resolved_turn_timeout_ms = turn_timeout_ms.unwrap_or(2000);
    let resolved_memory_limit_mb = memory_limit_mb.unwrap_or(512);
    let owner_thing = owner_id
        .map(|id| {
            id.parse::<Thing>()
                .map_err(|_| ApiError::BadRequest("Invalid owner id".to_string()))
        })
        .transpose()?;

    let game = Game {
        id: None,
        name,
        description,
        supported_languages,
        is_active: true,
        owner_id: owner_thing,
        dockerfile: None,
        docker_image: None,
        game_code: None,
        game_language: None,
        turn_timeout_ms: Some(resolved_turn_timeout_ms),
        memory_limit_mb: Some(resolved_memory_limit_mb),
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };
    let created: Option<Game> = db.create("game").content(game).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create game".to_string()))
}

pub async fn get_game(db: &Database, game_id: Thing) -> ApiResult<Game> {
    let key = (game_id.tb.as_str(), game_id.id.to_string());
    let game: Option<Game> = db.select(key).await?;
    game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))
}

pub async fn list_games(db: &Database, active_only: bool) -> ApiResult<Vec<Game>> {
    let query = if active_only {
        "SELECT * FROM game WHERE is_active = true ORDER BY created_at DESC"
    } else {
        "SELECT * FROM game ORDER BY created_at DESC"
    };
    let mut result = db.query(query).await?;
    let games: Vec<Game> = result.take(0)?;
    Ok(games)
}

pub async fn update_game(
    db: &Database,
    game_id: Thing,
    name: Option<String>,
    description: Option<String>,
    supported_languages: Option<Vec<ProgrammingLanguage>>,
    is_active: Option<bool>,
    turn_timeout_ms: Option<u64>,
    memory_limit_mb: Option<u64>,
) -> ApiResult<Game> {
    let mut game = get_game(db, game_id.clone()).await?;
    if let Some(n) = name {
        game.name = n;
    }
    if let Some(d) = description {
        game.description = d;
    }
    if let Some(sl) = supported_languages {
        game.supported_languages = sl;
    }
    if let Some(ia) = is_active {
        game.is_active = ia;
    }
    if turn_timeout_ms.is_some() {
        game.turn_timeout_ms = turn_timeout_ms;
    }
    if memory_limit_mb.is_some() {
        game.memory_limit_mb = memory_limit_mb;
    }
    game.updated_at = Datetime::default();
    let key = (game_id.tb.as_str(), game_id.id.to_string());
    db.update(key)
        .content(game)
        .await?
        .ok_or_else(|| ApiError::NotFound("Game not found".to_string()))
}

pub async fn delete_game(db: &Database, game_id: Thing) -> ApiResult<()> {
    let key = (game_id.tb.as_str(), game_id.id.to_string());
    let _: Option<Game> = db.delete(key).await?;
    Ok(())
}

pub async fn list_games_by_owner(db: &Database, owner_id: Thing) -> ApiResult<Vec<Game>> {
    let mut result = db
        .query("SELECT * FROM game WHERE owner_id = $owner_id ORDER BY created_at DESC")
        .bind(("owner_id", owner_id))
        .await?;
    let games: Vec<Game> = result.take(0)?;
    Ok(games)
}
