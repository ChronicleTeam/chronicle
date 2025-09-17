//! Route handlers for managing users.
//!
//! User sessions are managed through an authentication cookie which is sent to the
//! front-end and received in the back-end to confirm identity.
//!

use crate::{
    AppState,
    db::{self},
    error::{ApiError, ApiResult, IntoAnyhow},
    model::users::{Credentials, User, UserResponse, UserRole},
};
use axum::{
    Form, Json, Router,
    extract::State,
    routing::{get, post},
};
use axum_login::{AuthManagerLayerBuilder, AuthUser, AuthnBackend, UserId};
use password_auth::{generate_hash, verify_password};
use sqlx::PgPool;
use tokio::task;
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{Key, SameSite},
};
use tower_sessions_sqlx_store::PostgresStore;

const INVALID_CREDENTIALS: &str = "Invalid credentials";
const ALREADY_LOGGED_IN: &str = "Already logged in";
const USERNAME_IS_TAKEN: &str = "Username is taken";

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

/// Error type for [axum_login::AuthnBackend].
// #[derive(Debug, thiserror::Error)]
// pub enum Error {
//     #[error(transparent)]
//     Sqlx(#[from] sqlx::Error),

//     #[error(transparent)]
//     TaskJoin(#[from] task::JoinError),
// }

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

pub type AuthSession = axum_login::AuthSession<AuthBackend>;

pub async fn router(db: PgPool, admin_creds: Credentials) -> anyhow::Result<Router<AppState>> {
    let session_store = PostgresStore::new(db);
    session_store.migrate().await?;

    // Generate a cryptographic key to sign the session cookie.
    let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(SameSite::None)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(1)))
        .with_signed(key);

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.
    let backend = AuthBackend::new(db.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    tokio::spawn(async move { 
        let mut tx = db.begin().await?;
        if !db::user_exists(tx.as_mut(), admin_creds.username.clone()).await? {
            let password_hash = task::spawn_blocking(|| generate_hash(admin_creds.password))
                .await
                .anyhow()?;
            let user_id = db::create_user(tx.as_mut(), admin_creds.username, password_hash).await?.user_id;
            backend.set_role(user_id, UserRole::Admin).await?;
        }
    });

    Ok(Router::new()
        // .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/user", get(get_user))
        .route("/users", post(create_user)))
}

/// Login the user from the credentials.
///
/// # Errors
/// - [ApiError::BadRequest]: User is already authenticated
/// - [ApiError::UnprocessableEntity]:
///   - [INVALID_CREDENTIALS]
///
async fn login(
    mut auth_session: AuthSession,
    Form(creds): Form<Credentials>,
) -> ApiResult<Json<UserResponse>> {
    if auth_session.user.is_some() {
        return Err(ApiError::BadRequest(ALREADY_LOGGED_IN.into()));
    }

    let user = auth_session
        .authenticate(creds)
        .await
        .anyhow()?
        .ok_or(ApiError::UnprocessableEntity(INVALID_CREDENTIALS.into()))?;

    auth_session.login(&user).await.anyhow()?;

    Ok(Json(UserResponse {
        user_id: user.id(),
        username: user.username,
    }))
}

/// Logout the user. Does nothing if the user is not logged in.
pub async fn logout(mut auth_session: AuthSession) -> ApiResult<()> {
    auth_session.logout().await.anyhow()?;

    Ok(())
}
/// Get the currently logged in username and user ID.
/// Returns null if the user is not logged in.
async fn get_user(AuthSession { user, .. }: AuthSession) -> ApiResult<Json<Option<UserResponse>>> {
    Ok(Json(user.map(|user| UserResponse {
        user_id: user.id(),
        username: user.username,
    })))
}

/// Create a user from credentials. Request user must have the role [UserRole::Admin].
///
/// # Errors
/// - [ApiError::Unauthorized]: User not authenticated
/// - [ApiError::Forbidden]: User is not [UserRole::Admin]
/// - [ApiError::Conflict]: User name is already taken
async fn create_user(
    State(AppState { db }): State<AppState>,
    AuthSession { user, .. }: AuthSession,
    Form(creds): Form<Credentials>,
) -> ApiResult<Json<UserResponse>> {
    match user {
        Some(User {
            role: UserRole::Admin,
            ..
        }) => Ok(()),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::Unauthorized),
    }?;

    let mut tx = db.begin().await?;

    if db::user_exists(tx.as_mut(), creds.username.clone()).await? {
        return Err(ApiError::Conflict(USERNAME_IS_TAKEN.into()));
    }

    let password_hash = task::spawn_blocking(|| generate_hash(creds.password))
        .await
        .anyhow()?;

    let user = db::create_user(tx.as_mut(), creds.username, password_hash).await?;

    Ok(Json(UserResponse {
        user_id: user.id(),
        username: user.username,
    }))
}
