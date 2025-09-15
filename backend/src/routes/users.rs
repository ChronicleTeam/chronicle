//! Route handlers for managing users.
//! 
//! User sessions are managed through an authentication cookie which is sent to the
//! front-end and received in the back-end to confirm identity.
//! 


use crate::{
    db::AuthSession, error::{ApiError, ApiResult, ErrorMessage, IntoAnyhow}, model::users::{Credentials, User, UserResponse, UserRole}, AppState
};
use axum::{
    routing::{get, post},
    Form, Json, Router,
};
use axum_login::AuthUser;

const INVALID_CREDENTIALS: ErrorMessage = ("user", "Invalid credentials");

pub fn router() -> Router<AppState> {
    Router::new()
        // .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/user", get(get_user))
        .route("/users", post(create_user))
}


/// Register a user from credentials.
/// Logins the user after registration.
/// 
/// *WARNING*: Intentionally inaccessible because we don't want
/// to allow everyone to create accounts because of limited ressources.
/// 
/// # Errors
/// - [ApiError::BadRequest]: User is already authenticated
/// - [ApiError::Conflict]: User name is already taken
/// 
async fn _register(
    mut auth_session: AuthSession,
    Form(creds): Form<Credentials>,
) -> ApiResult<Json<UserResponse>> {

    if auth_session.user.is_some() {
        return Err(ApiError::BadRequest);
    }

    if auth_session.backend.exists(&creds).await? {
        return Err(ApiError::Conflict);
    }

    let user = auth_session.backend.create_user(creds).await?;

    auth_session.login(&user).await.into_anyhow()?;

    Ok(Json(UserResponse {
        user_id: user.id(),
        username: user.username,
    }))
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
        return Err(ApiError::BadRequest);
    }

    let user = match auth_session
        .authenticate(creds.clone())
        .await
        .into_anyhow()?
    {
        Some(user) => user,
        None => {
            return Err(ApiError::unprocessable_entity([INVALID_CREDENTIALS]));
        }
    };

    auth_session.login(&user).await.into_anyhow()?;

    Ok(Json(UserResponse {
        user_id: user.id(),
        username: user.username,
    }))
}

/// Logout the user. Does nothing if the user is not logged in.
/// 
pub async fn logout(mut auth_session: AuthSession) -> ApiResult<()> {
    _ = auth_session.logout().await.into_anyhow()?;

    Ok(())
}
/// Get the currently logged in username and user ID.
/// Returns null if the user is not logged in.
/// 
async fn get_user(
    AuthSession { user, .. }: AuthSession,
) -> ApiResult<Json<Option<UserResponse>>> {
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
/// 
async fn create_user(
    AuthSession { user, mut backend, ..}: AuthSession,
    Form(creds): Form<Credentials>,
) -> ApiResult<Json<UserResponse>> {

    match user {
        Some(User {role: UserRole::Admin, ..}) => Ok(()),
        Some(_) => Err(ApiError::Forbidden),
        None => Err(ApiError::Unauthorized),
    }?;


    if backend.exists(&creds).await? {
        return Err(ApiError::Conflict);
    }

    let user = backend.create_user(creds).await?;

    Ok(Json(UserResponse {
        user_id: user.id(),
        username: user.username,
    }))
}