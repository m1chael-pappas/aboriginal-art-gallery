use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Artist, ArtistInput};
use crate::error::{AppError, AppResult};

pub async fn list(pool: &PgPool) -> AppResult<Vec<Artist>> {
    let artists = sqlx::query_as!(
        Artist,
        r#"
        SELECT id, display_name, birth_year, death_year, region, biography,
               tribe_id, created_at, updated_at
        FROM artists
        ORDER BY display_name
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(artists)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Artist> {
    sqlx::query_as!(
        Artist,
        r#"
        SELECT id, display_name, birth_year, death_year, region, biography,
               tribe_id, created_at, updated_at
        FROM artists
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn create(pool: &PgPool, input: ArtistInput) -> AppResult<Artist> {
    sqlx::query_as!(
        Artist,
        r#"
        INSERT INTO artists (display_name, birth_year, death_year, region,
                             biography, tribe_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, display_name, birth_year, death_year, region, biography,
                  tribe_id, created_at, updated_at
        "#,
        input.display_name,
        input.birth_year,
        input.death_year,
        input.region,
        input.biography,
        input.tribe_id,
    )
    .fetch_one(pool)
    .await
    .map_err(map_tribe_fk_violation)
}

pub async fn update(pool: &PgPool, id: Uuid, input: ArtistInput) -> AppResult<Artist> {
    sqlx::query_as!(
        Artist,
        r#"
        UPDATE artists
        SET display_name = $2,
            birth_year   = $3,
            death_year   = $4,
            region       = $5,
            biography    = $6,
            tribe_id     = $7
        WHERE id = $1
        RETURNING id, display_name, birth_year, death_year, region, biography,
                  tribe_id, created_at, updated_at
        "#,
        id,
        input.display_name,
        input.birth_year,
        input.death_year,
        input.region,
        input.biography,
        input.tribe_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_tribe_fk_violation)?
    .ok_or(AppError::NotFound)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query!("DELETE FROM artists WHERE id = $1", id)
        .execute(pool)
        .await
        .map_err(|err| {
            // Postgres SQLSTATE 23503 = foreign_key_violation. Here it means
            // artifacts still reference this artist (artifacts.artist_id is
            // ON DELETE RESTRICT). Surface as a 409 Conflict.
            //
            // Note: the symmetric incoming FK (artists.tribe_id) is ON DELETE
            // SET NULL, so deleting a tribe never gets here — no other 23503
            // path needs disambiguating.
            if let sqlx::Error::Database(db_err) = &err {
                if db_err.code().as_deref() == Some("23503") {
                    return AppError::Conflict(
                        "cannot delete artist while artifacts reference them".into(),
                    );
                }
            }
            AppError::Database(err)
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

// On create/update, a bad input.tribe_id is the only way to trigger an
// outgoing FK violation (artists.tribe_id -> tribes.id). Translate to
// AppError::Validation (400) — the user's payload referenced a tribe that
// doesn't exist.
fn map_tribe_fk_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23503") {
            return AppError::Validation("tribe_id: tribe not found".into());
        }
    }
    AppError::Database(err)
}
