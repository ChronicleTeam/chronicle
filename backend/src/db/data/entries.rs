use super::{entry_from_row, select_columns, update_columns};
use crate::{
    db::{data::insert_columns, Relation},
    model::{
        data::{Entry, FieldIdentifier, FieldMetadata, TableIdentifier},
        Cell,
    },
    Id,
};
use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

/// Add an entry to the actual SQL table.
pub async fn create_entry(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    parent_id: Option<Id>,
    cells: Vec<(Cell, FieldMetadata)>,
) -> sqlx::Result<Entry> {
    let mut tx = conn.begin().await?;

    let (entry, fields): (Vec<_>, Vec<_>) = cells.into_iter().unzip();

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let parameters = (1..=entry.len())
        .map(|i| format!("${i}"))
        .chain(parent_id.map(|_| format!("${}", entry.len() + 1)))
        .join(", ");

    let insert_columns = insert_columns(parent_id.is_some(), &field_idents);

    let return_columns = select_columns(parent_id.is_some(), &field_idents);

    let table_ident = TableIdentifier::new(table_id, "data_table");

    let insert_query = format!(
        r#"
            INSERT INTO {table_ident} ({insert_columns})
            VALUES ({parameters})
            RETURNING {return_columns}

        "#,
    );
    let mut insert_query = sqlx::query(&insert_query);

    for cell in entry {
        insert_query = cell.bind(insert_query);
    }

    if let Some(parent_id) = parent_id {
        insert_query = insert_query.bind(parent_id);
    }

    let row = insert_query.fetch_one(tx.as_mut()).await?;
    let entry = entry_from_row(row, &fields).unwrap();

    tx.commit().await?;

    Ok(entry)
}

/// Add entries to the actual SQL table.
pub async fn create_entries(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    parent_id: Option<Id>,
    fields: Vec<FieldMetadata>,
    entries: Vec<Vec<Cell>>,
) -> sqlx::Result<Vec<Entry>> {
    assert!(entries
        .iter()
        .next()
        .map_or(true, |entry| entry.len() == fields.len()));
    let mut tx = conn.begin().await?;

    let table_ident = TableIdentifier::new(table_id, "data_table");

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let insert_columns = insert_columns(parent_id.is_some(), &field_idents);
    let return_columns = select_columns(parent_id.is_some(), &field_idents);

    let rows = QueryBuilder::new(format!(r#"INSERT INTO {table_ident} ({insert_columns})"#))
        .push_values(entries, |mut builder, entry| {
            for cell in entry {
                cell.push_bind(&mut builder);
            }
            if let Some(parent_id) = parent_id {
                builder.push_bind(parent_id);
            }
        })
        .push(format!(
            r#"
                RETURNING {return_columns}
            "#
        ))
        .build()
        .fetch_all(tx.as_mut())
        .await?;

    let entries = rows
        .into_iter()
        .map(|row| entry_from_row(row, &fields).unwrap())
        .collect_vec();

    tx.commit().await?;

    Ok(entries)
}

/// Update an entry in the actual SQL table.
pub async fn update_entry(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    entry_id: Id,
    parent_id: Option<Id>,
    fields: Vec<FieldMetadata>,
    cells: Vec<Cell>,
) -> sqlx::Result<Entry> {
    let mut tx = conn.begin().await?;

    let field_idents = fields
        .iter()
        .map(|field| FieldIdentifier::new(field.field_id))
        .collect_vec();

    let set_columns = update_columns(parent_id.is_some(), &field_idents, 2);

    let return_columns = select_columns(parent_id.is_some(), &field_idents);

    let table_ident = TableIdentifier::new(table_id, "data_table");

    let update_query = format!(
        r#"
            UPDATE {table_ident}
            SET {set_columns}
            WHERE entry_id = $1
            RETURNING {return_columns}
        "#,
    );
    let mut update_query = sqlx::query(&update_query).bind(entry_id);

    for cell in cells {
        update_query = cell.bind(update_query);
    }
    if let Some(parent_id) = parent_id {
        update_query = update_query.bind(parent_id);
    }

    let entry = entry_from_row(update_query.fetch_one(tx.as_mut()).await?, &fields)?;

    tx.commit().await?;

    Ok(entry)
}

/// Delete an entry in the actual SQL table.
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

/// Return the [Relation] between the table and this entry.
pub async fn check_entry_relation(
    executor: impl PgExecutor<'_>,
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