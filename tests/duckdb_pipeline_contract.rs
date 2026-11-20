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

fn duckdb_statement_sets(
    connection: &DuckConnection,
    statements: &[String],
) -> Vec<(String, ResultSet)> {
    statements
        .iter()
        .map(|sql| {
            let mut statement = connection.prepare(sql).expect("prepare DuckDB SQL");
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
            (
                sql.clone(),
                ResultSet {
                    columns,
                    rows: result_rows,
                },
            )
        })
        .collect()
}

fn duckdb_sets(connection: &DuckConnection, input: &str) -> Vec<ResultSet> {
    let statements = duckdb_sql(input);
    assert_eq!(
        statements.len(),
        1,
        "expected one translated statement: {statements:?}"
    );
    let mut sets = duckdb_statement_sets(connection, &statements);
    assert_eq!(sets.len(), 1, "expected one DuckDB result set");
    vec![sets.remove(0).1]
}

// Test-only adapter for the observed raw dialect gap: polyglot-sql leaves IIF
// untouched and DuckDB has no IIF function, so rewrite IIF(cond, a, b) to
// CASE WHEN cond THEN a ELSE b END for these probe-authored inputs. This is not
// a production transpiler fix and does not cover identifiers or general SQL.
fn lower_iif(sql: &str) -> String {
    let mut output = String::with_capacity(sql.len());
    let mut rest = sql;
    while let Some(start) = find_iif(rest) {
        output.push_str(&rest[..start]);
        let Some(close) = matching_paren(rest, start + 4) else {
            output.push_str(&rest[start..]);
            return output;
        };
        let arguments = split_arguments(&rest[start + 4..close]);
        assert_eq!(
            arguments.len(),
            3,
            "IIF requires three arguments: {}",
            &rest[start..=close]
        );
        output.push_str("CASE WHEN ");
        output.push_str(&lower_iif(arguments[0]));
        output.push_str(" THEN ");
        output.push_str(&lower_iif(arguments[1]));
        output.push_str(" ELSE ");
        output.push_str(&lower_iif(arguments[2]));
        output.push_str(" END");
        rest = &rest[close + 1..];
    }
    output.push_str(rest);
    output
}

fn find_iif(sql: &str) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut in_string = false;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 1,
            b'\'' => in_string = !in_string,
            _ if !in_string
                && index + 3 < bytes.len()
                && bytes[index].eq_ignore_ascii_case(&b'I')
                && bytes[index + 1].eq_ignore_ascii_case(&b'I')
                && bytes[index + 2].eq_ignore_ascii_case(&b'F')
                && bytes[index + 3] == b'(' =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn matching_paren(sql: &str, content_start: usize) -> Option<usize> {
    let bytes = sql.as_bytes();
    let mut depth = 1usize;
    let mut in_string = false;
    let mut index = content_start;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 1,
            b'\'' => in_string = !in_string,
            b'(' if !in_string => depth += 1,
            b')' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn split_arguments(content: &str) -> Vec<&str> {
    let bytes = content.as_bytes();
    let mut arguments = Vec::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 1,
            b'\'' => in_string = !in_string,
            b'(' if !in_string => depth += 1,
            b')' if !in_string => depth = depth.saturating_sub(1),
            b',' if !in_string && depth == 0 => {
                arguments.push(content[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    arguments.push(content[start..].trim());
    arguments
}

// Test-only response shaping: the DuckDB adapter collects the count row that
// `CREATE TABLE ... AS` returns, while SQLite's executor skips non-row-producing
// statements. Suppress only those DDL sets here; the shared serializer is
// unchanged and this does not claim a production batch contract.
fn is_non_row_producing_ctas(sql: &str) -> bool {
    let statement = sql.trim_start().to_ascii_uppercase();
    statement.starts_with("CREATE TEMPORARY TABLE") || statement.starts_with("CREATE TABLE")
}

fn duckdb_batch_sets(connection: &DuckConnection, input: &str) -> Vec<(String, ResultSet)> {
    duckdb_statement_sets(connection, &duckdb_sql(input))
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

#[test]
fn lowers_iif_and_matches_sqlite() {
    let input = "SELECT TOP (2) [id], ISNULL([optional], 'missing') AS [value], IIF([id] > 1, 'yes', 'no') AS [flag] FROM [sample data] ORDER BY [id]";
    let raw = duckdb_sql(input);
    assert_eq!(raw.len(), 1);
    assert!(raw[0].contains("IIF("), "{}", raw[0]);
    assert!(!raw[0].contains("CASE"), "{}", raw[0]);

    let connection = DuckConnection::open_in_memory().expect("open in-memory DuckDB");
    connection
        .execute_batch(DUCKDB_SCHEMA)
        .expect("seed DuckDB schema");
    let raw_error = match connection.prepare(&raw[0]) {
        Ok(_) => panic!("raw DuckDB IIF SQL must fail"),
        Err(error) => error,
    };
    assert!(
        raw_error.to_string().to_ascii_lowercase().contains("iif"),
        "{raw_error}"
    );

    let lowered = lower_iif(&raw[0]);
    assert_eq!(
        lowered,
        "SELECT \"id\", COALESCE(\"optional\", 'missing') AS \"value\", CASE WHEN \"id\" > 1 THEN 'yes' ELSE 'no' END AS \"flag\" FROM \"sample data\" ORDER BY \"id\" NULLS FIRST LIMIT 2"
    );
    let duckdb_sets = duckdb_statement_sets(&connection, std::slice::from_ref(&lowered));
    assert_eq!(duckdb_sets.len(), 1);
    let duckdb_set = &duckdb_sets[0].1;
    assert_eq!(duckdb_set.columns, ["id", "value", "flag"]);

    let sqlite_sets = sqlite_sets(input);
    assert_eq!(sqlite_sets.len(), 1);
    assert_eq!(duckdb_set.columns, sqlite_sets[0].columns);
    let expected = r#"{"Table":[["1","missing","no"],["2","present","yes"]]}"#;
    assert_eq!(
        serialize(std::slice::from_ref(duckdb_set), Format::Json)
            .expect("serialize lowered DuckDB results"),
        expected
    );
    assert_eq!(
        serialize(&sqlite_sets, Format::Json).expect("serialize SQLite results"),
        expected
    );
}

#[test]
fn suppresses_ctas_count_and_matches_sqlite_batch() {
    let input = "SELECT [id] INTO #picked FROM [sample data] WHERE [id] > 1; SELECT [id] FROM #picked ORDER BY [id]";
    let connection = DuckConnection::open_in_memory().expect("open in-memory DuckDB");
    connection
        .execute_batch(DUCKDB_SCHEMA)
        .expect("seed DuckDB schema");

    let batch = duckdb_batch_sets(&connection, input);
    assert_eq!(batch.len(), 2);
    assert_eq!(
        batch[0].0,
        "CREATE TEMPORARY TABLE picked AS SELECT \"id\" FROM \"sample data\" WHERE \"id\" > 1"
    );
    assert_eq!(
        batch[1].0,
        "SELECT \"id\" FROM picked ORDER BY \"id\" NULLS FIRST"
    );
    assert!(is_non_row_producing_ctas(&batch[0].0));
    assert!(!is_non_row_producing_ctas(&batch[1].0));

    let raw_sets: Vec<ResultSet> = batch
        .iter()
        .filter(|(_, set)| !set.rows.is_empty())
        .map(|(_, set)| set.clone())
        .collect();
    assert_eq!(
        serialize(&raw_sets, Format::Json).expect("serialize raw DuckDB batch"),
        r#"{"Table":[["2"]],"Table1":[["2"],["3"]]}"#
    );

    let adapted: Vec<ResultSet> = batch
        .iter()
        .filter(|(sql, _)| !is_non_row_producing_ctas(sql))
        .map(|(_, set)| set.clone())
        .collect();
    assert_eq!(adapted.len(), 1);
    assert_eq!(adapted[0].columns, ["id"]);
    let expected = r#"{"Table":[["2"],["3"]]}"#;
    assert_eq!(
        serialize(&adapted, Format::Json).expect("serialize adapted DuckDB batch"),
        expected
    );

    let sqlite_sets = sqlite_sets(input);
    assert_eq!(sqlite_sets.len(), 1);
    assert_eq!(adapted[0].columns, sqlite_sets[0].columns);
    assert_eq!(
        serialize(&sqlite_sets, Format::Json).expect("serialize SQLite batch"),
        expected
    );
}

#[test]
fn rejects_unsupported_temporal_value() {
    let connection = DuckConnection::open_in_memory().expect("open in-memory DuckDB");
    let value: DuckValue = connection
        .query_row("SELECT DATE '2024-01-02' AS date_value", [], |row| {
            row.get(0)
        })
        .expect("read DuckDB date value");
    let error = duck_value(value).expect_err("temporal values must be rejected");

    assert!(error.contains("unsupported/lossy"), "{error}");
    assert!(error.contains("Date32"), "{error}");
}
