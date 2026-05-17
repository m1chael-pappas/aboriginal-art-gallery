mod common;

use axum::http::StatusCode;
use common::TestClient;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

// Most artifact tests need an artist to attach to - this seeds one and
// returns its id.
async fn create_artist(client: &TestClient, name: &str) -> Uuid {
    let (status, body) = client
        .post("/artists", json!({ "display_name": name }))
        .await;
    assert_eq!(status, StatusCode::CREATED, "artist setup failed: {body}");
    let id_str = body["id"].as_str().expect("artist id missing");
    Uuid::parse_str(id_str).expect("artist id not a uuid")
}

#[sqlx::test]
async fn create_artifact_happy_path(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let artist_id = create_artist(&client, "Emily Kame Kngwarreye").await;

    let (status, body) = client
        .post(
            "/artifacts",
            json!({
                "title": "Earth's Creation",
                "artist_id": artist_id,
                "art_type": "painting",
                "art_style": "Western Desert",
                "medium": "Synthetic polymer paint on canvas",
                "year_created": 1994,
                "height_cm": 275,
                "width_cm": 632,
                "description": "A monumental four-panel painting."
            }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["title"], "Earth's Creation");
    assert_eq!(body["artist_id"], artist_id.to_string());
    assert_eq!(body["year_created"], 1994);
    assert_eq!(body["art_type"], "painting");
    assert!(body["id"].is_string());
}

#[sqlx::test]
async fn create_artifact_rejects_empty_title(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let artist_id = create_artist(&client, "Test").await;

    let (status, body) = client
        .post(
            "/artifacts",
            json!({ "title": "   ", "artist_id": artist_id }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("title"));
}

#[sqlx::test]
async fn create_artifact_rejects_unknown_artist(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let bogus = Uuid::new_v4();

    let (status, body) = client
        .post(
            "/artifacts",
            json!({ "title": "Untitled", "artist_id": bogus }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"].as_str().unwrap().contains("artist"),
        "error should mention artist, got: {body}"
    );
}

#[sqlx::test]
async fn create_artifact_rejects_non_positive_dimension(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let artist_id = create_artist(&client, "Test").await;

    let (status, body) = client
        .post(
            "/artifacts",
            json!({
                "title": "Bad Dims",
                "artist_id": artist_id,
                "height_cm": -1
            }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("height_cm"));
}

#[sqlx::test]
async fn list_artifacts_returns_array(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (status, body) = client.get("/artifacts").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));

    let artist_id = create_artist(&client, "Test").await;
    client
        .post(
            "/artifacts",
            json!({ "title": "A", "artist_id": artist_id }),
        )
        .await;
    client
        .post(
            "/artifacts",
            json!({ "title": "B", "artist_id": artist_id }),
        )
        .await;

    let (status, body) = client.get("/artifacts").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn get_artifact_by_id_returns_record(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let artist_id = create_artist(&client, "Test").await;

    let (_, created) = client
        .post(
            "/artifacts",
            json!({ "title": "Original", "artist_id": artist_id }),
        )
        .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = client.get(&format!("/artifacts/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], "Original");
}

#[sqlx::test]
async fn get_artifact_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let unknown = Uuid::new_v4();
    let (status, _) = client.get(&format!("/artifacts/{unknown}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn update_artifact_replaces_fields(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let artist_id = create_artist(&client, "Test").await;

    let (_, created) = client
        .post(
            "/artifacts",
            json!({ "title": "Old Title", "artist_id": artist_id }),
        )
        .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = client
        .put(
            &format!("/artifacts/{id}"),
            json!({
                "title": "New Title",
                "artist_id": artist_id,
                "year_created": 2020
            }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], "New Title");
    assert_eq!(body["year_created"], 2020);
}

#[sqlx::test]
async fn update_artifact_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let artist_id = create_artist(&client, "Test").await;
    let unknown = Uuid::new_v4();

    let (status, _) = client
        .put(
            &format!("/artifacts/{unknown}"),
            json!({ "title": "X", "artist_id": artist_id }),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn update_artifact_rejects_unknown_artist(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let artist_id = create_artist(&client, "Test").await;

    let (_, created) = client
        .post(
            "/artifacts",
            json!({ "title": "X", "artist_id": artist_id }),
        )
        .await;
    let id = created["id"].as_str().unwrap();
    let bogus = Uuid::new_v4();

    let (status, body) = client
        .put(
            &format!("/artifacts/{id}"),
            json!({ "title": "X", "artist_id": bogus }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("artist"));
}

#[sqlx::test]
async fn delete_artifact_removes_record(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let artist_id = create_artist(&client, "Test").await;

    let (_, created) = client
        .post(
            "/artifacts",
            json!({ "title": "To Delete", "artist_id": artist_id }),
        )
        .await;
    let id = created["id"].as_str().unwrap();

    let (status, _) = client.delete(&format!("/artifacts/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = client.get(&format!("/artifacts/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_artifact_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let unknown = Uuid::new_v4();
    let (status, _) = client.delete(&format!("/artifacts/{unknown}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
