use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Artifact {
    pub id: Uuid,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
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
