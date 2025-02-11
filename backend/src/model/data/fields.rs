use crate::Id;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
use std::collections::HashMap;

#[derive(Serialize, FromRow)]
pub struct Field {
    pub field_id: Id,
    pub table_id: Id,
    pub name: String,
    pub options: Json<FieldOptions>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum FieldOptions {
    Text {
        is_required: bool,
    },
    Integer {
        is_required: bool,
        range_start: Option<i64>,
        range_end: Option<i64>,
    },
    Decimal {
        is_required: bool,
        range_start: Option<f64>,
        range_end: Option<f64>,
        scientific_notation: bool,
        number_precision: Option<u32>,
        number_scale: Option<u32>,
    },
    Money {
        is_required: bool,
        range_start: Option<Decimal>,
        range_end: Option<Decimal>,
    },
    Progress {
        total_steps: u32,
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
        values: HashMap<Id, String>,
        default_value: Id,
    },
    // CreationDate {
    //     date_time_format: String,
    // },
    // ModificationDate {
    //     date_time_format: String,
    // },
    Image {
        is_required: bool,
    },
    File {
        is_required: bool,
    },
}

#[derive(Serialize)]
pub struct FieldId {
    pub field_id: Id,
}

#[derive(Deserialize)]
pub struct CreateField {
    pub name: String,
    pub options: FieldOptions,
}