//! JWT issuance and verification - HS256, with `sub` / `email` / `role` /
//! `iat` / `exp` claims and a 24-hour TTL.

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Token lifetime. Short enough that compromised tokens don't linger
/// indefinitely; the demo is fine with 24h.
const TOKEN_TTL_HOURS: i64 = 24;

/// Roles the app understands. Stored in the DB as plain strings (with a
/// CHECK constraint) and embedded in the JWT claim of the same name.
/// Keeping it as an enum on the Rust side gives us exhaustiveness when
/// matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum Role {
    User,
    Admin,
}

impl Role {
    /// String form used both in the DB column and on the wire (matches the
    /// default serde representation).
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "User",
            Role::Admin => "Admin",
        }
    }

    /// Parse the string form back into the enum. An unrecognised value is a
    /// data-integrity bug (the DB CHECK should have rejected it), so it
    /// surfaces as a 500.
    pub fn from_db_str(s: &str) -> AppResult<Self> {
        match s {
            "User" => Ok(Role::User),
            "Admin" => Ok(Role::Admin),
            other => Err(AppError::Internal(anyhow::anyhow!(
                "unexpected role in DB: {other}"
            ))),
        }
    }
}

/// JWT payload. `sub` is the user id; `exp` and `iat` are Unix timestamps
/// (the jsonwebtoken crate auto-checks `exp` during decode by default).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: Role,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    /// Build a fresh `Claims` set for `user_id`, with `iat` = now and
    /// `exp` = now + 24h (see `TOKEN_TTL_HOURS`).
    pub fn new(user_id: Uuid, email: String, role: Role) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            email,
            role,
            iat: now.timestamp(),
            exp: (now + Duration::hours(TOKEN_TTL_HOURS)).timestamp(),
        }
    }
}

/// Encoding + decoding keys derived from `JWT_SECRET`.
///
/// The keys are internally `Arc`-shaped, so cloning the wrapper across
/// requests is cheap - stash it in [`crate::state::AppState`] once at
/// startup.
#[derive(Clone)]
pub struct JwtSecret {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtSecret {
    /// Read the secret from the `JWT_SECRET` env var. Rejects anything
    /// shorter than 32 bytes so HS256's key length advice is enforced at
    /// startup rather than per-token.
    pub fn from_env() -> anyhow::Result<Self> {
        let secret = std::env::var("JWT_SECRET")
            .map_err(|_| anyhow::anyhow!("JWT_SECRET must be set (see .env.example)"))?;
        if secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 bytes for HS256");
        }
        Ok(Self::from_bytes(secret.as_bytes()))
    }

    /// Build keys directly from a byte slice. Exposed for the test harness;
    /// production code should go through [`Self::from_env`].
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(bytes),
            decoding: DecodingKey::from_secret(bytes),
        }
    }

    /// Encode and sign a fresh JWT for `claims`.
    pub fn encode(&self, claims: &Claims) -> AppResult<String> {
        encode(&Header::default(), claims, &self.encoding)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("jwt encode failed: {e}")))
    }

    /// Decode and verify a bearer token. Returns the parsed claims on
    /// success; collapses every failure mode (bad signature, expired,
    /// malformed) into `AppError::Unauthorized` so the FE just sees a 401.
    pub fn decode(&self, token: &str) -> AppResult<Claims> {
        decode::<Claims>(token, &self.decoding, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))
    }
}
