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
                    Cell::from_row(&row, name.as_str(), &options.0)
                        .or_else(|e| {
                            if matches!(e, sqlx::Error::ColumnNotFound(_)) {
                                Ok(None)
                            } else {
                                Err(e)
                            }
                        })
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
    fn from_row(
        row: &PgRow,
        index: &str,
        field_options: &FieldOptions,
    ) -> sqlx::Result<Option<Cell>> {
        Ok(match field_options {
            FieldOptions::Text { .. }
            | FieldOptions::WebLink { .. }
            | FieldOptions::Email { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::String),
            FieldOptions::Integer { .. }
            | FieldOptions::Progress { .. }
            | FieldOptions::Enumeration { .. } => {
                row.try_get::<Option<_>, _>(index)?.map(Cell::Integer)
            }
            FieldOptions::Decimal { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Float),
            FieldOptions::Money { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Decimal),
            FieldOptions::DateTime { .. } => {
                row.try_get::<Option<_>, _>(index)?.map(Cell::DateTime)
            }
            FieldOptions::Interval { .. } => {
                row.try_get::<Option<_>, _>(index)?.map(Cell::Interval)
            }
            FieldOptions::Checkbox => row.try_get::<Option<_>, _>(index)?.map(Cell::Boolean),
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
