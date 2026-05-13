//! Data access for the Artifacts table.

use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Artifact, ArtifactInput};
use crate::error::{AppError, AppResult};

/// All artifacts ordered alphabetically by title.
pub async fn list(pool: &PgPool) -> AppResult<Vec<Artifact>> {
    let artifacts = sqlx::query_as!(
        Artifact,
        r#"
        SELECT id, title, artist_id, art_type, art_style, medium,
               year_created, height_cm, width_cm, depth_cm, description,
               created_at, updated_at
        FROM artifacts
        ORDER BY title
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(artifacts)
}

/// Fetch a single artifact by id, or `AppError::NotFound`.
pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Artifact> {
    sqlx::query_as!(
        Artifact,
        r#"
        SELECT id, title, artist_id, art_type, art_style, medium,
               year_created, height_cm, width_cm, depth_cm, description,
               created_at, updated_at
        FROM artifacts
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Insert a new artifact. A bad `artist_id` is translated into 400 via
/// [`map_artist_fk_violation`].
pub async fn create(pool: &PgPool, input: ArtifactInput) -> AppResult<Artifact> {
    sqlx::query_as!(
        Artifact,
        r#"
        INSERT INTO artifacts (title, artist_id, art_type, art_style, medium,
                               year_created, height_cm, width_cm, depth_cm,
                               description)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, title, artist_id, art_type, art_style, medium,
                  year_created, height_cm, width_cm, depth_cm, description,
                  created_at, updated_at
        "#,
        input.title,
        input.artist_id,
        input.art_type,
        input.art_style,
        input.medium,
        input.year_created,
        input.height_cm,
        input.width_cm,
        input.depth_cm,
        input.description,
    )
    .fetch_one(pool)
    .await
    .map_err(map_artist_fk_violation)
}

/// Replace every field on an existing artifact.
pub async fn update(pool: &PgPool, id: Uuid, input: ArtifactInput) -> AppResult<Artifact> {
    sqlx::query_as!(
        Artifact,
        r#"
        UPDATE artifacts
        SET title         = $2,
            artist_id     = $3,
            art_type      = $4,
            art_style     = $5,
            medium        = $6,
            year_created  = $7,
            height_cm     = $8,
            width_cm      = $9,
            depth_cm      = $10,
            description   = $11
        WHERE id = $1
        RETURNING id, title, artist_id, art_type, art_style, medium,
                  year_created, height_cm, width_cm, depth_cm, description,
                  created_at, updated_at
        "#,
        id,
        input.title,
        input.artist_id,
        input.art_type,
        input.art_style,
        input.medium,
        input.year_created,
        input.height_cm,
        input.width_cm,
        input.depth_cm,
        input.description,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_artist_fk_violation)?
    .ok_or(AppError::NotFound)
}

/// Delete an artifact. 404 if the id wasn't there.
pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query!("DELETE FROM artifacts WHERE id = $1", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Translate an FK violation on `artist_id` into a 400 with a clear message,
/// instead of letting SQLSTATE 23503 surface as a generic 500.
fn map_artist_fk_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23503") {
            return AppError::Validation("artist_id: artist not found".into());
        }
    }
    AppError::Database(err)
}
