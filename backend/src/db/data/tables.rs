use crate::{model::data::Table, Id};
use sqlx::{Acquire, PgExecutor, Postgres};

use super::Relation;

pub async fn create_table(
    connection: impl Acquire<'_, Database = Postgres>,
    user_id: Id,
    name: String,
    description: String,
) -> sqlx::Result<Id> {
    let mut tx = connection.begin().await?;

    let (table_id, data_table_name): (Id, String) = sqlx::query_as(
        r#"
            INSERT INTO meta_table (user_id, name, description)
            VALUES ($1, $2, $3) 
            RETURNING table_id, data_table_name
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(description)
    .fetch_one(tx.as_mut())
    .await?;

    // data_table_name generated by database NO INJECTION POSSIBLE

    sqlx::query(&format!(
        r#"
            CREATE TABLE {data_table_name} (
                entry_id SERIAL PRIMARY KEY,
                is_valid BOOLEAN NOT NULL,
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

    Ok(table_id)
}

pub async fn update_table(
    executor: impl PgExecutor<'_>,
    table_id: Id,
    name: String,
    description: String,
) -> sqlx::Result<Table> {
    Ok(sqlx::query_as(
        r#"
            UPDATE meta_table
            SET name = $1, description = $2
            WHERE table_id = $3
            RETURNING
                table_id,
                user_id,
                name,
                description,
                created_at,
                updated_at
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(table_id)
    .fetch_one(executor)
    .await?)
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

    Ok(())
}

pub async fn get_user_tables(
    executor: impl PgExecutor<'_>,
    user_id: Id,
) -> sqlx::Result<Vec<Table>> {
    Ok(sqlx::query_as(
        r#"
            SELECT
                table_id,
                user_id,
                name,
                description,
                created_at,
                updated_at
            FROM meta_table
            WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
    .await?)
}

pub async fn check_table_ownership(
    executor: impl PgExecutor<'_>,
    user_id: Id,
    table_id: Id,
) -> sqlx::Result<Relation> {
    Ok(sqlx::query_scalar::<_, Id>(
        r#"
            SELECT user_id
            FROM meta_table
            WHERE table_id = $1
        "#,
    )
    .bind(table_id)
    .fetch_optional(executor)
    .await?
    .map_or(Relation::Absent, |x| {
        if x == user_id {
            Relation::Owned
        } else {
            Relation::NotOwned
        }
    }))
}
