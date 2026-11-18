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
    effective_geometry_column: Option<String>,
}

impl SpatialCapability {
    pub fn available(&self) -> bool {
        self.available
    }

    pub fn geometry_metadata(&self) -> Option<&GeometryMetadata> {
        self.geometry.as_ref()
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

    let geometry = match connection.query_row(
        "SELECT table_name, column_name, srs_id FROM gpkg_geometry_columns LIMIT 1",
        [],
        |row| {
            Ok(GeometryMetadata {
                table_name: row.get(0)?,
                geometry_column: row.get(1)?,
                srs_id: row.get(2)?,
            })
        },
    ) {
        Ok(metadata) => Some(metadata),
        Err(rusqlite::Error::QueryReturnedNoRows) => None,
        Err(error) => {
            // The table is optional in ordinary SQLite databases.
            let exists: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='gpkg_geometry_columns')",
                    [],
                    |row| row.get(0),
                )
                .map_err(SpatialError::Metadata)?;
            if exists {
                return Err(SpatialError::Metadata(error));
            }
            None
        }
    };
    let effective_geometry_column = geometry_override.map(str::to_owned).or_else(|| {
        geometry
            .as_ref()
            .map(|metadata| metadata.geometry_column.clone())
    });

    Ok(SpatialCapability {
        available,
        geometry,
        effective_geometry_column,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
