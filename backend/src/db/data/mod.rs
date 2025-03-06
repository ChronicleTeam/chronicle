mod entries;
mod fields;
mod tables;

use crate::{
    error::{ApiError, ApiResult},
    model::{data::{Entry, Field, Table, TableData}, Cell},
    Id,
};
use itertools::Itertools;
use sqlx::{postgres::PgRow, Acquire, Postgres, Row};
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
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
) -> sqlx::Result<TableData> {
    let mut tx = connection.begin().await?;

    let table: Table = sqlx::query_as(
        r#"
            SELECT 
                table_id,
                user_id,
                name,
                description,
                created_at,
                updated_at,
                data_table_name
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(tx.as_mut())
    .await?;

    let fields: Vec<Field> = sqlx::query_as(
        r#"
            SELECT
                field_id,
                table_id,
                name,
                field_kind,
                data_field_name,
                created_at,
                updated_at
            FROM meta_field
            WHERE table_id = $1
            ORDER BY field_id
        "#,
    )
    .bind(table_id)
    .fetch_all(tx.as_mut())
    .await?;

    let select_columns = fields
        .iter()
        .map(|field| field.data_field_name.as_str())
        .chain(["entry_id", "created_at", "updated_at"])
        .join(", ");

    let data_table_name = &table.data_table_name;
    let entries = sqlx::query::<Postgres>(&format!(
        r#"
            SELECT {select_columns}
            FROM {data_table_name}
        "#
    ))
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|row| entry_from_row(row, &fields).unwrap())
    .collect_vec();

    Ok(TableData {
        table,
        fields,
        entries,
    })
}

fn entry_from_row(row: PgRow, fields: &[Field]) -> sqlx::Result<Entry> {
    Ok(Entry {
        entry_id: row.get("entry_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        cells: fields
            .iter()
            .map(|field| {
                Cell::from_field_row(&row, &field.data_field_name, &field.field_kind.0)
                    .or_else(|e| {
                        if matches!(e, sqlx::Error::ColumnNotFound(_)) {
                            Ok(None)
                        } else {
                            Err(e)
                        }
                    })
                    .map(|v| (field.field_id, v))
            })
            .try_collect()?,
    })
}
