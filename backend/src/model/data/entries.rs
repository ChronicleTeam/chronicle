use crate::Id;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub enum Cell {
    Text(Option<String>),
    Integer(Option<i64>),
    Decimal(Option<f64>),
    Money(Option<Decimal>),
    Progress(Option<u32>),
    DateTime(Option<DateTime<Utc>>),
    Interval(Option<()>),
    WebLink(Option<String>),
    Email(Option<String>),
    Checkbox(bool),
    Enumeration(Option<u32>),
    Image(()),
    File(()),
}

#[derive(Serialize)]
pub struct EntryId {
    pub entry_id: Id,
}

// key: field_id
#[derive(Deserialize)]
pub struct CreateEntry(pub HashMap<Id, Cell>);

