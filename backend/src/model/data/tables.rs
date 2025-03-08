use crate::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// User table metadata response.
#[derive(Serialize, FromRow)]
pub struct Table {
    pub table_id: Id,
    pub user_id: Id,
    pub name: String,
    pub description: String,
    
    /// Private database identifier
    #[serde(skip)]
    pub data_table_name: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Create table request.
#[derive(Deserialize)]
pub struct CreateTable {
    pub name: String,
    pub description: String,
}


/// Update table request.
#[derive(Deserialize)]
pub struct UpdateTable {
    pub name: String,
    pub description: String,
}
