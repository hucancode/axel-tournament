use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::{Game, ProgrammingLanguage},
};
use surrealdb::sql::{Datetime, Thing};

pub async fn upload_game_code(
    db: &Database,
    game_id: Thing,
    language: ProgrammingLanguage,
    code_content: String,
) -> ApiResult<()> {
    // Update game record with game code content
    let key = (game_id.tb.as_str(), game_id.id.to_string());
    let game: Option<Game> = db.select(key.clone()).await?;
    let mut game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    game.game_code = Some(code_content);
    game.game_language = Some(language);
    game.updated_at = Datetime::default();

    let _: Option<Game> = db.update(key).content(game).await?;

    Ok(())
}

pub async fn get_game_code(db: &Database, game_id: Thing) -> ApiResult<String> {
    let key = (game_id.tb.as_str(), game_id.id.to_string());
    let game: Option<Game> = db.select(key).await?;
    let game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    game.game_code
        .ok_or_else(|| ApiError::NotFound("Game code not found for this game".to_string()))
}
