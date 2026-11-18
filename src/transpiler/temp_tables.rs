use std::collections::HashSet;

use polyglot_sql::{
    expressions::{Expression, Raw},
    traversal::transform_all,
};

use super::{normalize, TranspilerError};

pub(super) fn rewrite(statements: Vec<Expression>) -> Result<Vec<Expression>, TranspilerError> {
    let mut known = HashSet::new();
    let mut output = Vec::with_capacity(statements.len());
    for statement in statements {
        match statement {
            Expression::Select(mut select) if select.into.is_some() => {
                let into = select.into.take().unwrap();
                let Expression::Table(mut table) = into.this else {
                    return Err(TranspilerError::Rejected(
                        "unsupported SELECT INTO target".into(),
                    ));
                };
                let name = table.name.name.clone();
                if !name.starts_with('#') {
                    return Err(TranspilerError::Rejected(
                        "SELECT INTO is supported only for temporary tables".into(),
                    ));
                }
                let clean = name.trim_start_matches('#').to_string();
                known.insert(clean.to_ascii_lowercase());
                table.name.name = clean;
                let query = normalize::generate_sqlite(Expression::Select(select))?;
                output.push(Expression::Raw(Raw {
                    sql: format!("CREATE TEMP TABLE {} AS {query}", table.name.name),
                }));
            }
            expression => {
                let expression = transform_all(expression, &|node| {
                    Ok(match node {
                        Expression::Table(mut table)
                            if table.name.name.starts_with('#')
                                && known.contains(
                                    &table.name.name.trim_start_matches('#').to_ascii_lowercase(),
                                ) =>
                        {
                            table.name.name = table.name.name.trim_start_matches('#').to_string();
                            Expression::Table(table)
                        }
                        other => other,
                    })
                })
                .map_err(TranspilerError::Generate)?;
                output.push(expression);
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyglot_sql::{Dialect, DialectType};

    #[test]
    fn rewrites_select_into_and_later_references_in_order() {
        let ast = Dialect::get(DialectType::TSQL)
            .parse("SELECT * INTO #t FROM source; SELECT * FROM #t")
            .unwrap();
        let rewritten = rewrite(ast).unwrap();
        assert_eq!(rewritten.len(), 2);
        let first = normalize::generate_sqlite(rewritten[0].clone()).unwrap();
        let second = normalize::generate_sqlite(rewritten[1].clone()).unwrap();
        assert!(
            first.contains("CREATE TEMPORARY TABLE t AS SELECT"),
            "{first}"
        );
        assert!(second.contains("FROM t"), "{second}");
    }

    #[test]
    fn leaves_unknown_hash_tables_as_is() {
        let ast = Dialect::get(DialectType::TSQL)
            .parse("SELECT * FROM #external")
            .unwrap();
        assert!(rewrite(ast).is_ok());
    }
}
