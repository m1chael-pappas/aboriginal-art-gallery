use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Artist, ArtistInput};
use crate::error::{AppError, AppResult};

pub async fn list(pool: &PgPool) -> AppResult<Vec<Artist>> {
    let artists = sqlx::query_as!(
        Artist,
        r#"
        SELECT id, display_name, birth_year, death_year, region, biography,
               created_at, updated_at
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
               created_at, updated_at
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
    let artist = sqlx::query_as!(
        Artist,
        r#"
        INSERT INTO artists (display_name, birth_year, death_year, region, biography)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, display_name, birth_year, death_year, region, biography,
                  created_at, updated_at
        "#,
        input.display_name,
        input.birth_year,
        input.death_year,
        input.region,
        input.biography,
    )
    .fetch_one(pool)
    .await?;
    Ok(artist)
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
            biography    = $6
        WHERE id = $1
        RETURNING id, display_name, birth_year, death_year, region, biography,
                  created_at, updated_at
        "#,
        id,
        input.display_name,
        input.birth_year,
        input.death_year,
        input.region,
        input.biography,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query!("DELETE FROM artists WHERE id = $1", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
