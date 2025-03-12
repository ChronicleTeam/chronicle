use crate::{model::Cell, Id};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use std::collections::HashMap;

/// Table entry response.
#[serde_as]
#[derive(Serialize)]
pub struct Entry {
    pub entry_id: Id,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,

    /// Keys map to field IDs.
    pub cells: HashMap<Id, Option<Cell>>,
}

/// Create entry request. Keys map to field IDs.
#[serde_as]
#[derive(Deserialize)]
pub struct CreateEntry(pub HashMap<Id, Value>);

/// Update entry request. Keys map to field IDs.
#[derive(Deserialize)]
pub struct UpdateEntry(pub HashMap<Id, Value>);
