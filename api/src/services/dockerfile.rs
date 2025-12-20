use crate::{
    db::Database,
    error::{ApiError, ApiResult},
    models::Game,
};
use surrealdb::sql::{Datetime, Thing};

pub async fn upload_dockerfile(
    db: &Database,
    game_id: Thing,
    dockerfile_content: String,
) -> ApiResult<()> {
    // Update game record with dockerfile content
    let key = (game_id.tb.as_str(), game_id.id.to_string());
    let game: Option<Game> = db.select(key.clone()).await?;
    let mut game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    game.dockerfile = Some(dockerfile_content);
    game.updated_at = Datetime::default();

    let _: Option<Game> = db.update(key).content(game).await?;

    Ok(())
}

pub async fn get_dockerfile(db: &Database, game_id: Thing) -> ApiResult<String> {
    let key = (game_id.tb.as_str(), game_id.id.to_string());
    let game: Option<Game> = db.select(key).await?;
    let game = game.ok_or_else(|| ApiError::NotFound("Game not found".to_string()))?;

    game.dockerfile
        .ok_or_else(|| ApiError::NotFound("Dockerfile not found for this game".to_string()))
}
