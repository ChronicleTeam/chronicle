use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    error::{ApiError, ApiResult},
    Id,
};

use super::FieldOptions;

#[derive(Deserialize)]
pub enum Cell {
    Text(Option<String>),
    Integer(Option<i64>),
    Decimal(Option<f64>),
    Money(Option<Decimal>),
    Progress(Option<i32>),
    DateTime(Option<DateTime<Utc>>),
    Interval(Option<()>),
    WebLink(Option<String>),
    Email(Option<String>),
    Checkbox(bool),
    Enumeration(Option<Id>),
    Image(()),
    File(()),
}

// key: field_id
#[derive(Deserialize)]
pub struct CreateEntry(pub HashMap<Id, Cell>);

impl Cell {
    pub fn validate(&self, field_options: &FieldOptions) -> Option<(String, String)> {
        match (self, field_options) {
            (Cell::Text(text), FieldOptions::Text { is_required }) => {
                if *text == None && *is_required {
                    ("entries", "field is required")
                } else {
                    Ok(())
                }
            },
            (
                Cell::Integer(integer),
                FieldOptions::Integer {
                    is_required,
                    range_start,
                    range_end,
                },
            ) => {
                
            },
            (
                Cell::Decimal(_),
                FieldOptions::Decimal {
                    is_required,
                    range_start,
                    range_end,
                    scientific_notation,
                    number_precision,
                    number_scale,
                },
            ) => todo!(),
            (
                Cell::Money(decimal),
                FieldOptions::Money {
                    is_required,
                    range_start,
                    range_end,
                },
            ) => todo!(),
            (Cell::Progress(_), FieldOptions::Progress { total_steps }) => todo!(),
            (
                Cell::DateTime(date_time),
                FieldOptions::DateTime {
                    is_required,
                    range_start,
                    range_end,
                    date_time_format,
                },
            ) => todo!(),
            (Cell::Interval(_), FieldOptions::Interval { is_required }) => todo!(),
            (Cell::WebLink(_), FieldOptions::WebLink { is_required }) => todo!(),
            (Cell::Email(_), FieldOptions::Email { is_required }) => todo!(),
            (Cell::Checkbox(_), FieldOptions::Checkbox) => todo!(),
            (
                Cell::Enumeration(_),
                FieldOptions::Enumeration {
                    is_required,
                    values,
                    default_value,
                },
            ) => todo!(),
            (Cell::Image(_), FieldOptions::Image { is_required }) => todo!(),
            (Cell::File(_), FieldOptions::File { is_required }) => todo!(),
            _ => Err(ApiError::unprocessable_entity([(
                "entries",
                "invalid cell type for field",
            )])),
        }
    }
}


fn check_required<T>(value: Option<T>, is_required: bool) -> Option<(String, String)> {
    if value = None && is_required {
        Some()
    } else {
        None
    }
}