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

async fn list_artifacts(State(state): State<AppState>) -> AppResult<Json<Vec<Artifact>>> {
    let artifacts = repo::list(&state.pool).await?;
    Ok(Json(artifacts))
}

async fn get_artifact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Artifact>> {
    let artifact = repo::find(&state.pool, id).await?;
    Ok(Json(artifact))
}

async fn create_artifact(
    State(state): State<AppState>,
    _: AdminUser,
    Json(input): Json<ArtifactInput>,
) -> AppResult<(StatusCode, Json<Artifact>)> {
    input.validate().map_err(AppError::Validation)?;
    let artifact = repo::create(&state.pool, input).await?;
    Ok((StatusCode::CREATED, Json(artifact)))
}

async fn update_artifact(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ArtifactInput>,
) -> AppResult<Json<Artifact>> {
    input.validate().map_err(AppError::Validation)?;
    let artifact = repo::update(&state.pool, id, input).await?;
    Ok(Json(artifact))
}

async fn delete_artifact(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    repo::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
