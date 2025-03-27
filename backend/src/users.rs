use crate::Id;
use axum_login::{AuthUser, AuthnBackend, UserId};
use password_auth::{generate_hash, verify_password};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::task;

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    user_id: Id,
    pub username: String,
    password_hash: String,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("user_id", &self.user_id)
            .field("username", &self.username)
            .field("password_hash", &"[redacted]")
            .finish()
    }
}

impl AuthUser for User {
    type Id = Id;

    fn id(&self) -> Self::Id {
        self.user_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        // We use the password hash as the auth
        // hash--what this means
        // is when the user changes their password the
        // auth session becomes invalid.
        self.password_hash.as_bytes()
    }
}

// This allows us to extract the authentication fields from forms. We use this
// to authenticate requests with the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Backend {
    pool: PgPool,
}

impl Backend {
    pub fn new(db: PgPool) -> Self {
        Self { pool: db }
    }

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
                password_hash
            "#,
        )
        .bind(creds.username)
        .bind(password_hash)
        .fetch_one(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(user)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

#[async_trait::async_trait]
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
                password_hash
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
                password_hash
            FROM app_user
            WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
