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

fn field_columns<'a, T: IntoIterator<Item = &'a FieldIdentifier>>(
    field_idents: T,
) -> impl Iterator<Item = String> + use<'a, T> {
    field_idents.into_iter().map(|x| x.to_string()).chain(
        ["entry_id", "created_at", "updated_at"]
            .iter()
            .map(|x| x.to_string()),
    )
}

fn entry_from_row<'a>(row: PgRow, fields: &[FieldMetadata]) -> sqlx::Result<Entry> {
    Ok(Entry {
        entry_id: row.get("entry_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        cells: fields
            .into_iter()
            .map(|field| {
                Cell::from_field_row(
                    &row,
                    &FieldIdentifier::new(field.field_id).unquoted(),
                    &field.field_kind.0,
                )
                .map(|v| (field.field_id, v))
            })
            .try_collect()?,
    })
}
