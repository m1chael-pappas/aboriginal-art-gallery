use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use super::{
    model::{Artist, ArtistInput},
    repo,
};
use crate::{
    auth::AdminUser,
    error::{AppError, AppResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/artists", get(list_artists).post(create_artist))
        .route(
            "/artists/{id}",
            get(get_artist).put(update_artist).delete(delete_artist),
        )
}

#[utoipa::path(
    get,
    path = "/artists",
    tag = "artists",
    responses(
        (status = 200, description = "All artists, ordered by display_name", body = Vec<Artist>),
    ),
)]
pub(crate) async fn list_artists(State(state): State<AppState>) -> AppResult<Json<Vec<Artist>>> {
    let artists = repo::list(&state.pool).await?;
    Ok(Json(artists))
}

#[utoipa::path(
    get,
    path = "/artists/{id}",
    tag = "artists",
    params(("id" = Uuid, Path, description = "Artist UUID")),
    responses(
        (status = 200, body = Artist),
        (status = 404, description = "No artist with that id"),
    ),
)]
pub(crate) async fn get_artist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Artist>> {
    let artist = repo::find(&state.pool, id).await?;
    Ok(Json(artist))
}

#[utoipa::path(
    post,
    path = "/artists",
    tag = "artists",
    request_body = ArtistInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, body = Artist),
        (status = 400, description = "Validation failed (empty name, invalid lifespan, unknown tribe_id)"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
    ),
)]
pub(crate) async fn create_artist(
    State(state): State<AppState>,
    _: AdminUser,
    Json(input): Json<ArtistInput>,
) -> AppResult<(StatusCode, Json<Artist>)> {
    input.validate().map_err(AppError::Validation)?;
    let artist = repo::create(&state.pool, input).await?;
    Ok((StatusCode::CREATED, Json(artist)))
}

#[utoipa::path(
    put,
    path = "/artists/{id}",
    tag = "artists",
    params(("id" = Uuid, Path, description = "Artist UUID")),
    request_body = ArtistInput,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = Artist),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No artist with that id"),
    ),
)]
pub(crate) async fn update_artist(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ArtistInput>,
) -> AppResult<Json<Artist>> {
    input.validate().map_err(AppError::Validation)?;
    let artist = repo::update(&state.pool, id, input).await?;
    Ok(Json(artist))
}

#[utoipa::path(
    delete,
    path = "/artists/{id}",
    tag = "artists",
    params(("id" = Uuid, Path, description = "Artist UUID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Artist deleted"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No artist with that id"),
        (status = 409, description = "Artist still referenced by artifacts (ON DELETE RESTRICT)"),
    ),
)]
pub(crate) async fn delete_artist(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    repo::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
