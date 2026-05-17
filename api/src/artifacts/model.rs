//! Domain models for the Artifacts BC.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A single work in the collection - painting, bark, sculpture, ceremonial
/// piece. `artist_id` is required (every artifact has a maker); dimensions
/// and metadata are optional because not every record is fully catalogued.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Artifact {
    pub id: Uuid,
    pub title: String,
    /// FK to [`crate::artists::model::Artist`]; `ON DELETE RESTRICT`.
    pub artist_id: Uuid,
    /// Broad classification - e.g. "painting", "sculpture", "bark".
    pub art_type: Option<String>,
    /// Narrower stylistic label - e.g. "Western Desert", "Yirrkala bark".
    pub art_style: Option<String>,
    /// Materials and technique - e.g. "Synthetic polymer paint on canvas".
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

#[cfg(test)]
mod tests {
    //! Pure-logic unit tests for [`ArtifactInput::validate`] - no async, no DB.

    use super::ArtifactInput;
    use uuid::Uuid;

    fn base() -> ArtifactInput {
        ArtifactInput {
            title: "Bush Plum Dreaming".into(),
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

    #[test]
    fn accepts_a_well_formed_input() {
        assert!(base().validate().is_ok());
    }

    #[test]
    fn rejects_blank_title() {
        let mut a = base();
        a.title = "   ".into();
        assert!(a.validate().is_err());
    }

    #[test]
    fn rejects_non_positive_dimensions() {
        for setter in [
            |a: &mut ArtifactInput| a.height_cm = Some(0),
            |a: &mut ArtifactInput| a.width_cm = Some(-5),
            |a: &mut ArtifactInput| a.depth_cm = Some(-1),
        ] {
            let mut a = base();
            setter(&mut a);
            assert!(a.validate().is_err());
        }
    }

    #[test]
    fn allows_positive_dimensions_and_omitted_ones() {
        let mut a = base();
        a.height_cm = Some(120);
        a.width_cm = Some(90);
        assert!(a.validate().is_ok());
    }
}
