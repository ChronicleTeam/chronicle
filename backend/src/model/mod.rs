//! This module contains all the types for JSON
//! responses and requests and custom return types for
//! `sqlx` queries.
//!
//! Theses types model the database into code.
//!
//! The important trait implementation used are:
//! - Serialize: Convert into JSON for responses.
//! - Deserialize: Convert from JSON for requests.
//! - FromRow: Convert from an SQL query.

pub mod data;
pub mod users;
pub mod viz;

use std::str::FromStr;

use chrono::{DateTime, Utc};
use data::FieldKind;
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde::{Serialize, Serializer};
use sqlx::{
    postgres::{PgArgumentBuffer, PgRow},
    Encode, Postgres, Row,
};
use viz::Aggregate;

/// This represents all the data types in user entries and charts.
#[derive(Debug)]
pub enum Cell {
    Integer(i64),
    Float(f64),
    Decimal(Decimal),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    String(String),
    Null,
}

impl Serialize for Cell {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Cell::Integer(value) => serializer.serialize_i64(*value),
            Cell::Float(value) => serializer.serialize_f64(*value),
            Cell::Decimal(value) => serializer.serialize_str(&value.to_string()),
            Cell::Boolean(value) => serializer.serialize_bool(*value),
            Cell::DateTime(value) => serializer.serialize_str(&value.to_rfc3339()),
            Cell::String(value) => serializer.serialize_str(value),
            Cell::Null => serializer.serialize_none(),
        }
    }
}

impl<'q> Encode<'q, Postgres> for Cell {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            Cell::Integer(value) => <i64 as Encode<Postgres>>::encode_by_ref(value, buf),
            Cell::Float(value) => <f64 as Encode<Postgres>>::encode_by_ref(value, buf),
            Cell::Decimal(value) => <Decimal as Encode<Postgres>>::encode_by_ref(value, buf),
            Cell::Boolean(value) => <bool as Encode<Postgres>>::encode_by_ref(value, buf),
            Cell::DateTime(value) => <DateTime<Utc> as Encode<Postgres>>::encode_by_ref(value, buf),
            Cell::String(value) => <String as Encode<Postgres>>::encode_by_ref(value, buf),
            Cell::Null => <Option<bool> as Encode<Postgres>>::encode_by_ref(&None, buf),
        }
    }

    // fn produces(&self) -> Option<PgTypeInfo> {
    //     Some(match self {
    //         Cell::Integer(_) => <i64 as Type<Postgres>>::type_info(),
    //         Cell::Float(_) => <f64 as Type<Postgres>>::type_info(),
    //         Cell::Decimal(_) => <Decimal as Type<Postgres>>::type_info(),
    //         Cell::Boolean(_) => <bool as Type<Postgres>>::type_info(),
    //         Cell::DateTime(_) => <DateTime<Utc> as Type<Postgres>>::type_info(),
    //         Cell::String(_) => <String as Type<Postgres>>::type_info(),
    //         Cell::Null => PgTypeInfo::with_name("TEXT"), // Default type for NULL
    //     })
    // }

    // fn encode_by_ref(
    //     &self,
    //     buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    // ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
    //     self.encode(buf)
    // }
}

// impl Type<Postgres> for Cell {
//     fn type_info() -> PgTypeInfo {
//         PgTypeInfo::with_name("TEXT") // Default to TEXT (or JSONB if preferred)
//     }
// }
impl Cell {
    /// Get the `Cell` from this PostgreSQL row into the proper type based on `FieldKind`.
    pub fn from_field_row(row: &PgRow, index: &str, field_kind: &FieldKind) -> sqlx::Result<Self> {
        if let Ok(None) = row.try_get::<Option<bool>, _>(index) {
            return Ok(Cell::Null);
        }
        Ok(match field_kind {
            FieldKind::Text { .. } | FieldKind::WebLink { .. } => Cell::String(row.try_get(index)?),
            FieldKind::Integer { .. }
            | FieldKind::Progress { .. }
            | FieldKind::Enumeration { .. } => Cell::Integer(row.try_get(index)?),
            FieldKind::Float { .. } => Cell::Float(row.try_get(index)?),
            FieldKind::Money { .. } => Cell::Decimal(row.try_get(index)?),
            FieldKind::DateTime { .. } => Cell::DateTime(row.try_get(index)?),
            FieldKind::Checkbox => Cell::Boolean(row.try_get(index)?),
        })
    }

    /// Get the `Cell` from this PostgreSQL row into the proper type based on `Aggregate` and `FieldKind`.
    pub fn from_aggregate_row(
        row: &PgRow,
        index: &str,
        aggregate: &Aggregate,
        field_kind: &FieldKind,
    ) -> sqlx::Result<Self> {
        if let Ok(None) = row.try_get::<Option<bool>, _>(index) {
            return Ok(Cell::Null);
        }
        Ok(match aggregate {
            Aggregate::Sum | Aggregate::Average => match field_kind {
                FieldKind::Float { .. } => Cell::Float(row.try_get(index)?),
                _ => Cell::Decimal(row.try_get(index)?),
            },
            Aggregate::Min | Aggregate::Max => Self::from_field_row(row, index, field_kind)?,
            Aggregate::Count => Cell::Integer(row.try_get(index)?),
        })
    }

    pub fn convert_field_kind(self, field_kind: &FieldKind) -> Option<Self> {
        match field_kind {
            FieldKind::Text { .. } | FieldKind::WebLink { .. } => Some(Cell::String(match self {
                Cell::Integer(v) => v.to_string(),
                Cell::Float(v) => v.to_string(),
                Cell::Decimal(v) => v.to_string(),
                Cell::Boolean(v) => v.to_string(),
                Cell::DateTime(v) => v.to_string(),
                Cell::String(_) | Cell::Null => return Some(self),
            })),
            FieldKind::Integer {
                range_start,
                range_end,
                ..
            } => {
                let v: i64 = match self {
                    Cell::Float(v) => num_traits::cast(v)?,
                    Cell::Decimal(v) => v.to_i64()?,
                    Cell::Boolean(v) => v.into(),
                    Cell::DateTime(v) => v.timestamp(),
                    Cell::String(v) => v.parse().ok()?,
                    Cell::Integer(_) | Cell::Null => return Some(self),
                };
                if in_range(&v, range_start.as_ref(), range_end.as_ref()) {
                    Some(Cell::Integer(v))
                } else {
                    None
                }
            }
            FieldKind::Float {
                range_start,
                range_end,
                ..
            } => {
                let v: f64 = match self {
                    Cell::Integer(v) => num_traits::cast(v)?,
                    Cell::Decimal(v) => v.to_f64()?,
                    Cell::Boolean(v) => v.into(),
                    Cell::DateTime(_) => return None,
                    Cell::String(v) => v.parse().ok()?,
                    Cell::Float(_) | Cell::Null => return Some(self),
                };
                if in_range(&v, range_start.as_ref(), range_end.as_ref()) {
                    Some(Cell::Float(v))
                } else {
                    None
                }
            }
            FieldKind::Money {
                range_start,
                range_end,
                ..
            } => {
                let v: Decimal = match self {
                    Cell::Integer(v) => Decimal::from_i64(v)?,
                    Cell::Float(v) => Decimal::from_f64(v)?,
                    Cell::String(v) => v.parse().ok()?,
                    Cell::Boolean(_) | Cell::DateTime(_) => return None,
                    Cell::Decimal(_) | Cell::Null => return Some(self),
                };
                if in_range(&v, range_start.as_ref(), range_end.as_ref()) {
                    Some(Cell::Decimal(v))
                } else {
                    None
                }
            }
            FieldKind::Progress { total_steps } => {
                let v: i64 = match self {
                    Cell::Integer(v) => v,
                    Cell::Float(v) => num_traits::cast(v)?,
                    Cell::Decimal(v) => v.to_i64()?,
                    Cell::Boolean(v) => v.into(),
                    Cell::String(v) => v.parse().ok()?,
                    Cell::DateTime(_) => return None,
                    Cell::Null => return Some(self),
                };
                if v >= 0 && v <= *total_steps {
                    Some(Cell::Integer(v))
                } else {
                    None
                }
            }
            FieldKind::DateTime {
                range_start,
                range_end,
                ..
            } => {
                let v: DateTime<Utc> = match self {
                    Cell::Integer(v) => DateTime::from_timestamp(v, 0)?,
                    Cell::String(v) => DateTime::from_str(&v).ok()?,
                    Cell::Float(_) | Cell::Decimal(_) | Cell::Boolean(_) => return None,
                    Cell::DateTime(_) | Cell::Null => return Some(self),
                };
                if in_range(&v, range_start.as_ref(), range_end.as_ref()) {
                    Some(Cell::DateTime(v))
                } else {
                    None
                }
            }
            FieldKind::Checkbox => Some(Cell::Boolean(match self {
                Cell::Integer(v) => v != 0,
                Cell::String(v) => v.parse().ok()?,
                Cell::Float(_) | Cell::Decimal(_) | Cell::DateTime(_) => return None,
                Cell::Boolean(_) | Cell::Null => return Some(self),
            })),
            FieldKind::Enumeration {
                values,
                default_value,
                ..
            } => {
                let v: String = match self {
                    Cell::Integer(v) => v.to_string(),
                    Cell::Float(v) => v.to_string(),
                    Cell::Decimal(v) => v.to_string(),
                    Cell::Boolean(v) => v.to_string(),
                    Cell::DateTime(v) => v.to_string(),
                    Cell::String(v) => v,
                    Cell::Null => return Some(self),
                };
                Some(
                    if let Some((k, _)) = values.iter().find(|(_, value)| **value == v) {
                        Cell::Integer(*k)
                    } else {
                        Cell::Integer(*default_value)
                    },
                )
            }
        }
    }
}

fn in_range<T>(value: &T, range_start: Option<&T>, range_end: Option<&T>) -> bool
where
    T: PartialOrd,
{
    range_start.map_or(true, |start| value >= start) && range_end.map_or(true, |end| value <= end)
}
