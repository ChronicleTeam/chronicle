//! Query functions for the Data Management feature.

mod entries;
mod fields;
mod tables;

use crate::model::{
    Cell,
    data::{Entry, FieldIdentifier, FieldMetadata},
};
use itertools::Itertools;
use sqlx::{Row, postgres::PgRow};
pub use {entries::*, fields::*, tables::*};

/// All columns of a user dynamic SQL table prepared for a "select" query.
fn select_columns(with_parent: bool, field_idents: &[FieldIdentifier]) -> String {
    field_idents
        .iter()
        .map(|x| x.to_string())
        .chain(
            ["entry_id", "created_at", "updated_at"]
                .into_iter()
                .chain(with_parent.then_some("parent_id"))
                .map(|x| x.to_string()),
        )
        .join(", ")
}

/// All columns of a user dynamic SQL table prepared for an "insert" query.
fn insert_columns(with_parent: bool, field_idents: &[FieldIdentifier]) -> String {
    field_idents
        .iter()
        .map(|x| x.to_string())
        .chain(with_parent.then(|| "parent_id".to_string()))
        .join(", ")
}

/// All columns of a user dynamic SQL table prepared for an "update" query.
fn update_columns(with_parent: bool, field_idents: &[FieldIdentifier], position: usize) -> String {
    field_idents
        .iter()
        .map(|x| x.to_string())
        .chain(with_parent.then(|| "parent_id".to_string()))
        .enumerate()
        .map(|(i, field_ident)| format!("{field_ident} = ${}", position + i))
        .join(", ")
}

/// Convert this [PgRow] into an [Entry].
fn entry_from_row(row: PgRow, fields: &[FieldMetadata]) -> sqlx::Result<Entry> {
    Ok(Entry {
        entry_id: row.get("entry_id"),
        parent_id: row.try_get("parent_id").or_else(|e| match e {
            sqlx::Error::ColumnNotFound(_) => Ok(None),
            e => Err(e),
        })?,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        cells: fields
            .iter()
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::model::data::FieldIdentifier;
    use itertools::Itertools;

    #[test]
    fn select_columns() {
        let field_idents = [1, 2, 3]
            .into_iter()
            .map(|id| FieldIdentifier::new(id))
            .collect_vec();
        let select_columns = super::select_columns(false, &field_idents);
        assert_eq!(
            select_columns,
            r#""f1", "f2", "f3", entry_id, created_at, updated_at"#
        );

        let select_columns = super::select_columns(true, &field_idents);
        assert_eq!(
            select_columns,
            r#""f1", "f2", "f3", entry_id, created_at, updated_at, parent_id"#
        );
    }

    #[test]
    fn insert_columns() {
        let field_idents = [1, 2, 3]
            .into_iter()
            .map(|id| FieldIdentifier::new(id))
            .collect_vec();
        let insert_columns = super::insert_columns(false, &field_idents);
        assert_eq!(insert_columns, r#""f1", "f2", "f3""#);

        let insert_columns = super::insert_columns(true, &field_idents);
        assert_eq!(insert_columns, r#""f1", "f2", "f3", parent_id"#);
    }

    #[test]
    fn update_columns() {
        let field_idents = [1, 2, 3]
            .into_iter()
            .map(|id| FieldIdentifier::new(id))
            .collect_vec();
        let update_columns = super::update_columns(false, &field_idents, 1);
        assert_eq!(update_columns, r#""f1" = $1, "f2" = $2, "f3" = $3"#);

        let update_columns = super::update_columns(true, &field_idents, 2);
        assert_eq!(
            update_columns,
            r#""f1" = $2, "f2" = $3, "f3" = $4, parent_id = $5"#
        );
    }
}
