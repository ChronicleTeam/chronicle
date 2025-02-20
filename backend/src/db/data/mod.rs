mod entries;
mod fields;
mod tables;

use crate::{
    model::data::{Cell, DataTable, Entry, Field, FieldOptions, Table},
    Id,
};
use itertools::Itertools;
use sqlx::{postgres::PgRow, types::Json, Acquire, FromRow, Postgres, Row};
pub use {entries::*, fields::*, tables::*};

// All SELECT statements lock selected rows during the transaction.
// A regular connection will lock only for the duration of the function.

pub enum Relation {
    Owned,
    NotOwned,
    Absent,
}

pub async fn get_data_table(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
) -> sqlx::Result<DataTable> {
    let mut tx = connection.begin().await?;

    let row = sqlx::query(
        r#"
            SELECT 
                data_table_name
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
    .fetch_one(tx.as_mut())
    .await?;

    let data_table_name: String = row.get("data_table_name");
    let table = Table::from_row(&row)?;

    // let table: Table = sqlx::query_as(
    //     r#"
    //         SELECT
    //             table_id,
    //             user_id,
    //             name,
    //             description,
    //             created_at,
    //             updated_at
    //         FROM meta_table
    //         WHERE table_id = $1
    //     "#,
    // )
    // .fetch_one(tx.as_mut())
    // .await?;

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

    let query_columns = field_data.iter().map(|(_, name, _)| name).join(", ");

    let entries = sqlx::query::<Postgres>(&format!(
        r#"
            SELECT {query_columns}, entry_id
            FROM {data_table_name}
        "#
    ))
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|row| entry_from_row(row, &field_data))
    .collect::<sqlx::Result<Vec<_>>>()?;

    Ok(DataTable {
        table,
        fields,
        entries,
    })
}


fn entry_from_row(row: PgRow, field_data: &[(Id, String, Json<FieldOptions>)]) -> sqlx::Result<Entry> {
    Ok(Entry {
        entry_id: row.get("entry_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        cells: field_data
            .iter()
            .map(|(id, name, options)| {
                match cell_from_row(&row, name.as_str(), &options.0) {
                    Ok(v) => Ok(Some(v)),
                    Err(sqlx::Error::ColumnNotFound(_)) => Ok(None),
                    Err(e) => Err(e),
                }
                .map(|v| (*id, v))
            })
            .try_collect()?,
    })
}

fn cell_from_row(row: &PgRow, index: &str, field_options: &FieldOptions) -> sqlx::Result<Cell> {
    Ok(match field_options {
        FieldOptions::Text { .. } | FieldOptions::WebLink { .. } | FieldOptions::Email { .. } => {
            Cell::String(row.try_get(index)?)
        }
        FieldOptions::Integer { .. }
        | FieldOptions::Progress { .. }
        | FieldOptions::Enumeration { .. } => Cell::Integer(row.try_get(index)?),
        FieldOptions::Decimal { .. } => Cell::Float(row.try_get(index)?),
        FieldOptions::Money { .. } => Cell::Decimal(row.try_get(index)?),
        FieldOptions::DateTime { .. } => Cell::DateTime(row.try_get(index)?),
        FieldOptions::Interval { .. } => Cell::Interval(row.try_get(index)?),
        FieldOptions::Checkbox => Cell::Boolean(row.try_get(index)?),
    })
}
