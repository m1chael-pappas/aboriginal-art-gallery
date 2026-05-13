//! Argon2id password hashing and constant-time verification.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::error::{AppError, AppResult};

/// Hash a plaintext password with Argon2id and the crate's default
/// parameters (m = 19456 KiB, t = 2, p = 1).
///
/// The returned string is a self-describing PHC encoding — algorithm,
/// params, salt, and hash all in one column — so the DB row has everything
/// [`verify_password`] needs to authenticate later.
pub fn hash_password(plain: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let hash = argon2
        .hash_password(plain.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2 hash failed: {e}")))?;

    Ok(hash.to_string())
}

/// Verify a plaintext password against a PHC-encoded hash.
///
/// `Ok(true)` means the password matches. `Ok(false)` means the hash parsed
/// fine but the password is wrong (a user-facing failure — return 401). The
/// only `Err` path is a *malformed* stored hash, which is a data-integrity
/// bug and bubbles up as a 500.
pub fn verify_password(plain: &str, phc_hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(phc_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("stored password hash is invalid: {e}")))?;

    match Argon2::default().verify_password(plain.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::Internal(anyhow::anyhow!(
            "argon2 verify failed: {e}"
        ))),
    }
}
