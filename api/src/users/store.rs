//! Persistence abstraction for the Users BC.
//!
//! [`routes`](super::routes) depends on the [`UserStore`] trait. Password
//! hashing and JWT issuance stay in the handler - the store only persists
//! the already-hashed value, exactly like the SQL repo it replaces.
//!
//! - [`PgUserStore`] - production, backed by compile-time-checked `sqlx`.
//!   `email` is `CITEXT`, selected via an explicit `::TEXT` cast so sqlx
//!   maps it to `String`.
//! - [`InMemoryUserStore`] - a `#[cfg(test)]` fake. It reproduces the
//!   case-insensitive uniqueness of `CITEXT email` and the `COALESCE`
//!   partial-update semantics so handler tests are faithful without a DB.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::model::User;
use crate::error::{AppError, AppResult};

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use chrono::Utc;

/// CRUD + login-lookup contract for users.
#[async_trait]
pub trait UserStore: Send + Sync {
    /// Every user, ordered by `email`.
    async fn list(&self) -> AppResult<Vec<User>>;
    /// One user by id, [`AppError::NotFound`] if absent.
    async fn find(&self, id: Uuid) -> AppResult<User>;
    /// Look up by login email (case-insensitive). `Ok(None)` for no match.
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    /// Insert a new user with an already-hashed password.
    async fn create(&self, email: &str, password_hash: &str, role: &str) -> AppResult<User>;
    /// `COALESCE` partial update - `None` leaves that column untouched. The
    /// caller pre-hashes the password and authorises any role change.
    async fn update(
        &self,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
    ) -> AppResult<User>;
    /// Delete a user by id.
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Production [`UserStore`] backed by Postgres via `sqlx`.
#[derive(Clone)]
pub struct PgUserStore {
    pool: PgPool,
}

impl PgUserStore {
    /// Wrap a connection pool. The pool is `Arc`-shaped, so this is cheap.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserStore for PgUserStore {
    async fn list(&self) -> AppResult<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, email::TEXT AS "email!", password_hash, role,
                   created_at, updated_at
            FROM users
            ORDER BY email
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }

    async fn find(&self, id: Uuid) -> AppResult<User> {
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
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
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
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn create(&self, email: &str, password_hash: &str, role: &str) -> AppResult<User> {
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
        .fetch_one(&self.pool)
        .await
        .map_err(map_unique_violation)
    }

    async fn update(
        &self,
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
        .fetch_optional(&self.pool)
        .await
        .map_err(map_unique_violation)?
        .ok_or(AppError::NotFound)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}

/// SQLSTATE 23505 = unique_violation. The only UNIQUE constraint on `users`
/// is `email`, so any 23505 here is a duplicate email - surface as 409.
fn map_unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23505") {
            return AppError::Conflict("email already in use".into());
        }
    }
    AppError::Database(err)
}

/// In-memory [`UserStore`] for unit tests. Reproduces case-insensitive email
/// uniqueness (`CITEXT`) and `COALESCE` partial-update semantics.
///
/// `#[cfg(test)]`: a test double, compiled only for the test build.
#[cfg(test)]
#[derive(Default)]
pub struct InMemoryUserStore {
    rows: Mutex<HashMap<Uuid, User>>,
}

#[cfg(test)]
impl InMemoryUserStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
#[async_trait]
impl UserStore for InMemoryUserStore {
    async fn list(&self) -> AppResult<Vec<User>> {
        let rows = self.rows.lock().expect("user store mutex poisoned");
        let mut users: Vec<User> = rows.values().cloned().collect();
        users.sort_by(|a, b| a.email.cmp(&b.email));
        Ok(users)
    }

    async fn find(&self, id: Uuid) -> AppResult<User> {
        self.rows
            .lock()
            .expect("user store mutex poisoned")
            .get(&id)
            .cloned()
            .ok_or(AppError::NotFound)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let needle = email.to_lowercase();
        Ok(self
            .rows
            .lock()
            .expect("user store mutex poisoned")
            .values()
            .find(|u| u.email.to_lowercase() == needle)
            .cloned())
    }

    async fn create(&self, email: &str, password_hash: &str, role: &str) -> AppResult<User> {
        let mut rows = self.rows.lock().expect("user store mutex poisoned");
        let needle = email.to_lowercase();
        if rows.values().any(|u| u.email.to_lowercase() == needle) {
            return Err(AppError::Conflict("email already in use".into()));
        }
        let now = Utc::now();
        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            role: role.to_string(),
            created_at: now,
            updated_at: now,
        };
        rows.insert(user.id, user.clone());
        Ok(user)
    }

    async fn update(
        &self,
        id: Uuid,
        email: Option<&str>,
        password_hash: Option<&str>,
        role: Option<&str>,
    ) -> AppResult<User> {
        let mut rows = self.rows.lock().expect("user store mutex poisoned");
        if let Some(new_email) = email {
            let needle = new_email.to_lowercase();
            if rows
                .values()
                .any(|u| u.id != id && u.email.to_lowercase() == needle)
            {
                return Err(AppError::Conflict("email already in use".into()));
            }
        }
        let existing = rows.get(&id).ok_or(AppError::NotFound)?;
        let updated = User {
            id,
            email: email.map(str::to_string).unwrap_or_else(|| existing.email.clone()),
            password_hash: password_hash
                .map(str::to_string)
                .unwrap_or_else(|| existing.password_hash.clone()),
            role: role.map(str::to_string).unwrap_or_else(|| existing.role.clone()),
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };
        rows.insert(id, updated.clone());
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        self.rows
            .lock()
            .expect("user store mutex poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    //! Store-contract unit tests against [`InMemoryUserStore`], no DB.

    use super::*;

    #[tokio::test]
    async fn create_then_find_and_find_by_email() {
        let store = InMemoryUserStore::new();
        let u = store
            .create("Alice@Example.com", "hash", "User")
            .await
            .unwrap();
        assert_eq!(store.find(u.id).await.unwrap().id, u.id);
        // CITEXT: lookup is case-insensitive.
        let by = store.find_by_email("alice@example.com").await.unwrap();
        assert_eq!(by.map(|x| x.id), Some(u.id));
    }

    #[tokio::test]
    async fn duplicate_email_is_conflict_case_insensitive() {
        let store = InMemoryUserStore::new();
        store.create("bob@example.com", "h", "User").await.unwrap();
        assert!(matches!(
            store.create("BOB@example.com", "h", "User").await.unwrap_err(),
            AppError::Conflict(_)
        ));
    }

    #[tokio::test]
    async fn update_is_partial_coalesce() {
        let store = InMemoryUserStore::new();
        let u = store
            .create("carol@example.com", "old-hash", "User")
            .await
            .unwrap();
        // Only role provided: email + password_hash must be untouched.
        let updated = store
            .update(u.id, None, None, Some("Admin"))
            .await
            .unwrap();
        assert_eq!(updated.role, "Admin");
        assert_eq!(updated.email, "carol@example.com");
        assert_eq!(updated.password_hash, "old-hash");
        assert_eq!(updated.created_at, u.created_at);
    }

    #[tokio::test]
    async fn update_to_an_existing_email_is_conflict() {
        let store = InMemoryUserStore::new();
        store.create("a@example.com", "h", "User").await.unwrap();
        let b = store.create("b@example.com", "h", "User").await.unwrap();
        assert!(matches!(
            store
                .update(b.id, Some("A@example.com"), None, None)
                .await
                .unwrap_err(),
            AppError::Conflict(_)
        ));
    }

    #[tokio::test]
    async fn missing_ids_are_not_found() {
        let store = InMemoryUserStore::new();
        let ghost = Uuid::new_v4();
        assert!(matches!(
            store.find(ghost).await.unwrap_err(),
            AppError::NotFound
        ));
        assert!(matches!(
            store.update(ghost, None, None, Some("User")).await.unwrap_err(),
            AppError::NotFound
        ));
        assert!(matches!(
            store.delete(ghost).await.unwrap_err(),
            AppError::NotFound
        ));
    }

    #[tokio::test]
    async fn find_by_email_returns_none_for_unknown() {
        let store = InMemoryUserStore::new();
        assert!(store.find_by_email("nobody@example.com").await.unwrap().is_none());
    }
}
