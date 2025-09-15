use axum_login::{AuthnBackend, UserId};
use password_auth::{generate_hash, verify_password};
use sqlx::PgPool;
use tokio::task;

use crate::{model::users::{Credentials, User, UserRole}, Id};

/// The backend type for [axum_login::AuthSession].
#[derive(Debug, Clone)]
pub struct Backend {
    pool: PgPool,
}

impl Backend {
    pub fn new(db: PgPool) -> Self {
        Self { pool: db }
    }

    /// Returns true if a user with this username exists.
    pub async fn exists(&self, creds: &Credentials) -> sqlx::Result<bool> {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM app_user
                WHERE username = $1
            )
        "#,
        )
        .bind(&creds.username)
        .fetch_one(&self.pool)
        .await
    }

    /// Create a the user and hashes the password from the credentials.
    pub async fn create_user(&mut self, creds: Credentials) -> sqlx::Result<User> {
        let password_hash = generate_hash(creds.password);

        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = self.pool.begin().await?;

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
        .bind(creds.username)
        .bind(password_hash)
        .fetch_one(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(user)
    }

    /// Set the role of this user.
    pub async fn set_role(&mut self, user_id: Id, role: UserRole) -> sqlx::Result<User> {

        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = self.pool.begin().await?;

        let user = sqlx::query_as(
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
        .fetch_one(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(user)
    }
}


/// Error type for [axum_login::AuthnBackend].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user: Option<Self::User> = sqlx::query_as(
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
        .bind(creds.username)
        .fetch_optional(&self.pool)
        .await?;

        // Verifying the password is blocking and potentially slow, so we'll do so via
        // `spawn_blocking`.
        task::spawn_blocking(|| {
            // We're using password-based authentication--this works by comparing our form
            // input with an argon2 password hash.
            Ok(user.filter(|user| verify_password(creds.password, &user.password_hash).is_ok()))
        })
        .await?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as(
            "
            SELECT
                user_id,
                username,
                password_hash,
                role
            FROM app_user
            WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
