use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::Role;

/// User as stored. `password_hash` is held in the struct so the repo doesn't
/// need a second fetch on login, but `#[serde(skip_serializing)]` keeps it
/// out of every API response — clients only ever see the safe fields.
#[derive(Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

/// What we send back from `/auth/register` and `/auth/login` — the issued
/// JWT plus the (sanitised) user so the FE can stash both without a second
/// `/auth/me` round-trip.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

/// Patch shape for `PUT /users/:id`. Every field is optional; the repo applies
/// `COALESCE`, leaving omitted fields untouched. `role` is admin-only — the
/// handler enforces that before calling the repo.
#[derive(Debug, Deserialize)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
}

impl RegisterInput {
    pub fn validate(&self) -> Result<(), String> {
        validate_email(&self.email)?;
        validate_password(&self.password)?;
        Ok(())
    }
}

impl LoginInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.email.trim().is_empty() {
            return Err("email cannot be empty".into());
        }
        if self.password.is_empty() {
            return Err("password cannot be empty".into());
        }
        Ok(())
    }
}

impl UserUpdate {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(email) = &self.email {
            validate_email(email)?;
        }
        if let Some(password) = &self.password {
            validate_password(password)?;
        }
        Ok(())
    }
}

fn validate_email(email: &str) -> Result<(), String> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Err("email cannot be empty".into());
    }
    // Mirror the DB's CHECK so we fail fast with a useful message instead of
    // letting a 23514 (check_violation) bubble up as a generic 500.
    if !(trimmed.contains('@') && trimmed.contains('.')) {
        return Err("email must look like 'name@host.tld'".into());
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("password must be at least 8 characters".into());
    }
    Ok(())
}
