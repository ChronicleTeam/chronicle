use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::Id;


#[derive(Deserialize)]
pub enum Cell {
    Text(String),
    Integer(i64),
    Decimal(f64),
    Money(Decimal),
    Progress(i32),
    DateTime(DateTime<Utc>),
    Interval(()),
    WebLink(String),
    Email(String),
    Checkbox(bool),
    Enumeration(Id),
    Image(()),
    File(()),
}


// key: field_id
#[derive(Deserialize)]
pub struct CreateEntry(pub HashMap<Id, Cell>);