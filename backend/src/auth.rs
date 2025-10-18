use crate::{
    AppState, db,
    error::{ApiError, IntoAnyhow},
    model::users::{Credentials, User},
};
use aide::{NoApi, axum::ApiRouter};
use anyhow::anyhow;
use axum::{
    http::{HeaderValue, header},
    response::Response,
};
use axum_login::{AuthManagerLayerBuilder, AuthSession, AuthnBackend, UserId};
use base64::{Engine, prelude::BASE64_STANDARD};
use password_auth::{generate_hash, verify_password};
use sqlx::{Acquire, PgPool, Postgres};
use tokio::task;
use tower::ServiceBuilder;
use tower_sessions::{
    ExpiredDeletion, Expiry, SessionManagerLayer,
    cookie::{Key, SameSite},
};
use tower_sessions_sqlx_store::PostgresStore;

/// The backend type for [axum_login::AuthSession].
#[derive(Debug, Clone)]
pub struct AuthBackend {
    db: PgPool,
}

impl AuthBackend {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = ApiError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = db::get_user_from_username(&self.db, creds.username).await?;

        // Verifying the password is blocking and potentially slow, so we'll do so via
        // `spawn_blocking`.
        task::spawn_blocking(|| {
            // We're using password-based authentication--this works by comparing our form
            // input with an argon2 password hash.
            Ok(user.filter(|user| verify_password(creds.password, &user.password_hash).is_ok()))
        })
        .await
        .anyhow()?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(db::get_user(&self.db, *user_id).await?)
    }
}

pub type AppAuthSession = NoApi<AuthSession<AuthBackend>>;

pub async fn init(
    router: ApiRouter<AppState>,
    db: PgPool,
    session_key: String,
) -> anyhow::Result<ApiRouter<AppState>> {
    let session_store = PostgresStore::new(db.clone());
    session_store.migrate().await?;

    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    // Generate a cryptographic key to sign the session cookie.
    let session_key = Key::from(&BASE64_STANDARD.decode(session_key)?);

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(SameSite::None)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1)))
        .with_signed(session_key);

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.
    let backend = AuthBackend::new(db.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    let service = ServiceBuilder::new()
        .map_response(set_partitioned_cookie)
        .layer(auth_layer);

    Ok(router.layer(service))
}

pub async fn set_admin_user(
    conn: impl Acquire<'_, Database = Postgres>,
    creds: Credentials,
) -> anyhow::Result<()> {
    let mut tx = conn.begin().await?;
    if let Some(admin_user) =
        db::get_user_from_username(tx.as_mut(), creds.username.clone()).await?
    {
        if !admin_user.is_admin {
            return Err(anyhow!("provided admin user does not have the admin role"));
        }
        task::spawn_blocking(move || {
            verify_password(creds.password, &admin_user.password_hash)
                .or(Err(anyhow!("invalid admin password")))
        })
        .await??;
    } else if sqlx::query_scalar(
        r#"
            SELECT NOT EXISTS (
                SELECT 1
                FROM app_user
                WHERE is_admin
            )
        "#,
    )
    .fetch_one(tx.as_mut())
    .await?
    {
        let password_hash = task::spawn_blocking(|| generate_hash(creds.password)).await?;
        db::create_user(tx.as_mut(), creds.username, password_hash, true).await?;
    } else {
        return Err(anyhow!("admin user not found"));
    }
    tx.commit().await?;
    Ok(())
}

/// Sets the "Partitioned" attribute of the "set-cookie" header.
fn set_partitioned_cookie(mut res: Response) -> Response {
    if let Some(set_cookie) = res.headers().get(header::SET_COOKIE) {
        if let Ok(cookie_value) = set_cookie.to_str() {
            if !cookie_value.contains("Partitioned") {
                let cookie_value = format!("{}; Partitioned", cookie_value);
                let headers = res.headers_mut();
                headers.insert(
                    header::SET_COOKIE,
                    HeaderValue::from_str(&cookie_value).unwrap(),
                );
            }
        }
    }
    res
}
