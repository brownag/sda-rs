use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use sda_rs::{app_with_body_limit, config::Config};
use tower::ServiceExt;

fn config() -> Config {
    Config {
        backend_url: ":memory:".into(),
        port: 0,
        spatial_extension: None,
        geometry_column: None,
    }
}

#[tokio::test]
async fn accepts_body_at_limit_and_rejects_body_over_limit() {
    let limit = 128;
    let body = format!(
        r#"{{"query":"SELECT 1","format":"JSON"}}{}"#,
        " ".repeat(limit - r#"{"query":"SELECT 1","format":"JSON"}"#.len())
    );
    assert_eq!(body.len(), limit);

    let response = app_with_body_limit(config(), limit)
        .oneshot(
            Request::post("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let _ = to_bytes(response.into_body(), usize::MAX).await.unwrap();

    let response = app_with_body_limit(config(), limit)
        .oneshot(
            Request::post("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(format!("{body} ")))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(response.status(), StatusCode::OK);
}
