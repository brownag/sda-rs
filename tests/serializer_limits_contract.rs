use rusqlite::types::Value;
use sda_rs::{
    result::ResultSet,
    serializer::{serialize, Format},
};

#[test]
fn public_serializer_preserves_json_contract_below_limit() {
    let result = serialize(
        &[ResultSet {
            columns: vec!["id".into(), "label".into()],
            rows: vec![vec![Value::Integer(7), Value::Text("soil".into())]],
        }],
        Format::Json,
    )
    .unwrap();

    assert_eq!(result, r#"{"Table":[["7","soil"]]}"#);
}
