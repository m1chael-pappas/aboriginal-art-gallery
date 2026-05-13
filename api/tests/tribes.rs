mod common;

use axum::http::StatusCode;
use common::TestClient;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn create_tribe_happy_path(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (status, body) = client
        .post(
            "/tribes",
            json!({
                "name": "Pintupi",
                "region": "Western Desert",
                "language_group": "Western Desert language",
                "description": "Aboriginal Australian people from the Western Desert region."
            }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "Pintupi");
    assert_eq!(body["region"], "Western Desert");
    assert!(body["id"].is_string());
    assert!(body["created_at"].is_string());
}

#[sqlx::test]
async fn create_tribe_rejects_empty_name(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (status, body) = client
        .post("/tribes", json!({ "name": "   " }))
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("name"));
}

#[sqlx::test]
async fn create_tribe_rejects_duplicate_name(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (first, _) = client.post("/tribes", json!({ "name": "Pintupi" })).await;
    assert_eq!(first, StatusCode::CREATED);

    let (status, body) = client.post("/tribes", json!({ "name": "Pintupi" })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        body["error"].as_str().unwrap().contains("already exists"),
        "error should mention duplicate, got: {body}"
    );
}

#[sqlx::test]
async fn list_tribes_returns_array(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (status, body) = client.get("/tribes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));

    client.post("/tribes", json!({ "name": "A" })).await;
    client.post("/tribes", json!({ "name": "B" })).await;

    let (status, body) = client.get("/tribes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn get_tribe_by_id_returns_record(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let (_, created) = client.post("/tribes", json!({ "name": "Yolngu" })).await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = client.get(&format!("/tribes/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Yolngu");
}

#[sqlx::test]
async fn get_tribe_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let unknown = Uuid::new_v4();
    let (status, _) = client.get(&format!("/tribes/{unknown}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn update_tribe_replaces_fields(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let (_, created) = client.post("/tribes", json!({ "name": "Old" })).await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = client
        .put(
            &format!("/tribes/{id}"),
            json!({ "name": "New", "region": "Updated" }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "New");
    assert_eq!(body["region"], "Updated");
}

#[sqlx::test]
async fn update_tribe_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let unknown = Uuid::new_v4();

    let (status, _) = client
        .put(&format!("/tribes/{unknown}"), json!({ "name": "X" }))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn update_tribe_rejects_duplicate_name(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    client.post("/tribes", json!({ "name": "First" })).await;
    let (_, second) = client.post("/tribes", json!({ "name": "Second" })).await;
    let second_id = second["id"].as_str().unwrap();

    let (status, body) = client
        .put(&format!("/tribes/{second_id}"), json!({ "name": "First" }))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].as_str().unwrap().contains("already exists"));
}

#[sqlx::test]
async fn delete_tribe_removes_record(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let (_, created) = client.post("/tribes", json!({ "name": "Gone" })).await;
    let id = created["id"].as_str().unwrap();

    let (status, _) = client.delete(&format!("/tribes/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = client.get(&format!("/tribes/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_tribe_unknown_id_returns_404(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let unknown = Uuid::new_v4();
    let (status, _) = client.delete(&format!("/tribes/{unknown}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn delete_tribe_nulls_artist_tribe_id(pool: PgPool) {
    // ON DELETE SET NULL: deleting a tribe should leave artists from that
    // tribe in place, just with tribe_id cleared.
    let client = TestClient::new(pool).as_admin().await;

    let (_, tribe) = client
        .post("/tribes", json!({ "name": "Disappearing" }))
        .await;
    let tribe_id = tribe["id"].as_str().unwrap();

    let (_, artist) = client
        .post(
            "/artists",
            json!({ "display_name": "Affiliated", "tribe_id": tribe_id }),
        )
        .await;
    let artist_id = artist["id"].as_str().unwrap();
    assert_eq!(artist["tribe_id"], tribe_id);

    let (status, _) = client.delete(&format!("/tribes/{tribe_id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, after) = client.get(&format!("/artists/{artist_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(after["tribe_id"].is_null(), "tribe_id should be null after tribe deletion, got: {after}");
}
