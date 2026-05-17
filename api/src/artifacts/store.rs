//! Persistence abstraction for the Artifacts BC.
//!
//! Same "program to an interface, inject the implementation" layering as the
//! Artists context: [`routes`](super::routes) depends on the
//! [`ArtifactStore`] trait, not a concrete database type.
//!
//! - [`PgArtifactStore`] - production, backed by compile-time-checked `sqlx`.
//! - [`InMemoryArtifactStore`] - a `#[cfg(test)]` fake for unit tests.
//!
//! The `artist_id` foreign key is enforced by the database, not by the
//! in-memory fake; that path is covered by the DB-backed integration tests
//! in `tests/artifacts.rs`.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Artifact, ArtifactInput};
use crate::error::{AppError, AppResult};

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use chrono::Utc;

/// CRUD contract for artifacts. Object-safe via [`async_trait`] so it can be
/// held as `Arc<dyn ArtifactStore>` in [`crate::state::AppState`].
#[async_trait]
pub trait ArtifactStore: Send + Sync {
    /// Every artifact, ordered by `title`.
    async fn list(&self) -> AppResult<Vec<Artifact>>;
    /// One artifact by id, [`AppError::NotFound`] if absent.
    async fn find(&self, id: Uuid) -> AppResult<Artifact>;
    /// Insert a new artifact, returning the persisted row.
    async fn create(&self, input: ArtifactInput) -> AppResult<Artifact>;
    /// Replace every field on an existing artifact (PUT semantics).
    async fn update(&self, id: Uuid, input: ArtifactInput) -> AppResult<Artifact>;
    /// Delete an artifact by id.
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

/// Production [`ArtifactStore`] backed by Postgres via `sqlx`.
#[derive(Clone)]
pub struct PgArtifactStore {
    pool: PgPool,
}

impl PgArtifactStore {
    /// Wrap a connection pool. The pool is `Arc`-shaped, so this is cheap.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ArtifactStore for PgArtifactStore {
    async fn list(&self) -> AppResult<Vec<Artifact>> {
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
        .fetch_all(&self.pool)
        .await?;
        Ok(artifacts)
    }

    async fn find(&self, id: Uuid) -> AppResult<Artifact> {
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
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound)
    }

    async fn create(&self, input: ArtifactInput) -> AppResult<Artifact> {
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
        .fetch_one(&self.pool)
        .await
        .map_err(map_artist_fk_violation)
    }

    async fn update(&self, id: Uuid, input: ArtifactInput) -> AppResult<Artifact> {
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
        .fetch_optional(&self.pool)
        .await
        .map_err(map_artist_fk_violation)?
        .ok_or(AppError::NotFound)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query!("DELETE FROM artifacts WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}

/// Translate an FK violation on `artist_id` into a 400 with a clear message,
/// instead of letting raw SQLSTATE 23503 surface as a generic 500.
fn map_artist_fk_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23503") {
            return AppError::Validation("artist_id: artist not found".into());
        }
    }
    AppError::Database(err)
}

/// In-memory [`ArtifactStore`] for unit tests. Mirrors the observable
/// behaviour of [`PgArtifactStore`] for single-aggregate operations; it does
/// not emulate the `artist_id` foreign key (see the module docs).
///
/// `#[cfg(test)]`: a test double, compiled only for the test build.
#[cfg(test)]
#[derive(Default)]
pub struct InMemoryArtifactStore {
    rows: Mutex<HashMap<Uuid, Artifact>>,
}

#[cfg(test)]
impl InMemoryArtifactStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
#[async_trait]
impl ArtifactStore for InMemoryArtifactStore {
    async fn list(&self) -> AppResult<Vec<Artifact>> {
        let rows = self.rows.lock().expect("artifact store mutex poisoned");
        let mut artifacts: Vec<Artifact> = rows.values().cloned().collect();
        artifacts.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(artifacts)
    }

    async fn find(&self, id: Uuid) -> AppResult<Artifact> {
        self.rows
            .lock()
            .expect("artifact store mutex poisoned")
            .get(&id)
            .cloned()
            .ok_or(AppError::NotFound)
    }

    async fn create(&self, input: ArtifactInput) -> AppResult<Artifact> {
        let now = Utc::now();
        let artifact = Artifact {
            id: Uuid::new_v4(),
            title: input.title,
            artist_id: input.artist_id,
            art_type: input.art_type,
            art_style: input.art_style,
            medium: input.medium,
            year_created: input.year_created,
            height_cm: input.height_cm,
            width_cm: input.width_cm,
            depth_cm: input.depth_cm,
            description: input.description,
            created_at: now,
            updated_at: now,
        };
        self.rows
            .lock()
            .expect("artifact store mutex poisoned")
            .insert(artifact.id, artifact.clone());
        Ok(artifact)
    }

    async fn update(&self, id: Uuid, input: ArtifactInput) -> AppResult<Artifact> {
        let mut rows = self.rows.lock().expect("artifact store mutex poisoned");
        let created_at = rows.get(&id).ok_or(AppError::NotFound)?.created_at;
        let updated = Artifact {
            id,
            title: input.title,
            artist_id: input.artist_id,
            art_type: input.art_type,
            art_style: input.art_style,
            medium: input.medium,
            year_created: input.year_created,
            height_cm: input.height_cm,
            width_cm: input.width_cm,
            depth_cm: input.depth_cm,
            description: input.description,
            created_at,
            updated_at: Utc::now(),
        };
        rows.insert(id, updated.clone());
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        self.rows
            .lock()
            .expect("artifact store mutex poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    //! Store-contract unit tests against [`InMemoryArtifactStore`], no DB.

    use super::*;

    fn input(title: &str) -> ArtifactInput {
        ArtifactInput {
            title: title.to_string(),
            artist_id: Uuid::new_v4(),
            art_type: None,
            art_style: None,
            medium: None,
            year_created: None,
            height_cm: None,
            width_cm: None,
            depth_cm: None,
            description: None,
        }
    }

    #[tokio::test]
    async fn create_then_find_roundtrips() {
        let store = InMemoryArtifactStore::new();
        let created = store.create(input("Water Dreaming")).await.unwrap();
        let found = store.find(created.id).await.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.title, "Water Dreaming");
    }

    #[tokio::test]
    async fn find_missing_id_is_not_found() {
        let store = InMemoryArtifactStore::new();
        assert!(matches!(
            store.find(Uuid::new_v4()).await.unwrap_err(),
            AppError::NotFound
        ));
    }

    #[tokio::test]
    async fn list_is_ordered_by_title() {
        let store = InMemoryArtifactStore::new();
        store.create(input("Untitled")).await.unwrap();
        store.create(input("Bush Tucker")).await.unwrap();
        store.create(input("Mina Mina")).await.unwrap();
        let titles: Vec<_> = store
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|a| a.title)
            .collect();
        assert_eq!(titles, ["Bush Tucker", "Mina Mina", "Untitled"]);
    }

    #[tokio::test]
    async fn update_replaces_fields_and_preserves_created_at() {
        let store = InMemoryArtifactStore::new();
        let created = store.create(input("Old Title")).await.unwrap();
        let mut next = input("New Title");
        next.medium = Some("Ochre on bark".into());
        let updated = store.update(created.id, next).await.unwrap();
        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.medium.as_deref(), Some("Ochre on bark"));
        assert_eq!(updated.created_at, created.created_at);
    }

    #[tokio::test]
    async fn update_missing_id_is_not_found() {
        let store = InMemoryArtifactStore::new();
        assert!(matches!(
            store.update(Uuid::new_v4(), input("x")).await.unwrap_err(),
            AppError::NotFound
        ));
    }

    #[tokio::test]
    async fn delete_removes_then_404s_on_second_delete() {
        let store = InMemoryArtifactStore::new();
        let created = store.create(input("Temp")).await.unwrap();
        store.delete(created.id).await.unwrap();
        assert!(matches!(
            store.delete(created.id).await.unwrap_err(),
            AppError::NotFound
        ));
    }
}
