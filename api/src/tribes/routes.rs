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

#[utoipa::path(
    get,
    path = "/tribes",
    tag = "tribes",
    responses(
        (status = 200, description = "All tribes, ordered by name", body = Vec<Tribe>),
    ),
)]
pub(crate) async fn list_tribes(State(state): State<AppState>) -> AppResult<Json<Vec<Tribe>>> {
    let tribes = repo::list(&state.pool).await?;
    Ok(Json(tribes))
}

#[utoipa::path(
    get,
    path = "/tribes/{id}",
    tag = "tribes",
    params(("id" = Uuid, Path, description = "Tribe UUID")),
    responses(
        (status = 200, body = Tribe),
        (status = 404, description = "No tribe with that id"),
    ),
)]
pub(crate) async fn get_tribe(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Tribe>> {
    let tribe = repo::find(&state.pool, id).await?;
    Ok(Json(tribe))
}

#[utoipa::path(
    post,
    path = "/tribes",
    tag = "tribes",
    request_body = TribeInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, body = Tribe),
        (status = 400, description = "Validation failed (empty name)"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 409, description = "Tribe name already exists"),
    ),
)]
pub(crate) async fn create_tribe(
    State(state): State<AppState>,
    _: AdminUser,
    Json(input): Json<TribeInput>,
) -> AppResult<(StatusCode, Json<Tribe>)> {
    input.validate().map_err(AppError::Validation)?;
    let tribe = repo::create(&state.pool, input).await?;
    Ok((StatusCode::CREATED, Json(tribe)))
}

#[utoipa::path(
    put,
    path = "/tribes/{id}",
    tag = "tribes",
    params(("id" = Uuid, Path, description = "Tribe UUID")),
    request_body = TribeInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Tribe),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No tribe with that id"),
        (status = 409, description = "Tribe name already exists"),
    ),
)]
pub(crate) async fn update_tribe(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<TribeInput>,
) -> AppResult<Json<Tribe>> {
    input.validate().map_err(AppError::Validation)?;
    let tribe = repo::update(&state.pool, id, input).await?;
    Ok(Json(tribe))
}

#[utoipa::path(
    delete,
    path = "/tribes/{id}",
    tag = "tribes",
    params(("id" = Uuid, Path, description = "Tribe UUID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Tribe deleted (artists.tribe_id set to NULL)"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No tribe with that id"),
    ),
)]
pub(crate) async fn delete_tribe(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    repo::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
