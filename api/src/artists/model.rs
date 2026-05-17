//! Domain models for the Artists BC and the JSON shapes the HTTP layer uses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// An Aboriginal artist as stored in the gallery's archive. `tribe_id` is
/// nullable because the artist's affiliation may be unknown or unrecorded.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Artist {
    /// Stable UUIDv4 primary key.
    pub id: Uuid,
    /// Name we display in the UI - typically the artist's published name,
    /// not necessarily a legal name.
    pub display_name: String,
    /// Year of birth, or `null` if unknown.
    pub birth_year: Option<i16>,
    /// Year of death, or `null` if living/unknown. A DB CHECK constraint
    /// enforces `death_year >= birth_year` when both are set.
    pub death_year: Option<i16>,
    /// Free-text region descriptor (e.g. "Hermannsburg, Northern Territory").
    pub region: Option<String>,
    /// Biographical paragraph(s).
    pub biography: Option<String>,
    /// FK to [`crate::tribes::model::Tribe`]; `ON DELETE SET NULL`.
    pub tribe_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for `POST /artists` and the full-replacement `PUT /artists/{id}`.
/// Every field is optional except `display_name`; omitted optional fields
/// become `NULL` in the DB on PUT (replace, not patch).
#[derive(Debug, Deserialize, ToSchema)]
pub struct ArtistInput {
    pub display_name: String,
    pub birth_year: Option<i16>,
    pub death_year: Option<i16>,
    pub region: Option<String>,
    pub biography: Option<String>,
    pub tribe_id: Option<Uuid>,
}

impl ArtistInput {
    /// Application-side validation. Mirrors the DB CHECK constraints so the
    /// client gets a 400 with a useful message instead of a generic 500 from
    /// SQLSTATE 23514.
    pub fn validate(&self) -> Result<(), String> {
        if self.display_name.trim().is_empty() {
            return Err("display_name cannot be empty".into());
        }
        if let (Some(b), Some(d)) = (self.birth_year, self.death_year) {
            if d < b {
                return Err("death_year must be greater than or equal to birth_year".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! Pure-logic unit tests for [`ArtistInput::validate`] - no async, no
    //! database. The fast innermost layer of the test pyramid.

    use super::ArtistInput;

    fn base() -> ArtistInput {
        ArtistInput {
            display_name: "Albert Namatjira".into(),
            birth_year: None,
            death_year: None,
            region: None,
            biography: None,
            tribe_id: None,
        }
    }

    #[test]
    fn accepts_a_well_formed_input() {
        assert!(base().validate().is_ok());
    }

    #[test]
    fn rejects_empty_or_whitespace_display_name() {
        let mut a = base();
        a.display_name = String::new();
        assert!(a.validate().is_err());
        a.display_name = "   ".into();
        assert!(a.validate().is_err());
    }

    #[test]
    fn rejects_death_year_before_birth_year() {
        let mut a = base();
        a.birth_year = Some(1950);
        a.death_year = Some(1940);
        assert!(a.validate().is_err());
    }

    #[test]
    fn allows_equal_birth_and_death_year() {
        let mut a = base();
        a.birth_year = Some(1950);
        a.death_year = Some(1950);
        assert!(a.validate().is_ok());
    }

    #[test]
    fn allows_a_lone_birth_or_death_year() {
        let mut a = base();
        a.birth_year = Some(1902);
        assert!(a.validate().is_ok());
        let mut b = base();
        b.death_year = Some(1959);
        assert!(b.validate().is_ok());
    }
}
