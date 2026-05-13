//! Domain models for the Artifacts BC.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A single work in the collection — painting, bark, sculpture, ceremonial
/// piece. `artist_id` is required (every artifact has a maker); dimensions
/// and metadata are optional because not every record is fully catalogued.
#[derive(Debug, Serialize, ToSchema)]
pub struct Artifact {
    pub id: Uuid,
    pub title: String,
    /// FK to [`crate::artists::model::Artist`]; `ON DELETE RESTRICT`.
    pub artist_id: Uuid,
    /// Broad classification — e.g. "painting", "sculpture", "bark".
    pub art_type: Option<String>,
    /// Narrower stylistic label — e.g. "Western Desert", "Yirrkala bark".
    pub art_style: Option<String>,
    /// Materials and technique — e.g. "Synthetic polymer paint on canvas".
    pub medium: Option<String>,
    pub year_created: Option<i16>,
    /// Physical dimensions in centimetres. A DB CHECK enforces positivity.
    pub height_cm: Option<i16>,
    pub width_cm: Option<i16>,
    pub depth_cm: Option<i16>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for `POST /artifacts` and `PUT /artifacts/{id}`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ArtifactInput {
    pub title: String,
    pub artist_id: Uuid,
    pub art_type: Option<String>,
    pub art_style: Option<String>,
    pub medium: Option<String>,
    pub year_created: Option<i16>,
    pub height_cm: Option<i16>,
    pub width_cm: Option<i16>,
    pub depth_cm: Option<i16>,
    pub description: Option<String>,
}

impl ArtifactInput {
    /// Validate that title isn't blank and dimensions (when given) are
    /// positive. Mirrors the DB CHECK constraints; surfaces a useful message
    /// instead of letting SQLSTATE 23514 leak through as a generic 500.
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("title cannot be empty".into());
        }
        for (label, value) in [
            ("height_cm", self.height_cm),
            ("width_cm", self.width_cm),
            ("depth_cm", self.depth_cm),
        ] {
            if let Some(v) = value {
                if v <= 0 {
                    return Err(format!("{label} must be positive"));
                }
            }
        }
        Ok(())
    }
}
