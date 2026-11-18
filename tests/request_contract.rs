use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn decodes_request_encodings_executes_queries_and_echoes_query() {
    let path = format!(
        "file:request_contract_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let holder = Connection::open(&path).unwrap();
    holder
        .execute_batch("CREATE TABLE sample(value TEXT); INSERT INTO sample VALUES ('ready');")
        .unwrap();
    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: path,
        port: 0,
        spatial_extension: None,
        geometry_column: None,
    });

    for (content_type, body) in [
        (
            "application/json",
            r#"{"QuErY":"SELECT value FROM sample","FoRmAt":"JSON"}"#,
        ),
        (
            "application/x-www-form-urlencoded",
            "qUeRy=SELECT%20value%20FROM%20sample&fOrMaT=JSON",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/Tabular/post.rest")
                    .header("content-type", content_type)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX).await.unwrap(),
            r#"{"Table":[["ready"]]}"#
        );
    }

    let echo_query = "SELECT 'echoed' AS value";
    let echo = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"query":"{echo_query}","format":"~"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(echo.status(), StatusCode::OK);
    assert_eq!(echo.headers()["content-type"], "text/plain; charset=utf-8");
    assert_eq!(
        to_bytes(echo.into_body(), usize::MAX).await.unwrap(),
        echo_query
    );
}
