use std::{
    error::Error,
    fmt,
    time::{Duration, Instant},
};

use rusqlite::{
    hooks::{AuthAction, Authorization, TransactionOperation},
    limits::Limit,
    types::Value,
    Connection,
};

use crate::{
    result::ResultSet,
    spatial::{self, SpatialCapability},
};

const SQL_TEXT_LIMIT: i32 = 2 * 1024 * 1024;
const SQLITE_VALUE_LIMIT: i32 = 64 * 1024 * 1024;
const EXECUTION_DEADLINE: Duration = Duration::from_secs(60);
const MAX_RETURNED_ROWS: usize = 100_000;

#[derive(Clone, Copy)]
struct ExecutionLimits {
    sql_text: i32,
    sqlite_value: i32,
    deadline: Duration,
    returned_rows: usize,
}

impl ExecutionLimits {
    const DEFAULT: Self = Self {
        sql_text: SQL_TEXT_LIMIT,
        sqlite_value: SQLITE_VALUE_LIMIT,
        deadline: EXECUTION_DEADLINE,
        returned_rows: MAX_RETURNED_ROWS,
    };
}

#[derive(Debug)]
pub enum ExecutorError {
    Sqlite(rusqlite::Error),
    Spatial(spatial::SpatialError),
    Preparation(String),
    Join(tokio::task::JoinError),
    ReturnedRowLimit(usize),
    Rollback {
        execution: Box<Self>,
        rollback: rusqlite::Error,
    },
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "SQLite execution failed: {error}"),
            Self::Spatial(error) => write!(f, "{error}"),
            Self::Preparation(error) => write!(f, "SQL preparation failed: {error}"),
            Self::Join(error) => write!(f, "SQLite worker failed: {error}"),
            Self::ReturnedRowLimit(limit) => {
                write!(f, "SQLite returned-row limit exceeded ({limit})")
            }
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
    let (connection, spatial) = open_initialized(path, extension, geometry_override)?;
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
        let (connection, spatial) =
            open_initialized(&path, extension.as_deref(), geometry_override.as_deref())?;
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

fn open_initialized(
    path: &str,
    extension: Option<&str>,
    geometry_override: Option<&str>,
) -> Result<(Connection, SpatialCapability), ExecutorError> {
    open_initialized_with_limits(path, extension, geometry_override, ExecutionLimits::DEFAULT)
}

fn open_initialized_with_limits(
    path: &str,
    extension: Option<&str>,
    geometry_override: Option<&str>,
    limits: ExecutionLimits,
) -> Result<(Connection, SpatialCapability), ExecutorError> {
    let connection = open_connection(path)?;
    let spatial = spatial::initialize(&connection, extension, geometry_override)?;
    connection.set_limit(Limit::SQLITE_LIMIT_SQL_LENGTH, limits.sql_text);
    connection.set_limit(Limit::SQLITE_LIMIT_LENGTH, limits.sqlite_value);
    let started = Instant::now();
    connection.progress_handler(1000, Some(move || started.elapsed() >= limits.deadline));
    connection.authorizer(Some(authorize_sqlite_action));
    Ok((connection, spatial))
}

fn authorize_sqlite_action(context: rusqlite::hooks::AuthContext<'_>) -> Authorization {
    let temp_database = context.database_name == Some("temp");
    let allowed = match context.action {
        AuthAction::Select | AuthAction::Read { .. } => true,
        AuthAction::CreateTempTable { .. } | AuthAction::DropTempTable { .. } => temp_database,
        AuthAction::Insert { .. } | AuthAction::Update { .. } | AuthAction::Delete { .. } => {
            temp_database
        }
        AuthAction::Transaction {
            operation: TransactionOperation::Begin,
        } => true,
        AuthAction::Function { function_name } => {
            !function_name.eq_ignore_ascii_case("load_extension")
        }
        _ => false,
    };

    if allowed {
        Authorization::Allow
    } else {
        Authorization::Deny
    }
}

fn execute_on(
    connection: &Connection,
    statements: &[String],
) -> Result<Vec<ResultSet>, ExecutorError> {
    execute_on_with_limits(connection, statements, ExecutionLimits::DEFAULT)
}

fn execute_on_with_limits(
    connection: &Connection,
    statements: &[String],
    limits: ExecutionLimits,
) -> Result<Vec<ResultSet>, ExecutorError> {
    let transaction = connection.unchecked_transaction()?;
    let execution = (|| {
        let mut result_sets = Vec::new();
        let mut returned_rows = 0usize;

        for sql in statements {
            let mut statement = transaction.prepare(sql)?;
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
                if returned_rows >= limits.returned_rows {
                    return Err(ExecutorError::ReturnedRowLimit(limits.returned_rows));
                }
                let mut result_row = Vec::with_capacity(column_count);
                for index in 0..column_count {
                    result_row.push(row.get::<_, Value>(index)?);
                }
                values.push(result_row);
                returned_rows += 1;
            }
            result_sets.push(ResultSet {
                columns,
                rows: values,
            });
        }
        Ok::<_, ExecutorError>(result_sets)
    })();

    match execution {
        Ok(result_sets) => {
            // No submitted SQL runs while authorization is suspended; bypass
            // only the synchronous executor-owned commit operation.
            connection.authorizer(None::<fn(rusqlite::hooks::AuthContext<'_>) -> Authorization>);
            let commit = transaction.commit();
            connection.authorizer(Some(authorize_sqlite_action));
            commit?;
            Ok(result_sets)
        }
        Err(execution) => {
            // Explicitly roll back before returning, with authorization
            // suspended only for this synchronous executor-owned operation.
            connection.authorizer(None::<fn(rusqlite::hooks::AuthContext<'_>) -> Authorization>);
            let rollback = transaction.rollback();
            connection.authorizer(Some(authorize_sqlite_action));
            match rollback {
                Ok(()) => Err(execution),
                Err(rollback) => Err(ExecutorError::Rollback {
                    execution: Box::new(execution),
                    rollback,
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_SHARED_DB: AtomicUsize = AtomicUsize::new(0);

    fn shared_memory_uri() -> String {
        format!(
            "file:g16_{}_{}?mode=memory&cache=shared",
            std::process::id(),
            NEXT_SHARED_DB.fetch_add(1, Ordering::Relaxed)
        )
    }

    fn small_limits() -> ExecutionLimits {
        ExecutionLimits {
            sql_text: 4096,
            sqlite_value: 512,
            deadline: Duration::from_secs(60),
            returned_rows: 2,
        }
    }

    fn limited_connection(limits: ExecutionLimits) -> Connection {
        open_initialized_with_limits(":memory:", None, None, limits)
            .unwrap()
            .0
    }

    fn assert_value_length_boundary(
        connection: &Connection,
        limits: ExecutionLimits,
        query_for_length: impl Fn(usize) -> String,
    ) {
        let mut accepted = 0;
        let mut rejected = limits.sqlite_value as usize + 1;
        while rejected - accepted > 1 {
            let candidate = accepted + (rejected - accepted) / 2;
            if execute_on_with_limits(connection, &[query_for_length(candidate)], limits).is_ok() {
                accepted = candidate;
            } else {
                rejected = candidate;
            }
        }

        assert!(
            execute_on_with_limits(connection, &[query_for_length(accepted)], limits).is_ok(),
            "largest accepted value should remain accepted"
        );
        assert!(
            execute_on_with_limits(connection, &[query_for_length(rejected)], limits).is_err(),
            "first value beyond the effective SQLite boundary should be rejected"
        );
    }

    #[test]
    fn enforces_sql_text_limit_at_and_over_boundary() {
        let limits = ExecutionLimits {
            sql_text: 64,
            ..small_limits()
        };
        let connection = limited_connection(limits);
        let exact_sql = format!("SELECT 1 --{}", "x".repeat(64 - "SELECT 1 --".len()));
        let over_sql = format!("{exact_sql}x");

        assert_eq!(
            execute_on_with_limits(&connection, &[exact_sql], limits).unwrap()[0].rows,
            [vec![Value::Integer(1)]]
        );
        assert!(execute_on_with_limits(&connection, &[over_sql], limits).is_err());
    }

    #[test]
    fn enforces_string_and_blob_limits_at_and_over_boundary() {
        let limits = small_limits();
        let connection = limited_connection(limits);
        // SQLite also applies this limit to internal row/string bookkeeping;
        // find the effective byte boundary, then verify the next byte fails.
        assert_value_length_boundary(&connection, limits, |length| {
            format!("SELECT length('{}')", "x".repeat(length))
        });
        assert_value_length_boundary(&connection, limits, |length| {
            format!("SELECT length(zeroblob({length}))")
        });
    }

    #[test]
    fn progress_handler_allows_quick_query_and_interrupts_cpu_query() {
        let quick_limits = ExecutionLimits {
            deadline: Duration::from_millis(10),
            ..small_limits()
        };
        let connection = limited_connection(quick_limits);
        assert!(execute_on_with_limits(&connection, &["SELECT 1".into()], quick_limits).is_ok());

        let expired_limits = ExecutionLimits {
            deadline: Duration::ZERO,
            ..small_limits()
        };
        let connection = limited_connection(expired_limits);
        let cpu_query = "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x + 1 FROM n WHERE x < 10000000) SELECT sum(x) FROM n";
        assert!(execute_on_with_limits(&connection, &[cpu_query.into()], expired_limits).is_err());
    }

    #[test]
    fn enforces_total_row_limit_across_statements() {
        let limits = small_limits();
        let connection = limited_connection(limits);
        let boundary = vec!["SELECT 1 UNION ALL SELECT 2".into()];
        assert_eq!(
            execute_on_with_limits(&connection, &boundary, limits).unwrap()[0]
                .rows
                .len(),
            2
        );

        let over = vec!["SELECT 1 UNION ALL SELECT 2".into(), "SELECT 3".into()];
        assert!(matches!(
            execute_on_with_limits(&connection, &over, limits),
            Err(ExecutorError::ReturnedRowLimit(2))
        ));
    }

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
    fn isolates_select_into_temp_name_from_persistent_table() {
        let uri = shared_memory_uri();
        let holder = Connection::open(&uri).unwrap();
        holder
            .execute_batch(
                "CREATE TABLE sample(value INTEGER);
                 INSERT INTO sample VALUES (10), (20);",
            )
            .unwrap();
        let statements = crate::transpiler::transpile(
            "SELECT 99 AS value INTO #sample; \
             SELECT COUNT(*) AS count FROM sample; \
             SELECT COUNT(*) AS count FROM #sample",
        )
        .unwrap();

        let results = execute(&uri, &statements).unwrap();
        assert_eq!(results[0].rows, [vec![Value::Integer(2)]]);
        assert_eq!(results[1].rows, [vec![Value::Integer(1)]]);
    }

    #[test]
    fn rolls_back_failed_statement_batch() {
        let (connection, _) = open_initialized(":memory:", None, None).unwrap();
        connection
            .execute_batch("CREATE TEMP TABLE t(x INTEGER); INSERT INTO t VALUES (1);")
            .unwrap();

        let result = execute_on(
            &connection,
            &["UPDATE t SET x = 2".into(), "SELECT * FROM missing".into()],
        );

        assert!(result.is_err());
        let value: i64 = connection
            .query_row("SELECT x FROM t", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, 1);
    }

    #[test]
    fn commits_successful_statement_batch() {
        let (connection, _) = open_initialized(":memory:", None, None).unwrap();
        let results = execute_on(
            &connection,
            &[
                "CREATE TEMP TABLE t(x INTEGER)".into(),
                "INSERT INTO t VALUES (1)".into(),
                "UPDATE t SET x = 2".into(),
                "SELECT x FROM t".into(),
            ],
        )
        .unwrap();

        assert_eq!(results[0].rows, [vec![Value::Integer(2)]]);
        let value: i64 = connection
            .query_row("SELECT x FROM t", [], |row| row.get(0))
            .unwrap();
        assert_eq!(value, 2);
    }

    #[test]
    fn preserves_empty_result_set_columns() {
        let results =
            execute(":memory:", &["SELECT 1 AS id WHERE 0".into()]).expect("empty select succeeds");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].columns, ["id"]);
        assert!(results[0].rows.is_empty());
    }

    #[test]
    fn executes_single_select() {
        let results = execute(":memory:", &["SELECT 3 AS value".into()]).unwrap();
        assert_eq!(results[0].rows, [vec![Value::Integer(3)]]);
    }

    #[test]
    fn ordinary_executor_exposes_missing_spatial_metadata_without_failure() {
        let output = execute_spatial(":memory:", &["SELECT 1".into()], None, None).unwrap();
        assert!(!output.spatial.available());
        assert_eq!(
            output.spatial.geometry_metadata_for_table("mupolygon"),
            None
        );
    }
}
