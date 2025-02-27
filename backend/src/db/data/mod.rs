mod entries;
mod fields;
mod tables;

use std::iter;

use crate::{
    error::{ApiError, ApiResult},
    model::data::{DataTable, Entry, Field, FieldOptions, FullTable},
    Id,
};
use itertools::Itertools;
use sqlx::{types::Json, Acquire, Postgres};
pub use {entries::*, fields::*, tables::*};

// All SELECT statements lock selected rows during the transaction.
// A regular connection will lock only for the duration of the function.

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

pub async fn get_data_table(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
) -> sqlx::Result<DataTable> {
    let mut tx = connection.begin().await?;

    let FullTable {
        table,
        data_table_name,
    } = sqlx::query_as(
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
                options,
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

    let field_data: Vec<(Id, String, Json<FieldOptions>)> = sqlx::query_as(
        r#"
            SELECT field_id, data_field_name, options
            FROM meta_field
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(tx.as_mut())
    .await?;

    let select_columns = field_data
        .iter()
        .map(|(_, name, _)| name.as_str())
        .chain(["entry_id", "created_at", "updated_at"])
        .join(", ");

    let entries = sqlx::query::<Postgres>(&format!(
        r#"
            SELECT {select_columns}
            FROM {data_table_name}
        "#
    ))
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|row| Entry::from_row(row, &field_data).unwrap())
    .collect_vec();

    Ok(DataTable {
        table,
        fields,
        entries,
    })
}
