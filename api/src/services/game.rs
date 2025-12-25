use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{Game, ProgrammingLanguage, GameType},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn create_game(
    db: &Database,
    name: String,
    description: String,
    game_type: GameType,
    supported_languages: Vec<ProgrammingLanguage>,
    owner_id: String,
    game_code: String,
    game_language: ProgrammingLanguage,
    frontend_code: Option<String>,
    rounds_per_match: u32,
    repetitions: u32,
    timeout_ms: u32,
    cpu_limit: f64,
    turn_timeout_ms: u64,
    memory_limit_mb: u64,
) -> ApiResult<Game> {
    let owner_thing = owner_id
        .parse::<Thing>()
        .map_err(|_| ApiError::BadRequest("Invalid owner id".to_string()))?;

    let game = Game {
        id: None,
        name,
        description,
        game_type,
        supported_languages,
        is_active: true,
        owner_id: owner_thing,
        game_code,
        game_language,
        frontend_code,
        rounds_per_match,
        repetitions,
        timeout_ms,
        cpu_limit,
        turn_timeout_ms,
        memory_limit_mb,
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
    game_code: Option<String>,
    game_language: Option<ProgrammingLanguage>,
    rounds_per_match: Option<u32>,
    repetitions: Option<u32>,
    timeout_ms: Option<u32>,
    cpu_limit: Option<f64>,
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
    if let Some(code) = game_code {
        game.game_code = code;
    }
    if let Some(lang) = game_language {
        game.game_language = lang;
    }
    if let Some(rpm) = rounds_per_match {
        game.rounds_per_match = rpm;
    }
    if let Some(rep) = repetitions {
        game.repetitions = rep;
    }
    if let Some(ts) = timeout_ms {
        game.timeout_ms = ts;
    }
    if let Some(cpu) = cpu_limit {
        game.cpu_limit = cpu;
    }
    if let Some(ttm) = turn_timeout_ms {
        game.turn_timeout_ms = ttm;
    }
    if let Some(mlm) = memory_limit_mb {
        game.memory_limit_mb = mlm;
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
