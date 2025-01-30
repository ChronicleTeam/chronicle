use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::*,
    types::chrono::{DateTime, Utc},
    PgPool,
};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TableMetadata {
    pub table_id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: String,

    // Trigger defines these two
    pub real_table_name: Option<String>,
    pub field_table_name: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}