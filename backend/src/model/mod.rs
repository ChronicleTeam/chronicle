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

use chrono::{DateTime, Utc};
use data::FieldKind;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{postgres::PgRow, Row};
use std::collections::HashMap;
use viz::Aggregate;

use crate::Id;

/// This represents all the data types in user entries and charts.
#[derive(Serialize)]
#[serde(untagged)]
pub enum Cell {
    Integer(i64),
    Float(f64),
    Decimal(Decimal),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    String(String),
    Interval(()),
}

impl Cell {
    /// Get the `Cell` from this PostgreSQL row into the proper type based on `FieldKind`.
    ///
    /// Returns `Ok(None)` on `null`
    pub fn from_field_row(
        row: &PgRow,
        index: &str,
        field_kind: &FieldKind,
    ) -> sqlx::Result<Option<Cell>> {
        Ok(match field_kind {
            FieldKind::Text { .. } | FieldKind::WebLink { .. } | FieldKind::Email { .. } => {
                row.try_get::<Option<_>, _>(index)?.map(Cell::String)
            }
            FieldKind::Integer { .. }
            | FieldKind::Progress { .. }
            | FieldKind::Enumeration { .. } => {
                row.try_get::<Option<_>, _>(index)?.map(Cell::Integer)
            }
            FieldKind::Float { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Float),
            FieldKind::Money { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Decimal),
            FieldKind::DateTime { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::DateTime),
            FieldKind::Interval { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Interval),
            FieldKind::Checkbox => row.try_get::<Option<_>, _>(index)?.map(Cell::Boolean),
        })
    }

    /// Get the `Cell` from this PostgreSQL row into the proper type based on `Aggregate` and `FieldKind`.
    ///
    /// Returns `Ok(None)` on `null`
    pub fn from_aggregate_row(
        row: &PgRow,
        index: &str,
        aggregate: &Aggregate,
        field_kind: &FieldKind,
    ) -> sqlx::Result<Option<Cell>> {
        Ok(match aggregate {
            Aggregate::Sum | Aggregate::Average => match field_kind {
                FieldKind::Float { .. } => row.try_get::<Option<_>, _>(index)?.map(Cell::Float),
                _ => row.try_get::<Option<_>, _>(index)?.map(Cell::Decimal),
            },
            Aggregate::Min | Aggregate::Max => Self::from_field_row(row, index, field_kind)?,
            Aggregate::Count => row.try_get::<Option<_>, _>(index)?.map(Cell::Integer),
        })
    }
}

/// A hash map representing a row of cells in an table or chart.
pub type CellMap = HashMap<Id, Option<Cell>>;
