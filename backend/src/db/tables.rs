use crate::{model::Table, Id};
use sqlx::{Acquire, PgExecutor, Postgres};



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
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ,
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

    let (data_table_name,): (String,) = sqlx::query_as(
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
            FOR UPDATE
        "#,
    )
    .bind(user_id)
    .fetch_all(executor)
    .await?)
}

pub async fn get_table_user_id(
    executor: impl PgExecutor<'_>,
    table_id: Id,
) -> sqlx::Result<Option<Id>> {
    Ok(sqlx::query_as::<_, (_,)>(
        r#"
            SELECT user_id
            FROM meta_table
            WHERE table_id = $1
            FOR UPDATE
        "#,
    )
    .bind(table_id)
    .fetch_optional(executor)
    .await?
    .map(|x| x.0))
}


pub async fn get_data_table_name(
    executor: impl PgExecutor<'_>,
    table_id: Id,
) -> sqlx::Result<String> {
    let (data_table_name,): (String,) = sqlx::query_as(
        r#"
            SELECT data_table_name
            FROM meta_table
            WHERE table_id = $1
            FOR UPDATE
        "#,
    )
    .bind(table_id)
    .fetch_one(executor)
    .await?;
    Ok(data_table_name)
}