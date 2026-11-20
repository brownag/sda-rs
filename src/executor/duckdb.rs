use std::{
    error::Error,
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use duckdb::{types::Value as DuckValue, Connection};
use rusqlite::types::Value;

use crate::result::ResultSet;

#[derive(Debug)]
pub enum ExecutorError {
    DuckDb(duckdb::Error),
    Join(tokio::task::JoinError),
    UnsupportedValue(String),
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
    let connection = Connection::open_in_memory()?;
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
        for sql in statements {
            if started.elapsed() >= deadline {
                return Err(ExecutorError::DeadlineExceeded);
            }
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
        DuckValue::UBigInt(value) => Value::Text(value.to_string()),
        DuckValue::Float(value) => Value::Real(f64::from(value)),
        DuckValue::Double(value) => Value::Real(value),
        DuckValue::Decimal(value) => Value::Text(value.to_string()),
        DuckValue::Text(value) => Value::Text(value),
        DuckValue::Blob(value) => Value::Blob(value),
        other => return Err(ExecutorError::UnsupportedValue(format!("{other:?}"))),
    })
}
