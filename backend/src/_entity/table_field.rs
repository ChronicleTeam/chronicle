use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::*,
    types::chrono::{DateTime, Utc},
    PgPool,
};

#[derive(sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "field_kind")]
pub enum FieldKind {
    Text,
    Number,
    Progress,
    DateTime,
    Interval,
    WebLink,
    Email,
    Checkbox,
    Enumeration,
    CreationDate,
    ModificationDate,
    Image,
    File,
    Table,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TableField {
    pub field_id: i32,
    pub table_id: i32,
    pub field_kind: FieldKind,
    pub description: String,

    // Trigger defines these two
    pub real_table_name: Option<String>,
    pub field_table_name: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
