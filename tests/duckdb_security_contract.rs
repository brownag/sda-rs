use std::time::Duration;

use rusqlite::types::Value;
use sda_rs::executor::duckdb::execute;

async fn run(sql: &[&str]) -> Result<Vec<sda_rs::result::ResultSet>, String> {
    execute(
        sql.iter()
            .map(|statement| (*statement).to_owned())
            .collect(),
        Duration::from_secs(10),
    )
    .await
    .map_err(|error| error.to_string())
}

#[tokio::test]
async fn permits_temp_table_workflow_and_cleans_it_up() {
    let results = run(&[
        "CREATE TEMP TABLE chosen AS SELECT 7 AS id",
        "INSERT INTO chosen VALUES (8)",
        "SELECT id FROM chosen ORDER BY id",
        "DROP TABLE chosen",
    ])
    .await
    .unwrap();
    assert_eq!(
        results[2].rows,
        [vec![Value::Integer(7)], vec![Value::Integer(8)]]
    );
    assert!(run(&["SELECT * FROM chosen"]).await.is_err());
}

#[tokio::test]
async fn rejects_persistent_write_and_schema_statements() {
    for sql in [
        "CREATE TABLE durable (id INTEGER)",
        "CREATE SCHEMA durable",
        "INSERT INTO durable VALUES (1)",
        "UPDATE durable SET id = 2",
        "DELETE FROM durable",
        "COPY (SELECT 1) TO '/tmp/duckdb-security-output.csv'",
    ] {
        assert!(run(&[sql]).await.is_err(), "accepted {sql}");
    }
}

#[tokio::test]
async fn rejects_file_network_and_extension_access() {
    for sql in [
        "SELECT * FROM read_csv('/etc/passwd')",
        "SELECT * FROM read_parquet('https://example.invalid/data.parquet')",
        "INSTALL spatial",
        "LOAD spatial",
        "SET autoload_known_extensions = true",
        "SET autoinstall_known_extensions = true",
        "SET enable_external_access = true",
    ] {
        assert!(run(&[sql]).await.is_err(), "accepted {sql}");
    }
}

#[tokio::test]
async fn rejects_untracked_temp_table_targets_and_compound_sql() {
    for sql in [
        "INSERT INTO arbitrary VALUES (1)",
        "DROP TABLE arbitrary",
        "SELECT 1; CREATE TABLE durable (id INTEGER)",
        "SELECT 1 -- comment",
        "SELECT 1 # comment",
        "SELECT 1 AS value$tag",
        "SELECT 1; ATTACH ':memory:' AS probe",
        "SELECT 1 /* comment */",
        "SELECT 1; SET autoload_known_extensions = true",
    ] {
        assert!(run(&[sql]).await.is_err(), "accepted {sql}");
    }
}

#[tokio::test]
async fn rejects_mutating_keywords_inside_select_text() {
    for sql in [
        "SELECT 1 AS x ATTACH ':memory:' AS probe",
        "SELECT 1 AS x SET autoload_known_extensions = true",
        "SELECT 1 AS x CREATE TABLE durable (id INTEGER)",
        "SELECT 1 AS x INSTALL spatial",
    ] {
        assert!(run(&[sql]).await.is_err(), "accepted {sql}");
    }
}
