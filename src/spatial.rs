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
    geometry: Option<GeometryMetadata>,
    geometries: Vec<GeometryMetadata>,
    effective_geometry_column: Option<String>,
}

impl SpatialCapability {
    pub fn available(&self) -> bool {
        self.available
    }

    pub fn geometry_metadata(&self) -> Option<&GeometryMetadata> {
        self.geometry.as_ref()
    }

    pub fn geometry_metadata_for_table(&self, table: &str) -> Option<&GeometryMetadata> {
        self.geometries
            .iter()
            .find(|metadata| metadata.table_name.eq_ignore_ascii_case(table))
    }

    pub fn effective_geometry_column(&self) -> Option<&str> {
        self.effective_geometry_column.as_deref()
    }

    pub fn require_spatial(&self) -> Result<(), SpatialError> {
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
        "spatialite_",
    ]
    .iter()
    .any(|marker| sql.contains(marker))
}

#[derive(Debug)]
pub enum SpatialError {
    Unavailable,
    ExtensionLoad { path: String, message: String },
    Metadata(rusqlite::Error),
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
            Self::Metadata(error) => {
                write!(f, "Unable to inspect GeoPackage geometry metadata: {error}")
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

    let geometries = match connection
        .prepare("SELECT table_name, column_name, srs_id FROM gpkg_geometry_columns")
    {
        Ok(mut statement) => statement
            .query_map([], |row| {
                Ok(GeometryMetadata {
                    table_name: row.get(0)?,
                    geometry_column: row.get(1)?,
                    srs_id: row.get(2)?,
                })
            })
            .map_err(SpatialError::Metadata)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(SpatialError::Metadata)?,
        Err(rusqlite::Error::SqliteFailure(_, _)) | Err(rusqlite::Error::InvalidQuery) => {
            let exists: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='gpkg_geometry_columns')",
                    [],
                    |row| row.get(0),
                )
                .map_err(SpatialError::Metadata)?;
            if exists {
                return Err(SpatialError::Metadata(rusqlite::Error::InvalidQuery));
            }
            Vec::new()
        }
        Err(error) => {
            return Err(SpatialError::Metadata(error));
        }
    };
    let geometry = geometries.first().cloned();
    let effective_geometry_column = geometry_override.map(str::to_owned).or_else(|| {
        geometry
            .as_ref()
            .map(|metadata| metadata.geometry_column.clone())
    });

    Ok(SpatialCapability {
        available,
        geometry,
        geometries,
        effective_geometry_column,
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
        assert_eq!(capability.geometry_metadata(), None);
        assert_eq!(capability.effective_geometry_column(), None);
        assert_eq!(
            capability.require_spatial().unwrap_err().to_string(),
            "Spatial capability unavailable: SpatiaLite is not loaded"
        );
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
            capability.geometry_metadata(),
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
                let metadata = capability.geometry_metadata().unwrap();
                assert_eq!(metadata.table_name, "soil_map");
                assert_eq!(metadata.srs_id, 4326);
                assert_eq!(
                    capability.effective_geometry_column(),
                    Some("shape_override")
                );
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
