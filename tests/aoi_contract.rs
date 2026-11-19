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
        "file:aoi_contract_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

fn aoi_query() -> &'static str {
    "DECLARE @aoi geometry; SET @aoi = geometry::STGeomFromText('POINT(0 0)', 4326); SELECT id FROM features WHERE shape.STIntersects(@aoi) = 1"
}

#[tokio::test]
async fn aoi_variable_constructor_and_intersects_execute_with_spatialite() {
    let Ok(extension) = std::env::var("SDA_TEST_SPATIALITE") else {
        eprintln!("SDA_TEST_SPATIALITE not set; skipping configured AOI contract");
        return;
    };
    let uri = database_uri();
    let holder = Connection::open(&uri).unwrap();
    {
        let _guard = unsafe { rusqlite::LoadExtensionGuard::new(&holder) }.unwrap();
        unsafe { holder.load_extension(&extension, None) }.unwrap();
    }
    let initialized: i32 = holder
        .query_row("SELECT InitSpatialMetaData(1)", [], |row| row.get(0))
        .unwrap();
    assert_eq!(initialized, 1);
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

    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: uri,
        port: 0,
        spatial_extension: Some(extension),
        geometry_column: None,
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"query":"{}"}}"#, aoi_query())))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(response.into_body(), usize::MAX).await.unwrap(),
        r#"{"Table":[["1"]]}"#
    );
}

#[tokio::test]
async fn aoi_query_reports_structured_error_without_spatialite() {
    let uri = database_uri();
    let holder = Connection::open(&uri).unwrap();
    holder
        .execute_batch("CREATE TABLE features(id INTEGER, shape BLOB);")
        .unwrap();
    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: uri,
        port: 0,
        spatial_extension: None,
        geometry_column: None,
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"query":"{}"}}"#, aoi_query())))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        body,
        r#"{"error":"Spatial capability unavailable: SpatiaLite is not loaded"}"#
    );
}
