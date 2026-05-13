use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use super::{
    model::{Artifact, ArtifactInput},
    repo,
};
use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/artifacts", get(list_artifacts).post(create_artifact))
        .route(
            "/artifacts/{id}",
            get(get_artifact)
                .put(update_artifact)
                .delete(delete_artifact),
        )
}

#[utoipa::path(
    get,
    path = "/artifacts",
    tag = "artifacts",
    responses(
        (status = 200, description = "All artifacts in the collection", body = Vec<Artifact>),
    ),
)]
pub(crate) async fn list_artifacts(State(state): State<AppState>) -> AppResult<Json<Vec<Artifact>>> {
    let artifacts = repo::list(&state.pool).await?;
    Ok(Json(artifacts))
}

#[utoipa::path(
    get,
    path = "/artifacts/{id}",
    tag = "artifacts",
    params(("id" = Uuid, Path, description = "Artifact UUID")),
    responses(
        (status = 200, body = Artifact),
        (status = 404, description = "No artifact with that id"),
    ),
)]
pub(crate) async fn get_artifact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Artifact>> {
    let artifact = repo::find(&state.pool, id).await?;
    Ok(Json(artifact))
}

#[utoipa::path(
    post,
    path = "/artifacts",
    tag = "artifacts",
    request_body = ArtifactInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, body = Artifact),
        (status = 400, description = "Validation failed (empty title, non-positive dimension, unknown artist_id)"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
    ),
)]
pub(crate) async fn create_artifact(
    State(state): State<AppState>,
    _: AdminUser,
    Json(input): Json<ArtifactInput>,
) -> AppResult<(StatusCode, Json<Artifact>)> {
    input.validate().map_err(AppError::Validation)?;
    let artifact = repo::create(&state.pool, input).await?;
    Ok((StatusCode::CREATED, Json(artifact)))
}

#[utoipa::path(
    put,
    path = "/artifacts/{id}",
    tag = "artifacts",
    params(("id" = Uuid, Path, description = "Artifact UUID")),
    request_body = ArtifactInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Artifact),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No artifact with that id"),
    ),
)]
pub(crate) async fn update_artifact(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ArtifactInput>,
) -> AppResult<Json<Artifact>> {
    input.validate().map_err(AppError::Validation)?;
    let artifact = repo::update(&state.pool, id, input).await?;
    Ok(Json(artifact))
}

#[utoipa::path(
    delete,
    path = "/artifacts/{id}",
    tag = "artifacts",
    params(("id" = Uuid, Path, description = "Artifact UUID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Artifact deleted"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No artifact with that id"),
    ),
)]
pub(crate) async fn delete_artifact(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    repo::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
