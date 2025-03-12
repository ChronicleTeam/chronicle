use super::{entry_from_row, field_columns};
use crate::{
    db::Relation,
    model::{
        data::{Entry, FieldIdentifier, FieldMetadata, TableIdentifier},
        Cell,
    },
    Id,
};
use itertools::Itertools;
use sqlx::{postgres::PgArguments, query::Query, Acquire, PgExecutor, Postgres};

pub async fn create_entry(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    cell_entry: Vec<(Option<Cell>, FieldMetadata)>,
) -> sqlx::Result<Entry> {
    let mut tx = conn.begin().await?;

    let (cells, fields): (Vec<_>, Vec<_>) = cell_entry.into_iter().unzip();

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let parameters = (1..=cells.len()).map(|i| format!("${i}")).join(", ");
    let insert_columns = field_idents.iter().join(", ");
    let return_columns = field_columns(&field_idents).join(", ");

    let table_ident = TableIdentifier::new(table_id, "data_table");

    let insert_query = format!(
        r#"
            INSERT INTO {table_ident} ({insert_columns})
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
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    entry_id: Id,
    cell_entry: Vec<(Option<Cell>, FieldMetadata)>,
) -> sqlx::Result<Entry> {
    let mut tx = conn.begin().await?;

    let (cells, fields): (Vec<_>, Vec<_>) = cell_entry.into_iter().unzip();

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let parameters = field_idents
        .iter()
        .enumerate()
        .map(|(i, field_ident)| format!("{field_ident} = ${}", i + 2))
        .join(", ");

    let return_columns = field_columns(&field_idents).join(", ");

    let table_ident = TableIdentifier::new(table_id, "data_table");

    let update_query = format!(
        r#"
            UPDATE {table_ident}
            SET {parameters}
            WHERE entry_id = $1
            RETURNING {return_columns}
        "#,
    );
    let mut update_query = sqlx::query(&update_query).bind(entry_id);

    for cell in cells {
        update_query = bind_cell(update_query, cell);
    }

    let entry = entry_from_row(update_query.fetch_one(tx.as_mut()).await?, &fields)?;

    tx.commit().await?;

    Ok(entry)
}

pub async fn delete_entry(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    entry_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    let table_ident = TableIdentifier::new(table_id, "data_table");

    sqlx::query(&format!(
        r#"
            DELETE FROM {table_ident}
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
    let table_ident = TableIdentifier::new(table_id, "data_table");

    Ok(sqlx::query(&format!(
        r#"
            SELECT entry_id
            FROM {table_ident}
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
