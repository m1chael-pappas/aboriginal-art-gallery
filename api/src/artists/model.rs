use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct Artist {
    pub id: Uuid,
    pub display_name: String,
    pub birth_year: Option<i16>,
    pub death_year: Option<i16>,
    pub region: Option<String>,
    pub biography: Option<String>,
    pub tribe_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
