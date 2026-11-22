use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use duckdb::{types::Value as DuckValue, Config, Connection};
use rusqlite::{types::Value, OpenFlags};

use crate::{gpkg_geometry, result::ResultSet};

#[derive(Debug)]
pub enum ExecutorError {
    DuckDb(duckdb::Error),
    Join(tokio::task::JoinError),
    UnsupportedValue(String),
    GeoPackage(String),
    DeadlineExceeded,
    Rollback {
        execution: Box<Self>,
        rollback: duckdb::Error,
    },
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuckDb(error) => write!(f, "DuckDB execution failed: {error}"),
            Self::Join(error) => write!(f, "DuckDB worker failed: {error}"),
            Self::UnsupportedValue(value) => {
                write!(f, "unsupported DuckDB result type: {value}")
            }
            Self::GeoPackage(message) => write!(f, "GeoPackage input rejected: {message}"),
            Self::DeadlineExceeded => f.write_str("DuckDB execution deadline exceeded"),
            Self::Rollback {
                execution,
                rollback,
            } => write!(
                f,
                "{execution}; transaction rollback also failed: {rollback}"
            ),
        }
    }
}

impl Error for ExecutorError {}

impl From<duckdb::Error> for ExecutorError {
    fn from(value: duckdb::Error) -> Self {
        Self::DuckDb(value)
    }
}

impl From<tokio::task::JoinError> for ExecutorError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Join(value)
    }
}

/// Capability for an internally approved, canonical GeoPackage path.
///
/// This type is deliberately private to the executor: no SQL or request value
/// can construct or select a GeoPackage input.
#[allow(dead_code)]
struct TrustedGeoPackagePath(PathBuf);

impl TrustedGeoPackagePath {
    #[allow(dead_code)]
    fn from_internal_setup(path: &Path) -> Result<Self, ExecutorError> {
        let canonical = path
            .canonicalize()
            .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
        if !canonical.is_file() {
            return Err(ExecutorError::GeoPackage(
                "approved path is not a regular file".to_owned(),
            ));
        }
        Ok(Self(canonical))
    }
}

/// Load one trusted GeoPackage using SQLite's read-only VFS open, validate its
/// metadata and geometry payloads in Rust, and return only copied values.
/// No input path is interpolated into DuckDB SQL or exposed to request SQL.
#[allow(dead_code)]
fn load_geopackage(approved: &TrustedGeoPackagePath) -> Result<ResultSet, ExecutorError> {
    let connection = rusqlite::Connection::open_with_flags(
        &approved.0,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
    let validated = load_geopackage_connection(&connection)?;
    copy_validated_values_to_duckdb(validated)
}

#[allow(dead_code)]
fn copy_validated_values_to_duckdb(validated: ResultSet) -> Result<ResultSet, ExecutorError> {
    use duckdb::params_from_iter;

    let config = Config::default()
        .enable_autoload_extension(false)?
        .enable_external_access(false)?;
    let connection = Connection::open_in_memory_with_flags(config)?;
    let declarations = (0..validated.columns.len())
        .map(|index| {
            let types = validated
                .rows
                .iter()
                .filter_map(|row| row.get(index))
                .find(|value| !matches!(value, Value::Null))
                .map(duckdb_type_for_value)
                .unwrap_or("VARCHAR");
            if validated
                .rows
                .iter()
                .filter_map(|row| row.get(index))
                .any(|value| !matches!(value, Value::Null) && duckdb_type_for_value(value) != types)
            {
                return Err(ExecutorError::GeoPackage(
                    "column contains mixed SQLite value types".to_owned(),
                ));
            }
            Ok(format!("c{index} {types}"))
        })
        .collect::<Result<Vec<_>, ExecutorError>>()?;
    connection.execute_batch(&format!(
        "CREATE TEMP TABLE validated_values ({})",
        declarations.join(", ")
    ))?;
    let placeholders = (1..=validated.columns.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut insert = connection.prepare(&format!(
        "INSERT INTO validated_values VALUES ({placeholders})"
    ))?;
    for row in &validated.rows {
        if row.len() != validated.columns.len() {
            return Err(ExecutorError::GeoPackage(
                "row width differs from validated schema".to_owned(),
            ));
        }
        let values = row.iter().map(to_duckdb_value).collect::<Vec<_>>();
        insert.execute(params_from_iter(values.iter()))?;
    }
    drop(insert);
    let select = (0..validated.columns.len())
        .map(|index| format!("c{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut statement = connection.prepare(&format!(
        "SELECT {select} FROM validated_values ORDER BY rowid"
    ))?;
    let mut rows = statement.query([])?;
    let mut copied_rows = Vec::new();
    while let Some(row) = rows.next()? {
        let mut values = Vec::with_capacity(validated.columns.len());
        for index in 0..validated.columns.len() {
            let value = row.get::<_, DuckValue>(index)?;
            values.push(match value {
                DuckValue::Blob(blob) => Value::Blob(blob),
                other => map_value(other)?,
            });
        }
        copied_rows.push(values);
    }
    Ok(ResultSet {
        columns: validated.columns,
        rows: copied_rows,
    })
}

fn duckdb_type_for_value(value: &Value) -> &'static str {
    match value {
        Value::Null => "VARCHAR",
        Value::Integer(_) => "BIGINT",
        Value::Real(_) => "DOUBLE",
        Value::Text(_) => "VARCHAR",
        Value::Blob(_) => "BLOB",
    }
}

fn to_duckdb_value(value: &Value) -> DuckValue {
    match value {
        Value::Null => DuckValue::Null,
        Value::Integer(value) => DuckValue::BigInt(*value),
        Value::Real(value) => DuckValue::Double(*value),
        Value::Text(value) => DuckValue::Text(value.clone()),
        Value::Blob(value) => DuckValue::Blob(value.clone()),
    }
}

#[allow(dead_code)]
fn load_geopackage_connection(
    connection: &rusqlite::Connection,
) -> Result<ResultSet, ExecutorError> {
    let metadata = connection
        .prepare(
            "SELECT table_name, column_name, srs_id, z, m
             FROM gpkg_geometry_columns
             ORDER BY table_name, column_name",
        )
        .and_then(|mut statement| {
            statement
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i32>(2)?,
                        row.get::<_, i32>(3)?,
                        row.get::<_, i32>(4)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
    if metadata.len() != 1 {
        return Err(ExecutorError::GeoPackage(
            "exactly one geometry column is required".to_owned(),
        ));
    }
    let (table, geometry_column, declared_srs, z, m) = &metadata[0];
    if ![0, 1, 2].contains(z) || ![0, 1, 2].contains(m) {
        return Err(ExecutorError::GeoPackage(
            "invalid geometry dimensionality metadata".to_owned(),
        ));
    }
    validate_sqlite_identifier(table)?;
    validate_sqlite_identifier(geometry_column)?;

    let srs_count: i64 = connection
        .query_row(
            "SELECT count(*) FROM gpkg_spatial_ref_sys WHERE srs_id = ?1",
            [declared_srs],
            |row| row.get(0),
        )
        .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
    if srs_count != 1 {
        return Err(ExecutorError::GeoPackage(
            "geometry SRID has no unique CRS metadata row".to_owned(),
        ));
    }
    let table_count: i64 = connection
        .query_row(
            "SELECT count(*) FROM gpkg_contents
             WHERE table_name = ?1 AND data_type = 'features' AND srs_id = ?2",
            rusqlite::params![table, declared_srs],
            |row| row.get(0),
        )
        .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
    if table_count != 1 {
        return Err(ExecutorError::GeoPackage(
            "geometry table is not registered as a GeoPackage feature table".to_owned(),
        ));
    }

    let mut statement = connection
        .prepare(&format!("SELECT * FROM \"{table}\" ORDER BY rowid"))
        .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
    let columns = statement
        .column_names()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let geometry_index = columns
        .iter()
        .position(|column| column == geometry_column)
        .ok_or_else(|| ExecutorError::GeoPackage("geometry column is absent".to_owned()))?;
    let mut rows = Vec::new();
    let mut query = statement
        .query([])
        .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
    while let Some(row) = query
        .next()
        .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?
    {
        let mut values = Vec::with_capacity(columns.len());
        for index in 0..columns.len() {
            let value = row
                .get::<_, Value>(index)
                .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
            if index == geometry_index {
                values.push(validate_geometry_value(value, *declared_srs, *z, *m)?);
            } else {
                values.push(value);
            }
        }
        rows.push(values);
    }
    Ok(ResultSet { columns, rows })
}

#[allow(dead_code)]
fn validate_sqlite_identifier(value: &str) -> Result<(), ExecutorError> {
    if value.is_empty() || value.contains('\0') || value.contains('"') || value.contains('.') {
        return Err(ExecutorError::GeoPackage(
            "invalid metadata identifier".to_owned(),
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_geometry_value(
    value: Value,
    declared_srs: i32,
    declared_z: i32,
    declared_m: i32,
) -> Result<Value, ExecutorError> {
    let Value::Blob(blob) = value else {
        return if value == Value::Null {
            Ok(Value::Null)
        } else {
            Err(ExecutorError::GeoPackage(
                "geometry value must be GeoPackageBinary or NULL".to_owned(),
            ))
        };
    };
    let decoded = gpkg_geometry::decode(&blob)
        .map_err(|error| ExecutorError::GeoPackage(error.to_string()))?;
    if decoded.srs_id != declared_srs {
        return Err(ExecutorError::GeoPackage(
            "GeoPackageBinary SRID disagrees with metadata".to_owned(),
        ));
    }
    let (has_coordinates, dimensions) = validate_wkb(decoded.wkb)?;
    if decoded.empty == has_coordinates {
        return Err(ExecutorError::GeoPackage(
            "GeoPackage empty flag disagrees with WKB payload".to_owned(),
        ));
    }
    let has_z = matches!(dimensions, 1 | 3);
    let has_m = matches!(dimensions, 2 | 3);
    if (declared_z == 0 && has_z)
        || (declared_z == 1 && !has_z)
        || (declared_m == 0 && has_m)
        || (declared_m == 1 && !has_m)
    {
        return Err(ExecutorError::GeoPackage(
            "WKB dimensions disagree with geometry metadata".to_owned(),
        ));
    }
    Ok(Value::Blob(blob))
}

#[allow(dead_code)]
fn validate_wkb(wkb: &[u8]) -> Result<(bool, u32), ExecutorError> {
    let mut offset = 0;
    let (geometry, dimensions) = parse_wkb_geometry(wkb, &mut offset, 0)?;
    if offset != wkb.len() {
        return Err(ExecutorError::GeoPackage(
            "trailing bytes after WKB geometry".to_owned(),
        ));
    }
    Ok((geometry, dimensions))
}

#[allow(dead_code)]
fn parse_wkb_geometry(
    bytes: &[u8],
    offset: &mut usize,
    depth: usize,
) -> Result<(bool, u32), ExecutorError> {
    if depth > 32 || bytes.len().saturating_sub(*offset) < 5 {
        return Err(ExecutorError::GeoPackage(
            "truncated or excessively nested WKB".to_owned(),
        ));
    }
    let little = match bytes[*offset] {
        0 => false,
        1 => true,
        _ => {
            return Err(ExecutorError::GeoPackage(
                "invalid WKB byte order".to_owned(),
            ))
        }
    };
    *offset += 1;
    let raw_type = read_wkb_u32(bytes, offset, little)?;
    let (base_type, dimensions) = decode_wkb_type(raw_type)?;
    let ordinate_count = 2 + if dimensions == 0 {
        0
    } else if dimensions == 3 {
        2
    } else {
        1
    };
    let coordinate_bytes = ordinate_count * 8;
    match base_type {
        1 => {
            require_wkb_bytes(bytes, *offset, coordinate_bytes)?;
            let ordinates = (0..ordinate_count)
                .map(|index| read_wkb_f64(bytes, *offset + index * 8, little))
                .collect::<Result<Vec<_>, _>>()?;
            let empty = ordinates.iter().all(|ordinate| ordinate.is_nan());
            if !empty && ordinates.iter().any(|ordinate| !ordinate.is_finite()) {
                return Err(ExecutorError::GeoPackage(
                    "non-finite WKB point ordinate".to_owned(),
                ));
            }
            *offset += coordinate_bytes;
            Ok((!empty, dimensions))
        }
        2 => {
            let count = read_wkb_count(bytes, offset, little)?;
            require_wkb_count_bytes(bytes, *offset, count, coordinate_bytes)?;
            validate_wkb_coordinates(bytes, *offset, count, ordinate_count, little)?;
            *offset += count * coordinate_bytes;
            Ok((count > 0, dimensions))
        }
        3 => {
            let rings = read_wkb_count(bytes, offset, little)?;
            let mut has_coordinates = false;
            for _ in 0..rings {
                let count = read_wkb_count(bytes, offset, little)?;
                require_wkb_count_bytes(bytes, *offset, count, coordinate_bytes)?;
                validate_wkb_coordinates(bytes, *offset, count, ordinate_count, little)?;
                *offset += count * coordinate_bytes;
                has_coordinates |= count > 0;
            }
            Ok((has_coordinates, dimensions))
        }
        4..=7 => {
            let count = read_wkb_count(bytes, offset, little)?;
            let mut has_coordinates = false;
            for _ in 0..count {
                let (child_has_coordinates, child_dimensions) =
                    parse_wkb_geometry(bytes, offset, depth + 1)?;
                if child_dimensions != dimensions {
                    return Err(ExecutorError::GeoPackage(
                        "mixed dimensions in WKB collection".to_owned(),
                    ));
                }
                has_coordinates |= child_has_coordinates;
            }
            Ok((has_coordinates, dimensions))
        }
        _ => Err(ExecutorError::GeoPackage(format!(
            "unsupported WKB geometry type {base_type}"
        ))),
    }
}

#[allow(dead_code)]
fn decode_wkb_type(raw: u32) -> Result<(u32, u32), ExecutorError> {
    if (1..=7).contains(&raw) {
        return Ok((raw, 0));
    }
    let dimensions = match raw / 1000 {
        1 => 1,
        2 => 2,
        3 => 3,
        _ => {
            return Err(ExecutorError::GeoPackage(
                "unsupported WKB dimensions".to_owned(),
            ))
        }
    };
    let base = raw % 1000;
    if (1..=7).contains(&base) {
        Ok((base, dimensions))
    } else {
        Err(ExecutorError::GeoPackage("unsupported WKB type".to_owned()))
    }
}

#[allow(dead_code)]
fn read_wkb_u32(bytes: &[u8], offset: &mut usize, little: bool) -> Result<u32, ExecutorError> {
    require_wkb_bytes(bytes, *offset, 4)?;
    let value: [u8; 4] = bytes[*offset..*offset + 4].try_into().unwrap();
    *offset += 4;
    Ok(if little {
        u32::from_le_bytes(value)
    } else {
        u32::from_be_bytes(value)
    })
}

#[allow(dead_code)]
fn read_wkb_f64(bytes: &[u8], offset: usize, little: bool) -> Result<f64, ExecutorError> {
    require_wkb_bytes(bytes, offset, 8)?;
    let value: [u8; 8] = bytes[offset..offset + 8].try_into().unwrap();
    Ok(if little {
        f64::from_le_bytes(value)
    } else {
        f64::from_be_bytes(value)
    })
}

#[allow(dead_code)]
fn validate_wkb_coordinates(
    bytes: &[u8],
    offset: usize,
    count: usize,
    ordinate_count: usize,
    little: bool,
) -> Result<(), ExecutorError> {
    let total = count
        .checked_mul(ordinate_count)
        .ok_or_else(|| ExecutorError::GeoPackage("WKB ordinate count overflow".to_owned()))?;
    for index in 0..total {
        if !read_wkb_f64(bytes, offset + index * 8, little)?.is_finite() {
            return Err(ExecutorError::GeoPackage(
                "non-finite WKB coordinate ordinate".to_owned(),
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn read_wkb_count(bytes: &[u8], offset: &mut usize, little: bool) -> Result<usize, ExecutorError> {
    let count = read_wkb_u32(bytes, offset, little)? as usize;
    if count > 1_000_000 {
        return Err(ExecutorError::GeoPackage(
            "WKB element count exceeds validation bound".to_owned(),
        ));
    }
    Ok(count)
}

#[allow(dead_code)]
fn require_wkb_count_bytes(
    bytes: &[u8],
    offset: usize,
    count: usize,
    stride: usize,
) -> Result<(), ExecutorError> {
    let length = count
        .checked_mul(stride)
        .ok_or_else(|| ExecutorError::GeoPackage("WKB length overflow".to_owned()))?;
    require_wkb_bytes(bytes, offset, length)
}

#[allow(dead_code)]
fn require_wkb_bytes(bytes: &[u8], offset: usize, length: usize) -> Result<(), ExecutorError> {
    if offset
        .checked_add(length)
        .is_none_or(|end| end > bytes.len())
    {
        return Err(ExecutorError::GeoPackage("truncated WKB".to_owned()));
    }
    Ok(())
}

/// Execute a batch on a blocking worker with a fresh in-memory connection.
pub async fn execute(
    statements: Vec<String>,
    deadline: Duration,
) -> Result<Vec<ResultSet>, ExecutorError> {
    Ok(tokio::task::spawn_blocking(move || execute_blocking(&statements, deadline)).await??)
}

fn execute_blocking(
    statements: &[String],
    deadline: Duration,
) -> Result<Vec<ResultSet>, ExecutorError> {
    let config = Config::default()
        .enable_autoload_extension(false)?
        .enable_external_access(false)?;
    let connection = Connection::open_in_memory_with_flags(config)?;
    let interrupt = connection.interrupt_handle();
    let done = Arc::new(AtomicBool::new(false));
    let watchdog_done = Arc::clone(&done);
    let expired = Arc::new(AtomicBool::new(false));
    let watchdog_expired = Arc::clone(&expired);
    let started = Instant::now();
    let watchdog = thread::spawn(move || {
        while !watchdog_done.load(Ordering::Acquire) {
            if started.elapsed() >= deadline {
                watchdog_expired.store(true, Ordering::Release);
                // DuckDB resets an interrupt that races with preparation. Keep
                // signalling until execution observes it or the batch ends.
                interrupt.interrupt();
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let execution = execute_batch(&connection, statements, started, deadline);
    done.store(true, Ordering::Release);
    let _ = watchdog.join();
    if expired.load(Ordering::Acquire) {
        Err(ExecutorError::DeadlineExceeded)
    } else {
        execution
    }
}

fn execute_batch(
    connection: &Connection,
    statements: &[String],
    started: Instant,
    deadline: Duration,
) -> Result<Vec<ResultSet>, ExecutorError> {
    connection.execute_batch("BEGIN TRANSACTION")?;
    let execution = (|| {
        let mut results = Vec::new();
        let mut temp_tables = std::collections::HashSet::new();
        for sql in statements {
            if started.elapsed() >= deadline {
                return Err(ExecutorError::DeadlineExceeded);
            }
            validate_statement(sql, &mut temp_tables)?;
            let mut statement = connection.prepare(sql)?;
            let column_count = statement.column_count();
            if column_count == 0 {
                statement.execute([])?;
                continue;
            }
            let mut rows = statement.query([])?;
            let columns = rows
                .as_ref()
                .expect("query has statement metadata")
                .column_names();
            let mut values = Vec::new();
            while let Some(row) = rows.next()? {
                if started.elapsed() >= deadline {
                    return Err(ExecutorError::DeadlineExceeded);
                }
                let mut values_row = Vec::with_capacity(column_count);
                for index in 0..column_count {
                    values_row.push(map_value(row.get::<_, DuckValue>(index)?)?);
                }
                values.push(values_row);
            }
            results.push(ResultSet {
                columns,
                rows: values,
            });
        }
        if started.elapsed() >= deadline {
            return Err(ExecutorError::DeadlineExceeded);
        }
        Ok(results)
    })();

    match execution {
        Ok(results) => {
            connection.execute_batch("COMMIT")?;
            Ok(results)
        }
        Err(error) => match connection.execute_batch("ROLLBACK") {
            Ok(()) => Err(error),
            Err(rollback) => Err(ExecutorError::Rollback {
                execution: Box::new(error),
                rollback,
            }),
        },
    }
}

// DuckDB does not expose a SQLite-style authorizer in this pinned Rust API.
// Until a parser-backed policy seam exists, accept only read-only SELECT and a
// deliberately small temp-table grammar. Reject comments, quoted identifiers,
// and compound statements rather than trying to classify ambiguous SQL.
fn validate_statement(
    sql: &str,
    temp_tables: &mut std::collections::HashSet<String>,
) -> Result<(), ExecutorError> {
    let normalized = sql.trim().trim_end_matches(';').trim();
    let upper = normalized.to_ascii_uppercase();
    let simple = !normalized.contains(';')
        && !normalized.contains("--")
        && !normalized.contains("/*")
        && !normalized.contains('\'')
        && !normalized.contains('"')
        && !normalized.contains('`');
    if !simple {
        return Err(ExecutorError::DuckDb(duckdb::Error::InvalidQuery));
    }
    if upper.starts_with("SELECT ") || upper == "SELECT" {
        return Ok(());
    }
    let create_prefix = if upper.starts_with("CREATE TEMP TABLE ") {
        "CREATE TEMP TABLE "
    } else if upper.starts_with("CREATE TEMPORARY TABLE ") {
        "CREATE TEMPORARY TABLE "
    } else {
        ""
    };
    if !create_prefix.is_empty() {
        let tail = normalized[create_prefix.len()..].trim();
        let name = tail.split_whitespace().next().unwrap_or_default();
        let definition = tail[name.len()..].trim();
        let definition_upper = definition.to_ascii_uppercase();
        if identifier(name)
            && (definition_upper.starts_with("AS SELECT ")
                || definition_upper.starts_with("AS SELECT(")
                || definition.starts_with('('))
        {
            temp_tables.insert(name.to_ascii_lowercase());
            return Ok(());
        }
    }
    for (prefix, verb) in [("INSERT INTO ", "INSERT"), ("DROP TABLE ", "DROP")] {
        if upper.starts_with(prefix) {
            let tail = normalized[prefix.len()..].trim();
            let name = tail.split_whitespace().next().unwrap_or_default();
            let key = name.to_ascii_lowercase();
            if identifier(name) && temp_tables.contains(&key) {
                if verb == "DROP" {
                    temp_tables.remove(&key);
                }
                return Ok(());
            }
        }
    }
    Err(ExecutorError::DuckDb(duckdb::Error::InvalidQuery))
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn map_value(value: DuckValue) -> Result<Value, ExecutorError> {
    Ok(match value {
        DuckValue::Null => Value::Null,
        DuckValue::Boolean(value) => Value::Integer(i64::from(value)),
        DuckValue::TinyInt(value) => Value::Integer(i64::from(value)),
        DuckValue::SmallInt(value) => Value::Integer(i64::from(value)),
        DuckValue::Int(value) => Value::Integer(i64::from(value)),
        DuckValue::BigInt(value) => Value::Integer(value),
        DuckValue::UTinyInt(value) => Value::Integer(i64::from(value)),
        DuckValue::USmallInt(value) => Value::Integer(i64::from(value)),
        DuckValue::UInt(value) => Value::Integer(i64::from(value)),
        // rusqlite::Value has no unsigned integer variant; decimal text avoids
        // overflow and preserves the integer value exactly in SDA output.
        DuckValue::UBigInt(value) => Value::Text(value.to_string()),
        DuckValue::Float(value) => Value::Real(f64::from(value)),
        DuckValue::Double(value) => Value::Real(value),
        // Decimal has no equivalent in rusqlite::Value. Preserve its exact
        // decimal rendering as text rather than narrowing it to binary float.
        DuckValue::Decimal(value) => Value::Text(value.to_string()),
        DuckValue::Text(value) => Value::Text(value),
        // A BLOB may contain GeoPackageBinary. Without validating its header,
        // SRID, metadata row, and WKB dimensions, passing it to the shared
        // serializer would silently expose an unverified geometry payload.
        DuckValue::Blob(_) => {
            return Err(ExecutorError::UnsupportedValue(
                "BLOB/GeoPackageBinary payloads are not validated".to_owned(),
            ))
        }
        DuckValue::Geometry(_) => {
            return Err(ExecutorError::UnsupportedValue(
                "GEOMETRY payloads are not validated".to_owned(),
            ))
        }
        other => return Err(ExecutorError::UnsupportedValue(format!("{other:?}"))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializer::{serialize, Format};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Fixture(PathBuf);

    impl Fixture {
        fn new(geometry: Option<Vec<u8>>, srid: i32) -> Self {
            Self::new_dimensional(geometry, srid, 0, 0)
        }

        fn new_dimensional(geometry: Option<Vec<u8>>, srid: i32, z: i32, m: i32) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "sda-duckdb-gpkg-{}-{nonce}.gpkg",
                std::process::id()
            ));
            let connection = rusqlite::Connection::open(&path).unwrap();
            connection
                .execute_batch(
                    "CREATE TABLE gpkg_spatial_ref_sys (
                        srs_name TEXT, srs_id INTEGER PRIMARY KEY, organization TEXT,
                        organization_coordsys_id INTEGER, definition TEXT, description TEXT
                     );
                     CREATE TABLE gpkg_geometry_columns (
                        table_name TEXT, column_name TEXT, geometry_type_name TEXT,
                        srs_id INTEGER, z INTEGER, m INTEGER,
                        PRIMARY KEY (table_name, column_name)
                     );
                     CREATE TABLE gpkg_contents (
                        table_name TEXT PRIMARY KEY, data_type TEXT NOT NULL, srs_id INTEGER
                     );
                     CREATE TABLE features (id INTEGER PRIMARY KEY, geom BLOB, label TEXT);",
                )
                .unwrap();
            connection
                .execute(
                    "INSERT INTO gpkg_contents VALUES ('features', 'features', ?1)",
                    [srid],
                )
                .unwrap();
            connection
                .execute(
                    "INSERT INTO gpkg_spatial_ref_sys VALUES ('test', ?1, 'test', ?1, 'test', '')",
                    [srid],
                )
                .unwrap();
            connection
                .execute(
                    "INSERT INTO gpkg_geometry_columns VALUES ('features', 'geom', 'POINT', ?1, ?2, ?3)",
                    rusqlite::params![srid, z, m],
                )
                .unwrap();
            connection
                .execute("INSERT INTO features VALUES (1, ?1, 'fixture')", [geometry])
                .unwrap();
            drop(connection);
            Self(path)
        }

        fn trusted(&self) -> TrustedGeoPackagePath {
            TrustedGeoPackagePath::from_internal_setup(&self.0).unwrap()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    fn point_gpkg(srid: i32, flags: u8, ordinates: &[f64]) -> Vec<u8> {
        let mut bytes = b"GP\0".to_vec();
        bytes.push(flags | 1);
        bytes.extend_from_slice(&srid.to_le_bytes());
        bytes.push(1);
        let kind = if ordinates.len() == 3 {
            1001_u32
        } else {
            1_u32
        };
        bytes.extend_from_slice(&kind.to_le_bytes());
        for ordinate in ordinates {
            bytes.extend_from_slice(&ordinate.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn loads_synthetic_gpkg_read_only_and_preserves_validated_values() {
        let fixture = Fixture::new(Some(point_gpkg(4326, 0, &[1.0, 2.0])), 4326);
        let before = fs::read(&fixture.0).unwrap();
        let result = load_geopackage(&fixture.trusted()).unwrap();
        assert_eq!(result.columns, ["id", "geom", "label"]);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], Value::Integer(1));
        assert_eq!(result.rows[0][2], Value::Text("fixture".to_owned()));
        assert_eq!(
            serialize(std::slice::from_ref(&result), Format::Json).unwrap(),
            r#"{"Table":[["1","47500001E61000000101000000000000000000F03F0000000000000040","fixture"]]}"#
        );
        assert_eq!(fs::read(&fixture.0).unwrap(), before);

        let readonly = rusqlite::Connection::open_with_flags(
            &fixture.0,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .unwrap();
        assert!(readonly
            .execute("INSERT INTO features (id, label) VALUES (2, 'write')", [])
            .is_err());
    }

    #[test]
    fn rejects_untrusted_or_inconsistent_geo_packages() {
        let fixture = Fixture::new(Some(point_gpkg(3857, 0, &[1.0, 2.0])), 4326);
        let error =
            validate_geometry_value(Value::Blob(point_gpkg(3857, 0, &[1.0, 2.0])), 4326, 0, 0)
                .unwrap_err();
        assert!(error.to_string().contains("SRID"));
        assert!(load_geopackage(&fixture.trusted()).is_err());
        assert!(TrustedGeoPackagePath::from_internal_setup(Path::new(
            "/definitely/not/a/geopackage"
        ))
        .is_err());
    }

    #[test]
    fn validates_null_empty_and_dimensional_geometries() {
        let null_fixture = Fixture::new(None, 4326);
        assert_eq!(
            load_geopackage(&null_fixture.trusted()).unwrap().rows[0][1],
            Value::Null
        );

        let empty = point_gpkg(4326, 0x10, &[f64::NAN, f64::NAN]);
        let empty_result = validate_geometry_value(Value::Blob(empty), 4326, 0, 0);
        assert!(empty_result.is_ok(), "{empty_result:?}");
        let xyz = point_gpkg(4326, 0, &[1.0, 2.0, 3.0]);
        let xyz_fixture = Fixture::new_dimensional(Some(xyz), 4326, 1, 0);
        let loaded_xyz = load_geopackage(&xyz_fixture.trusted());
        assert!(loaded_xyz.is_ok(), "{loaded_xyz:?}");
        let mismatch = point_gpkg(4326, 0, &[1.0, 2.0, 3.0]);
        assert!(validate_geometry_value(Value::Blob(mismatch), 4326, 0, 0).is_err());

        let mut malformed = point_gpkg(4326, 0, &[1.0, 2.0]);
        malformed.pop();
        assert!(validate_geometry_value(Value::Blob(malformed), 4326, 0, 0).is_err());
    }

    #[test]
    fn geometry_values_are_rejected_without_geo_package_validation() {
        let error = map_value(DuckValue::Geometry(vec![1, 2, 3])).unwrap_err();
        assert!(matches!(
            error,
            ExecutorError::UnsupportedValue(ref kind)
                if kind == "GEOMETRY payloads are not validated"
        ));
    }
}
