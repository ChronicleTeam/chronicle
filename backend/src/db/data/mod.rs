//! Query functions for the Data Management feature.

mod entries;
mod fields;
mod tables;

use crate::{
    error::{ApiError, ApiResult},
    model::{
        data::{Entry, Field, FieldIdentifier, FieldMetadata, Table, TableData, TableIdentifier},
        Cell,
    },
    Id,
};
use itertools::Itertools;
use sqlx::{postgres::PgRow, PgExecutor, Postgres, Row};
pub use {entries::*, fields::*, tables::*};

pub enum Relation {
    Owned,
    NotOwned,
    Absent,
}

impl Relation {
    pub fn to_api_result(self) -> ApiResult<()> {
        match self {
            Relation::Owned => Ok(()),
            Relation::NotOwned => Err(ApiError::Forbidden),
            Relation::Absent => Err(ApiError::NotFound),
        }
    }
}

pub async fn get_table_data(
    executor: impl PgExecutor<'_> + Copy,
    table_id: Id,
) -> sqlx::Result<TableData> {
    let table: Table = sqlx::query_as(
        r#"
            SELECT 
                table_id,
                user_id,
                name,
                description,
                created_at,
                updated_at
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(executor)
    .await?;

    let fields: Vec<Field> = sqlx::query_as(
        r#"
            SELECT
                field_id,
                table_id,
                name,
                field_kind,
                created_at,
                updated_at
            FROM meta_field
            WHERE table_id = $1
            ORDER BY field_id
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await?;

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let select_columns = field_columns(&field_idents).join(", ");

    let table_ident = TableIdentifier::new(table_id, "data_table");
    let entries = sqlx::query::<Postgres>(&format!(
        r#"
            SELECT {select_columns}
            FROM {table_ident}
        "#
    ))
    .fetch_all(executor)
    .await?
    .into_iter()
    .map(|row| {
        entry_from_row(
            row,
            &fields
                .iter()
                .map(
                    |Field {
                         field_id,
                         field_kind,
                         ..
                     }| FieldMetadata {
                        field_id: field_id.clone(),
                        field_kind: field_kind.clone(),
                    },
                )
                .collect_vec(),
        )
    })
    .try_collect()?;

    Ok(TableData {
        table,
        fields,
        entries,
    })
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

fn field_columns<'a, T: IntoIterator<Item = &'a FieldIdentifier>>(
    field_idents: T,
) -> impl Iterator<Item = String> + use<'a, T> {
    field_idents.into_iter().map(|x| x.to_string()).chain(
        ["entry_id", "created_at", "updated_at"]
            .iter()
            .map(|x| x.to_string()),
    )
}
