use crate::{
    Id,
    model::{
        Cell,
        data::{
            CreateField, Field, FieldIdentifier, FieldKind, FieldMetadata, TableIdentifier,
            UpdateField,
        },
    },
};
use itertools::Itertools;
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder, Row, types::Json};
use std::{collections::HashMap, mem::discriminant};
use tracing::debug;

/// Add a field to this table and add a column to the actual SQL table.
pub async fn create_field(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    CreateField { name, field_kind }: CreateField,
) -> sqlx::Result<Field> {
    let mut tx = conn.begin().await?;

    let field: Field = sqlx::query_as(
        r#"
            INSERT INTO meta_field (table_id, name, field_kind)
            VALUES ($1, $2, $3)
            RETURNING
                field_id,
                table_id,
                name,
                ordering,
                field_kind,
                created_at,
                updated_at
        "#,
    )
    .bind(table_id)
    .bind(name)
    .bind(sqlx::types::Json(field_kind.clone()))
    .fetch_one(tx.as_mut())
    .await?;

    let column_type = field_kind.get_sql_type();
    let table_ident = TableIdentifier::new(table_id, "data_table");
    let field_ident = FieldIdentifier::new(field.field_id);

    debug!("column_type {column_type}");
    debug!("field_ident {field_ident}");

    sqlx::query(&format!(
        r#"
            ALTER TABLE {table_ident}
            ADD COLUMN {field_ident} {column_type}
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    return Ok(field);
}

/// Add fields to this table and add columns the actual SQL table.
pub async fn create_fields(
    conn: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    fields: Vec<CreateField>,
) -> sqlx::Result<Vec<Field>> {
    let mut tx = conn.begin().await?;

    let fields: Vec<Field> =
        QueryBuilder::new(r#"INSERT INTO meta_field (table_id, name, field_kind)"#)
            .push_values(fields, |mut builder, field| {
                builder
                    .push_bind(table_id)
                    .push_bind(field.name)
                    .push_bind(Json(field.field_kind));
            })
            .push(
                r#"
                    RETURNING
                        field_id,
                        table_id,
                        name,
                        ordering,
                        field_kind,
                        created_at,
                        updated_at
                "#,
            )
            .build_query_as()
            .fetch_all(tx.as_mut())
            .await?;

    let add_column_statement = fields
        .iter()
        .map(|field| {
            let column_type = field.field_kind.0.get_sql_type();
            let field_ident = FieldIdentifier::new(field.field_id);
            format!(r#"ADD COLUMN {field_ident} {column_type}"#)
        })
        .join(", ");

    let table_ident = TableIdentifier::new(table_id, "data_table");

    sqlx::query(&format!(
        r#"
            ALTER TABLE {table_ident}
            {add_column_statement}
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    return Ok(fields);
}

/// Update a field in this table and change the column in the actual SQL table.
/// This will create a new field and keep the old one as backup if the [FieldKind]
/// variant is different.
pub async fn update_field(
    conn: impl Acquire<'_, Database = Postgres>,
    field_id: Id,
    UpdateField { name, field_kind }: UpdateField,
) -> sqlx::Result<Field> {
    let mut tx = conn.begin().await?;

    let Json(old_field_kind): Json<FieldKind> = sqlx::query_scalar(
        r"SELECT field_kind
        FROM meta_field
        WHERE field_id = $1",
    )
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    let mut field: Field = sqlx::query_as(
        r#"
            UPDATE meta_field
            SET name = $1, field_kind = $2
            WHERE field_id = $3
            RETURNING
                field_id,
                table_id,
                name,
                ordering,
                field_kind,
                created_at,
                updated_at
        "#,
    )
    .bind(name)
    .bind(Json(field_kind.clone()))
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    if discriminant(&field_kind) != discriminant(&old_field_kind) {
        field = convert_field_kind(tx.as_mut(), field, old_field_kind).await?;
    }

    tx.commit().await?;

    Ok(field)
}

/// Create a new field with all the cells converted to the new [FieldKind].
/// Renames the old field to avoid conflict.
async fn convert_field_kind(
    conn: impl Acquire<'_, Database = Postgres>,
    field: Field,
    old_field_kind: FieldKind,
) -> sqlx::Result<Field> {
    let mut tx = conn.begin().await?;

    let field_ident = FieldIdentifier::new(field.field_id);
    let table_ident = TableIdentifier::new(field.table_id, "data_table");
    let rows = sqlx::query(&format!(
        r#"
            SELECT entry_id, {field_ident}
            FROM {table_ident}
        "#
    ))
    .fetch_all(tx.as_mut())
    .await?;

    let cells: Vec<(Id, Cell)> = rows
        .into_iter()
        .map(|row| {
            let cell = Cell::from_field_row(&row, &field_ident.unquote(), &old_field_kind)?;
            Ok((
                row.get("entry_id"),
                cell.convert_field_kind(&field.field_kind.0)
                    .unwrap_or(Cell::Null),
            ))
        })
        .collect::<sqlx::Result<_>>()?;

    sqlx::query(
        r#"
            UPDATE meta_field
            SET name = name || ' (BACKUP)', field_kind = $1
            WHERE field_id = $2
        "#,
    )
    .bind(Json(old_field_kind))
    .bind(field.field_id)
    .execute(tx.as_mut())
    .await?;

    let field = create_field(
        tx.as_mut(),
        field.table_id,
        CreateField {
            name: field.name,
            field_kind: field.field_kind.0,
        },
    )
    .await?;

    let field_ident = FieldIdentifier::new(field.field_id);

    QueryBuilder::<Postgres>::new(format!(
        r#"
            UPDATE {table_ident}
            SET {field_ident} = data.cell
            FROM (
        "#
    ))
    .push_values(cells, |mut builder, (id, cell)| {
        builder.push_bind(id);
        cell.push_bind(&mut builder);
    })
    .push(format!(
        r#"
            ) AS data (entry_id, cell)
            WHERE {table_ident}.entry_id = data.entry_id
        "#
    ))
    .build()
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(field)
}

/// Delete this field and remove the column from the actual SQL table.
pub async fn delete_field(
    conn: impl Acquire<'_, Database = Postgres>,
    field_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    let table_id = sqlx::query_scalar(
        r#"
            DELETE FROM meta_field
            WHERE field_id = $1
            RETURNING table_id
        "#,
    )
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    let table_ident = TableIdentifier::new(table_id, "data_table");
    let field_ident = FieldIdentifier::new(field_id);

    sqlx::query(&format!(
        r#"
            ALTER TABLE {table_ident}
            DROP COLUMN {field_ident}
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Get all fields of this table.
pub async fn get_fields(executor: impl PgExecutor<'_>, table_id: Id) -> sqlx::Result<Vec<Field>> {
    sqlx::query_as(
        r#"
            SELECT
                field_id,
                table_id,
                name,
                ordering,
                field_kind,
                created_at,
                updated_at
            FROM meta_field
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await
}

/// Get all field IDs of this table.
pub async fn get_field_ids(executor: impl PgExecutor<'_>, table_id: Id) -> sqlx::Result<Vec<Id>> {
    sqlx::query_scalar(
        r#"
            SELECT field_id
            FROM meta_field
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await
}

/// Set the order of all fields in this table.
pub async fn set_field_order(
    conn: impl Acquire<'_, Database = Postgres>,
    order: HashMap<Id, i32>,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    sqlx::query(
        r#"
            UPDATE meta_field AS f
            SET ordering = n.ordering
            FROM (
                SELECT
                    unnest($1::int[]) AS field_id,
                    unnest($2::int[]) AS ordering
            ) AS n
            WHERE f.field_id = n.field_id
        "#,
    )
    .bind(order.keys().collect_vec())
    .bind(order.values().collect_vec())
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Get the all [FieldMetadata] of this table.
pub async fn get_fields_metadata(
    executor: impl PgExecutor<'_>,
    table_id: Id,
) -> sqlx::Result<Vec<FieldMetadata>> {
    sqlx::query_as(
        r#"
            SELECT
                field_id,
                field_kind
            FROM meta_field
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_all(executor)
    .await
}

/// Return the [Relation] between the table and this field.
pub async fn field_exists(
    executor: impl PgExecutor<'_>,
    table_id: Id,
    field_id: Id,
) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM meta_field
                WHERE table_id = $1 field_id = $2
            )
        "#,
    )
    .bind(table_id)
    .bind(field_id)
    .fetch_one(executor)
    .await
}
