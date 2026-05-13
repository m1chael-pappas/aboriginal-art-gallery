use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use super::{
    model::{Tribe, TribeInput},
    repo,
};
use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tribes", get(list_tribes).post(create_tribe))
        .route(
            "/tribes/{id}",
            get(get_tribe).put(update_tribe).delete(delete_tribe),
        )
}

async fn list_tribes(State(state): State<AppState>) -> AppResult<Json<Vec<Tribe>>> {
    let tribes = repo::list(&state.pool).await?;
    Ok(Json(tribes))
}

async fn get_tribe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Tribe>> {
    let tribe = repo::find(&state.pool, id).await?;
    Ok(Json(tribe))
}

async fn create_tribe(
    State(state): State<AppState>,
    _: AdminUser,
    Json(input): Json<TribeInput>,
) -> AppResult<(StatusCode, Json<Tribe>)> {
    input.validate().map_err(AppError::Validation)?;
    let tribe = repo::create(&state.pool, input).await?;
    Ok((StatusCode::CREATED, Json(tribe)))
}

async fn update_tribe(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<TribeInput>,
) -> AppResult<Json<Tribe>> {
    input.validate().map_err(AppError::Validation)?;
    let tribe = repo::update(&state.pool, id, input).await?;
    Ok(Json(tribe))
}

async fn delete_tribe(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    repo::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
