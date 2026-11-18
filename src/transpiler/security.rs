use polyglot_sql::{expressions::Expression, traversal::ExpressionWalk};

use crate::transpiler::TranspilerError;

pub(super) fn validate(statement: &Expression) -> Result<(), TranspilerError> {
    for node in statement.dfs() {
        let forbidden = match node {
            Expression::DropTable(_)
            | Expression::DropView(_)
            | Expression::DropIndex(_)
            | Expression::DropSchema(_)
            | Expression::DropDatabase(_)
            | Expression::DropFunction(_)
            | Expression::DropProcedure(_)
            | Expression::DropSequence(_)
            | Expression::DropTrigger(_)
            | Expression::DropType(_)
            | Expression::DropNamespace(_) => Some("DROP"),
            Expression::AlterTable(_)
            | Expression::AlterView(_)
            | Expression::AlterIndex(_)
            | Expression::AlterSequence(_) => Some("ALTER"),
            Expression::Truncate(_) | Expression::TruncateTable(_) => Some("TRUNCATE"),
            Expression::Grant(_) => Some("GRANT"),
            Expression::Execute(_) | Expression::Call(_) => Some("EXEC"),
            Expression::Command(command)
                if command.this.to_ascii_uppercase().starts_with("WAITFOR") =>
            {
                Some("WAITFOR")
            }
            Expression::CreateTable(table)
                if !table.temporary && !table.name.name.name.starts_with('#') =>
            {
                Some("CREATE TABLE")
            }
            Expression::CreateTable(_) => None,
            Expression::Raw(_) => Some("unsupported raw SQL"),
            _ => None,
        };
        if let Some(reason) = forbidden {
            return Err(TranspilerError::Rejected(reason.into()));
        }
    }
    Ok(())
}
