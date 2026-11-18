use polyglot_sql::{
    expressions::{Expression, Function},
    traversal::transform_all,
};

use crate::transpiler::TranspilerError;

pub(super) fn normalize_sqlite(sql: &str) -> Result<String, TranspilerError> {
    let sqlite = polyglot_sql::Dialect::get(polyglot_sql::DialectType::SQLite);
    let expressions = sqlite.parse(sql).map_err(TranspilerError::Parse)?;
    expressions
        .into_iter()
        .map(|expression| {
            let expression = transform_all(expression, &|node| {
                Ok(match node {
                    Expression::Function(function)
                        if function.name.eq_ignore_ascii_case("ISNULL") =>
                    {
                        Expression::Function(Box::new(Function {
                            name: "COALESCE".into(),
                            ..*function
                        }))
                    }
                    other => other,
                })
            })
            .map_err(TranspilerError::Generate)?;
            sqlite
                .generate(&expression)
                .map_err(TranspilerError::Generate)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|statements| statements.join("; "))
}
