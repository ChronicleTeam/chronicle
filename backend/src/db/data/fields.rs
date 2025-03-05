use super::Relation;
use crate::{
    model::data::{CreateField, Field, FieldKind, UpdateField},
    Id,
};
use sqlx::{types::Json, Acquire, PgExecutor, Postgres};
use std::mem::discriminant;

pub async fn create_field(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
    CreateField { name, field_kind }: CreateField,
) -> sqlx::Result<Field> {
    let mut tx = connection.begin().await?;

    let field: Field = sqlx::query_as(
        r#"
            INSERT INTO meta_field (table_id, name, field_kind)
            VALUES ($1, $2, $3)
            RETURNING
                field_id,
                table_id,
                name,
                field_kind,
                data_field_name,
                created_at,
                updated_at
        "#,
    )
    .bind(table_id)
    .bind(name)
    .bind(sqlx::types::Json(field_kind.clone()))
    .fetch_one(tx.as_mut())
    .await?;

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

    let column_type = match field_kind {
        FieldKind::Text { .. } => "TEXT",
        FieldKind::Integer { .. } => "BIGINT",
        FieldKind::Decimal { .. } => "DOUBLE",
        FieldKind::Money { .. } => "numeric_money",
        FieldKind::Progress { .. } => "BIGINT NOT NULL DEFAULT 0",
        FieldKind::DateTime { .. } => "TIMESTAMPTZ",
        FieldKind::Interval { .. } => "INTERVAL",
        FieldKind::WebLink { .. } => "COLLATE case_insensitive TEXT",
        FieldKind::Email { .. } => "COLLATE case_insensitive TEXT",
        FieldKind::Checkbox => "BOOLEAN NOT NULL DEFAULT FALSE",
        FieldKind::Enumeration { .. } => "BIGINT",
    };

    let data_field_name = &field.data_field_name;

    sqlx::query(&format!(
        r#"
            ALTER TABLE {data_table_name}
            ADD COLUMN {data_field_name} {column_type}
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    return Ok(field);
}

pub async fn update_field(
    connection: impl Acquire<'_, Database = Postgres>,
    field_id: Id,
    UpdateField { name, field_kind }: UpdateField,
) -> sqlx::Result<Field> {
    let mut tx = connection.begin().await?;

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
                field_kind,
                data_field_name,
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
    connection: impl Acquire<'_, Database = Postgres>,
    field_id: Id,
) -> sqlx::Result<()> {
    let mut tx = connection.begin().await?;

    let data_table_name: String = sqlx::query_scalar(
        r#"
            SELECT data_table_name
            FROM meta_table AS t
            JOIN meta_field AS f
            ON t.table_id = f.table_id
            WHERE field_id = $1
        "#,
    )
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    let data_field_name: String = sqlx::query_scalar(
        r#"
            DELETE FROM meta_field
            WHERE field_id = $1
            RETURNING data_field_name
        "#,
    )
    .bind(field_id)
    .fetch_one(tx.as_mut())
    .await?;

    sqlx::query(&format!(
        r#"
            ALTER TABLE {data_table_name}
            DROP COLUMN {data_field_name} CASCADE
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
                field_kind,
                data_field_name,
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

pub async fn check_field_relation(
    executor: impl PgExecutor<'_>,
    table_id: Id,
    field_id: Id,
) -> sqlx::Result<Relation> {
    sqlx::query_scalar::<_, Id>(
        r#"
            SELECT table_id
            FROM meta_fieldF
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
