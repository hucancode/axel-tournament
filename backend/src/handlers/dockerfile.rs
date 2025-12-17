use crate::{
    AppState,
    error::ApiResult,
    models::{Claims, UploadDockerfileRequest, UserRole},
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use serde::Serialize;
use surrealdb::sql::Thing;
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct UploadDockerfileResponse {
    pub message: String,
}

pub async fn upload_dockerfile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(game_id): Path<String>,
    Json(payload): Json<UploadDockerfileRequest>,
) -> ApiResult<Json<UploadDockerfileResponse>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    // Verify ownership
    let game_thing: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    let game = services::game::get_game(&state.db, game_thing.clone()).await?;

    let user_id = claims.sub.clone();
    let is_owner = game
        .owner_id
        .as_ref()
        .map(|owner| owner.to_string() == user_id)
        .unwrap_or(false);

    if !is_owner && claims.role != UserRole::Admin {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have permission to upload Dockerfile for this game".to_string(),
        ));
    }

    services::dockerfile::upload_dockerfile(
        &state.db,
        game_thing,
        payload.dockerfile_content,
    )
    .await?;

    Ok(Json(UploadDockerfileResponse {
        message: "Dockerfile uploaded successfully".to_string(),
    }))
}
