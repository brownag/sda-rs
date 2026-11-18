use polyglot_sql::{
    expressions::{Expression, Function},
    traversal::transform_all,
    Dialect, DialectType,
};

use crate::transpiler::TranspilerError;

pub(super) fn generate_sqlite(expression: Expression) -> Result<String, TranspilerError> {
    let tsql = Dialect::get(DialectType::TSQL);
    let source = tsql
        .generate(&expression)
        .map_err(TranspilerError::Generate)?;
    let generated = tsql
        .transpile(&source, DialectType::SQLite)
        .map_err(TranspilerError::Generate)?;
    let sqlite = Dialect::get(DialectType::SQLite);
    let mut expressions = sqlite
        .parse(&generated.join("; "))
        .map_err(TranspilerError::Parse)?;
    if expressions.len() != 1 {
        return Err(TranspilerError::Rejected(
            "expected one generated SQL statement".into(),
        ));
    }
    let expression = transform_all(expressions.remove(0), &|node| {
        Ok(match node {
            Expression::Function(function) if function.name.eq_ignore_ascii_case("ISNULL") => {
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
}
