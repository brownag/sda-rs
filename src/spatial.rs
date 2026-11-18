use std::{error::Error, fmt};

use rusqlite::Connection;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometryMetadata {
    pub table_name: String,
    pub geometry_column: String,
    pub srs_id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpatialCapability {
    available: bool,
    geometries: Vec<GeometryMetadata>,
    geometry_override: Option<String>,
    effective_geometry_column: Option<String>,
    metadata_error: Option<String>,
}

impl SpatialCapability {
    pub fn available(&self) -> bool {
        self.available
    }

    pub fn geometry_metadata_for_table(&self, table: &str) -> Option<&GeometryMetadata> {
        self.geometries
            .iter()
            .find(|metadata| metadata.table_name.eq_ignore_ascii_case(table))
    }

    pub fn is_geometry_column(&self, column: &str) -> bool {
        self.geometries
            .iter()
            .any(|metadata| metadata.geometry_column.eq_ignore_ascii_case(column))
            || self
                .effective_geometry_column
                .as_deref()
                .is_some_and(|override_column| override_column.eq_ignore_ascii_case(column))
    }

    pub fn effective_geometry_column(&self) -> Option<&str> {
        self.effective_geometry_column.as_deref()
    }

    pub fn geometry_override(&self) -> Option<&str> {
        self.geometry_override.as_deref()
    }

    pub fn require_spatial(&self) -> Result<(), SpatialError> {
        if let Some(message) = &self.metadata_error {
            return Err(SpatialError::Metadata(message.clone()));
        }
        if self.available {
            Ok(())
        } else {
            Err(SpatialError::Unavailable)
        }
    }
}

/// Conservatively recognizes spatial operations currently reserved for the
/// spatial rewrite path; ordinary SQL remains executable without SpatiaLite.
pub fn requires_spatial(sql: &str) -> bool {
    let sql = sql.to_ascii_lowercase();
    [
        "stgeomfromtext",
        "stintersects",
        "sda_get_mukey_from_intersection_with_wktwgs84",
        "sda_get_mukey_from_intersection_with_wktwm",
        "sda_get_mupolygonkey_from_intersection_with_wktwgs84",
        "sda_get_mupolygonkey_from_intersection_with_wktwm",
        "sda_get_sapolygonkey_from_intersection_with_wktwgs84",
        "sda_get_sapolygonkey_from_intersection_with_wktwm",
        "sda_get_areasymbol_from_intersection_with_wktwgs84",
        "sda_get_areasymbol_from_intersection_with_wktwm",
        "sda_get_mupointkey_from_intersection_with_wktwgs84",
        "sda_get_mupointkey_from_intersection_with_wktwm",
        "sda_get_mulinekey_from_intersection_with_wktwgs84",
        "sda_get_mulinekey_from_intersection_with_wktwm",
        "sda_get_featlinekey_from_intersection_with_wktwgs84",
        "sda_get_featlinekey_from_intersection_with_wktwm",
        "sda_get_featpointkey_from_intersection_with_wktwgs84",
        "sda_get_featpointkey_from_intersection_with_wktwm",
        "sda_get_areasymbolwktwgs84_from_areasymbol",
        "sda_get_areasymbolwktwm_from_areasymbol",
        "sda_get_mupolygonwktwgs84_from_mukey",
        "sda_get_mupolygonwktwm_from_mukey",
        "sda_get_mupointwktwgs84_from_mukey",
        "sda_get_mupointwktwm_from_mukey",
        "sda_get_mulinewktwgs84_from_mukey",
        "sda_get_mulinewktwm_from_mukey",
        "sda_get_sapolygonkey_from_intersection_with_sastatusmap_wktwgs84",
        "sda_get_sapolygonkey_from_intersection_with_sastatusmap_wktwm",
        "spatialite_",
    ]
    .iter()
    .any(|marker| sql.contains(marker))
}

#[derive(Debug)]
pub enum SpatialError {
    Unavailable,
    ExtensionLoad { path: String, message: String },
    Metadata(String),
}

impl fmt::Display for SpatialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => write!(
                f,
                "Spatial capability unavailable: SpatiaLite is not loaded"
            ),
            Self::ExtensionLoad { path, message } => {
                write!(f, "Unable to load spatial extension '{path}': {message}")
            }
            Self::Metadata(message) => {
                write!(
                    f,
                    "Unable to inspect GeoPackage geometry metadata: {message}"
                )
            }
        }
    }
}

impl Error for SpatialError {}

/// Initialize optional SpatiaLite and discover GeoPackage geometry metadata.
/// Extension loading is enabled only around the trusted Rust load call.
pub fn initialize(
    connection: &Connection,
    extension: Option<&str>,
    geometry_override: Option<&str>,
) -> Result<SpatialCapability, SpatialError> {
    let available = if let Some(path) = extension {
        let load_result = {
            // No SQL from the request is executed while the guard is alive.
            let _guard =
                unsafe { rusqlite::LoadExtensionGuard::new(connection) }.map_err(|error| {
                    SpatialError::ExtensionLoad {
                        path: path.to_owned(),
                        message: error.to_string(),
                    }
                })?;
            unsafe { connection.load_extension(path, None) }
        };
        load_result.map_err(|error| SpatialError::ExtensionLoad {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        connection
            .query_row("SELECT spatialite_version()", [], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| SpatialError::ExtensionLoad {
                path: path.to_owned(),
                message: format!("loaded extension does not provide SpatiaLite: {error}"),
            })?;
        true
    } else {
        false
    };

    let (geometries, metadata_error) = match connection
        .prepare("SELECT table_name, column_name, srs_id FROM gpkg_geometry_columns")
    {
        Ok(mut statement) => match statement
            .query_map([], |row| {
                Ok(GeometryMetadata {
                    table_name: row.get(0)?,
                    geometry_column: row.get(1)?,
                    srs_id: row.get(2)?,
                })
            })
            .map_err(|error| error.to_string())
            .and_then(|rows| {
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|error| error.to_string())
            }) {
            Ok(geometries) => (geometries, None),
            Err(error) => (Vec::new(), Some(error)),
        },
        Err(rusqlite::Error::SqliteFailure(_, _)) | Err(rusqlite::Error::InvalidQuery) => {
            let exists: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='gpkg_geometry_columns')",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| SpatialError::Metadata(error.to_string()))?;
            if exists {
                (Vec::new(), Some(rusqlite::Error::InvalidQuery.to_string()))
            } else {
                (Vec::new(), None)
            }
        }
        Err(error) => (Vec::new(), Some(error.to_string())),
    };
    let effective_geometry_column = geometry_override
        .map(str::to_owned)
        .or_else(|| geometries.first().map(|m| m.geometry_column.clone()));

    Ok(SpatialCapability {
        available,
        geometries,
        geometry_override: geometry_override.map(str::to_owned),
        effective_geometry_column,
        metadata_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{execute_query_blocking, execute_spatial, ExecutorError};

    #[test]
    fn ordinary_sqlite_has_no_spatial_metadata_and_needs_no_extension() {
        let connection = Connection::open_in_memory().unwrap();
        let capability = initialize(&connection, None, None).unwrap();
        assert!(!capability.available());
        assert_eq!(capability.geometry_metadata_for_table("soil_map"), None);
        assert_eq!(capability.geometry_override(), None);
        assert_eq!(capability.effective_geometry_column(), None);
        assert_eq!(
            capability.require_spatial().unwrap_err().to_string(),
            "Spatial capability unavailable: SpatiaLite is not loaded"
        );
    }

    #[test]
    fn malformed_metadata_is_deferred_until_spatial_capability_is_required() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("CREATE TABLE gpkg_geometry_columns (table_name TEXT);")
            .unwrap();
        let capability = initialize(&connection, None, None).unwrap();
        assert!(!capability.available());
        assert!(matches!(
            capability.require_spatial(),
            Err(SpatialError::Metadata(_))
        ));
        assert!(capability
            .require_spatial()
            .unwrap_err()
            .to_string()
            .starts_with("Unable to inspect GeoPackage geometry metadata:"));
    }

    #[test]
    fn discovers_dynamic_geometry_metadata_and_applies_override() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(
            "CREATE TABLE gpkg_geometry_columns (table_name TEXT, column_name TEXT, geometry_type_name TEXT, srs_id INTEGER, z TINYINT, m TINYINT);
             INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'geom_custom', 'MULTIPOLYGON', 26910, 0, 0);",
        ).unwrap();
        let capability = initialize(&connection, None, Some("geom_override")).unwrap();
        assert_eq!(
            capability.geometry_metadata_for_table("soil_map"),
            Some(&GeometryMetadata {
                table_name: "soil_map".into(),
                geometry_column: "geom_custom".into(),
                srs_id: 26910,
            })
        );
        assert_eq!(
            capability.effective_geometry_column(),
            Some("geom_override")
        );
        assert_eq!(capability.geometry_override(), Some("geom_override"));
    }

    #[test]
    fn geometry_override_provenance_is_distinct_from_discovered_column() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(
            "CREATE TABLE gpkg_geometry_columns (table_name TEXT, column_name TEXT, geometry_type_name TEXT, srs_id INTEGER, z TINYINT, m TINYINT);
             INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'geom_custom', 'MULTIPOLYGON', 26910, 0, 0);",
        ).unwrap();

        let discovered = initialize(&connection, None, None).unwrap();
        assert_eq!(discovered.effective_geometry_column(), Some("geom_custom"));
        assert_eq!(discovered.geometry_override(), None);

        let overridden = initialize(&connection, None, Some("geom_override")).unwrap();
        assert_eq!(
            overridden.effective_geometry_column(),
            Some("geom_override")
        );
        assert_eq!(overridden.geometry_override(), Some("geom_override"));
    }

    #[test]
    fn configured_missing_extension_is_a_structured_error() {
        let connection = Connection::open_in_memory().unwrap();
        let error = initialize(
            &connection,
            Some("/definitely/missing/mod_spatialite.so"),
            None,
        )
        .unwrap_err();
        assert!(matches!(error, SpatialError::ExtensionLoad { .. }));
    }

    #[test]
    fn spatial_statement_fails_with_structured_capability_error() {
        let error = execute_spatial(
            ":memory:",
            &["SELECT STGeomFromText('POINT(0 0)', 4326)".into()],
            None,
            None,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ExecutorError::Spatial(SpatialError::Unavailable)
        ));
    }

    #[tokio::test]
    async fn preparation_receives_metadata_before_transpiling_query() {
        let path =
            std::env::temp_dir().join(format!("sda-spatial-prepare-{}.sqlite", std::process::id()));
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch(
            "CREATE TABLE gpkg_geometry_columns (table_name TEXT, column_name TEXT, geometry_type_name TEXT, srs_id INTEGER, z TINYINT, m TINYINT);
             INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'shape', 'MULTIPOLYGON', 4326, 0, 0);",
        ).unwrap();
        drop(connection);
        let output = execute_query_blocking(
            path.to_string_lossy().into_owned(),
            "SELECT 1".into(),
            None,
            Some("shape_override".into()),
            |capability, query| {
                assert_eq!(query, "SELECT 1");
                let metadata = capability.geometry_metadata_for_table("soil_map").unwrap();
                assert_eq!(metadata.table_name, "soil_map");
                assert_eq!(metadata.srs_id, 4326);
                assert_eq!(
                    capability.effective_geometry_column(),
                    Some("shape_override")
                );
                assert_eq!(capability.geometry_override(), Some("shape_override"));
                Ok(vec!["SELECT 1".into()])
            },
        )
        .await
        .unwrap();
        assert_eq!(
            output.result_sets[0].rows,
            [vec![rusqlite::types::Value::Integer(1)]]
        );
        std::fs::remove_file(path).unwrap();
    }
}
