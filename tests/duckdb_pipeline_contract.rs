use duckdb::Connection;
use polyglot_sql::{Dialect, DialectType};

fn duckdb_sql(input: &str) -> Vec<String> {
    Dialect::get(DialectType::TSQL)
        .transpile(input, DialectType::DuckDB)
        .unwrap_or_else(|error| panic!("T-SQL to DuckDB translation failed: {error}"))
}

#[test]
fn executes_scalar_query_in_memory() {
    let connection = Connection::open_in_memory().expect("open in-memory DuckDB");
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
