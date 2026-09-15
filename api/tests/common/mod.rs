use axum::{
    Router,
    body::{Body, Bytes},
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

use std::sync::Arc;

use gallery_api::{
    artifacts::PgArtifactStore, artists::PgArtistStore, auth::JwtSecret, build_router,
    state::AppState, tribes::PgTribeStore, users::PgUserStore,
};

/// 32+ bytes - the only constraint JwtSecret::from_env enforces. The exact
/// value is irrelevant: each test gets a fresh DB, and tokens never outlive
/// the test process.
const TEST_JWT_SECRET: &[u8] = b"test-secret-test-secret-test-secret-32b";

pub struct TestClient {
    app: Router,
    pool: PgPool,
    token: Option<String>,
}

// `common` is compiled once per integration-test binary. A helper used by
// some test files but not the one currently being built reads as dead code
// in that binary, even though it is used elsewhere. Silence that per-crate
// false positive rather than littering each call site.
#[allow(dead_code)]
impl TestClient {
    pub fn new(pool: PgPool) -> Self {
        let jwt_secret = JwtSecret::from_bytes(TEST_JWT_SECRET);
        let app = build_router(AppState {
            pool: pool.clone(),
            jwt_secret,
            artists: Arc::new(PgArtistStore::new(pool.clone())),
            artifacts: Arc::new(PgArtifactStore::new(pool.clone())),
            tribes: Arc::new(PgTribeStore::new(pool.clone())),
            users: Arc::new(PgUserStore::new(pool.clone())),
        });
        Self {
            app,
            pool,
            token: None,
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Inserts an admin user directly into the DB and logs them in via the
    /// public login endpoint, returning a client that carries the admin's
    /// bearer token. Bypasses the bootstrap problem (you can't promote to
    /// admin without an admin token) and gives every mutation test a clean
    /// admin context with one line.
    pub async fn with_admin(self) -> Self {
        self.with_admin_login("admin@test.local", "test-admin-pw-123")
            .await
    }

    pub async fn with_admin_login(mut self, email: &str, password: &str) -> Self {
        let hash = gallery_api::auth::password::hash_password(password).expect("hash admin pw");
        sqlx::query!(
            "INSERT INTO users (email, password_hash, role) VALUES ($1, $2, $3)",
            email,
            hash,
            "Admin",
        )
        .execute(&self.pool)
        .await
        .expect("insert admin user");

        let (status, body) = self
            .send(
                "POST",
                "/auth/login",
                Some(json!({ "email": email, "password": password })),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "admin login failed: {body}");
        self.token = Some(
            body["token"]
                .as_str()
                .expect("token in response")
                .to_string(),
        );
        self
    }

    /// Logs the client in as a regular `User`-role caller via the public
    /// `/auth/register` endpoint. Exercises the register path as a side effect.
    pub async fn with_user(self) -> Self {
        self.with_user_login("user@test.local", "test-user-pw-123")
            .await
    }

    pub async fn with_user_login(mut self, email: &str, password: &str) -> Self {
        let (status, body) = self
            .send(
                "POST",
                "/auth/register",
                Some(json!({ "email": email, "password": password })),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "user register failed: {body}");
        self.token = Some(
            body["token"]
                .as_str()
                .expect("token in response")
                .to_string(),
        );
        self
    }

    pub async fn get(&self, uri: &str) -> (StatusCode, Value) {
        self.send("GET", uri, None).await
    }

    pub async fn post(&self, uri: &str, body: Value) -> (StatusCode, Value) {
        self.send("POST", uri, Some(body)).await
    }

    pub async fn put(&self, uri: &str, body: Value) -> (StatusCode, Value) {
        self.send("PUT", uri, Some(body)).await
    }

    pub async fn delete(&self, uri: &str) -> (StatusCode, Value) {
        self.send("DELETE", uri, None).await
    }

    pub async fn send(&self, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = &self.token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request_body = match body {
            Some(json) => {
                builder = builder.header("content-type", "application/json");
                Body::from(serde_json::to_vec(&json).expect("serialize request body"))
            }
            None => Body::empty(),
        };
        let request = builder.body(request_body).expect("build request");
        let (status, bytes) = self.dispatch(request).await;

        let json = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };

        (status, json)
    }

    /// `GET` for non-JSON endpoints such as `/metrics`, returning the body
    /// as text.
    pub async fn get_text(&self, uri: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .expect("build request");
        let (status, bytes) = self.dispatch(request).await;
        (
            status,
            String::from_utf8(bytes.to_vec()).expect("utf-8 response body"),
        )
    }

    /// Wraps the router under test, for example with
    /// `gallery_api::with_metrics`, which `main` applies outside
    /// `build_router`.
    pub fn map_app(mut self, wrap: impl FnOnce(Router) -> Router) -> Self {
        self.app = wrap(self.app);
        self
    }

    async fn dispatch(&self, request: Request<Body>) -> (StatusCode, Bytes) {
        // Router::oneshot consumes self; cloning is cheap (Arc inside).
        let response = self
            .app
            .clone()
            .oneshot(request)
            .await
            .expect("router responded");

        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("collect response body")
            .to_bytes();
        (status, bytes)
    }
}
