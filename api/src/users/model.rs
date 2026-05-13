//! Domain models and request/response shapes for the Users + auth surface.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::Role;

/// User as stored.
///
/// `password_hash` is held in the struct so the repo doesn't need a second
/// fetch on login, but `#[serde(skip_serializing)]` keeps it out of every
/// API response — clients only ever see the safe fields. The matching
/// `#[schema(write_only)]` flag advertises the same constraint in the
/// OpenAPI doc.
#[derive(Debug, Serialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    /// Login email. Stored as PostgreSQL `CITEXT` so case differences don't
    /// create duplicates, and subject to a DB CHECK that the value contains
    /// `@` and `.`.
    pub email: String,
    /// Argon2id PHC-encoded hash. Never serialised to clients; never read by
    /// the FE.
    #[serde(skip_serializing)]
    #[schema(value_type = String, write_only = true)]
    pub password_hash: String,
    /// Either `"User"` or `"Admin"`, enforced by a DB CHECK constraint.
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for `POST /auth/register`. New accounts always start with
/// `role = "User"`; promotion to `"Admin"` is a separate admin-only PUT.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterInput {
    #[schema(example = "alice@example.com")]
    pub email: String,
    #[schema(example = "correct-horse-battery-staple", min_length = 8)]
    pub password: String,
}

/// Request body for `POST /auth/login`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginInput {
    #[schema(example = "alice@example.com")]
    pub email: String,
    pub password: String,
}

/// Response from `/auth/register` and `/auth/login` — the issued JWT plus
/// the (sanitised) user so the FE can stash both without a second
/// `/auth/me` round-trip.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

/// Patch shape for `PUT /users/{id}`.
///
/// Every field is optional; the repo applies `COALESCE`, leaving omitted
/// fields untouched. `role` is admin-only — the handler enforces that before
/// calling the repo, so a regular user can change their own email/password
/// without being able to escalate.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<Role>,
}

impl RegisterInput {
    /// Reject empty/malformed email and short passwords client-side so we
    /// surface a 400 instead of a 500 from the DB CHECK.
    pub fn validate(&self) -> Result<(), String> {
        validate_email(&self.email)?;
        validate_password(&self.password)?;
        Ok(())
    }
}

impl LoginInput {
    /// Reject empty inputs without wasting an Argon2 verify.
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
    /// Validate only the fields that were actually provided.
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
