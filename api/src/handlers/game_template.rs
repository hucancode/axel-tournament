use crate::{
    AppState,
    error::ApiResult,
    models::{
        Claims, CreateGameTemplateRequest, GameTemplate, UpdateGameTemplateRequest, UserRole,
    },
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use surrealdb::sql::Thing;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    pub game_id: String,
}

pub async fn create_template(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateGameTemplateRequest>,
) -> ApiResult<(StatusCode, Json<GameTemplate>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    // Verify ownership or admin
    let game_id_thing: Thing = payload
        .game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    let game = services::game::get_game(&state.db, game_id_thing.clone()).await?;

    // Check if user owns the game or is admin
    let user_id = claims.sub.clone();
    let is_owner = game.owner_id.to_string() == user_id;

    if !is_owner && claims.role != UserRole::Admin {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have permission to create templates for this game".to_string(),
        ));
    }

    let template = services::game_template::create_template(
        &state.db,
        game_id_thing,
        payload.language,
        payload.template_code,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(template)))
}

pub async fn get_template(
    State(state): State<AppState>,
    Path((game_id, language)): Path<(String, String)>,
) -> ApiResult<Json<GameTemplate>> {
    let game_id: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    let template = services::game_template::get_template(&state.db, game_id, &language).await?;
    Ok(Json(template))
}

pub async fn list_templates(
    State(state): State<AppState>,
    Query(query): Query<ListTemplatesQuery>,
) -> ApiResult<Json<Vec<GameTemplate>>> {
    let templates = services::game_template::list_templates(
        &state.db,
        query
            .game_id
            .parse()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?,
    )
    .await?;
    Ok(Json(templates))
}

pub async fn update_template(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((game_id, language)): Path<(String, String)>,
    Json(payload): Json<UpdateGameTemplateRequest>,
) -> ApiResult<Json<GameTemplate>> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;
    let game_id_thing: Thing = game_id
        .parse()
        .map_err(|_| crate::error::ApiError::BadRequest("Invalid game id".to_string()))?;
    let game = services::game::get_game(&state.db, game_id_thing.clone()).await?;

    let user_id = claims.sub.clone();
    let is_owner = game.owner_id.to_string() == user_id;

    if !is_owner && claims.role != UserRole::Admin {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have permission to update templates for this game".to_string(),
        ));
    }

    let template = services::game_template::update_template(
        &state.db,
        game_id_thing,
        &language,
        payload.template_code,
    )
    .await?;

    Ok(Json(template))
}
