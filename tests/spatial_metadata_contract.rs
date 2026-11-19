use rusqlite::Connection;

fn capability(rows: &str) -> sda_rs::spatial::SpatialCapability {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(&format!(
            "CREATE TABLE gpkg_geometry_columns(table_name TEXT, column_name TEXT, srs_id INTEGER);
             {rows}"
        ))
        .unwrap();
    sda_rs::spatial::initialize(&connection, None, None).unwrap()
}

#[test]
fn resolves_joined_relations_by_table_and_alias() {
    let capability = capability(
        "INSERT INTO gpkg_geometry_columns VALUES ('left_map', 'shape', 4326);
         INSERT INTO gpkg_geometry_columns VALUES ('right_map', 'shape', 4326);",
    );
    let sql = sda_rs::transpiler::transpile_with_spatial(
        "SELECT l.shape.STIntersects(r.shape) FROM left_map AS l JOIN right_map AS r ON 1 = 1",
        Some(&capability),
    )
    .unwrap()
    .remove(0)
    .to_ascii_uppercase();
    assert!(sql.contains("GEOMFROMGPB(L.SHAPE)"), "{sql}");
    assert!(sql.contains("GEOMFROMGPB(R.SHAPE)"), "{sql}");
}

#[test]
fn rejects_missing_duplicate_and_malformed_referenced_metadata_distinctly() {
    let missing = capability("");
    let error = sda_rs::transpiler::transpile_with_spatial(
        "SELECT m.shape.STIntersects(geometry.STGeomFromText('POINT(0 0)', 4326)) FROM missing_map m",
        Some(&missing),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("no geometry metadata"), "{error}");

    let duplicate = capability(
        "INSERT INTO gpkg_geometry_columns VALUES ('map', 'shape', 4326);
         INSERT INTO gpkg_geometry_columns VALUES ('MAP', 'other_shape', 4326);",
    );
    let error = sda_rs::transpiler::transpile_with_spatial(
        "SELECT m.shape.STIntersects(geometry.STGeomFromText('POINT(0 0)', 4326)) FROM map m",
        Some(&duplicate),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("ambiguous geometry metadata"), "{error}");

    let malformed = capability("INSERT INTO gpkg_geometry_columns VALUES ('map', '', 4326);");
    let error = sda_rs::transpiler::transpile_with_spatial(
        "SELECT m.shape.STIntersects(geometry.STGeomFromText('POINT(0 0)', 4326)) FROM map m",
        Some(&malformed),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("malformed geometry metadata"), "{error}");
}

#[test]
fn non_spatial_sql_does_not_require_spatial_metadata_or_extension() {
    let capability = capability("");
    assert_eq!(
        sda_rs::transpiler::transpile_with_spatial(
            "SELECT value FROM ordinary_table",
            Some(&capability)
        )
        .unwrap(),
        ["SELECT value FROM ordinary_table"]
    );
}

#[test]
fn mukey_still_requires_mupolygon_metadata_or_explicit_override() {
    let capability =
        capability("INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'soil_shape', 26910);");
    let query = "SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)')";
    let error = sda_rs::transpiler::transpile_with_spatial(query, Some(&capability)).unwrap_err();
    assert!(error
        .to_string()
        .contains("requires geometry metadata for mupolygon"));

    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(
            "CREATE TABLE gpkg_geometry_columns(table_name TEXT, column_name TEXT, srs_id INTEGER);
             INSERT INTO gpkg_geometry_columns VALUES ('mupolygon', 'shape', 3857);",
        )
        .unwrap();
    let capability =
        sda_rs::spatial::initialize(&connection, None, Some("override_shape")).unwrap();
    let sql = sda_rs::transpiler::transpile_with_spatial(query, Some(&capability)).unwrap();
    assert!(sql[0].contains("override_shape"));
    assert!(sql[0].contains("3857"));
}

#[tokio::test]
async fn configured_spatial_join_executes_with_per_relation_metadata() {
    let Ok(extension) = std::env::var("SDA_TEST_SPATIALITE") else {
        eprintln!("SDA_TEST_SPATIALITE not set; skipping configured spatial join");
        return;
    };
    let uri = format!(
        "file:spatial_metadata_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    );
    let connection = Connection::open(&uri).unwrap();
    {
        let _guard = unsafe { rusqlite::LoadExtensionGuard::new(&connection) }.unwrap();
        unsafe { connection.load_extension(&extension, None) }.unwrap();
    }
    connection
        .execute_batch(
            "CREATE TABLE gpkg_geometry_columns(table_name TEXT, column_name TEXT, srs_id INTEGER);
             INSERT INTO gpkg_geometry_columns VALUES ('left_map', 'shape', 4326);
             INSERT INTO gpkg_geometry_columns VALUES ('right_map', 'shape', 4326);
             CREATE TABLE left_map(shape BLOB);
             CREATE TABLE right_map(shape BLOB);
             INSERT INTO left_map VALUES (AsGPB(GeomFromText('POINT(0 0)', 4326)));
             INSERT INTO right_map VALUES (AsGPB(GeomFromText('POINT(0 0)', 4326)));",
        )
        .unwrap();
    let output = sda_rs::executor::execute_query_blocking(
        uri,
        "SELECT l.shape.STIntersects(r.shape) FROM left_map AS l JOIN right_map AS r ON 1 = 1"
            .into(),
        Some(extension),
        None,
        |capability, query| {
            sda_rs::transpiler::transpile_with_spatial(query, Some(capability))
                .map_err(|error| error.to_string())
        },
    )
    .await
    .unwrap();
    assert_eq!(
        output.result_sets[0].rows[0][0],
        rusqlite::types::Value::Integer(1)
    );
}
