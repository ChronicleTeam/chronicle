use super::ApiState;
use crate::{
    db::AuthSession, error::{ApiError, ApiResult, ErrorMessage, IntoAnyhow}, model::users::{Credentials, User, UserResponse, UserRole}
};
use axum::{
    routing::{get, post},
    Form, Json, Router,
};
use axum_login::AuthUser;

const INVALID_CREDENTIALS: ErrorMessage = ("user", "Invalid credentials");

pub fn router() -> Router<ApiState> {
    Router::new()
        // .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
        .route("/user", get(get_user))
        .route("/users", post(create_user))
}


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

/// Login the user from a credentials form.
/// 
/// # Errors:
/// - [ApiError::BadRequest]
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
pub async fn logout(mut auth_session: AuthSession) -> ApiResult<()> {
    _ = auth_session.logout().await.into_anyhow()?;

    Ok(())
}

async fn get_user(
    AuthSession { user, .. }: AuthSession,
) -> ApiResult<Json<Option<UserResponse>>> {
    Ok(Json(user.map(|user| UserResponse {
        user_id: user.id(),
        username: user.username,
    })))
}


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