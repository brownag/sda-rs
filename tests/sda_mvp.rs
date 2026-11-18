use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use rusqlite::Connection;
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};
use tower::ServiceExt;

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn router_keeps_nonspatial_query_working_and_reports_spatial_unavailable() {
    let path = std::env::temp_dir().join(format!(
        "sda-mvp-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE legend(areasymbol TEXT, areaname TEXT);
         CREATE TABLE mapunit(mukey TEXT);
         CREATE TABLE component(cokey TEXT, mukey TEXT);
         CREATE TABLE chorizon(chkey TEXT, cokey TEXT);
         CREATE TABLE muaggatt(mukey TEXT);
         CREATE TABLE mupolygon(mukey TEXT, geom_custom BLOB);
         CREATE TABLE gpkg_geometry_columns(table_name TEXT, column_name TEXT, srs_id INTEGER);
         INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'soil_shape', 26910);
         INSERT INTO gpkg_geometry_columns VALUES ('mupolygon', 'geom_custom', 4326);
         INSERT INTO legend VALUES ('CA001', 'Fixture');
         INSERT INTO mapunit VALUES ('1');
         INSERT INTO component VALUES ('10', '1');
         INSERT INTO chorizon VALUES ('100', '10');
         INSERT INTO muaggatt VALUES ('1');
         INSERT INTO mupolygon VALUES ('1', NULL);",
        )
        .unwrap();
    drop(connection);

    let config = sda_rs::config::Config {
        backend_url: path.to_string_lossy().into_owned(),
        port: 0,
        spatial_extension: None,
        geometry_column: None,
    };
    let app = sda_rs::app(config);
    let ordinary = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"query":"SELECT areaname FROM legend"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(ordinary.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(ordinary.into_body(), usize::MAX).await.unwrap(),
        r#"{"Table":[["Fixture"]]}"#
    );

    let spatial = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Tabular/post.rest")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"query":"SELECT mukey FROM mapunit WHERE mukey IN (SELECT SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)'))"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(spatial.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = to_bytes(spatial.into_body(), usize::MAX).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("Spatial capability unavailable"));
    fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn router_rolls_back_failed_statement_batch() {
    let uri = format!(
        "file:g16_router_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let holder = Connection::open(&uri).unwrap();
    holder
        .execute_batch("CREATE TABLE t(x INTEGER); INSERT INTO t VALUES (1);")
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
                .body(Body::from(
                    r#"{"query":"UPDATE t SET x = 2; SELECT * FROM missing"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let value: i64 = holder
        .query_row("SELECT x FROM t", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, 1);
}

#[test]
fn mukey_rewrite_uses_mupolygon_metadata_not_first_geometry_row() {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(
            "CREATE TABLE gpkg_geometry_columns(table_name TEXT, column_name TEXT, srs_id INTEGER);
             INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'soil_shape', 26910);
             INSERT INTO gpkg_geometry_columns VALUES ('mupolygon', 'geom_custom', 3857);",
        )
        .unwrap();
    let capability = sda_rs::spatial::initialize(&connection, None, None).unwrap();
    let sql = sda_rs::transpiler::transpile_with_spatial(
        "SELECT mukey FROM mapunit WHERE mukey IN (SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)'))",
        Some(&capability),
    )
    .unwrap()
    .remove(0)
    .to_ascii_uppercase();
    assert!(sql.contains("MP.\"GEOM_CUSTOM\""), "{sql}");
    assert!(
        sql.contains("ST_TRANSFORM(GEOMFROMGPB(MP.\"GEOM_CUSTOM\"), 3857)"),
        "{sql}"
    );
    assert!(!sql.contains("SOIL_SHAPE"), "{sql}");
}

#[tokio::test]
async fn spatial_tvf_executes_when_test_extension_is_configured() {
    let Ok(extension) = std::env::var("SDA_TEST_SPATIALITE") else {
        eprintln!("SDA_TEST_SPATIALITE not set; live spatial execution not tested");
        return;
    };
    let path = std::env::temp_dir().join(format!(
        "sda-mvp-spatial-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let connection = Connection::open(&path).unwrap();
    {
        let _guard = unsafe { rusqlite::LoadExtensionGuard::new(&connection) }.unwrap();
        unsafe { connection.load_extension(&extension, None) }.unwrap();
    }
    let initialized: i32 = connection
        .query_row("SELECT InitSpatialMetaData(1)", [], |row| row.get(0))
        .unwrap();
    assert_eq!(initialized, 1, "SpatiaLite metadata initialization failed");
    connection
        .execute_batch(
            "CREATE TABLE gpkg_geometry_columns(table_name TEXT, column_name TEXT, srs_id INTEGER);
         INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'soil_shape', 26910);
         INSERT INTO gpkg_geometry_columns VALUES ('mupolygon', 'shape_custom', 4326);
         CREATE TABLE mupolygon(mukey TEXT, areasymbol TEXT, shape_custom BLOB);
         CREATE TABLE mapunit(mukey TEXT);
         INSERT INTO mapunit VALUES ('1'), ('2'), ('3');
         INSERT INTO mupolygon VALUES
           ('1', 'CA001', AsGPB(GeomFromText('POINT(0 0)', 4326))),
           ('2', 'US', AsGPB(GeomFromText('POINT(0 0)', 4326))),
           ('3', 'CA001', AsGPB(GeomFromText('POINT(1 1)', 4326)));",
        )
        .unwrap();
    drop(connection);
    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: path.to_string_lossy().into_owned(),
        port: 0,
        spatial_extension: Some(extension),
        geometry_column: None,
    });
    for query in [
        "SELECT mukey FROM mapunit WHERE mukey IN (SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)'))",
        "SELECT k.mukey FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)') k",
        "DECLARE @aoi geometry; SET @aoi = geometry::STGeomFromText('POINT(0 0)', 4326); SELECT mukey FROM mupolygon WHERE areasymbol <> 'US' AND shape_custom.STIntersects(@aoi) = 1",
    ] {
        let body = format!(r#"{{"query":"{query}"}}"#);
        let response = app.clone().oneshot(
            Request::builder().method("POST").uri("/Tabular/post.rest")
                .header("content-type", "application/json").body(Body::from(body)).unwrap(),
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(to_bytes(response.into_body(), usize::MAX).await.unwrap(), r#"{"Table":[["1"]]}"#);
    }
    fs::remove_file(path).unwrap();
}
