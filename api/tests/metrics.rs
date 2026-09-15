//! `/metrics` exposure. Kept in its own test binary with a single test,
//! because `with_metrics` installs the process-wide recorder and can only
//! run once per process.

mod common;

use axum::http::StatusCode;
use common::TestClient;
use gallery_api::{UNMATCHED_ENDPOINT, with_metrics};
use sqlx::PgPool;

#[sqlx::test]
async fn metrics_count_routed_requests_but_not_probes(pool: PgPool) {
    let client = TestClient::new(pool).map_app(with_metrics);

    assert_eq!(client.get("/artists").await.0, StatusCode::OK);
    assert_eq!(
        client.get("/artists/not-a-uuid").await.0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(client.get("/health").await.0, StatusCode::OK);
    assert_eq!(client.get("/wp-login.php").await.0, StatusCode::NOT_FOUND);
    assert_eq!(client.get("/.env").await.0, StatusCode::NOT_FOUND);

    let (status, body) = client.get_text("/metrics").await;

    assert_eq!(status, StatusCode::OK);
    let request_lines: Vec<&str> = body
        .lines()
        .filter(|line| line.starts_with("axum_http_requests_total{"))
        .collect();
    assert!(
        request_lines
            .iter()
            .any(|l| l.contains(r#"endpoint="/artists""#) && l.contains(r#"status="200""#)),
        "missing 200 counter for /artists in:\n{body}"
    );
    assert!(
        request_lines.iter().any(|l| l.contains(r#"status="400""#)),
        "missing 400 counter in:\n{body}"
    );
    let unmatched = format!(r#"endpoint="{UNMATCHED_ENDPOINT}""#);
    assert!(
        request_lines
            .iter()
            .any(|l| l.contains(&unmatched) && l.contains(r#"status="404""#) && l.ends_with(" 2")),
        "unknown paths must share one label in:\n{body}"
    );
    assert!(
        !body.contains("wp-login") && !body.contains(".env"),
        "raw unknown paths must not become label values:\n{body}"
    );
    assert!(
        body.contains("axum_http_requests_duration_seconds_bucket"),
        "missing latency histogram in:\n{body}"
    );
    assert!(
        !body.contains(r#"endpoint="/health""#) && !body.contains(r#"endpoint="/metrics""#),
        "probe endpoints must be excluded:\n{body}"
    );
}
