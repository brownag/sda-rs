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
    let status = response.status();
    let body = String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert_eq!(status, StatusCode::OK, "SQL: {sql}\nResponse body: {body}");
    body
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
             INSERT INTO gpkg_geometry_columns
                (table_name, column_name, geometry_type_name, srs_id, z, m) VALUES
                    ('mupolygon', 'mu_shape', 'POLYGON', 4326, 0, 0),
                     ('sapolygon', 'sa_shape', 'POLYGON', 4326, 0, 0),
                     ('mupoint', 'mp_shape', 'POINT', 4326, 0, 0),
                    ('muline', 'ml_shape', 'LINESTRING', 4326, 0, 0),
                    ('featline', 'fl_shape', 'LINESTRING', 4326, 0, 0),
                    ('featpoint', 'fp_shape', 'POINT', 4326, 0, 0);
                 CREATE TABLE mupolygon(mukey TEXT, mupolygonkey INTEGER, areasymbol TEXT, mu_shape BLOB);
                CREATE TABLE sapolygon(sapolygonkey INTEGER, areasymbol TEXT, sa_shape BLOB);
                CREATE TABLE mupoint(mupointkey INTEGER, mukey TEXT, mp_shape BLOB);
                CREATE TABLE muline(mulinekey INTEGER, mukey TEXT, ml_shape BLOB);
                CREATE TABLE featline(featlinekey INTEGER, fl_shape BLOB);
                CREATE TABLE featpoint(featpointkey INTEGER, fp_shape BLOB);
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
                INSERT INTO mupoint VALUES
                    (401, 'inside', AsGPB(GeomFromText('POINT(0.5 0.5)', 4326)));
                INSERT INTO muline VALUES
                    (501, 'inside', AsGPB(GeomFromText('LINESTRING(0 0, 1 1)', 4326)));
                INSERT INTO featline VALUES
                    (601, AsGPB(GeomFromText('LINESTRING(0 0, 1 1)', 4326)));
                INSERT INTO featpoint VALUES
                    (701, AsGPB(GeomFromText('POINT(0.5 0.5)', 4326)));
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
        (
            "SELECT mupointkey, typeof(mupointkey) AS key_type FROM \
             SDA_Get_Mupointkey_from_intersection_with_WktWgs84('POINT(0.5 0.5)')",
            r#"{"Table":[["mupointkey","key_type"],["401","integer"]]}"#,
        ),
        (
            "SELECT mupointkey, typeof(mupointkey) AS key_type FROM \
             SDA_Get_Mupointkey_from_intersection_with_WktWm(\
               'POINT(55659.745396636783 55660.451865421521)')",
            r#"{"Table":[["mupointkey","key_type"],["401","integer"]]}"#,
        ),
        (
            "SELECT mulinekey, typeof(mulinekey) AS key_type FROM \
             SDA_Get_Mulinekey_from_intersection_with_WktWgs84('POINT(0.5 0.5)')",
            r#"{"Table":[["mulinekey","key_type"],["501","integer"]]}"#,
        ),
        (
            "SELECT mulinekey, typeof(mulinekey) AS key_type FROM \
             SDA_Get_Mulinekey_from_intersection_with_WktWm(\
               'LINESTRING(55659 55000, 55659 56000)')",
            r#"{"Table":[["mulinekey","key_type"],["501","integer"]]}"#,
        ),
        (
            "SELECT featlinekey, typeof(featlinekey) AS key_type FROM \
             SDA_Get_Featlinekey_from_intersection_with_WktWgs84('POINT(0.5 0.5)')",
            r#"{"Table":[["featlinekey","key_type"],["601","integer"]]}"#,
        ),
        (
            "SELECT featlinekey, typeof(featlinekey) AS key_type FROM \
             SDA_Get_Featlinekey_from_intersection_with_WktWm(\
               'LINESTRING(55659 55000, 55659 56000)')",
            r#"{"Table":[["featlinekey","key_type"],["601","integer"]]}"#,
        ),
        (
            "SELECT featpointkey, typeof(featpointkey) AS key_type FROM \
             SDA_Get_Featpointkey_from_intersection_with_WktWgs84('POINT(0.5 0.5)')",
            r#"{"Table":[["featpointkey","key_type"],["701","integer"]]}"#,
        ),
        (
            "SELECT featpointkey, typeof(featpointkey) AS key_type FROM \
             SDA_Get_Featpointkey_from_intersection_with_WktWm(\
               'POINT(55659.745396636783 55660.451865421521)')",
            r#"{"Table":[["featpointkey","key_type"],["701","integer"]]}"#,
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
    for sql in [
        "SELECT featpointkey FROM SDA_Get_Featpointkey_from_intersection_with_WktWgs84('POINT(5 5)')",
        "SELECT featpointkey FROM SDA_Get_Featpointkey_from_intersection_with_WktWm('POINT(556597.45 557305.257)')",
    ] {
        assert_eq!(query(&app, sql).await, "{}");
    }

    for (sql, alias, table, geometry_column, key_column, key, srid) in [
        (
            "SELECT * FROM SDA_Get_AreasymbolWKTWgs84_from_Areasymbol('CA001')",
            "AreasymbolWkt84",
            "sapolygon",
            "sa_shape",
            "areasymbol",
            "CA001",
            4326,
        ),
        (
            "SELECT * FROM SDA_Get_AreasymbolWKTWm_from_Areasymbol('CA001')",
            "AreasymbolWktWm",
            "sapolygon",
            "sa_shape",
            "areasymbol",
            "CA001",
            3857,
        ),
        (
            "SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey('inside')",
            "MupolygonWktWgs84",
            "mupolygon",
            "mu_shape",
            "mukey",
            "inside",
            4326,
        ),
        (
            "SELECT * FROM SDA_Get_MupolygonWktWm_from_Mukey('inside')",
            "MupolygonWktWm",
            "mupolygon",
            "mu_shape",
            "mukey",
            "inside",
            3857,
        ),
        (
            "SELECT * FROM SDA_Get_MupointWktWgs84_from_Mukey('inside')",
            "MupointWktWgs84",
            "mupoint",
            "mp_shape",
            "mukey",
            "inside",
            4326,
        ),
        (
            "SELECT * FROM SDA_Get_MupointWktWm_from_Mukey('inside')",
            "MupointWktWm",
            "mupoint",
            "mp_shape",
            "mukey",
            "inside",
            3857,
        ),
        (
            "SELECT * FROM SDA_Get_MulineWktWgs84_from_Mukey('inside')",
            "MulineWktWgs84",
            "muline",
            "ml_shape",
            "mukey",
            "inside",
            4326,
        ),
        (
            "SELECT * FROM SDA_Get_MulineWktWm_from_Mukey('inside')",
            "MulineWktWm",
            "muline",
            "ml_shape",
            "mukey",
            "inside",
            3857,
        ),
    ] {
        let wkt: String = holder
            .query_row(
                &format!(
                    "SELECT AsText(ST_Transform(GeomFromGPB(\"{geometry_column}\"), ?1)) \
                     FROM \"{table}\" WHERE \"{key_column}\" = ?2 LIMIT 1"
                ),
                rusqlite::params![srid, key],
                |row| row.get(0),
            )
            .unwrap();
        let expected = format!(r#"{{"Table":[["{alias}"],["{wkt}"]]}}"#);
        assert_eq!(query(&app, sql).await, expected);
    }

    holder
        .execute(
            "INSERT INTO mupolygon VALUES
             ('11', 11, 'CA001', AsGPB(GeomFromText(
               'POLYGON((-1 -1, 1 -1, 1 1, -1 1, -1 -1))', 4326)))",
            [],
        )
        .unwrap();
    let mukey_wkt: String = holder
        .query_row(
            "SELECT AsText(ST_Transform(GeomFromGPB(mu_shape), 4326)) \
             FROM mupolygon WHERE mukey = '11' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    for sql in [
        "SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey('inside')",
        "SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey(11)",
        "SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey('11')",
    ] {
        assert_eq!(
            query(&app, sql).await,
            format!(r#"{{"Table":[["MupolygonWktWgs84"],["{mukey_wkt}"]]}}"#)
        );
    }

    for sql in [
        "SELECT * FROM SDA_Get_AreasymbolWKTWgs84_from_Areasymbol('ZZ999')",
        "SELECT * FROM SDA_Get_AreasymbolWKTWm_from_Areasymbol(NULL)",
        "SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey('missing')",
        "SELECT * FROM SDA_Get_MupolygonWktWm_from_Mukey(NULL)",
        "SELECT * FROM SDA_Get_MupointWktWgs84_from_Mukey('missing')",
        "SELECT * FROM SDA_Get_MupointWktWm_from_Mukey(NULL)",
        "SELECT * FROM SDA_Get_MulineWktWgs84_from_Mukey('missing')",
        "SELECT * FROM SDA_Get_MulineWktWm_from_Mukey(NULL)",
    ] {
        assert_eq!(query(&app, sql).await, "{}");
    }

    holder
        .execute(
            "INSERT INTO mupoint VALUES
             (402, '11', AsGPB(GeomFromText('POINT(0.5 0.5)', 4326)))",
            [],
        )
        .unwrap();
    holder
        .execute(
            "INSERT INTO muline VALUES
             (502, '11', AsGPB(GeomFromText('LINESTRING(0 0, 1 1)', 4326)))",
            [],
        )
        .unwrap();

    for (table_sql, first_column, second_column, values) in [
        (
            "~DeclareVarcharTable(@areas)~ INSERT INTO @areas VALUES ('CA001'); \
             SELECT * FROM SDA_Get_AreasymbolWktWgs84_from_AreasymbolTable(@areas)",
            "areasymbol",
            "AreasymbolWkt84",
            "CA001",
        ),
        (
            "~DeclareVarcharTable(@areas)~ INSERT INTO @areas VALUES ('CA001'); \
             SELECT * FROM SDA_Get_AreasymbolWktWm_from_AreasymbolTable(@areas)",
            "areasymbol",
            "AreasymbolWktWm",
            "CA001",
        ),
        (
            "~DeclareIntTable(@mu)~ INSERT INTO @mu VALUES (11); \
             SELECT * FROM SDA_Get_MupolygonWktWgs84_from_MukeyTable(@mu)",
            "mukey",
            "MupolygonWktWgs84",
            "11",
        ),
        (
            "~DeclareIntTable(@mu)~ INSERT INTO @mu VALUES (11); \
             SELECT * FROM SDA_Get_MupolygonWktWm_from_MukeyTable(@mu)",
            "mukey",
            "MupolygonWktWm",
            "11",
        ),
        (
            "~DeclareIntTable(@mu)~ INSERT INTO @mu VALUES (11); \
             SELECT * FROM SDA_Get_MupointWktWgs84_from_MukeyTable(@mu)",
            "mukey",
            "MupointWktWgs84",
            "11",
        ),
        (
            "~DeclareIntTable(@mu)~ INSERT INTO @mu VALUES (11); \
             SELECT * FROM SDA_Get_MupointWktWm_from_MukeyTable(@mu)",
            "mukey",
            "MupointWktWm",
            "11",
        ),
        (
            "~DeclareIntTable(@mu)~ INSERT INTO @mu VALUES (11); \
             SELECT * FROM SDA_Get_MulineWktWgs84_from_MukeyTable(@mu)",
            "mukey",
            "MulineWktWgs84",
            "11",
        ),
        (
            "~DeclareIntTable(@mu)~ INSERT INTO @mu VALUES (11); \
             SELECT * FROM SDA_Get_MulineWktWm_from_MukeyTable(@mu)",
            "mukey",
            "MulineWktWm",
            "11",
        ),
    ] {
        let expected = if first_column == "areasymbol" {
            let wkt: String = holder
                .query_row(
                    &format!(
                        "SELECT AsText(ST_Transform(GeomFromGPB(sa_shape), {})) \
                         FROM sapolygon WHERE areasymbol = ?1 LIMIT 1",
                        if second_column.ends_with("84") {
                            4326
                        } else {
                            3857
                        }
                    ),
                    [values],
                    |row| row.get(0),
                )
                .unwrap();
            format!(r#"{{"Table":[["{first_column}","{second_column}"],["CA001","{wkt}"]]}}"#)
        } else {
            let (table, geometry) = if second_column.starts_with("Mupolygon") {
                ("mupolygon", "mu_shape")
            } else if second_column.starts_with("Mupoint") {
                ("mupoint", "mp_shape")
            } else {
                ("muline", "ml_shape")
            };
            let srid = if second_column.ends_with("Wgs84") {
                4326
            } else {
                3857
            };
            let wkt: String = holder
                .query_row(
                    &format!(
                        "SELECT AsText(ST_Transform(GeomFromGPB({geometry}), {srid})) \
                         FROM {table} WHERE mukey = '11' LIMIT 1"
                    ),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            format!(r#"{{"Table":[["{first_column}","{second_column}"],["11","{wkt}"]]}}"#)
        };
        assert_eq!(query(&app, table_sql).await, expected, "{table_sql}");
    }

    for sql in [
        "~DeclareVarcharTable(@areas)~ SELECT * FROM \
         SDA_Get_AreasymbolWktWgs84_from_AreasymbolTable(@areas)",
        "~DeclareIntTable(@mu)~ SELECT * FROM \
         SDA_Get_MupolygonWktWgs84_from_MukeyTable(@mu)",
    ] {
        assert_eq!(query(&app, sql).await, "{}");
    }
}
