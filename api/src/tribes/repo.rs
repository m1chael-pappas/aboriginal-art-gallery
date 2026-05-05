use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Tribe, TribeInput};
use crate::error::{AppError, AppResult};

pub async fn list(pool: &PgPool) -> AppResult<Vec<Tribe>> {
    let tribes = sqlx::query_as!(
        Tribe,
        r#"
        SELECT id, name, region, language_group, description,
               created_at, updated_at
        FROM tribes
        ORDER BY name
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(tribes)
}

pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<Tribe> {
    sqlx::query_as!(
        Tribe,
        r#"
        SELECT id, name, region, language_group, description,
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

pub async fn create(pool: &PgPool, input: TribeInput) -> AppResult<Tribe> {
    sqlx::query_as!(
        Tribe,
        r#"
        INSERT INTO tribes (name, region, language_group, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, region, language_group, description,
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

pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query!("DELETE FROM tribes WHERE id = $1", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

// Postgres SQLSTATE 23505 = unique_violation. The only UNIQUE constraint on
// tribes is `name`, so collisions surface as a 409 Conflict with a
// human-readable hint instead of a generic 500.
fn map_unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23505") {
            return AppError::Conflict("a tribe with this name already exists".into());
        }
    }
    AppError::Database(err)
}
