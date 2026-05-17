//! Persistence abstraction for the Tribes BC, including the PostGIS
//! round-trips for the `territory` column and the spatial "near a point"
//! query.
//!
//! [`routes`](super::routes) depends on the [`TribeStore`] trait.
//!
//! - [`PgTribeStore`] - production. `territory` is a
//!   `geography(MultiPolygon, 4326)`; it round-trips through GeoJSON
//!   (`ST_AsGeoJSON` on the way out, `ST_GeomFromGeoJSON`/`ST_Multi` on the
//!   way in), and `search_near` uses `ST_DWithin` over the GiST index.
//! - [`InMemoryTribeStore`] - a `#[cfg(test)]` fake for the CRUD/territory
//!   contract. It deliberately does **not** reproduce PostGIS spatial maths:
//!   `search_near` returns every tribe that has a territory, ignoring the
//!   radius. Spatial correctness is the database's job and is asserted by
//!   the DB-backed tests in `tests/territory.rs`.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Tribe, TribeInput};
use crate::error::{AppError, AppResult};

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use chrono::Utc;

/// CRUD + territory + spatial-search contract for tribes.
#[async_trait]
pub trait TribeStore: Send + Sync {
    /// Every tribe, ordered by `name`, territory as GeoJSON (or null).
    async fn list(&self) -> AppResult<Vec<Tribe>>;
    /// One tribe by id, [`AppError::NotFound`] if absent.
    async fn find(&self, id: Uuid) -> AppResult<Tribe>;
    /// Insert a new tribe (territory starts unset).
    async fn create(&self, input: TribeInput) -> AppResult<Tribe>;
    /// Replace name/region/language_group/description; territory untouched.
    async fn update(&self, id: Uuid, input: TribeInput) -> AppResult<Tribe>;
    /// Delete a tribe by id.
    async fn delete(&self, id: Uuid) -> AppResult<()>;
    /// Set (`Some`) or clear (`None`) the tribe's territory from GeoJSON.
    async fn set_territory(
        &self,
        id: Uuid,
        geojson: Option<&serde_json::Value>,
    ) -> AppResult<Tribe>;
    /// Tribes whose territory lies within `meters` of (`lng`, `lat`),
    /// nearest first.
    async fn search_near(&self, lng: f64, lat: f64, meters: f64) -> AppResult<Vec<Tribe>>;
}

/// Production [`TribeStore`] backed by Postgres + PostGIS via `sqlx`.
#[derive(Clone)]
pub struct PgTribeStore {
    pool: PgPool,
}

impl PgTribeStore {
    /// Wrap a connection pool. The pool is `Arc`-shaped, so this is cheap.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TribeStore for PgTribeStore {
    async fn list(&self) -> AppResult<Vec<Tribe>> {
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
        .fetch_all(&self.pool)
        .await?;
        Ok(tribes)
    }

    async fn find(&self, id: Uuid) -> AppResult<Tribe> {
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
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound)
    }

    async fn create(&self, input: TribeInput) -> AppResult<Tribe> {
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
        .fetch_one(&self.pool)
        .await
        .map_err(map_unique_violation)
    }

    async fn update(&self, id: Uuid, input: TribeInput) -> AppResult<Tribe> {
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
        .fetch_optional(&self.pool)
        .await
        .map_err(map_unique_violation)?
        .ok_or(AppError::NotFound)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query!("DELETE FROM tribes WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    async fn set_territory(
        &self,
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
        .fetch_optional(&self.pool)
        .await
        .map_err(map_geojson_parse_error)?
        .ok_or(AppError::NotFound)
    }

    async fn search_near(&self, lng: f64, lat: f64, meters: f64) -> AppResult<Vec<Tribe>> {
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
        .fetch_all(&self.pool)
        .await?;
        Ok(tribes)
    }
}

/// Translate Postgres unique-violation (SQLSTATE 23505) on `tribes.name`
/// into a 409, instead of letting it surface as a generic 500.
fn map_unique_violation(err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23505") {
            return AppError::Conflict("a tribe with this name already exists".into());
        }
    }
    AppError::Database(err)
}

/// Map a failure from the `set_territory` query to the right status.
///
/// The only way that query's `UPDATE ... ST_Multi(ST_SetSRID(
/// ST_GeomFromGeoJSON($2), ...))` can raise a *database* error is PostGIS
/// refusing the supplied geometry: malformed GeoJSON, an unknown `type`, or
/// a non-polygon shape. A non-existent tribe id is not an error here - the
/// row simply doesn't match and `fetch_optional` yields `Ok(None)`, which
/// the caller turns into 404. So any `Database` error on this path is the
/// client's input and maps to 400; everything else (pool exhaustion, IO) is
/// a genuine 500. We surface PostGIS's own message so the caller learns what
/// was wrong with their geometry, without sniffing for version-specific
/// substrings.
fn map_geojson_parse_error(err: sqlx::Error) -> AppError {
    match &err {
        sqlx::Error::Database(db_err) => {
            AppError::Validation(format!("invalid GeoJSON: {}", db_err.message()))
        }
        _ => AppError::Database(err),
    }
}

/// In-memory [`TribeStore`] for unit tests. Faithful for CRUD + territory
/// set/clear; `search_near` is intentionally non-spatial (see module docs).
///
/// `#[cfg(test)]`: a test double, compiled only for the test build.
#[cfg(test)]
#[derive(Default)]
pub struct InMemoryTribeStore {
    rows: Mutex<HashMap<Uuid, Tribe>>,
}

#[cfg(test)]
impl InMemoryTribeStore {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
#[async_trait]
impl TribeStore for InMemoryTribeStore {
    async fn list(&self) -> AppResult<Vec<Tribe>> {
        let rows = self.rows.lock().expect("tribe store mutex poisoned");
        let mut tribes: Vec<Tribe> = rows.values().cloned().collect();
        tribes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tribes)
    }

    async fn find(&self, id: Uuid) -> AppResult<Tribe> {
        self.rows
            .lock()
            .expect("tribe store mutex poisoned")
            .get(&id)
            .cloned()
            .ok_or(AppError::NotFound)
    }

    async fn create(&self, input: TribeInput) -> AppResult<Tribe> {
        let mut rows = self.rows.lock().expect("tribe store mutex poisoned");
        if rows.values().any(|t| t.name == input.name) {
            return Err(AppError::Conflict(
                "a tribe with this name already exists".into(),
            ));
        }
        let now = Utc::now();
        let tribe = Tribe {
            id: Uuid::new_v4(),
            name: input.name,
            region: input.region,
            language_group: input.language_group,
            description: input.description,
            territory: None,
            created_at: now,
            updated_at: now,
        };
        rows.insert(tribe.id, tribe.clone());
        Ok(tribe)
    }

    async fn update(&self, id: Uuid, input: TribeInput) -> AppResult<Tribe> {
        let mut rows = self.rows.lock().expect("tribe store mutex poisoned");
        if rows
            .values()
            .any(|t| t.id != id && t.name == input.name)
        {
            return Err(AppError::Conflict(
                "a tribe with this name already exists".into(),
            ));
        }
        let existing = rows.get(&id).ok_or(AppError::NotFound)?;
        let updated = Tribe {
            id,
            name: input.name,
            region: input.region,
            language_group: input.language_group,
            description: input.description,
            // PUT leaves territory untouched, like the SQL UPDATE.
            territory: existing.territory.clone(),
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };
        rows.insert(id, updated.clone());
        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        self.rows
            .lock()
            .expect("tribe store mutex poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or(AppError::NotFound)
    }

    async fn set_territory(
        &self,
        id: Uuid,
        geojson: Option<&serde_json::Value>,
    ) -> AppResult<Tribe> {
        let mut rows = self.rows.lock().expect("tribe store mutex poisoned");
        let existing = rows.get(&id).ok_or(AppError::NotFound)?;
        let updated = Tribe {
            territory: geojson.cloned(),
            updated_at: Utc::now(),
            ..existing.clone()
        };
        rows.insert(id, updated.clone());
        Ok(updated)
    }

    async fn search_near(&self, _lng: f64, _lat: f64, _meters: f64) -> AppResult<Vec<Tribe>> {
        // Non-spatial fake: every tribe that has a territory. Distance/radius
        // correctness is only guaranteed by PgTribeStore (tests/territory.rs).
        let rows = self.rows.lock().expect("tribe store mutex poisoned");
        Ok(rows
            .values()
            .filter(|t| t.territory.is_some())
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    //! Store-contract unit tests against [`InMemoryTribeStore`], no DB.

    use super::*;
    use serde_json::json;

    fn input(name: &str) -> TribeInput {
        TribeInput {
            name: name.to_string(),
            region: None,
            language_group: None,
            description: None,
        }
    }

    #[tokio::test]
    async fn create_then_find_roundtrips() {
        let store = InMemoryTribeStore::new();
        let created = store.create(input("Pintupi")).await.unwrap();
        let found = store.find(created.id).await.unwrap();
        assert_eq!(found.name, "Pintupi");
        assert!(found.territory.is_none());
    }

    #[tokio::test]
    async fn duplicate_name_is_conflict() {
        let store = InMemoryTribeStore::new();
        store.create(input("Yolngu")).await.unwrap();
        assert!(matches!(
            store.create(input("Yolngu")).await.unwrap_err(),
            AppError::Conflict(_)
        ));
    }

    #[tokio::test]
    async fn list_is_ordered_by_name() {
        let store = InMemoryTribeStore::new();
        store.create(input("Warlpiri")).await.unwrap();
        store.create(input("Anmatyerre")).await.unwrap();
        store.create(input("Tiwi")).await.unwrap();
        let names: Vec<_> = store
            .list()
            .await
            .unwrap()
            .into_iter()
            .map(|t| t.name)
            .collect();
        assert_eq!(names, ["Anmatyerre", "Tiwi", "Warlpiri"]);
    }

    #[tokio::test]
    async fn update_preserves_territory_and_created_at() {
        let store = InMemoryTribeStore::new();
        let created = store.create(input("Old")).await.unwrap();
        let geom = json!({"type": "Point", "coordinates": [131.0, -23.0]});
        store.set_territory(created.id, Some(&geom)).await.unwrap();

        let updated = store.update(created.id, input("Renamed")).await.unwrap();
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.territory.as_ref(), Some(&geom));
        assert_eq!(updated.created_at, created.created_at);
    }

    #[tokio::test]
    async fn set_then_clear_territory() {
        let store = InMemoryTribeStore::new();
        let created = store.create(input("Arrernte")).await.unwrap();
        let geom = json!({"type": "Point", "coordinates": [133.8, -23.7]});

        let with = store.set_territory(created.id, Some(&geom)).await.unwrap();
        assert_eq!(with.territory.as_ref(), Some(&geom));

        let without = store.set_territory(created.id, None).await.unwrap();
        assert!(without.territory.is_none());
    }

    #[tokio::test]
    async fn missing_ids_are_not_found() {
        let store = InMemoryTribeStore::new();
        let ghost = Uuid::new_v4();
        assert!(matches!(
            store.find(ghost).await.unwrap_err(),
            AppError::NotFound
        ));
        assert!(matches!(
            store.update(ghost, input("x")).await.unwrap_err(),
            AppError::NotFound
        ));
        assert!(matches!(
            store.delete(ghost).await.unwrap_err(),
            AppError::NotFound
        ));
        assert!(matches!(
            store.set_territory(ghost, None).await.unwrap_err(),
            AppError::NotFound
        ));
    }
}
