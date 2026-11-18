mod normalize;
mod security;
mod temp_tables;
mod vars;

use std::{error::Error as StdError, fmt};

use polyglot_sql::{Dialect, DialectType};

/// Errors produced while parsing, validating, or generating SQL.
#[derive(Debug)]
pub enum TranspilerError {
    Parse(polyglot_sql::Error),
    Generate(polyglot_sql::Error),
    Rejected(String),
}

impl fmt::Display for TranspilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "T-SQL parse failed: {error}"),
            Self::Generate(error) => write!(f, "SQLite generation failed: {error}"),
            Self::Rejected(reason) => write!(f, "SQL rejected: {reason}"),
        }
    }
}

impl StdError for TranspilerError {}

/// Parse T-SQL, reject unsafe or unsupported constructs, and generate SQLite SQL.
pub fn transpile(sql: &str) -> Result<Vec<String>, TranspilerError> {
    let ast = Dialect::get(DialectType::TSQL)
        .parse(sql)
        .map_err(TranspilerError::Parse)?;
    for statement in &ast {
        security::validate(statement)?;
    }
    let statements = vars::fold(ast)?;
    let statements = temp_tables::rewrite(statements)?;
    statements
        .into_iter()
        .map(normalize::generate_sqlite)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejected(sql: &str, fragment: &str) {
        let error = transpile(sql).expect_err("statement must be rejected");
        assert!(error.to_string().contains(fragment), "{error}");
    }

    #[test]
    fn parses_generates_in_order_and_normalizes_supported_syntax() {
        let output =
            transpile("SELECT TOP (2) [value], ISNULL(x, 0), IIF(x > 0, 1, 0) FROM t; SELECT 2")
                .unwrap();
        assert_eq!(output.len(), 2);
        assert!(output[0].contains("LIMIT 2"), "{:?}", output[0]);
        assert!(output[0].contains("COALESCE"), "{:?}", output[0]);
        assert!(output[0].contains("IIF"), "{:?}", output[0]);
    }

    #[test]
    fn rejects_administrative_and_destructive_statements() {
        for (sql, reason) in [
            ("DROP TABLE t", "DROP"),
            ("ALTER TABLE t ADD x INT", "ALTER"),
            ("TRUNCATE TABLE t", "TRUNCATE"),
            ("GRANT SELECT ON t TO x", "GRANT"),
            ("EXEC dbo.proc", "EXEC"),
            ("WAITFOR DELAY '00:00:01'", "WAITFOR"),
        ] {
            rejected(sql, reason);
        }
    }

    #[test]
    fn permits_only_temporary_create_table_from_ddl() {
        assert!(transpile("CREATE TABLE #sample (id INT)").is_ok());
        rejected("CREATE TABLE sample (id INT)", "CREATE TABLE");
    }

    #[test]
    fn lowers_string_agg_for_sqlite() {
        let output = transpile("SELECT STRING_AGG(name, ',') FROM people").unwrap();
        assert!(output[0].contains("GROUP_CONCAT"), "{:?}", output[0]);
    }

    #[test]
    fn folds_variables_and_select_into_batch() {
        let output = transpile(
            "DECLARE @minimum INT; SET @minimum = 2; SELECT * INTO #selected FROM source WHERE id > @minimum; SELECT * FROM #selected",
        )
        .unwrap();
        assert_eq!(output.len(), 2);
        assert!(
            output[0].contains("CREATE TEMPORARY TABLE selected AS SELECT"),
            "{output:?}"
        );
        assert!(output[0].contains("id > 2"), "{output:?}");
        assert!(output[1].contains("FROM selected"), "{output:?}");
    }
}
