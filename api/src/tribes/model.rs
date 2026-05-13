use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct Tribe {
    pub id: Uuid,
    pub name: String,
    pub region: Option<String>,
    pub language_group: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TribeInput {
    pub name: String,
    pub region: Option<String>,
    pub language_group: Option<String>,
    pub description: Option<String>,
}

impl TribeInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name cannot be empty".into());
        }
        Ok(())
    }
}
