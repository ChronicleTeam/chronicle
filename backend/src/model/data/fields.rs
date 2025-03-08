use crate::Id;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use sqlx::{types::Json, FromRow};
use std::collections::HashMap;

/// Table field response.
#[derive(Serialize, FromRow)]
pub struct Field {
    pub field_id: Id,
    pub table_id: Id,
    pub name: String,
    pub field_kind: Json<FieldKind>,

    /// Private database identifier.
    #[serde(skip)]
    pub data_field_name: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// The field kind and associated options.
#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum FieldKind {
    Text {
        is_required: bool,
    },
    Integer {
        is_required: bool,
        range_start: Option<i64>,
        range_end: Option<i64>,
    },
    Float {
        is_required: bool,
        range_start: Option<f64>,
        range_end: Option<f64>,
        scientific_notation: bool,
        number_precision: Option<i64>,
        number_scale: Option<i64>,
    },
    Money {
        is_required: bool,
        range_start: Option<Decimal>,
        range_end: Option<Decimal>,
    },
    Progress {
        total_steps: i64,
    },
    DateTime {
        is_required: bool,
        range_start: Option<DateTime<Utc>>,
        range_end: Option<DateTime<Utc>>,
        date_time_format: String,
    },
    Interval {
        is_required: bool,
        // range_start: Option<PgInterval>,
        // range_end: Option<PgInterval>,
    },
    WebLink {
        is_required: bool,
    },

    Email {
        is_required: bool,
    },
    Checkbox,
    Enumeration {
        is_required: bool,
        #[serde_as(as = "HashMap<DisplayFromStr, _>")] // This is necessary because of a bug with serde
        values: HashMap<i64, String>,
        default_value: i64,
    },
}

impl FieldKind {
    /// Map the field kind to the PostgreSQL data type.
    pub fn get_sql_type(&self) -> &'static str {
        match self {
            FieldKind::Text { .. } => "TEXT",
            FieldKind::Integer { .. } => "BIGINT",
            FieldKind::Float { .. } => "DOUBLE",
            FieldKind::Money { .. } => "numeric_money",
            FieldKind::Progress { .. } => "BIGINT NOT NULL DEFAULT 0",
            FieldKind::DateTime { .. } => "TIMESTAMPTZ",
            FieldKind::Interval { .. } => "INTERVAL",
            FieldKind::WebLink { .. } => "COLLATE case_insensitive TEXT",
            FieldKind::Email { .. } => "COLLATE case_insensitive TEXT",
            FieldKind::Checkbox => "BOOLEAN NOT NULL DEFAULT FALSE",
            FieldKind::Enumeration { .. } => "BIGINT",
        }
    }
}

/// Create field request.
#[derive(Deserialize)]
pub struct CreateField {
    pub name: String,
    pub field_kind: FieldKind,
}


/// Update field request.
#[derive(Deserialize)]
pub struct UpdateField {
    pub name: String,
    pub field_kind: FieldKind,
}
