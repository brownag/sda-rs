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
        "file:spatial_methods_contract_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

async fn query(app: axum::Router, sql: &str) -> (StatusCode, String) {
    let response = app
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
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn configured_spatial_methods_execute_supported_predicate_and_reject_unsupported_methods() {
    let Ok(extension) = std::env::var("SDA_TEST_SPATIALITE") else {
        eprintln!("SDA_TEST_SPATIALITE not set; skipping configured spatial methods contract");
        return;
    };
    let uri = database_uri();
    let holder = Connection::open(&uri).unwrap();
    {
        let _guard = unsafe { rusqlite::LoadExtensionGuard::new(&holder) }.unwrap();
        unsafe { holder.load_extension(&extension, None) }.unwrap();
    }
    assert_eq!(
        holder
            .query_row("SELECT InitSpatialMetaData(1)", [], |row| row
                .get::<_, i32>(0))
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
                 VALUES ('features', 'shape', 'POINT', 4326, 0, 0);
             CREATE TABLE features(id INTEGER, shape BLOB);
             INSERT INTO features VALUES
                 (1, AsGPB(GeomFromText('POINT(0 0)', 4326))),
                 (2, AsGPB(GeomFromText('POINT(2 2)', 4326)));",
        )
        .unwrap();

    let app = sda_rs::app(sda_rs::config::Config {
        backend_url: uri,
        port: 0,
        spatial_extension: Some(extension),
        geometry_column: None,
    });

    // Existing `.STIntersects` is exercised as a configured SpatiaLite
    // operation only; this does not claim SQL Server geography parity.
    let (status, body) = query(
        app.clone(),
        "SELECT id FROM features WHERE shape.STIntersects(geometry::STGeomFromText('POINT(0 0)', 4326)) = 1",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, r#"{"Table":[["1"]]}"#);

    let (status, body) = query(
        app.clone(),
        "SELECT id FROM features WHERE shape.STIntersects(geometry::STGeomFromText('POINT(9 9)', 4326)) = 1",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body, "{}");

    // G37 Sections 6.1-6.2: no exact transformation is established from the
    // available evidence for these predicates, so every receiver/arity shape
    // must be rejected before any backend execution. This asserts the
    // rejection contract only; it is not a SQL Server runtime-parity claim.
    for method in ["STContains", "stcontains", "STWithin", "StWithin"] {
        for receiver in ["shape", "geometry::STGeomFromText('POINT(0 0)', 4326)"] {
            for arguments in [
                "()",
                "(geometry::STGeomFromText('POINT(0 0)', 4326))",
                "(geometry::STGeomFromText('POINT(0 0)', 4326), 1)",
            ] {
                let sql = format!("SELECT {receiver}.{method}{arguments} FROM features");
                let (status, body) = query(app.clone(), &sql).await;
                assert_eq!(status, StatusCode::BAD_REQUEST, "{sql}: {body}");
                let lowered = body.to_ascii_lowercase();
                assert!(
                    lowered.contains(&format!(
                        ".{} is unsupported for sql server geography compatibility",
                        method.to_ascii_lowercase()
                    )),
                    "{sql}: {body}"
                );
                assert!(
                    body.contains("no exact transformation established from available evidence"),
                    "{sql}: {body}"
                );
                assert!(
                    body.contains("SQL Server's boundary rule is not established"),
                    "{sql}: {body}"
                );
                assert!(!lowered.contains("st_contains"), "{sql}: {body}");
                assert!(!lowered.contains("st_within"), "{sql}: {body}");
                assert!(!lowered.contains("no such function"), "{sql}: {body}");
            }
        }
    }

    for method in ["STIntersection"] {
        let sql = format!(
            "SELECT shape.{method}(geometry::STGeomFromText('POINT(0 0)', 4326)) FROM features"
        );
        let (status, body) = query(app.clone(), &sql).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{method}: {body}");
        assert!(
            body.contains(&format!(
                ".{method} is unsupported for SQL Server geography compatibility"
            )),
            "{method}: {body}"
        );
        assert!(!body.contains("no such function"), "{method}: {body}");
    }

    for (method, arguments) in [
        ("STBuffer", ["(1)", "(1, 2)"]),
        ("STArea", ["()", "(1)"]),
        ("STAsText", ["()", "(1)"]),
    ] {
        for receiver in ["shape", "geometry::STGeomFromText('POINT(0 0)', 4326)"] {
            for args in arguments {
                let sql = format!("SELECT {receiver}.{method}{args} FROM features");
                let (status, body) = query(app.clone(), &sql).await;
                assert_eq!(status, StatusCode::BAD_REQUEST, "{sql}: {body}");
                assert!(
                    body.contains(&format!(
                        ".{method} is unsupported for SQL Server geography compatibility"
                    )),
                    "{sql}: {body}"
                );
                assert!(!body.contains("no such function"), "{sql}: {body}");
            }
        }
    }
}
