use crate::result::ResultSet;
use rusqlite::types::Value;

const MAX_RESPONSE_SIZE: usize = 64 * 1024 * 1024;

/// Output formats supported by the SDA tabular endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    JsonColumnName,
    JsonColumnNameMetadata,
    Xml,
}

/// Serialize result sets in an SDA tabular response format.
pub fn serialize(result_sets: &[ResultSet], format: Format) -> Result<String, SerializeError> {
    serialize_with_limit(result_sets, format, MAX_RESPONSE_SIZE)
}

fn serialize_with_limit(
    result_sets: &[ResultSet],
    format: Format,
    limit: usize,
) -> Result<String, SerializeError> {
    match format {
        Format::Json => json_results(result_sets, Headers::None, limit),
        Format::JsonColumnName => json_results(result_sets, Headers::Names, limit),
        Format::JsonColumnNameMetadata => json_results(result_sets, Headers::Metadata, limit),
        Format::Xml => xml_results(result_sets, limit),
    }
}

/// Error raised when a result cannot be represented in the requested format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializeError {
    message: String,
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SerializeError {}

#[derive(Clone, Copy)]
enum Headers {
    None,
    Names,
    Metadata,
}

struct BoundedOutput {
    text: String,
    limit: usize,
}

impl BoundedOutput {
    fn push_str(&mut self, value: &str) -> Result<(), SerializeError> {
        if value.len() > self.limit.saturating_sub(self.text.len()) {
            return Err(SerializeError {
                message: format!("serialized response exceeds {} bytes", self.limit),
            });
        }
        self.text.push_str(value);
        Ok(())
    }

    fn push(&mut self, value: char) -> Result<(), SerializeError> {
        if value.len_utf8() > self.limit.saturating_sub(self.text.len()) {
            return Err(SerializeError {
                message: format!("serialized response exceeds {} bytes", self.limit),
            });
        }
        self.text.push(value);
        Ok(())
    }
}

fn json_results(
    result_sets: &[ResultSet],
    headers: Headers,
    limit: usize,
) -> Result<String, SerializeError> {
    let mut output = BoundedOutput {
        text: String::new(),
        limit,
    };
    output.push('{')?;
    let mut first_table = true;
    let mut emitted = 0usize;
    for set in result_sets {
        if set.rows.is_empty() {
            continue;
        }
        if !first_table {
            output.push(',')?;
        }
        first_table = false;
        let key = if emitted == 0 {
            "Table".to_owned()
        } else {
            format!("Table{emitted}")
        };
        emitted += 1;
        json_string(&mut output, &key)?;
        output.push_str(":[")?;
        let mut first_row = true;
        if !matches!(headers, Headers::None) {
            json_row(
                &mut output,
                set.columns.iter().map(String::as_str),
                &mut first_row,
            )?;
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
            )?;
        }
        for row in &set.rows {
            if !first_row {
                output.push(',')?;
            }
            first_row = false;
            output.push('[')?;
            for (i, value) in row.iter().enumerate() {
                if i > 0 {
                    output.push(',')?;
                }
                match value {
                    Value::Null => output.push_str("null")?,
                    Value::Text(value) => json_string(&mut output, value)?,
                    Value::Integer(value) => json_string(&mut output, &value.to_string())?,
                    Value::Real(value) => json_string(&mut output, &value.to_string())?,
                    Value::Blob(value) => {
                        output.push('"')?;
                        for byte in value {
                            output.push_str(&format!("{byte:02X}"))?;
                        }
                        output.push('"')?;
                    }
                }
            }
            output.push(']')?;
        }
        output.push(']')?;
    }
    output.push('}')?;
    Ok(output.text)
}

fn json_row<'a>(
    output: &mut BoundedOutput,
    values: impl Iterator<Item = &'a str>,
    first: &mut bool,
) -> Result<(), SerializeError> {
    if !*first {
        output.push(',')?;
    }
    *first = false;
    output.push('[')?;
    for (i, value) in values.enumerate() {
        if i > 0 {
            output.push(',')?;
        }
        json_string(output, value)?;
    }
    output.push(']')?;
    Ok(())
}

fn json_string(output: &mut BoundedOutput, value: &str) -> Result<(), SerializeError> {
    output.push('"')?;
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\"")?,
            '\\' => output.push_str("\\\\")?,
            '\u{08}' => output.push_str("\\b")?,
            '\u{0C}' => output.push_str("\\f")?,
            '\n' => output.push_str("\\n")?,
            '\r' => output.push_str("\\r")?,
            '\t' => output.push_str("\\t")?,
            ch if ch <= '\u{1F}' => output.push_str(&format!("\\u{:04x}", ch as u32))?,
            ch => output.push(ch)?,
        }
    }
    output.push('"')
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

// ASCII-only XML name check: first char letter or `_`; rest letters, digits,
// `_`, `-`, `.`. Names are rejected, never sanitized.
fn is_xml_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn xml_results(result_sets: &[ResultSet], limit: usize) -> Result<String, SerializeError> {
    let mut xml = BoundedOutput {
        text: String::new(),
        limit,
    };
    xml.push_str("<NewDataSet>")?;
    for set in result_sets.iter().filter(|set| !set.rows.is_empty()) {
        if let Some(bad) = set.columns.iter().find(|name| !is_xml_name(name)) {
            return Err(SerializeError {
                message: format!("invalid XML element name for column \"{bad}\""),
            });
        }
        for row in &set.rows {
            xml.push_str("<Table>")?;
            for (index, value) in row.iter().enumerate() {
                let name = set.columns.get(index).map(String::as_str).unwrap_or("");
                if !is_xml_name(name) {
                    return Err(SerializeError {
                        message: format!("invalid XML element name for column \"{name}\""),
                    });
                }
                if matches!(value, Value::Null) {
                    xml.push('<')?;
                    xml.push_str(name)?;
                    xml.push_str(" />")?;
                } else {
                    xml.push('<')?;
                    xml.push_str(name)?;
                    xml.push('>')?;
                    xml_value(&mut xml, value)?;
                    xml.push_str("</")?;
                    xml.push_str(name)?;
                    xml.push('>')?;
                }
            }
            xml.push_str("</Table>")?;
        }
    }
    xml.push_str("</NewDataSet>")?;
    Ok(xml.text)
}

fn xml_value(output: &mut BoundedOutput, value: &Value) -> Result<(), SerializeError> {
    match value {
        Value::Text(text) => xml_escape(output, text),
        Value::Blob(bytes) => {
            for byte in bytes {
                output.push_str(&format!("{byte:02X}"))?;
            }
            Ok(())
        }
        _ => xml_escape(output, &value_text(value)),
    }
}

fn xml_escape(output: &mut BoundedOutput, text: &str) -> Result<(), SerializeError> {
    for ch in text.chars() {
        output.push_str(match ch {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&apos;",
            _ => {
                output.push(ch)?;
                continue;
            }
        })?;
    }
    Ok(())
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
    fn bounded_serialization_accepts_exact_limit_and_rejects_overflow() {
        let sets = [set(&["x"], vec![vec![Value::Text("a".into())]])];
        for (format, exact) in [
            (Format::Json, r#"{"Table":[["a"]]}"#),
            (
                Format::Xml,
                "<NewDataSet><Table><x>a</x></Table></NewDataSet>",
            ),
        ] {
            assert_eq!(
                serialize_with_limit(&sets, format, exact.len()).unwrap(),
                exact
            );
            let error = serialize_with_limit(&sets, format, exact.len() - 1).unwrap_err();
            assert!(error.to_string().contains("exceeds"));
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
        )
        .unwrap();
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
        )
        .unwrap();
        assert_eq!(
            result,
            r#"{"Table":[["id","label"],["ColumnOrdinal=0,DataTypeName=int,ProviderType=8","ColumnOrdinal=1,DataTypeName=varchar,ProviderType=22"],["2","x"]],"Table1":[["flag"],["ColumnOrdinal=0,DataTypeName=int,ProviderType=8"],["1"]]}"#
        );
    }

    #[test]
    fn column_names_format_and_all_empty() {
        assert_eq!(
            serialize(
                &[set(&["x"], vec![vec![Value::Null]])],
                Format::JsonColumnName
            )
            .unwrap(),
            r#"{"Table":[["x"],[null]]}"#
        );
        assert_eq!(
            serialize(&[set(&["x"], vec![])], Format::Json).unwrap(),
            "{}"
        );
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
            )
            .unwrap(),
            "<NewDataSet><Table><name>&lt;A&amp;B&gt;</name><empty /></Table></NewDataSet>"
        );
    }

    #[test]
    fn first_empty_set_then_rows_uses_table_key() {
        let result = serialize(
            &[
                set(&["a"], vec![]),
                set(&["b"], vec![vec![Value::Integer(1)]]),
            ],
            Format::Json,
        )
        .unwrap();
        assert_eq!(result, r#"{"Table":[["1"]]}"#);
    }

    fn xml_for(column: &str) -> Result<String, SerializeError> {
        serialize(
            &[set(&[column], vec![vec![Value::Integer(1)]])],
            Format::Xml,
        )
    }

    #[test]
    fn xml_rejects_invalid_names_and_accepts_valid() {
        for bad in ["a b", "1a", ""] {
            let err = xml_for(bad).unwrap_err();
            assert!(err.to_string().contains(&format!("\"{bad}\"")), "{err}");
        }
        assert_eq!(
            xml_for("my-col.1_x").unwrap(),
            "<NewDataSet><Table><my-col.1_x>1</my-col.1_x></Table></NewDataSet>"
        );
    }
}
