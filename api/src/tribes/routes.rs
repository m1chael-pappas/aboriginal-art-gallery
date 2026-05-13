//! HTTP handlers for the Tribes BC, including the territory and
//! spatial-search endpoints backed by PostGIS.
//!
//! Reads (including `/tribes/near`) are public; writes — both the regular
//! CRUD and the territory mutation endpoints — require an Admin-role JWT.

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, put},
};
use serde::Deserialize;
use serde_json::Value;
use utoipa::IntoParams;
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

/// Build the `/tribes` sub-router.
///
/// Literal route segments (`/tribes/near`) are declared before the
/// parameterised one (`/tribes/{id}`) so the intent is obvious. axum's
/// matcher already prefers literals when both could match, but readers
/// shouldn't need to know that to understand the route table.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tribes/near", get(search_near))
        .route("/tribes", get(list_tribes).post(create_tribe))
        .route(
            "/tribes/{id}",
            get(get_tribe).put(update_tribe).delete(delete_tribe),
        )
        .route(
            "/tribes/{id}/territory",
            put(set_territory).delete(clear_territory),
        )
}

/// `GET /tribes` — every tribe in the archive, ordered by name.
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

/// `GET /tribes/{id}` — one tribe with its territory (if set) as GeoJSON.
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

/// `POST /tribes` — admin-only create.
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

/// `PUT /tribes/{id}` — admin-only full replacement. Territory is preserved
/// — use the dedicated endpoints to change it.
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

/// `DELETE /tribes/{id}` — admin-only. Affiliated artists have their
/// `tribe_id` set to NULL (`ON DELETE SET NULL`).
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

/// `PUT /tribes/{id}/territory` — admin-only. Body is a GeoJSON Polygon or
/// MultiPolygon in EPSG:4326; Polygon is auto-lifted to MultiPolygon.
#[utoipa::path(
    put,
    path = "/tribes/{id}/territory",
    tag = "tribes",
    params(("id" = Uuid, Path, description = "Tribe UUID")),
    request_body(
        content = Object,
        description = "GeoJSON Polygon or MultiPolygon geometry (EPSG:4326)",
        example = json!({
            "type": "Polygon",
            "coordinates": [[
                [129.0, -25.0], [134.0, -25.0],
                [134.0, -20.0], [129.0, -20.0],
                [129.0, -25.0]
            ]]
        })
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Tribe with new territory included as GeoJSON", body = Tribe),
        (status = 400, description = "Invalid or non-polygon GeoJSON"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No tribe with that id"),
    ),
)]
pub(crate) async fn set_territory(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
    Json(geometry): Json<Value>,
) -> AppResult<Json<Tribe>> {
    let tribe = repo::set_territory(&state.pool, id, Some(&geometry)).await?;
    Ok(Json(tribe))
}

/// `DELETE /tribes/{id}/territory` — admin-only. Sets the territory column
/// back to NULL; the tribe itself stays.
#[utoipa::path(
    delete,
    path = "/tribes/{id}/territory",
    tag = "tribes",
    params(("id" = Uuid, Path, description = "Tribe UUID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Territory cleared"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No tribe with that id"),
    ),
)]
pub(crate) async fn clear_territory(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    repo::set_territory(&state.pool, id, None).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Query parameters for `GET /tribes/near` — a point + a radius in kilometres.
#[derive(Debug, Deserialize, IntoParams)]
pub(crate) struct NearQuery {
    /// Latitude in WGS-84 degrees, range `[-90, 90]`.
    #[param(example = -22.5)]
    pub lat: f64,
    /// Longitude in WGS-84 degrees, range `[-180, 180]`.
    #[param(example = 130.0)]
    pub lng: f64,
    /// Search radius in kilometres. Defaults to 100 if omitted.
    #[param(example = 250.0)]
    pub km: Option<f64>,
}

/// `GET /tribes/near?lat=&lng=&km=` — public spatial query. Returns tribes
/// whose territory lies within `km` of the point, nearest first. Powered by
/// PostGIS `ST_DWithin` over the GiST index on `tribes.territory`.
#[utoipa::path(
    get,
    path = "/tribes/near",
    tag = "tribes",
    params(NearQuery),
    responses(
        (status = 200, description = "Tribes whose territory lies within `km` of (lat, lng), nearest first", body = Vec<Tribe>),
        (status = 400, description = "Invalid coordinates or radius"),
    ),
)]
pub(crate) async fn search_near(
    State(state): State<AppState>,
    Query(q): Query<NearQuery>,
) -> AppResult<Json<Vec<Tribe>>> {
    if !(-90.0..=90.0).contains(&q.lat) {
        return Err(AppError::Validation("lat must be in [-90, 90]".into()));
    }
    if !(-180.0..=180.0).contains(&q.lng) {
        return Err(AppError::Validation("lng must be in [-180, 180]".into()));
    }
    let km = q.km.unwrap_or(100.0);
    if km <= 0.0 {
        return Err(AppError::Validation("km must be positive".into()));
    }

    let tribes = repo::search_near(&state.pool, q.lng, q.lat, km * 1000.0).await?;
    Ok(Json(tribes))
}
