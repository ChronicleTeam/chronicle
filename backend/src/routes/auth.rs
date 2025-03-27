use super::ApiState;
use crate::{
    error::{ApiError, ApiResult, ErrorMessage, IntoAnyhow},
    users::{AuthSession, Credentials}, Id,
};
use axum::{
    routing::{get, post},
    Form, Json, Router,
};
use axum_login::AuthUser;
use serde::Serialize;

const INVALID_CREDENTIALS: ErrorMessage = ("user", "Invalid credentials");


pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", get(logout))
}

#[derive(Debug, Serialize)]
struct UserResponse {
    pub user_id: Id,
    pub username: String,
}


async fn register(
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
