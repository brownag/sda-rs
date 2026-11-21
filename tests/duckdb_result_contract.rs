use std::time::Duration;

use rusqlite::types::Value;
use sda_rs::{
    executor::duckdb::{execute, ExecutorError},
    serializer::{serialize, Format},
};

#[tokio::test]
async fn supported_scalars_preserve_values_and_sda_serialization() {
    let sets = execute(
        vec![
            "SELECT NULL AS missing, TRUE AS flag, -2::SMALLINT AS small, 3::INTEGER AS id, 4::BIGINT AS big, 5::UTINYINT AS tiny, 6::USMALLINT AS usmall, 7::UINTEGER AS uint, 18446744073709551615::UBIGINT AS ubig, 1.25::FLOAT AS single, 2.5::DOUBLE AS double, 9999999999999999.123450::DECIMAL(24,6) AS amount, 8::VARCHAR AS label".to_owned(),
        ],
        Duration::from_secs(5),
    )
    .await
    .unwrap();

    assert_eq!(
        sets[0].columns,
        [
            "missing", "flag", "small", "id", "big", "tiny", "usmall", "uint", "ubig", "single",
            "double", "amount", "label"
        ]
    );
    assert_eq!(
        sets[0].rows,
        [vec![
            Value::Null,
            Value::Integer(1),
            Value::Integer(-2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
            Value::Integer(6),
            Value::Integer(7),
            Value::Text("18446744073709551615".into()),
            Value::Real(1.25),
            Value::Real(2.5),
            Value::Text("9999999999999999.123450".into()),
            Value::Text("8".into()),
        ]]
    );
    assert_eq!(
        serialize(&sets, Format::Json).unwrap(),
        r#"{"Table":[[null,"1","-2","3","4","5","6","7","18446744073709551615","1.25","2.5","9999999999999999.123450","8"]]}"#
    );
    assert_eq!(
        serialize(&sets, Format::JsonColumnName).unwrap(),
        r#"{"Table":[["missing","flag","small","id","big","tiny","usmall","uint","ubig","single","double","amount","label"],[null,"1","-2","3","4","5","6","7","18446744073709551615","1.25","2.5","9999999999999999.123450","8"]]}"#
    );

    let response = serialize(&sets[..1], Format::JsonColumnNameMetadata).unwrap();
    assert_eq!(
        response,
        r#"{"Table":[["missing","flag","small","id","big","tiny","usmall","uint","ubig","single","double","amount","label"],["ColumnOrdinal=0,DataTypeName=varchar,ProviderType=22","ColumnOrdinal=1,DataTypeName=int,ProviderType=8","ColumnOrdinal=2,DataTypeName=int,ProviderType=8","ColumnOrdinal=3,DataTypeName=int,ProviderType=8","ColumnOrdinal=4,DataTypeName=int,ProviderType=8","ColumnOrdinal=5,DataTypeName=int,ProviderType=8","ColumnOrdinal=6,DataTypeName=int,ProviderType=8","ColumnOrdinal=7,DataTypeName=int,ProviderType=8","ColumnOrdinal=8,DataTypeName=varchar,ProviderType=22","ColumnOrdinal=9,DataTypeName=float,ProviderType=6","ColumnOrdinal=10,DataTypeName=float,ProviderType=6","ColumnOrdinal=11,DataTypeName=varchar,ProviderType=22","ColumnOrdinal=12,DataTypeName=varchar,ProviderType=22"],[null,"1","-2","3","4","5","6","7","18446744073709551615","1.25","2.5","9999999999999999.123450","8"]]}"#
    );
}

#[tokio::test]
async fn unsupported_values_fail_instead_of_being_coerced() {
    for (sql, expected) in [
        ("SELECT [1, 2] AS value", "List"),
        ("SELECT struct_pack(x := 1) AS value", "Struct"),
    ] {
        let error = execute(vec![sql.to_owned()], Duration::from_secs(5))
            .await
            .unwrap_err();
        assert!(
            matches!(error, ExecutorError::UnsupportedValue(ref kind) if kind.contains(expected)),
            "{sql}: {error}"
        );
    }
}

#[tokio::test]
async fn gpkg_metadata_can_be_read_but_geometry_payloads_are_rejected() {
    let sets = execute(
        vec![
            "CREATE TEMP TABLE gpkg_geometry_columns AS SELECT 4326 AS srs_id".to_owned(),
            "SELECT srs_id FROM gpkg_geometry_columns".to_owned(),
        ],
        Duration::from_secs(5),
    )
    .await
    .unwrap();
    assert_eq!(sets[1].columns, ["srs_id"]);
    assert_eq!(sets[1].rows, [vec![Value::Integer(4326)]]);
}
