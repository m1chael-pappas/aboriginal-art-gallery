//! HTTP handlers for the auth + users surface.
//!
//! `/auth/register` and `/auth/login` are public. Everything else requires a
//! valid bearer token, and the `/users` endpoints additionally require the
//! Admin role (the GET-by-id and PUT-by-id endpoints relax this for "self").

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

/// Build the `/auth` + `/users` sub-router.
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

/// `POST /auth/register` - public. Creates a `role=User` account and
/// returns an Auth response (token + user) so the FE doesn't need a
/// follow-up login call.
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterInput,
    responses(
        (status = 201, description = "Account created (role = User) and signed in", body = AuthResponse),
        (status = 400, description = "Validation failed (bad email, password < 8 chars)"),
        (status = 409, description = "Email already in use"),
    ),
)]
pub(crate) async fn register(
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

/// `POST /auth/login` - public.
///
/// Looks up the user by email *first*. Argon2 verify is the expensive step,
/// so it would be tempting to skip it when the user doesn't exist - but
/// that creates a timing oracle for enumerating valid emails. We accept
/// the small leak (no hash to verify on miss) in exchange for a simpler
/// handler; a fully timing-safe variant would verify against a dummy hash
/// on miss.
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginInput,
    responses(
        (status = 200, description = "Credentials accepted; token + user returned", body = AuthResponse),
        (status = 400, description = "Empty email or password"),
        (status = 401, description = "Invalid credentials"),
    ),
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> AppResult<Json<AuthResponse>> {
    input.validate().map_err(AppError::Validation)?;

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

/// `GET /auth/me` - the user identified by the bearer token.
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user (sub from the JWT)", body = User),
        (status = 401, description = "Missing or invalid bearer token"),
    ),
)]
pub(crate) async fn me(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<User>> {
    let user = repo::find(&state.pool, auth.claims.sub).await?;
    Ok(Json(user))
}

/// `GET /users` - admin-only listing of every account.
#[utoipa::path(
    get,
    path = "/users",
    tag = "users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All users (admin only)", body = Vec<User>),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
    ),
)]
pub(crate) async fn list_users(
    State(state): State<AppState>,
    _: AdminUser,
) -> AppResult<Json<Vec<User>>> {
    let users = repo::list(&state.pool).await?;
    Ok(Json(users))
}

/// `GET /users/{id}` - admin can read anyone; regular users can read only
/// themselves.
#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "User UUID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = User),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Not admin and not your own record"),
        (status = 404, description = "No user with that id"),
    ),
)]
pub(crate) async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<User>> {
    require_self_or_admin(&auth.claims, id)?;
    let user = repo::find(&state.pool, id).await?;
    Ok(Json(user))
}

/// `PUT /users/{id}` - admin can patch anyone; regular users can patch only
/// themselves, and never `role` (escalation guard).
#[utoipa::path(
    put,
    path = "/users/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "User UUID")),
    request_body = UserUpdate,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, body = User),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Not admin and not your own record, or non-admin trying to change role"),
        (status = 404, description = "No user with that id"),
        (status = 409, description = "Email already in use"),
    ),
)]
pub(crate) async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UserUpdate>,
) -> AppResult<Json<User>> {
    require_self_or_admin(&auth.claims, id)?;
    input.validate().map_err(AppError::Validation)?;

    if input.role.is_some() && auth.claims.role != Role::Admin {
        return Err(AppError::Forbidden("only admins can change role".into()));
    }

    let new_email = input.email.as_deref().map(str::trim);
    let new_role_str = input.role.as_ref().map(|r| r.as_str());

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

/// `DELETE /users/{id}` - admin-only, with a guard against deleting *yourself*.
///
/// Refusing the self-delete keeps the gallery from ending up in a state
/// where nobody can administer it. Cheap check, big payoff.
#[utoipa::path(
    delete,
    path = "/users/{id}",
    tag = "users",
    params(("id" = Uuid, Path, description = "User UUID")),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "User deleted"),
        (status = 401, description = "Missing or invalid bearer token"),
        (status = 403, description = "Admin role required"),
        (status = 404, description = "No user with that id"),
        (status = 409, description = "Admins cannot delete their own account"),
    ),
)]
pub(crate) async fn delete_user(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    if admin.claims.sub == id {
        return Err(AppError::Conflict(
            "admins cannot delete their own account".into(),
        ));
    }
    repo::delete(&state.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Authorisation helper: allow Admin tokens through unconditionally, and
/// allow any token whose `sub` matches the targeted user id.
fn require_self_or_admin(claims: &Claims, target: Uuid) -> AppResult<()> {
    if claims.role == Role::Admin || claims.sub == target {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "may only access your own user record".into(),
        ))
    }
}

/// Build a fresh JWT for `user`, encoded with the app's signing key.
fn issue_token(state: &AppState, user: &User) -> AppResult<String> {
    let role = Role::from_db_str(&user.role)?;
    let claims = Claims::new(user.id, user.email.clone(), role);
    state.jwt_secret.encode(&claims)
}
