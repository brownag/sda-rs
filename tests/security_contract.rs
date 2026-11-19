use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn app_with_seeded_table() -> (axum::Router, Connection) {
    let path = format!(
        "file:security_contract_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let holder = Connection::open(&path).unwrap();
    holder
        .execute_batch("CREATE TABLE sample(value INTEGER); INSERT INTO sample VALUES (7);")
        .unwrap();
    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: path,
        port: 0,
        spatial_extension: None,
        geometry_column: None,
    });
    (app, holder)
}

async fn post(app: axum::Router, query: &str) -> (StatusCode, String) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"query":{query:?},"format":"JSON"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn selects_and_request_local_temp_workflows_succeed() {
    let (app, _holder) = app_with_seeded_table();

    let (status, body) = post(app.clone(), "SELECT value FROM sample").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, r#"{"Table":[["7"]]}"#);

    let (status, body) = post(
        app,
        "CREATE TEMP TABLE #scratch(value INTEGER); \
         INSERT INTO #scratch VALUES (12); \
         SELECT value FROM #scratch",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, r#"{"Table":[["12"]]}"#);
}

#[tokio::test]
async fn rejects_persistent_writes_without_changing_seeded_rows() {
    let (app, holder) = app_with_seeded_table();

    let (status, _) = post(app, "UPDATE sample SET value = 99").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);

    let value: i64 = holder
        .query_row("SELECT value FROM sample", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, 7);
}

#[tokio::test]
async fn rejects_user_transaction_control_sql() {
    let (app, _holder) = app_with_seeded_table();

    let (status, _) = post(app.clone(), "BEGIN TRANSACTION").await;
    assert_ne!(status, StatusCode::OK);

    let (status, _) = post(app, "COMMIT").await;
    assert_ne!(status, StatusCode::OK);
}
