mod common;

use axum::http::StatusCode;
use common::TestClient;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn create_artist_happy_path(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (status, body) = client
        .post(
            "/artists",
            json!({
                "display_name": "Albert Namatjira",
                "birth_year": 1902,
                "death_year": 1959,
                "region": "Hermannsburg",
                "biography": "Watercolour painter."
            }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["display_name"], "Albert Namatjira");
    assert_eq!(body["birth_year"], 1902);
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[sqlx::test]
async fn create_artist_rejects_empty_display_name(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (status, body) = client
        .post("/artists", json!({ "display_name": "   " }))
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("display_name"),
        "error should mention display_name, got: {body}"
    );
}

#[sqlx::test]
async fn create_artist_rejects_invalid_lifespan(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (status, body) = client
        .post(
            "/artists",
            json!({
                "display_name": "Should Fail",
                "birth_year": 1990,
                "death_year": 1980
            }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("birth_year"),
        "error should mention birth_year, got: {body}"
    );
}

#[sqlx::test]
async fn list_artists_returns_array(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (status, body) = client.get("/artists").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));

    client
        .post("/artists", json!({ "display_name": "Artist A" }))
        .await;
    client
        .post("/artists", json!({ "display_name": "Artist B" }))
        .await;

    let (status, body) = client.get("/artists").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn get_artist_by_id_returns_record(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (_, created) = client
        .post("/artists", json!({ "display_name": "Rover Thomas" }))
        .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = client.get(&format!("/artists/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "Rover Thomas");
    assert_eq!(body["id"], id);
}

#[sqlx::test]
async fn get_artist_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let unknown = Uuid::new_v4();

    let (status, _) = client.get(&format!("/artists/{unknown}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn update_artist_replaces_fields(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (_, created) = client
        .post(
            "/artists",
            json!({ "display_name": "Old Name", "biography": "Old bio" }),
        )
        .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = client
        .put(
            &format!("/artists/{id}"),
            json!({
                "display_name": "New Name",
                "birth_year": 1900,
                "biography": "New bio"
            }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["display_name"], "New Name");
    assert_eq!(body["birth_year"], 1900);
    assert_eq!(body["biography"], "New bio");
    // PUT replaces - fields not sent become null
    assert!(body["region"].is_null());
}

#[sqlx::test]
async fn update_artist_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let unknown = Uuid::new_v4();

    let (status, _) = client
        .put(
            &format!("/artists/{unknown}"),
            json!({ "display_name": "X" }),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_artist_removes_record(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (_, created) = client
        .post("/artists", json!({ "display_name": "To Delete" }))
        .await;
    let id = created["id"].as_str().unwrap();

    let (status, _) = client.delete(&format!("/artists/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = client.get(&format!("/artists/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_artist_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let unknown = Uuid::new_v4();

    let (status, _) = client.delete(&format!("/artists/{unknown}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn create_artist_with_tribe_links_them(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (_, tribe) = client.post("/tribes", json!({ "name": "Pintupi" })).await;
    let tribe_id = tribe["id"].as_str().unwrap();

    let (status, body) = client
        .post(
            "/artists",
            json!({
                "display_name": "Affiliated Artist",
                "tribe_id": tribe_id
            }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["tribe_id"], tribe_id);
}

#[sqlx::test]
async fn create_artist_rejects_unknown_tribe(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let bogus = Uuid::new_v4();

    let (status, body) = client
        .post(
            "/artists",
            json!({
                "display_name": "X",
                "tribe_id": bogus
            }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"].as_str().unwrap().contains("tribe"),
        "error should mention tribe, got: {body}"
    );
}

#[sqlx::test]
async fn delete_artist_with_artifacts_returns_409(pool: PgPool) {
    // ON DELETE RESTRICT on artifacts.artist_id should surface as a 409
    // Conflict, not a 500 - the user can't delete an artist whose artworks
    // still exist.
    let client = TestClient::new(pool).as_admin().await;

    let (_, artist) = client
        .post("/artists", json!({ "display_name": "Has artworks" }))
        .await;
    let artist_id = artist["id"].as_str().unwrap();

    client
        .post(
            "/artifacts",
            json!({ "title": "An artwork", "artist_id": artist_id }),
        )
        .await;

    let (status, body) = client.delete(&format!("/artists/{artist_id}")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        body["error"].as_str().unwrap().contains("artifacts"),
        "error should mention artifacts, got: {body}"
    );
}
