use super::{entry_from_row, Relation};
use crate::{
    model::{data::{Entry, Field}, Cell, CellMap},
    Id,
};
use itertools::Itertools;
use sqlx::{postgres::PgArguments, query::Query, Acquire, PgExecutor, Postgres};

pub async fn create_entry(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    mut cell_entry: CellMap,
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
        "#,
    )
    .bind(table_id)
    .fetch_all(tx.as_mut())
    .await?;

    let (cells, data_field_names): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter_map(|field| {
            cell_entry
                .remove(&field.field_id)
                .zip(Some(field.data_field_name.as_str()))
        })
        .unzip();

    let parameters = (1..=cells.len()).map(|i| format!("${i}")).join(", ");
    let insert_columns = data_field_names.iter().join(", ");
    let return_columns = data_field_names
        .into_iter()
        .chain(["entry_id", "created_at", "updated_at"])
        .join(", ");

    let insert_query = format!(
        r#"
            INSERT INTO {data_table_name} ({insert_columns})
            VALUES ({parameters})
            RETURNING {return_columns}

        "#,
    );
    let mut insert_query = sqlx::query(&insert_query);

    for cell in cells {
        insert_query = bind_cell(insert_query, cell);
    }

    let row = insert_query.fetch_one(tx.as_mut()).await?;

    let entry = entry_from_row(row, &fields).unwrap();

    tx.commit().await?;

    Ok(entry)
}

pub async fn update_entry(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    entry_id: Id,
    mut cell_entry: CellMap,
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
        "#,
    )
    .bind(table_id)
    .fetch_all(tx.as_mut())
    .await?;

    let (cells, data_field_names): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter_map(|field| {
            cell_entry
                .remove(&field.field_id)
                .zip(Some(field.data_field_name.as_str()))
        })
        .unzip();

    let parameters = data_field_names
        .iter()
        .enumerate()
        .map(|(i, column)| format!("{column} = ${}", i + 2))
        .join(", ");

    let return_columns = data_field_names
        .into_iter()
        .chain(["entry_id", "created_at", "updated_at"])
        .join(", ");

    let update_query = format!(
        r#"
            UPDATE {data_table_name}
            SET {parameters}
            WHERE entry_id = $1
            RETURNING {return_columns}
        "#,
    );
    let mut update_query = sqlx::query(&update_query).bind(entry_id);

    for cell in cells {
        update_query = bind_cell(update_query, cell);
    }

    let entry = entry_from_row(update_query.fetch_one(tx.as_mut()).await?, &fields).unwrap();

    tx.commit().await?;

    Ok(entry)
}

pub async fn delete_entry(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    entry_id: Id,
) -> sqlx::Result<()> {
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

    sqlx::query(&format!(
        r#"
            DELETE FROM {data_table_name}
            WHERE entry_id = $1
        "#
    ))
    .bind(entry_id)
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
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

// fn bind_cell_scalar<'q, O>(
//     query: QueryScalar<'q, Postgres, O, PgArguments>,
//     cell: Option<Cell>,
// ) -> QueryScalar<'q, Postgres, O, PgArguments> {
//     if let Some(cell) = cell {
//         match cell {
//             Cell::Integer(v) => query.bind(v),
//             Cell::Float(v) => query.bind(v),
//             Cell::Decimal(v) => query.bind(v),
//             Cell::Boolean(v) => query.bind(v),
//             Cell::DateTime(v) => query.bind(v),
//             Cell::String(v) => query.bind(v),
//             Cell::Interval(_) => todo!(),
//         }
//     } else {
//         query.bind::<Option<bool>>(None)
//     }
// }

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
