use crate::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct Table {
    pub table_id: Id,
    pub user_id: Id,
    pub name: String,
    pub description: String,
    pub data_table_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateTable {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct UpdateTable {
    pub name: String,
    pub description: String,
}
