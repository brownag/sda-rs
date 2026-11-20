use std::{path::Path, process::Command};

use duckdb::{types::Value as DuckValue, Connection as DuckConnection};

const PINNED_SPATIAL_SHA256: &str =
    "8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc";

/// Open one in-memory connection with the locally provisioned official Spatial
/// artifact loaded by absolute path and autoload/autoinstall disabled.
///
/// Skip only when `SDA_DUCKDB_SPATIAL_EXTENSION` is unset. If it is set, an
/// absent path, a hash mismatch, or a failed load is a test failure: the
/// observations below are only evidence when they come from the pinned build.
fn connect() -> Option<DuckConnection> {
    let Ok(extension) = std::env::var("SDA_DUCKDB_SPATIAL_EXTENSION") else {
        eprintln!("SDA_DUCKDB_SPATIAL_EXTENSION not set; skipping DuckDB geography contract");
        return None;
    };

    let path = std::fs::canonicalize(&extension)
        .unwrap_or_else(|error| panic!("SDA_DUCKDB_SPATIAL_EXTENSION={extension}: {error}"));
    verify_pinned_artifact(&path);

    let connection = DuckConnection::open_in_memory().expect("open in-memory DuckDB connection");
    connection
        .execute_batch(
            "SET autoinstall_known_extensions = false;
             SET autoload_known_extensions = false;",
        )
        .expect("disable DuckDB extension autoload/autoinstall");

    let literal = path.display().to_string().replace('\'', "''");
    connection
        .execute_batch(&format!("LOAD '{literal}';"))
        .unwrap_or_else(|error| panic!("load pinned Spatial artifact {}: {error}", path.display()));

    // Best-effort observation only; some builds restrict extension metadata.
    match connection.query_row(
        "SELECT extension_name, extension_version FROM duckdb_extensions()
         WHERE extension_name = 'spatial'",
        [],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    ) {
        Ok((name, version)) => eprintln!(
            "observed pinned artifact {} sha256={PINNED_SPATIAL_SHA256} metadata=({name}, {version})",
            path.display()
        ),
        Err(error) => eprintln!("extension metadata unavailable after load: {error}"),
    }

    Some(connection)
}

fn verify_pinned_artifact(path: &Path) {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("run sha256sum for {}: {error}", path.display()));
    assert!(
        output.status.success(),
        "sha256sum failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("sha256sum output is UTF-8");
    let actual = stdout
        .split_whitespace()
        .next()
        .expect("sha256sum prints a digest");
    assert_eq!(
        actual,
        PINNED_SPATIAL_SHA256,
        "artifact identity mismatch for {}",
        path.display()
    );
    eprintln!(
        "verified pinned artifact {} sha256={actual}",
        path.display()
    );
}

fn value(connection: &DuckConnection, sql: &str) -> DuckValue {
    connection
        .query_row(sql, [], |row| row.get::<_, DuckValue>(0))
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
}

fn boolean(connection: &DuckConnection, sql: &str) -> bool {
    match value(connection, sql) {
        DuckValue::Boolean(value) => value,
        other => panic!("{sql}: expected BOOLEAN, got {other:?}"),
    }
}

const POLYGON: &str = "POLYGON((0 0, 2 0, 2 2, 0 2, 0 0))";

fn point(first: f64, second: f64) -> String {
    format!("ST_GeomFromText('POINT({first} {second})')")
}

#[test]
fn pinned_spatial_artifact_loads_and_answers_a_spatial_function() {
    let Some(connection) = connect() else {
        return;
    };

    let wkt: String = connection
        .query_row(
            "SELECT ST_AsText(ST_GeomFromText('POINT(1 2)'))",
            [],
            |row| row.get(0),
        )
        .expect("ST_AsText works on the loaded artifact");
    assert!(wkt.starts_with("POINT"), "{wkt}");
}

#[test]
fn duckdb_planar_predicates_are_boundary_exclusive_on_the_pinned_build() {
    let Some(connection) = connect() else {
        return;
    };

    // Observed on the pinned build: ST_Contains returns false for a point on
    // an edge and for a polygon vertex. The fetched DuckDB 2.0 Spatial page
    // instead states that ST_Contains also returns true on the boundary. That
    // documented-vs-observed divergence is itself recorded as evidence that
    // the documented rule cannot be relied on here; SQL Server's boundary rule
    // remains unestablished, so no equivalence is claimed either way.
    let contains_interior = format!(
        "SELECT ST_Contains(ST_GeomFromText('{POLYGON}'), {})",
        point(1.0, 1.0)
    );
    let contains_edge = format!(
        "SELECT ST_Contains(ST_GeomFromText('{POLYGON}'), {})",
        point(1.0, 0.0)
    );
    let contains_vertex = format!(
        "SELECT ST_Contains(ST_GeomFromText('{POLYGON}'), {})",
        point(0.0, 0.0)
    );
    let contains_exterior = format!(
        "SELECT ST_Contains(ST_GeomFromText('{POLYGON}'), {})",
        point(3.0, 3.0)
    );
    let within_interior = format!(
        "SELECT ST_Within({}, ST_GeomFromText('{POLYGON}'))",
        point(1.0, 1.0)
    );
    let within_exterior = format!(
        "SELECT ST_Within({}, ST_GeomFromText('{POLYGON}'))",
        point(3.0, 3.0)
    );

    assert!(boolean(&connection, &contains_interior), "interior point");
    assert!(
        !boolean(&connection, &contains_edge),
        "edge point observed boundary-exclusive"
    );
    assert!(
        !boolean(&connection, &contains_vertex),
        "polygon vertex observed boundary-exclusive"
    );
    assert!(!boolean(&connection, &contains_exterior), "exterior point");
    assert!(boolean(&connection, &within_interior), "interior point");
    assert!(!boolean(&connection, &within_exterior), "exterior point");

    for sql in [
        format!("SELECT ST_Contains(NULL, {})", point(1.0, 1.0)),
        format!("SELECT ST_Contains(ST_GeomFromText('{POLYGON}'), NULL)"),
        format!("SELECT ST_Within(NULL, ST_GeomFromText('{POLYGON}'))"),
    ] {
        assert!(
            matches!(value(&connection, &sql), DuckValue::Null),
            "{sql}: expected NULL propagation"
        );
    }
}

#[test]
fn duckdb_crs_mismatch_errors_instead_of_returning_null() {
    let Some(connection) = connect() else {
        return;
    };

    for sql in [
        format!(
            "SELECT ST_Contains(
                 ST_SetCRS(ST_GeomFromText('{POLYGON}'), 'EPSG:4326'),
                 ST_SetCRS({}, 'EPSG:3857'))",
            point(1.0, 1.0)
        ),
        format!(
            "SELECT ST_Within(
                 ST_SetCRS({}, 'EPSG:3857'),
                 ST_SetCRS(ST_GeomFromText('{POLYGON}'), 'EPSG:4326'))",
            point(1.0, 1.0)
        ),
    ] {
        let error = connection
            .query_row(&sql, [], |row| row.get::<_, DuckValue>(0))
            .expect_err("mismatched CRS must not become a boolean or NULL result");
        eprintln!("observed mismatched-CRS error: {error}");
    }
}

#[test]
fn duckdb_planar_overlay_and_buffer_are_observations_not_geography_compatibility() {
    let Some(connection) = connect() else {
        return;
    };

    // These describe only the pinned planar backend. SQL Server geography
    // overlay and its meter/tolerance buffer are explicitly rejected upstream.
    let intersection = format!(
        "SELECT ST_GeometryType(g), ST_AsText(g)
         FROM (SELECT ST_Intersection(
             ST_GeomFromText('{POLYGON}'),
             ST_GeomFromText('LINESTRING(-1 1, 3 1)')
         ) AS g)"
    );
    let (kind, text): (String, String) = connection
        .query_row(&intersection, [], |row| Ok((row.get(0)?, row.get(1)?)))
        .expect("observe planar intersection output");
    assert_eq!(kind, "LINESTRING");
    assert!(text.starts_with("LINESTRING"), "{text}");

    let disjoint = value(
        &connection,
        "SELECT ST_AsText(ST_Intersection(ST_GeomFromText('POINT(0 0)'), ST_GeomFromText('POINT(2 2)')))",
    );
    assert!(
        matches!(disjoint, DuckValue::Text(ref text) if text.contains("EMPTY")),
        "{disjoint:?}"
    );
    assert!(matches!(
        value(
            &connection,
            "SELECT ST_Intersection(NULL, ST_GeomFromText('POINT(0 0)'))"
        ),
        DuckValue::Null
    ));

    let (buffer_type, buffer_text): (String, String) = connection
        .query_row(
            "SELECT ST_GeometryType(g), ST_AsText(g)
             FROM (SELECT ST_Buffer(ST_GeomFromText('POINT(0 0)'), 1) AS g)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("observe planar buffer output");
    assert_eq!(buffer_type, "POLYGON");
    assert!(buffer_text.starts_with("POLYGON"), "{buffer_text}");
    assert!(buffer_text.contains("-1"), "{buffer_text}");
    assert!(matches!(
        value(&connection, "SELECT ST_Buffer(NULL, 1)"),
        DuckValue::Null
    ));

    let mismatch = connection.query_row(
        "SELECT ST_Intersection(
             ST_SetCRS(ST_GeomFromText('POINT(0 0)'), 'EPSG:4326'),
             ST_SetCRS(ST_GeomFromText('POINT(1 1)'), 'EPSG:3857'))",
        [],
        |row| row.get::<_, DuckValue>(0),
    );
    assert!(
        mismatch.is_err(),
        "pinned planar CRS mismatch must not be inferred as geography NULL"
    );
}

#[test]
fn duckdb_spheroid_primitives_are_point_pair_measurement_and_proximity_only() {
    let Some(connection) = connect() else {
        return;
    };

    // Documented [latitude, longitude] inputs (JFK and AMS from the DuckDB
    // Spatial function page). The pinned build requires the axis-order setting
    // to be explicit, so select the documented [latitude, longitude]
    // interpretation. These observations are descriptive: the pinned build
    // exposes point-pair spheroid measurement/proximity, not a containment
    // predicate, and this test asserts no polygon behavior.
    connection
        .execute_batch("SET geometry_always_xy = false;")
        .expect("set documented [latitude, longitude] axis order");
    let (lat1, lon1) = (40.6446, -73.7797);
    let (lat2, lon2) = (52.3130, 4.7725);
    let distance = format!(
        "SELECT ST_Distance_Spheroid({}, {})",
        point(lat1, lon1),
        point(lat2, lon2)
    );

    let metres = match value(&connection, &distance) {
        DuckValue::Double(metres) => metres,
        other => panic!("{distance}: expected DOUBLE, got {other:?}"),
    };
    eprintln!("observed ST_Distance_Spheroid metres={metres}");
    assert!(metres.is_finite(), "{metres}");
    assert!(
        metres > 5_000_000.0 && metres < 7_000_000.0,
        "JFK-AMS spheroid distance outside sanity bounds: {metres}"
    );

    for (offset, expected) in [(-1.0, false), (1.0, true)] {
        let threshold = format!(
            "SELECT ST_DWithin_Spheroid({}, {}, {})",
            point(lat1, lon1),
            point(lat2, lon2),
            metres + offset
        );
        assert_eq!(
            boolean(&connection, &threshold),
            expected,
            "threshold {metres} {offset:+} metres"
        );
    }
}

#[test]
fn transpiler_rejects_geography_methods_before_any_duckdb_execution() {
    let sqlite = rusqlite::Connection::open_in_memory().expect("open in-memory SQLite");
    sqlite
        .execute_batch(
            "CREATE TABLE gpkg_geometry_columns (table_name TEXT, column_name TEXT, srs_id INTEGER);
             INSERT INTO gpkg_geometry_columns VALUES ('features', 'shape', 4326);",
        )
        .expect("seed geometry metadata");
    let capability = sda_rs::spatial::initialize(&sqlite, None, None)
        .expect("initialize spatial capability without an extension");

    for method in [
        "STContains",
        "stcontains",
        "STWithin",
        "StWithin",
        "STIntersection",
        "stintersection",
        "STBuffer",
        "stbuffer",
    ] {
        for receiver in ["shape", "geometry::STGeomFromText('POINT(0 0)', 4326)"] {
            for arguments in [
                "()",
                "(geometry::STGeomFromText('POINT(0 0)', 4326))",
                "(geometry::STGeomFromText('POINT(0 0)', 4326), 1)",
            ] {
                let sql = format!("SELECT {receiver}.{method}{arguments} FROM features");
                let error = sda_rs::transpiler::transpile_with_spatial(&sql, Some(&capability))
                    .expect_err(&sql);
                let message = error.to_string();
                let lowered = message.to_ascii_lowercase();
                assert!(
                    lowered.contains(&format!(
                        ".{} is unsupported for sql server geography compatibility",
                        method.to_ascii_lowercase()
                    )),
                    "{sql}: {message}"
                );
                if method.to_ascii_lowercase().contains("intersection") {
                    assert!(
                        message.contains("no exact ellipsoidal overlay established"),
                        "{sql}: {message}"
                    );
                    assert!(!lowered.contains("st_intersection"), "{sql}: {message}");
                } else if method.to_ascii_lowercase().contains("buffer") {
                    assert!(
                        message.contains("no exact geography buffer established"),
                        "{sql}: {message}"
                    );
                    assert!(!lowered.contains("st_buffer"), "{sql}: {message}");
                } else {
                    assert!(
                        message.contains(
                            "no exact transformation established from available evidence"
                        ),
                        "{sql}: {message}"
                    );
                    assert!(
                        message.contains("SQL Server's boundary rule is not established"),
                        "{sql}: {message}"
                    );
                    assert!(!lowered.contains("st_contains"), "{sql}: {message}");
                    assert!(!lowered.contains("st_within"), "{sql}: {message}");
                }
            }
        }
    }
}
