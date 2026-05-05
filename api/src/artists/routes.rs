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

async fn list_artists(State(state): State<AppState>) -> AppResult<Json<Vec<Artist>>> {
    let artists = repo::list(&state.pool).await?;
    Ok(Json(artists))
}

async fn get_artist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Artist>> {
    let artist = repo::find(&state.pool, id).await?;
    Ok(Json(artist))
}

async fn create_artist(
    State(state): State<AppState>,
    Json(input): Json<ArtistInput>,
) -> AppResult<(StatusCode, Json<Artist>)> {
    input.validate().map_err(AppError::Validation)?;
    let artist = repo::create(&state.pool, input).await?;
    Ok((StatusCode::CREATED, Json(artist)))
}

async fn update_artist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<ArtistInput>,
) -> AppResult<Json<Artist>> {
    input.validate().map_err(AppError::Validation)?;
    let artist = repo::update(&state.pool, id, input).await?;
    Ok(Json(artist))
}

async fn delete_artist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    repo::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
