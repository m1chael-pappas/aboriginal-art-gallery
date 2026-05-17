//! Persistence abstraction for the Artists BC.
//!
//! [`routes`](super::routes) depends on the [`ArtistStore`] trait, not on a
//! concrete database type — the same "program to an interface, inject the
//! implementation" layering used in the prior ASP.NET project, where
//! controllers depended on `IRobotCommandDataAccess` with swappable ADO/EF
//! backends. Two implementations are provided:
//!
//! - [`PgArtistStore`] — the production implementation, backed by
//!   compile-time-checked `sqlx` queries.
//! - [`InMemoryArtistStore`] — a faithful in-memory fake, so the store
//!   contract and the handlers can be unit-tested without a database.
//!
//! Cross-aggregate rules that physically live in the database (the
//! `tribe_id` foreign key, the `ON DELETE RESTRICT` from artifacts) are
//! deliberately *not* re-implemented by the in-memory fake — those are the
//! database's responsibility and are covered by the DB-backed integration
//! tests under `tests/`.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
use chrono::Utc;

use super::model::{Artist, ArtistInput};
use crate::error::{AppError, AppResult};

/// CRUD contract for artists. Object-safe via [`async_trait`], so it can be
/// held as `Arc<dyn ArtistStore>` in [`crate::state::AppState`] and swapped
/// for a fake in tests.
#[async_trait]
pub trait ArtistStore: Send + Sync {
    /// Every artist, ordered by `display_name`.
    async fn list(&self) -> AppResult<Vec<Artist>>;
    /// One artist by id, [`AppError::NotFound`] if absent.
    async fn find(&self, id: Uuid) -> AppResult<Artist>;
    /// Insert a new artist, returning the persisted row.
    async fn create(&self, input: ArtistInput) -> AppResult<Artist>;
    /// Replace every field on an existing artist (PUT semantics).
    async fn update(&self, id: Uuid, input: ArtistInput) -> AppResult<Artist>;
    /// Delete an artist by id.
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Production [`ArtistStore`] backed by Postgres via `sqlx`.
#[derive(Clone)]
pub struct PgArtistStore {
    pool: PgPool,
}

impl PgArtistStore {
    /// Wrap a connection pool. The pool is `Arc`-shaped, so this is cheap.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ArtistStore for PgArtistStore {
    async fn list(&self) -> AppResult<Vec<Artist>> {
        let artists = sqlx::query_as!(
            Artist,
            r#"
            SELECT id, display_name, birth_year, death_year, region, biography,
                   tribe_id, created_at, updated_at
            FROM artists
            ORDER BY display_name
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(artists)
    }

    async fn find(&self, id: Uuid) -> AppResult<Artist> {
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
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound)
    }

    async fn create(&self, input: ArtistInput) -> AppResult<Artist> {
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
        .fetch_one(&self.pool)
        .await
        .map_err(map_tribe_fk_violation)
    }

    async fn update(&self, id: Uuid, input: ArtistInput) -> AppResult<Artist> {
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
        .fetch_optional(&self.pool)
        .await
        .map_err(map_tribe_fk_violation)?
        .ok_or(AppError::NotFound)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query!("DELETE FROM artists WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|err| {
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
}

/// Translate an outgoing FK violation on create/update — i.e. the input
/// referenced a `tribe_id` that doesn't exist — into a 400 with a helpful
/// message, rather than letting raw SQLSTATE 23503 bubble up as a 500.
fn map_tribe_fk_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23503") {
            return AppError::Validation("tribe_id: tribe not found".into());
        }
    }
    AppError::Database(err)
}

/// In-memory [`ArtistStore`] for unit tests. Mirrors the *observable*
/// behaviour of [`PgArtistStore`] for single-aggregate operations: `create`
/// assigns a fresh UUID and timestamps, missing ids yield
/// [`AppError::NotFound`], `update` preserves `created_at`, and `list` is
/// ordered by `display_name`. It does not emulate database-enforced
/// cross-aggregate rules (see the module docs).
///
/// `#[cfg(test)]`: this is a test double, so it is compiled only for the
/// test build and never ships in the binary.
#[cfg(test)]
#[derive(Default)]
pub struct InMemoryArtistStore {
    rows: Mutex<HashMap<Uuid, Artist>>,
}

#[cfg(test)]
impl InMemoryArtistStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
#[async_trait]
impl ArtistStore for InMemoryArtistStore {
    async fn list(&self) -> AppResult<Vec<Artist>> {
        let rows = self.rows.lock().expect("artist store mutex poisoned");
        let mut artists: Vec<Artist> = rows.values().cloned().collect();
        artists.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        Ok(artists)
    }

    async fn find(&self, id: Uuid) -> AppResult<Artist> {
        self.rows
            .lock()
            .expect("artist store mutex poisoned")
            .get(&id)
            .cloned()
            .ok_or(AppError::NotFound)
    }

    async fn create(&self, input: ArtistInput) -> AppResult<Artist> {
        let now = Utc::now();
        let artist = Artist {
            id: Uuid::new_v4(),
            display_name: input.display_name,
            birth_year: input.birth_year,
            death_year: input.death_year,
            region: input.region,
            biography: input.biography,
            tribe_id: input.tribe_id,
            created_at: now,
            updated_at: now,
        };
        self.rows
            .lock()
            .expect("artist store mutex poisoned")
            .insert(artist.id, artist.clone());
        Ok(artist)
    }

    async fn update(&self, id: Uuid, input: ArtistInput) -> AppResult<Artist> {
        let mut rows = self.rows.lock().expect("artist store mutex poisoned");
        let created_at = rows.get(&id).ok_or(AppError::NotFound)?.created_at;
        let updated = Artist {
            id,
            display_name: input.display_name,
            birth_year: input.birth_year,
            death_year: input.death_year,
            region: input.region,
            biography: input.biography,
            tribe_id: input.tribe_id,
            created_at,
            updated_at: Utc::now(),
        };
        rows.insert(id, updated.clone());
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        self.rows
            .lock()
            .expect("artist store mutex poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    //! Store-contract unit tests. They exercise the [`ArtistStore`] contract
    //! against [`InMemoryArtistStore`] with no database — the unit/component
    //! layer beneath the DB-backed integration tests in `tests/artists.rs`.

    use super::*;

    fn input(name: &str) -> ArtistInput {
        ArtistInput {
            display_name: name.to_string(),
            birth_year: None,
            death_year: None,
            region: None,
            biography: None,
            tribe_id: None,
        }
    }

    #[tokio::test]
    async fn create_then_find_roundtrips() {
        let store = InMemoryArtistStore::new();
        let created = store.create(input("Emily Kame Kngwarreye")).await.unwrap();
        let found = store.find(created.id).await.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.display_name, "Emily Kame Kngwarreye");
    }

    #[tokio::test]
    async fn find_missing_id_is_not_found() {
        let store = InMemoryArtistStore::new();
        let err = store.find(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[tokio::test]
    async fn list_is_ordered_by_display_name() {
        let store = InMemoryArtistStore::new();
        store.create(input("Rover Thomas")).await.unwrap();
        store.create(input("Albert Namatjira")).await.unwrap();
        store.create(input("Clifford Possum")).await.unwrap();
        let names: Vec<_> = store
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|a| a.display_name)
            .collect();
        assert_eq!(names, ["Albert Namatjira", "Clifford Possum", "Rover Thomas"]);
    }

    #[tokio::test]
    async fn update_replaces_fields_and_preserves_created_at() {
        let store = InMemoryArtistStore::new();
        let created = store.create(input("Old Name")).await.unwrap();
        let mut next = input("New Name");
        next.birth_year = Some(1940);
        let updated = store.update(created.id, next).await.unwrap();
        assert_eq!(updated.display_name, "New Name");
        assert_eq!(updated.birth_year, Some(1940));
        assert_eq!(updated.created_at, created.created_at);
    }

    #[tokio::test]
    async fn update_missing_id_is_not_found() {
        let store = InMemoryArtistStore::new();
        let err = store.update(Uuid::new_v4(), input("x")).await.unwrap_err();
        assert!(matches!(err, AppError::NotFound));
    }

    #[tokio::test]
    async fn delete_removes_then_404s_on_second_delete() {
        let store = InMemoryArtistStore::new();
        let created = store.create(input("Temp")).await.unwrap();
        store.delete(created.id).await.unwrap();
        assert!(matches!(
            store.find(created.id).await.unwrap_err(),
            AppError::NotFound
        ));
        assert!(matches!(
            store.delete(created.id).await.unwrap_err(),
            AppError::NotFound
        ));
    }
}
