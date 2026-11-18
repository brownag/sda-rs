use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use tower::ServiceExt;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn database_uri() -> String {
    format!(
        "file:extension_security_contract_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

fn app(uri: String, spatial_extension: Option<String>) -> axum::Router {
    sda_rs::app(sda_rs::config::Config {
        backend_url: uri,
        port: 0,
        spatial_extension,
        geometry_column: None,
    })
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
async fn ordinary_sqlite_query_succeeds_without_spatial_extension() {
    let uri = database_uri();
    let holder = Connection::open(&uri).unwrap();
    holder
        .execute_batch("CREATE TABLE sample(value INTEGER); INSERT INTO sample VALUES (7);")
        .unwrap();

    let (status, body) = post(app(uri, None), "SELECT value FROM sample").await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, r#"{"Table":[["7"]]}"#);
}

#[tokio::test]
async fn configured_extension_supports_spatial_queries_but_not_query_controlled_loading() {
    let extension = std::env::var("SDA_TEST_SPATIALITE")
        .expect("SDA_TEST_SPATIALITE must be set for configured spatial acceptance");
    let uri = database_uri();
    let holder = Connection::open(&uri).unwrap();
    {
        let _guard = unsafe { rusqlite::LoadExtensionGuard::new(&holder) }.unwrap();
        unsafe { holder.load_extension(&extension, None) }.unwrap();
    }
    holder
        .execute_batch(
            "CREATE TABLE gpkg_geometry_columns (
                 table_name TEXT, column_name TEXT, geometry_type_name TEXT,
                 srs_id INTEGER, z TINYINT, m TINYINT
             );
             INSERT INTO gpkg_geometry_columns
                 VALUES ('features', 'shape', 'POINT', 4326, 0, 0);
             CREATE TABLE features(id INTEGER, shape BLOB);
             INSERT INTO features VALUES
               (1, AsGPB(GeomFromText('POINT(0 0)', 4326))),
               (2, AsGPB(GeomFromText('POINT(1 1)', 4326)));",
        )
        .unwrap();

    let router = app(uri, Some(extension));
    let (status, body) = post(
        router.clone(),
        "DECLARE @aoi geometry; \
         SET @aoi = geometry::STGeomFromText('POINT(0 0)', 4326); \
         SELECT id FROM features WHERE shape.STIntersects(@aoi) = 1",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, r#"{"Table":[["1"]]}"#);

    let (status, body) = post(router, "SELECT load_extension('not-a-trusted-path')").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{body}");
    assert!(
        body.contains("load_extension") || body.contains("not authorized"),
        "{body}"
    );
}
