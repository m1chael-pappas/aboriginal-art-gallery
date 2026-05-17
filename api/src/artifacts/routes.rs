//! HTTP handlers for the Artifacts BC. Reads are public; writes require an
//! Admin-role JWT.
//!
//! Handlers talk to the [`ArtifactStore`](super::store::ArtifactStore) trait
//! object in [`AppState`], never a concrete database type.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use super::model::{Artifact, ArtifactInput};
use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    state::AppState,
};

/// Build the `/artifacts` sub-router.
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

/// `GET /artifacts` - every artifact in the collection, ordered by title.
#[utoipa::path(
    get,
    path = "/artifacts",
    tag = "artifacts",
    responses(
        (status = 200, description = "All artifacts in the collection", body = Vec<Artifact>),
    ),
)]
pub(crate) async fn list_artifacts(State(state): State<AppState>) -> AppResult<Json<Vec<Artifact>>> {
    let artifacts = state.artifacts.list().await?;
    Ok(Json(artifacts))
}

/// `GET /artifacts/{id}` - one artifact by id.
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
    let artifact = state.artifacts.find(id).await?;
    Ok(Json(artifact))
}

/// `POST /artifacts` - admin-only create.
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
    let artifact = state.artifacts.create(input).await?;
    Ok((StatusCode::CREATED, Json(artifact)))
}

/// `PUT /artifacts/{id}` - admin-only full replacement.
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
    let artifact = state.artifacts.update(id, input).await?;
    Ok(Json(artifact))
}

/// `DELETE /artifacts/{id}` - admin-only.
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
    state.artifacts.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
