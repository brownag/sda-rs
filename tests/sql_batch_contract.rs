use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;

static NEXT: AtomicUsize = AtomicUsize::new(0);

async fn post_query(app: axum::Router, query: &str) -> (StatusCode, String) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"query":"{query}","format":"JSON"}}"#
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
async fn executes_supported_scalar_sql_and_temp_table_batch() {
    let path = format!(
        "file:sql_batch_contract_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let holder = Connection::open(&path).unwrap();
    holder
        .execute_batch(
            "CREATE TABLE [sample data] ([id] INTEGER, [name] TEXT, [optional] TEXT);
             INSERT INTO [sample data] VALUES (1, 'alpha', NULL), (2, 'beta', 'present'), (3, 'gamma', NULL);",
        )
        .unwrap();
    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: path,
        port: 0,
        spatial_extension: None,
        geometry_column: None,
    });

    let (status, body) = post_query(
        app.clone(),
        "SELECT TOP (2) [id], ISNULL([optional], 'missing') AS [value], IIF([id] > 1, 'yes', 'no') AS [flag] FROM [sample data] ORDER BY [id]",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        r#"{"Table":[["1","missing","no"],["2","present","yes"]]}"#
    );

    let (status, body) = post_query(
        app.clone(),
        "SELECT STRING_AGG([name], ',') AS [names] FROM [sample data]",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"Table":[["alpha,beta,gamma"]]}"#);

    let (status, body) = post_query(
        app,
        "SELECT [id] INTO #picked FROM [sample data] WHERE [id] > 1; SELECT [id] FROM #picked ORDER BY [id]",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"Table":[["2"],["3"]]}"#);
}
