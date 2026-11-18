use rusqlite::types::Value;

/// Values returned by SQLite, retaining its native dynamic representation.
pub type ResultValue = Value;

/// One row-producing SQL statement's column metadata and rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<ResultValue>>,
}
