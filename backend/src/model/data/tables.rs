use crate::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

use super::{Entry, Field};

/// User table metadata response.
#[derive(Serialize, FromRow)]
pub struct Table {
    pub table_id: Id,
    pub user_id: Id,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Create table request.
#[derive(Deserialize)]
pub struct CreateTable {
    pub name: String,
    pub description: String,
}

/// Update table request.
#[derive(Deserialize)]
pub struct UpdateTable {
    pub name: String,
    pub description: String,
}

/// Response for fetching entire table data.
#[derive(Serialize)]
pub struct TableData {
    pub table: Table,
    pub fields: Vec<Field>,
    pub entries: Vec<Entry>,
}


pub struct TableIdentifier {
    table_id: Id,
    schema: String,
}
impl TableIdentifier {
    pub fn new(table_id: Id, schema: &str) -> Self {
        Self {
            table_id,
            schema: schema.to_string(),
        }
    }
}
impl fmt::Display for TableIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#""{}".t{}"#, self.schema, self.table_id)
    }
}
