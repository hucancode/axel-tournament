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
    rules: serde_json::Value,
    supported_languages: Vec<ProgrammingLanguage>,
    owner_id: Option<String>,
) -> ApiResult<Game> {
    let owner_thing = owner_id.map(|id| Thing::from(("user", id.trim_start_matches("user:"))));

    let game = Game {
        id: None,
        name,
        description,
        rules,
        supported_languages,
        is_active: true,
        owner_id: owner_thing,
        dockerfile_path: None,
        docker_image: None,
        game_code_path: None,
        game_language: None,
        created_at: Datetime::default(),
        updated_at: Datetime::default(),
    };
    let created: Option<Game> = db.create("game").content(game).await?;
    created.ok_or_else(|| ApiError::Internal("Failed to create game".to_string()))
}

pub async fn get_game(db: &Database, game_id: &str) -> ApiResult<Game> {
    let game: Option<Game> = db.select(("game", game_id)).await?;
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
    game_id: &str,
    name: Option<String>,
    description: Option<String>,
    rules: Option<serde_json::Value>,
    supported_languages: Option<Vec<ProgrammingLanguage>>,
    is_active: Option<bool>,
) -> ApiResult<Game> {
    let mut game = get_game(db, game_id).await?;
    if let Some(n) = name {
        game.name = n;
    }
    if let Some(d) = description {
        game.description = d;
    }
    if let Some(r) = rules {
        game.rules = r;
    }
    if let Some(sl) = supported_languages {
        game.supported_languages = sl;
    }
    if let Some(ia) = is_active {
        game.is_active = ia;
    }
    game.updated_at = Datetime::default();
    let updated: Option<Game> = db.update(("game", game_id)).content(game).await?;
    updated.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))
}

pub async fn delete_game(db: &Database, game_id: &str) -> ApiResult<()> {
    let _: Option<Game> = db.delete(("game", game_id)).await?;
    Ok(())
}

pub async fn list_games_by_owner(db: &Database, owner_id: &str) -> ApiResult<Vec<Game>> {
    let owner_thing = Thing::from(("user", owner_id.trim_start_matches("user:")));
    let mut result = db
        .query("SELECT * FROM game WHERE owner_id = $owner_id ORDER BY created_at DESC")
        .bind(("owner_id", owner_thing))
        .await?;
    let games: Vec<Game> = result.take(0)?;
    Ok(games)
}
