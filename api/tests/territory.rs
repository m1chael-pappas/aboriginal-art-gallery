mod common;

use axum::http::StatusCode;
use common::TestClient;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

// A simple square Polygon spanning a couple of degrees around 0°/0° - easy
// to reason about for "point inside / point outside" assertions.
fn square_polygon(min_lng: f64, min_lat: f64, max_lng: f64, max_lat: f64) -> serde_json::Value {
    json!({
        "type": "Polygon",
        "coordinates": [[
            [min_lng, min_lat],
            [max_lng, min_lat],
            [max_lng, max_lat],
            [min_lng, max_lat],
            [min_lng, min_lat],
        ]]
    })
}

#[sqlx::test]
async fn set_territory_returns_tribe_with_geojson(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (_, tribe) = client.post("/tribes", json!({ "name": "Demo" })).await;
    let id = tribe["id"].as_str().unwrap();
    assert!(tribe["territory"].is_null(), "fresh tribe has no territory");

    let polygon = square_polygon(0.0, 0.0, 1.0, 1.0);
    let (status, body) = client
        .put(&format!("/tribes/{id}/territory"), polygon)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["territory"]["type"], "MultiPolygon");
    assert!(
        body["territory"]["coordinates"].is_array(),
        "expected GeoJSON coordinates array, got {body}"
    );
}

#[sqlx::test]
async fn get_tribe_after_set_includes_territory(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let (_, tribe) = client.post("/tribes", json!({ "name": "Demo" })).await;
    let id = tribe["id"].as_str().unwrap();

    client
        .put(
            &format!("/tribes/{id}/territory"),
            square_polygon(0.0, 0.0, 1.0, 1.0),
        )
        .await;

    let (_, fetched) = client.get(&format!("/tribes/{id}")).await;
    assert_eq!(fetched["territory"]["type"], "MultiPolygon");
}

#[sqlx::test]
async fn clear_territory_nulls_the_field(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let (_, tribe) = client.post("/tribes", json!({ "name": "Demo" })).await;
    let id = tribe["id"].as_str().unwrap();

    client
        .put(
            &format!("/tribes/{id}/territory"),
            square_polygon(0.0, 0.0, 1.0, 1.0),
        )
        .await;

    let (status, _) = client.delete(&format!("/tribes/{id}/territory")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, fetched) = client.get(&format!("/tribes/{id}")).await;
    assert!(fetched["territory"].is_null(), "territory should be null after DELETE, got {fetched}");
}

#[sqlx::test]
async fn set_territory_rejects_invalid_geojson(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;
    let (_, tribe) = client.post("/tribes", json!({ "name": "Demo" })).await;
    let id = tribe["id"].as_str().unwrap();

    let (status, _) = client
        .put(
            &format!("/tribes/{id}/territory"),
            json!({ "type": "not-a-thing", "coordinates": [] }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn set_territory_requires_admin(pool: PgPool) {
    // No auth → 401
    let client = TestClient::new(pool.clone());
    let bogus = Uuid::new_v4();
    let (status, _) = client
        .put(
            &format!("/tribes/{bogus}/territory"),
            square_polygon(0.0, 0.0, 1.0, 1.0),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // User auth → 403
    let user = TestClient::new(pool).as_user().await;
    let (status, _) = user
        .put(
            &format!("/tribes/{bogus}/territory"),
            square_polygon(0.0, 0.0, 1.0, 1.0),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn search_near_returns_only_matching_tribes(pool: PgPool) {
    let client = TestClient::new(pool).as_admin().await;

    let (_, inside) = client.post("/tribes", json!({ "name": "Inside" })).await;
    let inside_id = inside["id"].as_str().unwrap();
    client
        .put(
            &format!("/tribes/{inside_id}/territory"),
            square_polygon(0.0, 0.0, 1.0, 1.0),
        )
        .await;

    let (_, far_away) = client.post("/tribes", json!({ "name": "FarAway" })).await;
    let far_id = far_away["id"].as_str().unwrap();
    client
        .put(
            &format!("/tribes/{far_id}/territory"),
            square_polygon(100.0, 50.0, 101.0, 51.0),
        )
        .await;

    let (_, no_territory) = client.post("/tribes", json!({ "name": "NoTerritory" })).await;
    let no_terr_id = no_territory["id"].as_str().unwrap();

    // Query a point inside the first polygon, with a small radius.
    let (status, body) = client.get("/tribes/near?lat=0.5&lng=0.5&km=50").await;
    assert_eq!(status, StatusCode::OK);

    let names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Inside"]);

    let ids: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert!(!ids.contains(&far_id), "far tribe must not appear");
    assert!(
        !ids.contains(&no_terr_id),
        "tribe with no territory must not appear"
    );
}

#[sqlx::test]
async fn search_near_rejects_out_of_range_coords(pool: PgPool) {
    let client = TestClient::new(pool);

    let (status, body) = client.get("/tribes/near?lat=95&lng=0&km=10").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("lat"));

    let (status, body) = client.get("/tribes/near?lat=0&lng=200&km=10").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("lng"));

    let (status, body) = client.get("/tribes/near?lat=0&lng=0&km=-5").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("km"));
}

#[sqlx::test]
async fn search_near_is_public(pool: PgPool) {
    // No token needed - reads are public, including this spatial one.
    let client = TestClient::new(pool);
    let (status, body) = client.get("/tribes/near?lat=0&lng=0&km=10").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));
}
