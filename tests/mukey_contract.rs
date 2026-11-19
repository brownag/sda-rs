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
        "file:mukey_contract_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

async fn query(app: &axum::Router, sql: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"query":"{sql}"}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
async fn mukey_tvf_executes_in_where_and_from_contexts() {
    let Ok(extension) = std::env::var("SDA_TEST_SPATIALITE") else {
        eprintln!("SDA_TEST_SPATIALITE not set; skipping configured Mukey contract");
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
    assert_eq!(initialized, 1, "SpatiaLite metadata initialization failed");
    holder
        .execute_batch(
            "CREATE TABLE gpkg_geometry_columns (
                 table_name TEXT, column_name TEXT, geometry_type_name TEXT,
                 srs_id INTEGER, z TINYINT, m TINYINT
             );
             INSERT INTO gpkg_geometry_columns
                 VALUES ('mupolygon', 'shape_custom', 'POLYGON', 4326, 0, 0);
             CREATE TABLE mupolygon(mukey TEXT, areasymbol TEXT, shape_custom BLOB);
             CREATE TABLE mapunit(mukey TEXT);
             INSERT INTO mapunit VALUES ('inside'), ('outside'), ('statgo');
             INSERT INTO mupolygon VALUES
               ('inside', 'CA001',
                AsGPB(GeomFromText('POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))', 4326))),
               ('outside', 'CA001',
                AsGPB(GeomFromText('POLYGON((9 9, 11 9, 11 11, 9 11, 9 9))', 4326))),
               ('statgo', 'US',
                AsGPB(GeomFromText('POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))', 4326)));",
        )
        .unwrap();
    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: uri,
        port: 0,
        spatial_extension: Some(extension),
        geometry_column: None,
    });

    assert_eq!(
        query(
            &app,
            "SELECT mukey FROM mapunit WHERE mukey IN \
             (SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)'))",
        )
        .await,
        r#"{"Table":[["inside"]]}"#
    );
    assert_eq!(
        query(
            &app,
            "SELECT k.mukey FROM \
             SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)') AS k",
        )
        .await,
        r#"{"Table":[["inside"]]}"#
    );
}
