use super::FieldKind;
use crate::Id;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use sqlx::{postgres::PgRow, Row};
use std::collections::HashMap;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub entry_id: Id,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    // #[serde_as(as = "Vec<(_, _)>")]
    pub cells: CellEntry,
}

// key: field_id
#[serde_as]
#[derive(Deserialize)]
pub struct CreateEntry(pub HashMap<Id, Value>);

// key: field_id
#[derive(Deserialize)]
pub struct UpdateEntry(pub HashMap<Id, Value>);

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
    pub fn from_row(
        row: &PgRow,
        index: &str,
        field_kind: &FieldKind,
    ) -> sqlx::Result<Option<Cell>> {
        Ok(match field_kind {
            FieldKind::Text { .. } | FieldKind::WebLink { .. } | FieldKind::Email { .. } => {
                row.try_get::<Option<_>, _>(index)?.map(Cell::String)
            }
            FieldKind::Integer { .. }
            | FieldKind::Progress { .. }
            | FieldKind::Enumeration { .. } => {
                row.try_get::<Option<_>, _>(index)?.map(Cell::Integer)
            }
            FieldKind::Decimal { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Float),
            FieldKind::Money { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Decimal),
            FieldKind::DateTime { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::DateTime),
            FieldKind::Interval { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Interval),
            FieldKind::Checkbox => row.try_get::<Option<_>, _>(index)?.map(Cell::Boolean),
        })
    }
}

pub type CellEntry = HashMap<Id, Option<Cell>>;
