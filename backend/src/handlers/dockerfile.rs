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
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct UploadDockerfileResponse {
    pub path: String,
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
    let game_id_clean = game_id.trim_start_matches("game:");
    let game = services::game::get_game(&state.db, game_id_clean).await?;

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

    let path = services::dockerfile::upload_dockerfile(
        &state.db,
        game_id_clean,
        payload.dockerfile_content,
    )
    .await?;

    Ok(Json(UploadDockerfileResponse {
        path,
        message: "Dockerfile uploaded successfully".to_string(),
    }))
}
