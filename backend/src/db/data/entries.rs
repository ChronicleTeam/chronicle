use crate::{model::data::Cell, Id};
use itertools::Itertools;
use sqlx::{postgres::PgArguments, query::QueryAs, Acquire, Postgres};
use std::collections::HashMap;

pub async fn create_entry(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    mut entry: HashMap<Id, Cell>,
) -> sqlx::Result<Id> {
    let mut tx = connection.begin().await?;

    let (data_table_name,): (String,) = sqlx::query_as(
        r#"
            SELECT data_table_name
            FROM meta_table
            WHERE table_id = $1
            FOR UPDATE
        "#,
    )
    .bind(table_id)
    .fetch_one(tx.as_mut())
    .await?;

    let data_field_names: HashMap<Id, String> = sqlx::query_as(
        r#"
            SELECT field_id, data_field_name
            FROM meta_field
            WHERE table_id = $1
            FOR UPDATE
        "#,
    )
    .bind(table_id)
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .collect();

    let (cells, data_field_names): (Vec<_>, Vec<_>) = data_field_names
        .into_iter()
        .filter_map(|(field_id, identifier)| entry.remove(&field_id).zip(Some(identifier)))
        .unzip();

    let data_field_names = data_field_names.into_iter().join(", ");
    let parameters = (1..cells.len() + 1).map(|i| format!("${i}")).join(", ");

    let insert_query = format!(
        r#"
            INSERT INTO {data_table_name} ({data_field_names})
            VALUES ({parameters})
            RETURNING entry_id
        "#
    );

    let mut insert_query = sqlx::query_as(&insert_query);

    for cell in cells {
        insert_query = bind_cell(insert_query, cell);
    }

    let (entry_id,): (Id,) = insert_query.fetch_one(tx.as_mut()).await?;

    tx.commit().await?;
    Ok(entry_id)
}

fn bind_cell<'q, O>(
    query: QueryAs<'q, Postgres, O, PgArguments>,
    cell: Cell,
) -> QueryAs<'q, Postgres, O, PgArguments> {
    match cell {
        Cell::Integer { i: v } => query.bind(v),
        Cell::Float { f: v } => query.bind(v),
        Cell::Decimal { d: v } => query.bind(v),
        Cell::Boolean(v) => query.bind(v),
        Cell::DateTime(v) => query.bind(v),
        Cell::String(v) => query.bind(v),
        Cell::Interval(_) => todo!(),
        Cell::Image(_) => todo!(),
        Cell::File(_) => todo!(),
    }
}
