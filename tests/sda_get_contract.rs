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
        "file:sda_get_contract_{}_{}?mode=memory&cache=shared",
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
                .body(Body::from(format!(
                    r#"{{"query":"{sql}","format":"JSON+COLUMNNAME"}}"#
                )))
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
async fn active_intersection_tvfs_return_expected_keys_for_both_crs() {
    let extension = std::env::var("SDA_TEST_SPATIALITE")
        .expect("SDA_TEST_SPATIALITE must be set for configured spatial acceptance");
    let uri = database_uri();
    let holder = Connection::open(&uri).unwrap();
    {
        let _guard = unsafe { rusqlite::LoadExtensionGuard::new(&holder) }.unwrap();
        unsafe { holder.load_extension(&extension, None) }.unwrap();
    }
    assert_eq!(
        holder
            .query_row("SELECT InitSpatialMetaData(1)", [], |row| {
                row.get::<_, i32>(0)
            })
            .unwrap(),
        1
    );
    holder
        .execute_batch(
            "CREATE TABLE gpkg_geometry_columns (
                 table_name TEXT, column_name TEXT, geometry_type_name TEXT,
                 srs_id INTEGER, z TINYINT, m TINYINT
             );
              INSERT INTO gpkg_geometry_columns VALUES
                  ('mupolygon', 'mu_shape', 'POLYGON', 4326, 0, 0),
                   ('sapolygon', 'sa_shape', 'POLYGON', 4326, 0, 0);
                CREATE TABLE mupolygon(mukey TEXT, mupolygonkey INTEGER, areasymbol TEXT, mu_shape BLOB);
               CREATE TABLE sapolygon(sapolygonkey INTEGER, areasymbol TEXT, sa_shape BLOB);
               CREATE TABLE sastatusmap(areasymbol TEXT);
             INSERT INTO mupolygon VALUES
                  ('inside', 11, 'CA001', AsGPB(GeomFromText(
                    'POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))', 4326))),
                  ('outside', 22, 'CA001', AsGPB(GeomFromText(
                    'POLYGON((9 9, 11 9, 11 11, 9 11, 9 9))', 4326))),
                  ('statgo', 33, 'US', AsGPB(GeomFromText(
                    'POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))', 4326)));
               INSERT INTO sapolygon VALUES
                  (101, 'CA001', AsGPB(GeomFromText(
                    'POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))', 4326))),
                   (202, 'CA002', AsGPB(GeomFromText(
                     'POLYGON((9 9, 11 9, 11 11, 9 11, 9 9))', 4326))),
                   (303, 'CA003', AsGPB(GeomFromText(
                     'POLYGON((19 19, 21 19, 21 21, 19 21, 19 19))', 4326)));
               INSERT INTO sastatusmap VALUES ('CA001'), ('CA001'), ('CA002');",
        )
        .unwrap();
    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: uri,
        port: 0,
        spatial_extension: Some(extension),
        geometry_column: None,
    });

    for (sql, expected) in [
        (
            "SELECT mukey, typeof(mukey) AS key_type FROM \
             SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0.5 0.5)')",
            r#"{"Table":[["mukey","key_type"],["inside","text"]]}"#,
        ),
        (
            "SELECT mukey, typeof(mukey) AS key_type FROM \
             SDA_Get_Mukey_from_intersection_with_WktWm('POINT(55659.745 55660.451)')",
            r#"{"Table":[["mukey","key_type"],["inside","text"]]}"#,
        ),
        (
            "SELECT mupolygonkey, typeof(mupolygonkey) AS key_type FROM \
             SDA_Get_Mupolygonkey_from_intersection_with_WktWgs84('POINT(0.5 0.5)')",
            r#"{"Table":[["mupolygonkey","key_type"],["11","integer"]]}"#,
        ),
        (
            "SELECT mupolygonkey, typeof(mupolygonkey) AS key_type FROM \
             SDA_Get_Mupolygonkey_from_intersection_with_WktWm('POINT(55659.745 55660.451)')",
            r#"{"Table":[["mupolygonkey","key_type"],["11","integer"]]}"#,
        ),
        (
            "SELECT sapolygonkey, typeof(sapolygonkey) AS key_type FROM \
             SDA_Get_Sapolygonkey_from_intersection_with_WktWgs84('POINT(0.5 0.5)')",
            r#"{"Table":[["sapolygonkey","key_type"],["101","integer"]]}"#,
        ),
        (
            "SELECT sapolygonkey, typeof(sapolygonkey) AS key_type FROM \
             SDA_Get_Sapolygonkey_from_intersection_with_WktWm('POINT(55659.745 55660.451)')",
            r#"{"Table":[["sapolygonkey","key_type"],["101","integer"]]}"#,
        ),
        (
            "SELECT areasymbol FROM \
             SDA_Get_Areasymbol_from_intersection_with_WktWgs84('POINT(0.5 0.5)')",
            r#"{"Table":[["areasymbol"],["CA001"]]}"#,
        ),
        (
            "SELECT areasymbol FROM \
             SDA_Get_Areasymbol_from_intersection_with_WktWm('POINT(55659.745 55660.451)')",
            r#"{"Table":[["areasymbol"],["CA001"]]}"#,
        ),
        (
            "SELECT sapolygonkey, typeof(sapolygonkey) AS key_type FROM \
             SDA_Get_Sapolygonkey_from_intersection_with_sastatusmap_WktWgs84(\
               'MULTIPOINT((0.5 0.5),(20 20))')",
            r#"{"Table":[["sapolygonkey","key_type"],["101","integer"]]}"#,
        ),
        (
            "SELECT sapolygonkey, typeof(sapolygonkey) AS key_type FROM \
             SDA_Get_Sapolygonkey_from_intersection_with_sastatusmap_WktWm(\
               'MULTIPOINT((55659.745 55660.451),(2226389.816 2273030.927))')",
            r#"{"Table":[["sapolygonkey","key_type"],["101","integer"]]}"#,
        ),
    ] {
        assert_eq!(query(&app, sql).await, expected);
    }
    for sql in [
        "SELECT mupolygonkey FROM \
         SDA_Get_Mupolygonkey_from_intersection_with_WktWgs84('POINT(5 5)')",
        "SELECT mupolygonkey FROM \
         SDA_Get_Mupolygonkey_from_intersection_with_WktWm('POINT(556597.45 557305.257)')",
    ] {
        // The SDA JSON serializer omits result sets with no rows, even when
        // the SQL result has a known column shape.
        assert_eq!(query(&app, sql).await, "{}");
    }
}
