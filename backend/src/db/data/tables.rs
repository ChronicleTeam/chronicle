use super::Relation;
use crate::{
    model::data::{CreateTable, Table, UpdateTable},
    Id,
};
use sqlx::{Acquire, PgExecutor, Postgres};

pub async fn create_table(
    connection: impl Acquire<'_, Database = Postgres>,
    user_id: Id,
    CreateTable { name, description }: CreateTable,
) -> sqlx::Result<Table> {
    let mut tx = connection.begin().await?;

    let table: Table = sqlx::query_as(
        r#"
            INSERT INTO meta_table (user_id, name, description)
            VALUES ($1, $2, $3) 
            RETURNING
                table_id,
                user_id,
                name,
                description,
                data_table_name,
                created_at,
                updated_at
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(description)
    .fetch_one(tx.as_mut())
    .await?;

    let data_table_name = &table.data_table_name;

    sqlx::query(&format!(
        r#"
            CREATE TABLE {data_table_name} (
                entry_id SERIAL PRIMARY KEY,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ
            )
        "#,
    ))
    .execute(tx.as_mut())
    .await?;

    sqlx::query(&format!(
        r#"SELECT trigger_updated_at('{data_table_name}')"#
    ))
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;

    Ok(table)
}

pub async fn update_table(
    executor: impl PgExecutor<'_>,
    table_id: Id,
    UpdateTable { name, description }: UpdateTable,
) -> sqlx::Result<Table> {
    sqlx::query_as(
        r#"
            UPDATE meta_table
            SET name = $1, description = $2
            WHERE table_id = $3
            RETURNING
                table_id,
                user_id,
                name,
                description,
                data_table_name,
                created_at,
                updated_at
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(table_id)
    .fetch_one(executor)
    .await
}

pub async fn delete_table(
    connection: impl Acquire<'_, Database = Postgres>,
    table_id: Id,
) -> sqlx::Result<()> {
    let mut tx = connection.begin().await?;

    let data_table_name: String = sqlx::query_scalar(
        r#"
            DELETE FROM meta_table
            WHERE table_id = $1
            RETURNING data_table_name
        "#,
    )
    .bind(table_id)
    .fetch_one(tx.as_mut())
    .await?;

    sqlx::query(&format!(r#"DROP TABLE {data_table_name}"#))
        .execute(tx.as_mut())
        .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn get_tables(executor: impl PgExecutor<'_>, user_id: Id) -> sqlx::Result<Vec<Table>> {
    sqlx::query_as(
        r#"
            SELECT
                table_id,
                user_id,
                name,
                description,
                data_table_name,
                created_at,
                updated_at
            FROM meta_table
            WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
    .await
}

pub async fn check_table_relation(
    executor: impl PgExecutor<'_>,
    user_id: Id,
    table_id: Id,
) -> sqlx::Result<Relation> {
    sqlx::query_scalar::<_, Id>(
        r#"
            SELECT user_id
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_optional(executor)
    .await
    .map(|id| match id {
        None => Relation::Absent,
        Some(id) if id == user_id => Relation::Owned,
        Some(_) => Relation::NotOwned,
    })
}
