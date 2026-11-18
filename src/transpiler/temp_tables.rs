use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use polyglot_sql::{
    expressions::{Expression, Raw},
    traversal::transform_all,
};

use super::{normalize, TranspilerError};

static NEXT_TEMP_TABLE: AtomicUsize = AtomicUsize::new(0);

fn generated_name() -> String {
    format!(
        "__sda_temp_{}_{}",
        std::process::id(),
        NEXT_TEMP_TABLE.fetch_add(1, Ordering::Relaxed)
    )
}

pub(super) fn rewrite(statements: Vec<Expression>) -> Result<Vec<Expression>, TranspilerError> {
    let mut known = HashMap::new();
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
                let generated = generated_name();
                known.insert(name.to_ascii_lowercase(), generated.clone());
                table.name.name = generated;
                let query = normalize::generate_sqlite(Expression::Select(select))?;
                output.push(Expression::Raw(Raw {
                    sql: format!("CREATE TEMP TABLE {} AS {query}", table.name.name),
                }));
            }
            expression => {
                let mut create_name = None;
                if let Expression::CreateTable(create) = &expression {
                    if create.name.name.name.starts_with('#') {
                        let key = create.name.name.name.to_ascii_lowercase();
                        let generated = known.entry(key).or_insert_with(generated_name).clone();
                        create_name = Some(generated);
                    }
                }
                let expression = transform_all(expression, &|node| {
                    Ok(match node {
                        Expression::Insert(mut insert)
                            if insert.table.name.name.starts_with('#')
                                && known
                                    .contains_key(&insert.table.name.name.to_ascii_lowercase()) =>
                        {
                            insert.table.name.name =
                                known[&insert.table.name.name.to_ascii_lowercase()].clone();
                            Expression::Insert(insert)
                        }
                        Expression::CreateTable(mut create)
                            if create.name.name.name.starts_with('#') =>
                        {
                            create.name.name.name = create_name.clone().unwrap();
                            create.temporary = true;
                            Expression::CreateTable(create)
                        }
                        Expression::Table(mut table)
                            if table.name.name.starts_with('#')
                                && known.contains_key(&table.name.name.to_ascii_lowercase()) =>
                        {
                            table.name.name = known[&table.name.name.to_ascii_lowercase()].clone();
                            Expression::Table(table)
                        }
                        Expression::Column(mut column)
                            if column.table.is_none()
                                && column.name.name.starts_with('#')
                                && known.contains_key(&column.name.name.to_ascii_lowercase()) =>
                        {
                            column.name.name =
                                known[&column.name.name.to_ascii_lowercase()].clone();
                            Expression::Column(column)
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
        let name = first
            .split("CREATE TEMPORARY TABLE ")
            .nth(1)
            .unwrap()
            .split(" AS SELECT")
            .next()
            .unwrap();
        assert!(name.starts_with("__sda_temp_"), "{first}");
        assert!(second.contains(&format!("FROM {name}")), "{second}");
    }

    #[test]
    fn rewrites_create_table_and_later_insert() {
        let ast = Dialect::get(DialectType::TSQL)
            .parse("CREATE TABLE #x (id INT); INSERT INTO #x VALUES (1)")
            .unwrap();
        let rewritten = rewrite(ast).unwrap();
        let create = normalize::generate_sqlite(rewritten[0].clone()).unwrap();
        let insert = normalize::generate_sqlite(rewritten[1].clone()).unwrap();
        let name = create
            .split("CREATE TABLE ")
            .nth(1)
            .or_else(|| create.split("CREATE TEMPORARY TABLE ").nth(1))
            .unwrap()
            .split(" ")
            .next()
            .unwrap();
        assert!(name.starts_with("__sda_temp_"), "{create}");
        assert!(insert.contains(&format!("INTO {name}")), "{insert}");
    }

    #[test]
    fn leaves_unknown_hash_tables_as_is() {
        let ast = Dialect::get(DialectType::TSQL)
            .parse("SELECT * FROM #external")
            .unwrap();
        assert!(rewrite(ast).is_ok());
    }
}
