mod sqlite;

#[cfg(feature = "duckdb")]
pub mod duckdb;

pub use sqlite::{
    execute, execute_query_blocking, execute_spatial, ExecutionOutput, ExecutorError,
};
