use std::sync::atomic::{AtomicUsize, Ordering};

use duckdb::{types::Value as DuckValue, Connection as DuckConnection};
use polyglot_sql::{Dialect, DialectType};
use rusqlite::types::Value;
use sda_rs::{
    result::ResultSet,
    serializer::{serialize, Format},
};

const DUCKDB_SCHEMA: &str = r#"
CREATE TABLE "sample data" (id INTEGER, name VARCHAR, optional VARCHAR);
INSERT INTO "sample data" VALUES (1, 'alpha', NULL), (2, 'beta', 'present'), (3, 'gamma', NULL);
CREATE TABLE legend (lkey INTEGER);
CREATE TABLE mapunit (lkey INTEGER, mukey VARCHAR);
CREATE TABLE muaggatt (mukey VARCHAR, val VARCHAR);
INSERT INTO legend VALUES (1);
INSERT INTO mapunit VALUES (1, '1857266');
INSERT INTO muaggatt VALUES ('1857266', 'synthetic');
"#;

const SQLITE_SCHEMA: &str = r#"
CREATE TABLE [sample data] ([id] INTEGER, [name] TEXT, [optional] TEXT);
INSERT INTO [sample data] VALUES (1, 'alpha', NULL), (2, 'beta', 'present'), (3, 'gamma', NULL);
CREATE TABLE legend (lkey INTEGER);
CREATE TABLE mapunit (lkey INTEGER, mukey TEXT);
CREATE TABLE muaggatt (mukey TEXT, val TEXT);
INSERT INTO legend VALUES (1);
INSERT INTO mapunit VALUES (1, '1857266');
INSERT INTO muaggatt VALUES ('1857266', 'synthetic');
"#;

fn duckdb_sql(input: &str) -> Vec<String> {
    Dialect::get(DialectType::TSQL)
        .transpile(input, DialectType::DuckDB)
        .unwrap_or_else(|error| panic!("T-SQL to DuckDB translation failed: {error}"))
}

// Test-only adapter: map only DuckDB values whose textual SDA representation
// matches the SQLite executor. Everything else is rejected rather than
// stringified, because the shared serializer contract cannot represent it.
fn duck_value(value: DuckValue) -> Result<Value, String> {
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
        other => return Err(format!("unsupported/lossy DuckDB result type: {other:?}")),
    })
}

fn duckdb_sets(connection: &DuckConnection, input: &str) -> Vec<ResultSet> {
    let statements = duckdb_sql(input);
    assert_eq!(
        statements.len(),
        1,
        "expected one translated statement: {statements:?}"
    );
    let mut statement = connection
        .prepare(&statements[0])
        .expect("prepare DuckDB SQL");
    let mut rows = statement.query([]).expect("execute DuckDB SQL");
    // DuckDB column metadata is only available once the statement has run.
    let columns = rows
        .as_ref()
        .expect("DuckDB produced a statement")
        .column_names();
    let mut result_rows = Vec::new();
    while let Some(row) = rows.next().expect("read DuckDB row") {
        let mut values = Vec::new();
        for index in 0..columns.len() {
            let value = row.get::<_, DuckValue>(index).expect("read DuckDB value");
            values.push(duck_value(value).expect("supported DuckDB value"));
        }
        result_rows.push(values);
    }
    vec![ResultSet {
        columns,
        rows: result_rows,
    }]
}

fn sqlite_sets(input: &str) -> Vec<ResultSet> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = format!(
        "file:duckdb_pipeline_contract_{}_{}?mode=memory&cache=shared",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let holder = rusqlite::Connection::open(&path).expect("open shared SQLite database");
    holder
        .execute_batch(SQLITE_SCHEMA)
        .expect("seed SQLite schema");
    let statements = sda_rs::transpiler::transpile(input).expect("transpile to SQLite");
    sda_rs::executor::execute(&path, &statements).expect("execute on SQLite")
}

fn assert_matches_sqlite(input: &str, expected_columns: &[&str], expected_json: &str) {
    let connection = DuckConnection::open_in_memory().expect("open in-memory DuckDB");
    connection
        .execute_batch(DUCKDB_SCHEMA)
        .expect("seed DuckDB schema");
    let duckdb_sets = duckdb_sets(&connection, input);
    let sqlite_sets = sqlite_sets(input);

    assert_eq!(duckdb_sets.len(), 1, "DuckDB result sets");
    assert_eq!(sqlite_sets.len(), 1, "SQLite result sets");
    assert_eq!(
        duckdb_sets[0].columns, sqlite_sets[0].columns,
        "column names and order"
    );
    assert_eq!(duckdb_sets[0].columns, expected_columns);
    assert_eq!(
        serialize(&duckdb_sets, Format::Json).expect("serialize DuckDB results"),
        expected_json
    );
    assert_eq!(
        serialize(&sqlite_sets, Format::Json).expect("serialize SQLite results"),
        expected_json
    );
}

#[test]
fn executes_scalar_query_in_memory() {
    let connection = DuckConnection::open_in_memory().expect("open in-memory DuckDB");
    let answer: i32 = connection
        .query_row("SELECT 40 + 2 AS answer", [], |row| row.get(0))
        .expect("execute scalar query");

    assert_eq!(answer, 42);
}

#[test]
fn translates_top_and_isnull_to_duckdb_dialect() {
    let generated = duckdb_sql(
        "SELECT TOP (2) [id], ISNULL([optional], 'missing') AS [value] FROM [sample data] ORDER BY [id]",
    );

    assert_eq!(generated.len(), 1);
    assert_eq!(
        generated[0],
        "SELECT \"id\", COALESCE(\"optional\", 'missing') AS \"value\" FROM \"sample data\" ORDER BY \"id\" NULLS FIRST LIMIT 2"
    );
    assert!(generated[0].contains("LIMIT 2"), "{}", generated[0]);
    assert!(generated[0].contains("COALESCE"), "{}", generated[0]);
    assert!(!generated[0].contains("ISNULL"), "{}", generated[0]);
}

#[test]
fn translates_bracketed_identifiers_to_quoted_identifiers() {
    let generated =
        duckdb_sql("SELECT [sample data].[id] AS [identifier with space] FROM [sample data]");

    assert_eq!(generated.len(), 1);
    assert_eq!(
        generated[0],
        "SELECT \"sample data\".\"id\" AS \"identifier with space\" FROM \"sample data\""
    );
    assert!(!generated[0].contains('['), "{}", generated[0]);
    assert!(!generated[0].contains(']'), "{}", generated[0]);
}

#[test]
fn translates_string_agg_to_duckdb_listagg() {
    let generated = duckdb_sql("SELECT STRING_AGG([name], ',') AS [names] FROM [sample data]");

    assert_eq!(generated.len(), 1);
    assert_eq!(
        generated[0],
        "SELECT LISTAGG(\"name\", ',') AS \"names\" FROM \"sample data\""
    );
    assert!(generated[0].contains("LISTAGG"), "{}", generated[0]);
    assert!(!generated[0].contains("STRING_AGG"), "{}", generated[0]);
}

#[test]
fn raw_dialect_leaves_iif_unresolved() {
    let generated =
        duckdb_sql("SELECT IIF([id] > 1, 'yes', 'no') AS [flag] FROM [sample data] ORDER BY [id]");

    assert_eq!(generated.len(), 1);
    assert_eq!(
        generated[0],
        "SELECT IIF(\"id\" > 1, 'yes', 'no') AS \"flag\" FROM \"sample data\" ORDER BY \"id\" NULLS FIRST"
    );
    assert!(generated[0].contains("IIF("), "{}", generated[0]);
    assert!(!generated[0].contains("CASE"), "{}", generated[0]);
}

#[test]
fn executes_top_brackets_isnull_and_matches_sqlite() {
    assert_matches_sqlite(
        "SELECT TOP (2) [id], ISNULL([optional], 'missing') AS [value] FROM [sample data] ORDER BY [id]",
        &["id", "value"],
        r#"{"Table":[["1","missing"],["2","present"]]}"#,
    );
}

#[test]
fn executes_captured_muaggatt_join_and_matches_sqlite() {
    assert_matches_sqlite(
        include_str!("fixtures/soildb/get-sda-muaggatt.sql"),
        &["mukey", "mukey", "val"],
        r#"{"Table":[["1857266","1857266","synthetic"]]}"#,
    );
}

#[test]
fn executes_string_agg_and_matches_sqlite() {
    assert_matches_sqlite(
        "SELECT STRING_AGG([name], ',') AS [names] FROM [sample data]",
        &["names"],
        r#"{"Table":[["alpha,beta,gamma"]]}"#,
    );
}

#[test]
fn executes_bracketed_identifiers_and_preserves_nulls() {
    assert_matches_sqlite(
        "SELECT [id] AS [row id], [optional] AS [maybe text] FROM [sample data] ORDER BY [id]",
        &["row id", "maybe text"],
        r#"{"Table":[["1",null],["2","present"],["3",null]]}"#,
    );
}
