//! Data access for the Users table.
//!
//! `CITEXT` columns are selected via an explicit `::TEXT` cast so sqlx maps
//! them to `String` without needing a custom type — the cast is free at
//! runtime (CITEXT is binary-compatible with TEXT) and makes the macro's
//! type introspection happy on every Postgres version.

use sqlx::PgPool;
use uuid::Uuid;

use super::model::User;
use crate::error::{AppError, AppResult};

/// All users ordered alphabetically by email.
pub async fn list(pool: &PgPool) -> AppResult<Vec<User>> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, email::TEXT AS "email!", password_hash, role,
               created_at, updated_at
        FROM users
        ORDER BY email
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(users)
}

/// Fetch a single user by id, or `AppError::NotFound`.
pub async fn find(pool: &PgPool, id: Uuid) -> AppResult<User> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, email::TEXT AS "email!", password_hash, role,
               created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

/// Look up a user by login email. Returns `Ok(None)` for "no such user" —
/// the login handler turns that into a 401 with the same generic message
/// as a password mismatch.
pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email::TEXT AS "email!", password_hash, role,
               created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

/// Insert a new user with the supplied (already-hashed) password. Duplicate
/// emails surface as 409 via [`map_unique_violation`].
pub async fn create(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    role: &str,
) -> AppResult<User> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, role)
        VALUES ($1, $2, $3)
        RETURNING id, email::TEXT AS "email!", password_hash, role,
                  created_at, updated_at
        "#,
        email,
        password_hash,
        role,
    )
    .fetch_one(pool)
    .await
    .map_err(map_unique_violation)
}

/// Partial update via `COALESCE` — passing `None` for a column leaves it
/// untouched.
///
/// The caller is responsible for already-hashing the password and for
/// authorising the role change (regular users may not promote themselves).
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    email: Option<&str>,
    password_hash: Option<&str>,
    role: Option<&str>,
) -> AppResult<User> {
    sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET email         = COALESCE($2, email),
            password_hash = COALESCE($3, password_hash),
            role          = COALESCE($4, role)
        WHERE id = $1
        RETURNING id, email::TEXT AS "email!", password_hash, role,
                  created_at, updated_at
        "#,
        id,
        email,
        password_hash,
        role,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_unique_violation)?
    .ok_or(AppError::NotFound)
}

/// Delete a user. 404 if the id wasn't there.
pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// SQLSTATE 23505 = unique_violation. The only UNIQUE constraint on `users`
/// is `email`, so any 23505 here means a duplicate email — surface as 409.
fn map_unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23505") {
            return AppError::Conflict("email already in use".into());
        }
    }
    AppError::Database(err)
}
