use super::FieldKind;
use crate::{model::{viz::Aggregate, CellMap}, Id};
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
    pub cells: CellMap,
}

// key: field_id
#[serde_as]
#[derive(Deserialize)]
pub struct CreateEntry(pub HashMap<Id, Value>);

// key: field_id
#[derive(Deserialize)]
pub struct UpdateEntry(pub HashMap<Id, Value>);


