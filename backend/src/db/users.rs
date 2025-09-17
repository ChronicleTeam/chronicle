use axum_login::{AuthnBackend, UserId};
use password_auth::{generate_hash, verify_password};
use sqlx::{Acquire, PgExecutor, PgPool, Postgres};
use tokio::task;

use crate::{
    Id,
    error::{ApiError, IntoAnyhow},
    model::users::{Credentials, User, UserRole},
};

pub async fn create_user(
    conn: impl Acquire<'_, Database = Postgres>,
    username: String,
    password_hash: String,
) -> sqlx::Result<User> {
    let mut tx = conn.begin().await?;
    let user = sqlx::query_as(
        r#"
            INSERT INTO app_user (username, password_hash)
            VALUES ($1, $2)
            RETURNING
                user_id,
                username,
                password_hash,
                role
        "#,
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(user)
}

pub async fn create_admin_user(conn: impl Acquire<'_, Database = Postgres>,) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;
    if !db::user_exists(tx.as_mut(), admin_creds.username.clone()).await? {
        let password_hash = task::spawn_blocking(|| generate_hash(admin_creds.password))
            .await
            .anyhow()?;
        let user_id = db::create_user(tx.as_mut(), admin_creds.username, password_hash).await?.user_id;
        backend.set_role(user_id, UserRole::Admin).await?;
    }
}

pub async fn set_user_role(
    conn: impl Acquire<'_, Database = Postgres>,
    user_id: Id,
    role: UserRole,
) -> sqlx::Result<()> {
    let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = conn.begin().await?;
    sqlx::query(
        r#"
            UPDATE app_user
            SET role = $1
            WHERE user_id = $2
            RETURNING
                user_id,
                username,
                password_hash,
                role
        "#,
    )
    .bind(role)
    .bind(user_id)
    .execute(tx.as_mut())
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_user(executor: impl PgExecutor<'_>, user_id: Id) -> sqlx::Result<Option<User>> {
    sqlx::query_as(
        r#"
            SELECT
                user_id,
                username,
                password_hash,
                role
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
                role
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
