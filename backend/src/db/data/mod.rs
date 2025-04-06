//! Query functions for the Data Management feature.

mod entries;
mod fields;
mod tables;

use crate::model::{
    data::{Entry, FieldIdentifier, FieldMetadata},
    Cell,
};
use itertools::Itertools;
use sqlx::{postgres::PgRow, Row};
pub use {entries::*, fields::*, tables::*};

/// All columns of a user dynamic SQL table prepared for a "select" query.
fn select_columns(with_parent: bool, field_idents: &[FieldIdentifier]) -> String {
    field_idents
        .into_iter()
        .map(|x| x.to_string())
        .chain(
            ["entry_id", "created_at", "updated_at"]
                .into_iter()
                .chain(if with_parent { Some("parent_id") } else { None })
                .map(|x| x.to_string()),
        )
        .join(", ")
}

/// All columns of a user dynamic SQL table prepared for an "insert" query.
fn insert_columns(with_parent: bool, field_idents: &[FieldIdentifier]) -> String {
    field_idents
        .iter()
        .map(|x| x.to_string())
        .chain(if with_parent {
            Some("parent_id".to_string())
        } else {
            None
        })
        .join(", ")
}

/// All columns of a user dynamic SQL table prepared for an "update" query.
fn update_columns(with_parent: bool, field_idents: &[FieldIdentifier], position: usize) -> String {
    field_idents
        .iter()
        .map(|x| x.to_string())
        .chain(if with_parent {
            Some("parent_id".to_string())
        } else {
            None
        })
        .enumerate()
        .map(|(i, field_ident)| format!("{field_ident} = ${}", position + i))
        .join(", ")
}


/// Convert this [PgRow] into an [Entry].
fn entry_from_row<'a>(row: PgRow, fields: &[FieldMetadata]) -> sqlx::Result<Entry> {
    Ok(Entry {
        entry_id: row.get("entry_id"),
        parent_id: row.try_get("parent_id").or_else(|e| match e {
            sqlx::Error::ColumnNotFound(_) => Ok(None),
            e => Err(e),
        })?,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        cells: fields
            .into_iter()
            .map(|field| {
                Cell::from_field_row(
                    &row,
                    &FieldIdentifier::new(field.field_id).unquote(),
                    &field.field_kind.0,
                )
                .map(|v| (field.field_id, v))
            })
            .try_collect()?,
    })
}
