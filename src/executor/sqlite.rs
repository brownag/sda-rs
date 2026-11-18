use std::{error::Error, fmt};

use rusqlite::{types::Value, Connection};

use crate::{
    result::ResultSet,
    spatial::{self, SpatialCapability},
};

/// An owned batch of SQL statements, executed on one request-local connection.
#[derive(Debug, Clone)]
pub struct SqliteExecutor {
    path: String,
    statements: Vec<String>,
    spatial_extension: Option<String>,
    geometry_column: Option<String>,
}

impl SqliteExecutor {
    pub fn new(
        path: impl Into<String>,
        statements: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            path: path.into(),
            statements: statements.into_iter().map(Into::into).collect(),
            spatial_extension: None,
            geometry_column: None,
        }
    }

    pub fn with_spatial_config(
        mut self,
        extension: Option<String>,
        geometry_column: Option<String>,
    ) -> Self {
        self.spatial_extension = extension;
        self.geometry_column = geometry_column;
        self
    }

    pub fn execute(self) -> Result<Vec<ResultSet>, ExecutorError> {
        Ok(execute_spatial(
            &self.path,
            &self.statements,
            self.spatial_extension.as_deref(),
            self.geometry_column.as_deref(),
        )?
        .result_sets)
    }
}

#[derive(Debug)]
pub enum ExecutorError {
    Sqlite(rusqlite::Error),
    Spatial(spatial::SpatialError),
    Preparation(String),
    Join(tokio::task::JoinError),
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "SQLite execution failed: {error}"),
            Self::Spatial(error) => write!(f, "{error}"),
            Self::Preparation(error) => write!(f, "SQL preparation failed: {error}"),
            Self::Join(error) => write!(f, "SQLite worker failed: {error}"),
        }
    }
}

impl Error for ExecutorError {}

impl From<rusqlite::Error> for ExecutorError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<spatial::SpatialError> for ExecutorError {
    fn from(value: spatial::SpatialError) -> Self {
        Self::Spatial(value)
    }
}

impl From<tokio::task::JoinError> for ExecutorError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::Join(value)
    }
}

/// Execute statements sequentially with a fresh in-memory connection.
pub fn execute(path: &str, statements: &[String]) -> Result<Vec<ResultSet>, ExecutorError> {
    Ok(execute_spatial(path, statements, None, None)?.result_sets)
}

#[derive(Debug)]
pub struct ExecutionOutput {
    pub result_sets: Vec<ResultSet>,
    pub spatial: SpatialCapability,
}

pub fn execute_spatial(
    path: &str,
    statements: &[String],
    extension: Option<&str>,
    geometry_override: Option<&str>,
) -> Result<ExecutionOutput, ExecutorError> {
    let connection = if path == ":memory:" {
        Connection::open_in_memory()?
    } else {
        Connection::open(path)?
    };
    let spatial = spatial::initialize(&connection, extension, geometry_override)?;
    if statements
        .iter()
        .any(|statement| spatial::requires_spatial(statement))
    {
        spatial.require_spatial()?;
    }
    let result_sets = execute_on(&connection, statements)?;
    Ok(ExecutionOutput {
        result_sets,
        spatial,
    })
}

/// Initialize request-local spatial state, then let the caller prepare SQL
/// while the capability/GeoPackage metadata is available, before execution.
pub async fn execute_query_blocking<F>(
    path: String,
    query: String,
    extension: Option<String>,
    geometry_override: Option<String>,
    prepare: F,
) -> Result<ExecutionOutput, ExecutorError>
where
    F: FnOnce(&SpatialCapability, &str) -> Result<Vec<String>, String> + Send + 'static,
{
    Ok(tokio::task::spawn_blocking(move || {
        let connection = open_connection(&path)?;
        let spatial = spatial::initialize(
            &connection,
            extension.as_deref(),
            geometry_override.as_deref(),
        )?;
        if spatial::requires_spatial(&query) {
            spatial.require_spatial()?;
        }
        let statements = prepare(&spatial, &query).map_err(ExecutorError::Preparation)?;
        if statements
            .iter()
            .any(|statement| spatial::requires_spatial(statement))
        {
            spatial.require_spatial()?;
        }
        let result_sets = execute_on(&connection, &statements)?;
        Ok::<_, ExecutorError>(ExecutionOutput {
            result_sets,
            spatial,
        })
    })
    .await??)
}

fn open_connection(path: &str) -> Result<Connection, rusqlite::Error> {
    if path == ":memory:" {
        Connection::open_in_memory()
    } else {
        Connection::open(path)
    }
}

/// Run SQLite work on Tokio's blocking pool.
pub async fn execute_blocking(
    path: String,
    statements: Vec<String>,
) -> Result<Vec<ResultSet>, ExecutorError> {
    Ok(tokio::task::spawn_blocking(move || execute(&path, &statements)).await??)
}

pub async fn execute_spatial_blocking(
    path: String,
    statements: Vec<String>,
    extension: Option<String>,
    geometry_override: Option<String>,
) -> Result<ExecutionOutput, ExecutorError> {
    Ok(tokio::task::spawn_blocking(move || {
        execute_spatial(
            &path,
            &statements,
            extension.as_deref(),
            geometry_override.as_deref(),
        )
    })
    .await??)
}

fn execute_on(
    connection: &Connection,
    statements: &[String],
) -> Result<Vec<ResultSet>, ExecutorError> {
    let mut result_sets = Vec::new();

    for sql in statements {
        let mut statement = connection.prepare(sql)?;
        let column_count = statement.column_count();
        if column_count == 0 {
            statement.execute([])?;
            continue;
        }

        let columns = statement
            .column_names()
            .into_iter()
            .map(str::to_owned)
            .collect();
        let mut rows = statement.query([])?;
        let mut values = Vec::new();
        while let Some(row) = rows.next()? {
            let mut result_row = Vec::with_capacity(column_count);
            for index in 0..column_count {
                result_row.push(row.get::<_, Value>(index)?);
            }
            values.push(result_row);
        }
        result_sets.push(ResultSet {
            columns,
            rows: values,
        });
    }

    Ok(result_sets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_column_order_values_and_nulls() {
        let results = execute(
            ":memory:",
            &["SELECT 7 AS second, 'text' AS first, NULL AS absent".into()],
        )
        .unwrap();
        assert_eq!(results[0].columns, ["second", "first", "absent"]);
        assert_eq!(
            results[0].rows,
            [vec![
                Value::Integer(7),
                Value::Text("text".into()),
                Value::Null
            ]]
        );
    }

    #[test]
    fn executes_multiple_statements_and_keeps_temp_tables_in_session() {
        let results = execute(
            ":memory:",
            &[
                "CREATE TEMP TABLE sample(value INTEGER)".into(),
                "INSERT INTO sample VALUES (11)".into(),
                "SELECT value FROM sample".into(),
                "SELECT 2 AS later".into(),
            ],
        )
        .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].rows, [vec![Value::Integer(11)]]);
        assert_eq!(results[1].columns, ["later"]);
    }

    #[test]
    fn preserves_empty_result_set_columns() {
        let results =
            execute(":memory:", &["SELECT 1 AS id WHERE 0".into()]).expect("empty select succeeds");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].columns, ["id"]);
        assert!(results[0].rows.is_empty());
    }

    #[tokio::test]
    async fn exposes_blocking_execution_boundary() {
        let results = execute_blocking(":memory:".into(), vec!["SELECT 3 AS value".into()])
            .await
            .unwrap();
        assert_eq!(results[0].rows, [vec![Value::Integer(3)]]);
    }

    #[test]
    fn ordinary_executor_exposes_missing_spatial_metadata_without_failure() {
        let output = execute_spatial(":memory:", &["SELECT 1".into()], None, None).unwrap();
        assert!(!output.spatial.available());
        assert_eq!(output.spatial.geometry_metadata(), None);
    }
}
