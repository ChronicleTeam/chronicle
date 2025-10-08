use crate::{
    Id,
    model::users::{AccessRole, User, UserResponse},
};
use sqlx::{Acquire, PgExecutor, Postgres, QueryBuilder};

pub async fn create_user(
    conn: impl Acquire<'_, Database = Postgres>,
    username: String,
    password_hash: String,
    is_admin: bool,
) -> sqlx::Result<User> {
    let mut tx = conn.begin().await?;
    let user = sqlx::query_as(
        r#"
            INSERT INTO app_user (username, password_hash, is_admin)
            VALUES ($1, $2, $3)
            RETURNING *
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .bind(is_admin)
    .fetch_one(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(user)
}

pub async fn update_user(
    conn: impl Acquire<'_, Database = Postgres>,
    user_id: Id,
    username: Option<String>,
    password_hash: Option<String>,
    is_admin: Option<bool>,
) -> sqlx::Result<User> {
    let mut tx: sqlx::Transaction<'_, _> = conn.begin().await?;
    let mut query = QueryBuilder::new(
        r#"
        UPDATE app_user SET
    "#,
    );
    let mut comma = false;
    let mut check_comma = |query: &mut QueryBuilder<'_, Postgres>| {
        if comma {
            query.push(" , ");
        }
        comma = true;
    };
    if let Some(username) = username {
        check_comma(&mut query);
        query.push(" username = ").push_bind(username);
    }
    if let Some(password_hash) = password_hash {
        check_comma(&mut query);
        query.push(" password_hash = ").push_bind(password_hash);
    }
    if let Some(is_admin) = is_admin {
        check_comma(&mut query);
        query.push(" is_admin = ").push_bind(is_admin);
    }
    let user: User = query
        .push(r#" WHERE user_id = "#)
        .push_bind(user_id)
        .push(r#" RETURNING *"#)
        .build_query_as()
        .fetch_one(tx.as_mut())
        .await?;
    tx.commit().await?;
    Ok(user)
}

pub async fn delete_user(
    conn: impl Acquire<'_, Database = Postgres>,
    user_id: Id,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;
    sqlx::query(
        r#"
        DELETE FROM app_user
        WHERE user_id = $1
    "#,
    )
    .bind(user_id)
    .execute(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_all_users(executor: impl PgExecutor<'_>) -> sqlx::Result<Vec<UserResponse>> {
    sqlx::query_as(
        r#"
            SELECT
                user_id,
                username,
                is_admin
            FROM app_user
        "#,
    )
    .fetch_all(executor)
    .await
}

pub async fn get_user(executor: impl PgExecutor<'_>, user_id: Id) -> sqlx::Result<Option<User>> {
    sqlx::query_as(
        r#"
            SELECT
                user_id,
                username,
                password_hash,
                is_admin
            FROM app_user
            WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(executor)
    .await
}

pub async fn get_user_from_username(
    executor: impl PgExecutor<'_>,
    username: String,
) -> sqlx::Result<Option<User>> {
    sqlx::query_as(
        r#"
            SELECT
                user_id,
                username,
                password_hash,
                is_admin
            FROM app_user
            WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(executor)
    .await
}

pub async fn user_exists(executor: impl PgExecutor<'_>, username: String) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        r#"
            SELECT EXISTS (
                SELECT 1
                FROM app_user
                WHERE username = $1
            )
        "#,
    )
    .bind(username)
    .fetch_one(executor)
    .await
}

pub async fn create_access(
    conn: impl Acquire<'_, Database = Postgres>,
    users: impl IntoIterator<Item = (Id, AccessRole)>,
    resource_id: Id,
    table_name: &str,
    resource_id_name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    QueryBuilder::new(format!(
        r#"INSERT INTO {table_name} (user_id, {resource_id_name}, access_role)"#
    ))
    .push_values(users, |mut builder, (user_id, access_role)| {
        builder
            .push_bind(user_id)
            .push_bind(resource_id)
            .push_bind(access_role);
    })
    .build()
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn update_access(
    conn: impl Acquire<'_, Database = Postgres>,
    users: impl IntoIterator<Item = (Id, AccessRole)>,
    resource_id: Id,
    table_name: &str,
    resource_id_name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    QueryBuilder::new(format!(
        r#"
            UPDATE {table_name} AS t
            SET access_role = v.access_role
            FROM (
        "#
    ))
    .push_values(users, |mut builder, (user_id, access_role)| {
        builder.push_bind(user_id).push_bind(access_role);
    })
    .push(format!(
        r#"
            ) AS v(user_id, access_role)
            WHERE t.user_id = v.user_id
            AND t.{resource_id_name} = 
        "#
    ))
    .push_bind(resource_id)
    .build()
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn delete_access(
    conn: impl Acquire<'_, Database = Postgres>,
    users: impl IntoIterator<Item = Id>,
    resource_id: Id,
    table_name: &str,
    resource_id_name: &str,
) -> sqlx::Result<()> {
    let mut tx = conn.begin().await?;

    QueryBuilder::new(format!(
        r#"DELETE FROM {table_name} WHERE {resource_id_name} = "#
    ))
    .push_bind(resource_id)
    .push(format!(" AND user_id IN ("))
    .push_values(users, |mut builder, user_id| {
        builder.push_bind(user_id);
    })
    .push(")")
    .build()
    .execute(tx.as_mut())
    .await?;

    tx.commit().await?;
    Ok(())
}
