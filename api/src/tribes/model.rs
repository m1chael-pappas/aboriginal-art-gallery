//! Domain models for the Tribes BC, including the PostGIS-backed territory
//! polygon returned to clients as GeoJSON.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

/// An Aboriginal tribe / language group. `territory` is a PostGIS
/// `geography(MultiPolygon, 4326)` column on the DB side, serialised to
/// GeoJSON on the way out.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Tribe {
    pub id: Uuid,
    /// Display name (e.g. "Pintupi"). Subject to a `UNIQUE` constraint -
    /// duplicate creates surface as 409 Conflict.
    pub name: String,
    /// Broad geographic descriptor (e.g. "Western Desert").
    pub region: Option<String>,
    /// Language family / group (e.g. "Yolngu Matha").
    pub language_group: Option<String>,
    pub description: Option<String>,
    /// GeoJSON geometry (MultiPolygon, EPSG:4326) of the tribe's traditional
    /// Country, or `null` if not set. Demo approximation, not authoritative.
    /// Managed via the dedicated `/tribes/{id}/territory` endpoints rather
    /// than the main PUT.
    #[schema(value_type = Object)]
    pub territory: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for `POST /tribes` and `PUT /tribes/{id}`. Territory is
/// intentionally excluded - it has its own endpoint pair.
#[derive(Debug, Deserialize, ToSchema)]
pub struct TribeInput {
    pub name: String,
    pub region: Option<String>,
    pub language_group: Option<String>,
    pub description: Option<String>,
}

impl TribeInput {
    /// Application-side validation. Mirrors the DB `NOT NULL` on `name`.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name cannot be empty".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! Pure-logic unit tests for [`TribeInput::validate`] - no async, no DB.

    use super::TribeInput;

    fn base() -> TribeInput {
        TribeInput {
            name: "Pintupi".into(),
            region: None,
            language_group: None,
            description: None,
        }
    }

    #[test]
    fn accepts_a_non_empty_name() {
        assert!(base().validate().is_ok());
    }

    #[test]
    fn rejects_empty_or_whitespace_name() {
        let mut t = base();
        t.name = String::new();
        assert!(t.validate().is_err());
        t.name = "   ".into();
        assert!(t.validate().is_err());
    }
}
