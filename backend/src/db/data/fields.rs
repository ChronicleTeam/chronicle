use crate::{
    db::Relation, model::data::{
        CreateField, Field, FieldIdentifier, FieldKind, FieldMetadata,
        TableIdentifier, UpdateField,
    }, Id
};
use itertools::Itertools;
use sqlx::{types::Json, Acquire, PgExecutor, Postgres};
use std::{collections::HashMap, mem::discriminant};

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

pub async fn update_field(
    conn: impl Acquire<'_, Database = Postgres>,
    field_id: Id,
    UpdateField {
        name,
        field_kind,
    }: UpdateField,
) -> sqlx::Result<Field> {
    let mut tx = conn.begin().await?;

    let Json(old_field_kind): Json<FieldKind> = sqlx::query_scalar(
        r#"
            SELECT field_kind
            FROM meta_field
            WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    // Should create a new field and attempt to convert
    if discriminant(&field_kind) != discriminant(&old_field_kind) {
        todo!("Not implemented")
    }

    let field: Field = sqlx::query_as(
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
    .bind(Json(field_kind))
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(field)
}

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

pub async fn check_field_relation(
    executor: impl PgExecutor<'_>,
    table_id: Id,
    field_id: Id,
) -> sqlx::Result<Relation> {
    sqlx::query_scalar::<_, Id>(
        r#"
            SELECT table_id
            FROM meta_field
            WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_optional(executor)
    .await
    .map(|id| match id {
        None => Relation::Absent,
        Some(id) if id == table_id => Relation::Owned,
        Some(_) => Relation::NotOwned,
    })
}
