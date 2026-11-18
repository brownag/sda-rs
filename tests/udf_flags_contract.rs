use rusqlite::{functions::FunctionFlags, Connection};

#[test]
fn direct_only_fixture_udf_is_directly_callable_but_not_from_a_view() {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .create_scalar_function(
            "fixture_value",
            0,
            FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DIRECTONLY,
            |_| Ok(42_i64),
        )
        .unwrap();

    let direct_value: i64 = connection
        .query_row("SELECT fixture_value()", [], |row| row.get(0))
        .unwrap();
    assert_eq!(direct_value, 42);

    connection
        .execute_batch("CREATE VIEW fixture_view AS SELECT fixture_value() AS value")
        .unwrap();
    let schema_invocation = connection.query_row("SELECT value FROM fixture_view", [], |row| {
        row.get::<_, i64>(0)
    });
    let error = schema_invocation.unwrap_err();
    assert!(error.to_string().contains("unsafe use of fixture_value"));
}
