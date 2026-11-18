use sda_rs::executor::execute_spatial;

#[test]
fn public_executor_accepts_normal_sql_and_rejects_default_sql_text_over_limit() {
    let output = execute_spatial(":memory:", &["SELECT 1".into()], None, None)
        .expect("normal public executor query should succeed");
    assert_eq!(output.result_sets[0].rows.len(), 1);

    let prefix = "SELECT 1 --";
    let over_limit_sql = format!("{prefix}{}", "x".repeat(2 * 1024 * 1024 + 1 - prefix.len()));
    assert!(
        execute_spatial(":memory:", &[over_limit_sql], None, None).is_err(),
        "public executor must reject SQL text over the default limit"
    );
}
