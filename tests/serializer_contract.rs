use rusqlite::types::Value;
use sda_rs::{
    result::ResultSet,
    serializer::{serialize, Format},
};

fn result_set(columns: &[&str], rows: Vec<Vec<Value>>) -> ResultSet {
    ResultSet {
        columns: columns.iter().map(|column| (*column).to_owned()).collect(),
        rows,
    }
}

#[test]
fn json_columnname_metadata_serializes_multiple_result_sets_exactly() {
    let result = serialize(
        &[
            result_set(
                &["id", "label", "missing"],
                vec![
                    vec![Value::Integer(2), Value::Text("x".into()), Value::Null],
                    vec![Value::Integer(3), Value::Text("y".into()), Value::Null],
                ],
            ),
            result_set(&["omitted"], vec![]),
            result_set(&["flag"], vec![vec![Value::Integer(1)], vec![Value::Null]]),
        ],
        Format::JsonColumnNameMetadata,
    )
    .unwrap();

    assert_eq!(
        result,
        r#"{"Table":[["id","label","missing"],["ColumnOrdinal=0,DataTypeName=int,ProviderType=8","ColumnOrdinal=1,DataTypeName=varchar,ProviderType=22","ColumnOrdinal=2,DataTypeName=varchar,ProviderType=22"],["2","x",null],["3","y",null]],"Table1":[["flag"],["ColumnOrdinal=0,DataTypeName=int,ProviderType=8"],["1"],[null]]}"#
    );
}
