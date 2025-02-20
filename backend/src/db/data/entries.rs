use super::{entry_from_row, Relation};
use crate::{
    model::data::{Cell, Entry, FieldOptions},
    Id,
};
use itertools::Itertools;
use sqlx::{
    postgres::PgArguments,
    query::{Query, QueryScalar},
    types::Json,
    Acquire, PgExecutor, Postgres,
};
use std::collections::HashMap;

pub async fn create_entry(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    mut entry: HashMap<Id, Option<Cell>>,
) -> sqlx::Result<Id> {
    let mut tx = connection.begin().await?;

    let data_table_name: String = sqlx::query_scalar(
        r#"
            SELECT data_table_name
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(tx.as_mut())
    .await?;

    let field_data: Vec<(Id, String)> = sqlx::query_as(
        r#"
            SELECT field_id, data_field_name
            FROM meta_field
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(tx.as_mut())
    .await?;

    let (cells, data_field_names): (Vec<_>, Vec<_>) = field_data
        .into_iter()
        .filter_map(|(field_id, identifier)| entry.remove(&field_id).zip(Some(identifier)))
        .unzip();

    let query_parameters = (1..=cells.len()).map(|i| format!("${i}")).join(", ");
    let query_columns = data_field_names.into_iter().join(", ");

    let insert_query = format!(
        r#"
            INSERT INTO {data_table_name} ({query_columns})
            VALUES ({query_parameters})
            RETURNING entry_id
        "#,
    );
    let mut insert_query = sqlx::query_scalar(&insert_query);

    for cell in cells {
        insert_query = bind_cell_scalar(insert_query, cell);
    }

    let entry_id: Id = insert_query.fetch_one(tx.as_mut()).await?;

    tx.commit().await?;
    Ok(entry_id)
}

pub async fn update_entry(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    entry_id: Id,
    mut entry: HashMap<Id, Option<Cell>>,
) -> sqlx::Result<Entry> {
    let mut tx = connection.begin().await?;

    let data_table_name: String = sqlx::query_scalar(
        r#"
            SELECT data_table_name
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(tx.as_mut())
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

    let (cells, data_field_names): (Vec<_>, Vec<_>) = field_data
        .iter()
        .filter_map(|(field_id, identifier, _)| entry.remove(&field_id).zip(Some(identifier)))
        .unzip();

    let query_parameters = data_field_names
        .iter()
        .enumerate()
        .map(|(i, column)| format!("{column} = ${}", i + 2))
        .join(", ");
    let query_columns = data_field_names.into_iter().join(", ");

    let update_query = format!(
        r#"
            UPDATE {data_table_name}
            SET {query_parameters}
            WHERE entry_id = $1
            RETURNING {query_columns}, entry_id, created_at, updated_at
        "#,
    );
    let mut update_query = sqlx::query(&update_query).bind(entry_id);

    for cell in cells {
        update_query = bind_cell(update_query, cell);
    }

    let entry = entry_from_row(update_query.fetch_one(tx.as_mut()).await?, &field_data)?;

    tx.commit().await?;

    Ok(entry)
}

pub async fn check_entry_relation(
    executor: impl PgExecutor<'_> + Copy,
    table_id: Id,
    entry_id: Id,
) -> sqlx::Result<Relation> {
    let data_table_name: String = sqlx::query_scalar(
        r#"
            SELECT data_table_name
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_one(executor)
    .await?;

    Ok(sqlx::query(&format!(
        r#"
            SELECT entry_id
            FROM {data_table_name}
            WHERE entry_id = $1
        "#
    ))
    .bind(entry_id)
    .fetch_optional(executor)
    .await?
    .map_or(Relation::Absent, |_| Relation::Owned))
}

fn bind_cell_scalar<'q, O>(
    query: QueryScalar<'q, Postgres, O, PgArguments>,
    cell: Option<Cell>,
) -> QueryScalar<'q, Postgres, O, PgArguments> {
    if let Some(cell) = cell {
        match cell {
            Cell::Integer(v) => query.bind(v),
            Cell::Float(v) => query.bind(v),
            Cell::Decimal(v) => query.bind(v),
            Cell::Boolean(v) => query.bind(v),
            Cell::DateTime(v) => query.bind(v),
            Cell::String(v) => query.bind(v),
            Cell::Interval(_) => todo!(),
        }
    } else {
        query.bind::<Option<bool>>(None)
    }
}

fn bind_cell<'q>(
    query: Query<'q, Postgres, PgArguments>,
    cell: Option<Cell>,
) -> Query<'q, Postgres, PgArguments> {
    if let Some(cell) = cell {
        match cell {
            Cell::Integer(v) => query.bind(v),
            Cell::Float(v) => query.bind(v),
            Cell::Decimal(v) => query.bind(v),
            Cell::Boolean(v) => query.bind(v),
            Cell::DateTime(v) => query.bind(v),
            Cell::String(v) => query.bind(v),
            Cell::Interval(_) => todo!(),
        }
    } else {
        query.bind::<Option<bool>>(None)
    }
}
