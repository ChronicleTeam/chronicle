use super::FieldOptions;
use crate::Id;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{postgres::PgRow, types::Json, Row};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub entry_id: Id,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub cells: HashMap<Id, Option<Cell>>,
}

impl Entry {
    pub fn from_row(
        row: PgRow,
        field_data: &[(Id, String, Json<FieldOptions>)],
    ) -> sqlx::Result<Entry> {
        Ok(Entry {
            entry_id: row.get("entry_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            cells: field_data
                .iter()
                .map(|(id, name, options)| {
                    match Cell::from_row(&row, name.as_str(), &options.0) {
                        Ok(v) => Ok(Some(v)),
                        Err(sqlx::Error::ColumnNotFound(_)) => Ok(None),
                        Err(e) => Err(e),
                    }
                    .map(|v| (*id, v))
                })
                .try_collect()?,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Cell {
    Integer(i64),
    Float(f64),
    Decimal(Decimal),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    String(String),
    Interval(()),
}

impl Cell {
    fn from_row(row: &PgRow, index: &str, field_options: &FieldOptions) -> sqlx::Result<Cell> {
        Ok(match field_options {
            FieldOptions::Text { .. }
            | FieldOptions::WebLink { .. }
            | FieldOptions::Email { .. } => Cell::String(row.try_get(index)?),
            FieldOptions::Integer { .. }
            | FieldOptions::Progress { .. }
            | FieldOptions::Enumeration { .. } => Cell::Integer(row.try_get(index)?),
            FieldOptions::Decimal { .. } => Cell::Float(row.try_get(index)?),
            FieldOptions::Money { .. } => Cell::Decimal(row.try_get(index)?),
            FieldOptions::DateTime { .. } => Cell::DateTime(row.try_get(index)?),
            FieldOptions::Interval { .. } => Cell::Interval(row.try_get(index)?),
            FieldOptions::Checkbox => Cell::Boolean(row.try_get(index)?),
        })
    }
}

// #[derive(Serialize)]
// pub struct EntryId {
//     pub entry_id: Id,
// }

// key: field_id
#[derive(Deserialize)]
pub struct CreateEntry(pub HashMap<Id, Value>);

// key: field_id
#[derive(Deserialize)]
pub struct UpdateEntry(pub HashMap<Id, Value>);
