//! Route handlers for managing users.
//!
//! User sessions are managed through an authentication cookie which is sent to the
//! front-end and received in the back-end to confirm identity.
//!

use crate::{
    AppState,
    auth::AuthSession,
    db::{self},
    error::{ApiError, ApiResult, IntoAnyhow},
    model::users::{CreateUser, Credentials, SelectUser, UpdateUser, UserResponse},
};
use axum::{
    Form, Json, Router,
    extract::{Path, State},
    routing::{get, patch, post},
};
use password_auth::generate_hash;
use tokio::task;

const INVALID_CREDENTIALS: &str = "Invalid credentials";
const ALREADY_LOGGED_IN: &str = "Already logged in";
const USERNAME_IS_TAKEN: &str = "Username is taken";

pub fn router() -> Router<AppState> {
    Router::new()
        // .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/user", get(get_auth_user))
        .nest(
            "/users",
            Router::new()
                .route("/", post(create_user).get(get_all_users))
                .route("/{user_id}", patch(update_user).delete(delete_user)),
        )
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
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

/// Logout the user. Does nothing if the user is not logged in.
pub async fn logout(mut auth_session: AuthSession) -> ApiResult<()> {
    auth_session.logout().await.anyhow()?;
    Ok(())
}
/// Get the currently logged in username and user ID.
/// Returns null if the user is not logged in.
async fn get_auth_user(
    AuthSession { user, .. }: AuthSession,
) -> ApiResult<Json<Option<UserResponse>>> {
    Ok(Json(user.map(|user| UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
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
    AuthSession {
        user: auth_user, ..
    }: AuthSession,
    Form(create_user): Form<CreateUser>,
) -> ApiResult<Json<UserResponse>> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }

    let mut tx = db.begin().await?;

    if db::user_exists(tx.as_mut(), create_user.username.clone()).await? {
        return Err(ApiError::Conflict(USERNAME_IS_TAKEN.into()));
    }

    let password_hash = task::spawn_blocking(|| generate_hash(create_user.password))
        .await
        .anyhow()?;
    let new_user = db::create_user(tx.as_mut(), create_user.username, password_hash, false).await?;

    tx.commit().await?;
    Ok(Json(UserResponse {
        user_id: new_user.user_id,
        username: new_user.username,
        is_admin: new_user.is_admin,
    }))
}

async fn update_user(
    State(AppState { db }): State<AppState>,
    AuthSession {
        user: auth_user, ..
    }: AuthSession,
    Path(SelectUser { user_id }): Path<SelectUser>,
    Form(update_user): Form<UpdateUser>,
) -> ApiResult<()> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }

    let mut tx = db.begin().await?;

    if let Some(username) = update_user.username.clone()
        && db::user_exists(tx.as_mut(), username).await?
    {
        return Err(ApiError::Conflict(USERNAME_IS_TAKEN.into()));
    }

    let password_hash = if let Some(password) = update_user.password {
        Some(
            task::spawn_blocking(|| generate_hash(password))
                .await
                .anyhow()?,
        )
    } else {
        None
    };

    db::update_user(
        tx.as_mut(),
        user_id,
        update_user.username,
        password_hash,
        None,
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

async fn delete_user(
    AuthSession {
        user: auth_user, ..
    }: AuthSession,
    State(AppState { db }): State<AppState>,
    Path(SelectUser { user_id }): Path<SelectUser>,
) -> ApiResult<()> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }
    let mut tx = db.begin().await?;
    db::delete_user(tx.as_mut(), user_id).await?;
    tx.commit().await?;
    Ok(())
}

async fn get_all_users(
    AuthSession {
        user: auth_user, ..
    }: AuthSession,
    State(AppState { db }): State<AppState>,
) -> ApiResult<Json<Vec<UserResponse>>> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }
    let users = db::get_all_users(&db).await?;
    Ok(Json(users))
}
