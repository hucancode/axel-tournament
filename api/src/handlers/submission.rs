use crate::{
    AppState,
    error::ApiResult,
    models::{Claims, CreateSubmissionRequest, ProgrammingLanguage, SubmissionResponse, rid},
    services,
};
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use surrealdb::types::{RecordId, ToSql};
use validator::Validate;

pub async fn create_submission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateSubmissionRequest>,
) -> ApiResult<(StatusCode, Json<SubmissionResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    let max_bytes = state.config.app.max_code_size_mb * 1024 * 1024;
    if payload.code.len() > max_bytes {
        return Err(crate::error::ApiError::Validation(format!(
            "Code size exceeds maximum of {} MB",
            state.config.app.max_code_size_mb
        )));
    }

    let language = ProgrammingLanguage::from_str(&payload.language).ok_or_else(|| {
        crate::error::ApiError::Validation("Invalid programming language".to_string())
    })?;
    let tournament_id = rid("tournament", payload.tournament_id);
    let tournament =
        services::tournament::get_tournament(&state.db, tournament_id.clone()).await?;
    let game_id = tournament.game_id.clone();

    let game = crate::models::game::find_game_by_id(&game_id)
        .ok_or_else(|| crate::error::ApiError::NotFound("Game not found".to_string()))?;

    if !game.supported_languages.contains(&language) {
        return Err(crate::error::ApiError::Validation(format!(
            "Language {:?} is not supported by this game. Supported languages: {:?}",
            language, game.supported_languages,
        )));
    }
    let submission = services::submission::create_submission(
        &state.db,
        RecordId::parse_simple(&claims.sub)
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?,
        tournament_id,
        game_id,
        language,
        payload.code,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(submission.into())))
}

pub async fn get_submission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(submission_id): Path<String>,
) -> ApiResult<Json<SubmissionResponse>> {
    let submission =
        services::submission::get_submission(&state.db, rid("submission", submission_id)).await?;
    if submission.user_id.to_sql() != claims.sub {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have access to this submission".to_string(),
        ));
    }
    Ok(Json(submission.into()))
}

#[derive(Deserialize)]
pub struct ListSubmissionsQuery {
    tournament_id: Option<String>,
}

pub async fn select_submission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(submission_id): Path<String>,
) -> ApiResult<Json<SubmissionResponse>> {
    let submission = services::submission::select_active_submission(
        &state.db,
        RecordId::parse_simple(&claims.sub)
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?,
        rid("submission", submission_id),
    )
    .await?;
    Ok(Json(submission.into()))
}

pub async fn submission_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(submission_id): Path<String>,
) -> ApiResult<Json<services::stats::SubmissionStats>> {
    let sid = rid("submission", submission_id);
    let submission = services::submission::get_submission(&state.db, sid.clone()).await?;
    if submission.user_id.to_sql() != claims.sub {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have access to this submission".to_string(),
        ));
    }
    let stats = services::stats::submission_stats(&state.db, sid).await?;
    Ok(Json(stats))
}

pub async fn list_submissions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListSubmissionsQuery>,
) -> ApiResult<Json<Vec<SubmissionResponse>>> {
    let submissions = services::submission::list_user_submissions(
        &state.db,
        RecordId::parse_simple(&claims.sub)
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?,
        query.tournament_id.map(|id| rid("tournament", id)),
    )
    .await?;
    Ok(Json(submissions.into_iter().map(Into::into).collect()))
}
