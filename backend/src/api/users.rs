//! Route handlers for managing users.
//!
//! User sessions are managed through an authentication cookie which is sent to the
//! front-end and received in the back-end to confirm identity.
//!

use crate::{
    AppState,
    auth::AppAuthSession,
    db::{self},
    docs::{self, AUTHENTICATION_TAG, TransformOperationExt, USERS_TAG},
    error::{ApiError, ApiResult, IntoAnyhow},
    model::users::{CreateUser, Credentials, SelectUser, UpdateUser, UserResponse},
};
use aide::{
    NoApi, OperationOutput,
    axum::{
        ApiRouter,
        routing::{get_with, patch_with, post_with},
    },
    transform::TransformOperation,
};
use axum::{
    Form, Json,
    extract::{Path, State},
};
use axum_login::AuthSession;
use password_auth::generate_hash;
use tokio::task;

const INVALID_CREDENTIALS: &str = "Invalid credentials";
const ALREADY_LOGGED_IN: &str = "Already logged in";
const USERNAME_IS_TAKEN: &str = "Username is taken";

pub fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/login", post_with(login, login_docs))
        .api_route("/logout", get_with(logout, logout_docs))
        .api_route("/user", get_with(get_auth_user, get_auth_user_docs))
        .nest(
            "/users",
            ApiRouter::new()
                .api_route(
                    "/",
                    post_with(create_user, create_user_docs)
                        .get_with(get_all_users, get_all_users_docs),
                )
                .api_route(
                    "/{user_id}",
                    patch_with(update_user, update_user_docs)
                        .delete_with(delete_user, delete_user_docs),
                ),
        )
}

async fn login(
    mut session: AppAuthSession,
    Form(creds): Form<Credentials>,
) -> ApiResult<Json<UserResponse>> {
    if session.user.is_some() {
        return Err(ApiError::BadRequest(ALREADY_LOGGED_IN.into()));
    }

    let user = session
        .authenticate(creds)
        .await
        .anyhow()?
        .ok_or(ApiError::UnprocessableEntity(INVALID_CREDENTIALS.into()))?;

    session.login(&user).await.anyhow()?;

    Ok(Json(UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

fn login_docs(op: TransformOperation) -> TransformOperation {
    docs::template::<Json<UserResponse>>(
        op,
        "login",
        "ogin the user from the credentials.",
        false,
        AUTHENTICATION_TAG,
    )
    .response_description::<400, ()>("User is already authenticated")
    .response_description::<422, String>(INVALID_CREDENTIALS)
}

pub async fn logout(mut session: AppAuthSession) -> ApiResult<()> {
    session.logout().await.anyhow()?;
    Ok(())
}

fn logout_docs(op: TransformOperation) -> TransformOperation {
    docs::template::<()>(
        op,
        "logout",
        "Logout the user. Does nothing if the user is not logged in.",
        false,
        AUTHENTICATION_TAG,
    )
}

async fn get_auth_user(
    NoApi(AuthSession { user, .. }): AppAuthSession,
) -> ApiResult<Json<Option<UserResponse>>> {
    Ok(Json(user.map(|user| UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    })))
}

fn get_auth_user_docs(op: TransformOperation) -> TransformOperation {
    docs::template::<Json<Option<UserResponse>>>(
        op,
        "get_auth_user",
        "Get the currently logged in user or nothing if not logged in.",
        false,
        AUTHENTICATION_TAG,
    )
}

async fn create_user(
    State(AppState { db }): State<AppState>,
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
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
    let user = db::create_user(tx.as_mut(), create_user.username, password_hash, false).await?;

    tx.commit().await?;
    Ok(Json(UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

fn create_user_docs(op: TransformOperation) -> TransformOperation {
    users_docs::<Json<UserResponse>>(
        op,
        "create_user",
        "Create a new user. Requires admin priviledges.",
    )
    .response_description::<409, ()>("Username is taken")
}

async fn update_user(
    State(AppState { db }): State<AppState>,
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    Path(SelectUser { user_id }): Path<SelectUser>,
    Form(update_user): Form<UpdateUser>,
) -> ApiResult<Json<UserResponse>> {
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

    let user = db::update_user(
        tx.as_mut(),
        user_id,
        update_user.username,
        password_hash,
        None,
    )
    .await?;

    tx.commit().await?;
    Ok(Json(UserResponse {
        user_id: user.user_id,
        username: user.username,
        is_admin: user.is_admin,
    }))
}

fn update_user_docs(op: TransformOperation) -> TransformOperation {
    users_docs::<Json<UserResponse>>(
        op,
        "update_user",
        "Update the user's username of password. Requires admin privileges.",
    )
    .response_description::<409, ()>("Username is taken")
}

async fn delete_user(
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
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

fn delete_user_docs(op: TransformOperation) -> TransformOperation {
    users_docs::<()>(
        op,
        "delete_user",
        "Delete a user. Requires admin privileges.",
    )
}

async fn get_all_users(
    NoApi(AuthSession {
        user: auth_user, ..
    }): AppAuthSession,
    State(AppState { db }): State<AppState>,
) -> ApiResult<Json<Vec<UserResponse>>> {
    let auth_user = auth_user.ok_or(ApiError::Unauthorized)?;
    if !auth_user.is_admin {
        return Err(ApiError::Forbidden);
    }
    let users = db::get_all_users(&db).await?;
    Ok(Json(users))
}

fn get_all_users_docs(op: TransformOperation) -> TransformOperation {
    users_docs::<Json<Vec<UserResponse>>>(
        op,
        "get_all_users",
        "Retrieve all users. Requires admin privileges.",
    )
}

fn users_docs<'a, R: OperationOutput>(
    op: TransformOperation<'a>,
    summary: &'a str,
    description: &'a str,
) -> TransformOperation<'a> {
    docs::template::<R>(op, summary, description, true, USERS_TAG)
        .response_description::<403, ()>("User is not an admin")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{AppConfig, init_app, setup_tracing};
    use sqlx::PgPool;

    #[sqlx::test]
    async fn login(db: PgPool) -> anyhow::Result<()> {
        setup_tracing();
        let mut app = init_app(AppConfig::build()?).await?;

        let password = String::from("test123");
        let user = db::create_user(
            &db,
            "test@example.com".into(),
            generate_hash(password),
            true,
        )
        .await?;

        Ok(())
    }
}
