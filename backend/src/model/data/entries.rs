use crate::Id;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Entry {
    pub entry_id: Id,
    pub cells: Vec<Cell>,
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
    Image(()),
    File(()),
}

#[derive(Serialize)]
pub struct EntryId {
    pub entry_id: Id,
}

// key: field_id
#[derive(Deserialize)]
pub struct CreateEntry(pub HashMap<Id, Value>);