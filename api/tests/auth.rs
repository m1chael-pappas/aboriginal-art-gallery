mod common;

use axum::http::StatusCode;
use common::TestClient;
use serde_json::json;
use sqlx::PgPool;

// ---------------------------------------------------------------------------
// Register
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn register_happy_path_returns_token_and_user(pool: PgPool) {
    let client = TestClient::new(pool);

    let (status, body) = client
        .post(
            "/auth/register",
            json!({ "email": "alice@example.com", "password": "correct horse" }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["token"].as_str().is_some(), "token missing: {body}");
    assert_eq!(body["user"]["email"], "alice@example.com");
    assert_eq!(body["user"]["role"], "User");
    assert!(
        body["user"]["password_hash"].is_null(),
        "password_hash must never leak in API responses, got: {body}"
    );
}

#[sqlx::test]
async fn register_rejects_short_password(pool: PgPool) {
    let client = TestClient::new(pool);

    let (status, body) = client
        .post(
            "/auth/register",
            json!({ "email": "bob@example.com", "password": "short" }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("8 characters"));
}

#[sqlx::test]
async fn register_rejects_malformed_email(pool: PgPool) {
    let client = TestClient::new(pool);

    let (status, body) = client
        .post(
            "/auth/register",
            json!({ "email": "not-an-email", "password": "longenough" }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("email"));
}

#[sqlx::test]
async fn register_rejects_duplicate_email_case_insensitive(pool: PgPool) {
    // citext on the email column means "Alice@…" and "alice@…" collide.
    let client = TestClient::new(pool);

    let (first, _) = client
        .post(
            "/auth/register",
            json!({ "email": "alice@example.com", "password": "longenough" }),
        )
        .await;
    assert_eq!(first, StatusCode::CREATED);

    let (status, body) = client
        .post(
            "/auth/register",
            json!({ "email": "ALICE@example.com", "password": "longenough" }),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].as_str().unwrap().contains("already in use"));
}

// ---------------------------------------------------------------------------
// Login
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn login_happy_path(pool: PgPool) {
    let client = TestClient::new(pool);
    client
        .post(
            "/auth/register",
            json!({ "email": "alice@example.com", "password": "longenough" }),
        )
        .await;

    let (status, body) = client
        .post(
            "/auth/login",
            json!({ "email": "alice@example.com", "password": "longenough" }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].as_str().is_some());
    assert_eq!(body["user"]["email"], "alice@example.com");
}

#[sqlx::test]
async fn login_with_wrong_password_returns_401(pool: PgPool) {
    let client = TestClient::new(pool);
    client
        .post(
            "/auth/register",
            json!({ "email": "alice@example.com", "password": "longenough" }),
        )
        .await;

    let (status, _) = client
        .post(
            "/auth/login",
            json!({ "email": "alice@example.com", "password": "wrong-password" }),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn login_with_unknown_email_returns_401(pool: PgPool) {
    let client = TestClient::new(pool);

    let (status, _) = client
        .post(
            "/auth/login",
            json!({ "email": "nobody@example.com", "password": "longenough" }),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// /auth/me
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn me_without_token_returns_401(pool: PgPool) {
    let client = TestClient::new(pool);
    let (status, _) = client.get("/auth/me").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn me_with_garbage_token_returns_401(pool: PgPool) {
    let client = TestClient::new(pool).with_token("not-a-jwt");
    let (status, _) = client.get("/auth/me").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn me_with_valid_token_returns_user(pool: PgPool) {
    let client = TestClient::new(pool).as_user().await;
    let (status, body) = client.get("/auth/me").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "user@test.local");
    assert_eq!(body["role"], "User");
}

// ---------------------------------------------------------------------------
// Role-gated routes on existing BCs
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn reads_on_artists_are_public(pool: PgPool) {
    let client = TestClient::new(pool);
    let (status, body) = client.get("/artists").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}

#[sqlx::test]
async fn write_to_artists_without_token_returns_401(pool: PgPool) {
    let client = TestClient::new(pool);
    let (status, _) = client
        .post("/artists", json!({ "display_name": "Should Fail" }))
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn write_to_artists_as_user_returns_403(pool: PgPool) {
    let client = TestClient::new(pool).as_user().await;
    let (status, _) = client
        .post("/artists", json!({ "display_name": "Forbidden" }))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn write_to_artists_as_admin_succeeds(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let (status, _) = client
        .post("/artists", json!({ "display_name": "Allowed" }))
        .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[sqlx::test]
async fn write_to_artifacts_without_token_returns_401(pool: PgPool) {
    let client = TestClient::new(pool);
    // artist_id can be any uuid - auth runs before validation
    let (status, _) = client
        .post(
            "/artifacts",
            json!({ "title": "X", "artist_id": uuid::Uuid::new_v4() }),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn write_to_tribes_without_token_returns_401(pool: PgPool) {
    let client = TestClient::new(pool);
    let (status, _) = client.post("/tribes", json!({ "name": "X" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ---------------------------------------------------------------------------
// /users (admin) + self-vs-other authorization
// ---------------------------------------------------------------------------

#[sqlx::test]
async fn list_users_requires_admin(pool: PgPool) {
    let user = TestClient::new(pool.clone()).as_user().await;
    let (status, _) = user.get("/users").await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let admin = TestClient::new(pool).as_admin().await;
    let (status, body) = admin.get("/users").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.is_array());
}

#[sqlx::test]
async fn user_can_fetch_self_but_not_others(pool: PgPool) {
    let admin = TestClient::new(pool.clone()).as_admin().await;
    // Admin creates a second user via register-as-someone-else: easiest path
    // is to spin up a separate client to register them.
    let other_client = TestClient::new(pool.clone())
        .as_user_with("other@test.local", "other-pw-12345")
        .await;
    let (_, other_me) = other_client.get("/auth/me").await;
    let other_id = other_me["id"].as_str().unwrap();

    // Now a "normal" user (different email) tries to read `other`.
    let user = TestClient::new(pool)
        .as_user_with("normal@test.local", "normal-pw-12345")
        .await;

    let (_, self_me) = user.get("/auth/me").await;
    let self_id = self_me["id"].as_str().unwrap();

    let (status, _) = user.get(&format!("/users/{self_id}")).await;
    assert_eq!(status, StatusCode::OK, "user can read themselves");

    let (status, _) = user.get(&format!("/users/{other_id}")).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "user cannot read another");

    // Admin can read either.
    let (status, _) = admin.get(&format!("/users/{other_id}")).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn user_cannot_promote_themselves_to_admin(pool: PgPool) {
    let user = TestClient::new(pool).as_user().await;
    let (_, me) = user.get("/auth/me").await;
    let id = me["id"].as_str().unwrap();

    let (status, body) = user
        .put(&format!("/users/{id}"), json!({ "role": "Admin" }))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body["error"].as_str().unwrap().contains("role"));
}

#[sqlx::test]
async fn admin_can_promote_user_to_admin(pool: PgPool) {
    let admin = TestClient::new(pool.clone()).as_admin().await;
    let _user_client = TestClient::new(pool.clone())
        .as_user_with("promote@test.local", "long-enough")
        .await;

    // Look up the new user's id via admin's user list.
    let (_, list) = admin.get("/users").await;
    let target_id = list
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["email"] == "promote@test.local")
        .expect("new user in list")["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, body) = admin
        .put(&format!("/users/{target_id}"), json!({ "role": "Admin" }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["role"], "Admin");
}

#[sqlx::test]
async fn admin_cannot_delete_themselves(pool: PgPool) {
    let admin = TestClient::new(pool).as_admin().await;
    let (_, me) = admin.get("/auth/me").await;
    let id = me["id"].as_str().unwrap();

    let (status, body) = admin.delete(&format!("/users/{id}")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].as_str().unwrap().contains("own account"));
}
