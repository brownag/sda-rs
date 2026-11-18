use std::collections::HashMap;

use polyglot_sql::{
    expressions::{Expression, Null},
    traversal::{transform_all, ExpressionWalk},
};

use super::TranspilerError;

pub(super) fn fold(statements: Vec<Expression>) -> Result<Vec<Expression>, TranspilerError> {
    let mut values = HashMap::<String, Expression>::new();
    let mut output = Vec::new();

    for statement in statements {
        match statement {
            Expression::Declare(declare) => {
                for item in declare.expressions {
                    let Expression::DeclareItem(item) = item else {
                        return Err(TranspilerError::Rejected("unsupported DECLARE item".into()));
                    };
                    let name = variable_name(&item.this)?;
                    values.insert(
                        name,
                        item.default
                            .map(|v| *v)
                            .unwrap_or_else(|| Expression::Null(Null {})),
                    );
                }
            }
            Expression::SetStatement(set) => {
                if set.items.len() != 1 {
                    return Err(TranspilerError::Rejected(
                        "unsupported SET assignment form".into(),
                    ));
                }
                let item = &set.items[0];
                if item.no_equals {
                    return Err(TranspilerError::Rejected(
                        "unsupported SET assignment form".into(),
                    ));
                }
                let name = variable_name(&item.name)?;
                if !values.contains_key(&name) {
                    return Err(TranspilerError::Rejected(format!(
                        "assignment to undeclared variable {}",
                        name
                    )));
                }
                let value = substitute(item.value.clone(), &values)?;
                values.insert(name, value);
            }
            Expression::Select(select) if select.expressions.len() == 1 => {
                if let Expression::Eq(assignment) = &select.expressions[0] {
                    if let Expression::Column(column) = &assignment.left {
                        if column.name.name.starts_with('@') {
                            let name = canonical(&column.name.name);
                            if !values.contains_key(&name) {
                                return Err(TranspilerError::Rejected(format!(
                                    "assignment to undeclared variable {}",
                                    column.name.name
                                )));
                            }
                            let value = substitute(assignment.right.clone(), &values)?;
                            values.insert(name, value);
                            continue;
                        }
                    }
                }
                let expression = substitute(Expression::Select(select), &values)?;
                output.push(expression);
            }
            other => output.push(substitute(other, &values)?),
        }
    }
    Ok(output)
}

fn variable_name(expression: &Expression) -> Result<String, TranspilerError> {
    let name = match expression {
        Expression::Identifier(identifier) => identifier.name.as_str(),
        _ => {
            return Err(TranspilerError::Rejected(
                "unsupported variable name".into(),
            ))
        }
    };
    if !name.starts_with('@') {
        return Err(TranspilerError::Rejected(
            "unsupported variable name".into(),
        ));
    }
    Ok(canonical(name))
}

fn canonical(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn substitute(
    expression: Expression,
    values: &HashMap<String, Expression>,
) -> Result<Expression, TranspilerError> {
    for node in expression.dfs() {
        let name = match node {
            Expression::Column(column) if column.table.is_none() => Some(column.name.name.as_str()),
            Expression::Parameter(parameter) => parameter.name.as_deref(),
            _ => None,
        };
        if let Some(name) = name.filter(|name| name.starts_with('@')) {
            if !values.contains_key(&canonical(name)) {
                return Err(TranspilerError::Rejected(format!(
                    "unbound variable {name}"
                )));
            }
        }
    }
    transform_all(expression, &|node| {
        let name = match &node {
            Expression::Column(column) if column.table.is_none() => Some(column.name.name.as_str()),
            Expression::Parameter(parameter) => parameter.name.as_deref(),
            _ => None,
        };
        if let Some(name) = name.filter(|name| name.starts_with('@')) {
            if let Some(value) = values.get(&canonical(name)) {
                return Ok(value.clone());
            }
        }
        Ok(node)
    })
    .map_err(TranspilerError::Generate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyglot_sql::{Dialect, DialectType};

    #[test]
    fn folds_declare_set_and_select_assignment_case_insensitively() {
        let ast = Dialect::get(DialectType::TSQL)
            .parse("DECLARE @x INT; SET @x = 1; SELECT @X = @x + 1; SELECT @X")
            .unwrap();
        let folded = fold(ast).unwrap();
        assert_eq!(folded.len(), 1);
    }

    #[test]
    fn rejects_set_of_undeclared_variable() {
        let ast = Dialect::get(DialectType::TSQL).parse("SET @x = 1").unwrap();
        assert!(fold(ast).is_err());
    }
}
