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
        "file:soildb_query_fixtures_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

async fn query(app: &axum::Router, sql: &str) -> (StatusCode, String) {
    let escaped_sql = sql
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"query":"{escaped_sql}","format":"JSON+COLUMNNAME+METADATA"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    (status, body)
}

fn app(uri: String, extension: Option<String>) -> axum::Router {
    sda_rs::app(sda_rs::config::Config {
        backend_url: uri,
        port: 0,
        spatial_extension: extension,
        geometry_column: None,
    })
}

#[tokio::test]
async fn captured_muaggatt_join_runs_without_soildb_or_spatialite() {
    let uri = database_uri();
    let holder = Connection::open(&uri).unwrap();
    holder
        .execute_batch(
            "CREATE TABLE legend(lkey TEXT, areasymbol TEXT);
             CREATE TABLE mapunit(lkey TEXT, mukey TEXT);
             CREATE TABLE muaggatt(mukey TEXT, representative_value TEXT);
             INSERT INTO legend VALUES ('l1', 'IA001');
             INSERT INTO mapunit VALUES ('l1', '1857266');
             INSERT INTO muaggatt VALUES ('1857266', 'fixture value');",
        )
        .unwrap();

    let (status, body) = query(
        &app(uri, None),
        include_str!("fixtures/soildb/get-sda-muaggatt.sql"),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        r#"{"Table":[["mukey","mukey","representative_value"],["ColumnOrdinal=0,DataTypeName=varchar,ProviderType=22","ColumnOrdinal=1,DataTypeName=varchar,ProviderType=22","ColumnOrdinal=2,DataTypeName=varchar,ProviderType=22"],["1857266","1857266","fixture value"]]}"#
    );
}

#[tokio::test]
async fn captured_spatial_query_ctes_run_without_soildb() {
    let Ok(extension) = std::env::var("SDA_TEST_SPATIALITE") else {
        eprintln!("SDA_TEST_SPATIALITE not set; skipping captured spatial query fixtures");
        return;
    };
    let uri = database_uri();
    let holder = Connection::open(&uri).unwrap();
    {
        let _guard = unsafe { rusqlite::LoadExtensionGuard::new(&holder) }.unwrap();
        unsafe { holder.load_extension(&extension, None) }.unwrap();
    }
    holder
        .execute_batch(
            "SELECT InitSpatialMetaData(1);
             CREATE TABLE gpkg_geometry_columns (
                 table_name TEXT, column_name TEXT, geometry_type_name TEXT,
                 srs_id INTEGER, z TINYINT, m TINYINT
             );
             INSERT INTO gpkg_geometry_columns VALUES
                 ('mupolygon', 'geom', 'POLYGON', 4326, 0, 0),
                 ('sapolygon', 'geom', 'POLYGON', 4326, 0, 0);
             CREATE TABLE mupolygon(mukey TEXT, muname TEXT, areasymbol TEXT, geom BLOB);
             CREATE TABLE mapunit(mukey TEXT, muname TEXT);
             CREATE TABLE sapolygon(sapolygonkey INTEGER, areasymbol TEXT, geom BLOB);
             INSERT INTO mapunit VALUES ('1857266', 'Inside map unit');
             INSERT INTO mupolygon VALUES (
                 '1857266', 'Inside map unit', 'IA001',
                 AsGPB(GeomFromText('POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))', 4326))
             );
             INSERT INTO sapolygon VALUES (
                 101, 'IA001',
                 AsGPB(GeomFromText('POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))', 4326))
             );",
        )
        .unwrap();

    let app = app(uri, Some(extension));
    for (sql, expected) in [
        (
            include_str!("fixtures/soildb/sda-spatial-mukey.sql"),
            r#"{"Table":[["mukey","muname"],["ColumnOrdinal=0,DataTypeName=varchar,ProviderType=22","ColumnOrdinal=1,DataTypeName=varchar,ProviderType=22"],["1857266","Inside map unit"]]}"#,
        ),
        (
            include_str!("fixtures/soildb/sda-spatial-areasymbol.sql"),
            r#"{"Table":[["areasymbol"],["ColumnOrdinal=0,DataTypeName=varchar,ProviderType=22"],["IA001"]]}"#,
        ),
    ] {
        let (status, body) = query(&app, sql).await;
        assert_eq!(status, StatusCode::OK, "SQL: {sql}\n{body}");
        assert_eq!(body, expected, "SQL: {sql}");
    }
}
