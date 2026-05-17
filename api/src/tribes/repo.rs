//! Data access for the Tribes table, including the PostGIS round-trips for
//! the `territory` column.
//!
//! `territory` is a `geography(MultiPolygon, 4326)` - not a type sqlx knows
//! natively. We round-trip it through GeoJSON: `ST_AsGeoJSON` produces a
//! text representation that the `::JSONB` cast turns into a JSONB column,
//! which the `json` sqlx feature decodes straight into `serde_json::Value`.
//! The `?` suffix in the column alias forces nullability (the expression
//! isn't a base column, so sqlx can't infer it).

use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Tribe, TribeInput};
use crate::error::{AppError, AppResult};

/// All tribes ordered alphabetically by name, each with its territory as
/// GeoJSON (or null).
pub async fn list(pool: &PgPool) -> AppResult<Vec<Tribe>> {
    let tribes = sqlx::query_as!(
        Tribe,
        r#"
        SELECT id, name, region, language_group, description,
               ST_AsGeoJSON(territory)::JSONB AS "territory?: serde_json::Value",
               created_at, updated_at
        FROM tribes
        ORDER BY name
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(tribes)
}

/// Fetch a single tribe by id, or `AppError::NotFound`.
pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Tribe> {
    sqlx::query_as!(
        Tribe,
        r#"
        SELECT id, name, region, language_group, description,
               ST_AsGeoJSON(territory)::JSONB AS "territory?: serde_json::Value",
               created_at, updated_at
        FROM tribes
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Insert a new tribe. Duplicate names surface as 409 via
/// [`map_unique_violation`].
pub async fn create(pool: &PgPool, input: TribeInput) -> AppResult<Tribe> {
    sqlx::query_as!(
        Tribe,
        r#"
        INSERT INTO tribes (name, region, language_group, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, region, language_group, description,
                  ST_AsGeoJSON(territory)::JSONB AS "territory?: serde_json::Value",
                  created_at, updated_at
        "#,
        input.name,
        input.region,
        input.language_group,
        input.description,
    )
    .fetch_one(pool)
    .await
    .map_err(map_unique_violation)
}

/// Replace every field on an existing tribe (PUT semantics). Territory is
/// untouched - use [`set_territory`] for that.
pub async fn update(pool: &PgPool, id: Uuid, input: TribeInput) -> AppResult<Tribe> {
    sqlx::query_as!(
        Tribe,
        r#"
        UPDATE tribes
        SET name           = $2,
            region         = $3,
            language_group = $4,
            description    = $5
        WHERE id = $1
        RETURNING id, name, region, language_group, description,
                  ST_AsGeoJSON(territory)::JSONB AS "territory?: serde_json::Value",
                  created_at, updated_at
        "#,
        id,
        input.name,
        input.region,
        input.language_group,
        input.description,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_unique_violation)?
    .ok_or(AppError::NotFound)
}

/// Delete a tribe. `artists.tribe_id` is `ON DELETE SET NULL`, so deletes
/// don't cascade - affiliated artists just lose the link.
pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query!("DELETE FROM tribes WHERE id = $1", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Replace the tribe's territory with the supplied GeoJSON geometry, or
/// clear it if `geojson` is `None`.
///
/// `ST_GeomFromGeoJSON` parses the geometry, `ST_Multi` lifts a Polygon →
/// MultiPolygon so single-region tribes don't need to wrap their input
/// themselves, and the SRID-4326 cast aligns it with the column's declared
/// spatial reference. Malformed GeoJSON is reshaped into 400 via
/// [`map_geojson_parse_error`].
pub async fn set_territory(
    pool: &PgPool,
    id: Uuid,
    geojson: Option<&serde_json::Value>,
) -> AppResult<Tribe> {
    let geojson_text = geojson.map(|v| v.to_string());

    sqlx::query_as!(
        Tribe,
        r#"
        UPDATE tribes
        SET territory = CASE
            WHEN $2::TEXT IS NULL THEN NULL
            ELSE ST_Multi(ST_SetSRID(ST_GeomFromGeoJSON($2), 4326))::geography
        END
        WHERE id = $1
        RETURNING id, name, region, language_group, description,
                  ST_AsGeoJSON(territory)::JSONB AS "territory?: serde_json::Value",
                  created_at, updated_at
        "#,
        id,
        geojson_text,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_geojson_parse_error)?
    .ok_or(AppError::NotFound)
}

/// Return all tribes whose territory lies within `meters` of (`lng`, `lat`),
/// ordered by distance so the closest match comes first.
///
/// Uses `ST_DWithin` on the `geography` column, which goes through the GiST
/// index added in the territory migration - without that index this query
/// would full-scan and call `ST_Distance` on every row.
pub async fn search_near(
    pool: &PgPool,
    lng: f64,
    lat: f64,
    meters: f64,
) -> AppResult<Vec<Tribe>> {
    let tribes = sqlx::query_as!(
        Tribe,
        r#"
        SELECT id, name, region, language_group, description,
               ST_AsGeoJSON(territory)::JSONB AS "territory?: serde_json::Value",
               created_at, updated_at
        FROM tribes
        WHERE territory IS NOT NULL
          AND ST_DWithin(
                territory,
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
                $3
              )
        ORDER BY ST_Distance(
                   territory,
                   ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography
                 )
        "#,
        lng,
        lat,
        meters,
    )
    .fetch_all(pool)
    .await?;
    Ok(tribes)
}

/// Translate Postgres unique-violation (SQLSTATE 23505) on `tribes.name`
/// into a 409 with a human-readable hint, instead of letting it surface as
/// a generic 500.
fn map_unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23505") {
            return AppError::Conflict("a tribe with this name already exists".into());
        }
    }
    AppError::Database(err)
}

/// `ST_GeomFromGeoJSON` raises SQLSTATE XX000 with a helpful message when
/// the GeoJSON is malformed. Translate to 400 - the client's input is the
/// cause, not our server.
fn map_geojson_parse_error(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        let msg = db_err.message();
        if msg.contains("GeoJSON") || msg.contains("geojson") {
            return AppError::Validation(format!("invalid GeoJSON: {msg}"));
        }
    }
    AppError::Database(err)
}
