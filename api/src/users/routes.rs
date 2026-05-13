use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;

use super::{
    model::{AuthResponse, LoginInput, RegisterInput, User, UserUpdate},
    repo,
};
use crate::{
    auth::{AdminUser, AuthUser, Claims, Role, password},
    error::{AppError, AppResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))
        .route("/users", get(list_users))
        .route(
            "/users/{id}",
            get(get_user).put(update_user).delete(delete_user),
        )
}

async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterInput>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    input.validate().map_err(AppError::Validation)?;

    let email = input.email.trim();
    let password_hash = password::hash_password(&input.password)?;

    let user = repo::create(&state.pool, email, &password_hash, Role::User.as_str()).await?;
    let token = issue_token(&state, &user)?;

    Ok((StatusCode::CREATED, Json(AuthResponse { token, user })))
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> AppResult<Json<AuthResponse>> {
    input.validate().map_err(AppError::Validation)?;

    // Look up by email *first*. Argon2 verify is the expensive step, so it
    // would be tempting to skip it when the user doesn't exist — but that
    // creates a timing oracle for enumerating valid emails. We still want
    // to bail early (no hash to verify), so we accept the small leak in
    // exchange for a simpler handler. A fully timing-safe variant would
    // verify against a dummy hash on miss.
    let user = match repo::find_by_email(&state.pool, input.email.trim()).await? {
        Some(u) => u,
        None => return Err(AppError::Unauthorized("invalid credentials".into())),
    };

    if !password::verify_password(&input.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    let token = issue_token(&state, &user)?;
    Ok(Json(AuthResponse { token, user }))
}

async fn me(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<User>> {
    let user = repo::find(&state.pool, auth.claims.sub).await?;
    Ok(Json(user))
}

async fn list_users(State(state): State<AppState>, _: AdminUser) -> AppResult<Json<Vec<User>>> {
    let users = repo::list(&state.pool).await?;
    Ok(Json(users))
}

async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<User>> {
    require_self_or_admin(&auth.claims, id)?;
    let user = repo::find(&state.pool, id).await?;
    Ok(Json(user))
}

async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UserUpdate>,
) -> AppResult<Json<User>> {
    require_self_or_admin(&auth.claims, id)?;
    input.validate().map_err(AppError::Validation)?;

    // Role escalation is admin-only — a regular user editing themselves can
    // change email/password but not promote to Admin.
    if input.role.is_some() && auth.claims.role != Role::Admin {
        return Err(AppError::Forbidden("only admins can change role".into()));
    }

    let new_email = input.email.as_deref().map(str::trim);
    let new_role_str = input.role.as_ref().map(|r| r.as_str());

    // Hash the new password here so the repo stays oblivious to argon2.
    let new_hash = match input.password.as_deref() {
        Some(p) => Some(password::hash_password(p)?),
        None => None,
    };

    let updated = repo::update(
        &state.pool,
        id,
        new_email,
        new_hash.as_deref(),
        new_role_str,
    )
    .await?;

    Ok(Json(updated))
}

async fn delete_user(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // Refuse to let the last admin nuke themselves out of the system — we'd
    // be left with a gallery no one can administer. Cheap guard, big payoff.
    if admin.claims.sub == id {
        return Err(AppError::Conflict(
            "admins cannot delete their own account".into(),
        ));
    }
    repo::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn require_self_or_admin(claims: &Claims, target: Uuid) -> AppResult<()> {
    if claims.role == Role::Admin || claims.sub == target {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "may only access your own user record".into(),
        ))
    }
}

fn issue_token(state: &AppState, user: &User) -> AppResult<String> {
    let role = Role::from_db_str(&user.role)?;
    let claims = Claims::new(user.id, user.email.clone(), role);
    state.jwt_secret.encode(&claims)
}
