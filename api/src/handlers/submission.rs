use crate::{
    AppState,
    error::ApiResult,
    models::{
        Claims, CreateSubmissionRequest, ProgrammingLanguage, Submission, SubmissionResponse,
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

pub async fn create_submission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateSubmissionRequest>,
) -> ApiResult<(StatusCode, Json<SubmissionResponse>)> {
    payload
        .validate()
        .map_err(|e| crate::error::ApiError::Validation(e.to_string()))?;

    // Validate code size
    let max_bytes = state.config.app.max_code_size_mb * 1024 * 1024;
    if payload.code.len() > max_bytes {
        return Err(crate::error::ApiError::Validation(format!(
            "Code size exceeds maximum of {} MB",
            state.config.app.max_code_size_mb
        )));
    }

    // Validate language
    let language = ProgrammingLanguage::from_str(&payload.language).ok_or_else(|| {
        crate::error::ApiError::Validation("Invalid programming language".to_string())
    })?;
    // Get tournament to verify it exists and get game_id
    let tournament = services::tournament::get_tournament(
        &state.db,
        payload
            .tournament_id
            .parse()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
    )
    .await?;
    let game_id = tournament.game_id.clone();
    let game = services::game::get_game(&state.db, game_id.clone()).await?;
    if !game.supported_languages.contains(&language) && game.game_language != language {
        return Err(crate::error::ApiError::Validation(format!(
            "Language {:?} is not supported ({:?}, game_language: {:?}) by this game",
            language,
            game.supported_languages,
            game.game_language,
        )));
    }
    // Create submission
    let submission = services::submission::create_submission(
        &state.db,
        claims
            .sub
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?,
        payload
            .tournament_id
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid tournament id".to_string()))?,
        game_id,
        language,
        payload.code,
    )
    .await?;
    let response = SubmissionResponse {
        id: submission.id.as_ref().unwrap().id.to_string(),
        tournament_id: submission.tournament_id.id.to_string(),
        language: submission.language,
        status: submission.status,
        created_at: submission.created_at,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_submission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(submission_id): Path<String>,
) -> ApiResult<Json<Submission>> {
    let submission = services::submission::get_submission(
        &state.db,
        submission_id
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid submission id".to_string()))?,
    )
    .await?;
    // Check if user owns this submission
    if submission.user_id.to_string() != claims.sub {
        return Err(crate::error::ApiError::Forbidden(
            "You don't have access to this submission".to_string(),
        ));
    }
    Ok(Json(submission))
}

#[derive(Deserialize)]
pub struct ListSubmissionsQuery {
    tournament_id: Option<String>,
}

pub async fn list_submissions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListSubmissionsQuery>,
) -> ApiResult<Json<Vec<Submission>>> {
    let submissions = services::submission::list_user_submissions(
        &state.db,
        claims
            .sub
            .parse::<Thing>()
            .map_err(|_| crate::error::ApiError::BadRequest("Invalid user id".to_string()))?,
        query
            .tournament_id
            .as_deref()
            .map(|id| {
                id.parse::<Thing>().map_err(|_| {
                    crate::error::ApiError::BadRequest("Invalid tournament id".to_string())
                })
            })
            .transpose()?,
    )
    .await?;
    Ok(Json(submissions))
}
