use crate::{model::{users::AccessRole, Cell}, Id};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

use super::{CreateField, Entry, Field};

/// User table entity.
#[derive(Debug, Serialize, FromRow, JsonSchema)]
pub struct Table {
    pub table_id: Id,
    pub parent_id: Option<Id>,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Create table request.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateTable {
    pub parent_id: Option<Id>,
    pub name: String,
    pub description: String,
}

/// Update table request.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateTable {
    pub name: String,
    pub description: String,
}

#[derive(Debug, FromRow, Serialize, JsonSchema)]
pub struct GetTable {
    #[sqlx(flatten)]
    pub table: Table,
    pub access_role: AccessRole,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SelectTable {
    pub table_id: Id,
}

/// Response for fetching entire table data.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TableData {
    pub table: Table,
    pub fields: Vec<Field>,
    pub entries: Vec<Entry>,
    pub children: Vec<TableData>,
}

/// DTO for creating tables from imports.
#[derive(Debug)]
pub struct CreateTableData {
    pub table: CreateTable,
    pub fields: Vec<CreateField>,
    pub entries: Vec<Vec<Cell>>,
}

/// Database identifier of the actual SQL table that a user table points to.
#[derive(Debug)]
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
        write!(f, r#""{}"."t{}""#, self.schema, self.table_id)
    }
}
