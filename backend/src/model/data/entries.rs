use crate::{Id, model::Cell};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Table entry entity.
#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema)]
pub struct Entry {
    pub entry_id: Id,
    pub parent_id: Option<Id>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,

    /// Keys map to field IDs.
    pub cells: HashMap<Id, Cell>,
}

/// Create entry request. Keys map to field IDs.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateEntries {
    pub parent_id: Option<Id>,
    pub entries: Vec<HashMap<Id, Value>>,
}

/// Update entry request. Keys map to field IDs.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateEntry {
    pub parent_id: Option<Id>,
    pub cells: HashMap<Id, Value>,
}
