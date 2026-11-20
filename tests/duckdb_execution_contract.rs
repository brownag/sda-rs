use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use duckdb::Connection;
use rusqlite::types::Value;
use sda_rs::executor::duckdb::{execute, ExecutorError};

#[test]
fn pinned_interrupt_retries_across_prepare_execution_race() {
    let connection = Connection::open_in_memory().unwrap();
    let interrupt = connection.interrupt_handle();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = connection
            .prepare("SELECT count(*) FROM range(10000000) t1, range(1000000) t2")
            .and_then(|mut statement| statement.execute([]))
            .map(|_| ())
            .map_err(|error| error.to_string());
        sender.send(result).unwrap();
    });

    let deadline = Instant::now() + Duration::from_secs(5);
    let result = loop {
        interrupt.interrupt();
        match receiver.recv_timeout(Duration::from_millis(1)) {
            Ok(result) => break result,
            Err(mpsc::RecvTimeoutError::Timeout) if Instant::now() < deadline => {}
            Err(mpsc::RecvTimeoutError::Timeout) => panic!("pinned DuckDB interrupt not observed"),
            Err(mpsc::RecvTimeoutError::Disconnected) => panic!("query worker exited"),
        }
    };
    assert!(
        result.as_ref().is_err_and(|error| error.contains("INTERRUPT")),
        "expected DuckDB interrupt error, got {result:?}"
    );
}

#[tokio::test]
async fn temporary_tables_work_and_are_request_local() {
    let batch = vec![
        "CREATE TEMP TABLE chosen AS SELECT 7 AS id".to_owned(),
        "SELECT id FROM chosen".to_owned(),
    ];
    let results = execute(batch, Duration::from_secs(5)).await.unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[1].columns, ["id"]);
    assert_eq!(results[1].rows, [vec![Value::Integer(7)]]);

    let next_request = execute(
        vec!["SELECT id FROM chosen".to_owned()],
        Duration::from_secs(5),
    )
    .await;
    assert!(next_request.is_err(), "temporary table leaked to next request");
}

#[tokio::test]
async fn later_statement_failure_fails_whole_batch_and_cleans_up() {
    let failed = execute(
        vec![
            "CREATE TEMP TABLE earlier AS SELECT 1 AS id".to_owned(),
            "SELECT * FROM missing_table".to_owned(),
        ],
        Duration::from_secs(5),
    )
    .await;
    assert!(failed.is_err());

    let next_request = execute(
        vec!["SELECT * FROM earlier".to_owned()],
        Duration::from_secs(5),
    )
    .await;
    assert!(next_request.is_err(), "failed batch state leaked to next request");
}

#[tokio::test]
async fn deadline_interrupts_query_despite_prepare_execution_race() {
    let result = execute(
        vec![
            "SELECT count(*) FROM range(10000000) t1, range(1000000) t2".to_owned(),
        ],
        Duration::from_millis(10),
    )
    .await;
    assert!(matches!(result, Err(ExecutorError::DeadlineExceeded)));
}
