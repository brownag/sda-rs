use duckdb::Connection;

#[test]
fn executes_scalar_query_in_memory() {
    let connection = Connection::open_in_memory().expect("open in-memory DuckDB");
    let answer: i32 = connection
        .query_row("SELECT 40 + 2 AS answer", [], |row| row.get(0))
        .expect("execute scalar query");

    assert_eq!(answer, 42);
}
