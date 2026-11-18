use std::fmt::Write as _;

use crate::result::ResultSet;
use rusqlite::types::Value;

/// Output formats supported by the SDA tabular endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    JsonColumnName,
    JsonColumnNameMetadata,
    Xml,
}

/// Serialize result sets in an SDA tabular response format.
pub fn serialize(result_sets: &[ResultSet], format: Format) -> String {
    match format {
        Format::Json => json_results(result_sets, Headers::None),
        Format::JsonColumnName => json_results(result_sets, Headers::Names),
        Format::JsonColumnNameMetadata => json_results(result_sets, Headers::Metadata),
        Format::Xml => xml_results(result_sets),
    }
}

#[derive(Clone, Copy)]
enum Headers {
    None,
    Names,
    Metadata,
}

fn json_results(result_sets: &[ResultSet], headers: Headers) -> String {
    let mut output = String::from("{");
    let mut first_table = true;
    for (index, set) in result_sets.iter().enumerate() {
        if set.rows.is_empty() {
            continue;
        }
        if !first_table {
            output.push(',');
        }
        first_table = false;
        let key = if index == 0 {
            "Table".to_owned()
        } else {
            format!("Table{index}")
        };
        output.push_str(&json_string(&key));
        output.push_str(":[");
        let mut first_row = true;
        if !matches!(headers, Headers::None) {
            json_row(
                &mut output,
                set.columns.iter().map(String::as_str),
                &mut first_row,
            );
        }
        if matches!(headers, Headers::Metadata) {
            let metadata: Vec<_> = set
                .columns
                .iter()
                .enumerate()
                .map(|(i, _)| metadata(set, i))
                .collect();
            json_row(
                &mut output,
                metadata.iter().map(String::as_str),
                &mut first_row,
            );
        }
        for row in &set.rows {
            if !first_row {
                output.push(',');
            }
            first_row = false;
            output.push('[');
            for (i, value) in row.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                match value {
                    Value::Null => output.push_str("null"),
                    _ => output.push_str(&json_string(&value_text(value))),
                }
            }
            output.push(']');
        }
        output.push(']');
    }
    output.push('}');
    output
}

fn json_row<'a>(output: &mut String, values: impl Iterator<Item = &'a str>, first: &mut bool) {
    if !*first {
        output.push(',');
    }
    *first = false;
    output.push('[');
    for (i, value) in values.enumerate() {
        if i > 0 {
            output.push(',');
        }
        output.push_str(&json_string(value));
    }
    output.push(']');
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0C}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch <= '\u{1F}' => {
                let _ = write!(output, "\\u{:04x}", ch as u32);
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
    output
}

fn value_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Integer(value) => value.to_string(),
        Value::Real(value) => value.to_string(),
        Value::Text(value) => value.clone(),
        Value::Blob(value) => value.iter().map(|byte| format!("{byte:02X}")).collect(),
    }
}

// SQLite's dynamic values do not carry declared SQL types. These stable mappings
// are provisional MVP inferences, not verified SDA/GetSchemaTable parity.
fn metadata(set: &ResultSet, column: usize) -> String {
    let value = set.rows.iter().find_map(|row| row.get(column));
    let (type_name, provider_type) = match value {
        Some(Value::Integer(_)) => ("int", int_provider()),
        Some(Value::Real(_)) => ("float", float_provider()),
        Some(Value::Text(_)) => ("varchar", varchar_provider()),
        Some(Value::Blob(_)) => ("varbinary", varbinary_provider()),
        _ => ("varchar", varchar_provider()),
    };
    format!("ColumnOrdinal={column},DataTypeName={type_name},ProviderType={provider_type}")
}

// Provisional SqlDbType enum values.
const fn int_provider() -> i32 {
    8
}
const fn float_provider() -> i32 {
    6
}
const fn varchar_provider() -> i32 {
    22
}
const fn varbinary_provider() -> i32 {
    29
}

fn xml_results(result_sets: &[ResultSet]) -> String {
    let mut xml = String::from("<NewDataSet>");
    for set in result_sets.iter().filter(|set| !set.rows.is_empty()) {
        for row in &set.rows {
            xml.push_str("<Table>");
            for (index, value) in row.iter().enumerate() {
                let name = set.columns.get(index).map(String::as_str).unwrap_or("");
                if matches!(value, Value::Null) {
                    let _ = write!(xml, "<{name} />");
                } else {
                    let _ = write!(xml, "<{name}>{}</{name}>", xml_escape(&value_text(value)));
                }
            }
            xml.push_str("</Table>");
        }
    }
    xml.push_str("</NewDataSet>");
    xml
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(columns: &[&str], rows: Vec<Vec<Value>>) -> ResultSet {
        ResultSet {
            columns: columns.iter().map(|s| (*s).to_owned()).collect(),
            rows,
        }
    }

    #[test]
    fn json_stringifies_values_and_keeps_null() {
        let result = serialize(
            &[set(
                &["id", "name", "missing"],
                vec![vec![
                    Value::Integer(12),
                    Value::Text("A&B".into()),
                    Value::Null,
                ]],
            )],
            Format::Json,
        );
        assert_eq!(result, r#"{"Table":[["12","A&B",null]]}"#);
    }

    #[test]
    fn names_metadata_multiple_sets_and_empty_sets() {
        let result = serialize(
            &[
                set(
                    &["id", "label"],
                    vec![vec![Value::Integer(2), Value::Text("x".into())]],
                ),
                set(&["ignored"], vec![]),
                set(&["flag"], vec![vec![Value::Integer(1)]]),
            ],
            Format::JsonColumnNameMetadata,
        );
        assert_eq!(
            result,
            r#"{"Table":[["id","label"],["ColumnOrdinal=0,DataTypeName=int,ProviderType=8","ColumnOrdinal=1,DataTypeName=varchar,ProviderType=22"],["2","x"]],"Table2":[["flag"],["ColumnOrdinal=0,DataTypeName=int,ProviderType=8"],["1"]]}"#
        );
    }

    #[test]
    fn column_names_format_and_all_empty() {
        assert_eq!(
            serialize(
                &[set(&["x"], vec![vec![Value::Null]])],
                Format::JsonColumnName
            ),
            r#"{"Table":[["x"],[null]]}"#
        );
        assert_eq!(serialize(&[set(&["x"], vec![])], Format::Json), "{}");
    }

    #[test]
    fn xml_escapes_values_and_emits_table_rows() {
        assert_eq!(
            serialize(
                &[set(
                    &["name", "empty"],
                    vec![vec![Value::Text("<A&B>".into()), Value::Null]]
                )],
                Format::Xml
            ),
            "<NewDataSet><Table><name>&lt;A&amp;B&gt;</name><empty /></Table></NewDataSet>"
        );
    }
}
