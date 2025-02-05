use crate::{
    error::{ApiError, ApiResult},
    Id,
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Field {
    pub field_id: Id,
    pub table_id: Id,
    pub name: String,
    pub options: FieldOptions,
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
        total_steps: i32,
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
        values: HashMap<u32, String>,
        default_value: u32,
    },
    CreationDate {
        date_time_format: String,
    },
    ModificationDate {
        date_time_format: String,
    },
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


impl FieldOptions {
    pub fn validate(&self) -> ApiResult<()> {
        match self {
            FieldOptions::Integer {
                range_start,
                range_end,
                ..
            } => validate_range(*range_start, *range_end),
            FieldOptions::Decimal {
                range_start,
                range_end,
                number_precision,
                number_scale,
                ..
            } => validate_range(*range_start, *range_end),
            FieldOptions::Money {
                range_start,
                range_end,
                ..
            } => validate_range(*range_start, *range_end),
            FieldOptions::DateTime {
                range_start,
                range_end,
                // date_time_format,
                ..
            } => validate_range(*range_start, *range_end),
            FieldOptions::Interval { .. } => Ok(()),
            FieldOptions::Enumeration {
                values,
                default_value,
                ..
            } => {
                if !values.contains_key(default_value) {
                    Err(anyhow!("enumeration field default value does not map to a value").into())
                } else {
                    Ok(())
                }
            }
            // FieldOptions::CreationDate { date_time_format } => Ok(()),
            // FieldOptions::ModificationDate { date_time_format } => Ok(()),
            _ => Ok(()),
        }
    }
}

fn validate_range<T>(range_start: Option<T>, range_end: Option<T>) -> ApiResult<()>
where
    T: PartialOrd,
{
    if range_start
        .zip(range_end)
        .map_or(true, |(start, end)| start <= end)
    {
        Ok(())
    } else {
        Err(ApiError::unprocessable_entity([(
            "range",
            "range start bound is greater than end bound",
        )]))
    }
}
